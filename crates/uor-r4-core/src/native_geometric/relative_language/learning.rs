//! Bounded output-supervised multiple-instance rule induction. Candidate word
//! emissions can match a target at more than one occurrence; no gold path is used.
use super::{
    data::Example,
    runtime::{self, Artifact, Candidate, Control, FEATURES, MAX_RULES},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::runtime::{Error, Result},
    text_attention::runtime as text,
};
use serde::{Deserialize, Serialize};
pub fn target(e: &Example) -> Vec<u16> {
    e.answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prepared {
    pub candidates: Vec<Candidate>,
    pub compatible_outputs: Vec<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub proposals: usize,
    pub selected_rules: Vec<u32>,
    pub covered_per_rule: Vec<usize>,
    pub training_rows: usize,
    pub actual_training_exact: usize,
    pub changed_actions: Vec<usize>,
    pub ambiguous_target_occurrences: usize,
    pub stopped: String,
}
pub fn initialize(
    parent: crate::native_geometric::language_relation::runtime::Artifact,
    train: &[Example],
) -> Result<Artifact> {
    let sources = concat!(
        include_str!("runtime.rs"),
        include_str!("learning.rs"),
        include_str!("data.rs")
    );
    Ok(Artifact{schema:2,parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),parent,rules:vec![],data_digest:*blake3::hash(&serde_json::to_vec(train).map_err(|_|Error::Artifact)?).as_bytes(),source_digest:*blake3::hash(sources.as_bytes()).as_bytes(),feature_names:runtime::FEATURE_NAMES.iter().map(|x|x.to_string()).collect(),training:"Greedy bounded monotone conjunction cover (1..4 of18 relative match-order predicates,<=8rules), requiring unique candidate and no wrong-output exposure; actual candidate byte/EOS emissions provide latent compatibility, no source/word labels. No absolute token positions, source IDs or lexical dictionary enter features. Retained learned byte/EOS writer supplies lexical stopping; actual full generation decides acceptance.".into()})
}
pub fn prepare(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
) -> Result<Vec<Prepared>> {
    train
        .iter()
        .map(|e| {
            let candidates = runtime::candidates(a, g, m, &e.records, &e.question, Control::Full)?;
            let mut compatible_outputs = Vec::new();
            for c in &candidates {
                let mut actual = Vec::new();
                for &b in &c.bytes {
                    actual.push(text::symbol(&a.parent.parent.parent.parent, b, true)?);
                }
                actual.push(text::symbol(&a.parent.parent.parent.parent, 0, false)?);
                compatible_outputs.push(actual == target(e));
            }
            Ok(Prepared {
                candidates,
                compatible_outputs,
            })
        })
        .collect()
}
fn selected(a: &Artifact, p: &Prepared, extra: u32) -> (usize, bool) {
    let mut n = 0;
    let mut good = true;
    for (c, &yes) in p.candidates.iter().zip(&p.compatible_outputs) {
        if a.matches(c.features) || (extra != 0 && c.features & extra == extra) {
            n += 1;
            good &= yes;
        }
    }
    (n, good)
}
pub fn fit(
    initial: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    prepared: &[Prepared],
) -> Result<(Artifact, Fit)> {
    if train.is_empty() || train.len() != prepared.len() {
        return Err(Error::Shape);
    }
    let mut a = initial.clone();
    let changed_actions = Vec::new();
    let mut proposals = Vec::new();
    for i in 0..FEATURES {
        proposals.push(1u32 << i);
        for j in i + 1..FEATURES {
            proposals.push((1 << i) | (1 << j));
            for k in j + 1..FEATURES {
                proposals.push((1 << i) | (1 << j) | (1 << k));
                for l in k + 1..FEATURES {
                    proposals.push((1 << i) | (1 << j) | (1 << k) | (1 << l));
                }
            }
        }
    }
    proposals.sort_by_key(|m| (m.count_ones(), *m));
    let mut covered_per_rule = Vec::new();
    let mut covered = 0;
    for _ in 0..MAX_RULES {
        let old: Vec<_> = prepared.iter().map(|p| selected(&a, p, 0)).collect();
        let mut best = 0;
        let mut best_count = covered;
        for &rule in &proposals {
            let mut count = 0;
            let mut valid = true;
            for (p, &(old_n, old_good)) in prepared.iter().zip(&old) {
                let (n, good) = selected(&a, p, rule);
                if !good || (old_n == 1 && old_good && n != 1) {
                    valid = false;
                    break;
                }
                count += usize::from(n == 1 && good);
            }
            if valid && count > best_count {
                best = rule;
                best_count = count;
            }
        }
        if best == 0 {
            break;
        }
        a.rules.push(best);
        covered = best_count;
        covered_per_rule.push(covered);
        if covered == train.len() {
            break;
        }
    }
    let mut exact = 0;
    for e in train {
        let out = runtime::generate(&a, g, m, &e.records, &e.question, Control::Full)?;
        exact += usize::from(!out.actual.exhausted && out.actual.tokens == target(e));
    }
    let fit = Fit {
        proposals: proposals.len(),
        selected_rules: a.rules.clone(),
        covered_per_rule,
        training_rows: train.len(),
        actual_training_exact: exact,
        changed_actions,
        ambiguous_target_occurrences: prepared
            .iter()
            .filter(|p| p.compatible_outputs.iter().filter(|&&v| v).count() > 1)
            .count(),
        stopped: if covered == train.len() {
            "complete_cover"
        } else {
            "no_safe_improving_rule"
        }
        .into(),
    };
    Ok((a, fit))
}
