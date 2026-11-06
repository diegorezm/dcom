#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod mock;
mod theme;

fn main() -> eframe::Result {
    let bench_output = std::env::var_os("DCOM_BENCH");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("dcom")
            .with_inner_size([1240.0, 780.0])
            .with_min_inner_size([900.0, 560.0]),
        // No benchmark, o vsync limitaria a medição à taxa do monitor.
        glow_options: eframe::egui_glow::GlowConfiguration {
            vsync: bench_output.is_none(),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "dcom",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(app::App::new(bench_output)))
        }),
    )
}
