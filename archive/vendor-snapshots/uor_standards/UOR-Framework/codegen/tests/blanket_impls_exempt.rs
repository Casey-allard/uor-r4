//! Phase 11c verification: `emit::write_file` preserves files starting
//! with `// @codegen-exempt`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use uor_codegen::emit::write_file;

fn unique_path(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("uor_codegen_{name}_{nanos}.rs"))
}

#[test]
fn codegen_exempt_banner_preserves_existing_content() {
    let path = unique_path("preserved");
    let original = "// @codegen-exempt — hand-written test file.\n\nfn original_content() {}\n";
    std::fs::write(&path, original).expect("write tmp");

    // Attempt to overwrite. Phase 11c says the original content is preserved.
    let regen = "// regenerated content\nfn regen_content() {}\n";
    write_file(&path, regen).expect("write_file");

    let after = std::fs::read_to_string(&path).expect("read tmp");
    assert_eq!(after, original, "@codegen-exempt file was overwritten");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn unmarked_files_are_overwritten_normally() {
    let path = unique_path("overwritten");
    let original = "// regular generated file\nfn old() {}\n";
    std::fs::write(&path, original).expect("write tmp");

    let regen = "// regenerated\nfn brand_new() {}\n";
    write_file(&path, regen).expect("write_file");

    let after = std::fs::read_to_string(&path).expect("read tmp");
    assert!(
        after.contains("brand_new"),
        "unmarked file was not overwritten — got: {after}"
    );

    let _ = std::fs::remove_file(&path);
}
