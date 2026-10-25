//! Build script that embeds `crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs`
//! into `$OUT_DIR/mk_synthetic_stack_included.rs` with inner doc comments (`//!`)
//! normalized to regular comments (`//`) so `include!` compiles cleanly without
//! duplicating the synthetic `UORLUT01` fixture builder.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir.join("../crates/uor-r4-integer/src/bin/mk-synthetic-stack.rs");
    println!("cargo:rerun-if-changed={}", src_path.display());

    let raw = fs::read_to_string(&src_path).expect("read mk-synthetic-stack.rs");
    let mut normalized = String::with_capacity(raw.len());
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("//!") {
            normalized.push_str("//");
            normalized.push_str(rest);
        } else {
            normalized.push_str(line);
        }
        normalized.push('\n');
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(out_dir.join("mk_synthetic_stack_included.rs"), normalized)
        .expect("write mk_synthetic_stack_included.rs");

    #[cfg(feature = "gui")]
    {
        let icons_dir = manifest_dir.join("icons");
        fs::create_dir_all(&icons_dir).expect("create src-tauri/icons");
        let icon_path = icons_dir.join("icon.png");
        if !icon_path.exists() {
            // Valid 1x1 RGBA PNG (68 bytes, color #0ea5e9) generated at build time so no binary asset is committed.
            const ONE_BY_ONE_RGBA_PNG: &[u8] = &[
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
                0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
                0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78,
                0x9C, 0x63, 0x64, 0xD8, 0xDF, 0xFF, 0x0F, 0x00, 0x04, 0x7E, 0x02, 0x37, 0x7D, 0x76,
                0x2B, 0x35, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
            ];
            fs::write(&icon_path, ONE_BY_ONE_RGBA_PNG).expect("write generated icon.png");
        }
        tauri_build::build();
    }
}
