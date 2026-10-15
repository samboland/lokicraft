use craft_settings::Location;
use egui_kittest::{Harness, kittest::Queryable};
use lokicraft::Shell;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&out)?;
    for (name, width, scale, split, start) in [
        ("default", 1600.0, 1.0, 0.5, 0),
        ("narrow", 800.0, 1.0, 0.5, 0),
        ("resized", 1000.0, 1.0, 0.25, 0),
        ("long-labels", 1600.0, 1.0, 0.5, 11),
        ("scaled", 1000.0, 2.0, 0.5, 11),
    ] {
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, 700.0))
            .with_pixels_per_point(scale)
            .wgpu()
            .build_eframe(|cc| {
                let mut app = Shell::new(&cc.egui_ctx, Location::Memory);
                app.shelf_split = split;
                app.shelf_sets[0].tool_start = start;
                app.shelf_sets[1].tool_start = 7;
                app
            });
        if start == 0 {
            h.state_mut().shelf_sets[1].tool_start = 0;
        }
        h.run_steps(5);
        h.render()
            .map_err(std::io::Error::other)?
            .save(format!("{out}/{name}.png"))?;
        if name == "default" {
            h.get_by_label("Add to left shelf set").click();
            h.run();
            h.render()
                .map_err(std::io::Error::other)?
                .save(format!("{out}/plus-menu.png"))?;
            h.get_by_label("Add Tool...").click();
            h.run();
            h.render()
                .map_err(std::io::Error::other)?
                .save(format!("{out}/tool-picker.png"))?;
        }
    }
    Ok(())
}
