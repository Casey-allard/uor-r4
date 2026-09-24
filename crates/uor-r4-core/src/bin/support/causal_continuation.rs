//! Bounded causal-learning continuation. No production default is changed.
use super::*;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::learner::transferable_lexical::{TlReadPlan, TrainingCursor};
use uor_r4_core::native_geometric::learner::{
    hamilton_transport as ht, relative_action_learning as ra,
};
fn hash(bytes: &[u8]) -> String {
    super::sha256_hex(&Sha256::digest(bytes))
}
fn write_json(root: &Path, name: &str, j: &serde_json::Value) -> Result<(), String> {
    write_checked(
        root,
        name,
        &serde_json::to_vec_pretty(j).map_err(|e| e.to_string())?,
    )
}
fn next(r: &mut u64) -> u64 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    *r
}
fn body(text: &str) -> &str {
    let start = text
        .find("*** START OF")
        .and_then(|p| text[p..].find('\n').map(|i| p + i + 1))
        .unwrap_or(0);
    let end = text[start..]
        .find("*** END OF")
        .map(|i| start + i)
        .unwrap_or(text.len());
    &text[start..end]
}
pub(super) fn sources(
    tok: &HfBpeTokenizer,
    dir: &Path,
    cap: usize,
) -> Result<(Vec<ProseWindow>, Vec<serde_json::Value>), String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    files.sort();
    let mut windows = Vec::new();
    let mut manifest = Vec::new();
    for file in files {
        if !file.is_file() {
            continue;
        }
        let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let tokens = tok.encode(body(text));
        let all = prose_windows(&tokens, manifest.len());
        let take = cap.min(all.len());
        for k in 0..take {
            windows.push(all[k * all.len() / take].clone());
        }
        manifest.push(serde_json::json!({"name":file.file_name().and_then(|s|s.to_str()),"sha256":hash(&bytes),"bytes":bytes.len(),"tokens":tokens.len(),"available_windows":all.len(),"sampled_windows":take}));
    }
    if windows.is_empty() {
        return Err("empty source population".into());
    }
    Ok((windows, manifest))
}
fn authored(tok: &HfBpeTokenizer, held: bool) -> Vec<TlExample> {
    let mut out = Vec::new();
    for n in if held { 8..12 } else { 0..8 } {
        let examples = [
            (
                format!("User: Repeat exactly the number {n}.\nAssistant:"),
                format!(" {n}"),
            ),
            (
                format!("User: Write a Rust program that prints {n}.\nAssistant:\n"),
                format!("fn main() {{ println!(\"{n}\"); }}"),
            ),
        ];
        for (prompt, answer) in examples {
            let mut actions: Vec<_> = tok
                .encode(&answer)
                .into_iter()
                .map(TlAction::Generate)
                .collect();
            actions.push(TlAction::Stop);
            out.push(TlExample {
                sel: vec![],
                res: vec![],
                facts: SlFacts::default(),
                observed: tok.encode(&prompt),
                actions,
                weight: 1.,
                doc: n as usize,
                grounded: false,
                terminal_stop: true,
            });
        }
    }
    out
}
fn label(w: &Words, i: usize) -> u32 {
    if i < 2 {
        w.class_a
    } else {
        w.class_b
    }
}
fn crossed_case(w: &Words, source: usize, feedback: usize) -> TlExample {
    TlExample {
        sel: vec![w.values[source]],
        res: vec![],
        facts: SlFacts {
            history: 2,
            ..SlFacts::default()
        },
        observed: vec![],
        actions: vec![
            TlAction::Copy,
            TlAction::Generate(label(w, feedback)),
            TlAction::Stop,
        ],
        weight: 4.,
        doc: 0,
        grounded: true,
        terminal_stop: true,
    }
}
fn feedback_panel(m: &TlModel, w: &Words) -> serde_json::Value {
    let mut rows = Vec::new();
    for s in 0..4 {
        for f in 0..4 {
            let sel = [w.values[s]];
            let meaning = m.content_feature(&sel, &[]);
            let facts = m.typed_block(
                &sel,
                &[],
                SlFacts {
                    history: 2,
                    ..SlFacts::default()
                },
            );
            let h = m.init_state(&meaning, &facts);
            let h = m.transition(&h, TL_EV_COPY, Some(w.values[f]), &meaning, &facts);
            let predicted = m.decide(&h, &meaning, &facts, TL_EV_COPY, false);
            rows.push(serde_json::json!({"source_index":s,"feedback_index":f,"withheld_combination":f==(s+2)%4,"correct":predicted==TlAction::Generate(label(w,f)),"predicted":action_name(&predicted),"expected":label(w,f)}));
        }
    }
    serde_json::json!({"correct":rows.iter().filter(|r|r["correct"]==true).count(),"total":rows.len(),"withheld_correct":rows.iter().filter(|r|r["withheld_combination"]==true&&r["correct"]==true).count(),"withheld_total":4,"rows":rows,"scope":"Crossed diagnostic: fixed evidence and independently intervened emitted token. Not permission to copy an unowned value."})
}
pub(super) fn panels(m: &TlModel, w: &Words) -> serde_json::Value {
    let (tr, cl, ho) = authored_world(w);
    let p = |cases: &[Grounded]| {
        let rows = grounded_panel(m, cases);
        serde_json::json!({"correct":rows.iter().filter(|r|r["exact"]==true).count(),"total":rows.len(),"rows":rows})
    };
    let temporal: Vec<_> = tr
        .iter()
        .filter(|c| c.role == "preflight_temporal")
        .cloned()
        .collect();
    serde_json::json!({"temporal":p(&temporal),"train":p(&tr),"class":p(&cl),"heldout":p(&ho),"feedback":feedback_panel(m,w),"legacy_crossed":crossed_feedback(m,w)})
}
fn generated(m: &TlModel, tok: &HfBpeTokenizer) -> serde_json::Value {
    let mut rows = Vec::new();
    for ex in authored(tok, true) {
        let r = m.rollout(
            &[],
            &[],
            SlFacts::default(),
            &ex.observed,
            &[],
            80,
            false,
            false,
        );
        rows.push(serde_json::json!({"prompt":tok.decode(&ex.observed),"generated":tok.decode(&r.tokens),"tokens":r.tokens,"stopped":r.stopped,"exact":r.actions==ex.actions,"kind":if tok.decode(&ex.observed).contains("Rust"){"rust"}else{"dialogue"}}));
    }
    serde_json::json!(rows)
}
fn score(m: &TlModel, ws: &[ProseWindow]) -> serde_json::Value {
    let mut groups = std::collections::BTreeMap::<usize, (f64, usize)>::new();
    for w in ws {
        let s = m.score_example(&prose_example(w));
        let e = groups.entry(w.doc).or_default();
        e.0 += s.bits_generate;
        e.1 += s.generate_targets;
    }
    let bits = groups.values().map(|v| v.0).sum::<f64>();
    let n = groups.values().map(|v| v.1).sum::<usize>();
    serde_json::json!({"bits":bits/n as f64,"targets":n,"documents":groups})
}
#[allow(clippy::too_many_arguments)]
pub fn run(
    root: &Path,
    m: &TlModel,
    repo_fit: &[ProseWindow],
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let mode = std::env::var("UOR_CAUSAL_MODE").unwrap_or_else(|_| "train".into());
    if !["train", "geometry", "cost", "evaluate"].contains(&mode.as_str()) {
        return Err("unknown causal mode".into());
    }
    write_json(
        root,
        "causal-source.json",
        &serde_json::json!({
            "mode":mode,"source_sha256":hash(include_bytes!("causal_continuation.rs")),
            "model_sha256":hash(&m.to_bytes()),"fixture_seed":0x20260924u64
        }),
    )?;
    if mode == "geometry" {
        return geometric(root);
    }
    if mode == "cost" {
        return cost(root, m, dev, tok);
    }
    let dir =
        PathBuf::from(std::env::var("UOR_CAUSAL_DATA").map_err(|_| "UOR_CAUSAL_DATA required")?);
    if mode == "evaluate" {
        let (ws, manifest) = sources(tok, &dir.join("final"), 16)?;
        return Ok(
            serde_json::json!({"final_sources":manifest,"score":score(m,&ws),"repository":score(m,dev),"panels":panels(m,&read_words(tok)?),"generation":generated(m,tok),"scope":"Final source-family measurement; no adaptation or coefficient selection on this population."}),
        );
    }
    let (broad, broad_meta) = sources(tok, &dir.join("fit"), 256)?;
    let (tune, tune_meta) = sources(tok, &dir.join("tune"), 16)?;
    let w = read_words(tok)?;
    let (ground, _, _) = authored_world(&w);
    let dialogue = authored(tok, false);
    let arm = std::env::var("UOR_CAUSAL_ARM").unwrap_or_else(|_| "intervened".into());
    if arm != "intervened" && arm != "ordinary" {
        return Err("unknown feedback arm".into());
    }
    let steps: u64 = std::env::var("UOR_CAUSAL_STEPS")
        .unwrap_or_else(|_| "256".into())
        .parse()
        .map_err(|_| "invalid steps")?;
    if !(2..=2048).contains(&steps) {
        return Err("step budget outside2..2048".into());
    }
    let metadata = serde_json::json!({"parent_model_sha256":hash(&m.to_bytes()),"broad_fit":broad_meta,"tune":tune_meta,"repo_windows":repo_fit.len(),"arm":arm,"steps":steps,"sampling":"three repository, three broad, one dialogue, two grounded, two crossed-copy examples per update; all token windows64 with8 observed prefix","feedback_weight":4.0,"crossed_fit_cells":12,"withheld_crossed_cells":"feedback_index == (source_index+2)%4, all four opposite-class combinations","source":hash(include_bytes!("causal_continuation.rs"))});
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(&metadata).map_err(|e| e.to_string())?);
    for win in repo_fit {
        for t in &win.tokens {
            digest.update(t.to_le_bytes());
        }
    }
    let data: [u8; 32] = digest.finalize().into();
    let token_hash: [u8; 32] = Sha256::digest(
        &std::fs::read(
            "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json",
        )
        .map_err(|e| e.to_string())?,
    )
    .into();
    write_json(root, "training-plan.json", &metadata)?;
    let mut cursor = TrainingCursor {
        data_sha256: data,
        tokenizer_sha256: token_hash,
        rng_state: 20260924,
        span_rng_state: 20260926,
        next_batch: 0,
        schedule_total: steps,
        lr_start_bits: 0.01f64.to_bits(),
        lr_end_bits: 0.001f64.to_bits(),
    };
    let mut checkpoint_origin: Option<PathBuf> = None;
    let mut tr = TlTrainer::from_model_fresh_optimizer(
        m,
        TlTrainConfig {
            lr: 0.01,
            seed: 20260924,
            ..TlTrainConfig::default()
        },
    )?;
    if let Ok(path) = std::env::var("UOR_CAUSAL_CHECKPOINT") {
        checkpoint_origin = Some(PathBuf::from(&path));
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        (tr, cursor) = TlTrainer::from_checkpoint(&bytes, data, token_hash)?;
        if cursor.schedule_total != steps {
            return Err("checkpoint schedule mismatch".into());
        }
    }
    let first = cursor.next_batch;
    let started = Instant::now();
    let mut records = Vec::new();
    let mut best: Option<(f64, Vec<u8>, u64)> = None;
    if let Some(path) = &checkpoint_origin {
        let parent = path.parent().ok_or("checkpoint directory missing")?;
        let control: serde_json::Value = serde_json::from_slice(
            &std::fs::read(parent.join(format!("control-{}.json", cursor.next_batch)))
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if control["checkpoint_sha256"] != hash(&std::fs::read(path).map_err(|e| e.to_string())?) {
            return Err("checkpoint/control identity mismatch".into());
        }
        records = serde_json::from_value(control["records"].clone()).map_err(|e| e.to_string())?;
        if let Some(step) = control["best_step"].as_u64() {
            if step > cursor.next_batch {
                return Err("future selected state in checkpoint".into());
            }
            let bytes = std::fs::read(parent.join(format!("model-{step}.tlx")))
                .map_err(|e| e.to_string())?;
            if control["best_sha256"] != hash(&bytes) {
                return Err("selected state hash mismatch".into());
            }
            TlModel::from_bytes(&bytes)?;
            write_checked(root, &format!("model-{step}.tlx"), &bytes)?;
            best = Some((
                control["best_loss"]
                    .as_f64()
                    .ok_or("missing selected loss")?,
                bytes,
                step,
            ));
        }
    }

    let stop: u64 = std::env::var("UOR_CAUSAL_STOP_AFTER")
        .unwrap_or_else(|_| steps.to_string())
        .parse()
        .map_err(|_| "invalid stop-after")?;
    if stop > steps || stop < first {
        return Err("invalid checkpoint stop boundary".into());
    }
    while cursor.next_batch < stop {
        let mut batch = Vec::new();
        let mut feedback = Vec::new();
        for _ in 0..3 {
            batch.push(prose_example(
                &repo_fit[next(&mut cursor.rng_state) as usize % repo_fit.len()],
            ));
            feedback.push(None);
        }
        for _ in 0..3 {
            batch.push(prose_example(
                &broad[next(&mut cursor.rng_state) as usize % broad.len()],
            ));
            feedback.push(None);
        }
        batch.push(dialogue[next(&mut cursor.rng_state) as usize % dialogue.len()].clone());
        feedback.push(None);
        for _ in 0..2 {
            batch.push(ground[next(&mut cursor.rng_state) as usize % ground.len()].example(4., 0));
            feedback.push(None);
        }
        for _ in 0..2 {
            let mut s = next(&mut cursor.rng_state) as usize % 4;
            let mut f = next(&mut cursor.rng_state) as usize % 4;
            while f == (s + 2) % 4 {
                s = next(&mut cursor.rng_state) as usize % 4;
                f = next(&mut cursor.rng_state) as usize % 4;
            }
            let actual = if arm == "intervened" { f } else { s };
            batch.push(crossed_case(&w, s, actual));
            feedback.push(if arm == "intervened" {
                Some(vec![w.values[f]])
            } else {
                None
            });
        }
        let x = cursor.next_batch as f64 / (steps - 1) as f64;
        tr.tcfg.lr = f64::from_bits(cursor.lr_start_bits) * (1. - x)
            + f64::from_bits(cursor.lr_end_bits) * x;
        let report = tr.train_intervened_batch(&batch, &feedback)?;
        cursor.next_batch += 1;
        if cursor.next_batch % 32 == 0 {
            println!(
                "CAUSAL arm={} step={}/{} loss={:.4} elapsed={:.2}",
                arm,
                cursor.next_batch,
                steps,
                report.bits_per_target(),
                started.elapsed().as_secs_f64()
            );
        }
        if cursor.next_batch % 64 == 0 || cursor.next_batch == stop {
            let model = tr.model()?;
            let bytes = model.to_bytes();
            let checkpoint = tr.checkpoint(&cursor)?;
            write_checked(
                root,
                &format!("checkpoint-{}.tlk", cursor.next_batch),
                &checkpoint,
            )?;
            write_checked(root, &format!("model-{}.tlx", cursor.next_batch), &bytes)?;
            let p = panels(&model, &w);
            let tune_loss = evaluate_single(&model, &tune);
            let retained = p["temporal"]["correct"] == 32
                && p["class"]["correct"] == 4
                && p["heldout"]["correct"] == 3;
            let causal = arm != "intervened" || p["feedback"]["correct"] == 16;
            let eligible = retained && causal;
            if eligible && best.as_ref().map_or(true, |b| tune_loss < b.0) {
                best = Some((tune_loss, bytes, cursor.next_batch));
            }
            records.push(serde_json::json!({"step":cursor.next_batch,"tune_bits":tune_loss,"repo_bits":evaluate_single(&model,dev),"panels":p,"selection_eligible":eligible,"checkpoint_sha256":hash(&checkpoint)}));
            write_json(
                root,
                &format!("control-{}.json", cursor.next_batch),
                &serde_json::json!({"checkpoint_sha256":hash(&checkpoint),"records":records,"best_step":best.as_ref().map(|b|b.2),"best_loss":best.as_ref().map(|b|b.0),"best_sha256":best.as_ref().map(|b|hash(&b.1))}),
            )?;
        }
    }
    let final_model = tr.model()?;
    let bytes = final_model.to_bytes();
    write_checked(root, "final.tlx", &bytes)?;
    write_checked(root, "final.tlk", &tr.checkpoint(&cursor)?)?;
    let selection = if let Some((loss, bytes, step)) = best {
        write_checked(root, "selected.tlx", &bytes)?;
        serde_json::json!({"selected":true,"step":step,"tune_bits":loss,"sha256":hash(&bytes)})
    } else {
        serde_json::json!({"selected":false,"reason":"No candidate met the declared grounding and causal feedback gates; retain every checkpoint, do not promote."})
    };
    Ok(
        serde_json::json!({"training":metadata,"resume_started_at":first,"completed_steps":cursor.next_batch,"seconds":started.elapsed().as_secs_f64(),"records":records,"selection":selection,"final_panels":panels(&final_model,&w),"final_tune":score(&final_model,&tune),"final_repository":score(&final_model,dev),"generation":generated(&final_model,tok),"data_sha256":data,"final_sha256":hash(&bytes),"scope":"Finite intervention supervision and bounded mixed-source fitting. Legacy held-out grounding is now exposed regression data, not a fresh acceptance set."}),
    )
}
fn l1(a: [i32; 4], b: [i32; 4]) -> u64 {
    a.iter()
        .zip(b)
        .map(|(&x, y)| (i64::from(x) - i64::from(y)).unsigned_abs())
        .sum()
}
#[derive(Clone, Copy)]
struct Perm {
    index: [usize; 4],
    sign: u8,
}
fn perm(p: Perm, x: [i32; 4]) -> [i32; 4] {
    std::array::from_fn(|i| {
        if p.sign & (1 << i) != 0 {
            -x[p.index[i]]
        } else {
            x[p.index[i]]
        }
    })
}
fn permutations() -> Vec<Perm> {
    let mut out = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    if [a, b, c, d]
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == 4
                    {
                        for sign in 0..16 {
                            out.push(Perm {
                                index: [a, b, c, d],
                                sign,
                            });
                        }
                    }
                }
            }
        }
    }
    out
}
fn geometric(root: &Path) -> Result<serde_json::Value, String> {
    let truth = [6u8, 3, 0, 5, 2, 7, 1, 4];
    let mut rng = 0x20260924u64;
    let mut training = Vec::new();
    for relation in 0..8 {
        for _ in 0..8 {
            let keys: Vec<[i32; 4]> = (0..8)
                .map(|_| std::array::from_fn(|_| (next(&mut rng) % 63) as i32 - 31))
                .collect();
            let target = (next(&mut rng) % 8) as usize;
            let query = ht::apply(truth[relation], keys[target])?;
            training.push(ra::ActionExample {
                relation,
                query,
                keys,
                target,
            });
        }
    }
    let model = ra::RelativeActionModel::fit(8, &training)?;
    let encoded = model.to_bytes();
    let loaded = ra::RelativeActionModel::from_bytes(&encoded)?;
    if loaded != model {
        return Err("relative model reload differs".into());
    }
    write_checked(root, "relative.q8l", &encoded)?;
    let candidates = permutations();
    let mut ordinary = Vec::new();
    for relation in 0..8 {
        let mut best = (u64::MAX, 0);
        for (i, &p) in candidates.iter().enumerate() {
            let mut loss = 0;
            for ex in training.iter().filter(|e| e.relation == relation) {
                let correct = l1(ex.query, perm(p, ex.keys[ex.target]));
                let other = ex
                    .keys
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != ex.target)
                    .map(|(_, k)| l1(ex.query, perm(p, *k)))
                    .min()
                    .ok_or("no competing key")?;
                loss += (correct + 1).saturating_sub(other);
            }
            if loss < best.0 {
                best = (loss, i);
            }
        }
        ordinary.push(candidates[best.1]);
    }
    let mut counts = [0usize; 4];
    let mut rows = Vec::new();
    for test in 0..1024 {
        let keys: Vec<[i32; 4]> = (0..8)
            .map(|_| std::array::from_fn(|_| (next(&mut rng) % 2047) as i32 - 1023))
            .collect();
        let target = (next(&mut rng) % 8) as usize;
        let length = 1 + test % 6;
        let path: Vec<usize> = (0..length).map(|_| (next(&mut rng) % 8) as usize).collect();
        let mut query = keys[target];
        for &r in &path {
            query = ht::apply(truth[r], query)?;
        }
        let got = loaded.select(&path, query, &keys)?;
        let generic = (0..keys.len())
            .min_by_key(|&i| {
                let mut v = keys[i];
                for &r in &path {
                    v = perm(ordinary[r], v);
                }
                l1(query, v)
            })
            .ok_or("empty keys")?;
        let disabled = (0..keys.len())
            .min_by_key(|&i| l1(query, keys[i]))
            .ok_or("empty keys")?;
        let mut reversed = path.clone();
        reversed.reverse();
        let rev = loaded.select(&reversed, query, &keys)?;
        for (k, p) in [got, generic, disabled, rev].into_iter().enumerate() {
            counts[k] += usize::from(p == target);
        }
        rows.push(serde_json::json!({"length":length,"target":target,"q8":got,"ordinary":generic,"disabled":disabled,"reversed":rev}));
    }
    write_json(root, "geometric-rows.json", &serde_json::json!(rows))?;
    Ok(
        serde_json::json!({"fit_examples":training.len(),"test_examples":1024,"correct":counts,"arms":["learned_q8","learned_signed_permutation","disabled_transport","reversed_operations"],"learned_actions":model.actions(),"q8_artifact_bytes":encoded.len(),"ordinary_parameter_bytes":ordinary.len()*5,"training_search_candidates_per_relation":[8,384],"source_query_vectors":"Test magnitudes and compositions absent from one-step fit; exact payload chosen by selected occurrence index","scope":"Synthetic typed relation/composition task, not language attention. Ordinary arm has same information, more candidate operators and a larger explicit representation. Matching it is competence/compression, not geometric predictive superiority."}),
    )
}
fn generate_with_plan(
    m: &TlModel,
    plan: Option<&TlReadPlan>,
    prompt: &[u32],
    limit: usize,
) -> Result<Vec<u32>, String> {
    let meaning = vec![0; m.h_dim];
    let facts = m.typed_block(&[], &[], SlFacts::default());
    let mut h = m.init_state(&meaning, &facts);
    for &t in prompt {
        h = m.transition(&h, TL_EV_OBSERVE, Some(t), &meaning, &facts);
    }
    let mut event = TL_EV_OBSERVE;
    let mut scratch = vec![0; 2 * m.h_dim + TL_F_DIM + TL_EVENTS];
    let mut out = vec![0; m.vocab + 2];
    let mut tokens = Vec::new();
    for _ in 0..limit {
        if let Some(p) = plan {
            p.score_into(&h, &meaning, &facts, event, &mut scratch, &mut out)?;
        } else {
            out = m.readout(&h, &meaning, &facts, event);
        }
        let mut best = 0;
        for i in (1..m.vocab).chain(std::iter::once(m.stop_row())) {
            if out[i] > out[best] {
                best = i;
            }
        }
        if best == m.stop_row() {
            break;
        }
        tokens.push(best as u32);
        h = m.transition(&h, TL_EV_GENERATE, Some(best as u32), &meaning, &facts);
        event = TL_EV_GENERATE;
    }
    Ok(tokens)
}
fn cost(
    root: &Path,
    m: &TlModel,
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let cold = Instant::now();
    let plan = TlReadPlan::compile(m)?;
    let compile_seconds = cold.elapsed().as_secs_f64();
    let mut scratch = vec![0; plan.workspace_len()];
    let mut out = vec![0; plan.output_len()];
    let meaning = vec![0; m.h_dim];
    let facts = m.typed_block(&[], &[], SlFacts::default());
    let mut compared = 0;
    for w in dev.iter().take(16) {
        let mut h = m.init_state(&meaning, &facts);
        for (i, &t) in w.tokens.iter().enumerate() {
            let event = if i < 8 { TL_EV_OBSERVE } else { TL_EV_GENERATE };
            h = m.transition(&h, event, Some(t), &meaning, &facts);
            plan.score_into(&h, &meaning, &facts, event, &mut scratch, &mut out)?;
            if out != m.readout(&h, &meaning, &facts, event) {
                return Err("sparse readout differs from original".into());
            }
            compared += out.len();
        }
    }
    let prompts: Vec<_> = dev
        .iter()
        .take(8)
        .map(|w| w.tokens[..16.min(w.tokens.len())].to_vec())
        .collect();
    let mut times = [Vec::new(), Vec::new()];
    let mut completions = Vec::new();
    let mut checksum = 0u64;
    for round in 0..9 {
        for k in 0..2 {
            let arm = (round + k) % 2;
            let start = Instant::now();
            let mut results = Vec::new();
            let mut n = 0;
            for prompt in &prompts {
                let t =
                    generate_with_plan(m, if arm == 1 { Some(&plan) } else { None }, prompt, 48)?;
                n += t.len();
                for &v in &t {
                    checksum = checksum.wrapping_add(u64::from(v));
                }
                results.push(t);
            }
            let elapsed = start.elapsed().as_secs_f64();
            times[arm].push(serde_json::json!({"seconds":elapsed,"generated_tokens":n,"prompts":prompts.len(),"microseconds_per_token":elapsed*1e6/n.max(1)as f64}));
            if round == 0 && arm == 0 {
                completions = results;
            } else if results != completions {
                return Err("sparse full-generation output mismatch".into());
            }
        }
    }
    write_json(root,"cost-completions.json",&serde_json::json!(prompts.iter().zip(&completions).map(|(p,c)|serde_json::json!({"prompt":tok.decode(p),"completion":tok.decode(c),"tokens":c})).collect::<Vec<_>>()))?;
    Ok(
        serde_json::json!({"all_row_comparisons":compared,"all_rows_equal":true,"all_generated_sequences_equal":true,"native_readout_coefficients":m.wo.rows*m.wo.cols,"compiled_nonzeros":plan.nonzeros(),"compiled_plan_bytes":plan.bytes(),"compile_seconds":compile_seconds,"interleaved_rounds":times,"checksum":std::hint::black_box(checksum),"scope":"Full prompt ingestion plus greedy generation, including allocations and recurrent updates; model loading and one-time read-plan compilation reported separately. No energy or whole-model zero-allocation claim."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoint_and_intervention_public_contract() {
        let mut cfg = TlConfig::new(8);
        cfg.h_dim = 8;
        let ex = TlExample {
            sel: vec![1],
            res: vec![],
            facts: SlFacts {
                history: 2,
                ..SlFacts::default()
            },
            observed: vec![],
            actions: vec![TlAction::Copy, TlAction::Generate(3), TlAction::Stop],
            weight: 1.,
            doc: 0,
            grounded: true,
            terminal_stop: true,
        };
        let cursor = TrainingCursor {
            data_sha256: [1; 32],
            tokenizer_sha256: [2; 32],
            rng_state: 13,
            span_rng_state: 20260926,
            next_batch: 0,
            schedule_total: 8,
            lr_start_bits: 0.01f64.to_bits(),
            lr_end_bits: 0.001f64.to_bits(),
        };
        let mut a = TlTrainer::new(cfg, TlTrainConfig::default()).unwrap();
        let mut b = TlTrainer::new(cfg, TlTrainConfig::default()).unwrap();
        assert!(a
            .train_intervened_batch(&[ex.clone()], &[Some(vec![])])
            .is_err());
        assert_eq!(a.step, 0);
        a.train_intervened_batch(&[ex.clone()], &[Some(vec![1])])
            .unwrap();
        b.train_intervened_batch(&[ex.clone()], &[Some(vec![2])])
            .unwrap();
        assert_ne!(
            a.checkpoint(&cursor).unwrap(),
            b.checkpoint(&cursor).unwrap()
        );
        let bytes = a.checkpoint(&cursor).unwrap();
        let (mut resumed, c) = TlTrainer::from_checkpoint(&bytes, [1; 32], [2; 32]).unwrap();
        assert_eq!(cursor, c);
        for _ in 0..3 {
            a.train_batch(&[ex.clone()]);
            resumed.train_batch(&[ex.clone()]);
        }
        assert_eq!(
            a.checkpoint(&cursor).unwrap(),
            resumed.checkpoint(&cursor).unwrap()
        );
    }
    #[test]
    fn read_plan_public_contract_nonzero() {
        let mut cfg = TlConfig::new(8);
        cfg.h_dim = 8;
        let mut m = TlTrainer::new(cfg, TlTrainConfig::default())
            .unwrap()
            .model()
            .unwrap();
        m.wo.packed.fill(0x19);
        let p = TlReadPlan::compile(&m).unwrap();
        let h = vec![2; 8];
        let f = m.typed_block(&[], &[], SlFacts::default());
        let mut scratch = vec![0; p.workspace_len()];
        let mut out = vec![0; p.output_len()];
        p.score_into(&h, &h, &f, 2, &mut scratch, &mut out).unwrap();
        assert_eq!(out, m.readout(&h, &h, &f, 2));
    }
}
