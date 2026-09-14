//! Output-constrained action search for explicit completion outcomes. Paths and
//! blocked clause labels remain evaluation-only; no new reader/update training.
use super::{
    completion::{self as runtime, Artifact, Control, Generated, Outcome, ROWS},
    completion_data::Example,
    scheduling,
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    recurrent_text::runtime::Action,
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub fn split(rows: &[Example]) -> (Vec<Example>, Vec<Example>) {
    rows.iter()
        .cloned()
        .partition(|e| (0..4).any(|i| e.family.starts_with(&format!("depth-n{i}-"))))
}
pub fn initialize(parent: scheduling::Artifact, train: &[Example]) -> Result<Artifact> {
    Ok(Artifact{schema:1,parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),actions:parent.actions.iter().chain(parent.actions.iter()).copied().collect(),parent,source_digest:*blake3::hash(concat!(include_str!("completion.rs"),include_str!("completion_learning.rs"),include_str!("completion_data.rs")).as_bytes()).as_bytes(),data_digest:*blake3::hash(&serde_json::to_vec(train).map_err(|_|Error::Artifact)?).as_bytes(),training:"Warm-start both pending banks from the retained scheduling table; actual transition search supervised by final answer/EOS or typed unresolved reason. No source/path/intermediate/blocked-clause labels train actions. Frozen reader/updater/writer. Pending-clause bit distinct from next-route availability. Authored familiar grammar; not general prose.".into()})
}
fn target(e: &Example) -> Vec<u16> {
    e.answer
        .as_ref()
        .map(|a| a.iter().map(|&b| u16::from(b)).chain([256]).collect())
        .unwrap_or_default()
}
fn reason_matches(outcome: &Outcome, e: &Example) -> bool {
    match outcome {
        Outcome::Unresolved { reason, .. } => {
            e.answer.is_none() && serde_json::to_value(reason).is_ok_and(|v| v == e.expected_reason)
        }
        _ => false,
    }
}
pub fn matches_target(out: &Generated, e: &Example) -> bool {
    !out.trace.exhausted
        && match &out.outcome {
            Outcome::Answered => e.answer.is_some() && out.trace.tokens == target(e),
            Outcome::Unresolved { .. } => {
                out.trace.tokens.is_empty() && reason_matches(&out.outcome, e)
            }
            Outcome::Exhausted => false,
        }
}
struct Search<'a> {
    a: &'a Artifact,
    g: &'a BoundGeometry,
    m: &'a Metric,
    e: &'a Example,
    qs: Vec<Vec<u8>>,
    target: Vec<u16>,
    visited: usize,
}
impl Search<'_> {
    fn path(
        &mut self,
        s: scheduling::Frame,
        offset: usize,
        depth: usize,
    ) -> Result<Option<Vec<(usize, u8)>>> {
        if self.visited >= 512 || depth >= scheduling::MAX_STEPS || s.core.exhausted {
            return Ok(None);
        }
        self.visited += 1;
        if s.core.done {
            return Ok((self.e.answer.is_some() && offset == self.target.len()).then(Vec::new));
        }
        let o = runtime::observe(
            self.a,
            self.g,
            self.m,
            &self.e.records,
            &self.qs,
            &s,
            Control::Full,
        )?;
        for choice in [0u8, 2, 1, 3] {
            if choice == 3 {
                if offset == 0
                    && runtime::unresolved(&o, &s).is_some_and(|v| reason_matches(&v, self.e))
                {
                    return Ok(Some(vec![(o.row, 3)]));
                }
                continue;
            }
            let mut next = s.clone();
            let token = scheduling::execute(
                &self.a.parent,
                self.g,
                &o.core,
                &mut next,
                Action::from_byte(choice)?,
                scheduling::Control::Full,
            )?;
            let next_offset = if let Some(t) = token {
                if self.target.get(offset) != Some(&t) {
                    continue;
                }
                offset + 1
            } else {
                offset
            };
            if let Some(mut suffix) = self.path(next, next_offset, depth + 1)? {
                suffix.insert(0, (o.row, choice));
                return Ok(Some(suffix));
            }
        }
        Ok(None)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preparation {
    pub counts: Vec<[usize; 4]>,
    pub missing: Vec<String>,
    pub conflicts: Vec<Value>,
    pub collapsed_conflicts: Vec<Value>,
    pub trajectories: usize,
    pub search_nodes: usize,
    pub max_search_nodes: usize,
}
pub fn prepare(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
) -> Result<Preparation> {
    a.validate(g)?;
    let mut p = Preparation {
        counts: vec![[0; 4]; ROWS],
        missing: vec![],
        conflicts: vec![],
        collapsed_conflicts: vec![],
        trajectories: 0,
        search_nodes: 0,
        max_search_nodes: 0,
    };
    let mut witnesses = vec![vec![String::new(); 4]; ROWS];
    for e in train {
        let qs = scheduling::clauses(&e.prompt)?;
        let state = scheduling::start(&a.parent, g, &qs)?;
        let mut search = Search {
            a,
            g,
            m,
            e,
            qs,
            target: target(e),
            visited: 0,
        };
        if let Some(path) = search.path(state, 0, 0)? {
            p.trajectories += 1;
            for (row, act) in path {
                p.counts[row][usize::from(act)] += 1;
                witnesses[row][usize::from(act)] = e.id.clone();
            }
        } else {
            p.missing.push(e.id.clone());
        }
        p.search_nodes += search.visited;
        p.max_search_nodes = p.max_search_nodes.max(search.visited);
    }
    for (row, c) in p.counts.iter().enumerate() {
        if c.iter().filter(|&&x| x > 0).count() > 1 {
            p.conflicts
                .push(json!({"row":row,"counts":c,"witnesses":witnesses[row]}));
        }
    }
    for row in 0..8 {
        let counts: Vec<_> = (0..4)
            .map(|i| p.counts[row][i] + p.counts[row + 8][i])
            .collect();
        if counts.iter().filter(|&&x| x > 0).count() > 1 {
            p.collapsed_conflicts.push(json!({"row_without_pending":row,"counts":counts,"low_witnesses":witnesses[row],"pending_witnesses":witnesses[row+8]}));
        }
    }
    Ok(p)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub changed_rows: Vec<usize>,
    pub training_exact: usize,
    pub training_rows: usize,
    pub actions: Vec<u8>,
    pub supervised_rows: Vec<usize>,
}
pub fn fit(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    p: &Preparation,
) -> Result<(Artifact, Fit)> {
    a.validate(g)?;
    if train.is_empty()
        || p.counts.len() != ROWS
        || !p.missing.is_empty()
        || !p.conflicts.is_empty()
        || p.trajectories != train.len()
        || a.data_digest
            != *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
    {
        return Err(Error::Shape);
    }
    let mut next = a.clone();
    let mut f = Fit {
        changed_rows: vec![],
        training_exact: 0,
        training_rows: train.len(),
        actions: vec![],
        supervised_rows: vec![],
    };
    for (row, c) in p.counts.iter().enumerate() {
        let acts: Vec<_> = c
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| i as u8)
            .collect();
        if acts.len() > 1 {
            return Err(Error::Shape);
        }
        if let Some(&act) = acts.first() {
            f.supervised_rows.push(row);
            if next.actions[row] != act {
                next.actions[row] = act;
                f.changed_rows.push(row);
            }
        }
    }
    next.validate(g)?;
    for e in train {
        f.training_exact += usize::from(matches_target(
            &runtime::generate(&next, g, m, &e.records, &e.prompt, Control::Full)?,
            e,
        ));
    }
    f.actions = next.actions.clone();
    Ok((next, f))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_keeps_whole_matched_families_and_disjoint_cohorts() -> std::result::Result<(), String>
    {
        let rows = super::super::completion_data::corpus()?;
        let (a, b) = split(&rows);
        assert_eq!((a.len(), b.len()), (896, 896));
        let names: std::collections::BTreeSet<_> = a.iter().map(|e| &e.family).collect();
        assert!(b.iter().all(|e| !names.contains(&e.family)));
        Ok(())
    }
}
