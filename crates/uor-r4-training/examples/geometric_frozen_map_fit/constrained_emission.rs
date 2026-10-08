//! One shared Generate unary/pair update at the pinned retained Context endpoint.
use super::native_proposals as np;
use super::*;
use uor_r4_core::native_geometric::learner::geometric_generate::GenerateReadCounts;

const REPORT_SHA: &str = "fba3f4cbcdc475147062105bf175a82f0556e873c6a6b6918714d23c5aac57e4";
const MANIFEST_SHA: &str = "6be19fd174c0feb55b0bfa07815fb86dbac75739b2b86e7b2546b7bc5dcbc35f";
const CONTEXT_SHA: &str = "f19d1f8bd21315e58e7b3e165bea68b57d93cc6323e9838d6c598098c62389c3";
const SOURCE_SHA: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const GENERATE_SHA: &str = "5fe2b5e287f5e27dc700ce8b94c4d6f38b41e73950a1a145cd1c0c2c56563544";
const FIELD_SHA: &str = "e18a47a16c68b275f72299558fcfb5af601aa9f40b72213f2ee7cc745e8b7c9d";
const PACKED_FIELD_SHA: &str = "4b48f21a4b7a02bfbec19ef880a967a02334a3cdcef8ae83de2ef327ba8bc5dd";

pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.native-constrained-emission/1","rounds":1,"maximum_candidates":1,
        "construction":"one frozen ascending g*actual-master-displacement/name/index pass; at most one adjacent negative-gradient legal code per shared Generate unary/pair coordinate; simultaneous pooled-winner constraints at current coordinated incumbent",
        "eligible":{"generate.unary":[8,120],"generate.pair":[4,120,120]},"coordinates":58560,
        "cache":"A: sparse same-reference simultaneous patch; authoritative full Generate/physical Copy reduction when clipped reference changes; atomic all-row revision-bound commit",
        "objective":frontier::OBJECTIVE,"optimizer_updates":0,"protected_positions":84,
        "frozen":"Context/token coefficients/Potential/categorical map/cue/prefix/tokenizer/algebra/topology, Generate bias/prototype master bits, U numerical coefficients",
        "native_score_quantum_q24":1i64<<20,"master_code_unit":0.25,
        "selection":"failed-frontier emission effect and strict native CE descent >1e-10*(1+abs(parent)); otherwise exact parent restore",
        "utility":"retain all8 actual complete replies and gain complete replies; lowerCE alone is optimization evidence",
        "scope":"one greedy feasible-prefix construction, sequential emission coadaptation; no global feasibility, Context attribution, transfer/chat/energy qualification"})
}
fn root(a: &Args) -> Result<&Path> {
    a.retained_context_root
        .as_deref()
        .ok_or_else(|| bad("retained Context root absent"))
}
pub(super) fn input_identity(a: &Args) -> Result<Value> {
    Ok(
        json!({"root":root(a)?,"checkpoint":"checkpoint-0001","report_sha256":REPORT_SHA,"manifest_sha256":MANIFEST_SHA,
        "context_sha256":CONTEXT_SHA,"source_metadata_sha256":SOURCE_SHA,"generate_sha256":GENERATE_SHA,"continuation_sha256":FIELD_SHA,
        "packed_continuation_sha256":PACKED_FIELD_SHA,"accepted_model":"original Source48/Generate64 remains accepted; Context is experimental immediate input"}),
    )
}
pub(super) fn load_parent(a: &Args, original: &ContinuationParent) -> Result<ContinuationParent> {
    let root = root(a)?;
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == REPORT_SHA
            && sha256_file(&root.join("manifest.json"))? == MANIFEST_SHA,
        "retained Context report/seal differs",
    )?;
    let report = read(&root.join("report.json"))?;
    let cp = root.join("checkpoint-0001");
    replay_require(
        report["status"] == "COMPLETED"
            && report["source_commit"] == "7f474caa819612c8f3fa6aafba2b0b2902368a26"
            && report["constrained_context_learning"] == true
            && report["native_code_proposals"]["winner"] == 0
            && report["final_receipt"] == read(&cp.join("receipt.json"))?,
        "retained Context selection/receipt differs",
    )?;
    let p = ContinuationParent::from_checkpoint(&cp)?;
    replay_require(
        p.binding.identity == original.binding.identity
            && p.binding.metadata_sha256 == SOURCE_SHA
            && p.generate_sha256 == GENERATE_SHA
            && p.generate == original.generate
            && p.bridge == original.bridge
            && p.exp == original.exp,
        "retained Context frozen identity differs",
    )?;
    replay_require(
        sha256_file(&cp.join("native/consumer/context-q4.bin"))? == CONTEXT_SHA
            && sha256_file(&cp.join("continuation-field.bin"))? == FIELD_SHA,
        "retained Context payload identity differs",
    )?;
    replay_require(
        p.cue == original.cue && p.joint == original.joint && p.prefix == original.prefix,
        "retained Context changed frozen cue/prefix payload",
    )?;
    let tokbytes = fs::read(cp.join("native/tokenizer.json"))?;
    let old_source = SourceRealizerWeights::load_source(&a.checkpoint.join("source"), &tokbytes)?;
    let new_source = SourceRealizerWeights::load_source(&cp.join("source"), &tokbytes)?;
    let old = np::snapshot(&old_source.parameters())?;
    let new = np::snapshot(&new_source.parameters())?;
    let allowed = np::BASIS
        .iter()
        .map(|n| format!("consumer.context.{n}"))
        .collect::<BTreeSet<_>>();
    replay_require(
        old.keys().eq(new.keys()),
        "retained source master inventory differs",
    )?;
    for (name, values) in &old {
        if !allowed.contains(name) {
            replay_require(
                values
                    .iter()
                    .zip(&new[name])
                    .all(|(a, b)| a.to_bits() == b.to_bits())
                    && values.len() == new[name].len(),
                "retained Context changed frozen source master bits",
            )?;
        }
    }
    for directory in ["generate-source", "read-state-bridge-source"] {
        let before = a.checkpoint.join(directory);
        let after = cp.join(directory);
        let files = fs::read_dir(&before)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<std::io::Result<BTreeSet<_>>>()?;
        let actual = fs::read_dir(&after)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<std::io::Result<BTreeSet<_>>>()?;
        replay_require(files == actual, "retained frozen master file set differs")?;
        for file in files {
            if file.to_string_lossy().ends_with(".f32le") {
                replay_require(
                    fs::read(before.join(&file))? == fs::read(after.join(&file))?,
                    "retained Context changed frozen Generate/bridge master bytes",
                )?;
            }
        }
    }
    Ok(p)
}
pub(super) fn restore_field(
    a: &Args,
    p: &ContinuationParent,
    weights: &ContinuationLearningWeights,
    d: &Device,
) -> Result<()> {
    let cp = root(a)?.join("checkpoint-0001");
    let bytes = fs::read(cp.join("continuation-field.bin"))?;
    let generator = p.generator()?;
    let field =
        NativeContinuationField::from_bytes(&bytes, &p.binding, generator.generate_model())?;
    replay_require(
        sha256_bytes(field.packed_unary()) == PACKED_FIELD_SHA,
        "retained U numeric packing differs",
    )?;
    restore(
        &cp.join("continuation-source"),
        &p.receipt["continuation_parameters"],
        &weights.parameters(),
        d,
    )?;
    replay_require(
        weights
            .export_native(&p.binding, generator.generate_model())?
            .to_bytes()?
            == bytes,
        "restored retained U master/native differs",
    )
}
fn terms(a: &Args) -> Result<Vec<np::Term>> {
    let reached = frontier::read_plan(a)?;
    frontier::components(&reached)?;
    Ok(reached
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
        .collect())
}
fn capture(
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    terms: &[np::Term],
) -> Result<(Vec<emission_constraints::ProtectedPool>, Value)> {
    let bytes = field.to_bytes()?;
    let sha = sha256_bytes(&bytes);
    let mut generator = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &sha,
    })?;
    let mut banks = BTreeMap::new();
    let mut pools = Vec::new();
    let mut witnesses = Vec::new();
    for term in terms.iter().filter(|t| t.component == 1) {
        if let std::collections::btree_map::Entry::Vacant(entry) = banks.entry(term.index) {
            entry.insert(generator.admit_bank(continuation_snapshot(&eps[term.index].packet)?)?);
        }
        let prefix = term
            .parent_actual_prefix_ids
            .as_deref()
            .ok_or_else(|| bad("protected actual prefix absent"))?;
        let step = generator.step(&banks[&term.index], prefix)?;
        replay_require(
            step.actions.summary.chosen_token_id == term.target,
            "protected retained Context output differs",
        )?;
        replay_require(
            step.generate_raw_scores_q24.len() == 4096
                && generator.generate_model().lanes() == 8
                && generator.generate_model().energy().edges().len() == 4
                && step.actions.summary.legal_generate_actions == 4096,
            "protected emission native topology/support differs",
        )?;
        let mut factors = Vec::with_capacity(4096 * 12);
        let mut keys = [0u32; 12];
        let mut counts = GenerateReadCounts::default();
        for token in 0..4096 {
            generator.generate_model().factor_incidence_into(
                &step.post_state,
                token,
                &mut keys,
                &mut counts,
            )?;
            factors.extend_from_slice(&keys);
        }
        witnesses.push(json!({"term":term,"factual_state":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),"generate_q24":step.generate_raw_scores_q24,"copy_ids":step.copy_token_ids,"copy_q24":step.copy_raw_scores_q24,"pool":step.actions.summary,"token_masses":step.actions.token_masses,"factor_keys_sha256":sha256_bytes(&serde_json::to_vec(&factors)?)}));
        pools.push(emission_constraints::ProtectedPool {
            required_token: term.target,
            generate_q24: step.generate_raw_scores_q24,
            copy_ids: step.copy_token_ids,
            copy_q24: step.copy_raw_scores_q24,
            factor_keys: factors,
        });
    }
    replay_require(
        pools.len() == 84 && banks.len() == 8,
        "protected emission capture population differs",
    )?;
    Ok((pools, json!(witnesses)))
}
fn verify_pools(
    actual: &Value,
    construction: &emission_constraints::Construction,
    baseline: &Value,
) -> Result<()> {
    let rows = actual
        .as_array()
        .ok_or_else(|| bad("protected replay rows absent"))?;
    let old = baseline
        .as_array()
        .ok_or_else(|| bad("protected baseline rows absent"))?;
    replay_require(
        rows.len() == 84 && old.len() == 84,
        "protected replay coverage differs",
    )?;
    for (i, row) in rows.iter().enumerate() {
        let masses: Vec<u64> = row["token_masses"]
            .as_array()
            .ok_or_else(|| bad("protected replay masses absent"))?
            .iter()
            .map(|m| {
                m["weight_q31"]
                    .as_u64()
                    .ok_or_else(|| bad("protected replay mass absent"))
            })
            .collect::<Result<_>>()?;
        let expected = &construction.final_pool_summaries[i];
        replay_require(
            row["term"] == old[i]["term"]
                && row["factual_state"] == old[i]["factual_state"]
                && row["copy_ids"] == old[i]["copy_ids"]
                && row["copy_q24"] == old[i]["copy_q24"]
                && row["factor_keys_sha256"] == old[i]["factor_keys_sha256"]
                && row["generate_q24"] == json!(construction.final_generate_scores[i])
                && masses == construction.final_token_masses[i]
                && row["pool"]["chosen_token_id"] == expected.chosen_token_id
                && row["pool"]["total_weight_q31"] == expected.total_weight_q31
                && row["pool"]["max_score_q24"] == expected.reference_q24,
            "independent emission native pools differ from cumulative cache",
        )?;
    }
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
    plan: &ReferencePlan,
    start: Instant,
) -> Result<Value> {
    let parent = np::snapshot(params)?;
    let terms = terms(a)?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    let inventory = np::save_ranking_gradients(a, params, gradients)?;
    let mut g = np::Shadows::new();
    for name in ["generate.unary", "generate.pair"] {
        let values = gradients
            .get(name)
            .ok_or_else(|| bad("fresh emission gradient absent"))?
            .flatten_all()?
            .to_device(&Device::Cpu)?
            .to_vec1::<f32>()?;
        g.insert(name.to_owned(), values);
    }
    np::verify_codes(initial, initial_field, &parent)?;
    let baseline = np::score(a, initial, initial_field, eps, plan, &terms, start)?;
    let initial_validation = read(&a.out.join("frontier-initial-validation.json"))?;
    let baseline_loss = np::objective(&baseline)?;
    let admitted = initial_validation["combined_losses_before_after"][0]
        .as_f64()
        .ok_or_else(|| bad("emission baseline absent"))?;
    replay_require(
        (baseline_loss - admitted).abs() <= 1e-10 * (1. + admitted.abs())
            && (baseline_loss - 8.971047852627056).abs() < 1e-9,
        "emission immediate-input objective differs",
    )?;
    write(a, "native-code-parent-objective.json", &baseline)?;
    let (pools, protected) = capture(initial, initial_field, eps, &terms)?;
    write(a, "emission-parent-pools.json", &protected)?;
    let mut reducer =
        NativeVocabularyActions::new(initial.integer.binding().clone(), &initial.exp)?;
    replay_require(
        reducer.legal_token_ids().iter().copied().eq(0u32..4096),
        "emission complete legal vocabulary differs",
    )?;
    let clock = Instant::now();
    let construction = emission_constraints::construct(&parent, &g, pools, &mut reducer)?;
    let construction_seconds = clock.elapsed().as_secs_f64();
    write(
        a,
        "emission-construction.json",
        &json!({"policy":policy(),"construction":construction,"construction_seconds":construction_seconds,"gradient_inventory":inventory,"parent_master_identities":identities(params)?,"immediate_input":input_identity(a)?}),
    )?;
    let mut selected = false;
    let mut measured_loss = None;
    let mut status = "EMPTY_FEASIBLE_PREFIX_CONSTRUCTION";
    let mut results = Vec::new();
    let mut checkpoint_seconds = 0.;
    let mut native_seconds = baseline["elapsed_seconds"]
        .as_f64()
        .ok_or_else(|| bad("emission baseline timing absent"))?;
    if !construction.edits.is_empty() {
        let mut candidate = a.clone();
        candidate.out = a.out.join("native-candidate-00");
        candidate.maximum_report_bytes = a
            .maximum_report_bytes
            .checked_sub(size(&a.out)?)
            .ok_or_else(|| bad("emission report cap"))?;
        replay_require(
            candidate.maximum_report_bytes > 2_097_152,
            "emission candidate reserve exhausted",
        )?;
        report_output::claim(&candidate.out)?;
        let outcome = np::attempt_restored(params, &parent, || {
            for file in ["reference-plan.json", "frontier-plan.json"] {
                fs::copy(a.out.join(file), candidate.out.join(file))?;
            }
            write(&candidate, "proposal.json", &json!(construction.edits))?;
            np::apply_edits(params, &parent, &construction.edits)?;
            let clock = Instant::now();
            let (current, field, receipt) = joint_checkpoint(&candidate, 0, l, weights)?;
            let cp_seconds = clock.elapsed().as_secs_f64();
            np::verify_codes(&current, &field, &np::edited(&parent, &construction.edits)?)?;
            replay_require(
                current.binding == initial.binding
                    && current.bridge == initial.bridge
                    && field.packed_unary() == initial_field.packed_unary(),
                "emission changed frozen Source/map/U numeric payload",
            )?;
            let (_, actual) = capture(&current, &field, eps, &terms)?;
            verify_pools(&actual, &construction, &protected)?;
            write(&candidate, "emission-reloaded-pools.json", &actual)?;
            let measured = np::score(&candidate, &current, &field, eps, plan, &terms, start)?;
            let before = baseline["terms"]
                .as_array()
                .ok_or_else(|| bad("emission baseline terms absent"))?;
            let after = measured["terms"]
                .as_array()
                .ok_or_else(|| bad("emission candidate terms absent"))?;
            replay_require(before.len() == after.len(), "emission term count differs")?;
            let mut effect = 0;
            for (b, n) in before.iter().zip(after) {
                replay_require(
                    b["term"] == n["term"] && b["factual_state"] == n["factual_state"],
                    "emission frozen factual state/term differs",
                )?;
                if b["term"]["component"] == 0
                    && b["generate_raw_scores_sha256"] != n["generate_raw_scores_sha256"]
                {
                    effect += 1;
                }
            }
            write(&candidate, "objective.json", &measured)?;
            Ok(
                json!({"status":"COMPLETED","combined":np::objective(&measured)?,"failed_frontier_generate_changed":effect,"exact_protected_pools_verified":true,"checkpoint_seconds":cp_seconds,"native_evaluation_seconds":measured["elapsed_seconds"],"receipt":receipt,"all_parent_bits_restored":true}),
            )
        });
        let result = match &outcome {
            Ok(v) => v.clone(),
            Err(e) => {
                json!({"status":"FAILED","error":e.to_string(),"all_parent_bits_restored":np::same_bits(&parent,&np::snapshot(params)?),"model_verdict":"execution failure, not model-quality evidence"})
            }
        };
        write(&candidate, "report.json", &result)?;
        report_output::seal(&candidate.out)?;
        report_output::verify(&candidate.out)?;
        let result = outcome?;
        let loss = result["combined"]
            .as_f64()
            .ok_or_else(|| bad("candidate loss absent"))?;
        let effect = result["failed_frontier_generate_changed"]
            .as_u64()
            .ok_or_else(|| bad("candidate effect absent"))?;
        selected = effect > 0 && loss < baseline_loss - 1e-10 * (1. + baseline_loss.abs());
        status = if selected {
            "NATIVE_CONSTRAINED_EMISSION_DESCENT_SELECTED"
        } else if effect == 0 {
            "NO_FAILED_FRONTIER_EMISSION_EFFECT"
        } else {
            "CONSTRAINED_EMISSION_NATIVE_OBJECTIVE_NOT_IMPROVED"
        };
        measured_loss = Some(loss);
        checkpoint_seconds = result["checkpoint_seconds"]
            .as_f64()
            .ok_or_else(|| bad("candidate checkpoint timing absent"))?;
        native_seconds += result["native_evaluation_seconds"]
            .as_f64()
            .ok_or_else(|| bad("candidate evaluation timing absent"))?;
        results.push(json!({"index":0,"report":result,"report_sha256":sha256_file(&candidate.out.join("report.json"))?,"manifest_sha256":sha256_file(&candidate.out.join("manifest.json"))?}));
    }
    let committed = (|| -> Result<Value> {
        if selected {
            np::apply_edits(params, &parent, &construction.edits)?;
        }
        let winner = if selected { Some(0usize) } else { None };
        let result = json!({"policy":policy(),"status":status,"winner":winner,"winner_family":winner.map(|_|"constrained_emission"),"parent_combined":baseline_loss,"candidate_combined":measured_loss,"selected_combined":measured_loss.filter(|_|selected).unwrap_or(baseline_loss),"accepted_code_proposals":usize::from(selected),"optimizer_updates":0,"construction_seconds":construction_seconds,"candidate_checkpoint_seconds":checkpoint_seconds,"native_objective_seconds":native_seconds,"results":results,"selected_master_identities":identities(params)?,"emission_candidates_evaluated":usize::from(measured_loss.is_some())});
        write(a, "native-code-proposals.json", &result)?;
        Ok(result)
    })();
    if committed.is_err() {
        return np::attempt_restored(params, &parent, || committed);
    }
    committed
}
