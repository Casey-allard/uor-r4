//! One alternating output-directed fit: teacher-position byte/EOS likelihood,
//! actual next-symbol utility for state action, and complete-response utility
//! for the warm-started geometric reader. No target cursor advance label.
//! Fixed query encoding and integer cursor increment remain declared operators.
use super::{
    data::Example,
    runtime::{self, Artifact, Control},
};
use crate::native_geometric::{
    adaptive_attention::runtime as prior,
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    relational_attention::{
        circuit::{self, Circuit, Node},
        runtime::{self as read, Error, Result, FEATURES},
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
            learning_rate: 0.1,
            wall_seconds: 300,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub config: Config,
    pub epochs: usize,
    pub selected_epoch: usize,
    pub stopped: String,
    pub progress: Vec<serde_json::Value>,
    pub source_no_useful_epochs: usize,
    pub elapsed_ms: u128,
}
pub fn initialize(parent: prior::Artifact, train: &[Example]) -> Result<Artifact> {
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
        reader: parent.parent.reader.clone(),
        parent_digest: *blake3::hash(&parent.encode()?).as_bytes(),
        parent,
        writer: Circuit {
            input_count: 9,
            nodes: (0..9).map(|i| Node { inputs: [i, 8] }).collect(),
            outputs: (9..18).collect(),
        },
        tables: vec![0; 9],
        advance: [0; 4],
        data_digest: *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?)
            .as_bytes(),
        source_digest: *h.finalize().as_bytes(),
        training: "warm learned reader; untrained text writer and advance policy".into(),
    })
}
fn expected(e: &Example, p: usize) -> u16 {
    e.answer.get(p).map_or(256, |&b| u16::from(b))
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
        || train.iter().any(|e| {
            e.answer.len() > runtime::MAX_TEXT
                || e.records.iter().any(|r| r.text.len() > runtime::MAX_TEXT)
        })
    {
        return Err(Error::Shape);
    }
    initial.validate(g)?;
    if initial.reader.scorer
        != crate::native_geometric::relational_attention::learning::structures().0
        || initial.reader.score_tables[FEATURES..]
            .iter()
            .any(|&v| v != circuit::AND)
    {
        return Err(Error::Shape);
    }
    if initial.data_digest
        != *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes()
    {
        return Err(Error::Artifact);
    }
    let start = std::time::Instant::now();
    let mut a = initial.clone();
    let mut writer = circuit::parameters_from_tables(&a.tables)?;
    let mut state = std::array::from_fn::<_, 4, _>(|i| if a.advance[i] == 0 { 0.1 } else { 0.9 });
    let mut reader = circuit::parameters_from_tables(&a.reader.score_tables)?;
    let mut fit = Fit {
        config: config.clone(),
        epochs: 0,
        selected_epoch: 0,
        stopped: "completed_dose".into(),
        progress: Vec::new(),
        source_no_useful_epochs: 0,
        elapsed_ms: 0,
    };
    let mut best = 0;
    let mut best_a = a.clone();
    'epochs: for epoch in 1..=config.epochs {
        let mut dw = vec![[0.; 4]; 9];
        let mut ds = [0.; 4];
        let mut dr = vec![[0.; 4]; FEATURES];
        let mut positions = 0;
        let mut state_cases = 0;
        let mut loss = 0.;
        let mut useful_rows = 0;
        for e in train {
            if start.elapsed().as_secs() >= config.wall_seconds {
                fit.stopped = "wall_limit".into();
                break 'epochs;
            }
            let selected =
                runtime::select(&a, g, m, &e.records, e.query, false)?.ok_or(Error::State)?;
            let text = &e.records[selected].text;
            for p in 0..=e.answer.len() {
                let byte = text.get(p).copied();
                let present = byte.is_some();
                let tape = circuit::soft_forward(
                    &a.writer,
                    &writer,
                    &runtime::input(byte.unwrap_or(0), present).map(f64::from),
                )?;
                let target = expected(e, p);
                let mut dg = vec![0.; 9];
                for i in 0..9 {
                    if i == 8 || target != 256 {
                        let label = if i == 8 {
                            f64::from(target == 256)
                        } else {
                            f64::from(target & (1 << i) != 0)
                        };
                        let probability = tape.outputs[i].clamp(1e-9, 1. - 1e-9);
                        loss -= label * probability.ln() + (1. - label) * (1. - probability).ln();
                        dg[i] = (probability - label) / (probability * (1. - probability));
                    }
                }
                let (gradient, _) = circuit::soft_backward(&a.writer, &writer, &tape, &dg)?;
                for (i, row) in gradient.iter().enumerate() {
                    for (j, &v) in row.iter().enumerate() {
                        dw[i][j] += v;
                    }
                }
                positions += 1;
                if p < e.answer.len() {
                    let actual = runtime::symbol(&a, byte.unwrap_or(0), present)?;
                    let next = text.get(p + 1).copied();
                    let moved = runtime::symbol(&a, next.unwrap_or(0), next.is_some())?;
                    let wanted = expected(e, p + 1);
                    let stay_cost = f64::from(actual != wanted);
                    let move_cost = f64::from(moved != wanted);
                    let row = usize::from(present) + 2 * usize::from(actual != 256);
                    ds[row] += move_cost - stay_cost;
                    state_cases += 1;
                }
            }
            // Update routing only when an actual forced span decodes the whole target.
            // Initially an untrained writer may supply no useful route; no authored
            // key equality or record index is used as its routing label.
            let useful: Vec<bool> = e
                .records
                .iter()
                .map(|r| {
                    let mut ok = true;
                    for p in 0..=e.answer.len() {
                        let b = r.text.get(p).copied();
                        ok &= runtime::symbol(&a, b.unwrap_or(0), b.is_some())? == expected(e, p);
                    }
                    Ok(ok)
                })
                .collect::<Result<_>>()?;
            let positives = useful.iter().filter(|&&b| b).count();
            if positives > 0 && positives < 4 {
                useful_rows += 1;
                for (i, r) in e.records.iter().enumerate() {
                    let bits = read::features(&read::relation(g, m, e.query, r.key)?);
                    let hard =
                        circuit::hard(&a.reader.scorer, &a.reader.score_tables, &bits)?.outputs[0];
                    let weight = if useful[i] {
                        0.5 / (positives as f64)
                    } else {
                        0.5 / ((4 - positives) as f64)
                    };
                    let residual = f64::from(hard) - f64::from(useful[i]);
                    for (j, &b) in bits.iter().enumerate() {
                        let row = if b { 3 } else { 0 };
                        dr[j][row] +=
                            weight * residual / reader.cells[j][row] / (train.len() as f64);
                    }
                }
            }
        }
        for (i, row) in writer.cells.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v = (*v - config.learning_rate * (dw[i][j] / positions as f64).clamp(-10., 10.))
                    .clamp(0.001, 0.999);
            }
        }
        for (i, v) in state.iter_mut().enumerate() {
            *v =
                (*v - config.learning_rate * ds[i] / state_cases.max(1) as f64).clamp(0.001, 0.999);
            a.advance[i] = u8::from(*v >= 0.5);
        }
        for (j, row) in dr.iter().enumerate() {
            for (k, &v) in row.iter().enumerate() {
                reader.cells[j][k] =
                    (reader.cells[j][k] - 0.01 * v.clamp(-10., 10.)).clamp(0.001, 0.999);
            }
        }
        a.tables = writer.export()?;
        a.reader.score_tables = reader.export()?;
        fit.epochs = epoch;
        if useful_rows == 0 {
            fit.source_no_useful_epochs += 1;
        }
        if epoch == 1 || epoch % 16 == 0 || epoch == config.epochs {
            let mut exact = 0;
            for e in train {
                if start.elapsed().as_secs() >= config.wall_seconds {
                    fit.stopped = "wall_limit".into();
                    break 'epochs;
                }
                let out = runtime::generate(&a, g, m, &e.records, e.query, Control::Full)?;
                let mut target: Vec<u16> = e.answer.iter().map(|&b| u16::from(b)).collect();
                target.push(256);
                exact += usize::from(out.tokens == target);
            }
            fit.progress.push(serde_json::json!({"epoch":epoch,"teacher_position_bce_sum_per_position":loss/positions as f64,"exact":exact,"useful_routing_rows":useful_rows,"advance":a.advance}));
            if exact > best {
                best = exact;
                best_a = a.clone();
                fit.selected_epoch = epoch;
            }
        }
    }
    a = best_a;
    a.training=serde_json::to_string(&serde_json::json!({"config":config,"selected_epoch":fit.selected_epoch,"method":"alternating output-directed updates: teacher-position writer BCE, actual next-symbol state-action utility, complete forced-output routing residual","fixed":["canonical query encoding","cursor integer increment","exact span boundaries","retained parent"],"trainable":["reader compatibility","writer byte/EOS","advance policy"],"joint_end_to_end_gradient":false})).map_err(|_|Error::Artifact)?;
    a.reader.data_digest = a.data_digest;
    a.reader.source_digest = a.source_digest;
    a.reader.training = a.training.clone();
    fit.elapsed_ms = start.elapsed().as_millis();
    Ok((a, fit))
}
