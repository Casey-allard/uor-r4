//! Example direct-chat application built on R⁴ transformerless inference.
//!
//! Chat is a consumer of the core runtime, not a separate inference layer.

use std::fmt;
use std::io::{BufRead, Read, Write};

use crate::model::{default_model_reference, ModelError, ModelObject, ModelStore};
use uor_r4_core::transformerless::compiler::{self, Compiled};
use uor_r4_core::transformerless::runtime::{self, Runtime, Store};
use uor_r4_core::transformerless::scenarios::Tokenizer;

const MAX_CHAT_TOKENS: usize = 256;
const MAX_CHAT_HISTORY: usize = 4096;
const MAX_ANSWER_BYTES: usize = 16 * 1024;

/// A completed local chat turn.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatAnswer {
    /// Generated assistant text.
    pub text: String,
    /// Number of tokens generated for this turn.
    pub generated_tokens: usize,
    /// Fraction of generated tokens that already appeared earlier in
    /// this same generation (see [`recent_window_repetition_rate`],
    /// called with the full generated length as its window — a bounded
    /// chat turn is short enough that "recent" can mean "the whole
    /// answer so far"). A fixed 32-token window, the convention Gate C's
    /// `generate_greedy_repetition_rate` uses for its much longer
    /// generations, empirically under-detects real degenerate output
    /// here: `#744`'s own verification against the actual `#745`
    /// word-salad bundle measured only 0.10-0.25 at window 32, because
    /// the small pool of recurring fragments (`tetra`/`gru`/`caption`/…)
    /// mixes with enough distinct short filler subword pieces that few
    /// *exact same-token* recurrences land inside any single 32-token
    /// slice, even though the visible vocabulary is obviously collapsed
    /// to a human reader. Widening the window to the full answer raised
    /// the same transcripts to 0.31-0.41, which is why
    /// [`crate::model::evaluate_live_quality`]'s repetition bar was
    /// recalibrated down from 0.5 to 0.25 alongside this change — see
    /// that function's bar constant for the honest caveat on how that
    /// number was chosen. Unlike
    /// [`repeated_suffix`]'s exact-block detector (this module's other
    /// repetition guard), this also catches small-vocabulary cycling
    /// through several distinct tokens in varying order, which never
    /// forms an identical repeated block.
    pub repeated_token_rate: f64,
}

/// Failure to load or run the local transformerless chat engine.
#[derive(Debug)]
#[non_exhaustive]
pub enum ChatError {
    /// A required file could not be read.
    Io(std::io::Error),
    /// The compiled artifact container was invalid.
    InvalidArtifacts,
    /// The graded store container was invalid.
    InvalidStore,
    /// Generation produced no tokens, or the question/answer could not be
    /// tokenized or decoded into its caller-owned buffer.
    EmptyGeneration,
    /// The bundle carries a tagged decode-only tokenizer, but direct chat has
    /// no exact registered host encoder for that family/version.
    TokenizerUnavailable { family: String, version: u32 },
    /// Generation entered a repeated-token loop and was rejected.
    RepetitiveGeneration,
    /// No CID-addressed, capability-attested model was selected.
    MissingModel,
    /// The model bundle or its CID verification failed.
    Model(ModelError),
}

impl fmt::Display for ChatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to load chat data: {error}"),
            Self::InvalidArtifacts => formatter.write_str("invalid transformerless artifacts"),
            Self::InvalidStore => formatter.write_str("invalid transformerless store"),
            Self::EmptyGeneration => formatter.write_str("transformerless produced no text"),
            Self::TokenizerUnavailable { family, version } => write!(
                formatter,
                "tokenizer unavailable: exact host adapter {family}/{version} is required"
            ),
            Self::RepetitiveGeneration => formatter.write_str(
                "transformerless generation became repetitive; refusing a low-quality answer",
            ),
            Self::MissingModel => {
                formatter.write_str("no chat model selected; set TLESS_MODEL or pass --model")
            }
            Self::Model(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ChatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidArtifacts
            | Self::InvalidStore
            | Self::EmptyGeneration
            | Self::TokenizerUnavailable { .. }
            | Self::RepetitiveGeneration
            | Self::MissingModel => None,
            Self::Model(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ChatError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ModelError> for ChatError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

/// Builder for a direct local [`ChatEngine`].
#[derive(Debug, Clone)]
pub struct ChatEngineBuilder {
    max_tokens: usize,
    model: Option<String>,
}

impl Default for ChatEngineBuilder {
    fn default() -> Self {
        Self {
            max_tokens: 96,
            model: Some(default_model_reference()),
        }
    }
}

impl ChatEngineBuilder {
    /// Set the maximum number of generated tokens per turn.
    #[must_use]
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens.clamp(1, MAX_CHAT_TOKENS);
        self
    }

    /// Select a CID-addressed model manifest by name or UOR CID.
    #[must_use]
    pub fn model(mut self, reference: impl Into<String>) -> Self {
        self.model = Some(reference.into());
        self
    }

    /// Load all local data and construct the engine.
    pub fn build(self) -> Result<ChatEngine, ChatError> {
        let reference = self.model.as_deref().ok_or(ChatError::MissingModel)?;
        let model_store = ModelStore::from_env();
        let manifest = match model_store.read_manifest(reference) {
            Ok(manifest) => manifest,
            Err(ModelError::CompiledNotImported(path)) => {
                return build_local_compiled_engine(
                    &model_store,
                    &path,
                    reference,
                    self.max_tokens,
                );
            }
            Err(error) => return Err(error.into()),
        };
        manifest.validate_for_chat()?;
        if let Some(report) = &manifest.evaluation_report {
            let _ = model_store.get(report)?;
        }
        let artifact_bytes = model_store.get(&manifest.artifacts)?;
        let artifacts =
            compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
        let store_bytes = model_store.get(&manifest.store)?;
        let store = runtime::parse_store(&store_bytes).ok_or(ChatError::InvalidStore)?;
        let r4g1_path = std::path::PathBuf::from(format!(
            ".uor-models/compiled/{}/compiled.r4g1",
            manifest.name
        ));
        let r4g1_bytes = read_optional_chat_file(&r4g1_path)?;
        validate_chat_r4g1_structure(r4g1_bytes.as_deref())?;
        let bundled_tokenizer = match model_store.get(&manifest.tokenizer) {
            Ok(bytes) => bytes,
            Err(error) => match matching_ref_tokenizer(&manifest.tokenizer, r4g1_bytes.as_deref())?
            {
                Some(bytes) => bytes,
                None => return Err(error.into()),
            },
        };
        let tokenizer_bytes = select_chat_tokenizer_bytes(&bundled_tokenizer)?;
        let r4g1_bytes = bind_chat_r4g1(r4g1_bytes, &tokenizer_bytes)?;
        let tokenizer = parse_chat_tokenizer(&tokenizer_bytes)?;
        tracing::info!(
            model = %manifest.name,
            source_model = %manifest.source_model,
            artifact_cid = %manifest.artifacts.cid,
            store_cid = %manifest.store.cid,
            r4g1_loaded = r4g1_bytes.is_some(),
            max_tokens = self.max_tokens,
            "transformerless chat engine loaded"
        );
        Ok(ChatEngine {
            artifacts,
            store,
            r4g1_bytes,
            tokenizer,
            history: [0; MAX_CHAT_HISTORY],
            history_len: 0,
            max_tokens: self.max_tokens,
        })
    }
}

fn build_local_compiled_engine(
    model_store: &ModelStore,
    directory: &std::path::Path,
    reference: &str,
    max_tokens: usize,
) -> Result<ChatEngine, ChatError> {
    let artifact_bytes = std::fs::read(directory.join("tless_artifacts.bin"))?;
    let store_bytes = std::fs::read(directory.join("tless_store.bin"))?;
    let r4g1_bytes = read_optional_chat_file(&directory.join("compiled.r4g1"))?;
    validate_chat_r4g1_structure(r4g1_bytes.as_deref())?;
    let tok_file = directory.join("tokenizer.bin");
    tracing::debug!(?tok_file, "resolved chat tokenizer path");
    let bundled_tokenizer = std::fs::read(&tok_file)?;
    let tokenizer_bytes = select_chat_tokenizer_bytes(&bundled_tokenizer)?;
    let r4g1_bytes = bind_chat_r4g1(r4g1_bytes, &tokenizer_bytes)?;
    let artifacts =
        compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
    let store = runtime::parse_store(&store_bytes).ok_or(ChatError::InvalidStore)?;

    // Content-address all local compiler outputs immediately. A manifest and
    // quality report remain optional metadata; integrity does not.
    let artifact_object = model_store.put(&artifact_bytes)?;
    let store_object = model_store.put(&store_bytes)?;
    let tokenizer_object = model_store.put(&tokenizer_bytes)?;
    let tokenizer = parse_chat_tokenizer(&tokenizer_bytes)?;
    tracing::warn!(
        model = reference,
        directory = %directory.display(),
        artifact_cid = %artifact_object.cid,
        store_cid = %store_object.cid,
        tokenizer_cid = %tokenizer_object.cid,
        "using a locally compiled bundle without an instruction-quality attestation"
    );
    Ok(ChatEngine {
        artifacts,
        store,
        r4g1_bytes,
        tokenizer,
        history: [0; MAX_CHAT_HISTORY],
        history_len: 0,
        max_tokens,
    })
}

/// Stateful local chat engine with no HTTP server or background worker.
pub struct ChatEngine {
    artifacts: Compiled,
    store: Store,
    r4g1_bytes: Option<Vec<u8>>,
    tokenizer: Tokenizer,
    history: [u32; MAX_CHAT_HISTORY],
    history_len: usize,
    max_tokens: usize,
}

impl ChatEngine {
    /// Start configuring a local chat engine.
    #[must_use]
    pub fn builder() -> ChatEngineBuilder {
        ChatEngineBuilder::default()
    }

    /// Generate one answer and retain its tokens as context for the next turn.
    pub fn ask(&mut self, question: &str) -> Result<ChatAnswer, ChatError> {
        let span = tracing::debug_span!("ask", question_bytes = question.len());
        let _guard = span.enter();
        hologram_answer(
            &self.artifacts,
            &self.store,
            self.r4g1_bytes.as_deref(),
            &self.tokenizer,
            &mut self.history,
            &mut self.history_len,
            question,
            self.max_tokens,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn hologram_answer(
    artifacts: &Compiled,
    store: &Store,
    r4g1_bytes: Option<&[u8]>,
    tokenizer: &Tokenizer,
    history: &mut [u32; MAX_CHAT_HISTORY],
    history_len: &mut usize,
    question: &str,
    max_tokens: usize,
) -> Result<ChatAnswer, ChatError> {
    ensure_chat_prompt_encoder(tokenizer)?;
    let mut question_tokens = [0u32; MAX_CHAT_HISTORY];
    let question_count = tokenizer
        .encode_into(question, &mut question_tokens)
        .ok_or(ChatError::EmptyGeneration)?;
    let question_tokens = if *history_len == 0 {
        &question_tokens[..question_count]
    } else {
        &question_tokens[1..question_count]
    };
    append_history(history, history_len, question_tokens);
    let session_signature = uor_r4_router::session_signature_from_tokens(&history[..*history_len]);

    if let Some(bytes) = r4g1_bytes {
        if let Ok(r4g1) = uor_r4_graph_runtime::R4G1Runtime::parse(bytes) {
            let rot = compiler::derive_rotations();
            let num_nodes = r4g1.node_count() as usize;
            let mut node_scores =
                vec![uor_r4_core::transformerless::score_q::ScoreQ::MIN; num_nodes];

            struct BeamHypothesis {
                tokens: Vec<u32>,
                score: i32,
                terminated: bool,
            }

            let mut beams = vec![BeamHypothesis {
                tokens: Vec::new(),
                score: 0,
                terminated: false,
            }];

            let steps = max_tokens.min(MAX_CHAT_TOKENS);
            for _ in 0..steps {
                let mut all_candidates = Vec::new();
                let mut any_active = false;

                for beam in &beams {
                    if beam.terminated {
                        all_candidates.push(BeamHypothesis {
                            tokens: beam.tokens.clone(),
                            score: beam.score,
                            terminated: true,
                        });
                        continue;
                    }
                    any_active = true;

                    let mut beam_history = history[..*history_len].to_vec();
                    beam_history.extend_from_slice(&beam.tokens);

                    let len = core::cmp::min(beam_history.len(), compiler::WINDOW);
                    let window = &beam_history[beam_history.len() - len..];
                    let bundle = runtime::bundle_window_plain(artifacts, &rot, window);
                    let sig = runtime::sig_plain(artifacts, &bundle);

                    let mut cands =
                        [(0u32, uor_r4_core::transformerless::score_q::ScoreQ::ZERO); 8];
                    let num_cands = r4g1.predict_candidates_with_signature_lanes(
                        &beam_history,
                        Some(&sig),
                        Some(&session_signature),
                        &mut node_scores,
                        &mut cands,
                    );

                    for &(cand_tok, cand_score) in &cands[..num_cands] {
                        let is_eos = cand_tok == 0 || cand_tok == 2;
                        let mut new_tokens = beam.tokens.clone();

                        let mut repeat_count = 0i32;
                        for &t in new_tokens.iter().rev() {
                            if t == cand_tok {
                                repeat_count += 1;
                            } else {
                                break;
                            }
                        }
                        let repeat_penalty = if repeat_count > 0 {
                            repeat_count * 3000
                        } else {
                            0
                        };

                        let adjusted_score = beam
                            .score
                            .saturating_add(cand_score.raw())
                            .saturating_sub(repeat_penalty);
                        new_tokens.push(cand_tok);

                        all_candidates.push(BeamHypothesis {
                            tokens: new_tokens,
                            score: adjusted_score,
                            terminated: is_eos,
                        });
                    }
                }

                if !any_active || all_candidates.is_empty() {
                    break;
                }

                all_candidates.sort_by_key(|b| std::cmp::Reverse(b.score));
                all_candidates.truncate(4);
                beams = all_candidates;
            }

            let best_beam = beams
                .into_iter()
                .max_by_key(|b| b.score)
                .unwrap_or_else(|| BeamHypothesis {
                    tokens: Vec::new(),
                    score: 0,
                    terminated: false,
                });

            let generated_tokens_buf = best_beam.tokens;
            let generated = generated_tokens_buf.as_slice();
            append_history(history, history_len, generated);
            if generated.is_empty() {
                return Err(ChatError::EmptyGeneration);
            }
            let mut answer_bytes = [0u8; MAX_ANSWER_BYTES];
            let answer_len = tokenizer
                .decode_into(generated, &mut answer_bytes)
                .ok_or(ChatError::EmptyGeneration)?;
            let text = String::from_utf8_lossy(&answer_bytes[..answer_len])
                .trim()
                .to_owned();
            if text.is_empty() {
                return Err(ChatError::EmptyGeneration);
            }
            tracing::debug!(generated_tokens = generated.len(), "R4G1 answer generated");
            return Ok(ChatAnswer {
                text,
                generated_tokens: generated.len(),
                repeated_token_rate: recent_window_repetition_rate(generated, generated.len()),
            });
        }
    }

    let mut runtime = Runtime::new(artifacts);
    let mut predictions = [runtime::Prediction::default(); MAX_CHAT_TOKENS];
    let prediction_count = runtime.generate_greedy_into(
        store,
        &history[..*history_len],
        &mut predictions[..max_tokens.min(MAX_CHAT_TOKENS)],
    );
    let mut generated = [0u32; MAX_CHAT_TOKENS];
    let mut generated_count = 0usize;
    for prediction in &predictions[..prediction_count] {
        if prediction.token == 1 {
            break;
        }
        generated[generated_count] = prediction.token;
        generated_count += 1;
        if repeated_suffix(&generated[..generated_count], 8) {
            generated_count -= 1;
            break;
        }
    }
    let generated = &generated[..generated_count];
    if generated.is_empty() {
        return Err(ChatError::EmptyGeneration);
    }
    let mut answer_bytes = [0u8; MAX_ANSWER_BYTES];
    let answer_len = tokenizer
        .decode_into(generated, &mut answer_bytes)
        .ok_or(ChatError::EmptyGeneration)?;
    let text = String::from_utf8_lossy(&answer_bytes[..answer_len])
        .trim()
        .to_owned();
    if text.is_empty() {
        return Err(ChatError::EmptyGeneration);
    }
    append_history(history, history_len, generated);
    tracing::debug!(generated_tokens = generated.len(), "answer generated");
    Ok(ChatAnswer {
        text,
        generated_tokens: generated.len(),
        repeated_token_rate: recent_window_repetition_rate(generated, generated.len()),
    })
}

fn ensure_chat_prompt_encoder(tokenizer: &Tokenizer) -> Result<(), ChatError> {
    match tokenizer.adapter_key() {
        Some((family, version)) if tokenizer.is_decode_only() => {
            Err(ChatError::TokenizerUnavailable {
                family: family.to_owned(),
                version,
            })
        }
        _ => Ok(()),
    }
}

fn invalid_chat_data(message: impl Into<String>) -> ChatError {
    ChatError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    ))
}

fn read_optional_chat_file(path: &std::path::Path) -> Result<Option<Vec<u8>>, ChatError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ChatError::Io(error)),
    };
    let is_regular = if metadata.file_type().is_symlink() {
        std::fs::metadata(path)?.is_file()
    } else {
        metadata.is_file()
    };
    if !is_regular {
        return Err(invalid_chat_data(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    std::fs::read(path).map(Some).map_err(ChatError::Io)
}

fn validate_chat_r4g1_structure(graph: Option<&[u8]>) -> Result<(), ChatError> {
    let Some(graph) = graph else {
        return Ok(());
    };
    let view = uor_r4_graph_format::GraphView::parse(graph)
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.reason)))?;
    view.verify_cids()
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.as_format())))?;
    if view.head().is_none() {
        return Err(invalid_chat_data(
            "invalid R4G1 graph: missing HEAD section",
        ));
    }
    uor_r4_graph_runtime::R4G1Runtime::parse(graph)
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 runtime: {}", error.reason)))?;
    Ok(())
}

fn bind_chat_r4g1(
    graph: Option<Vec<u8>>,
    tokenizer_bytes: &[u8],
) -> Result<Option<Vec<u8>>, ChatError> {
    let Some(graph) = graph else {
        return Ok(None);
    };
    let view = uor_r4_graph_format::GraphView::parse(&graph)
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.reason)))?;
    view.verify_cids()
        .map_err(|error| invalid_chat_data(format!("invalid R4G1 graph: {}", error.as_format())))?;
    let expected = view
        .head()
        .ok_or_else(|| invalid_chat_data("invalid R4G1 graph: missing HEAD section"))?
        .tokenizer_cid()
        .0;
    let actual = blake3::hash(tokenizer_bytes);
    if expected != [0; 32] && expected != *actual.as_bytes() {
        return Err(invalid_chat_data(format!(
            "R4G1 tokenizer CID mismatch: header expected blake3:{}, selected blake3:{actual}",
            blake3::Hash::from(expected).to_hex()
        )));
    }
    if expected == [0; 32] && Tokenizer::is_tagged_container_bytes(tokenizer_bytes) {
        return Err(invalid_chat_data(
            "a tagged tokenizer requires a nonzero R4G1 header tokenizer CID",
        ));
    }
    Ok(Some(graph))
}

/// Validate and retain the exact tokenizer object selected by the bundle.
/// Present bytes are never replaced by `/tmp/ref/tokenizer.bin`: doing so
/// would write different content under the manifest CID and could cross
/// tokenizer id spaces.
fn select_chat_tokenizer_bytes(bundled: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    if Tokenizer::is_tagged_container_bytes(bundled) {
        let tokenizer = Tokenizer::from_bytes(bundled).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "malformed tagged tokenizer.bin",
            )
        })?;
        if !tokenizer.is_decode_only() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "tagged tokenizer.bin did not declare decode-only semantics",
            ));
        }
        return Ok(bundled.to_vec());
    }
    if Tokenizer::from_bytes(bundled).is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "malformed tokenizer.bin",
        ));
    }
    Ok(bundled.to_vec())
}

/// Compatibility path for historical 32k bundles whose content-addressed
/// object is temporarily unavailable. The old `/tmp/ref` shortcut is admitted
/// only when its bytes exactly match the manifest object, so an unavailable
/// tagged object can never be replaced by unrelated legacy bytes.
fn matching_ref_tokenizer(
    expected: &ModelObject,
    r4g1_bytes: Option<&[u8]>,
) -> Result<Option<Vec<u8>>, ChatError> {
    let is_32k_graph = r4g1_bytes.is_some_and(|bytes| {
        uor_r4_graph_runtime::R4G1Runtime::parse(bytes)
            .is_ok_and(|runtime| runtime.node_count() > 0)
    });
    if !is_32k_graph {
        return Ok(None);
    }
    let Some(bytes) = read_optional_chat_file(std::path::Path::new("/tmp/ref/tokenizer.bin"))?
    else {
        return Ok(None);
    };
    if u64::try_from(bytes.len()).ok() != Some(expected.bytes) {
        return Ok(None);
    }
    let actual = format!("blake3:{}", blake3::hash(&bytes).to_hex());
    Ok((actual == expected.cid).then_some(bytes))
}

fn append_history(history: &mut [u32; MAX_CHAT_HISTORY], len: &mut usize, tokens: &[u32]) {
    let tokens = &tokens[tokens.len().saturating_sub(MAX_CHAT_HISTORY)..];
    let overflow = len
        .saturating_add(tokens.len())
        .saturating_sub(MAX_CHAT_HISTORY);
    if overflow > 0 {
        history.copy_within(overflow..*len, 0);
        *len -= overflow;
    }
    history[*len..*len + tokens.len()].copy_from_slice(tokens);
    *len += tokens.len();
}

fn repeated_suffix(tokens: &[u32], width: usize) -> bool {
    if tokens.len() < width * 2 {
        return false;
    }
    let suffix = &tokens[tokens.len() - width..];
    tokens[..tokens.len() - width]
        .windows(width)
        .any(|window| window == suffix)
}

/// Fraction of `tokens` that already appeared among the preceding
/// `window` tokens of the same generation — the same recent-window
/// repetition definition Gate C's `generate_greedy_repetition_rate` uses
/// (`uor-r4-graph-certify::score`), reimplemented here for the live
/// serving path (`chat`'s `Compiled`/`Store` types are private to
/// `uor-r4-core::transformerless`, a different crate boundary than
/// graph-certify's `GraphScorer`, so the definition is duplicated rather
/// than shared code — see `#744`). Empty input is defined as
/// non-repetitive (`0.0`), matching the natural reading of "no
/// repetition observed" rather than the alternative "undefined".
fn recent_window_repetition_rate(tokens: &[u32], window: usize) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }
    let mut recent: std::collections::VecDeque<u32> =
        std::collections::VecDeque::with_capacity(window);
    let mut duplicate_count = 0usize;
    for &token in tokens {
        if recent.contains(&token) {
            duplicate_count += 1;
        }
        if recent.len() == window {
            recent.pop_front();
        }
        recent.push_back(token);
    }
    duplicate_count as f64 / tokens.len() as f64
}

/// Build a chat engine directly from raw artifact/store/tokenizer bytes,
/// bypassing manifest lookup and the on-disk compiled-bundle directory
/// convention (`build_local_compiled_engine`'s fixed file names). Used by
/// `model::evaluate_live_quality` (`#744`) to measure generation quality
/// against the exact bytes being imported, before any manifest or
/// `.uor-models/compiled/<name>` directory exists for them. No R4G1
/// graph is attempted (`r4g1_bytes: None`) — this always exercises the
/// plain TLA/TLS1 runtime path, the common baseline every bundle must
/// clear regardless of which runtime ultimately serves it, and the one
/// whose repetition guard (`repeated_suffix`) is weaker, making it the
/// conservative choice for a pass/fail gate.
pub fn engine_from_bytes(
    artifact_bytes: &[u8],
    store_bytes: &[u8],
    tokenizer_bytes: &[u8],
    max_tokens: usize,
) -> Result<ChatEngine, ChatError> {
    let artifacts = compiler::parse_artifacts(artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
    let store = runtime::parse_store(store_bytes).ok_or(ChatError::InvalidStore)?;
    let tokenizer = parse_chat_tokenizer(tokenizer_bytes)?;
    Ok(ChatEngine {
        artifacts,
        store,
        r4g1_bytes: None,
        tokenizer,
        history: [0; MAX_CHAT_HISTORY],
        history_len: 0,
        max_tokens: max_tokens.clamp(1, MAX_CHAT_TOKENS),
    })
}

fn parse_chat_tokenizer(bytes: &[u8]) -> Result<Tokenizer, ChatError> {
    Tokenizer::from_bytes(bytes).ok_or_else(|| {
        ChatError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid tokenizer.bin bytes",
        ))
    })
}

struct SlashCommandDef {
    cmd: &'static str,
    desc: &'static str,
}

const COMMAND_DEFS: &[SlashCommandDef] = &[
    SlashCommandDef {
        cmd: "/help",
        desc: "Display available client slash commands",
    },
    SlashCommandDef {
        cmd: "/status",
        desc: "View R4G1 sub-millisecond 4-stage pipeline readiness",
    },
    SlashCommandDef {
        cmd: "/models",
        desc: "List supported teacher models & disk compilation status",
    },
    SlashCommandDef {
        cmd: "/switch",
        desc: "Dynamically switch active teacher model in-session",
    },
    SlashCommandDef {
        cmd: "/engine",
        desc: "Select synthesis engine (r4g1, attention, r4-attention, geometric)",
    },
    SlashCommandDef {
        cmd: "/corpus",
        desc: "Manage extra reading corpus datasets & server index",
    },
    SlashCommandDef {
        cmd: "/compile",
        desc: "Trigger full automated 4-stage graph compilation",
    },
    SlashCommandDef {
        cmd: "/audit",
        desc: "Audit Q&A token trace, UOR coordinates & R4 geometry",
    },
    SlashCommandDef {
        cmd: "/clear",
        desc: "Clear terminal screen & session history",
    },
    SlashCommandDef {
        cmd: "/reset",
        desc: "Reset history, corpus & geometric manifold state back to base",
    },
    SlashCommandDef {
        cmd: "/export",
        desc: "Export manifold state & corpus to .uor-models/exported/exported_manifold.json",
    },
    SlashCommandDef {
        cmd: "/quit",
        desc: "Exit client session",
    },
    SlashCommandDef {
        cmd: "/exit",
        desc: "Exit client session",
    },
];
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
use rustyline::completion::Completer;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::highlight::Highlighter;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::hint::Hinter;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::validate::Validator;
#[cfg(not(target_arch = "wasm32"))]
use rustyline::Helper;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct SlashCommandHelper;

#[cfg(not(target_arch = "wasm32"))]
impl Completer for SlashCommandHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        if line.starts_with('/') {
            let candidates: Vec<String> = COMMAND_DEFS
                .iter()
                .filter(|d| d.cmd.starts_with(&line[..pos]))
                .map(|d| d.cmd.to_string())
                .collect();
            Ok((0, candidates))
        } else {
            Ok((0, Vec::new()))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Hinter for SlashCommandHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Highlighter for SlashCommandHelper {}
#[cfg(not(target_arch = "wasm32"))]
impl Validator for SlashCommandHelper {}
#[cfg(not(target_arch = "wasm32"))]
impl Helper for SlashCommandHelper {}

fn read_line_with_history<W: Write>(
    prompt: &str,
    history: &mut Vec<String>,
    input: &mut impl BufRead,
    output: &mut W,
) -> Result<Option<String>, std::io::Error> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if unsafe { libc::isatty(libc::STDIN_FILENO) } != 0 {
            let config = rustyline::Config::builder()
                .auto_add_history(true)
                .completion_type(rustyline::CompletionType::List)
                .build();
            let mut rl = rustyline::Editor::<SlashCommandHelper, _>::with_config(config)
                .map_err(std::io::Error::other)?;
            rl.set_helper(Some(SlashCommandHelper));
            for entry in history.iter() {
                let _ = rl.add_history_entry(entry);
            }

            match rl.readline(prompt) {
                Ok(line) => {
                    let trimmed = line.trim().to_string();
                    if !trimmed.is_empty() {
                        history.push(trimmed.clone());
                    }
                    return Ok(Some(trimmed));
                }
                Err(rustyline::error::ReadlineError::Interrupted) => return Ok(None),
                Err(rustyline::error::ReadlineError::Eof) => return Ok(None),
                Err(e) => return Err(std::io::Error::other(e)),
            }
        }
    }

    write!(output, "{}", prompt)?;
    output.flush()?;

    let mut line_bytes = Vec::new();
    loop {
        let mut buf = [0u8; 1];
        let n = input.read(&mut buf)?;
        if n == 0 {
            if line_bytes.is_empty() {
                return Ok(None);
            }
            break;
        }
        let b = buf[0];
        if b == b'\r' || b == b'\n' {
            break;
        }
        line_bytes.push(b);
    }

    let line = String::from_utf8_lossy(&line_bytes);
    let trimmed = line.trim().to_string();
    if !trimmed.is_empty() {
        history.push(trimmed.clone());
    }
    Ok(Some(trimmed))
}

fn select_menu_interactive<W: Write>(
    title: &str,
    options: &[(&str, &str)],
    output: &mut W,
) -> Result<Option<usize>, std::io::Error> {
    writeln!(output, "\n\x1b[1m{}\x1b[0m", title)?;
    for (idx, (item_name, item_desc)) in options.iter().enumerate() {
        writeln!(
            output,
            "  \x1b[1;36m[{}]\x1b[0m \x1b[1m{:<24}\x1b[0m {}",
            idx + 1,
            item_name,
            item_desc
        )?;
    }
    write!(
        output,
        "\x1b[1;33mSelect option [1-{}]: \x1b[0m",
        options.len()
    )?;
    output.flush()?;

    let mut history_dummy = Vec::new();
    let mut stdin_buf = std::io::BufReader::new(std::io::stdin());
    let resp = read_line_with_history("", &mut history_dummy, &mut stdin_buf, output)?;
    if let Some(line) = resp {
        let trimmed = line.trim();
        if let Ok(num) = trimmed.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                return Ok(Some(num - 1));
            }
        }
        for (idx, (item_name, _)) in options.iter().enumerate() {
            if item_name
                .trim_start_matches('/')
                .eq_ignore_ascii_case(trimmed.trim_start_matches('/'))
            {
                return Ok(Some(idx));
            }
        }
    }
    Ok(None)
}

fn check_model_artifact_status(model_id: &str) -> (bool, bool) {
    let target_key = match model_id {
        "smollm2-135m-instruct" => "smollm2-135m",
        "smollm2-360m-instruct" => "smollm2-360m",
        "smollm2-1-7b-instruct" => "smollm2-1-7b",
        other => other,
    };

    let downloaded = if let Ok(entries) = std::fs::read_dir(".uor-models/sources") {
        entries.filter_map(|e| e.ok()).any(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            entry.path().is_dir() && name.contains(target_key)
        })
    } else {
        false
    };

    let compiled = if let Ok(entries) = std::fs::read_dir(".uor-models/compiled") {
        entries.filter_map(|e| e.ok()).any(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            path.is_dir()
                && name.contains(target_key)
                && (path.join("tless_artifacts.bin").is_file()
                    || path.join("graph/score.r4g1").is_file()
                    || path.join("compiled.r4g1").is_file())
        })
    } else {
        false
    };

    (downloaded, compiled)
}

fn trigger_in_client_compilation<W: Write>(
    target_model: &str,
    host: &str,
    port: u16,
    output: &mut W,
) -> Result<bool, std::io::Error> {
    let (repo, rev) = match target_model {
        "smollm2-360m-instruct" => (
            "HuggingFaceTB/SmolLM2-360M-Instruct",
            "9d9ff7299a9a3b6d289ff100d0246a48d88c0326",
        ),
        "smollm2-1-7b-instruct" => ("HuggingFaceTB/SmolLM2-1.7B-Instruct", "main"),
        _ => (
            "HuggingFaceTB/SmolLM2-135M-Instruct",
            "7e27bd9f95328f0f3b08261d1252705110c806f8",
        ),
    };

    let r4_exe =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./target/release/r4"));
    let source_dir = format!(".uor-models/sources/{}", target_model);
    let compiled_dir = format!(".uor-models/compiled/{}", target_model);
    let graph_dir = format!("{}/graph", compiled_dir);
    let score_file = format!("{}/score.r4g1", graph_dir);

    writeln!(
        output,
        "\n\x1b[1;36m[*] Initiating automated 4-stage graph compilation for '{}'...\x1b[0m",
        target_model
    )?;

    // Stage 1: Download if missing
    if !std::path::Path::new(&source_dir).is_dir() {
        writeln!(
            output,
            "[*] [Stage 1/4] Downloading HF teacher weights ({})",
            repo
        )?;
        output.flush()?;
        let status = std::process::Command::new(&r4_exe)
            .args([
                "download",
                "--repository",
                repo,
                "--revision",
                rev,
                "--name",
                target_model,
            ])
            .status()?;
        if !status.success() {
            writeln!(output, "\x1b[31m[!] Stage 1 download failed.\x1b[0m")?;
            return Ok(false);
        }
    } else {
        writeln!(
            output,
            "\x1b[32m[✓] [Stage 1/4] Pinned teacher source ready: {}\x1b[0m",
            source_dir
        )?;
    }

    // Stage 2: Compile observation corpus. Projection ownership is fixed by
    // the loaded architecture; the legacy --exact-scalar switch is a no-op.
    writeln!(
        output,
        "[*] [Stage 2/4] Compiling zero-multiply observation corpus (exact uor-matmul projections; certified-exact current attention)..."
    )?;
    output.flush()?;
    std::fs::create_dir_all(&compiled_dir).ok();
    std::fs::create_dir_all(&graph_dir).ok();

    let (target_tokens, compile_seconds) = match target_model {
        "smollm2-135m-instruct" => ("176800", "300"),
        "smollm2-360m-instruct" => ("500000", "600"),
        "smollm2-1-7b-instruct" => ("1768000", "1200"),
        _ => ("176800", "300"),
    };

    let compile_cmd_args = vec![
        "compile".to_string(),
        "--source".to_string(),
        source_dir.clone(),
        "--output".to_string(),
        compiled_dir.clone(),
        "--seconds".to_string(),
        compile_seconds.to_string(),
        "--target".to_string(),
        target_tokens.to_string(),
        "--sequence-length".to_string(),
        "128".to_string(),
    ];
    let status = std::process::Command::new(&r4_exe)
        .args(&compile_cmd_args)
        .status()?;
    if !status.success() {
        writeln!(
            output,
            "\x1b[31m[!] Stage 2 bundle compilation failed.\x1b[0m"
        )?;
        return Ok(false);
    }
    writeln!(
        output,
        "\x1b[32m[✓] [Stage 2/4] Transformerless bundle compiled successfully.\x1b[0m"
    )?;

    // Stage 3: Score residual graph
    writeln!(
        output,
        "[*] [Stage 3/4] Inducing multiresolution cover & scoring R4G1 residual graph..."
    )?;
    output.flush()?;
    let c_meta = if std::path::Path::new(&format!("{}/corpus.meta", compiled_dir)).is_file() {
        format!("{}/corpus.meta", compiled_dir)
    } else {
        format!("{}/c_meta.bin", compiled_dir)
    };

    let c_recs = if std::path::Path::new(&format!("{}/corpus.records", compiled_dir)).is_file() {
        format!("{}/corpus.records", compiled_dir)
    } else {
        format!("{}/c_recs.bin", compiled_dir)
    };
    let tless_artifacts = format!("{}/tless_artifacts.bin", compiled_dir);

    let mut status = std::process::Command::new(&r4_exe)
        .args([
            "transformerless",
            "score",
            "--corpus-meta",
            &c_meta,
            "--corpus-recs",
            &c_recs,
            "--artifacts",
            &tless_artifacts,
            "--quality-profile",
            "relative_tla",
            "--out",
            &graph_dir,
        ])
        .status()?;

    // If corpus was incomplete, re-run compile to finish remaining tokens then retry score
    if !status.success() {
        writeln!(
            output,
            "\x1b[33m[*] Finishing corpus generation to complete all required tokens...\x1b[0m"
        )?;
        output.flush()?;
        let _ = std::process::Command::new(&r4_exe)
            .args([
                "compile",
                "--source",
                &source_dir,
                "--output",
                &compiled_dir,
                "--seconds",
                compile_seconds,
                "--target",
                target_tokens,
                "--sequence-length",
                "128",
            ])
            .status()?;
        status = std::process::Command::new(&r4_exe)
            .args([
                "transformerless",
                "score",
                "--corpus-meta",
                &c_meta,
                "--corpus-recs",
                &c_recs,
                "--artifacts",
                &tless_artifacts,
                "--quality-profile",
                "relative_tla",
                "--out",
                &graph_dir,
            ])
            .status()?;
    }

    if !status.success() {
        writeln!(output, "\x1b[31m[!] Stage 3 graph scoring failed.\x1b[0m")?;
        return Ok(false);
    }
    writeln!(
        output,
        "\x1b[32m[✓] [Stage 3/4] Scored R4G1 residual graph ready: {}\x1b[0m",
        score_file
    )?;

    // Stage 4: Reload server
    writeln!(
        output,
        "[*] [Stage 4/4] Reloading server runtime with new R4G1 graph..."
    )?;
    output.flush()?;
    let req_body = serde_json::json!({ "model": target_model });
    match send_server_post_request(host, port, "/v1/reload", &req_body) {
        Ok(res) if res["status"] == "success" => {
            writeln!(
                output,
                "\x1b[1;32m[+] Compilation complete! Successfully loaded model '{}' in-session.\x1b[0m\n",
                target_model
            )?;
            Ok(true)
        }
        _ => {
            writeln!(
                output,
                "\x1b[31m[!] Server reload failed after compilation.\x1b[0m\n"
            )?;
            Ok(false)
        }
    }
}

fn handle_model_switch_with_remediation<W: Write>(
    target_model: &str,
    host: &str,
    port: u16,
    current_active_model: &mut String,
    current_active_engine: &mut String,
    output: &mut W,
) -> Result<(), std::io::Error> {
    writeln!(
        output,
        "\n[*] Requesting in-session server reload for model '{}'...",
        target_model
    )?;
    let req_body = serde_json::json!({ "model": target_model });
    match send_server_post_request(host, port, "/v1/reload", &req_body) {
        Ok(res) => {
            if res["status"] == "success" {
                *current_active_model = target_model.to_string();
                let _ = std::fs::write(
                    ".uor-models/last_model_name.txt",
                    current_active_model.as_str(),
                );
                writeln!(
                    output,
                    "\x1b[32m[+] {}\x1b[0m\n",
                    res["message"]
                        .as_str()
                        .unwrap_or("Model reloaded successfully")
                )?;
            } else {
                let err_msg = res["message"].as_str().unwrap_or("Failed to reload model");
                writeln!(
                    output,
                    "\n\x1b[1;31m┌─────────────────────────────────────────────────────────────────────────────┐\x1b[0m"
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m│ [!] MODEL RELOAD FAILURE & DIAGNOSTIC REMEDIATION:                         │\x1b[0m"
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m│\x1b[0m Target Model : \x1b[1m{:<60}\x1b[0m \x1b[1;31m│\x1b[0m",
                    target_model
                )?;
                let display_err = if err_msg.len() > 60 {
                    &err_msg[..60]
                } else {
                    err_msg
                };
                writeln!(
                    output,
                    "\x1b[1;31m│\x1b[0m Error        : \x1b[33m{:<60}\x1b[0m \x1b[1;31m│\x1b[0m",
                    display_err
                )?;
                writeln!(
                    output,
                    "\x1b[1;31m└─────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n"
                )?;

                let remediation_options = [
                    (
                        "1) Re-compile Model Graph",
                        "Re-run 4-stage compilation in-client to fix CID mismatch / out-of-date graph",
                    ),
                    (
                        "2) Switch to Oracle Mode",
                        "Switch engine to 'attention' oracle mode (runs model without graph)",
                    ),
                    (
                        "3) Keep Active Model",
                        "Cancel reload and stay on current working model",
                    ),
                ];

                if let Ok(Some(rem_idx)) = select_menu_interactive(
                    "Select Remediation Action:",
                    &remediation_options,
                    output,
                ) {
                    match rem_idx {
                        0 => {
                            if trigger_in_client_compilation(target_model, host, port, output)
                                .unwrap_or(false)
                            {
                                *current_active_model = target_model.to_string();
                                *current_active_engine = "r4g1".to_string();
                                let _ = std::fs::write(
                                    ".uor-models/last_model_name.txt",
                                    current_active_model.as_str(),
                                );
                                let _ = std::fs::write(
                                    ".uor-models/last_engine.txt",
                                    current_active_engine.as_str(),
                                );
                            }
                        }
                        1 => {
                            *current_active_engine = "attention".to_string();
                            let _ = std::fs::write(
                                ".uor-models/last_engine.txt",
                                current_active_engine.as_str(),
                            );
                            writeln!(
                                output,
                                "\x1b[32m[+] Engine switched to 'attention' oracle fallback mode.\x1b[0m\n"
                            )?;
                        }
                        _ => {
                            writeln!(
                                output,
                                "[*] Staying on active model: {}\n",
                                current_active_model
                            )?;
                        }
                    }
                }
            }
        }
        Err(e) => {
            writeln!(output, "[!] Error communicating with server: {}\n", e)?;
        }
    }
    Ok(())
}

/// Run an interactive client chat session against a remote local HTTP vendor endpoint.
pub fn remote_interactive_chat(
    remote_url: &str,
    model: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), std::io::Error> {
    let (host, port, path) = parse_remote_url(remote_url);

    // Read initial active model and engine from disk if present
    let mut current_active_model =
        if let Ok(m) = std::fs::read_to_string(".uor-models/last_model_name.txt") {
            let trimmed = m.trim().to_string();
            if !trimmed.is_empty() {
                trimmed
            } else {
                model.to_string()
            }
        } else {
            model.to_string()
        };

    let mut current_active_engine =
        if let Ok(e) = std::fs::read_to_string(".uor-models/last_engine.txt") {
            let trimmed = e.trim().to_string();
            if !trimmed.is_empty() {
                trimmed
            } else {
                "r4g1".to_string()
            }
        } else {
            "r4g1".to_string()
        };

    // Render Rich Intro Banner
    writeln!(output, "\x1b[1;36m")?;
    writeln!(
        output,
        "██╗   ██╗██████╗ ██████╗        ██████╗ ██╗  ██╗        ██████╗██╗     ██╗\n\
         ██║   ██║██╔══██╗██╔══██╗       ██╔══██╗██║  ██║       ██╔════╝██║     ██║\n\
         ██║   ██║██║  ██║██████╔╝ ████╗ ██████╔╝███████║ ████╗ ██║     ██║     ██║\n\
         ██║   ██║██║  ██║██╔══██╗ ╚═══╝ ██╔══██╗╚════██║ ╚═══╝ ██║     ██║     ██║\n\
         ╚██████╔╝██████╔╝██║  ██║       ██║  ██║     ██║       ╚██████╗███████╗██║\n\
          ╚═════╝ ╚═════╝ ╚═╝  ╚═╝       ╚═╝  ╚═╝     ╚═╝        ╚═════╝╚══════╝╚═╝"
    )?;
    writeln!(output, "\x1b[0m")?;
    writeln!(
        output,
        "\x1b[1mUOR-R4 Holographic Graph & Transformerless Engine v0.1.0\x1b[0m"
    )?;
    writeln!(
        output,
        "Zero-Multiply Local Intelligence Runtime • Pinned Multiplication-Free Execution\n"
    )?;
    writeln!(
        output,
        "Connected to local vendor endpoint: \x1b[36mhttp://{}:{}{}\x1b[0m",
        host, port, path
    )?;
    writeln!(
        output,
        "Active teacher model             : \x1b[32m{}\x1b[0m",
        current_active_model
    )?;
    writeln!(
        output,
        "Active synthesis engine          : \x1b[36m{}\x1b[0m\n",
        current_active_engine
    )?;
    writeln!(output, "\x1b[1mCommands & Shortcuts:\x1b[0m")?;
    writeln!(
        output,
        "  • Type \x1b[33m/help\x1b[0m to view available slash commands (/status, /models, /engine, /corpus, /export, /reset, /clear, /quit)"
    )?;
    writeln!(
        output,
        "  • Type \x1b[33m/\x1b[0m for interactive slash command suggestions & autocomplete"
    )?;
    writeln!(
        output,
        "  • Type \x1b[33mexit\x1b[0m or press \x1b[33mCtrl-D\x1b[0m to quit session\n"
    )?;
    output.flush()?;

    let mut history: Vec<String> = Vec::new();
    let mut audit_history: Vec<(String, String, Option<crate::server::UorAuditTrace>)> = Vec::new();

    loop {
        let prompt_lbl = format!(
            "\x1b[1;36muor-r4\x1b[0m \x1b[33m[model: {} | engine: {}]\x1b[0m \x1b[1;32m>\x1b[0m ",
            current_active_model, current_active_engine
        );
        let line_opt = match read_line_with_history(&prompt_lbl, &mut history, input, output) {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(e) => {
                writeln!(output, "[!] Input error: {}", e)?;
                break;
            }
        };

        let question = line_opt.trim();
        if matches!(question, "exit" | "quit") {
            break;
        }
        if question.is_empty() {
            continue;
        }

        if question.starts_with('/') {
            let mut input_cmd = question.trim();
            if input_cmd == "/" {
                let menu_options = [
                    ("/models", "Manage & switch active teacher model in-session"),
                    (
                        "/engine",
                        "Manage & switch synthesis engine (r4g1, attention, etc.)",
                    ),
                    (
                        "/status",
                        "View R4G1 sub-millisecond 4-stage pipeline readiness",
                    ),
                    (
                        "/corpus",
                        "Manage, import & paste extra reading corpus datasets into manifold",
                    ),
                    (
                        "/compile",
                        "Trigger full automated 4-stage graph compilation",
                    ),
                    (
                        "/audit",
                        "Audit Q&A token trace, UOR coordinates & R4 geometry",
                    ),
                    (
                        "/export",
                        "Export manifold state & corpus to .uor-models/exported/exported_manifold.json",
                    ),
                    (
                        "/reset",
                        "Reset chat history, corpus & geometric state back to base",
                    ),
                    ("/clear", "Clear terminal screen & session history"),
                    ("/quit", "Exit client session"),
                ];

                if let Ok(Some(idx)) = select_menu_interactive(
                    "R⁴ Interactive Slash Command Selector:",
                    &menu_options,
                    output,
                ) {
                    input_cmd = menu_options[idx].0;
                } else {
                    output.flush()?;
                    continue;
                }
            }

            let parts: Vec<&str> = input_cmd.split_whitespace().collect();
            let primary_token = parts[0];

            let matches: Vec<&SlashCommandDef> = COMMAND_DEFS
                .iter()
                .filter(|def| def.cmd.starts_with(primary_token))
                .collect();

            let target_cmd = match matches.len() {
                1 => matches[0].cmd,
                0 => {
                    writeln!(
                        output,
                        "[!] Unknown command: '{}'. Type '/help' or '/' for suggestions.\n",
                        input_cmd
                    )?;
                    output.flush()?;
                    continue;
                }
                _ => {
                    writeln!(output, "\nMultiple matching commands for '{}':", input_cmd)?;
                    for m in &matches {
                        writeln!(output, "  \x1b[33m{:<10}\x1b[0m - {}", m.cmd, m.desc)?;
                    }
                    writeln!(output)?;
                    output.flush()?;
                    continue;
                }
            };

            match target_cmd {
                "/help" => {
                    writeln!(output, "\n\x1b[1mR⁴ Interactive Slash Commands:\x1b[0m")?;
                    for def in COMMAND_DEFS {
                        writeln!(output, "  \x1b[33m{:<10}\x1b[0m - {}", def.cmd, def.desc)?;
                    }
                    writeln!(output)?;
                    output.flush()?;
                    continue;
                }
                "/models" | "/switch" => {
                    let target_model_opt = if parts.len() > 1 {
                        let sel = parts[1];
                        match sel {
                            "1" | "135m" => Some("smollm2-135m-instruct"),
                            "2" | "360m" => Some("smollm2-360m-instruct"),
                            "3" | "1.7b" | "1-7b" => Some("smollm2-1-7b-instruct"),
                            other => Some(other),
                        }
                    } else {
                        None
                    };

                    let target_model = match target_model_opt {
                        Some(m) => m.to_string(),
                        None => {
                            let (d1, c1) = check_model_artifact_status("smollm2-135m-instruct");
                            let (d2, c2) = check_model_artifact_status("smollm2-360m-instruct");
                            let (d3, c3) = check_model_artifact_status("smollm2-1-7b-instruct");

                            let desc1 = format!(
                                "Fast & Light (~270MB) [DL: {} | CP: {}]",
                                if d1 { "✓" } else { " " },
                                if c1 { "✓" } else { " " }
                            );
                            let desc2 = format!(
                                "Balanced Quality (~720MB) [DL: {} | CP: {}]",
                                if d2 { "✓" } else { " " },
                                if c2 { "✓" } else { " " }
                            );
                            let desc3 = format!(
                                "High-Fidelity (~3.4GB) [DL: {} | CP: {}]",
                                if d3 { "✓" } else { " " },
                                if c3 { "✓" } else { " " }
                            );

                            let model_options = [
                                ("smollm2-135m-instruct", desc1.as_str()),
                                ("smollm2-360m-instruct", desc2.as_str()),
                                ("smollm2-1-7b-instruct", desc3.as_str()),
                            ];
                            match select_menu_interactive(
                                "R⁴ Interactive Model Selector & Engine Manager:",
                                &model_options,
                                output,
                            )? {
                                Some(idx) => model_options[idx].0.to_string(),
                                None => {
                                    output.flush()?;
                                    continue;
                                }
                            }
                        }
                    };

                    handle_model_switch_with_remediation(
                        &target_model,
                        &host,
                        port,
                        &mut current_active_model,
                        &mut current_active_engine,
                        output,
                    )?;
                    output.flush()?;
                    continue;
                }
                "/engine" => {
                    let target_engine_opt = if parts.len() > 1 {
                        match parts[1] {
                            "1" | "r4g1" => Some("r4g1"),
                            "2" | "attention" => Some("attention"),
                            "3" | "r4-attention" => Some("r4-attention"),
                            "4" | "geometric" => Some("geometric"),
                            "5" | "legacy" | "transformerless-legacy" => {
                                Some("transformerless-legacy")
                            }
                            other => Some(other),
                        }
                    } else {
                        None
                    };

                    let target_engine = match target_engine_opt {
                        Some(eng) => eng.to_string(),
                        None => {
                            let engine_options = [
                                ("r4g1", "Sub-ms Zero-Multiply Residual Graph Engine"),
                                ("attention", "Full Attention Teacher Oracle Fallback"),
                                (
                                    "r4-attention",
                                    "Llama Certified Leading-4 Attention (GPT-2 Learned-Absolute)",
                                ),
                                ("geometric", "f64 Geometric Router Engine"),
                                ("transformerless-legacy", "Legacy Table Store Kernel"),
                            ];
                            match select_menu_interactive(
                                "R⁴ Interactive Synthesis Engine Manager:",
                                &engine_options,
                                output,
                            )? {
                                Some(idx) => engine_options[idx].0.to_string(),
                                None => {
                                    output.flush()?;
                                    continue;
                                }
                            }
                        }
                    };

                    current_active_engine = target_engine.clone();
                    let _ = std::fs::write(".uor-models/last_engine.txt", &current_active_engine);
                    writeln!(
                        output,
                        "\x1b[32m[+] Active synthesis engine set to '{}'\x1b[0m\n",
                        current_active_engine
                    )?;
                    output.flush()?;
                    continue;
                }
                "/export" => {
                    let req_body = serde_json::json!({ "action": "export" });
                    match send_server_post_request(&host, port, "/v1/corpus", &req_body) {
                        Ok(res) => {
                            let msg = res["message"]
                                .as_str()
                                .unwrap_or("Exported manifold state to .uor-models/exported/exported_manifold.json");
                            writeln!(output, "\x1b[32m[✓] {}\x1b[0m\n", msg)?;
                        }
                        Err(e) => {
                            writeln!(output, "[!] Export error: {}\n", e)?;
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/corpus" => {
                    if parts.len() > 2 && parts[1] == "add" {
                        let file_path = parts[2];
                        match std::fs::read_to_string(file_path) {
                            Ok(content) => {
                                let filename = std::path::Path::new(file_path)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "custom_corpus.txt".to_string());

                                let req_body = serde_json::json!({
                                    "action": "add",
                                    "filename": filename,
                                    "content": content
                                });

                                match send_server_post_request(&host, port, "/v1/corpus", &req_body)
                                {
                                    Ok(res) => {
                                        writeln!(
                                            output,
                                            "\x1b[32m[+] {}\x1b[0m\n",
                                            res["message"].as_str().unwrap_or("Corpus added")
                                        )?;
                                    }
                                    Err(e) => {
                                        writeln!(output, "[!] Error updating corpus: {}\n", e)?;
                                    }
                                }
                            }
                            Err(e) => {
                                writeln!(
                                    output,
                                    "[!] Failed to read file '{}': {}\n",
                                    file_path, e
                                )?;
                            }
                        }
                    } else {
                        let corpus_options = [
                            ("1. List Indexed Files", "View reading corpus datasets indexed on server"),
                            ("2. Import Local File", "Browse and select local text file to index into manifold"),
                            ("3. Paste Plain Text", "Paste raw text content to index into geometric manifold hashes"),
                            ("4. Export Manifold", "Export manifold state to .uor-models/exported/exported_manifold.json"),
                        ];
                        if let Ok(Some(opt_idx)) = select_menu_interactive(
                            "R⁴ Corpus & Geometric Manifold Management:",
                            &corpus_options,
                            output,
                        ) {
                            match opt_idx {
                                0 => {
                                    let req_body = serde_json::json!({ "action": "list" });
                                    match send_server_post_request(
                                        &host,
                                        port,
                                        "/v1/corpus",
                                        &req_body,
                                    ) {
                                        Ok(res) => {
                                            writeln!(
                                                output,
                                                "\n\x1b[1mR⁴ Extra Reading Corpus Datasets:\x1b[0m"
                                            )?;
                                            if let Some(files) = res["files"].as_array() {
                                                if files.is_empty() {
                                                    writeln!(
                                                        output,
                                                        "  (No extra reading corpus files indexed yet)"
                                                    )?;
                                                } else {
                                                    for f in files {
                                                        writeln!(
                                                            output,
                                                            "  • {}",
                                                            f.as_str().unwrap_or("")
                                                        )?;
                                                    }
                                                }
                                            }
                                            writeln!(output)?;
                                        }
                                        Err(e) => {
                                            writeln!(output, "[!] Error listing corpus: {}\n", e)?;
                                        }
                                    }
                                }
                                1 => {
                                    let mut candidates = Vec::new();
                                    if let Ok(entries) = std::fs::read_dir(".uor-models/sources") {
                                        for entry in entries.filter_map(|e| e.ok()) {
                                            let p = entry.path();
                                            if p.is_file() {
                                                candidates.push(p.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                    if let Ok(entries) = std::fs::read_dir(".") {
                                        for entry in entries.filter_map(|e| e.ok()) {
                                            let p = entry.path();
                                            if p.is_file()
                                                && (p.extension()
                                                    == Some(std::ffi::OsStr::new("txt"))
                                                    || p.extension()
                                                        == Some(std::ffi::OsStr::new("md")))
                                            {
                                                candidates.push(p.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                    candidates.sort();
                                    candidates.dedup();

                                    let mut menu_items: Vec<(&str, &str)> = candidates
                                        .iter()
                                        .map(|path| (path.as_str(), "Local corpus document"))
                                        .collect();
                                    menu_items.push((
                                        "Custom File Path...",
                                        "Enter arbitrary file path manually",
                                    ));

                                    if let Ok(Some(file_idx)) = select_menu_interactive(
                                        "Select File to Import into Geometric Manifold:",
                                        &menu_items,
                                        output,
                                    ) {
                                        let target_path = if file_idx < candidates.len() {
                                            candidates[file_idx].clone()
                                        } else {
                                            writeln!(output, "Enter file path to import: ")?;
                                            output.flush()?;
                                            let mut path_buf = String::new();
                                            std::io::stdin().read_line(&mut path_buf).ok();
                                            path_buf.trim().to_string()
                                        };

                                        if !target_path.is_empty() {
                                            match std::fs::read_to_string(&target_path) {
                                                Ok(content) => {
                                                    let filename =
                                                        std::path::Path::new(&target_path)
                                                            .file_name()
                                                            .map(|f| {
                                                                f.to_string_lossy().to_string()
                                                            })
                                                            .unwrap_or_else(|| {
                                                                "imported_corpus.txt".to_string()
                                                            });

                                                    let req_body = serde_json::json!({
                                                        "action": "add",
                                                        "filename": filename,
                                                        "content": content
                                                    });

                                                    match send_server_post_request(
                                                        &host,
                                                        port,
                                                        "/v1/corpus",
                                                        &req_body,
                                                    ) {
                                                        Ok(res) => {
                                                            writeln!(
                                                                output,
                                                                "\x1b[32m[✓] {}\x1b[0m\n",
                                                                res["message"]
                                                                    .as_str()
                                                                    .unwrap_or("Corpus imported")
                                                            )?;
                                                        }
                                                        Err(e) => {
                                                            writeln!(
                                                                output,
                                                                "[!] Error importing corpus: {}\n",
                                                                e
                                                            )?;
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    writeln!(
                                                        output,
                                                        "[!] Failed to read '{}': {}\n",
                                                        target_path, e
                                                    )?;
                                                }
                                            }
                                        }
                                    }
                                }
                                2 => {
                                    writeln!(
                                        output,
                                        "\x1b[1mPaste plain text content to index into R⁴ geometric manifold:\x1b[0m"
                                    )?;
                                    writeln!(output, "(Type 'END' or press Enter on an empty line when finished)\n")?;
                                    output.flush()?;

                                    let mut lines = Vec::new();
                                    let stdin = std::io::stdin();
                                    let stdin_handle = stdin.lock();
                                    use std::io::BufRead;
                                    for line_res in stdin_handle.lines() {
                                        let line = match line_res {
                                            Ok(l) => l,
                                            Err(e) => {
                                                writeln!(output, "[!] Error reading stdin: {}", e)?;
                                                break;
                                            }
                                        };
                                        let trimmed = line.trim();
                                        if trimmed == "END" || trimmed.is_empty() {
                                            break;
                                        }
                                        lines.push(line);
                                    }
                                    if !lines.is_empty() {
                                        let content = lines.join("\n");
                                        let ts = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs();
                                        let filename = format!("pasted_corpus_{}.txt", ts);

                                        let req_body = serde_json::json!({
                                            "action": "add",
                                            "filename": filename,
                                            "content": content
                                        });

                                        match send_server_post_request(
                                            &host,
                                            port,
                                            "/v1/corpus",
                                            &req_body,
                                        ) {
                                            Ok(res) => {
                                                writeln!(
                                                    output,
                                                    "\x1b[32m[✓] {}\x1b[0m\n",
                                                    res["message"]
                                                        .as_str()
                                                        .unwrap_or("Pasted corpus indexed")
                                                )?;
                                            }
                                            Err(e) => {
                                                writeln!(
                                                    output,
                                                    "[!] Error indexing pasted corpus: {}\n",
                                                    e
                                                )?;
                                            }
                                        }
                                    } else {
                                        writeln!(
                                            output,
                                            "[!] Empty text content submitted. Nothing indexed.\n"
                                        )?;
                                    }
                                }
                                3 => {
                                    let req_body = serde_json::json!({ "action": "export" });
                                    match send_server_post_request(
                                        &host,
                                        port,
                                        "/v1/corpus",
                                        &req_body,
                                    ) {
                                        Ok(res) => {
                                            let msg = res["message"]
                                                .as_str()
                                                .unwrap_or("Exported manifold state to .uor-models/exported/exported_manifold.json");
                                            writeln!(output, "\x1b[32m[✓] {}\x1b[0m\n", msg)?;
                                        }
                                        Err(e) => {
                                            writeln!(output, "[!] Export error: {}\n", e)?;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/clear" => {
                    history.clear();
                    audit_history.clear();
                    write!(output, "\x1b[2J\x1b[1H")?;
                    output.flush()?;
                    continue;
                }
                "/reset" => {
                    history.clear();
                    audit_history.clear();
                    let _ = send_vendor_reset(&host, port);
                    write!(output, "\x1b[2J\x1b[1H")?;
                    writeln!(
                        output,
                        "\x1b[32m[✓] Chat history, extra corpus index & geometric manifold state reset back to base defaults.\x1b[0m\n"
                    )?;
                    output.flush()?;
                    continue;
                }
                "/quit" | "/exit" => {
                    break;
                }
                "/compile" => {
                    let model_options = [
                        ("smollm2-135m-instruct", "Fast & Ultra-Light (~270MB)"),
                        ("smollm2-360m-instruct", "Balanced Quality (~720MB)"),
                        ("smollm2-1-7b-instruct", "High-Fidelity Teacher (~3.4GB)"),
                    ];
                    if let Ok(Some(m_idx)) = select_menu_interactive(
                        "Select Model to Compile into R4G1 Zero-Multiply Graph:",
                        &model_options,
                        output,
                    ) {
                        let target_model = model_options[m_idx].0;
                        if trigger_in_client_compilation(target_model, &host, port, output)
                            .unwrap_or(false)
                        {
                            current_active_model = target_model.to_string();
                            current_active_engine = "r4g1".to_string();
                            let _ = std::fs::write(
                                ".uor-models/last_model_name.txt",
                                &current_active_model,
                            );
                            let _ = std::fs::write(
                                ".uor-models/last_engine.txt",
                                &current_active_engine,
                            );
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/status" => {
                    writeln!(
                        output,
                        "[*] Querying R4G1 sub-millisecond pipeline status..."
                    )?;
                    match fetch_server_status(&host, port) {
                        Ok(st) => {
                            let model_name =
                                st["model_name"].as_str().unwrap_or("smollm2-135m-instruct");
                            let s1 = st["stages"]["stage_1_download"].as_bool().unwrap_or(false);
                            let s2 = st["stages"]["stage_2_compile"].as_bool().unwrap_or(false);
                            let s3 = st["stages"]["stage_3_graph_score"]
                                .as_bool()
                                .unwrap_or(false);
                            let s4 = st["stages"]["stage_4_r4g1_active"]
                                .as_bool()
                                .unwrap_or(false);

                            let mark = |b| if b { "[✓]" } else { "[ ]" };

                            writeln!(
                                output,
                                "\n\x1b[1mR⁴ Sub-Millisecond R4G1 Compilation Pipeline Status ({})\x1b[0m",
                                model_name
                            )?;
                            writeln!(output, "┌───────┬───────────────────────────────────┬────────┬──────────────────────────────────────────────┐")?;
                            writeln!(output, "│ Stage │ Description                       │ Status │ Target Artifact / Location                   │")?;
                            writeln!(output, "├───────┼───────────────────────────────────┼────────┼──────────────────────────────────────────────┤")?;
                            writeln!(output, "│   1   │ Pinned Teacher Source Download    │  {:^5} │ .uor-models/sources/{:<25} │", mark(s1), model_name)?;
                            writeln!(output, "│   2   │ Transformerless Bundle Compile    │  {:^5} │ .uor-models/compiled/{:<24} │", mark(s2), model_name)?;
                            writeln!(output, "│   3   │ Scored R4G1 Graph Cover & Score   │  {:^5} │ .../{:<35} │", mark(s3), format!("{}/graph/score.r4g1", model_name))?;
                            writeln!(output, "│   4   │ Sub-ms Zero-Multiply Engine       │  {:^5} │ Active (R4G1 Scored Graph Runtime)           │", mark(s4))?;
                            writeln!(output, "└───────┴───────────────────────────────────┴────────┴──────────────────────────────────────────────┘")?;
                            writeln!(output, "Target Performance Goal: < 1.0 ms / token (Zero-Multiply Table-Native Kernel)\n")?;
                        }
                        Err(e) => {
                            writeln!(output, "[!] Error fetching pipeline status: {}\n", e)?;
                        }
                    }
                    output.flush()?;
                    continue;
                }
                "/audit" => {
                    if audit_history.is_empty() {
                        writeln!(
                            output,
                            "\n\x1b[33m[!] No Q&A turns audited in this session yet.\x1b[0m"
                        )?;
                        writeln!(
                            output,
                            "    Ask a question first, then run '/audit' to inspect UOR coordinates.\n"
                        )?;
                    } else {
                        let audit_options = [
                            (
                                "1) View Last Q&A Audit Trace",
                                "Inspect UOR coordinates, kappa pass status, and token provenance for last turn",
                            ),
                            (
                                "2) List Audit History",
                                "Browse and select from recent session Q&A turns",
                            ),
                            (
                                "3) Export Audit Log",
                                "Export full session audit trace to .uor-models/audit_log.json",
                            ),
                        ];

                        if let Ok(Some(a_idx)) = select_menu_interactive(
                            "R⁴ UOR Auditability & Tracing Inspector:",
                            &audit_options,
                            output,
                        ) {
                            match a_idx {
                                0 => {
                                    let last_rec = audit_history.last().unwrap();
                                    render_audit_trace_record(last_rec, output)?;
                                }
                                1 => {
                                    let history_options: Vec<(String, String)> = audit_history
                                        .iter()
                                        .enumerate()
                                        .map(|(i, (q, _, audit))| {
                                            let kappa_str = audit
                                                .as_ref()
                                                .map(|a| format!("κ={:.4}", a.kappa))
                                                .unwrap_or_else(|| "N/A".to_string());
                                            let short_q = if q.len() > 35 {
                                                format!("{}...", &q[..35])
                                            } else {
                                                q.clone()
                                            };
                                            (
                                                format!("Turn #{}", i + 1),
                                                format!("{} [{}]", short_q, kappa_str),
                                            )
                                        })
                                        .collect();

                                    let view_refs: Vec<(&str, &str)> = history_options
                                        .iter()
                                        .map(|(label, desc)| (label.as_str(), desc.as_str()))
                                        .collect();

                                    if let Ok(Some(h_idx)) = select_menu_interactive(
                                        "Select Q&A Turn to Audit:",
                                        &view_refs,
                                        output,
                                    ) {
                                        render_audit_trace_record(&audit_history[h_idx], output)?;
                                    }
                                }
                                2 => {
                                    let export_path = ".uor-models/audit_log.json";
                                    if let Ok(json_str) =
                                        serde_json::to_string_pretty(&audit_history)
                                    {
                                        if std::fs::write(export_path, json_str).is_ok() {
                                            writeln!(
                                                output,
                                                "\x1b[32m[+] Successfully exported session audit trace ({} turns) to {}\x1b[0m\n",
                                                audit_history.len(),
                                                export_path
                                            )?;
                                        } else {
                                            writeln!(
                                                output,
                                                "[!] Failed to write audit log file.\n"
                                            )?;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    output.flush()?;
                    continue;
                }
                _ => {}
            }
        }

        let start_time = std::time::Instant::now();
        let (host_c, port_c, path_c, model_c, engine_c, q_c) = (
            host.clone(),
            port,
            path.clone(),
            current_active_model.clone(),
            current_active_engine.clone(),
            question.to_string(),
        );

        let worker_handle = std::thread::spawn(move || {
            send_vendor_chat_completion(&host_c, port_c, &path_c, &model_c, &engine_c, &q_c)
        });

        writeln!(output)?;
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut frame_idx = 0;

        while !worker_handle.is_finished() {
            let elapsed_secs = start_time.elapsed().as_secs();
            let frame = frames[frame_idx % frames.len()];
            write!(
                output,
                "\r\x1b[1;32mr4\x1b[0m \x1b[1;36m>\x1b[0m {} lifting... ({}s)\x1b[K",
                frame, elapsed_secs
            )?;
            output.flush()?;
            frame_idx += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let res = worker_handle
            .join()
            .unwrap_or_else(|_| Err("Worker thread panicked".to_string()));

        match res {
            Ok((answer_text, completion_tokens, engine_mode, uor_audit)) => {
                audit_history.push((question.to_string(), answer_text.clone(), uor_audit));
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let latency_ms = elapsed_secs * 1000.0;
                let tok_per_sec = if elapsed_secs > 0.0001 {
                    (completion_tokens as f64) / elapsed_secs
                } else {
                    0.0
                };
                write!(
                    output,
                    "\r\x1b[1;32mr4\x1b[0m \x1b[1;36m>\x1b[0m {}\x1b[K\n",
                    answer_text
                )?;
                writeln!(
                    output,
                    "\x1b[90m[stats: {} tokens | {:.2} ms | {:.1} tok/s | mode: {} | model: {}]\x1b[0m\n",
                    completion_tokens, latency_ms, tok_per_sec, engine_mode, current_active_model
                )?;
                output.flush()?;
            }
            Err(err) => {
                write!(
                    output,
                    "\r\x1b[1;32mr4\x1b[0m \x1b[1;36m>\x1b[0m [!] Error communicating with local server: {}\x1b[K\n\n",
                    err
                )?;
                output.flush()?;
            }
        }
    }
    Ok(())
}

fn parse_remote_url(raw_url: &str) -> (String, u16, String) {
    let clean = raw_url
        .trim()
        .strip_prefix("http://")
        .or_else(|| raw_url.trim().strip_prefix("https://"))
        .unwrap_or(raw_url.trim());

    let (host_port, path_part) = match clean.find('/') {
        Some(idx) => (&clean[..idx], &clean[idx..]),
        None => (clean, ""),
    };

    let mut parts = host_port.split(':');
    let host = parts.next().unwrap_or("127.0.0.1").trim();
    let host_str = if host.is_empty() { "127.0.0.1" } else { host };
    let port: u16 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(8000);

    let path = if path_part.contains("/chat/completions") {
        path_part.to_string()
    } else if path_part.ends_with("/v1") || path_part.ends_with("/v1/") {
        let base = path_part.trim_end_matches('/');
        format!("{}/chat/completions", base)
    } else if path_part.is_empty() || path_part == "/" {
        "/v1/chat/completions".to_string()
    } else {
        format!("{}/chat/completions", path_part.trim_end_matches('/'))
    };

    (host_str.to_string(), port, path)
}

fn send_vendor_chat_completion(
    host: &str,
    port: u16,
    path: &str,
    model: &str,
    engine: &str,
    user_message: &str,
) -> Result<(String, usize, String, Option<crate::server::UorAuditTrace>), String> {
    let payload = serde_json::json!({
        "model": model,
        "engine": engine,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ],
        "max_tokens": 384,
        "temperature": 0.7
    });
    let body_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("Serialization error: {}", e))?;

    let req_str = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        path,
        host,
        port,
        body_bytes.len()
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(300)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();

    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send request headers: {}", e))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("Failed to send request body: {}", e))?;
    stream
        .flush()
        .map_err(|e| format!("Failed to flush stream: {}", e))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let resp_text = String::from_utf8_lossy(&response_bytes);
    let body_start = resp_text.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    let json_body = &resp_text[body_start..];

    let parsed: serde_json::Value = serde_json::from_str(json_body)
        .map_err(|e| format!("Invalid response JSON: {} (body: {:?})", e, json_body))?;

    let choice = parsed["choices"]
        .get(0)
        .ok_or_else(|| "Missing choices in response".to_string())?;
    let content = choice["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let completion_tokens = parsed["usage"]["completion_tokens"]
        .as_u64()
        .unwrap_or_else(|| content.split_whitespace().count() as u64)
        as usize;
    let mode = parsed["system_fingerprint"]
        .as_str()
        .unwrap_or("uor-r4")
        .strip_prefix("uor-r4-")
        .unwrap_or_else(|| parsed["system_fingerprint"].as_str().unwrap_or("uor-r4"))
        .to_string();

    let uor_audit: Option<crate::server::UorAuditTrace> = parsed
        .get("uor_audit")
        .and_then(|val| serde_json::from_value(val.clone()).ok());

    Ok((content, completion_tokens, mode, uor_audit))
}

fn fetch_server_status(host: &str, port: u16) -> Result<serde_json::Value, String> {
    let req_str = format!(
        "GET /v1/status HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Connection: close\r\n\r\n",
        host, port
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .ok();
    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send status request: {}", e))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| format!("Failed to read status response: {}", e))?;

    let resp_text = String::from_utf8_lossy(&response_bytes);
    let body_start = resp_text.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    let json_body = &resp_text[body_start..];

    serde_json::from_str(json_body).map_err(|e| format!("Invalid status response JSON: {}", e))
}

pub fn send_vendor_reset(host: &str, port: u16) -> Result<(), String> {
    let payload = serde_json::json!({});
    let body_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("Serialization error: {}", e))?;
    let req_str = format!(
        "POST /api/reset HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        host,
        port,
        body_bytes.len()
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send request: {}", e))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("Failed to send body: {}", e))?;
    stream.flush().ok();

    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).ok();
    Ok(())
}

fn send_server_post_request(
    host: &str,
    port: u16,
    path: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let body_bytes =
        serde_json::to_vec(payload).map_err(|e| format!("Serialization error: {}", e))?;
    let req_str = format!(
        "POST {} HTTP/1.1\r\n\
         Host: {}:{}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n",
        path,
        host,
        port,
        body_bytes.len()
    );

    let sockaddr: std::net::SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| format!("Invalid socket address {}:{}: {}", host, port, e))?;

    let mut stream =
        std::net::TcpStream::connect_timeout(&sockaddr, std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to connect to {}:{}: {}", host, port, e))?;

    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();

    stream
        .write_all(req_str.as_bytes())
        .map_err(|e| format!("Failed to send request headers: {}", e))?;
    stream
        .write_all(&body_bytes)
        .map_err(|e| format!("Failed to send request body: {}", e))?;
    stream
        .flush()
        .map_err(|e| format!("Failed to flush stream: {}", e))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let resp_text = String::from_utf8_lossy(&response_bytes);
    let body_start = resp_text.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(0);
    let json_body = &resp_text[body_start..];

    serde_json::from_str(json_body)
        .map_err(|e| format!("Invalid response JSON: {} (body: {:?})", e, json_body))
}

fn render_audit_trace_record(
    record: &(String, String, Option<crate::server::UorAuditTrace>),
    output: &mut impl Write,
) -> Result<(), std::io::Error> {
    let (question, answer, audit_opt) = record;
    writeln!(
        output,
        "\n\x1b[1;36m┌─────────────────────────────────────────────────────────────────────────────┐\x1b[0m"
    )?;
    writeln!(
        output,
        "\x1b[1;36m│  R⁴ UOR Auditability & Tracing Inspector                                    │\x1b[0m"
    )?;
    writeln!(
        output,
        "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
    )?;
    let short_q = if question.len() > 65 {
        format!("{}...", &question[..65])
    } else {
        question.clone()
    };
    let short_a = if answer.len() > 65 {
        format!("{}...", &answer[..65])
    } else {
        answer.clone()
    };
    writeln!(output, "  \x1b[1mQuery Prompt \x1b[0m : {}", short_q)?;
    writeln!(output, "  \x1b[1mGenerated Ans\x1b[0m : {}", short_a)?;
    writeln!(
        output,
        "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
    )?;

    if let Some(audit) = audit_opt {
        let pass_badge = if audit.kappa_pass {
            "\x1b[32m[✓ PASS]\x1b[0m"
        } else {
            "\x1b[33m[! DRIFT]\x1b[0m"
        };
        writeln!(
            output,
            "  \x1b[1mUOR Address     \x1b[0m : \x1b[36m{}\x1b[0m",
            audit.uor_address
        )?;
        writeln!(
            output,
            "  \x1b[1mCurvature κ     \x1b[0m : {} {}",
            audit.kappa, pass_badge
        )?;
        writeln!(
            output,
            "  \x1b[1mDeficit Angle θd\x1b[0m : {} rad",
            audit.deficit_angle
        )?;
        writeln!(
            output,
            "  \x1b[1mQIMC Bias uor_b \x1b[0m : {}",
            audit.entropy_bias
        )?;
        writeln!(output, "  \x1b[1mDampening γ     \x1b[0m : {}", audit.gamma)?;
        writeln!(
            output,
            "  \x1b[1mTemperature T   \x1b[0m : {}",
            audit.temperature
        )?;
        writeln!(
            output,
            "  \x1b[1mEngine Mode     \x1b[0m : \x1b[32m{}\x1b[0m",
            audit.generation_mode
        )?;
        writeln!(
            output,
            "  \x1b[1mTotal Latency   \x1b[0m : {:.2} ms",
            audit.total_latency_ms
        )?;
        writeln!(
            output,
            "\x1b[1;36m├─────────────────────────────────────────────────────────────────────────────┤\x1b[0m"
        )?;
        writeln!(
            output,
            "  \x1b[1mToken Provenance Trace ({} tokens):\x1b[0m",
            audit.tokens_detail.len()
        )?;
        for t in audit.tokens_detail.iter().take(20) {
            writeln!(
                output,
                "   [{:>2}] '{:<15}' -> {:<38} ({:.2} ms)",
                t.token_id, t.text, t.origin_rule, t.latency_ms
            )?;
        }
        if audit.tokens_detail.len() > 20 {
            writeln!(
                output,
                "   ... ({} remaining tokens omitted for display)",
                audit.tokens_detail.len() - 20
            )?;
        }
    } else {
        writeln!(output, "  (No UOR audit trace payload returned by backend)")?;
    }
    writeln!(
        output,
        "\x1b[1;36m└─────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n"
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        bind_chat_r4g1, ensure_chat_prompt_encoder, parse_remote_url,
        recent_window_repetition_rate, repeated_suffix, select_chat_tokenizer_bytes, ChatError,
    };
    use uor_r4_core::transformerless::scenarios::{
        export_runtime_tokenizer_table, RuntimeTokenizerDecodePolicy, RuntimeTokenizerDecodeTable,
        RuntimeTokenizerEncodePolicy, RuntimeTokenizerIdentity, Tokenizer,
    };
    use uor_r4_router::session_signature_from_tokens;

    fn minimal_graph(tokenizer_cid: [u8; 32]) -> Vec<u8> {
        use uor_r4_graph_format::{ArtifactBuilder, SectionId};

        let mut head = Vec::with_capacity(224);
        head.extend_from_slice(&[0x11; 32]);
        head.extend_from_slice(&tokenizer_cid);
        head.extend_from_slice(&[0x33; 32]);
        head.extend_from_slice(&[0x44; 32]);
        head.extend_from_slice(b"0123456789abcdef0123");
        head.extend_from_slice(&[0x55; 32]);
        head.extend_from_slice(&32u16.to_le_bytes());
        head.extend_from_slice(&16u16.to_le_bytes());
        head.extend_from_slice(&8u16.to_le_bytes());
        head.extend_from_slice(&8u16.to_le_bytes());
        head.extend_from_slice(&64u32.to_le_bytes());
        head.extend_from_slice(&64u32.to_le_bytes());
        head.extend_from_slice(&0u32.to_le_bytes());
        head.extend_from_slice(&0u32.to_le_bytes());
        head.push(1);
        head.extend_from_slice(&[0; 5]);
        head.extend_from_slice(&[0; 2]);
        head.extend_from_slice(&64u16.to_le_bytes());
        head.extend_from_slice(&1u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&0u16.to_le_bytes());
        head.extend_from_slice(&100u32.to_le_bytes());
        assert_eq!(head.len(), 224);
        let mut builder = ArtifactBuilder::new(3);
        builder.add_section(SectionId::HEAD, 0, &head);
        builder.build().expect("minimal graph")
    }

    #[test]
    fn repetition_guard_detects_repeated_token_windows() {
        assert!(repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 4], 4));
        assert!(!repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 5], 4));
    }

    /// `repeated_suffix`'s exact-block detector is blind to the #745
    /// failure mode: cycling through a handful of distinct tokens in
    /// varying (non-block-repeating) order. `recent_window_repetition_rate`
    /// (#744) is built specifically to catch it.
    #[test]
    fn recent_window_repetition_rate_catches_small_vocabulary_cycling_that_repeated_suffix_misses()
    {
        let cycling = [1u32, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3];
        assert!(
            !repeated_suffix(&cycling, 8),
            "too short for the exact 8-token block detector to see anything"
        );
        assert!(
            recent_window_repetition_rate(&cycling, 8) > 0.5,
            "but a small-vocabulary cycle is mostly recently-seen tokens"
        );

        let varied: Vec<u32> = (0..40).collect();
        assert_eq!(
            recent_window_repetition_rate(&varied, 8),
            0.0,
            "monotonically novel tokens carry no repetition"
        );

        assert_eq!(
            recent_window_repetition_rate(&[], 8),
            0.0,
            "empty generation is defined as non-repetitive, not undefined"
        );
    }

    #[test]
    fn direct_chat_reports_decode_only_tokenizer_as_unavailable() {
        let path = std::env::temp_dir().join(format!(
            "uor-r4-chat-decode-only-{}.bin",
            std::process::id()
        ));
        let table = RuntimeTokenizerDecodeTable {
            identity: RuntimeTokenizerIdentity {
                family: "future-sentencepiece-family".to_owned(),
                version: 41,
                tokenizer_cid: format!("blake3:{}", "3".repeat(64)),
                adapter_digest: format!("blake3:{}", "4".repeat(64)),
            },
            pieces: vec![Vec::new(), b"piece".to_vec()],
            encode_policy: RuntimeTokenizerEncodePolicy::Unavailable,
            decode_policy: RuntimeTokenizerDecodePolicy::SentencePiece {
                strip_dummy_prefix: true,
            },
            source_byte_lengths: None,
        };
        export_runtime_tokenizer_table(&table, &path).expect("tagged export");
        let bytes = std::fs::read(&path).expect("read tagged");
        assert_eq!(
            select_chat_tokenizer_bytes(&bytes).expect("valid tagged bytes stay selected"),
            bytes
        );
        let mut malformed = bytes.clone();
        malformed.pop();
        let error = select_chat_tokenizer_bytes(&malformed)
            .expect_err("malformed tagged bytes cannot downgrade to a legacy tokenizer");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

        let tokenizer = Tokenizer::from_bytes(&bytes).expect("parse tagged");
        let error = ensure_chat_prompt_encoder(&tokenizer).expect_err("encoder is unavailable");
        assert!(matches!(
            error,
            ChatError::TokenizerUnavailable {
                ref family,
                version: 41,
            } if family == "future-sentencepiece-family"
        ));
        assert!(error.to_string().contains("exact host adapter"));
        let error = bind_chat_r4g1(Some(minimal_graph([0; 32])), &bytes)
            .expect_err("zero-CID graph cannot admit a tagged tokenizer");
        assert!(error.to_string().contains("nonzero"), "{error}");
        assert!(bind_chat_r4g1(
            Some(minimal_graph(*blake3::hash(&bytes).as_bytes())),
            &bytes,
        )
        .expect("nonzero exact tagged binding")
        .is_some());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn present_untagged_chat_tokenizer_keeps_its_exact_bytes() {
        let mut bytes = Vec::new();
        for piece in [b"<unk>".as_slice(), b"a".as_slice(), b"b".as_slice()] {
            bytes.extend_from_slice(&(piece.len() as i32).to_le_bytes());
            bytes.extend_from_slice(piece);
        }
        assert!(!Tokenizer::is_tagged_container_bytes(&bytes));
        assert_eq!(
            select_chat_tokenizer_bytes(&bytes).expect("valid historical tokenizer"),
            bytes
        );
    }

    #[test]
    fn direct_chat_rejects_a_graph_tokenizer_swap_and_preserves_zero_cid_legacy() {
        let tokenizer_a = b"\x01\0\0\0a".to_vec();
        let tokenizer_b = b"\x01\0\0\0b".to_vec();
        let graph = minimal_graph(*blake3::hash(&tokenizer_a).as_bytes());
        let error = bind_chat_r4g1(Some(graph.clone()), &tokenizer_b)
            .expect_err("graph A plus tokenizer B must be rejected");
        assert!(
            error.to_string().contains("tokenizer CID mismatch"),
            "{error}"
        );
        assert_eq!(
            bind_chat_r4g1(Some(graph.clone()), &tokenizer_a)
                .expect("exact graph/tokenizer pair")
                .as_deref(),
            Some(graph.as_slice())
        );

        let legacy = minimal_graph([0; 32]);
        assert_eq!(
            bind_chat_r4g1(Some(legacy.clone()), &tokenizer_b)
                .expect("zero-CID graph retains legacy compatibility")
                .as_deref(),
            Some(legacy.as_slice())
        );
    }

    #[test]
    fn parse_remote_url_parses_various_formats() {
        let (host, port, path) = parse_remote_url("http://127.0.0.1:8000/v1");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 8000);
        assert_eq!(path, "/v1/chat/completions");

        let (host, port, path) = parse_remote_url("http://localhost:9000/v1/chat/completions");
        assert_eq!(host, "localhost");
        assert_eq!(port, 9000);
        assert_eq!(path, "/v1/chat/completions");
    }

    #[test]
    fn session_lane_sees_history_beyond_the_shared_eight_token_context() {
        let shared_context = [10, 11, 12, 13, 14, 15, 16, 17];
        let mut first_history = vec![1, 2, 3, 4];
        first_history.extend_from_slice(&shared_context);
        let mut second_history = vec![91, 92, 93, 94];
        second_history.extend_from_slice(&shared_context);

        assert_eq!(
            &first_history[first_history.len() - 8..],
            &second_history[second_history.len() - 8..]
        );
        assert_ne!(
            session_signature_from_tokens(&first_history),
            session_signature_from_tokens(&second_history)
        );
    }
}
