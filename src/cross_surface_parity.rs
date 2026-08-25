//! Canonical, teacher-free cross-surface evidence production for #933.
//!
//! This harness selects an increasing held-out corpus position where the
//! normative SKMX/PSIB lane is actually reachable, then executes the exact
//! shared serving adapters. It does not invent HTTP, WASM, or frontend
//! coverage labels: native HTTP and WASM both consume `R4g1State`/the API
//! adapter, but this producer records that mechanically executed adapter by
//! its own name. Endpoint-wrapper and browser integration remain separate
//! tests.

use std::io::Write;
use std::path::{Path, PathBuf};

use uor_r4_api::engine::EngineParts;
use uor_r4_api::{
    CrossSurfaceDisposition, CrossSurfaceParityEvidence, CrossSurfaceParityEvidenceBuilder,
    CrossSurfaceParityObservation, NormativeServingDecision, NormativeServingEngine,
    SourceUnavailable, CROSS_SURFACE_PARITY_BUNDLE_PATH,
};
use uor_r4_core::transformerless::compiler::{self, Corpus, WINDOW};
use uor_r4_core::transformerless::runtime::SampleRng;

use crate::chat::replayable_normative_chat_step_for_evidence;
use crate::r4g1::{CapturedR4g1Bundle, R4g1State};

/// Immutable inputs for one canonical cross-surface artifact.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalCrossSurfaceMaterial<'a> {
    pub graph: &'a [u8],
    pub signature_artifact: &'a [u8],
    pub tokenizer: Option<&'a [u8]>,
    pub score_report: Option<&'a [u8]>,
    pub corpus_meta: &'a [u8],
    pub corpus_records: &'a [u8],
}

/// Production inputs plus the exact evaluator population from which the
/// deterministic lane-reachable position is selected.
#[derive(Debug, Clone, Copy)]
pub struct CanonicalCrossSurfaceSpec<'a> {
    pub material: CanonicalCrossSurfaceMaterial<'a>,
    pub evaluated_positions: &'a [u64],
    pub sample_seed: u32,
}

/// Evidence plus the selected corpus position, exposed for terminal logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCrossSurfaceOutcome {
    pub selected_position: u64,
    pub scanned_positions: usize,
    pub evidence: CrossSurfaceParityEvidence,
}

/// Produce and independently replay one deterministic cross-surface artifact.
///
/// This is teacher-free: recorded corpus inputs choose the context, but no
/// live model forward occurs. The selected position must both serve and make
/// the learned lane reachable; otherwise evidence is UNAVAILABLE rather than
/// a vacuous PASS over base-only positions. A mechanically observed mismatch
/// is returned as a deterministic evidence row (not discarded as an error) so
/// the caller can persist the artifact and a STOP terminal before refusing
/// production admission.
pub fn produce_canonical_cross_surface_parity(
    spec: CanonicalCrossSurfaceSpec<'_>,
) -> Result<CanonicalCrossSurfaceOutcome, SourceUnavailable> {
    produce_canonical_cross_surface_parity_with_progress(spec, |_, _| {})
}

/// Produce canonical evidence while reporting deterministic scan progress.
///
/// The callback is observational only: it receives the number of positions
/// examined and the fixed population size, and cannot affect selection or
/// evidence bytes.
pub fn produce_canonical_cross_surface_parity_with_progress(
    spec: CanonicalCrossSurfaceSpec<'_>,
    mut progress: impl FnMut(usize, usize),
) -> Result<CanonicalCrossSurfaceOutcome, SourceUnavailable> {
    validate_position_population(spec.evaluated_positions)?;
    let corpus = compiler::load_corpus_bytes(
        spec.material.corpus_meta,
        spec.material.corpus_records,
        None,
    )
    .ok_or_else(|| SourceUnavailable::new("cross-surface corpus bytes are invalid"))?;

    let (selected_position, scanned_positions, context) =
        select_lane_reachable_context(spec, &corpus, &mut progress)?;
    let mut builder = CrossSurfaceParityEvidenceBuilder::new_for_bundle(
        spec.material.graph,
        spec.material.signature_artifact,
        spec.material.tokenizer,
        spec.material.score_report,
    );

    record_direct_api_rows(&mut builder, spec, &context)?;
    record_r4g1_state_rows(&mut builder, spec, &context)?;
    record_cli_chat_rows(&mut builder, spec, &corpus, selected_position, &context)?;

    let evidence = builder.finish()?;
    let bytes = evidence.deterministic_json_bytes().map_err(|error| {
        SourceUnavailable::new(format!("serialize cross-surface evidence: {error}"))
    })?;
    let evidence = CrossSurfaceParityEvidence::parse_and_validate_for_bundle(
        &bytes,
        spec.material.graph,
        spec.material.signature_artifact,
        spec.material.tokenizer,
        spec.material.score_report,
    )?;
    Ok(CanonicalCrossSurfaceOutcome {
        selected_position,
        scanned_positions,
        evidence,
    })
}

/// Write canonical bytes only to the prescribed bundle-relative evidence
/// path. The caller owns generation locking/atomic publication of the bundle.
pub fn write_canonical_cross_surface_parity(
    bundle_root: &Path,
    evidence: &CrossSurfaceParityEvidence,
) -> Result<PathBuf, SourceUnavailable> {
    let path = bundle_root.join(CROSS_SURFACE_PARITY_BUNDLE_PATH);
    let parent = path.parent().ok_or_else(|| {
        SourceUnavailable::new("cross-surface evidence path has no parent directory")
    })?;
    std::fs::create_dir_all(parent).map_err(|error| {
        SourceUnavailable::new(format!(
            "create cross-surface evidence directory {}: {error}",
            parent.display()
        ))
    })?;
    let bytes = evidence.deterministic_json_bytes().map_err(|error| {
        SourceUnavailable::new(format!("serialize cross-surface evidence: {error}"))
    })?;
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() {
                return Err(SourceUnavailable::new(format!(
                    "cross-surface evidence target {} is not a regular non-symlink file",
                    path.display()
                )));
            }
            let existing = std::fs::read(&path).map_err(|error| {
                SourceUnavailable::new(format!(
                    "read existing cross-surface evidence {}: {error}",
                    path.display()
                ))
            })?;
            if existing == bytes {
                return Ok(path);
            }
            return Err(SourceUnavailable::new(format!(
                "cross-surface evidence {} already exists with different bytes; refusing to overwrite another generation",
                path.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(SourceUnavailable::new(format!(
                "inspect cross-surface evidence {}: {error}",
                path.display()
            )));
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| {
            SourceUnavailable::new(format!(
                "create cross-surface evidence {}: {error}",
                path.display()
            ))
        })?;
    file.write_all(&bytes).map_err(|error| {
        SourceUnavailable::new(format!(
            "write cross-surface evidence {}: {error}",
            path.display()
        ))
    })?;
    file.sync_all().map_err(|error| {
        SourceUnavailable::new(format!(
            "sync cross-surface evidence {}: {error}",
            path.display()
        ))
    })?;
    Ok(path)
}

fn validate_position_population(positions: &[u64]) -> Result<(), SourceUnavailable> {
    if positions.is_empty() {
        return Err(SourceUnavailable::new(
            "cross-surface evaluator population is empty",
        ));
    }
    if positions.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(SourceUnavailable::new(
            "cross-surface evaluator population is not strictly increasing",
        ));
    }
    Ok(())
}

fn select_lane_reachable_context(
    spec: CanonicalCrossSurfaceSpec<'_>,
    corpus: &Corpus,
    progress: &mut impl FnMut(usize, usize),
) -> Result<(u64, usize, Vec<u32>), SourceUnavailable> {
    let mut engine = load_direct_engine(spec.material)?;
    let population = spec.evaluated_positions.len();
    for (index, &position) in spec.evaluated_positions.iter().enumerate() {
        let scanned = index + 1;
        if scanned == 1 || scanned % 256 == 0 || scanned == population {
            progress(scanned, population);
        }
        let position_usize = usize::try_from(position)
            .ok()
            .filter(|&position| position < corpus.n)
            .ok_or_else(|| {
                SourceUnavailable::new(format!(
                    "cross-surface position {position} is outside the bound corpus"
                ))
            })?;
        let context = context_window(corpus, position_usize);
        engine.reset_policy_state();
        let decision = engine.predict(&context).map_err(|error| {
            SourceUnavailable::new(format!(
                "cross-surface position {position} selection failed: {error}"
            ))
        })?;
        let NormativeServingDecision::Serve(serve) = decision else {
            continue;
        };
        if !serve.lane_reachable || matches!(serve.token, 1 | 2) {
            continue;
        }
        let mut rng = SampleRng::new(spec.sample_seed);
        if matches!(serve.select_sampled_token(&[], &mut rng), 1 | 2) {
            continue;
        }

        // CLI chat adds the content-derived session lane. Select a position
        // where both of its first-step policies really emit too; otherwise a
        // merely earlier unusable row could hide a later executable one.
        let history = story_history(corpus, position_usize);
        let session_signature = uor_r4_router::session_signature_from_tokens(&history);
        engine.reset_policy_state();
        let session_decision = engine
            .predict_with_session_signature(&context, Some(&session_signature))
            .map_err(|error| {
                SourceUnavailable::new(format!(
                    "cross-surface session position {position} selection failed: {error}"
                ))
            })?;
        let NormativeServingDecision::Serve(session_serve) = session_decision else {
            continue;
        };
        let mut rng = SampleRng::new(spec.sample_seed);
        if session_serve.lane_reachable
            && !matches!(session_serve.token, 1 | 2)
            && !matches!(session_serve.select_sampled_token(&[], &mut rng), 1 | 2)
        {
            if scanned != 1 && scanned % 256 != 0 && scanned != population {
                progress(scanned, population);
            }
            return Ok((position, scanned, context));
        }
    }
    Err(SourceUnavailable::new(
        "cross-surface evidence UNAVAILABLE: evaluated population has no non-terminal served position where the SKMX/PSIB lane is reachable",
    ))
}

fn record_direct_api_rows(
    builder: &mut CrossSurfaceParityEvidenceBuilder,
    spec: CanonicalCrossSurfaceSpec<'_>,
    context: &[u32],
) -> Result<(), SourceUnavailable> {
    let greedy = direct_decision(spec.material, context, None)?;
    let greedy_token = served_token(greedy)?;
    let observed_greedy = direct_decision(spec.material, context, None)?;
    let observed_greedy_serve = served_outcome(observed_greedy)?;
    builder.record(CrossSurfaceParityObservation {
        surface: "direct-api",
        decode_policy: "greedy",
        context_tokens: context,
        session_signature: None,
        authoritative: greedy,
        authoritative_token: Some(greedy_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(observed_greedy_serve.token),
        observed_candidates: Some(observed_greedy_serve.candidates),
    })?;

    let sampled = direct_decision(spec.material, context, None)?;
    let sampled_token = sampled_token(sampled, spec.sample_seed)?;
    let observed_sampled = direct_decision(spec.material, context, None)?;
    let observed_sampled_serve = served_outcome(observed_sampled)?;
    let mut observed_rng = SampleRng::new(spec.sample_seed);
    let observed_sampled_token =
        observed_sampled_serve.select_sampled_token(&[], &mut observed_rng);
    let sampled_policy = sampled_policy(spec.sample_seed);
    builder.record(CrossSurfaceParityObservation {
        surface: "direct-api",
        decode_policy: &sampled_policy,
        context_tokens: context,
        session_signature: None,
        authoritative: sampled,
        authoritative_token: Some(sampled_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(observed_sampled_token),
        observed_candidates: Some(observed_sampled_serve.candidates),
    })
}

fn record_r4g1_state_rows(
    builder: &mut CrossSurfaceParityEvidenceBuilder,
    spec: CanonicalCrossSurfaceSpec<'_>,
    context: &[u32],
) -> Result<(), SourceUnavailable> {
    let state = load_r4g1_state(spec.material)?;
    let mut out = [0u32; 1];
    let (status, observed) = state
        .generate_into_status_with_first_step(context, &mut out)
        .map_err(|error| {
            SourceUnavailable::new(format!("R4g1State greedy parity step: {error}"))
        })?;
    if status.count != 1 || status.abstained {
        return Err(SourceUnavailable::new(
            "R4g1State greedy parity position did not emit exactly one token",
        ));
    }
    let authoritative = direct_decision(spec.material, context, None)?;
    let authoritative_token = served_token(authoritative)?;
    let observed_candidates = served_outcome(observed.ok_or_else(|| {
        SourceUnavailable::new("R4g1State greedy parity step did not capture a first decision")
    })?)?
    .candidates;
    builder.record(CrossSurfaceParityObservation {
        surface: "r4g1-state-native-host-adapter",
        decode_policy: "greedy",
        context_tokens: context,
        session_signature: None,
        authoritative,
        authoritative_token: Some(authoritative_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(out[0]),
        observed_candidates: Some(observed_candidates),
    })?;

    let state = load_r4g1_state(spec.material)?;
    let mut out = [0u32; 1];
    let mut rng = SampleRng::new(spec.sample_seed);
    let (status, observed) = state
        .generate_sampled_into_status_with_first_step(context, &mut out, &mut rng)
        .map_err(|error| {
            SourceUnavailable::new(format!("R4g1State sampled parity step: {error}"))
        })?;
    if status.count != 1 || status.abstained {
        return Err(SourceUnavailable::new(
            "R4g1State sampled parity position did not emit exactly one token",
        ));
    }
    let authoritative = direct_decision(spec.material, context, None)?;
    let authoritative_token = sampled_token(authoritative, spec.sample_seed)?;
    let observed_candidates = served_outcome(observed.ok_or_else(|| {
        SourceUnavailable::new("R4g1State sampled parity step did not capture a first decision")
    })?)?
    .candidates;
    let sampled_policy = sampled_policy(spec.sample_seed);
    builder.record(CrossSurfaceParityObservation {
        surface: "r4g1-state-native-host-adapter",
        decode_policy: &sampled_policy,
        context_tokens: context,
        session_signature: None,
        authoritative,
        authoritative_token: Some(authoritative_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(out[0]),
        observed_candidates: Some(observed_candidates),
    })
}

fn record_cli_chat_rows(
    builder: &mut CrossSurfaceParityEvidenceBuilder,
    spec: CanonicalCrossSurfaceSpec<'_>,
    corpus: &Corpus,
    position: u64,
    context: &[u32],
) -> Result<(), SourceUnavailable> {
    let position = usize::try_from(position)
        .map_err(|_| SourceUnavailable::new("cross-surface position exceeds usize"))?;
    let history = story_history(corpus, position);
    let session_signature = uor_r4_router::session_signature_from_tokens(&history);

    record_session_direct_api_rows(builder, spec, context, &session_signature)?;

    let authoritative = direct_decision(spec.material, context, Some(&session_signature))?;
    let authoritative_token = served_token(authoritative)?;
    let (observed, observed_candidates) = replayable_normative_chat_step_for_evidence(
        spec.material.graph,
        spec.material.signature_artifact,
        spec.material.tokenizer,
        spec.material.score_report,
        context,
        &session_signature,
        None,
    )
    .map_err(SourceUnavailable::new)?;
    builder.record(CrossSurfaceParityObservation {
        surface: "cli-chat-shared-production-step",
        decode_policy: "beam-first-step",
        context_tokens: context,
        session_signature: Some(&session_signature),
        authoritative,
        authoritative_token: Some(authoritative_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(observed),
        observed_candidates: Some(observed_candidates),
    })?;

    let authoritative = direct_decision(spec.material, context, Some(&session_signature))?;
    let authoritative_token = sampled_token(authoritative, spec.sample_seed)?;
    let (observed, observed_candidates) = replayable_normative_chat_step_for_evidence(
        spec.material.graph,
        spec.material.signature_artifact,
        spec.material.tokenizer,
        spec.material.score_report,
        context,
        &session_signature,
        Some(spec.sample_seed),
    )
    .map_err(SourceUnavailable::new)?;
    let sampled_policy = sampled_policy(spec.sample_seed);
    builder.record(CrossSurfaceParityObservation {
        surface: "cli-chat-shared-production-step",
        decode_policy: &sampled_policy,
        context_tokens: context,
        session_signature: Some(&session_signature),
        authoritative,
        authoritative_token: Some(authoritative_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(observed),
        observed_candidates: Some(observed_candidates),
    })
}

fn record_session_direct_api_rows(
    builder: &mut CrossSurfaceParityEvidenceBuilder,
    spec: CanonicalCrossSurfaceSpec<'_>,
    context: &[u32],
    session_signature: &[u8],
) -> Result<(), SourceUnavailable> {
    let authoritative = direct_decision(spec.material, context, Some(session_signature))?;
    let authoritative_token = served_token(authoritative)?;
    let observed = direct_decision(spec.material, context, Some(session_signature))?;
    let observed_serve = served_outcome(observed)?;
    builder.record(CrossSurfaceParityObservation {
        surface: "direct-api-session-bound",
        decode_policy: "beam-first-step",
        context_tokens: context,
        session_signature: Some(session_signature),
        authoritative,
        authoritative_token: Some(authoritative_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(observed_serve.token),
        observed_candidates: Some(observed_serve.candidates),
    })?;

    let authoritative = direct_decision(spec.material, context, Some(session_signature))?;
    let authoritative_token = sampled_token(authoritative, spec.sample_seed)?;
    let observed = direct_decision(spec.material, context, Some(session_signature))?;
    let observed_serve = served_outcome(observed)?;
    let mut rng = SampleRng::new(spec.sample_seed);
    let observed_token = observed_serve.select_sampled_token(&[], &mut rng);
    let sampled_policy = sampled_policy(spec.sample_seed);
    builder.record(CrossSurfaceParityObservation {
        surface: "direct-api-session-bound",
        decode_policy: &sampled_policy,
        context_tokens: context,
        session_signature: Some(session_signature),
        authoritative,
        authoritative_token: Some(authoritative_token),
        observed_disposition: CrossSurfaceDisposition::Serve,
        observed_token: Some(observed_token),
        observed_candidates: Some(observed_serve.candidates),
    })
}

fn load_direct_engine(
    material: CanonicalCrossSurfaceMaterial<'_>,
) -> Result<NormativeServingEngine<'_>, SourceUnavailable> {
    NormativeServingEngine::load_for_research(EngineParts {
        graph: material.graph,
        signature_artifact: material.signature_artifact,
        tokenizer: material.tokenizer,
        score_report: material.score_report,
    })
}

fn direct_decision(
    material: CanonicalCrossSurfaceMaterial<'_>,
    context: &[u32],
    session_signature: Option<&[u8]>,
) -> Result<NormativeServingDecision, SourceUnavailable> {
    let mut engine = load_direct_engine(material)?;
    engine
        .predict_with_session_signature(context, session_signature)
        .map_err(|error| SourceUnavailable::new(format!("cross-surface direct step: {error}")))
}

fn load_r4g1_state(
    material: CanonicalCrossSurfaceMaterial<'_>,
) -> Result<R4g1State, SourceUnavailable> {
    let captured = CapturedR4g1Bundle {
        graph: material.graph.to_vec(),
        signature_artifact: material.signature_artifact.to_vec(),
        tokenizer: material.tokenizer.map(ToOwned::to_owned),
        score_report: material.score_report.map(ToOwned::to_owned),
        production_admission: None,
    };
    R4g1State::load_captured_for_research_with_source(
        Path::new("cross-surface/graph/score.r4g1"),
        Path::new("cross-surface/tless_artifacts.bin"),
        &captured,
        None,
    )
    .map_err(SourceUnavailable::new)
}

fn served_token(decision: NormativeServingDecision) -> Result<u32, SourceUnavailable> {
    match decision {
        NormativeServingDecision::Serve(serve) => Ok(serve.token),
        NormativeServingDecision::Abstain(_) => Err(SourceUnavailable::new(
            "cross-surface selected position abstained",
        )),
        NormativeServingDecision::Decline(_) => Err(SourceUnavailable::new(
            "cross-surface selected position declined",
        )),
    }
}

fn served_outcome(
    decision: NormativeServingDecision,
) -> Result<uor_r4_api::NormativeServe, SourceUnavailable> {
    match decision {
        NormativeServingDecision::Serve(serve) => Ok(serve),
        NormativeServingDecision::Abstain(_) => Err(SourceUnavailable::new(
            "cross-surface observed position abstained",
        )),
        NormativeServingDecision::Decline(_) => Err(SourceUnavailable::new(
            "cross-surface observed position declined",
        )),
    }
}

fn sampled_token(decision: NormativeServingDecision, seed: u32) -> Result<u32, SourceUnavailable> {
    match decision {
        NormativeServingDecision::Serve(serve) => {
            let mut rng = SampleRng::new(seed);
            Ok(serve.select_sampled_token(&[], &mut rng))
        }
        NormativeServingDecision::Abstain(_) => Err(SourceUnavailable::new(
            "cross-surface sampled position abstained",
        )),
        NormativeServingDecision::Decline(_) => Err(SourceUnavailable::new(
            "cross-surface sampled position declined",
        )),
    }
}

fn sampled_policy(seed: u32) -> String {
    format!("default-sampled-seed-{seed}")
}

fn context_window(corpus: &Corpus, position: usize) -> Vec<u32> {
    let mut start = position;
    while start > 0
        && corpus.story[start - 1] == corpus.story[position]
        && position + 1 - start < WINDOW
    {
        start -= 1;
    }
    (start..=position)
        .map(|index| corpus.input[index])
        .collect()
}

fn story_history(corpus: &Corpus, position: usize) -> Vec<u32> {
    let mut start = position;
    while start > 0 && corpus.story[start - 1] == corpus.story[position] {
        start -= 1;
    }
    (start..=position)
        .map(|index| corpus.input[index])
        .collect()
}
