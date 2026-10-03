//! Zero-update two-factor crossover. Saved endpoints are never recomputed.
use super::*;
#[derive(Deserialize)]
struct EventRow {
    index: usize,
    episode: data::Episode,
    q4_trace: Value,
}
fn equal(actual: &impl Serialize, expected: &Value, label: &str) -> Result<()> {
    if serde_json::to_value(actual)? != *expected {
        return Err(invalid(format!("fixed-event invariant differs:{label}")));
    }
    Ok(())
}
fn endpoint(dir: &Path, panel: &Panel, row: &PriorRow) -> Result<(Value, String)> {
    let bytes = fs::read(
        dir.join(panel.name)
            .join(format!("row-{:03}.json", row.index)),
    )?;
    let value: Value = serde_json::from_slice(&bytes)?;
    if value["index"] != row.index || value["episode"] != serde_json::to_value(&row.episode)? {
        return Err(invalid("crossover endpoint row identity differs"));
    }
    Ok((value, sha256_bytes(&bytes)))
}
fn saved_answer(v: &Value, target: usize) -> Result<(usize, Vec<f32>, f64)> {
    let x: Vec<f32> = serde_json::from_value(v["answer_logits"]["native"].clone())?;
    let t = Tensor::from_vec(x, (1, 40), &Device::Cpu)?;
    let (p, l, c) = answer(&t, 0, target)?;
    if v["native_prediction"] != p {
        return Err(invalid("saved endpoint prediction/logits differ"));
    }
    Ok((p, l, c))
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    out: &Path,
    fit_root: &Path,
    base: &Path,
    tokenizer: &[u8],
    binding: &ReadSourceBinding,
    parent: &Value,
    parent_event: &EventWeights,
    parent_age: &Tensor,
    panels: &[Panel],
    deadline: Instant,
    forward: impl Fn(&[u32], &EventWeights, &Tensor) -> Result<(Tensor, EventAgeCreditOutput)>,
) -> Result<Value> {
    report_output::verify(fit_root)?;
    let report_bytes = fs::read(fit_root.join("report.json"))?;
    let report: Value = serde_json::from_slice(&report_bytes)?;
    let inputs_bytes = fs::read(fit_root.join("inputs.json"))?;
    let inputs: Value = serde_json::from_slice(&inputs_bytes)?;
    let args: Value = serde_json::from_slice(&fs::read(fit_root.join("arguments.json"))?)?;
    if parent["seed"] != 2
        || report["seed"] != 2
        || report["schema"] != "uor-r4.retained-reader-event-age-fit/1"
        || report["complete"] != true
        || report["fit"]["completed"] != true
        || report["fit"]["completed_updates"] != 640
        || report["baseline"]["complete"] != true
        || report["baseline"]["rows"] != 256
        || report["final"]["complete"] != true
        || report["final"]["rows"] != 256
        || report["frozen_files_unchanged"] != true
        || report["frozen_parameters_unchanged"] != true
        || report["independent_event_age_reload_exact"] != true
        || report["hard_backward_policy"] != POLICY
        || inputs != *parent
        || args["credit_check"] != parent["credit_check"]
        || !args["counterfactual_fit"].is_null()
    {
        return Err(invalid(
            "completed sealed same-parent seed2 fit required for crossover",
        ));
    }
    let initial_rng = parent["initial_generator_state"]
        .as_u64()
        .ok_or_else(|| invalid("initial RNG absent"))?;
    if report["fit"]["initial_generator_state"].as_u64() != Some(initial_rng) {
        return Err(invalid("fit initial RNG differs"));
    }
    let cp = fit_root.join("checkpoints/step-0640");
    report_output::verify(&cp)?;
    let cp_meta: Value = serde_json::from_slice(&fs::read(cp.join("checkpoint.json"))?)?;
    if cp_meta["completed_updates"] != 640
        || cp_meta["next_absolute_step"] != 3840
        || cp_meta["parent"] != *parent
        || cp_meta["next_generator_state"] != report["fit"]["next_generator_state"]
    {
        return Err(invalid("final checkpoint identity differs"));
    }
    let es = fit_root.join("event-source");
    let ages = fit_root.join("age-source");
    let final_files = snapshot(&[("event", es.as_path()), ("age", ages.as_path())])?;
    if final_files
        != snapshot(&[
            ("event", cp.join("event-source").as_path()),
            ("age", cp.join("age-source").as_path()),
        ])?
    {
        return Err(invalid(
            "final raw source files differ from completed checkpoint",
        ));
    }
    let event = EventWeights::load_source(&es, base, tokenizer)?;
    let age_source = AgeSource::load(&ages, binding)?;
    let age = age_source.tensor()?;
    let parent_var = Var::from_tensor(parent_age)?;
    let final_var = Var::from_tensor(&age)?;
    let parent_parameters = fit::parameter_hashes(parent_event, &parent_var)?;
    let final_parameters = fit::parameter_hashes(&event, &final_var)?;
    if serde_json::to_value(&parent_parameters)? != parent["initial_parameters"]
        || serde_json::to_value(&final_parameters)? != cp_meta["parameters"]
        || serde_json::to_value(hashes(parent_event.parameters())?)?
            != report["event_initial_parameters"]
        || report["age_initial_bits_sha256"]
            != sha256_bytes(
                &bits(parent_age)?
                    .iter()
                    .flat_map(|x| x.to_le_bytes())
                    .collect::<Vec<_>>(),
            )
    {
        return Err(invalid("raw parent/final shadow identity differs"));
    }
    let parent_age_source = AgeSource::new(parent_age, binding)?;
    let conversion = path(parent, "conversion_attempt")?;
    write(
        &out.join("crossover-inputs.json"),
        &json!({"fit_root":fit_root,"fit_report_sha256":sha256_bytes(&report_bytes),
        "fit_inputs_sha256":sha256_bytes(&inputs_bytes),"source_files":final_files,
        "parent_parameters":parent_parameters,"final_parameters":final_parameters,"parent":parent,
        "optimizer_updates":0,"new_predictions_requested":512,"RNG":"no sampler or generator call",
        "arms":["event_final_age_parent","event_parent_age_final"],"policy":POLICY}),
    )?;
    let mut summaries = Vec::new();
    let mut predictions = 0usize;
    let started = Instant::now();
    for panel in panels {
        let events_bytes = fs::read(conversion.join(format!("{}-event-rows.json", panel.name)))?;
        let saved_events: Vec<EventRow> = serde_json::from_slice(&events_bytes)?;
        if saved_events.len() != 128
            || saved_events
                .iter()
                .zip(&panel.rows)
                .any(|(a, b)| a.index != b.index || a.episode != b.episode)
        {
            return Err(invalid(
                "saved parent event rows differ from crossover panel",
            ));
        }
        let mut correct = [0i64; 4];
        let mut ce_sums = [0.; 4];
        let mut rows = 0usize;
        let mut gains = [0usize; 2];
        let mut losses = [0usize; 2];
        let dir = out.join(panel.name);
        fs::create_dir(&dir)?;
        for (r, saved_event) in panel.rows.iter().zip(&saved_events) {
            if Instant::now() >= deadline {
                break;
            }
            let e = &r.episode;
            let (old, old_hash) = endpoint(&fit_root.join("baseline"), panel, r)?;
            let (new, new_hash) = endpoint(&fit_root.join("final"), panel, r)?;
            let (op, ol, oc) = saved_answer(&old, e.answer as usize)?;
            let (np, nl, nc) = saved_answer(&new, e.answer as usize)?;
            if op != r.native_prediction
                || ol.iter().map(|x| x.to_bits()).ne(r
                    .answer_logits
                    .native
                    .iter()
                    .map(|x| x.to_bits()))
            {
                return Err(invalid(
                    "saved parent-parent endpoint differs from conversion",
                ));
            }
            let mut arm_rows = Vec::new();
            let mut arm_answers = Vec::new();
            for arm in 0..2 {
                if Instant::now() >= deadline {
                    break;
                }
                let (ew, aw, reference, expected_age) = if arm == 0 {
                    (&event, parent_age, &new, parent_age_source.age_q24())
                } else {
                    (parent_event, &age, &old, age_source.age_q24())
                };
                let (logits, o) = forward(&e.ids, ew, aw)?;
                let (p, l, c) = answer(&logits, e.query, e.answer as usize)?;
                equal(
                    &o.values.trace,
                    &reference["values"],
                    "packets and primitive Q16",
                )?;
                equal(
                    &o.read.values_q16,
                    &reference["read"]["values_q16"],
                    "composed Q16 payload",
                )?;
                equal(
                    &o.read.no_read_q24,
                    &reference["read"]["no_read_q24"],
                    "raw NoRead",
                )?;
                if o.age_q24 != expected_age {
                    return Err(invalid("crossed age differs from exact chosen source"));
                }
                let actual_event = trace_record(&o.event.trace);
                if arm == 0 {
                    equal(
                        &actual_event,
                        &new["current"]["event"],
                        "final event state/actions",
                    )?;
                    equal(&o.span, &new["current"]["span"], "final OLD held")?;
                    equal(
                        &o.potential.scores_q24,
                        &new["current"]["potential_q24"],
                        "final raw potential",
                    )?;
                } else {
                    equal(
                        &actual_event,
                        &saved_event.q4_trace,
                        "parent event state/actions",
                    )?;
                }
                arm_answers.push((p, c));
                predictions += 1;
                arm_rows.push(json!({"arm":if arm==0{"event_final_age_parent"}else{"event_parent_age_final"},
                    "prediction":p,"answer_logits":l,"answer_ce":c,"event":actual_event,"span":o.span,
                    "values":o.values.trace,"potential_q24":o.potential.scores_q24,"no_read_q24":o.no_read.scores_q24,
                    "age_q24":o.age_q24,"read":o.read,"policy":o.policy,
                    "fixed_event_packets_composed_payload_null_and_events_equal":true,
                    "fixed_event_potential_and_held_equal":if arm==0{json!(true)}else{Value::Null},
                    "parent_endpoint_limit":"baseline has no saved current potential/held; these new raw fields are retained, not falsely marked independently equal"}));
            }
            // Preserve even one completed arm if the deadline interrupts the pair.
            write(
                &dir.join(format!("row-{:03}.json", r.index)),
                &json!({"index":r.index,"episode":e,
                "saved_endpoints":{"parent_parent":{"prediction":op,"answer_logits":ol,"answer_ce":oc,"row_sha256":old_hash},
                    "final_final":{"prediction":np,"answer_logits":nl,"answer_ce":nc,"row_sha256":new_hash}},"arms":arm_rows}),
            )?;
            if arm_answers.len() != 2 {
                break;
            }
            let results = [(op, oc), arm_answers[0], arm_answers[1], (np, nc)];
            for (i, (p, c)) in results.into_iter().enumerate() {
                correct[i] += i64::from(p == e.answer as usize);
                ce_sums[i] += c;
            }
            for (i, (p, _)) in arm_answers.iter().enumerate() {
                gains[i] += usize::from(*p == e.answer as usize && op != e.answer as usize);
                losses[i] += usize::from(*p != e.answer as usize && op == e.answer as usize);
            }
            rows += 1;
            write(
                &out.join("progress.json"),
                &json!({"new_predictions":predictions,"current_panel":panel.name,"complete_pairs":rows,"optimizer_updates":0}),
            )?;
        }
        summaries.push(json!({"panel":panel.name,"rows":rows,"input_sha256":panel.input_sha256,
            "conversion_rows_sha256":panel.saved_rows_sha256,"parent_event_rows_sha256":sha256_bytes(&events_bytes),
            "cell_order":["parent_parent_saved","event_final_age_parent","event_parent_age_final","final_final_saved"],
            "correct":correct,"answer_ce_sum":ce_sums,"gains_vs_parent_parent":gains,"losses_vs_parent_parent":losses,
            "event_only_correct_change":correct[1]-correct[0],"age_only_correct_change":correct[2]-correct[0],
            "joint_correct_change":correct[3]-correct[0],"accuracy_interaction_count":correct[3]-correct[1]-correct[2]+correct[0],
            "ce_interaction_sum":ce_sums[3]-ce_sums[1]-ce_sums[2]+ce_sums[0]}));
        if rows != 128 {
            break;
        }
    }
    if snapshot(&[("event", es.as_path()), ("age", ages.as_path())])? != final_files
        || fit::parameter_hashes(parent_event, &parent_var)? != parent_parameters
        || fit::parameter_hashes(&event, &final_var)? != final_parameters
    {
        return Err(invalid("crossover mutated a selected source"));
    }
    let complete = predictions == 512;
    Ok(
        json!({"schema":"uor-r4.event-age-crossover/1","complete":complete,"seed":2,"optimizer_updates":0,
        "new_predictions":predictions,"saved_endpoint_predictions":512,"sampler_draws":0,"fit_root":fit_root,
        "fit_report_sha256":sha256_bytes(&report_bytes),"fit_inputs_sha256":sha256_bytes(&inputs_bytes),
        "parent_parameters":parent_parameters,"final_parameters":final_parameters,"sources_unchanged":true,
        "source_policy":POLICY,"all_routes_cut":true,"panels":summaries,"comparison_seconds":started.elapsed().as_secs_f64(),
        "decision":if complete{"COMPLETE_FIXED_EVENT_AGE_CROSSOVER_RETAIN_ROWS"}else{"PARTIAL_NO_CAUSAL_INTERPRETATION"},
        "scope":"Two missing zero-update cells on the same exposed seed2 rows. Saved endpoints remain bound to their executed fit source. No new fit, optimizer, sampler, selector, scale or estimator; no held-out language or geometry advantage claim."}),
    )
}
