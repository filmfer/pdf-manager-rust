use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, GenericImageView, ImageReader};
use lopdf::{Document, Object};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

use crate::setup::HiddenCommand;

pub const EXPORT_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "tiff"];

/// Downscale an image so its longest side is at most `max_long_side` pixels.
fn downscale_to_fit(img: DynamicImage, max_long_side: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let long = w.max(h);
    if long > max_long_side {
        let scale = max_long_side as f64 / long as f64;
        let new_w = (w as f64 * scale).round() as u32;
        let new_h = (h as f64 * scale).round() as u32;
        img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    }
}

/// Convert one image file to (encoded bytes, filter name, color space name, w, h).
/// Uses JPEG @ 85% quality for photos, PNG (lossless) for grayscale text-style images.
fn encode_image(path_str: &str) -> Result<(Vec<u8>, String, String, u32, u32)> {
    let bytes =
        fs::read(path_str).with_context(|| format!("Failed to read image: {}", path_str))?;
    let img = ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .with_context(|| format!("Failed to detect format: {}", path_str))?
        .decode()
        .with_context(|| format!("Failed to decode image: {}", path_str))?;
    let img = downscale_to_fit(img, 2480);
    let (w, h) = img.dimensions();
    let use_jpeg = !matches!(
        img,
        DynamicImage::ImageLuma8(_) | DynamicImage::ImageLumaA8(_)
    );
    let (filter, encoded, color_space) = if use_jpeg {
        let rgb = img.to_rgb8();
        let mut buf: Vec<u8> = Vec::new();
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 85);
        use image::ImageEncoder as _;
        enc.write_image(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)?;
        ("DCTDecode".to_string(), buf, "DeviceRGB".to_string())
    } else {
        let gray = img.to_luma8();
        let mut buf: Vec<u8> = Vec::new();
        let enc = image::codecs::png::PngEncoder::new(&mut buf);
        use image::ImageEncoder as _;
        enc.write_image(gray.as_raw(), w, h, image::ExtendedColorType::L8)?;
        ("FlateDecode".to_string(), buf, "DeviceGray".to_string())
    };
    Ok((encoded, filter, color_space, w, h))
}
/// Build a new PDF document from a list of image file paths.
/// Each image becomes a page sized to the image dimensions (1pt = 1/72 inch).
pub fn images_to_pdf(image_paths: &[String], output: &str) -> Result<()> {
    if image_paths.is_empty() {
        return Err(anyhow!("No images provided"));
    }
    let mut doc = Document::with_version("1.5");
    let mut page_refs: Vec<Object> = Vec::new();

    for (idx, path_str) in image_paths.iter().enumerate() {
        let (encoded, filter, color_space, w, h) = encode_image(path_str)?;
        // The actual image stream - we save it to reuse its object id.
        let img_stream = lopdf::Stream::new(lopdf::Dictionary::new(), encoded.clone());
        let _img_obj_id = doc.add_object(img_stream);

        let mut img_info = lopdf::Dictionary::new();
        img_info.set("Type", Object::Name(b"XObject".to_vec()));
        img_info.set("Subtype", Object::Name(b"Image".to_vec()));
        img_info.set("Width", Object::Integer(w as i64));
        img_info.set("Height", Object::Integer(h as i64));
        img_info.set("ColorSpace", Object::Name(color_space.into_bytes()));
        img_info.set("BitsPerComponent", Object::Integer(8));
        img_info.set("Filter", Object::Name(filter.into_bytes()));
        let xobject_id = doc.add_object(Object::Dictionary(img_info));

        // 1pt = 1/72 inch. We size each page to image dimensions (1px = 1pt) which
        // is a common "screen" size (~72 DPI). The image preserves its full quality.
        let w_pt = w as f32;
        let h_pt = h as f32;
        let content = format!("q\n{} 0 0 {} 0 0 cm\n/Im{} Do\nQ\n", w_pt, h_pt, idx);
        let content_stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
        let content_id = doc.add_object(content_stream);

        let mut resources = lopdf::Dictionary::new();
        let mut xobjects = lopdf::Dictionary::new();
        xobjects.set(format!("Im{}", idx), xobject_id);
        resources.set("XObject", Object::Dictionary(xobjects));
        resources.set(
            "ProcSet",
            Object::Array(vec![
                Object::Name(b"PDF".to_vec()),
                Object::Name(b"ImageC".to_vec()),
                Object::Name(b"ImageB".to_vec()),
                Object::Name(b"ImageI".to_vec()),
            ]),
        );

        let mut page_dict = lopdf::Dictionary::new();
        page_dict.set("Type", Object::Name(b"Page".to_vec()));
        page_dict.set("Parent", Object::Reference((0, 0)));
        page_dict.set(
            "MediaBox",
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(w_pt),
                Object::Real(h_pt),
            ]),
        );
        page_dict.set("Resources", Object::Dictionary(resources));
        page_dict.set("Contents", Object::Reference(content_id));

        let page_id = doc.add_object(Object::Dictionary(page_dict));
        page_refs.push(Object::Reference(page_id));
    }

    let pages_id = doc.add_object({
        let mut d = lopdf::Dictionary::new();
        d.set("Type", Object::Name(b"Pages".to_vec()));
        d.set("Kids", Object::Array(page_refs.clone()));
        d.set("Count", Object::Integer(page_refs.len() as i64));
        Object::Dictionary(d)
    });

    let mut catalog = lopdf::Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    let catalog_id = doc.add_object(Object::Dictionary(catalog));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    doc.save(output).context("Failed to save PDF")?;
    Ok(())
}
/// Convert a range of pages of a PDF to image files using `pdftoppm` (Poppler).
/// Returns the list of output file paths produced.
pub fn pdf_to_images(
    pdf_path: &str,
    output_dir: &str,
    first_page: u32,
    last_page: u32,
    dpi: u32,
    image_format: &str,
) -> Result<Vec<String>> {
    if first_page == 0 || last_page < first_page {
        return Err(anyhow!("Invalid page range"));
    }
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output folder: {}", output_dir))?;

    let pdftoppm = crate::setup::find_pdftoppm()
        .ok_or_else(|| anyhow!("pdftoppm.exe not found. It should be bundled with the app."))?;

    let stem = Path::new(pdf_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "page".to_string());

    // -r DPI  -f first  -l last  -png | -jpeg  prefix
    let fmt_flag = match image_format.to_lowercase().as_str() {
        "jpg" | "jpeg" => "-jpeg",
        "bmp" => "-bmp",
        "tiff" | "tif" => "-tiff",
        _ => "-png", // png (default)
    };

    let status = Command::new(&pdftoppm)
        .arg(fmt_flag)
        .arg("-r")
        .arg(dpi.to_string())
        .arg("-f")
        .arg(first_page.to_string())
        .arg("-l")
        .arg(last_page.to_string())
        .arg(pdf_path)
        .arg(format!(
            "{}/{}",
            output_dir.trim_end_matches('/').trim_end_matches('\\'),
            stem
        ))
        // On Windows, pdftoppm.exe is a *console* application, so without
        // this flag the OS would inherit (or create) a console window that
        // flashes briefly every time the user runs PDF -> Images. The
        // CREATE_NO_WINDOW flag (0x0800_0000) tells the OS to launch it
        // silently, which is exactly what we want for a GUI app.
        .apply_no_window_flag()
        .status()
        .with_context(|| format!("Failed to launch pdftoppm at {:?}", pdftoppm))?;

    if !status.success() {
        return Err(anyhow!("pdftoppm exited with code {:?}", status.code()));
    }

    // Collect the output files.
    let ext = match image_format.to_lowercase().as_str() {
        "jpg" | "jpeg" => "jpg",
        "bmp" => "bmp",
        "tiff" | "tif" => "tif",
        _ => "png",
    };
    let mut out: Vec<String> = Vec::new();
    let dir = Path::new(output_dir);
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if let Some(e) = p.extension() {
                if e.eq_ignore_ascii_case(ext) {
                    if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                        if name.starts_with(&stem) {
                            out.push(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    out.sort();
    Ok(out)
}
