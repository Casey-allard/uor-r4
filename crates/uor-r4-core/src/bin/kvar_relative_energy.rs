//! Matched ordinary/C8 versus exact Q8 read-energy residual on frozen KVAR memory.
//! This is a finite pilot. It does not train a language model or claim a
//! Hamiltonian physical system; serving uses only table lookup, shift and add.

use super::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::hamilton_transport as ht;

const SOURCE_SHA256: &str = "7aec17b87fa8a005ca87855f8a68a156c094d82fc08b1011b3f3401e4b251401";
const SWEEPS: usize = 5;
const TEMP: f32 = 32.0;
const BOOST: i32 = 1 << STORE_SHIFT;

#[derive(Clone)]
struct Trial {
    trace: TerminalTrace,
    off_bits: f64,
    on_bits: f64,
    cell: usize,
    seed: u64,
}

#[derive(Clone, serde::Serialize)]
struct Fitted {
    actions: Vec<u8>,
    kernel: [i8; 8],
    shift: u8,
}

#[derive(Clone)]
struct Outcome {
    pred: usize,
    scores: Vec<i32>,
    bits: f64,
    read_on: bool,
}

fn relations_q8() -> Result<[[u8; 8]; 8], String> {
    let mut table = [[0u8; 8]; 8];
    let unit = [1, 0, 0, 0];
    let images: Vec<[i32; 4]> = (0..8)
        .map(|a| ht::apply(a, unit))
        .collect::<Result<_, _>>()?;
    for query in 0..8 {
        for value in 0..8 {
            let rel = ht::apply(ht::inverse(query as u8)?, images[value])?;
            table[query][value] = images
                .iter()
                .position(|&v| v == rel)
                .ok_or("Q8 product not in signed-unit codebook")?
                as u8;
        }
    }
    Ok(table)
}

fn relations_c8() -> [[u8; 8]; 8] {
    std::array::from_fn(|query| std::array::from_fn(|value| ((value + 8 - query) & 7) as u8))
}

fn shift_from_train(trials: &[Trial]) -> u8 {
    let mut values: Vec<u32> = trials
        .iter()
        .filter(|r| r.trace.valid)
        .map(|r| r.trace.gate_logit.unsigned_abs())
        .collect();
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let p90 = values[(values.len() * 9 / 10).min(values.len() - 1)];
    let mut shift = 0u8;
    while shift < 8 && (7u32 << shift) < p90 {
        shift += 1;
    }
    shift
}

fn augmented_logit(row: &Trial, fit: &Fitted, rel: &[[u8; 8]; 8], transport_off: bool) -> i32 {
    let address = if transport_off {
        rel[0][0] as usize
    } else {
        let query = fit.actions[row.trace.query as usize] as usize;
        let value = fit.actions[row.trace.value as usize] as usize;
        rel[query][value] as usize
    };
    row.trace.gate_logit + ((fit.kernel[address] as i32) << fit.shift)
}

fn objective(trials: &[Trial], fit: &Fitted, rel: &[[u8; 8]; 8]) -> f64 {
    let sum: f64 = trials
        .iter()
        .map(|row| {
            if row.trace.valid && augmented_logit(row, fit, rel, false) > 0 {
                row.on_bits
            } else {
                row.off_bits
            }
        })
        .sum();
    sum / trials.len() as f64
}

fn fit(trials: &[Trial], shift: u8, rel: &[[u8; 8]; 8]) -> Fitted {
    let mut fit = Fitted {
        actions: (0..VOCAB).map(|x| (x & 7) as u8).collect(),
        kernel: [0; 8],
        shift,
    };
    for _ in 0..SWEEPS {
        for class in 0..8 {
            let mut best = fit.kernel[class];
            let mut best_loss = objective(trials, &fit, rel);
            for candidate in -7..=7 {
                fit.kernel[class] = candidate;
                let loss = objective(trials, &fit, rel);
                if loss + 1e-10 < best_loss {
                    best = candidate;
                    best_loss = loss;
                }
            }
            fit.kernel[class] = best;
        }
        for token in 0..VOCAB {
            let mut best = fit.actions[token];
            let mut best_loss = objective(trials, &fit, rel);
            for candidate in 0..8 {
                fit.actions[token] = candidate;
                let loss = objective(trials, &fit, rel);
                if loss + 1e-10 < best_loss {
                    best = candidate;
                    best_loss = loss;
                }
            }
            fit.actions[token] = best;
        }
    }
    fit
}

fn evaluate(row: &Trial, fit: &Fitted, rel: &[[u8; 8]; 8], transport_off: bool) -> Outcome {
    let read_on = row.trace.valid && augmented_logit(row, fit, rel, transport_off) > 0;
    let mut scores = row.trace.base_scores.clone();
    if read_on {
        scores[row.trace.value as usize] += BOOST;
    }
    let pred = argmax_i32(&scores);
    let float_scores: Vec<f32> = scores.iter().map(|&v| v as f32).collect();
    Outcome {
        pred,
        bits: bits_at(&float_scores, row.trace.target, TEMP),
        scores,
        read_on,
    }
}

fn trace_rows(q: &QParams, episodes: &[Episode], cells: &[(usize, usize)]) -> Vec<Trial> {
    let dims = Dims {
        d: D,
        out: OUT,
        vocab: VOCAB,
        gate: true,
        store: true,
    };
    episodes
        .iter()
        .map(|ep| {
            let (_, _, trace) = serve_scores_trace(q, dims, ep, true);
            let mut on_scores = trace.base_scores.clone();
            on_scores[trace.value as usize] += BOOST;
            let off_scores: Vec<f32> = trace.base_scores.iter().map(|&v| v as f32).collect();
            let on_scores: Vec<f32> = on_scores.iter().map(|&v| v as f32).collect();
            Trial {
                off_bits: bits_at(&off_scores, trace.target, TEMP),
                on_bits: bits_at(&on_scores, trace.target, TEMP),
                cell: cells
                    .iter()
                    .position(|&(k, lag)| (k, lag) == (ep.k, ep.lag))
                    .unwrap_or(0),
                seed: ep.seed,
                trace,
            }
        })
        .collect()
}

fn load_params(path: &std::path::Path) -> Result<[(u64, QParams); 2], String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read params: {e}"))?;
    let sha = hex::encode(Sha256::digest(&bytes));
    if sha != SOURCE_SHA256 {
        return Err(format!("parameter SHA-256 mismatch: {sha}"));
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let arms = value["arms"].as_array().ok_or("missing arms")?;
    let mut found = Vec::new();
    for arm in arms {
        if arm["arm"] != "c" {
            continue;
        }
        let seed = arm["seed"].as_u64().ok_or("missing seed")?;
        let q: QParams = serde_json::from_value(arm["quantized_parameters"].clone())
            .map_err(|e| format!("quantized parameters: {e}"))?;
        if q.wh.len() != D * D
            || q.wf.len() != D * NT
            || q.wo.len() != OUT * D
            || q.emb.len() != VOCAB * D
            || q.bh.len() != D
            || q.ag.len() != VOCAB * D
            || q.bw.len() != VOCAB * D
            || q.br.len() != VOCAB * D
            || q.cg.len() != VOCAB
            || q.cw.len() != VOCAB
            || q.cr.len() != VOCAB
        {
            return Err("parameter shape mismatch".into());
        }
        found.push((seed, q));
    }
    found.sort_by_key(|x| x.0);
    if found.len() != 2 || found[0].0 != 1 || found[1].0 != 2 {
        return Err("expected c seeds 1 and 2".into());
    }
    Ok([found.remove(0), found.remove(0)])
}

fn arm_metrics(rows: &[Trial], outs: &[Outcome], cells: &[(usize, usize)]) -> Value {
    let acc: Vec<f64> = rows
        .iter()
        .zip(outs)
        .map(|(r, o)| f64::from((o.pred == r.trace.target) as u8))
        .collect();
    let bits: Vec<f64> = outs.iter().map(|o| o.bits).collect();
    let per_cell: Vec<Value> = cells
        .iter()
        .enumerate()
        .map(|(cell, &(k, lag))| {
            let idx: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter(|(_, r)| r.cell == cell)
                .map(|(i, _)| i)
                .collect();
            json!({
                "k": k, "lag": lag, "n": idx.len(),
                "accuracy": idx.iter().map(|&i| acc[i]).sum::<f64>() / idx.len() as f64,
                "bits_per_query": idx.iter().map(|&i| bits[i]).sum::<f64>() / idx.len() as f64,
            })
        })
        .collect();
    json!({"accuracy": mean(&acc), "bits_per_query": mean(&bits), "per_cell": per_cell})
}

pub(super) fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 5 {
        return Err("usage: kvar-recall NEW_REPORT_ROOT --relative-energy PINNED_PARAMETERS_JSON --quick|--development|--final".into());
    }
    let root = PathBuf::from(&args[1]);
    let source = PathBuf::from(&args[3]);
    let (train_per_cell, held_per_cell, held_group, mode) = match args[4].as_str() {
        "--quick" => (8, 8, 2u64, "quick"),
        "--development" => (100, 34, 2u64, "development"),
        "--final" => (100, 34, 4u64, "final"),
        _ => return Err("expected --quick, --development or --final".into()),
    };
    claim(&root).map_err(|e| e.to_string())?;
    let models = load_params(&source)?;
    let cells = panel_cells();
    let mut train = Vec::new();
    let mut held = Vec::new();
    for &(k, lag) in &cells {
        for i in 0..train_per_cell {
            train.push(gen_episode(seed_for(1, k, lag, i), k, lag));
        }
        for i in 0..held_per_cell {
            held.push(gen_episode(seed_for(held_group, k, lag, i), k, lag));
        }
    }
    let mut count = Count2::new(VOCAB, OUT);
    count.train(&train);
    let count_correct = held
        .iter()
        .filter(|e| count.predict(e) == e.target as usize)
        .count();
    let overwrite_correct = held
        .iter()
        .filter(|e| overwrite_predict(e) == e.target as usize)
        .count();
    let count_at_chance =
        ((count_correct as f64 / held.len() as f64) - 1.0 / OUT as f64).abs() <= 0.02;
    let panel_valid = count_at_chance && overwrite_correct as f64 / held.len() as f64 >= 0.99;
    let f_table = relations_c8();
    let h_table = relations_q8()?;
    let mut summaries = Vec::new();
    let mut fitted = Vec::new();
    let mut predictions = Vec::new();
    let mut rng = Rng::new(0xA11C_E024_0000_0001);
    for (seed, q) in &models {
        let train_rows = trace_rows(q, &train, &cells);
        let held_rows = trace_rows(q, &held, &cells);
        let shift = shift_from_train(&train_rows);
        let f_fit = fit(&train_rows, shift, &f_table);
        let h_fit = fit(&train_rows, shift, &h_table);
        let off_fit = Fitted {
            actions: h_fit.actions.clone(),
            kernel: [0; 8],
            shift,
        };
        let f: Vec<Outcome> = held_rows
            .iter()
            .map(|r| evaluate(r, &f_fit, &f_table, false))
            .collect();
        let h: Vec<Outcome> = held_rows
            .iter()
            .map(|r| evaluate(r, &h_fit, &h_table, false))
            .collect();
        let h_off: Vec<Outcome> = held_rows
            .iter()
            .map(|r| evaluate(r, &h_fit, &h_table, true))
            .collect();
        let base: Vec<Outcome> = held_rows
            .iter()
            .map(|r| evaluate(r, &off_fit, &h_table, false))
            .collect();
        for (i, (row, b)) in held_rows.iter().zip(&base).enumerate() {
            let (original_pred, original_scores) = serve_scores(
                q,
                Dims {
                    d: D,
                    out: OUT,
                    vocab: VOCAB,
                    gate: true,
                    store: true,
                },
                &held[i],
                true,
            );
            if b.pred != original_pred || b.scores != original_scores {
                return Err(format!(
                    "kernel-off failed base replay seed {seed}, row {i}"
                ));
            }
            predictions.push(json!({
                "model_seed": seed, "index": i, "episode_seed": row.seed,
                "cell": row.cell, "query": row.trace.query, "stored_value": row.trace.value,
                "valid": row.trace.valid, "gate_logit": row.trace.gate_logit,
                "target": row.trace.target,
                "ordinary": {"pred":f[i].pred,"scores":f[i].scores,"read":f[i].read_on},
                "q8": {"pred":h[i].pred,"scores":h[i].scores,"read":h[i].read_on},
                "transport_off": {"pred":h_off[i].pred,"scores":h_off[i].scores,"read":h_off[i].read_on},
                "kernel_off": {"pred":b.pred,"scores":b.scores,"read":b.read_on},
            }));
        }
        let f_bits: Vec<f64> = f.iter().map(|o| o.bits).collect();
        let h_bits: Vec<f64> = h.iter().map(|o| o.bits).collect();
        let off_bits: Vec<f64> = h_off.iter().map(|o| o.bits).collect();
        let base_bits: Vec<f64> = base.iter().map(|o| o.bits).collect();
        let (hf_lo, hf_hi) = bootstrap_paired(&h_bits, &f_bits, BOOTSTRAP, &mut rng);
        let (ho_lo, ho_hi) = bootstrap_paired(&h_bits, &off_bits, BOOTSTRAP, &mut rng);
        let (hb_lo, hb_hi) = bootstrap_paired(&h_bits, &base_bits, BOOTSTRAP, &mut rng);
        summaries.push(json!({
            "seed": seed, "shift": shift,
            "ordinary": arm_metrics(&held_rows,&f,&cells),
            "q8": arm_metrics(&held_rows,&h,&cells),
            "transport_off": arm_metrics(&held_rows,&h_off,&cells),
            "kernel_off": arm_metrics(&held_rows,&base,&cells),
            "h_minus_f_bits": mean(&h_bits)-mean(&f_bits),
            "h_minus_f_ci95": [hf_lo,hf_hi],
            "h_minus_transport_off_bits": mean(&h_bits)-mean(&off_bits),
            "h_minus_transport_off_ci95": [ho_lo,ho_hi],
            "h_minus_kernel_off_bits": mean(&h_bits)-mean(&base_bits),
            "h_minus_kernel_off_ci95": [hb_lo,hb_hi],
            "ordinary_train_bits": objective(&train_rows,&f_fit,&f_table),
            "q8_train_bits": objective(&train_rows,&h_fit,&h_table),
            "base_d5": d5_report(q,Dims{d:D,out:OUT,vocab:VOCAB,gate:true,store:true}),
        }));
        fitted.push(json!({"seed":seed,"ordinary":f_fit,"q8":h_fit}));
    }
    let geometry_gate = panel_valid
        && summaries.iter().all(|s| {
            s["h_minus_f_bits"].as_f64().unwrap_or(f64::INFINITY) <= -0.5
                && s["h_minus_f_ci95"][1].as_f64().unwrap_or(f64::INFINITY) < 0.0
                && s["h_minus_transport_off_ci95"][1]
                    .as_f64()
                    .unwrap_or(f64::INFINITY)
                    < 0.0
        });
    let report = json!({
        "schema":"uor-r4.kvar-relative-energy/1", "mode":mode,
        "plan":"docs/integration/kvar-relative-energy-plan-2026-09-24.md",
        "parent_pr":1380, "parent_commit":"a6c4ace5",
        "source_parameters":source, "source_sha256":SOURCE_SHA256,
        "panel":{"train_seed_group":1,"held_seed_group":held_group,"train_per_cell":train_per_cell,"held_per_cell":held_per_cell,"cells":cells},
        "controls":{"count_correct":count_correct,"overwrite_correct":overwrite_correct,"held_total":held.len(),"count_at_chance":count_at_chance,"panel_valid":panel_valid},
        "arms":{"ordinary":"C8-indexed delta-memory read energy","geometric":"Q8 relative signed-vector-action read energy","transport_off":"identity frames, same Q8 kernel and base memory","kernel_off":"zero kernel, exact base c replay"},
        "cost":{"learned_action_labels_each":VOCAB,"learned_4bit_kernel_weights_each":8,"fixed_relation_table_entries_each":64,"additional_final_query_reads_each":4,"shift_add_compare_each":true,"base_parameter_slots_per_token":8771,"training_sweeps_each":SWEEPS},
        "results":summaries,"geometry_margin_met":geometry_gate,
        "scope":"Matched finite residual pilot on synthetic KVAR. The original full f/h arms and language/geometric-attention/energy qualifications remain unrun.",
    });
    let panel = json!({
        "train":train.iter().map(|e|json!({"seed":e.seed,"k":e.k,"lag":e.lag,"key":e.key,"target":e.target,"inputs":e.inputs})).collect::<Vec<_>>(),
        "held":held.iter().map(|e|json!({"seed":e.seed,"k":e.k,"lag":e.lag,"key":e.key,"target":e.target,"inputs":e.inputs})).collect::<Vec<_>>(),
    });
    let write = |name: &str, value: &Value| -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
        std::fs::write(root.join(name), bytes).map_err(|e| format!("{name}: {e}"))
    };
    write("receipt.json", &report)?;
    write("fitted.json", &json!({"models":fitted}))?;
    write("predictions.json", &json!({"rows":predictions}))?;
    write("panel.json", &panel)?;
    seal(&root).map_err(|e| e.to_string())?;
    let unlisted = verify(&root).map_err(|e| e.to_string())?;
    if !unlisted.is_empty() {
        return Err(format!("unlisted sealed files: {unlisted:?}"));
    }
    println!(
        "root={} mode={mode} valid={panel_valid} geometry_gate={geometry_gate}",
        root.display()
    );
    for s in report["results"].as_array().ok_or("missing summary")? {
        println!(
            "seed={} f_bits={:.4} h_bits={:.4} off_bits={:.4} base_bits={:.4} h-f={:.4}",
            s["seed"],
            s["ordinary"]["bits_per_query"].as_f64().unwrap_or(f64::NAN),
            s["q8"]["bits_per_query"].as_f64().unwrap_or(f64::NAN),
            s["transport_off"]["bits_per_query"]
                .as_f64()
                .unwrap_or(f64::NAN),
            s["kernel_off"]["bits_per_query"]
                .as_f64()
                .unwrap_or(f64::NAN),
            s["h_minus_f_bits"].as_f64().unwrap_or(f64::NAN)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q8_relative_table_preserves_direction_and_sign() {
        let rel = relations_q8().unwrap();
        for a in 0..8 {
            assert_eq!(rel[a][a], 0);
        }
        assert_ne!(rel[2][4], rel[4][2]);
        assert_ne!(rel[0][2], rel[0][3]);
    }

    #[test]
    fn kernel_off_exactly_replays_base_serving() {
        let dims = Dims {
            d: 4,
            out: 4,
            vocab: 6,
            gate: true,
            store: true,
        };
        let lay = layout(dims.d, dims.out, dims.vocab);
        let p = init_params(&lay, dims, 7);
        let q = served_quantize(&p, &lay, dims);
        let ep = Episode {
            inputs: vec![4, 0, 1, 4, 2, 3, 4, 0, 2, 1, 5, 0],
            target: 2,
            k: 2,
            lag: 1,
            key: 0,
            seed: 0,
        };
        let (pred, scores, trace) = serve_scores_trace(&q, dims, &ep, true);
        let mut reconstructed = trace.base_scores;
        if trace.valid && trace.gate_logit > 0 {
            reconstructed[trace.value as usize] += BOOST;
        }
        assert_eq!(scores, reconstructed);
        assert_eq!(pred, argmax_i32(&reconstructed));
    }
}
