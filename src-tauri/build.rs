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
}
