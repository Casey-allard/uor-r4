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
/// Declared fixed size of the matched span draw's value pool. The span block consumes exactly
/// `1 + 2 * SPAN_POOL` draws per span example regardless of arm or of the drawn `len`, so two arms
/// that differ only in `max_span`/`span_draw_max` share an identical span pool.
const SPAN_POOL: usize = 32;
/// Declared seed of the dedicated span stream. It must be nonzero: `next()` is in-place with a fixed
/// point at zero.
const SPAN_SEED: u64 = 20260926;
/// Span examples drawn per batch.
const SPAN_EXAMPLES: usize = 2;
/// Non-span draws per batch: 2 repository, 2 broad, 4 instruction, 2 legacy grounded.
const NON_SPAN_DRAWS: usize = 10;
/// Declared copy/stop-length curriculum sets, each a slice of inclusive value ranges. `two`, `three`
/// and `four` each select exactly 64 values, so the decisive `two` vs `three` comparison differs only
/// in the added values' token length. The ranges avoid every non-`A3_pos` held-out panel value;
/// `COPY_A3` contains the three declared `A3_pos` positives by design.
const COPY_BASE: &[(u32, u32)] = &[(0, 31)];
const COPY_A2: &[(u32, u32)] = &[(50, 76), (78, 82)];
const COPY_A3: &[(u32, u32)] = &[(140, 171)];
const COPY_A4: &[(u32, u32)] = &[(1000, 1023), (1025, 1032)];
/// Frozen held-out evaluation panel `(value, class)`, hashed into the receipt.
const EVAL_PANEL: &[(&str, &str)] = &[
    ("37", "L2"),
    ("42", "L2"),
    ("84", "L2"),
    ("96", "L2"),
    ("77", "L2_run"),
    ("88", "L2_run"),
    ("105", "L3_near"),
    ("132", "L3_near"),
    ("137", "L3_near"),
    ("184", "L3_near"),
    ("196", "L3_near"),
    ("208", "L3_far"),
    ("317", "L3_far"),
    ("512", "L3_far"),
    ("745", "L3_far"),
    ("777", "L3_run"),
    ("888", "L3_run"),
    ("999", "L3_run"),
    ("1024", "L4"),
    ("2048", "L4"),
    ("3072", "L4"),
    ("4096", "L4"),
    ("7777", "L4_run"),
    ("8888", "L4_run"),
    ("140", "A3_pos"),
    ("155", "A3_pos"),
    ("171", "A3_pos"),
];
/// Declared diagnostic panel: the failing multi-token panel plus the three `A3_pos` values.
const DIAGNOSE_VALUES: [&str; 11] = [
    "37", "42", "105", "208", "317", "512", "1024", "2048", "140", "155", "171",
];
/// The selected inclusive value ranges of one declared curriculum name.
fn copy_ranges(name: &str) -> Result<Vec<(u32, u32)>, String> {
    Ok(match name {
        "base" => COPY_BASE.to_vec(),
        "two" => [COPY_BASE, COPY_A2].concat(),
        "three" => [COPY_BASE, COPY_A3].concat(),
        "four" => [COPY_BASE, COPY_A4].concat(),
        _ => return Err("UOR_LANGUAGE_COPY_SET must be base|two|three|four".into()),
    })
}
/// The selected fit value set of one declared curriculum name, ascending.
fn copy_values(name: &str) -> Result<Vec<u32>, String> {
    Ok(copy_ranges(name)?
        .iter()
        .flat_map(|(a, b)| *a..=*b)
        .collect())
}

/// One drawn span: the drawn copy length and the exact pool its source and feedback prefixes are
/// cut from. The example and its feedback are rebuilt deterministically from these two fields.
struct SpanDraw {
    len: usize,
    pool: Vec<u32>,
}
impl SpanDraw {
    fn source(&self) -> Vec<u32> {
        self.pool[..self.len].to_vec()
    }
    fn feedback(&self) -> Vec<u32> {
        self.pool[SPAN_POOL..SPAN_POOL + self.len].to_vec()
    }
    fn example(&self, w: &Words) -> TlExample {
        span_case(w, self.source(), &self.feedback())
    }
}
/// One batch's two independent draw streams. The non-span draws use only `non_span` (raw
/// `rng_state` outputs, in consumption order); spans use only the dedicated span stream.
struct BatchDraw {
    non_span: Vec<u64>,
    spans: Vec<SpanDraw>,
}
/// Draw one batch's indices and spans from the two independent streams, without assembling the
/// non-span examples. Consumes exactly `NON_SPAN_DRAWS` outputs of `rng` and exactly
/// `SPAN_EXAMPLES * (1 + 2 * SPAN_POOL)` outputs of `span_rng`, independent of `span_draw_max`.
fn draw_batch(rng: &mut u64, span_rng: &mut u64, span_draw_max: usize, w: &Words) -> BatchDraw {
    let non_span = (0..NON_SPAN_DRAWS).map(|_| next(rng)).collect();
    let mut spans = Vec::with_capacity(SPAN_EXAMPLES);
    for _ in 0..SPAN_EXAMPLES {
        let len = 1 + next(span_rng) as usize % span_draw_max;
        let pool: Vec<u32> = (0..2 * SPAN_POOL)
            .map(|_| w.values[next(span_rng) as usize % 4])
            .collect();
        spans.push(SpanDraw { len, pool });
    }
    BatchDraw { non_span, spans }
}
/// Count the state coordinates pinned at either clamp after a transition.
fn saturated(h: &[i32], clamp: i32) -> usize {
    h.iter().filter(|x| **x == clamp || **x == -clamp).count()
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
    values: &[(&str, &str)],
    variant: usize,
) -> Result<serde_json::Value, String> {
    let exec = fast::Execution::compile(m)?;
    let mut work = exec.workspace();
    let mut tokens = vec![0; 96];
    let mut acts = vec![TlAction::Stop; 96];
    let mut rows = Vec::new();
    for (value, class) in values {
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
            let len = ex.sel.len();
            let copy_count_correct =
                r.actions >= len && acts[..len].iter().all(|a| matches!(a, TlAction::Copy));
            let stop_at_len = len < acts.len() && r.actions > len && acts[len] == TlAction::Stop;
            rows.push(serde_json::json!({"value":value,"class":class,"kind":if rust{"rust"}else{"echo"},"prompt":tok.decode(&ex.observed),"text":tok.decode(&tokens[..r.tokens]),"tokens":&tokens[..r.tokens],"token_length":len,"actions":acts[..r.actions].iter().map(action_name).collect::<Vec<_>>(),"expected_actions":ex.actions.iter().map(action_name).collect::<Vec<_>>(),"exact_actions":acts[..r.actions]==ex.actions,"copy_count_correct":copy_count_correct,"stop_at_len":stop_at_len,"stopped":r.stopped,"copied":r.copied}));
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
/// Cheap pre-training preflight: the selected fit values and the frozen held-out panel are disjoint,
/// every held-out value's BPE length and token ids are recorded, and the held-out token set is fully
/// covered by the selected fit token set.
fn panel_preflight(
    tok: &HfBpeTokenizer,
    copy_set: &str,
    copy: &[u32],
) -> Result<serde_json::Value, String> {
    let all_declared: Vec<u32> = [COPY_BASE, COPY_A2, COPY_A3, COPY_A4]
        .concat()
        .iter()
        .flat_map(|(a, b)| *a..=*b)
        .collect();
    let a3: Vec<u32> = COPY_A3.iter().flat_map(|(a, b)| *a..=*b).collect();
    let fit_tokens: std::collections::BTreeSet<u32> = copy
        .iter()
        .flat_map(|v| tok.encode(&v.to_string()))
        .collect();
    let mut rows = Vec::new();
    let mut held_tokens: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut panel_values_in_fit = Vec::new();
    for (value, class) in EVAL_PANEL {
        let held: u32 = value.parse().map_err(|_| "eval panel value")?;
        if *class == "A3_pos" {
            if !a3.contains(&held) {
                return Err(format!("A3_pos panel value {held} is not in COPY_A3"));
            }
            if copy.contains(&held) {
                panel_values_in_fit.push(serde_json::json!({"value": held, "class": class}));
            }
        } else {
            if copy.contains(&held) {
                return Err(format!(
                    "held-out panel value {held} ({class}) is in the selected fit set"
                ));
            }
            if all_declared.contains(&held) {
                return Err(format!(
                    "held-out panel value {held} ({class}) is in a declared fit range"
                ));
            }
        }
        let ids = tok.encode(value);
        held_tokens.extend(ids.iter().copied());
        rows.push(serde_json::json!({"value":value,"class":class,"token_length":ids.len(),"token_ids":ids}));
    }
    let missing: Vec<u32> = held_tokens.difference(&fit_tokens).copied().collect();
    let out = serde_json::json!({"copy_set":copy_set,"selected_fit_values":copy,"selected_fit_token_count":fit_tokens.len(),"rows":rows,"panel_values_in_fit":panel_values_in_fit,"held_out_tokens_minus_fit":missing,"coverage_complete":missing.is_empty()});
    if !missing.is_empty() {
        return Err(format!("EVAL_PANEL token coverage gap: {missing:?}"));
    }
    Ok(out)
}
/// Read-only teacher-forced diagnostic on a loaded artifact. It never trains. For each declared
/// value and prompt variant it teacher-forces the expected `Copy^L + Stop` script and records the
/// served integer margins at the copy and stop rows, the top-1 action at each decision, and the
/// state saturation after each transition.
fn diagnose(root: &Path, m: &TlModel, tok: &HfBpeTokenizer) -> Result<serde_json::Value, String> {
    let mut rows = Vec::new();
    let mut summary = Vec::new();
    for value in DIAGNOSE_VALUES {
        for variant in 0..2usize {
            let ex = instruction(tok, value, false, variant);
            let mm = m.content_feature(&ex.sel, &ex.res);
            let facts = m.typed_block(&ex.sel, &ex.res, ex.facts);
            let mut h = m.init_state(&mm, &facts);
            for &t in &ex.observed {
                h = m.transition(&h, TL_EV_OBSERVE, Some(t), &mm, &facts);
            }
            let mut copy_margins = Vec::new();
            let mut copy_decisions = Vec::new();
            let mut saturation = Vec::new();
            for (i, &t) in ex.sel.iter().enumerate() {
                let event = if i == 0 { TL_EV_OBSERVE } else { TL_EV_COPY };
                let logits = m.readout(&h, &mm, &facts, event);
                let vocab_max = logits[..m.vocab]
                    .iter()
                    .copied()
                    .max()
                    .ok_or("empty vocab")?;
                copy_margins.push(logits[m.copy_row()] - vocab_max);
                copy_decisions.push(action_name(&m.decide(&h, &mm, &facts, event, true)));
                h = m.transition(&h, TL_EV_COPY, Some(t), &mm, &facts);
                saturation.push(saturated(&h, m.h_clamp));
            }
            let logits = m.readout(&h, &mm, &facts, TL_EV_COPY);
            let vocab_max = logits[..m.vocab]
                .iter()
                .copied()
                .max()
                .ok_or("empty vocab")?;
            let stop_margin = logits[m.stop_row()] - vocab_max;
            let post_action = m.decide(&h, &mm, &facts, TL_EV_COPY, false);
            h = m.transition(&h, TL_EV_COPY, None, &mm, &facts);
            let post_saturation = saturated(&h, m.h_clamp);
            summary.push(serde_json::json!({"value":value,"variant":variant,"token_length":ex.sel.len(),"last_copy_margin":copy_margins.last(),"stop_margin":stop_margin,"post_copy_decision":action_name(&post_action),"saturation_last_copy":saturation.last(),"saturation_post_copy":post_saturation}));
            rows.push(serde_json::json!({"value":value,"variant":variant,"token_ids":ex.sel,"token_length":ex.sel.len(),"copy_margins":copy_margins,"copy_decisions":copy_decisions,"saturation_after_copy":saturation,"stop_margin":stop_margin,"post_copy_decision":action_name(&post_action),"post_copy_is_stop":matches!(post_action,TlAction::Stop),"saturation_after_post_copy":post_saturation,"h_clamp":m.h_clamp}));
        }
    }
    let report = serde_json::json!({"rows":rows,"scope":"Read-only teacher-forced diagnostic on the loaded artifact. Margins are served integer logit differences at the copy/stop rows, not probabilities, and are not a capability claim. `copy_margins[i]` is the i-th copy decision; `stop_margin` is the post-copy decision after all copies."});
    json(root, "diagnostics.json", &report)?;
    Ok(
        serde_json::json!({"summary":summary,"scope":"Read-only diagnostic; full rows in diagnostics.json."}),
    )
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
    if mode == "diagnose" {
        return diagnose(root, parent, tok);
    }
    if mode == "evaluate" {
        let anchors =
            ["37", "42", "105", "208", "317", "512", "1024", "2048"].map(|v| (v, "anchor"));
        let generated = outputs(parent, tok, &anchors, 1)?;
        json(root, "generated.json", &generated)?;
        let v0 = outputs(parent, tok, EVAL_PANEL, 0)?;
        let v1 = outputs(parent, tok, EVAL_PANEL, 1)?;
        json(root, "eval-panel-v0.json", &v0)?;
        json(root, "eval-panel-v1.json", &v1)?;
        return Ok(
            serde_json::json!({"panels":retained(parent,&w),"span":span_panel(parent,&w),"fresh_span":span_lengths(parent,&w,&[13,21,34]),"repository_bits":evaluate_single(parent,dev),"generated":generated,"eval_panel":{"variant0":v0,"variant1":v1},"scope":"Held-out EVAL_PANEL over the frozen value/class rows; the primary endpoint is prompt variant 0. `generated` remains the prior variant-1 anchor list so the earlier comparison is preserved. Bounded authored copy/stop-length panel, not general language."}),
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
    if max_span > SPAN_POOL {
        return Err("span fit bound".into());
    }
    let span_draw_max: usize = std::env::var("UOR_LANGUAGE_SPAN_DRAW_MAX")
        .unwrap_or_else(|_| max_span.to_string())
        .parse()
        .map_err(|_| "span draw bound")?;
    if !(1..=SPAN_POOL).contains(&span_draw_max) {
        return Err("span draw bound".into());
    }
    let copy_set = std::env::var("UOR_LANGUAGE_COPY_SET").unwrap_or_else(|_| "base".into());
    let ranges = copy_ranges(&copy_set)?;
    let copy = copy_values(&copy_set)?;
    let broad_dir =
        PathBuf::from("/Users/casey.allard/uor-r4-investigations/causal-20260924T000007/data");
    let (broad, broad_meta) =
        super::causal_continuation::sources(tok, &broad_dir.join("fit"), 256)?;
    let (tune, tune_meta) = super::causal_continuation::sources(tok, &broad_dir.join("tune"), 16)?;
    let (train_ground, _, _) = authored_world(&w);
    let instructions = copy
        .iter()
        .flat_map(|n| [false, true].map(move |r| (n, r)))
        .map(|(n, r)| instruction(tok, &n.to_string(), r, 0))
        .collect::<Vec<_>>();
    let preflight = panel_preflight(tok, &copy_set, &copy)?;
    json(root, "preflight.json", &preflight)?;
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
    let parent_source: Vec<u8> = head["source_sha256"]
        .as_array()
        .ok_or("parent checkpoint source_sha256 missing")?
        .iter()
        .map(|x| x.as_u64().unwrap_or(0) as u8)
        .collect();
    let parent_checkpoint_source = sha256_hex(&parent_source);
    let eval_panel_sha256 = hash(&serde_json::to_vec(EVAL_PANEL).map_err(|e| e.to_string())?);
    let metadata = serde_json::json!({"parent_model":hash(&parent.to_bytes()),"old_checkpoint":hash(&old),"arm":arm,"steps":total,"old_optimizer_step":start_step,"phase_lr":[0.01,0.001],"seed":20260925u64,"copy_ranges":ranges,"validation_numbers":[32,33,34,35],"gradient_span_lengths":(1..=max_span).collect::<Vec<_>>(),"span_draw_max":span_draw_max,"span_pool":SPAN_POOL,"span_seed":SPAN_SEED,"copy_set":copy_set,"copy_values":copy,"copy_pool_examples":instructions.len(),"eval_panel_sha256":eval_panel_sha256,"parent_checkpoint_source":parent_checkpoint_source,"broad_fit":broad_meta,"broad_tune":tune_meta,"sampling":"2 repo,2 broad,4 instruction,2 legacy grounded,2 matched span examples","runner_source":hash(include_bytes!("language_continuation.rs")),"checkpoint_source":hash(include_bytes!("../../native_geometric/learner/tl_checkpoint.rs")),"causal_runner_source":hash(include_bytes!("causal_continuation.rs")),"source":hash(include_bytes!("language_continuation.rs"))});
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
        span_rng_state: SPAN_SEED,
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
        let drawn = draw_batch(
            &mut cursor.rng_state,
            &mut cursor.span_rng_state,
            span_draw_max,
            &w,
        );
        let mut draws = drawn.non_span.iter().copied();
        let mut slot = || -> Result<usize, String> {
            Ok(draws.next().ok_or("non-span draw exhaustion")? as usize)
        };
        for _ in 0..2 {
            batch.push(prose_example(&repo[slot()? % repo.len()]));
            feedback.push(None);
            batch.push(prose_example(&broad[slot()? % broad.len()]));
            feedback.push(None);
        }
        for _ in 0..4 {
            batch.push(instructions[slot()? % instructions.len()].clone());
            feedback.push(None);
        }
        for _ in 0..2 {
            batch.push(train_ground[slot()? % train_ground.len()].example(4., 0));
            feedback.push(None);
        }
        for span in &drawn.spans {
            batch.push(span.example(&w));
            feedback.push(Some(span.feedback()));
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
            let out = outputs(
                &m,
                tok,
                &["32", "33", "34", "35"].map(|v| (v, "validation")),
                0,
            )?;
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
    fn unit_words() -> Words {
        Words {
            temporal: 0,
            copula: [1, 2],
            class_a: 3,
            class_b: 4,
            values: vec![5, 6, 7, 8],
        }
    }
    #[test]
    fn span_stream_is_isolated_and_matched_across_arms() {
        let w = unit_words();
        let mut rng_a = 20260925u64;
        let mut rng_b = 20260925u64;
        let mut span_a = SPAN_SEED;
        let mut span_b = SPAN_SEED;
        let (mut shared_equal, mut pools_equal, mut len_differs) = (true, true, false);
        for _ in 0..16 {
            let a = draw_batch(&mut rng_a, &mut span_a, 3, &w);
            let b = draw_batch(&mut rng_b, &mut span_b, 8, &w);
            shared_equal &= a.non_span == b.non_span && rng_a == rng_b;
            for (sa, sb) in a.spans.iter().zip(&b.spans) {
                pools_equal &= sa.pool == sb.pool;
                len_differs |= sa.len != sb.len;
                assert_eq!(sa.example(&w).sel, sa.pool[..sa.len].to_vec());
                assert_eq!(
                    sa.feedback(),
                    sa.pool[SPAN_POOL..SPAN_POOL + sa.len].to_vec()
                );
            }
        }
        assert!(shared_equal, "non-span rng_state stream must not shift");
        assert!(pools_equal, "span pools must be identical across span arms");
        assert!(len_differs, "span_draw_max must change the drawn length");
        let mut rng_c = 20260925u64;
        let mut rng_d = 20260925u64;
        let mut span_c = SPAN_SEED;
        let mut span_d = SPAN_SEED;
        for _ in 0..16 {
            let c = draw_batch(&mut rng_c, &mut span_c, 3, &w);
            let d = draw_batch(&mut rng_d, &mut span_d, 3, &w);
            assert_eq!(c.non_span, d.non_span);
            for (sc, sd) in c.spans.iter().zip(&d.spans) {
                assert_eq!(sc.len, sd.len);
                assert_eq!(sc.example(&w).sel, sd.example(&w).sel);
                assert_eq!(sc.example(&w).actions, sd.example(&w).actions);
                assert_eq!(sc.feedback(), sd.feedback());
            }
        }
    }
    #[test]
    fn span_draw_counts_and_curriculum_sets_are_declared() {
        let w = unit_words();
        let mut rng = 1u64;
        let mut span = SPAN_SEED;
        let d = draw_batch(&mut rng, &mut span, SPAN_POOL, &w);
        assert_eq!(d.non_span.len(), NON_SPAN_DRAWS);
        assert_eq!(d.spans.len(), SPAN_EXAMPLES);
        assert!(d.spans.iter().all(|s| s.pool.len() == 2 * SPAN_POOL));
        assert!(d.spans.iter().all(|s| (1..=SPAN_POOL).contains(&s.len)));
        assert_eq!(copy_values("base").unwrap().len(), 32);
        assert_eq!(copy_values("two").unwrap().len(), 64);
        assert_eq!(copy_values("three").unwrap().len(), 64);
        assert_eq!(copy_values("four").unwrap().len(), 64);
        assert!(copy_values("nope").is_err());
    }
    #[test]
    fn declared_copy_ranges_avoid_held_out_panel_values() {
        let all: Vec<u32> = [COPY_BASE, COPY_A2, COPY_A3, COPY_A4]
            .concat()
            .iter()
            .flat_map(|(a, b)| *a..=*b)
            .collect();
        let a3: Vec<u32> = COPY_A3.iter().flat_map(|(a, b)| *a..=*b).collect();
        for (value, class) in EVAL_PANEL {
            let v: u32 = value.parse().unwrap();
            if *class == "A3_pos" {
                assert!(a3.contains(&v), "A3_pos {v} must be in COPY_A3");
            } else {
                assert!(
                    !all.contains(&v),
                    "held-out panel value {v} ({class}) must not be in any declared fit range"
                );
            }
        }
        for name in ["two", "three", "four"] {
            assert_eq!(copy_values(name).unwrap().len(), 64);
        }
    }
    #[test]
    #[ignore = "reads the pinned local SmolLM2 tokenizer; run explicitly to verify held-out coverage"]
    fn panel_preflight_holds_the_disjointness_and_coverage_rule() {
        let bytes = std::fs::read(super::DEFAULT_TOKENIZER).unwrap();
        let tok = derive_tokenizer(&bytes, VOCAB).unwrap();
        for (name, expected_positives) in [
            ("base", serde_json::json!([])),
            ("two", serde_json::json!([])),
            (
                "three",
                serde_json::json!([
                    {"value":140,"class":"A3_pos"},
                    {"value":155,"class":"A3_pos"},
                    {"value":171,"class":"A3_pos"}
                ]),
            ),
            ("four", serde_json::json!([])),
        ] {
            let pre = panel_preflight(&tok, name, &copy_values(name).unwrap()).unwrap();
            assert_eq!(pre["coverage_complete"], serde_json::json!(true));
            assert!(pre["held_out_tokens_minus_fit"]
                .as_array()
                .unwrap()
                .is_empty());
            assert_eq!(pre["panel_values_in_fit"], expected_positives);
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
