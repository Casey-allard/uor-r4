//! One ordinary all-answer fit from the accepted Source48/Generate64 parent.
//! Only Potential and Generate coefficients train; no U or proposal machinery.
use super::*;

const QUALIFICATION_INDICES: [usize; 24] = [
    0, 1, 4, 5, 8, 9, 12, 13, 130, 131, 138, 139, 256, 257, 264, 265, 386, 387, 394, 395, 448, 449,
    456, 457,
];

const QUALIFICATION_IDS: [&str; 24] = [
    "development-diverse-length2-00-swap0-q0-forward-job",
    "development-diverse-length2-00-swap0-q0-forward-home",
    "development-diverse-length2-00-swap0-q3-forward-job",
    "development-diverse-length2-00-swap0-q3-forward-home",
    "development-diverse-length2-00-swap1-q0-forward-job",
    "development-diverse-length2-00-swap1-q0-forward-home",
    "development-diverse-length2-00-swap1-q3-forward-job",
    "development-diverse-length2-00-swap1-q3-forward-home",
    "development-diverse-length4-00-swap0-q2-reverse-job",
    "development-diverse-length4-00-swap0-q2-reverse-home",
    "development-diverse-length4-00-swap1-q2-reverse-job",
    "development-diverse-length4-00-swap1-q2-reverse-home",
    "development-diverse-length8-00-swap0-q4-forward-job",
    "development-diverse-length8-00-swap0-q4-forward-home",
    "development-diverse-length8-00-swap1-q4-forward-job",
    "development-diverse-length8-00-swap1-q4-forward-home",
    "development-diverse-update-00-swap0-q0-reverse-job",
    "development-diverse-update-00-swap0-q0-reverse-home",
    "development-diverse-update-00-swap1-q0-reverse-job",
    "development-diverse-update-00-swap1-q0-reverse-home",
    "development-diverse-reassert-00-swap0-q4-forward-job",
    "development-diverse-reassert-00-swap0-q4-forward-home",
    "development-diverse-reassert-00-swap1-q4-forward-job",
    "development-diverse-reassert-00-swap1-q4-forward-home",
];

pub(super) fn is_mode(mode: Mode) -> bool {
    matches!(mode, Mode::ReplyCompletion | Mode::ReplyQualification)
}

fn expected_updates(mode: Mode) -> Result<usize> {
    match mode {
        Mode::ReplyCompletion => Ok(64),
        Mode::ReplyQualification => Ok(96),
        _ => Err(bad("not a reply training mode")),
    }
}

/// Fixed indices are declared before outputs. Shuffle once; repeat the same
/// 24-row order for 32 epochs, with no loss-based sampling or endpoint selection.
fn draw_schedule(mode: Mode, seed: u64) -> Result<Vec<usize>> {
    match mode {
        Mode::ReplyCompletion => Ok(order(seed, 512)),
        Mode::ReplyQualification => {
            let one_epoch = order(seed, QUALIFICATION_INDICES.len())
                .into_iter()
                .map(|i| QUALIFICATION_INDICES[i])
                .collect::<Vec<_>>();
            Ok(one_epoch.iter().copied().cycle().take(96 * BATCH).collect())
        }
        _ => Err(bad("not a reply training schedule")),
    }
}

pub(super) fn settings(a: &Args) -> Result<()> {
    if !is_mode(a.mode) {
        return Ok(());
    }
    if a.updates != expected_updates(a.mode)?
        || a.seed != 1001
        || a.credit != Credit::RawIdentity
        || a.read_state_pullback != ReadStatePullback::Categorical
        || a.loss_scope != LossScope::All
        || a.ceiling_scorer
        || a.ceiling_float
        || a.query_conditioned_read
        || a.panel_inputs.is_some()
        || a.panel_labels.is_some()
        || a.baseline.is_some()
        || a.prediction_control_updates.is_some()
        || a.prediction_control_rates.is_some()
        || a.prediction_control_resume.is_some()
        || a.prediction_control_resume_source_step.is_some()
        || a.prediction_control_resume_generate_step.is_some()
        || a.prediction_control_trainable != ControlTrainable::Joint
        || a.continuation.is_some()
        || a.joint_continuation.is_some()
        || a.reference_replay.is_some()
        || a.native_code_proposals
        || a.reached_frontier_objective
        || a.categorical_action_learning
        || a.categorical_action_only
        || a.constrained_context_learning
        || a.constrained_emission_learning
        || a.retained_context_root.is_some()
        || a.prototype_compensation.is_some()
        || a.reached_u.is_some()
        || a.readout_coadaptation.is_some()
        || a.prefix_context_credit.is_some()
        || a.context_path_credit.is_some()
        || a.context_cue_coadapt.is_some()
        || a.prefix_fragment_learning.is_some()
        || a.generate_episode_learning.is_some()
        || a.coupled_episode_learning.is_some()
        || a.generate_episode_completion.is_some()
        || a.prefix_artifact_check.is_some()
    {
        return Err(bad("reply modes require their fixed64-or96 dose/seed1001/raw_identity/categorical/all-answer fit without continuation, proposal or diagnostic options"));
    }
    Ok(())
}

fn selected_gradients(
    grads: BTreeMap<String, Tensor>,
    coefficients: &BTreeMap<String, Var>,
    potential: &BTreeMap<String, Var>,
) -> Result<BTreeMap<String, Tensor>> {
    // Frozen groups must not affect the active gradient's clipping denominator.
    let selected: BTreeMap<_, _> = grads
        .into_iter()
        .filter(|(name, _)| coefficients.contains_key(name) || potential.contains_key(name))
        .collect();
    if selected.is_empty() {
        return Err(bad("reply completion active gradients absent"));
    }
    Ok(selected)
}

fn outcomes(initial: &Value, final_eval: &Value) -> Result<Value> {
    let before = initial["rows"]
        .as_array()
        .ok_or_else(|| bad("initial rows absent"))?;
    let after = final_eval["rows"]
        .as_array()
        .ok_or_else(|| bad("final rows absent"))?;
    if before.len() != 512 || after.len() != 512 {
        return Err(bad("reply comparison requires full512 endpoints"));
    }
    let mut gained = Vec::new();
    let mut lost = Vec::new();
    let mut retained = Vec::new();
    let mut changed = Vec::new();
    for (a, b) in before.iter().zip(after) {
        if a["id"] != b["id"] || a["id"].as_str().is_none() {
            return Err(bad("reply comparison row identity differs"));
        }
        let old = a["complete"]
            .as_bool()
            .ok_or_else(|| bad("initial completion absent"))?;
        let new = b["complete"]
            .as_bool()
            .ok_or_else(|| bad("final completion absent"))?;
        match (old, new) {
            (false, true) => gained.push(b["id"].clone()),
            (true, false) => lost.push(b["id"].clone()),
            (true, true) => retained.push(b["id"].clone()),
            _ => {}
        }
        if a["generated_ids"] != b["generated_ids"] {
            changed.push(b["id"].clone());
        }
    }
    Ok(
        json!({"gained_complete_ids":gained,"lost_complete_ids":lost,
        "retained_complete_ids":retained,"changed_output_ids":changed,
        "keep":!gained.is_empty() && lost.is_empty(),
        "bar":"retain all eight original complete replies and gain at least one; milestone target remains256/512"}),
    )
}

fn qualification_panel(eps: &[Episode]) -> Result<Value> {
    if eps.len() != 512 {
        return Err(bad("qualification panel requires fixed512 rows"));
    }
    let mut selected = Vec::new();
    for (&index, &id) in QUALIFICATION_INDICES.iter().zip(&QUALIFICATION_IDS) {
        let e = &eps[index];
        if e.packet.id != id {
            return Err(bad("qualification selected index/ID differs"));
        }
        let stratum = id
            .split('-')
            .nth(2)
            .ok_or_else(|| bad("qualification stratum absent"))?;
        selected.push(json!({"index":index,"id":id,"stratum":stratum,
            "input_packet_sha256":sha256_bytes(&serde_json::to_vec(&e.packet)?),
            "complete_canonical_target_with_eos_sha256":sha256_bytes(&serde_json::to_vec(&e.target)?),
            "target_positions_per_draw":e.target.len(),"draws":32}));
    }
    Ok(
        json!({"selection":"fixed prospective indices and exact IDs; no output-based sampling",
        "rows":selected,"distinct_rows":24,"epochs":32,"episode_draws":768,
        "target_position_draws":QUALIFICATION_INDICES.iter().map(|&i|eps[i].target.len()*32).sum::<usize>()}),
    )
}

/// Read the already-scored endpoint, authenticating each row and using exactly
/// the loaded frozen answer oracle. This does not invoke inference or rescore a
/// checkpoint for selection. Teacher correctness is separate from own feedback.
fn qualification_rows(
    a: &Args,
    evaluation: &Value,
    initial: &Value,
    eps: &[Episode],
) -> Result<Vec<Value>> {
    let refs = evaluation["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("qualification requires full512 saved evaluation"))?;
    let initial_refs = initial["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("qualification requires full512 baseline"))?;
    let mut measurements = Vec::new();
    for (&index, &id) in QUALIFICATION_INDICES.iter().zip(&QUALIFICATION_IDS) {
        let e = &eps[index];
        let r = &refs[index];
        if e.packet.id != id || r["id"] != id || initial_refs[index]["id"] != id {
            return Err(bad("qualification saved row identity differs"));
        }
        let filename = r["row_file"]
            .as_str()
            .ok_or_else(|| bad("qualification row file absent"))?;
        let path = a.out.join(filename);
        if r["row_sha256"] != sha256_file(&path)? {
            return Err(bad("qualification saved row hash differs"));
        }
        let row = read(&path)?;
        let canonical = row["canonical"]
            .as_array()
            .filter(|r| r.len() == e.target.len())
            .ok_or_else(|| bad("qualification canonical coverage differs"))?;
        if row["id"] != id || row["canonical_target_ids_labels_only"] != json!(e.target) {
            return Err(bad("qualification canonical labels differ"));
        }
        let mut teacher_correct = 0;
        for (step, &target) in canonical.iter().zip(&e.target) {
            if step["target_label_only"] != target {
                return Err(bad("qualification canonical target differs"));
            }
            let chosen = step["native"]["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("qualification canonical winner absent"))?;
            teacher_correct += usize::from(chosen == u64::from(target));
        }
        let generated: Vec<u32> = serde_json::from_value(row["generated_ids"].clone())?;
        let eos = row["eos"]
            .as_bool()
            .ok_or_else(|| bad("qualification EOS absent"))?;
        let text = row["decoded"]
            .as_str()
            .ok_or_else(|| bad("qualification decoded answer absent"))?;
        let complete = eos && e.answers.accepts(text);
        if row["complete"] != complete
            || r["complete"] != complete
            || row["generated_ids"] != r["generated_ids"]
            || r["eos"] != eos
        {
            return Err(bad(
                "qualification frozen answer membership/evaluation differs",
            ));
        }
        let original_complete = initial_refs[index]["complete"]
            .as_bool()
            .ok_or_else(|| bad("qualification baseline completion absent"))?;
        let stratum = id
            .split('-')
            .nth(2)
            .ok_or_else(|| bad("qualification stratum absent"))?;
        measurements.push(json!({"index":index,"id":id,"stratum":stratum,
            "complete":complete,"teacher_all_tokens_correct":teacher_correct==e.target.len(),
            "teacher_correct_tokens":teacher_correct,"teacher_total_tokens":e.target.len(),
            "entry_correct":generated.first()==e.target.first(),"eos":eos,
            "original_complete":original_complete,"row_file":filename,"row_sha256":r["row_sha256"]}));
    }
    Ok(measurements)
}

fn measurement_counts(rows: &[Value]) -> Value {
    let complete = rows.iter().filter(|r| r["complete"] == true).count();
    let teacher = rows
        .iter()
        .filter(|r| r["teacher_all_tokens_correct"] == true)
        .count();
    let entry = rows.iter().filter(|r| r["entry_correct"] == true).count();
    let eos = rows.iter().filter(|r| r["eos"] == true).count();
    let gained = rows
        .iter()
        .filter(|r| r["complete"] == true && r["original_complete"] == false)
        .map(|r| r["id"].clone())
        .collect::<Vec<_>>();
    let lost = rows
        .iter()
        .filter(|r| r["complete"] == false && r["original_complete"] == true)
        .map(|r| r["id"].clone())
        .collect::<Vec<_>>();
    json!({"cases":rows.len(),"complete":complete,"teacher_all_tokens_correct":teacher,
        "entry_correct":entry,"eos":eos,"gained_complete_ids":gained,"lost_complete_ids":lost})
}

fn qualification_outcomes(
    a: &Args,
    initial: &Value,
    final_eval: &Value,
    eps: &[Episode],
) -> Result<Value> {
    qualification_panel(eps)?;
    let before = qualification_rows(a, initial, initial, eps)?;
    let after = qualification_rows(a, final_eval, initial, eps)?;
    let initial_counts = measurement_counts(&before);
    let final_counts = measurement_counts(&after);
    let mut strata = Vec::new();
    for name in ["length2", "length4", "length8", "update", "reassert"] {
        let old = before
            .iter()
            .filter(|r| r["stratum"] == name)
            .cloned()
            .collect::<Vec<_>>();
        let new = after
            .iter()
            .filter(|r| r["stratum"] == name)
            .cloned()
            .collect::<Vec<_>>();
        strata.push(json!({"stratum":name,"initial":measurement_counts(&old),"final":measurement_counts(&new)}));
    }
    let qualified_fit = final_counts["complete"] == 24;
    let model_keep = outcomes(initial, final_eval)?["keep"] == true;
    Ok(
        json!({"qualification_bar":"24/24 complete own-feedback replies with EOS, including original8",
        "qualified_fit":qualified_fit,"model_keep":model_keep,
        "decision":if qualified_fit {"QUALIFIED_FIT"} else if model_keep {"PARTIAL_GAIN_NOT_QUALIFIED"} else {"REJECT"},
        "initial":initial_counts,"final":final_counts,"by_stratum":strata,
        "initial_rows":before,"final_rows":after,
        "scope":"saved full512 endpoints and unchanged answer oracle; teacher all-token correctness is diagnostic;24-row fit qualification is not milestone256/512 or transfer"}),
    )
}

pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    settings(a)?;
    let parent = ContinuationParent::load(a)?;
    let l = load_joint_continuation(a, &parent, d)?;
    let public = NativeVocabularyActions::new(parent.integer.binding().clone(), &parent.exp)?;
    let legal = public
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    // load_panel encodes the complete canonical answer and appends EOS. This is
    // the existing frozen panel, not a first-source-word objective or new split.
    let train = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    let dev = load_panel(
        &a.development_inputs,
        &a.development_labels,
        &l.integer,
        &l.tokenizer,
        &legal,
        512,
    )?;
    if train.len() != 512 || dev.len() != 512 {
        return Err(bad("reply completion requires both frozen512 panels"));
    }
    pairs(&train)?;
    pairs(&dev)?;
    let (prototype, coefficients): (BTreeMap<_, _>, BTreeMap<_, _>) = l
        .generate
        .parameters()
        .into_iter()
        .partition(|(name, _)| name == "generate.prototype_choices");
    if prototype.is_empty() || coefficients.is_empty() {
        return Err(bad("reply completion Generate parameter groups absent"));
    }
    let potential = l.source.potential_parameters();
    let mut frozen = prototype;
    frozen.extend(
        l.source
            .parameters()
            .into_iter()
            .filter(|(name, _)| !potential.contains_key(name)),
    );
    frozen.extend(l.original_bridge.parameters());
    frozen.extend(
        l.marker
            .parameters()
            .into_iter()
            .map(|(n, v)| (format!("marker.{n}"), v)),
    );
    let frozen_identity = identities(&frozen)?;
    let mut active = coefficients.clone();
    active.extend(potential.clone());
    let qualification = a.mode == Mode::ReplyQualification;
    let mode_name = if qualification {
        "reply_qualification"
    } else {
        "reply_completion"
    };
    let schedule = draw_schedule(a.mode, a.seed)?;
    if qualification {
        let selected = qualification_panel(&train)?;
        qualification_panel(&dev)?;
        write(a, "qualification-panel.json", &selected)?;
    }
    let schedule_policy = if qualification {
        "fixed shuffled24 repeated32 epochs; 96 batches of8; no checkpoint reselection"
    } else {
        "one full512 pass; 64 fixed batches of8; no checkpoint reselection"
    };
    let split = if qualification {
        "fixed24 training subset of the exposed open-development512; full512 evaluation; no held-out claim"
    } else {
        "train and evaluation are the identical exposed open-development512; no held-out claim"
    };
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,
        "policy":schedule_policy}),
    )?;
    write(
        a,
        "admission.json",
        &json!({"mode":mode_name,
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "parent_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
        "parent_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,
        "parent_checkpoint_receipt_sha256":sha256_file(&a.checkpoint.join("receipt.json"))?,
        "training_input_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA,
        "development_input_sha256":INPUT_SHA,"development_labels_sha256":LABEL_SHA,
        "split":split,
        "active_parameter_names":active.keys().collect::<Vec<_>>(),
        "initial_active_masters":identities(&active)?,"frozen_masters":frozen_identity,
        "optimizer":"fresh AdamW, beta1=.9 beta2=.999 eps=1e-8 weight_decay=0",
        "rates":{"generate_coefficients":0.003,"potential":0.003},"updates":a.updates,"batch":8,
        "loss_scope":"all complete-answer tokens including EOS",
        "phase_policy":loss_weight_policy(true,LossScope::All),
        "credit":a.credit.name(),"read_state_pullback":a.read_state_pullback.name(),
        "continuation":"absent",
        "fixed_sidecar_payloads":{"cue_sha256":sha256_bytes(&parent.cue),
            "cue_joint_sha256":parent.joint.as_ref().map(|v|sha256_bytes(v)),
            "prefix_sha256":sha256_bytes(&parent.prefix),"exp_sha256":parent.exp_sha256},
        "routing_scope":"Potential coefficients train and can change physical Source selection; Context and all non-Potential Source masters remain frozen",
        "wall_time":"estimate only; disk/report limits enforced"}),
    )?;
    let (initial_model, initial_generate, initial_bridge, initial_receipt) = checkpoint(a, 0, &l)?;
    if initial_receipt["parent"] != parent.receipt["parent"]
        || initial_generate.to_bytes()? != parent.generate
        || initial_bridge.to_bytes()? != parent.bridge
    {
        return Err(bad("reply completion zero-update parent export differs"));
    }
    let (_, admission) = batch(
        a,
        &l,
        &train,
        &schedule[..BATCH],
        d,
        start,
        Some(&initial_model),
    )?;
    write(a, "zero-update-admission.json", &admission)?;
    let eval_start = Instant::now();
    let initial = evaluate(
        a,
        "development-0000",
        &initial_model,
        &initial_generate,
        Some(&initial_bridge),
        &l.exp,
        &dev,
        &l.tokenizer,
        &l.cue,
        &l.prefix,
        start,
        None,
    )?;
    let initial_metrics = metrics(a, &initial, &dev)?;
    write(a, "metrics-0000.json", &initial_metrics)?;
    let baseline_complete_indices = initial["rows"]
        .as_array()
        .ok_or_else(|| bad("reply baseline rows absent"))?
        .iter()
        .enumerate()
        .filter_map(|(i, row)| (row["complete"] == true).then_some(i))
        .collect::<Vec<_>>();
    if initial["complete"] != 8 || baseline_complete_indices != CONTROL_INDICES {
        return Err(bad(
            "accepted48/64 baseline no longer reproduces exact original8/512",
        ));
    }
    let initial_evaluation_seconds = eval_start.elapsed().as_secs_f64();
    let mut go = optimizer(&coefficients, 0.003)?;
    let mut po = optimizer(&potential, 0.003)?;
    let fit_start = Instant::now();
    let mut updates = Vec::new();
    let checkpoint_every = if qualification { 24 } else { 32 };
    for step in 0..a.updates {
        disk_floor(a)?;
        deadline(a, start)?;
        let indices = &schedule[step * BATCH..(step + 1) * BATCH];
        let (grads, receipt) = batch(a, &l, &train, indices, d, start, None)?;
        let grads = selected_gradients(grads, &coefficients, &potential)?;
        let (denominator, norm) = clip_denominator(&grads, d)?;
        apply(&mut go, &coefficients, &grads, &denominator)?;
        apply(&mut po, &potential, &grads, &denominator)?;
        l.generate.project_shadow_range()?;
        l.source.project_potential_range()?;
        if identities(&frozen)? != frozen_identity {
            return Err(bad(
                "reply completion changed frozen Context/prototype/bridge masters",
            ));
        }
        d.synchronize()?;
        updates.push(json!({"step":step+1,"indices":indices,"before_update":receipt,"global_active_gradient_norm":norm}));
        write(a, "updates.json", &json!(updates))?;
        if (step + 1) % checkpoint_every == 0 {
            checkpoint(a, step + 1, &l)?;
        }
    }
    let fit_seconds_including_checkpoints = fit_start.elapsed().as_secs_f64();
    let final_parent =
        ContinuationParent::from_checkpoint(&a.out.join(format!("checkpoint-{:04}", a.updates)))?;
    if final_parent.cue != parent.cue
        || final_parent.joint != parent.joint
        || final_parent.prefix != parent.prefix
        || final_parent.exp != parent.exp
    {
        return Err(bad(
            "reply completion changed frozen Cue/Prefix/exp payloads",
        ));
    }
    let final_generate = NativeGeometricGenerate::from_bytes(
        &final_parent.generate,
        final_parent.integer.binding(),
    )?;
    let final_bridge = NativeGeometricReadStateBridge::from_bytes(
        &final_parent.bridge,
        final_parent.integer.binding(),
    )?;
    let eval_start = Instant::now();
    let final_eval = evaluate(
        a,
        &format!("development-{:04}", a.updates),
        &final_parent.integer,
        &final_generate,
        Some(&final_bridge),
        &final_parent.exp,
        &dev,
        &final_parent.tokenizer,
        &l.cue,
        &l.prefix,
        start,
        None,
    )?;
    let final_metrics = metrics(a, &final_eval, &dev)?;
    write(a, &format!("metrics-{:04}.json", a.updates), &final_metrics)?;
    let comparison = outcomes(&initial, &final_eval)?;
    write(a, "row-comparison.json", &comparison)?;
    let final_evaluation_seconds = eval_start.elapsed().as_secs_f64();
    let qualification_report = if qualification {
        Some(qualification_outcomes(a, &initial, &final_eval, &dev)?)
    } else {
        None
    };
    if let Some(result) = &qualification_report {
        write(a, "qualification.json", result)?;
    }
    let mut report = json!({"schema":if qualification {"uor-r4.geometric-reply-qualification/1"} else {"uor-r4.geometric-reply-completion/1"},"status":"COMPLETED",
        "mode":mode_name,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "device":"cuda:0","updates":a.updates,"batch":8,"training_row_draws":schedule.len(),
        "target_position_draws":schedule.iter().map(|&i| train[i].target.len()).sum::<usize>(),
        "initial_receipt":initial_receipt,"final_receipt":final_parent.receipt,
        "initial_evaluation":initial,"final_evaluation":final_eval,
        "initial_metrics":initial_metrics,"final_metrics":final_metrics,"outcomes":comparison,
        "initial_evaluation_seconds":initial_evaluation_seconds,
        "fit_seconds_including_checkpoints":fit_seconds_including_checkpoints,
        "final_evaluation_seconds":final_evaluation_seconds,
        "elapsed_seconds":start.elapsed().as_secs_f64(),
        "final_active_masters":identities(&active)?,"frozen_masters":frozen_identity,
        "scope":"ordinary Potential/Generate coefficient learning from accepted48/64 over exposed512 complete-answer/EOS positions; Context/prototypes/bridge/Cue/Prefix fixed; no U, protected constructor or solver; native reloaded own-feedback endpoint; no held-out transfer/chat/energy qualification"});
    if let Some(result) = qualification_report {
        report["qualification"] = result;
        report["training_indices"] = json!(QUALIFICATION_INDICES);
        report["scope"] = json!("ordinary Potential/Generate coefficient learning on fixed24 strata repeated32 epochs; full512 native reloaded own-feedback evaluation; qualified_fit requires24/24; modelKEEP separately requires original8 retention and full512 gain; Context/prototypes/bridge/Cue/Prefix frozen, no U or constructor; no held-out transfer/chat/energy qualification");
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Value {
        json!({"mode":"reply_completion","credit":"raw_identity","read_state_pullback":"categorical",
            "seed":1001,"updates":64,"checkpoint":"cp","saved_fit":"fit","categorical":"cat",
            "parent_config":"parent","training_inputs":"input","training_labels":"labels",
            "development_inputs":"input","development_labels":"labels","maximum_seconds":60,
            "maximum_report_bytes":134217728,"out":"out"})
    }
    #[test]
    fn reply_completion_admission_rejects_other_objectives_and_mechanisms() -> Result<()> {
        settings(&serde_json::from_value(config())?)?;
        for (key, value) in [
            ("updates", json!(32)),
            ("seed", json!(1002)),
            ("credit", json!("clipped")),
            ("loss_scope", json!("entry_only")),
            ("read_state_pullback", json!("legacy")),
            ("native_code_proposals", json!(true)),
            ("query_conditioned_read", json!(true)),
            ("prediction_control_resume", json!("other")),
        ] {
            let mut c = config();
            c[key] = value;
            assert!(settings(&serde_json::from_value(c)?).is_err(), "{key}");
        }
        Ok(())
    }
    #[test]
    fn reply_qualification_fixed_schedule_covers_only_24_rows_32_times() -> Result<()> {
        let schedule = draw_schedule(Mode::ReplyQualification, 1001)?;
        assert_eq!(schedule.len(), 768);
        assert_eq!(schedule.chunks_exact(BATCH).len(), 96);
        let mut counts = BTreeMap::new();
        for &index in &schedule {
            *counts.entry(index).or_insert(0usize) += 1;
        }
        assert_eq!(counts.len(), 24);
        assert_eq!(
            counts.keys().copied().collect::<BTreeSet<_>>(),
            QUALIFICATION_INDICES.into_iter().collect::<BTreeSet<_>>()
        );
        for index in QUALIFICATION_INDICES {
            assert_eq!(counts.get(&index), Some(&32));
        }
        for epoch in schedule.chunks_exact(24) {
            assert_eq!(epoch, &schedule[..24]);
        }
        assert!(CONTROL_INDICES.iter().all(|i| counts.contains_key(i)));
        let legacy = draw_schedule(Mode::ReplyCompletion, 1001)?;
        assert_eq!(legacy, order(1001, 512));
        assert_eq!(legacy.len(), 512);
        assert_eq!(
            legacy.into_iter().collect::<BTreeSet<_>>(),
            (0..512).collect::<BTreeSet<_>>()
        );
        assert!(draw_schedule(Mode::Fit, 1001).is_err());
        Ok(())
    }
    #[test]
    fn reply_qualification_admission_fixes96_and_rejects_experimental_options() -> Result<()> {
        let mut base = config();
        base["mode"] = json!("reply_qualification");
        base["updates"] = json!(96);
        settings(&serde_json::from_value(base.clone())?)?;
        for (key, value) in [
            ("updates", json!(64)),
            ("updates", json!(97)),
            ("seed", json!(1002)),
            ("credit", json!("clipped")),
            ("loss_scope", json!("entry_only")),
            ("read_state_pullback", json!("legacy")),
            ("native_code_proposals", json!(true)),
            ("query_conditioned_read", json!(true)),
            ("prediction_control_resume", json!("other")),
        ] {
            let mut changed = base.clone();
            changed[key] = value;
            assert!(
                settings(&serde_json::from_value(changed)?).is_err(),
                "{key}"
            );
        }
        let mut legacy = config();
        legacy["updates"] = json!(96);
        assert!(settings(&serde_json::from_value(legacy)?).is_err());
        Ok(())
    }
    #[test]
    fn reply_qualification_teacher_success_is_not_complete_reply_success() {
        let rows = vec![json!({"id":"teacher-only","complete":false,
            "teacher_all_tokens_correct":true,"entry_correct":true,"eos":false,"original_complete":true})];
        let counts = measurement_counts(&rows);
        assert_eq!(counts["complete"], 0);
        assert_eq!(counts["teacher_all_tokens_correct"], 1);
        assert_eq!(counts["entry_correct"], 1);
        assert_eq!(counts["eos"], 0);
        assert_eq!(counts["lost_complete_ids"], json!(["teacher-only"]));
    }
    #[test]
    fn reply_completion_gain_does_not_hide_original_reply_loss() -> Result<()> {
        let rows = (0..512)
            .map(|i| {
                json!({"id":format!("row-{i}"),
            "complete":i<8,"generated_ids":[i]})
            })
            .collect::<Vec<_>>();
        let before = json!({"rows":rows});
        let mut after = before.clone();
        after["rows"][8]["complete"] = json!(true);
        assert_eq!(outcomes(&before, &after)?["keep"], true);
        after["rows"][0]["complete"] = json!(false);
        let comparison = outcomes(&before, &after)?;
        assert_eq!(comparison["keep"], false);
        assert_eq!(comparison["lost_complete_ids"], json!(["row-0"]));
        assert_eq!(comparison["gained_complete_ids"], json!(["row-8"]));
        after["rows"][0]["id"] = json!("wrong");
        assert!(outcomes(&before, &after).is_err());
        Ok(())
    }
    #[test]
    fn reply_completion_frozen_gradient_cannot_change_active_clipping() -> Result<()> {
        let d = Device::Cpu;
        let coefficients = BTreeMap::from([("generate.field".into(), Var::new(0f32, &d)?)]);
        let potential = BTreeMap::from([("potential.field".into(), Var::new(0f32, &d)?)]);
        let grads = BTreeMap::from([
            ("generate.field".into(), Tensor::new(3f32, &d)?),
            ("potential.field".into(), Tensor::new(4f32, &d)?),
            ("consumer.context".into(), Tensor::new(1000f32, &d)?),
            (
                "generate.prototype_choices".into(),
                Tensor::new(1000f32, &d)?,
            ),
        ]);
        let filtered = selected_gradients(grads, &coefficients, &potential)?;
        assert_eq!(filtered.len(), 2);
        let (_, norm) = clip_denominator(&filtered, &d)?;
        assert!((norm - 5.).abs() < 1e-6);
        assert!(selected_gradients(BTreeMap::new(), &coefficients, &potential).is_err());
        Ok(())
    }
}
