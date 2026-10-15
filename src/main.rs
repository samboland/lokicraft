fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 960.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LokiCraft",
        options,
        Box::new(|cc| {
            Ok(Box::new(lokicraft::Shell::new(
                &cc.egui_ctx,
                craft_settings::Location::Auto,
            )))
        }),
    )
}
