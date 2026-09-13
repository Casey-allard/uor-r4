//! Offline answer-supervised action curriculum. On each retained geometric
//! rollout, emitting is useful iff the actual decoder output matches the answer.
//! Continue and recompute otherwise. No depth or terminal-domain oracle is read.
use super::{
    data::Example,
    runtime::{self, Artifact, Control, MAX_READS},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    dependent_attention::runtime as dependent,
    hamming_refinement::metric::Metric,
    relational_attention::runtime::{self as read, Error, Result},
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
            epochs: 128,
            learning_rate: 0.1,
            wall_seconds: 300,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub epoch: usize,
    pub bce: f64,
    pub exact: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub config: Config,
    pub completed_epochs: usize,
    pub selected_epoch: usize,
    pub stopped: String,
    pub progress: Vec<Progress>,
    pub cell_counts: Vec<[usize; 2]>,
    pub rollout_reads: usize,
    pub elapsed_ms: u128,
}
pub fn initialize(parent: dependent::Artifact, train: &[Example]) -> Result<Artifact> {
    let mut h = blake3::Hasher::new();
    for b in [
        include_bytes!("runtime.rs").as_slice(),
        include_bytes!("learning.rs").as_slice(),
        include_bytes!("data.rs").as_slice(),
    ] {
        h.update(b);
    }
    Ok(Artifact {
        schema: 1,
        parent_digest: *blake3::hash(&parent.encode()?).as_bytes(),
        parent,
        emit: vec![0; 256],
        data_digest: *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?)
            .as_bytes(),
        source_digest: *h.finalize().as_bytes(),
        training: "untrained content-action policy; frozen reader/update/decoder".into(),
    })
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
        || config.epochs > 1024
        || config.wall_seconds == 0
        || !config.learning_rate.is_finite()
        || config.learning_rate <= 0.
        || config.learning_rate > 0.1
    {
        return Err(Error::Shape);
    }
    initial.validate(g)?;
    if *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
        != initial.data_digest
    {
        return Err(Error::Artifact);
    }
    let start = std::time::Instant::now();
    let mut counts = vec![[0usize; 2]; 256];
    let mut reads = 0;
    for e in train {
        let mut query = e.query;
        let mut found = false;
        for _ in 0..MAX_READS {
            if start.elapsed().as_secs() >= config.wall_seconds {
                return Err(Error::State);
            }
            let out = read::generate(
                &initial.parent.reader,
                g,
                m,
                &e.records,
                query,
                read::Control::Full,
            )?;
            let selected = out.selected.ok_or(Error::State)?;
            let payload = e.records[selected].value;
            let useful = out.tokens == [u16::from(e.answer), 256];
            counts[usize::from(payload)][usize::from(useful)] += 1;
            reads += 1;
            if useful {
                found = true;
                break;
            }
            query = dependent::update_query(&initial.parent, query, payload)?;
        }
        if !found {
            return Err(Error::State);
        }
    }
    let mut a = initial.clone();
    let mut p: [f64; 256] = std::array::from_fn(|i| if initial.emit[i] == 0 { 0.1 } else { 0.9 });
    let mut audit = Fit {
        config: config.clone(),
        completed_epochs: 0,
        selected_epoch: 0,
        stopped: "completed_dose".into(),
        progress: Vec::new(),
        cell_counts: counts.clone(),
        rollout_reads: reads,
        elapsed_ms: 0,
    };
    let mut best = 0;
    let mut best_tables = a.emit.clone();
    'epochs: for epoch in 1..=config.epochs {
        if start.elapsed().as_secs() >= config.wall_seconds {
            audit.stopped = "wall_limit".into();
            break;
        }
        let mut loss = 0.;
        for (i, c) in counts.iter().enumerate() {
            let negative = c[0] as f64;
            let positive = c[1] as f64;
            loss -= negative * (1. - p[i]).ln() + positive * p[i].ln();
            let gradient = (negative / (1. - p[i]) - positive / p[i]) / (reads as f64);
            p[i] = (p[i] - config.learning_rate * gradient).clamp(0.001, 0.999);
            a.emit[i] = u8::from(p[i] >= 0.5);
        }
        audit.completed_epochs = epoch;
        if epoch == 1 || epoch % 16 == 0 || epoch == config.epochs {
            let mut exact = 0;
            for e in train {
                if start.elapsed().as_secs() >= config.wall_seconds {
                    audit.stopped = "wall_limit".into();
                    break 'epochs;
                }
                let out = runtime::generate(&a, g, m, &e.records, e.query, Control::Full)?;
                exact += usize::from(out.tokens == [u16::from(e.answer), 256]);
            }
            if exact > best {
                best = exact;
                best_tables = a.emit.clone();
                audit.selected_epoch = epoch;
            }
            audit.progress.push(Progress {
                epoch,
                bce: loss / (reads as f64),
                exact,
            });
        }
    }
    a.emit = best_tables;
    a.training=serde_json::to_string(&serde_json::json!({"configuration":config,"selected_epoch":audit.selected_epoch,"method":"answer-supervised retained rollouts; content-action BCE; earliest best training hard output at epoch1/every16","learned":"256 byte-indexed read/emit decisions","fixed":["reader","query update","decoder","four-read resource maximum"],"depth_input":false,"joint_gradient_through_reads":false})).map_err(|_|Error::Artifact)?;
    audit.elapsed_ms = start.elapsed().as_millis();
    Ok((a, audit))
}
