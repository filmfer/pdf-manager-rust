<div align="center">

# simple PDF Manager

<img width="1100" height="614" alt="image" src="https://github.com/user-attachments/assets/1fd8dd9f-9182-41c0-a9a7-2d83921b77b0" />

**Free, open-source PDF toolbox for 🪟 Windows · 🐧 Linux · 🍎 macOS**

Merge, split, extract and remove pages from PDF files — and convert **images ⇄ PDF** — 100% offline, in a single executable. Built with **Rust** and **egui** 🇵🇹

[![CI](https://github.com/filmfer/pdf-manager-rust/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/filmfer/pdf-manager-rust/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/filmfer/pdf-manager-rust?color=blue&label=release&logo=github)](https://github.com/filmfer/pdf-manager-rust/releases)
[![Downloads](https://img.shields.io/github/downloads/filmfer/pdf-manager-rust/total?color=green)](https://github.com/filmfer/pdf-manager-rust/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust&logoColor=white)](https://rustup.rs/)

![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white&style=flat-square)
![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black&style=flat-square)
![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white&style=flat-square)

</div>

---

## 📚 Table of contents

1. [The story](#the-story)
2. [Features at a glance](#features-at-a-glance)
3. [How it looks](#how-it-looks)
4. [Why Rust? The numbers](#why-rust-the-numbers)
5. [Tech stack](#tech-stack)
6. [Installation](#installation)
7. [How to use](#how-to-use)
8. [Project layout](#project-layout)
9. [FAQ](#faq)
10. [Roadmap](#roadmap)
11. [License](#license)

---

## 📖 The story

> 📎 You receive a 40-page contract split across **five** PDFs.
> Your boss wants it as **one** clean file… *now*.
> So you open a random «free merge PDF» website and quietly hand your
> private contract to a stranger's server 🫣

**simple PDF Manager exists to end that story.**

Every operation runs **entirely on your machine**: nothing is uploaded,
stored or tracked. Sensitive contracts, tax returns and signed documents
never leave your hands 🔒

One small window. **Six big buttons.** Every everyday PDF task done in
seconds — no account, no ads, no wait.

## ✨ Features at a glance

| # | Button | What it does |
|---|--------|--------------|
| 1 | 🧲 **Merge PDFs** | Join several PDF files into one document |
| 2 | ✂️ **Split PDF** | Export every page as its own PDF file |
| 3 | 📄 **Extract Pages** | Save a page range as a brand-new PDF |
| 4 | 🗑️ **Remove Pages** | Delete a page list such as `1,3,5-7` |
| 5 | 🖼️ **Create PDF from Images** | PNG · JPG · BMP · TIFF · GIF · WEBP · ICO → one tidy PDF |
| 6 | 🏞️ **Export Pages to Images** | PDF pages → PNG · JPG · BMP · TIFF · GIF · WEBP · PPM, at your chosen DPI |

---

## 🖥️ How it looks

```
┌───────────────────────────────────────────┐
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│ ░░        simple PDF Manager           ░░ │
│ ░░   Merge · Extract · Remove · Split  ░░ │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
│                                           │
│  ┌─────────────────────────────────────┐  │
│  │            MERGE PDFS               │  │
│  └─────────────────────────────────────┘  │
│  ┌─────────────────────────────────────┐  │
│  │           EXTRACT PAGES             │  │
│  └─────────────────────────────────────┘  │
│  ┌─────────────────────────────────────┐  │
│  │            REMOVE PAGES             │  │
│  └─────────────────────────────────────┘  │
│  ┌─────────────────────────────────────┐  │
│  │       SPLIT PDF 1 PAGE PER FILE     │  │
│  └─────────────────────────────────────┘  │
│  ┌─────────────────────────────────────┐  │
│  │       CREATE PDF FROM IMAGES        │  │
│  └─────────────────────────────────────┘  │
│  ┌─────────────────────────────────────┐  │
│  │       EXPORT PAGES TO IMAGES        │  │
│  └─────────────────────────────────────┘  │
│         (c) 2026 Filipe Fernandes         │
└───────────────────────────────────────────┘
```

*No web view, no Electron bloat — a crisp native window, coloured header,
full-width buttons and instant feedback.*

## ⚡ Why Rust? The numbers

The original Python/Tkinter app worked, but it *felt* like it. Same
feature set, rebuilt with [Rust](https://www.rust-lang.org/):

| Metric | Python (Tkinter + PyInstaller) | Rust (eframe/egui) |
|---|---|---|
| Binary size | ~50 MB | **~25 MB** ¹ |
| Cold start | 1–3 s | **~35 ms** ² |
| RAM (idle) | ~80 MB | **~19 MB** ³ |
| Distribution | Needs a Python runtime | **One single `.exe`** ⁴ |

¹ The Windows `.exe` is a **true single-file app**: all 39 Poppler
binaries (~22 MB of `pdftoppm` + DLLs) are embedded inside it and
self-extracted to `%LOCALAPPDATA%\pdf-manager-rust\poppler\` on first
run. Linux (~7.5 MB) and macOS (~4 MB) builds use the system's
`poppler-utils` instead.  
² Measured cold start on Windows 11 — the native egui window appears in
well under 50 ms.  
³ Working set ~19 MB, of which only **~0.5 MB** is the app's private
memory; the rest is shared OS/GPU resources.  
⁴ Windows 10/11: download `pdf-manager-rust-x86_64-pc-windows-msvc.exe`,
double-click, done. **No installer, no side folder, no runtime.**

### 🌟 Bonus perks

- 🔒 **100% offline & private** — no cloud, no upload, no analytics
- ⚡ **Insta-launch** — open in ~35 ms and get straight to work
- 🪶 **Featherweight** — ~20 MB of RAM; runs happily on old laptops
- 🖱️ **Native file dialogs** — the OS dialog you already know (`rfd`)
- 🎯 **Lossless edits** — page operations copy PDF objects directly with
  `lopdf`, preserving original content instead of re-rendering it
- 🪪 **MIT licensed** — free for work, school and side projects

## 🛠️ Tech stack

| Piece | Choice | Why |
|---|---|---|
| GUI | [eframe / egui](https://github.com/emilk/egui) | Immediate-mode, GPU-rendered, tiny & cross-platform |
| PDF core | [lopdf](https://github.com/nickel-org/lopdf) | Pure Rust — no C toolchain needed for the main path |
| Images | [image](https://github.com/image-rs/image) crate | 8 input formats, 7 output formats |
| PDF → Images | [Poppler](https://poppler.freedesktop.org/) `pdftoppm` | Best-in-class renderer; bundled into the Windows `.exe` |
| Dialogs | [rfd](https://github.com/PolyMeilex/rfd) | Native pickers on Windows, Linux and macOS |
| Icon | [embed-resource](https://crates.io/crates/embed-resource) | `.ico` baked into the Windows binary |

---

## 🚀 Installation

### 🪟 Windows — one file, no install

1. Download **`pdf-manager-rust-x86_64-pc-windows-msvc.exe`** from the
   [latest release](https://github.com/filmfer/pdf-manager-rust/releases).
2. Double-click it. That's it. 🎉
3. First run automatically extracts the bundled Poppler engine to
   `%LOCALAPPDATA%\pdf-manager-rust\poppler\`.

### 🐧 Linux / 🍎 macOS

Install the system Poppler, then grab the matching binary:

```bash
# Debian / Ubuntu
sudo apt install poppler-utils
# Fedora
sudo dnf install poppler-utils
# macOS
brew install poppler
```

### ⌨️ Build from source

Requirements: [Rust](https://rustup.rs/) (stable, edition 2021) and, on
Linux/macOS, Poppler as above.

```bash
git clone https://github.com/filmfer/pdf-manager-rust.git
cd pdf-manager-rust
cargo build --release
```

Binary: `target/release/pdf-manager-rust` (Windows:
`target\release\pdf-manager-rust.exe`).

The release profile is tuned for size: `opt-level = "z"`, full LTO,
`codegen-units = 1`, `strip = true`, `panic = "abort"`.

## 🎮 How to use

1. Launch the app.
2. Click the button for the job:

- **Merge PDFs** → pick two or more PDFs → choose destination → done.
- **Split PDF** → pick one PDF → every page becomes its own file.
- **Extract Pages** → pick a PDF, set the range (e.g. `3–7`) → save.
- **Remove Pages** → pick a PDF, type `1,3,5-7` → save.
- **Create PDF from Images** → select images → one clean PDF.
  *(Pages auto-fit to A4 and large JPEGs are re-encoded to keep the
  output small.)*
- **Export Pages to Images** → PDF, page range, format and DPI (default
  150) → export.

## 📁 Project layout

```text
src/
├── main.rs     # entry-point: window setup + Poppler extraction
├── lib.rs      # module exports
├── app.rs      # GUI: layout, dialogs, background workers
├── pdf_ops.rs  # merge / split / extract / remove (lopdf)
├── img_ops.rs  # images ⇄ PDF (image crate + pdftoppm)
└── setup.rs    # finds / extracts the Poppler binaries
```

## ❓ FAQ

**Why is the Windows executable ~25 MB?**  
All 39 Poppler binaries (~22 MB) are embedded inside it, making
`pdf-manager-rust.exe` a true single-file application.

**Does it send my PDFs anywhere?**  
No. Every operation runs locally — the app contains no networking code.

**“Export Pages to Images” doesn’t work on Linux/macOS?**  
Install `poppler-utils` / `poppler` (see [Installation](#installation)).
Windows needs nothing — its own copy is embedded.

**Can I remove non-sequential pages?**  
Yes: type a list such as `2,5,9-12`. (Extract is a contiguous range.)

**Is there a command-line version?**  
Not yet — it’s on the [Roadmap](#roadmap).

## 🗺️ Roadmap

- [ ] Password-protect / decrypt PDFs
- [ ] Rotate & crop pages
- [ ] Page-thumbnail preview
- [ ] Drag & drop files onto the window
- [ ] CLI mode for scripting
- [ ] SVG and PDF/A export

## 🤝 Contributing

Questions, bugs and ideas are welcome — open an
[issue](https://github.com/filmfer/pdf-manager-rust/issues) or send a
[pull request](https://github.com/filmfer/pdf-manager-rust/pulls).
Please keep `cargo fmt` and `cargo clippy --all-targets -- -D warnings`
green.

## 👤 Author

**Filipe Fernandes**
📧 filmfer@gmail.com

## ⚖️ License

MIT © [Filipe Fernandes](mailto:filmfer@gmail.com).

Enjoying it? A ⭐ on GitHub is the fastest way to say thanks! 🇵🇹

---

<sub>✨ **simple PDF Manager** — free PDF merger · PDF splitter · extract
pages from PDF · remove pages from PDF · images to PDF converter · PDF to
JPG/PNG converter · Rust PDF tool · open source PDF editor.</sub>

<!-- SEO: pdf merge split extract remove, images to pdf, pdf to images,
rust egui pdf manager, free offline pdf tools, standalone exe pdf -->
