//! Explicit finite categorical proposals: a conditional two-block construction or
//! one parent-local action intervention. Neither mode refills rejected candidates.
use super::native_proposals as np;
use super::*;

const ACTION: &str = "categorical_bridge.action_choices";

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
struct ActionReplacement {
    lane: usize,
    relative: usize,
    before: usize,
    after: usize,
    before_gradient: f32,
    after_gradient: f32,
    contrast: f64,
}

pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.categorical-context-two-block/1","rounds":1,"maximum_candidates":1,
        "context":"one maximum-L1 row across six shared Context bases; lexical name/row ties; negative-gradient signed-quarter edit with saturation; no refill",
        "action":"recompute the same588 weighted gradients at changed Context; one globally most-negative shared-key g_new-g_current contrast; lane/relative/action ties; onehot replacement; no refill",
        "objective":frontier::OBJECTIVE,"optimizer_updates":0,"global_clipping_applied":false,
        "acceptance":"combined Context+action only; strict decrease >1e-10*(1+abs(parent)); otherwise restore parent",
        "control":"Context-only snapshot is a construction control, never a selection candidate",
        "scope":"conditional two-block surrogate construction with exact native acceptance; not exact hard derivative or guaranteed coadaptation/utility"})
}

pub(super) fn policy_for(action_only: bool) -> Value {
    if !action_only {
        return policy();
    }
    json!({"schema":"uor-r4.categorical-parent-action/1","rounds":1,"maximum_candidates":1,
        "action":"original-parent same588 weighted gradient; one globally most-negative shared-key g_new-g_current contrast; lane/relative/action ties; onehot replacement; no refill",
        "objective":frontier::OBJECTIVE,"optimizer_updates":0,"global_clipping_applied":false,
        "acceptance":"one parent-local action only; strict decrease >1e-10*(1+abs(parent)); otherwise restore parent",
        "frozen":"all Context/Potential/Generate/U/prototype and other masters; no Context construction or conditional gradient recomputation",
        "scope":"parent-local action attribution with exact native acceptance; not joint Context adaptation, permanent Context freezing, exact hard derivative or guaranteed utility"})
}

fn action_replacement(
    values: &[f32],
    gradient: &[f32],
    lanes: usize,
) -> Result<Option<ActionReplacement>> {
    replay_require(
        lanes > 0 && values.len() == lanes * 120 * 120 && gradient.len() == values.len(),
        "categorical action master/gradient shape",
    )?;
    let mut best: Option<ActionReplacement> = None;
    for lane in 0..lanes {
        for relative in 0..120 {
            let offset = (lane * 120 + relative) * 120;
            let row = &values[offset..offset + 120];
            replay_require(
                row.iter().all(|&v| v == 0. || v == 1.)
                    && row.iter().filter(|&&v| v == 1.).count() == 1,
                "categorical proposal requires exact onehot parent rows",
            )?;
            let before = row
                .iter()
                .position(|&v| v == 1.)
                .ok_or_else(|| bad("action row winner absent"))?;
            let g = &gradient[offset..offset + 120];
            replay_require(
                g.iter().all(|v| v.is_finite()),
                "nonfinite shared action gradient",
            )?;
            for after in 0..120 {
                if after == before {
                    continue;
                }
                let contrast = f64::from(g[after]) - f64::from(g[before]);
                // Iteration order is the prospective lane/relative/action tie rule.
                if contrast < 0. && best.as_ref().is_none_or(|old| contrast < old.contrast) {
                    best = Some(ActionReplacement {
                        lane,
                        relative,
                        before,
                        after,
                        before_gradient: g[before],
                        after_gradient: g[after],
                        contrast,
                    });
                }
            }
        }
    }
    Ok(best)
}

fn action_expected(context: &np::Shadows, replacement: &ActionReplacement) -> Result<np::Shadows> {
    let mut expected = context.clone();
    let values = expected
        .get_mut(ACTION)
        .ok_or_else(|| bad("action masters absent"))?;
    let offset = (replacement.lane * 120 + replacement.relative) * 120;
    replay_require(
        replacement.before < 120
            && replacement.after < 120
            && replacement.before != replacement.after
            && values.get(offset + replacement.before) == Some(&1.)
            && values.get(offset + replacement.after) == Some(&0.),
        "action replacement parent differs",
    )?;
    values[offset + replacement.before] = 0.;
    values[offset + replacement.after] = 1.;
    Ok(expected)
}

fn apply_action(
    params: &BTreeMap<String, Var>,
    context: &np::Shadows,
    replacement: &ActionReplacement,
) -> Result<np::Shadows> {
    replay_require(
        np::same_bits(context, &np::snapshot(params)?),
        "action proposal does not start at fixed Context snapshot",
    )?;
    let expected = action_expected(context, replacement)?;
    let var = params
        .get(ACTION)
        .ok_or_else(|| bad("categorical action Var absent"))?;
    var.set(&Tensor::from_vec(
        expected[ACTION].clone(),
        var.shape(),
        var.device(),
    )?)?;
    replay_require(
        np::same_bits(&expected, &np::snapshot(params)?),
        "unselected masters changed during action replacement",
    )?;
    Ok(expected)
}

fn verify_native(
    p: &ContinuationParent,
    field: &NativeContinuationField,
    expected: &np::Shadows,
    lanes: usize,
) -> Result<()> {
    let ordinary = expected
        .iter()
        .filter(|(name, _)| name.as_str() != ACTION)
        .map(|(name, values)| (name.clone(), values.clone()))
        .collect::<np::Shadows>();
    np::verify_codes(p, field, &ordinary)?;
    let bridge = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
    replay_require(
        bridge.lanes() == lanes && sha256_bytes(&p.bridge) == p.bridge_sha256,
        "dynamic categorical bridge binding",
    )?;
    let values = expected
        .get(ACTION)
        .ok_or_else(|| bad("expected categorical masters absent"))?;
    replay_require(
        values.len() == lanes * 120 * 120,
        "native categorical shape",
    )?;
    for lane in 0..lanes {
        for action in 0..120 {
            replay_require(
                bridge.coefficient_bias(lane, action)? == 0,
                "categorical bias changed",
            )?;
            for relative in 0..120 {
                let value = values[(lane * 120 + relative) * 120 + action];
                replay_require(
                    (value == 0. || value == 1.)
                        && bridge.coefficient_relative(lane, action, relative)? == value as i8,
                    "native categorical marker differs from onehot master",
                )?;
            }
        }
    }
    Ok(())
}

fn save_gradients(
    a: &Args,
    directory: &str,
    names: &[String],
    params: &BTreeMap<String, Var>,
    grads: &BTreeMap<String, Tensor>,
) -> Result<Value> {
    let folder = a.out.join(directory);
    fs::create_dir(&folder)?;
    let mut inventory = BTreeMap::new();
    for name in names {
        let var = params
            .get(name)
            .ok_or_else(|| bad("gradient parameter absent"))?;
        let g = grads
            .get(name)
            .ok_or_else(|| bad("required categorical construction gradient absent"))?;
        replay_require(
            g.dims() == var.dims(),
            "categorical construction gradient dimensions",
        )?;
        let values = g.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?;
        replay_require(
            values.iter().all(|v| v.is_finite()),
            "categorical construction gradient nonfinite",
        )?;
        let bytes = values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        let relative = format!("{directory}/{name}.f32le");
        replay_require(
            size(&a.out)? + (bytes.len() as u64) < a.maximum_report_bytes - 1_048_576,
            "categorical gradient report cap",
        )?;
        fs::write(a.out.join(&relative), &bytes)?;
        inventory.insert(name.clone(),json!({"file":relative,"bytes":bytes.len(),"shape":var.dims(),"sha256":sha256_bytes(&bytes)}));
    }
    write(a, &format!("{directory}.json"), &json!(inventory))?;
    Ok(json!(inventory))
}

fn stage_args(a: &Args, name: &str) -> Result<Args> {
    disk_floor(a)?;
    let remaining = a
        .maximum_report_bytes
        .checked_sub(size(&a.out)?)
        .ok_or_else(|| bad("categorical global report cap"))?;
    replay_require(
        remaining > 2_097_152,
        "categorical report reserve exhausted",
    )?;
    let mut stage = a.clone();
    stage.out = a.out.join(name);
    stage.maximum_report_bytes = remaining;
    report_output::claim(&stage.out)?;
    for file in ["reference-plan.json", "frontier-plan.json"] {
        fs::copy(a.out.join(file), stage.out.join(file))?;
    }
    Ok(stage)
}

fn seal_stage(stage: &Args, outcome: &Result<Value>) -> Result<()> {
    let report = match outcome {
        Ok(value) => value.clone(),
        Err(error) => {
            json!({"status":"FAILED","error":error.to_string(),"model_verdict":"execution failure, not candidate quality"})
        }
    };
    write(stage, "report.json", &report)?;
    report_output::seal(&stage.out)?;
    report_output::verify(&stage.out)?;
    Ok(())
}

pub(super) fn run(
    a: &Args,
    l: &Loaded,
    weights: &ContinuationLearningWeights,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
    initial: &ContinuationParent,
    initial_field: &NativeContinuationField,
    eps: &[Episode],
    indices: &[usize],
    plan: &ReferencePlan,
    start: Instant,
    d: &Device,
) -> Result<Value> {
    replay_require(
        a.categorical_action_learning && a.native_code_proposals && a.reached_frontier_objective,
        "categorical proposals require explicit reached/native opt-ins",
    )?;
    replay_require(
        indices == (0..eps.len()).collect::<Vec<_>>().as_slice(),
        "categorical construction requires all episode indices",
    )?;
    if a.categorical_action_only {
        return run_parent_action(
            a,
            l,
            weights,
            params,
            gradients,
            initial,
            initial_field,
            eps,
            plan,
            start,
        );
    }
    let action_var = params
        .get(ACTION)
        .ok_or_else(|| bad("categorical trainable action Var absent"))?;
    replay_require(
        action_var.dims().len() == 3 && action_var.dims()[1..] == [120, 120],
        "categorical action tensor shape",
    )?;
    let lanes = action_var.dims()[0];
    let parent = np::snapshot(params)?;
    let names = np::BASIS
        .iter()
        .map(|n| format!("consumer.context.{n}"))
        .collect::<Vec<_>>();
    let mut g = np::Shadows::new();
    for name in &names {
        let var = params
            .get(name)
            .ok_or_else(|| bad("shared Context basis absent"))?;
        replay_require(
            var.dims().len() == 4 && var.dims()[3] == 4,
            "shared Context row shape",
        )?;
        let gradient = gradients
            .get(name)
            .ok_or_else(|| bad("parent Context gradient absent"))?;
        replay_require(
            gradient.dims() == var.dims(),
            "parent Context gradient shape",
        )?;
        g.insert(
            name.clone(),
            gradient
                .flatten_all()?
                .to_device(&Device::Cpu)?
                .to_vec1::<f32>()?,
        );
    }
    let context = np::ranked_pair("context", &names, 4, &parent, &g)?
        .into_iter()
        .next()
        .ok_or_else(|| bad("Context proposal absent"))?;
    replay_require(
        context.direction == -1,
        "Context proposal direction differs",
    )?;
    let reached = frontier::read_plan(a)?;
    let (task_mask, success_mask) = frontier::components(&reached)?;
    let terms = reached
        .terms
        .iter()
        .map(|t| np::Term {
            index: t.index,
            position: t.position,
            target: t.target,
            component: t.component,
            weight: t.weight,
            parent_actual_prefix_ids: Some(t.parent_actual_prefix_ids.clone()),
        })
        .collect::<Vec<_>>();
    replay_require(terms.len() == 588, "categorical objective population")?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    let parent_gradients =
        save_gradients(a, "categorical-parent-gradients", &names, params, gradients)?;
    write(
        a,
        "categorical-construction-plan.json",
        &json!({"policy":policy(),"context":context,"parent_master_identities":identities(params)?,
        "parent_gradients":parent_gradients,"objective_terms_sha256":sha256_file(&a.out.join("native-code-objective-terms.json"))?,
        "frontier_plan_sha256":sha256_file(&a.out.join("frontier-plan.json"))?,"parent_receipt":initial.receipt}),
    )?;
    verify_native(initial, initial_field, &parent, lanes)?;
    let baseline = np::score(a, initial, initial_field, eps, plan, &terms, start)?;
    let expected = read(&a.out.join("frontier-initial-validation.json"))?;
    for (key, saved) in [
        ("task", "frontier_losses_before_after"),
        ("reference", "success_losses_before_after"),
        ("combined", "combined_losses_before_after"),
    ] {
        let x = baseline[key]
            .as_f64()
            .ok_or_else(|| bad("baseline component absent"))?;
        let y = expected[saved][0]
            .as_f64()
            .ok_or_else(|| bad("saved baseline component absent"))?;
        replay_require(
            (x - y).abs() <= 1e-10 * (1. + y.abs()),
            "categorical parent objective differs from evaluation",
        )?;
    }
    write(a, "native-code-parent-objective.json", &baseline)?;
    let baseline_loss = np::objective(&baseline)?;
    let mut checkpoint_seconds = 0.;
    let mut native_seconds = baseline["elapsed_seconds"]
        .as_f64()
        .ok_or_else(|| bad("baseline timing absent"))?;
    let mut native_calls = terms.len();
    let mut conditional_gradient_seconds = 0.;
    let mut replacement = None;
    let mut paired_expected = None;
    let mut paired_loss = None;
    let mut construction = Value::Null;
    let mut candidate = Value::Null;
    if !context.edits.is_empty() {
        let attempted = np::attempt_restored(params, &parent, || -> Result<()> {
            deadline(a, start)?;
            np::apply_edits(params, &parent, &context.edits)?;
            let context_expected = np::edited(&parent, &context.edits)?;
            let control = stage_args(a, "categorical-context-control")?;
            let control_result = (|| -> Result<Value> {
                let clock = Instant::now();
                let (p, field, receipt) = joint_checkpoint(&control, 0, l, weights)?;
                let seconds = clock.elapsed().as_secs_f64();
                checkpoint_seconds += seconds;
                verify_native(&p, &field, &context_expected, lanes)?;
                let measured = np::score(&control, &p, &field, eps, plan, &terms, start)?;
                native_seconds += measured["elapsed_seconds"]
                    .as_f64()
                    .ok_or_else(|| bad("control timing absent"))?;
                native_calls += terms.len();
                write(&control, "objective.json", &measured)?;
                let clock = Instant::now();
                let (task, task_receipt) = batch_live_weighted(
                    a,
                    l,
                    eps,
                    indices,
                    d,
                    start,
                    None,
                    Some(weights),
                    Some(&task_mask),
                )?;
                let (success, success_receipt) = batch_live_weighted(
                    a,
                    l,
                    eps,
                    indices,
                    d,
                    start,
                    None,
                    Some(weights),
                    Some(&success_mask),
                )?;
                replay_require(
                    np::same_bits(&context_expected, &np::snapshot(params)?),
                    "masters changed during conditional action credit",
                )?;
                for key in [
                    "current_source_binding",
                    "current_generate_sha256",
                    "current_continuation_sha256",
                    "current_categorical_sha256",
                    "current_categorical_master_sha256",
                ] {
                    replay_require(
                        task_receipt[key] == success_receipt[key],
                        "conditional gradient native snapshots differ",
                    )?;
                }
                replay_require(
                    task_receipt["current_source_binding"] == serde_json::to_value(&p.binding)?
                        && task_receipt["current_generate_sha256"] == p.generate_sha256
                        && task_receipt["current_continuation_sha256"]
                            == sha256_bytes(&field.to_bytes()?)
                        && task_receipt["current_categorical_sha256"] == p.bridge_sha256
                        && task_receipt["current_categorical_master_sha256"]
                            == sha256_bytes(
                                &context_expected[ACTION]
                                    .iter()
                                    .flat_map(|v| v.to_le_bytes())
                                    .collect::<Vec<_>>(),
                            ),
                    "conditional graph/control artifact binding differs",
                )?;
                replay_require(
                    task_receipt["positions"] == 504 && success_receipt["positions"] == 84,
                    "conditional gradient objective coverage differs",
                )?;
                for (observed, key) in [
                    (&task_receipt["weighted_native_loss"], "task"),
                    (&success_receipt["weighted_native_loss"], "reference"),
                ] {
                    let x = observed
                        .as_f64()
                        .ok_or_else(|| bad("conditional graph loss absent"))?;
                    let y = measured[key]
                        .as_f64()
                        .ok_or_else(|| bad("conditional native loss absent"))?;
                    replay_require(
                        (x - y).abs() <= 1e-10 * (1. + y.abs()),
                        "conditional graph/native objective differs",
                    )?;
                }
                let combined = combine_reference_gradients(&task, &success, 1.)?;
                let action_gradient = combined
                    .get(ACTION)
                    .ok_or_else(|| bad("no gradient-bearing categorical action path"))?;
                replay_require(
                    action_gradient.dims() == action_var.dims(),
                    "conditional action gradient shape",
                )?;
                let host = action_gradient
                    .flatten_all()?
                    .to_device(&Device::Cpu)?
                    .to_vec1::<f32>()?;
                replacement = action_replacement(&context_expected[ACTION], &host, lanes)?;
                let inventory = save_gradients(
                    &control,
                    "action-gradients",
                    &[ACTION.into()],
                    params,
                    &combined,
                )?;
                conditional_gradient_seconds = clock.elapsed().as_secs_f64();
                write(
                    &control,
                    "conditional-credit.json",
                    &json!({"task":task_receipt,"success":success_receipt,"lambda":1.,
                    "action_gradient_inventory":inventory,"selected_action":replacement,"same_master_snapshot":true,
                    "shared_key_aggregation":"sum all weighted token contributions before g_new-g_current ranking", "elapsed_seconds":conditional_gradient_seconds}),
                )?;
                Ok(
                    json!({"status":"COMPLETED","selection_eligible":false,"context":context,"receipt":receipt,
                    "checkpoint_seconds":seconds,"native_objective":measured,"selected_action":replacement,
                    "scope":"changed-Context construction control, never independently accepted"}),
                )
            })();
            seal_stage(&control, &control_result)?;
            construction = control_result?;
            if let Some(action) = &replacement {
                deadline(a, start)?;
                let proposed = apply_action(params, &context_expected, action)?;
                let out = stage_args(a, "native-candidate-00")?;
                let result = (|| -> Result<Value> {
                    let clock = Instant::now();
                    let (p, field, receipt) = joint_checkpoint(&out, 0, l, weights)?;
                    let seconds = clock.elapsed().as_secs_f64();
                    checkpoint_seconds += seconds;
                    verify_native(&p, &field, &proposed, lanes)?;
                    let measured = np::score(&out, &p, &field, eps, plan, &terms, start)?;
                    native_seconds += measured["elapsed_seconds"]
                        .as_f64()
                        .ok_or_else(|| bad("paired timing absent"))?;
                    native_calls += terms.len();
                    write(&out, "objective.json", &measured)?;
                    paired_loss = Some(np::objective(&measured)?);
                    paired_expected = Some(proposed);
                    Ok(
                        json!({"status":"COMPLETED","context":context,"action":action,"receipt":receipt,
                        "combined":paired_loss,"checkpoint_seconds":seconds,"native_evaluation_seconds":measured["elapsed_seconds"],
                        "objective_sha256":sha256_file(&out.out.join("objective.json"))?,"optimizer_updates":0,
                        "both_blocks_changed":true,"exact_native_code_and_map_edits_verified":true}),
                    )
                })();
                seal_stage(&out, &result)?;
                candidate = result?;
            }
            Ok(())
        });
        if let Err(error) = attempted {
            let restored = np::snapshot(params)
                .map(|v| np::same_bits(&parent, &v))
                .unwrap_or(false);
            write(
                a,
                "categorical-construction-failure.json",
                &json!({"error":error.to_string(),"all_parent_bits_restored":restored,
                "model_verdict":"execution failure, not candidate quality"}),
            )?;
            return Err(error);
        }
    }
    replay_require(
        np::same_bits(&parent, &np::snapshot(params)?),
        "categorical construction did not restore parent",
    )?;
    let accepted =
        paired_loss.is_some_and(|loss| loss < baseline_loss - 1e-10 * (1. + baseline_loss.abs()));
    let result = (|| -> Result<Value> {
        if accepted {
            let expected = paired_expected
                .as_ref()
                .ok_or_else(|| bad("paired accepted masters absent"))?;
            np::restore(params, expected)?;
        }
        let status = if context.edits.is_empty() {
            "NO_NONTRIVIAL_CONTEXT_PROPOSAL"
        } else if replacement.is_none() {
            "NO_NEGATIVE_SHARED_ACTION_CONTRAST"
        } else if accepted {
            "NATIVE_PAIRED_OBJECTIVE_DESCENT_SELECTED"
        } else {
            "PAIRED_NATIVE_OBJECTIVE_NOT_IMPROVED"
        };
        let summary = json!({"policy":policy(),"status":status,"winner":if accepted{Some(0)}else{None},
            "winner_family":if accepted{Some("context+categorical_action")}else{None},"parent_combined":baseline_loss,
            "selected_combined":if accepted{paired_loss.unwrap_or(baseline_loss)}else{baseline_loss},
            "paired_combined":paired_loss,"accepted_code_proposals":usize::from(accepted),"optimizer_updates":0,
            "context":context,"action":replacement,"construction_control":construction,"paired_candidate":candidate,
            "candidate_checkpoint_seconds":checkpoint_seconds,"native_objective_seconds":native_seconds,
            "native_objective_calls_before_final":native_calls,"conditional_gradient_seconds":conditional_gradient_seconds,
            "all_parent_bits_restored_before_selection":true,"selected_master_identities":identities(params)?,
            "context_candidates_evaluated":0,"context_construction_controls":usize::from(!context.edits.is_empty()),
            "categorical_construction_plan_sha256":sha256_file(&a.out.join("categorical-construction-plan.json"))?});
        write(a, "native-code-proposals.json", &summary)?;
        Ok(summary)
    })();
    if result.is_err() {
        return np::attempt_restored(params, &parent, || result);
    }
    result
}

/// The supplied gradient is the original-parent combined588-term gradient.
/// This path never changes Context or performs another backward pass.
fn run_parent_action(
    a: &Args,
    l: &Loaded,
    weights: &ContinuationLearningWeights,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
    initial: &ContinuationParent,
    initial_field: &NativeContinuationField,
    eps: &[Episode],
    plan: &ReferencePlan,
    start: Instant,
) -> Result<Value> {
    deadline(a, start)?;
    let action_var = params
        .get(ACTION)
        .ok_or_else(|| bad("parent action Var absent"))?;
    replay_require(
        action_var.dims().len() == 3 && action_var.dims()[1..] == [120, 120],
        "parent action shape",
    )?;
    let lanes = action_var.dims()[0];
    let parent = np::snapshot(params)?;
    let gradient = gradients
        .get(ACTION)
        .ok_or_else(|| bad("original-parent action gradient absent"))?;
    replay_require(
        gradient.dims() == action_var.dims(),
        "parent action gradient shape",
    )?;
    let host = gradient
        .flatten_all()?
        .to_device(&Device::Cpu)?
        .to_vec1::<f32>()?;
    let replacement = action_replacement(&parent[ACTION], &host, lanes)?;
    let reached = frontier::read_plan(a)?;
    let terms = reached
        .terms
        .iter()
        .map(|t| np::Term {
            index: t.index,
            position: t.position,
            target: t.target,
            component: t.component,
            weight: t.weight,
            parent_actual_prefix_ids: Some(t.parent_actual_prefix_ids.clone()),
        })
        .collect::<Vec<_>>();
    replay_require(terms.len() == 588, "parent action objective population")?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    let inventory = save_gradients(
        a,
        "categorical-parent-gradients",
        &[ACTION.into()],
        params,
        gradients,
    )?;
    write(
        a,
        "categorical-construction-plan.json",
        &json!({
            "policy":policy_for(true), "action":replacement, "parent_master_identities":identities(params)?,
            "parent_gradients":inventory, "objective_terms_sha256":sha256_file(&a.out.join("native-code-objective-terms.json"))?,
            "frontier_plan_sha256":sha256_file(&a.out.join("frontier-plan.json"))?, "parent_receipt":initial.receipt,
            "gradient_scope":"original-parent combined588 objective; weighted shared-key contributions summed before contrast ranking",
            "parent_action_master_sha256":sha256_bytes(&parent[ACTION].iter().flat_map(|v|v.to_le_bytes()).collect::<Vec<_>>()),
            "additional_backward_passes":0
        }),
    )?;
    verify_native(initial, initial_field, &parent, lanes)?;
    let baseline = np::score(a, initial, initial_field, eps, plan, &terms, start)?;
    let expected = read(&a.out.join("frontier-initial-validation.json"))?;
    for (key, saved) in [
        ("task", "frontier_losses_before_after"),
        ("reference", "success_losses_before_after"),
        ("combined", "combined_losses_before_after"),
    ] {
        let x = baseline[key]
            .as_f64()
            .ok_or_else(|| bad("parent action baseline component absent"))?;
        let y = expected[saved][0]
            .as_f64()
            .ok_or_else(|| bad("saved parent action baseline absent"))?;
        replay_require(
            (x - y).abs() <= 1e-10 * (1. + y.abs()),
            "parent action native objective differs from initial evaluation",
        )?;
    }
    write(a, "native-code-parent-objective.json", &baseline)?;
    let baseline_loss = np::objective(&baseline)?;
    let mut checkpoint_seconds = 0.;
    let mut native_seconds = baseline["elapsed_seconds"]
        .as_f64()
        .ok_or_else(|| bad("parent action baseline timing absent"))?;
    let mut native_calls = terms.len();
    let mut candidate = Value::Null;
    let mut candidate_expected = None;
    let mut candidate_loss = None;
    if let Some(action) = &replacement {
        let attempted = np::attempt_restored(params, &parent, || -> Result<()> {
            deadline(a, start)?;
            let proposed = apply_action(params, &parent, action)?;
            let out = stage_args(a, "native-candidate-00")?;
            let result = (|| -> Result<Value> {
                let clock = Instant::now();
                let (p, field, receipt) = joint_checkpoint(&out, 0, l, weights)?;
                checkpoint_seconds = clock.elapsed().as_secs_f64();
                verify_native(&p, &field, &proposed, lanes)?;
                let measured = np::score(&out, &p, &field, eps, plan, &terms, start)?;
                native_seconds += measured["elapsed_seconds"]
                    .as_f64()
                    .ok_or_else(|| bad("parent action candidate timing absent"))?;
                native_calls += terms.len();
                write(&out, "objective.json", &measured)?;
                candidate_loss = Some(np::objective(&measured)?);
                candidate_expected = Some(proposed);
                Ok(
                    json!({"status":"COMPLETED", "mode":"parent_action_only", "action":action,
                    "receipt":receipt, "combined":candidate_loss, "checkpoint_seconds":checkpoint_seconds,
                    "native_evaluation_seconds":measured["elapsed_seconds"],
                    "objective_sha256":sha256_file(&out.out.join("objective.json"))?, "optimizer_updates":0,
                    "only_action_master_changed":true, "exact_native_code_and_map_edits_verified":true}),
                )
            })();
            seal_stage(&out, &result)?;
            candidate = result?;
            Ok(())
        });
        if let Err(error) = attempted {
            let restored = np::snapshot(params)
                .map(|v| np::same_bits(&parent, &v))
                .unwrap_or(false);
            write(
                a,
                "categorical-construction-failure.json",
                &json!({"mode":"parent_action_only",
                "error":error.to_string(),"all_parent_bits_restored":restored,
                "model_verdict":"execution failure, not candidate quality"}),
            )?;
            return Err(error);
        }
    }
    replay_require(
        np::same_bits(&parent, &np::snapshot(params)?),
        "parent action construction did not restore parent",
    )?;
    let accepted = candidate_loss
        .is_some_and(|loss| loss < baseline_loss - 1e-10 * (1. + baseline_loss.abs()));
    let result = (|| -> Result<Value> {
        if accepted {
            np::restore(
                params,
                candidate_expected
                    .as_ref()
                    .ok_or_else(|| bad("accepted parent action masters absent"))?,
            )?;
        }
        let status = if replacement.is_none() {
            "NO_NEGATIVE_SHARED_ACTION_CONTRAST"
        } else if accepted {
            "NATIVE_PARENT_ACTION_OBJECTIVE_DESCENT_SELECTED"
        } else {
            "PARENT_ACTION_NATIVE_OBJECTIVE_NOT_IMPROVED"
        };
        let summary = json!({"policy":policy_for(true),"status":status,"winner":if accepted{Some(0)}else{None},
            "winner_family":if accepted{Some("categorical_action")}else{None},"parent_combined":baseline_loss,
            "selected_combined":if accepted{candidate_loss.unwrap_or(baseline_loss)}else{baseline_loss},
            "parent_action_combined":candidate_loss,"accepted_code_proposals":usize::from(accepted),"optimizer_updates":0,
            "action":replacement,"parent_action_candidate":candidate,
            "candidate_checkpoint_seconds":checkpoint_seconds,"native_objective_seconds":native_seconds,
            "native_objective_calls_before_final":native_calls,"conditional_gradient_seconds":0.,"additional_backward_passes":0,
            "all_parent_bits_restored_before_selection":true,"selected_master_identities":identities(params)?,
            "categorical_construction_plan_sha256":sha256_file(&a.out.join("categorical-construction-plan.json"))?});
        write(a, "native-code-proposals.json", &summary)?;
        Ok(summary)
    })();
    if result.is_err() {
        return np::attempt_restored(params, &parent, || result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn onehot(lanes: usize) -> Vec<f32> {
        let mut v = vec![0.; lanes * 120 * 120];
        for row in v.chunks_exact_mut(120) {
            row[1] = 1.;
        }
        v
    }
    #[test]
    fn shared_action_contrasts_cancel_constants_and_respect_conflicts() -> Result<()> {
        let values = onehot(1);
        let constant = vec![3.; values.len()];
        assert!(action_replacement(&values, &constant, 1)?.is_none());
        let mut gradients = vec![0.; values.len()];
        // Two uses of the same key prefer opposite changes: weighted utilities sum first.
        gradients[2] = (-5f32) + 6.;
        assert!(action_replacement(&values, &gradients, 1)?.is_none());
        gradients[2] = -2.;
        gradients[3] = -2.;
        gradients[120 + 2] = -2.;
        let choice = action_replacement(&values, &gradients, 1)?
            .ok_or_else(|| bad("missing fixture action"))?;
        assert_eq!(
            (choice.lane, choice.relative, choice.before, choice.after),
            (0, 0, 1, 2)
        );
        assert_eq!(choice.contrast, -2.);
        Ok(())
    }
    #[test]
    fn parent_action_ranks_contrast_not_absolute_gradient_and_preserves_policy() -> Result<()> {
        let values = onehot(2);
        let mut gradients = vec![0.; values.len()];
        // The lowest absolute gradient is only a constant shift at another key.
        gradients[..120].fill(-100.);
        let offset = (120 + 4) * 120;
        gradients[offset + 1] = 3.;
        gradients[offset + 7] = -2.;
        gradients[offset + 8] = -2.;
        let chosen = action_replacement(&values, &gradients, 2)?
            .ok_or_else(|| bad("parent action contrast absent"))?;
        assert_eq!(
            (chosen.lane, chosen.relative, chosen.before, chosen.after),
            (1, 4, 1, 7)
        );
        assert_eq!(chosen.contrast, -5.);
        assert_eq!(policy_for(false), policy());
        assert_eq!(
            policy_for(true)["schema"],
            "uor-r4.categorical-parent-action/1"
        );
        assert_ne!(policy_for(true), policy_for(false));
        Ok(())
    }
    #[test]
    fn global_context_row_ranking_does_not_refill_a_saturated_winner() -> Result<()> {
        let names = vec![
            "consumer.context.z".to_owned(),
            "consumer.context.a".to_owned(),
        ];
        let parent = np::Shadows::from([
            (names[0].clone(), vec![0.; 8]),
            (names[1].clone(), vec![-1.75; 8]),
        ]);
        let gradients = np::Shadows::from([
            (names[0].clone(), vec![1.; 8]),
            (names[1].clone(), vec![2.; 8]),
        ]);
        let proposals = np::ranked_pair("context", &names, 4, &parent, &gradients)?;
        assert_eq!(proposals[0].name, names[1]);
        assert_eq!(proposals[0].row, 0);
        assert_eq!(proposals[0].direction, -1);
        assert!(proposals[0].edits.is_empty());
        // The opposite sign and lower-ranked rows exist but are not fallback candidates.
        assert!(!proposals[1].edits.is_empty());
        Ok(())
    }
    #[test]
    fn action_edit_changes_only_two_onehot_coordinates_and_restores_on_error() -> Result<()> {
        let mut values = onehot(1);
        values[7] = -0.;
        let params = BTreeMap::from([
            (
                ACTION.into(),
                Var::from_vec(values, (1, 120, 120), &Device::Cpu)?,
            ),
            (
                "other".into(),
                Var::from_vec(vec![0.031f32, -0.], 2, &Device::Cpu)?,
            ),
        ]);
        let parent = np::snapshot(&params)?;
        let action = ActionReplacement {
            lane: 0,
            relative: 0,
            before: 1,
            after: 4,
            before_gradient: 0.,
            after_gradient: -1.,
            contrast: -1.,
        };
        let expected = action_expected(&parent, &action)?;
        assert_eq!(expected[ACTION][1], 0.);
        assert_eq!(expected[ACTION][4], 1.);
        assert_eq!(expected[ACTION][7].to_bits(), parent[ACTION][7].to_bits());
        assert_eq!(
            expected["other"]
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>(),
            parent["other"]
                .iter()
                .map(|v| v.to_bits())
                .collect::<Vec<_>>()
        );
        let result: Result<()> = np::attempt_restored(&params, &parent, || {
            apply_action(&params, &parent, &action)?;
            Err(bad("synthetic paired export failure"))
        });
        assert!(result.is_err());
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        Ok(())
    }
}
