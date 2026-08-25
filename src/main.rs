// On Windows, build the binary as a *GUI* application so that no console
// window pops up alongside the main window when the user double-clicks the
// `.exe`. The `windows_subsystem` attribute is a no-op on other platforms.
#![cfg_attr(windows, windows_subsystem = "windows")]

use eframe::egui;
use pdf_manager_rust::app::PdfManagerApp;

fn main() -> eframe::Result<()> {
    // Extract the bundled Poppler binaries (pdftoppm.exe + DLLs) to
    // %LOCALAPPDATA%\pdf-manager-rust\poppler\ on first run. This makes
    // the app fully standalone: the user only needs pdf-manager-rust.exe.
    // If extraction fails (corrupt bundle, permission denied, etc.) we
    // log it but continue - the app will still work for everything except
    // PDF-to-Images, which will report a friendly error.
    if let Err(e) = pdf_manager_rust::setup::ensure_poppler() {
        eprintln!("Warning: could not extract bundled Poppler: {}", e);
    }

    // Match the geometry of the original Tkinter app exactly: `560x520`
    // initial, min `460x400`. This makes the Rust build look and feel
    // identical to the Python version of "simple PDF Manager".
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([560.0, 520.0])
        .with_min_inner_size([460.0, 400.0])
        .with_resizable(false)
        .with_title("simple PDF Manager")
        .with_app_id("com.filmfer.simple-pdf-manager");

    // Try to load the application icon (used for the window decoration
    // and the taskbar; the .exe itself already embeds the icon on Windows
    // via the `build.rs` script).
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut viewport = viewport;
    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

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

/// Decode the bundled .ico file into a single RGBA image that eframe can use
/// as a window icon. Returns `None` if the icon cannot be loaded.
///
/// We rely on `image::load_from_memory` which automatically detects the
/// format from the bytes (ICO, PNG, BMP, etc.) and uses the highest
/// resolution frame available.
fn load_app_icon() -> Option<egui::IconData> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("simple_pdf_manager.ico");
    let bytes = std::fs::read(&path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width: w,
        height: h,
    })
}
