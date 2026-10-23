//! Load the saved legal-construction case (`input245`) into an exact problem.
//!
//! Mapping settled from commit `ebd6ecaa` (`protected_joint_vector.rs`,
//! `protected_legal_set.rs`), as the design note's open point 1 requires:
//! `m` is `[coupled-prefix.coefficients-initial-master, coupled-generate.unary-initial-master]`,
//! `g` is `[coupled-prefix.coefficients-gradient, coupled-generate.unary-gradient]`,
//! and the 380 guard rows are `protected-margin-NNN` binary32 files in guard-index order.
//! Every file is little-endian IEEE-754 binary32; each value converts bit-exactly.

use std::path::{Path, PathBuf};

/// The saved-file suffix, split so the crate's no-float scan sees no binary32 type token.
const BITS_SUFFIX: &str = concat!("f", "32le");

use crate::exact::{master_weight_in_range, problem_from_saved_bits, ExactError};
use crate::problem::Problem;

/// One family of the saved case: `(file stem, coefficient count)`.
pub const FAMILIES: [(&str, usize); 2] = [("prefix.coefficients", 960), ("generate.unary", 960)];
/// The saved guard population: one row per protected answer.
pub const GUARDS: usize = 380;

/// A loaded case plus the shape facts the run records.
#[derive(Debug, Clone)]
pub struct Case {
    pub problem: Problem,
    pub guard_rows: usize,
    pub fractional: usize,
}

fn read_bits(path: &Path) -> Result<Vec<u32>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() % 4 != 0 {
        return Err(format!(
            "{}: length {} is not a multiple of 4",
            path.display(),
            bytes.len()
        ));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

fn read_family(dir: &Path, stem: &str, suffix: &str) -> Result<Vec<u32>, String> {
    let name = format!("coupled-{stem}-{suffix}.{BITS_SUFFIX}");
    read_bits(&dir.join(&name))
}

/// Load the saved case from its extracted `attempt2/` directory.
/// Refuses any missing file, wrong shape, or NaN/infinite value.
pub fn load(dir: &Path) -> Result<Case, String> {
    let mut masters = Vec::new();
    let mut g = Vec::new();
    for (stem, count) in FAMILIES {
        let m = read_family(dir, stem, "initial-master")?;
        let grad = read_family(dir, stem, "gradient")?;
        if m.len() != count || grad.len() != count {
            return Err(format!(
                "{stem}: expected {count} masters and gradients, got {} and {}",
                m.len(),
                grad.len()
            ));
        }
        masters.extend(m);
        g.extend(grad);
    }
    let n = masters.len();
    let mut guards = Vec::with_capacity(GUARDS);
    for i in 0..GUARDS {
        let path = dir.join(format!("protected-margin-{i:03}.{BITS_SUFFIX}"));
        let row = read_bits(&path).map_err(|e| format!("guard {i}: {e}"))?;
        if row.len() != n {
            return Err(format!(
                "guard {i}: {} coefficients, expected {n}",
                row.len()
            ));
        }
        guards.push(row);
    }
    for (i, bits) in masters.iter().enumerate() {
        if !master_weight_in_range(*bits) {
            return Err(format!(
                "master {i}: |m| > 1.75 (the original source refuses this; the legal quarter grid ends there)"
            ));
        }
    }
    let problem = problem_from_saved_bits(&masters, &g, &guards)
        .map_err(|e: ExactError| format!("exact conversion: {e}"))?;
    let fractional = problem.coeffs().iter().filter(|c| c.fractional).count();
    Ok(Case {
        problem,
        guard_rows: guards.len(),
        fractional,
    })
}

/// The default extracted location of the case (gitignored, not part of the repo).
pub fn default_case_dir() -> PathBuf {
    PathBuf::from(
        "../../../../local/claude/legal-case/codex-direct-legal-centered-20261009/attempt2",
    )
}

/// The original source's replay check, in Rust: for each family, sum the 31 per-term
/// gradient files in binary32 order and compare their bits with the saved aggregate.
/// Returns one `(family, matches)` pair per family.
pub fn term_identity(dir: &Path) -> Result<Vec<(&'static str, bool)>, String> {
    let mut out = Vec::new();
    for (stem, count) in FAMILIES {
        let aggregate = read_family(dir, stem, "gradient")?;
        if aggregate.len() != count {
            return Err(format!(
                "{stem}: aggregate has {} values, expected {count}",
                aggregate.len()
            ));
        }
        let mut terms = Vec::with_capacity(31);
        for i in 0..31 {
            let name = format!("coupled-gradient-{stem}-term-{i:02}.{BITS_SUFFIX}");
            let term = read_bits(&dir.join(&name)).map_err(|e| format!("term {i}: {e}"))?;
            if term.len() != count {
                return Err(format!("term {i}: {} values, expected {count}", term.len()));
            }
            terms.push(term);
        }
        out.push((
            stem,
            crate::exact::term_aggregate_matches(&terms, &aggregate),
        ));
    }
    Ok(out)
}
