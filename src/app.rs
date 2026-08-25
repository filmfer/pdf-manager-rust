use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui::{self, Color32, Frame, Vec2};

use crate::img_ops;
use crate::pdf_ops;

// Colour palette
const ACCENT: Color32 = Color32::from_rgb(0x00, 0x78, 0xd4);
const BG: Color32 = Color32::from_rgb(0xf0, 0xf0, 0xf0);
const FOOTER: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x8a);
const ERROR_COLOR: Color32 = Color32::from_rgb(0xcc, 0x33, 0x33);
const SUCCESS_COLOR: Color32 = Color32::from_rgb(0x33, 0x99, 0x33);

// App metadata
const APP_NAME: &str = "simple PDF Manager";
const AUTHOR: &str = "Filipe Fernandes";
const AUTHOR_EMAIL: &str = "filmfer@gmail.com";

#[derive(Clone)]
enum OperationMsg {
    Success(String),
    Error(String),
}

#[derive(Default)]
struct DialogState {
    open: bool,
    message: Option<String>,
    is_error: bool,
}

impl DialogState {
    fn show(&mut self, msg: String, is_error: bool) {
        self.open = true;
        self.message = Some(msg);
        self.is_error = is_error;
    }
    fn close(&mut self) {
        self.open = false;
        self.message = None;
        self.is_error = false;
    }
}

pub struct PdfManagerApp {
    tx: Sender<OperationMsg>,
    rx: Receiver<OperationMsg>,
    dialog: DialogState,
    extract_open: bool,
    selected_pdf_path: Option<String>,
    selected_pdf_page_count: u32,
    extract_start: u32,
    extract_end: u32,
    remove_open: bool,
    remove_input_path: Option<String>,
    remove_input_page_count: u32,
    remove_pages_text: String,
    pdf_to_img_open: bool,
    export_input_path: Option<String>,
    export_input_page_count: u32,
    export_start: u32,
    export_end: u32,
    export_dpi: u32,
    export_format: String,
}

impl Default for PdfManagerApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            tx,
            rx,
            dialog: DialogState::default(),
            extract_open: false,
            selected_pdf_path: None,
            selected_pdf_page_count: 0,
            extract_start: 1,
            extract_end: 1,
            remove_open: false,
            remove_input_path: None,
            remove_input_page_count: 0,
            remove_pages_text: String::new(),
            pdf_to_img_open: false,
            export_input_path: None,
            export_input_page_count: 0,
            export_start: 1,
            export_end: 1,
            export_dpi: 150,
            export_format: "png".to_string(),
        }
    }
}

impl eframe::App for PdfManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages(ctx);

        egui::CentralPanel::default()
            .frame(Frame::default().fill(BG))
            .show(ctx, |ui| {
                self.draw_header(ui);
                self.draw_buttons(ui);
                self.draw_footer(ui);
            });

        self.show_extract_dialog(ctx);
        self.show_remove_dialog(ctx);
        self.show_pdf_to_images_dialog(ctx);
        self.show_message_dialog(ctx);
    }
}

impl PdfManagerApp {
    fn draw_header(&self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.heading(egui::RichText::new(APP_NAME).size(28.0).strong());
        });
        ui.add_space(24.0);
    }

    fn draw_buttons(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let btn_width = 280.0;
                    let btn_height = 44.0;
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Merge PDFs").size(15.0))
                                .min_size(Vec2::new(btn_width, btn_height))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.action_merge();
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Extract Pages").size(15.0))
                                .min_size(Vec2::new(btn_width, btn_height))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.action_open_extract();
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Remove Pages").size(15.0))
                                .min_size(Vec2::new(btn_width, btn_height))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.action_open_remove();
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Split PDF").size(15.0))
                                .min_size(Vec2::new(btn_width, btn_height))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.action_split();
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("Images to PDF").size(15.0))
                                .min_size(Vec2::new(btn_width, btn_height))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.action_images_to_pdf();
                    }
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("PDF to Images").size(15.0))
                                .min_size(Vec2::new(btn_width, btn_height))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.action_open_pdf_to_images();
                    }
                });
            });
    }

    fn draw_footer(&self, ui: &mut egui::Ui) {
        ui.add_space(24.0);
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(format!("{} <{}>", AUTHOR, AUTHOR_EMAIL))
                    .small()
                    .color(FOOTER),
            );
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Built with Rust + egui")
                    .small()
                    .color(FOOTER),
            );
        });
    }

    fn process_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                OperationMsg::Success(m) => {
                    self.dialog.show(m, false);
                    ctx.request_repaint();
                }
                OperationMsg::Error(m) => {
                    self.dialog.show(m, true);
                    ctx.request_repaint();
                }
            }
        }
    }

    fn action_merge(&mut self) {
        let tx = self.tx.clone();
        let files = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select PDFs to merge (Ctrl+click for multiple)")
            .pick_files();
        let files: Vec<String> = match files {
            Some(f) => f.iter().map(|f| f.to_string_lossy().to_string()).collect(),
            None => return,
        };
        if files.len() < 2 {
            let _ = tx.send(OperationMsg::Error(
                "Please select at least 2 PDF files.".to_string(),
            ));
            return;
        }
        let output = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Save merged PDF as")
            .save_file();
        let output = match output {
            Some(o) => ensure_pdf_extension(o),
            None => return,
        };
        let count = files.len();
        thread::spawn(move || match pdf_ops::merge_pdfs(&files, &output) {
            Ok(_) => {
                let _ = tx.send(OperationMsg::Success(format!(
                    "Merged {} PDFs successfully!\nSaved to:\n{}",
                    count, output
                )));
            }
            Err(e) => {
                let _ = tx.send(OperationMsg::Error(format!("Failed to merge PDFs:\n{}", e)));
            }
        });
    }

    fn action_open_extract(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select a PDF")
            .pick_file();
        let path = match file {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };
        match pdf_ops::get_page_count(&path) {
            Ok(count) => {
                self.selected_pdf_path = Some(path);
                self.selected_pdf_page_count = count;
                self.extract_start = 1;
                self.extract_end = count;
                self.extract_open = true;
            }
            Err(e) => {
                let _ = self
                    .tx
                    .send(OperationMsg::Error(format!("Failed to read PDF:\n{}", e)));
            }
        }
    }

    fn show_extract_dialog(&mut self, ctx: &egui::Context) {
        if !self.extract_open {
            return;
        }
        let mut open = self.extract_open;
        egui::Window::new("Extract Pages")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(p) = &self.selected_pdf_path {
                    ui.label(format!(
                        "PDF: {}",
                        std::path::Path::new(p)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.clone())
                    ));
                }
                ui.label(format!("Total pages: {}", self.selected_pdf_page_count));
                ui.add_space(8.0);

                egui::Grid::new("extract_grid")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("From page:");
                        ui.add(
                            egui::DragValue::new(&mut self.extract_start)
                                .range(1..=self.selected_pdf_page_count.max(1))
                                .clamp_existing_to_range(true),
                        );
                        ui.end_row();
                        ui.label("To page:");
                        ui.add(
                            egui::DragValue::new(&mut self.extract_end)
                                .range(1..=self.selected_pdf_page_count.max(1))
                                .clamp_existing_to_range(true),
                        );
                        ui.end_row();
                    });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Extract")
                                .min_size(Vec2::new(100.0, 32.0))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.perform_extract();
                    }
                    if ui
                        .add(egui::Button::new("Cancel").min_size(Vec2::new(100.0, 32.0)))
                        .clicked()
                    {
                        self.extract_open = false;
                    }
                });
            });
        self.extract_open = open;
    }

    fn perform_extract(&mut self) {
        let path = match self.selected_pdf_path.clone() {
            Some(p) => p,
            None => return,
        };
        let start = self.extract_start;
        let end = self.extract_end;
        if start < 1 || end > self.selected_pdf_page_count || start > end {
            let _ = self.tx.send(OperationMsg::Error(format!(
                "Invalid page range. PDF has {} pages, requested {}-{}",
                self.selected_pdf_page_count, start, end
            )));
            return;
        }
        let output = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Save extracted PDF as")
            .save_file();
        let output = match output {
            Some(o) => ensure_pdf_extension(o),
            None => return,
        };
        self.extract_open = false;
        let tx = self.tx.clone();
        thread::spawn(move || match pdf_ops::extract_pages(&path, &output, start, end) {
            Ok(_) => {
                let _ = tx.send(OperationMsg::Success(format!(
                    "Pages extracted successfully!\nSaved to:\n{}",
                    output
                )));
            }
            Err(e) => {
                let _ = tx.send(OperationMsg::Error(format!(
                    "Failed to extract pages:\n{}",
                    e
                )));
            }
        });
    }

    fn action_open_remove(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select a PDF")
            .pick_file();
        let path = match file {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };
        match pdf_ops::get_page_count(&path) {
            Ok(count) => {
                self.remove_input_path = Some(path);
                self.remove_input_page_count = count;
                self.remove_pages_text.clear();
                self.remove_open = true;
            }
            Err(e) => {
                let _ = self
                    .tx
                    .send(OperationMsg::Error(format!("Failed to read PDF:\n{}", e)));
            }
        }
    }

    fn show_remove_dialog(&mut self, ctx: &egui::Context) {
        if !self.remove_open {
            return;
        }
        let mut open = self.remove_open;
        egui::Window::new("Remove Pages")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(p) = &self.remove_input_path {
                    ui.label(format!(
                        "PDF: {}",
                        std::path::Path::new(p)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.clone())
                    ));
                }
                ui.label(format!("Total pages: {}", self.remove_input_page_count));
                ui.add_space(4.0);
                ui.label("Pages to remove (e.g. 1,3,5-7):");
                ui.add(
                    egui::TextEdit::singleline(&mut self.remove_pages_text)
                        .hint_text("1,3,5-7")
                        .desired_width(220.0),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Remove")
                                .min_size(Vec2::new(100.0, 32.0))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.perform_remove();
                    }
                    if ui
                        .add(egui::Button::new("Cancel").min_size(Vec2::new(100.0, 32.0)))
                        .clicked()
                    {
                        self.remove_open = false;
                    }
                });
            });
        self.remove_open = open;
    }

    fn perform_remove(&mut self) {
        let path = match self.remove_input_path.clone() {
            Some(p) => p,
            None => return,
        };
        let pages = match parse_page_list(&self.remove_pages_text) {
            Ok(p) => p,
            Err(e) => {
                let _ = self
                    .tx
                    .send(OperationMsg::Error(format!("Invalid page list:\n{}", e)));
                return;
            }
        };
        if pages.is_empty() {
            let _ = self
                .tx
                .send(OperationMsg::Error("Please specify at least one page.".to_string()));
            return;
        }
        let total = self.remove_input_page_count;
        for &p in &pages {
            if p < 1 || p > total {
                let _ = self.tx.send(OperationMsg::Error(format!(
                    "Page {} is out of range (PDF has {} pages)",
                    p, total
                )));
                return;
            }
        }
        let output = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Save modified PDF as")
            .save_file();
        let output = match output {
            Some(o) => ensure_pdf_extension(o),
            None => return,
        };
        self.remove_open = false;
        let tx = self.tx.clone();
        thread::spawn(move || match pdf_ops::remove_pages(&path, &output, &pages) {
            Ok(_) => {
                let _ = tx.send(OperationMsg::Success(format!(
                    "Pages removed successfully!\nSaved to:\n{}",
                    output
                )));
            }
            Err(e) => {
                let _ = tx.send(OperationMsg::Error(format!(
                    "Failed to remove pages:\n{}",
                    e
                )));
            }
        });
    }

    fn action_split(&mut self) {
        let tx = self.tx.clone();
        let file = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select a PDF to split")
            .pick_file();
        let path = match file {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };
        let output_dir = rfd::FileDialog::new()
            .set_title("Select output folder")
            .pick_folder();
        let output_dir = match output_dir {
            Some(d) => d.to_string_lossy().to_string(),
            None => return,
        };
        thread::spawn(move || match pdf_ops::split_pdf(&path, &output_dir) {
            Ok(files) => {
                let _ = tx.send(OperationMsg::Success(format!(
                    "PDF split successfully!\n{} page(s) saved to:\n{}",
                    files.len(),
                    output_dir
                )));
            }
            Err(e) => {
                let _ = tx.send(OperationMsg::Error(format!("Failed to split PDF:\n{}", e)));
            }
        });
    }

    fn action_images_to_pdf(&mut self) {
        let tx = self.tx.clone();
        let files = rfd::FileDialog::new()
            .add_filter(
                "Image Files",
                &[
                    "png", "jpg", "jpeg", "jpe", "jfif", "gif", "bmp", "dib", "tif", "tiff",
                    "webp", "ico", "ppm", "pgm", "pbm",
                ],
            )
            .set_title("Select images (Ctrl+click for multiple)")
            .pick_files();
        let images: Vec<String> = match files {
            Some(f) => f.iter().map(|f| f.to_string_lossy().to_string()).collect(),
            None => return,
        };
        if images.is_empty() {
            let _ = tx.send(OperationMsg::Error(
                "No supported images were found.".to_string(),
            ));
            return;
        }
        let output = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Save PDF as")
            .save_file();
        let output = match output {
            Some(o) => ensure_pdf_extension(o),
            None => return,
        };
        let count = images.len();
        thread::spawn(move || match img_ops::images_to_pdf(&images, &output) {
            Ok(_) => {
                let _ = tx.send(OperationMsg::Success(format!(
                    "Created PDF successfully!\n{} image(s) saved to:\n{}",
                    count, output
                )));
            }
            Err(e) => {
                let _ = tx.send(OperationMsg::Error(format!(
                    "Failed to create PDF:\n{}",
                    e
                )));
            }
        });
    }

    fn action_open_pdf_to_images(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select a PDF")
            .pick_file();
        let path = match file {
            Some(f) => f.to_string_lossy().to_string(),
            None => return,
        };
        match pdf_ops::get_page_count(&path) {
            Ok(count) => {
                self.export_input_path = Some(path);
                self.export_input_page_count = count;
                self.export_start = 1;
                self.export_end = count;
                self.export_dpi = 150;
                self.export_format = "png".to_string();
                self.pdf_to_img_open = true;
            }
            Err(e) => {
                let _ = self
                    .tx
                    .send(OperationMsg::Error(format!("Failed to read PDF:\n{}", e)));
            }
        }
    }

    fn show_pdf_to_images_dialog(&mut self, ctx: &egui::Context) {
        if !self.pdf_to_img_open {
            return;
        }
        let mut open = self.pdf_to_img_open;
        egui::Window::new("PDF to Images")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if let Some(p) = &self.export_input_path {
                    ui.label(format!(
                        "PDF: {}",
                        std::path::Path::new(p)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.clone())
                    ));
                }
                ui.label(format!("Total pages: {}", self.export_input_page_count));
                ui.add_space(8.0);

                egui::Grid::new("export_grid")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("From page:");
                        ui.add(
                            egui::DragValue::new(&mut self.export_start)
                                .range(1..=self.export_input_page_count.max(1))
                                .clamp_existing_to_range(true),
                        );
                        ui.end_row();
                        ui.label("To page:");
                        ui.add(
                            egui::DragValue::new(&mut self.export_end)
                                .range(1..=self.export_input_page_count.max(1))
                                .clamp_existing_to_range(true),
                        );
                        ui.end_row();
                        ui.label("DPI:");
                        ui.add(egui::DragValue::new(&mut self.export_dpi).range(50..=600));
                        ui.end_row();
                        ui.label("Format:");
                        egui::ComboBox::from_label("")
                            .selected_text(self.export_format.clone())
                            .show_ui(ui, |ui| {
                                for ext in img_ops::EXPORT_IMAGE_EXTENSIONS {
                                    ui.selectable_value(
                                        &mut self.export_format,
                                        ext.to_string(),
                                        ext.to_uppercase(),
                                    );
                                }
                            });
                        ui.end_row();
                    });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new("Export")
                                .min_size(Vec2::new(100.0, 32.0))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.perform_pdf_to_images();
                    }
                    if ui
                        .add(egui::Button::new("Cancel").min_size(Vec2::new(100.0, 32.0)))
                        .clicked()
                    {
                        self.pdf_to_img_open = false;
                    }
                });
            });
        self.pdf_to_img_open = open;
    }

    fn perform_pdf_to_images(&mut self) {
        let path = match self.export_input_path.clone() {
            Some(p) => p,
            None => return,
        };
        let start = self.export_start;
        let end = self.export_end;
        let dpi = self.export_dpi;
        let format = self.export_format.clone();
        if start < 1 || end > self.export_input_page_count || start > end {
            let _ = self.tx.send(OperationMsg::Error(format!(
                "Invalid page range. PDF has {} pages, requested {}-{}",
                self.export_input_page_count, start, end
            )));
            return;
        }
        let output_dir = rfd::FileDialog::new()
            .set_title("Select output folder for images")
            .pick_folder();
        let output_dir = match output_dir {
            Some(d) => d.to_string_lossy().to_string(),
            None => return,
        };
        self.pdf_to_img_open = false;
        let tx = self.tx.clone();
        thread::spawn(move || {
            match img_ops::pdf_to_images(&path, &output_dir, start, end, dpi, &format) {
                Ok(files) => {
                    let _ = tx.send(OperationMsg::Success(format!(
                        "Exported {} image(s) successfully!\nSaved to:\n{}",
                        files.len(),
                        output_dir
                    )));
                }
                Err(e) => {
                    let _ = tx.send(OperationMsg::Error(format!(
                        "Failed to export images:\n{}",
                        e
                    )));
                }
            }
        });
    }

    fn show_message_dialog(&mut self, ctx: &egui::Context) {
        if !self.dialog.open {
            return;
        }
        let msg = self.dialog.message.clone().unwrap_or_default();
        let is_error = self.dialog.is_error;

        egui::Window::new(if is_error { "Error" } else { "Success" })
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    let color = if is_error { ERROR_COLOR } else { SUCCESS_COLOR };
                    ui.label(egui::RichText::new(msg).color(color).size(13.0));
                    ui.add_space(12.0);
                    if ui
                        .add(
                            egui::Button::new("OK")
                                .min_size(Vec2::new(120.0, 32.0))
                                .fill(ACCENT),
                        )
                        .clicked()
                    {
                        self.dialog.close();
                    }
                });
            });
    }
}

// end impl PdfManagerApp

fn ensure_pdf_extension(path: std::path::PathBuf) -> String {
    let mut s = path.to_string_lossy().to_string();
    if !s.to_lowercase().ends_with(".pdf") {
        s.push_str(".pdf");
    }
    s
}

fn parse_page_list(text: &str) -> anyhow::Result<Vec<u32>> {
    let mut out: Vec<u32> = Vec::new();
    for part in text.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(dash) = part.find('-') {
            let a: u32 = part[..dash].trim().parse()?;
            let b: u32 = part[dash + 1..].trim().parse()?;
            if a == 0 || b == 0 {
                anyhow::bail!("Page numbers must start at 1");
            }
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            for p in lo..=hi {
                if !out.contains(&p) {
                    out.push(p);
                }
            }
        } else {
            let p: u32 = part.parse()?;
            if p == 0 {
                anyhow::bail!("Page numbers must start at 1");
            }
            if !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out.sort_unstable();
    Ok(out)
}
