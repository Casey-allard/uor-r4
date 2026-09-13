//! Output-constrained search supplies action credit through the real retained
//! writer, query update and geometric reads. Only the shared policy is fitted.
//! This is trajectory-derived action supervision, not end-to-end gradients.
use super::{
    data::Example,
    runtime::{self, Action, Artifact, Control, State},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::runtime::{Error, Result},
    text_attention::runtime as text,
};
use serde::{Deserialize, Serialize};
pub fn initialize(parent: text::Artifact, train: &[Example]) -> Result<Artifact> {
    let mut h = blake3::Hasher::new();
    for b in [
        include_bytes!("runtime.rs").as_slice(),
        include_bytes!("learning.rs").as_slice(),
        include_bytes!("data.rs").as_slice(),
    ] {
        h.update(b);
    }
    Ok(Artifact {
        schema: 2,
        parent_digest: *blake3::hash(&parent.encode()?).as_bytes(),
        parent,
        actions: vec![0; runtime::ROWS],
        source_digest: *h.finalize().as_bytes(),
        data_digest: *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?)
            .as_bytes(),
        training:
            "untrained shared policy; retained reader, writer, advance and query-update operators"
                .into(),
    })
}
fn target(e: &Example) -> Vec<u16> {
    e.answer
        .iter()
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
struct Search<'a> {
    a: &'a Artifact,
    g: &'a BoundGeometry,
    m: &'a Metric,
    e: &'a Example,
    target: Vec<u16>,
    visited: usize,
}
impl Search<'_> {
    fn path(&mut self, s: State, offset: usize, depth: usize) -> Result<Option<Vec<(usize, u8)>>> {
        if self.visited >= 512 || depth >= runtime::MAX_STEPS || s.exhausted {
            return Ok(None);
        }
        self.visited += 1;
        if s.done {
            return Ok((offset == self.target.len()).then(Vec::new));
        }
        let o = runtime::observe(self.a, self.g, self.m, &self.e.records, &s, Control::Full)?;
        // Emission is attempted before a silent read; complete output, including
        // EOS, decides success. No expected key, depth, record index or XOR oracle.
        let options = if o.present {
            [Action::Emit, Action::Read]
        } else {
            [Action::Stop, Action::Read]
        };
        for action in options {
            let mut next = s.clone();
            let token = runtime::execute(self.a, &o, &mut next, action, Control::Full)?;
            let next_offset = if let Some(t) = token {
                if self.target.get(offset) != Some(&t) {
                    continue;
                }
                offset + 1
            } else {
                offset
            };
            if let Some(mut suffix) = self.path(next, next_offset, depth + 1)? {
                suffix.insert(0, (o.row, action.index() as u8));
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
    pub conflicts: Vec<serde_json::Value>,
    pub rows: usize,
    pub search_nodes: usize,
    pub max_search_nodes: usize,
    pub trajectories: usize,
}
pub fn prepare(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    examples: &[Example],
) -> Result<Preparation> {
    a.validate(g)?;
    let mut p = Preparation {
        counts: vec![[0; 3]; runtime::ROWS],
        missing: Vec::new(),
        conflicts: Vec::new(),
        rows: 0,
        search_nodes: 0,
        max_search_nodes: 0,
        trajectories: 0,
    };
    let mut witnesses = vec![[String::new(), String::new(), String::new()]; runtime::ROWS];
    for e in examples {
        if e.answer.len() > runtime::MAX_BYTES || e.records.iter().any(|r| r.text.len() > 32) {
            return Err(Error::Shape);
        }
        let mut search = Search {
            a,
            g,
            m,
            e,
            target: target(e),
            visited: 0,
        };
        if let Some(path) = search.path(State::new(e.query), 0, 0)? {
            p.trajectories += 1;
            for (row, action) in path {
                p.counts[row][usize::from(action)] += 1;
                witnesses[row][usize::from(action)] = e.id.clone();
            }
        } else {
            p.missing.push(e.id.clone());
        }
        p.search_nodes += search.visited;
        p.max_search_nodes = p.max_search_nodes.max(search.visited);
    }
    for (i, c) in p.counts.iter().enumerate() {
        if c.iter().any(|&v| v > 0) {
            p.rows += 1;
        }
        if c.iter().filter(|&&v| v > 0).count() > 1 {
            p.conflicts
                .push(serde_json::json!({"row":i,"counts":c,"witnesses":witnesses[i]}));
        }
    }
    Ok(p)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub epochs: usize,
    pub selected_epoch: usize,
    pub rate: f64,
    pub elapsed_ms: u128,
    pub changed_rows: usize,
    pub progress: Vec<serde_json::Value>,
    pub stopped: String,
}
pub fn fit(
    initial: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    prepared: &Preparation,
) -> Result<(Artifact, Fit)> {
    initial.validate(g)?;
    if prepared.counts.len() != runtime::ROWS
        || !prepared.missing.is_empty()
        || !prepared.conflicts.is_empty()
        || prepared.trajectories != train.len()
        || initial.data_digest
            != *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
    {
        return Err(Error::Shape);
    }
    let start = std::time::Instant::now();
    let mut a = initial.clone();
    let mut logits = vec![[0f64; 3]; runtime::ROWS];
    let mut fit = Fit {
        epochs: 0,
        selected_epoch: 0,
        rate: 0.1,
        elapsed_ms: 0,
        changed_rows: 0,
        progress: Vec::new(),
        stopped: "completed_dose".into(),
    };
    let mut best = 0;
    let mut best_a = a.clone();
    'epochs: for epoch in 1..=1 {
        if start.elapsed().as_secs() >= 300 {
            fit.stopped = "wall_limit".into();
            break;
        }
        let mut loss = 0.;
        for (i, counts) in prepared.counts.iter().enumerate() {
            let n = counts.iter().sum::<usize>() as f64;
            if n == 0. {
                continue;
            }
            let max = logits[i].iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let exp = logits[i].map(|v| (v - max).exp());
            let sum = exp.iter().sum::<f64>();
            for j in 0..3 {
                let p = exp[j] / sum;
                let label = counts[j] as f64 / n;
                loss -= label * p.ln();
                logits[i][j] -= 0.1 * (p - label);
            }
            let mut selected = 0;
            for j in 1..3 {
                if logits[i][j] > logits[i][selected] {
                    selected = j;
                }
            }
            a.actions[i] = selected as u8;
        }
        fit.epochs = epoch;
        if epoch == 1 || epoch % 16 == 0 {
            let mut exact = 0;
            for e in train {
                if start.elapsed().as_secs() >= 300 {
                    fit.stopped = "wall_limit".into();
                    break 'epochs;
                }
                exact += usize::from(
                    runtime::generate(&a, g, m, &e.records, e.query, Control::Full)?.tokens
                        == target(e),
                );
            }
            fit.progress.push(serde_json::json!({"epoch":epoch,"exact":exact,"training_total":train.len(),"row_balanced_action_ce":loss/prepared.rows.max(1) as f64}));
            if exact > best {
                best = exact;
                best_a = a.clone();
                fit.selected_epoch = epoch;
            }
        }
    }
    a = best_a;
    fit.changed_rows = a
        .actions
        .iter()
        .zip(&initial.actions)
        .filter(|(x, y)| x != y)
        .count();
    fit.elapsed_ms = start.elapsed().as_millis();
    a.training=serde_json::json!({"method":"row-balanced categorical cross entropy on successful output-constrained real-operator trajectories","search_nodes_per_example_cap":512,"epochs":fit.epochs,"selected_epoch":fit.selected_epoch,"rate":fit.rate,"trainable":"one shared read/emit/stop policy","frozen":"geometric reader, byte/EOS writer, cursor advance, learned discrete query update","inference_inputs":"query and exact supplied records only","not_joint_end_to_end_gradients":true}).to_string();
    a.validate(g)?;
    Ok((a, fit))
}
