//! One frozen-parent Cue/Prefix coefficient construction with physical-donor replay.
use super::native_proposals as np;
use super::*;
use uor_r4_integer::h4_tables::H4Code;
use uor_r4_training::geometric_occurrence_consumer::source_realizer::{
    CueAngularWeights, PrefixAngularWeights,
};
const REPORT: &str = "2a9f967a955c6dc40b422c94f1b6d1d54f2511a24fb601af97218d49440b9923";
const MANIFEST: &str = "6cbfabf807427f3baa2bfcfda7187e1b00f3a205ec9466b30a7437a410baad94";
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const GENERATE: &str = "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737";
const FIELD: &str = "82ae9daeb402b288e64492d5b299110b36849907019c952609a1cae6612673ee";
#[path = "readout_intermediate.rs"]
mod intermediate;
const DONOR_CACHE_BYTES: usize = 64 * 1024 * 1024;
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_reached_u_root: PathBuf,
    #[serde(default)]
    pub intermediate_candidate: Option<intermediate::Config>,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if a.readout_coadaptation.is_some() {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.native_code_proposals
                && a.reached_frontier_objective
                && a.reference_replay.is_some()
                && a.reached_u.is_none()
                && a.prototype_compensation.is_none()
                && a.retained_context_root.is_none()
                && !a.constrained_emission_learning
                && !a.constrained_context_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only,
            "readout coadaptation requires exclusive two-family joint native proposal mode",
        )?;
    }
    Ok(())
}
fn policy() -> Value {
    json!({"schema":"uor-r4.readout-coadaptation/1","active_names":["cue.coefficients","prefix.coefficients"],"coordinates":1920,"gradient":"one fresh full Generate+allphysicalCopy+frozenU marginal; exact native-bin coefficient gathers; detached conditional donor-state selector credit","construction":"one frozen adjacent Q4 pass; earliest BASEphysicalCopy donor before U/alias pooling; exact frozen bridge/Generate replay on donor change; transactional377winnerguards","objective":"five actual pos3 factual frontiers equal weight.2 each + unchanged original84 successful trajectory weights;89terms; no duplicate8sourcecontrols","guards":"all377 parentcorrectprefixes incl84 completeEOS +293 incompletecorrectprefixes","frozen":"Source/Context/Potential/Generate/prototypes/bias/readmap/U/cue-joint/tokenizer/geometry","selection":"strict fresh revised nativeCE descent +>=1fact corrected +all377 preserved","scope":"exposed authored panel; useful only new completeEOS with original8 retained; no heldout/chat/energy qualification"})
}
fn to_term(t: &frontier::Term) -> np::Term {
    np::Term {
        index: t.index,
        position: t.position,
        target: t.target,
        component: t.component,
        weight: t.weight,
        parent_actual_prefix_ids: Some(t.parent_actual_prefix_ids.clone()),
    }
}
fn authenticate(root: &Path) -> Result<ContinuationParent> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == REPORT
            && sha256_file(&root.join("manifest.json"))? == MANIFEST,
        "readout selectedU report/seal differs",
    )?;
    let r = read(&root.join("report.json"))?;
    let cp = root.join("checkpoint-0001");
    let p = ContinuationParent::from_checkpoint(&cp)?;
    replay_require(
        r["status"] == "COMPLETED"
            && r["mode"] == "reached_u"
            && r["native_code_proposals"]["winner"] == 0
            && r["final_receipt"] == p.receipt
            && p.binding.metadata_sha256 == SOURCE
            && p.generate_sha256 == GENERATE
            && sha256_file(&cp.join("continuation-field.bin"))? == FIELD,
        "readout selectedU checkpoint binding differs",
    )?;
    for (path, pin) in [
        (
            "cue/cue-q4.bin",
            "771864b7581b79a9d7b93137eb5e28e863da1361e04f41c0bf8810e442469781",
        ),
        (
            "prefix/prefix-q4.bin",
            "8d2d2cdf307e08ac0d4021d91b38c9718ceeeb5b17c16344f2330fe27d36e6ee",
        ),
        (
            "cue/cue-joint-q4.bin",
            "33c6021151e84b27cb222a333303ccff093219e46c661b0ca689bd9b92929254",
        ),
    ] {
        replay_require(
            sha256_file(&cp.join(path))? == pin,
            "readout retained sidecar pin differs",
        )?;
    }
    Ok(p)
}
// Conservative construction-only numerical cache, not model/serde/graph RAM.
fn numerical_projection(rows: usize, physical: usize) -> Result<usize> {
    let vectors = rows
        .checked_mul(4096 * 8 * 8)
        .ok_or_else(|| bad("readout numerical projection overflow"))?;
    let postings = physical
        .checked_mul(1024)
        .ok_or_else(|| bad("readout posting projection overflow"))?;
    let trials = rows
        .checked_mul(1920 * 64)
        .ok_or_else(|| bad("readout trial projection overflow"))?;
    DONOR_CACHE_BYTES
        .checked_add(vectors)
        .and_then(|x| x.checked_add(postings))
        .and_then(|x| x.checked_add(trials))
        .and_then(|x| x.checked_add(16 * 1024 * 1024))
        .ok_or_else(|| bad("readout auxiliary projection overflow"))
}
fn gate(before: f64, after: f64, correct: usize) -> bool {
    before.is_finite()
        && after.is_finite()
        && correct > 0
        && after < before - 1e-10 * (1. + before.abs())
}
fn export(
    a: &Args,
    step: usize,
    p: &ContinuationParent,
    l: &Loaded,
    cue_weights: &CueAngularWeights,
    prefix_weights: &mut PrefixAngularWeights,
) -> Result<(ContinuationParent, NativeContinuationField, Value)> {
    disk_floor(a)?;
    let root = a.out.join(format!("checkpoint-{step:04}"));
    fs::create_dir(&root)?;
    let input = a
        .readout_coadaptation
        .as_ref()
        .ok_or_else(|| bad("readout config absent"))?
        .retained_reached_u_root
        .join("checkpoint-0001");
    replay_require(
        size(&a.out)?
            .checked_add(size(&input)?)
            .and_then(|x| x.checked_add(1 << 20))
            .is_some_and(|x| x < a.maximum_report_bytes),
        "readout export report projection exceeds cap",
    )?;
    for e in fs::read_dir(&input)? {
        let e = e?;
        let name = e.file_name();
        if name == "receipt.json" || name == "cue" || name == "prefix" {
            continue;
        }
        if e.file_type()?.is_dir() {
            copy_frozen_tree(&e.path(), &root.join(name))?;
        } else if e.file_type()?.is_file() {
            fs::copy(e.path(), root.join(&name))?;
            replay_require(
                sha256_file(&e.path())? == sha256_file(&root.join(name))?,
                "readout frozen file copy differs",
            )?;
        } else {
            return Err(bad("readout nonregular checkpoint member"));
        }
    }
    cue_weights.save(&root.join("cue"))?;
    let cue = l.frozen.compile_cue_carrier(cue_weights.native()?)?;
    prefix_weights.rebind_cue(&l.frozen, &root.join("cue"), &cue)?;
    prefix_weights.save(&root.join("prefix"))?;
    let cpu_cue = CueAngularWeights::load(&root.join("cue"), &l.frozen, &p.native_directory)?;
    let cpu_carrier = l.frozen.compile_cue_carrier(cpu_cue.native()?)?;
    let cpu_prefix = PrefixAngularWeights::load(
        &root.join("prefix"),
        &l.frozen,
        &p.native_directory,
        &root.join("cue"),
        &cpu_carrier,
    )?;
    replay_require(
        cpu_cue.packed_coefficients()? == cue_weights.packed_coefficients()?
            && cpu_prefix.packed_coefficients()? == prefix_weights.packed_coefficients()?,
        "readout independent master/native reload differs",
    )?;
    let inventory = save_masters(
        &root.join("readout-source"),
        &cue_weights
            .parameters()
            .into_iter()
            .chain(prefix_weights.parameters())
            .collect(),
    )?;
    let mut receipt = p.receipt.clone();
    receipt["step"] = json!(step);
    receipt["source_commit"] = json!(option_env!("UOR_BUILD_SOURCE_COMMIT"));
    receipt["cue_sha256"] = json!(sha256_file(&root.join("cue/cue-q4.bin"))?);
    receipt["prefix_sha256"] = json!(sha256_file(&root.join("prefix/prefix-q4.bin"))?);
    receipt["cue_metadata"] = serde_json::to_value(cue.metadata())?;
    receipt["prefix_metadata"] = read(&root.join("prefix/native-metadata.json"))?;
    receipt["readout_parameters"] = inventory;
    receipt["readout_coadaptation"] = if a
        .readout_coadaptation
        .as_ref()
        .and_then(|c| c.intermediate_candidate.as_ref())
        .is_some()
    {
        intermediate::policy()
    } else {
        policy()
    };
    receipt["active_parameter_names"] = json!(["cue.coefficients", "prefix.coefficients"]);
    receipt["optimizer_updates"] = json!(0);
    receipt["reference_replay"] = reference_binding(a)?;
    receipt["masters_independently_reloaded"] = json!(true);
    receipt["native_independently_reloaded"] = json!(true);
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    let current = ContinuationParent::from_checkpoint(&root)?;
    replay_require(
        current.binding == p.binding
            && current.generate == p.generate
            && current.bridge == p.bridge
            && current.joint == p.joint
            && current.exp == p.exp
            && sha256_file(&root.join("continuation-field.bin"))? == FIELD,
        "readout export changed frozen upstream/U/joint",
    )?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        &p.binding,
        current.generator()?.generate_model(),
    )?;
    Ok((current, field, receipt))
}
struct DonorInput {
    query: Vec<H4Code>,
    sources: Vec<Vec<H4Code>>,
}
fn capture(
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    terms: &[np::Term],
    expected: Option<&readout_constraints::Construction>,
) -> Result<(
    Vec<readout_constraints::ReadoutProtectedPool>,
    Vec<DonorInput>,
    Value,
)> {
    let bytes = field.to_bytes()?;
    let hash = sha256_bytes(&bytes);
    let mut g = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &hash,
    })?;
    let mut banks = BTreeMap::new();
    let mut pools = Vec::new();
    let mut donors = Vec::new();
    let mut rows = Vec::new();
    for (i, t) in terms.iter().enumerate() {
        if let std::collections::btree_map::Entry::Vacant(e) = banks.entry(t.index) {
            e.insert(g.admit_bank(continuation_snapshot(&eps[t.index].packet)?)?);
        }
        let prefix = t
            .parent_actual_prefix_ids
            .as_deref()
            .ok_or_else(|| bad("readout actual prefix absent"))?;
        let s = g.step(&banks[&t.index], prefix)?;
        replay_require(
            s.actions.summary.chosen_token_id == t.target,
            "readout exported guard winner differs",
        )?;
        if let Some(c) = expected {
            replay_require(
                s.generate_raw_scores_q24 == c.final_generate_scores[i]
                    && s.copy_raw_scores_q24 == c.final_copy_scores[i]
                    && s.actions.summary == c.final_pool_summaries[i],
                "readout constructed/exported pool differs",
            )?;
        }
        let bank = s
            .bank_trace
            .as_ref()
            .ok_or_else(|| bad("readout full bank trace absent"))?;
        let b = &bank.cue_bank.bank;
        let u = &s
            .continuation
            .as_ref()
            .ok_or_else(|| bad("readout frozenU witness absent"))?
            .delta_scores_q24;
        let bridge = s
            .bridge
            .as_ref()
            .ok_or_else(|| bad("readout native bridge absent"))?;
        let base_g = s
            .generate_raw_scores_q24
            .iter()
            .zip(u)
            .map(|(a, b)| {
                a.checked_sub(*b)
                    .ok_or_else(|| bad("readout baseG overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        let base_c = s
            .copy_raw_scores_q24
            .iter()
            .zip(&s.copy_token_ids)
            .map(|(a, id)| {
                a.checked_sub(u[*id as usize])
                    .ok_or_else(|| bad("readout baseCopy overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        replay_require(
            readout_constraints::earliest_base_donor(&base_c)? == bridge.selected_ordinal,
            "readout base donor differs",
        )?;
        let mut cue_keys = Vec::new();
        let mut prefix_keys = Vec::new();
        for j in 0..b.candidates.len() {
            for lane in 0..8 {
                cue_keys.push(
                    bank.cue_bank.carrier.angular_indices[lane][j]
                        .map(|bin| (lane * 120 + usize::from(bin)) as u16),
                );
                prefix_keys
                    .push((lane * 120 + usize::from(bank.prefix.angular_indices[lane][j])) as u16);
            }
        }
        donors.push(DonorInput {
            query: bridge.query_state.clone(),
            sources: b
                .candidates
                .iter()
                .map(|c| {
                    b.context.states[c.context_position]
                        .iter()
                        .map(|x| H4Code::try_from(*x).map_err(Into::into))
                        .collect()
                })
                .collect::<Result<Vec<Vec<H4Code>>>>()?,
        });
        rows.push(json!({"term":t,"id":eps[t.index].packet.id,"actual_prefix_ids":prefix,"generate_q24":s.generate_raw_scores_q24,"base_generate_q24":base_g,"copy_ids":s.copy_token_ids,"copy_q24":s.copy_raw_scores_q24,"base_copy_q24":base_c,"frozen_u_q24":u,"cue_keys":cue_keys,"prefix_keys":prefix_keys,"pool":s.actions.summary,"token_masses":s.actions.token_masses,"bridge":json!({"selected_ordinal":bridge.selected_ordinal,"selected_candidate":bridge.selected_candidate,"query_state":bridge.query_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"source_state":bridge.source_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_codes":bridge.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_scores_q24":bridge.action_scores_q24}),"base_donor":bridge.selected_ordinal,"bank_trace":bank}));
        pools.push(readout_constraints::ReadoutProtectedPool {
            required_token: t.target,
            base_copy_q24: base_c,
            copy_ids: s.copy_token_ids,
            frozen_u_q24: u.clone(),
            cue_keys,
            prefix_keys,
            base_generate_q24: base_g,
            donor: bridge.selected_ordinal,
        });
    }
    Ok((pools, donors, json!(rows)))
}
fn require_gradient_inventory(inventory: &Value) -> Result<()> {
    let entries = inventory
        .as_object()
        .ok_or_else(|| bad("readout gradient inventory is not an object"))?;
    replay_require(
        entries.len() == 2
            && ["cue.coefficients", "prefix.coefficients"]
                .iter()
                .all(|name| entries.contains_key(*name)),
        "readout saved gradient inventory must contain exactly both active families",
    )?;
    for name in ["cue.coefficients", "prefix.coefficients"] {
        let row = &inventory[name];
        replay_require(
            row["shape"] == json!([960])
                && row["bytes"] == 3840
                && row["missing_gradient_filled_zero"] == false,
            "readout saved gradient must contain960 measured values, not synthetic zeros",
        )?;
        replay_require(
            row["file"] == format!("native-code-ranking-gradients/{name}.f32le"),
            "readout saved gradient path differs",
        )?;
    }
    Ok(())
}
fn gradient(
    a: &Args,
    start: Instant,
    d: &Device,
    l: &Loaded,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    cue_weights: &CueAngularWeights,
    prefix_weights: &PrefixAngularWeights,
    params: &BTreeMap<String, Var>,
    eps: &[Episode],
    terms: &[np::Term],
) -> Result<(np::Shadows, Value, f64)> {
    let prepared = l.source.prepare_context_potential_on_device(&l.frozen, d)?;
    let cue = l.frozen.compile_cue_carrier(cue_weights.native()?)?;
    let prefix = l
        .frozen
        .compile_prefix_transport(&cue, prefix_weights.native()?)?;
    let gs = l.generate.prepare_native()?;
    let cat = l.prepared_categorical(d)?;
    let u = ContinuationLearningWeights::from_native(field, &gs.native, l.integer.binding(), d)?;
    let us = u.prepare_native(&p.binding, &gs.native)?;
    replay_require(
        us.native.to_bytes()? == field.to_bytes()?,
        "readout frozenU gradient native differs",
    )?;
    let mut learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_categorical_read_state_bridge(&cat)?
        .with_read_selector_credit(true)
        .with_readout_coefficient_credit(cue_weights, prefix_weights)?
        .with_continuation_field(&u, &us)?
        .with_continuation_context_credit(false)?;
    let bytes = field.to_bytes()?;
    let hash = sha256_bytes(&bytes);
    let mut native = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &hash,
    })?;
    let mut banks = BTreeMap::new();
    let clock = Instant::now();
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut combined = 0.;
    let mut rows = Vec::new();
    for t in terms {
        deadline(a, start)?;
        let e = &eps[t.index];
        let actual = t
            .parent_actual_prefix_ids
            .as_deref()
            .ok_or_else(|| bad("gradient actual prefix absent"))?;
        let out =
            learner.forward_bank(&e.segments()?, &e.packet.query_ids, actual, &cue, &prefix)?;
        if let std::collections::btree_map::Entry::Vacant(v) = banks.entry(t.index) {
            v.insert(native.admit_bank(continuation_snapshot(&e.packet)?)?);
        }
        let hard = native.step(&banks[&t.index], actual)?;
        replay_require(
            out.generate.scores_q24 == hard.generate_raw_scores_q24
                && out.copy_token_ids == hard.copy_token_ids
                && out.copy_scores_q24 == hard.copy_raw_scores_q24
                && out.final_state_codes == hard.post_state
                && out.actions.summary == hard.actions.summary,
            "readout fresh CPU/CUDA graph vs actual native fullpool differs",
        )?;
        let loss = (out.loss_with_credit(t.target, a.credit.policy())? * t.weight)?;
        combined += loss.to_scalar::<f32>()? as f64;
        let grads = loss.backward()?;
        for (name, var) in params {
            let g = grads
                .get(var.as_tensor())
                .ok_or_else(|| bad("readout coefficient family gradient disconnected"))?;
            let next = if let Some(old) = sums.remove(name) {
                (&old + g)?
            } else {
                g.clone()
            };
            sums.insert(name.clone(), next.detach());
        }
        rows.push(json!({"term":t,"id":e.packet.id,"actual_prefix_ids":actual,"generate_q24":hard.generate_raw_scores_q24,"copy_ids":hard.copy_token_ids,"copy_q24":hard.copy_raw_scores_q24,"bank_trace":hard.bank_trace,"pool":hard.actions.summary,"post_state":hard.post_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"continuation":continuation_witness(&hard)?,"coefficient_credit":"both exact native-bin families before physical-selector credit"}));
    }
    d.synchronize()?;
    let seconds = clock.elapsed().as_secs_f64();
    write(
        a,
        "coefficient-gradient-receipt.json",
        &json!({"positions":terms.len(),"combined":combined,"elapsed_seconds":seconds,"active_names":params.keys().collect::<Vec<_>>(),"upstream_parameters_applied":false,"rows":rows}),
    )?;
    let inventory = np::save_ranking_gradients(a, params, &sums)?;
    require_gradient_inventory(&inventory)?;
    for name in ["cue.coefficients", "prefix.coefficients"] {
        let path = a
            .out
            .join(format!("native-code-ranking-gradients/{name}.f32le"));
        replay_require(
            fs::metadata(&path)?.len() == 3840 && inventory[name]["sha256"] == sha256_file(&path)?,
            "readout saved measured gradient file bytes/hash differ",
        )?;
    }

    let values = sums
        .iter()
        .map(|(name, t)| {
            Ok((
                name.clone(),
                t.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?,
            ))
        })
        .collect::<Result<np::Shadows>>()?;
    Ok((
        values,
        json!({"inventory":inventory,"combined":combined,"elapsed_seconds":seconds}),
        combined,
    ))
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    let root = &a
        .readout_coadaptation
        .as_ref()
        .ok_or_else(|| bad("readout config absent"))?
        .retained_reached_u_root;
    if let Some(c) = a
        .readout_coadaptation
        .as_ref()
        .and_then(|c| c.intermediate_candidate.as_ref())
    {
        intermediate::authenticate_inputs(c)?;
    }
    let p = authenticate(root)?;
    let mut selected_args = a.clone();
    selected_args.checkpoint = root.join("checkpoint-0001");
    let l = load_joint_continuation(&selected_args, &p, d)?;
    let cue = l.frozen.compile_cue_carrier(l.cue.clone())?;
    let prefix = l
        .frozen
        .compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let cw = CueAngularWeights::from_native(&l.frozen, &p.native_directory, &cue, d)?;
    let mut pw = PrefixAngularWeights::from_native(
        &l.frozen,
        &p.native_directory,
        &root.join("checkpoint-0001/cue"),
        &cue,
        &prefix,
        d,
    )?;
    let params = cw
        .parameters()
        .into_iter()
        .chain(pw.parameters())
        .collect::<BTreeMap<_, _>>();
    let original = np::snapshot(&params)?;
    replay_require(
        original.len() == 2 && original.values().all(|v| v.len() == 960),
        "readout1920 master population differs",
    )?;
    let result = if let Some(c) = a
        .readout_coadaptation
        .as_ref()
        .and_then(|c| c.intermediate_candidate.as_ref())
    {
        intermediate::run(a, start, c, &p, &l, &cw, &mut pw, &params, &original)
    } else {
        run_inner(a, start, d, &p, &l, &cw, &mut pw, &params, &original)
    };
    if result.is_err() {
        np::restore(&params, &original)?;
    }
    result
}
fn run_inner(
    a: &Args,
    start: Instant,
    d: &Device,
    p: &ContinuationParent,
    l: &Loaded,
    cw: &CueAngularWeights,
    pw: &mut PrefixAngularWeights,
    params: &BTreeMap<String, Var>,
    original: &np::Shadows,
) -> Result<Value> {
    let root = &a
        .readout_coadaptation
        .as_ref()
        .ok_or_else(|| bad("readout config absent"))?
        .retained_reached_u_root;
    let legal = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?
        .legal_token_ids()
        .iter()
        .copied()
        .collect();
    replay_require(
        sha256_file(&a.training_inputs)? == INPUT_SHA
            && sha256_file(&a.training_labels)? == LABEL_SHA
            && a.training_inputs == a.development_inputs
            && a.training_labels == a.development_labels,
        "readout frozen512 paired panel differs",
    )?;
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &p.integer,
        &p.tokenizer,
        &legal,
        512,
    )?;
    pairs(&eps)?;
    let old: frontier::Plan = serde_json::from_slice(&fs::read(root.join("frontier-plan.json"))?)?;
    let (plan, guard_terms, plan_receipt) =
        frontier::readout_endpoint_plan(root, &old, &eps, &p.tokenizer)?;
    let _ = frontier::readout_components(&plan)?;
    let terms = plan.terms.iter().map(to_term).collect::<Vec<_>>();
    let guards = guard_terms.iter().map(to_term).collect::<Vec<_>>();
    replay_require(
        terms.len() == 89 && guards.len() == 377,
        "readout89objective377guards differ",
    )?;
    write(a, "frontier-plan.json", &serde_json::to_value(&plan)?)?;
    write(
        a,
        "reference-plan.json",
        &serde_json::to_value(&plan.canonical_reference)?,
    )?;
    write(a, "selected-plan-derivation.json", &plan_receipt)?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    write(a, "protected-terms.json", &json!(guards))?;
    let mut initial = read(&root.join("development-0001.json"))?;
    for (i, row) in initial["rows"]
        .as_array_mut()
        .ok_or_else(|| bad("readout parent summary rows absent"))?
        .iter_mut()
        .enumerate()
    {
        let saved = reference_saved_row(root, row)?;
        let name = format!("original-row-{i:04}.json");
        write(a, &name, &saved)?;
        row["row_file"] = json!(name);
        row["row_sha256"] = json!(sha256_file(
            &a.out.join(format!("original-row-{i:04}.json"))
        )?);
    }
    write(a, "original-development.json", &initial)?;
    write(
        a,
        "admission.json",
        &json!({"policy":policy(),"selected_report_sha256":REPORT,"selected_manifest_sha256":MANIFEST,"coordinates":1920,"objective_terms":89,"protected_pools":377,"donor_cache_limit_bytes":DONOR_CACHE_BYTES,"maximum_auxiliary_cache_bytes":256*1024*1024,"quarter_masters":"derived exactlyfrom960signednativeQ4codes per family","optimizer_updates":0}),
    )?;
    let (before, field, initial_receipt) = export(a, 0, p, l, cw, pw)?;
    replay_require(
        before.cue == p.cue && before.prefix == p.prefix,
        "baseline quarter restoration changed native sidecars",
    )?;
    let baseline = np::score(
        a,
        &before,
        &field,
        &eps,
        &plan.canonical_reference,
        &terms,
        start,
    )?;
    write(a, "native-code-parent-objective.json", &baseline)?;
    let (pools, donor_inputs, original_pools) = capture(&before, &field, &eps, &guards, None)?;
    write(a, "original-protected-pools.json", &original_pools)?;
    drop(original_pools);
    let physical = pools.iter().map(|p| p.copy_ids.len()).sum::<usize>();
    let auxiliary = numerical_projection(pools.len(), physical)?;
    replay_require(
        auxiliary <= 256 * 1024 * 1024,
        "readout numerical auxiliary cache projection exceeds256MiB",
    )?;
    let captured_bytes = fs::metadata(a.out.join("original-protected-pools.json"))?.len();
    let original_rows_bytes = (0..512)
        .map(|i| fs::metadata(a.out.join(format!("original-row-{i:04}.json"))).map(|m| m.len()))
        .collect::<std::io::Result<Vec<_>>>()?
        .iter()
        .sum::<u64>();
    let checkpoint_bytes = size(&root.join("checkpoint-0001"))?;
    let projected_report = size(&a.out)?
        .checked_add(captured_bytes.saturating_mul(2))
        .and_then(|x| x.checked_add(100 * 1024 * 1024))
        .and_then(|x| x.checked_add(256 * 1024 * 1024))
        .and_then(|x| x.checked_add(original_rows_bytes.saturating_mul(2)))
        .and_then(|x| x.checked_add(64 * 1024 * 1024))
        .and_then(|x| x.checked_add(checkpoint_bytes.saturating_mul(3)))
        .ok_or_else(|| bad("readout report projection overflow"))?;
    write(
        a,
        "resource-projection.json",
        &json!({"actual_guard_count":pools.len(),"actual_physical_occurrences":physical,"numerical_auxiliary_upper_bound_bytes":auxiliary,"maximum_numerical_auxiliary_bytes":256*1024*1024,"bound_components":{"donor_cache_payload":DONOR_CACHE_BYTES,"eight_coexisting4096_i64_vectors_per_guard":pools.len()*4096*8*8,"physical_postings_keys_copy_buffers_sources_and_growth":physical*1024,"complete_trial_affectedrows_and_donortransitions_capacity":pools.len()*1920*64,"map_vec_headers_and_remaining_small_buffers":16*1024*1024},"excluded_from_cache_scope":"model masters/autodiff device graph and JSON serialization charged to8GiB supervisedRAM","baseline_guard_report_bytes":captured_bytes,"original512_report_bytes":original_rows_bytes,"selected_checkpoint_bytes":checkpoint_bytes,"projected_complete_report_bytes":projected_report,"report_cap":a.maximum_report_bytes,"projection_scope":"pregradient conservative estimate; actual writes remain bounded"}),
    )?;
    replay_require(
        projected_report < a.maximum_report_bytes,
        "readout complete report projection exceeds cap beforegradient",
    )?;
    let (gradients, gradient_receipt, combined) = gradient(
        a, start, d, l, &before, &field, cw, pw, params, &eps, &terms,
    )?;
    let base = baseline["combined"]
        .as_f64()
        .ok_or_else(|| bad("readout baselineCE absent"))?;
    replay_require(
        (combined - base).abs() < 1e-5,
        "readout gradient/native baseline objective differs",
    )?;
    let native = before.generator()?;
    let model = native.generate_model();
    let bridge =
        NativeGeometricReadStateBridge::from_bytes(&before.bridge, before.integer.binding())?;
    let mut cache = BTreeMap::<(usize, usize), Vec<i64>>::new();
    let mut cache_bytes = 0usize;
    let mut peak_cache_bytes = 0usize;
    let mut callback_calls = 0usize;
    let mut native_donor_replays = 0usize;
    let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let clock = Instant::now();
    let construction =
        readout_constraints::construct(original, &gradients, pools, &mut reducer, |i, donor| {
            callback_calls += 1;
            if let Some(scores) = cache.get(&(i, donor)) {
                return Ok(scores.clone());
            }
            deadline(a, start)?;
            let data = donor_inputs
                .get(i)
                .ok_or_else(|| bad("readout donor row absent"))?;
            let source = data
                .sources
                .get(donor)
                .ok_or_else(|| bad("readout donor ordinal absent"))?;
            let mut post = vec![H4Code::IDENTITY; 8];
            let mut actions = post.clone();
            let mut action_scores = vec![0i64; 960];
            let mut bc = BridgeReadCounts::default();
            bridge.apply_into(
                &data.query,
                source,
                &mut post,
                &mut actions,
                &mut action_scores,
                &mut bc,
            )?;
            let mut scores = vec![0i64; 4096];
            model.score_into(&post, &mut scores, &mut GenerateReadCounts::default())?;
            native_donor_replays += 1;
            let bytes = scores.len() * std::mem::size_of::<i64>();
            if cache_bytes + bytes > DONOR_CACHE_BYTES {
                cache.clear();
                cache_bytes = 0;
            }
            cache.insert((i, donor), scores.clone());
            cache_bytes += bytes;
            peak_cache_bytes = peak_cache_bytes.max(cache_bytes);
            Ok(scores)
        })?;
    let construction_seconds = clock.elapsed().as_secs_f64();
    write(
        a,
        "readout-construction.json",
        &json!({"construction":construction,"construction_seconds":construction_seconds,"gradient_receipt":gradient_receipt,"donor_callback_calls":callback_calls,"native_donor_replays":native_donor_replays,"peak_donor_cache_payload_bytes":peak_cache_bytes,"cache_policy":"immutable exact donor vectors;clearall on64MiB numerical payload ceiling; rejected reads canremain","policy":policy()}),
    )?;
    drop(cache);
    drop(donor_inputs);
    let mut child = a.clone();
    child.out = a.out.join("native-candidate-00");
    report_output::claim(&child.out)?;
    copy_child_plans(a, &child)?;
    let candidate = np::attempt_restored(params, original, || -> Result<(Value, Value)> {
        np::apply_edits(params, original, &construction.edits)?;
        let (candidate, field, receipt) = export(&child, 0, p, l, cw, pw)?;
        verify_codes(cw, pw, &np::edited(original, &construction.edits)?)?;
        let (_, _, rows) = capture(&candidate, &field, &eps, &guards, Some(&construction))?;
        write(&child, "reloaded-protected-pools.json", &rows)?;
        let objective = np::score(
            &child,
            &candidate,
            &field,
            &eps,
            &plan.canonical_reference,
            &terms,
            start,
        )?;
        write(&child, "native-objective.json", &objective)?;
        Ok((objective, receipt))
    });
    let child_report = match &candidate {
        Ok((objective, receipt)) => {
            json!({"status":"COMPLETED","objective":objective,"receipt":receipt})
        }
        Err(e) => {
            json!({"status":"FAILED","error":e.to_string(),"scope":"executionfailure notmodelnegative"})
        }
    };
    write(&child, "report.json", &child_report)?;
    report_output::seal(&child.out)?;
    report_output::verify(&child.out)?;
    let (candidate_objective, candidate_receipt) = candidate?;
    let corrected = candidate_objective["terms"]
        .as_array()
        .ok_or_else(|| bad("readout candidate terms absent"))?
        .iter()
        .filter(|r| {
            r["term"]["component"] == 0 && r["pool"]["chosen_token_id"] == r["term"]["target"]
        })
        .count();
    let loss = candidate_objective["combined"]
        .as_f64()
        .ok_or_else(|| bad("readout candidateCE absent"))?;
    let selected = gate(base, loss, corrected);
    if selected {
        np::apply_edits(params, original, &construction.edits)?;
    } else {
        np::restore(params, original)?;
    }
    let (final_parent, final_field, final_receipt) = export(a, 1, p, l, cw, pw)?;
    verify_codes(
        cw,
        pw,
        &if selected {
            np::edited(original, &construction.edits)?
        } else {
            original.clone()
        },
    )?;
    let final_objective = np::score(
        a,
        &final_parent,
        &final_field,
        &eps,
        &plan.canonical_reference,
        &terms,
        start,
    )?;
    replay_require(
        final_objective["combined"]
            == if selected {
                candidate_objective["combined"].clone()
            } else {
                baseline["combined"].clone()
            },
        "readout final selection objective differs",
    )?;
    write(a, "native-code-final-objective.json", &final_objective)?;
    let (_, _, final_pools) = capture(
        &final_parent,
        &final_field,
        &eps,
        &guards,
        if selected { Some(&construction) } else { None },
    )?;
    write(a, "final-protected-pools.json", &final_pools)?;
    drop(final_pools);
    let success = frontier::successful_indices(&plan);
    let subset = success.iter().map(|i| &eps[*i]).collect::<Vec<_>>();
    let early = continuation_evaluate_rows(
        a,
        "pilot-original8",
        &final_parent,
        &final_field,
        &subset,
        start,
    )?;
    write(
        a,
        "pilot-original8-receipt.json",
        &json!({"indices":success,"evaluation":early}),
    )?;
    replay_require(early["complete"] == 8, "readout early8 lostcompleteanswers")?;
    let final_evaluation = continuation_evaluate(
        a,
        "development-0001",
        &final_parent,
        &final_field,
        &eps,
        start,
    )?;
    let final_metrics = metrics(a, &final_evaluation, &eps)?;
    write(a, "metrics-0001.json", &final_metrics)?;
    let rowwise = joint_row_comparison(&initial, &final_evaluation)?;
    write(a, "complete-row-comparison.json", &rowwise)?;
    replay_require(
        rowwise["lost_complete_ids"]
            .as_array()
            .is_some_and(|x| x.is_empty()),
        "readout full512 lostparentcompleteanswers",
    )?;
    let prior = read(&root.join("report.json"))?;
    let prior_comparison = joint_row_comparison(&prior["initial_evaluation"], &final_evaluation)?;
    write(a, "pre-u-row-comparison.json", &prior_comparison)?;
    let mut class_correct = [0usize; 3];
    for r in candidate_objective["terms"]
        .as_array()
        .ok_or_else(|| bad("readout final terms absent"))?
    {
        if r["term"]["component"] == 0 && r["chosen"] == r["term"]["target"] {
            let index = r["term"]["index"]
                .as_u64()
                .ok_or_else(|| bad("readout task index absent"))?;
            let class = if index == 97 || index == 156 {
                0
            } else if index == 151 || index == 392 {
                1
            } else {
                2
            };
            class_correct[class] += 1;
        }
    }
    let proposals = json!({"winner":if selected{json!(0)}else{Value::Null},"parent_combined":base,"candidate_combined":loss,"corrected_factual_frontiers":corrected,"mechanism_classes_corrected":{"wrong_source":class_correct[0],"correct_source_wrong_offset":class_correct[1],"mixed_245":class_correct[2]},"two_class_mechanism_positive":selected && class_correct[0]>0 && class_correct[1]>0,"policy":policy(),"optimizer_updates":0});
    write(a, "native-code-proposals.json", &proposals)?;
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"readout_coadaptation","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"immediate_input":{"root":root,"report_sha256":REPORT,"manifest_sha256":MANIFEST,"generate_sha256":GENERATE,"continuation_sha256":FIELD},"initial_receipt":initial_receipt,"candidate_receipt":candidate_receipt,"final_receipt":final_receipt,"baseline_objective":baseline,"candidate_objective":candidate_objective,"final_objective":final_objective,"selected_plan_derivation":plan_receipt,"initial_evaluation":initial,"final_evaluation":final_evaluation,"metrics":final_metrics,"rowwise":rowwise,"native_code_proposals":proposals,"optimizer_updates":0,"useful_candidate":selected && final_evaluation["complete"].as_u64().is_some_and(|x|x>8),"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"one two-family frozen-parent readout construction; CE/source-role/offset/frontier gains distinctfromactual ownprefixEOS; no generalchat/transfer/energy claim"}),
    )
}
fn verify_codes(
    cue: &CueAngularWeights,
    prefix: &PrefixAngularWeights,
    expected: &np::Shadows,
) -> Result<()> {
    let actual = cue
        .parameters()
        .into_iter()
        .chain(prefix.parameters())
        .collect::<BTreeMap<_, _>>();
    replay_require(
        np::same_bits(&np::snapshot(&actual)?, expected),
        "readout active master selection differs",
    )?;
    for (name, packed) in [
        ("cue.coefficients", cue.packed_coefficients()?),
        ("prefix.coefficients", prefix.packed_coefficients()?),
    ] {
        let values = expected
            .get(name)
            .ok_or_else(|| bad("readout expected family absent"))?;
        replay_require(
            values.len() == 960 && packed.len() == 480,
            "readout Q4shape differs",
        )?;
        for (i, v) in values.iter().enumerate() {
            let n = (packed[i / 2] >> ((i & 1) * 4)) & 15;
            let q = if n >= 8 { n as i8 - 16 } else { n as i8 };
            replay_require(
                q != -8 && q == (v * 4.).round() as i8,
                "readout expected packing differs",
            )?;
        }
    }
    Ok(())
}

fn copy_frozen_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir(to)?;
    for e in fs::read_dir(from)? {
        let e = e?;
        let kind = e.file_type()?;
        if kind.is_dir() {
            copy_frozen_tree(&e.path(), &to.join(e.file_name()))?;
        } else if kind.is_file() {
            fs::copy(e.path(), to.join(e.file_name()))?;
            replay_require(
                sha256_file(&e.path())? == sha256_file(&to.join(e.file_name()))?,
                "frozenfile copyhash differs",
            )?;
        } else {
            return Err(bad("nonregular frozenartifact excluded"));
        }
    }
    Ok(())
}

fn copy_child_plans(a: &Args, child: &Args) -> Result<()> {
    let parsed = read(&a.out.join("frontier-plan.json"))?;
    replay_require(
        read(&a.out.join("reference-plan.json"))? == parsed["canonical_reference"],
        "revisedreference binding differs",
    )?;
    let mut files = BTreeMap::new();
    for name in ["frontier-plan.json", "reference-plan.json"] {
        let raw = fs::read(a.out.join(name))?;
        fs::write(child.out.join(name), &raw)?;
        replay_require(
            fs::read(child.out.join(name))? == raw,
            "childplan exactbytes differ",
        )?;
        files.insert(name, json!({"bytes":raw.len(),"sha256":sha256_bytes(&raw)}));
    }
    replay_require(
        reference_binding(a)? == reference_binding(child)?,
        "child revisedreceipt binding differs",
    )?;
    write(
        child,
        "child-plan-binding.json",
        &json!({"files":files,"reference_binding":reference_binding(child)?,"authority":"parent claimed revisedplan derived from sealedselectedcompensation; originalreference remains separatelyauthenticated"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readout_gradient_inventory_rejects_missing_or_synthetic_family() -> Result<()> {
        let mut value = json!({"cue.coefficients":{"shape":[960],"bytes":3840,"file":"native-code-ranking-gradients/cue.coefficients.f32le","missing_gradient_filled_zero":false},"prefix.coefficients":{"shape":[960],"bytes":3840,"file":"native-code-ranking-gradients/prefix.coefficients.f32le","missing_gradient_filled_zero":false}});
        require_gradient_inventory(&value)?;
        value["prefix.coefficients"]["missing_gradient_filled_zero"] = json!(true);
        assert!(require_gradient_inventory(&value).is_err());
        value["prefix.coefficients"]["missing_gradient_filled_zero"] = json!(false);
        value["prefix.coefficients"]["shape"] = json!([8, 120]);
        assert!(require_gradient_inventory(&value).is_err());
        value
            .as_object_mut()
            .ok_or_else(|| bad("test inventory absent"))?
            .remove("prefix.coefficients");
        assert!(require_gradient_inventory(&value).is_err());
        Ok(())
    }
    #[test]
    fn readout_gate_requires_new_fact_and_fresh_descent() {
        assert!(!gate(4., 3., 0));
        assert!(!gate(4., 4., 1));
        assert!(!gate(f64::NAN, 3., 1));
        assert!(gate(4., 3.9, 1));
    }
    #[test]
    fn readout_late_error_and_rejection_restore_both_actual_master_families() -> Result<()> {
        let d = Device::Cpu;
        let cue = Var::from_vec(vec![-0f32, 0.25], 2, &d)?;
        let prefix = Var::from_vec(vec![0f32, -0.25], 2, &d)?;
        let params = BTreeMap::from([
            ("cue.coefficients".into(), cue),
            ("prefix.coefficients".into(), prefix),
        ]);
        let parent = np::snapshot(&params)?;
        let edits = vec![
            np::Edit {
                name: "cue.coefficients".into(),
                index: 0,
                before: 0,
                after: 1,
            },
            np::Edit {
                name: "prefix.coefficients".into(),
                index: 1,
                before: -1,
                after: 0,
            },
        ];
        let late = np::attempt_restored(&params, &parent, || -> Result<()> {
            np::apply_edits(&params, &parent, &edits)?;
            Err(bad("late changedCue Prefix binding/export failure"))
        });
        assert!(late.is_err());
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        np::apply_edits(&params, &parent, &edits)?;
        assert!(!gate(4., 4.1, 2));
        np::restore(&params, &parent)?;
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        assert_eq!(
            np::snapshot(&params)?["cue.coefficients"][0].to_bits(),
            (-0f32).to_bits()
        );
        Ok(())
    }
}
