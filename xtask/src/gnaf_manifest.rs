//! GNAF `MANIFEST.json` generator (SPEC sections 4 and 5), ported from the
//! vendored `proofs/wasm-gemm-gnaf/Tools/manifest.py` (#653 phase-2
//! follow-up, fifth `Tools/*.py` port after `gnaf_firewall.rs`,
//! `gnaf_root.rs`, `gnaf_scan.rs`, and `gnaf_release_path.rs`).
//!
//! SPEC 4 fixes three ordered identity stages plus one external
//! attestation, and requires them ACYCLIC: no manifest contains its own
//! identity, and no two stages hash each other.
//!
//! 1. `SourceManifestCore` -- immutable authority + handwritten Lean +
//!    fixtures + tool inputs. Excludes every manifest and every
//!    generated output.
//! 2. `GeneratedProofInputBody` -- binds the source-core identity plus
//!    every generated Lean source on the final theorem path, and
//!    `PreFinalEnvironmentBody` -- binds that identity plus
//!    toolchain/dependency identities and the compiled environment
//!    digest.
//! 3. `OutputManifestBody` -- binds the three identities above plus
//!    artifact, seal, proof registry, generated docs, and the frozen
//!    reproducibility plan. Excludes `MANIFEST.json` itself.
//!
//! `MANIFEST.json` is the canonical encoding of stage 3; its own external
//! digest is reported by CI and never included in its own preimage.
//! Each stage's *identity* is `sha256` of its body's canonical JSON
//! encoding (sorted keys, no whitespace, ASCII-escaped -- Python's
//! `json.dumps(body, sort_keys=True, separators=(",", ":"))` with its
//! default `ensure_ascii=True`), and those identities are themselves
//! embedded as fields in later stages' bodies. That makes exact,
//! byte-for-byte JSON canonicalization load-bearing: this port does not
//! use `serde_json`'s own serializer (which, matching this workspace's
//! resolved feature set, only preserves *ordering* -- not `ensure_ascii`
//! escaping) and instead hand-rolls a minimal canonical-JSON writer
//! (below) so a hypothetical non-ASCII byte anywhere in a hashed body
//! still produces the identical digest a Python run would, not merely
//! the identical digest for today's all-ASCII content.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::Fail;

/// A minimal JSON value, just expressive enough for this manifest's shape
/// (no floats or booleans appear in it). `Obj`'s pairs are sorted by key
/// at serialization time, not at construction time, so callers can build
/// them in whatever order reads best.
enum Json {
    Null,
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(&'static str, Json)>),
}

impl Json {
    fn str(s: impl Into<String>) -> Json {
        Json::Str(s.into())
    }
}

/// Canonical encoding matching Python's
/// `json.dumps(v, sort_keys=True, separators=(",", ":"))`: object keys
/// sorted by codepoint, no whitespace, and (see the module doc) ASCII-only
/// output via `\uXXXX` escapes -- `ensure_ascii=True`, Python's default.
fn canonical(v: &Json) -> String {
    let mut out = String::new();
    write_canonical(v, &mut out);
    out
}

fn write_canonical(v: &Json, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Int(n) => out.push_str(&n.to_string()),
        Json::Str(s) => write_json_string(s, out),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Json::Obj(pairs) => {
            let mut sorted: Vec<&(&str, Json)> = pairs.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            out.push('{');
            for (i, (k, val)) in sorted.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_json_string(k, out);
                out.push(':');
                write_canonical(val, out);
            }
            out.push('}');
        }
    }
}

/// A JSON string literal, escaped the way CPython's
/// `json.encoder.py_encode_basestring_ascii` escapes: `"` and `\`
/// backslash-escaped, the named short escapes for the common control
/// characters, every other byte below `0x20` and every codepoint at or
/// above `0x7F` as `\uXXXX` (surrogate-paired above the BMP), and the
/// printable ASCII range `0x20..=0x7E` (other than `"`/`\`) passed through
/// unchanged.
fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c if (c as u32) > 0x7E => {
                let cp = c as u32;
                if cp > 0xFFFF {
                    let cp2 = cp - 0x10000;
                    let hi = 0xD800 + (cp2 >> 10);
                    let lo = 0xDC00 + (cp2 & 0x3FF);
                    out.push_str(&format!("\\u{hi:04x}\\u{lo:04x}"));
                } else {
                    out.push_str(&format!("\\u{cp:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `sha256` of a first-order body: the same "identity" `Tools/manifest.py`
/// computes.
fn ident(body: &Json) -> String {
    sha256_hex_bytes(canonical(body).as_bytes())
}

fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_file(path: &Path) -> Result<String, Fail> {
    Ok(sha256_hex_bytes(&std::fs::read(path)?))
}

/// The file written to disk, matching Python's
/// `json.dumps(manifest, indent=2)` (note: no `sort_keys` here -- unlike
/// `ident()`'s canonical encoding, the on-disk file preserves each
/// object's construction (insertion) order, 2-space indented, with a
/// trailing newline). Empty arrays/objects render inline (`[]`/`{}`),
/// matching Python's own behavior for empty containers under `indent`.
fn pretty(v: &Json) -> String {
    let mut out = String::new();
    write_pretty(v, 0, &mut out);
    out
}

fn write_pretty(v: &Json, level: usize, out: &mut String) {
    match v {
        Json::Null => out.push_str("null"),
        Json::Int(n) => out.push_str(&n.to_string()),
        Json::Str(s) => write_json_string(s, out),
        Json::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                indent(level + 1, out);
                write_pretty(item, level + 1, out);
            }
            out.push('\n');
            indent(level, out);
            out.push(']');
        }
        Json::Obj(pairs) => {
            if pairs.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, val)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                indent(level + 1, out);
                write_json_string(k, out);
                out.push_str(": ");
                write_pretty(val, level + 1, out);
            }
            out.push('\n');
            indent(level, out);
            out.push('}');
        }
    }
}

fn indent(level: usize, out: &mut String) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

/// Files excluded regardless of the predicate, matching `Tools/manifest.py`'s
/// `SKIP_DIRS` (redundant with the `.git`/`.lake` directory pruning below
/// for those two, but kept for the same reason the Python does: defence in
/// depth, plus the one literal file exclusion neither prefix covers).
const SKIP_PREFIXES: [&str; 3] = [".git/", ".lake/", "vendor/wasm-spec/README.md"];

/// Every file under `gnaf_dir` (relative-path-sorted) for which `pred`
/// holds, walking exactly like `Tools/manifest.py`'s `collect()`: `.git`
/// and `.lake` pruned from descent, `SKIP_PREFIXES` excluded, final result
/// sorted lexicographically by path regardless of directory traversal
/// order.
fn collect(gnaf_dir: &Path, pred: impl Fn(&str) -> bool) -> Result<Vec<String>, Fail> {
    let mut out = Vec::new();
    walk_collect(gnaf_dir, gnaf_dir, &pred, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_collect(
    dir: &Path,
    base: &Path,
    pred: &impl Fn(&str) -> bool,
    out: &mut Vec<String>,
) -> Result<(), Fail> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name == ".git" || name == ".lake" {
                continue;
            }
            walk_collect(&path, base, pred, out)?;
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .display()
            .to_string();
        if SKIP_PREFIXES.iter().any(|s| rel.starts_with(s)) {
            continue;
        }
        if pred(&rel) {
            out.push(rel);
        }
    }
    Ok(())
}

/// Basenames of GENERATED files (SPEC 4): excluded from the source core no
/// matter what directory they sit in.
const GENERATED_BASENAMES: [&str; 3] = ["CONFORMANCE.md", "MANIFEST.json", "WasmGemmGnaf.lean"];
/// Output directories: excluded from the source core.
const OUTPUT_DIR_PREFIXES: [&str; 1] = ["artifacts/"];
/// Source-tree prefixes that constitute `SourceManifestCore`.
const SOURCE_PREFIXES: [&str; 7] = [
    "WasmGemmGnaf/",
    "authority/",
    "model/",
    "Tools/",
    "fixtures/",
    "Tests/",
    ".github/",
];
/// Individually-named root files that constitute `SourceManifestCore`.
const SOURCE_EXACT: [&str; 12] = [
    "SPEC.md",
    "README.md",
    "AGENTS.md",
    "VERIFICATION.md",
    "CERTIFICATION.md",
    "Justfile",
    "lakefile.lean",
    "lean-toolchain",
    "lake-manifest.json",
    ".gitignore",
    "LICENSE-APACHE",
    "LICENSE-MIT",
];

fn is_source(p: &str) -> bool {
    let basename = p.rsplit('/').next().unwrap_or(p);
    if GENERATED_BASENAMES.contains(&basename) {
        return false;
    }
    if OUTPUT_DIR_PREFIXES.iter().any(|d| p.starts_with(d)) {
        return false;
    }
    SOURCE_PREFIXES.iter().any(|pre| p.starts_with(pre)) || SOURCE_EXACT.contains(&p)
}

fn extra_file(path: &'static str, layer: &'static str, justification: &'static str) -> Json {
    Json::Obj(vec![
        ("path", Json::str(path)),
        ("layer", Json::str(layer)),
        ("justification", Json::str(justification)),
    ])
}

fn file_entry(gnaf_dir: &Path, rel: &str, include_bytes: bool) -> Result<Json, Fail> {
    let full = gnaf_dir.join(rel);
    let sha = sha256_file(&full)?;
    let mut pairs = vec![("path", Json::str(rel)), ("sha256", Json::str(sha))];
    if include_bytes {
        let bytes = std::fs::metadata(&full)?.len();
        pairs.push(("bytes", Json::Int(bytes as i64)));
    }
    Ok(Json::Obj(pairs))
}

/// Build the manifest's three stages plus their identities. Pure
/// computation over the tree at `gnaf_dir` -- no writes.
fn build(gnaf_dir: &Path) -> Result<(Json, String, String, String, usize), Fail> {
    let source_files = collect(gnaf_dir, is_source)?;
    let source_file_count = source_files.len();
    let source_core_files = source_files
        .iter()
        .map(|p| file_entry(gnaf_dir, p, true))
        .collect::<Result<Vec<_>, Fail>>()?;
    let source_core = Json::Obj(vec![
        ("schemaVersion", Json::Int(1)),
        ("stage", Json::str("SourceManifestCore")),
        (
            "excludes",
            Json::Arr(vec![
                Json::str("every manifest"),
                Json::str("every generated output"),
                Json::str("artifacts/"),
            ]),
        ),
        ("files", Json::Arr(source_core_files)),
    ]);
    let source_core_id = ident(&source_core);

    let generated_lean_sources = ["WasmGemmGnaf.lean"]
        .into_iter()
        .filter(|p| gnaf_dir.join(p).exists())
        .map(|p| file_entry(gnaf_dir, p, true))
        .collect::<Result<Vec<_>, Fail>>()?;
    let generated_proof_input = Json::Obj(vec![
        ("schemaVersion", Json::Int(1)),
        ("stage", Json::str("GeneratedProofInputBody")),
        (
            "sourceManifestCoreIdentity",
            Json::str(source_core_id.clone()),
        ),
        (
            "excludes",
            Json::Arr(vec![
                Json::str("its own JSON encoding"),
                Json::str("every later output"),
            ]),
        ),
        ("generatedLeanSources", Json::Arr(generated_lean_sources)),
        (
            "note",
            Json::str(
                "Artifact/Bytes.lean is NOT present: no artifact has been emitted, \
                 because emission requires WS-001 and BI-002. SPEC 13 Phase F step 4 \
                 is therefore not reached.",
            ),
        ),
    ]);
    let generated_proof_input_id = ident(&generated_proof_input);

    let toolchain = std::fs::read_to_string(gnaf_dir.join("lean-toolchain"))?
        .trim()
        .to_string();
    let pre_final_environment = Json::Obj(vec![
        ("schemaVersion", Json::Int(1)),
        ("stage", Json::str("PreFinalEnvironmentBody")),
        (
            "generatedProofInputIdentity",
            Json::str(generated_proof_input_id.clone()),
        ),
        ("leanToolchain", Json::str(toolchain)),
        (
            "leanCommit",
            Json::str("d024af099ca4bf2c86f649261ebf59565dc8c622"),
        ),
        ("dependencies", Json::Arr(vec![])),
        ("compiledEnvironmentDigest", Json::Null),
        (
            "note",
            Json::str(
                "compiledEnvironmentDigest is null: SPEC 13 Phase F step 5 records the \
                 checked final declaration-environment digest, which is only meaningful \
                 once the final theorem is on the path. GO-001 is outstanding.",
            ),
        ),
    ]);
    let pre_final_environment_id = ident(&pre_final_environment);

    let artifacts = collect(gnaf_dir, |p| {
        p.starts_with("artifacts/") && !p.ends_with("README.md")
    })?;
    let artifact_entries = artifacts
        .iter()
        .map(|p| file_entry(gnaf_dir, p, false))
        .collect::<Result<Vec<_>, Fail>>()?;
    let generated_documentation = if gnaf_dir.join("CONFORMANCE.md").exists() {
        vec![file_entry(gnaf_dir, "CONFORMANCE.md", false)?]
    } else {
        vec![]
    };

    let output_manifest = Json::Obj(vec![
        ("schemaVersion", Json::Int(1)),
        ("stage", Json::str("OutputManifestBody")),
        (
            "sourceManifestCoreIdentity",
            Json::str(source_core_id.clone()),
        ),
        (
            "generatedProofInputIdentity",
            Json::str(generated_proof_input_id.clone()),
        ),
        (
            "preFinalEnvironmentIdentity",
            Json::str(pre_final_environment_id.clone()),
        ),
        (
            "excludes",
            Json::Arr(vec![Json::str("MANIFEST.json itself")]),
        ),
        ("artifact", Json::Arr(artifact_entries)),
        ("atlasSeal", Json::Null),
        (
            "proofRegistry",
            file_entry(gnaf_dir, "model/claims.json", false)?,
        ),
        ("generatedDocumentation", Json::Arr(generated_documentation)),
        (
            "reproducibilityPlan",
            file_entry(gnaf_dir, "model/reproducibility-plan.json", false)?,
        ),
        (
            "releaseStatus",
            Json::Obj(vec![
                ("GO-001", Json::str("outstanding")),
                ("answerClass", Json::str("WorkloadIncomplete")),
                ("authority", Json::str("UOR-GNAF v1-draft.2 section 10.9")),
            ]),
        ),
        (
            "extraFilesBeyondSpecTree",
            Json::Obj(vec![
                (
                    "rule",
                    Json::str(
                        "SPEC 5: additional files permitted only when owned by a layer \
                         and listed here.",
                    ),
                ),
                (
                    "files",
                    Json::Arr(vec![
                        extra_file(
                            "WasmGemmGnaf/Wasm/Fault.lean",
                            "Wasm",
                            "SPEC 7.1's SpecMachine exposes ONE Fault used by both decode \
                             and initial. Binary and Config own genuinely different \
                             failure sets under SPEC 7.1's ownership rule, so the unified \
                             type needs its own module. Both injections proved injective, \
                             images proved disjoint.",
                        ),
                        extra_file(
                            "WasmGemmGnaf/Foundation/SchemaRegistry.lean",
                            "Foundation",
                            "Required verbatim by SPEC 6.2 ('Foundation/SchemaRegistry.lean \
                             SHALL retain the finite registry'), which the SPEC 5 tree \
                             omits.",
                        ),
                        extra_file(
                            "WasmGemmGnaf/Atlas/CoverageScope.lean",
                            "Atlas",
                            "Hardening. Proves the seal's cover check is blind to the byte \
                             universe, so it cannot be cited as universal coverage (claim \
                             AT-001, falsifier M7).",
                        ),
                        extra_file(
                            "WasmGemmGnaf/Universal/BilinearLowerBound.lean",
                            "Universal",
                            "Proved partial lower bound (claim LB-002); establishes that \
                             the open tensor-rank problem does not gate the release \
                             theorem.",
                        ),
                    ]),
                ),
            ]),
        ),
    ]);

    let manifest = Json::Obj(vec![
        ("schemaVersion", Json::Int(1)),
        (
            "description",
            Json::str("Ordered acyclic identity stages. SPEC sections 4 and 5."),
        ),
        (
            "acyclicity",
            Json::str(
                "Each stage binds only EARLIER stage identities. No stage contains \
                 its own identity. MANIFEST.json is the canonical encoding of \
                 OutputManifestBody and its own digest is never in its own preimage.",
            ),
        ),
        ("sourceManifestCore", source_core),
        (
            "sourceManifestCoreIdentity",
            Json::str(source_core_id.clone()),
        ),
        ("generatedProofInputBody", generated_proof_input),
        (
            "generatedProofInputIdentity",
            Json::str(generated_proof_input_id.clone()),
        ),
        ("preFinalEnvironmentBody", pre_final_environment),
        (
            "preFinalEnvironmentIdentity",
            Json::str(pre_final_environment_id.clone()),
        ),
        ("outputManifestBody", output_manifest),
        ("reproducibilityAttestation", Json::Null),
    ]);

    Ok((
        manifest,
        source_core_id,
        generated_proof_input_id,
        pre_final_environment_id,
        source_file_count,
    ))
}

/// SPEC 4/5: `MANIFEST.json`'s three identity stages are current and
/// acyclic. `check = true` (the CLI default, matching `check-model`'s own
/// convention) reports staleness without writing; `check = false`
/// (`--write`) regenerates the file.
pub fn gnaf_manifest(root: &Path, check: bool) -> Result<(), Fail> {
    let gnaf_dir = root.join("proofs/wasm-gemm-gnaf");
    if !gnaf_dir.exists() {
        return Err(format!(
            "{} not found; the vendored GNAF proof tree (#742) is expected \
             at this path",
            gnaf_dir.display()
        )
        .into());
    }

    let (
        manifest,
        source_core_id,
        generated_proof_input_id,
        pre_final_environment_id,
        source_file_count,
    ) = build(&gnaf_dir)?;
    let manifest_path = gnaf_dir.join("MANIFEST.json");

    if check {
        let text = std::fs::read_to_string(&manifest_path).map_err(|e| {
            format!(
                "{}: {e} -- run `cargo xtask gnaf-manifest --write`",
                manifest_path.display()
            )
        })?;
        let current: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("{}: invalid JSON: {e}", manifest_path.display()))?;
        let checks = [
            ("sourceManifestCoreIdentity", &source_core_id),
            ("generatedProofInputIdentity", &generated_proof_input_id),
            ("preFinalEnvironmentIdentity", &pre_final_environment_id),
        ];
        for (key, expected) in checks {
            let got = current.get(key).and_then(|v| v.as_str()).unwrap_or("");
            if got != expected.as_str() {
                return Err(format!(
                    "{} STALE: {key} differs -- run `cargo xtask gnaf-manifest --write`",
                    manifest_path.display()
                )
                .into());
            }
        }
        println!(
            "gnaf-manifest: manifest current: {source_file_count} source files, \
             3 identity stages (SPEC 4/5)"
        );
    } else {
        std::fs::write(&manifest_path, format!("{}\n", pretty(&manifest)))?;
        println!("gnaf-manifest: MANIFEST.json: {source_file_count} source files");
        println!("  sourceManifestCore      {}...", &source_core_id[..16]);
        println!(
            "  generatedProofInput     {}...",
            &generated_proof_input_id[..16]
        );
        println!(
            "  preFinalEnvironment     {}...",
            &pre_final_environment_id[..16]
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_sorts_keys_and_uses_compact_separators() {
        let v = Json::Obj(vec![
            ("b", Json::Int(2)),
            ("a", Json::Str("x".to_string())),
            ("c", Json::Null),
        ]);
        assert_eq!(canonical(&v), "{\"a\":\"x\",\"b\":2,\"c\":null}");
    }

    #[test]
    fn canonical_escapes_control_chars_like_python_ensure_ascii() {
        let v = Json::Str("quote\" backslash\\ tab\tnewline\ncontrol\u{01}".to_string());
        assert_eq!(
            canonical(&v),
            "\"quote\\\" backslash\\\\ tab\\tnewline\\ncontrol\\u0001\""
        );
    }

    #[test]
    fn canonical_escapes_non_ascii_including_above_the_bmp() {
        // e-acute, U+00E9: below the BMP, single \u escape.
        assert_eq!(canonical(&Json::str("caf\u{e9}")), "\"caf\\u00e9\"");
        // U+1F600 (grinning face): above the BMP, UTF-16 surrogate pair.
        assert_eq!(canonical(&Json::str("\u{1F600}")), "\"\\ud83d\\ude00\"");
    }

    #[test]
    fn pretty_preserves_insertion_order_and_indents_two_spaces() {
        let v = Json::Obj(vec![
            ("z", Json::Int(1)),
            ("a", Json::Arr(vec![Json::Int(1), Json::Int(2)])),
            ("empty", Json::Arr(vec![])),
        ]);
        let expected = "{\n  \"z\": 1,\n  \"a\": [\n    1,\n    2\n  ],\n  \"empty\": []\n}";
        assert_eq!(pretty(&v), expected);
    }

    #[test]
    fn is_source_matches_generated_output_and_prefix_rules() {
        assert!(is_source("WasmGemmGnaf/Foundation/Types.lean"));
        assert!(is_source("authority/manifest.json"));
        assert!(is_source("SPEC.md"));
        assert!(is_source(".github/workflows/ci.yml"));
        // GENERATED basenames are excluded no matter the directory.
        assert!(!is_source("WasmGemmGnaf.lean"));
        assert!(!is_source("nested/dir/MANIFEST.json"));
        // Output directory excluded even though its basename isn't GENERATED.
        assert!(!is_source("artifacts/wasm-gemm-gnaf.wasm"));
        // Not on any recognized prefix or exact-name list.
        assert!(!is_source("scratch/Notes.md"));
    }

    fn scratch_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "xtask-gnaf-manifest-test-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    /// End-to-end regression guard against a synthetic tree, never the
    /// real vendored one: write mode produces a MANIFEST.json whose three
    /// top-level identity fields are internally consistent with the
    /// stage bodies' own recomputed identities, and check mode against
    /// exactly that output reports current.
    #[test]
    fn gnaf_manifest_write_then_check_round_trips_on_a_synthetic_tree() {
        let root = scratch_dir("roundtrip");
        let gnaf = root.join("proofs/wasm-gemm-gnaf");
        std::fs::create_dir_all(gnaf.join("WasmGemmGnaf/Foundation")).unwrap();
        std::fs::create_dir_all(gnaf.join("model")).unwrap();
        std::fs::write(
            gnaf.join("WasmGemmGnaf/Foundation/Types.lean"),
            "def x := 1\n",
        )
        .unwrap();
        std::fs::write(gnaf.join("lean-toolchain"), "leanprover/lean4:v4.9.0\n").unwrap();
        std::fs::write(gnaf.join("model/claims.json"), "{}").unwrap();
        std::fs::write(gnaf.join("model/reproducibility-plan.json"), "{}").unwrap();

        gnaf_manifest(&root, false).unwrap();
        assert!(gnaf_manifest(&root, true).is_ok());

        let written = std::fs::read_to_string(gnaf.join("MANIFEST.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(parsed["schemaVersion"], 1);
        assert!(parsed["sourceManifestCoreIdentity"].as_str().unwrap().len() == 64);
        assert!(written.ends_with('\n'));

        // A single-byte edit to a source file changes the recomputed
        // identity, so check mode against the now-stale file must reject.
        std::fs::write(
            gnaf.join("WasmGemmGnaf/Foundation/Types.lean"),
            "def x := 2\n",
        )
        .unwrap();
        assert!(gnaf_manifest(&root, true).is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
