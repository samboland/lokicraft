use crate::{ACCENT, BG, LINE, Shell, workspace::Panel};
use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
impl Shell {
    pub(crate) fn panel(&mut self, ui: &mut Ui, panel: Panel) {
        ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
        match panel {
            Panel::Scene => self.scene(ui),
            Panel::Network => self.network(ui),
            Panel::Parameters => self.parameters(ui),
            Panel::Tree => self.tree(ui),
            Panel::Animation => self.animation(ui),
            Panel::Log => {
                ui.label("Session log");
                ui.separator();
                ui.label(&self.status);
            }
            Panel::Spreadsheet => {
                ui.horizontal(|ui| {
                    for text in ["Points", "Vertices", "Primitives", "Detail"] {
                        ui.label(text);
                    }
                });
                ui.separator();
                ui.weak("No evaluated geometry");
            }
            Panel::Materials => {
                ui.heading("Material Palette");
                ui.separator();
                ui.weak("Material editing is not implemented yet.");
            }
            Panel::Performance => {
                ui.label("Performance Monitor");
                ui.separator();
                ui.weak("No evaluation samples");
            }
            Panel::Composite => empty(ui, "Composite View", "Compositing is not implemented yet."),
            Panel::Tasks => empty(ui, "Task Graph", "Task execution is not implemented yet."),
            Panel::Python => {
                ui.monospace("LokiCraft");
                ui.weak("Scripting is not implemented yet.");
            }
        }
    }
    fn path_bar(ui: &mut Ui, path: &str) {
        ui.horizontal(|ui| {
            ui.weak("‹   ›");
            ui.separator();
            ui.label(path);
        });
        ui.separator();
    }
    fn scene(&mut self, ui: &mut Ui) {
        use crate::viewport::{Axis, Mode, Navigation, Orbit, Projection};
        use glam::{Vec2, Vec3};
        Self::path_bar(ui, "/obj");
        egui::ScrollArea::horizontal()
            .id_salt("view-toolbar")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("View", |ui| {
                        for (label, axis) in [
                            ("Front", Axis::Front),
                            ("Back", Axis::Back),
                            ("Right", Axis::Right),
                            ("Left", Axis::Left),
                            ("Top", Axis::Top),
                            ("Bottom", Axis::Bottom),
                        ] {
                            if ui.button(label).clicked() {
                                self.navigation.cancel();
                                self.navigation.view.set_axis(axis);
                                ui.close();
                            }
                        }
                        if ui.button("Reset view").clicked() {
                            self.navigation = Navigation::default();
                            ui.close();
                        }
                    });
                    if ui.small_button("Home").clicked() {
                        self.navigation = Navigation::default();
                    }
                    let projection = self.navigation.view.projection;
                    if ui
                        .selectable_label(projection == Projection::Orthographic, "Orthographic")
                        .clicked()
                    {
                        self.navigation.cancel();
                        self.navigation.view.projection = if projection == Projection::Orthographic
                        {
                            Projection::Perspective
                        } else {
                            Projection::Orthographic
                        };
                    }
                    egui::ComboBox::from_id_salt("orbit-mode")
                        .selected_text(match self.navigation.orbit {
                            Orbit::Turntable => "Turntable",
                            Orbit::Trackball => "Trackball",
                        })
                        .width(95.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.navigation.orbit,
                                Orbit::Turntable,
                                "Turntable",
                            );
                            ui.selectable_value(
                                &mut self.navigation.orbit,
                                Orbit::Trackball,
                                "Trackball",
                            );
                        });
                    ui.checkbox(&mut self.navigation.zoom_to_cursor, "Zoom to cursor");
                });
            });
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let size = Vec2::new(rect.width(), rect.height());
        // Blender event coordinates start at bottom-left; egui starts at top-left.
        let position = |p: egui::Pos2| Vec2::new(p.x - rect.left(), rect.bottom() - p.y);
        let mode = ui.input(|i| {
            if i.modifiers.shift {
                Mode::Pan
            } else if i.modifiers.ctrl {
                Mode::Zoom
            } else {
                Mode::Rotate
            }
        });
        if response.drag_started_by(egui::PointerButton::Middle) {
            if let Some(p) = ui.input(|i| i.pointer.press_origin()) {
                self.navigation.begin(mode, position(p));
            }
        }
        if self.navigation.active() {
            if ui.input(|i| i.key_pressed(egui::Key::Escape) || !i.focused) {
                self.navigation.cancel();
            } else {
                if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                    self.navigation.update(mode, position(p), size);
                }
                if !ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
                    self.navigation.confirm();
                }
            }
        }
        if response.hovered() && !self.navigation.active() {
            let pointer = ui.input(|i| i.pointer.hover_pos()).map(position);
            let steps = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::MouseWheel { unit, delta, .. } => Some(match unit {
                            egui::MouseWheelUnit::Line => delta.y,
                            egui::MouseWheelUnit::Point => delta.y / 50.0,
                            egui::MouseWheelUnit::Page => delta.y * 3.0,
                        }),
                        _ => None,
                    })
                    .sum::<f32>()
            });
            if steps != 0.0 {
                self.navigation.view.zoom_steps(
                    steps,
                    if self.navigation.zoom_to_cursor {
                        pointer
                    } else {
                        None
                    },
                    size,
                );
            }
            if !ui.ctx().egui_wants_keyboard_input() {
                for (key, axis) in [
                    (egui::Key::Num1, Axis::Front),
                    (egui::Key::Num3, Axis::Right),
                    (egui::Key::Num7, Axis::Top),
                ] {
                    if ui.input(|i| i.key_pressed(key)) {
                        self.navigation.view.set_axis(axis);
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::Num5)) {
                    self.navigation.view.projection =
                        if self.navigation.view.projection == Projection::Perspective {
                            Projection::Orthographic
                        } else {
                            Projection::Perspective
                        };
                }
                if ui.input(|i| i.key_pressed(egui::Key::Home)) {
                    self.navigation = Navigation::default();
                }
            }
        }
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 0, Color32::from_gray(49));
        let screen = |v: Vec2| pos2(rect.left() + v.x, rect.bottom() - v.y);
        let line = |a: Vec3, b: Vec3, stroke: Stroke| {
            if let Some([a, b]) = self.navigation.view.project_segment(a, b, size) {
                p.line_segment([screen(a), screen(b)], stroke);
            }
        };
        let (u, v, color_u, color_v) = match self.navigation.view.axis {
            Some(Axis::Front | Axis::Back) => (
                Vec3::X,
                Vec3::Z,
                Color32::from_rgb(135, 75, 72),
                Color32::from_rgb(75, 135, 80),
            ),
            Some(Axis::Right | Axis::Left) => (
                Vec3::Y,
                Vec3::Z,
                Color32::from_rgb(75, 95, 150),
                Color32::from_rgb(75, 135, 80),
            ),
            _ => (
                Vec3::X,
                Vec3::Y,
                Color32::from_rgb(135, 75, 72),
                Color32::from_rgb(75, 95, 150),
            ),
        };
        for i in -40..=40 {
            let stroke = Stroke::new(
                1.0,
                if i % 5 == 0 {
                    Color32::from_gray(69)
                } else {
                    Color32::from_gray(57)
                },
            );
            line(u * i as f32 - v * 40.0, u * i as f32 + v * 40.0, stroke);
            line(v * i as f32 - u * 40.0, v * i as f32 + u * 40.0, stroke);
        }
        line(-u * 40.0, u * 40.0, Stroke::new(1.2, color_u));
        line(-v * 40.0, v * 40.0, Stroke::new(1.2, color_v));
        let title = format!(
            "{} {}",
            self.navigation
                .view
                .axis
                .map(|a| format!("{a:?}"))
                .unwrap_or_else(|| "User".into()),
            if self.navigation.view.projection == Projection::Perspective {
                "Perspective"
            } else {
                "Orthographic"
            }
        );
        p.text(
            rect.left_top() + vec2(12.0, 12.0),
            Align2::LEFT_TOP,
            title,
            FontId::proportional(12.0),
            Color32::from_gray(180),
        );
        let origin = rect.left_bottom() + vec2(36.0, -38.0);
        // Houdini Y-up to Blender Z-up: (x,y,z) -> (x,-z,y).
        for (axis, label, color) in [
            (Vec3::X, "X", Color32::from_rgb(220, 100, 95)),
            (Vec3::Z, "Y", Color32::from_rgb(100, 200, 100)),
            (-Vec3::Y, "Z", Color32::from_rgb(100, 140, 225)),
        ] {
            let v = self.navigation.view.rotation * axis;
            let end = origin + vec2(v.x, -v.y) * 27.0;
            p.line_segment([origin, end], Stroke::new(2.0, color));
            p.text(
                end,
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                color,
            );
        }
        p.text(
            rect.right_bottom() - vec2(12.0, 12.0),
            Align2::RIGHT_BOTTOM,
            "Grid preview · No geometry evaluation",
            FontId::proportional(11.0),
            Color32::from_gray(150),
        );
    }
    fn network(&mut self, ui: &mut Ui) {
        Self::path_bar(ui, "/obj");
        ui.horizontal(|ui| {
            if ui.small_button("+ Add node").clicked() {
                self.show_search = true;
            }
            if ui.small_button("Home").clicked() {
                self.frame_graph = true;
            }
            if ui
                .add_enabled(
                    self.session.selected.is_some(),
                    egui::Button::new("Delete").small(),
                )
                .clicked()
            {
                self.session.remove_selected();
            }
        });
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        if self.frame_graph && !self.session.nodes.is_empty() {
            let mut bounds = Rect::NOTHING;
            for node in &self.session.nodes {
                bounds = bounds.union(Rect::from_min_size(
                    pos2(node.position[0], node.position[1]),
                    vec2(112.0, 42.0),
                ));
            }
            self.graph_zoom = ((rect.width() - 32.0) / bounds.width())
                .min((rect.height() - 32.0) / bounds.height())
                .clamp(0.2, 1.0);
            self.graph_pan = (rect.size() - bounds.size() * self.graph_zoom) * 0.5
                - bounds.min.to_vec2() * self.graph_zoom;
            self.frame_graph = false;
        }
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 0, Color32::from_gray(32));
        if response.dragged_by(egui::PointerButton::Middle) {
            self.graph_pan += ui.input(|i| i.pointer.delta());
        }
        for x in (0..rect.width() as i32).step_by(24) {
            for y in (0..rect.height() as i32).step_by(24) {
                p.circle_filled(
                    rect.min + vec2(x as f32, y as f32),
                    0.7,
                    Color32::from_gray(43),
                );
            }
        }
        if self.session.nodes.is_empty() {
            p.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Empty Network\nPress Tab to add nodes",
                FontId::proportional(16.0),
                Color32::from_gray(102),
            );
        }
        for node in &mut self.session.nodes {
            let r = Rect::from_min_size(
                rect.min
                    + vec2(node.position[0], node.position[1]) * self.graph_zoom
                    + self.graph_pan,
                vec2(112.0, 38.0) * self.graph_zoom,
            );
            let response = ui.interact(r, ui.id().with(node.id), Sense::click_and_drag());
            if response.clicked() || response.drag_started() {
                self.session.selected = Some(node.id);
            }
            if response.dragged() {
                let delta = ui.input(|i| i.pointer.delta());
                node.position[0] += delta.x / self.graph_zoom;
                node.position[1] += delta.y / self.graph_zoom;
            }
            let selected = self.session.selected == Some(node.id);
            p.rect_filled(r, 3, Color32::from_gray(64));
            p.rect_stroke(
                r,
                3,
                Stroke::new(
                    if selected { 2.0 } else { 1.0 },
                    if selected {
                        ACCENT
                    } else {
                        Color32::from_gray(98)
                    },
                ),
                StrokeKind::Inside,
            );
            p.rect_filled(
                Rect::from_min_size(r.min, vec2(6.0, r.height())),
                2,
                Color32::from_rgb(95, 123, 150),
            );
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                &node.name,
                FontId::proportional((11.0 * self.graph_zoom).max(8.0)),
                Color32::from_gray(225),
            );
            p.circle_filled(
                r.center_bottom() + vec2(0.0, 2.0),
                3.0,
                Color32::from_gray(160),
            );
        }
    }
    fn parameters(&mut self, ui: &mut Ui) {
        Self::path_bar(ui, "/obj");
        let Some(node) = self.session.selected_mut() else {
            empty(ui, "No node selected", "Select a node in the network.");
            return;
        };
        ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
        egui::ScrollArea::both()
            .max_height(ui.available_height())
            .max_width(ui.available_width())
            .id_salt("parameter-scroll")
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(ACCENT, &node.label);
                    ui.add(egui::TextEdit::singleline(&mut node.name).desired_width(150.0));
                });
                ui.weak("UI placeholder");
                ui.separator();
                ui.label("Transform");
                egui::Grid::new("transforms")
                    .spacing(vec2(8.0, 12.0))
                    .show(ui, |ui| {
                        for (label, values) in [
                            ("Translate", &mut node.translate),
                            ("Rotate", &mut node.rotate),
                            ("Scale", &mut node.scale),
                        ] {
                            ui.label(label);
                            for value in values {
                                ui.add(egui::DragValue::new(value).speed(0.1).max_decimals(3));
                            }
                            ui.end_row();
                        }
                    });
                ui.add_space(10.0);
                if ui.button("Reset transform").clicked() {
                    node.translate = [0.0; 3];
                    node.rotate = [0.0; 3];
                    node.scale = [1.0; 3];
                }
                ui.separator();
                ui.weak("Parameters do not evaluate geometry yet.");
            });
    }
    fn tree(&mut self, ui: &mut Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            egui::CollapsingHeader::new("obj")
                .default_open(true)
                .show(ui, |ui| {
                    for node in &self.session.nodes {
                        if ui
                            .selectable_label(self.session.selected == Some(node.id), &node.name)
                            .clicked()
                        {
                            self.session.selected = Some(node.id);
                        }
                    }
                });
            for path in ["mat", "out", "stage", "tasks"] {
                ui.weak(path);
            }
        });
    }
    fn animation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Channels");
            ui.separator();
            ui.weak("No animated channels");
        });
        let rect = ui.available_rect_before_wrap();
        ui.allocate_rect(rect, Sense::hover());
        let p = ui.painter_at(rect);
        p.rect_filled(rect, 0, BG);
        for i in 0..=10 {
            let x = egui::lerp(rect.x_range(), i as f32 / 10.0);
            p.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0, LINE),
            );
        }
        for i in 0..=6 {
            let y = egui::lerp(rect.y_range(), i as f32 / 6.0);
            p.line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                Stroke::new(1.0, LINE),
            );
        }
        let x = egui::lerp(rect.x_range(), (self.frame - 1) as f32 / 239.0);
        p.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.5, ACCENT),
        );
    }
}
fn empty(ui: &mut Ui, title: &str, subtitle: &str) {
    let r = ui.available_rect_before_wrap();
    let p = ui.painter();
    p.text(
        r.center() - vec2(0.0, 10.0),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(15.0),
        Color32::from_gray(155),
    );
    p.text(
        r.center() + vec2(0.0, 14.0),
        Align2::CENTER_CENTER,
        subtitle,
        FontId::proportional(11.0),
        Color32::from_gray(115),
    );
}
