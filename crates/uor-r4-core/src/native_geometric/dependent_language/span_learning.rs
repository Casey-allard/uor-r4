//! Direct answer/EOS supervision of span rules, followed by actual generation.
//! Intermediate paths and span boundaries are not training arguments.
use super::{
    completion, scheduling,
    span::{self as runtime, Artifact, Candidate, Control, FEATURES, MAX_RULES},
    span_data::Example,
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    language_relation::runtime::{Route, RouteStatus},
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
pub fn target(e: &Example) -> Vec<u16> {
    e.answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
pub fn initialize(
    parent: completion::Artifact,
    g: &BoundGeometry,
    train: &[Example],
) -> Result<Artifact> {
    Ok(Artifact{schema:2,context_words:super::span_boundary::learn(g,train)?,parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),parent,rules:vec![],feature_names:runtime::feature_names(),source_digest:*blake3::hash(concat!(include_str!("span.rs"),include_str!("span_learning.rs"),include_str!("span_data.rs"),include_str!("span_boundary.rs")).as_bytes()).as_bytes(),data_digest:*blake3::hash(&serde_json::to_vec(train).map_err(|_|Error::Artifact)?).as_bytes(),training:"Context-word roles learned from training records excluding every word used in a training answer, with positive usage overriding negative context. Direct raw-question final answer/EOS supervision of 1..5-of19 span predicates, <=8 monotone rules. Every candidate uses actual retained completion/writer generation. No gold span, source or intermediate labels train rules. Completion policy/updater/writer frozen; dependent transfer is evaluation-only.".into()})
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prepared {
    pub candidates: Vec<Candidate>,
    pub compatible: Vec<bool>,
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
            if scheduling::clauses(&e.prompt)?.len() != 1 {
                return Err(Error::Shape);
            }
            let candidates = runtime::candidates(a, g, m, &e.records, &e.prompt, Control::Full)?;
            let mut compatible = Vec::new();
            for v in &candidates {
                let out = completion::generate_routed(
                    &a.parent,
                    g,
                    m,
                    &e.records,
                    &e.prompt,
                    completion::Control::Full,
                    scheduling::Control::Full,
                    |_, _| {
                        Ok(Route {
                            status: RouteStatus::Selected,
                            selected: Some(v.value.clone()),
                            compatible: vec![[v.value.source, v.value.word]],
                        })
                    },
                )?;
                compatible.push(
                    out.trace.tokens == target(e)
                        && !out.trace.exhausted
                        && out.outcome == completion::Outcome::Answered,
                );
            }
            Ok(Prepared {
                candidates,
                compatible,
            })
        })
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub proposals: usize,
    pub rules: Vec<u32>,
    pub covered: Vec<usize>,
    pub training_rows: usize,
    pub training_exact: usize,
    pub missing_target_rows: usize,
    pub ambiguous_target_rows: usize,
    pub stopping: String,
}
fn selected(rules: &[u32], p: &Prepared, extra: u32) -> (usize, bool) {
    let mut n = 0;
    let mut good = true;
    for (v, &yes) in p.candidates.iter().zip(&p.compatible) {
        if rules.iter().any(|&r| v.features & r == r) || (extra != 0 && v.features & extra == extra)
        {
            n += 1;
            good &= yes;
        }
    }
    (n, good)
}
fn proposals(start: usize, left: usize, mask: u32, out: &mut Vec<u32>) {
    if mask != 0 {
        out.push(mask)
    }
    if left == 0 {
        return;
    }
    for i in start..FEATURES {
        proposals(i + 1, left - 1, mask | (1 << i), out)
    }
}
pub fn fit(
    initial: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    prepared: &[Prepared],
) -> Result<(Artifact, Fit)> {
    initial.validate(g)?;
    if train.is_empty()
        || train.len() != prepared.len()
        || initial.data_digest
            != *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
    {
        return Err(Error::Shape);
    }
    let mut options = Vec::new();
    proposals(0, 5, 0, &mut options);
    options.sort_by_key(|r| (r.count_ones(), *r));
    options.dedup();
    let mut a = initial.clone();
    let mut covered = 0;
    let mut history = Vec::new();
    for _ in 0..MAX_RULES {
        let old: Vec<_> = prepared.iter().map(|p| selected(&a.rules, p, 0)).collect();
        let mut best = 0;
        let mut best_count = covered;
        for &r in &options {
            let mut count = 0;
            let mut valid = true;
            for (p, &(on, og)) in prepared.iter().zip(&old) {
                let (n, good) = selected(&a.rules, p, r);
                if !good || (on == 1 && og && n != 1) {
                    valid = false;
                    break;
                }
                count += usize::from(n == 1 && good);
            }
            if valid && count > best_count {
                best = r;
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
    let mut exact = 0;
    for e in train {
        let out = runtime::generate(&a, g, m, &e.records, &e.prompt, Control::Full)?;
        exact += usize::from(
            out.trace.tokens == target(e)
                && !out.trace.exhausted
                && out.outcome == completion::Outcome::Answered,
        );
    }
    let report = Fit {
        proposals: options.len(),
        rules: a.rules.clone(),
        covered: history,
        training_rows: train.len(),
        training_exact: exact,
        missing_target_rows: prepared
            .iter()
            .filter(|p| !p.compatible.iter().any(|&b| b))
            .count(),
        ambiguous_target_rows: prepared
            .iter()
            .filter(|p| p.compatible.iter().filter(|&&b| b).count() > 1)
            .count(),
        stopping: if covered == train.len() {
            "complete_cover"
        } else {
            "no_safe_improving_rule"
        }
        .into(),
    };
    Ok((a, report))
}
/// Refine boundary credit while keeping the fitted routing rule and recurrent
/// operators fixed. Candidate identity/order/output compatibility must survive.
pub fn refine_boundary(
    initial: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    prepared: &[Prepared],
) -> Result<(Artifact, Vec<Prepared>)> {
    initial.validate(g)?;
    if train.len() != prepared.len()
        || initial.data_digest
            != *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
    {
        return Err(Error::Shape);
    }
    let mut a = initial.clone();
    a.context_words = super::span_boundary::learn_from_compatible(g, train, prepared)?;
    a.source_digest = *blake3::hash(
        concat!(
            include_str!("span.rs"),
            include_str!("span_learning.rs"),
            include_str!("span_data.rs"),
            include_str!("span_boundary.rs")
        )
        .as_bytes(),
    )
    .as_bytes();
    a.training=format!("Boundary evidence refined from the intersection of outside words across every actual final-answer/EOS-compatible source span, minus all answerwords. Fitted span rules and completion/updater/writer frozen. No goldsource/span/path or development labels. Refinement parent blake3:{}.",blake3::hash(&initial.encode()?));
    a.validate(g)?;
    let mut next = Vec::new();
    for (e, p) in train.iter().zip(prepared) {
        let candidates = runtime::candidates(&a, g, m, &e.records, &e.prompt, Control::Full)?;
        if candidates.len() != p.candidates.len()
            || candidates
                .iter()
                .zip(&p.candidates)
                .any(|(x, y)| x.value != y.value || x.last_word != y.last_word)
        {
            return Err(Error::State);
        }
        next.push(Prepared {
            candidates,
            compatible: p.compatible.clone(),
        });
    }
    Ok((a, next))
}
pub fn evaluate_frozen_rules(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    prepared: &[Prepared],
) -> Result<Fit> {
    if train.len() != prepared.len() {
        return Err(Error::Shape);
    }
    let mut exact = 0;
    for e in train {
        let out = runtime::generate(a, g, m, &e.records, &e.prompt, Control::Full)?;
        exact += usize::from(
            out.trace.tokens == target(e)
                && !out.trace.exhausted
                && out.outcome == completion::Outcome::Answered,
        );
    }
    Ok(Fit {
        proposals: 0,
        rules: a.rules.clone(),
        covered: vec![],
        training_rows: train.len(),
        training_exact: exact,
        missing_target_rows: prepared
            .iter()
            .filter(|p| !p.compatible.iter().any(|&b| b))
            .count(),
        ambiguous_target_rows: prepared
            .iter()
            .filter(|p| p.compatible.iter().filter(|&&b| b).count() > 1)
            .count(),
        stopping: "boundary_credit_refined_routing_rules_frozen".into(),
    })
}
