use craft_settings::Location;
use egui::{Event, Modifiers, PointerButton, pos2};
use egui_kittest::Harness;
use lokicraft::{
    Shell,
    viewport::{Axis, Orbit},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".captures/navigation".into());
    std::fs::create_dir_all(&out)?;
    for (name, axis, orbit) in [
        ("perspective", None, Orbit::Turntable),
        ("top", Some(Axis::Top), Orbit::Turntable),
        ("front", Some(Axis::Front), Orbit::Turntable),
        ("trackball", None, Orbit::Trackball),
    ] {
        let mut h = Harness::builder()
            .with_size(egui::vec2(1600.0, 960.0))
            .wgpu()
            .build_eframe(|cc| {
                let mut app = Shell::new(&cc.egui_ctx, Location::Memory);
                app.navigation.orbit = orbit;
                if let Some(axis) = axis {
                    app.navigation.view.set_axis(axis);
                }
                app
            });
        h.run_steps(5);
        if name == "trackball" {
            h.event(Event::PointerMoved(pos2(400.0, 350.0)));
            h.event(Event::PointerButton {
                pos: pos2(400.0, 350.0),
                button: PointerButton::Middle,
                pressed: true,
                modifiers: Modifiers::NONE,
            });
            h.run_steps(2);
            h.event(Event::PointerMoved(pos2(590.0, 440.0)));
            h.run_steps(3);
            h.event(Event::PointerButton {
                pos: pos2(590.0, 440.0),
                button: PointerButton::Middle,
                pressed: false,
                modifiers: Modifiers::NONE,
            });
            h.run_steps(3);
        }
        h.render()
            .map_err(std::io::Error::other)?
            .save(format!("{out}/{name}.png"))?;
        println!("Captured {name}");
    }
    Ok(())
}
