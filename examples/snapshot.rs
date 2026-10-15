use craft_settings::Location;
use craft_ui::docking::Action;
use egui_kittest::Harness;
use lokicraft::{Shell, workspace::Panel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".captures/shell".into());
    std::fs::create_dir_all(&out)?;
    for (name, width, height, scale, desktop, demo) in [
        ("build", 1600.0, 960.0, 1.0, "Build", false),
        ("modeling", 1600.0, 960.0, 1.0, "Build", true),
        ("animate", 1600.0, 960.0, 1.0, "Animate", true),
        ("narrow", 800.0, 600.0, 1.0, "Build", true),
        ("scaled", 1100.0, 760.0, 2.0, "Build", true),
    ] {
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, height))
            .with_pixels_per_point(scale)
            .wgpu()
            .build_eframe(|cc| {
                let mut app = Shell::new(&cc.egui_ctx, Location::Memory);
                app.switch_desktop(desktop);
                if demo {
                    for label in ["Box", "Sphere", "Grid"] {
                        let tool = app
                            .shelves
                            .iter()
                            .flat_map(|s| s.tools.iter())
                            .find(|t| t.label == label)
                            .expect("captured tool")
                            .clone();
                        app.add_tool(tool);
                    }
                }
                app
            });
        for _ in 0..5 {
            h.step();
        }
        h.render()
            .map_err(std::io::Error::other)?
            .save(format!("{out}/{name}.png"))?;
        if name == "modeling" {
            h.state_mut().dock(Action::Float {
                panel: Panel::Parameters,
                rect: [800.0, 200.0, 440.0, 400.0],
            });
            for _ in 0..5 {
                h.step();
            }
            h.render()
                .map_err(std::io::Error::other)?
                .save(format!("{out}/floating.png"))?;
        }
        println!("Captured {name}");
    }
    Ok(())
}
