//! Phase 7e + Phase 13a conformance check: count orphan traits and
//! cross-check Phase-0 classification predictions against the actual
//! impl surface.
//!
//! Algorithm (per §Phase 13a of docs/orphan-closure/completion-plan.md):
//!
//! 1. **Trait enumeration.** Parse every
//!    `pub trait {Name}<H: HostTypes>` declaration across
//!    `foundation/src/**/*.rs`. Skip `*Resolver` traits (they're
//!    host-implemented, not foundation-internal).
//! 2. **Impl search.** For each trait, search the workspace for impl
//!    sites via the regex
//!    `^\s*impl(<[^>]*>)?\s+(crate::)?([\w_]+::)*{Name}(<[^>]*>)?\s+for\s+`,
//!    excluding `#[cfg(test)]` blocks.
//! 3. **Categorize matches.** Each impl is bucketed by target prefix:
//!    `null_stub` (`Null{Name}`), `resolved_wrapper` (`Resolved{Name}`),
//!    `validated_blanket` (`Validated<...>`), `verified_mint`
//!    (`Mint{Name}` / `*Witness` / `*Certificate`), or `hand_written`.
//! 4. **Report.** A trait is **closed** iff ≥ 1 impl matches; the pass
//!    threshold is `≤ 0` orphans. Per-category closure counts are
//!    surfaced as part of the validator's report message.
//!
//! Phase 13a also adds a **classifier cross-check**: every Path-2
//! class must have a matching `verified_mint` impl, and every
//! Path-4 class must close via the Phase-7d `null_stub`. Mismatch =
//! hard fail with a diagnostic listing the divergent class.

use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/orphan_counts";

/// Permitted orphan count: matches the Path-4 theory-deferred class
/// count (see `spec/src/counts.rs::CLASSIFICATION_PATH4`). Each Path-4
/// class has a Phase-7d `#[doc(hidden)]` Null stub, but the stub IS an
/// impl — so Path-4 traits are NOT orphans. The ratchet is therefore
/// the full expected-closed count — Phase 7e asserts **zero** orphan
/// traits after the cascade unblockers.
const MAX_PERMITTED_ORPHANS: usize = 0;

/// Phase 13a impl categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ImplCategory {
    /// `impl Foo<H> for NullFoo<H>` — Phase 7 resolver-absent stub.
    NullStub,
    /// `impl Foo<H> for ResolvedFoo<...>` — Phase 8 content-addressed.
    ResolvedWrapper,
    /// `impl Foo<H> for Validated<...>` — Phase 11 primitive-backed.
    ValidatedBlanket,
    /// `impl Foo<H> for *Witness` / `Mint*` / `*Certificate` —
    /// Phase 10 / amendment.
    VerifiedMint,
    /// Anything else — hand-written impls (foundation amendments etc.).
    HandWritten,
}

impl ImplCategory {
    fn label(self) -> &'static str {
        match self {
            ImplCategory::NullStub => "null_stub",
            ImplCategory::ResolvedWrapper => "resolved_wrapper",
            ImplCategory::ValidatedBlanket => "validated_blanket",
            ImplCategory::VerifiedMint => "verified_mint",
            ImplCategory::HandWritten => "hand_written",
        }
    }
}

/// Runs the Phase 13a orphan-count + classifier-cross-check validator.
///
/// # Errors
///
/// Returns an error if a workspace file cannot be read or the impl
/// regex fails to compile.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    // 1. Trait enumeration.
    let foundation_src = workspace.join("foundation/src");
    let mut trait_names: Vec<String> = Vec::new();
    collect_traits(&foundation_src, &mut trait_names)?;
    trait_names.retain(|n| !n.ends_with("Resolver"));
    trait_names.sort();
    trait_names.dedup();

    if trait_names.is_empty() {
        report.push(TestResult::fail(
            VALIDATOR,
            "No `pub trait ... <H: HostTypes>` declarations found — \
             foundation regeneration regressed"
                .to_string(),
        ));
        return Ok(report);
    }

    // 2. Collect every candidate `.rs` file under the workspace.
    let search_roots = [
        workspace.join("foundation/src"),
        workspace.join("uor-foundation-sdk/src"),
        workspace.join("conformance/src"),
        workspace.join("uor-foundation-test-helpers/src"),
        workspace.join("uor-foundation-verify/src"),
        workspace.join("clients/src"),
        workspace.join("cargo-uor/src"),
    ];
    let mut sources: Vec<String> = Vec::new();
    for root in &search_roots {
        collect_source_text(root, &mut sources)?;
    }
    let cleaned: Vec<String> = sources.iter().map(|s| strip_cfg_test_blocks(s)).collect();

    // 3. Per-trait impl search + categorization.
    let mut orphans: Vec<String> = Vec::new();
    let mut category_counts: std::collections::BTreeMap<ImplCategory, usize> =
        std::collections::BTreeMap::new();
    let mut traits_with_category: std::collections::BTreeMap<String, Vec<ImplCategory>> =
        std::collections::BTreeMap::new();

    for name in &trait_names {
        // Multi-line-aware impl regex. The plain `<[^>]*>` form can't
        // cope with nested generics like
        // `impl<'r, R: SourceResolver<H>, H: HostTypes> Source<H>` —
        // the `<H>` inside `R: SourceResolver<H>` makes the outer
        // bracket pair non-trivial. `[^{}]*?` skips arbitrary content
        // (excluding the impl block's opening brace) until the trait
        // name is reached. `\b` word boundaries prevent `Source` from
        // matching the prefix of `SourceResolver`.
        let pattern = format!(
            r"(?ms)^\s*impl\b[^{{}}]*?\b{name}\b(?:<[^>]*>)?\s+for\s+(?P<target>(?:crate::|super::)?(?:\w+::)*\w+)",
        );
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(e) => {
                report.push(TestResult::fail(
                    VALIDATOR,
                    format!("regex compile failed for {name}: {e}"),
                ));
                return Ok(report);
            }
        };

        let mut categories_for_trait: Vec<ImplCategory> = Vec::new();
        for src in &cleaned {
            for cap in re.captures_iter(src) {
                let target = cap.name("target").map(|m| m.as_str()).unwrap_or("");
                let cat = classify_target(target, name);
                *category_counts.entry(cat).or_insert(0) += 1;
                if !categories_for_trait.contains(&cat) {
                    categories_for_trait.push(cat);
                }
            }
        }

        if categories_for_trait.is_empty() {
            orphans.push(name.clone());
        } else {
            traits_with_category.insert(name.clone(), categories_for_trait);
        }
    }

    // 4. Classifier cross-check (Phase 13a).
    //
    // Skip rules:
    //   - `*Resolver` classes are host-implemented; foundation does not
    //     emit Null/Resolved wrappers for them (Phase 8 design).
    //   - `Skip` and `AlreadyImplemented` classifications carry no
    //     ontology-trait orphan.
    let mut cross_check_failures: Vec<String> = Vec::new();
    let ontology = uor_ontology::Ontology::full();
    for entry in uor_codegen::classification::classify_all(ontology) {
        use uor_codegen::classification::PathKind;
        let trait_name = entry.class_local;
        if trait_name.ends_with("Resolver") {
            continue;
        }
        let cats = traits_with_category.get(trait_name);
        match &entry.path_kind {
            PathKind::Path2TheoremWitness { .. } => {
                // Path-2 closure is via the Phase-7 Null stub. The
                // Phase-10 Mint{Foo} witness scaffolds are parallel
                // infrastructure (separate concrete types implementing
                // OntologyVerifiedMint, not the ontology trait).
                let has_null = cats
                    .map(|cs| cs.contains(&ImplCategory::NullStub))
                    .unwrap_or(false);
                if !has_null {
                    cross_check_failures.push(format!(
                        "Path-2 class `{}` must close via Null{} stub",
                        entry.class_iri, trait_name,
                    ));
                }
            }
            PathKind::Path4TheoryDeferred => {
                // Path-4 must close via null_stub (Phase 7d).
                let has_null_stub = cats
                    .map(|cs| cs.contains(&ImplCategory::NullStub))
                    .unwrap_or(false);
                if !has_null_stub {
                    cross_check_failures.push(format!(
                        "Path-4 theory-deferred class `{}` must close via Null{} stub",
                        entry.class_iri, trait_name,
                    ));
                }
            }
            PathKind::Path1HandleResolver => {
                // Path-1 should have null_stub + resolved_wrapper.
                let has_null = cats
                    .map(|cs| cs.contains(&ImplCategory::NullStub))
                    .unwrap_or(false);
                let has_resolved = cats
                    .map(|cs| cs.contains(&ImplCategory::ResolvedWrapper))
                    .unwrap_or(false);
                if !has_null || !has_resolved {
                    let mut missing: Vec<&str> = Vec::new();
                    if !has_null {
                        missing.push("null_stub");
                    }
                    if !has_resolved {
                        missing.push("resolved_wrapper");
                    }
                    cross_check_failures.push(format!(
                        "Path-1 class `{}` is missing {} impl(s)",
                        entry.class_iri,
                        missing.join(" + "),
                    ));
                }
            }
            PathKind::Path3PrimitiveBacked { .. } => {
                // Path-3 should have null_stub + resolved_wrapper +
                // validated_blanket. Phase 7+8 emit the first two;
                // Phase 11 hand-writes the third in blanket_impls.rs.
                let has_null = cats
                    .map(|cs| cs.contains(&ImplCategory::NullStub))
                    .unwrap_or(false);
                let has_resolved = cats
                    .map(|cs| cs.contains(&ImplCategory::ResolvedWrapper))
                    .unwrap_or(false);
                let has_validated = cats
                    .map(|cs| cs.contains(&ImplCategory::ValidatedBlanket))
                    .unwrap_or(false);
                let mut missing: Vec<&str> = Vec::new();
                if !has_null {
                    missing.push("null_stub");
                }
                if !has_resolved {
                    missing.push("resolved_wrapper");
                }
                if !has_validated {
                    missing.push("validated_blanket");
                }
                if !missing.is_empty() {
                    cross_check_failures.push(format!(
                        "Path-3 class `{}` is missing {} impl(s)",
                        entry.class_iri,
                        missing.join(" + "),
                    ));
                }
            }
            PathKind::Skip | PathKind::AlreadyImplemented => {
                // Skip — not part of the trait surface.
            }
        }
    }

    let total_traits = trait_names.len();
    let closed = total_traits - orphans.len();

    let mut summary_pieces: Vec<String> = Vec::new();
    summary_pieces.push(format!(
        "{closed} / {total_traits} traits closed (≤ {MAX_PERMITTED_ORPHANS} permitted)"
    ));
    let cat_summary: Vec<String> = category_counts
        .iter()
        .map(|(cat, count)| format!("{}={}", cat.label(), count))
        .collect();
    if !cat_summary.is_empty() {
        summary_pieces.push(format!("categories: {}", cat_summary.join(", ")));
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    let orphan_budget_ok = orphans.len() <= MAX_PERMITTED_ORPHANS;
    let cross_check_ok = cross_check_failures.is_empty();

    if orphan_budget_ok && cross_check_ok {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "Orphan count: {}; classifier cross-check OK across {} classifications",
                summary_pieces.join("; "),
                uor_codegen::classification::classify_all(ontology).len(),
            ),
        ));
    } else {
        let mut summary = String::new();
        if !orphan_budget_ok {
            let preview: Vec<&str> = orphans.iter().take(20).map(String::as_str).collect();
            summary.push_str(&format!(
                "{} orphan trait(s) (max permitted {MAX_PERMITTED_ORPHANS}). First {}: {:?}",
                orphans.len(),
                preview.len(),
                preview,
            ));
        }
        if !cross_check_ok {
            if !summary.is_empty() {
                summary.push_str("\n       ");
            }
            summary.push_str(&format!(
                "classifier cross-check: {} divergence(s):",
                cross_check_failures.len()
            ));
            for f in cross_check_failures.iter().take(10) {
                summary.push_str("\n         - ");
                summary.push_str(f);
            }
            if cross_check_failures.len() > 10 {
                summary.push_str(&format!(
                    "\n         - ... ({} more)",
                    cross_check_failures.len() - 10
                ));
            }
        }
        report.push(TestResult::fail(VALIDATOR, summary));
    }

    Ok(report)
}

/// Phase 13a — bucket an impl-target identifier into one of five categories.
///
/// `target` is the bare identifier (possibly path-prefixed but without
/// type-args). Examples: `NullBoundaryEffect`, `ResolvedSource`,
/// `Validated`, `MintLiftObstruction`, `PartitionProductWitness`.
fn classify_target(target: &str, trait_name: &str) -> ImplCategory {
    let stripped = target
        .trim_start_matches("crate::")
        .trim_start_matches("super::");

    let null_prefix = format!("Null{trait_name}");
    let resolved_prefix = format!("Resolved{trait_name}");
    let mint_prefix = format!("Mint{trait_name}");

    if stripped.starts_with(&null_prefix) {
        return ImplCategory::NullStub;
    }
    if stripped.starts_with(&resolved_prefix) {
        return ImplCategory::ResolvedWrapper;
    }
    if stripped.starts_with(&mint_prefix) {
        return ImplCategory::VerifiedMint;
    }
    // Phase 16: per-class observable views land on
    // `Validated{Foo}View<T, Phase>` newtypes (not on `Validated<T, Phase>`).
    if stripped == "Validated" || (stripped.starts_with("Validated") && stripped.ends_with("View"))
    {
        return ImplCategory::ValidatedBlanket;
    }
    if stripped.ends_with("Witness") || stripped.ends_with("Certificate") {
        return ImplCategory::VerifiedMint;
    }
    // Match Mint-prefixed cross-namespace types (e.g.,
    // MintMorphismGroundingWitness for trait GroundingWitness).
    if stripped.starts_with("Mint") && stripped.ends_with(trait_name) {
        return ImplCategory::VerifiedMint;
    }
    ImplCategory::HandWritten
}

fn collect_traits(root: &Path, out: &mut Vec<String>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let re = Regex::new(r"(?m)^pub trait (\w+)<H: HostTypes>")?;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                let src = std::fs::read_to_string(&path)?;
                for cap in re.captures_iter(&src) {
                    if let Some(m) = cap.get(1) {
                        out.push(m.as_str().to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

fn collect_source_text(root: &Path, out: &mut Vec<String>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                out.push(std::fs::read_to_string(&path)?);
            }
        }
    }
    Ok(())
}

/// Removes `#[cfg(test)] mod { ... }` blocks (brace-counted) from a
/// source string so impl sites inside test modules don't count as
/// closures of the main trait.
fn strip_cfg_test_blocks(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    loop {
        match rest.find("#[cfg(test)]") {
            None => {
                out.push_str(rest);
                break;
            }
            Some(pos) => {
                out.push_str(&rest[..pos]);
                let tail = &rest[pos + "#[cfg(test)]".len()..];
                match tail.find('{') {
                    None => {
                        out.push_str(&rest[pos..]);
                        break;
                    }
                    Some(brace_off) => {
                        let mut depth: i32 = 0;
                        let mut closed_at: Option<usize> = None;
                        for (byte_idx, ch) in tail[brace_off..].char_indices() {
                            match ch {
                                '{' => depth += 1,
                                '}' => {
                                    depth -= 1;
                                    if depth == 0 {
                                        closed_at = Some(brace_off + byte_idx + ch.len_utf8());
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        match closed_at {
                            Some(end) => {
                                rest = &tail[end..];
                            }
                            None => break,
                        }
                    }
                }
            }
        }
    }
    out
}
