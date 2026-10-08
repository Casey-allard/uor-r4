//! Recover a recorded accepted intermediate; no gradient or new proposal path.
use super::*;
const ORIGINAL_REPORT: &str = "d1d47739b897c4d6cd7d7e01500d0ba05749dce885dd64bb3530001b7007222b";
const ORIGINAL_MANIFEST: &str = "01fa3c74d69a6e1d9ba45e82c58cf84a9d0fa203488b77a4757cdbe137f5248b";
const MARGIN_REPORT: &str = "1337b6e9f280543844cd2cb6de0c14408a9ff5416702867526eac789492829f0";
const MARGIN_MANIFEST: &str = "ad62944cb96e33ac73e9c777a42cc62720b3c3ad9c9c968127ca10faa6eac06d";
const FACTS: [usize; 5] = [97, 156, 151, 245, 392];
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub original_readout_root: PathBuf,
    pub margin_root: PathBuf,
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.readout-intermediate-candidate/1","mechanism":"recover globally earliest recorded accepted factual crossing; original accepted prefix inclusive","proposal_path":"same saved1920gradient/order/statuses; no new gradient/dose/seed/reorder/refill/latch","objective":"unchanged89term weights: five actual factual terms plus84 original successful trajectory positions","guards":377,"gate":"strict original89baselineCE descent +atleastonefact +all377winnerguards","frozen":"Source/Context/Potential/Generate/bridge/U/cue-joint/tokenizer/geometry","serving_policy_change":false})
}
fn pinned(root: &Path, r: &str, m: &str) -> Result<Value> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == r
            && sha256_file(&root.join("manifest.json"))? == m,
        "intermediate sealed authority pin differs",
    )?;
    let v = read(&root.join("report.json"))?;
    replay_require(
        v["status"] == "COMPLETED",
        "intermediate authority incomplete",
    )?;
    Ok(v)
}
pub(super) fn authenticate_inputs(c: &Config) -> Result<()> {
    let r = pinned(&c.original_readout_root, ORIGINAL_REPORT, ORIGINAL_MANIFEST)?;
    replay_require(
        r["mode"] == "readout_coadaptation"
            && r["native_code_proposals"]["winner"].is_null()
            && r["baseline_objective"]["combined"].as_f64() == Some(5.465593983756579),
        "original rejected endpoint/baseline differs",
    )?;
    let m = pinned(&c.margin_root, MARGIN_REPORT, MARGIN_MANIFEST)?;
    replay_require(
        m["schema"] == "uor-r4.native-readout-path-attribution/1"
            && m["readout_report_sha256"] == ORIGINAL_REPORT
            && m["readout_manifest_sha256"] == ORIGINAL_MANIFEST,
        "margin original path binding differs",
    )?;
    Ok(())
}
fn number(v: &Value) -> Result<usize> {
    v.as_u64()
        .ok_or_else(|| bad("saved index absent"))?
        .try_into()
        .map_err(Into::into)
}
fn decode<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T> {
    Ok(serde_json::from_value(v.clone())?)
}
fn state(v: &Value) -> Result<Vec<H4Code>> {
    let codes: Vec<u8> = decode(v)?;
    replay_require(codes.len() == 8, "saved finite state lanes")?;
    codes
        .into_iter()
        .map(|c| H4Code::try_from(c).map_err(Into::into))
        .collect()
}
fn earliest_crossing(frames: &[(usize, Value)], trials: &[Value]) -> Result<(usize, usize)> {
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, frame) in frames {
        replay_require(
            FACTS.contains(index)
                && seen.insert(*index)
                && frame["input_index"] == *index
                && frame["position"] == 3,
            "crossing factual frame cohort",
        )?;
        let rows = frame["trials"]
            .as_array()
            .ok_or_else(|| bad("margin trials absent"))?;
        replay_require(rows.len() == trials.len(), "margin original trial count")?;
        for (order, (row, t)) in rows.iter().zip(trials).enumerate() {
            replay_require(
                row["order"] == order
                    && row["name"] == t["name"]
                    && row["index"] == t["index"]
                    && row["original_status"] == t["status"]
                    && row["before"] == t["before"]
                    && row["after"] == t["after"],
                "margin trial identity/order differs",
            )?;
            if row["accepted_crossing"] == true {
                replay_require(
                    t["status"] == "accepted"
                        && row["committed"] == true
                        && row["previous_correct"] == false
                        && row["staged_correct"] == true
                        && row["staged"]["pool"]["chosen_token_id"] == frame["term"]["target"],
                    "recorded accepted crossing authority",
                )?;
                found.push((order, *index));
            }
        }
    }
    replay_require(seen.len() == 5, "allfive crossing frames required")?;
    found
        .into_iter()
        .min()
        .ok_or_else(|| bad("no accepted factual crossing recorded"))
}
fn prefix_edits(trials: &[Value], through: usize, original: &np::Shadows) -> Result<Vec<np::Edit>> {
    replay_require(
        trials.len() == 1920 && through < trials.len(),
        "accepted prefix census",
    )?;
    let mut seen = BTreeSet::new();
    let mut edits = Vec::new();
    for (order, t) in trials.iter().enumerate() {
        let name = t["name"]
            .as_str()
            .ok_or_else(|| bad("saved family absent"))?;
        let i = number(&t["index"])?;
        let masters = original
            .get(name)
            .ok_or_else(|| bad("saved unexpected family"))?;
        replay_require(
            i < 960 && masters.len() == 960 && seen.insert((name.to_owned(), i)),
            "saved coordinate duplicate/shape",
        )?;
        let before = t["before"]
            .as_i64()
            .ok_or_else(|| bad("saved before absent"))?;
        let after = t["after"]
            .as_i64()
            .ok_or_else(|| bad("saved after absent"))?;
        replay_require(
            before as f32 * 0.25 == masters[i]
                && (-7..=7).contains(&after)
                && (after - before).abs() <= 1,
            "saved original quarter/adjacent code",
        )?;
        let status = t["status"]
            .as_str()
            .ok_or_else(|| bad("saved status absent"))?;
        replay_require(
            ["accepted", "rejected", "saturated", "zero_gradient"].contains(&status),
            "saved proposal status",
        )?;
        if status == "accepted" && order <= through {
            replay_require(after != before, "accepted noop")?;
            edits.push(np::Edit {
                name: name.to_owned(),
                index: i,
                before: before as i8,
                after: after as i8,
            });
        }
    }
    Ok(edits)
}
struct Numerical {
    base_copy: Vec<i64>,
    base_generate: Vec<i64>,
    generate: Vec<i64>,
    copy: Vec<i64>,
    masses: Vec<u64>,
    pool: uor_r4_integer::geometric_vocabulary_actions::VocabularyReduction,
    donor: usize,
    post: Vec<u8>,
}
fn score_saved(
    row: &Value,
    old: &np::Shadows,
    new: &np::Shadows,
    model: &uor_r4_core::native_geometric::learner::geometric_generate::NativeGeometricGenerate,
    bridge: &NativeGeometricReadStateBridge,
    field: &NativeContinuationField,
    reducer: &mut NativeVocabularyActions,
) -> Result<Numerical> {
    let ids: Vec<u32> = decode(&row["copy_ids"])?;
    replay_require(
        !ids.is_empty() && ids.iter().all(|t| *t < 4096),
        "saved physical alias domain",
    )?;
    let mut u = vec![0i64; 4096];
    field.score_delta_into(&state(&row["continuation"]["state_codes"])? ,model,&mut u,&mut uor_r4_core::native_geometric::learner::geometric_continuation_field::ContinuationReadCounts::default())?;
    if let Some(saved) = row.get("frozen_u_q24") {
        let saved: Vec<i64> = decode(saved)?;
        replay_require(
            saved == u,
            "guard packedU versus authenticated canonical state differs",
        )?;
    }
    let original_copy: Vec<i64> = decode(&row["copy_q24"])?;
    replay_require(original_copy.len() == ids.len(), "saved alias scores shape")?;
    let mut copy = original_copy
        .iter()
        .zip(&ids)
        .map(|(c, t)| {
            c.checked_sub(u[*t as usize])
                .ok_or_else(|| bad("saved BASECopy overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let bank = &row["bank_trace"]["cue_bank"]["bank"];
    let candidates = bank["candidates"]
        .as_array()
        .ok_or_else(|| bad("saved physical candidates absent"))?;
    replay_require(candidates.len() == copy.len(), "saved candidate shape")?;
    for (j, c) in candidates.iter().enumerate() {
        replay_require(
            c["occurrence"]["token_id"] == ids[j],
            "saved occurrence alias identity",
        )?;
        let heads = bank["heads"]
            .as_array()
            .ok_or_else(|| bad("saved final heads absent"))?;
        let total = heads.iter().try_fold(0i64, |s, h| {
            s.checked_add(
                h["scores_q24"][j]
                    .as_i64()
                    .ok_or_else(|| bad("saved final head score absent"))?,
            )
            .ok_or_else(|| bad("saved head sum overflow"))
        })?;
        replay_require(total == copy[j], "saved BASECopy final head parity")?;
        for (family, path) in [
            (
                "cue.coefficients",
                &row["bank_trace"]["cue_bank"]["carrier"],
            ),
            ("prefix.coefficients", &row["bank_trace"]["prefix"]),
        ] {
            for lane in 0..8 {
                let k = &path["angular_indices"][lane][j];
                if family == "cue.coefficients" && k.is_null() {
                    continue;
                }
                let bin = number(k)?;
                replay_require(bin < 120, "saved angular bin domain")?;
                let i = lane * 120 + bin;
                let delta = (new[family][i] - old[family][i]) * 4.;
                replay_require(
                    delta.is_finite() && delta == delta.round(),
                    "saved coefficient drift quarter grid",
                )?;
                copy[j] = copy[j]
                    .checked_add(delta as i64 * (1 << 22))
                    .ok_or_else(|| bad("saved drift overflow"))?;
            }
        }
    }
    let donor = readout_constraints::earliest_base_donor(&copy)?;
    let contexts = bank["context"]["states"]
        .as_array()
        .ok_or_else(|| bad("saved context states absent"))?;
    let query = state(
        contexts
            .last()
            .ok_or_else(|| bad("saved prebridge state absent"))?,
    )?;
    let source = state(
        contexts
            .get(number(&candidates[donor]["context_position"])?)
            .ok_or_else(|| bad("saved donor cumulative state absent"))?,
    )?;
    let mut post = vec![H4Code::IDENTITY; 8];
    let mut actions = post.clone();
    let mut scores = vec![0; 960];
    bridge.apply_into(
        &query,
        &source,
        &mut post,
        &mut actions,
        &mut scores,
        &mut BridgeReadCounts::default(),
    )?;
    let mut generate = vec![0; 4096];
    model.score_into(&post, &mut generate, &mut GenerateReadCounts::default())?;
    let r = readout_constraints::reduce_pool(reducer, &generate, &copy, &ids, &u)?;
    Ok(Numerical {
        base_copy: copy,
        base_generate: generate,
        generate: r.generate_q24,
        copy: r.copy_q24,
        masses: r.token_masses,
        pool: r.summary,
        donor,
        post: post.iter().map(|c| c.index()).collect(),
    })
}
fn result_row(term: &Value, n: &Numerical) -> Result<Value> {
    let target = number(&term["target"])?;
    let mass = *n
        .masses
        .get(target)
        .ok_or_else(|| bad("saved target domain"))?;
    let total = n.pool.total_weight_q31;
    replay_require(mass > 0 && total > 0, "saved objective support")?;
    Ok(
        json!({"term":term,"target_mass":mass,"denominator":total,"ce":-(mass as f64/total as f64).ln(),"chosen":n.pool.chosen_token_id,"pool":n.pool,"factual_state":n.post,"generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&n.generate)?),"copy_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&n.copy)?),"donor":n.donor}),
    )
}
pub(super) fn run(
    a: &Args,
    start: Instant,
    c: &Config,
    p: &ContinuationParent,
    l: &Loaded,
    cw: &CueAngularWeights,
    pw: &mut PrefixAngularWeights,
    params: &BTreeMap<String, Var>,
    original: &np::Shadows,
) -> Result<Value> {
    let previous = read(&c.original_readout_root.join("report.json"))?;
    let margin = read(&c.margin_root.join("report.json"))?;
    let cr = read(&c.original_readout_root.join("readout-construction.json"))?;
    let trials = cr["construction"]["trials"]
        .as_array()
        .ok_or_else(|| bad("original trial inventory absent"))?;
    let mut frames = Vec::new();
    for f in margin["frames"]
        .as_array()
        .ok_or_else(|| bad("margin frame inventory absent"))?
    {
        let index = number(&f["input_index"])?;
        let name = f["file"]
            .as_str()
            .ok_or_else(|| bad("margin frame path absent"))?;
        replay_require(
            name == format!("frame-{index:04}-position-03.json"),
            "margin frame path unsafe/differs",
        )?;
        let path = c.margin_root.join(name);
        replay_require(
            f["sha256"] == sha256_file(&path)?,
            "margin frame hash differs",
        )?;
        frames.push((index, read(&path)?));
    }
    let (through, crossing_index) = earliest_crossing(&frames, trials)?;
    replay_require(
        through == 147
            && crossing_index == 245
            && trials[through]["name"] == "prefix.coefficients"
            && trials[through]["index"] == 915,
        "recorded first crossing differs",
    )?;
    let edits = prefix_edits(trials, through, original)?;
    let recovered = np::edited(original, &edits)?;
    // Bind exact initial signed native tables, then bind the complete saved path endpoint.
    verify_codes(cw, pw, original)?;
    let complete = prefix_edits(trials, 1919, original)?;
    let full = np::edited(original, &complete)?;
    for (family, file) in [
        ("cue.coefficients", "cue/cue-q4.bin"),
        ("prefix.coefficients", "prefix/prefix-q4.bin"),
    ] {
        let packed = fs::read(
            c.original_readout_root
                .join("native-candidate-00/checkpoint-0000")
                .join(file),
        )?;
        replay_require(packed.len() == 480, "saved endpoint packed shape")?;
        for i in 0..960 {
            let b = packed[i / 2];
            let n = if i % 2 == 0 { b & 15 } else { b >> 4 };
            let code = if n >= 8 { n as i8 - 16 } else { n as i8 };
            replay_require(
                full[family][i] == f32::from(code) * 0.25,
                "complete saved accepted endpoint packed parity",
            )?;
        }
    }
    write(
        a,
        "intermediate-recovery.json",
        &json!({"schema":"uor-r4.readout-intermediate-recovery/1","derived_crossing":{"order":through,"input_index":crossing_index,"family":trials[through]["name"],"coordinate":trials[through]["index"]},"accepted_prefix_edits":edits,"original_masters":original,"recovered_masters":recovered,"original_trials_sha256":sha256_bytes(&serde_json::to_vec(trials)?),"construction_sha256":sha256_file(&c.original_readout_root.join("readout-construction.json"))?,"margin_report_sha256":MARGIN_REPORT,"global_first_crossing_verified":true,"complete_original_endpoint_verified":true,"policy":policy()}),
    )?;
    replay_require(
        sha256_file(&a.training_inputs)? == INPUT_SHA
            && sha256_file(&a.training_labels)? == LABEL_SHA
            && a.training_inputs == a.development_inputs
            && a.training_labels == a.development_labels,
        "intermediate frozen512panel differs",
    )?;
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
    pairs(&eps)?;
    let root = &a
        .readout_coadaptation
        .as_ref()
        .ok_or_else(|| bad("readout config absent"))?
        .retained_reached_u_root;
    let old: frontier::Plan = serde_json::from_slice(&fs::read(root.join("frontier-plan.json"))?)?;
    let (plan, guard_terms, derivation) =
        frontier::readout_endpoint_plan(root, &old, &eps, &p.tokenizer)?;
    frontier::readout_components(&plan)?;
    let original_plan = read(&c.original_readout_root.join("frontier-plan.json"))?;
    replay_require(
        serde_json::to_value(&plan)? == original_plan,
        "recovered objective plan changed",
    )?;
    let terms = plan.terms.iter().map(to_term).collect::<Vec<_>>();
    let guards = guard_terms.iter().map(to_term).collect::<Vec<_>>();
    replay_require(
        terms.len() == 89 && guards.len() == 377,
        "recovered89/377scope",
    )?;
    for name in ["frontier-plan.json", "reference-plan.json"] {
        let raw = fs::read(c.original_readout_root.join(name))?;
        fs::write(a.out.join(name), &raw)?;
        replay_require(
            fs::read(a.out.join(name))? == raw,
            "copied original plan bytes differ",
        )?;
    }
    write(a, "selected-plan-derivation.json", &derivation)?;
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    write(a, "protected-terms.json", &json!(guards))?;
    let baseline = previous["baseline_objective"].clone();
    write(a, "native-code-parent-objective.json", &baseline)?;
    let mut initial = read(&c.original_readout_root.join("original-development.json"))?;
    for (i, r) in initial["rows"]
        .as_array_mut()
        .ok_or_else(|| bad("verified parent rows absent"))?
        .iter_mut()
        .enumerate()
    {
        let saved = reference_saved_row(&c.original_readout_root, r)?;
        replay_require(
            saved["id"] == eps[i].packet.id,
            "reused parent input ID differs",
        )?;
        let name = format!("original-row-{i:04}.json");
        write(a, &name, &saved)?;
        r["row_file"] = json!(name);
        r["row_sha256"] = json!(sha256_file(
            &a.out.join(format!("original-row-{i:04}.json"))
        )?);
    }
    write(a, "original-development.json", &initial)?;
    let numerical_upper = 466u64 * 4096 * 8 * 6 + 16 * 1024 * 1024;
    replay_require(
        numerical_upper <= 256 * 1024 * 1024,
        "intermediate numerical auxiliary admission",
    )?;
    let projected = size(&a.out)?
        .checked_add(size(&root.join("checkpoint-0001"))?)
        .and_then(|n| n.checked_add(1024 * 1024 * 1024))
        .ok_or_else(|| bad("intermediate report projection overflow"))?;
    replay_require(
        projected < a.maximum_report_bytes,
        "intermediate report projection beforegate",
    )?;
    write(
        a,
        "admission.json",
        &json!({"policy":policy(),"authority":authority(c),"baseline_objective":"reused sealed original89receipt, notrescored","parent512":"reused sealed originalrowfiles, notregenerated","new_finite_gate":{"objective_terms":89,"guard_pools":377,"lane":"host finite native bridge/Generate/U/allalias scoring fromsavedstates"},"model_master_loading":"configuredCUDA restoration only; no gradient/backward/trainingstep","numerical_auxiliary_upper_bound_bytes":numerical_upper,"maximum_numerical_auxiliary_bytes":256*1024*1024,"excluded_from_cache_scope":"savedJSON/reportserialization and modelmasters charged8GiBsupervisedRAM","projected_report_bytes":projected,"maximum_report_bytes":a.maximum_report_bytes,"candidate_export_and_actual_rollout":"NOT_RUN until complete89+377 gatepasses"}),
    )?;
    let native = p.generator()?;
    let model = native.generate_model();
    let bridge = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(root.join("checkpoint-0001/continuation-field.bin"))?,
        &p.binding,
        model,
    )?;
    let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let gradient = read(
        &c.original_readout_root
            .join("coefficient-gradient-receipt.json"),
    )?;
    let source_rows = gradient["rows"]
        .as_array()
        .ok_or_else(|| bad("original89row witnesses absent"))?;
    replay_require(source_rows.len() == 89, "saved89row census")?;
    let clock = Instant::now();
    let mut losses = [0f64; 2];
    let mut outcomes = Vec::new();
    let mut full_objective = Vec::new();
    for (i, (row, term)) in source_rows.iter().zip(&terms).enumerate() {
        deadline(a, start)?;
        replay_require(
            row["term"] == serde_json::to_value(term)?
                && row["id"] == eps[term.index].packet.id
                && row["actual_prefix_ids"] == json!(term.parent_actual_prefix_ids),
            "original objective witness identity",
        )?;
        let n = score_saved(
            row,
            original,
            &recovered,
            model,
            &bridge,
            &field,
            &mut reducer,
        )?;
        let r = result_row(&row["term"], &n)?;
        losses[term.component] += term.weight
            * r["ce"]
                .as_f64()
                .ok_or_else(|| bad("recovered objective CE absent"))?;
        if i < 5 {
            let frame = &frames
                .iter()
                .find(|(idx, _)| *idx == term.index)
                .ok_or_else(|| bad("factual crossing frame absent"))?
                .1;
            let ms = frame["trials"]
                .as_array()
                .ok_or_else(|| bad("margin trial rows absent"))?;
            let mut last = &frame["baseline"];
            for trial in ms.iter().take(through + 1) {
                if trial["committed"] == true {
                    last = &trial["staged"];
                }
            }
            replay_require(
                last["pool"] == r["pool"]
                    && last["target_mass"] == r["target_mass"]
                    && last["post_state"] == r["factual_state"]
                    && last["generate_q24_sha256"] == r["generate_raw_scores_sha256"]
                    && last["copy_q24_sha256"] == r["copy_raw_scores_sha256"],
                "recovered snapshot margin fullpool parity",
            )?;
        }
        full_objective.push(json!({"term":term,"source_row":i,"source_file":"coefficient-gradient-receipt.json","id":row["id"],"base_copy_q24":n.base_copy,"base_generate_q24":n.base_generate,"generate_q24":n.generate,"copy_ids":row["copy_ids"],"copy_q24":n.copy,"token_masses":n.masses,"result":r}));
        outcomes.push(r);
    }
    let objective = json!({"task":losses[0],"reference":losses[1],"combined":losses[0]+losses[1],"terms":outcomes,"native_calls":89,"elapsed_seconds":clock.elapsed().as_secs_f64(),"scope":"finite native scoring from original saved states, same89weights"});
    write(a, "saved-candidate-objective.json", &objective)?;
    write(
        a,
        "saved-candidate-objective-pools.json",
        &json!(full_objective),
    )?;
    drop(full_objective);
    let original_guards = read(
        &c.original_readout_root
            .join("original-protected-pools.json"),
    )?;
    let rows = original_guards
        .as_array()
        .ok_or_else(|| bad("original377guardwitnesses absent"))?;
    replay_require(rows.len() == 377, "original377guard census")?;
    let mut guard_results = Vec::new();
    let mut failed = Vec::new();
    for (i, (row, t)) in rows.iter().zip(&guards).enumerate() {
        deadline(a, start)?;
        replay_require(
            row["term"] == serde_json::to_value(t)? && row["id"] == eps[t.index].packet.id,
            "original guard term identity",
        )?;
        let saved = read(&a.out.join(format!("original-row-{:04}.json", t.index)))?;
        let pos = t.position;
        replay_require(
            saved["canonical_target_ids_labels_only"][pos] == t.target
                && saved["generated_ids"].as_array().is_some_and(|ids| {
                    ids.get(..pos)
                        == json!(t.parent_actual_prefix_ids)
                            .as_array()
                            .map(|a| a.as_slice())
                }),
            "guard actual correct prefix authority",
        )?;
        let mut witness = row.clone();
        witness["continuation"] = saved["canonical"][pos]["native"]["continuation"].clone();
        let n = score_saved(
            &witness,
            original,
            &recovered,
            model,
            &bridge,
            &field,
            &mut reducer,
        )?;
        if n.pool.chosen_token_id != t.target {
            failed.push(i);
        }
        guard_results.push(json!({"term":t,"id":row["id"],"source_row":i,"source_file":"original-protected-pools.json","continuation":witness["continuation"],"base_copy_q24":n.base_copy,"base_generate_q24":n.base_generate,"generate_q24":n.generate,"copy_ids":row["copy_ids"],"copy_q24":n.copy,"token_masses":n.masses,"pool":n.pool,"donor":n.donor,"post_state":n.post,"preserved":n.pool.chosen_token_id==t.target}));
    }
    write(
        a,
        "saved-candidate-protected-pools.json",
        &json!(guard_results),
    )?;
    let correct = objective["terms"]
        .as_array()
        .ok_or_else(|| bad("recovered outcomes absent"))?
        .iter()
        .filter(|r| r["term"]["component"] == 0 && r["chosen"] == r["term"]["target"])
        .count();
    let base = baseline["combined"]
        .as_f64()
        .ok_or_else(|| bad("original baseline absent"))?;
    let loss = losses.iter().sum();
    let selected = failed.is_empty() && gate(base, loss, correct);
    let verdict = json!({"selected":selected,"original_baseline_combined":base,"candidate_combined":loss,"corrected_factual_frontiers":correct,"original_guards":377,"guard_failures":failed,"strict_ce_descent":gate(base,loss,1),"first_crossing":{"order":through,"input_index":crossing_index,"family":"prefix.coefficients","coordinate":915},"gradient_evaluations":0,"optimizer_updates":0,"new_proposals":0,"reused_parent_evaluation":true,"actual_guard_and_objective_reductions":466,"policy":policy()});
    write(a, "candidate-gate.json", &verdict)?;
    if !selected {
        return Ok(
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"readout_intermediate_candidate","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"candidate_gate":verdict,"baseline_objective":baseline,"candidate_objective":objective,"initial_evaluation":initial,"candidate_export":"NOT_RUN: complete snapshot gate failed","early8":"NOT_RUN","final_evaluation":"NOT_RUN","useful_candidate":false,"authority":authority(c),"execution_lanes":{"model_master_restoration":"configured CUDA device; no backward or training","gate":"host finite native scoring from saved states","replies":"native own-prefix evaluator if admitted"},"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"preserved finite recovered-checkpoint negative; no selected export or actual rollout"}),
        );
    }
    np::apply_edits(params, original, &edits)?;
    let (candidate, candidate_field, receipt) = export(a, 1, p, l, cw, pw)?;
    verify_codes(cw, pw, &recovered)?;
    let fresh = np::score(
        a,
        &candidate,
        &candidate_field,
        &eps,
        &plan.canonical_reference,
        &terms,
        start,
    )?;
    for (saved, reloaded) in objective["terms"]
        .as_array()
        .ok_or_else(|| bad("saved objective absent"))?
        .iter()
        .zip(
            fresh["terms"]
                .as_array()
                .ok_or_else(|| bad("export objective absent"))?,
        )
    {
        for k in [
            "term",
            "pool",
            "target_mass",
            "denominator",
            "chosen",
            "factual_state",
            "generate_raw_scores_sha256",
        ] {
            replay_require(saved[k] == reloaded[k], "exported89pool parity differs")?;
        }
    }
    replay_require(
        (fresh["combined"]
            .as_f64()
            .ok_or_else(|| bad("export CE absent"))?
            - loss)
            .abs()
            < 1e-10,
        "exported89CEparity",
    )?;
    write(a, "exported-candidate-objective.json", &fresh)?;
    let (_, _, exported) = capture(&candidate, &candidate_field, &eps, &guards, None)?;
    for (saved, reloaded) in guard_results.iter().zip(
        exported
            .as_array()
            .ok_or_else(|| bad("export377pools absent"))?,
    ) {
        for k in [
            "term",
            "id",
            "base_copy_q24",
            "base_generate_q24",
            "generate_q24",
            "copy_ids",
            "copy_q24",
            "pool",
        ] {
            replay_require(saved[k] == reloaded[k], "exported377pool parity differs")?;
        }
    }
    write(a, "exported-candidate-protected-pools.json", &exported)?;
    drop(guard_results);
    drop(exported);
    let success = frontier::successful_indices(&plan);
    let subset = success.iter().map(|i| &eps[*i]).collect::<Vec<_>>();
    let early = continuation_evaluate_rows(
        a,
        "pilot-original8",
        &candidate,
        &candidate_field,
        &subset,
        start,
    )?;
    write(
        a,
        "pilot-original8-receipt.json",
        &json!({"indices":success,"evaluation":early}),
    )?;
    let early_complete = early["complete"]
        .as_u64()
        .ok_or_else(|| bad("actual early8 completeness receipt absent"))?;
    if early_complete != 8 {
        np::restore(params, original)?;
        return Ok(
            json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"readout_intermediate_candidate","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"authority":authority(c),"candidate_gate":verdict,"baseline_objective":baseline,"candidate_objective":objective,"final_objective":fresh,"candidate_receipt":receipt,"initial_evaluation":initial,"early8":early,"reply_qualification":{"status":"FAILED","retained_original8":false,"reason":"actual original-eight pilot lost complete reply","candidate512":"NOT_RUN"},"candidate_artifact_status":"RETAINED_NEGATIVE","selected_model":false,"final_evaluation":"NOT_RUN","gradient_evaluations":0,"optimizer_updates":0,"useful_candidate":false,"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"measured actual candidate qualification negative; exported artifact and pilot retained; not setup failure"}),
        );
    }
    let evaluation = continuation_evaluate(
        a,
        "development-0001",
        &candidate,
        &candidate_field,
        &eps,
        start,
    )?;
    let final_metrics = metrics(a, &evaluation, &eps)?;
    write(a, "metrics-0001.json", &final_metrics)?;
    let rowwise = joint_row_comparison(&initial, &evaluation)?;
    write(a, "complete-row-comparison.json", &rowwise)?;
    let retained = rowwise["lost_complete_ids"]
        .as_array()
        .ok_or_else(|| bad("candidate comparison missing losses"))?
        .is_empty();
    if !retained {
        np::restore(params, original)?;
    }
    Ok(
        json!({"schema":"uor-r4.geometric-frozen-map-fit/1","status":"COMPLETED","mode":"readout_intermediate_candidate","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"authority":authority(c),"candidate_gate":verdict,"baseline_objective":baseline,"candidate_objective":objective,"final_objective":fresh,"final_receipt":receipt,"initial_evaluation":initial,"final_evaluation":evaluation,"metrics":final_metrics,"rowwise":rowwise,"early8":"COMPLETED","reply_qualification":{"status":if retained {"PASSED"}else{"FAILED"},"retained_original8":retained,"candidate512":"COMPLETED"},"candidate_artifact_status":if retained {"QUALIFIED_NATIVE_GATE_AND_RETENTION"}else{"RETAINED_NEGATIVE"},"selected_model":retained,"gradient_evaluations":0,"optimizer_updates":0,"execution_lanes":{"model_master_restoration":"configured CUDA device; no backward or training","gate":"host finite native scoring from saved states","replies":"native own-prefix evaluator"},"useful_candidate":retained&&evaluation["complete"].as_u64().is_some_and(|n|n>8),"elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"recover existing accepted learning intermediate; factual CE/frontier/completeEOS distinct; exposed authored panel only, no heldout/generalchat/energy claim"}),
    )
}
fn authority(c: &Config) -> Value {
    json!({"original_readout_root":c.original_readout_root,"original_readout_report_sha256":ORIGINAL_REPORT,"original_readout_manifest_sha256":ORIGINAL_MANIFEST,"margin_root":c.margin_root,"margin_report_sha256":MARGIN_REPORT,"margin_manifest_sha256":MARGIN_MANIFEST,"selected_reached_u_report_sha256":REPORT,"selected_reached_u_manifest_sha256":MANIFEST,"source_metadata_sha256":SOURCE,"generate_sha256":GENERATE,"continuation_sha256":FIELD})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_crossing_is_global_and_accepted_only() -> Result<()> {
        let trials=(0..3).map(|i|json!({"name":"prefix.coefficients","index":i,"before":0,"after":1,"status":if i==0{"rejected"}else{"accepted"}})).collect::<Vec<_>>();
        let mut frames = Vec::new();
        for (j, index) in FACTS.iter().enumerate() {
            let rows=trials.iter().enumerate().map(|(i,t)|json!({"order":i,"name":t["name"],"index":t["index"],"before":0,"after":1,"original_status":t["status"],"accepted_crossing":(j==1&&i==1)||(j==0&&i==2),"committed":t["status"]=="accepted","previous_correct":false,"staged_correct":true,"staged":{"pool":{"chosen_token_id":4}}})).collect::<Vec<_>>();
            frames.push((
                *index,
                json!({"input_index":index,"position":3,"term":{"target":4},"trials":rows}),
            ));
        }
        assert_eq!(earliest_crossing(&frames, &trials)?, (1, 156));
        frames[0].1["trials"][0]["accepted_crossing"] = json!(true);
        assert!(earliest_crossing(&frames, &trials).is_err());
        Ok(())
    }
    #[test]
    fn recovered_prefix_contains_both_families_and_excludes_rejected_later() -> Result<()> {
        let original = ["cue.coefficients", "prefix.coefficients"]
            .into_iter()
            .map(|n| (n.to_owned(), vec![0.; 960]))
            .collect::<np::Shadows>();
        let mut trials = Vec::new();
        for name in ["cue.coefficients", "prefix.coefficients"] {
            for i in 0..960 {
                trials.push(json!({"name":name,"index":i,"before":0,"after":1,"status":if i==1{"rejected"}else{"accepted"}}));
            }
        }
        let edits = prefix_edits(&trials, 960, &original)?;
        let recovered = np::edited(&original, &edits)?;
        assert_eq!(recovered["cue.coefficients"][0], 0.25);
        assert_eq!(recovered["cue.coefficients"][1], 0.);
        assert_eq!(recovered["prefix.coefficients"][0], 0.25);
        assert_eq!(recovered["prefix.coefficients"][2], 0.);
        trials[961]["index"] = json!(0);
        assert!(prefix_edits(&trials, 960, &original).is_err());
        Ok(())
    }
    #[test]
    fn recovered_gate_keeps_original_ce_and_requires_guard_preservation() {
        let baseline = 5.465593983756579;
        assert!(gate(baseline, baseline - 0.1, 1));
        assert!(!gate(baseline, baseline, 1));
        assert!(!gate(baseline, baseline - 0.1, 0));
        let failed_guards = [1usize];
        assert!(!(failed_guards.is_empty() && gate(baseline, baseline - 0.1, 1)));
    }
}
