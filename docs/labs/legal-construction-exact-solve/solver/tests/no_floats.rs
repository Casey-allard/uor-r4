//! Binary floating point may appear only in the input conversion file.

use std::fs;
use std::path::Path;

#[test]
fn floats_only_in_exact_conversion() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut scanned = 0;
    let mut saw_exact = false;
    for entry in fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let text = fs::read_to_string(&path).unwrap();
        scanned += 1;
        if name == "exact.rs" {
            saw_exact = true;
            continue;
        }
        for token in ["f32", "f64"] {
            assert!(!text.contains(token), "{name} contains `{token}`; floats belong only in exact.rs");
        }
    }
    assert!(saw_exact, "exact.rs not found");
    assert!(scanned >= 6, "only {scanned} source files scanned");
}
