//! Training-free observer diagnostic over unchanged, deserialized model states.
//! No model weights are fitted or modified by this module.
use super::*;
#[path = "observer_probe_math.rs"]
mod math;
use math::{nearest, peel};
use std::path::Path;
use uor_r4_core::native_geometric::learner::realtext_support::{paired_interval, Agg};

fn offset(model: &TlModel, event: usize, f: &[i32]) -> Result<Vec<i32>, String> {
    let mut extra = vec![0; model.h_dim];
    extra.extend_from_slice(f);
    extra.extend((0..TL_EVENTS).map(|i| i32::from(i == event)));
    let mut add = model.wf.forward_i32(&extra)?;
    for (a, b) in add.iter_mut().zip(&model.bh) {
        *a += *b;
    }
    Ok(add)
}

fn transported(
    model: &TlModel,
    embeddings: &[Vec<i32>],
    add: &[i32],
) -> Result<Vec<Vec<i32>>, String> {
    embeddings
        .iter()
        .map(|e| {
            let previous: Vec<i32> = e.iter().zip(add).map(|(a, b)| a + b).collect();
            Ok(model
                .wh
                .forward_i32(&previous)?
                .iter()
                .map(|x| x >> model.recurrent_shift)
                .collect())
        })
        .collect()
}

fn accuracy(rows: &[serde_json::Value], field: &str, restricted: bool) -> serde_json::Value {
    let kept: Vec<_> = rows
        .iter()
        .filter(|r| !restricted || r["restricted"] == true)
        .collect();
    let correct = kept.iter().filter(|r| r[field] == true).count();
    serde_json::json!({"correct": correct, "total": kept.len(),
        "accuracy": correct as f64 / kept.len().max(1) as f64})
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    root: &Path,
    model: &TlModel,
    windows: &[ProseWindow],
    names: &[String],
    in_top: &[bool],
    c1: &Cond,
    c2: &Cond,
    uni: &Uni,
    lambdas: (f64, f64),
    metadata: serde_json::Value,
) -> Result<ExitCode, String> {
    let started = Instant::now();
    let states = record_served_states(model, windows);
    println!("TRANSPORT: recorded {} actual served states", states.len());
    let reference_bits: f64 = windows
        .iter()
        .map(|w| model.score_example(&prose_example(w)).bits_generate)
        .sum();
    let recorded_bits: f64 = states.iter().map(|s| s.bits).sum();
    if (reference_bits - recorded_bits).abs() > 1e-7 {
        return Err("recorded states do not reproduce the model scorer".into());
    }
    let embeddings: Vec<Vec<i32>> = (0..model.e.rows)
        .map(|v| (0..model.h_dim).map(|j| model.e.value(v, j)).collect())
        .collect();
    let f = model.typed_block(&[], &[], SlFacts::default());
    let observe_add = offset(model, TL_EV_OBSERVE, &f)?;
    let generate_add = offset(model, TL_EV_GENERATE, &f)?;
    let observe_dictionary = transported(model, &embeddings, &observe_add)?;
    let generate_dictionary = transported(model, &embeddings, &generate_add)?;
    let raw_transport_dictionary = transported(model, &embeddings, &vec![0; model.h_dim])?;
    let mut previous_events = Vec::new();
    for w in windows {
        previous_events.extend((0..w.tokens.len() - PREFIX).map(|i| {
            if i < 2 {
                TL_EV_OBSERVE
            } else {
                TL_EV_GENERATE
            }
        }));
    }
    if previous_events.len() != states.len() {
        return Err("event count mismatch".into());
    }
    let mut rows = Vec::with_capacity(states.len());
    let mut losses = vec![vec![(0.0, 0usize); names.len()]; 5];
    let mut saturated = 0usize;
    for (i, s) in states.iter().enumerate() {
        let cur = s.cur.ok_or("missing current token")? as usize;
        let prev = s.prev.ok_or("missing previous token")? as usize;
        let add = if s.event == TL_EV_OBSERVE {
            &observe_add
        } else {
            &generate_add
        };
        let dict = if previous_events[i] == TL_EV_OBSERVE {
            &observe_dictionary
        } else {
            &generate_dictionary
        };
        let raw = nearest(&s.h, &embeddings);
        let centered_h: Vec<i32> = s.h.iter().zip(add).map(|(a, b)| a - b).collect();
        let centered_cur = nearest(&centered_h, &embeddings);
        let self_residual = peel(&s.h, &embeddings[raw], add);
        let centered_residual = peel(&s.h, &embeddings[centered_cur], add);
        let oracle_residual = peel(&s.h, &embeddings[cur], add);
        let decoded = nearest(&self_residual, dict);
        let centered_decoded = nearest(&centered_residual, dict);
        let oracle_decoded = nearest(&oracle_residual, dict);
        let wrong_frame = nearest(&self_residual, &embeddings);
        let uncentered_transport = nearest(&self_residual, &raw_transport_dictionary);
        saturated += s.h.iter().filter(|x| x.abs() >= model.h_clamp).count();
        // Label rotation is a fixed negative control; no label enters either self decoder.
        let null_prev = states[(i + 977) % states.len()].prev;
        let restricted = in_top[cur] && in_top[prev] && in_top[s.target as usize];
        let cps = |p: usize, c: usize| {
            if p >= model.vocab || c >= model.vocab {
                uni.p(s.target)
            } else {
                family_p(c1, c2, uni, p, c, s.target, lambdas)
            }
        };
        let values = [
            s.bits,
            -cps(prev, cur).log2(),
            -cps(decoded, raw).log2(),
            -cps(centered_decoded, centered_cur).log2(),
            -cps(oracle_decoded, cur).log2(),
        ];
        for (arm, bits) in values.iter().enumerate() {
            losses[arm][s.doc].0 += bits;
            losses[arm][s.doc].1 += 1;
        }
        rows.push(
            serde_json::json!({"index":i,"doc":s.doc,"restricted":restricted,
            "cur":cur,"prev":prev,"target":s.target,"raw_id":raw,
            "decoded_prev":decoded,"centered_cur":centered_cur,
            "centered_prev":centered_decoded,"oracle_prev":oracle_decoded,
            "raw_cur_correct":raw==cur,"raw_prev_correct":raw==prev,
            "self_prev_correct":decoded==prev,"centered_cur_correct":centered_cur==cur,
            "centered_prev_correct":centered_decoded==prev,
            "oracle_prev_correct":oracle_decoded==prev,"wrong_frame_correct":wrong_frame==prev,
            "uncentered_transport_correct":uncentered_transport==prev,
            "rotated_label_correct":Some(decoded as u32)==null_prev,
            "self_pair_correct":raw==cur && decoded==prev,
            "centered_pair_correct":centered_cur==cur && centered_decoded==prev,
            "bits":values}),
        );
    }
    let fields = [
        "raw_cur_correct",
        "raw_prev_correct",
        "self_prev_correct",
        "centered_cur_correct",
        "centered_prev_correct",
        "oracle_prev_correct",
        "wrong_frame_correct",
        "uncentered_transport_correct",
        "rotated_label_correct",
        "self_pair_correct",
        "centered_pair_correct",
    ];
    let mut scores = serde_json::Map::new();
    for field in fields {
        scores.insert(
            field.to_owned(),
            serde_json::json!({
        "all":accuracy(&rows,field,false),"legacy_restricted":accuracy(&rows,field,true)}),
        );
    }
    let arm_names = [
        "artifact",
        "exact_context_count",
        "self_decoded_context_count",
        "centered_self_decoded_context_count",
        "oracle_current_decoded_previous_count",
    ];
    let aggs: Vec<Agg> = losses
        .iter()
        .map(|arm| Agg {
            rows: arm
                .iter()
                .enumerate()
                .map(|(i, (bits, n))| (names[i].clone(), *bits, *n))
                .collect(),
        })
        .collect();
    let loss_report: Vec<_> = aggs
        .iter()
        .enumerate()
        .map(|(i, a)| {
            serde_json::json!({
        "arm":arm_names[i],"bits_per_target":a.micro(),
        "minus_artifact_doc_bootstrap":paired_interval(a,&aggs[0],20260923),
        "minus_exact_count_doc_bootstrap":paired_interval(a,&aggs[1],20260923),
        "documents":a.rows})
        })
        .collect();
    let report = serde_json::json!({"schema":"uor-r4.observer-transport-probe/1",
        "metadata":metadata,"states_sha256":recorded_states_sha256(&states),
        "positions":states.len(),"documents":names.len(),"scores":scores,
        "scorer_parity_absolute_bits":(reference_bits-recorded_bits).abs(),
        "saturated_coordinates":saturated,"count_lambdas":lambdas,"losses":loss_report,
        "elapsed_seconds":started.elapsed().as_secs_f64(),
        "hypothesis":"Failure of a raw-embedding observer is not absence of transported token information.",
        "decoder":"Decode cur from h alone; subtract its embedding and known event/fact offset; match residual to floor(Wh*(E[v]+previous-event offset)/2^shift).",
        "oracle_boundary":"oracle_current_* receives true cur; self and centered_self receive no token labels. All labels are used only for scoring.",
        "claims":"Frozen-state diagnostic and offline count rescore only. No fitted successor, product runtime, useful generation or geometric advantage established."});
    write_checked(
        root,
        "positions.json",
        &serde_json::to_vec(&rows).map_err(|e| e.to_string())?,
    )?;
    write_checked(
        root,
        "receipt.json",
        &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )?;
    seal(root).map_err(|e| format!("seal: {e}"))?;
    let errors = verify(root).map_err(|e| format!("verify: {e}"))?;
    if !errors.is_empty() {
        return Err(format!("manifest verification: {errors:?}"));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    Ok(ExitCode::SUCCESS)
}
