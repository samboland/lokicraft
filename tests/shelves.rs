use craft_settings::Location;
use egui_kittest::{Harness, kittest::Queryable};
use lokicraft::{Shell, shelves::page_end};

#[test]
fn defaults_match_houdini_build_and_pages_do_not_skip_tools() {
    let app = Shell::new(&egui::Context::default(), Location::Memory);
    assert_eq!(app.shelf_split, 0.5);
    assert_eq!(app.shelf_sets[0].members.len(), 14);
    assert_eq!(app.shelf_sets[1].members.len(), 17);
    assert_eq!(app.shelves[app.shelf_sets[0].selected].name, "create");
    assert_eq!(
        app.shelves[app.shelf_sets[1].selected].name,
        "lights_and_cameras"
    );
    let widths = [48.0, 80.0, 120.0, 65.0, 180.0];
    let mut start = 0;
    while start < widths.len() {
        let end = page_end(&widths, start, 200.0);
        assert!(end > start);
        assert!(widths[start..end].iter().sum::<f32>() + (end - start - 1) as f32 * 4.0 <= 200.0);
        start = end;
    }
}
#[test]
fn arrow_paging_and_tabs_are_independent() {
    let mut h = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_eframe(|cc| Shell::new(&cc.egui_ctx, Location::Memory));
    h.run();
    let right = h.state().shelf_sets[1].selected;
    h.get_by_label("Next left shelf tools").click();
    h.run();
    assert!(h.state().shelf_sets[0].tool_start > 0);
    assert_eq!(h.state().shelf_sets[1].tool_start, 0);
    h.get_by_label("Previous left shelf tools").click();
    h.run();
    assert_eq!(h.state().shelf_sets[0].tool_start, 0);
    h.get_by_label("Modify").click();
    h.run();
    assert_eq!(
        h.state().shelves[h.state().shelf_sets[0].selected].name,
        "modify"
    );
    assert_eq!(h.state().shelf_sets[1].selected, right);
    h.get_by_label("Next right shelf tabs").click();
    h.run();
    assert!(h.state().shelf_sets[1].tab_start > 0);
    assert_eq!(h.state().shelf_sets[0].tab_start, 0);
}
#[test]
fn divider_drag_and_selection_survive_reload() {
    use egui::{Event, Modifiers, PointerButton, pos2};
    let dir = std::env::temp_dir().join(format!("lokicraft-shelves-{}", std::process::id()));
    assert!(!dir.exists());
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .build_eframe(|cc| Shell::new(&cc.egui_ctx, Location::Dir(dir.clone())));
    h.run();
    h.get_by_label("Modify").click();
    h.run();
    h.get_by_label("Collisions").click();
    h.run();
    h.event(Event::PointerMoved(pos2(500.0, 70.0)));
    h.event(Event::PointerButton {
        pos: pos2(500.0, 70.0),
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    h.event(Event::PointerMoved(pos2(640.0, 70.0)));
    h.run_steps(3);
    h.event(Event::PointerButton {
        pos: pos2(640.0, 70.0),
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert!((h.state().shelf_split - 0.64).abs() < 0.01);
    h.state_mut().flush().unwrap();
    drop(h);
    let app = Shell::new(&egui::Context::default(), Location::Dir(dir.clone()));
    assert!((app.shelf_split - 0.64).abs() < 0.01);
    assert_eq!(app.shelves[app.shelf_sets[0].selected].name, "modify");
    assert_eq!(app.shelves[app.shelf_sets[1].selected].name, "collisions");
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn every_imported_tool_label_fits_at_minimum_shelf_width() {
    let ctx = egui::Context::default();
    let app = Shell::new(&ctx, Location::Memory);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ctx| {
        for shelf in &app.shelves {
            for tool in &shelf.tools {
                let galley = ctx.fonts_mut(|fonts| {
                    fonts.layout(
                        tool.label.clone(),
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                        168.0,
                    )
                });
                assert!(
                    galley.size().y <= 29.0,
                    "{} requires {} pixels",
                    tool.label,
                    galley.size().y
                );
            }
        }
    });
    output.textures_delta.clear();
}
