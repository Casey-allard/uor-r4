//! Final-output-constrained trajectory search over the exact serving transition.
//! The action table learns from successful byte/EOS suffixes, not gold read depth.
use super::{
    runtime as binding,
    schedule_data::Example,
    scheduling::{self as runtime, Artifact, Control, Frame, ROWS},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    recurrent_text::runtime::Action,
    relational_attention::runtime::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
pub fn target(e: &Example) -> Vec<u16> {
    e.answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
pub fn initialize(parent: binding::Artifact, train: &[Example]) -> Result<Artifact> {
    let sources = concat!(
        include_str!("scheduling.rs"),
        include_str!("schedule_learning.rs"),
        include_str!("schedule_data.rs")
    );
    Ok(Artifact{schema:1,parent_digest:*blake3::hash(&parent.encode()?).as_bytes(),parent,actions:vec![2;ROWS],
        source_digest:*blake3::hash(sources.as_bytes()).as_bytes(),data_digest:*blake3::hash(&serde_json::to_vec(train).map_err(|_|Error::Artifact)?).as_bytes(),
        training:"Output-constrained actual-transition search; consistent shared row action selected from successful final byte/EOS trajectories. No gold read count/source path/intermediate labels; frozen reader/update/writer. Explicit one/two question punctuation interface and usable-continuation feature; not joint primitive gradients or learned parsing.".into()})
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
    fn path(&mut self, s: Frame, offset: usize, depth: usize) -> Result<Option<Vec<(usize, u8)>>> {
        if self.visited >= 512 || depth >= runtime::MAX_STEPS || s.core.exhausted {
            return Ok(None);
        }
        self.visited += 1;
        if s.core.done {
            return Ok((offset == self.target.len()).then(Vec::new));
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
        for action in [Action::Emit, Action::Stop, Action::Read] {
            let mut next = s.clone();
            let token = runtime::execute(self.a, self.g, &o, &mut next, action, Control::Full)?;
            let next_offset = if let Some(t) = token {
                if self.target.get(offset) != Some(&t) {
                    continue;
                }
                offset + 1
            } else {
                offset
            };
            if let Some(mut suffix) = self.path(next, next_offset, depth + 1)? {
                suffix.insert(0, (o.core.row, action.index() as u8));
                return Ok(Some(suffix));
            }
        }
        Ok(None)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preparation {
    pub counts: Vec<[usize; 3]>,
    pub missing: Vec<String>,
    pub conflicts: Vec<Value>,
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
        counts: vec![[0; 3]; ROWS],
        missing: vec![],
        conflicts: vec![],
        trajectories: 0,
        search_nodes: 0,
        max_search_nodes: 0,
    };
    let mut witnesses = vec![[String::new(), String::new(), String::new()]; ROWS];
    for e in train {
        let qs = runtime::clauses(&e.prompt)?;
        let state = runtime::start(a, g, &qs)?;
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
    let mut fit = Fit {
        changed_rows: vec![],
        training_exact: 0,
        training_rows: train.len(),
        actions: vec![],
        supervised_rows: vec![],
    };
    for (row, counts) in p.counts.iter().enumerate() {
        let acts: Vec<_> = counts
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, _)| i as u8)
            .collect();
        if acts.len() > 1 {
            return Err(Error::Shape);
        }
        if let Some(&action) = acts.first() {
            fit.supervised_rows.push(row);
            if action != next.actions[row] {
                fit.changed_rows.push(row);
            }
            next.actions[row] = action;
        }
    }
    for e in train {
        let out = runtime::generate(&next, g, m, &e.records, &e.prompt, Control::Full)?;
        fit.training_exact += usize::from(!out.exhausted && out.tokens == target(e));
    }
    fit.actions = next.actions.clone();
    Ok((next, fit))
}
