//! Two fixed-position Context credit arms, followed by one native displacement.
//! This probe performs no optimizer update or autoregressive rollout.
use super::native_proposals as np;
use super::*;
use uor_r4_integer::geometric_context_q4::{unpack_coefficients, ContextQ4Config};

const REPORT: &str = "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee";
const SEAL: &str = "de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5";
const CAPTURE_REPORT: &str = "d65698bec3d57aa2415841b5507205d5f23cdab401a826c8e24978e9990a0143";
const CAPTURE_SEAL: &str = "0fd94f36f05051a024f7c1765eb16b357bbc752b8f1a77c3fe78577b921c2012";
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const GENERATE: &str = "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737";
const FIELD: &str = "82ae9daeb402b288e64492d5b299110b36849907019c952609a1cae6612673ee";
const COHORT: [usize; 9] = [245, 0, 1, 4, 5, 8, 9, 12, 13];

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_intermediate_root: PathBuf,
    pub retained_capture_root: PathBuf,
    #[serde(default)]
    pub recorded_finite_contrast: Option<RecordedFiniteContrastConfig>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecordedFiniteContrastConfig {
    pub retained_path_root: PathBuf,
    pub retained_probe_root: PathBuf,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.prefix_context_credit {
        replay_require(a.mode == Mode::JointContinuation && a.updates == 1
            && a.loss_scope == LossScope::All && a.readout_coadaptation.is_none()
            && a.reached_u.is_none() && a.prototype_compensation.is_none()
            && a.retained_context_root.is_none() && !a.constrained_context_learning
            && !a.constrained_emission_learning && !a.categorical_action_learning
            && !a.categorical_action_only && !a.native_code_proposals
            && !a.reached_frontier_objective && a.reference_replay.is_none(),
            "Prefix Context probe requires exclusive joint mode without historical proposal/reference paths")?;
        if c.recorded_finite_contrast.is_some() {
            replay_require(
                a.context_path_credit.is_none() && a.maximum_report_bytes <= 256 * 1024 * 1024,
                "Recorded finite Context resource/mode admission differs",
            )?;
        }
        replay_require(
            fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(c.retained_intermediate_root.join("checkpoint-0001"))?,
            "Prefix Context checkpoint must be recovered endpoint checkpoint0001",
        )?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.prefix-context-credit/1","arms":[false,true],"prefix_temporal_utility_only_toggle":true,
        "categorical_bridge_credit":true,"read_selector_credit":true,"continuation_context_credit":true,
        "objective":"task245position4 weight1 +other17 correct actual frames each1/17; combined taskCE+referenceCE",
        "proposal":"one most-negative ON-gradient actual-master displacement/name/index; adjacent rounded native Q4 code in six Context bases",
        "optimizer_updates":0,"autoregressive_rollouts":0,"positive":"strict combined18 CE descent +245pos4 target probability improvement +all17 correct-frame winners retained",
        "scope":"18 fixed actual-prefix probe only; no all-prefix/full-reply/useful-model qualification; unchanged single adjacent move is a limited negative, not a seam or global capacity failure"})
}
fn sealed(root: &Path, report: &str, seal: &str) -> Result<Value> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == report
            && sha256_file(&root.join("manifest.json"))? == seal,
        "Prefix Context sealed authority differs",
    )?;
    let r = read(&root.join("report.json"))?;
    replay_require(
        r["status"] == "COMPLETED",
        "Prefix Context authority incomplete",
    )?;
    Ok(r)
}
fn decode<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T> {
    Ok(serde_json::from_value(v.clone())?)
}
fn index(v: &Value) -> Result<usize> {
    Ok(v.as_u64()
        .ok_or_else(|| bad("Prefix Context index missing"))?
        .try_into()?)
}
#[derive(Clone)]
struct Frame {
    input: usize,
    position: usize,
    id: String,
    prefix: Vec<u32>,
    target: u32,
    weight: f64,
    saved: Value,
}
fn cohort(pairs: &[(usize, usize)]) -> Result<()> {
    let expected = COHORT
        .into_iter()
        .flat_map(|i| [(i, 3), (i, 4)])
        .collect::<Vec<_>>();
    replay_require(
        pairs == expected,
        "Prefix Context requires exact ordered18 frame cohort",
    )
}
fn snapshot(step: &NativeBankGenerateStep) -> Result<Value> {
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("Prefix Context U witness absent"))?;
    let bridge = step
        .bridge
        .as_ref()
        .ok_or_else(|| bad("Prefix Context bridge absent"))?;
    Ok(
        json!({"generate_q24":step.generate_raw_scores_q24,"copy_ids":step.copy_token_ids,"copy_q24":step.copy_raw_scores_q24,
        "post_state":step.post_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"pool":step.actions,
        "bridge":{"selected_ordinal":bridge.selected_ordinal,"selected_candidate":bridge.selected_candidate,"query_state":bridge.query_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"source_state":bridge.source_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_codes":bridge.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_scores_q24":bridge.action_scores_q24,"counts":bridge.counts},"continuation":{"query_tokens":u.query_tokens,"actual_prefix_tokens":u.actual_prefix_tokens,"state_codes":u.state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"delta_scores_q24":u.delta_scores_q24,"encoding_coefficient_reads":u.encoding_coefficient_reads,"counts":u.counts},
        "bank_trace":step.bank_trace}),
    )
}
fn parity(v: &Value, saved: &Value) -> Result<()> {
    for key in ["generate_q24", "copy_ids", "copy_q24", "post_state", "pool"] {
        replay_require(
            v[key] == saved[key],
            &format!("Prefix Context saved {key} parity differs"),
        )?;
    }
    for key in [
        "selected_ordinal",
        "selected_candidate",
        "query_state",
        "source_state",
        "action_codes",
        "action_scores_q24",
        "counts",
    ] {
        replay_require(
            v["bridge"][key] == saved["bridge"][key],
            &format!("Prefix Context bridge {key} differs"),
        )?;
    }
    replay_require(
        v["continuation"] == saved["continuation"] && v["bank_trace"] == saved["bank_trace"],
        "Prefix Context complete U/bank trace parity differs",
    )
}
fn masses(v: &Value, target: u32) -> Result<(u64, u64)> {
    let rows = v["pool"]["token_masses"]
        .as_array()
        .ok_or_else(|| bad("Prefix Context token masses absent"))?;
    let row = rows
        .iter()
        .find(|r| r["token_id"] == target)
        .ok_or_else(|| bad("Prefix Context target mass absent"))?;
    Ok((
        row["weight_q31"]
            .as_u64()
            .ok_or_else(|| bad("Prefix Context target mass invalid"))?,
        v["pool"]["summary"]["total_weight_q31"]
            .as_u64()
            .ok_or_else(|| bad("Prefix Context denominator absent"))?,
    ))
}
fn evaluate(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    frames: &[Frame],
    name: &str,
    saved_parity: bool,
) -> Result<Value> {
    evaluate_inner(a, start, p, field, eps, frames, name, saved_parity, None)
}
fn evaluate_inner(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    frames: &[Frame],
    name: &str,
    saved_parity: bool,
    capture: Option<&constrained_context::CapturedDecisions>,
) -> Result<Value> {
    let mut events = Vec::new();
    let mut calls = Vec::new();
    let mut differences = Vec::new();
    let bytes = field.to_bytes()?;
    let hash = sha256_bytes(&bytes);
    let mut native = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &hash,
    })?;
    let mut rows = Vec::new();
    let mut task = 0.;
    let mut reference = 0.;
    let mut retained = 0;
    for f in frames {
        deadline(a, start)?;
        let tracing = capture.is_some() && f.input == 245;
        let guard = if tracing {
            Some(native.capture_context_decisions(20_000 - events.len())?)
        } else {
            None
        };
        let bank = native.admit_bank(continuation_snapshot(&eps[f.input].packet)?)?;
        if let Some(guard) = guard {
            let first = events.len();
            events.extend(guard.finish()?);
            replay_require(
                events.len() == first,
                "Finite candidate bank admission Context coverage differs",
            )?;
            calls.push(json!({"kind":"bank_admission","index":245,"position":f.position,"first":first,"count":events.len()-first}));
        }
        let guard = if tracing {
            Some(native.capture_context_decisions(20_000 - events.len())?)
        } else {
            None
        };
        let step = native.step(&bank, &f.prefix)?;
        if let Some(guard) = guard {
            let actual = guard.finish()?;
            let baseline = capture.ok_or_else(|| bad("Finite baseline capture absent"))?;
            let matches = baseline
                .calls
                .iter()
                .filter(|c| {
                    c["kind"] == "native_step" && c["index"] == 245 && c["position"] == f.position
                })
                .collect::<Vec<_>>();
            replay_require(
                matches.len() == 1 && matches[0]["prefix"] == json!(f.prefix),
                "Finite baseline actual capture scope differs",
            )?;
            let first = index(&matches[0]["first"])?;
            let count = index(&matches[0]["count"])?;
            let old = baseline
                .events
                .get(first..first + count)
                .ok_or_else(|| bad("Finite baseline capture bounds"))?;
            differences.push(capture_difference(f.position, old, &actual));
            let first = events.len();
            events.extend(actual);
            calls.push(json!({"kind":"native_step","index":245,"position":f.position,"prefix":f.prefix,"first":first,"count":events.len()-first}));
        }
        let v = snapshot(&step)?;
        if saved_parity {
            parity(&v, &f.saved)?;
        }
        let (mass, total) = masses(&v, f.target)?;
        replay_require(
            mass > 0 && total >= mass,
            "Prefix Context native target mass/denominator invalid",
        )?;
        let ce = -(mass as f64 / total as f64).ln();
        let correct = step.actions.summary.chosen_token_id == f.target;
        if f.input == 245 && f.position == 4 {
            task += f.weight * ce;
        } else {
            reference += f.weight * ce;
            retained += usize::from(correct);
        }
        write(
            a,
            &format!("{name}-row-{:04}-position-{:02}.json", f.input, f.position),
            &json!({"input_index":f.input,"position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,"target_label_only":f.target,"weight":f.weight,"native":v}),
        )?;
        rows.push(json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,"weight":f.weight,"chosen":step.actions.summary.chosen_token_id,"target_mass":mass,"total_mass":total,"ce":ce,"pool":step.actions.summary}));
    }
    let mut out = json!({"task":task,"reference":reference,"combined":task+reference,"correct_reference_frames":retained,"terms":rows,"scope":"fixed18 actual prefixes; no rollout"});
    if capture.is_some() {
        let file = "candidate-context-decisions.json";
        let path = a.out.join(file);
        let remaining = a
            .maximum_report_bytes
            .checked_sub(size(&a.out)? + 65536)
            .ok_or_else(|| bad("Finite Context output cap"))?;
        let mut writer = constrained_context::BudgetWriter {
            inner: io::BufWriter::new(
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)?,
            ),
            remaining,
        };
        serde_json::to_writer(
            &mut writer,
            &constrained_context::CapturedDecisions { calls, events },
        )?;
        io::Write::flush(&mut writer)?;
        out["actual_context_capture"] =
            json!({"file":file,"sha256":sha256_file(&path)?,"bytes":fs::metadata(path)?.len()});
        out["actual_context_divergence"] = json!(differences);
    }
    write(a, &format!("{name}.json"), &out)?;
    Ok(out)
}
fn raw(a: &Args, relative: &str, values: &[f32]) -> Result<Value> {
    replay_require(
        values.iter().all(|v| v.is_finite()),
        "Prefix Context raw nonfinite values",
    )?;
    let bytes = values
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect::<Vec<_>>();
    replay_require(
        size(&a.out)? + bytes.len() as u64 + 1_048_576 < a.maximum_report_bytes,
        "Prefix Context raw gradient storage cap",
    )?;
    let path = a.out.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, &bytes)?;
    Ok(
        json!({"file":relative,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),"elements":values.len()}),
    )
}
struct Arm {
    values: np::Shadows,
    inventory: Value,
    missing: BTreeSet<String>,
    combined: f64,
}
fn arm(
    a: &Args,
    start: Instant,
    d: &Device,
    l: &Loaded,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    frames: &[Frame],
    params: &BTreeMap<String, Var>,
    enabled: bool,
    backward: bool,
) -> Result<Arm> {
    let label = if enabled { "on" } else { "off" };
    let clock = Instant::now();
    let prepared = l.source.prepare_context_potential_on_device(&l.frozen, d)?;
    let cue = l.frozen.compile_cue_carrier(l.cue.clone())?;
    let prefix = l
        .frozen
        .compile_prefix_transport(&cue, prefix_clone(&l.prefix)?)?;
    let gs = l.generate.prepare_native()?;
    let cat = l.prepared_categorical(d)?;
    let u = ContinuationLearningWeights::from_native(field, &gs.native, l.integer.binding(), d)?;
    restore(
        &a.checkpoint.join("continuation-source"),
        &p.receipt["continuation_parameters"],
        &u.parameters(),
        d,
    )?;
    let us = u.prepare_native(&p.binding, &gs.native)?;
    replay_require(
        us.native.to_bytes()? == field.to_bytes()?,
        "Prefix Context gradient U native differs",
    )?;
    let mut learner = PreparedBankGenerate::new(&prepared, &l.generate, &gs, &l.exp)?
        .with_categorical_read_state_bridge(&cat)?
        .with_read_selector_credit(true)
        .with_prefix_temporal_utility(enabled)
        .with_continuation_field(&u, &us)?
        .with_continuation_context_credit(true)?;
    let mut sums = BTreeMap::<String, Tensor>::new();
    let mut missing = BTreeSet::new();
    let mut term_receipts = Vec::new();
    let mut combined = 0.;
    for f in frames {
        deadline(a, start)?;
        let e = &eps[f.input];
        let out = learner.forward_bank(
            &e.segments()?,
            &e.packet.query_ids,
            &f.prefix,
            &cue,
            &prefix,
        )?;
        replay_require(
            json!(out.generate.scores_q24) == f.saved["generate_q24"]
                && json!(out.copy_token_ids) == f.saved["copy_ids"]
                && json!(out.copy_scores_q24) == f.saved["copy_q24"]
                && json!(out
                    .final_state_codes
                    .iter()
                    .map(|c| c.index())
                    .collect::<Vec<_>>())
                    == f.saved["post_state"]
                && serde_json::to_value(&out.actions)? == f.saved["pool"],
            "Prefix Context CUDA graph full pool parity differs",
        )?;
        let cont = out
            .continuation
            .as_ref()
            .ok_or_else(|| bad("Prefix Context graph U absent"))?;
        replay_require(
            json!(cont
                .final_state_codes
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>())
                == f.saved["continuation"]["state_codes"]
                && json!(cont.field.delta_scores_q24)
                    == f.saved["continuation"]["delta_scores_q24"],
            "Prefix Context CUDA U parity differs",
        )?;
        let bridge = out
            .read_state_bridge
            .as_ref()
            .ok_or_else(|| bad("Prefix Context graph bridge absent"))?;
        replay_require(
            json!(bridge.0) == f.saved["bridge"]["selected_ordinal"]
                && json!(bridge
                    .1
                    .action_codes
                    .iter()
                    .map(|c| c.index())
                    .collect::<Vec<_>>())
                    == f.saved["bridge"]["action_codes"]
                && json!(bridge.1.action_scores_q24) == f.saved["bridge"]["action_scores_q24"]
                && json!(bridge
                    .1
                    .post_state_codes
                    .iter()
                    .map(|c| c.index())
                    .collect::<Vec<_>>())
                    == f.saved["post_state"]
                && serde_json::to_value(&bridge.1.counts)? == f.saved["bridge"]["counts"],
            "Prefix Context CUDA bridge parity differs",
        )?;
        let copy = out
            .copy
            .as_ref()
            .ok_or_else(|| bad("Prefix Context graph physical bank absent"))?;
        replay_require(
            serde_json::to_value(&copy.trace)? == f.saved["bank_trace"],
            "Prefix Context CUDA full bank/context trace parity differs",
        )?;
        let loss = (out.loss_with_credit(f.target, a.credit.policy())? * f.weight)?;
        combined += f64::from(loss.to_scalar::<f32>()?);
        if !backward {
            term_receipts.push(json!({"input_index":f.input,"position":f.position,"native_forward_parity":true,"prefix_temporal_utility":enabled,"graph_credit_scope":out.credit_scope}));
            continue;
        }
        let grads = loss.backward()?;
        let mut presence = BTreeMap::new();
        for (name, var) in params {
            if let Some(g) = grads.get(var.as_tensor()) {
                replay_require(
                    g.shape() == var.shape(),
                    "Prefix Context gradient shape differs",
                )?;
                let next = if let Some(old) = sums.remove(name) {
                    (&old + g)?
                } else {
                    g.clone()
                };
                sums.insert(name.clone(), next.detach());
                presence.insert(name.clone(), json!("PRESENT"));
            } else {
                missing.insert(name.clone());
                presence.insert(name.clone(), json!("MISSING"));
            }
        }
        term_receipts.push(json!({"input_index":f.input,"position":f.position,"weight":f.weight,"gradient_presence":presence,"native_forward_parity":true,"graph_credit_scope":out.credit_scope,"prefix_temporal_utility":enabled}));
    }
    d.synchronize()?;
    let mut values = np::Shadows::new();
    let mut inventory = serde_json::Map::new();
    for (name, var) in params {
        if !backward {
            continue;
        }
        let mut receipt = json!({"shape":var.dims(),"missing_gradient_filled_zero":false,"status":if missing.contains(name){"MISSING"}else{"PRESENT"}});
        if let Some(sum) = sums.remove(name) {
            let v = sum
                .flatten_all()?
                .to_device(&Device::Cpu)?
                .to_vec1::<f32>()?;
            receipt["raw"] = raw(a, &format!("context-gradients/{label}/{name}.f32le"), &v)?;
            receipt["raw_scope"] = json!(if missing.contains(name) {
                "partial sum of connected terms; unavailable for proposal"
            } else {
                "complete weighted18 term sum"
            });
            if !missing.contains(name) {
                values.insert(name.clone(), v);
            }
        }
        inventory.insert(name.clone(), receipt);
    }
    let inventory = Value::Object(inventory);
    write(
        a,
        &format!(
            "{}-{label}-receipt.json",
            if backward {
                "gradient"
            } else {
                "forward-parity"
            }
        ),
        &json!({"prefix_temporal_utility":enabled,"prefix_temporal_credit_scope":"full120 source-before and response retained-state conditional Prefix utility","continuation_context_credit":true,"read_selector_credit":true,"categorical_bridge_credit":true,"combined_cuda_forward":combined,"elapsed_seconds":clock.elapsed().as_secs_f64(),"families":inventory,"terms":term_receipts,"optimizer_updates":0,"backward_calls":if backward{frames.len()}else{0},"full_cohort_validated_before_any_backward":true}),
    )?;
    Ok(Arm {
        values,
        inventory,
        missing,
        combined,
    })
}
#[derive(Clone, serde::Serialize)]
struct Proposal {
    name: String,
    index: usize,
    before: i8,
    after: i8,
    master_before: f32,
    master_after: f32,
    actual_delta: f64,
    g_off: f32,
    g_on: f32,
    g_difference: f64,
    dot_off: f64,
    dot_on: f64,
}
fn choose(parent: &np::Shadows, off: &np::Shadows, on: &np::Shadows) -> Result<Option<Proposal>> {
    let mut best: Option<Proposal> = None;
    for (name, masters) in parent {
        let g = on
            .get(name)
            .ok_or_else(|| bad("Prefix Context ON basis gradient unavailable"))?;
        let o = off
            .get(name)
            .ok_or_else(|| bad("Prefix Context OFF basis gradient unavailable"))?;
        replay_require(
            g.len() == masters.len() && o.len() == masters.len(),
            "Prefix Context rank shape",
        )?;
        for (i, (&m, &v)) in masters.iter().zip(g).enumerate() {
            replay_require(
                m.is_finite() && v.is_finite() && o[i].is_finite(),
                "Prefix Context rank nonfinite/offdomain",
            )?;
            let rounded = (m * 4.).round();
            replay_require(
                (-7.0..=7.0).contains(&rounded),
                "Prefix Context parent rounds outside native Q4 domain",
            )?;
            let before = rounded as i8;
            // Freeze one descent-direction adjacent code per coordinate.
            let after = (before
                + if v > 0. {
                    -1
                } else if v < 0. {
                    1
                } else {
                    0
                })
            .clamp(-7, 7);
            if after == before {
                continue;
            }
            let master_after = f32::from(after) * 0.25;
            let delta = f64::from(master_after) - f64::from(m);
            let dot = f64::from(v) * delta;
            if dot >= 0. {
                continue;
            }
            let candidate = Proposal {
                name: name.clone(),
                index: i,
                before,
                after,
                master_before: m,
                master_after,
                actual_delta: delta,
                g_off: o[i],
                g_on: v,
                g_difference: f64::from(v) - f64::from(o[i]),
                dot_off: f64::from(o[i]) * delta,
                dot_on: dot,
            };
            if best.as_ref().is_none_or(|old| {
                dot.total_cmp(&old.dot_on)
                    .then(name.cmp(&old.name))
                    .then(i.cmp(&old.index))
                    .is_lt()
            }) {
                best = Some(candidate);
            }
        }
    }
    Ok(best)
}
fn frozen_guard(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
    changed: &str,
) -> Result<()> {
    replay_require(
        before.len() == after.len()
            && before
                .iter()
                .all(|(n, h)| n == changed || after.get(n) == Some(h)),
        "Prefix Context frozen master family changed",
    )
}
fn packed_one(before: &[u8], after: &[u8], config: ContextQ4Config, p: &Proposal) -> Result<Value> {
    let count = config.coefficient_count()?;
    let a = unpack_coefficients(count, before)?;
    let b = unpack_coefficients(count, after)?;
    let mut offset = 0;
    let mut selected = None;
    for (n, shape) in config.coefficient_shapes()? {
        let size = shape.iter().product::<usize>();
        if format!("consumer.context.{n}") == p.name {
            replay_require(p.index < size, "Prefix Context packed index out of bounds")?;
            selected = Some(offset + p.index);
        }
        offset += size;
    }
    let selected = selected.ok_or_else(|| bad("Prefix Context packed family absent"))?;
    replay_require(
        a[selected] == p.before
            && b[selected] == p.after
            && a.iter()
                .zip(&b)
                .enumerate()
                .all(|(i, (x, y))| i == selected || x == y),
        "Prefix Context native code displacement differs",
    )?;
    Ok(
        json!({"packed_coordinate":selected,"before":p.before,"after":p.after,"changed_codes":1,"before_sha256":sha256_bytes(before),"after_sha256":sha256_bytes(after)}),
    )
}
fn export(
    a: &Args,
    l: &Loaded,
    field: &NativeContinuationField,
    p: &ContinuationParent,
    d: &Device,
) -> Result<(ContinuationParent, NativeContinuationField, Value)> {
    export_with_policy(a, l, field, p, d, "prefix_context_credit_probe", policy())
}
fn export_with_policy(
    a: &Args,
    l: &Loaded,
    field: &NativeContinuationField,
    p: &ContinuationParent,
    d: &Device,
    mode: &str,
    receipt_policy: Value,
) -> Result<(ContinuationParent, NativeContinuationField, Value)> {
    let (_, generate, _, mut receipt) = checkpoint(a, 1, l)?;
    let root = a.out.join("checkpoint-0001");
    let current = ContinuationParent::from_checkpoint(&root)?;
    let u = ContinuationLearningWeights::from_native(
        field,
        &NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?,
        p.integer.binding(),
        d,
    )?;
    restore(
        &a.checkpoint.join("continuation-source"),
        &p.receipt["continuation_parameters"],
        &u.parameters(),
        d,
    )?;
    let old_u = identities(&u.parameters())?;
    let rebound = u.export_native(&current.binding, &generate)?;
    replay_require(
        rebound.packed_unary() == field.packed_unary(),
        "Prefix Context U coefficients changed",
    )?;
    let masters = save_masters(&root.join("continuation-source"), &u.parameters())?;
    fs::write(root.join("continuation-field.bin"), rebound.to_bytes()?)?;
    let cpu = ContinuationLearningWeights::from_native(
        &rebound,
        &generate,
        current.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &root.join("continuation-source"),
        &masters,
        &cpu.parameters(),
        &Device::Cpu,
    )?;
    replay_require(
        identities(&cpu.parameters())? == old_u
            && cpu.export_native(&current.binding, &generate)?.to_bytes()? == rebound.to_bytes()?,
        "Prefix Context U independent master/native reload differs",
    )?;
    replay_require(
        current.generate == p.generate
            && current.bridge == p.bridge
            && current.cue == p.cue
            && current.prefix == p.prefix
            && current.joint == p.joint
            && current.exp == p.exp,
        "Prefix Context frozen payload changed",
    )?;
    receipt["mode"] = json!(mode);
    receipt["fresh_adam"] = json!(false);
    receipt["optimizer_updates"] = json!(0);
    receipt["policy"] = receipt_policy;
    if mode == "context_recorded_finite_contrast" {
        receipt["new_gradients"] = json!(0);
        receipt["training_updates"] = json!(0);
        receipt["master_loader_device"] = json!("cuda");
        receipt["native_evaluation_device"] = json!("host");
    }
    receipt["continuation_parameters"] = masters;
    receipt["continuation_sha256"] = json!(sha256_bytes(&rebound.to_bytes()?));
    receipt["parent_report_sha256"] = json!(REPORT);
    receipt["parent_manifest_sha256"] = json!(SEAL);
    receipt["source_commit"] = json!(option_env!("UOR_BUILD_SOURCE_COMMIT"));
    for path in [
        root.join("receipt.json"),
        root.join("continuation-source/metadata.json"),
    ] {
        fs::write(path, serde_json::to_vec_pretty(&receipt)?)?;
    }
    let loaded = ContinuationParent::from_checkpoint(&root)?;
    let disk = NativeContinuationField::from_bytes(
        &fs::read(root.join("continuation-field.bin"))?,
        &loaded.binding,
        &generate,
    )?;
    Ok((loaded, disk, receipt))
}

pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;
    replay_require(d.is_cuda(), "Prefix Context arms require actual CUDA")?;
    let c = a
        .prefix_context_credit
        .as_ref()
        .ok_or_else(|| bad("Prefix Context config absent"))?;
    let r = sealed(&c.retained_intermediate_root, REPORT, SEAL)?;
    let captures = sealed(&c.retained_capture_root, CAPTURE_REPORT, CAPTURE_SEAL)?;
    let p = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        r["mode"] == "readout_intermediate_candidate"
            && r["selected_model"] == true
            && r["candidate_artifact_status"] == "QUALIFIED_NATIVE_GATE_AND_RETENTION"
            && r["final_receipt"] == p.receipt
            && p.binding.metadata_sha256 == SOURCE
            && p.generate_sha256 == GENERATE
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == FIELD,
        "Prefix Context recovered selected parent differs",
    )?;
    replay_require(
        captures["schema"] == "uor-r4.native-reached-prefix-attribution/3"
            && captures["endpoint_kind"] == "selected_readout_intermediate"
            && captures["compensation_report_sha256"] == REPORT
            && captures["compensation_manifest_sha256"] == SEAL
            && captures["generate_sha256"] == GENERATE
            && captures["continuation_sha256"] == FIELD
            && captures["source_binding"] == serde_json::to_value(&p.binding)?,
        "Prefix Context capture endpoint binding differs",
    )?;
    for (path, pin) in [
        (&a.training_inputs, INPUT_SHA),
        (&a.training_labels, LABEL_SHA),
    ] {
        report_output::verify(&seal_for(path)?)?;
        replay_require(
            sha256_file(path)? == pin,
            "Prefix Context panel authority differs",
        )?;
    }
    replay_require(
        a.training_inputs == a.development_inputs && a.training_labels == a.development_labels,
        "Prefix Context panel paths differ",
    )?;
    let reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let legal = reducer
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
    replay_require(eps.len() == 512, "Prefix Context panel length differs")?;
    let listed = captures["frames"]
        .as_array()
        .ok_or_else(|| bad("Prefix Context capture frames absent"))?;
    let pairs = listed
        .iter()
        .map(|v| Ok((index(&v["input_index"])?, index(&v["position"])?)))
        .collect::<Result<Vec<_>>>()?;
    cohort(&pairs)?;
    let mut frames = Vec::new();
    for (entry, (input, position)) in listed.iter().zip(pairs) {
        let file = format!("frame-{input:04}-position-{position:02}.json");
        replay_require(entry["file"] == file, "Prefix Context frame path differs")?;
        let path = c.retained_capture_root.join(file);
        replay_require(
            entry["sha256"] == sha256_file(&path)?,
            "Prefix Context capture frame hash differs",
        )?;
        let mut saved = read(&path)?;
        let prefix: Vec<u32> = decode(&saved["actual_prefix_ids"])?;
        let e = eps
            .get(input)
            .ok_or_else(|| bad("Prefix Context input absent"))?;
        let actualpath = c
            .retained_intermediate_root
            .join(format!("development-0001-row-{input:04}.json"));
        let actual = read(&actualpath)?;
        replay_require(
            saved["schema"] == "uor-r4.native-reached-prefix-frame/3"
                && saved["id"] == e.packet.id
                && entry["id"] == e.packet.id
                && actual["id"] == e.packet.id
                && saved["saved_row_sha256"] == sha256_file(&actualpath)?
                && prefix.len() == position
                && saved["capture_target_free"] == true
                && saved["label_access_before_capture"] == false
                && saved["saved_actual"] == actual["generation"][position]
                && saved["saved_canonical_native"] == actual["canonical"][position]["native"],
            "Prefix Context actual frame authority differs",
        )?;
        replay_require(
            actual["generation"][position]["actual_prefix_ids"] == json!(prefix)
                && prefix.iter().enumerate().all(|(i, t)| {
                    actual["generation"][i]["pool"]["summary"]["chosen_token_id"] == *t
                }),
            "Prefix Context actual prefix chain differs",
        )?;
        let target = *e
            .target
            .get(position)
            .ok_or_else(|| bad("Prefix Context canonical target absent"))?;
        let task = input == 245 && position == 4;
        replay_require(
            (task && target == 267 && entry["winner"] == 307)
                || (!task && entry["winner"] == target),
            "Prefix Context fixed task/reference winner differs",
        )?;
        // The immutable capture seal binds full factor/provenance evidence. Avoid
        // keeping verbose factor arrays alongside both CUDA graphs.
        for key in [
            "factors",
            "u_factors",
            "controls",
            "saved_actual",
            "saved_canonical_native",
        ] {
            if let Some(v) = saved.as_object_mut() {
                v.remove(key);
            }
        }
        frames.push(Frame {
            input,
            position,
            id: e.packet.id.clone(),
            prefix,
            target,
            weight: if task { 1. } else { 1. / 17. },
            saved,
        });
    }
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &p.binding,
        &NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?,
    )?;
    if let Some(finite) = &c.recorded_finite_contrast {
        return run_finite(a, start, d, finite, &p, &field, &eps, frames);
    }
    let l = load_joint_continuation(a, &p, d)?;
    let params = l
        .source
        .parameters()
        .into_iter()
        .filter(|(n, _)| n.starts_with("consumer.context."))
        .collect::<BTreeMap<_, _>>();
    replay_require(
        params.len() == 9,
        "Prefix Context requires all9 Context families",
    )?;
    let basis = params
        .iter()
        .filter(|(n, _)| {
            np::BASIS
                .iter()
                .any(|s| n.as_str() == format!("consumer.context.{s}"))
        })
        .map(|(n, v)| (n.clone(), v.clone()))
        .collect::<BTreeMap<_, _>>();
    replay_require(
        basis.len() == 6 && basis.values().map(|v| v.elem_count()).sum::<usize>() == 17472,
        "Prefix Context six basis shape population differs",
    )?;
    let masters = np::snapshot(&basis)?;
    let mut master_inventory = serde_json::Map::new();
    for (name, values) in &masters {
        let mut v = raw(a, &format!("context-initial-masters/{name}.f32le"), values)?;
        v["shape"] = json!(basis[name].dims());
        master_inventory.insert(name.clone(), v);
    }
    let config = ContextQ4Config {
        vocab_size: 4096,
        heads: 2,
        lanes_per_head: 4,
    };
    write(
        a,
        "context-initial-masters.json",
        &json!({"families":master_inventory,"native_layout":config.coefficient_shapes()?,"proposal_scalars":17472,"parent_report_sha256":REPORT}),
    )?;
    let source_before = identities(&l.source.parameters())?;
    let generate_before = identities(&l.generate.parameters())?;
    let retained_serialized = frames.iter().try_fold(0usize, |sum, f| {
        Ok::<usize, Box<dyn std::error::Error>>(sum + serde_json::to_vec(&f.saved)?.len())
    })?;
    let context_raw = params.values().map(|v| v.elem_count() * 4).sum::<usize>();
    let largest_family = params
        .values()
        .map(|v| v.elem_count() * 4)
        .max()
        .ok_or_else(|| bad("Context families absent"))?;
    let numeric_projection = retained_serialized
        .checked_mul(4)
        .and_then(|n| {
            n.checked_add(2 * context_raw + 2 * largest_family + 32 * 1024 * 1024 + 17472 * 4)
        })
        .ok_or_else(|| bad("Prefix Context cache projection overflow"))?;
    let report_projection = size(&a.checkpoint)?
        + 3 * context_raw as u64
        + 4 * retained_serialized as u64
        + 64 * 1024 * 1024;
    replay_require(
        numeric_projection <= 256 * 1024 * 1024
            && report_projection < a.maximum_report_bytes - 1_048_576,
        "Prefix Context projected numeric/report resource cap",
    )?;
    write(
        a,
        "resource-projection.json",
        &json!({"numeric_cache_cap_bytes":256*1024*1024,"numeric_projection_bytes":numeric_projection,"retained_frame_serialized_bytes":retained_serialized,"cpu_off_on_bytes":2*context_raw,"one_family_difference_and_raw_buffers":2*largest_family,"descriptor_and_native_pool_margin":32*1024*1024,"report_projection_bytes":report_projection,"report_cap_bytes":a.maximum_report_bytes,"scope":"host numerical maps/retained captures/one streamed family; CUDA model and graph tensors and serde temporaries separately charged to8GiB process RAM; no18 retained graphs"}),
    )?;
    let baseline = evaluate(a, start, &p, &field, &eps, &frames, "baseline", true)?;
    // Each arm finishes/backpropagates one term at a time. No retained 18-graph batch.
    let _off_parity = arm(
        a, start, d, &l, &p, &field, &eps, &frames, &params, false, false,
    )?;
    let _on_parity = arm(
        a, start, d, &l, &p, &field, &eps, &frames, &params, true, false,
    )?;
    let off = arm(
        a, start, d, &l, &p, &field, &eps, &frames, &params, false, true,
    )?;
    let on = arm(
        a, start, d, &l, &p, &field, &eps, &frames, &params, true, true,
    )?;
    replay_require(
        source_before == identities(&l.source.parameters())?
            && generate_before == identities(&l.generate.parameters())?,
        "Prefix Context gradient arms modified masters",
    )?;
    let mut differences = serde_json::Map::new();
    for (name, var) in &params {
        let receipt = if let (Some(x), Some(y)) = (off.values.get(name), on.values.get(name)) {
            let delta = y.iter().zip(x).map(|(y, x)| *y - *x).collect::<Vec<_>>();
            let mut receipt = raw(
                a,
                &format!("context-gradients/difference/{name}.f32le"),
                &delta,
            )?;
            receipt["shape"] = json!(var.dims());
            receipt["status"] = json!("PRESENT");
            receipt["subtraction"] = json!("f32 ON-minus-OFF after weighted f32 accumulation");
            receipt
        } else {
            json!({"shape":var.dims(),"status":"UNAVAILABLE","missing_gradient_filled_zero":false})
        };
        differences.insert(name.clone(), receipt);
    }
    write(
        a,
        "gradient-difference-receipt.json",
        &Value::Object(differences),
    )?;
    let eligible = masters
        .iter()
        .filter(|(n, _)| !off.missing.contains(*n) && !on.missing.contains(*n))
        .map(|(n, v)| (n.clone(), v.clone()))
        .collect::<np::Shadows>();
    let skipped = masters
        .keys()
        .filter(|n| !eligible.contains_key(*n))
        .collect::<Vec<_>>();
    let unavailable = eligible.is_empty();
    let proposal = if unavailable {
        None
    } else {
        choose(&eligible, &off.values, &on.values)?
    };
    write(
        a,
        "single-proposal.json",
        &json!({"status":if unavailable{"NOT_RUN_MISSING_BASIS_GRADIENT"}else if proposal.is_none(){"NO_NEGATIVE_LEGAL_DISPLACEMENT"}else{"SELECTED"},"proposal":proposal,"rank":"g_ON*actual master delta/name/index","population":17472,"eligible_scalars":eligible.values().map(Vec::len).sum::<usize>(),"skipped_missing_families":skipped,"rank_uses_difference":false}),
    )?;
    let mut effect = Value::Null;
    let mut receipt = Value::Null;
    let mut positive = false;
    if let Some(proposal) = &proposal {
        // Restore the actual fractional parent even on a late export/score error.
        (effect, receipt) = np::attempt_restored(&basis, &masters, || {
            let mut changed = masters[&proposal.name].clone();
            changed[proposal.index] = proposal.master_after;
            let v = &basis[&proposal.name];
            v.set(&Tensor::from_vec(changed, v.shape(), v.device())?)?;
            let expected = masters
                .iter()
                .map(|(n, x)| {
                    let mut x = x.clone();
                    if n == &proposal.name {
                        x[proposal.index] = proposal.master_after;
                    }
                    (n.clone(), x)
                })
                .collect::<np::Shadows>();
            replay_require(
                np::same_bits(&expected, &np::snapshot(&basis)?),
                "Prefix Context displacement changed unselected bits",
            )?;
            frozen_guard(
                &source_before,
                &identities(&l.source.parameters())?,
                &proposal.name,
            )?;
            replay_require(
                generate_before == identities(&l.generate.parameters())?,
                "Prefix Context Generate masters changed",
            )?;
            let (candidate, candidate_field, receipt) = export(a, &l, &field, &p, d)?;
            let old = fs::read(a.checkpoint.join("native/consumer/context-q4.bin"))?;
            let new = fs::read(a.out.join("checkpoint-0001/native/consumer/context-q4.bin"))?;
            write(
                a,
                "native-context-displacement.json",
                &packed_one(&old, &new, config, proposal)?,
            )?;
            let objective = evaluate(
                a,
                start,
                &candidate,
                &candidate_field,
                &eps,
                &frames,
                "candidate",
                false,
            )?;
            Ok((objective, receipt))
        })?;
        let before = baseline["combined"]
            .as_f64()
            .ok_or_else(|| bad("Prefix Context baseline CE absent"))?;
        let after = effect["combined"]
            .as_f64()
            .ok_or_else(|| bad("Prefix Context candidate CE absent"))?;
        let old = &baseline["terms"][1];
        let new = &effect["terms"][1];
        let old_m = old["target_mass"]
            .as_u64()
            .ok_or_else(|| bad("Prefix Context old mass absent"))?;
        let old_d = old["total_mass"]
            .as_u64()
            .ok_or_else(|| bad("Prefix Context old denominator absent"))?;
        let new_m = new["target_mass"]
            .as_u64()
            .ok_or_else(|| bad("Prefix Context new mass absent"))?;
        let new_d = new["total_mass"]
            .as_u64()
            .ok_or_else(|| bad("Prefix Context new denominator absent"))?;
        positive = after < before - 1e-10 * (1. + before.abs())
            && (new_m as u128) * (old_d as u128) > (old_m as u128) * (new_d as u128)
            && effect["correct_reference_frames"] == 17;
    }
    replay_require(
        source_before == identities(&l.source.parameters())?
            && generate_before == identities(&l.generate.parameters())?,
        "Prefix Context final parent restoration differs",
    )?;
    Ok(
        json!({"schema":"uor-r4.prefix-context-credit-probe/1","status":"COMPLETED","mode":"prefix_context_credit","policy":policy(),"parent_report_sha256":REPORT,"parent_manifest_sha256":SEAL,"capture_report_sha256":CAPTURE_REPORT,"capture_manifest_sha256":CAPTURE_SEAL,
        "baseline_objective":baseline,"candidate_objective":effect,"finite_probe_positive":positive,"selected_model":false,"useful_candidate":false,"autoregressive_rollout":"NOT_RUN","full_prefix_qualification":"NOT_RUN","optimizer_updates":0,
        "gradient_off":off.inventory,"gradient_on":on.inventory,"gradient_off_forward":off.combined,"gradient_on_forward":on.combined,"single_proposal":proposal,"candidate_receipt":receipt,"parent_master_bits_restored":true,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT")}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefix_context_actual_fractional_delta_and_deterministic_ties() -> Result<()> {
        let p = np::Shadows::from([("a".into(), vec![0.12, 0.12]), ("b".into(), vec![0.12])]);
        let off = np::Shadows::from([("a".into(), vec![1., 1.]), ("b".into(), vec![1.])]);
        let on = np::Shadows::from([("a".into(), vec![-2., -2.]), ("b".into(), vec![-2.])]);
        let q = choose(&p, &off, &on)?.ok_or_else(|| bad("test proposal absent"))?;
        assert_eq!(
            (q.name.as_str(), q.index, q.before, q.after),
            ("a", 0, 0, 1)
        );
        assert_eq!(q.actual_delta, 0.25 - f64::from(0.12f32));
        assert!(q.dot_off > 0. && q.dot_on < 0.);
        Ok(())
    }
    #[test]
    fn prefix_context_missing_gradient_is_unavailable_not_zero() -> Result<()> {
        let p = np::Shadows::from([("a".into(), vec![0.12])]);
        let on = np::Shadows::from([("a".into(), vec![-1.])]);
        assert!(choose(&p, &np::Shadows::new(), &on).is_err());
        assert!(choose(&p, &on, &np::Shadows::new()).is_err());
        Ok(())
    }
    #[test]
    fn prefix_context_exact_cohort_includes_correct_preceding_frame() -> Result<()> {
        let mut x = COHORT
            .into_iter()
            .flat_map(|i| [(i, 3), (i, 4)])
            .collect::<Vec<_>>();
        cohort(&x)?;
        x[0] = (245, 4);
        assert!(cohort(&x).is_err());
        x[0] = (245, 3);
        x.swap(2, 4);
        assert!(cohort(&x).is_err());
        Ok(())
    }
    #[test]
    fn prefix_context_fractional_restore_after_late_error_and_frozen_guard() -> Result<()> {
        let v = Var::from_vec(vec![-0.0f32, 0.12], 2, &Device::Cpu)?;
        let params = BTreeMap::from([("a".into(), v)]);
        let original = np::snapshot(&params)?;
        let result: Result<()> = np::attempt_restored(&params, &original, || {
            np::apply_edits(
                &params,
                &original,
                &[np::Edit {
                    name: "a".into(),
                    index: 1,
                    before: 0,
                    after: 1,
                }],
            )?;
            Err(bad("late finite scoring failure"))
        });
        assert!(result.is_err());
        assert!(np::same_bits(&original, &np::snapshot(&params)?));
        let before = BTreeMap::from([
            ("a".into(), "old".into()),
            ("frozen".into(), "bitsha".into()),
        ]);
        let mut after = before.clone();
        after.insert("a".into(), "new".into());
        frozen_guard(&before, &after, "a")?;
        after.insert("frozen".into(), "different".into());
        assert!(frozen_guard(&before, &after, "a").is_err());
        Ok(())
    }
}

fn finite_policy() -> Value {
    json!({"schema":"uor-r4.context-recorded-finite-contrast/1","intervention":"already recorded self_transition3610 actual master to-.5; nativeq-1to-2","new_gradients":0,"optimizer_updates":0,"new_proposals":0,"reranking":false,"baseline":"reused sealed1947 eighteen raw native pools; no baseline encoder replay","candidate":"one coherent export/independentreload followed by18fixed actual-prefix steps; task Contextcapture within same2steps","objective":"task245pos4 weight1+other17each1/17","positive":"strictcombinedCEdescent+exact tasktargetprobabilityimprovement+all17referencewinnerguards","selected_model":false,"scope":"finite18contrast only; no full-prefix/reply/useful qualification"})
}
fn capture_difference(
    position: usize,
    old: &[uor_r4_integer::geometric_context::ContextDecisionEvent],
    new: &[uor_r4_integer::geometric_context::ContextDecisionEvent],
) -> Value {
    let mut first_any = None;
    let mut first_topology = None;
    let mut first_scores = None;
    let mut first_action_or_state = None;
    let mut winner_changes = 0;
    let mut transition_winner_changes = 0;
    for i in 0..old.len().max(new.len()) {
        let a = old.get(i);
        let b = new.get(i);
        let topology = match (a, b) {
            (Some(a), Some(b)) => {
                a.call_index != b.call_index
                    || a.invocation != b.invocation
                    || a.family != b.family
                    || a.token_id != b.token_id
                    || a.flat_lane != b.flat_lane
                    || a.head != b.head
                    || a.lane != b.lane
            }
            _ => true,
        };
        let states = match (a, b) {
            (Some(a), Some(b)) => a.own != b.own || a.neighbor != b.neighbor,
            _ => false,
        };
        let scores = match (a, b) {
            (Some(a), Some(b)) => a.scores_q24 != b.scores_q24,
            _ => false,
        };
        let winner = match (a, b) {
            (Some(a), Some(b)) => a.winner != b.winner,
            _ => false,
        };
        if !topology && winner {
            winner_changes += 1;
            if a.is_some_and(|a| {
                a.family == uor_r4_integer::geometric_context::ContextDecisionFamily::Transition
            }) {
                transition_winner_changes += 1;
            }
        }
        let receipt = || json!({"event_index":i,"baseline":a,"candidate":b,"topology_difference":topology,"input_state_difference":states,"score_difference":scores,"winner_difference":winner,"comparison_scope":"aligned ordinal events; a topology mismatch invalidates subsequent call-site correspondence"});
        if topology && first_topology.is_none() {
            first_topology = Some(receipt());
        }
        if scores && !topology && first_scores.is_none() {
            first_scores = Some(receipt());
        }
        if (states || winner) && !topology && first_action_or_state.is_none() {
            first_action_or_state = Some(receipt());
        }
        if (topology || states || scores || winner) && first_any.is_none() {
            first_any = Some(receipt());
        }
    }
    json!({"input_index":245,"position":position,"baseline_events":old.len(),"candidate_events":new.len(),"first_any_event_difference":first_any,"first_topology_difference":first_topology,"first_score_difference":first_scores,"first_action_or_state_difference":first_action_or_state,"aligned_winner_changes":winner_changes,"aligned_transition_winner_changes":transition_winner_changes,"scope":"actual candidate recurrence versus saved baseline events; distinct from four baseline held-input hypothetical crossings"})
}
fn recorded_proposal(v: &Value) -> Result<Proposal> {
    replay_require(
        v["name"] == "consumer.context.self_transition"
            && v["index"] == 3610
            && v["before"] == -1
            && v["after"] == -2
            && v["decoded"] == json!({"head":1,"lane":3,"class":62,"basis_component":2}),
        "Recorded finite coordinate differs",
    )?;
    let scalar = |n: &str| {
        v[n].as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| bad("Recorded finite scalar absent/nonfinite"))
    };
    let before = scalar("master_before")?;
    let after = scalar("master_after")?;
    replay_require(
        before == f64::from(-0.1445201337337494f32)
            && after == -0.5
            && scalar("actual_delta")? == after - before,
        "Recorded finite fractional displacement differs",
    )?;
    let proposal = Proposal {
        name: "consumer.context.self_transition".into(),
        index: 3610,
        before: -1,
        after: -2,
        master_before: before as f32,
        master_after: after as f32,
        actual_delta: after - before,
        g_off: scalar("g_off")? as f32,
        g_on: scalar("g_on")? as f32,
        g_difference: scalar("g_difference_f64")?,
        dot_off: scalar("dot_off")?,
        dot_on: scalar("dot_on")?,
    };
    replay_require(
        proposal.dot_off < 0.
            && proposal.dot_on < 0.
            && scalar("temporal_dot")? < 0.
            && proposal.dot_on == f64::from(proposal.g_on) * proposal.actual_delta
            && proposal.dot_off == f64::from(proposal.g_off) * proposal.actual_delta,
        "Recorded finite gradient prediction differs",
    )?;
    Ok(proposal)
}
fn finite_gate(baseline: &Value, candidate: &Value) -> Result<Value> {
    let before = baseline["combined"]
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| bad("Finite baseline CE absent"))?;
    let after = candidate["combined"]
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| bad("Finite candidate CE absent"))?;
    let term = |v: &Value| -> Result<Value> {
        let rows = v["terms"]
            .as_array()
            .ok_or_else(|| bad("Finite objective terms absent"))?;
        let matched = rows
            .iter()
            .filter(|t| t["input_index"] == 245 && t["position"] == 4)
            .collect::<Vec<_>>();
        replay_require(matched.len() == 1, "Finite task term uniqueness")?;
        Ok(matched[0].clone())
    };
    let old = term(baseline)?;
    let new = term(candidate)?;
    let mass = |v: &Value, key: &str| {
        v[key]
            .as_u64()
            .ok_or_else(|| bad("Finite exact mass absent"))
    };
    let (om, od, nm, nd) = (
        mass(&old, "target_mass")?,
        mass(&old, "total_mass")?,
        mass(&new, "target_mass")?,
        mass(&new, "total_mass")?,
    );
    replay_require(
        od > 0 && nd > 0 && om > 0 && nm > 0 && om <= od && nm <= nd,
        "Finite mass support invalid",
    )?;
    let ce = after < before - 1e-10 * (1. + before.abs());
    let probability = (nm as u128) * (od as u128) > (om as u128) * (nd as u128);
    let guards = candidate["correct_reference_frames"] == 17;
    Ok(
        json!({"strict_ce_descent":ce,"task_target_probability_improved":probability,"all17_reference_winners_retained":guards,"finite_probe_positive":ce&&probability&&guards,"task_correct":new["chosen"]==new["target"],"baseline_combined":before,"candidate_combined":after,"task_baseline_mass":om,"task_baseline_denominator":od,"task_candidate_mass":nm,"task_candidate_denominator":nd,"scope":"fixed18 finite gate only; no selected/reply qualification"}),
    )
}
fn run_finite(
    a: &Args,
    start: Instant,
    d: &Device,
    c: &RecordedFiniteContrastConfig,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    mut frames: Vec<Frame>,
) -> Result<Value> {
    const PATH_REPORT: &str = "ee12bfc8275768886024ec5784624315b2df2f70e9f2d2bfe7e30167ca3cba24";
    const PATH_SEAL: &str = "e545b8598294b9bbdd7d0c72227796425300bfebdbf0edb7c73c3a1ed4f5f96e";
    const PROBE_REPORT: &str = "1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9";
    const PROBE_SEAL: &str = "c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332";
    let path = sealed(&c.retained_path_root, PATH_REPORT, PATH_SEAL)?;
    let probe = sealed(&c.retained_probe_root, PROBE_REPORT, PROBE_SEAL)?;
    replay_require(
        path["schema"] == "uor-r4.context-path-credit/1"
            && path["task_path_unchanged"] == false
            && path["task_action_crossings"] == 4
            && path["source_binding"] == serde_json::to_value(&p.binding)?
            && path["generate_sha256"] == GENERATE
            && path["continuation_sha256"] == FIELD
            && probe["schema"] == "uor-r4.prefix-context-credit-probe/1"
            && probe["parent_report_sha256"] == REPORT
            && probe["parent_manifest_sha256"] == SEAL,
        "Finite recorded path/probe authority differs",
    )?;
    let proposal = recorded_proposal(&path["candidate"])?;
    let baseline = read(&c.retained_probe_root.join("baseline.json"))?;
    replay_require(
        baseline == probe["baseline_objective"]
            && baseline["combined"] == 4.391651926613631
            && baseline["correct_reference_frames"] == 17,
        "Finite original18 baseline differs",
    )?;
    let mut raw_receipts = Vec::new();
    let mut reconstructed = [0f64; 2];
    for (i, f) in frames.iter_mut().enumerate() {
        let file = format!(
            "baseline-row-{:04}-position-{:02}.json",
            f.input, f.position
        );
        let saved = read(&c.retained_probe_root.join(&file))?;
        replay_require(
            saved["input_index"] == f.input
                && saved["position"] == f.position
                && saved["id"] == f.id
                && saved["actual_prefix_ids"] == json!(f.prefix)
                && saved["target_label_only"] == f.target
                && saved["weight"] == f.weight,
            "Finite reused baseline frame differs",
        )?;
        parity(&saved["native"], &f.saved)?;
        let (mass, total) = masses(&saved["native"], f.target)?;
        replay_require(
            mass > 0 && mass <= total,
            "Finite baseline native masses invalid",
        )?;
        let term = &baseline["terms"][i];
        replay_require(
            term["input_index"] == f.input
                && term["position"] == f.position
                && term["id"] == f.id
                && term["target"] == f.target
                && term["weight"] == f.weight
                && term["target_mass"] == mass
                && term["total_mass"] == total
                && term["pool"] == saved["native"]["pool"]["summary"],
            "Finite reused baseline objective/raw binding differs",
        )?;
        reconstructed[usize::from(!(f.input == 245 && f.position == 4))] +=
            f.weight * (-(mass as f64 / total as f64).ln());
        f.saved = saved["native"].clone();
        raw_receipts.push(json!({"file":file,"sha256":sha256_file(&c.retained_probe_root.join(&file))?,"input_index":f.input,"position":f.position}));
    }
    replay_require(
        (reconstructed[0] + reconstructed[1] - 4.391651926613631).abs() < 1e-12,
        "Finite baseline weight reconstruction differs",
    )?;
    let baseline_capture_path = c.retained_path_root.join("task-context-decisions.json");
    replay_require(
        path["fresh_capture"]["file"] == "task-context-decisions.json"
            && path["fresh_capture"]["sha256"] == sha256_file(&baseline_capture_path)?,
        "Finite baseline Context capture differs",
    )?;
    let baseline_capture: constrained_context::CapturedDecisions =
        serde_json::from_reader(io::BufReader::new(fs::File::open(&baseline_capture_path)?))?;
    let numerical_projection = frames.iter().try_fold(0u64, |sum, f| {
        Ok::<_, Box<dyn std::error::Error>>(sum + serde_json::to_vec(&f.saved)?.len() as u64)
    })? * 4
        + 128 * 1024 * 1024;
    let report_projection = size(&a.checkpoint)? + numerical_projection / 4 + 32 * 1024 * 1024;
    replay_require(
        numerical_projection <= 512 * 1024 * 1024
            && report_projection + 1048576 < a.maximum_report_bytes,
        "Finite numerical/report projection exceedscap",
    )?;
    write(
        a,
        "resource-projection.json",
        &json!({"numeric_projection_bytes":numerical_projection,"maximum_numeric_cache_bytes":536870912u64,"report_projection_bytes":report_projection,"maximum_report_bytes":a.maximum_report_bytes,"maximum_process_ram_bytes":4294967296u64,"maximum_temporary_bytes":536870912u64,"maximum_cpu_threads":2,"scope":"host numeric snapshot/capture/native array projection; CUDA model tensors and serde report temporaries separately charged processRAM"}),
    )?;
    let l = load_joint_continuation(a, p, d)?;
    let source_before = identities(&l.source.parameters())?;
    let generate_before = identities(&l.generate.parameters())?;
    let bridge_before = json!({"original":identities(&l.original_bridge.parameters())?,"marker":identities(&l.marker.parameters())?,"categorical":l.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?});
    let basis = l
        .source
        .parameters()
        .into_iter()
        .filter(|(n, _)| {
            np::BASIS
                .iter()
                .any(|s| n == &format!("consumer.context.{s}"))
        })
        .collect::<BTreeMap<_, _>>();
    let masters = np::snapshot(&basis)?;
    replay_require(
        basis.len() == 6
            && masters[&proposal.name][proposal.index].to_bits()
                == proposal.master_before.to_bits(),
        "Finite restored fractional coordinate differs",
    )?;
    write(
        a,
        "recorded-finite-proposal.json",
        &json!({"authority":path["candidate"],"actual_restored_master":masters[&proposal.name][proposal.index],"reranking":false,"new_gradient":false}),
    )?;
    let (candidate_objective, receipt, code_receipt) =
        np::attempt_restored(&basis, &masters, || {
            let mut expected = masters.clone();
            expected
                .get_mut(&proposal.name)
                .ok_or_else(|| bad("Finite basis absent"))?[proposal.index] = proposal.master_after;
            let var = basis
                .get(&proposal.name)
                .ok_or_else(|| bad("Finite Var absent"))?;
            var.set(&Tensor::from_vec(
                expected[&proposal.name].clone(),
                var.shape(),
                var.device(),
            )?)?;
            replay_require(
                np::same_bits(&expected, &np::snapshot(&basis)?),
                "Finite unselected Context scalar changed",
            )?;
            frozen_guard(
                &source_before,
                &identities(&l.source.parameters())?,
                &proposal.name,
            )?;
            replay_require(
                generate_before == identities(&l.generate.parameters())?,
                "Finite Generate masters changed",
            )?;
            let (candidate, candidate_field, mut receipt) = export_with_policy(
                a,
                &l,
                field,
                p,
                d,
                "context_recorded_finite_contrast",
                finite_policy(),
            )?;
            receipt["recorded_path_report_sha256"] = json!(PATH_REPORT);
            receipt["recorded_path_manifest_sha256"] = json!(PATH_SEAL);
            receipt["recorded_proposal"] = path["candidate"].clone();
            for file in [
                a.out.join("checkpoint-0001/receipt.json"),
                a.out
                    .join("checkpoint-0001/continuation-source/metadata.json"),
            ] {
                fs::write(file, serde_json::to_vec_pretty(&receipt)?)?;
            }
            let old = fs::read(a.checkpoint.join("native/consumer/context-q4.bin"))?;
            let new = fs::read(a.out.join("checkpoint-0001/native/consumer/context-q4.bin"))?;
            let code_receipt = packed_one(
                &old,
                &new,
                ContextQ4Config {
                    vocab_size: 4096,
                    heads: 2,
                    lanes_per_head: 4,
                },
                &proposal,
            )?;
            write(a, "native-context-displacement.json", &code_receipt)?;
            let objective = evaluate_inner(
                a,
                start,
                &candidate,
                &candidate_field,
                eps,
                &frames,
                "candidate",
                false,
                Some(&baseline_capture),
            )?;
            Ok((objective, receipt, code_receipt))
        })?;
    let source_after = identities(&l.source.parameters())?;
    let generate_after = identities(&l.generate.parameters())?;
    let bridge_after = json!({"original":identities(&l.original_bridge.parameters())?,"marker":identities(&l.marker.parameters())?,"categorical":l.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?});
    replay_require(
        source_before == source_after
            && generate_before == generate_after
            && bridge_before == bridge_after,
        "Finite parent master restoration differs",
    )?;
    let gate = finite_gate(&baseline, &candidate_objective)?;
    write(
        a,
        "reused-baseline.json",
        &json!({"report_sha256":PROBE_REPORT,"manifest_sha256":PROBE_SEAL,"objective_file":"baseline.json","objective_sha256":sha256_file(&c.retained_probe_root.join("baseline.json"))?,"raw_frames":raw_receipts,"objective":baseline,"native_baseline_steps":0}),
    )?;
    write(
        a,
        "frozen-master-identities.json",
        &json!({"source_before":source_before,"source_restored":source_after,"generate_before":generate_before,"generate_restored":generate_after,"bridge_before":bridge_before,"bridge_restored":bridge_after,"selected_family_unselected_bits_checked":true,"parent_restored":true}),
    )?;
    Ok(
        json!({"schema":"uor-r4.context-recorded-finite-contrast/1","status":"COMPLETED","mode":"context_recorded_finite_contrast","policy":finite_policy(),"parent_report_sha256":REPORT,"parent_manifest_sha256":SEAL,"path_report_sha256":PATH_REPORT,"path_manifest_sha256":PATH_SEAL,"probe_report_sha256":PROBE_REPORT,"probe_manifest_sha256":PROBE_SEAL,"baseline_objective":baseline,"baseline_provenance":"reused sealed18 raw arrays; zero baseline encoding steps","candidate_objective":candidate_objective,"candidate_receipt":receipt,"native_context_displacement":code_receipt,"gate":gate,"finite_probe_positive":gate["finite_probe_positive"],"selected_model":false,"useful_candidate":false,"parent_master_bits_restored":true,"new_gradients":0,"optimizer_updates":0,"reranking":"NOT_RUN","master_loader_device":"cuda","native_evaluation_device":"host","gpu_training":"NOT_RUN","native_candidate_steps":18,"native_candidate_task_capture_steps":2,"actual_context_capture":"same two task steps; no additional capture step","autoregressive_rollout":"NOT_RUN","full_prefix_qualification":"NOT_RUN","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"one recorded coordinate finite18 contrast; baseline hypothetical crossings are not actual candidate recurrence changes"}),
    )
}

#[cfg(test)]
mod finite_tests {
    use super::*;
    fn proposal() -> Result<Proposal> {
        recorded_proposal(
            &json!({"name":"consumer.context.self_transition","index":3610,"before":-1,"after":-2,"decoded":{"head":1,"lane":3,"class":62,"basis_component":2},"master_before":f64::from(-0.1445201337337494f32),"master_after":-0.5,"actual_delta":-0.5-f64::from(-0.1445201337337494f32),"g_off":f64::from(0.0013931867433711886f32),"g_on":f64::from(0.0070373560301959515f32),"g_difference_f64":0.005644169286824763,"dot_off":-0.0004952498372175033,"dot_on":-0.002501638380482049,"temporal_dot":-0.0020063885432645458}),
        )
    }
    #[test]
    fn recorded_finite_code_rejects_second_native_change() -> Result<()> {
        use uor_r4_integer::geometric_context_q4::pack_coefficients;
        let p = proposal()?;
        let config = ContextQ4Config {
            vocab_size: 4096,
            heads: 2,
            lanes_per_head: 4,
        };
        let mut offset = 0;
        let mut selected = None;
        for (name, shape) in config.coefficient_shapes()? {
            if name == "self_transition" {
                selected = Some(offset + p.index);
            }
            offset += shape.iter().product::<usize>();
        }
        let selected = selected.ok_or_else(|| bad("testbasisabsent"))?;
        let mut old = vec![0i8; config.coefficient_count()?];
        old[selected] = -1;
        let a = pack_coefficients(&old)?;
        let mut changed = old;
        changed[selected] = -2;
        let b = pack_coefficients(&changed)?;
        packed_one(&a, &b, config, &p)?;
        changed[selected + 1] = 1;
        assert!(packed_one(&a, &pack_coefficients(&changed)?, config, &p).is_err());
        Ok(())
    }
    #[test]
    fn recorded_finite_late_error_and_negative_restore_fractional_signed_bits() -> Result<()> {
        let p = proposal()?;
        let mut values = vec![-0f32; 3840];
        values[p.index] = p.master_before;
        let params = BTreeMap::from([(p.name.clone(), Var::from_vec(values, 3840, &Device::Cpu)?)]);
        let parent = np::snapshot(&params)?;
        let apply = || -> Result<()> {
            let mut x = parent[&p.name].clone();
            x[p.index] = p.master_after;
            let var = &params[&p.name];
            var.set(&Tensor::from_vec(x, var.shape(), var.device())?)?;
            Ok(())
        };
        let failed: Result<()> = np::attempt_restored(&params, &parent, || {
            apply()?;
            Err(bad("late export/score fixtureerror"))
        });
        assert!(failed.is_err());
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        let negative = np::attempt_restored(&params, &parent, || {
            apply()?;
            Ok(false)
        })?;
        assert!(!negative);
        assert!(np::same_bits(&parent, &np::snapshot(&params)?));
        Ok(())
    }
    #[test]
    fn recorded_finite_gate_uses_exact_mass_ratio_and_reference_guard() -> Result<()> {
        let m = 1u64 << 63;
        let d = m + 10;
        let old = json!({"combined":4.0,"correct_reference_frames":17,"terms":[{"input_index":245,"position":4,"target":267,"chosen":307,"target_mass":m,"total_mass":d}]});
        let mut new = old.clone();
        new["combined"] = json!(3.0);
        new["terms"][0]["target_mass"] = json!(m + 1);
        assert_eq!(m as f64 / d as f64, (m + 1) as f64 / d as f64);
        assert_eq!(finite_gate(&old, &new)?["finite_probe_positive"], true);
        new["correct_reference_frames"] = json!(16);
        assert_eq!(finite_gate(&old, &new)?["finite_probe_positive"], false);
        new["correct_reference_frames"] = json!(17);
        new["terms"][0]["target_mass"] = json!(m);
        assert_eq!(finite_gate(&old, &new)?["finite_probe_positive"], false);
        Ok(())
    }
    #[test]
    fn recorded_finite_divergence_separates_score_action_and_topology() -> Result<()> {
        use uor_r4_integer::geometric_context::{
            ContextDecisionEvent, ContextDecisionFamily, ContextInvocation,
        };
        let e = ContextDecisionEvent {
            call_index: 0,
            invocation: ContextInvocation::Step,
            family: ContextDecisionFamily::Transition,
            token_id: 7,
            head: 1,
            lane: 3,
            flat_lane: 7,
            own: 1,
            neighbor: 1,
            winner: 0,
            scores_q24: vec![2, 1],
        };
        let old = vec![e.clone(), e.clone()];
        let mut new = old.clone();
        new[0].scores_q24[1] = 2;
        new[1].winner = 1;
        let r = capture_difference(4, &old, &new);
        assert_eq!(r["first_score_difference"]["event_index"], 0);
        assert_eq!(r["first_action_or_state_difference"]["event_index"], 1);
        assert!(r["first_topology_difference"].is_null());
        new[0].token_id = 8;
        let r = capture_difference(4, &old, &new);
        assert_eq!(r["first_topology_difference"]["event_index"], 0);
        Ok(())
    }
}
