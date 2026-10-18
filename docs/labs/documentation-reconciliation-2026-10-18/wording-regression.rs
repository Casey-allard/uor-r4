//! Narrow regressions for the claims independently rejected on PR #2643.
//! This catches those wording reversions; it is not a scientific validator.
//! From the repository root: rustc --edition 2021 <this file> -o local/doc-wording
//! then: local/doc-wording
use std::{fs, io, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let checks: &[(&str, &[&str])] = &[
        (
            "README.md",
            &[
                "D5 seed-1 completion running",
                "source-only design card revised per D34",
            ],
        ),
        ("docs/architecture.md", &["`input_block` in discrete.cu"]),
        (
            "docs/geometry.md",
            &[
                "the specified bivectors preserve them",
                "Q30 integer Hopf map",
                "Hamming-rank read (**`active`** discrete",
                "`(2(ac+bd), 2(bc−ad), a²+b²−c²−d²)` in fixed point",
            ],
        ),
        (
            "docs/figures/geometry/generate.py",
            &[
                "Contrast: today's read uses",
                "is running. Bit patterns",
                "distinct, non-repeating offsets",
                "preserve the 240 roots",
            ],
        ),
        (
            "docs/figures/geometry/hamming-rank-read.svg",
            &["today's read uses"],
        ),
        (
            "docs/figures/geometry/vsa-hypervectors.svg",
            &["is running"],
        ),
        (
            "docs/figures/geometry/zeta-phases.svg",
            &["distinct, non-repeating offsets"],
        ),
        (
            "docs/figures/geometry/connections.svg",
            &["preserve the 240 roots"],
        ),
        ("xtask/src/figures.rs", &["Select, transport, sum and snap"]),
        (
            "docs/figures/architecture.svg",
            &["Select, transport, sum and snap"],
        ),
    ];
    let mut count = 0;
    for (file, rejected) in checks {
        let text = fs::read_to_string(Path::new(file))?;
        for phrase in *rejected {
            count += 1;
            if text.contains(phrase) {
                return Err(io::Error::other(format!("rejected claim in {file}: {phrase}")).into());
            }
        }
    }
    println!(
        "documentation wording regressions: {count} rejected claims absent in {} files",
        checks.len()
    );
    Ok(())
}
