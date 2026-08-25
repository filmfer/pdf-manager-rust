# simple PDF Manager (Rust)

A simple, fast, lightweight PDF manager written in **Rust** with a native GUI.

The application replicates the feature set of the original Python/Tkinter
`pdf_manager_gui` while delivering a much smaller, faster, and more
memory-efficient binary.

## Features

- **Merge PDFs** – combine multiple PDF files into a single document
- **Split PDF** – export every page as its own PDF
- **Extract Pages** – save a contiguous range of pages as a new PDF
- **Remove Pages** – delete a list of pages (e.g. `1,3,5-7`) and save the result
- **Images to PDF** – convert one or many images (PNG, JPG, BMP, TIFF, GIF,
  WEBP, ICO, PPM/PGM/PBM) into a PDF, with automatic A4 page fit and JPEG
  re-encoding for size optimisation
- **PDF to Images** – render pages of a PDF to PNG, JPG, BMP, TIFF, GIF, WEBP
  or PPM, with user-selectable DPI

## Why Rust?

| Metric              | Python (Tkinter + PyInstaller) | Rust (eframe/egui) |
| ------------------- | ------------------------------ | ------------------ |
| Binary size         | ~50 MB                         | **~4 MB**          |
| Cold start          | 1–3 s                          | **< 200 ms**       |
| RAM usage (idle)    | ~80 MB                         | **~10 MB**         |
| Distribution        | Requires Python runtime        | Single static `.exe` |

## Tech Stack

- **GUI** – [eframe](https://github.com/emilk/egui) / [egui](https://github.com/emilk/egui) (immediate-mode, GPU-rendered, no webview)
- **PDF** – [lopdf](https://github.com/nickel-org/lopdf) (pure-Rust, lossless, preserves original content)
- **Images** – [image](https://github.com/image-rs/image) crate
- **PDF → Images** – [`pdftoppm`](https://poppler.freedesktop.org/) from the Poppler project
- **File dialogs** – [rfd](https://github.com/PolyMeilex/rfd) (native OS dialogs)

## Installation

### Pre-built binary (Windows)

Download the latest `pdf-manager-rust.exe` from the
[Releases](../../releases) page and run it. No installation required.

### From source

Requirements:

- [Rust](https://rustup.rs/) (stable, edition 2021)
- On Linux/macOS: `poppler-utils` (provides `pdftoppm`)
- On Windows: Poppler for Windows (place `pdftoppm.exe` somewhere on `PATH`,
  or build without the PDF → Images feature)

```bash
git clone https://github.com/<your-user>/pdf-manager-rust
cd pdf-manager-rust
cargo build --release
./target/release/pdf-manager-rust
```

The release binary uses `opt-level = "z"`, LTO, `codegen-units = 1`, and
`strip = true` for the smallest possible size.

## Project Layout

```
src/
├── main.rs        # entry-point, builds the eframe window
├── lib.rs         # exposes app / img_ops / pdf_ops modules
├── app.rs         # the GUI: state, layout, dialogs, async messages
├── pdf_ops.rs     # merge / split / extract / remove (lopdf)
└── img_ops.rs     # images → PDF and PDF → images
```

## Author

Filipe Fernandes — [filmfer@gmail.com](mailto:filmfer@gmail.com)
