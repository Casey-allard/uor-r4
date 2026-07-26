//! Example direct-chat application built on R⁴ transformerless inference.
//!
//! Chat is a consumer of the core runtime, not a separate inference layer.

use std::fmt;
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;

use crate::model::{default_model_reference, ModelError, ModelStore};
use uor_r4_core::transformerless::compiler::{self, Compiled};
use uor_r4_core::transformerless::runtime::{self, Runtime, Store};
use uor_r4_core::transformerless::scenarios::Tokenizer;

const MAX_CHAT_TOKENS: usize = 256;
const MAX_CHAT_HISTORY: usize = 4096;
const MAX_ANSWER_BYTES: usize = 16 * 1024;

/// A completed local chat turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatAnswer {
    /// Generated assistant text.
    pub text: String,
    /// Number of tokens generated for this turn.
    pub generated_tokens: usize,
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
    /// Generation produced no tokens or could not be decoded.
    EmptyGeneration,
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
        let r4g1_bytes = std::fs::read(format!(
            ".uor-models/compiled/{}/compiled.r4g1",
            manifest.name
        ))
        .ok();
        let is_32k_graph = r4g1_bytes.as_ref().is_some_and(|b| {
            uor_r4_graph_runtime::R4G1Runtime::parse(b).is_ok_and(|r| r.node_count() > 0)
        });
        let tokenizer_bytes =
            if is_32k_graph && std::path::Path::new("/tmp/ref/tokenizer.bin").exists() {
                std::fs::read("/tmp/ref/tokenizer.bin")?
            } else {
                model_store.get(&manifest.tokenizer)?
            };
        let tokenizer_path = write_tokenizer_cache(&manifest.tokenizer.cid, &tokenizer_bytes)?;
        let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
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
    let r4g1_bytes = std::fs::read(directory.join("compiled.r4g1")).ok();
    let is_32k_graph = r4g1_bytes.as_ref().is_some_and(|b| {
        uor_r4_graph_runtime::R4G1Runtime::parse(b).is_ok_and(|r| r.node_count() > 0)
    });
    let tok_file = if is_32k_graph && std::path::Path::new("/tmp/ref/tokenizer.bin").exists() {
        std::path::PathBuf::from("/tmp/ref/tokenizer.bin")
    } else {
        directory.join("tokenizer.bin")
    };
    tracing::debug!(is_32k_graph, ?tok_file, "resolved chat tokenizer path");
    let tokenizer_bytes = std::fs::read(&tok_file)?;
    let artifacts =
        compiler::parse_artifacts(&artifact_bytes).ok_or(ChatError::InvalidArtifacts)?;
    let store = runtime::parse_store(&store_bytes).ok_or(ChatError::InvalidStore)?;

    // Content-address all local compiler outputs immediately. A manifest and
    // quality report remain optional metadata; integrity does not.
    let artifact_object = model_store.put(&artifact_bytes)?;
    let store_object = model_store.put(&store_bytes)?;
    let tokenizer_object = model_store.put(&tokenizer_bytes)?;
    let tokenizer_path = write_tokenizer_cache(&tokenizer_object.cid, &tokenizer_bytes)?;
    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
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
    let mut question_tokens = [0u32; MAX_CHAT_HISTORY];
    let question_count = tokenizer.encode_into(question, &mut question_tokens)?;
    let question_tokens = if *history_len == 0 {
        &question_tokens[..question_count]
    } else {
        &question_tokens[1..question_count]
    };
    append_history(history, history_len, question_tokens);

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
                    let num_cands = r4g1.predict_candidates(
                        &beam_history,
                        Some(&sig),
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
            let answer_len = tokenizer.decode_into(generated, &mut answer_bytes)?;
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
    let answer_len = tokenizer.decode_into(generated, &mut answer_bytes)?;
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
    })
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

fn write_tokenizer_cache(cid: &str, bytes: &[u8]) -> Result<PathBuf, ChatError> {
    let hash = cid.strip_prefix("blake3:").unwrap_or(cid);
    let directory = std::env::temp_dir().join("uor-r4-tokenizers");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{hash}.bin"));
    std::fs::write(&path, bytes)?;
    Ok(path)
}

/// Run an interactive client chat session against a remote local HTTP vendor endpoint.
pub fn remote_interactive_chat(
    remote_url: &str,
    model: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<(), std::io::Error> {
    let (host, port, path) = parse_remote_url(remote_url);
    writeln!(
        output,
        "R⁴ Client — interactive local vendor chat (remote: http://{}:{}{})",
        host, port, path
    )?;
    writeln!(output, "type 'exit' or Ctrl-D to quit\n")?;

    loop {
        write!(output, "you > ")?;
        output.flush()?;
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let question = line.trim();
        if matches!(question, "exit" | "quit") {
            break;
        }
        if question.is_empty() {
            continue;
        }

        let start_time = std::time::Instant::now();
        let (host_c, port_c, path_c, model_c, q_c) = (
            host.clone(),
            port,
            path.clone(),
            model.to_string(),
            question.to_string(),
        );

        let worker_handle = std::thread::spawn(move || {
            send_vendor_chat_completion(&host_c, port_c, &path_c, &model_c, &q_c)
        });

        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut frame_idx = 0;

        while !worker_handle.is_finished() {
            let elapsed_secs = start_time.elapsed().as_secs();
            let frame = frames[frame_idx % frames.len()];
            write!(
                output,
                "\rr4 > {} cooking... ({}s)\x1b[K",
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
            Ok((answer_text, completion_tokens, engine_mode)) => {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let latency_ms = elapsed_secs * 1000.0;
                let tok_per_sec = if elapsed_secs > 0.0001 {
                    (completion_tokens as f64) / elapsed_secs
                } else {
                    0.0
                };
                write!(output, "\rr4 > {}\x1b[K\n", answer_text)?;
                writeln!(
                    output,
                    "[stats: {} tokens | {:.2} ms | {:.1} tok/s | mode: {}]\n",
                    completion_tokens, latency_ms, tok_per_sec, engine_mode
                )?;
                output.flush()?;
            }
            Err(err) => {
                write!(
                    output,
                    "\rr4 > [!] Error communicating with local server: {}\x1b[K\n\n",
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
    user_message: &str,
) -> Result<(String, usize, String), String> {
    let payload = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": user_message
            }
        ],
        "max_tokens": 128,
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

    Ok((content, completion_tokens, mode))
}

#[cfg(test)]
mod tests {
    use super::{parse_remote_url, repeated_suffix};

    #[test]
    fn repetition_guard_detects_repeated_token_windows() {
        assert!(repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 4], 4));
        assert!(!repeated_suffix(&[1, 2, 3, 4, 1, 2, 3, 5], 4));
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
}
