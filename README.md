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
| Binary size         | ~50 MB                         | **~25 MB**¹        |
| Cold start          | 1–3 s                          | **~35 ms**²        |
| RAM usage (idle)    | ~80 MB                         | **~19 MB**³        |
| Distribution        | Requires Python runtime        | Single folder⁴     |

¹ The Windows binary is **~25 MB** because it bundles all 42 Poppler DLLs and
   tools needed for the *PDF → Images* feature, so the `.exe` is fully
   self-contained — copy it to any Windows 10/11 machine and it just runs.
   On Linux/macOS the binary is **~3 MB** (uses the system-installed
   `poppler-utils`).
² Measured cold-start time on a Windows 11 machine (no window pre-warming).
   The native `eframe` window is on screen in well under 50 ms.
³ Working-set size (private + shared) ~19 MB, of which only **~0.5 MB** is
   the app's *private* memory. The rest is shared OS/GPU resources.
⁴ On Windows the binary must live next to the `poppler/` folder (it loads
   `pdftoppm.exe` from there on first use). The GitHub release is distributed
   as a single ZIP that contains both — no installer required.

## Tech Stack

- **GUI** – [eframe](https://github.com/emilk/egui) / [egui](https://github.com/emilk/egui) (immediate-mode, GPU-rendered, no webview)
- **PDF** – [lopdf](https://github.com/nickel-org/lopdf) (pure-Rust, lossless, preserves original content)
- **Images** – [image](https://github.com/image-rs/image) crate
- **PDF → Images** – [`pdftoppm`](https://poppler.freedesktop.org/) from the Poppler project
  (bundled as static binaries on Windows, system-installed `poppler-utils` on Linux/macOS)
- **File dialogs** – [rfd](https://github.com/PolyMeilex/rfd) (native OS dialogs)
- **App icon embedding** – [embed-resource](https://crates.io/crates/embed-resource) (Windows only)

## Installation

### Pre-built binary (Windows)

Download the latest `pdf-manager-rust-windows.zip` from the
[Releases](https://github.com/filmfer/pdf-manager-rust/releases) page,
unzip it anywhere, and double-click `pdf-manager-rust.exe`. No installation
required — the ZIP contains the binary plus the bundled `poppler/` folder.

### From source

Requirements:

- [Rust](https://rustup.rs/) (stable, edition 2021)
- On Linux/macOS: `poppler-utils` (provides `pdftoppm`)
- On Windows: Poppler for Windows (place `pdftoppm.exe` somewhere on `PATH`,
  or build without the PDF → Images feature)

```bash
git clone https://github.com/filmfer/pdf-manager-rust
cd pdf-manager-rust
cargo build --release
./target/release/pdf-manager-rust
```

The release binary uses `opt-level = "z"`, LTO, `codegen-units = 1`, and
`strip = true` for the smallest possible size. On Windows the `poppler/`
folder is also required next to the binary for the *PDF → Images* feature
(it is already in the repo and copied to `target/release/poppler/` by the
build script).

## Project Layout

```
src/
├── main.rs        # entry-point, builds the eframe window
├── lib.rs         # exposes app / img_ops / pdf_ops / setup modules
├── app.rs         # the GUI: state, layout, dialogs, async messages
├── pdf_ops.rs     # merge / split / extract / remove (lopdf)
├── img_ops.rs     # images → PDF and PDF → images (calls pdftoppm)
└── setup.rs       # resolves the Poppler folder at runtime

assets/
├── simple_pdf_manager.ico   # app + task-bar icon
└── poppler/                 # 42 Poppler DLLs and tools (Windows)
```

## Author

Filipe Fernandes — [filmfer@gmail.com](mailto:filmfer@gmail.com)
