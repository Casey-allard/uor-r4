//! Bounded offline refinement of inherited rules over structured occurrence witnesses.
//! Supervision is final generated answer/EOS only; source/span/path labels are
//! not consulted. The completion, writer, updater and context roles stay fixed.
use super::{
    completion, correspondence as runtime, scheduling, span_data::Example, span_learning::target,
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime::{Route, RouteStatus},
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub proposals: usize,
    pub proposal_masks: Vec<u32>,
    pub rules: Vec<u32>,
    pub covered: Vec<usize>,
    pub training_rows: usize,
    pub training_exact: usize,
    pub preparation_candidates: usize,
    pub forced_candidate_generations: usize,
    pub max_preparation_search_nodes: usize,
    pub missing_target_rows: usize,
    pub ambiguous_target_rows: usize,
    pub target_rows_without_reachable_proposal: usize,
    pub conflicting_candidate_proposal_signatures: usize,
    pub rows_with_conflicting_compatible_candidate: usize,
    pub conflict_scope: String,
    pub stopping: String,
}
struct Candidate {
    features: Vec<u32>,
    compatible: bool,
}
struct Prepared {
    candidates: Vec<Candidate>,
}
fn proposals(base: u32) -> Vec<u32> {
    let bits: Vec<_> = (0..runtime::FEATURES)
        .filter(|i| base & (1u32 << i) == 0)
        .collect();
    let mut masks = vec![base];
    for (offset, &i) in bits.iter().enumerate() {
        let one = base | (1u32 << i);
        if one.count_ones() <= 5 {
            masks.push(one);
        }
        for &j in &bits[offset + 1..] {
            let two = one | (1u32 << j);
            if two.count_ones() <= 5 {
                masks.push(two);
            }
        }
    }
    masks.sort_by_key(|r| (r.count_ones(), *r));
    masks.dedup();
    masks
}
fn admitted(candidate: &Candidate, rules: &[u32], extra: u32) -> bool {
    candidate
        .features
        .iter()
        .any(|&f| rules.iter().any(|&r| f & r == r) || (extra != 0 && f & extra == extra))
}
fn selected(p: &Prepared, rules: &[u32], extra: u32) -> (usize, bool) {
    let mut count = 0;
    let mut good = true;
    for candidate in &p.candidates {
        if admitted(candidate, rules, extra) {
            count += 1;
            good &= candidate.compatible;
        }
    }
    (count, good)
}
/// Equality here means identical candidate admission under every member of the
/// declared proposal family. It is a scoped collision, not a proof that no other
/// representation or compatible candidate could solve the training row.
fn signature(c: &Candidate, options: &[u32]) -> Vec<u64> {
    let mut bits = vec![0; options.len().div_ceil(64)];
    for (i, &rule) in options.iter().enumerate() {
        if admitted(c, &[], rule) {
            bits[i / 64] |= 1u64 << (i % 64);
        }
    }
    bits
}
pub fn fit(
    initial: &runtime::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
) -> Result<(runtime::Artifact, Fit)> {
    initial.validate(g)?;
    if train.is_empty() || initial.rules.len() != 1 {
        return Err(Error::Shape);
    }
    let base = initial.rules[0];
    let options = proposals(base);
    let mut prepared = Vec::with_capacity(train.len());
    let mut preparation_candidates = 0;
    let mut max_search = 0;
    for e in train {
        if scheduling::clauses(&e.prompt)?.len() != 1 {
            return Err(Error::Shape);
        }
        let search =
            runtime::candidates(initial, g, m, &e.records, &e.prompt, runtime::Control::Full)?;
        max_search = max_search.max(search.nodes);
        let expected = target(e);
        let mut candidates = Vec::with_capacity(search.candidates.len());
        for candidate in search.candidates {
            let value = candidate.span.value;
            let out = completion::generate_routed(
                &initial.parent.parent,
                g,
                m,
                &e.records,
                &e.prompt,
                completion::Control::Full,
                scheduling::Control::Full,
                |_, _| {
                    Ok(Route {
                        status: RouteStatus::Selected,
                        selected: Some(value.clone()),
                        compatible: vec![[value.source, value.word]],
                    })
                },
            )?;
            preparation_candidates += 1;
            let mut features: Vec<_> = candidate
                .witnesses
                .into_iter()
                .map(|w| w.features)
                .collect();
            features.sort_unstable();
            features.dedup();
            candidates.push(Candidate {
                features,
                compatible: out.trace.tokens == expected
                    && !out.trace.exhausted
                    && out.outcome == completion::Outcome::Answered,
            });
        }
        prepared.push(Prepared { candidates });
    }
    let mut a = initial.clone();
    a.rules.clear();
    a.data_digest =
        *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes();
    a.source_digest = *blake3::hash(
        concat!(
            include_str!("correspondence.rs"),
            include_str!("correspondence_learning.rs"),
            include_str!("occurrence.rs"),
            include_str!("span.rs"),
            include_str!("span_boundary.rs"),
            include_str!("span_learning.rs")
        )
        .as_bytes(),
    )
    .as_bytes();
    a.training = format!("Bounded offline structured occurrence rule refinement from parent blake3:{}. Every enumerated source/span is supervised by actual retained completion/writer final answer/EOS; expected source, span and intermediate labels are not used. A candidate is admitted by any one complete witness satisfying one rule; features are never combined across witnesses. Proposals are the inherited rule {base} plus zero, one or two existing predicate bits, at most five set bits, sorted by bit count then numeric value. Greedy cover starts empty, rejects every incompatible admission and preserves previously unique coverage; at most eight rules. The added twentieth predicate observes ordered unit source adjacency inside matched query runs separated by inherited learned context-only words or unmatched occurrences. Context roles, completion, updater, writer and geometric encoding are frozen. Training rows are artifact-bound; development is not an input to this learner.", blake3::hash(&initial.encode()?));
    let mut covered = 0;
    let mut history = Vec::new();
    for _ in 0..runtime::MAX_RULES {
        let old: Vec<_> = prepared.iter().map(|p| selected(p, &a.rules, 0)).collect();
        let mut best = 0;
        let mut best_count = covered;
        for &rule in &options {
            let mut count = 0;
            let mut valid = true;
            for (p, &(old_count, old_good)) in prepared.iter().zip(&old) {
                let (n, good) = selected(p, &a.rules, rule);
                if !good || (old_count == 1 && old_good && n != 1) {
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
        history.push(covered);
        if covered == train.len() {
            break;
        }
    }
    a.validate(g)?;
    let mut training_exact = 0;
    for e in train {
        let out = runtime::generate(&a, g, m, &e.records, &e.prompt, runtime::Control::Full)?;
        training_exact += usize::from(
            out.trace.tokens == target(e)
                && !out.trace.exhausted
                && out.outcome == completion::Outcome::Answered,
        );
    }
    let mut signatures = BTreeMap::<Vec<u64>, u8>::new();
    for candidate in prepared.iter().flat_map(|p| &p.candidates) {
        *signatures
            .entry(signature(candidate, &options))
            .or_default() |= if candidate.compatible { 1 } else { 2 };
    }
    let conflicting: BTreeSet<_> = signatures
        .iter()
        .filter(|(_, state)| **state == 3)
        .map(|(bits, _)| bits.clone())
        .collect();
    let report = Fit {
        proposals: options.len(),
        proposal_masks: options.clone(),
        rules: a.rules.clone(),
        covered: history,
        training_rows: train.len(),
        training_exact,
        preparation_candidates,
        forced_candidate_generations: preparation_candidates,
        max_preparation_search_nodes: max_search,
        missing_target_rows: prepared.iter().filter(|p| !p.candidates.iter().any(|c| c.compatible)).count(),
        ambiguous_target_rows: prepared.iter().filter(|p| p.candidates.iter().filter(|c| c.compatible).count() > 1).count(),
        target_rows_without_reachable_proposal: prepared.iter().filter(|p| !p.candidates.iter().any(|c| c.compatible && options.iter().any(|r| admitted(c, &[], *r)))).count(),
        conflicting_candidate_proposal_signatures: conflicting.len(),
        rows_with_conflicting_compatible_candidate: prepared.iter().filter(|p| p.candidates.iter().any(|c| c.compatible && conflicting.contains(&signature(c, &options)))).count(),
        conflict_scope: "A compatible and incompatible source/span share an identical admission signature across all declared proposals; another compatible candidate may remain separable. Zero signatures are included. This is not general representational impossibility.".into(),
        stopping: if covered == train.len() {"complete_cover"} else if a.rules.len() == runtime::MAX_RULES {"rule_limit"} else {"no_safe_improving_rule"}.into(),
    };
    Ok((a, report))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_proposals_preserve_parent_and_single_witness_admission() {
        let options = proposals(327682);
        assert_eq!(options.len(), 154);
        assert!(options
            .iter()
            .all(|r| r & 327682 == 327682 && r.count_ones() <= 5));
        assert!(options
            .windows(2)
            .all(|p| (p[0].count_ones(), p[0]) < (p[1].count_ones(), p[1])));
        let split = Candidate {
            features: vec![2, 327680],
            compatible: false,
        };
        assert!(!admitted(&split, &[327682], 0));
        let one = Prepared {
            candidates: vec![Candidate {
                features: vec![327682, 327683],
                compatible: true,
            }],
        };
        assert_eq!(selected(&one, &[327682], 0), (1, true));
        let ambiguous = Prepared {
            candidates: vec![
                Candidate {
                    features: vec![327682],
                    compatible: true,
                },
                Candidate {
                    features: vec![327682],
                    compatible: false,
                },
            ],
        };
        assert_eq!(selected(&ambiguous, &[327682], 0), (2, false));
    }
}
