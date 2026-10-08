//! One current-parent reached-frontier shared-action U construction; all upstream frozen.
use super::native_proposals as np;
use super::*;
const REPORT: &str = "9e19da31b0b35088d1d2e1f054de62769e5b3e118016342540f5fe7af12eb49a";
const MANIFEST: &str = "1bca3281c00d7bcd90554eadbd4b32b65cc996c9a58b66b6d48410aadb67e0a5";
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const GENERATE: &str = "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737";
const FIELD: &str = "a42cc8d9a9d04bdcddb9f1a513128f66361611556f0e6d768de37da0bc9d1030";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_compensation_root: PathBuf,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if a.reached_u.is_some() {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.native_code_proposals
                && a.reached_frontier_objective
                && a.reference_replay.is_some()
                && a.prototype_compensation.is_none()
                && a.retained_context_root.is_none()
                && !a.constrained_emission_learning
                && !a.constrained_context_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only,
            "reached U requires exclusive joint reached native proposal mode with frozen upstream",
        )?;
    }
    Ok(())
}
fn policy() -> Value {
    json!({"schema":"uor-r4.reached-shared-action-u/1","active":"continuation.unary960 only; frozen U-localstate/prototypes; fresh588 weighted coefficientgradient","construction":"once-only adjacent legalQ4 pass sorted gradient times actual master displacement;86 transactional fullpoolguards; no refill/Adam/clip","objective":"CURRENT504 actual reached first-errors + unchanged84 originalsuccess trajectory terms; componentmeans added lambda1","protected":"original84 + allcorrect prefixes ofcurrentlyfailed rows99and399;86 constraints; no objectiveweight added","selection":"strict freshly scored revised nativeCE descent and >=1currentfrontier correct; historical8.2168 is not baseline","frozen":"Source/Context/Potential/Generate/prototypes/bias/readmap/bridge/cue/prefix/tokenizer/geometry/labels","scope":"exposed512 construction; useful only newcompleteEOS plus original8retained; no transfer/chat/energy qualification"})
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
fn gate(before: f64, after: f64, correct: usize) -> bool {
    before.is_finite()
        && after.is_finite()
        && correct > 0
        && after < before - 1e-10 * (1. + before.abs())
}
fn settle(
    params: &BTreeMap<String, Var>,
    original: &np::Shadows,
    edits: &[np::Edit],
    retain: bool,
) -> Result<()> {
    if retain {
        np::apply_edits(params, original, edits)
    } else {
        np::restore(params, original)
    }
}
fn verify_packed_u(packed: &[u8], masters: &[f32]) -> Result<()> {
    replay_require(
        packed.len() == 480 && masters.len() == 960,
        "U-only packing shape differs",
    )?;
    for (index, &master) in masters.iter().enumerate() {
        replay_require(
            master.is_finite() && (-1.75..=1.75).contains(&master),
            "U-only expected master outside legal quarter range",
        )?;
        let nibble = (packed[index / 2] >> ((index & 1) * 4)) & 15;
        let code = if nibble >= 8 {
            nibble as i8 - 16
        } else {
            nibble as i8
        };
        replay_require(
            code != -8 && code == (master * 4.).round() as i8,
            "U-only native packing differs from expected960codes",
        )?;
    }
    Ok(())
}
fn verify_u_codes(field: &NativeContinuationField, expected: &np::Shadows) -> Result<()> {
    replay_require(
        expected.len() == 1 && field.applies_to_copy(),
        "U-only expected family/shared-action policy differs",
    )?;
    let masters = expected
        .get("continuation.unary")
        .ok_or_else(|| bad("U-only expected masters absent"))?;
    verify_packed_u(field.packed_unary(), masters)
}
fn authenticate(root: &Path) -> Result<ContinuationParent> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == REPORT
            && sha256_file(&root.join("manifest.json"))? == MANIFEST,
        "selectedcompensation authority differs",
    )?;
    let r = read(&root.join("report.json"))?;
    let cp = root.join("checkpoint-0001");
    let p = ContinuationParent::from_checkpoint(&cp)?;
    replay_require(
        r["status"] == "COMPLETED"
            && r["mode"] == "prototype_compensation"
            && r["native_code_proposals"]["winner"] == 0
            && r["final_receipt"] == p.receipt
            && p.receipt["step"] == 1
            && p.binding.metadata_sha256 == SOURCE
            && p.generate_sha256 == GENERATE
            && sha256_file(&cp.join("continuation-field.bin"))? == FIELD,
        "selectedcompensation checkpoint binding differs",
    )?;
    Ok(p)
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
fn export(
    a: &Args,
    step: usize,
    p: &ContinuationParent,
    w: &ContinuationLearningWeights,
) -> Result<(ContinuationParent, NativeContinuationField, Value)> {
    disk_floor(a)?;
    let root = a.out.join(format!("checkpoint-{step:04}"));
    fs::create_dir(&root)?;
    let input = a
        .reached_u
        .as_ref()
        .ok_or_else(|| bad("reachedUconfig absent"))?
        .retained_compensation_root
        .join("checkpoint-0001");
    authenticate(input.parent().ok_or_else(|| bad("parentroot absent"))?)?;
    replay_require(
        size(&a.out)?
            .checked_add(size(&input)?)
            .and_then(|v| v.checked_add(1 << 20))
            .is_some_and(|v| v < a.maximum_report_bytes),
        "Ucheckpoint report projection exceeds cap",
    )?;
    for e in fs::read_dir(&input)? {
        let e = e?;
        let name = e.file_name();
        if name == "receipt.json"
            || name == "continuation-source"
            || name == "continuation-field.bin"
        {
            continue;
        }
        if e.file_type()?.is_dir() {
            copy_frozen_tree(&e.path(), &root.join(name))?;
        } else if e.file_type()?.is_file() {
            fs::copy(e.path(), root.join(&name))?;
            replay_require(
                sha256_file(&e.path())? == sha256_file(&root.join(name))?,
                "frozen topfilecopy differs",
            )?;
        } else {
            return Err(bad("checkpoint nonregularfile"));
        }
    }
    let g = p.generator()?;
    let field = w.export_native(g.source_binding(), g.generate_model())?;
    let raw = field.to_bytes()?;
    fs::write(root.join("continuation-field.bin"), &raw)?;
    let inventory = save_masters(&root.join("continuation-source"), &w.parameters())?;
    let disk = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        &p.binding,
        g.generate_model(),
    )?;
    let cpu = ContinuationLearningWeights::from_native(
        &disk,
        g.generate_model(),
        p.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &root.join("continuation-source"),
        &inventory,
        &cpu.parameters(),
        &Device::Cpu,
    )?;
    replay_require(
        cpu.export_native(&p.binding, g.generate_model())?
            .to_bytes()?
            == raw
            && disk.to_bytes()? == raw
            && disk.applies_to_copy()
            && cpu.score_shift() == 22,
        "U independentnative/masterreload orpolicy differs",
    )?;
    let mut receipt = p.receipt.clone();
    receipt["step"] = json!(step);
    receipt["source_commit"] = json!(option_env!("UOR_BUILD_SOURCE_COMMIT"));
    receipt["continuation_sha256"] = json!(sha256_bytes(&raw));
    receipt["continuation_parameters"] = inventory.clone();
    receipt["reached_u"] = policy();
    receipt["active_parameter_names"] = json!(["continuation.unary"]);
    receipt["optimizer_updates"] = json!(0);
    receipt["masters_independently_reloaded"] = json!(true);
    receipt["native_independently_reloaded"] = json!(true);
    receipt["revised_plan_sha256"] = json!(sha256_file(&a.out.join("frontier-plan.json"))?);
    receipt["reference_replay"] = reference_binding(a)?;
    receipt["reference_replay"]["selected_endpoint_report_sha256"] = json!(REPORT);
    receipt["reference_replay"]["derivation"] = json!(
        "current sealed compensation trajectories; original canonical reference remains separate"
    );
    fs::write(
        root.join("continuation-source/metadata.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    let current = ContinuationParent::from_checkpoint(&root)?;
    replay_require(
        current.generate == p.generate
            && current.binding == p.binding
            && current.bridge == p.bridge
            && current.cue == p.cue
            && current.joint == p.joint
            && current.prefix == p.prefix
            && current.exp == p.exp,
        "Uexport changedfrozen upstream",
    )?;
    Ok((current, disk, receipt))
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
fn verify_guard_export(
    p: &ContinuationParent,
    f: &NativeContinuationField,
    eps: &[Episode],
    guards: &[np::Term],
    construction: Option<&u_constraints::Construction>,
) -> Result<Value> {
    let raw = f.to_bytes()?;
    let hash = sha256_bytes(&raw);
    let mut g = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &raw,
        sha256: &hash,
    })?;
    let mut banks = BTreeMap::new();
    let mut records = Vec::new();
    for (i, t) in guards.iter().enumerate() {
        if let std::collections::btree_map::Entry::Vacant(e) = banks.entry(t.index) {
            e.insert(g.admit_bank(continuation_snapshot(&eps[t.index].packet)?)?);
        }
        let step = g.step(
            &banks[&t.index],
            t.parent_actual_prefix_ids
                .as_deref()
                .ok_or_else(|| bad("guardprefix absent"))?,
        )?;
        replay_require(
            step.actions.summary.chosen_token_id == t.target,
            "exported protectedwinner changed",
        )?;
        if let Some(c) = construction {
            replay_require(
                step.generate_raw_scores_q24 == c.final_generate_scores[i]
                    && step.copy_raw_scores_q24 == c.final_copy_scores[i]
                    && step.actions.summary == c.final_pool_summaries[i],
                "constructed andexported fullpools differ",
            )?;
        }
        records.push(json!({"term":t,"id":eps[t.index].packet.id,"post_state":step.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),"continuation":continuation_witness(&step)?,"generate_q24":step.generate_raw_scores_q24,"copy_ids":step.copy_token_ids,"copy_q24":step.copy_raw_scores_q24,"pool":step.actions.summary,"token_masses":step.actions.token_masses}));
    }
    Ok(json!(records))
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    let root = &a
        .reached_u
        .as_ref()
        .ok_or_else(|| bad("reachedUconfig absent"))?
        .retained_compensation_root;
    let p = authenticate(root)?;
    let g = p.generator()?;
    let old = NativeContinuationField::from_bytes(
        &fs::read(root.join("checkpoint-0001/continuation-field.bin"))?,
        &p.binding,
        g.generate_model(),
    )?;
    replay_require(old.applies_to_copy(), "selectedzeroU isnot shared-actionv2")?;
    for lane in 0..8 {
        for r in 0..120 {
            replay_require(
                old.coefficient_unary(lane, r)? == 0,
                "selectedU notnumericzero",
            )?;
        }
    }
    let weights =
        ContinuationLearningWeights::from_native(&old, g.generate_model(), p.integer.binding(), d)?;
    let master_meta = read(&root.join("checkpoint-0001/continuation-source/metadata.json"))?;
    restore(
        &root.join("checkpoint-0001/continuation-source"),
        &master_meta["continuation_parameters"],
        &weights.parameters(),
        d,
    )?;
    replay_require(
        weights
            .export_native(&p.binding, g.generate_model())?
            .to_bytes()?
            == old.to_bytes()?,
        "retainedUmaster/export differs",
    )?;
    let params = weights.parameters();
    let original = np::snapshot(&params)?;
    let result = run_inner(a, start, d, &p, &weights, &params, &original);
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
    weights: &ContinuationLearningWeights,
    params: &BTreeMap<String, Var>,
    original: &np::Shadows,
) -> Result<Value> {
    let root = &a
        .reached_u
        .as_ref()
        .ok_or_else(|| bad("reachedUconfig absent"))?
        .retained_compensation_root;
    let legal = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?
        .legal_token_ids()
        .iter()
        .copied()
        .collect();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &p.integer,
        &p.tokenizer,
        &legal,
        512,
    )?;
    replay_require(
        sha256_file(&a.training_inputs)? == INPUT_SHA
            && sha256_file(&a.training_labels)? == LABEL_SHA
            && a.training_inputs == a.development_inputs
            && a.training_labels == a.development_labels,
        "reachedU fixedpairedpanel differs",
    )?;
    pairs(&eps)?;
    let original_plan: frontier::Plan =
        serde_json::from_slice(&fs::read(root.join("frontier-plan.json"))?)?;
    let (plan, guard_terms, plan_receipt) =
        frontier::selected_endpoint_plan(root, &original_plan, &eps, &p.tokenizer)?;
    write(a, "frontier-plan.json", &serde_json::to_value(&plan)?)?;
    write(
        a,
        "reference-plan.json",
        &serde_json::to_value(&plan.canonical_reference)?,
    )?;
    write(a, "selected-plan-derivation.json", &plan_receipt)?;
    let terms = plan.terms.iter().map(to_term).collect::<Vec<_>>();
    let guards = guard_terms.iter().map(to_term).collect::<Vec<_>>();
    replay_require(
        terms.len() == 588 && guards.len() == 86,
        "revised588/86population differs",
    )?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    write(a, "protected-terms.json", &json!(guards))?;
    let mut initial = read(&root.join("development-0001.json"))?;
    for (index, row) in initial["rows"]
        .as_array_mut()
        .ok_or_else(|| bad("selectedoriginalrows absent"))?
        .iter_mut()
        .enumerate()
    {
        let saved = reference_saved_row(root, row)?;
        let name = format!("original-row-{index:04}.json");
        write(a, &name, &saved)?;
        row["row_file"] = json!(name);
        row["row_sha256"] = json!(sha256_file(
            &a.out.join(format!("original-row-{index:04}.json"))
        )?);
    }
    write(a, "original-development.json", &initial)?;
    write(
        a,
        "admission.json",
        &json!({"policy":policy(),"selected_input_report_sha256":REPORT,"selected_input_manifest_sha256":MANIFEST,"revised_plan_sha256":sha256_file(&a.out.join("frontier-plan.json"))?,"objective_positions":588,"protected_pools":86,"shared_coefficients":960,"frozen_factor_keys_bytes":86*4096*8*2,"guard_raw_generate_bytes":86*4096*8,"maximum_auxiliary_cache_bytes":128*1024*1024,"cache_policy":"positions prepared one at a time;86guard factor/raw buffers retained; fullreducer staging during one960construction","gradient":"exact coefficient-only API; currenttarget supplied after complete alias pool"}),
    )?;
    let (before, zero, initial_receipt) = export(a, 0, p, weights)?;
    replay_require(
        sha256_bytes(&zero.to_bytes()?) == FIELD,
        "zeroUmetadata changedunnecessarily",
    )?;
    let baseline = np::score(
        a,
        &before,
        &zero,
        &eps,
        &plan.canonical_reference,
        &terms,
        start,
    )?;
    write(a, "native-code-parent-objective.json", &baseline)?;
    let original_pools = verify_guard_export(&before, &zero, &eps, &guards, None)?;
    write(a, "original-protected-pools.json", &original_pools)?;
    drop(original_pools);
    let owner = PreparedFixedContinuationBank::new(
        before
            .generator()?
            .with_continuation_field(BoundNativeBytes {
                bytes: &zero.to_bytes()?,
                sha256: FIELD,
            })?,
        weights,
        &p.exp,
    )?;
    let prepared = owner.prepare_field(weights)?;
    let mut banks = BTreeMap::new();
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut gradient_rows = Vec::new();
    let mut combined = 0f64;
    let gradient_clock = Instant::now();
    for term in &terms {
        deadline(a, start)?;
        if let std::collections::btree_map::Entry::Vacant(e) = banks.entry(term.index) {
            e.insert(owner.admit_bank(continuation_snapshot(&eps[term.index].packet)?)?);
        }
        let position = owner.prepare_position(
            banks[&term.index].clone(),
            term.parent_actual_prefix_ids
                .as_deref()
                .ok_or_else(|| bad("gradientactualprefix absent"))?,
        )?;
        let out = position.forward_coefficients_only(weights, &prepared)?;
        let loss = (out.loss_with_credit(term.target, a.credit.policy())? * term.weight)?;
        combined += loss.to_scalar::<f32>()? as f64;
        let grads = loss.backward()?;
        for (name, var) in params {
            if let Some(g) = grads.get(var.as_tensor()) {
                let next = if let Some(old) = sums.remove(name) {
                    (&old + g)?
                } else {
                    g.clone()
                };
                sums.insert(name.clone(), next.detach());
            }
        }
        gradient_rows.push(json!({"term":term,"local_state":position.local_state().iter().map(|x|x.index()).collect::<Vec<_>>(),"post_state":position.post_state().iter().map(|x|x.index()).collect::<Vec<_>>(),"generate_q24":out.generate_scores_q24,"copy_ids":position.copy_token_ids(),"copy_q24":out.copy_scores_q24,"zero_field_sha256":FIELD,"generate_sha256":sha256_bytes(&serde_json::to_vec(&out.generate_scores_q24)?),"copy_sha256":sha256_bytes(&serde_json::to_vec(&out.copy_scores_q24)?),"pool":out.actions.summary,"physical_candidates":position.physical_candidates(),"factual_bank_trace_sha256":position.factual_bank_trace_sha256(),"counts":out.continuation.costs}));
    }
    d.synchronize()?;
    let gradient_seconds = gradient_clock.elapsed().as_secs_f64();
    let base = baseline["combined"]
        .as_f64()
        .ok_or_else(|| bad("baselineCEabsent"))?;
    replay_require(
        combined.is_finite() && (combined - base).abs() < 1e-5,
        "freshUgradient objective differsfromrevisednativebaseline",
    )?;
    write(
        a,
        "coefficient-gradient-receipt.json",
        &json!({"positions":588,"combined":combined,"native_baseline":base,"elapsed_seconds":gradient_seconds,"active_names":params.keys().collect::<Vec<_>>(),"state_credit":false,"prototype_credit":false,"upstream_credit":false,"rows":gradient_rows}),
    )?;
    drop(gradient_rows);
    let inventory = np::save_ranking_gradients(a, params, &sums)?;
    let gradients = sums
        .iter()
        .map(|(n, t)| {
            Ok((
                n.clone(),
                t.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?,
            ))
        })
        .collect::<Result<np::Shadows>>()?;
    let mut pools = Vec::new();
    let native = before.generator()?;
    let model = native.generate_model();
    for t in &guards {
        if let std::collections::btree_map::Entry::Vacant(e) = banks.entry(t.index) {
            e.insert(owner.admit_bank(continuation_snapshot(&eps[t.index].packet)?)?);
        }
        let position = owner.prepare_position(
            banks[&t.index].clone(),
            t.parent_actual_prefix_ids
                .as_deref()
                .ok_or_else(|| bad("guardactualprefix absent"))?,
        )?;
        let out = position.forward_coefficients_only(weights, &prepared)?;
        let mut keys = Vec::with_capacity(4096 * 8);
        for token in 0..4096 {
            for lane in 0..8 {
                let relative = model.algebra().compose(
                    model
                        .algebra()
                        .inverse(position.local_state()[lane].index())?,
                    model.prototypes()[token * 8 + lane],
                )?;
                keys.push((lane * 120 + usize::from(relative)) as u16);
            }
        }
        pools.push(u_constraints::UProtectedPool {
            required_token: t.target,
            generate_q24: out.generate_scores_q24,
            copy_ids: position.copy_token_ids().to_vec(),
            copy_q24: out.copy_scores_q24,
            factor_keys: keys,
        });
    }
    let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let clock = Instant::now();
    let construction = u_constraints::construct(original, &gradients, pools, &mut reducer)?;
    let construction_seconds = clock.elapsed().as_secs_f64();
    write(
        a,
        "u-construction.json",
        &json!({"construction":construction,"construction_seconds":construction_seconds,"gradient_inventory":inventory,"policy":policy()}),
    )?;
    let mut child = a.clone();
    child.out = a.out.join("native-candidate-00");
    report_output::claim(&child.out)?;
    copy_child_plans(a, &child)?;
    let candidate = np::attempt_restored(params, original, || -> Result<(Value, Value)> {
        np::apply_edits(params, original, &construction.edits)?;
        let (candidate, field, receipt) = export(&child, 0, p, weights)?;
        verify_u_codes(&field, &np::edited(original, &construction.edits)?)?;
        let full = verify_guard_export(&candidate, &field, &eps, &guards, Some(&construction))?;
        write(&child, "reloaded-protected-pools.json", &full)?;
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
            json!({"status":"FAILED","error":e.to_string(),"scope":"executionfailure;nomodelnegative"})
        }
    };
    write(&child, "report.json", &child_report)?;
    report_output::seal(&child.out)?;
    report_output::verify(&child.out)?;
    let (candidate_objective, candidate_receipt) = candidate?;
    let corrected = candidate_objective["terms"]
        .as_array()
        .ok_or_else(|| bad("candidateobjectiverows absent"))?
        .iter()
        .filter(|r| {
            r["term"]["component"] == 0 && r["pool"]["chosen_token_id"] == r["term"]["target"]
        })
        .count();
    let loss = candidate_objective["combined"]
        .as_f64()
        .ok_or_else(|| bad("candidateCE absent"))?;
    let selected = gate(base, loss, corrected);
    settle(params, original, &construction.edits, selected)?;
    let (final_parent, final_field, final_receipt) = export(a, 1, p, weights)?;
    verify_u_codes(
        &final_field,
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
    write(a, "native-code-final-objective.json", &final_objective)?;
    replay_require(
        final_objective["combined"]
            == if selected {
                candidate_objective["combined"].clone()
            } else {
                baseline["combined"].clone()
            },
        "finalselectionobjective differs",
    )?;
    let final_pools = verify_guard_export(
        &final_parent,
        &final_field,
        &eps,
        &guards,
        if selected { Some(&construction) } else { None },
    )?;
    write(a, "final-protected-pools.json", &final_pools)?;
    drop(final_pools);
    let success = frontier::successful_indices(&plan);
    let selected_eps = success.iter().map(|i| &eps[*i]).collect::<Vec<_>>();
    let early = continuation_evaluate_rows(
        a,
        "pilot-original8",
        &final_parent,
        &final_field,
        &selected_eps,
        start,
    )?;
    write(
        a,
        "pilot-original8-receipt.json",
        &json!({"evaluation":early,"indices":success,"checkpoint_receipt_sha256":sha256_file(&a.out.join("checkpoint-0001/receipt.json"))?}),
    )?;
    replay_require(early["complete"] == 8, "earlyoriginal8 lostcompleteanswers")?;
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
        "full512 lostoriginalcompleteanswers",
    )?;
    let prior = read(&root.join("report.json"))?;
    let prior_comparison = joint_row_comparison(&prior["initial_evaluation"], &final_evaluation)?;
    write(a, "precompensation-row-comparison.json", &prior_comparison)?;
    let proposals = json!({"status":if selected{"REVISED_NATIVE_DESCENT_AND_FRONTIER_GAIN_SELECTED"}else{"ORIGINAL_ZERO_U_RETAINED"},"winner":if selected{json!(0)}else{Value::Null},"parent_combined":base,"candidate_combined":loss,"corrected_current_frontiers":corrected,"policy":policy(),"optimizer_updates":0});
    write(a, "native-code-proposals.json", &proposals)?;
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"reached_u","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"immediate_input":{"root":root,"report_sha256":REPORT,"manifest_sha256":MANIFEST,"generate_sha256":GENERATE,"continuation_sha256":FIELD},"initial_receipt":initial_receipt,"candidate_receipt":candidate_receipt,"baseline_objective":baseline,"candidate_objective":candidate_objective,"final_objective":final_objective,"selected_plan_derivation":plan_receipt,"final_receipt":final_receipt,"initial_evaluation":initial,"final_evaluation":final_evaluation,"metrics":final_metrics,"rowwise":rowwise,"native_code_proposals":proposals,"gradient_seconds":gradient_seconds,"construction_seconds":construction_seconds,"optimizer_updates":0,"useful_candidate":selected && final_evaluation["complete"].as_u64().is_some_and(|x|x>8),"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"newreachedfrontier U-only shared-action construction; nativeCE/frontiercorrectness distinctfromownprefixEOS; noheldout/chat/energyqualification"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revised_gate_requires_both_descent_and_frontier_correction() {
        assert!(!gate(9., 8., 0));
        assert!(!gate(9., 9., 1));
        assert!(gate(9., 8.9, 1));
        assert!(!gate(f64::NAN, 8., 1));
    }
    #[test]
    fn rejected_selection_and_late_error_restore_actual_u_master_bits() -> Result<()> {
        let d = Device::Cpu;
        let var = Var::from_vec(vec![-0.0f32, 0.249f32], (1, 2), &d)?;
        let params = BTreeMap::from([("continuation.unary".into(), var.clone())]);
        let parent = np::snapshot(&params)?;
        let edits = vec![np::Edit {
            name: "continuation.unary".into(),
            index: 0,
            before: 0,
            after: 1,
        }];
        let outcome = np::attempt_restored(&params, &parent, || -> Result<()> {
            np::apply_edits(&params, &parent, &edits)?;
            Err(bad("late export error"))
        });
        assert!(outcome.is_err());
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        np::apply_edits(&params, &parent, &edits)?;
        settle(&params, &parent, &edits, gate(9., 8., 0))?;
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        assert_eq!(
            var.flatten_all()?.to_vec1::<f32>()?[0].to_bits(),
            (-0.0f32).to_bits()
        );
        Ok(())
    }
    #[test]
    fn u_only_packing_checks_all960_signed_codes_without_upstream_vars() -> Result<()> {
        let mut values = vec![0f32; 960];
        values[0] = -1.75;
        values[1] = 1.75;
        values[959] = -0.25;
        let mut packed = vec![0u8; 480];
        packed[0] = 0x79;
        packed[479] = 0xf0;
        verify_packed_u(&packed, &values)?;
        packed[479] = 0x70;
        assert!(verify_packed_u(&packed, &values).is_err());
        packed[479] = 0xf0;
        packed[20] = 8;
        assert!(verify_packed_u(&packed, &values).is_err());
        Ok(())
    }
}
