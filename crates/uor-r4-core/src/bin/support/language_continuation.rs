//! Bounded language-conditioned source selection, span supervision and complete native execution.
use super::*;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::learner::transferable_lexical::TrainingCursor;
use uor_r4_core::native_geometric::learner::{
    hamilton_transport as ht, language_transport as lt, tl_execution as fast,
};
fn hash(b: &[u8]) -> String {
    super::sha256_hex(&Sha256::digest(b))
}
fn json(root: &Path, name: &str, v: &serde_json::Value) -> Result<(), String> {
    write_checked(
        root,
        name,
        &serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?,
    )
}
fn next(x: &mut u64) -> u64 {
    *x ^= *x << 13;
    *x ^= *x >> 7;
    *x ^= *x << 17;
    *x
}
const ROLES: [&str; 4] = ["current", "previous", "initial", "operand"];
const OPS: [u8; 4] = [0, 2, 4, 6];
fn inverse(a: u8) -> u8 {
    if a < 2 {
        a
    } else {
        a ^ 1
    }
}
fn geom_example(
    tok: &HfBpeTokenizer,
    role: usize,
    variant: usize,
    seed: u64,
) -> Result<lt::QueryExample, String> {
    let q = [
        (seed % 47) as i32 + 3,
        (seed % 67) as i32 + 5,
        -((seed % 89) as i32 + 7),
        (seed % 113) as i32 + 11,
    ];
    let mut keys = (0..4)
        .map(|j| ht::apply(inverse(OPS[j]), q))
        .collect::<Result<Vec<_>, _>>()?;
    let shift = seed as usize % 4;
    keys.rotate_left(shift);
    let text = match variant % 4 {
        0 => format!("Read the {} value.", ROLES[role]),
        1 => format!("Retrieve the {} record.", ROLES[role]),
        2 => format!("Give {} data.", ROLES[role]),
        _ => format!("Please retrieve the {} value now.", ROLES[role]),
    };
    Ok(lt::QueryExample {
        tokens: tok.encode(&text),
        query: q,
        keys,
        target: (role + 4 - shift) % 4,
    })
}
fn geometry(
    root: &Path,
    tok: &HfBpeTokenizer,
    tok_hash: [u8; 32],
) -> Result<lt::LanguageTransport, String> {
    let mut fit = Vec::new();
    for role in 0..4 {
        for variant in 0..3 {
            for n in 1..=12 {
                fit.push(geom_example(tok, role, variant, n)?);
            }
        }
    }
    let model = lt::LanguageTransport::fit(VOCAB, tok_hash, &fit, 128)?;
    let bytes = model.to_bytes();
    write_checked(root, "language-selector.lqt", &bytes)?;
    let model = lt::LanguageTransport::from_bytes(&bytes, tok_hash)?;
    let mut rows = Vec::new();
    for role in 0..4 {
        for n in 101..=164 {
            let ex = geom_example(tok, role, 3, n)?;
            let owner = n % 13 + 1;
            let mut keys = ex
                .keys
                .iter()
                .enumerate()
                .map(|(i, &vector)| lt::OwnedKey {
                    owner,
                    occurrence: n * 16 + i as u64,
                    vector,
                })
                .collect::<Vec<_>>();
            keys.push(lt::OwnedKey {
                owner: owner + 100,
                occurrence: n * 16 + 8,
                vector: ex.query,
            });
            let selected = model.select_owned(&ex.tokens, ex.query, owner, &keys)?;
            let action = model.action(&ex.tokens)?;
            let wrong = geom_example(tok, (role + 1) % 4, 3, n)?;
            let shuffled = model.select_owned(&wrong.tokens, ex.query, owner, &keys)?;
            let disabled = lt::LanguageTransport::select_action(0, ex.query, owner, &keys)?;
            rows.push(serde_json::json!({"role":ROLES[role],"correct":selected==ex.target,"selected":selected,"target":ex.target,"action":action,"wrong_instruction_correct":shuffled==ex.target,"disabled_correct":disabled==ex.target,"owner_absence_rejected":model.select_owned(&ex.tokens,ex.query,owner+200,&keys).is_err()}));
        }
    }
    let count = |key: &str| rows.iter().filter(|r| r[key] == true).count();
    json(
        root,
        "geometry.json",
        &serde_json::json!({"fit_examples":fit.len(),"test_examples":rows.len(),"correct":count("correct"),"wrong_instruction_correct":count("wrong_instruction_correct"),"disabled_correct":count("disabled_correct"),"absent_owner_rejected":count("owner_absence_rejected"),"bytes":bytes.len(),"features":model.feature_count(),"rows":rows,"scope":"Typed vector keys and owner IDs are supplied exact metadata. Only the instruction-to-operation map is learned from selected-key examples. Not arbitrary semantic parsing or unique geometric advantage."}),
    )?;
    Ok(model)
}
fn instruction(tok: &HfBpeTokenizer, value: &str, rust: bool, variant: usize) -> TlExample {
    let prompt = if rust {
        if variant == 0 {
            "Write Rust that prints the selected value.\nRust:"
        } else {
            "Produce a Rust program for the selected value.\nRust:"
        }
    } else if variant == 0 {
        "Repeat the selected value exactly.\nAnswer:"
    } else {
        "Return the selected value.\nAnswer:"
    };
    let sel = tok.encode(value);
    let mut actions = Vec::new();
    if rust {
        actions.extend(
            tok.encode("fn main(){print!(\"")
                .into_iter()
                .map(TlAction::Generate),
        );
    }
    actions.extend(sel.iter().map(|_| TlAction::Copy));
    if rust {
        actions.extend(tok.encode("\");}").into_iter().map(TlAction::Generate));
    }
    actions.push(TlAction::Stop);
    TlExample {
        sel,
        res: vec![],
        facts: SlFacts::default(),
        observed: tok.encode(prompt),
        actions,
        weight: 2.,
        doc: 0,
        grounded: true,
        terminal_stop: true,
    }
}
fn span_case(w: &Words, source: Vec<u32>, feed: &[u32]) -> TlExample {
    let last = *feed.last().unwrap_or(&w.values[0]);
    let label = if last == w.values[0] || last == w.values[1] {
        w.class_a
    } else {
        w.class_b
    };
    let mut actions = source.iter().map(|_| TlAction::Copy).collect::<Vec<_>>();
    actions.push(TlAction::Generate(label));
    actions.push(TlAction::Stop);
    TlExample {
        sel: source,
        res: vec![],
        facts: SlFacts {
            history: 2,
            ..SlFacts::default()
        },
        observed: vec![],
        actions,
        weight: 4.,
        doc: 0,
        grounded: true,
        terminal_stop: true,
    }
}
fn retained(m: &TlModel, w: &Words) -> serde_json::Value {
    super::causal_continuation::panels(m, w)
}
fn outputs(
    m: &TlModel,
    tok: &HfBpeTokenizer,
    values: &[String],
    variant: usize,
) -> Result<serde_json::Value, String> {
    let exec = fast::Execution::compile(m)?;
    let mut work = exec.workspace();
    let mut tokens = vec![0; 96];
    let mut acts = vec![TlAction::Stop; 96];
    let mut rows = Vec::new();
    for value in values {
        for rust in [false, true] {
            let ex = instruction(tok, value, rust, variant);
            let r = exec.run(
                &ex.sel,
                &ex.res,
                ex.facts,
                &ex.observed,
                &ex.sel,
                false,
                false,
                &mut work,
                &mut tokens,
                &mut acts,
            )?;
            rows.push(serde_json::json!({"value":value,"kind":if rust{"rust"}else{"echo"},"prompt":tok.decode(&ex.observed),"text":tok.decode(&tokens[..r.tokens]),"tokens":&tokens[..r.tokens],"actions":acts[..r.actions].iter().map(action_name).collect::<Vec<_>>(),"expected_actions":ex.actions.iter().map(action_name).collect::<Vec<_>>(),"exact_actions":acts[..r.actions]==ex.actions,"stopped":r.stopped,"copied":r.copied}));
        }
    }
    Ok(serde_json::json!(rows))
}
fn span_panel(m: &TlModel, w: &Words) -> serde_json::Value {
    span_lengths(m, w, &[1, 2, 3, 5, 8])
}
fn span_lengths(m: &TlModel, w: &Words, lengths: &[usize]) -> serde_json::Value {
    let mut rows = Vec::new();
    for &len in lengths {
        for s in 0..4 {
            for f in 0..4 {
                let src = (0..len).map(|i| w.values[(i + s) % 4]).collect::<Vec<_>>();
                let mut feed = src.clone();
                feed[len - 1] = w.values[f];
                let mm = m.content_feature(&src, &[]);
                let facts = m.typed_block(
                    &src,
                    &[],
                    SlFacts {
                        history: 2,
                        ..SlFacts::default()
                    },
                );
                let mut h = m.init_state(&mm, &facts);
                for &t in &feed {
                    h = m.transition(&h, TL_EV_COPY, Some(t), &mm, &facts);
                }
                let action = m.decide(&h, &mm, &facts, TL_EV_COPY, false);
                let expected = if f < 2 { w.class_a } else { w.class_b };
                rows.push(serde_json::json!({"length":len,"source":s,"feedback":f,"correct":action==TlAction::Generate(expected),"prediction":action_name(&action)}));
            }
        }
    }
    serde_json::json!({"correct":rows.iter().filter(|r|r["correct"]==true).count(),"total":rows.len(),"by_length":lengths.iter().map(|&n|serde_json::json!({"length":n,"correct":rows.iter().filter(|r|r["length"]==n&&r["correct"]==true).count(),"total":16})).collect::<Vec<_>>(),"rows":rows,"scope":"Forced causal span-feedback diagnostic; no permission to copy unowned values. Length5,8 excluded from fit."})
}
pub fn run(
    root: &Path,
    parent: &TlModel,
    repo: &[ProseWindow],
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let mode = std::env::var("UOR_LANGUAGE_MODE").unwrap_or_else(|_| "train".into());
    let token_bytes = std::fs::read(
        "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct/tokenizer.json",
    )
    .map_err(|e| e.to_string())?;
    let th: [u8; 32] = Sha256::digest(&token_bytes).into();
    let w = read_words(tok)?;
    if mode == "geometry" {
        let model = geometry(root, tok, th)?;
        return Ok(
            serde_json::json!({"model_sha256":hash(&model.to_bytes()),"bytes":model.to_bytes().len()}),
        );
    }
    if mode == "cost" {
        return execution_audit(root, parent, dev, tok);
    }
    if mode == "integrate" {
        return integrate(root, parent, tok, th);
    }
    if mode == "evaluate" {
        let values = ["37", "42", "105", "208", "317", "512", "1024", "2048"].map(str::to_string);
        let out = outputs(parent, tok, &values, 1)?;
        json(root, "generated.json", &out)?;
        return Ok(
            serde_json::json!({"panels":retained(parent,&w),"span":span_panel(parent,&w),"fresh_span":span_lengths(parent,&w,&[13,21,34]),"repository_bits":evaluate_single(parent,dev),"generated":out}),
        );
    }
    if mode != "train" {
        return Err("unknown language mode".into());
    }
    let arm = std::env::var("UOR_LANGUAGE_ARM").unwrap_or_else(|_| "moments".into());
    if arm != "moments" && arm != "reset" {
        return Err("invalid optimization arm".into());
    }
    let total: u64 = std::env::var("UOR_LANGUAGE_STEPS")
        .unwrap_or_else(|_| "512".into())
        .parse()
        .map_err(|_| "steps")?;
    if !(128..=2048).contains(&total) {
        return Err("training phase bound".into());
    }
    let max_span: usize = std::env::var("UOR_LANGUAGE_MAX_SPAN")
        .unwrap_or_else(|_| "3".into())
        .parse()
        .map_err(|_| "span length")?;
    if !(1..=32).contains(&max_span) {
        return Err("span fit bound".into());
    }
    let broad_dir =
        PathBuf::from("/Users/casey.allard/uor-r4-investigations/causal-20260924T000007/data");
    let (broad, broad_meta) =
        super::causal_continuation::sources(tok, &broad_dir.join("fit"), 256)?;
    let (tune, tune_meta) = super::causal_continuation::sources(tok, &broad_dir.join("tune"), 16)?;
    let (train_ground, _, _) = authored_world(&w);
    let instructions = (0..32)
        .flat_map(|n| [false, true].map(move |r| (n, r)))
        .map(|(n, r)| instruction(tok, &n.to_string(), r, 0))
        .collect::<Vec<_>>();
    let old = std::fs::read(
        "/Users/casey.allard/uor-r4/.uor-models/causal-continuation-2026-09-24/final.tlk",
    )
    .map_err(|e| e.to_string())?;
    let n = u32::from_le_bytes(
        old.get(4..8)
            .ok_or("old checkpoint length")?
            .try_into()
            .map_err(|_| "header")?,
    ) as usize;
    let head: serde_json::Value =
        serde_json::from_slice(old.get(8..8 + n).ok_or("old checkpoint header")?)
            .map_err(|e| e.to_string())?;
    let old_cursor: TrainingCursor =
        serde_json::from_value(head["cursor"].clone()).map_err(|e| e.to_string())?;
    let (mut trainer, _) =
        TlTrainer::from_checkpoint(&old, old_cursor.data_sha256, old_cursor.tokenizer_sha256)?;
    if trainer.model()?.to_bytes() != parent.to_bytes() {
        return Err("checkpoint is not the requested served parent".into());
    }
    let start_step = if arm == "moments" { trainer.step } else { 0 };
    if arm == "reset" {
        trainer = TlTrainer::from_model_fresh_optimizer(
            parent,
            TlTrainConfig {
                lr: 0.01,
                seed: 20260925,
                ..TlTrainConfig::default()
            },
        )?;
    }
    let metadata = serde_json::json!({"parent_model":hash(&parent.to_bytes()),"old_checkpoint":hash(&old),"arm":arm,"steps":total,"old_optimizer_step":start_step,"phase_lr":[0.01,0.001],"seed":20260925u64,"fit_numbers":"0..31","validation_numbers":[32,33,34,35],"gradient_span_lengths":(1..=max_span).collect::<Vec<_>>(),"broad_fit":broad_meta,"broad_tune":tune_meta,"sampling":"2 repo,2 broad,4 instruction,2 legacy grounded,2 crossed span examples","source":hash(include_bytes!("language_continuation.rs"))});
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(&metadata).map_err(|e| e.to_string())?);
    for win in repo {
        for t in &win.tokens {
            digest.update(t.to_le_bytes());
        }
    }
    let data: [u8; 32] = digest.finalize().into();
    json(root, "phase-plan.json", &metadata)?;
    let mut cursor = TrainingCursor {
        data_sha256: data,
        tokenizer_sha256: th,
        rng_state: 20260925,
        next_batch: 0,
        schedule_total: total,
        lr_start_bits: 0.01f64.to_bits(),
        lr_end_bits: 0.001f64.to_bits(),
    };
    if let Ok(cp) = std::env::var("UOR_LANGUAGE_CHECKPOINT") {
        let raw = std::fs::read(cp).map_err(|e| e.to_string())?;
        (trainer, cursor) = TlTrainer::from_checkpoint(&raw, data, th)?;
        if cursor.schedule_total != total {
            return Err("resumed phase mismatch".into());
        }
    }
    let stop: u64 = std::env::var("UOR_LANGUAGE_STOP")
        .unwrap_or_else(|_| total.to_string())
        .parse()
        .map_err(|_| "stop after")?;
    if stop > total || stop < cursor.next_batch {
        return Err("invalid phase stop".into());
    }
    let first = cursor.next_batch;
    let initial = retained(parent, &w);
    let timer = Instant::now();
    let mut records = Vec::new();
    while cursor.next_batch < stop {
        let mut batch = Vec::new();
        let mut feedback = Vec::new();
        for _ in 0..2 {
            batch.push(prose_example(
                &repo[next(&mut cursor.rng_state) as usize % repo.len()],
            ));
            feedback.push(None);
            batch.push(prose_example(
                &broad[next(&mut cursor.rng_state) as usize % broad.len()],
            ));
            feedback.push(None);
        }
        for _ in 0..4 {
            batch.push(
                instructions[next(&mut cursor.rng_state) as usize % instructions.len()].clone(),
            );
            feedback.push(None);
        }
        for _ in 0..2 {
            batch.push(
                train_ground[next(&mut cursor.rng_state) as usize % train_ground.len()]
                    .example(4., 0),
            );
            feedback.push(None);
        }
        for _ in 0..2 {
            let len = 1 + next(&mut cursor.rng_state) as usize % max_span;
            let src = (0..len)
                .map(|_| w.values[next(&mut cursor.rng_state) as usize % 4])
                .collect::<Vec<_>>();
            let feed = (0..len)
                .map(|_| w.values[next(&mut cursor.rng_state) as usize % 4])
                .collect::<Vec<_>>();
            batch.push(span_case(&w, src, &feed));
            feedback.push(Some(feed));
        }
        let x = cursor.next_batch as f64 / (total - 1) as f64;
        trainer.tcfg.lr = 0.01 * (1. - x) + 0.001 * x;
        let report = trainer.train_intervened_batch(&batch, &feedback)?;
        cursor.next_batch += 1;
        if cursor.next_batch % 32 == 0 {
            println!(
                "LANGUAGE arm={arm} batch={}/{} bits={:.4} seconds={:.2}",
                cursor.next_batch,
                total,
                report.bits_per_target(),
                timer.elapsed().as_secs_f64()
            );
        }
        if cursor.next_batch % 128 == 0 || cursor.next_batch == stop {
            let m = trainer.model()?;
            let bytes = m.to_bytes();
            write_checked(root, &format!("model-{}.tlx", cursor.next_batch), &bytes)?;
            write_checked(
                root,
                &format!("checkpoint-{}.tlk", cursor.next_batch),
                &trainer.checkpoint(&cursor)?,
            )?;
            let out = outputs(&m, tok, &["32", "33", "34", "35"].map(str::to_string), 0)?;
            records.push(serde_json::json!({"batch":cursor.next_batch,"optimizer_step":trainer.step,"model_sha256":hash(&bytes),"tune_bits":evaluate_single(&m,&tune),"repository_bits":evaluate_single(&m,dev),"panels":retained(&m,&w),"span":span_panel(&m,&w),"validation_outputs":out}));
        }
    }
    let model = trainer.model()?;
    write_checked(root, "final.tlx", &model.to_bytes())?;
    write_checked(root, "final.tlk", &trainer.checkpoint(&cursor)?)?;
    json(root, "records.json", &serde_json::json!(records))?;
    Ok(
        serde_json::json!({"metadata":metadata,"initial":initial,"resumed_batch":first,"completed":cursor.next_batch,"optimizer_step":trainer.step,"seconds":timer.elapsed().as_secs_f64(),"final_panels":retained(&model,&w),"span":span_panel(&model,&w),"tune_bits":evaluate_single(&model,&tune),"repository_bits":evaluate_single(&model,dev),"records":records,"final_sha256":hash(&model.to_bytes()),"scope":"Bounded instruction/span learning; no general dialogue, arbitrary coding or scoped-session qualification."}),
    )
}
fn execution_audit(
    root: &Path,
    m: &TlModel,
    dev: &[ProseWindow],
    tok: &HfBpeTokenizer,
) -> Result<serde_json::Value, String> {
    let p = fast::Execution::compile(m)?;
    let mut work = p.workspace();
    let mut ts = vec![0; 96];
    let mut acts = vec![TlAction::Stop; 96];
    let mut comparisons = 0;
    let w = read_words(tok)?;
    let (mut cases, _, held) = authored_world(&w);
    cases.extend(held);
    for c in &cases {
        for blind in [false, true] {
            for disabled in [false, true] {
                let native = m.rollout(&c.sel, &c.res, c.facts, &[], &c.sel, 96, blind, disabled);
                let result = p.run(
                    &c.sel,
                    &c.res,
                    c.facts,
                    &[],
                    &c.sel,
                    blind,
                    disabled,
                    &mut work,
                    &mut ts,
                    &mut acts,
                )?;
                if native.tokens!=ts[..result.tokens]||native.actions!=acts[..result.actions]||native.state_digest!=uor_r4_core::native_geometric::learner::transferable_lexical::state_digest(&work.state){return Err("compiled grounded rollout parity".into());}
                comparisons += 1;
            }
        }
    }
    let mut timings = [Vec::new(), Vec::new()];
    let mut checksum = 0u64;
    for round in 0..9 {
        for k in 0..2 {
            let arm = (round + k) % 2;
            let time = Instant::now();
            let mut generated = 0;
            for win in dev.iter().take(16) {
                let prompt = &win.tokens[..16.min(win.tokens.len())];
                if arm == 0 {
                    let r = m.rollout(&[], &[], SlFacts::default(), prompt, &[], 64, false, false);
                    generated += r.tokens.len();
                    checksum ^= r.state_digest;
                } else {
                    let r = p.run(
                        &[],
                        &[],
                        SlFacts::default(),
                        prompt,
                        &[],
                        false,
                        false,
                        &mut work,
                        &mut ts[..64],
                        &mut acts[..64],
                    )?;
                    generated += r.tokens;
                    checksum ^=
                        uor_r4_core::native_geometric::learner::transferable_lexical::state_digest(
                            &work.state,
                        );
                }
            }
            timings[arm].push(time.elapsed().as_secs_f64() * 1e6 / generated.max(1) as f64);
        }
    }
    let medians = timings.each_ref().map(|v| {
        let mut q = v.clone();
        q.sort_by(f64::total_cmp);
        q[4]
    });
    let report = serde_json::json!({"source_sha256":hash(&m.to_bytes()),"grounded_blind_disabled_rollouts_identical":comparisons,"rounds":timings,"median_us_per_token":medians,"ratio":medians[0]/medians[1],"plan_payload_bytes":p.logical_plan_bytes(),"checkpoint":checksum,"scope":"Equal-model complete pretokenized preparation plus decode. Model/plan/tokenization excluded; allocations instrumented separately. No energy or equal-quality incumbent claim."});
    json(root, "execution-cost.json", &report)?;
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_roles_are_resolved_by_the_inverse_action() {
        let q = [3, 5, -7, 11];
        for a in OPS {
            assert_eq!(ht::apply(a, ht::apply(inverse(a), q).unwrap()).unwrap(), q);
        }
    }
}
fn integrate(
    root: &Path,
    m: &TlModel,
    tok: &HfBpeTokenizer,
    th: [u8; 32],
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(
        std::env::var("UOR_LANGUAGE_SELECTOR").map_err(|_| "selector path required")?,
    );
    let parent = path.parent().ok_or("selector root")?;
    let errors = verify(parent).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err("selector report is not sealed and intact".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let selector = lt::LanguageTransport::from_bytes(&bytes, th)?;
    write_checked(root, "selector.lqt", &bytes)?;
    write_checked(root, "model.tlx", &m.to_bytes())?;
    let exec = fast::Execution::compile(m)?;
    let mut work = exec.workspace();
    let mut tokens = vec![0; 128];
    let mut acts = vec![TlAction::Stop; 128];
    let values = ["37", "42", "105", "208", "317", "512", "1024", "2048"];
    let mut rows = Vec::new();
    let mut missing = 0;
    let mut absent = 0;
    let mut ordinary_equal = 0;
    for world in 0..8 {
        let owner = 1000 + world as u64;
        for role in 0..4 {
            let ex = geom_example(tok, role, 3, 301 + world as u64)?;
            let shift = (301 + world) % 4;
            let keys = ex
                .keys
                .iter()
                .enumerate()
                .map(|(i, &vector)| lt::OwnedKey {
                    owner,
                    occurrence: owner * 16 + i as u64,
                    vector,
                })
                .collect::<Vec<_>>();
            let selected = selector.select_owned(&ex.tokens, ex.query, owner, &keys)?;
            let a = selector.action(&ex.tokens)?;
            let mat: Vec<[i32; 4]> = (0..4)
                .map(|c| {
                    let mut e = [0; 4];
                    e[c] = 1;
                    ht::apply(a, e)
                })
                .collect::<Result<_, _>>()?;
            let mut ordinary = 0;
            let mut distance = i64::MAX;
            for (i, key) in keys.iter().enumerate() {
                let v: Vec<i64> = (0..4)
                    .map(|r| {
                        (0..4)
                            .map(|c| i64::from(mat[c][r]) * i64::from(key.vector[c]))
                            .sum()
                    })
                    .collect();
                let d = v
                    .iter()
                    .zip(ex.query)
                    .map(|(a, b)| (a - i64::from(b)).abs())
                    .sum();
                if d < distance {
                    distance = d;
                    ordinary = i;
                }
            }
            ordinary_equal += usize::from(ordinary == selected);
            let filtered = keys
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != ex.target)
                .map(|(_, k)| *k)
                .collect::<Vec<_>>();
            missing += usize::from(
                selector
                    .select_owned(&ex.tokens, ex.query, owner, &filtered)
                    .is_err(),
            );
            absent += usize::from(
                selector
                    .select_owned(&ex.tokens, ex.query, owner + 99, &keys)
                    .is_err(),
            );
            for rust in [false, true] {
                let index = (selected + shift) % 4;
                let value = values[(world + index) % values.len()];
                let target = values[(world + role) % values.len()];
                let ins = instruction(tok, value, rust, 1);
                let expected = instruction(tok, target, rust, 1);
                let out = exec.run(
                    &ins.sel,
                    &[],
                    ins.facts,
                    &ins.observed,
                    &ins.sel,
                    false,
                    false,
                    &mut work,
                    &mut tokens,
                    &mut acts,
                )?;
                let actual_text = tok.decode(&tokens[..out.tokens]);
                let native = m.rollout(
                    &ins.sel,
                    &[],
                    ins.facts,
                    &ins.observed,
                    &ins.sel,
                    128,
                    false,
                    false,
                );
                if native.tokens != tokens[..out.tokens] || native.actions != acts[..out.actions] {
                    return Err("integrated native/compiled mismatch".into());
                }
                let altered_value = values[(world + index + 3) % values.len()];
                let alt = instruction(tok, altered_value, rust, 1);
                let changed = exec.run(
                    &alt.sel,
                    &[],
                    alt.facts,
                    &alt.observed,
                    &alt.sel,
                    false,
                    false,
                    &mut work,
                    &mut tokens,
                    &mut acts,
                )?;
                let altered_text = tok.decode(&tokens[..changed.tokens]);
                rows.push(serde_json::json!({"world":world,"owner":owner,"role":ROLES[role],"kind":if rust{"rust"}else{"echo"},"source_index":selected,"expected_source_index":ex.target,"source_occurrence":keys[selected].occurrence,"query":tok.decode(&ex.tokens),"value":value,"expected_value":target,"text":actual_text,"actions":native.actions.iter().map(action_name).collect::<Vec<_>>(),"correct":selected==ex.target&&native.actions==expected.actions,"stopped":out.stopped,"changed_source_value":altered_value,"changed_text":altered_text,"changed_correct":acts[..changed.actions]==alt.actions}));
            }
        }
    }
    let report = serde_json::json!({"requests":rows.len(),"correct":rows.iter().filter(|r|r["correct"]==true).count(),"changed_source_correct":rows.iter().filter(|r|r["changed_correct"]==true).count(),"missing_relation_rejected":missing,"absent_owner_rejected":absent,"equivalent_explicit_signed_matrix_selection_agreements":ordinary_equal,"source_selection_requests":32,"selector_sha256":hash(&bytes),"model_sha256":hash(&m.to_bytes()),"rows":rows,"scope":"Finite learned role phrases and typed vector/owner inputs; response syntax generated by the loaded native model. No response renderer, no semantic owner parser, no general coding or geometric-superiority claim."});
    json(root, "integrated-generated.json", &report)?;
    Ok(report)
}
