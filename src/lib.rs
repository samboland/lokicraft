pub mod model;
mod panels;
pub mod workspace;

use craft_settings::{Location, Settings, SettingsStore};
use craft_ui::docking::{Action, DockArea, DockStyle, Layout, Permissions};
use egui::{Color32, Id, Rect, Ui, UiBuilder, pos2, vec2};
use model::{Session, Shelf, Tool};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Instant};
use workspace::{DESKTOPS, Panel};

pub const ACCENT: Color32 = Color32::from_rgb(224, 163, 62);
pub const BG: Color32 = Color32::from_rgb(38, 39, 41);
pub const LINE: Color32 = Color32::from_rgb(57, 59, 62);

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub desktop: String,
    pub layouts: BTreeMap<String, String>,
    pub shelf: String,
}
impl Default for Prefs {
    fn default() -> Self {
        Self {
            desktop: "Build".into(),
            layouts: BTreeMap::new(),
            shelf: "Create".into(),
        }
    }
}
impl Settings for Prefs {
    const APP_ID: &'static str = "lokicraft";
    const SCHEMA_VERSION: u32 = 1;
}

pub struct Shell {
    pub layout: Layout<Panel>,
    pub desktop: String,
    pub session: Session,
    pub shelves: Vec<Shelf>,
    pub shelf: usize,
    pub frame: i32,
    pub playing: bool,
    pub status: String,
    pub search: String,
    pub show_search: bool,
    pub graph_pan: egui::Vec2,
    pub graph_zoom: f32,
    pub frame_graph: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    prefs: SettingsStore<Prefs>,
    playback_fraction: f32,
}
impl Shell {
    pub fn new(ctx: &egui::Context, location: Location) -> Self {
        let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = Color32::from_gray(46);
        style.visuals.extreme_bg_color = Color32::from_gray(31);
        style.visuals.selection.bg_fill = Color32::from_rgb(79, 87, 106);
        style.visuals.selection.stroke.color = ACCENT;
        style.visuals.widgets.inactive.bg_fill = Color32::from_gray(53);
        style.visuals.widgets.inactive.corner_radius = 2.into();
        style.spacing.item_spacing = vec2(6.0, 4.0);
        style.spacing.button_padding = vec2(7.0, 3.0);
        for font in style.text_styles.values_mut() {
            font.size = 12.0;
        }
        ctx.set_theme(egui::Theme::Dark);
        ctx.set_style_of(egui::Theme::Dark, style);
        let (prefs, _) = SettingsStore::<Prefs>::open(location);
        let desktop = if DESKTOPS.contains(&prefs.get().desktop.as_str()) {
            prefs.get().desktop.clone()
        } else {
            "Build".into()
        };
        let layout = load_layout(prefs.get().layouts.get(&desktop))
            .unwrap_or_else(|| workspace::preset(&desktop));
        let shelves: Vec<Shelf> = serde_json::from_str(include_str!("../assets/shelves.json"))
            .expect("checked embedded catalog");
        let shelf = shelves
            .iter()
            .position(|s| s.label == prefs.get().shelf)
            .unwrap_or(0);
        Self {
            layout,
            desktop,
            session: Session::default(),
            shelves,
            shelf,
            frame: 1,
            playing: false,
            status: "Ready".into(),
            search: String::new(),
            show_search: false,
            graph_pan: vec2(0.0, 0.0),
            graph_zoom: 1.0,
            frame_graph: true,
            yaw: 0.62,
            pitch: 0.62,
            zoom: 32.0,
            prefs,
            playback_fraction: 0.0,
        }
    }
    fn remember(&mut self) {
        if let Ok(json) = serde_json::to_string(&self.layout) {
            let desktop = self.desktop.clone();
            let shelf = self.shelves[self.shelf].label.clone();
            self.prefs.update(|p| {
                p.desktop = desktop.clone();
                p.layouts.insert(desktop, json);
                p.shelf = shelf;
            });
        }
    }
    pub fn switch_desktop(&mut self, name: &str) {
        if !DESKTOPS.contains(&name) {
            return;
        }
        self.remember();
        self.desktop = name.into();
        self.layout = load_layout(self.prefs.get().layouts.get(name))
            .unwrap_or_else(|| workspace::preset(name));
    }
    pub fn dock(&mut self, action: Action<Panel>) {
        if let Err(error) = self.layout.apply(action) {
            self.status = error.to_string();
        }
    }
    pub fn reveal(&mut self, panel: Panel) {
        if self.layout.apply(Action::Activate { panel }).is_err() {
            self.dock(Action::Open {
                panel,
                anchor: None,
            });
        }
    }
    pub fn add_tool(&mut self, tool: Tool) {
        self.session.add(&tool);
        self.frame_graph = true;
        self.status = format!("{} placeholder added", tool.label);
        self.reveal(Panel::Parameters);
        self.reveal(Panel::Network);
    }
    pub fn flush(&mut self) -> Result<(), craft_settings::SettingsError> {
        self.remember();
        self.prefs.flush()
    }
    pub fn draw(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if !ctx.egui_wants_keyboard_input() && ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.show_search = true;
        }
        if self.playing {
            self.playback_fraction += ctx.input(|i| i.stable_dt.min(0.1)) * 24.0;
            let frames = self.playback_fraction.floor() as i32;
            self.playback_fraction -= frames as f32;
            self.frame = (self.frame - 1 + frames) % 240 + 1;
            ctx.request_repaint();
        }
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0, BG);
        region(
            ui,
            Rect::from_min_max(rect.min, pos2(rect.right(), rect.top() + 29.0)),
            |ui| self.menu(ui),
        );
        region(
            ui,
            Rect::from_min_max(
                pos2(rect.left(), rect.top() + 29.0),
                pos2(rect.right(), rect.top() + 113.0),
            ),
            |ui| self.shelf_ui(ui),
        );
        region(
            ui,
            Rect::from_min_max(pos2(rect.left(), rect.bottom() - 81.0), rect.max),
            |ui| self.timeline(ui),
        );
        let center = Rect::from_min_max(
            pos2(rect.left() + 3.0, rect.top() + 116.0),
            pos2(rect.right() - 3.0, rect.bottom() - 84.0),
        );
        region(ui, center, |ui| {
            let mut style = DockStyle::from_ui(ui);
            style.tab_height = 25.0;
            style.gap = 4.0;
            style.active_background = Color32::from_gray(52);
            style.tab_background = Color32::from_gray(35);
            style.accent = ACCENT;
            style.background = BG;
            let layout = self.layout.clone();
            let out = DockArea::new(Id::new("workspace")).show(
                ui,
                &layout,
                &style,
                |p| p.label().into(),
                |_| Permissions::default(),
                |ui, p| self.panel(ui, *p),
            );
            for action in out.actions {
                self.dock(action);
            }
            if let Some(error) = out.error {
                self.status = error.to_string();
            }
        });
        if self.show_search {
            self.search_window(&ctx);
        }
        self.remember();
        let tick = self.prefs.tick(Instant::now());
        if let Some(deadline) = tick.next_deadline {
            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
        }
        if let craft_settings::SaveStatus::Failed(error) = tick.status {
            self.status = format!("Workspace save failed: {error}");
        }
    }
    fn menu(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                ui.add_enabled(false, egui::Button::new("Open Houdini project..."))
                    .on_disabled_hover_text("HIP import is planned.");
                ui.add_enabled(false, egui::Button::new("Save project..."));
                ui.separator();
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui
                    .add_enabled(
                        self.session.selected.is_some(),
                        egui::Button::new("Delete selected node"),
                    )
                    .clicked()
                {
                    self.session.remove_selected();
                    ui.close();
                }
            });
            for title in ["Render", "Assets"] {
                ui.menu_button(title, |ui| {
                    ui.label("Not implemented yet");
                });
            }
            ui.menu_button("Windows", |ui| {
                for panel in Panel::ALL {
                    if ui.button(panel.label()).clicked() {
                        self.reveal(panel);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Reset current layout").clicked() {
                    self.layout = workspace::preset(&self.desktop);
                    ui.close();
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("LokiCraft 0.1 · UI shell");
                ui.label("Drag tabs to dock or float panels.");
                ui.label("Right-drag the grid to orbit. Scroll to zoom.");
                ui.label("Shelf tools create UI placeholders only.");
            });
            ui.separator();
            let old = self.desktop.clone();
            let mut desktop = old.clone();
            egui::ComboBox::from_id_salt("desktop")
                .selected_text(&desktop)
                .width(145.0)
                .show_ui(ui, |ui| {
                    for name in DESKTOPS {
                        ui.selectable_value(&mut desktop, name.to_string(), name);
                    }
                });
            if old != desktop {
                craft_ui::docking::cancel_drag::<Panel>(ui.ctx(), Id::new("workspace"));
                self.switch_desktop(&desktop);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("LOKICRAFT")
                        .color(Color32::from_gray(160))
                        .strong(),
                );
            });
        });
    }
    fn shelf_ui(&mut self, ui: &mut Ui) {
        egui::ScrollArea::horizontal()
            .id_salt("shelf-tabs")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Keep familiar modeling shelves first, preserving each imported tool's order.
                    let first = [
                        "Create",
                        "Modify",
                        "Model",
                        "Polygon",
                        "Deform",
                        "Texture",
                        "Rigging",
                        "Characters",
                    ];
                    let mut order: Vec<usize> = (0..self.shelves.len()).collect();
                    order.sort_by_key(|i| {
                        first
                            .iter()
                            .position(|s| *s == self.shelves[*i].label)
                            .unwrap_or(100)
                    });
                    for i in order {
                        if ui
                            .selectable_label(self.shelf == i, &self.shelves[i].label)
                            .clicked()
                        {
                            self.shelf = i;
                        }
                    }
                });
            });
        ui.add_space(2.0);
        let tools = self.shelves[self.shelf].tools.clone();
        egui::ScrollArea::horizontal()
            .id_salt("shelf-tools")
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for tool in tools {
                        if tool_button(ui, &tool).clicked() {
                            self.add_tool(tool);
                        }
                    }
                    if self.shelves[self.shelf].tools.is_empty() {
                        ui.weak("No tools in this shelf");
                    }
                });
            });
    }
    fn timeline(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("|◀").on_hover_text("First frame").clicked() {
                self.frame = 1;
            }
            if ui.button("◀").on_hover_text("Previous frame").clicked() {
                self.frame = (self.frame - 1).max(1);
            }
            if ui
                .button(if self.playing { "Pause" } else { "Play" })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button("▶|").on_hover_text("Last frame").clicked() {
                self.frame = 240;
            }
            ui.add(egui::DragValue::new(&mut self.frame).range(1..=240));
            let width = (ui.available_width() - 90.0).max(30.0);
            ui.spacing_mut().slider_width = width;
            ui.add(egui::Slider::new(&mut self.frame, 1..=240).show_value(false));
            ui.label("24 fps");
        });
        ui.horizontal(|ui| {
            ui.weak("Range");
            ui.label("1 — 240");
            ui.separator();
            ui.weak("0 keys");
            ui.separator();
            ui.weak("No simulation");
        });
        ui.horizontal(|ui| {
            ui.label(&self.status);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak("UI shell · Geometry engine pending");
            });
        });
    }
    fn search_window(&mut self, ctx: &egui::Context) {
        let mut open = true;
        let mut chosen = None;
        egui::Window::new("Add node")
            .open(&mut open)
            .collapsible(false)
            .default_width(350.0)
            .show(ctx, |ui| {
                let response = ui.text_edit_singleline(&mut self.search);
                if self.search.is_empty() {
                    response.request_focus();
                }
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        let mut seen = std::collections::HashSet::new();
                        for shelf in &self.shelves {
                            for tool in &shelf.tools {
                                if seen.insert(&tool.name)
                                    && tool
                                        .label
                                        .to_lowercase()
                                        .contains(&self.search.to_lowercase())
                                    && ui.button(&tool.label).clicked()
                                {
                                    chosen = Some(tool.clone());
                                }
                            }
                        }
                    });
            });
        if let Some(tool) = chosen {
            self.add_tool(tool);
            open = false;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            open = false;
        }
        self.show_search = open;
    }
}
pub fn load_layout(json: Option<&String>) -> Option<Layout<Panel>> {
    let json = json?;
    if json.len() > 262144 {
        return None;
    }
    let layout: Layout<Panel> = serde_json::from_str(json).ok()?;
    layout.validate().ok()?;
    Some(layout)
}
fn tool_button(ui: &mut Ui, tool: &Tool) -> egui::Response {
    use egui::{Align2, FontId, Sense, Stroke, StrokeKind};
    let (rect, response) = ui.allocate_exact_size(vec2(66.0, 48.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &tool.label));
    let p = ui.painter_at(rect);
    if response.hovered() {
        p.rect_filled(rect, 2, Color32::from_gray(61));
    }
    let c = rect.center_top() + vec2(0.0, 15.0);
    let stroke = Stroke::new(1.5, Color32::from_rgb(169, 187, 195));
    let lower = tool.label.to_lowercase();
    if lower.contains("sphere") || lower.contains("circle") || lower.contains("torus") {
        p.circle_stroke(c, 10.0, stroke);
        if lower.contains("sphere") {
            p.line_segment([c - vec2(10.0, 0.0), c + vec2(10.0, 0.0)], stroke);
            p.add(egui::Shape::ellipse_stroke(c, vec2(4.0, 10.0), stroke));
        }
        if lower.contains("torus") {
            p.circle_stroke(c, 5.0, stroke);
        }
    } else if lower.contains("box") {
        let points = [
            c + vec2(-9.0, -5.0),
            c + vec2(0.0, -10.0),
            c + vec2(10.0, -5.0),
            c + vec2(10.0, 6.0),
            c + vec2(0.0, 11.0),
            c + vec2(-9.0, 6.0),
        ];
        p.add(egui::Shape::closed_line(points.to_vec(), stroke));
        p.line_segment([points[0], c], stroke);
        p.line_segment([points[2], c], stroke);
        p.line_segment([points[4], c], stroke);
    } else if lower.contains("grid") {
        for i in -1..=1 {
            let v = i as f32 * 7.0;
            p.line_segment([c + vec2(v, -7.0), c + vec2(v, 7.0)], stroke);
            p.line_segment([c + vec2(-7.0, v), c + vec2(7.0, v)], stroke);
        }
    } else if lower.contains("line") || lower.contains("curve") || lower.contains("path") {
        p.add(egui::Shape::line(
            vec![
                c + vec2(-10.0, 8.0),
                c + vec2(-3.0, -7.0),
                c + vec2(4.0, 5.0),
                c + vec2(10.0, -8.0),
            ],
            stroke,
        ));
    } else {
        p.rect_stroke(
            Rect::from_center_size(c, vec2(16.0, 16.0)),
            3,
            stroke,
            StrokeKind::Inside,
        );
        p.line_segment(
            [c - vec2(5.0, 0.0), c + vec2(5.0, 0.0)],
            Stroke::new(1.5, ACCENT),
        );
        p.line_segment(
            [c - vec2(0.0, 5.0), c + vec2(0.0, 5.0)],
            Stroke::new(1.5, ACCENT),
        );
    }
    p.text(
        rect.center_bottom() - vec2(0.0, 2.0),
        Align2::CENTER_BOTTOM,
        &tool.label,
        FontId::proportional(11.0),
        ui.visuals().text_color(),
    );
    response.on_hover_text(format!("{} · Add UI placeholder", tool.label))
}
pub(crate) fn region(ui: &mut Ui, rect: Rect, body: impl FnOnce(&mut Ui)) {
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.set_clip_rect(rect.intersect(ui.clip_rect()));
        body(ui);
    });
}
impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
    fn on_exit(&mut self) {
        if let Err(error) = self.flush() {
            eprintln!("Could not save workspace: {error}");
        }
    }
}
