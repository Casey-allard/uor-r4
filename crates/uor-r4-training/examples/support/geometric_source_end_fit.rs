//! Source-end-only offline learner, native factual route and independent serving reload.
use super::*;
use uor_r4_integer::geometric_source_end_transport::{
    NativeSourceEndTransport, SourceEndAngularQ4, SourceEndScoreMode,
};
use uor_r4_integer::geometric_source_realizer::SourceEndBankRealizerTrace;
use uor_r4_training::geometric_occurrence_consumer::source_realizer::SourceEndAngularWeights;

fn native_load<'a>(
    root: &Path,
    parent: &'a IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    prefix: &NativePrefixTransport<'_>,
) -> Result<NativeSourceEndTransport<'a>> {
    let meta = read_json(&root.join("native-metadata.json"))?;
    let angular = SourceEndAngularQ4::new(
        serde_json::from_value(meta["potential"].clone())?,
        &fs::read(root.join("source-end-period-q4.bin"))?,
        &fs::read(root.join("source-end-stop-q4.bin"))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let end = parent.compile_source_end_transport(cue, prefix, angular)?;
    if serde_json::to_value(end.metadata())? != meta {
        return Err(invalid("source-end native metadata/payload/parent differs").into());
    }
    Ok(end)
}
fn training_prefix<'a>(
    a: &Args,
    parent: &'a NativeSourceRealizer,
    cue: &NativeCueCarrier<'_>,
) -> Result<NativePrefixTransport<'a>> {
    let root = a
        .frozen_prefix_bundle
        .as_ref()
        .ok_or_else(|| invalid("prefix absent"))?;
    let meta = read_json(&root.join("native-metadata.json"))?;
    let prefix = parent.compile_prefix_transport(
        cue,
        PrefixAngularQ4::new(
            serde_json::from_value(meta["potential"].clone())?,
            &fs::read(root.join("prefix-q4.bin"))?,
        )
        .map_err(|e| invalid(e.to_string()))?,
    )?;
    if serde_json::to_value(prefix.metadata())? != meta {
        return Err(invalid("source-end frozen training prefix differs").into());
    }
    Ok(prefix)
}
fn compact_end(out: &SourceEndBankRealizerTrace) -> Result<Value> {
    let old = &out.prefix_bank.cue_bank.bank.actions.head_scores;
    let new = &out.actions.head_scores;
    if old.len() != new.len() || old.iter().zip(new).any(|(x, y)| x.copy_q24 != y.copy_q24) {
        return Err(invalid("source-end authoritative final Copy heads changed").into());
    }
    let mut v = compact(&out.prefix_bank.cue_bank.bank)?;
    v["actions"] = serde_json::to_value(&out.actions)?;
    Ok(v)
}
fn zero_fidelity(old: &Value, new: &Value) -> Result<()> {
    let aa = old["rows"]
        .as_array()
        .ok_or_else(|| invalid("old rows absent"))?;
    let bb = new["rows"]
        .as_array()
        .ok_or_else(|| invalid("new rows absent"))?;
    if aa.len() != bb.len() {
        return Err(invalid("source-end zero row count differs").into());
    }
    for (a, b) in aa.iter().zip(bb) {
        if a["id"] != b["id"] {
            return Err(invalid("source-end zero row identity differs").into());
        }
        let x = a["tokens"]
            .as_array()
            .ok_or_else(|| invalid("old tokens absent"))?;
        let y = b["tokens"]
            .as_array()
            .ok_or_else(|| invalid("new tokens absent"))?;
        if x.len() != y.len() {
            return Err(invalid("source-end zero token count differs").into());
        }
        for (x, y) in x.iter().zip(y) {
            for k in ["native", "cue_carrier", "prefix_transport"] {
                if x[k] != y[k] {
                    return Err(invalid(format!("source-end zero {k} differs")).into());
                }
            }
        }
    }
    Ok(())
}
fn canonical(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    carrier: &NativePrefixTransport<'_>,
    end: &NativeSourceEndTransport<'_>,
    episodes: &[Episode],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut total = 0.;
    let mut zeros = Vec::new();
    let mut count = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut tokens = Vec::new();
        let mut rowce = 0.;
        let mut rowzero = false;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let out = native.read_bank_with_source_end_transport(
                &segments,
                &e.packet.query_ids,
                &e.target[..step],
                cue,
                carrier,
                end,
            )?;
            let mass = out
                .actions
                .token_masses
                .iter()
                .filter(|x| x.token_id == target)
                .map(|x| x.weight_q31)
                .sum::<u64>();
            let den = out.actions.total_weight_q31;
            if den == 0 {
                return Err(invalid("cue native zero normalizer").into());
            }
            let ce = if mass == 0 {
                rowzero = true;
                zeros.push(json!({"id":e.packet.id,"step":step}));
                None
            } else {
                let v = -(mass as f64 / den as f64).ln();
                rowce += v;
                Some(v)
            };
            tokens.push(json!({"step":step,"teacherforced_prefix_ids_labels_only":&e.target[..step],"target_label_only_after_read":target,"native_ce":ce,"target_mass_q31":mass,"total_weight_q31":den,"native":compact_end(&out)?,"cue_carrier":out.prefix_bank.cue_bank.carrier,"prefix_transport":out.prefix_bank.prefix,"source_end":out.source_end}));
            count += 1;
        }
        let mean = if rowzero {
            None
        } else {
            Some(rowce / e.target.len() as f64)
        };
        if let Some(v) = mean {
            total += v / episodes.len() as f64;
        }
        rows.push(json!({"id":e.packet.id,"native_mean_token_ce":mean,"tokens":tokens}));
    }
    Ok(
        json!({"cases":episodes.len(),"target_positions":count,"native_equal_episode_ce":if zeros.is_empty(){Some(total)}else{None},"zero_support_positions":zeros,"rows":rows,"probability_floor":false,"infinite_native_objective_when_zero":true}),
    )
}
fn generation(
    native: &IntegerRealizer,
    cue: &NativeCueCarrier<'_>,
    carrier: &NativePrefixTransport<'_>,
    end: &NativeSourceEndTransport<'_>,
    episodes: &[Episode],
    tok: &ByteBpeTokenizer,
    a: &Args,
    start: Instant,
) -> Result<Value> {
    let mut rows = Vec::new();
    let mut complete = 0;
    for e in episodes {
        let segments = e.segments()?;
        let mut ids = Vec::new();
        let mut traces = Vec::new();
        for step in 0..a.maximum_generation_tokens {
            deadline(a, start)?;
            let out = native.read_bank_with_source_end_transport(
                &segments,
                &e.packet.query_ids,
                &ids,
                cue,
                carrier,
                end,
            )?;
            let chosen = out.actions.chosen_token_id;
            traces.push(json!({"step":step,"actual_prefix_ids":ids,"native":compact_end(&out)?,"cue_carrier":out.prefix_bank.cue_bank.carrier,"prefix_transport":out.prefix_bank.prefix,"source_end":out.source_end}));
            ids.push(chosen);
            if chosen == native.binding().eos_token_id() {
                break;
            }
        }
        let eos = ids.last() == Some(&native.binding().eos_token_id());
        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
        let bytes = tok.decode_bytes(plain);
        let raw = String::from_utf8_lossy(&bytes);
        let text = raw.strip_prefix(' ').unwrap_or(&raw);
        let accepted = eos && String::from_utf8(bytes.clone()).is_ok() && e.answers.accepts(text);
        complete += usize::from(accepted);
        rows.push(json!({"id":e.packet.id,"generated_ids_including_eos":ids,"eos":eos,"reply_text":text,"raw_decoded_bytes_hex":hex::encode(bytes),"accepted_complete_answer":accepted,"tokens":traces}));
    }
    Ok(
        json!({"cases":episodes.len(),"accepted_complete":complete,"maximum_generated_tokens":a.maximum_generation_tokens,"canonical_prefixes_used":false,"native_only_source_end_load":true,"rows":rows}),
    )
}
fn batch(
    indices: &[usize],
    episodes: &[Episode],
    source: &SourceRealizerWeights,
    parent: &NativeSourceRealizer,
    weights: &SourceEndAngularWeights,
    integer: Option<&IntegerRealizer>,
    a: &Args,
    start: Instant,
) -> Result<Batch> {
    let began = Instant::now();
    let prepared = source.prepare(parent)?;
    let cr = a
        .frozen_cue_bundle
        .as_ref()
        .ok_or_else(|| invalid("cue absent"))?;
    let cue = prefix_training_cue_load(cr, parent)?;
    let prefix = training_prefix(a, parent, &cue)?;
    let end = parent.compile_source_end_transport(&cue, &prefix, weights.native()?)?;
    let ic = integer.map(|p| cue_native_load(cr, p)).transpose()?;
    let ip = integer
        .map(|p| {
            prefix_native_load(
                a.frozen_prefix_bundle
                    .as_ref()
                    .ok_or_else(|| invalid("prefix absent"))?,
                p,
                ic.as_ref()
                    .ok_or_else(|| invalid("independent cue absent"))?,
            )
        })
        .transpose()?;
    let ie = integer
        .map(|p| {
            p.compile_source_end_transport(
                ic.as_ref()
                    .ok_or_else(|| invalid("independent cue absent"))?,
                ip.as_ref()
                    .ok_or_else(|| invalid("independent prefix absent"))?,
                weights.native()?,
            )
            .map_err(|e| Box::<dyn std::error::Error>::from(e))
        })
        .transpose()?;
    let params = weights.parameters();
    let parentvars = source.parameters();
    let mut gradients = BTreeMap::<String, Tensor>::new();
    let mut rows = Vec::new();
    let mut positions = 0;
    let mut mean = 0.;
    let mut visited =
        vec![BTreeSet::<u8>::new(); weights.config().heads * weights.config().lanes_per_head];
    let mut no_route = 0;
    for &index in indices {
        let e = episodes
            .get(index)
            .ok_or_else(|| invalid("batch index out of panel"))?;
        let seg = e.segments()?;
        let mut ce = 0.;
        for (step, &target) in e.target.iter().enumerate() {
            deadline(a, start)?;
            let result = prepared.loss_bank_source_end(
                &seg,
                &e.packet.query_ids,
                &e.target[..step],
                target,
                weights,
                &cue,
                &prefix,
                &end,
            );
            let out = match result {
                Ok(v) => v,
                Err(err) => {
                    let factual = parent.read_bank_with_source_end_transport(
                        &seg,
                        &e.packet.query_ids,
                        &e.target[..step],
                        &cue,
                        &prefix,
                        &end,
                    )?;
                    write_json(
                        &a.out,
                        "failed-prefix.json",
                        &json!({"error":err.to_string(),"id":e.packet.id,"step":step,"target_label_only_after_read":target,"canonical_prefix_ids_labels_only":&e.target[..step],"current_actual_trace":factual}),
                    )?;
                    return Err(err.into());
                }
            };
            if let (Some(p), Some(c), Some(pr), Some(en)) =
                (integer, ic.as_ref(), ip.as_ref(), ie.as_ref())
            {
                let independent = p.read_bank_with_source_end_transport(
                    &seg,
                    &e.packet.query_ids,
                    &e.target[..step],
                    c,
                    pr,
                    en,
                )?;
                if out.trace != independent {
                    return Err(invalid("source-end loss independent native trace differs").into());
                }
            }
            let old = parent.read_bank_with_prefix_transport(
                &seg,
                &e.packet.query_ids,
                &e.target[..step],
                &cue,
                &prefix,
            )?;
            if old != out.trace.prefix_bank {
                return Err(invalid("source-end modified frozen Copy/replay/carriers").into());
            }
            if out.trace.source_end.selected_source_index.is_none() {
                no_route += 1;
            } else {
                if out.trace.source_end.angular_indices.len() != visited.len() {
                    return Err(invalid("source-end angular width differs").into());
                }
                for (lane, &bin) in out.trace.source_end.angular_indices.iter().enumerate() {
                    visited[lane].insert(bin);
                }
            }
            let expected = -out.target_probability.ln();
            let scalar = f64::from(out.loss.to_scalar::<f32>()?);
            if !scalar.is_finite() || (scalar - expected).abs() > 1e-4 + 1e-5 * expected.abs() {
                return Err(invalid("source-end native hard likelihood differs").into());
            }
            ce += expected;
            positions += 1;
            let store = (&out.loss * scale(indices.len(), e.target.len())?)?.backward()?;
            if parentvars
                .values()
                .any(|v| store.get(v.as_tensor()).is_some())
            {
                return Err(invalid("source-end connects frozen parent variable").into());
            }
            for (name, var) in &params {
                if let Some(g) = store.get(var.as_tensor()) {
                    let g = g.detach();
                    let next = if let Some(old) = gradients.remove(name) {
                        (&old + &g)?.detach()
                    } else {
                        g
                    };
                    gradients.insert(name.clone(), next);
                }
            }
        }
        mean += ce / e.target.len() as f64 / indices.len() as f64;
        rows.push(json!({"id":e.packet.id,"target_steps":e.target.len(),"native_mean_token_ce":ce/e.target.len() as f64}));
    }
    let mut stats = BTreeMap::new();
    let mut sq = 0.;
    for (name, g) in &gradients {
        let v = g.flatten_all()?.to_vec1::<f32>()?;
        if v.iter().any(|x| !x.is_finite()) {
            return Err(invalid("source-end nonfinite gradient").into());
        }
        sq += v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
        stats.insert(name.clone(),json!({"elements":v.len(),"finite":true,"nonzero":v.iter().filter(|x|**x!=0.).count(),"l1":v.iter().map(|x|f64::from(x.abs())).sum::<f64>()}));
    }
    if gradients.len() != params.len() || (integer.is_some() && sq <= 0.) || !sq.is_finite() {
        return Err(invalid("source-end full coefficient gradient absent/zero/nonfinite").into());
    }
    Ok(Batch {
        gradients,
        report: json!({"episodes":indices.len(),"episode_indices":indices,"target_positions":positions,"native_equal_episode_ce":mean,"gradient_global_norm":sq.sqrt(),"gradient_families":stats,"rows":rows,"angular_visited_bins":visited,"no_route_positions":no_route,"parent_gradient_graph_absent":true,"independent_native_trace_parity":integer.is_some(),"fixed_input_Copy_unchanged":true,"source_end_score_mode":weights.config().mode,"active_families":SOURCE_END_FAMILIES,"elapsed_seconds":began.elapsed().as_secs_f64(),"objective":"equalepisode fullanswer+EOS native alias CE; no support floors","route_adjoint":"frozen factual rawCopy argmax; new endpoint tables only"}),
    })
}
fn apply(
    weights: &SourceEndAngularWeights,
    opt: &mut AdamW,
    grad: BTreeMap<String, Tensor>,
) -> Result<f64> {
    let mut sq = 0.;
    for g in grad.values() {
        for x in g.flatten_all()?.to_vec1::<f32>()? {
            if !x.is_finite() {
                return Err(invalid("source-end clip nonfinite").into());
            }
            sq += f64::from(x).powi(2);
        }
    }
    let norm = sq.sqrt();
    if !norm.is_finite() {
        return Err(invalid("source-end norm nonfinite").into());
    }
    let clip = if norm > 1. { 1. / norm } else { 1. };
    let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
    let params = weights.parameters();
    for (name, g) in grad {
        let v = params
            .get(&name)
            .ok_or_else(|| invalid("unexpected source-end gradient family"))?;
        store.insert(v.as_tensor(), (&g * clip)?.detach());
    }
    opt.step(&store)?;
    weights.project_shadow_range()?;
    Ok(clip)
}
fn recovery(weights: &SourceEndAngularWeights, a: &Args, update: usize) -> Result<()> {
    let receipts = parameter_receipts(&weights.parameters())?;
    for (name, var) in weights.parameters() {
        let raw = var
            .flatten_all()?
            .to_vec1::<f32>()?
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let file = if name.contains("period") {
            "current-source-end-period-f32.bin"
        } else {
            "current-source-end-stop-f32.bin"
        };
        if directory_bytes(&a.out)?.saturating_add(raw.len()) > a.maximum_report_bytes {
            return Err(invalid("source-end recovery cap").into());
        }
        fs::write(a.out.join(file), raw)?;
    }
    write_json(
        &a.out,
        "current-source-end-recovery.json",
        &json!({"optimizer_updates":update,"fractional_source_receipts":receipts,"period_packed_sha256":sha256_bytes(&weights.period_packed_coefficients()?),"stop_packed_sha256":sha256_bytes(&weights.stop_packed_coefficients()?)}),
    )
}
fn checkpoint(
    weights: &SourceEndAngularWeights,
    parent: &NativeSourceRealizer,
    integer: &IntegerRealizer,
    episodes: &[Episode],
    step: usize,
    prior: &[Value],
    a: &Args,
    start: Instant,
) -> Result<Value> {
    if directory_bytes(&a.out)?.saturating_add(80 * 1024 * 1024) > a.maximum_report_bytes {
        return Err(invalid("source-end checkpoint reserve cap").into());
    }
    let root = a.out.join(format!("checkpoint-{step:04}"));
    report_output::claim(&root)?;
    let result = (|| -> Result<Value> {
        let cr = a
            .frozen_cue_bundle
            .as_ref()
            .ok_or_else(|| invalid("cue absent"))?;
        let pr = a
            .frozen_prefix_bundle
            .as_ref()
            .ok_or_else(|| invalid("prefix absent"))?;
        weights.save(&root.join("source-end"))?;
        let tc = prefix_training_cue_load(cr, parent)?;
        let tp = training_prefix(a, parent, &tc)?;
        let restored = SourceEndAngularWeights::load(
            &root.join("source-end"),
            parent,
            &a.native_artifact,
            cr,
            pr,
            &tc,
            &tp,
        )?;
        if parameter_receipts(&restored.parameters())? != parameter_receipts(&weights.parameters())?
        {
            return Err(invalid("source-end saved shadows differ").into());
        }
        let cue = cue_native_load(cr, integer)?;
        let prefix = prefix_native_load(pr, integer, &cue)?;
        let end = native_load(&root.join("source-end"), integer, &cue, &prefix)?;
        let c = canonical(integer, &cue, &prefix, &end, episodes, a, start)?;
        write_json(&root, "canonical.json", &c)?;
        let mut comparisons = Vec::new();
        for prior in prior {
            let step = prior["updates"]
                .as_u64()
                .ok_or_else(|| invalid("prior step absent"))?;
            let old = read_json(&a.out.join(format!("checkpoint-{step:04}/canonical.json")))?;
            comparisons.push(json!({"prior_updates":step,"comparison":compare(&old,&c)?}));
        }
        write_json(&root, "comparisons.json", &json!(comparisons))?;
        let rec = json!({"updates":step,"native_equal_episode_ce":c["native_equal_episode_ce"],"zero_support_positions":c["zero_support_positions"],"source_end_source_parameter_receipts":parameter_receipts(&weights.parameters())?,"source_end_native_metadata_sha256":sha256_file(&root.join("source-end/native-metadata.json"))?,"period_packed_sha256":sha256_file(&root.join("source-end/source-end-period-q4.bin"))?,"stop_packed_sha256":sha256_file(&root.join("source-end/source-end-stop-q4.bin"))?,"canonical_sha256":sha256_file(&root.join("canonical.json"))?,"frozen_parent_native_binding":integer.artifact_binding(),"fixed_input_Copy_unchanged":true,"native_only_generation_contract":true});
        write_json(&root, "receipt.json", &rec)?;
        Ok(rec)
    })();
    if let Err(e) = &result {
        write_json(
            &root,
            "failure.json",
            &json!({"error":e.to_string(),"updates":step}),
        )?;
    }
    report_output::seal(&root)?;
    report_output::verify(&root)?;
    result
}
fn project_storage(canonical: &Value, generation: &Value, a: &Args) -> Result<Value> {
    let mut max_token = 0usize;
    for row in generation["rows"]
        .as_array()
        .ok_or_else(|| invalid("storage generation rows absent"))?
    {
        for token in row["tokens"]
            .as_array()
            .ok_or_else(|| invalid("storage tokens absent"))?
        {
            max_token = max_token.max(serde_json::to_vec(token)?.len());
        }
    }
    // Initial canonical plus all five checkpoints; two development, two fresh,
    // two optional six-case controls at the actual full generation cap.
    let canonical_bytes = serde_json::to_vec(canonical)?.len();
    let rows = 2 * 128
        + 2 * 32
        + if a.exposed_controls.is_some() {
            2 * 6
        } else {
            0
        };
    let tokens = rows * a.maximum_generation_tokens;
    let projected = canonical_bytes
        .saturating_mul(6)
        .saturating_add(tokens.saturating_mul(max_token.saturating_add(4096)))
        .saturating_add(16 * 1024 * 1024);
    Ok(
        json!({"canonical_copies":6,"observed_canonical_bytes":canonical_bytes,"observed_max_compact_token_bytes":max_token,"token_growth_reserve_bytes":4096,"generation_full_cap_tokens":tokens,"administration_reserve_bytes":16*1024*1024,"observed_shape_full_cap_bytes":projected,"configured_report_cap_bytes":a.maximum_report_bytes,"scope":"observed-shape conservative projection, not a formal bound; actual cap is authoritative"}),
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EndAuthorization {
    schema: String,
    fit_admitted: bool,
    parent_frozen: bool,
    active_families: String,
    source_end_score_mode: SourceEndScoreMode,
    admission_report_sha256: String,
    development_manifest_sha256: String,
    fresh_manifest_sha256: String,
    trusted_binding_sha256: String,
    frozen_cue_native_metadata_sha256: String,
    frozen_cue_packed_sha256: String,
    frozen_prefix_native_metadata_sha256: String,
    frozen_prefix_packed_sha256: String,
    zero_source_end_native_metadata_sha256: String,
    updates: usize,
    batch_episodes: usize,
    maximum_fit_seconds: u64,
}
fn packed_hashes(w: &SourceEndAngularWeights) -> Result<Vec<String>> {
    Ok(vec![
        sha256_bytes(&w.period_packed_coefficients()?),
        sha256_bytes(&w.stop_packed_coefficients()?),
    ])
}
fn frozen(a: &Args, source: &SourceRealizerWeights, receipts: &Value) -> Result<()> {
    if parameter_receipts(&source.parameters())? != *receipts {
        return Err(invalid("source-end frozen source parameters changed").into());
    }
    for (root, name, expected) in [
        (
            a.frozen_cue_bundle.as_ref(),
            "native-metadata.json",
            &a.frozen_cue_native_metadata_sha256,
        ),
        (
            a.frozen_cue_bundle.as_ref(),
            "cue-q4.bin",
            &a.frozen_cue_packed_sha256,
        ),
        (
            a.frozen_prefix_bundle.as_ref(),
            "native-metadata.json",
            &a.frozen_prefix_native_metadata_sha256,
        ),
        (
            a.frozen_prefix_bundle.as_ref(),
            "prefix-q4.bin",
            &a.frozen_prefix_packed_sha256,
        ),
    ] {
        let root = root.ok_or_else(|| invalid("frozen sidecar root absent"))?;
        if Some(sha256_file(&root.join(name))?) != *expected {
            return Err(invalid("source-end frozen sidecar changed").into());
        }
    }
    Ok(())
}
fn immutable(inputs: &BTreeMap<String, String>, sealed: &BTreeSet<PathBuf>) -> Result<()> {
    for (p, h) in inputs {
        if sha256_file(Path::new(p))? != *h {
            return Err(invalid("source-end immutable input changed").into());
        }
    }
    for p in sealed {
        report_output::verify(p)?;
    }
    Ok(())
}
fn admission_matches(
    r: &Value,
    a: &Args,
    mode: SourceEndScoreMode,
    trusted: &str,
    zero: &str,
    receipts: &Value,
) -> bool {
    r["schema"] == "uor-r4.geometric-source-end-fit/1"
        && r["mode"] == "source-end-broadbatch"
        && r["status"] == "completed"
        && r["optimizer_updates"] == 0
        && r["cases"] == 128
        && r["source_end_score_mode"] == json!(mode)
        && r["parent_frozen"] == true
        && r["active_families"] == SOURCE_END_FAMILIES
        && r["complete_objective_finite"] == true
        && r["gradient_report"]["parent_gradient_graph_absent"] == true
        && r["gradient_report"]["independent_native_trace_parity"] == true
        && r["gradient_report"]["fixed_input_Copy_unchanged"] == true
        && r["development_manifest_sha256"] == a.development_manifest_sha256
        && r["fresh_manifest_sha256"] == a.fresh_manifest_sha256
        && r["trusted_binding_sha256"] == trusted
        && r["zero_source_end_native_metadata_sha256"] == zero
        && r["frozen_parent_source_receipts"] == *receipts
        && r["frozen_cue_native_metadata_sha256"] == json!(a.frozen_cue_native_metadata_sha256)
        && r["frozen_cue_packed_sha256"] == json!(a.frozen_cue_packed_sha256)
        && r["frozen_prefix_native_metadata_sha256"]
            == json!(a.frozen_prefix_native_metadata_sha256)
        && r["frozen_prefix_packed_sha256"] == json!(a.frozen_prefix_packed_sha256)
}
pub(super) fn run(a: &Args, start: Instant) -> Result<Value> {
    let cr = a
        .frozen_cue_bundle
        .as_ref()
        .ok_or_else(|| invalid("source-end cue absent"))?;
    let pr = a
        .frozen_prefix_bundle
        .as_ref()
        .ok_or_else(|| invalid("source-end prefix absent"))?;
    let mode = a
        .source_end_score_mode
        .ok_or_else(|| invalid("source-end mode absent"))?;
    let mut inputs = BTreeMap::new();
    let mut sealed = BTreeSet::new();
    for (root, expected) in [
        (&a.development_panel, &a.development_manifest_sha256),
        (&a.fresh_panel, &a.fresh_manifest_sha256),
    ] {
        report_output::verify(root)?;
        if sha256_file(&root.join("manifest.json"))? != *expected {
            return Err(invalid("source-end panel manifest differs").into());
        }
    }
    for p in [
        cr,
        pr,
        &a.source_weights,
        &a.native_artifact,
        &a.development_panel,
        &a.fresh_panel,
    ] {
        let root = nearest_seal(p)?;
        report_output::verify(&root)?;
        inputs.insert(
            root.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&root.join("manifest.json"))?,
        );
        sealed.insert(root);
    }
    if let Some(p) = &a.exposed_controls {
        let root = nearest_seal(p)?;
        report_output::verify(&root)?;
        inputs.insert(
            root.join("manifest.json").to_string_lossy().into_owned(),
            sha256_file(&root.join("manifest.json"))?,
        );
        sealed.insert(root);
    }
    for (p, n) in [
        (cr, "native-metadata.json"),
        (cr, "cue-q4.bin"),
        (pr, "native-metadata.json"),
        (pr, "prefix-q4.bin"),
    ] {
        inputs.insert(
            p.join(n).to_string_lossy().into_owned(),
            sha256_file(&p.join(n))?,
        );
    }
    let binding: NativeArtifactBinding =
        serde_json::from_slice(&read_capped(&a.trusted_native_binding)?)?;
    let trusted_sha = sha256_file(&a.trusted_native_binding)?;
    inputs.insert(
        a.trusted_native_binding.to_string_lossy().into_owned(),
        trusted_sha.clone(),
    );
    let integer = IntegerRealizer::load_native(&a.native_artifact, &binding)?;
    let bytes = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("source-end ByteBPE absent"))?;
    let source = SourceRealizerWeights::load_source(&a.source_weights, &bytes)?;
    let identity: ConsumerIdentity = serde_json::from_value(
        read_json(&a.native_artifact.join("metadata.json"))?["identity"].clone(),
    )?;
    let parent = NativeSourceRealizer::load(&a.native_artifact, &source, &identity)?;
    if parent.artifact_binding()? != binding {
        return Err(invalid("source-end source/native parent differs").into());
    }
    let receipts = parameter_receipts(&source.parameters())?;
    frozen(a, &source, &receipts)?;
    let tc = prefix_training_cue_load(cr, &parent)?;
    let tp = training_prefix(a, &parent, &tc)?;
    let cue = cue_native_load(cr, &integer)?;
    let prefix = prefix_native_load(pr, &integer, &cue)?;
    let weights =
        SourceEndAngularWeights::zero(&parent, &a.native_artifact, cr, pr, &tc, &tp, mode)?;
    let development = load_panel(&a.development_panel, 128, &integer, &tok, true)?;
    let fresh = load_panel(&a.fresh_panel, 32, &integer, &tok, true)?;
    write_json(
        &a.out,
        "fresh-preparation-scope.json",
        &read_json(&a.fresh_panel.join("report.json"))?,
    )?;
    weights.save(&a.out.join("initial-source-end"))?;
    recovery(&weights, a, 0)?;
    let initial_sha = sha256_file(&a.out.join("initial-source-end/native-metadata.json"))?;
    let end = native_load(&a.out.join("initial-source-end"), &integer, &cue, &prefix)?;
    let baseline = canonical(&integer, &cue, &prefix, &end, &development, a, start)?;
    if baseline["native_equal_episode_ce"].is_null() {
        return Err(invalid("source-end complete objective infinite; no support filter").into());
    }
    write_json(&a.out, "initial-canonical.json", &baseline)?;
    let old = prefix_canonical(&integer, &cue, &prefix, &development, a, start)?;
    zero_fidelity(&old, &baseline)?;
    let parent_gen = generation(&integer, &cue, &prefix, &end, &development, &tok, a, start)?;
    let oldgen = prefix_generation(&integer, &cue, &prefix, &development, &tok, a, start)?;
    zero_fidelity(&oldgen, &parent_gen)?;
    for (a, b) in oldgen["rows"]
        .as_array()
        .ok_or_else(|| invalid("old generation rows absent"))?
        .iter()
        .zip(
            parent_gen["rows"]
                .as_array()
                .ok_or_else(|| invalid("end generation rows absent"))?,
        )
    {
        for key in [
            "generated_ids_including_eos",
            "eos",
            "reply_text",
            "raw_decoded_bytes_hex",
            "accepted_complete_answer",
        ] {
            if a[key] != b[key] {
                return Err(invalid("source-end zero generation output differs").into());
            }
        }
    }
    let storage_projection = project_storage(&baseline, &parent_gen, a)?;
    write_json(&a.out, "storage-projection.json", &storage_projection)?;
    write_json(&a.out, "development-parent-generation.json", &parent_gen)?;
    write_json(
        &a.out,
        "parent-query-pair-diagnostics.json",
        &factor_pair_report(&a.development_panel, &baseline, &parent_gen)?,
    )?;
    if a.mode == "source-end-broadbatch" {
        let b = batch(
            &(0..128).collect::<Vec<_>>(),
            &development,
            &source,
            &parent,
            &weights,
            Some(&integer),
            a,
            start,
        )?;
        write_json(&a.out, "broadbatch.json", &b.report)?;
        frozen(a, &source, &receipts)?;
        immutable(&inputs, &sealed)?;
        return Ok(
            json!({"schema":"uor-r4.geometric-source-end-fit/1","mode":a.mode,"status":"completed","optimizer_updates":0,"cases":128,"source_end_score_mode":mode,"parent_frozen":true,"active_families":SOURCE_END_FAMILIES,"complete_objective_finite":true,"native_equal_episode_ce":baseline["native_equal_episode_ce"],"gradient_report":b.report,"zero_source_end_native_metadata_sha256":initial_sha,"development_manifest_sha256":a.development_manifest_sha256,"fresh_manifest_sha256":a.fresh_manifest_sha256,"trusted_binding_sha256":trusted_sha,"frozen_parent_source_receipts":receipts,"frozen_cue_native_metadata_sha256":a.frozen_cue_native_metadata_sha256,"frozen_cue_packed_sha256":a.frozen_cue_packed_sha256,"frozen_prefix_native_metadata_sha256":a.frozen_prefix_native_metadata_sha256,"frozen_prefix_packed_sha256":a.frozen_prefix_packed_sha256,"input_manifests_sha256":inputs,"storage_projection":storage_projection,"fit_admitted":false,"fresh_predictions":"NOT_RUN","elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib()}),
        );
    }
    if a.mode != "source-end-fit" {
        return Err(invalid("source-end mode unknown").into());
    }
    let admission = a
        .admission
        .as_ref()
        .ok_or_else(|| invalid("source-end admission absent"))?;
    report_output::verify(admission)?;
    if Some(sha256_file(&admission.join("manifest.json"))?) != a.admission_manifest_sha256 {
        return Err(invalid("source-end admission manifest differs").into());
    }
    let report = read_json(&admission.join("report.json"))?;
    let authpath = a
        .fit_authorization
        .as_ref()
        .ok_or_else(|| invalid("source-end authorization absent"))?;
    let auth: EndAuthorization = serde_json::from_slice(&read_capped(authpath)?)?;
    if !admission_matches(&report, a, mode, &trusted_sha, &initial_sha, &receipts)
        || auth.schema != "uor-r4.source-end-fit-authorization/1"
        || !auth.fit_admitted
        || !auth.parent_frozen
        || auth.active_families != SOURCE_END_FAMILIES
        || auth.source_end_score_mode != mode
        || auth.admission_report_sha256 != sha256_file(&admission.join("report.json"))?
        || auth.development_manifest_sha256 != a.development_manifest_sha256
        || auth.fresh_manifest_sha256 != a.fresh_manifest_sha256
        || auth.trusted_binding_sha256 != trusted_sha
        || auth.zero_source_end_native_metadata_sha256 != initial_sha
        || Some(auth.frozen_cue_native_metadata_sha256) != a.frozen_cue_native_metadata_sha256
        || Some(auth.frozen_cue_packed_sha256) != a.frozen_cue_packed_sha256
        || Some(auth.frozen_prefix_native_metadata_sha256) != a.frozen_prefix_native_metadata_sha256
        || Some(auth.frozen_prefix_packed_sha256) != a.frozen_prefix_packed_sha256
        || auth.updates != UPDATES
        || auth.batch_episodes != BATCH
        || auth.maximum_fit_seconds != a.maximum_seconds
    {
        return Err(invalid(
            "source-end distinct mode/parent/carrier/data/zero-artifact admission differs",
        )
        .into());
    }
    inputs.insert(
        admission
            .join("manifest.json")
            .to_string_lossy()
            .into_owned(),
        sha256_file(&admission.join("manifest.json"))?,
    );
    sealed.insert(admission.clone());
    inputs.insert(
        authpath.to_string_lossy().into_owned(),
        sha256_file(authpath)?,
    );
    let mut stages = Vec::new();
    stages.push(checkpoint(
        &weights,
        &parent,
        &integer,
        &development,
        0,
        &[],
        a,
        start,
    )?);
    if baseline != read_json(&a.out.join("checkpoint-0000/canonical.json"))? {
        return Err(invalid("source-end independent zero checkpoint differs").into());
    }
    let mut optimizer = AdamW::new(
        weights.parameters().into_values().collect(),
        ParamsAdamW {
            lr: 0.003,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            weight_decay: 0.,
        },
    )?;
    let initial_packed = packed_hashes(&weights)?;
    let mut first_cross = None;
    let mut batches = Vec::new();
    for update in 0..UPDATES {
        deadline(a, start)?;
        frozen(a, &source, &receipts)?;
        let b = batch(
            &balanced_indices(update),
            &development,
            &source,
            &parent,
            &weights,
            None,
            a,
            start,
        )?;
        if update == 0 {
            let measured = b.report["elapsed_seconds"]
                .as_f64()
                .ok_or_else(|| invalid("source-end first batch timing absent"))?;
            let remaining = measured * UPDATES as f64 + 240.;
            let projection = storage_projection["observed_shape_full_cap_bytes"]
                .as_u64()
                .ok_or_else(|| invalid("source-end storage projection absent"))?
                as usize;
            write_json(
                &a.out,
                "first-batch-projection.json",
                &json!({"optimizer_updates":0,"measured_batch_seconds":measured,"projected_remaining_fit_seconds":remaining,"storage_projection_bytes":projection}),
            )?;
            if start.elapsed().as_secs_f64() + remaining > a.maximum_seconds as f64
                || projection + 1024 * 1024 > a.maximum_report_bytes
            {
                return Err(invalid(
                    "source-end measured first batch resource projection refuses optimizer1",
                )
                .into());
            }
        }
        let clip = apply(&weights, &mut optimizer, b.gradients)?;
        recovery(&weights, a, update + 1)?;
        let packed = packed_hashes(&weights)?;
        if first_cross.is_none() && packed != initial_packed {
            first_cross = Some(update + 1);
        }
        batches.push(json!({"update":update+1,"clip_factor":clip,"source_end_packed_sha256":packed,"batch":b.report}));
        write_json(
            &a.out,
            "progress.json",
            &json!({"optimizer_updates":update+1,"first_native_packed_crossing_update":first_cross,"batches":batches,"elapsed_seconds":start.elapsed().as_secs_f64()}),
        )?;
        frozen(a, &source, &receipts)?;
        if (update + 1) % 16 == 0 {
            stages.push(checkpoint(
                &weights,
                &parent,
                &integer,
                &development,
                update + 1,
                &stages,
                a,
                start,
            )?);
        }
    }
    let selected = select(&stages)?;
    let step = stages[selected]["updates"]
        .as_u64()
        .ok_or_else(|| invalid("source-end selected step absent"))?;
    write_json(
        &a.out,
        "selection-before-fresh.json",
        &json!({"selected_updates":step,"criterion":"full128 native equalepisodeCE includesparent0 earliestties","fresh_predictions_before_selection":0}),
    )?;
    let selected_end = native_load(
        &a.out.join(format!("checkpoint-{step:04}/source-end")),
        &integer,
        &cue,
        &prefix,
    )?;
    let selected_can = read_json(&a.out.join(format!("checkpoint-{step:04}/canonical.json")))?;
    let selected_gen = generation(
        &integer,
        &cue,
        &prefix,
        &selected_end,
        &development,
        &tok,
        a,
        start,
    )?;
    write_json(
        &a.out,
        "development-selected-generation.json",
        &selected_gen,
    )?;
    write_json(
        &a.out,
        "selected-query-pair-diagnostics.json",
        &factor_pair_report(&a.development_panel, &selected_can, &selected_gen)?,
    )?;
    write_json(
        &a.out,
        "fresh-parent-generation.json",
        &generation(&integer, &cue, &prefix, &end, &fresh, &tok, a, start)?,
    )?;
    write_json(
        &a.out,
        "fresh-selected-generation.json",
        &generation(
            &integer,
            &cue,
            &prefix,
            &selected_end,
            &fresh,
            &tok,
            a,
            start,
        )?,
    )?;
    if let Some(root) = &a.exposed_controls {
        let controls = load_panel(root, 6, &integer, &tok, false)?;
        write_json(
            &a.out,
            "controls-parent-generation.json",
            &generation(&integer, &cue, &prefix, &end, &controls, &tok, a, start)?,
        )?;
        write_json(
            &a.out,
            "controls-selected-generation.json",
            &generation(
                &integer,
                &cue,
                &prefix,
                &selected_end,
                &controls,
                &tok,
                a,
                start,
            )?,
        )?;
    }
    frozen(a, &source, &receipts)?;
    immutable(&inputs, &sealed)?;
    Ok(
        json!({"schema":"uor-r4.geometric-source-end-fit/1","mode":a.mode,"status":"completed","source_end_score_mode":mode,"parent_frozen":true,"active_families":SOURCE_END_FAMILIES,"optimizer_updates":UPDATES,"batch_episodes":BATCH,"episode_visits":4,"batch_schedule":"balanced4single+4bank intactpairs","checkpoints":stages,"selected_updates":step,"first_native_packed_crossing_update":first_cross,"frozen_parent_source_receipts":receipts,"frozen_cue_native_metadata_sha256":a.frozen_cue_native_metadata_sha256,"frozen_cue_packed_sha256":a.frozen_cue_packed_sha256,"frozen_prefix_native_metadata_sha256":a.frozen_prefix_native_metadata_sha256,"frozen_prefix_packed_sha256":a.frozen_prefix_packed_sha256,"input_manifests_sha256":inputs,"fresh_predictions_before_selection":0,"native_only_source_end_generation":true,"elapsed_seconds":start.elapsed().as_secs_f64(),"peak_rss_kib_linux":peak_rss_kib(),"scope":"factual-reader-bound Source-end relation; actualprefix; fixedCopy and terminalparent; no gold/source gate/cursor; no generalchat qualification"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_end_harness_zero_fidelity_rejects_truncation_and_final_action_change() -> Result<()> {
        let row = json!({"id":"r","tokens":[{"native":{"actions":{"chosen_token_id":7,"total_weight_q31":20}},"cue_carrier":{"bin":2},"prefix_transport":{"bin":3}}]});
        let old = json!({"rows":[row.clone(),row.clone()]});
        let same = old.clone();
        zero_fidelity(&old, &same)?;
        assert!(zero_fidelity(&old, &json!({"rows":[row]})).is_err());
        let mut changed = same.clone();
        changed["rows"][1]["tokens"][0]["native"]["actions"]["chosen_token_id"] = json!(1);
        assert!(zero_fidelity(&old, &changed).is_err());
        let mut changed = same;
        changed["rows"][1]["tokens"][0]["prefix_transport"]["bin"] = json!(4);
        assert!(zero_fidelity(&old, &changed).is_err());
        Ok(())
    }
    #[test]
    fn source_end_harness_projection_counts_initial_and_five_exports_and_full_generation_cap(
    ) -> Result<()> {
        let a: Args = serde_json::from_value(
            json!({"mode":"source-end-fit","source_weights":"s","native_artifact":"n","trusted_native_binding":"b","development_panel":"d","development_manifest_sha256":"x","fresh_panel":"f","fresh_manifest_sha256":"y","out":"o","maximum_seconds":900,"maximum_context_tokens":128,"maximum_generation_tokens":32,"maximum_report_bytes":536870912,"exposed_controls":"c"}),
        )?;
        let c = json!({"rows":[]});
        let g = json!({"rows":[{"tokens":[{"actual_prefix_ids":[7,8],"source_end":{"sources":[1,2]}}]}]});
        let p = project_storage(&c, &g, &a)?;
        assert_eq!(p["canonical_copies"], 6);
        assert_eq!(p["generation_full_cap_tokens"], 10624);
        let bytes = p["observed_max_compact_token_bytes"]
            .as_u64()
            .ok_or_else(|| invalid("projection token size absent"))?;
        assert_eq!(
            p["observed_shape_full_cap_bytes"],
            json!(
                6 * serde_json::to_vec(&c)?.len() as u64
                    + 10624 * (bytes + 4096)
                    + 16 * 1024 * 1024
            )
        );
        Ok(())
    }
}
