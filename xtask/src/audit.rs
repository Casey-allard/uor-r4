//! The grep-shaped gates.
//!
//! These are crude on purpose. A gate that needs interpretation is a gate that
//! gets argued with; these ones read the source, find a token, and fail. Each
//! carries the rule it enforces in its failure message, because the point of a
//! red gate is to name the promise that was broken.

use std::path::{Path, PathBuf};

use crate::Fail;

/// The crates that ship: every crate under `crates/` that is not
/// `publish = false`. The rules below apply to those and not to the
/// dev-and-CI-only crates, which may use `std`, `alloc`, and floats freely.
///
/// Derived from the manifests rather than listed here. A list would be a second
/// place to remember, and the failure mode of a second place is that a crate
/// added to one is missing from the other --- silently, because a gate that
/// skips a crate cannot report on it. Reading `publish = false` asks the same
/// question `cargo publish` asks.
fn shipped(root: &Path) -> Result<Vec<String>, Fail> {
    let mut out = Vec::new();
    let dir = root.join("crates");
    if !dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&dir)? {
        let path = entry?.path();
        let manifest = path.join("Cargo.toml");
        let Ok(toml) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        if toml.lines().any(|l| l.trim() == "publish = false") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            out.push(name.to_string());
        }
    }
    out.sort();
    Ok(out)
}

struct Source {
    rel: String,
    text: String,
}

fn shipped_sources(root: &Path) -> Result<Vec<Source>, Fail> {
    let mut out = Vec::new();
    for name in shipped(root)? {
        let dir = root.join("crates").join(&name).join("src");
        if !dir.exists() {
            continue;
        }
        collect(&dir, root, &mut out)?;
    }
    Ok(out)
}

fn collect(dir: &Path, root: &Path, out: &mut Vec<Source>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, root, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = std::fs::read_to_string(&path)?;
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string();
            out.push(Source { rel, text });
        }
    }
    Ok(())
}

/// Lines of `text` outside comments and outside `#[cfg(test)]` modules.
///
/// A rule about what the code *does* must not be tripped by a doc comment
/// explaining what it does not do, nor by a test that deliberately constructs
/// the forbidden thing to prove it is caught.
fn effective_lines(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut in_test = false;
    let mut test_depth = 0i32;
    let mut depth = 0i32;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with("//") || line.starts_with("#!") {
            // A `#![deny(...)]` or a comment states policy; it never is the
            // behaviour the gate is looking for.
            continue;
        }
        let opens = raw.matches('{').count() as i32;
        let closes = raw.matches('}').count() as i32;
        if line.starts_with("#[cfg(test)]") {
            in_test = true;
            test_depth = depth;
        }
        if !in_test {
            let code = line.split("//").next().unwrap_or("");
            if !code.trim().is_empty() {
                out.push((i + 1, code));
            }
        }
        depth += opens - closes;
        if in_test && depth <= test_depth && closes > 0 {
            in_test = false;
        }
    }
    out
}

/// Net angle-bracket depth change of `s`: `<` opens a generic, `>` closes one.
/// Used to rejoin a `Result<...>` return type that rustfmt wrapped across
/// several lines, so the whole error type is visible to the sanctioned checks.
fn angle_delta(s: &str) -> i32 {
    s.matches('<').count() as i32 - s.matches('>').count() as i32
}

/// Byte index of the first `Result<` in `line` that begins the `Result` type
/// itself, rather than the tail of a longer identifier. A struct named
/// `...Result` (e.g. `ShardReductionResult<R>`) is not `std::result::Result`
/// and carries no error surface, so `Result<` preceded by an identifier
/// character is skipped --- this is the type, not a name that ends in it.
fn find_result_type(line: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(rel) = line[from..].find("Result<") {
        let pos = from + rel;
        let preceded_by_ident = line[..pos]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        if !preceded_by_ident {
            return Some(pos);
        }
        from = pos + "Result".len();
    }
    None
}

/// R5: no arbitrary limitation. Every bound is a property of the caller's
/// chosen instantiation, never of the code.
///
/// Concretely: no shipped crate may return an error the model does not
/// sanction. The absence of a class of error is checked here --- the absence of
/// negative testing is only honest if there is nothing to test negatively.
pub fn audit_limits(root: &Path) -> Result<(), Fail> {
    let sources = shipped_sources(root)?;
    // The only error type a shipped crate may name, plus the declaration check at
    // a declared boundary, which is not the operation failing.
    //
    // `SourceUnavailable` is the host-ingestion analogue: the teacher-loading
    // boundary (uor-r4 #510) reports exactly one condition — a declared external
    // source artifact (model directory, safetensors, config, or a named tensor)
    // could not be ingested into a valid teacher at construction. It is the
    // host-side counterpart of `NotAProduct`, not a runtime limitation, so it is
    // sanctioned here alongside the graph substrate's three.
    let sanctioned = [
        "NotAProduct",
        "ObservedBound",
        "KappaError",
        "SourceUnavailable",
    ];

    let mut violations = Vec::new();
    for src in &sources {
        let lines = effective_lines(&src.text);
        for (idx, &(line_no, line)) in lines.iter().enumerate() {
            let Some(pos) = find_result_type(line) else {
                continue;
            };
            // The return type may wrap across lines: rustfmt puts a long
            // associated type on its own lines and leaves `-> Result<` with the
            // error type below it (an external-trait `forward` impl does this).
            // Join continuation lines until the `Result<...>` generic closes, so
            // a sanctioned or external-contract error type on a following line is
            // seen by the checks below instead of read as a bare `Result<` with
            // no sanctioned type --- a false positive the single-line scan cannot
            // avoid.
            let mut tail = line[pos..].to_string();
            let mut depth = angle_delta(&line[pos + "Result".len()..]);
            let mut j = idx + 1;
            while depth > 0 && j < lines.len() && j < idx + 16 {
                let cont = lines[j].1;
                tail.push(' ');
                tail.push_str(cont);
                depth += angle_delta(cont);
                j += 1;
            }
            let tail = tail.as_str();
            if sanctioned.iter().any(|s| tail.contains(s)) {
                continue;
            }
            // serde's `Serialize`/`Deserialize`/`Visitor` contracts are not the
            // model's error surface. Their signatures return the caller's data
            // format error (`S::Error`, `D::Error`, `A::Error`, or the generic
            // `E` on a `visit_*` scalar callback), not a limitation this model
            // reports. These signatures are fixed by the trait and cannot name
            // a sanctioned type, so they are carved out exactly like a
            // sanctioned return. Keep the generic carve-out syntactically tied
            // to a Visitor callback and its fixed `Self::Value` result so an
            // ordinary helper named `visit_*` cannot hide an unrelated error.
            let serde_visitor_scalar = [
                "fn visit_bool<",
                "fn visit_i64<",
                "fn visit_u64<",
                "fn visit_f64<",
                "fn visit_str<",
                "fn visit_string<",
                "fn visit_none<",
                "fn visit_unit<",
            ]
            .iter()
            .any(|signature| line.trim_start().starts_with(signature))
                && tail.contains("Result<Self::Value, E>");
            let serde_visitor_access = (line.trim_start().starts_with("fn visit_seq")
                || line.trim_start().starts_with("fn visit_map"))
                && tail.contains("Result<Self::Value, A::Error>");
            if tail.contains("S::Ok")
                || tail.contains("S::Error")
                || tail.contains("D::Error")
                || serde_visitor_scalar
                || serde_visitor_access
            {
                continue;
            }
            // The uor_foundation SDK's enforcement and pipeline traits fix their
            // own error surfaces the same way serde does. An `axis!`-declared
            // `route_query` must return `Result<usize, ShapeViolation>`, and a
            // `PrismModel::forward` impl must return
            // `Result<Grounded<..>, PipelineFailure>`. Both error types are the
            // framework's shape-conformance reports at a declared host boundary
            // --- the caller's external contract, not a limitation this model
            // imposes --- and neither signature can name a sanctioned type, so
            // they are carved out exactly like the serde associated types.
            if tail.contains("ShapeViolation") || tail.contains("PipelineFailure") {
                continue;
            }
            violations.push(format!("{}:{line_no}:{}", src.rel, line.trim()));
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "R5: every bound is derived from declared parameters and is a \
             property of the caller's chosen instantiation. The only reportable \
             condition is that the requested object does not exist, reported at view \
             construction. A `Result` over anything else is a limitation the model does \
             not sanction.\n\n{}",
            violations.join("\n")
        )
        .into());
    }
    println!("audit-limits: no shipped crate returns an unsanctioned error (R5)");
    Ok(())
}

/// R4: nothing is deferred. No deferral marker, no stub, no placeholder
/// document section, no capability behind a flag that turns it off.
///
/// The markers are spelled in halves. This gate reads every crate *and*
/// `xtask`, which means it reads this file, and a list of forbidden tokens
/// written out in full is a list that matches itself --- the gate would fail on
/// its own definition, for ever. The alternative is exempting this file, and an
/// exemption is a hole: a real deferral parked in the gate would then be the one
/// place nothing looks. Split, the gate scans itself like anything else.
pub fn audit_deferral(root: &Path) -> Result<(), Fail> {
    let markers = [
        concat!("TO", "DO"),
        concat!("FIX", "ME"),
        concat!("XX", "X"),
        concat!("unimplemented", "!"),
        concat!("to", "do!"),
        concat!("for ", "now"),
        concat!("later ", "version"),
    ];

    // The carve-out (uor-r4 #510): a deferral is a violation only if it is
    // *unregistered*. A capability kept dormant behind a pre-declared activation
    // gate is registered in `model/ledger.toml` as an `open` claim --- measured
    // and reported, never asserted (R2) --- and a marker on a line that cites
    // that claim's id is the honest form of "this is deferred, and here is where
    // it is declared". A marker with no such citation is a deferral nobody
    // agreed to, which is exactly what R4 exists to catch. The set is read from
    // the ledger, not listed here, so a claim retired from the ledger re-arms
    // the gate on the lines that cited it.
    let open_ids: Vec<String> = repo_model::Model::load(&root.join("model"))
        .map(|m| {
            m.ledger
                .claim
                .iter()
                .filter(|c| c.level == repo_model::Level::Open)
                .map(|c| c.id.clone())
                .collect()
        })
        .unwrap_or_default();

    let mut violations = Vec::new();

    // Every crate, not only the shipped ones, and `xtask` with them: R4 is a
    // promise about the repository, and a deferral parked in a gate is the same
    // deferral as one parked in shipped code.
    let mut files: Vec<PathBuf> = Vec::new();
    for dir in [root.join("crates"), root.join("xtask")] {
        if dir.exists() {
            gather_all(&dir, &mut files)?;
        }
    }
    // Every Markdown file at the root, discovered rather than listed. A list
    // here would go stale the first time a document was added or renamed, and
    // the failure would be silent: a document nobody scans reports nothing.
    for entry in std::fs::read_dir(root)? {
        let p = entry?.path();
        if p.extension().is_some_and(|e| e == "md") {
            files.push(p);
        }
    }

    for path in files {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for (i, line) in text.lines().enumerate() {
            for marker in markers {
                if !line.contains(marker) {
                    continue;
                }
                // A backticked marker is a *mention*, not a use: the
                // documentation has to be able to name what the gate catches
                // without tripping it. Anything outside code spans is real.
                if !outside_code_spans(line, marker) {
                    continue;
                }
                // The carve-out: a real marker is sanctioned when its line cites
                // an `open` ledger claim, i.e. the deferral is registered.
                if open_ids.iter().any(|id| line.contains(id.as_str())) {
                    continue;
                }
                violations.push(format!("{rel}:{}: {}", i + 1, line.trim()));
            }
        }
    }

    if !violations.is_empty() {
        return Err(format!(
            "R4: nothing is deferred *and unregistered*. None of {} may appear \
             unless the line cites an `open` claim in model/ledger.toml (a \
             capability kept dormant behind a pre-declared activation gate). \
             Otherwise: no stub, no placeholder section, no capability behind a \
             flag that turns it off. Register the deferral as `open`, or remove \
             it.\n\n{}",
            markers.join(", "),
            violations.join("\n")
        )
        .into());
    }
    println!("audit-deferral: nothing is deferred (R4)");
    Ok(())
}

/// Does `marker` occur in `line` outside every backtick-delimited span?
fn outside_code_spans(line: &str, marker: &str) -> bool {
    let mut rest = line;
    let mut at = 0usize;
    while let Some(pos) = rest.find(marker) {
        let absolute = at + pos;
        // An odd number of backticks before this occurrence means it is inside
        // a span.
        if line[..absolute].matches('`').count().is_multiple_of(2) {
            return true;
        }
        at = absolute + marker.len();
        rest = &line[at..];
    }
    false
}

fn gather_all(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Fail> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            gather_all(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|e| e == "rs" || e == "md" || e == "toml")
        {
            out.push(path);
        }
    }
    Ok(())
}
