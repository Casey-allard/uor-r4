//! Finite empirical-risk selection of two shared typed update operations.
//! This is output-supervised discrete optimization, not gradient end-to-end training.
use super::{
    data::Example,
    runtime::{self, Artifact, Control, Operator, OPERATORS},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
    relational_attention::runtime::Result,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Trial {
    pub operators: [Operator; 2],
    pub exact: usize,
    pub token_errors: usize,
}
pub fn target(e: &Example) -> Vec<u16> {
    e.answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
pub fn fit(
    initial: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
) -> Result<(Artifact, Vec<Trial>)> {
    let mut best = initial.clone();
    let mut best_loss = usize::MAX;
    let mut trials = Vec::new();
    for x in OPERATORS {
        for y in OPERATORS {
            let mut a = initial.clone();
            a.operators = [x, y];
            let mut exact = 0;
            let mut errors = 0;
            for e in train {
                let out = runtime::generate(&a, g, m, &e.records, &e.query, Control::Full)?;
                let expected = target(e);
                exact += usize::from(!out.exhausted && out.tokens == expected);
                errors += out
                    .tokens
                    .iter()
                    .zip(&expected)
                    .filter(|(a, b)| a != b)
                    .count()
                    + out.tokens.len().abs_diff(expected.len())
                    + usize::from(out.exhausted);
            }
            trials.push(Trial {
                operators: [x, y],
                exact,
                token_errors: errors,
            });
            if errors < best_loss {
                best_loss = errors;
                best = a;
            }
        }
    }
    Ok((best, trials))
}
