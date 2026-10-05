//! DRAFT ONLY: install beside geometric-bank-fit.rs after source/API review.
//! Dedicated natural-cue/query root-observation learning; no execution admission here.
//! Rust source-only banks, hard native global alias CE, three existing root Vars.
//! Legacy main/run are never invoked. Reuse validated packets, seals, objective
//! scaling, deterministic B8 schedule, native bundle loaders and codecs.
//! DRAFT PREREQUISITE: narrow pub(super) visibility in bank helper module (root owned):
//! Args and accessed fields; Episode{packet,target,answers}/segments; Packet{id,query_ids};
//! Batch{gradients,report}; invalid/read_json/read_capped/parameter_receipts/peak_rss_kib/
//! write_json/write_json_limited/nearest_seal/deadline/executable/directory_bytes/scale/load_panel/
//! cue_native_load/prefix_native_load. Existing legacy main/run stay private.
//! No visibility prerequisite has been applied to production source in this draft.
//! CLI (after integration only): geometric-native-bank-observation CONFIG.json.
//! Config: schema=uor-r4.native-bank-observation-args/1; bank={existing strict Args,
//! mode:observation-broadbatch|observation-fit; maxcontext128/maxgen32; broad<=300s,
//! 64MiB; fit<=1200s,512MiB}; frozen_end_bundle and exact metadata/period/stop SHA;
//! data_scope explicitly selects retained original cues or supported current-role authored cues.
//! Existing cue/prefix SHA fields required. Development128, sealed fresh32 REQUIRED
//! before fit; this draft never draws data and does not inspect fresh until selection.
//! Each panel report must bind the matching original/authored cue policy.
//! Root-owned preparer must supply original-statement provenance receipt before execution.
//! New fit authorization schema=uor-r4.native-bank-observation-fit-authorization/1:
//! fit_admitted,admission_report_sha256,development_manifest_sha256,fresh_manifest_sha256,
//! trusted_binding_sha256,frozen_sidecars_sha256 (receipt object),active_families,
//! updates64,learning_seed,schedule_sha256,batch_episodes8,maximum_fit_seconds. No automatic fit admission.
//! learning_seed is REQUIRED for fit, NULL for broad; only actual episode-order seed.
//! Zero-parent root initialization is identical across seeded schedules, explicitly labelled.
//! Broad exports0parents; fit projects5*(actual source+native bytes)+128MiB trace reserve.
//! Resource constants are PROSPECTIVE maxima, not charged or measured execution costs.
//! NOT_COMPILED: source-realizer draft APIs + private helper visibility require root review.
#[path = "geometric-bank-fit.rs"]
mod bank;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
mod reuse {
    use super::bank::*;
    use super::output_support;
    use candle_core::{Device, Tensor, Var};
    use candle_nn::{AdamW, Optimizer, ParamsAdamW};
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs, io,
        path::{Path, PathBuf},
        time::Instant,
    };
    use uor_r4_core::report_output;
    use uor_r4_integer::{
        geometric_cue_carrier::{CueAngularConfig, CueAngularQ4, NativeCueCarrier},
        geometric_prefix_transport::{NativePrefixTransport, PrefixAngularConfig, PrefixAngularQ4},
        geometric_source_realizer::{
            NativeArtifactBinding, NativeSourceRealizer as IntegerRealizer, SourceBankSegment,
            SourceEndBankRealizerTrace,
        },
    };
    use uor_r4_tokenizer::ByteBpeTokenizer;
    use uor_r4_training::{
        geometric_occurrence_consumer::source_realizer::{
            NativeSourceRealizer, SourceRealizerWeights,
        },
        sha256_bytes, sha256_file,
    };
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use uor_r4_integer::geometric_source_end_transport::{
        NativeSourceEndTransport, SourceEndAngularConfig, SourceEndAngularQ4,
    };
    const ROOT_FAMILIES: &str = "consumer.context.{token_root,self_root,neighbor_root}/1";
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ObservationArgs {
        schema: String,
        bank: Args,
        frozen_end_bundle: PathBuf,
        frozen_end_native_metadata_sha256: String,
        frozen_end_period_packed_sha256: String,
        frozen_end_stop_packed_sha256: String,
        // Full exact source-only input and sidecar manifests, not an expected source winner.
        data_scope: String,
        learning_seed: Option<u64>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ObservationAuthorization {
        schema: String,
        fit_admitted: bool,
        admission_report_sha256: String,
        development_manifest_sha256: String,
        fresh_manifest_sha256: String,
        trusted_binding_sha256: String,
        frozen_sidecars_sha256: Value,
        active_families: String,
        updates: usize,
        learning_seed: u64,
        schedule_sha256: String,
        batch_episodes: usize,
        maximum_fit_seconds: u64,
        calibration_report_sha256: String,
        projected_complete_fit_seconds: f64,
    }
    struct FrozenAngular {
        cue_config: CueAngularConfig,
        cue: Vec<u8>,
        prefix_config: PrefixAngularConfig,
        prefix: Vec<u8>,
        end_config: SourceEndAngularConfig,
        period: Vec<u8>,
        stop: Vec<u8>,
        receipt: Value,
    }
    impl FrozenAngular {
        fn load(a: &ObservationArgs) -> Result<Self> {
            let cr = a
                .bank
                .frozen_cue_bundle
                .as_ref()
                .ok_or_else(|| invalid("cue absent"))?;
            let pr = a
                .bank
                .frozen_prefix_bundle
                .as_ref()
                .ok_or_else(|| invalid("prefix absent"))?;
            let er = &a.frozen_end_bundle;
            for (root, name, expected) in [
                (
                    cr,
                    "native-metadata.json",
                    a.bank.frozen_cue_native_metadata_sha256.as_ref(),
                ),
                (cr, "cue-q4.bin", a.bank.frozen_cue_packed_sha256.as_ref()),
                (
                    pr,
                    "native-metadata.json",
                    a.bank.frozen_prefix_native_metadata_sha256.as_ref(),
                ),
                (
                    pr,
                    "prefix-q4.bin",
                    a.bank.frozen_prefix_packed_sha256.as_ref(),
                ),
                (
                    er,
                    "native-metadata.json",
                    Some(&a.frozen_end_native_metadata_sha256),
                ),
                (
                    er,
                    "source-end-period-q4.bin",
                    Some(&a.frozen_end_period_packed_sha256),
                ),
                (
                    er,
                    "source-end-stop-q4.bin",
                    Some(&a.frozen_end_stop_packed_sha256),
                ),
            ] {
                if Some(sha256_file(&root.join(name))?) != expected.cloned() {
                    return Err(invalid("frozen angular input SHA differs").into());
                }
            }
            Ok(Self {
                cue_config: serde_json::from_value(
                    read_json(&cr.join("native-metadata.json"))?["potential"].clone(),
                )?,
                cue: fs::read(cr.join("cue-q4.bin"))?,
                prefix_config: serde_json::from_value(
                    read_json(&pr.join("native-metadata.json"))?["potential"].clone(),
                )?,
                prefix: fs::read(pr.join("prefix-q4.bin"))?,
                end_config: serde_json::from_value(
                    read_json(&er.join("native-metadata.json"))?["potential"].clone(),
                )?,
                period: fs::read(er.join("source-end-period-q4.bin"))?,
                stop: fs::read(er.join("source-end-stop-q4.bin"))?,
                receipt: json!({"cue_metadata":a.bank.frozen_cue_native_metadata_sha256,"cue_packed":a.bank.frozen_cue_packed_sha256,"prefix_metadata":a.bank.frozen_prefix_native_metadata_sha256,"prefix_packed":a.bank.frozen_prefix_packed_sha256,"end_metadata":a.frozen_end_native_metadata_sha256,"end_period_packed":a.frozen_end_period_packed_sha256,"end_stop_packed":a.frozen_end_stop_packed_sha256}),
            })
        }
    }
    fn learning_schedule(seed: u64) -> Result<Vec<Vec<usize>>> {
        if seed == 0 {
            return Err(invalid("fit schedule seed must be nonzero").into());
        }
        let mut state = seed;
        let mut rows = Vec::new();
        for _epoch in 0..4 {
            let mut blocks = (0..16usize).collect::<Vec<_>>();
            for i in (1..blocks.len()).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let j = (state % (i as u64 + 1)) as usize;
                blocks.swap(i, j);
            }
            for b in blocks {
                let mut batch = (0..4).map(|i| b * 4 + i).collect::<Vec<_>>();
                batch.extend((0..4).map(|i| 64 + b * 4 + i));
                rows.push(batch);
            }
        }
        Ok(rows)
    }
    fn root_parameter(n: &str) -> bool {
        matches!(
            n,
            "consumer.context.token_root"
                | "consumer.context.self_root"
                | "consumer.context.neighbor_root"
        )
    }
    fn frozen_receipts(s: &SourceRealizerWeights) -> Result<Value> {
        parameter_receipts(
            &s.parameters()
                .into_iter()
                .filter(|(n, _)| !root_parameter(n))
                .collect(),
        )
    }
    fn root_params(s: &SourceRealizerWeights) -> Result<BTreeMap<String, Var>> {
        let p = s.observation_root_parameters();
        if p.len() != 3 || p.keys().any(|n| !root_parameter(n)) {
            return Err(invalid("root API/filter inventory differs").into());
        }
        Ok(p)
    }
    fn training_cue<'a>(
        n: &'a NativeSourceRealizer,
        f: &FrozenAngular,
    ) -> Result<NativeCueCarrier<'a>> {
        Ok(n.compile_cue_carrier(
            CueAngularQ4::new(f.cue_config.clone(), &f.cue).map_err(|e| invalid(e.to_string()))?,
        )?)
    }
    fn training_prefix<'a>(
        n: &'a NativeSourceRealizer,
        c: &NativeCueCarrier<'_>,
        f: &FrozenAngular,
    ) -> Result<NativePrefixTransport<'a>> {
        Ok(n.compile_prefix_transport(
            c,
            PrefixAngularQ4::new(f.prefix_config.clone(), &f.prefix)
                .map_err(|e| invalid(e.to_string()))?,
        )?)
    }
    fn training_end<'a>(
        n: &'a NativeSourceRealizer,
        c: &NativeCueCarrier<'_>,
        p: &NativePrefixTransport<'_>,
        f: &FrozenAngular,
    ) -> Result<NativeSourceEndTransport<'a>> {
        Ok(n.compile_source_end_transport(
            c,
            p,
            SourceEndAngularQ4::new(f.end_config.clone(), &f.period, &f.stop)
                .map_err(|e| invalid(e.to_string()))?,
        )?)
    }
    fn end_native_load<'a>(
        root: &Path,
        n: &'a IntegerRealizer,
        c: &NativeCueCarrier<'_>,
        p: &NativePrefixTransport<'_>,
    ) -> Result<NativeSourceEndTransport<'a>> {
        let meta = read_json(&root.join("native-metadata.json"))?;
        let end = n.compile_source_end_transport(
            c,
            p,
            SourceEndAngularQ4::new(
                serde_json::from_value(meta["potential"].clone())?,
                &fs::read(root.join("source-end-period-q4.bin"))?,
                &fs::read(root.join("source-end-stop-q4.bin"))?,
            )
            .map_err(|e| invalid(e.to_string()))?,
        )?;
        if serde_json::to_value(end.metadata())? != meta {
            return Err(invalid("native end payload/binding differs").into());
        }
        Ok(end)
    }
    fn canonical_observation(
        n: &IntegerRealizer,
        c: &NativeCueCarrier<'_>,
        p: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
        es: &[Episode],
        a: &Args,
        start: Instant,
    ) -> Result<Value> {
        let mut rows = Vec::new();
        let mut total = 0.;
        let mut zeros = Vec::new();
        let mut positions = 0;
        for e in es {
            let segments = e.segments()?;
            let mut ce = 0.;
            let mut tokens = Vec::new();
            let mut rowzero = false;
            for (step, &target) in e.target.iter().enumerate() {
                deadline(a, start)?;
                let trace = n.read_bank_with_source_end_transport(
                    &segments,
                    &e.packet.query_ids,
                    &e.target[..step],
                    c,
                    p,
                    end,
                )?;
                let den = trace.actions.total_weight_q31;
                if den == 0 {
                    return Err(invalid("zero global native normalizer").into());
                }
                let mass = trace
                    .actions
                    .token_masses
                    .iter()
                    .filter(|x| x.token_id == target)
                    .map(|x| x.weight_q31)
                    .sum::<u64>();
                let loss = if mass == 0 {
                    rowzero = true;
                    zeros.push(json!({"id":e.packet.id,"step":step}));
                    None
                } else {
                    let x = -(mass as f64 / den as f64).ln();
                    ce += x;
                    Some(x)
                };
                tokens.push(json!({"step":step,"canonical_prefix_ids_labels_only":&e.target[..step],"target_label_after_read":target,"target_mass_q31":mass,"total_weight_q31":den,"native_ce":loss,"actions":trace.actions,"source_end":trace.source_end}));
                positions += 1;
            }
            let mean = if rowzero {
                None
            } else {
                Some(ce / e.target.len() as f64)
            };
            if let Some(x) = mean {
                total += x / es.len() as f64;
            }
            rows.push(json!({"id":e.packet.id,"native_mean_token_ce":mean,"tokens":tokens}));
        }
        Ok(
            json!({"cases":es.len(),"target_positions":positions,"native_equal_episode_ce":if zeros.is_empty(){Some(total)}else{None},"zero_support_positions":zeros,"rows":rows,"probability_floor":false}),
        )
    }
    fn frozen_path_signature(out: &SourceEndBankRealizerTrace) -> Value {
        let b = &out.prefix_bank.cue_bank.bank;
        let c = &out.prefix_bank.cue_bank.carrier;
        let p = &out.prefix_bank.prefix;
        let cues=c.cues.iter().map(|x|json!({"source_segment_index":x.source_segment_index,"context_segment_index":x.context_segment_index,"token_ids":x.state.token_ids,"states":x.state.states,"categories":x.state.categories})).collect::<Vec<_>>();
        json!({"context_tokens":b.context.tokens,"context_states":b.context.states,"context_actions":b.context.actions,"context_categories":b.context.categories,"query_ids":c.query.token_ids,"query_states":c.query.states,"query_categories":c.query.categories,"cue_states_categories":cues,"candidate_cue_indices":c.candidate_cue_indices,"response_ids":p.response.token_ids,"response_states":p.response.states,"source_prefix_states":p.sources,"prefix_indices":p.angular_indices})
    }
    fn batch_roots(
        indices: &[usize],
        es: &[Episode],
        s: &SourceRealizerWeights,
        parent: &NativeSourceRealizer,
        frozen: &IntegerRealizer,
        f: &FrozenAngular,
        a: &ObservationArgs,
        start: Instant,
        zero_parity: bool,
    ) -> Result<Batch> {
        let began = Instant::now();
        let current = s.compile_observation_rebound(parent)?;
        let prepared = s.prepare(&current)?;
        let cue = training_cue(&current, f)?;
        let prefix = training_prefix(&current, &cue, f)?;
        let end = training_end(&current, &cue, &prefix, f)?;
        let cr = a
            .bank
            .frozen_cue_bundle
            .as_ref()
            .ok_or_else(|| invalid("cue absent"))?;
        let pr = a
            .bank
            .frozen_prefix_bundle
            .as_ref()
            .ok_or_else(|| invalid("prefix absent"))?;
        let oldcue = cue_native_load(cr, frozen)?;
        let oldprefix = prefix_native_load(pr, frozen, &oldcue)?;
        let oldend = end_native_load(&a.frozen_end_bundle, frozen, &oldcue, &oldprefix)?;
        let params = root_params(s)?; // BEFORE accumulation and clipping: no inactive gradients enter norm.
        let mut gradients = BTreeMap::<String, Tensor>::new();
        let mut mean = 0.;
        let mut positions = 0;
        let mut rows = Vec::new();
        let mut connected_inactive = BTreeSet::new();
        let mut consumed_packets = Vec::new();
        for &i in indices {
            let e = es
                .get(i)
                .ok_or_else(|| invalid("episode index outside frozen panel"))?;
            let segments = e.segments()?;
            let mut ce = 0.;
            for (step, &target) in e.target.iter().enumerate() {
                deadline(&a.bank, start)?;
                // Label appears only after complete source-only native cue/prefix/end execution.
                let out = prepared.loss_bank_observation(
                    &segments,
                    &e.packet.query_ids,
                    &e.target[..step],
                    target,
                    &cue,
                    &prefix,
                    &end,
                )?;
                let baseline = frozen.read_bank_with_source_end_transport(
                    &segments,
                    &e.packet.query_ids,
                    &e.target[..step],
                    &oldcue,
                    &oldprefix,
                    &oldend,
                )?;
                if frozen_path_signature(&baseline) != frozen_path_signature(&out.trace) {
                    return Err(invalid(
                        "fixed-input latent/action/category/prefix freeze differs",
                    )
                    .into());
                }
                if zero_parity
                    && serde_json::to_value(&baseline)? != serde_json::to_value(&out.trace)?
                {
                    return Err(invalid("zero root full native trace parity differs").into());
                }
                if step == 0 {
                    let carrier = &out.trace.prefix_bank.cue_bank.carrier;
                    consumed_packets.push(json!({"id":e.packet.id,"query":carrier.query,"cues":carrier.cues,"candidate_cue_indices":carrier.candidate_cue_indices,"scope":"independent identity-reset source-only encodes; labels excluded; root/cat/final-latent collision inspection"}));
                }
                let native = -out.target_probability.ln();
                let scalar = out.loss.to_scalar::<f32>()?;
                if !native.is_finite()
                    || !scalar.is_finite()
                    || (f64::from(scalar) - native).abs() > 1e-4 + 1e-5 * native.abs()
                {
                    return Err(invalid(
                        "root native support/scalar invalid; no probability floor",
                    )
                    .into());
                }
                ce += native;
                positions += 1;
                let store = (&out.loss * scale(indices.len(), e.target.len())?)?.backward()?;
                // Shared context forward can build frozen-family graph work. Report it honestly,
                // but retain/average ONLY root Vars; frozen graphs are discarded per target.
                for (name, var) in s.parameters() {
                    if !root_parameter(&name) && store.get(var.as_tensor()).is_some() {
                        connected_inactive.insert(name);
                    }
                }
                for (name, var) in &params {
                    let g = store
                        .get(var.as_tensor())
                        .ok_or_else(|| invalid(format!("root Var disconnected: {name}")))?
                        .detach();
                    let sum = match gradients.remove(name) {
                        Some(old) => (&old + &g)?.detach(),
                        None => g,
                    };
                    gradients.insert(name.clone(), sum);
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
                return Err(invalid("root gradient nonfinite").into());
            }
            sq += v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>();
            stats.insert(name.clone(),json!({"shape":g.dims(),"elements":v.len(),"finite":true,"nonzero":v.iter().filter(|x|**x!=0.).count(),"l1":v.iter().map(|x|f64::from(x.abs())).sum::<f64>()}));
        }
        Ok(Batch {
            gradients,
            report: json!({"episodes":indices.len(),"episode_indices":indices,"target_positions":positions,"native_equal_episode_ce":mean,"gradient_global_norm":sq.sqrt(),"gradient_families":stats,"connected_frozen_families_discarded":connected_inactive,"carrier_packets_once_per_row":consumed_packets,"rows":rows,"elapsed_seconds":began.elapsed().as_secs_f64(),"active_families":ROOT_FAMILIES,"native_zero_full_trace_parity":zero_parity,"fixed_input_latent_actions_categories_prefix_states_equal_to_original":true,"extra_original_native_invariant_reads":positions,"objective":"equalepisode mean fullanswer+EOS ordinary native globalaliasCE; pertoken detached F32 accumulation; no floor; no source gate","credit_scope":"offline biased finite-choice root surrogate; stopped-gradient SourceEnd source argmax; no descent guarantee"}),
        })
    }
    fn apply_roots(
        s: &SourceRealizerWeights,
        opt: &mut AdamW,
        grad: BTreeMap<String, Tensor>,
    ) -> Result<f64> {
        let params = root_params(s)?;
        let mut sq = 0.;
        if grad.len() != params.len() {
            return Err(invalid("three-root gradient count differs").into());
        }
        for (name, g) in &grad {
            if !params.contains_key(name) {
                return Err(invalid("inactive gradient rejected before norm").into());
            }
            for x in g.flatten_all()?.to_vec1::<f32>()? {
                if !x.is_finite() {
                    return Err(invalid("root clip gradient nonfinite").into());
                }
                sq += f64::from(x).powi(2);
            }
        }
        let norm = sq.sqrt();
        let clip = if norm > 1. { 1. / norm } else { 1. };
        let mut store = Tensor::new(0f32, &Device::Cpu)?.backward()?;
        for (name, g) in grad {
            store.insert(params[&name].as_tensor(), (&g * clip)?.detach());
        }
        opt.step(&store)?;
        for var in params.values() {
            let v = var.flatten_all()?.to_vec1::<f32>()?;
            if v.iter().any(|x| !x.is_finite()) {
                return Err(invalid("root shadow nonfinite").into());
            }
            var.set(&Tensor::from_vec(
                v.into_iter()
                    .map(|x| x.clamp(-1.75, 1.75))
                    .collect::<Vec<_>>(),
                var.shape(),
                &Device::Cpu,
            )?)?;
        }
        Ok(clip)
    }
    fn generation_read_budget(base_len: usize, maximum_generation_tokens: usize) -> Result<usize> {
        let limit = uor_r4_integer::geometric_occurrence_read::MAX_SEQUENCE;
        if base_len > limit {
            return Err(
                invalid("generation bank/query/initial-prefix exceeds native context cap").into(),
            );
        }
        let remaining_decisions = limit
            .checked_sub(base_len)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| invalid("generation read-budget arithmetic overflow"))?;
        Ok(maximum_generation_tokens.min(remaining_decisions))
    }
    fn public_generation_base_length(
        native: &IntegerRealizer,
        segments: &[SourceBankSegment<'_>],
        query_length: usize,
        initial_prefix_length: usize,
    ) -> Result<usize> {
        let mut base = query_length
            .checked_add(initial_prefix_length)
            .ok_or_else(|| invalid("generation query/prefix length overflow"))?;
        for segment in segments {
            let length = match segment {
                SourceBankSegment::Context { token_ids, .. } => token_ids.len(),
                SourceBankSegment::Source { frame, .. } => native
                    .compile_view(frame.token_ids)?
                    .emitted_token_ids()
                    .len(),
            };
            base = base
                .checked_add(length)
                .ok_or_else(|| invalid("generation public bank length overflow"))?;
        }
        Ok(base)
    }
    fn generation_observation(
        n: &IntegerRealizer,
        c: &NativeCueCarrier<'_>,
        p: &NativePrefixTransport<'_>,
        end: &NativeSourceEndTransport<'_>,
        es: &[Episode],
        tok: &ByteBpeTokenizer,
        a: &Args,
        start: Instant,
        frozen: &IntegerRealizer,
        f: &FrozenAngular,
    ) -> Result<Value> {
        let oldcue = frozen.compile_cue_carrier(
            CueAngularQ4::new(f.cue_config.clone(), &f.cue).map_err(|e| invalid(e.to_string()))?,
        )?;
        let oldprefix = frozen.compile_prefix_transport(
            &oldcue,
            PrefixAngularQ4::new(f.prefix_config.clone(), &f.prefix)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        let oldend = frozen.compile_source_end_transport(
            &oldcue,
            &oldprefix,
            SourceEndAngularQ4::new(f.end_config.clone(), &f.period, &f.stop)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        let mut rows = Vec::new();
        let mut complete = 0;
        let mut invariant_reads = 0;
        for e in es {
            let segments = e.segments()?;
            // Loader rejects initial prefixes; serving begins from the actual empty prefix.
            let base_len =
                public_generation_base_length(n, &segments, e.packet.query_ids.len(), 0)?;
            let read_budget = generation_read_budget(base_len, a.maximum_generation_tokens)?;
            let mut ids = Vec::new();
            let mut actions = Vec::new();
            let mut states = None;
            for step in 0..read_budget {
                deadline(a, start)?;
                let out = n.read_bank_with_source_end_transport(
                    &segments,
                    &e.packet.query_ids,
                    &ids,
                    c,
                    p,
                    end,
                )?;
                let baseline = frozen.read_bank_with_source_end_transport(
                    &segments,
                    &e.packet.query_ids,
                    &ids,
                    &oldcue,
                    &oldprefix,
                    &oldend,
                )?;
                if frozen_path_signature(&baseline) != frozen_path_signature(&out) {
                    return Err(invalid(
                        "actual own-prefix frozen latent/category signature differs",
                    )
                    .into());
                }
                invariant_reads += 1;
                let carrier = &out.prefix_bank.cue_bank.carrier;
                let packet = json!({"query":carrier.query,"cues":carrier.cues,"candidate_cue_indices":carrier.candidate_cue_indices});
                match &states {
                    None => states = Some(packet),
                    Some(initial) => {
                        if initial != &packet {
                            return Err(
                                invalid("prefix-stable carrier encode packet changed").into()
                            );
                        }
                    }
                }
                let chosen = out.actions.chosen_token_id;
                actions.push(json!({"step":step,"actual_prefix_ids":ids,"actions":out.actions,"source_end":out.source_end}));
                ids.push(chosen);
                if chosen == n.binding().eos_token_id() {
                    break;
                }
            }
            let eos = ids.last() == Some(&n.binding().eos_token_id());
            let budget_exhausted = !eos && ids.len() == read_budget;
            let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
            let bytes = tok.decode_bytes(plain);
            let raw = String::from_utf8_lossy(&bytes);
            let text = raw.strip_prefix(' ').unwrap_or(&raw);
            let accepted =
                eos && String::from_utf8(bytes.clone()).is_ok() && e.answers.accepts(text);
            complete += usize::from(accepted);
            rows.push(json!({"id":e.packet.id,"generated_ids_including_eos":ids,"public_base_context_tokens":base_len,"read_budget":read_budget,"budget_exhausted":budget_exhausted,"termination":if eos {"eos"} else {"read_budget_exhausted"},"eos":eos,"reply_text":text,"accepted_complete_answer":accepted,"carrier_state_packet_once":states,"fixed_input_frozen_signature_checked":true,"tokens":actions}));
        }
        Ok(
            json!({"cases":es.len(),"accepted_complete":complete,"maximum_generated_tokens":a.maximum_generation_tokens,"canonical_prefixes_used":false,"native_only_parent_and_rebound_sidecars":true,"extra_original_native_invariant_reads":invariant_reads,"rows":rows}),
        )
    }
    fn checkpoint(
        s: &SourceRealizerWeights,
        parent: &NativeSourceRealizer,
        f: &FrozenAngular,
        es: &[Episode],
        tok: &ByteBpeTokenizer,
        a: &ObservationArgs,
        start: Instant,
        step: usize,
    ) -> Result<Value> {
        let root = a.bank.out.join(format!("checkpoint-{step:04}"));
        report_output::claim(&root)?;
        // Source shadows are checkpoint evidence only. All predictions reload integer artifacts.
        let current = s.compile_observation_rebound(parent)?;
        let expected = current.execution_binding()?;
        s.save_source(&root.join("source"))?;
        current.save(&root.join("native"))?;
        let native = IntegerRealizer::load_native(&root.join("native"), &expected)?;
        let cue = native.compile_cue_carrier(
            CueAngularQ4::new(f.cue_config.clone(), &f.cue).map_err(|e| invalid(e.to_string()))?,
        )?;
        let prefix = native.compile_prefix_transport(
            &cue,
            PrefixAngularQ4::new(f.prefix_config.clone(), &f.prefix)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        let end = native.compile_source_end_transport(
            &cue,
            &prefix,
            SourceEndAngularQ4::new(f.end_config.clone(), &f.period, &f.stop)
                .map_err(|e| invalid(e.to_string()))?,
        )?;
        for (dir, meta, files) in [
            (
                "cue",
                serde_json::to_value(cue.metadata())?,
                vec![("cue-q4.bin", &f.cue)],
            ),
            (
                "prefix",
                serde_json::to_value(prefix.metadata())?,
                vec![("prefix-q4.bin", &f.prefix)],
            ),
            (
                "end",
                serde_json::to_value(end.metadata())?,
                vec![
                    ("source-end-period-q4.bin", &f.period),
                    ("source-end-stop-q4.bin", &f.stop),
                ],
            ),
        ] {
            let d = root.join(dir);
            fs::create_dir(&d)?;
            fs::write(
                d.join("native-metadata.json"),
                serde_json::to_vec_pretty(&meta)?,
            )?;
            for (name, bytes) in files {
                fs::write(d.join(name), bytes)?;
            }
        }
        // Replay from the files actually written, enforcing newly exported parent identities.
        let cue = cue_native_load(&root.join("cue"), &native)?;
        let prefix = prefix_native_load(&root.join("prefix"), &native, &cue)?;
        let end = end_native_load(&root.join("end"), &native, &cue, &prefix)?;
        let canon = canonical_observation(&native, &cue, &prefix, &end, es, &a.bank, start)?;
        let generation = generation_observation(
            &native,
            &cue,
            &prefix,
            &end,
            es,
            tok,
            &a.bank,
            start,
            &IntegerRealizer::load_native(&a.bank.native_artifact, &parent.artifact_binding()?)?,
            f,
        )?;
        write_json(&root, "canonical.json", &canon)?;
        write_json(&root, "generation.json", &generation)?;
        let receipt = json!({"step":step,"native_equal_episode_ce":canon["native_equal_episode_ce"],"accepted_complete":generation["accepted_complete"],"new_parent":expected,"frozen_payloads":f.receipt,"active_root_receipts":parameter_receipts(&root_params(s)?)?,"frozen_parameter_receipts":frozen_receipts(s)?,"native_reload_generation":true,"sidecars_rebound_to_actual_updated_context":true});
        write_json(&root, "receipt.json", &receipt)?;
        report_output::seal(&root)?;
        report_output::verify(&root)?;
        Ok(receipt)
    }
    fn validate(a: &ObservationArgs) -> Result<()> {
        let fit = a.bank.mode == "observation-fit";
        let broad = a.bank.mode == "observation-broadbatch";
        if (fit && a.learning_seed.unwrap_or(0) == 0) || (broad && a.learning_seed.is_some()) {
            return Err(
                invalid("broad has no seed verdict;fit requires explicit seeded schedule").into(),
            );
        }
        if a.schema != "uor-r4.native-bank-observation-args/1"
            || (!fit && !broad)
            || !matches!(a.data_scope.as_str(),
                "original-assertion-cues/raw-queries/all-source-candidates/1"
                | "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2")
            || a.bank.maximum_context_tokens != 128
            || a.bank.maximum_generation_tokens != 32
            || a.bank.maximum_seconds == 0
            || a.bank.maximum_seconds > if fit { 1200 } else { 300 }
            || a.bank.maximum_report_bytes
                > if fit {
                    512 * 1024 * 1024
                } else {
                    64 * 1024 * 1024
                }
        {
            return Err(invalid("observation mode/bounds/scope differs").into());
        }
        if a.bank.maximum_report_bytes == 0
            || a.bank.exposed_controls.is_some()
            || a.bank.learned_source_weights.is_some()
            || a.bank.learned_native_artifact.is_some()
            || a.bank.learned_trusted_native_binding.is_some()
        {
            return Err(
                invalid("unsupported legacy overlay/control arguments in dedicated draft").into(),
            );
        }
        if fit
            && (a.bank.admission.is_none()
                || a.bank.admission_manifest_sha256.is_none()
                || a.bank.fit_authorization.is_none())
        {
            return Err(invalid(
                "fit requires independent exact-data broad admission and authorization",
            )
            .into());
        }
        if broad && (a.bank.admission.is_some() || a.bank.fit_authorization.is_some()) {
            return Err(invalid("broad mode cannot auto-fit").into());
        }
        Ok(())
    }
    fn validate_raw_cues(
        root: &Path,
        es: &[Episode],
        tok: &ByteBpeTokenizer,
        tokenizer_sha: &str,
    ) -> Result<usize> {
        let report = read_json(&root.join("report.json"))?;
        let receipt_path = root.join("raw-cue-provenance.json");
        if report["raw_cue_provenance_sha256"] != sha256_file(&receipt_path)? {
            return Err(invalid("raw cue provenance SHA differs").into());
        }
        let receipt = read_json(&receipt_path)?;
        if receipt["schema"] != "uor-r4.raw-assertion-cue-provenance/1" {
            return Err(invalid("raw cue provenance schema differs").into());
        }
        let rows = receipt["rows"]
            .as_array()
            .ok_or_else(|| invalid("cue provenance rows absent"))?;
        let mut used = BTreeSet::new();
        let mut count = 0;
        for e in es {
            let segments = e.segments()?;
            for (i, segment) in segments.iter().enumerate() {
                if let SourceBankSegment::Context {
                    token_ids, event, ..
                } = segment
                {
                    let matching = rows
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| {
                            r["case_id"] == e.packet.id
                                && r["context_segment_index"].as_u64() == Some(i as u64)
                        })
                        .collect::<Vec<_>>();
                    if matching.len() != 1 {
                        return Err(invalid(
                            "each raw Context needs one recorded-assertion receipt",
                        )
                        .into());
                    }
                    let (j, r) = matching[0];
                    if !used.insert(j) {
                        return Err(invalid("duplicate cue receipt admission").into());
                    }
                    let text = r["utterance_utf8"]
                        .as_str()
                        .ok_or_else(|| invalid("original utterance absent"))?;
                    if r["utterance_sha256"] != sha256_bytes(text.as_bytes())
                        || r["tokenizer_sha256"] != tokenizer_sha
                        || tok.encode(text).as_slice() != *token_ids
                        || r["context_token_ids"] != json!(token_ids)
                        || r["source_event"].as_u64() != Some(*event)
                    {
                        return Err(invalid("original cue byte/BPE/event binding differs").into());
                    }
                    match segments.get(i+1){Some(SourceBankSegment::Source{frame,event:source_event,..})=>{
                    if r["source_segment_index"].as_u64()!=Some((i+1)as u64)||r["record"].as_u64()!=Some(frame.identity.record)||r["commit"].as_u64()!=Some(frame.identity.commit)||r["stored_original_source_ids"]!=json!(frame.token_ids)||*source_event!=*event{return Err(invalid("cue-to-actual-following-store-record receipt differs").into());}
                },_=>return Err(invalid("natural reader cue Context must immediately precede actual stored Source").into())}
                    count += 1;
                }
            }
        }
        if count != rows.len() {
            return Err(invalid("extra or omitted cue receipt rows").into());
        }
        Ok(count)
    }
    fn run_observation(a: &ObservationArgs, start: Instant) -> Result<Value> {
        let f = FrozenAngular::load(a)?;
        let schedule = if let Some(seed) = a.learning_seed {
            Some(learning_schedule(seed)?)
        } else {
            None
        };
        let schedule_sha = if let Some(rows) = &schedule {
            Some(sha256_bytes(&serde_json::to_vec(rows)?))
        } else {
            None
        };
        // Admission can only project allocated output after actually loading nothing:
        // five source+native exports exceed64MiB. Broad creates ZERO parent exports.
        let payload_per_checkpoint = directory_bytes(&a.bank.source_weights)?
            + directory_bytes(&a.bank.native_artifact)?
            + f.cue.len()
            + f.prefix.len()
            + f.period.len()
            + f.stop.len()
            + 65536;
        let projected_payload_bytes = if a.bank.mode == "observation-fit" {
            5 * payload_per_checkpoint
        } else {
            0
        };
        let reserved_trace_bytes = if a.bank.mode == "observation-fit" {
            128 * 1024 * 1024
        } else {
            48 * 1024 * 1024
        };
        if projected_payload_bytes + reserved_trace_bytes > a.bank.maximum_report_bytes {
            return Err(invalid("declared report cap cannot hold five exact parents plus trace reserve;no silent extension").into());
        }
        write_json(
            &a.bank.out,
            "report-size-projection.json",
            &json!({"existing_source_bytes":directory_bytes(&a.bank.source_weights)?,"existing_native_bytes":directory_bytes(&a.bank.native_artifact)?,"full_parent_exports":if a.bank.mode=="observation-fit"{5}else{0},"prospective_static_payload_bytes":projected_payload_bytes,"prospective_trace_reserve_bytes":reserved_trace_bytes,"maximum_report_bytes":a.bank.maximum_report_bytes,"trace_estimate":"NOT_MEASURED;cap enforced during each write;no automatic resource admission"}),
        )?;
        let mut inputs = BTreeMap::new();
        let mut seals = BTreeSet::new();
        for p in [
            &a.bank.source_weights,
            &a.bank.native_artifact,
            &a.bank.development_panel,
            &a.bank.fresh_panel,
            a.bank
                .frozen_cue_bundle
                .as_ref()
                .ok_or_else(|| invalid("cue absent"))?,
            a.bank
                .frozen_prefix_bundle
                .as_ref()
                .ok_or_else(|| invalid("prefix absent"))?,
            &a.frozen_end_bundle,
        ] {
            let seal = nearest_seal(p)?;
            report_output::verify(&seal)?;
            inputs.insert(
                seal.join("manifest.json").to_string_lossy().into_owned(),
                sha256_file(&seal.join("manifest.json"))?,
            );
            seals.insert(seal);
        }
        for (root, hash) in [
            (
                &a.bank.development_panel,
                &a.bank.development_manifest_sha256,
            ),
            (&a.bank.fresh_panel, &a.bank.fresh_manifest_sha256),
        ] {
            if sha256_file(&root.join("manifest.json"))? != *hash {
                return Err(invalid("frozen panel manifest mismatch").into());
            }
        }
        let binding: NativeArtifactBinding =
            serde_json::from_slice(&read_capped(&a.bank.trusted_native_binding)?)?;
        let trusted = sha256_file(&a.bank.trusted_native_binding)?;
        inputs.insert(
            a.bank.trusted_native_binding.to_string_lossy().into_owned(),
            trusted.clone(),
        );
        let native = IntegerRealizer::load_native(&a.bank.native_artifact, &binding)?;
        let tokenizer = fs::read(a.bank.native_artifact.join("tokenizer.json"))?;
        let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer)
            .ok_or_else(|| invalid("ByteBPE absent"))?;
        let s = SourceRealizerWeights::load_source(&a.bank.source_weights, &tokenizer)?;
        let identity = serde_json::from_value(
            read_json(&a.bank.native_artifact.join("metadata.json"))?["identity"].clone(),
        )?;
        let parent = NativeSourceRealizer::load(&a.bank.native_artifact, &s, &identity)?;
        if parent.artifact_binding()? != binding {
            return Err(invalid("source/native binding differs").into());
        }
        // Natural raw-cue provenance is a required panel-construction receipt in this
        // dedicated task; old authored cue packets cannot masquerade as raw statements.
        // Root-owned preparation must hash actual original utterance bytes/tokenization;
        // this field is provenance admission, never an inference filter.
        let expected_cue_origin = match a.data_scope.as_str() {
            "original-assertion-cues/raw-queries/all-source-candidates/1" => {
                "original-assertion-bytes/bound-byteBPE/1"
            }
            "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2" => {
                "prospectively-authored-current-role-bytes/bound-byteBPE/2"
            }
            _ => return Err(invalid("unrecognized cue data scope").into()),
        };
        // Metadata admission does not evaluate fresh predictions or use its labels.
        for panel in [&a.bank.development_panel, &a.bank.fresh_panel] {
            let panel_report = read_json(&panel.join("report.json"))?;
            if panel_report["cue_origin_policy"] != expected_cue_origin
            || (a.data_scope == "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2"
                && panel_report["source_policy"] != a.data_scope) {
            return Err(invalid(
                "natural cue construction receipt absent; no synthetic Memory-line fallback",
            )
            .into());
        }
        }
        // Validate all frozen sidecars against original loaded parent BEFORE rebinding payloads.
        let oldcue = cue_native_load(
            a.bank
                .frozen_cue_bundle
                .as_ref()
                .ok_or_else(|| invalid("cue absent"))?,
            &native,
        )?;
        let oldprefix = prefix_native_load(
            a.bank
                .frozen_prefix_bundle
                .as_ref()
                .ok_or_else(|| invalid("prefix absent"))?,
            &native,
            &oldcue,
        )?;
        let _oldend = end_native_load(&a.frozen_end_bundle, &native, &oldcue, &oldprefix)?;
        let episodes = load_natural_panel(&a.bank.development_panel, 128, &native, &tok)?;
        let raw_cues = validate_raw_cues(
            &a.bank.development_panel,
            &episodes,
            &tok,
            &sha256_bytes(&tokenizer),
        )?;
        let frozen = frozen_receipts(&s)?;
        let all = (0..episodes.len()).collect::<Vec<_>>();
        write_json(
            &a.bank.out,
            "frozen-inputs.json",
            &json!({"schema":a.schema,"host":std::env::consts::OS,"architecture":std::env::consts::ARCH,"executable_sha256":sha256_file(&executable()?.0)?,"executable_lookup":executable()?.1,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"mode":a.bank.mode,"input_sha256":inputs,"frozen_sidecars":f.receipt,"data_scope":a.data_scope,"validated_cue_records":raw_cues,"active_families":ROOT_FAMILIES,"learning_seed":a.learning_seed,"learning_schedule":schedule,"learning_schedule_sha256":schedule_sha,"initialization":"same unchanged learned parent;no random root perturbation","updates":if a.bank.mode=="observation-fit"{64}else{0},"fresh_predictions":"NOT_RUN_UNTIL_SELECTION","configuration_subset_sha256":sha256_bytes(&serde_json::to_vec(&json!({"scope":a.data_scope,"bounds":[a.bank.maximum_context_tokens,a.bank.maximum_generation_tokens],"frozen":f.receipt}))?),"panel_layout_policy":NATURAL_PANEL_LAYOUT,"development_allbank_rows":128,"development_adjacent_query_pairs":64,"runtime_packet_schema":"uor-r4.native-source-bank-probe-input/1","membership_labels_runtime":false}),
        )?;
        let fit = a.bank.mode == "observation-fit";
        let initial_evaluation_start = Instant::now();
        let mut stages = if fit {
            vec![checkpoint(&s, &parent, &f, &episodes, &tok, a, start, 0)?]
        } else {
            // Broad admission stays below64MiB: no duplicated parent/source export.
            let original = canonical_observation(
                &native, &oldcue, &oldprefix, &_oldend, &episodes, &a.bank, start,
            )?;
            let generation = generation_observation(
                &native, &oldcue, &oldprefix, &_oldend, &episodes, &tok, &a.bank, start, &native,
                &f,
            )?;
            write_json(&a.bank.out, "canonical-zero.json", &original)?;
            write_json(&a.bank.out, "generation-zero.json", &generation)?;
            vec![
                json!({"step":0,"native_equal_episode_ce":original["native_equal_episode_ce"],"accepted_complete":generation["accepted_complete"],"new_parent":binding,"frozen_payloads":f.receipt,"independent_native_parent":"original bound parent;zero loss fulltrace equality checked before backward"}),
            ]
        };
        let initial_evaluation_seconds = initial_evaluation_start.elapsed().as_secs_f64();
        let broad = batch_roots(&all, &episodes, &s, &parent, &native, &f, a, start, true)?;
        write_json(&a.bank.out, "broadbatch.json", &broad.report)?;
        let baseline = stages[0]["native_equal_episode_ce"]
            .as_f64()
            .ok_or_else(|| invalid("baseline objective infinite; no support filtering"))?;
        if (broad.report["native_equal_episode_ce"]
            .as_f64()
            .ok_or_else(|| invalid("broad objective absent"))?
            - baseline)
            .abs()
            > 1e-10
        {
            return Err(invalid("broad/checkpoint objective differs").into());
        }
        let full128_gradient_seconds = broad.report["elapsed_seconds"]
            .as_f64()
            .ok_or_else(|| invalid("broad time absent"))?;
        drop(broad); // FULL128 GRADIENT RELEASE precedes first B8 calibration.
        let calibration_indices = learning_schedule(1001)?[0].clone();
        let calibration = batch_roots(
            &calibration_indices,
            &episodes,
            &s,
            &parent,
            &native,
            &f,
            a,
            start,
            true,
        )?;
        let gradient_bytes = calibration
            .gradients
            .values()
            .map(|g| g.elem_count() * std::mem::size_of::<f32>())
            .sum::<usize>();
        let calibration_seconds = calibration.report["elapsed_seconds"]
            .as_f64()
            .ok_or_else(|| invalid("B8 calibration time absent"))?;
        let calibration_receipt = json!({"schema":"uor-r4.native-bank-observation-B8-calibration/1","status":"COMPLETED","updates":0,"gradient_bytes":gradient_bytes,"peak_rss_kib":peak_rss_kib(),"batch":calibration.report,"full128_gradient_seconds":full128_gradient_seconds,"initial_evaluation_seconds":initial_evaluation_seconds,"development_manifest_sha256":a.bank.development_manifest_sha256,"fresh_manifest_sha256":a.bank.fresh_manifest_sha256,"trusted_binding_sha256":trusted,"frozen_sidecars":f.receipt,"active_families":ROOT_FAMILIES,"scope":"fixed prospective B8 cost instrument, not fitted seed;actual final native pipeline"});
        write_json(&a.bank.out, "B8-calibration.json", &calibration_receipt)?;
        drop(calibration);
        if fit {
            let ar = a
                .bank
                .admission
                .as_ref()
                .ok_or_else(|| invalid("admission absent"))?;
            report_output::verify(ar)?;
            if Some(sha256_file(&ar.join("manifest.json"))?) != a.bank.admission_manifest_sha256 {
                return Err(invalid("admission seal differs").into());
            }
            let r = read_json(&ar.join("report.json"))?;
            let auth: ObservationAuthorization = serde_json::from_slice(&fs::read(
                a.bank
                    .fit_authorization
                    .as_ref()
                    .ok_or_else(|| invalid("auth absent"))?,
            )?)?;
            if auth.schema != "uor-r4.native-bank-observation-fit-authorization/1"
                || !auth.fit_admitted
                || auth.admission_report_sha256 != sha256_file(&ar.join("report.json"))?
                || auth.development_manifest_sha256 != a.bank.development_manifest_sha256
                || auth.fresh_manifest_sha256 != a.bank.fresh_manifest_sha256
                || auth.trusted_binding_sha256 != trusted
                || auth.frozen_sidecars_sha256 != f.receipt
                || auth.active_families != ROOT_FAMILIES
                || auth.updates != 64
                || Some(auth.learning_seed) != a.learning_seed
                || Some(auth.schedule_sha256) != schedule_sha
                || auth.batch_episodes != 8
                || auth.maximum_fit_seconds != a.bank.maximum_seconds
                || r["mode"] != "observation-broadbatch"
                || r["status"] != "COMPLETED"
                || r["development_manifest_sha256"] != a.bank.development_manifest_sha256
                || r["fresh_manifest_sha256"] != a.bank.fresh_manifest_sha256
                || r["trusted_binding_sha256"] != trusted
                || r["frozen_sidecars"] != f.receipt
                || r["source_commit"] != option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND")
            {
                return Err(invalid("exact observation admission/auth differs").into());
            }
            let calibration_path = ar.join("B8-calibration.json");
            let oldcal = read_json(&calibration_path)?;
            let oldseconds = oldcal["batch"]["elapsed_seconds"]
                .as_f64()
                .ok_or_else(|| invalid("admitted B8 seconds absent"))?;
            let projected_minimum = start.elapsed().as_secs_f64()
                + 64.
                    * calibration_seconds
                        .max(oldseconds)
                        .max(full128_gradient_seconds / 16.)
                    * 1.25
                + 5. * initial_evaluation_seconds * 1.25;
            if auth.calibration_report_sha256 != sha256_file(&calibration_path)?
                || oldcal["status"] != "COMPLETED"
                || oldcal["updates"] != 0
                || oldcal["development_manifest_sha256"] != a.bank.development_manifest_sha256
                || oldcal["fresh_manifest_sha256"] != a.bank.fresh_manifest_sha256
                || oldcal["trusted_binding_sha256"] != trusted
                || oldcal["frozen_sidecars"] != f.receipt
                || !oldseconds.is_finite()
                || oldseconds <= 0.
                || !auth.projected_complete_fit_seconds.is_finite()
                || auth.projected_complete_fit_seconds < projected_minimum
                || auth.projected_complete_fit_seconds > a.bank.maximum_seconds as f64
            {
                return Err(invalid(
                    "exact B8 calibration/full fit cost not admitted;zero optimizer updates",
                )
                .into());
            }
            write_json(
                &a.bank.out,
                "fit-cost-admission.json",
                &json!({"calibration_report_sha256":auth.calibration_report_sha256,"minimum_measured_projection_seconds":projected_minimum,"declared_complete_projection_seconds":auth.projected_complete_fit_seconds,"maximum_seconds":a.bank.maximum_seconds,"formula":"elapsed admission +64*max(admitted/current B8,full128/16)*1.25 +5*measured initial evaluation*1.25;no descent guarantee","optimizer":{"lr":0.003,"beta1":0.9,"beta2":0.999,"eps":1e-8,"weight_decay":0.}}),
            )?;
            let mut opt = AdamW::new(
                root_params(&s)?.values().cloned().collect(),
                ParamsAdamW {
                    lr: 0.003,
                    beta1: 0.9,
                    beta2: 0.999,
                    eps: 1e-8,
                    weight_decay: 0.,
                },
            )?;
            for update in 0..64 {
                deadline(&a.bank, start)?;
                if frozen_receipts(&s)? != frozen {
                    return Err(invalid("frozen source parameter drift").into());
                }
                let batch = batch_roots(
                    &schedule
                        .as_ref()
                        .ok_or_else(|| invalid("fit schedule absent"))?[update],
                    &episodes,
                    &s,
                    &parent,
                    &native,
                    &f,
                    a,
                    start,
                    false,
                )?;
                let clip = apply_roots(&s, &mut opt, batch.gradients)?;
                write_json(
                    &a.bank.out,
                    &format!("update-{:04}.json", update + 1),
                    &json!({"update":update+1,"batch":batch.report,"global_clip_factor":clip}),
                )?;
                if frozen_receipts(&s)? != frozen {
                    return Err(invalid("frozen source parameter drift after update").into());
                }
                if (update + 1) % 16 == 0 {
                    stages.push(checkpoint(
                        &s,
                        &parent,
                        &f,
                        &episodes,
                        &tok,
                        a,
                        start,
                        update + 1,
                    )?);
                }
            }
        }
        let mut selected = 0;
        let mut best = baseline;
        for (i, stage) in stages.iter().enumerate().skip(1) {
            if let Some(ce) = stage["native_equal_episode_ce"].as_f64() {
                if ce.is_finite() && ce < best {
                    best = ce;
                    selected = i;
                }
            }
        }
        write_json(
            &a.bank.out,
            "selection.json",
            &json!({"schema":"uor-r4.native-bank-observation-selection/1","criterion":"native_equal_episode_fullanswer_plus_EOS_alias_CE;baseline-inclusive;earliest-strict-minimum","selected_step":stages[selected]["step"],"selected_native_ce":best,"checkpoints":stages,"selection_frozen_before_fresh":true}),
        )?;
        // No new draw here. Existing independently prepared fresh32 is opened ONLY after selector freeze.
        if fit {
            let fresh = load_natural_panel(&a.bank.fresh_panel, 32, &native, &tok)?;
            validate_raw_cues(&a.bank.fresh_panel, &fresh, &tok, &sha256_bytes(&tokenizer))?;
            for (name, step) in [
                ("parent", 0u64),
                (
                    "selected",
                    stages[selected]["step"]
                        .as_u64()
                        .ok_or_else(|| invalid("selected step absent"))?,
                ),
            ] {
                let root = a.bank.out.join(format!("checkpoint-{step:04}"));
                let receipt = read_json(&root.join("receipt.json"))?;
                let binding = serde_json::from_value(receipt["new_parent"].clone())?;
                let n = IntegerRealizer::load_native(&root.join("native"), &binding)?;
                let c = cue_native_load(&root.join("cue"), &n)?;
                let p = prefix_native_load(&root.join("prefix"), &n, &c)?;
                let e = end_native_load(&root.join("end"), &n, &c, &p)?;
                write_json(
                    &a.bank.out,
                    &format!("fresh-{name}.json"),
                    &generation_observation(
                        &n, &c, &p, &e, &fresh, &tok, &a.bank, start, &native, &f,
                    )?,
                )?;
            }
        }
        for p in &seals {
            report_output::verify(p)?;
        }
        for (path, expected) in &inputs {
            if sha256_file(Path::new(path))? != *expected {
                return Err(invalid("input hash changed during execution").into());
            }
        }
        Ok(
            json!({"schema":"uor-r4.native-bank-observation-report/1","status":"COMPLETED","mode":a.bank.mode,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"development_manifest_sha256":a.bank.development_manifest_sha256,"fresh_manifest_sha256":a.bank.fresh_manifest_sha256,"trusted_binding_sha256":trusted,"frozen_sidecars":f.receipt,"active_families":ROOT_FAMILIES,"frozen_parameter_bits_equal":frozen_receipts(&s)?==frozen,"updates":if fit{64}else{0},"broad_gradient_report":"broadbatch.json","zero_fulltrace_parity_executed":true,"selected_step":stages[selected]["step"],"selected_native_ce":best,"peak_rss_kib":peak_rss_kib(),"elapsed_seconds":start.elapsed().as_secs_f64(),"support_floor":false,"supplied_selected_record":false,"fresh_predictions":if fit{"PARENT_AND_SELECTED_ONLY"}else{"NOT_RUN"},"learning_seed":a.learning_seed,"learning_schedule_sha256":schedule_sha,"replication_scope":if fit{"same learned root initialization and frozen encoder;seed changes episode order only;not independent initialization or chat lineages"}else{"zero-update gradient admission is not a fitted seed verdict"},"panel_layout_policy":NATURAL_PANEL_LAYOUT,"development_allbank_rows":128,"development_adjacent_query_pairs":64,"fresh_allbank_rows":32,"fresh_adjacent_query_pairs":16,"optimizer_exposure":if fit{"64 seeded block-balanced B8;4rows fromeach64-bank-row half;two intact querypairs perhalf;4visits/episode"}else{"zero updates;full128 admission only"},"runtime":"unchanged integer full-bank cue/prefix/SourceEnd/globalalias;no sourceF32 generation","claim":"bounded native attention observation learner;general chat and transfer unqualified"}),
        )
    }
    #[cfg(test)]
    mod observation_tests {
        use super::*;
        #[test]
        fn generation_budget_reserves_only_actual_prefix_reads() -> Result<()> {
            assert_eq!(generation_read_budget(98, 32)?, 31);
            assert_eq!(generation_read_budget(97, 32)?, 32);
            assert_eq!(generation_read_budget(0, 32)?, 32);
            assert_eq!(generation_read_budget(128, 32)?, 1);
            assert_eq!(generation_read_budget(98, 4)?, 4);
            assert!(generation_read_budget(129, 32).is_err());
            assert!(generation_read_budget(usize::MAX, 32).is_err());
            Ok(())
        }
        #[test]
        fn root_selector_does_not_admit_transition_category_or_heads() {
            for n in [
                "consumer.context.token_root",
                "consumer.context.self_root",
                "consumer.context.neighbor_root",
            ] {
                assert!(root_parameter(n));
            }
            for n in [
                "consumer.context.token_transition",
                "consumer.context.token_category",
                "consumer.context.self_category",
                "consumer.potential.context_unary",
                "consumer.no_read.coefficients",
                "period.coefficients",
            ] {
                assert!(!root_parameter(n));
            }
        }
        #[test]
        fn balanced_schedule_preserves_four_visits_and_query_pair_blocks() -> Result<()> {
            let mut signatures = BTreeSet::new();
            for seed in [1001, 1002, 1003] {
                let mut visits = [0usize; 128];
                let schedule = learning_schedule(seed)?;
                for rows in &schedule {
                    assert_eq!(rows.len(), 8);
                    assert!(rows[..4].iter().all(|i| *i < 64));
                    assert!(rows[4..].iter().all(|i| *i >= 64));
                    assert_eq!(rows[4] % 2, 0);
                    assert_eq!(rows[5], rows[4] + 1);
                    assert_eq!(rows[7], rows[6] + 1);
                    for i in rows {
                        visits[*i] += 1;
                    }
                }
                assert!(visits.iter().all(|n| *n == 4));
                signatures.insert(sha256_bytes(&serde_json::to_vec(&schedule)?));
            }
            assert_eq!(signatures.len(), 3);
            Ok(())
        }
    }
    pub fn entry() -> Result<()> {
        let config = std::env::args()
            .nth(1)
            .ok_or_else(|| invalid("usage: geometric-native-bank-observation CONFIG.json"))?;
        let a: ObservationArgs = serde_json::from_slice(&fs::read(&config)?)?;
        validate(&a)?;
        let input_paths = [
            &a.bank.source_weights,
            &a.bank.native_artifact,
            &a.bank.development_panel,
            &a.bank.fresh_panel,
            &a.frozen_end_bundle,
        ];
        let out = output_support::prospective_output(&a.bank.out)?;
        for p in input_paths {
            let original = fs::canonicalize(p)?;
            if out.starts_with(&original) || original.starts_with(&out) {
                return Err(invalid("output intersects immutable input").into());
            }
        }
        for p in [
            &a.bank.trusted_native_binding,
            a.bank
                .frozen_cue_bundle
                .as_ref()
                .ok_or_else(|| invalid("cue absent"))?,
            a.bank
                .frozen_prefix_bundle
                .as_ref()
                .ok_or_else(|| invalid("prefix absent"))?,
        ] {
            let original = fs::canonicalize(p)?;
            if out.starts_with(&original) || original.starts_with(&out) {
                return Err(invalid("output intersects cue/prefix/binding input").into());
            }
        }
        for p in [a.bank.admission.as_ref(), a.bank.fit_authorization.as_ref()] {
            if let Some(p) = p {
                let original = fs::canonicalize(p)?;
                if out.starts_with(&original) || original.starts_with(&out) {
                    return Err(invalid("output intersects admission/authorization").into());
                }
            }
        }
        report_output::claim(&a.bank.out)?; // BEFORE any model load.
        write_json_limited(
            &a.bank.out,
            "resource-cap.json",
            &json!({"maximum_report_bytes":a.bank.maximum_report_bytes}),
            a.bank.maximum_report_bytes,
        )?;
        let start = Instant::now();
        let result = run_observation(&a, start);
        let receipt = match &result {
            Ok(v) => v.clone(),
            Err(e) => {
                json!({"schema":"uor-r4.native-bank-observation-report/1","status":"FAILED","mode":a.bank.mode,"error":e.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"elapsed_seconds":start.elapsed().as_secs_f64(),"model_quality_verdict":"NOT_RUN_OR_INCOMPLETE"})
            }
        };
        write_json(&a.bank.out, "report.json", &receipt)?;
        report_output::seal(&a.bank.out)?;
        report_output::verify(&a.bank.out)?;
        result.map(|_| ())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    reuse::entry()
}
