use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Read the total number of pages in a PDF file.
pub fn get_page_count(path: &str) -> Result<u32> {
    let doc = Document::load(path).context("Failed to open PDF")?;
    Ok(doc.get_pages().len() as u32)
}

/// Merge multiple PDFs into a single output PDF.
pub fn merge_pdfs(input_paths: &[String], output_path: &str) -> Result<()> {
    if input_paths.len() < 2 {
        return Err(anyhow!("Please provide at least 2 PDFs to merge."));
    }

    let mut base = Document::load(&input_paths[0])
        .with_context(|| format!("Failed to open {}", input_paths[0]))?;
    let mut base_objects = base.objects.clone();

    for input in &input_paths[1..] {
        let other = Document::load(input)
            .with_context(|| format!("Failed to open {}", input))?;
        let other_pages = other.get_pages();
        let other_objects = other.objects;

        let max_id = base_objects
            .keys()
            .map(|(n, _)| *n)
            .max()
            .unwrap_or(0);

        let mut id_remap: BTreeMap<u32, ObjectId> = BTreeMap::new();
        for &(n, _) in other_objects.keys() {
            id_remap.insert(n, (max_id + 1 + n, 0));
        }

        for ((n, _g), obj) in other_objects {
            let remapped = remap_object(obj, &id_remap);
            base_objects.insert(id_remap[&n], remapped);
        }

        for (_pn, page_id) in &other_pages {
            let new_page_id = id_remap[&page_id.0];
            base_objects.insert(new_page_id, Object::Reference(new_page_id));
        }

        let catalog = base
            .catalog()
            .context("Base PDF is missing a catalog.")?
            .clone();
        let pages_id = catalog
            .get(b"Pages")
            .context("Catalog missing /Pages reference")?
            .as_reference()
            .context("/Pages is not a reference")?;
        let pages_dict = base
            .objects
            .get(&pages_id)
            .context("Pages dictionary not found")?
            .clone();
        let mut pages_dict = pages_dict
            .as_dict()
            .context("/Pages is not a dictionary")?
            .clone();
        let kids = pages_dict
            .get_mut(b"Kids")
            .context("Pages dictionary missing /Kids")?
            .as_array_mut()
            .context("/Kids is not an array")?;
        for (_pn, page_id) in &other_pages {
            kids.push(Object::Reference(id_remap[&page_id.0]));
        }
        let count_obj = pages_dict
            .get(b"Count")
            .context("/Pages missing /Count")?
            .as_i64()
            .context("/Count is not an integer")?;
        pages_dict.set(
            b"Count",
            Object::Integer(count_obj + other_pages.len() as i64),
        );
        base.objects
            .insert(pages_id, Object::Dictionary(pages_dict));
    }

    base.objects = base_objects;
    base.compress();
    base.save(output_path)
        .context("Failed to save merged PDF")?;
    Ok(())
}

fn remap_object(obj: Object, id_remap: &BTreeMap<u32, ObjectId>) -> Object {
    match obj {
        Object::Reference((n, _g)) => {
            if let Some(&(new_n, _)) = id_remap.get(&n) {
                Object::Reference((new_n, 0))
            } else {
                Object::Reference((n, _g))
            }
        }
        Object::Array(a) => {
            let new_a: Vec<Object> = a
                .into_iter()
                .map(|o| remap_object(o, id_remap))
                .collect();
            Object::Array(new_a)
        }
        Object::Dictionary(d) => {
            let mut new_d = d;
            for (_, v) in new_d.iter_mut() {
                let original = std::mem::replace(v, Object::Null);
                *v = remap_object(original, id_remap);
            }
            Object::Dictionary(new_d)
        }
        Object::Stream(s) => {
            let mut new_s = s;
            let remapped = remap_object(Object::Dictionary(new_s.dict), id_remap);
            new_s.dict = match remapped {
                Object::Dictionary(d) => d,
                _ => unreachable!(),
            };
            Object::Stream(new_s)
        }
        other => other,
    }
}

/// Extract a range of pages from a PDF and save as a new PDF.
pub fn extract_pages(input: &str, output: &str, start: u32, end: u32) -> Result<()> {
    let mut doc = Document::load(input).context("Failed to open source PDF")?;
    let total = doc.get_pages().len() as u32;
    if start < 1 || end > total || start > end {
        anyhow::bail!(
            "Invalid page range: {}-{} (PDF has {} pages)",
            start,
            end,
            total
        );
    }
    let pages_to_remove: Vec<u32> = (1..=total).filter(|p| *p < start || *p > end).collect();
    if pages_to_remove.is_empty() {
        fs::copy(input, output).context("Failed to copy PDF")?;
        return Ok(());
    }
    let mut reverse = pages_to_remove;
    reverse.reverse();
    doc.delete_pages(&reverse);
    doc.compress();
    doc.save(output).context("Failed to save extracted PDF")?;
    Ok(())
}

/// Split a PDF into N files (one page per file) inside the given output directory.
pub fn split_pdf(input: &str, output_dir: &str) -> Result<Vec<String>> {
    let total = {
        let d = Document::load(input).context("Failed to open source PDF")?;
        d.get_pages().len() as u32
    };
    let stem = Path::new(input)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "page".to_string());

    fs::create_dir_all(output_dir).context("Failed to create output directory")?;
    let mut produced: Vec<String> = Vec::with_capacity(total as usize);

    for page_num in 1..=total {
        let mut doc = Document::load(input).context("Failed to reopen source PDF")?;
        let to_remove: Vec<u32> = (1..=total).filter(|p| *p != page_num).collect();
        let mut reverse = to_remove;
        reverse.reverse();
        doc.delete_pages(&reverse);
        let out = Path::new(output_dir).join(format!("{}_page_{:04}.pdf", stem, page_num));
        doc.save(&out)
            .with_context(|| format!("Failed to write page {} of split", page_num))?;
        produced.push(out.to_string_lossy().to_string());
    }
    Ok(produced)
}

/// Remove specific pages from a PDF and save as a new file.
pub fn remove_pages(input: &str, output: &str, pages_to_remove: &[u32]) -> Result<()> {
    let mut doc = Document::load(input).context("Failed to open source PDF")?;
    let total_pages = doc.get_pages().len() as u32;
    for &p in pages_to_remove {
        if p < 1 || p > total_pages {
            anyhow::bail!(
                "Page {} is out of range (PDF has {} pages)",
                p,
                total_pages
            );
        }
    }
    let mut unique: Vec<u32> = pages_to_remove
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if unique.len() as u32 >= total_pages {
        anyhow::bail!("Cannot remove all pages of a PDF.");
    }
    unique.reverse();
    doc.delete_pages(&unique);
    doc.compress();
    doc.save(output).context("Failed to save modified PDF")?;
    Ok(())
}
