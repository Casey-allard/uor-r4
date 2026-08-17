//! GNAF `CONFORMANCE.md` generator (SPEC 17.3), ported from the vendored
//! `proofs/wasm-gemm-gnaf/Tools/gen_conformance.py` (#653 phase-3, sixth
//! `Tools/*.py` port after `gnaf_firewall.rs`/`gnaf_root.rs`/`gnaf_scan.rs`/
//! `gnaf_release_path.rs`/`gnaf_manifest.rs`; the remaining four
//! (`axioms.py`, `required.py`, `gate.py`, `mutation.py`) all invoke the
//! Lean toolchain and stay out of scope for this port -- see
//! `docs/gnaf_integration_653.md`).
//!
//! Renders `CONFORMANCE.md` deterministically from `model/claims.json`: a
//! live inventory (Lean module/line/theorem-line counts under
//! `WasmGemmGnaf/`), the claims table, each `formalProof` claim's axiom
//! closure, refuted framings, and outstanding obligations. `--check` mode
//! (this repository's own gate ladder) reports staleness without writing;
//! write mode regenerates the file.
//!
//! Byte-for-byte the same shape `Tools/gen_conformance.py` produces,
//! including its `f"{n:,}"`-style thousands-separated counts and its two
//! distinct truncation rules: the claims table escapes `|` then truncates
//! to 107 chars + `"..."` past a 110-char threshold, while outstanding
//! obligations take a bare first-100-char cut with no ellipsis. Both
//! operate on Unicode scalar values (Python's `len`/slicing on `str`),
//! matched here with `.chars()` rather than byte indexing.
//!
//! The theorem count is the same crude heuristic the Python source uses:
//! lines (after left-strip) starting with `"theorem "` -- literally
//! `t.startswith("theorem ") or t.startswith("instance ") and False`,
//! which due to Python's `and`-before-`or` precedence is exactly
//! `t.startswith("theorem ")` and nothing else; not a real theorem count
//! from Lean compilation, reproduced as-is for parity.

use std::path::Path;

use crate::Fail;

/// SPEC 17.3: `CONFORMANCE.md` is deterministically generated from
/// `model/claims.json` and current.
pub fn gnaf_conformance(root: &Path, check: bool) -> Result<(), Fail> {
    let gnaf_dir = root.join("proofs/wasm-gemm-gnaf");
    let base = gnaf_dir.join("WasmGemmGnaf");
    if !base.exists() {
        return Err(format!(
            "{} not found; the vendored GNAF proof tree (#742) is expected \
             at this path",
            base.display()
        )
        .into());
    }

    let claims_path = gnaf_dir.join("model/claims.json");
    let text = std::fs::read_to_string(&claims_path)
        .map_err(|e| format!("{}: {e}", claims_path.display()))?;
    let doc: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("{}: invalid JSON: {e}", claims_path.display()))?;

    let (nmod, nlines, nthm) = inventory(&base)?;
    let rendered = render(&doc, nmod, nlines, nthm)?;

    let conformance_path = gnaf_dir.join("CONFORMANCE.md");
    if check {
        let current = std::fs::read_to_string(&conformance_path).unwrap_or_default();
        if current != rendered {
            return Err(format!(
                "{} is STALE -- run `cargo xtask gnaf-conformance --write`",
                conformance_path.display()
            )
            .into());
        }
        println!(
            "gnaf-conformance: CONFORMANCE.md current: {nmod} modules, {nlines} lines, \
             {nthm} theorem lines (SPEC 17.3)"
        );
    } else {
        std::fs::write(&conformance_path, &rendered)?;
        println!("gnaf-conformance: CONFORMANCE.md regenerated: {nmod} modules, {nlines} lines");
    }
    Ok(())
}

/// Live inventory: `.lean` file count, total line count, and
/// `"theorem "`-prefixed line count under `base`, matching
/// `Tools/gen_conformance.py`'s `inventory()`.
fn inventory(base: &Path) -> Result<(usize, usize, usize), Fail> {
    let mut files = Vec::new();
    collect_lean(base, &mut files)?;

    let mut total_lines = 0usize;
    let mut total_thms = 0usize;
    for path in &files {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        // Python opens with errors="replace"; from_utf8_lossy is the same
        // idea (substitute, never fail) for the rare non-UTF-8 byte.
        let content = String::from_utf8_lossy(&bytes);
        for line in content.lines() {
            total_lines += 1;
            if line.trim_start().starts_with("theorem ") {
                total_thms += 1;
            }
        }
    }
    Ok((files.len(), total_lines, total_thms))
}

fn collect_lean(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_lean(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "lean") {
            out.push(path);
        }
    }
    Ok(())
}

/// Render `CONFORMANCE.md`'s full text from the parsed claims document
/// and the live inventory counts. Every literal line matches
/// `Tools/gen_conformance.py`'s `L.append(...)` sequence verbatim,
/// joined the same way: `"\n".join(L) + "\n"`.
fn render(
    doc: &serde_json::Value,
    nmod: usize,
    nlines: usize,
    nthm: usize,
) -> Result<String, Fail> {
    let claims = doc
        .get("claims")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Fail::from("claims.json: missing or invalid \"claims\" array"))?;

    let mut lines: Vec<String> = Vec::new();
    lines.push("# Conformance\n".to_string());
    lines.push(format!(
        "**Inventory:** {nmod} Lean modules, {} lines, {} proved theorems.",
        thousands(nlines),
        thousands(nthm),
    ));
    lines.push(
        "Generated live; prose documents cite this table rather than repeating counts.\n"
            .to_string(),
    );
    lines.push(
        "Generated from `model/claims.json` by `just docs`. Do not edit by hand.\n".to_string(),
    );
    lines.push(
        "Claim levels are load-bearing (SPEC 17.1). Only `formalProof` supports the".to_string(),
    );
    lines.push("words \"proved\", \"theorem\", or \"globally optimal\".\n".to_string());
    lines.push("## Claims\n".to_string());
    lines.push("| ID | Level | Status | Statement | Lean declaration |".to_string());
    lines.push("| --- | --- | --- | --- | --- |".to_string());

    for claim in claims {
        let id = required_str(claim, "id")?;
        let level = required_str(claim, "level")?;
        let status = required_str(claim, "status")?;
        let statement = required_str(claim, "statement")?;
        let decl = claim
            .get("leanDeclaration")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("—");
        let st = truncate_table_statement(statement);
        lines.push(format!("| `{id}` | {level} | {status} | {st} | `{decl}` |"));
    }

    lines.push("\n## Axiom closure\n".to_string());
    lines
        .push("Every `formalProof` claim's transitive axioms, from `#print axioms`:\n".to_string());
    for claim in claims {
        let level = required_str(claim, "level")?;
        if level != "formalProof" {
            continue;
        }
        let id = required_str(claim, "id")?;
        let axioms: Vec<&str> = claim
            .get("axioms")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let ax = if axioms.is_empty() {
            "none".to_string()
        } else {
            axioms
                .iter()
                .map(|a| format!("`{a}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        lines.push(format!("- `{id}` — {ax}"));
    }
    lines.push(
        "\nPermitted: `propext`, `Quot.sound`, `Classical.choice` (Lean core logical".to_string(),
    );
    lines.push(
        "axioms, SPEC 4). Any `sorryAx` or project-declared axiom fails the gate.\n".to_string(),
    );

    lines.push("## Refuted framings\n".to_string());
    lines.push("Recorded so they are not silently re-asserted:\n".to_string());
    let refuted = doc
        .get("refutedFramings")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for r in &refuted {
        let verdict = required_str(r, "verdict")?;
        let confidence = required_str(r, "confidence")?;
        let framing = required_str(r, "framing")?;
        let reason = required_str(r, "reason")?;
        lines.push(format!("- **{verdict}** ({confidence}) — {framing}"));
        lines.push(format!("  - {reason}"));
    }

    lines.push("\n## Outstanding obligations\n".to_string());
    let outstanding: Vec<&serde_json::Value> = claims
        .iter()
        .filter(|c| c.get("status").and_then(|v| v.as_str()) == Some("outstanding"))
        .collect();
    lines.push(format!(
        "{} outstanding. Terminal answer for `GO-001`: `WorkloadIncomplete`",
        outstanding.len()
    ));
    lines.push("(UOR-GNAF v1-draft.2 section 10.9). See `CERTIFICATION.md`.\n".to_string());
    for claim in &outstanding {
        let id = required_str(claim, "id")?;
        let statement = required_str(claim, "statement")?;
        let obligation = claim
            .get("obligation")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("—");
        let st = truncate_chars(statement, 100);
        lines.push(format!("- `{id}` ({obligation}) — {st}"));
    }

    let mut text = lines.join("\n");
    text.push('\n');
    Ok(text)
}

fn required_str<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str, Fail> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| Fail::from(format!("claims.json: missing or non-string \"{key}\"")))
}

/// Claims-table truncation: escape `|` first, then cut to 107 chars +
/// `"..."` if the escaped statement exceeds 110 chars (Unicode scalar
/// values, matching Python's `str.replace` then `len`/slice).
fn truncate_table_statement(statement: &str) -> String {
    let escaped = statement.replace('|', "\\|");
    let char_count = escaped.chars().count();
    if char_count <= 110 {
        escaped
    } else {
        let mut s: String = escaped.chars().take(107).collect();
        s.push_str("...");
        s
    }
}

/// A bare first-`n`-char cut with no ellipsis, matching Python's
/// `statement[:100]` for outstanding obligations.
fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Python's `f"{n:,}"`: digit-group with commas every three digits from
/// the right. `n` here is always non-negative (a count).
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_groups_by_three_from_the_right() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(97), "97");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(55_504), "55,504");
        assert_eq!(thousands(3_146), "3,146");
        assert_eq!(thousands(1_000_000), "1,000,000");
    }

    #[test]
    fn table_truncation_escapes_pipes_then_cuts_at_110() {
        assert_eq!(truncate_table_statement("short"), "short");
        let exactly_110 = "a".repeat(110);
        assert_eq!(truncate_table_statement(&exactly_110), exactly_110);
        let over = "a".repeat(111);
        let truncated = truncate_table_statement(&over);
        assert_eq!(truncated.chars().count(), 110);
        assert!(truncated.ends_with("..."));
        assert_eq!(truncate_table_statement("a|b"), "a\\|b");
    }

    #[test]
    fn outstanding_truncation_is_a_bare_cut_with_no_ellipsis() {
        assert_eq!(truncate_chars("short", 100), "short");
        let long = "a".repeat(150);
        let cut = truncate_chars(&long, 100);
        assert_eq!(cut.chars().count(), 100);
        assert!(!cut.contains('.'));
    }

    #[test]
    fn render_defaults_missing_lean_declaration_and_empty_axioms_and_obligation() {
        let doc = serde_json::json!({
            "claims": [
                {
                    "id": "X-1",
                    "level": "formalProof",
                    "status": "outstanding",
                    "statement": "a claim with no declaration, no axioms, no obligation"
                }
            ],
            "refutedFramings": []
        });
        let rendered = render(&doc, 1, 1, 0).unwrap();
        assert!(rendered.contains(
            "| `X-1` | formalProof | outstanding | \
            a claim with no declaration, no axioms, no obligation | `—` |"
        ));
        assert!(rendered.contains("- `X-1` — none"));
        assert!(rendered.contains("- `X-1` (—) — a claim with no declaration"));
    }

    #[test]
    fn render_rejects_a_claim_missing_a_required_field() {
        let doc = serde_json::json!({
            "claims": [{"id": "X-1", "level": "formalProof", "status": "outstanding"}],
            "refutedFramings": []
        });
        assert!(render(&doc, 0, 0, 0).is_err());
    }
}
