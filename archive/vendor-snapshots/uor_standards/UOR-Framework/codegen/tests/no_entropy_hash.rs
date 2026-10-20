//! Phase 1b drift gate — R14 of the orphan-closure plan.
//!
//! Parses `foundation/src/enforcement.rs` as text and asserts: no struct whose
//! name ends in `Witness` or `Certificate` derives `Hash` AND carries any R7
//! entropy field inline. Witnesses and certificates are content-addressed;
//! their identity must key on `content_fingerprint`, not on entropy.
//!
//! Current matches: zero. No allow-list. Drift fails this test.

use std::fs;
use std::path::PathBuf;

const ENTROPY_FIELD_NAMES: &[&str] = &[
    "bits",
    "bits_dissipated",
    "landauer_cost",
    "landauer_nats",
    "entropy",
    "cross_entropy",
    "free_energy",
];

/// A parsed `#[derive(...)]` + `pub struct` pair.
struct Decl {
    struct_name: String,
    derives_hash: bool,
    fields: Vec<String>,
}

fn find_workspace_root() -> PathBuf {
    // Cargo runs tests with CWD = the crate dir (codegen/). Walk up.
    let mut dir = std::env::current_dir().expect("cwd");
    loop {
        if dir.join("foundation/src/enforcement.rs").exists() {
            return dir;
        }
        dir = match dir.parent() {
            Some(p) => p.to_path_buf(),
            None => panic!("no workspace root"),
        };
    }
}

fn parse_structs(source: &str) -> Vec<Decl> {
    let mut out = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim_start().starts_with("#[derive(") {
            let derives_hash = line.contains("Hash");
            let mut j = i + 1;
            // Walk forward past any additional attributes until `pub struct` or a blank.
            while j < lines.len()
                && !lines[j].trim_start().starts_with("pub struct ")
                && !lines[j].trim_start().starts_with("struct ")
            {
                if lines[j].trim_start().starts_with("#[") {
                    j += 1;
                    continue;
                }
                break;
            }
            if j >= lines.len() {
                i += 1;
                continue;
            }
            let struct_line = lines[j].trim_start();
            if !struct_line.starts_with("pub struct ") && !struct_line.starts_with("struct ") {
                i += 1;
                continue;
            }
            let tail = struct_line
                .strip_prefix("pub struct ")
                .or_else(|| struct_line.strip_prefix("struct "))
                .unwrap_or("");
            let struct_name: String = tail
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            // Collect fields between this line and the closing brace, at top-level of the struct.
            let mut fields = Vec::new();
            let mut k = j + 1;
            let mut depth: i32 =
                struct_line.matches('{').count() as i32 - struct_line.matches('}').count() as i32;
            while k < lines.len() && depth > 0 {
                let l = lines[k];
                let trimmed = l.trim_start();
                // Collect field names at depth == 1.
                if depth == 1 {
                    if let Some(colon) = trimmed.find(':') {
                        let name_part = &trimmed[..colon];
                        let name = name_part.trim();
                        if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            fields.push(name.to_string());
                        }
                    }
                }
                depth += l.matches('{').count() as i32;
                depth -= l.matches('}').count() as i32;
                k += 1;
            }
            out.push(Decl {
                struct_name,
                derives_hash,
                fields,
            });
            i = k;
            continue;
        }
        i += 1;
    }
    out
}

#[test]
fn no_content_addressed_type_hashes_entropy() {
    let root = find_workspace_root();
    let path = root.join("foundation/src/enforcement.rs");
    let source = fs::read_to_string(&path).expect("read enforcement.rs");
    let decls = parse_structs(&source);

    let mut violations: Vec<String> = Vec::new();
    for d in &decls {
        // Only flag content-addressed types (witnesses and certificates).
        let is_content_addressed =
            d.struct_name.ends_with("Witness") || d.struct_name.ends_with("Certificate");
        if !(is_content_addressed && d.derives_hash) {
            continue;
        }
        for f in &d.fields {
            if ENTROPY_FIELD_NAMES.contains(&f.as_str()) {
                violations.push(format!(
                    "struct `{}` derives Hash and carries entropy field `{}`",
                    d.struct_name, f
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Phase 1b drift gate (R14) — {} violation(s):\n  {}\n\n\
         Fix: either drop `Hash` from the derive, or move the entropy field to a \
         sibling `{{Name}}Evidence` struct that does not participate in hashing/equality.",
        violations.len(),
        violations.join("\n  ")
    );
}

#[test]
fn parser_finds_sensible_number_of_structs() {
    // Sanity: the parser should find dozens of struct declarations; if it
    // finds 0 or 5, the parser is broken and the main test is a false positive.
    let root = find_workspace_root();
    let path = root.join("foundation/src/enforcement.rs");
    let source = fs::read_to_string(&path).expect("read enforcement.rs");
    let decls = parse_structs(&source);
    assert!(
        decls.len() > 30,
        "parser only found {} struct declarations — likely broken",
        decls.len()
    );
}
