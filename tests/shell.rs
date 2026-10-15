use craft_settings::Location;
use craft_ui::docking::Action;
use lokicraft::{
    Shell, load_layout,
    workspace::{self, DESKTOPS, Panel},
};

#[test]
fn layouts_preserve_nodes_across_workspace_changes_and_docking() {
    let ctx = egui::Context::default();
    let mut app = Shell::new(&ctx, Location::Memory);
    let tool = app
        .shelves
        .iter()
        .flat_map(|s| &s.tools)
        .find(|t| t.label == "Box")
        .unwrap()
        .clone();
    let id = app.session.add(&tool);
    app.session.selected_mut().unwrap().translate[0] = 7.5;
    app.dock(Action::Float {
        panel: Panel::Parameters,
        rect: [200.0, 100.0, 320.0, 300.0],
    });
    let original = app.layout.clone();
    app.switch_desktop("Animate");
    app.switch_desktop("Build");
    assert_eq!(app.layout, original);
    assert_eq!(app.session.selected, Some(id));
    assert_eq!(app.session.selected_mut().unwrap().translate[0], 7.5);
    app.dock(Action::Close {
        panel: Panel::Parameters,
    });
    app.reveal(Panel::Parameters);
    assert!(app.layout.validate().is_ok());
}
#[test]
fn presets_and_invalid_saved_layouts() {
    for desktop in DESKTOPS {
        assert!(workspace::preset(desktop).validate().is_ok());
    }
    assert!(load_layout(Some(&"not json".into())).is_none());
    let layout = craft_ui::docking::Layout {
        root: Some(craft_ui::docking::Node::Tabs {
            panels: vec![Panel::Scene, Panel::Scene],
            active: 0,
        }),
        floating: vec![],
    };
    assert!(load_layout(Some(&serde_json::to_string(&layout).unwrap())).is_none());
}
#[test]
fn imported_tools_have_unique_shelf_ids_and_node_ids_do_not_recycle() {
    let mut app = Shell::new(&egui::Context::default(), Location::Memory);
    let names: std::collections::HashSet<_> = app.shelves.iter().map(|s| &s.name).collect();
    assert_eq!(names.len(), 58);
    let tool = app.shelves[0].tools[0].clone();
    let first = app.session.add(&tool);
    app.session.remove_selected();
    assert!(app.session.add(&tool) > first);
}

#[test]
fn pointer_actions_create_delete_and_play_without_desktop_input() {
    use egui_kittest::{Harness, kittest::Queryable};
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 960.0))
        .build_eframe(|cc| Shell::new(&cc.egui_ctx, Location::Memory));
    h.run();
    h.get_by_label("Box").click();
    h.run();
    assert_eq!(h.state().session.nodes.len(), 1);
    assert_eq!(h.state().session.nodes[0].label, "Box");
    h.get_by_label("Delete").click();
    h.run();
    assert!(h.state().session.nodes.is_empty());
    h.get_by_label("Play").click();
    h.run_steps(5);
    assert!(h.state().playing);
    h.get_by_label("Pause").click();
    h.run_steps(3);
    assert!(!h.state().playing);
    h.get_by_label("+ Add node").click();
    h.run();
    assert!(h.state().show_search);
}

#[test]
fn workspace_survives_settings_reload() {
    let dir = std::env::temp_dir().join(format!("lokicraft-settings-{}", std::process::id()));
    assert!(!dir.exists());
    let ctx = egui::Context::default();
    {
        let mut app = Shell::new(&ctx, Location::Dir(dir.clone()));
        app.switch_desktop("Animate");
        app.dock(Action::Float {
            panel: Panel::Parameters,
            rect: [100.0, 100.0, 300.0, 300.0],
        });
        app.flush().unwrap();
    }
    let app = Shell::new(&ctx, Location::Dir(dir.clone()));
    assert_eq!(app.desktop, "Animate");
    assert_eq!(app.layout.floating.len(), 1);
    assert_eq!(app.layout.floating[0].panels, vec![Panel::Parameters]);
    drop(app);
    std::fs::remove_dir_all(dir).unwrap();
}
