//! One ordinary all-answer fit from the accepted Source48/Generate64 parent.
//! Only Potential and Generate coefficients train; no U or proposal machinery.
use super::*;

pub(super) fn settings(a: &Args) -> Result<()> {
    if a.mode != Mode::ReplyCompletion {
        return Ok(());
    }
    if a.updates != 64
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
        return Err(bad("reply_completion requires fixed64/seed1001/raw_identity/categorical/all-answer fit without continuation, proposal or diagnostic options"));
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
    let schedule = order(a.seed, train.len());
    write(
        a,
        "order.json",
        &json!({"seed":a.seed,"order":schedule,
        "policy":"one full512 pass; 64 fixed batches of8; no checkpoint reselection"}),
    )?;
    write(
        a,
        "admission.json",
        &json!({"mode":"reply_completion",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "parent_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
        "parent_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,
        "parent_checkpoint_receipt_sha256":sha256_file(&a.checkpoint.join("receipt.json"))?,
        "training_input_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA,
        "development_input_sha256":INPUT_SHA,"development_labels_sha256":LABEL_SHA,
        "split":"train and evaluation are the identical exposed open-development512; no held-out claim",
        "active_parameter_names":active.keys().collect::<Vec<_>>(),
        "initial_active_masters":identities(&active)?,"frozen_masters":frozen_identity,
        "optimizer":"fresh AdamW, beta1=.9 beta2=.999 eps=1e-8 weight_decay=0",
        "rates":{"generate_coefficients":0.003,"potential":0.003},"updates":64,"batch":8,
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
    for step in 0..64 {
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
        if (step + 1) % 32 == 0 {
            checkpoint(a, step + 1, &l)?;
        }
    }
    let fit_seconds_including_checkpoints = fit_start.elapsed().as_secs_f64();
    let final_parent = ContinuationParent::from_checkpoint(&a.out.join("checkpoint-0064"))?;
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
        "development-0064",
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
    write(a, "metrics-0064.json", &final_metrics)?;
    let comparison = outcomes(&initial, &final_eval)?;
    write(a, "row-comparison.json", &comparison)?;
    Ok(
        json!({"schema":"uor-r4.geometric-reply-completion/1","status":"COMPLETED",
        "mode":"reply_completion","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "device":"cuda:0","updates":64,"batch":8,"training_row_draws":512,
        "target_position_draws":train.iter().map(|e| e.target.len()).sum::<usize>(),
        "initial_receipt":initial_receipt,"final_receipt":final_parent.receipt,
        "initial_evaluation":initial,"final_evaluation":final_eval,
        "initial_metrics":initial_metrics,"final_metrics":final_metrics,"outcomes":comparison,
        "initial_evaluation_seconds":initial_evaluation_seconds,
        "fit_seconds_including_checkpoints":fit_seconds_including_checkpoints,
        "final_evaluation_seconds":eval_start.elapsed().as_secs_f64(),
        "elapsed_seconds":start.elapsed().as_secs_f64(),
        "final_active_masters":identities(&active)?,"frozen_masters":frozen_identity,
        "scope":"ordinary Potential/Generate coefficient learning from accepted48/64 over exposed512 complete-answer/EOS positions; Context/prototypes/bridge/Cue/Prefix fixed; no U, protected constructor or solver; native reloaded own-feedback endpoint; no held-out transfer/chat/energy qualification"}),
    )
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
