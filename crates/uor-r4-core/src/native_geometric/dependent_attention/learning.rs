//! Offline suffix-utility curriculum. Query labels are discovered by actual
//! reader/decoder responses, never supplied by the authored transition oracle.
//! The frozen reader is not differentiated; this is auxiliary query learning,
//! not a claim of end-to-end gradient credit through discrete retrieval.
use super::{
    data::Example,
    runtime::{self, Artifact, Control},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::{
        circuit::{self, Circuit, Node, Parameters},
        runtime::{self as read, Error, Result},
    },
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub epochs: usize,
    pub learning_rate: f64,
    pub wall_seconds: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            epochs: 256,
            learning_rate: 0.05,
            wall_seconds: 300,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub epoch: usize,
    pub auxiliary_bce: f64,
    pub exact: usize,
    pub changed_tables: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub config: Config,
    pub completed_epochs: usize,
    pub selected_epoch: usize,
    pub stopped: String,
    pub progress: Vec<Progress>,
    pub suffix_queries: usize,
    pub elapsed_ms: u128,
    pub input_cell_counts: Vec<[usize; 4]>,
}
pub fn initialize(reader: read::Artifact, train: &[Example]) -> Result<(Artifact, Parameters)> {
    let update = Circuit {
        input_count: 24,
        nodes: (0..16)
            .map(|i| Node {
                inputs: [i, 16 + i % 8],
            })
            .collect(),
        outputs: (24..40).collect(),
    };
    let parameters = Parameters {
        cells: vec![[0.1; 4]; 16],
    };
    let mut h = blake3::Hasher::new();
    for b in [
        include_bytes!("runtime.rs").as_slice(),
        include_bytes!("learning.rs").as_slice(),
        include_bytes!("data.rs").as_slice(),
    ] {
        h.update(b);
    }
    let a = Artifact {
        schema: 1,
        reader_digest: *blake3::hash(&reader.encode()?).as_bytes(),
        reader,
        update,
        tables: parameters.export()?,
        source_digest: *h.finalize().as_bytes(),
        data_digest: *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?)
            .as_bytes(),
        training: "untrained update; retained learned reader and decoder".into(),
    };
    Ok((a, parameters))
}
struct Target {
    input: [bool; 24],
    bits: [bool; 16],
}
fn targets(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    start: std::time::Instant,
    wall_seconds: u64,
) -> Result<(Vec<Target>, usize)> {
    let mut result = Vec::new();
    let mut calls = 0;
    for e in train {
        if start.elapsed().as_secs() >= wall_seconds {
            return Err(Error::State);
        }
        let first = read::generate(&a.reader, g, m, &e.records, e.query, read::Control::Full)?;
        let selected = first.selected.ok_or(Error::State)?;
        let mut useful = Vec::new();
        for record in &e.records {
            let suffix =
                read::generate(&a.reader, g, m, &e.records, record.key, read::Control::Full)?;
            calls += 1;
            if suffix.tokens == [u16::from(e.answer), 256] {
                useful.push(record.key);
            }
        }
        if useful.len() != 1 {
            return Err(Error::State);
        }
        // This key is discovered by actual final-answer utility. No e.next_query or
        // expected_query transition function is consulted in this training module.
        let key = useful[0];
        result.push(Target {
            input: runtime::update_input(e.query, e.records[selected].value),
            bits: std::array::from_fn(|i| key[i / 8] & (1 << (i % 8)) != 0),
        });
    }
    Ok((result, calls))
}
pub fn fit(
    initial: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    config: Config,
) -> Result<(Artifact, Fit)> {
    if train.is_empty()
        || config.epochs == 0
        || config.epochs > 2048
        || config.wall_seconds == 0
        || !config.learning_rate.is_finite()
        || config.learning_rate <= 0.
        || config.learning_rate > 0.1
    {
        return Err(Error::Shape);
    }
    initial.validate(g)?;
    if initial.data_digest
        != *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
    {
        return Err(Error::Artifact);
    }
    let start = std::time::Instant::now();
    let (targets, calls) = targets(initial, g, m, train, start, config.wall_seconds)?;
    let mut a = initial.clone();
    let mut p = circuit::parameters_from_tables(&a.tables)?;
    let mut audit = Fit {
        config: config.clone(),
        completed_epochs: 0,
        selected_epoch: 0,
        stopped: "completed_dose".into(),
        progress: Vec::new(),
        suffix_queries: calls,
        elapsed_ms: 0,
        input_cell_counts: vec![[0; 4]; 16],
    };
    for t in &targets {
        for (i, node) in a.update.nodes.iter().enumerate() {
            audit.input_cell_counts[i][usize::from(t.input[node.inputs[0]])
                + 2 * usize::from(t.input[node.inputs[1]])] += 1;
        }
    }
    let mut best = 0;
    let mut best_tables = a.tables.clone();
    for epoch in 1..=config.epochs {
        if start.elapsed().as_secs() >= config.wall_seconds {
            audit.stopped = "wall_limit".into();
            break;
        }
        let mut gradient = vec![[0.; 4]; p.cells.len()];
        let mut loss = 0.;
        for t in &targets {
            let tape = circuit::soft_forward(&a.update, &p, &t.input.map(f64::from))?;
            let mut dg = vec![0.; 16];
            for i in 0..16 {
                let v = tape.outputs[i].clamp(1e-9, 1. - 1e-9);
                let label = f64::from(t.bits[i]);
                loss -= label * v.ln() + (1. - label) * (1. - v).ln();
                dg[i] = (v - label) / (v * (1. - v));
            }
            let (grad, _) = circuit::soft_backward(&a.update, &p, &tape, &dg)?;
            for (i, row) in grad.iter().enumerate() {
                for (j, &value) in row.iter().enumerate() {
                    gradient[i][j] += value / (targets.len() as f64);
                }
            }
        }
        for (row, grad) in p.cells.iter_mut().zip(gradient) {
            for (value, delta) in row.iter_mut().zip(grad) {
                *value =
                    (*value - config.learning_rate * delta.clamp(-10., 10.)).clamp(0.001, 0.999);
            }
        }
        a.tables = p.export()?;
        audit.completed_epochs = epoch;
        if epoch == 1 || epoch % 16 == 0 || epoch == config.epochs {
            let mut exact = 0;
            for e in train {
                let out = runtime::generate(&a, g, m, &e.records, e.query, Control::Full)?;
                exact += usize::from(out.second.tokens == [u16::from(e.answer), 256]);
            }
            if exact > best {
                best = exact;
                best_tables = a.tables.clone();
                audit.selected_epoch = epoch;
            }
            audit.progress.push(Progress {
                epoch,
                auxiliary_bce: loss / (16 * targets.len()) as f64,
                exact,
                changed_tables: a
                    .tables
                    .iter()
                    .zip(&initial.tables)
                    .filter(|(a, b)| a != b)
                    .count(),
            });
        }
    }
    a.tables = best_tables;
    a.training=serde_json::to_string(&serde_json::json!({"configuration":config,"selected_epoch":audit.selected_epoch,"method":"suffix-answer-utility derived query auxiliary; multilinear BCE then hard export; actual two-read hard output selects training checkpoint","fixed":["retained learned reader/decoder","paired bit topology","two read schedule","typed records"],"learned":"16 LUT2 update tables","oracle_transition_used_by_learner":false,"joint_gradient_through_reads":false})).map_err(|_|Error::Artifact)?;
    audit.elapsed_ms = start.elapsed().as_millis();
    Ok((a, audit))
}
