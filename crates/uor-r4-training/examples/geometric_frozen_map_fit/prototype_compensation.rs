//! One archived shared-prototype intervention followed by one coefficient compensation.
use super::native_proposals as np;
use super::*;
use uor_r4_training::geometric_generate_learning::vocabulary_marginal_loss_with_credit;

const EMISSION_REPORT: &str = "d9e118b53eb82c4c1c458d561fd271d6a3d96faccd2df92f25c00ab062a9a801";
const EMISSION_MANIFEST: &str = "f7bc3f344270c84856f803b78bec9e6f0d35ae5511ce20ae5ec0f4749fc75e20";
const DIAGNOSTIC_REPORT: &str = "cb7ab288e0d5bc11172b54e877a56c864a5fd64696358c86fbd76dfde46a72e7";
const DIAGNOSTIC_MANIFEST: &str =
    "e53c2594295f5f27f1b8ce2129b760167f0d25adda0904e9905d323dcc155f13";
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const GENERATE: &str = "d0f98a54652ca5892ff821d842fca450ff6e31b4785fc71f43fe821926aa293e";
const FIELD: &str = "b4e15b3bbae4cdf7e93cb21ed13664967cbec4bc66559ea7b33e853a9084b107";
const ORIGINAL_LOSS: f64 = 8.52634374894637;
const INTERMEDIATE_LOSS: f64 = 8.634086267334682;
const ENTRY_TERM: usize = 391;
const TOKEN: usize = 617;
const LANE: usize = 0;
const OLD: u8 = 58;
const NEW: u8 = 48;

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_emission_root: PathBuf,
    pub prototype_diagnostic_root: PathBuf,
}
fn config(a: &Args) -> Result<&Config> {
    a.prototype_compensation
        .as_ref()
        .ok_or_else(|| bad("compensation config absent"))
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.single-prototype-compensation/1","intervention":{"token":TOKEN,"lane":LANE,"before":OLD,"after":NEW,"master_encoding":"one selected120 row: unique2.0 at48, all other entries0.0; all unselected rows bitwise fixed"},
        "gradient":"one fresh full588 weighted coefficient-only gradient at changed prototype; no state/prototype utility graph; only unary/pair gradients retained; bias frozen",
        "construction":"one existing frozen global g*actual-master-displacement/name/index order; once-only adjacent legal unary/pair Q4 moves, no refill",
        "protected":"original84 complete-trajectory pooled winners plus existing objective term391 canonical entry;85 constraints,588 unchanged loss terms",
        "selection":"strict nativeCE below ORIGINAL selected emission8.52634374894637 with existing1e-10*(1+abs(parent)) tolerance; descent from intermediate alone insufficient",
        "frozen":"Source/Context/Potential,bias,unselected prototype masters,categorical read-to-emission map,cue/prefix,U numerical0,tokenizer/algebra/edges/data/weights",
        "scope":"one exposed-development intervention plus one greedy coefficient construction; no483-fit bank, global feasibility/capacity, transfer/chat/energy qualification"})
}
pub(super) fn input_identity(a: &Args) -> Result<Value> {
    Ok(
        json!({"retained_emission_root":config(a)?.retained_emission_root,"emission_report_sha256":EMISSION_REPORT,"emission_manifest_sha256":EMISSION_MANIFEST,
        "prototype_diagnostic_root":config(a)?.prototype_diagnostic_root,"diagnostic_report_sha256":DIAGNOSTIC_REPORT,"diagnostic_manifest_sha256":DIAGNOSTIC_MANIFEST,
        "original_source_metadata_sha256":SOURCE,"original_generate_sha256":GENERATE,"original_continuation_sha256":FIELD,"original_combined":ORIGINAL_LOSS,"prototype_intermediate_combined":INTERMEDIATE_LOSS,
        "selected":{"token":TOKEN,"lane":LANE,"before":OLD,"after":NEW,"new_entry_objective_term_index":ENTRY_TERM},"accepted_model":"original Source48/Generate64 remains accepted"}),
    )
}
fn loss_gate(candidate: f64) -> bool {
    candidate.is_finite() && candidate < ORIGINAL_LOSS - 1e-10 * (1.0 + ORIGINAL_LOSS.abs())
}
fn canonical_row(
    values: &[f32],
    vocab: usize,
    lanes: usize,
    token: usize,
    lane: usize,
    code: u8,
) -> Result<Vec<f32>> {
    replay_require(
        values.len() == vocab * lanes * 120
            && token < vocab
            && lane < lanes
            && code < 120
            && values.iter().all(|v| v.is_finite()),
        "invalid prototype master intervention",
    )?;
    let mut out = values.to_vec();
    let offset = (token * lanes + lane) * 120;
    out[offset..offset + 120].fill(0.0);
    out[offset + usize::from(code)] = 2.0;
    Ok(out)
}
fn set_choices(l: &Loaded, values: &[f32]) -> Result<()> {
    let var = &l.generate.prototype_choices;
    var.set(&Tensor::from_vec(
        values.to_vec(),
        var.shape(),
        var.device(),
    )?)?;
    Ok(())
}
fn settle_model(
    params: &BTreeMap<String, Var>,
    original: &np::Shadows,
    prototype: &Var,
    original_choices: &[f32],
    retain: bool,
) -> Result<()> {
    if retain {
        return Ok(());
    }
    // Attempt both restorations even when one device write fails.
    let coefficients = np::restore(params, original);
    let prototypes = (|| -> Result<()> {
        prototype.set(&Tensor::from_vec(
            original_choices.to_vec(),
            prototype.shape(),
            prototype.device(),
        )?)?;
        Ok(())
    })();
    coefficients?;
    prototypes?;
    Ok(())
}
fn choices(l: &Loaded) -> Result<Vec<f32>> {
    Ok(l.generate
        .prototype_choices
        .flatten_all()?
        .to_device(&Device::Cpu)?
        .to_vec1::<f32>()?)
}
fn load_parent(a: &Args, accepted: &ContinuationParent) -> Result<ContinuationParent> {
    let c = config(a)?;
    for (root, report, manifest) in [
        (
            &c.retained_emission_root,
            EMISSION_REPORT,
            EMISSION_MANIFEST,
        ),
        (
            &c.prototype_diagnostic_root,
            DIAGNOSTIC_REPORT,
            DIAGNOSTIC_MANIFEST,
        ),
    ] {
        report_output::verify(root)?;
        replay_require(
            sha256_file(&root.join("report.json"))? == report
                && sha256_file(&root.join("manifest.json"))? == manifest,
            "compensation archived report/seal pin differs",
        )?;
    }
    let r = read(&c.retained_emission_root.join("report.json"))?;
    let cp = c.retained_emission_root.join("checkpoint-0001");
    replay_require(
        r["status"] == "COMPLETED"
            && r["constrained_emission_learning"] == true
            && r["native_code_proposals"]["winner"] == 0
            && r["final_receipt"] == read(&cp.join("receipt.json"))?,
        "retained emission selected checkpoint differs",
    )?;
    let p = ContinuationParent::from_checkpoint(&cp)?;
    replay_require(
        p.binding.identity == accepted.binding.identity
            && p.binding.metadata_sha256 == SOURCE
            && p.generate_sha256 == GENERATE
            && p.bridge == accepted.bridge
            && p.exp == accepted.exp
            && p.cue == accepted.cue
            && p.joint == accepted.joint
            && p.prefix == accepted.prefix,
        "retained emission frozen/native identity differs",
    )?;
    replay_require(
        sha256_file(&cp.join("continuation-field.bin"))? == FIELD,
        "retained emission U differs",
    )?;
    let diagnostic = read(&c.prototype_diagnostic_root.join("report.json"))?;
    replay_require(
        diagnostic["status"] == "COMPLETED"
            && diagnostic["positive_neighborhood_witnesses"] == 483
            && diagnostic["identity"]["report_sha256"] == EMISSION_REPORT,
        "prototype diagnostic identity differs",
    )?;
    let triples = diagnostic["triples"]
        .as_array()
        .ok_or_else(|| bad("diagnostic triples absent"))?;
    replay_require(triples.len() == 1920, "diagnostic population differs")?;
    let mut positive = triples
        .iter()
        .filter(|v| v["positive_neighborhood_witness"] == true)
        .collect::<Vec<_>>();
    for row in &positive {
        replay_require(
            row["combined"].as_f64().is_some_and(f64::is_finite),
            "nonfinite diagnostic objective",
        )?;
    }
    positive.sort_by(|a, b| {
        a["combined"]
            .as_f64()
            .unwrap_or(f64::INFINITY)
            .total_cmp(&b["combined"].as_f64().unwrap_or(f64::INFINITY))
            .then_with(|| a["token"].as_u64().cmp(&b["token"].as_u64()))
            .then_with(|| a["lane"].as_u64().cmp(&b["lane"].as_u64()))
            .then_with(|| a["code"].as_u64().cmp(&b["code"].as_u64()))
    });
    let selected = positive
        .first()
        .ok_or_else(|| bad("diagnostic has no positive witness"))?;
    replay_require(
        selected["token"] == TOKEN
            && selected["lane"] == LANE
            && selected["code"] == NEW
            && selected["current_code"] == OLD
            && selected["newly_correct_failed_entries"] == json!([ENTRY_TERM])
            && selected["combined"] == INTERMEDIATE_LOSS,
        "archived deterministic selection differs",
    )?;
    write(
        a,
        "prototype-selection.json",
        &json!({"input":input_identity(a)?,"selected":selected,"rule":"minimum combinedCE positive witness, ascending token/lane/code ties; exposed retrospective selection fixed before successor"}),
    )?;
    Ok(p)
}
fn restore_field(
    a: &Args,
    p: &ContinuationParent,
    l: &Loaded,
    d: &Device,
) -> Result<ContinuationLearningWeights> {
    let cp = config(a)?.retained_emission_root.join("checkpoint-0001");
    let bytes = fs::read(cp.join("continuation-field.bin"))?;
    let native =
        NativeContinuationField::from_bytes(&bytes, &p.binding, p.generator()?.generate_model())?;
    for lane in 0..8 {
        for relative in 0..120 {
            replay_require(
                native.coefficient_unary(lane, relative)? == 0,
                "compensation requires numeric U0",
            )?;
        }
    }
    let weights =
        ContinuationLearningWeights::zeroed_shared_action(l.integer.binding(), &p.binding, 8, d)?;
    restore(
        &cp.join("continuation-source"),
        &p.receipt["continuation_parameters"],
        &weights.parameters(),
        d,
    )?;
    replay_require(
        np::snapshot(&weights.parameters())?
            .values()
            .flatten()
            .all(|&v| v == 0.0),
        "retained U floating masters are not numerically zero",
    )?;
    replay_require(
        weights
            .export_native(&p.binding, p.generator()?.generate_model())?
            .to_bytes()?
            == bytes,
        "retained U master/native restore differs",
    )?;
    Ok(weights)
}
fn verify_intervention(before: &ContinuationParent, after: &ContinuationParent) -> Result<()> {
    let old = before.generator()?;
    let new = after.generator()?;
    let a = old.generate_model();
    let b = new.generate_model();
    replay_require(
        before.binding == after.binding
            && before.bridge == after.bridge
            && before.cue == after.cue
            && before.joint == after.joint
            && before.prefix == after.prefix
            && before.exp == after.exp
            && a.energy() == b.energy()
            && a.algebra() == b.algebra()
            && a.packed_biases() == b.packed_biases(),
        "prototype intervention changed frozen native payload",
    )?;
    let changed = a
        .prototypes()
        .iter()
        .zip(b.prototypes())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .collect::<Vec<_>>();
    replay_require(
        changed.len() == 1
            && changed[0].0 == TOKEN * 8 + LANE
            && *changed[0].1 .0 == OLD
            && *changed[0].1 .1 == NEW,
        "native intervention is not exact one shared prototype",
    )
}
fn fixed_gradients(
    a: &Args,
    l: &Loaded,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    terms: &[np::Term],
    start: Instant,
) -> Result<(BTreeMap<String, Tensor>, Value)> {
    let clock = Instant::now();
    let prepared = l.generate.prepare_native()?;
    replay_require(
        prepared.native.to_bytes()? == p.generate,
        "fresh coefficient snapshot differs from intermediate export",
    )?;
    let bytes = field.to_bytes()?;
    let sha = sha256_bytes(&bytes);
    let mut generator = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &sha,
    })?;
    let eligible = l
        .generate
        .parameters()
        .into_iter()
        .filter(|(name, _)| matches!(name.as_str(), "generate.unary" | "generate.pair"))
        .collect::<BTreeMap<_, _>>();
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut banks = BTreeMap::new();
    let mut loss = [0.0f64; 2];
    let mut row_receipts = Vec::new();
    for term in terms {
        deadline(a, start)?;
        let e = &eps[term.index];
        let prefix = term
            .parent_actual_prefix_ids
            .as_deref()
            .ok_or_else(|| bad("actual prefix absent"))?;
        if let std::collections::btree_map::Entry::Vacant(entry) = banks.entry(term.index) {
            entry.insert(generator.admit_bank(continuation_snapshot(&e.packet)?)?);
        }
        let step = generator.step(&banks[&term.index], prefix)?;
        let output = l
            .generate
            .forward_prepared_coefficients_only(&prepared, &step.post_state)?;
        replay_require(
            output.scores_q24 == step.generate_raw_scores_q24,
            "coefficient-only factual Generate differs from full native U0 pool",
        )?;
        let copy = if step.copy_raw_scores_q24.is_empty() {
            None
        } else {
            Some(Tensor::from_vec(
                step.copy_raw_scores_q24
                    .iter()
                    .map(|&v| (v as f64 / (1u64 << 24) as f64) as f32)
                    .collect::<Vec<_>>(),
                step.copy_raw_scores_q24.len(),
                l.generate.device(),
            )?)
        };
        let mass = step
            .actions
            .token_masses
            .iter()
            .find(|v| v.token_id == term.target)
            .ok_or_else(|| bad("full pool target absent"))?
            .weight_q31;
        let ce = -(mass as f64 / step.actions.summary.total_weight_q31 as f64).ln();
        loss[term.component] += term.weight * ce;
        let grad = vocabulary_marginal_loss_with_credit(
            &step.actions,
            &output.raw_scores,
            copy.as_ref(),
            term.target,
            VocabularyScoreAdjoint::RawIdentity,
        )?
        .affine(term.weight, 0.0)?
        .backward()?;
        replay_require(
            grad.get(l.generate.prototype_choices.as_tensor()).is_none(),
            "coefficient-only graph contains prototype credit",
        )?;
        for (name, var) in &eligible {
            if let Some(g) = grad.get(var.as_tensor()) {
                replay_require(
                    g.device().same_device(l.generate.device())
                        && g.sqr()?.sum_all()?.to_scalar::<f32>()?.is_finite(),
                    "coefficient gradient finite/device failure",
                )?;
                sums.insert(
                    name.clone(),
                    if let Some(old) = sums.get(name) {
                        old.add(g)?.detach()
                    } else {
                        g.detach()
                    },
                );
            }
        }
        row_receipts.push(json!({"term":term,"state":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),"generate_sha256":sha256_bytes(&serde_json::to_vec(&output.scores_q24)?),"target_mass":mass,"denominator":step.actions.summary.total_weight_q31}));
    }
    replay_require(
        terms.len() == 588 && sums.len() == 2,
        "fresh coefficient gradient coverage differs",
    )?;
    Ok((
        sums,
        json!({"positions":terms.len(),"task":loss[0],"reference":loss[1],"combined":loss[0]+loss[1],"elapsed_seconds":clock.elapsed().as_secs_f64(),"eligible":["generate.unary","generate.pair"],"prototype_credit":false,"context_credit":false,"U_credit":false,"bias":"graph coefficient credit may exist; frozen and excluded from retained gradient inventory","rows":row_receipts,"intermediate_generate_sha256":p.generate_sha256}),
    ))
}
/// Child checkpoints retain the authenticated root plan before receipt rebinding.
/// Copies preserve exact root bytes; the authoritative preparation can use a
/// different JSON serialization, so compare its complete parsed value first.
fn copy_child_reference(parent: &Args, child: &Args) -> Result<()> {
    replay_require(
        parent.reached_frontier_objective
            && child.reached_frontier_objective
            && serde_json::to_value(&parent.reference_replay)?
                == serde_json::to_value(&child.reference_replay)?,
        "child reference configuration differs",
    )?;
    let c = parent
        .reference_replay
        .as_ref()
        .ok_or_else(|| bad("child reference configuration absent"))?;
    let root_frontier = read(&parent.out.join("frontier-plan.json"))?;
    replay_require(
        root_frontier == read(&c.plan_root.join("plan.json"))?
            && read(&parent.out.join("reference-plan.json"))?
                == root_frontier["canonical_reference"],
        "root copied frontier/reference differs from authenticated preparation",
    )?;
    let binding = reference_binding(parent)?;
    let mut copies = serde_json::Map::new();
    for name in ["frontier-plan.json", "reference-plan.json"] {
        let input = parent.out.join(name);
        let output = child.out.join(name);
        fs::copy(&input, &output)?;
        let expected = sha256_file(&input)?;
        replay_require(
            sha256_file(&output)? == expected,
            "child reference plan byte identity differs",
        )?;
        copies.insert(
            name.to_owned(),
            json!({"sha256":expected,"bytes":fs::metadata(&output)?.len()}),
        );
    }
    let rebound = reference_binding(child)?;
    replay_require(
        rebound == binding,
        "child checkpoint reference receipt differs from root",
    )?;
    write(
        child,
        "child-plan-binding.json",
        &json!({"files":copies,"reference_binding":rebound}),
    )
}
fn original_outputs(a: &Args) -> Result<Value> {
    let root = &config(a)?.retained_emission_root;
    let mut output = read(&root.join("development-0001.json"))?;
    let rows = output["rows"]
        .as_array_mut()
        .ok_or_else(|| bad("retained original outputs absent"))?;
    replay_require(
        rows.len() == 512,
        "retained original output coverage differs",
    )?;
    for (i, row) in rows.iter_mut().enumerate() {
        let file = row["row_file"]
            .as_str()
            .ok_or_else(|| bad("retained row file absent"))?;
        replay_require(
            Path::new(file).components().count() == 1
                && !matches!(
                    Path::new(file).components().next(),
                    Some(std::path::Component::ParentDir)
                ),
            "retained row file is not leaf",
        )?;
        let bytes = fs::read(root.join(file))?;
        replay_require(
            row["row_sha256"] == sha256_bytes(&bytes),
            "retained original row digest differs",
        )?;
        let name = format!("original-row-{i:04}.json");
        fs::write(a.out.join(&name), bytes)?;
        row["row_file"] = json!(name);
    }
    write(a, "original-development.json", &output)?;
    Ok(output)
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    reference_replay_settings(a)?;
    joint_continuation_settings(a)?;
    let accepted = ContinuationParent::load(a)?;
    let p = load_parent(a, &accepted)?;
    let mut load = a.clone();
    load.checkpoint = config(a)?.retained_emission_root.join("checkpoint-0001");
    let l = load_joint_continuation(&load, &p, d)?;
    let weights = restore_field(a, &p, &l, d)?;
    let params = joint_active(&l, &weights)?;
    let original = np::snapshot(&params)?;
    let original_choices = choices(&l)?;
    let result = execute(
        a,
        &l,
        &weights,
        &params,
        &original,
        &original_choices,
        &accepted,
        &p,
        start,
        d,
    );
    let retain = result
        .as_ref()
        .is_ok_and(|r| r["native_code_proposals"]["winner"] == 0);
    settle_model(
        &params,
        &original,
        &l.generate.prototype_choices,
        &original_choices,
        retain,
    )?;
    result
}
fn execute(
    a: &Args,
    l: &Loaded,
    weights: &ContinuationLearningWeights,
    params: &BTreeMap<String, Var>,
    original: &np::Shadows,
    original_choices: &[f32],
    accepted: &ContinuationParent,
    p: &ContinuationParent,
    start: Instant,
    d: &Device,
) -> Result<Value> {
    write(
        a,
        "admission.json",
        &json!({"mode":"prototype_compensation","policy":policy(),"immediate_input":input_identity(a)?,"device":format!("{d:?}"),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
    )?;
    let legal = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &p.integer,
        &p.tokenizer,
        &legal,
        512,
    )?;
    pairs(&eps)?;
    let reached = frontier::load(a, accepted, &eps)?
        .ok_or_else(|| bad("compensation frontier plan absent"))?;
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
    replay_require(
        terms.len() == 588
            && terms[ENTRY_TERM].component == 0
            && terms[ENTRY_TERM].position == 0
            && terms[ENTRY_TERM].target == TOKEN as u32,
        "selected new entry term provenance differs",
    )?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    write(
        a,
        "new-entry-provenance.json",
        &json!({"objective_term_index":ENTRY_TERM,"term":terms[ENTRY_TERM],"packet_id":eps[terms[ENTRY_TERM].index].packet.id,"input_index":terms[ENTRY_TERM].index,"canonical_label_only":TOKEN}),
    )?;
    let original_outputs = original_outputs(a)?;
    let (before, before_field, before_receipt) = joint_checkpoint(a, 0, l, weights)?;
    replay_require(
        before.generate == p.generate
            && before.binding == p.binding
            && sha256_bytes(&before_field.to_bytes()?) == FIELD,
        "original independently exported emission differs",
    )?;
    let before_objective = np::score(
        a,
        &before,
        &before_field,
        &eps,
        &reached.canonical_reference,
        &terms,
        start,
    )?;
    replay_require(
        (np::objective(&before_objective)? - ORIGINAL_LOSS).abs() < 1e-9,
        "original fixed objective differs",
    )?;
    write(a, "native-code-parent-objective.json", &before_objective)?;
    let altered = canonical_row(original_choices, 4096, 8, TOKEN, LANE, NEW)?;
    set_choices(l, &altered)?;
    replay_require(
        choices(l)?
            .iter()
            .zip(&altered)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "intervention master copy differs",
    )?;
    write(
        a,
        "prototype-intervention.json",
        &json!({"policy":policy(),"original_master_sha256":sha256_bytes(&original_choices.iter().copied().flat_map(f32::to_le_bytes).collect::<Vec<_>>()),"intermediate_master_sha256":sha256_bytes(&altered.iter().copied().flat_map(f32::to_le_bytes).collect::<Vec<_>>()),"before_row":original_choices[TOKEN*8*120..TOKEN*8*120+120],"after_row":altered[TOKEN*8*120..TOKEN*8*120+120]}),
    )?;
    let mut intermediate_args = a.clone();
    intermediate_args.out = a.out.join("prototype-intermediate");
    report_output::claim(&intermediate_args.out)?;
    copy_child_reference(a, &intermediate_args)?;
    let (intermediate, intermediate_field, intermediate_receipt) =
        joint_checkpoint(&intermediate_args, 0, l, weights)?;
    verify_intervention(&before, &intermediate)?;
    replay_require(
        before_field.packed_unary() == intermediate_field.packed_unary()
            && np::same_bits(original, &np::snapshot(params)?),
        "intervention changed coefficient/frozen master bits",
    )?;
    let intermediate_objective = np::score(
        a,
        &intermediate,
        &intermediate_field,
        &eps,
        &reached.canonical_reference,
        &terms,
        start,
    )?;
    replay_require(
        (np::objective(&intermediate_objective)? - INTERMEDIATE_LOSS).abs() < 1e-9,
        "prototype intermediate differs from diagnostic",
    )?;
    write(
        a,
        "prototype-intermediate-objective.json",
        &intermediate_objective,
    )?;
    write(
        &intermediate_args,
        "report.json",
        &json!({"status":"COMPLETED","receipt":intermediate_receipt,"objective":intermediate_objective,"policy":policy()}),
    )?;
    report_output::seal(&intermediate_args.out)?;
    report_output::verify(&intermediate_args.out)?;
    let mut protected_terms = terms
        .iter()
        .filter(|t| t.component == 1)
        .cloned()
        .collect::<Vec<_>>();
    protected_terms.push(terms[ENTRY_TERM].clone());
    let (pools, protected) = constrained_emission::capture_selected(
        &intermediate,
        &intermediate_field,
        &eps,
        &protected_terms,
        85,
    )?;
    write(a, "compensation-intermediate-pools.json", &protected)?;
    let (gradients, gradient_receipt) = fixed_gradients(
        a,
        l,
        &intermediate,
        &intermediate_field,
        &eps,
        &terms,
        start,
    )?;
    replay_require(
        (gradient_receipt["combined"]
            .as_f64()
            .ok_or_else(|| bad("fresh gradient objective absent"))?
            - INTERMEDIATE_LOSS)
            .abs()
            < 1e-9,
        "fresh gradient objective differs from intermediate",
    )?;
    write(a, "coefficient-gradient-receipt.json", &gradient_receipt)?;
    let inventory = np::save_ranking_gradients(a, params, &gradients)?;
    let mut g = np::Shadows::new();
    for (name, value) in &gradients {
        g.insert(
            name.clone(),
            value
                .flatten_all()?
                .to_device(&Device::Cpu)?
                .to_vec1::<f32>()?,
        );
    }
    let mut reducer =
        NativeVocabularyActions::new(intermediate.integer.binding().clone(), &intermediate.exp)?;
    let clock = Instant::now();
    let construction = emission_constraints::construct_with_protected_count(
        original,
        &g,
        pools,
        &mut reducer,
        85,
    )?;
    let construction_seconds = clock.elapsed().as_secs_f64();
    write(
        a,
        "emission-construction.json",
        &json!({"construction":construction,"construction_seconds":construction_seconds,"policy":policy(),"gradient_inventory":inventory}),
    )?;
    let mut selected = false;
    let mut candidate_loss = None;
    let mut candidate_receipt = Value::Null;
    let mut results = Vec::new();
    if !construction.edits.is_empty() {
        let mut candidate = a.clone();
        candidate.out = a.out.join("native-candidate-00");
        report_output::claim(&candidate.out)?;
        copy_child_reference(a, &candidate)?;
        let outcome = np::attempt_restored(params, original, || {
            np::apply_edits(params, original, &construction.edits)?;
            let (current, field, receipt) = joint_checkpoint(&candidate, 0, l, weights)?;
            np::verify_codes(
                &current,
                &field,
                &np::edited(original, &construction.edits)?,
            )?;
            replay_require(
                current.generator()?.generate_model().prototypes()
                    == intermediate.generator()?.generate_model().prototypes()
                    && choices(l)?
                        .iter()
                        .zip(&altered)
                        .all(|(a, b)| a.to_bits() == b.to_bits())
                    && current.binding == before.binding
                    && current.bridge == before.bridge
                    && field.packed_unary() == before_field.packed_unary(),
                "coefficient compensation changed frozen prototype/Source/map/U",
            )?;
            let (_, actual) = constrained_emission::capture_selected(
                &current,
                &field,
                &eps,
                &protected_terms,
                85,
            )?;
            constrained_emission::verify_pools(&actual, &construction, &protected)?;
            write(&candidate, "emission-reloaded-pools.json", &actual)?;
            let objective = np::score(
                &candidate,
                &current,
                &field,
                &eps,
                &reached.canonical_reference,
                &terms,
                start,
            )?;
            let loss = np::objective(&objective)?;
            write(&candidate, "objective.json", &objective)?;
            Ok(
                json!({"status":"COMPLETED","combined":loss,"passes_original_objective_gate":loss_gate(loss),"protected_positions":85,"receipt":receipt,"all_coefficient_parent_bits_restored":true}),
            )
        });
        let saved = match &outcome {
            Ok(v) => v.clone(),
            Err(e) => {
                json!({"status":"FAILED","error":e.to_string(),"model_verdict":"execution failure, not quality negative"})
            }
        };
        write(&candidate, "report.json", &saved)?;
        report_output::seal(&candidate.out)?;
        report_output::verify(&candidate.out)?;
        let result = outcome?;
        let loss = result["combined"]
            .as_f64()
            .ok_or_else(|| bad("candidate objective absent"))?;
        selected = loss_gate(loss);
        candidate_loss = Some(loss);
        candidate_receipt = result["receipt"].clone();
        results.push(result);
    }
    if selected {
        np::apply_edits(params, original, &construction.edits)?;
    } else {
        settle_model(
            params,
            original,
            &l.generate.prototype_choices,
            original_choices,
            false,
        )?;
    }
    let proposals = json!({"policy":policy(),"winner":if selected {Some(0usize)} else {None},"winner_family":if selected {Some("prototype_compensation")} else {None},"status":if selected {"NATIVE_PROTOTYPE_COMPENSATION_ORIGINAL_DESCENT_SELECTED"} else {"SINGLE_PROTOTYPE_COMPENSATION_NOT_BELOW_ORIGINAL"},
        "parent_combined":ORIGINAL_LOSS,"intermediate_combined":INTERMEDIATE_LOSS,"candidate_combined":candidate_loss,"results":results,"optimizer_updates":0,"selected_master_identities":identities(params)?});
    write(a, "native-code-proposals.json", &proposals)?;
    let (current, field, final_receipt) = joint_checkpoint(a, 1, l, weights)?;
    native_proposals::verify_final(
        a,
        &current,
        &field,
        &eps,
        &reached.canonical_reference,
        start,
    )?;
    if selected {
        let (_, actual) =
            constrained_emission::capture_selected(&current, &field, &eps, &protected_terms, 85)?;
        constrained_emission::verify_pools(&actual, &construction, &protected)?;
        write(a, "final-protected-pools.json", &actual)?;
    } else {
        replay_require(
            current.generate == before.generate
                && field.to_bytes()? == before_field.to_bytes()?
                && np::same_bits(original, &np::snapshot(params)?)
                && choices(l)?
                    .iter()
                    .zip(original_choices)
                    .all(|(a, b)| a.to_bits() == b.to_bits()),
            "rejected compensation did not restore complete original model",
        )?;
    }
    let success_indices = frontier::successful_indices(&reached);
    let selected_eps = success_indices.iter().map(|&i| &eps[i]).collect::<Vec<_>>();
    let early =
        continuation_evaluate_rows(a, "pilot-original8", &current, &field, &selected_eps, start)?;
    replay_require(
        early["complete"] == 8,
        "final compensation violates original actual complete replies",
    )?;
    write(
        a,
        "pilot-original8-receipt.json",
        &json!({"indices":success_indices,"evaluation":early,"checkpoint_step":1}),
    )?;
    let final_evaluation =
        continuation_evaluate(a, "development-0001", &current, &field, &eps, start)?;
    let rowwise = joint_row_comparison(&original_outputs, &final_evaluation)?;
    write(a, "complete-row-comparison.json", &rowwise)?;
    let frontier_outcomes =
        frontier::outcomes(a, &reached, &original_outputs, &final_evaluation, false)?;
    write(a, "reached-frontier-outcomes.json", &frontier_outcomes)?;
    let metric = metrics(a, &final_evaluation, &eps)?;
    write(a, "metrics-0001.json", &metric)?;
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"prototype_compensation","prototype_compensation":true,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"updates":1,
        "policy":policy(),"immediate_input":input_identity(a)?,"initial_receipt":before_receipt,"prototype_intermediate_receipt":intermediate_receipt,"candidate_receipt":candidate_receipt,"final_receipt":final_receipt,
        "initial_evaluation":original_outputs,"final_evaluation":final_evaluation,"metrics":metric,"rowwise":rowwise,"reached_frontier_outcomes":frontier_outcomes,"useful_candidate":selected && final_evaluation["complete"].as_u64().unwrap_or(0)>8,"native_code_proposals":proposals,"gradient_receipt":gradient_receipt,"construction_seconds":construction_seconds,"optimizer_updates":0,
        "scope":"single exposed prototype intervention plus one coefficient compensation; original/intermediate/final identities separate; actual own-prefix outputs and native objective reported separately; no transfer/chat/energy qualification"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn claimed_child_checkpoint_receipt_reuses_exact_authenticated_root_plan() -> Result<()> {
        struct Temporary(PathBuf);
        impl Drop for Temporary {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let temp = Temporary(std::env::temp_dir().join(format!(
            "uor-compensation-child-{}-{unique}",
            std::process::id()
        )));
        fs::create_dir(&temp.0)?;
        let authority = temp.0.join("authority");
        fs::create_dir(&authority)?;
        let frontier = json!({"schema":"uor-r4.reached-frontier-plan/1","terms":[{"index":399,"target":617}],"canonical_reference":{"rows":[{"id":"frozen"}]}});
        fs::write(authority.join("plan.json"), serde_json::to_vec(&frontier)?)?;
        let parent: Args = serde_json::from_value(
            json!({"mode":"joint_continuation","credit":"raw_identity","seed":1001,
            "checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"parent","training_inputs":"inputs","training_labels":"labels","development_inputs":"inputs","development_labels":"labels",
            "maximum_seconds":3600,"maximum_report_bytes":1073741824,"out":temp.0.join("outer"),"reached_frontier_objective":true,
            "reference_replay":{"plan_root":authority,"reference_root":"donor","expected_plan_report_sha256":"unused-test-pin","expected_plan_manifest_sha256":"unused-test-pin","lambda":1.0}}),
        )?;
        report_output::claim(&parent.out)?;
        fs::write(
            parent.out.join("frontier-plan.json"),
            serde_json::to_vec_pretty(&frontier)?,
        )?;
        fs::write(
            parent.out.join("reference-plan.json"),
            serde_json::to_vec_pretty(&frontier["canonical_reference"])?,
        )?;
        let binding = reference_binding(&parent)?;
        assert_ne!(binding["plan_sha256"], binding["copied_plan_sha256"]);
        for name in ["prototype-intermediate", "native-candidate-00"] {
            let mut child = parent.clone();
            child.out = parent.out.join(name);
            report_output::claim(&child.out)?;
            // This is the actual previously failing checkpoint-receipt seam.
            assert!(reference_binding(&child).is_err());
            copy_child_reference(&parent, &child)?;
            assert_eq!(reference_binding(&child)?, binding);
            assert_eq!(
                read(&child.out.join("child-plan-binding.json"))?["reference_binding"],
                binding
            );
            for file in ["frontier-plan.json", "reference-plan.json"] {
                assert_eq!(
                    fs::read(parent.out.join(file))?,
                    fs::read(child.out.join(file))?
                );
            }
        }
        Ok(())
    }
    #[test]
    fn prototype_intervention_changes_only_selected_master_row_and_unique_argmax() -> Result<()> {
        let mut values = vec![0.0; 3 * 2 * 120];
        values[0] = -0.0;
        values[(1 * 2) * 120 + 58] = 2.0;
        values[5 * 120 + 11] = 0.37;
        let altered = canonical_row(&values, 3, 2, 1, 0, 48)?;
        for i in 0..values.len() {
            if !(240..360).contains(&i) {
                assert_eq!(altered[i].to_bits(), values[i].to_bits());
            }
        }
        assert_eq!(altered[240 + 48], 2.0);
        assert_eq!(altered[240 + 58], 0.0);
        assert_eq!(altered[240..360].iter().filter(|&&v| v == 2.0).count(), 1);
        assert!(canonical_row(&values, 3, 2, 3, 0, 48).is_err());
        Ok(())
    }
    #[test]
    fn late_error_and_rejected_gate_restore_prototype_and_coefficients_bitwise() -> Result<()> {
        let device = Device::Cpu;
        let coefficient = Var::from_vec(vec![-0.0f32, 0.07], 2, &device)?;
        let params = BTreeMap::from([("generate.unary".to_owned(), coefficient.clone())]);
        let original = np::snapshot(&params)?;
        let mut original_choices = vec![0.0f32; 120];
        original_choices[0] = -0.0;
        original_choices[58] = 2.0;
        let prototype = Var::from_vec(original_choices.clone(), (1, 1, 120), &device)?;
        for fail_late in [true, false] {
            coefficient.set(&Tensor::from_vec(vec![0.25f32, 0.5], 2, &device)?)?;
            let changed = canonical_row(&original_choices, 1, 1, 0, 0, 48)?;
            prototype.set(&Tensor::from_vec(changed, (1, 1, 120), &device)?)?;
            let outcome: Result<f64> = if fail_late {
                Err(bad("late export error"))
            } else {
                Ok(INTERMEDIATE_LOSS - 0.05)
            };
            let retain = outcome.as_ref().is_ok_and(|&loss| loss_gate(loss));
            settle_model(&params, &original, &prototype, &original_choices, retain)?;
            assert!(np::same_bits(&original, &np::snapshot(&params)?));
            let actual = prototype.flatten_all()?.to_vec1::<f32>()?;
            assert!(actual
                .iter()
                .zip(&original_choices)
                .all(|(a, b)| a.to_bits() == b.to_bits()));
        }
        Ok(())
    }
    #[test]
    fn compensation_gate_uses_original_not_worse_prototype_intermediate() {
        assert!(!loss_gate(INTERMEDIATE_LOSS - 0.05));
        assert!(!loss_gate(ORIGINAL_LOSS));
        assert!(loss_gate(ORIGINAL_LOSS - 0.01));
        assert!(!loss_gate(f64::NAN));
    }
}
