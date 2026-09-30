//! Model-source `score_and_normalize` flock transport and its G1a plumbing.
//!
//! [`FlockCausalAttentionTransport`] implements the model-source
//! [`CausalAttentionTransport`](uor_r4_model_source::attention::CausalAttentionTransport)
//! seam with identity frame hooks and flock weights at the score hook. It is
//! the B0 arm host for the amended G1a subset: the coordinate transport,
//! RoPE, residuals, MLP and head remain the model-source decoder's, and the
//! only difference from
//! [`DenseIdentityTransport`] is
//! [`score_and_normalize`](uor_r4_model_source::attention::CausalAttentionTransport::score_and_normalize).
//!
//! Scores are computed from the post-RoPE query and the packed causal key
//! prefix exactly as the pure selector expects: rank scores are the Minkowski
//! product `-q0 k0 + q . k` (monotone, no `arcosh`) and model scores are the
//! checkpoint's scaled dot product `q . k / sqrt(width)`. The selector itself
//! stays in [`crate::flock`]; this module only lifts rows and fills the
//! weight vector.
//!
//! Status: **implemented, NOT RUN.** The focused tests below either need no
//! model (pure row/weight checks and fail-closed shape checks) or are the
//! `#[ignore]`d G1a subset, which requires the pinned SmolLM2 checkpoint and
//! the u16 token file. No Cargo or model work is authorized while the recovery
//! hold (#1520) is in force.

use std::result::Result as StdResult;

use uor_r4_model_source::attention::{
    CausalAttentionHeadContext, CausalAttentionSourceContext, CausalAttentionTransport,
};

use crate::flock::{
    flock_row_weights, FlockSpec, FlockStats, FlockWeights, FLOCK_SELECTOR_VERSION,
};
use crate::{invalid, Result};

/// Dense same-path baseline: identity frame hooks and the model-source default
/// stable-softmax score hook.
pub struct DenseIdentityTransport;

impl CausalAttentionTransport for DenseIdentityTransport {
    fn policy_identity(&self) -> &str {
        "dense-identity-transport/1"
    }

    fn begin_position(&mut self, _token: usize, _position: usize) {}

    fn transform_query(
        &mut self,
        _context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }

    fn transport_key(
        &mut self,
        _context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }

    fn transport_value(
        &mut self,
        _context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }

    fn output_to_model_frame(
        &mut self,
        _context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }
}

/// The Lorentz rank row and the host model score row for one query over a
/// packed causal prefix.
///
/// `packed_keys` holds one `query.len()`-wide row per causally visible source
/// position `0..=query_position`. Rank scores are the Minkowski product
/// `-q0 k0 + q . k` of the lifted vectors `(sqrt(1 + |x|^2), x)`; model scores
/// are `q . k / sqrt(width)`. Higher rank scores rank nearer.
pub fn flock_row_scores(
    query: &[f32],
    packed_keys: &[f32],
    query_position: usize,
) -> Result<(Vec<f32>, Vec<f32>)> {
    let width = query.len();
    if width == 0 {
        return Err(invalid("flock transport query row is empty"));
    }
    let visible = query_position + 1;
    if packed_keys.len() != visible * width {
        return Err(invalid(format!(
            "flock transport needs {} packed keys for {} visible rows of width {width}, got {}",
            visible * width,
            visible,
            packed_keys.len()
        )));
    }
    if query.iter().any(|v| !v.is_finite()) || packed_keys.iter().any(|v| !v.is_finite()) {
        return Err(invalid("flock transport scores must be finite"));
    }
    let q0 = (1.0 + query.iter().map(|v| v * v).sum::<f32>()).sqrt();
    let inverse_scale = 1.0 / (width as f32).sqrt();
    let mut rank_row = Vec::with_capacity(visible);
    let mut model_row = Vec::with_capacity(visible);
    for source in 0..visible {
        let row = &packed_keys[source * width..(source + 1) * width];
        let k0 = (1.0 + row.iter().map(|v| v * v).sum::<f32>()).sqrt();
        let dot: f32 = query.iter().zip(row).map(|(a, b)| a * b).sum();
        rank_row.push(dot - q0 * k0);
        model_row.push(dot * inverse_scale);
    }
    Ok((rank_row, model_row))
}

/// Flock attention at the model-source score hook.
///
/// Identity frame hooks; [`FlockSpec`] selects the support and weight arm. Any
/// internal shape or selector fault is recorded in `fault` and surfaced
/// through [`CausalAttentionTransport::status`], so the decoder step fails
/// closed instead of scoring a partial row.
pub struct FlockCausalAttentionTransport {
    spec: FlockSpec,
    stats: FlockStats,
    fault: Option<String>,
    width: Option<usize>,
}

impl FlockCausalAttentionTransport {
    pub fn new(spec: FlockSpec) -> Result<Self> {
        if spec.k == 0 {
            return Err(invalid("flock transport k must be positive"));
        }
        if spec.window == 0 {
            return Err(invalid("flock transport window must be positive"));
        }
        Ok(Self {
            spec,
            stats: FlockStats::default(),
            fault: None,
            width: None,
        })
    }

    pub fn spec(&self) -> &FlockSpec {
        &self.spec
    }

    pub fn stats(&self) -> &FlockStats {
        &self.stats
    }

    /// Supports collected since the previous call, for the harness report.
    pub fn take_stats(&mut self) -> FlockStats {
        std::mem::take(&mut self.stats)
    }

    fn fail(&mut self, message: impl Into<String>) {
        if self.fault.is_none() {
            self.fault = Some(message.into());
        }
    }

    fn weights_for(
        &mut self,
        query: &[f32],
        packed_keys: &[f32],
        query_position: usize,
    ) -> Result<Vec<f32>> {
        let (rank_row, model_row) = flock_row_scores(query, packed_keys, query_position)?;
        let weighted = flock_row_weights(&rank_row, &model_row, query_position, &self.spec)?;
        self.stats.record(&weighted.selection.scan);
        Ok(weighted.weights)
    }
}

impl CausalAttentionTransport for FlockCausalAttentionTransport {
    fn policy_identity(&self) -> &str {
        "flock-causal-attention-transport/1"
    }

    fn status(&self) -> StdResult<(), String> {
        match &self.fault {
            Some(fault) => Err(fault.clone()),
            None => Ok(()),
        }
    }

    fn implementation_evidence(&self) -> StdResult<Option<String>, String> {
        Ok(Some(
            serde_json::json!({
                "selector": FLOCK_SELECTOR_VERSION,
                "k": self.spec.k,
                "window": self.spec.window,
                "weights": match self.spec.weights {
                    FlockWeights::Softmax => "softmax",
                    FlockWeights::Rank => "rank",
                },
                "stats": {
                    "queries": self.stats.queries,
                    "selected": self.stats.selected,
                    "sink": self.stats.sink,
                    "window": self.stats.window,
                    "top_k": self.stats.top_k,
                    "candidates_scanned": self.stats.candidates_scanned,
                    "short_prefix": self.stats.short_prefix,
                    "top_k_short": self.stats.top_k_short,
                    "cutoff_ties": self.stats.cutoff_ties,
                },
            })
            .to_string(),
        ))
    }

    fn begin_position(&mut self, _token: usize, _position: usize) {}

    fn transform_query(
        &mut self,
        _context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }

    fn transport_key(
        &mut self,
        _context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }

    fn transport_value(
        &mut self,
        _context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }

    fn score_and_normalize(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
        _canonical_math: bool,
    ) {
        if self.fault.is_some() {
            return;
        }
        let width = query.len();
        match self.width {
            None => self.width = Some(width),
            Some(seen) if seen != width => {
                self.fail(format!(
                    "flock transport head width changed from {seen} to {width}"
                ));
                return;
            }
            Some(_) => {}
        }
        if output_weights.len() != context.query_position + 1 {
            self.fail(format!(
                "flock transport weight row has {} slots for query position {}",
                output_weights.len(),
                context.query_position
            ));
            return;
        }
        match self.weights_for(query, packed_keys, context.query_position) {
            Ok(weights) => output_weights.copy_from_slice(&weights),
            Err(error) => self.fail(format!("flock transport selector: {error}")),
        }
    }

    fn output_to_model_frame(
        &mut self,
        _context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flock::{flock_select, raw_rank_weights, FlockSelect};
    use crate::kappa_llama::held_out_starts;
    use std::path::{Path, PathBuf};

    fn query_row() -> Vec<f32> {
        vec![1.0, -0.5, 0.25, 2.0]
    }

    fn key_rows(visible: usize) -> Vec<f32> {
        let mut keys = Vec::with_capacity(visible * 4);
        for source in 0..visible {
            for lane in 0..4 {
                keys.push((source as f32) * 0.2 + (lane as f32) * 0.1 - 0.7);
            }
        }
        keys
    }

    #[test]
    fn flock_rows_match_the_selector_inputs_and_weights() {
        let query = query_row();
        let keys = key_rows(5);
        let (rank_row, model_row) = flock_row_scores(&query, &keys, 4).expect("rows");
        assert_eq!(rank_row.len(), 5);
        assert_eq!(model_row.len(), 5);
        for source in 0..5usize {
            let row = &keys[source * 4..(source + 1) * 4];
            let q0 = (1.0 + query.iter().map(|v| v * v).sum::<f32>()).sqrt();
            let k0 = (1.0 + row.iter().map(|v| v * v).sum::<f32>()).sqrt();
            let dot: f32 = query.iter().zip(row).map(|(a, b)| a * b).sum();
            assert_eq!(rank_row[source], dot - q0 * k0);
            assert_eq!(model_row[source], dot / 2.0);
        }

        let spec = FlockSpec {
            k: 1,
            window: 2,
            weights: FlockWeights::Rank,
        };
        let expected = flock_row_weights(&rank_row, &model_row, 4, &spec).expect("weights");
        let mut transport = FlockCausalAttentionTransport::new(spec).expect("transport");
        let mut output = vec![0f32; 5];
        transport.score_and_normalize(
            CausalAttentionHeadContext {
                layer: 3,
                head: 1,
                query_position: 4,
            },
            &query,
            &keys,
            &mut output,
            false,
        );
        assert!(transport.status().is_ok(), "{:?}", transport.status());
        assert_eq!(output, expected.weights);
        assert_eq!(transport.stats().queries, 1);
        assert_eq!(transport.stats().selected, expected.selection.len() as u64);
        assert_eq!(
            transport.stats().candidates_scanned,
            expected.selection.scan.candidates_scanned as u64
        );
    }

    #[test]
    fn rank_arm_matches_the_normalized_table_convention() {
        let query = query_row();
        let keys = key_rows(4);
        let (rank_row, _model_row) = flock_row_scores(&query, &keys, 3).expect("rows");
        let spec = FlockSpec {
            k: 2,
            window: 1,
            weights: FlockWeights::Rank,
        };
        let selection = flock_select(
            &rank_row,
            3,
            FlockSelect {
                sink: 0,
                window: spec.window,
                k: spec.k,
            },
        )
        .expect("selection");
        let mut transport = FlockCausalAttentionTransport::new(spec).expect("transport");
        let mut output = vec![0f32; 4];
        transport.score_and_normalize(
            CausalAttentionHeadContext {
                layer: 0,
                head: 0,
                query_position: 3,
            },
            &query,
            &keys,
            &mut output,
            false,
        );
        assert!(transport.status().is_ok(), "{:?}", transport.status());
        let total: f64 = output.iter().map(|value| f64::from(*value)).sum();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "arm R normalizes to one: {total}"
        );
        let raw = raw_rank_weights(selection.len());
        assert_eq!(raw[0], 1.0);
        assert!((raw[1] - 0.5).abs() < 1e-6);
        for entry in &selection.entries {
            assert!(
                output[entry.position] > 0.0,
                "kept position {} must carry weight",
                entry.position
            );
        }
        let first = selection.entries[0].position;
        for entry in &selection.entries[1..] {
            assert!(
                output[first] > output[entry.position],
                "rank 0 must carry the largest weight"
            );
        }
    }

    #[test]
    fn transport_fails_closed_on_shape_mismatch() {
        let mut transport = FlockCausalAttentionTransport::new(FlockSpec {
            k: 1,
            window: 1,
            weights: FlockWeights::Softmax,
        })
        .expect("transport");
        let query = query_row();
        let short_keys = key_rows(2);
        let mut output = vec![0f32; 2];
        transport.score_and_normalize(
            CausalAttentionHeadContext {
                layer: 0,
                head: 0,
                query_position: 2,
            },
            &query,
            &short_keys,
            &mut output,
            false,
        );
        assert!(transport.status().is_err(), "a short row must fail closed");
        let evidence = transport
            .implementation_evidence()
            .expect("evidence")
            .expect("some");
        assert!(evidence.contains(FLOCK_SELECTOR_VERSION));
    }

    fn env_usize(name: &str, default: usize) -> usize {
        match std::env::var(name) {
            Ok(value) => value
                .parse()
                .unwrap_or_else(|_| panic!("{name} is an integer")),
            Err(_) => default,
        }
    }

    fn env_csv(name: &str, default: &str) -> Vec<String> {
        std::env::var(name)
            .unwrap_or_else(|_| default.to_owned())
            .split(',')
            .map(|item| item.trim().to_owned())
            .filter(|item| !item.is_empty())
            .collect()
    }

    fn read_u16_tokens(path: &Path) -> Vec<u16> {
        let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("tokens: {error}"));
        assert!(
            bytes.len().is_multiple_of(2),
            "token file is little-endian u16"
        );
        bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    }

    fn argmax(row: &[f32]) -> usize {
        row.iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| index)
            .expect("a nonempty logit row")
    }

    fn next_token_logprob(logits: &[f32], target: usize) -> f64 {
        let maximum = logits
            .iter()
            .fold(f64::NEG_INFINITY, |best, &value| best.max(f64::from(value)));
        let sum: f64 = logits
            .iter()
            .map(|&value| (f64::from(value) - maximum).exp())
            .sum();
        f64::from(logits[target]) - maximum - sum.ln()
    }

    /// G1a subset through the model-source seam: the dense identity transport
    /// must reproduce `step_state`, then the flock arms report their NLL gap.
    #[test]
    #[ignore = "needs the pinned SmolLM2 checkpoint and u16 token file: UOR_B0_MODEL=/path UOR_B0_TOKENS=/path/x.u16 UOR_B0_HELD_OUT=N [UOR_B0_TIME=512] [UOR_B0_WINDOWS=4] [UOR_B0_SEED=9001] [UOR_B0_K=7,64] [UOR_B0_WEIGHTS=softmax,rank] [UOR_B0_G1A_ROOT=/fresh/root] cargo test -p uor-r4-training --release --offline --lib g1a_subset -- --ignored --nocapture"]
    fn g1a_subset_dense_and_flock_through_the_model_source_seam() {
        use uor_r4_model_source::attention::CausalAttentionLayerSelection;
        use uor_r4_model_source::{HuggingFaceLlamaOracle, TeacherExecutionConfig};

        let started = std::time::Instant::now();
        let model_dir = PathBuf::from(
            std::env::var("UOR_B0_MODEL")
                .expect("set UOR_B0_MODEL to the SmolLM2-135M-Instruct directory"),
        );
        let token_path = PathBuf::from(
            std::env::var("UOR_B0_TOKENS").expect("set UOR_B0_TOKENS to the pinned u16 file"),
        );
        let held_out: usize = std::env::var("UOR_B0_HELD_OUT")
            .expect("set UOR_B0_HELD_OUT to the held-out token count")
            .parse()
            .expect("UOR_B0_HELD_OUT is a token count");
        let time = env_usize("UOR_B0_TIME", 512);
        assert!(
            time <= 2048,
            "the pinned G1a draw uses 2048-token windows; keep UOR_B0_TIME <= 2048"
        );
        let windows = env_usize("UOR_B0_WINDOWS", 4);
        let seed = env_usize("UOR_B0_SEED", 9001) as u64;
        let ks: Vec<usize> = env_csv("UOR_B0_K", "7,64")
            .iter()
            .map(|value| value.parse().expect("UOR_B0_K entries are integers"))
            .collect();
        let weight_arms: Vec<FlockWeights> = env_csv("UOR_B0_WEIGHTS", "softmax,rank")
            .iter()
            .map(|value| FlockWeights::parse(value).expect("UOR_B0_WEIGHTS entries"))
            .collect();
        let root = std::env::var("UOR_B0_G1A_ROOT").ok().map(PathBuf::from);

        let tokens = read_u16_tokens(&token_path);
        let starts = held_out_starts(tokens.len(), 2048, windows, held_out, seed)
            .expect("held-out starts (the pinned draw uses 2048-token windows)");
        if let Some(root) = &root {
            uor_r4_core::report_output::claim(root).expect("claim the g1a report root");
        }

        let oracle = HuggingFaceLlamaOracle::load_with_sequence_length_and_execution(
            &model_dir,
            time,
            TeacherExecutionConfig::available_parallelism(),
        )
        .map_err(|error| invalid(format!("oracle: {error}")))
        .expect("exact oracle");
        let vocab = oracle.cfg().vocab;
        let weights_sha =
            crate::sha256_file(&model_dir.join("model.safetensors")).expect("weights sha");
        let tokens_sha = crate::sha256_file(&token_path).expect("tokens sha");

        let mut arm_labels: Vec<String> = vec!["dense".to_owned()];
        for &k in &ks {
            for &weight in &weight_arms {
                arm_labels.push(format!(
                    "flock-k{k}-{}",
                    match weight {
                        FlockWeights::Softmax => "softmax",
                        FlockWeights::Rank => "rank",
                    }
                ));
            }
        }
        let mut arm_nll_totals = vec![0f64; arm_labels.len()];
        let mut window_reports = Vec::new();
        let mut positions = 0usize;
        let mut dense_agreement = 0usize;
        let mut dense_bitwise_mismatches = 0u64;
        let mut maximum_logit_delta = 0f64;
        let mut plain_nll_total = 0f64;

        for (window_index, &start) in starts.iter().enumerate() {
            let token_window = &tokens[start..start + time];
            let targets = &tokens[start + 1..start + time + 1];
            let mut sessions = Vec::new();
            sessions.push((
                "dense".to_owned(),
                oracle
                    .new_causal_attention_transport_session(
                        Box::new(DenseIdentityTransport),
                        CausalAttentionLayerSelection::All,
                        time,
                    )
                    .expect("dense transport session"),
            ));
            for &k in &ks {
                for &weight in &weight_arms {
                    let spec = FlockSpec {
                        k,
                        window: 64,
                        weights: weight,
                    };
                    sessions.push((
                        format!(
                            "flock-k{k}-{}",
                            match weight {
                                FlockWeights::Softmax => "softmax",
                                FlockWeights::Rank => "rank",
                            }
                        ),
                        oracle
                            .new_causal_attention_transport_session(
                                Box::new(
                                    FlockCausalAttentionTransport::new(spec)
                                        .expect("flock transport"),
                                ),
                                CausalAttentionLayerSelection::All,
                                time,
                            )
                            .expect("flock transport session"),
                    ));
                }
            }
            let mut state = oracle.new_state_bounded(time).expect("bounded state");
            let mut plain_logits = vec![0f32; vocab];
            let mut transport_logits = vec![0f32; vocab];
            let mut window_nll = vec![0f64; sessions.len()];
            for (position, &token) in token_window.iter().enumerate() {
                let target = targets[position] as usize;
                oracle
                    .step_state(&mut state, token as usize, position, &mut plain_logits)
                    .expect("plain decoder step");
                plain_nll_total += next_token_logprob(&plain_logits, target);
                for (index, (_, session)) in sessions.iter_mut().enumerate() {
                    oracle
                        .step_causal_attention_transport(
                            session,
                            token as usize,
                            position,
                            &mut transport_logits,
                        )
                        .expect("transport decoder step");
                    window_nll[index] += next_token_logprob(&transport_logits, target);
                    if index == 0 {
                        if argmax(&plain_logits) == argmax(&transport_logits) {
                            dense_agreement += 1;
                        }
                        for (reference, transported) in
                            plain_logits.iter().zip(transport_logits.iter())
                        {
                            let delta = (f64::from(*reference) - f64::from(*transported)).abs();
                            maximum_logit_delta = maximum_logit_delta.max(delta);
                            if delta != 0.0 {
                                dense_bitwise_mismatches += 1;
                            }
                        }
                    }
                }
                positions += 1;
            }
            let mut evidence = Vec::new();
            for (index, (label, session)) in sessions.iter().enumerate() {
                arm_nll_totals[index] += window_nll[index];
                if index > 0 {
                    let value = session
                        .transport_implementation_evidence()
                        .expect("transport evidence")
                        .map(|raw| serde_json::from_str::<serde_json::Value>(&raw).expect("json"))
                        .expect("flock sessions carry evidence");
                    evidence.push(serde_json::json!({ "arm": label, "evidence": value }));
                }
            }
            window_reports.push(serde_json::json!({
                "window": window_index,
                "start": start,
                "arm_nll": arm_labels
                    .iter()
                    .zip(window_nll.iter())
                    .map(|(label, nll)| serde_json::json!({ "arm": label, "nll": nll }))
                    .collect::<Vec<_>>(),
                "flock_evidence": evidence,
            }));
            eprintln!(
                "G1a window {window_index}/{} start={start} done at {:.0}s",
                starts.len(),
                started.elapsed().as_secs_f64()
            );
        }

        let plain_nll = plain_nll_total / positions as f64;
        let dense_nll = arm_nll_totals[0] / positions as f64;
        let dense_delta = (plain_nll - dense_nll).abs();
        let arm_summary: Vec<serde_json::Value> = arm_labels
            .iter()
            .zip(arm_nll_totals.iter())
            .map(|(label, total)| {
                let nll = total / positions as f64;
                serde_json::json!({
                    "arm": label,
                    "nll": nll,
                    "delta_vs_dense": nll - dense_nll,
                })
            })
            .collect();
        let report = serde_json::json!({
            "schema": "uor-r4.b0-g1a-transport/1",
            "selector": FLOCK_SELECTOR_VERSION,
            "identity": {
                "model": model_dir.display().to_string(),
                "weights_sha256": weights_sha,
                "tokens": token_path.display().to_string(),
                "tokens_sha256": tokens_sha,
                "time": time,
                "draw_time": 2048,
                "windows": windows,
                "seed": seed,
                "held_out_tokens": held_out,
                "window_starts": starts,
            },
            "gate": {
                "positions": positions,
                "argmax_agreement": dense_agreement,
                "max_abs_logit_delta": maximum_logit_delta,
                "bitwise_mismatched_logits": dense_bitwise_mismatches,
                "plain_nll": plain_nll,
                "dense_transport_nll": dense_nll,
                "abs_nll_delta": dense_delta,
            },
            "arms": arm_summary,
            "windows": window_reports,
            "status": "this-run",
        });
        match &root {
            Some(root) => {
                std::fs::write(
                    root.join("g1a.json"),
                    serde_json::to_vec_pretty(&report).expect("serialize"),
                )
                .expect("write g1a.json");
                uor_r4_core::report_output::seal(root).expect("seal the g1a root");
                let unlisted = uor_r4_core::report_output::verify(root).expect("verify");
                assert!(unlisted.is_empty(), "unlisted files: {unlisted:?}");
            }
            None => {
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("serialize")
                );
            }
        }
        assert_eq!(
            dense_agreement, positions,
            "the dense transport must agree with step_state on every argmax"
        );
        assert!(
            maximum_logit_delta <= 1e-4,
            "dense |delta logit| {maximum_logit_delta} exceeds 1e-4"
        );
        assert!(
            dense_delta <= 1e-4,
            "dense NLL differs from step_state by {dense_delta} nats"
        );
    }
}
