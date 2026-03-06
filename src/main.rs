mod app;
mod motor;
mod serial;
mod ui;

use app::App;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Collector Controller")
            .with_inner_size([420.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Collector Controller",
        options,
        Box::new(|_cc| Box::new(App::new())),
    )
}
