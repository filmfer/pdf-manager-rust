use image::DynamicImage;
use lopdf::{Document, Object, Stream, Dictionary};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(r"D:\scripts\pdf-manager-rust\test_imgs\red.png")?;
    let img = image::load_from_memory(&bytes)?;
    let (w, h) = img.dimensions();
    let rgb = img.to_rgb8();
    let mut buf: Vec<u8> = Vec::new();
    let enc = image::codecs::png::PngEncoder::new(&mut buf);
    use image::ImageEncoder as _;
    enc.write_image(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)?;
    let img_stream = Stream::new(Dictionary::new(), buf);
    let mut doc = Document::with_version("1.5");
    let img_id = doc.add_object(img_stream);
    let mut info = Dictionary::new();
    info.set("Type", Object::Name(b"XObject".to_vec()));
    info.set("Subtype", Object::Name(b"Image".to_vec()));
    info.set("Width", Object::Integer(w as i64));
    info.set("Height", Object::Integer(h as i64));
    info.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    info.set("BitsPerComponent", Object::Integer(8));
    info.set("Filter", Object::Name(b"FlateDecode".to_vec()));
    let xobject_id = doc.add_object(Object::Dictionary(info));
    let mut resources = Dictionary::new();
    resources.set("XObject", lopdf::Object::Dictionary({
        let mut m = Dictionary::new();
        m.set("Im0", xobject_id);
        m
    }));
    let content = format!("q\n{} 0 0 {} 0 0 cm\n/Im0 Do\nQ\n", w as f32 / 2.835, h as f32 / 2.835);
    let content_id = doc.add_object(Stream::new(Dictionary::new(), content.into_bytes()));
    let mut page_dict = Dictionary::new();
    page_dict.set("Type", Object::Name(b"Page".to_vec()));
    page_dict.set("MediaBox", Object::Array(vec![
        Object::Real(0.0), Object::Real(0.0),
        Object::Real(w as f32 / 2.835), Object::Real(h as f32 / 2.835),
    ]));
    page_dict.set("Resources", Object::Dictionary(resources));
    page_dict.set("Contents", content_id);
    let page_id = doc.add_object(Object::Dictionary(page_dict));
    let mut pages = Dictionary::new();
    pages.set("Type", Object::Name(b"Pages".to_vec()));
    pages.set("Count", Object::Integer(1));
    pages.set("Kids", Object::Array(vec![Object::Reference(page_id)]));
    let pages_id = doc.add_object(Object::Dictionary(pages));
    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", pages_id);
    let _img_id_unused = img_id;
    let catalog_id = doc.add_object(Object::Dictionary(catalog));
    doc.trailer.set("Root", catalog_id);
    doc.save(r"D:\scripts\pdf-manager-rust\test_imgs\test.pdf")?;
    println!("Created test PDF");
    Ok(())
}
