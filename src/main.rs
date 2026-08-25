use eframe::egui;
use pdf_manager_rust::app::PdfManagerApp;

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([780.0, 580.0])
        .with_min_inner_size([680.0, 520.0])
        .with_title("simple PDF Manager");

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "simple PDF Manager",
        options,
        Box::new(|_cc| Ok(Box::new(PdfManagerApp::default()))),
    )
}
