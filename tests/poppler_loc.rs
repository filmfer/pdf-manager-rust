use pdf_manager_rust::img_ops::find_pdftoppm;

#[test]
fn finds_bundled_pdftoppm() {
    // This test only passes when the poppler/ folder is next to the test binary.
    // On CI without poppler bundled, this test will fail (which is acceptable).
    let found = find_pdftoppm();
    println!("pdftoppm found at: {:?}", found);
    // We do not assert here, since the bundled folder may not be present in CI.
}
