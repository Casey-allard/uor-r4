//! Offline curriculum: learn the byte codec, then fit compatibility from actual
//! counterfactual generated-response utility. The latter is a biased surrogate:
//! hard-decision residual through the log multilinear conjunction, not an argmax derivative.
//! Query/key encoding is fixed. No target index is supplied to the predictor.
use super::{
    circuit::{self, Circuit, Node, Parameters},
    data::Example,
    runtime::{self, Artifact, Result, FEATURES},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub codec_epochs: usize,
    pub routing_epochs: usize,
    pub learning_rate: f64,
    pub wall_seconds: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            codec_epochs: 256,
            routing_epochs: 800,
            learning_rate: 0.01,
            wall_seconds: 600,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub epoch: usize,
    pub objective_loss: f64,
    pub train_exact: usize,
    pub changed_tables: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fit {
    pub configuration: Config,
    pub codec_updates: usize,
    pub routing_updates: usize,
    pub skipped_no_useful_read: usize,
    pub selected_epoch: usize,
    pub stopped: String,
    pub progress: Vec<Progress>,
    pub elapsed_ms: u128,
}
pub fn structures() -> (Circuit, Circuit) {
    // Every feature has a learned unary predicate, including both possible bit
    // values. Balanced AND reductions combine them; no target-specific mask.
    // Constants belong to topology, not a hand-authored matching predicate.
    let mut nodes: Vec<Node> = (0..FEATURES).map(|i| Node { inputs: [i, i] }).collect();
    let mut level: Vec<usize> = (FEATURES..FEATURES + FEATURES).collect();
    while level.len() > 1 {
        let mut next = Vec::new();
        for pair in level.chunks(2) {
            if pair.len() == 1 {
                next.push(pair[0])
            } else {
                let index = FEATURES + nodes.len();
                nodes.push(Node {
                    inputs: [pair[0], pair[1]],
                });
                next.push(index)
            }
        }
        level = next;
    }
    let scorer = Circuit {
        input_count: FEATURES,
        nodes,
        outputs: level,
    };
    let decoder = Circuit {
        input_count: 9,
        nodes: (0..9).map(|i| Node { inputs: [i, 8] }).collect(),
        outputs: (9..18).collect(),
    };
    (scorer, decoder)
}
pub fn initialized(
    g: &BoundGeometry,
    training: &[Example],
) -> Result<(Artifact, Parameters, Parameters)> {
    let (scorer, decoder) = structures();
    let mut sc = circuit::parameters_from_tables(&vec![circuit::AND; scorer.nodes.len()])?;
    for (i, c) in sc.cells.iter_mut().enumerate() {
        *c = if i < FEATURES {
            [0.999; 4]
        } else {
            [0., 0., 0., 1.]
        };
    }
    let dc = Parameters {
        cells: vec![[0.1; 4]; 9],
    };
    let bytes = serde_json::to_vec(training).map_err(|_| runtime::Error::Artifact)?;
    let mut source = blake3::Hasher::new();
    for bytes in [
        include_bytes!("learning.rs").as_slice(),
        include_bytes!("runtime.rs").as_slice(),
        include_bytes!("circuit.rs").as_slice(),
        include_bytes!("data.rs").as_slice(),
    ] {
        source.update(bytes);
    }
    let artifact = Artifact {
        schema: 1,
        geometry: g.id(),
        data_digest: *blake3::hash(&bytes).as_bytes(),
        source_digest: *source.finalize().as_bytes(),
        score_tables: sc.export()?,
        decode_tables: dc.export()?,
        scorer,
        decoder,
        training: "initialized uniform compatibility; untrained codec".into(),
    };
    artifact.validate(g)?;
    Ok((artifact, sc, dc))
}
fn bce(p: f64, t: f64) -> (f64, f64) {
    let p = p.clamp(1e-9, 1. - 1e-9);
    (
        -(t * p.ln() + (1. - t) * (1. - p).ln()),
        (p - t) / (p * (1. - p)),
    )
}
pub fn fit(
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    config: Config,
) -> Result<(Artifact, Fit)> {
    if train.is_empty()
        || config.codec_epochs > 4096
        || config.routing_epochs > 10000
        || !config.learning_rate.is_finite()
        || config.learning_rate <= 0.
        || config.learning_rate > 0.1
        || config.wall_seconds == 0
    {
        return Err(runtime::Error::Shape);
    }
    let start = std::time::Instant::now();
    let (a0, mut sc, mut dc) = initialized(g, train)?;
    let mut a = a0.clone();
    let mut audit = Fit {
        configuration: config.clone(),
        codec_updates: 0,
        routing_updates: 0,
        skipped_no_useful_read: 0,
        selected_epoch: 0,
        stopped: "completed_dose".into(),
        progress: Vec::new(),
        elapsed_ms: 0,
    };
    // Declared codec auxiliary task on training answer bytes only. It learns
    // content bits and termination tables; no fixed Copy opcode is installed.
    for _ in 0..config.codec_epochs {
        if start.elapsed().as_secs() >= config.wall_seconds {
            audit.stopped = "wall_limit".into();
            break;
        }
        let mut grad = vec![[0.; 4]; dc.cells.len()];
        for e in train {
            for seen in [false, true] {
                let input = runtime::decoder_input(e.answer, seen).map(f64::from);
                let tape = circuit::soft_forward(&a.decoder, &dc, &input)?;
                let mut dg = vec![0.; 9];
                for (i, v) in dg.iter_mut().enumerate() {
                    if i == 8 || !seen {
                        let t = if i == 8 {
                            f64::from(seen)
                        } else {
                            f64::from(e.answer & (1 << i) != 0)
                        };
                        *v = bce(tape.outputs[i], t).1;
                    }
                }
                let (gs, _) = circuit::soft_backward(&a.decoder, &dc, &tape, &dg)?;
                for (j, row) in gs.iter().enumerate() {
                    for (k, v) in row.iter().enumerate() {
                        grad[j][k] += v / (train.len() as f64)
                    }
                }
            }
        }
        for (j, row) in dc.cells.iter_mut().enumerate() {
            for (k, v) in row.iter_mut().enumerate() {
                *v = (*v - config.learning_rate * grad[j][k].clamp(-10., 10.)).clamp(0.001, 0.999);
            }
        }
        audit.codec_updates += 1;
    }
    a.decode_tables = dc.export()?;
    let packets: Vec<Vec<Vec<f64>>> = train
        .iter()
        .map(|e| {
            e.records
                .iter()
                .map(|r| {
                    runtime::relation(g, m, e.query, r.key)
                        .map(|p| runtime::features(&p).into_iter().map(f64::from).collect())
                })
                .collect::<Result<_>>()
        })
        .collect::<Result<_>>()?;
    let mut momentum = vec![[0.; 4]; FEATURES];
    let mut variance = momentum.clone();
    let mut best_exact = 0;
    let mut best_tables = a.score_tables.clone();
    for epoch in 0..config.routing_epochs {
        if start.elapsed().as_secs() >= config.wall_seconds {
            audit.stopped = "wall_limit".into();
            break;
        }
        let mut grad = vec![[0.; 4]; FEATURES];
        let mut loss = 0.;
        // Refresh complete forced-read byte/EOS responses with current decoder.
        // Training labels come from response membership, not key comparison or
        // a hidden source index. No frozen pointer enters a subsequent forward.
        for (e, features) in train.iter().zip(&packets) {
            let mut useful = [false; 4];
            for (i, r) in e.records.iter().enumerate() {
                useful[i] = runtime::decode_symbol(&a, r.value, false)? == u16::from(e.answer)
                    && runtime::decode_symbol(&a, r.value, true)? == 256;
            }
            let positives = useful.iter().filter(|&&x| x).count();
            if positives == 0 {
                audit.skipped_no_useful_read += 1;
                continue;
            }
            for (i, input) in features.iter().enumerate() {
                let hard_input: Vec<bool> = input.iter().map(|&x| x != 0.).collect();
                let hard = circuit::hard(&a.scorer, &a.score_tables, &hard_input)?;
                let (label, weight) = if useful[i] {
                    (1., 0.5 / (positives as f64))
                } else {
                    (0., 0.5 / ((4 - positives) as f64))
                };
                let residual = f64::from(hard.outputs[0]) - label;
                loss += weight * 0.5 * residual * residual / (train.len() as f64);
                // Fixed AND composition gives p_soft = product(active unary cells).
                // Use the hard decision in the loss and derivative of log(p_soft)
                // as a biased backward chart: d log(p_soft)/d cell = 1/cell.
                // This avoids multiplying 288 attenuating paths. No threshold is
                // tuned after fitting, and every hard forward uses exported tables.
                for (j, &bit) in hard_input.iter().enumerate() {
                    let row = if bit { 3 } else { 0 };
                    grad[j][row] += weight * residual / (train.len() as f64) / sc.cells[j][row];
                }
            }
        }
        for j in 0..FEATURES {
            for k in 0..4 {
                let d = grad[j][k].clamp(-10., 10.);
                momentum[j][k] = 0.9 * momentum[j][k] + 0.1 * d;
                variance[j][k] = 0.999 * variance[j][k] + 0.001 * d * d;
                let mh = momentum[j][k] / (1. - 0.9f64.powi((epoch + 1) as i32));
                let vh = variance[j][k] / (1. - 0.999f64.powi((epoch + 1) as i32));
                sc.cells[j][k] = (sc.cells[j][k] - config.learning_rate * mh / (vh.sqrt() + 1e-8))
                    .clamp(0.001, 0.999);
            }
        }
        audit.routing_updates += 1;
        a.score_tables = sc.export()?;
        if epoch == 0 || (epoch + 1) % 25 == 0 || epoch + 1 == config.routing_epochs {
            let mut exact = 0;
            for e in train {
                exact += usize::from(
                    runtime::generate(&a, g, m, &e.records, e.query, runtime::Control::Full)?
                        .tokens
                        == [u16::from(e.answer), 256],
                );
            }
            if exact > best_exact {
                best_exact = exact;
                best_tables = a.score_tables.clone();
                audit.selected_epoch = epoch + 1;
            }
            audit.progress.push(Progress {
                epoch: epoch + 1,
                objective_loss: loss,
                train_exact: exact,
                changed_tables: a
                    .score_tables
                    .iter()
                    .zip(&a0.score_tables)
                    .filter(|(a, b)| a != b)
                    .count(),
            });
        }
    }
    a.score_tables = best_tables;
    a.training=serde_json::to_string(&serde_json::json!({"configuration":config,"selected_epoch":audit.selected_epoch,"selection":"highest training hard exact at epoch 1 and fixed 25-epoch checkpoints, earliest tie","method":"codec auxiliary then hard-decision residual through log-multilinear-conjunction surrogate","fixed":["canonical query/key","AND composition"],"learned":["unary compatibility","byte/EOS codec"],"raw_prose":false,"recurrent_query_learning":false})).map_err(|_|runtime::Error::Artifact)?;
    audit.elapsed_ms = start.elapsed().as_millis();
    Ok((a, audit))
}
