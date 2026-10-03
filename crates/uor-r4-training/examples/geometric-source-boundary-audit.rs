//! Frozen-parent token-support audit on the 20 exposed development cases.
//! This is teacher-forced diagnosis, not generation, fitting, or a serving change.
//! The alternate token view is derived only from the predicted stored source bytes.
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{
    sha256_file,
    stack_checkpoint::{load_checkpoint, sealed_manifest_sha256},
    stack_grounded_session::{CompiledAction, MemoryEffect, TurnOutcome},
    stack_store::StoreRead,
    Result, TrainingError,
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    out: PathBuf,
    retained_report: PathBuf,
    tokenizer: PathBuf,
    checkpoint: PathBuf,
    maximum_seconds: u64,
}
#[derive(Deserialize)]
struct Report {
    schema: String,
    tokenizer_sha256: String,
    checkpoint_manifest_sha256: String,
    rows: Vec<Row>,
}
#[derive(Deserialize)]
struct Row {
    case: Labels,
    read: bool,
    outcomes: Vec<TurnOutcome>,
}
#[derive(Deserialize)]
struct Labels {
    id: String,
    answers: Answers,
}
#[derive(Deserialize)]
struct Answers {
    accepted: Vec<String>,
}
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn write(p: &Path, v: &impl Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn hash(scores: &[f32]) -> String {
    let mut h = Sha256::new();
    for x in scores {
        h.update(x.to_bits().to_le_bytes());
    }
    hex::encode(h.finalize())
}
fn commit(v: Option<&'static str>, name: &str) -> Result<&'static str> {
    v.filter(|x| x.len() == 40 && x.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| invalid(format!("full {name} build binding required")))
}
fn pieces(tok: &ByteBpeTokenizer, ids: &[u32]) -> Result<Vec<Value>> {
    let mut pos = 0usize;
    let mut rows = Vec::new();
    for (offset, &id) in ids.iter().enumerate() {
        let bytes = tok.decode_bytes(&[id]);
        if bytes.is_empty() {
            return Err(invalid("empty token byte piece"));
        }
        let end = pos
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("piece byte offset overflow"))?;
        rows.push(json!({"token_offset":offset,"token_id":id,"byte_start":pos,"byte_end":end,"bytes_hex":hex::encode(&bytes),"decoded_piece_lossy_diagnostic":String::from_utf8_lossy(&bytes)}));
        pos = end;
    }
    Ok(rows)
}
fn rank(
    scores: &[f32],
    target: u32,
    pool: &BTreeSet<u32>,
    tok: &ByteBpeTokenizer,
) -> Result<Value> {
    let t = *scores
        .get(target as usize)
        .ok_or_else(|| invalid("target outside full parent scores"))?;
    let mut higher = 0usize;
    let mut tied = 0usize;
    let mut best = None::<(u32, f32)>;
    for (id, &s) in scores.iter().enumerate() {
        let id = u32::try_from(id).map_err(|_| invalid("id exceeds u32"))?;
        if pool.contains(&id) {
            continue;
        }
        if id != target {
            if s > t {
                higher += 1;
            }
            if s == t {
                tied += 1;
            }
        }
        if best.map_or(true, |(_, old)| s > old) {
            best = Some((id, s));
        }
    }
    let (best_id, best_score) = best.ok_or_else(|| invalid("no nonsource competitor"))?;
    let target_present = pool.contains(&target);
    let strictly_higher_global = scores.iter().filter(|s| **s > t).count();
    let mut parent_greedy = 0u32;
    let mut global_best = f32::NEG_INFINITY;
    for (id, &s) in scores.iter().enumerate() {
        if s > global_best {
            global_best = s;
            parent_greedy = id as u32;
        }
    }
    Ok(
        json!({"target_present_in_source_pool":target_present,"target_parent_score":t,"higher_nonsource_competitor_count":higher,"tied_nonsource_competitor_count":tied,"target_rank_among_nonsource":if target_present {None}else{Some(higher+1)},"best_nonsource_token_id":best_id,"best_nonsource_decoded":tok.decode(&[best_id]),"best_nonsource_parent_score":best_score,"strictly_higher_parent_token_count":strictly_higher_global,"full_parent_rank":strictly_higher_global+1,"full_parent_top_token_id":parent_greedy,"full_parent_top_score":global_best,"full_parent_top_decoded":tok.decode(&[parent_greedy]),"unsupported_target_blocked_by_full_parent_order":!target_present&&strictly_higher_global>0,"parent_greedy_token_id":parent_greedy,"absent_target_blocked_by_higher_nonsource":!target_present&&higher>0,"absent_target_blocked_by_higher_parent_token":!target_present&&strictly_higher_global>0,"structurally_possible_under_arbitrary_source_pool_copy_mass":target_present||parent_greedy==target,"proof_scope":"necessary support/ordering test allowing arbitrary source-occurrence weights and any aggregate NoRead mass; possible does not establish geometric attainability"}),
    )
}
fn run(a: &Args) -> Result<()> {
    let start = Instant::now();
    let audit_commit = commit(
        option_env!("BOUNDARY_AUDIT_SOURCE_COMMIT"),
        "BOUNDARY_AUDIT_SOURCE_COMMIT",
    )?;
    let library_commit = commit(
        option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "UOR_BUILD_SOURCE_COMMIT",
    )?;
    report_output::verify(
        a.retained_report
            .parent()
            .ok_or_else(|| invalid("report envelope missing"))?,
    )?;
    let d: Report = serde_json::from_slice(&fs::read(&a.retained_report)?)?;
    if d.schema != "uor-r4.source-binding-diagnostic/1"
        || d.tokenizer_sha256 != sha256_file(&a.tokenizer)?
        || d.checkpoint_manifest_sha256
            != sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid(
            "retained source/tokenizer/checkpoint identity differs",
        ));
    }
    let tokbytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokbytes)
        .ok_or_else(|| invalid("unreadable tokenizer"))?;
    let parent =
        load_checkpoint(&a.checkpoint, &Device::Cpu).map_err(|e| invalid(e.to_string()))?;
    parent
        .identity()
        .check_tokenizer(&tokbytes)
        .map_err(|e| invalid(e.to_string()))?;
    if parent.identity().protocol.schema != SCHEMA_V2
        || parent.model.config.context != 384
        || parent.model.config.vocab_size != 4096
        || tok.vocab_size() != 4096
    {
        return Err(invalid(
            "verified protocol2 V4096/context384 parent required",
        ));
    }
    let mut rows = Vec::<Value>::new();
    let mut first_absent = 0usize;
    let mut first_blocked = 0usize;
    let mut all_absent = 0usize;
    let mut all_blocked = 0usize;
    let mut view_absent = 0usize;
    let mut view_blocked = 0usize;
    let mut inferences = 0usize;
    let mut first_full_blocked = 0usize;
    let mut original_full_blocked = 0usize;
    let mut view_full_blocked = 0usize;
    for row in d.rows.into_iter().filter(|x| x.read) {
        let turn = row
            .outcomes
            .last()
            .ok_or_else(|| invalid("empty retained outcomes"))?;
        let found = match &turn.memory {
            MemoryEffect::Read {
                read: StoreRead::Found(f),
            } => f,
            _ => return Err(invalid("expected actual predicted Found read")),
        };
        if found.conflict || !turn.controls.read || !turn.controls.write {
            return Err(invalid("conflict/intervention not admitted"));
        }
        let relation = match turn.action {
            CompiledAction::QueryCurrent { relation } => relation,
            _ => return Err(invalid("predicted current query required")),
        };
        let original_bytes = tok.decode_bytes(&found.tokens);
        let original_text = String::from_utf8(original_bytes.clone())
            .map_err(|e| invalid(format!("source bytes not UTF8: {e}")))?;
        let original_pieces = pieces(&tok, &found.tokens)?;
        let rendered_text = format!(" {original_text}");
        let rendered = tok.encode(&rendered_text);
        if tok.decode_bytes(&rendered) != rendered_text.as_bytes() {
            return Err(invalid(
                "source-only boundary token view does not roundtrip exact bytes",
            ));
        }
        let mut rendered_pieces = pieces(&tok, &rendered)?;
        for piece in &mut rendered_pieces {
            let start = piece["byte_start"]
                .as_u64()
                .ok_or_else(|| invalid("invalid rendered piece start"))?
                as usize;
            let end = piece["byte_end"]
                .as_u64()
                .ok_or_else(|| invalid("invalid rendered piece end"))?
                as usize;
            let lo = start.saturating_sub(1);
            let hi = end.saturating_sub(1);
            let overlaps=original_pieces.iter().filter_map(|p|{let s=p["byte_start"].as_u64()? as usize;let e=p["byte_end"].as_u64()? as usize;if s<hi&&e>lo{Some(json!({"original_token_offset":p["token_offset"],"original_token_id":p["token_id"],"original_byte_start":lo.max(s),"original_byte_end":hi.min(e)}))}else{None}}).collect::<Vec<_>>();
            piece["maps_original_byte_start"] = json!(lo);
            piece["maps_original_byte_end"] = json!(hi);
            piece["contains_added_protocol_space"] = json!(start == 0);
            piece["original_occurrence_overlaps"] = json!(overlaps);
        }
        let accepted = row
            .case
            .answers
            .accepted
            .first()
            .ok_or_else(|| invalid("missing accepted complete label"))?;
        let mut targets = tok.encode(&format!(" {accepted}"));
        targets.push(parent.identity().protocol.eos_id);
        if found.tokens.is_empty()
            || targets.len() > 64
            || turn.emitter_input_ids.len() + targets.len() > 384
        {
            return Err(invalid("whole labeled response outside bounded context"));
        }
        let original_pool: BTreeSet<u32> = found.tokens.iter().copied().collect();
        let view_pool: BTreeSet<u32> = rendered.iter().copied().collect();
        let mut steps = Vec::new();
        for (step, &target) in targets.iter().enumerate() {
            if start.elapsed().as_secs() >= a.maximum_seconds {
                return Err(invalid("declared audit wall limit reached"));
            }
            let mut input = turn.emitter_input_ids.clone();
            input.extend_from_slice(&targets[..step]);
            let scores = parent.model.next_scores(&input)?;
            inferences += 1;
            if scores.len() != 4096 || scores.iter().any(|x| !x.is_finite()) {
                return Err(invalid("invalid full pointer-aware parent scores"));
            }
            let orig_rank = rank(&scores, target, &original_pool, &tok)?;
            let view_rank = rank(&scores, target, &view_pool, &tok)?;
            let absent = !original_pool.contains(&target);
            let blocked = orig_rank["absent_target_blocked_by_higher_nonsource"] == true;
            let full_blocked = orig_rank["unsupported_target_blocked_by_full_parent_order"] == true;
            original_full_blocked += usize::from(full_blocked);
            view_full_blocked +=
                usize::from(view_rank["unsupported_target_blocked_by_full_parent_order"] == true);
            all_absent += usize::from(absent);
            all_blocked += usize::from(blocked);
            view_absent += usize::from(!view_pool.contains(&target));
            view_blocked +=
                usize::from(view_rank["absent_target_blocked_by_higher_nonsource"] == true);
            if step == 0 {
                first_full_blocked += usize::from(full_blocked);
                first_absent += usize::from(absent);
                first_blocked += usize::from(blocked);
            }
            steps.push(json!({"step":step,"label_target_id":target,"label_target_piece":tok.decode(&[target]),"teacherforced_prefix_ids":&targets[..step],"parent_input_ids":input,"parent_scores_count":scores.len(),"parent_scores_sha256":hash(&scores),"original_source_pool":orig_rank,"source_only_protocol_view_pool":view_rank}));
        }
        let first = steps
            .first()
            .cloned()
            .ok_or_else(|| invalid("empty labeled response"))?;
        rows.push(json!({"case":row.case.id,"predicted_relation":relation,"source_record":found.record,"source_commit":found.commit,"original_source_ids":found.tokens,"original_source_text":original_text,"original_source_bytes_hex":hex::encode(&original_bytes),"original_source_pieces":original_pieces,"source_only_protocol_view_ids":rendered,"source_only_protocol_view_pieces":rendered_pieces,"view_derivation":"encode(one protocol space + exact decoded predicted source); no accepted label or evaluator-selected value enters view","label_response_ids":targets,"first_target":first,"steps":steps}));
        write(
            &a.out.join("progress.json"),
            &json!({"completed_cases":rows.len(),"parent_inferences":inferences,"rows":rows,"wall_seconds":start.elapsed().as_secs_f64(),"optimizer_updates":0}),
        )?;
    }
    if rows.len() != 20 {
        return Err(invalid("exact20read-enabled exposed cases required"));
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-source-boundary-audit/1","scope":"all20exposed development labeled teacherforced sequence support/order diagnosis; not a proof that correct text is unreachable through alternative tokenizations or generated prefixes; no model change, fit, or autonomous chat measurement","optimizer_updates":0,"boundary_audit_source_commit":audit_commit,"linked_library_source_commit_declared":library_commit,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"retained_report_sha256":sha256_file(&a.retained_report)?,"tokenizer_sha256":sha256_file(&a.tokenizer)?,"parent_model_sha256":parent.record.model_sha256,"checkpoint_manifest_sha256":sealed_manifest_sha256(&a.checkpoint).map_err(|e|invalid(e.to_string()))?,"protocol_identity":parent.identity().protocol.identity().map_err(|e|invalid(e.to_string()))?,"parent_inferences":inferences,"first_targets_absent_original":first_absent,"first_targets_blocked_full_parent_order_original":first_full_blocked,"all_targets_blocked_full_parent_order_original":original_full_blocked,"all_targets_blocked_full_parent_order_source_view":view_full_blocked,"first_targets_blocked_higher_nonsource_original":first_blocked,"all_targets_absent_original":all_absent,"all_targets_blocked_higher_nonsource_original":all_blocked,"all_targets_absent_source_view":view_absent,"all_targets_blocked_higher_nonsource_source_view":view_blocked,"proof":"For target outside copy pool, mixed P(target)=a*parent P(target). Every outside-pool competitor gets the same a scaling. Any competitor with larger parent probability remains larger for a>0: an outside-pool competitor has equal scaling, while a source competitor also receives nonnegative copy mass. For a=0 some source token has positive mass and target has zero. No choice of geometric source occurrence weights or NoRead mass can make such an unsupported target win at this labeled teacher-forced prefix. View membership removes a support obstruction only; it does not establish learning or semantic advantage.","wall_seconds":start.elapsed().as_secs_f64(),"rows":rows}),
    )?;
    Ok(())
}
fn main() -> Result<()> {
    let mut argv = std::env::args().skip(1);
    let file = argv
        .next()
        .ok_or_else(|| invalid("usage: geometric-source-boundary-audit ARGS.json"))?;
    if argv.next().is_some() {
        return Err(invalid("one JSON argument required"));
    }
    let a: Args = serde_json::from_slice(&fs::read(file)?)?;
    if !(1..=300).contains(&a.maximum_seconds) {
        return Err(invalid("audit wall limit must be 1..300 seconds"));
    }
    report_output::claim(&a.out)?;
    write(&a.out.join("args.json"), &a)?;
    let result = run(&a);
    if let Err(e) = &result {
        write(
            &a.out.join("error.json"),
            &json!({"error":e.to_string(),"optimizer_updates":0}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
