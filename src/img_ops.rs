use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, GenericImageView};
use lopdf::Document;
use std::fs;
use std::path::Path;
use std::process::Command;

pub const EXPORT_IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "tiff"];

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

pub fn images_to_pdf(image_paths: &[String], output: &str) -> Result<()> {
    if image_paths.is_empty() {
        return Err(anyhow!("No images provided"));
    }
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_refs: Vec<lopdf::Object> = Vec::new();
    for path_str in image_paths {
        let bytes = fs::read(path_str)
            .with_context(|| format!("Failed to read image: {}", path_str))?;
        let img = image::load_from_memory(&bytes)
            .with_context(|| format!("Failed to decode image: {}", path_str))?;
        let img = downscale_to_fit(img, 2480);
        let (w, h) = img.dimensions();
        let use_jpeg = !matches!(
            img,
            DynamicImage::ImageLuma8(_) | DynamicImage::ImageLumaA8(_)
        );
        let (filter, encoded, color_space) = if use_jpeg {
            let rgb = img.to_rgb8();
            let (iw, ih) = (rgb.width() as i32, rgb.height() as i32);
            let mut buf: Vec<u8> = Vec::new();
            let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 85);
            use image::ImageEncoder as _;
            enc.write_image(
                rgb.as_raw(),
                iw as u32,
                ih as u32,
                image::ExtendedColorType::Rgb8,
            )?;
            ("DCTDecode".to_string(), buf, "DeviceRGB".to_string())
        } else {
            let gray = img.to_luma8();
            let (iw, ih) = (gray.width() as i32, gray.height() as i32);
            let mut buf: Vec<u8> = Vec::new();
            let enc = image::codecs::png::PngEncoder::new(&mut buf);
            use image::ImageEncoder as _;
            enc.write_image(
                gray.as_raw(),
                iw as u32,
                ih as u32,
                image::ExtendedColorType::L8,
            )?;
            ("FlateDecode".to_string(), buf, "DeviceGray".to_string())
        };
        let img_stream = lopdf::Stream::new(lopdf::Dictionary::new(), encoded);
        let img_id = doc.add_object(img_stream);
        let mut img_info = lopdf::Dictionary::new();
        img_info.set("Type", lopdf::Object::Name(b"XObject".to_vec()));
        img_info.set("Subtype", lopdf::Object::Name(b"Image".to_vec()));
        img_info.set("Width", lopdf::Object::Integer(w as i64));
        img_info.set("Height", lopdf::Object::Integer(h as i64));
        img_info.set("ColorSpace", lopdf::Object::Name(color_space.into_bytes()));
        img_info.set("BitsPerComponent", lopdf::Object::Integer(8));
        img_info.set("Filter", lopdf::Object::Name(filter.into_bytes()));
        let xobject_id = doc.add_object(lopdf::Object::Dictionary(img_info));
        let content = format!(
            "q\n{} 0 0 {} 0 0 cm\n/Im0 Do\nQ\n",
            w as f32 / 2.835,
            h as f32 / 2.835
        );
        let content_stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
        let content_id = doc.add_object(content_stream);
        let mut resources = lopdf::Dictionary::new();
        let mut xobjects = lopdf::Dictionary::new();
        xobjects.set("Im0", xobject_id);
        resources.set("XObject", lopdf::Object::Dictionary(xobjects));
        resources.set(
            "ProcSet",
            lopdf::Object::Array(vec![
                lopdf::Object::Name(b"PDF".to_vec()),
                lopdf::Object::Name(b"Text".to_vec()),
                lopdf::Object::Name(b"ImageC".to_vec()),
            ]),
        );
        let mut page_dict = lopdf::Dictionary::new();
        page_dict.set("Type", lopdf::Object::Name(b"Page".to_vec()));
        page_dict.set(
            "MediaBox",
            lopdf::Object::Array(vec![
                lopdf::Object::Real(0.0),
                lopdf::Object::Real(0.0),
                lopdf::Object::Real(w as f32 / 2.835),
                lopdf::Object::Real(h as f32 / 2.835),
            ]),
        );
        page_dict.set("Resources", lopdf::Object::Dictionary(resources));
        page_dict.set("Contents", content_id);
        let page_id = doc.add_object(lopdf::Object::Dictionary(page_dict));
        page_refs.push(lopdf::Object::Reference(page_id));
    }
    let mut pages_dict = lopdf::Dictionary::new();
    pages_dict.set("Type", lopdf::Object::Name(b"Pages".to_vec()));
    pages_dict.set("Count", page_refs.len() as i64);
    pages_dict.set("Kids", page_refs);
    doc.objects
        .insert(pages_id, lopdf::Object::Dictionary(pages_dict));
    let mut catalog_dict = lopdf::Dictionary::new();
    catalog_dict.set("Type", lopdf::Object::Name(b"Catalog".to_vec()));
    catalog_dict.set("Pages", pages_id);
    let catalog_id = doc.add_object(lopdf::Object::Dictionary(catalog_dict));
    doc.trailer.set("Root", catalog_id);
    doc.compress();
    doc.save(output).context("Failed to save PDF")?;
    Ok(())
}

#[allow(dead_code)]
fn _unused() {}

/// Render specific pages of a PDF to image files using the
/// `pdftoppm` command (part of Poppler). This avoids bundling
/// a heavy native PDFium dependency.
pub fn pdf_to_images(
    pdf_path: &str,
    output_dir: &str,
    start: u32,
    end: u32,
    dpi: u32,
    format: &str,
) -> Result<Vec<String>> {
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    let stem = Path::new(pdf_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "page".to_string());
    let template = Path::new(output_dir).join(format!("{}_", stem));
    let template_str = template.to_string_lossy().to_string();
    let mut cmd = Command::new("pdftoppm");
    cmd.arg("-r").arg(dpi.to_string());
    cmd.arg("-f").arg(start.to_string());
    cmd.arg("-l").arg(end.to_string());
    let fmt = format.to_lowercase();
    match fmt.as_str() {
        "png" => {
            cmd.arg("-png");
        }
        "jpg" | "jpeg" => {
            cmd.arg("-jpeg");
        }
        "bmp" => {
            cmd.arg("-bmp");
        }
        "tiff" => {
            cmd.arg("-tiff");
        }
        _ => {
            cmd.arg("-png");
        }
    }
    cmd.arg(pdf_path);
    cmd.arg(&template_str);
    let output = cmd.output().map_err(|e| {
        anyhow!(
            "Could not run 'pdftoppm'.\nPlease install Poppler (https://poppler.freedesktop.org/) and make sure 'pdftoppm' is on your PATH.\n\nUnderlying error: {}",
            e
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("pdftoppm failed:\n{}", stderr));
    }
    let mut produced: Vec<String> = Vec::new();
    let prefix = template
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) {
                produced.push(entry.path().to_string_lossy().to_string());
            }
        }
    }
    produced.sort();
    Ok(produced)
}
