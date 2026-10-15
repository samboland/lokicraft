//! Houdini Build shelf membership with independent, item-aligned overflow controls.
use crate::{Shell, model::Shelf, region, tool_button};
use egui::{Color32, FontId, Id, Rect, Sense, Ui, pos2, vec2};
use serde::Deserialize;

#[derive(Deserialize)]
struct Definition {
    shelves: Vec<String>,
}
#[derive(Debug)]
pub struct ShelfSetState {
    pub members: Vec<usize>,
    pub selected: usize,
    pub tab_start: usize,
    pub tool_start: usize,
}
pub fn initial_sets(catalog: &[Shelf], saved: [&str; 2]) -> [ShelfSetState; 2] {
    let definitions: Vec<Definition> =
        serde_json::from_str(include_str!("../assets/shelf-sets.json"))
            .expect("checked shelf sets");
    std::array::from_fn(|side| {
        let members: Vec<_> = definitions[side]
            .shelves
            .iter()
            .map(|name| {
                catalog
                    .iter()
                    .position(|s| s.name == *name)
                    .expect("imported shelf member")
            })
            .collect();
        let selected = members
            .iter()
            .copied()
            .find(|i| catalog[*i].label == saved[side])
            .unwrap_or(members[0]);
        let tab_start = members.iter().position(|i| *i == selected).unwrap_or(0);
        ShelfSetState {
            members,
            selected,
            tab_start,
            tool_start: 0,
        }
    })
}

/// Return only complete items. The caller constrains individual widths to the row.
pub fn page_end(widths: &[f32], start: usize, available: f32) -> usize {
    let mut used = 0.0;
    let mut end = start;
    while end < widths.len() {
        let next = widths[end] + if end > start { 4.0 } else { 0.0 };
        if used + next > available + 0.01 {
            break;
        }
        used += next;
        end += 1;
    }
    end
}
fn text_width(ui: &Ui, label: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(label.into(), FontId::proportional(size), Color32::WHITE)
        .size()
        .x
}
fn arrow(ui: &mut Ui, rect: Rect, enabled: bool, right: bool, label: &str) -> bool {
    let response = ui.interact(
        rect,
        Id::new(label),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    let c = rect.center();
    let sign = if right { 1.0 } else { -1.0 };
    let color = Color32::from_gray(if enabled { 185 } else { 70 });
    ui.painter().add(egui::Shape::line(
        vec![
            c + vec2(-sign * 2.0, -4.0),
            c + vec2(sign * 2.0, 0.0),
            c + vec2(-sign * 2.0, 4.0),
        ],
        egui::Stroke::new(1.5, color),
    ));
    response.on_hover_text(label).clicked() && enabled
}
fn pager(
    ui: &mut Ui,
    rect: Rect,
    widths: &[f32],
    start: &mut usize,
    label: &str,
) -> (Rect, std::ops::Range<usize>) {
    let total = widths.iter().sum::<f32>() + widths.len().saturating_sub(1) as f32 * 4.0;
    if total <= rect.width() {
        *start = 0;
        return (rect, 0..widths.len());
    }
    *start = (*start).min(widths.len().saturating_sub(1));
    let content = Rect::from_min_max(rect.min + vec2(18.0, 0.0), rect.max - vec2(18.0, 0.0));
    let end = page_end(widths, *start, content.width());
    if arrow(
        ui,
        Rect::from_min_max(rect.min, pos2(content.left(), rect.bottom())),
        *start > 0,
        false,
        &format!("Previous {label}"),
    ) {
        let mut previous = start.saturating_sub(1);
        while previous > 0 && page_end(widths, previous - 1, content.width()) >= *start {
            previous -= 1;
        }
        *start = previous;
    }
    if arrow(
        ui,
        Rect::from_min_max(pos2(content.right(), rect.top()), rect.max),
        end < widths.len(),
        true,
        &format!("Next {label}"),
    ) {
        *start = end;
    }
    (content, *start..page_end(widths, *start, content.width()))
}
impl Shell {
    pub(crate) fn shelf_ui(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        let min_side = 220.0_f32.min((rect.width() - 8.0) * 0.5);
        let split = (rect.width() * self.shelf_split).clamp(min_side, rect.width() - min_side);
        let divider = Rect::from_min_max(
            pos2(rect.left() + split - 4.0, rect.top()),
            pos2(rect.left() + split + 4.0, rect.bottom()),
        );
        let response = ui
            .interact(divider, Id::new("shelf-set-divider"), Sense::drag())
            .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Other, true, "Resize shelf sets")
        });
        if response.dragged() {
            if let Some(p) = response.interact_pointer_pos() {
                self.shelf_split = ((p.x - rect.left()).clamp(min_side, rect.width() - min_side)
                    / rect.width())
                .clamp(0.1, 0.9);
            }
        }
        ui.painter().vline(
            divider.center().x,
            rect.y_range(),
            egui::Stroke::new(1.0, Color32::from_gray(75)),
        );
        for y in [36.0, 41.0, 46.0] {
            ui.painter().circle_filled(
                pos2(divider.center().x, rect.top() + y),
                1.0,
                Color32::GRAY,
            );
        }
        for side in 0..2 {
            let area = if side == 0 {
                Rect::from_min_max(rect.min, divider.left_bottom())
            } else {
                Rect::from_min_max(divider.right_top(), rect.max)
            };
            region(ui, area, |ui| self.shelf_set_ui(ui, side));
        }
    }
    fn shelf_set_ui(&mut self, ui: &mut Ui, side: usize) {
        let rect = ui.max_rect();
        let side_name = if side == 0 { "left" } else { "right" };
        let tabs = Rect::from_min_max(rect.min, pos2(rect.right(), rect.top() + 22.0));
        let state = &mut self.shelf_sets[side];
        let widths: Vec<_> = state
            .members
            .iter()
            .map(|i| text_width(ui, &self.shelves[*i].label, 12.0) + 16.0)
            .collect();
        let (content, range) = pager(
            ui,
            tabs,
            &widths,
            &mut state.tab_start,
            &format!("{side_name} shelf tabs"),
        );
        let mut x = content.left();
        for index in range {
            let i = state.members[index];
            let tab = Rect::from_min_size(pos2(x, tabs.top()), vec2(widths[index], tabs.height()));
            region(ui, tab, |ui| {
                if ui
                    .selectable_label(state.selected == i, &self.shelves[i].label)
                    .clicked()
                {
                    state.selected = i;
                    state.tool_start = 0;
                }
            });
            x += widths[index] + 4.0;
        }
        let tools = self.shelves[state.selected].tools.clone();
        let row = Rect::from_min_max(pos2(rect.left(), tabs.bottom() + 2.0), rect.max);
        let widths: Vec<_> = tools
            .iter()
            .map(|tool| {
                (text_width(ui, &tool.label, 11.0) + 14.0)
                    .max(48.0)
                    .min((row.width() - 36.0).max(1.0))
            })
            .collect();
        let (content, range) = pager(
            ui,
            row,
            &widths,
            &mut state.tool_start,
            &format!("{side_name} shelf tools"),
        );
        let mut x = content.left();
        for i in range {
            let item = Rect::from_min_size(pos2(x, row.top()), vec2(widths[i], row.height()));
            region(ui, item, |ui| {
                if tool_button(ui, &tools[i], widths[i]).clicked() {
                    self.add_tool(tools[i].clone());
                }
            });
            x += widths[i] + 4.0;
        }
    }
}
