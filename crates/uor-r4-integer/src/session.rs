//! Conversational chat session, role tokens, and telemetry for zero-matmul serving.

use crate::bundle::Bundle;
use crate::math::{HopfFiberPointQ30, T8ZetaState};
use crate::model::{
    SessionState, SlotTarget, DIALOGUE_CAPACITY, KEY_DIM, PERSISTENT_CAPACITY, VAL_DIM,
};
use crate::sampling::{SamplePolicy, Sampler};
use crate::{invalid, IntegerError, IntegerStep, ReadMode, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uor_r4_tokenizer::ByteBpeTokenizer;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RoleToken {
    System = 3,
    User = 4,
    Assistant = 5,
    TurnEnd = 6,
}

impl RoleToken {
    pub const BOS_ID: u32 = 0;
    pub const EOS_ID: u32 = 1;
    pub const UNK_ID: u32 = 2;
    pub const SYSTEM_ID: u32 = 3;
    pub const USER_ID: u32 = 4;
    pub const ASSISTANT_ID: u32 = 5;
    pub const TURN_END_ID: u32 = 6;

    pub const SYSTEM_STR: &'static str = "<|system|>";
    pub const USER_STR: &'static str = "<|user|>";
    pub const ASSISTANT_STR: &'static str = "<|assistant|>";
    pub const TURN_END_STR: &'static str = "<|turn_end|>";

    #[inline]
    pub const fn id(self) -> u32 {
        self as u32
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => Self::SYSTEM_STR,
            Self::User => Self::USER_STR,
            Self::Assistant => Self::ASSISTANT_STR,
            Self::TurnEnd => Self::TURN_END_STR,
        }
    }

    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            Self::SYSTEM_ID => Some(Self::System),
            Self::USER_ID => Some(Self::User),
            Self::ASSISTANT_ID => Some(Self::Assistant),
            Self::TURN_END_ID => Some(Self::TurnEnd),
            _ => None,
        }
    }

    pub fn from_tag(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl std::str::FromStr for RoleToken {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            Self::SYSTEM_STR => Ok(Self::System),
            Self::USER_STR => Ok(Self::User),
            Self::ASSISTANT_STR => Ok(Self::Assistant),
            Self::TURN_END_STR => Ok(Self::TurnEnd),
            _ => Err(()),
        }
    }
}

/// Resolved role token IDs from a serving tokenizer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleTokens {
    pub system_id: u32,
    pub user_id: u32,
    pub assistant_id: u32,
    pub turn_end_id: u32,
    pub eos_id: u32,
}

impl RoleTokens {
    pub fn from_tokenizer(tokenizer: &ByteBpeTokenizer) -> Result<Self> {
        Ok(Self {
            system_id: tokenizer
                .token_id(RoleToken::SYSTEM_STR)
                .ok_or_else(|| invalid("missing <|system|> token in tokenizer"))?,
            user_id: tokenizer
                .token_id(RoleToken::USER_STR)
                .ok_or_else(|| invalid("missing <|user|> token in tokenizer"))?,
            assistant_id: tokenizer
                .token_id(RoleToken::ASSISTANT_STR)
                .ok_or_else(|| invalid("missing <|assistant|> token in tokenizer"))?,
            turn_end_id: tokenizer
                .token_id(RoleToken::TURN_END_STR)
                .ok_or_else(|| invalid("missing <|turn_end|> token in tokenizer"))?,
            eos_id: tokenizer.token_id("<|eos|>").unwrap_or(RoleToken::EOS_ID),
        })
    }
}

/// Instantaneous memory telemetry for the active conversational session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMemoryTelemetry {
    pub persistent_slots_used: usize,
    pub persistent_capacity: usize,
    pub persistent_sealed: bool,
    pub dialogue_slots_used: usize,
    pub dialogue_capacity: usize,
    pub dialogue_cursor: usize,
    pub dialogue_tokens_seen: u64,
    pub current_turn_id: u32,
    pub cumulative_holonomy_q30: i64,
    pub zeta_phases: [i32; 8],
}

/// Incremental UTF-8 byte decode buffer for streaming token generation.
///
/// Accumulates raw bytes from byte-level BPE tokens and emits only valid UTF-8
/// strings. Multi-byte sequences (2-byte accents, 3-byte CJK, 4-byte emojis)
/// split across token boundaries are safely retained in `pending_bytes` until
/// all continuation bytes arrive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IncrementalUtf8Decoder {
    pub pending_bytes: Vec<u8>,
}

impl IncrementalUtf8Decoder {
    pub fn new() -> Self {
        Self {
            pending_bytes: Vec::new(),
        }
    }

    /// Ingests new raw token bytes and returns any completed valid UTF-8 string chunk.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Option<String> {
        self.pending_bytes.extend_from_slice(bytes);
        self.extract_valid()
    }

    /// Extracts the maximal valid UTF-8 prefix currently in `pending_bytes`.
    pub fn extract_valid(&mut self) -> Option<String> {
        if self.pending_bytes.is_empty() {
            return None;
        }

        match core::str::from_utf8(&self.pending_bytes) {
            Ok(_) => {
                let bytes = std::mem::take(&mut self.pending_bytes);
                let text = String::from_utf8(bytes).ok()?;
                if text.is_empty() {
                    None
                } else {
                    Some(text)
                }
            }
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                if valid_up_to > 0 {
                    let valid_bytes: Vec<u8> = self.pending_bytes.drain(..valid_up_to).collect();
                    String::from_utf8(valid_bytes).ok()
                } else if let Some(err_len) = e.error_len() {
                    // Malformed byte sequence: drain to prevent infinite loop and emit replacement
                    let invalid_bytes: Vec<u8> = self.pending_bytes.drain(..err_len).collect();
                    Some(String::from_utf8_lossy(&invalid_bytes).into_owned())
                } else {
                    // Incomplete sequence at end of buffer: wait for subsequent token
                    None
                }
            }
        }
    }

    /// Flushes any remaining bytes losslessly on generation termination.
    pub fn flush(&mut self) -> Option<String> {
        if self.pending_bytes.is_empty() {
            None
        } else {
            let flushed = String::from_utf8_lossy(&self.pending_bytes).into_owned();
            self.pending_bytes.clear();
            if flushed.is_empty() {
                None
            } else {
                Some(flushed)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.pending_bytes.is_empty()
    }

    pub fn pending_len(&self) -> usize {
        self.pending_bytes.len()
    }
}

pub type Utf8StreamBuffer = IncrementalUtf8Decoder;

/// Reason why token streaming terminated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamStopReason {
    TurnEnd { token_id: u32 },
    Eos { token_id: u32 },
    CustomStop { token_id: u32 },
    MaxTokens { count: usize },
    CycleDetected { period: usize },
    ExhaustedContext,
    ModelError,
}

/// Active streaming token iterator for a conversational session.
pub struct ChatTokenStream<'s, 'a> {
    session: &'s mut ChatSession<'a>,
    stop_tokens: Vec<u32>,
    max_tokens: usize,
    tokens_generated: usize,
    decoder: IncrementalUtf8Decoder,
    stopped: bool,
    stop_reason: Option<StreamStopReason>,
    generated_tokens: Vec<u32>,
    error: Option<IntegerError>,
    policy: SamplePolicy,
}

pub type TokenStream<'s, 'a> = ChatTokenStream<'s, 'a>;

impl<'s, 'a> ChatTokenStream<'s, 'a> {
    pub fn new(session: &'s mut ChatSession<'a>, max_tokens: usize, stop_tokens: &[u32]) -> Self {
        Self {
            session,
            stop_tokens: stop_tokens.to_vec(),
            max_tokens,
            tokens_generated: 0,
            decoder: IncrementalUtf8Decoder::new(),
            stopped: false,
            stop_reason: None,
            generated_tokens: Vec::new(),
            error: None,
            policy: SamplePolicy::Greedy,
        }
    }

    pub fn with_policy(mut self, policy: SamplePolicy) -> Self {
        self.policy = policy;
        self
    }

    pub fn tokens_generated(&self) -> usize {
        self.tokens_generated
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn stop_reason(&self) -> Option<StreamStopReason> {
        self.stop_reason
    }

    pub fn generated_tokens(&self) -> &[u32] {
        &self.generated_tokens
    }

    pub fn error(&self) -> Option<&IntegerError> {
        self.error.as_ref()
    }
}

fn short_cycle_check(tokens: &[u32]) -> Option<usize> {
    for period in 1..=4 {
        let twice = period << 1;
        let span = twice + period;
        if tokens.len() >= span {
            let tail = &tokens[tokens.len() - span..];
            if tail[..period] == tail[period..twice] && tail[..period] == tail[twice..] {
                return Some(period);
            }
        }
    }
    None
}

impl<'s, 'a> Iterator for ChatTokenStream<'s, 'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stopped {
            return None;
        }

        while self.tokens_generated < self.max_tokens {
            let step = match self.session.last_step.as_ref() {
                Some(s) => s,
                None => {
                    self.stopped = true;
                    self.stop_reason = Some(StreamStopReason::ModelError);
                    self.error = Some(invalid("missing model prediction in chat session"));
                    return self.decoder.flush();
                }
            };

            let selected_idx = match self
                .session
                .sampler
                .select(&step.probabilities, self.policy)
            {
                Ok(idx) => idx,
                Err(err) => {
                    self.stopped = true;
                    self.stop_reason = Some(StreamStopReason::ModelError);
                    self.error = Some(invalid(format!("sampling error: {err}")));
                    return self.decoder.flush();
                }
            };
            let selected_token = selected_idx as u32;

            let is_turn_end = selected_token == self.session.roles.turn_end_id
                || selected_token == 4099
                || selected_token == RoleToken::TURN_END_ID;
            let is_eos =
                selected_token == self.session.roles.eos_id || selected_token == RoleToken::EOS_ID;
            let is_custom_stop = self.stop_tokens.contains(&selected_token);

            if is_turn_end || is_eos || is_custom_stop {
                self.stopped = true;
                if is_turn_end {
                    self.stop_reason = Some(StreamStopReason::TurnEnd {
                        token_id: selected_token,
                    });
                } else if is_eos {
                    self.stop_reason = Some(StreamStopReason::Eos {
                        token_id: selected_token,
                    });
                } else {
                    self.stop_reason = Some(StreamStopReason::CustomStop {
                        token_id: selected_token,
                    });
                }

                // Step the stop token into dialogue memory to conclude the turn cleanly
                let _ = self.session.bundle.model().step_conversational(
                    &mut self.session.state,
                    selected_token,
                    SlotTarget::Dialogue,
                    self.session.read_mode,
                );

                // Stop token is NOT emitted to user stream; flush any pending bytes
                return self.decoder.flush();
            }

            self.tokens_generated += 1;
            self.generated_tokens.push(selected_token);

            // Step token into conversational dialogue memory
            match self.session.bundle.model().step_conversational(
                &mut self.session.state,
                selected_token,
                SlotTarget::Dialogue,
                self.session.read_mode,
            ) {
                Ok(next_step) => {
                    self.session.last_step = Some(next_step);
                }
                Err(err) => {
                    self.stopped = true;
                    self.stop_reason = Some(StreamStopReason::ExhaustedContext);
                    self.error = Some(err);
                    return self.decoder.flush();
                }
            }

            if let Some(period) = short_cycle_check(&self.generated_tokens) {
                self.stopped = true;
                self.stop_reason = Some(StreamStopReason::CycleDetected { period });
                let _ = self.session.bundle.model().step_conversational(
                    &mut self.session.state,
                    self.session.roles.turn_end_id,
                    SlotTarget::Dialogue,
                    self.session.read_mode,
                );
                return self.decoder.flush();
            }

            let raw_bytes = self
                .session
                .bundle
                .tokenizer()
                .decode_bytes(&[selected_token]);
            if let Some(chunk) = self.decoder.push_bytes(&raw_bytes) {
                return Some(chunk);
            }
        }

        self.stopped = true;
        self.stop_reason = Some(StreamStopReason::MaxTokens {
            count: self.tokens_generated,
        });
        let _ = self.session.bundle.model().step_conversational(
            &mut self.session.state,
            self.session.roles.turn_end_id,
            SlotTarget::Dialogue,
            self.session.read_mode,
        );
        self.decoder.flush()
    }
}

/// The authoritative serialization schema identifier for chat sessions.
pub const SESSION_SCHEMA_V1: &str = "uor-r4.integer-session/1";

/// Top-level serialized chat session container matching `uor-r4.integer-session/1`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializedChatSession {
    pub schema: String,
    pub version: u32,
    pub bundle_sha256: String,
    pub timestamp: String,
    pub read_mode: ReadMode,
    pub session_state: SerializedSessionState,
    pub sampler_state: SerializedSamplerState,
}

/// Serialized partitioned memory and geometric recurrent state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializedSessionState {
    pub identity: String,
    pub state: Vec<i32>,
    pub persistent_keys: Vec<Vec<i32>>,
    pub persistent_values: Vec<Vec<i32>>,
    pub persistent_tokens: Vec<u32>,
    pub persistent_capacity: usize,
    pub persistent_sealed: bool,
    pub dialogue_keys: Vec<Vec<i32>>,
    pub dialogue_values: Vec<Vec<i32>>,
    pub dialogue_tokens: Vec<u32>,
    pub dialogue_sequences: Vec<u64>,
    pub dialogue_turn_ids: Vec<u32>,
    pub dialogue_capacity: usize,
    pub dialogue_cursor: usize,
    pub dialogue_len: usize,
    pub dialogue_seen: u64,
    pub current_turn_id: u32,
    pub zeta_state: T8ZetaState,
    pub hopf_state: HopfFiberPointQ30,
    pub cumulative_holonomy_q30: i64,
    pub age_horizon_clamp: usize,
}

/// Serialized sampler state and active policy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializedSamplerState {
    pub state: u64,
    pub policy: SamplePolicy,
}

/// Stateful chat session managing prompt formatting, persona retention, and multi-turn state.
pub struct ChatSession<'a> {
    bundle: &'a Bundle,
    state: SessionState,
    roles: RoleTokens,
    sampler: Sampler,
    last_step: Option<IntegerStep>,
    read_mode: ReadMode,
    policy: SamplePolicy,
}

impl<'a> ChatSession<'a> {
    /// Initialize a new chat session with optional system prompt.
    pub fn new(bundle: &'a Bundle, system_prompt: Option<&str>, seed: u64) -> Result<Self> {
        let roles = RoleTokens::from_tokenizer(bundle.tokenizer())?;
        let state = bundle.model().new_conversational_session();
        let mut session = Self {
            bundle,
            state,
            roles,
            sampler: Sampler::new(seed),
            last_step: None,
            read_mode: ReadMode::Enabled,
            policy: SamplePolicy::Greedy,
        };
        if let Some(prompt) = system_prompt {
            session.ingest_system_prompt(prompt)?;
        } else {
            session.state.seal_persistent();
        }
        Ok(session)
    }

    /// Ingest a system prompt into the persistent session slots (0..32) and seal the partition.
    pub fn ingest_system_prompt(&mut self, system_text: &str) -> Result<usize> {
        if self.state.is_persistent_sealed() {
            return Err(invalid("persistent persona partition is already sealed"));
        }
        let formatted = format!(
            "{}{}{}",
            RoleToken::SYSTEM_STR,
            system_text,
            RoleToken::TURN_END_STR
        );
        let tokens = self.bundle.tokenizer().encode(&formatted);
        if tokens.is_empty() {
            return Err(invalid("empty system prompt"));
        }
        if tokens.len() > PERSISTENT_CAPACITY {
            return Err(invalid(format!(
                "system prompt token count {} exceeds persistent slot capacity of {}",
                tokens.len(),
                PERSISTENT_CAPACITY
            )));
        }
        for &token in &tokens {
            let step = self.bundle.model().step_conversational(
                &mut self.state,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )?;
            self.last_step = Some(step);
        }
        self.state.seal_persistent();
        Ok(tokens.len())
    }

    /// Ingest a user turn into the dialogue ring buffer, advancing the turn counter.
    pub fn ingest_user_turn(&mut self, user_text: &str) -> Result<usize> {
        self.state.start_turn();
        let formatted = format!(
            "{}{}{}{}",
            RoleToken::USER_STR,
            user_text,
            RoleToken::TURN_END_STR,
            RoleToken::ASSISTANT_STR
        );
        let tokens = self.bundle.tokenizer().encode(&formatted);
        if tokens.is_empty() {
            return Err(invalid("empty user turn prompt"));
        }
        for &token in &tokens {
            let step = self.bundle.model().step_conversational(
                &mut self.state,
                token,
                SlotTarget::Dialogue,
                self.read_mode,
            )?;
            self.last_step = Some(step);
        }
        Ok(tokens.len())
    }

    /// Step an individual token into the conversational session.
    pub fn step_token(&mut self, token: u32, target: SlotTarget) -> Result<IntegerStep> {
        let step = self.bundle.model().step_conversational(
            &mut self.state,
            token,
            target,
            self.read_mode,
        )?;
        self.last_step = Some(step);
        Ok(self.last_step.as_ref().unwrap().clone())
    }

    /// Reset dialogue history while keeping persistent persona intact.
    pub fn reset_dialogue(&mut self) {
        self.state.dialogue_cursor = 0;
        self.state.dialogue_len = 0;
        self.state.dialogue_seen = 0;
        self.state.current_turn_id = 0;
    }

    /// Report current memory telemetry.
    pub fn telemetry(&self) -> ChatMemoryTelemetry {
        ChatMemoryTelemetry {
            persistent_slots_used: self.state.persistent_len(),
            persistent_capacity: PERSISTENT_CAPACITY,
            persistent_sealed: self.state.is_persistent_sealed(),
            dialogue_slots_used: self.state.dialogue_len(),
            dialogue_capacity: DIALOGUE_CAPACITY,
            dialogue_cursor: self.state.dialogue_cursor,
            dialogue_tokens_seen: self.state.dialogue_seen,
            current_turn_id: self.state.current_turn(),
            cumulative_holonomy_q30: self.state.holonomy_accumulator(),
            zeta_phases: self.state.zeta_phases(),
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SessionState {
        &mut self.state
    }

    pub fn last_step(&self) -> Option<&IntegerStep> {
        self.last_step.as_ref()
    }

    pub fn roles(&self) -> RoleTokens {
        self.roles
    }

    pub fn bundle(&self) -> &'a Bundle {
        self.bundle
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn sampler_mut(&mut self) -> &mut Sampler {
        &mut self.sampler
    }

    pub fn read_mode(&self) -> ReadMode {
        self.read_mode
    }

    pub fn set_read_mode(&mut self, mode: ReadMode) {
        self.read_mode = mode;
    }

    pub fn policy(&self) -> SamplePolicy {
        self.policy
    }

    pub fn set_policy(&mut self, policy: SamplePolicy) {
        self.policy = policy;
    }

    /// Generates an autoregressive streaming token iterator for a prompt or user turn.
    pub fn generate_stream<'s>(
        &'s mut self,
        prompt: &str,
        max_tokens: usize,
        stop_tokens: &'s [u32],
    ) -> Result<ChatTokenStream<'s, 'a>> {
        if !prompt.trim().is_empty() {
            self.ingest_user_turn(prompt)?;
        } else if self.last_step.is_none() {
            return Err(invalid(
                "cannot generate stream: empty prompt and no prior step",
            ));
        }
        let policy = self.policy;
        Ok(ChatTokenStream::new(self, max_tokens, stop_tokens).with_policy(policy))
    }

    /// Step a single token in the conversational stream and decode incremental UTF-8.
    pub fn step_stream(&mut self, token: u32, pending: &mut Vec<u8>) -> Result<Option<String>> {
        self.step_token(token, SlotTarget::Dialogue)?;
        let raw_bytes = self.bundle.tokenizer().decode_bytes(&[token]);
        pending.extend_from_slice(&raw_bytes);
        if pending.is_empty() {
            return Ok(None);
        }
        match core::str::from_utf8(pending) {
            Ok(_) => {
                let bytes = std::mem::take(pending);
                let text =
                    String::from_utf8(bytes).map_err(|e| invalid(format!("utf-8 error: {e}")))?;
                if text.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(text))
                }
            }
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                if valid_up_to > 0 {
                    let valid_bytes: Vec<u8> = pending.drain(..valid_up_to).collect();
                    let text = String::from_utf8(valid_bytes)
                        .map_err(|e| invalid(format!("utf-8 error: {e}")))?;
                    Ok(Some(text))
                } else if let Some(err_len) = e.error_len() {
                    let invalid_bytes: Vec<u8> = pending.drain(..err_len).collect();
                    Ok(Some(String::from_utf8_lossy(&invalid_bytes).into_owned()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Export active session state to DTO structure.
    pub fn to_serialized(&self, bundle_sha256: Option<&str>) -> SerializedChatSession {
        let sha = bundle_sha256
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.bundle.identity().to_string());
        SerializedChatSession {
            schema: SESSION_SCHEMA_V1.to_string(),
            version: 1,
            bundle_sha256: sha,
            timestamp: "2026-09-26T00:00:00Z".to_string(),
            read_mode: self.read_mode,
            session_state: SerializedSessionState {
                identity: self.state.identity.clone(),
                state: self.state.state.clone(),
                persistent_keys: self
                    .state
                    .persistent_keys
                    .iter()
                    .map(|k| k.to_vec())
                    .collect(),
                persistent_values: self
                    .state
                    .persistent_values
                    .iter()
                    .map(|v| v.to_vec())
                    .collect(),
                persistent_tokens: self.state.persistent_tokens.clone(),
                persistent_capacity: self.state.persistent_capacity,
                persistent_sealed: self.state.persistent_sealed,
                dialogue_keys: self
                    .state
                    .dialogue_keys
                    .iter()
                    .map(|k| k.to_vec())
                    .collect(),
                dialogue_values: self
                    .state
                    .dialogue_values
                    .iter()
                    .map(|v| v.to_vec())
                    .collect(),
                dialogue_tokens: self.state.dialogue_tokens.clone(),
                dialogue_sequences: self.state.dialogue_sequences.clone(),
                dialogue_turn_ids: self.state.dialogue_turn_ids.clone(),
                dialogue_capacity: self.state.dialogue_capacity,
                dialogue_cursor: self.state.dialogue_cursor,
                dialogue_len: self.state.dialogue_len,
                dialogue_seen: self.state.dialogue_seen,
                current_turn_id: self.state.current_turn_id,
                zeta_state: self.state.zeta_state,
                hopf_state: self.state.hopf_state,
                cumulative_holonomy_q30: self.state.cumulative_holonomy_q30,
                age_horizon_clamp: self.state.age_horizon_clamp,
            },
            sampler_state: SerializedSamplerState {
                state: self.sampler.state(),
                policy: self.policy,
            },
        }
    }

    /// Save active conversational session and PRNG state to JSON file at `path`.
    /// `bundle_sha256` explicitly binds the model bundle identity. If empty, defaults to active bundle identity.
    pub fn save_session(&self, path: &Path, bundle_sha256: &str) -> Result<()> {
        let sha = if bundle_sha256.is_empty() {
            self.bundle.identity()
        } else {
            bundle_sha256
        };
        let serialized = self.to_serialized(Some(sha));
        let json_bytes = serde_json::to_vec_pretty(&serialized)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, json_bytes)?;
        Ok(())
    }

    /// Save session defaulting to active bundle SHA-256.
    pub fn save_session_default(&self, path: &Path) -> Result<()> {
        self.save_session(path, self.bundle.identity())
    }

    /// Load and restore session from JSON file at `path`, strictly verifying bundle SHA-256 checksum.
    pub fn load_session(
        bundle: &'a Bundle,
        path: &Path,
        expected_bundle_sha256: &str,
    ) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let serialized: SerializedChatSession = serde_json::from_slice(&bytes)?;
        Self::from_serialized(bundle, serialized, expected_bundle_sha256)
    }

    /// Restore session from deserialized DTO with strict schema and bundle verification.
    pub fn from_serialized(
        bundle: &'a Bundle,
        serialized: SerializedChatSession,
        expected_bundle_sha256: &str,
    ) -> Result<Self> {
        if serialized.schema != SESSION_SCHEMA_V1 {
            return Err(invalid(format!(
                "unsupported session schema: expected '{}', found '{}'",
                SESSION_SCHEMA_V1, serialized.schema
            )));
        }
        if serialized.version != 1 {
            return Err(invalid(format!(
                "unsupported session version: expected 1, found {}",
                serialized.version
            )));
        }

        let active_bundle_sha = bundle.identity();
        if !expected_bundle_sha256.is_empty() && serialized.bundle_sha256 != expected_bundle_sha256
        {
            return Err(invalid(format!(
                "session bundle checksum mismatch: session has '{}', expected '{}'",
                serialized.bundle_sha256, expected_bundle_sha256
            )));
        }
        if serialized.bundle_sha256 != active_bundle_sha {
            return Err(invalid(format!(
                "session bundle checksum mismatch: session requires '{}', active bundle is '{}'",
                serialized.bundle_sha256, active_bundle_sha
            )));
        }

        let s_state = serialized.session_state;
        if s_state.state.len() != 256 {
            return Err(invalid(format!(
                "corrupted session state: expected recurrent state dimension 256, found {}",
                s_state.state.len()
            )));
        }
        if s_state.persistent_keys.len() > PERSISTENT_CAPACITY {
            return Err(invalid(format!(
                "persistent slot count {} exceeds capacity {}",
                s_state.persistent_keys.len(),
                PERSISTENT_CAPACITY
            )));
        }
        if s_state.dialogue_keys.len() != DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "corrupted dialogue keys: expected {} slots, found {}",
                DIALOGUE_CAPACITY,
                s_state.dialogue_keys.len()
            )));
        }
        if s_state.dialogue_values.len() != DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "corrupted dialogue values: expected {} slots, found {}",
                DIALOGUE_CAPACITY,
                s_state.dialogue_values.len()
            )));
        }
        if s_state.dialogue_cursor >= DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "dialogue cursor {} out of bounds [0, {})",
                s_state.dialogue_cursor, DIALOGUE_CAPACITY
            )));
        }

        for (i, key) in s_state.dialogue_keys.iter().enumerate() {
            if key.len() != KEY_DIM {
                return Err(invalid(format!(
                    "corrupted dialogue key at slot {i}: expected dimension {KEY_DIM}, found {}",
                    key.len()
                )));
            }
        }
        for (i, val) in s_state.dialogue_values.iter().enumerate() {
            if val.len() != VAL_DIM {
                return Err(invalid(format!(
                    "corrupted dialogue value at slot {i}: expected dimension {VAL_DIM}, found {}",
                    val.len()
                )));
            }
        }

        let mut dialogue_keys = Box::new([[0i32; KEY_DIM]; DIALOGUE_CAPACITY]);
        for (i, key) in s_state.dialogue_keys.into_iter().enumerate() {
            dialogue_keys[i].copy_from_slice(&key);
        }

        let mut dialogue_values = Box::new([[0i32; VAL_DIM]; DIALOGUE_CAPACITY]);
        for (i, val) in s_state.dialogue_values.into_iter().enumerate() {
            dialogue_values[i].copy_from_slice(&val);
        }

        let mut persistent_keys = Vec::with_capacity(s_state.persistent_keys.len());
        for (i, key) in s_state.persistent_keys.into_iter().enumerate() {
            if key.len() != KEY_DIM {
                return Err(invalid(format!(
                    "corrupted persistent key at slot {i}: expected dimension {KEY_DIM}, found {}",
                    key.len()
                )));
            }
            let mut arr = [0i32; KEY_DIM];
            arr.copy_from_slice(&key);
            persistent_keys.push(arr);
        }

        let mut persistent_values = Vec::with_capacity(s_state.persistent_values.len());
        for (i, val) in s_state.persistent_values.into_iter().enumerate() {
            if val.len() != VAL_DIM {
                return Err(invalid(format!(
                    "corrupted persistent value at slot {i}: expected dimension {VAL_DIM}, found {}",
                    val.len()
                )));
            }
            let mut arr = [0i32; VAL_DIM];
            arr.copy_from_slice(&val);
            persistent_values.push(arr);
        }

        let state = SessionState {
            identity: s_state.identity,
            state: s_state.state,
            persistent_keys,
            persistent_values,
            persistent_tokens: s_state.persistent_tokens,
            persistent_capacity: s_state.persistent_capacity,
            persistent_sealed: s_state.persistent_sealed,
            dialogue_keys,
            dialogue_values,
            dialogue_tokens: s_state.dialogue_tokens,
            dialogue_sequences: s_state.dialogue_sequences,
            dialogue_turn_ids: s_state.dialogue_turn_ids,
            dialogue_capacity: s_state.dialogue_capacity,
            dialogue_cursor: s_state.dialogue_cursor,
            dialogue_len: s_state.dialogue_len,
            dialogue_seen: s_state.dialogue_seen,
            current_turn_id: s_state.current_turn_id,
            zeta_state: s_state.zeta_state,
            hopf_state: s_state.hopf_state,
            cumulative_holonomy_q30: s_state.cumulative_holonomy_q30,
            age_horizon_clamp: s_state.age_horizon_clamp,
        };

        let roles = RoleTokens::from_tokenizer(bundle.tokenizer())?;
        let sampler = Sampler::from_state(serialized.sampler_state.state);

        Ok(Self {
            bundle,
            state,
            roles,
            sampler,
            last_step: None,
            read_mode: serialized.read_mode,
            policy: serialized.sampler_state.policy,
        })
    }
}

impl Bundle {
    pub fn load_chat_session<'a>(
        &'a self,
        path: &Path,
        expected_bundle_sha256: &str,
    ) -> Result<ChatSession<'a>> {
        ChatSession::load_session(self, path, expected_bundle_sha256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_utf8_split_emoji_4bytes() {
        let mut decoder = IncrementalUtf8Decoder::new();
        // Rocket emoji 🚀 = [0xF0, 0x9F, 0x9A, 0x80]
        let token1_bytes = [240u8, 159]; // First 2 bytes
        let token2_bytes = [154u8, 128]; // Last 2 bytes

        assert_eq!(decoder.push_bytes(&token1_bytes), None);
        assert_eq!(decoder.pending_len(), 2);

        assert_eq!(decoder.push_bytes(&token2_bytes), Some("🚀".to_string()));
        assert_eq!(decoder.pending_len(), 0);
        assert!(decoder.is_empty());
    }

    #[test]
    fn test_incremental_utf8_ascii_prefix_with_split_emoji() {
        let mut decoder = IncrementalUtf8Decoder::new();
        let mut token1 = b"Launch: ".to_vec();
        token1.push(0xF0);

        let token2 = [0x9F, 0x9A];

        let mut token3 = vec![0x80];
        token3.extend_from_slice(b" done!");

        assert_eq!(decoder.push_bytes(&token1), Some("Launch: ".to_string()));
        assert_eq!(decoder.pending_len(), 1);

        assert_eq!(decoder.push_bytes(&token2), None);
        assert_eq!(decoder.pending_len(), 3);

        assert_eq!(decoder.push_bytes(&token3), Some("🚀 done!".to_string()));
        assert!(decoder.is_empty());
    }

    #[test]
    fn test_incremental_utf8_split_accented_2bytes() {
        let mut decoder = IncrementalUtf8Decoder::new();
        // 'é' = [0xC3, 0xA9]
        assert_eq!(decoder.push_bytes(&[0xC3]), None);
        assert_eq!(decoder.push_bytes(&[0xA9]), Some("é".to_string()));
        assert!(decoder.is_empty());
    }

    #[test]
    fn test_incremental_utf8_split_cjk_3bytes() {
        let mut decoder = IncrementalUtf8Decoder::new();
        // '中' = [0xE4, 0xB8, 0xAD]
        assert_eq!(decoder.push_bytes(&[0xE4]), None);
        assert_eq!(decoder.push_bytes(&[0xB8]), None);
        assert_eq!(decoder.push_bytes(&[0xAD]), Some("中".to_string()));
        assert!(decoder.is_empty());
    }

    #[test]
    fn test_incremental_utf8_flush_incomplete_at_turn_end() {
        let mut decoder = IncrementalUtf8Decoder::new();
        assert_eq!(decoder.push_bytes(&[0xC3]), None);
        assert_eq!(decoder.flush(), Some("\u{FFFD}".to_string()));
        assert!(decoder.is_empty());
        assert_eq!(decoder.flush(), None);
    }
}
