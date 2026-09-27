//! Conversational chat session, role tokens, and telemetry for zero-matmul serving.

use crate::bundle::Bundle;
use crate::math::{HopfFiberPointQ30, T8ZetaState};
use crate::model::{
    L2PrimePage, SessionState, SlotTarget, DIALOGUE_CAPACITY, KEY_DIM, L2_PAGE_CAPACITY,
    PERSISTENT_CAPACITY, TOTAL_MEMORY_CANDIDATES, VAL_DIM,
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
        let eos_id = tokenizer.token_id("<|eos|>").unwrap_or(RoleToken::EOS_ID);
        let turn_end_id = tokenizer
            .token_id(RoleToken::TURN_END_STR)
            .or_else(|| tokenizer.token_id("<|eos|>"))
            .unwrap_or(RoleToken::EOS_ID);
        let system_id = tokenizer
            .token_id(RoleToken::SYSTEM_STR)
            .or_else(|| tokenizer.token_id("<|bos|>"))
            .unwrap_or(RoleToken::BOS_ID);
        let user_id = tokenizer
            .token_id(RoleToken::USER_STR)
            .or_else(|| tokenizer.token_id("<|unk|>"))
            .unwrap_or(RoleToken::UNK_ID);
        let assistant_id = tokenizer
            .token_id(RoleToken::ASSISTANT_STR)
            .or_else(|| tokenizer.token_id("<|bos|>"))
            .unwrap_or(RoleToken::BOS_ID);

        // Verify that none of the resolved role tokens collide with ASCII punctuation tokens (!, ", #, $)
        let punctuation = ["!", "\"", "#", "$"];
        for punct in &punctuation {
            if let Some(punct_id) = tokenizer.token_id(punct) {
                if system_id == punct_id
                    || user_id == punct_id
                    || assistant_id == punct_id
                    || turn_end_id == punct_id
                {
                    return Err(invalid(format!(
                        "role token collision detected: token id {punct_id} is assigned to ASCII punctuation '{punct}'"
                    )));
                }
            }
        }

        Ok(Self {
            system_id,
            user_id,
            assistant_id,
            turn_end_id,
            eos_id,
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
    #[serde(default)]
    pub l2_pages_used: usize,
    #[serde(default)]
    pub l2_capacity: usize,
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

    /// Commit the terminal token before reporting a successful stop. A failed
    /// model step may have changed state; retain its error rather than implying
    /// that the turn closed successfully. Already buffered text is still flushed.
    fn finish_turn(&mut self, token: u32, reason: StreamStopReason) -> Option<String> {
        self.stopped = true;
        match self.session.bundle.model().step_conversational_into(
            &mut self.session.state,
            token,
            SlotTarget::Dialogue,
            self.session.read_mode,
        ) {
            Ok(()) => {
                self.stop_reason = Some(reason);
                self.session.sync_last_step();
            }
            Err(error) => {
                self.stop_reason = Some(StreamStopReason::ModelError);
                self.error = Some(error);
            }
        }
        self.decoder.flush()
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
            if !self.session.state.has_step && self.session.last_step.is_none() {
                self.stopped = true;
                self.stop_reason = Some(StreamStopReason::ModelError);
                self.error = Some(invalid("missing model prediction in chat session"));
                return self.decoder.flush();
            }

            let vocab_size = self.session.bundle.model().config().vocab_size;
            let mut selected_idx = match self.session.sampler.select(
                &self.session.state.last_probabilities[..vocab_size],
                self.policy,
            ) {
                Ok(idx) => idx,
                Err(err) => {
                    self.stopped = true;
                    self.stop_reason = Some(StreamStopReason::ModelError);
                    self.error = Some(invalid(format!("sampling error: {err}")));
                    return self.decoder.flush();
                }
            };
            let mut selected_token = selected_idx as u32;

            // Empty response mitigation: if turn_end_id is selected on token 0,
            // allow a one-step retry if categorical sampling is active.
            if self.tokens_generated == 0
                && selected_token == self.session.roles.turn_end_id
                && matches!(self.policy, SamplePolicy::Categorical { .. })
            {
                let turn_end_idx = selected_token as usize;
                if turn_end_idx < self.session.state.last_probabilities.len() {
                    let saved_prob = self.session.state.last_probabilities[turn_end_idx];
                    self.session.state.last_probabilities[turn_end_idx] = 0;
                    if let Ok(retry_idx) = self.session.sampler.select(
                        &self.session.state.last_probabilities[..vocab_size],
                        self.policy,
                    ) {
                        selected_idx = retry_idx;
                        selected_token = selected_idx as u32;
                    }
                    self.session.state.last_probabilities[turn_end_idx] = saved_prob;
                }
            }

            let is_turn_end = selected_token == self.session.roles.turn_end_id;
            let is_eos = selected_token == self.session.roles.eos_id;
            let is_custom_stop = self.stop_tokens.contains(&selected_token);

            if is_turn_end || is_eos || is_custom_stop {
                let reason = if is_turn_end {
                    StreamStopReason::TurnEnd {
                        token_id: selected_token,
                    }
                } else if is_eos {
                    StreamStopReason::Eos {
                        token_id: selected_token,
                    }
                } else {
                    StreamStopReason::CustomStop {
                        token_id: selected_token,
                    }
                };
                // Stop token is NOT emitted to user stream; flush any pending bytes
                return self.finish_turn(selected_token, reason);
            }

            self.tokens_generated += 1;
            self.generated_tokens.push(selected_token);

            // Step token into conversational dialogue memory
            match self.session.bundle.model().step_conversational_into(
                &mut self.session.state,
                selected_token,
                SlotTarget::Dialogue,
                self.session.read_mode,
            ) {
                Ok(()) => {}
                Err(err) => {
                    self.stopped = true;
                    self.stop_reason = Some(StreamStopReason::ExhaustedContext);
                    self.error = Some(err);
                    return self.decoder.flush();
                }
            }

            if let Some(period) = short_cycle_check(&self.generated_tokens) {
                return self.finish_turn(
                    self.session.roles.turn_end_id,
                    StreamStopReason::CycleDetected { period },
                );
            }

            // Suppress literal `<|bos|>` (token 0) and `<|unk|>` (token 2) string tags
            // from being emitted to user stream if generated by model.
            let is_control_suppressed =
                selected_token == RoleToken::BOS_ID || selected_token == RoleToken::UNK_ID;
            if is_control_suppressed {
                continue;
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

        self.finish_turn(
            self.session.roles.turn_end_id,
            StreamStopReason::MaxTokens {
                count: self.tokens_generated,
            },
        )
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
    #[serde(default)]
    pub l2_pages: Vec<L2PrimePage>,
    #[serde(default)]
    pub l2_cursor: usize,
    #[serde(default)]
    pub l2_len: usize,
    #[serde(default)]
    pub l2_seen: u64,
    #[serde(default = "default_last_compressed_turn_id")]
    pub last_compressed_turn_id: u32,
    #[serde(default)]
    pub allow_hyperbolic_cache: bool,
}

fn default_last_compressed_turn_id() -> u32 {
    u32::MAX
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
        let mut state = bundle.model().new_conversational_session();
        if bundle.model().is_lorentz() {
            state.enable_hyperbolic_cache();
        }
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
        }
        Ok(session)
    }

    /// Synchronize the optional cached `IntegerStep` in-place without heap reallocations.
    pub fn sync_last_step(&mut self) {
        let vocab_size = self.bundle.model().config().vocab_size;
        let width = self.bundle.model().config().width;
        if let Some(ref mut step) = self.last_step {
            step.probabilities.clear();
            step.probabilities
                .extend_from_slice(&self.state.last_probabilities[..vocab_size]);
            step.state.clear();
            step.state.extend_from_slice(&self.state.state[..width]);
            step.read_masses.clear();
            step.read_masses
                .extend_from_slice(&self.state.last_read_masses);
            step.no_read_mass = self.state.last_no_read_mass;
            step.copy_gate = self.state.last_copy_gate;
        } else {
            self.last_step = Some(IntegerStep {
                probabilities: self.state.last_probabilities[..vocab_size].to_vec(),
                state: self.state.state[..width].to_vec(),
                no_read_mass: self.state.last_no_read_mass,
                read_masses: self.state.last_read_masses.clone(),
                copy_gate: self.state.last_copy_gate,
            });
        }
    }

    /// Ingest a system prompt into the persistent session slots (0..32) and seal the partition.
    pub fn ingest_system_prompt(&mut self, system_text: &str) -> Result<usize> {
        if self.state.is_persistent_sealed() {
            return Err(invalid("persistent persona partition is already sealed"));
        }
        if system_text.trim().is_empty() {
            return Err(invalid("empty system prompt"));
        }
        let text_tokens = self.bundle.tokenizer().encode(system_text);
        let mut tokens = Vec::with_capacity(text_tokens.len() + 2);
        tokens.push(self.roles.system_id);
        tokens.extend(text_tokens);
        tokens.push(self.roles.turn_end_id);

        if tokens.len() > PERSISTENT_CAPACITY {
            return Err(invalid(format!(
                "system prompt token count {} exceeds persistent slot capacity of {}",
                tokens.len(),
                PERSISTENT_CAPACITY
            )));
        }
        for &token in &tokens {
            self.bundle.model().step_conversational_into(
                &mut self.state,
                token,
                SlotTarget::Persistent,
                ReadMode::Enabled,
            )?;
        }
        self.sync_last_step();
        self.state.seal_persistent();
        Ok(tokens.len())
    }

    /// Ingest a user turn into the dialogue ring buffer, advancing the turn counter.
    pub fn ingest_user_turn(&mut self, user_text: &str) -> Result<usize> {
        self.state.start_turn();
        if user_text.trim().is_empty() {
            return Err(invalid("empty user turn prompt"));
        }
        let user_tokens = self.bundle.tokenizer().encode(user_text);
        let mut tokens = Vec::with_capacity(user_tokens.len() + 3);
        tokens.push(self.roles.user_id);
        tokens.extend(user_tokens);
        tokens.push(self.roles.turn_end_id);
        tokens.push(self.roles.assistant_id);

        for &token in &tokens {
            self.bundle.model().step_conversational_into(
                &mut self.state,
                token,
                SlotTarget::Dialogue,
                self.read_mode,
            )?;
        }
        self.sync_last_step();
        Ok(tokens.len())
    }

    /// Step an individual token into the conversational session.
    pub fn step_token(&mut self, token: u32, target: SlotTarget) -> Result<IntegerStep> {
        self.bundle.model().step_conversational_into(
            &mut self.state,
            token,
            target,
            self.read_mode,
        )?;
        self.sync_last_step();
        Ok(self.last_step.as_ref().unwrap().clone())
    }

    /// Reset dialogue history while keeping persistent persona intact.
    pub fn reset_dialogue(&mut self) {
        self.state.dialogue_cursor = 0;
        self.state.dialogue_len = 0;
        self.state.dialogue_seen = 0;
        self.state.current_turn_id = 0;
        self.state.l2_cursor = 0;
        self.state.l2_len = 0;
        self.state.l2_seen = 0;
        self.state.last_compressed_turn_id = u32::MAX;
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
            l2_pages_used: self.state.l2_len,
            l2_capacity: L2_PAGE_CAPACITY,
        }
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SessionState {
        &mut self.state
    }

    pub fn active_slots_count(&self) -> usize {
        self.state.active_slots_count()
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

    pub fn step_stream(&mut self, token: u32, pending: &mut Vec<u8>) -> Result<Option<String>> {
        self.bundle.model().step_conversational_into(
            &mut self.state,
            token,
            SlotTarget::Dialogue,
            self.read_mode,
        )?;

        let raw_bytes = self.bundle.tokenizer().decode_bytes(&[token]);
        pending.extend_from_slice(&raw_bytes);
        if pending.is_empty() {
            return Ok(None);
        }
        match core::str::from_utf8(pending) {
            Ok(s) => {
                if s.is_empty() {
                    pending.clear();
                    Ok(None)
                } else {
                    let text = s.to_owned();
                    pending.clear();
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
                l2_pages: self.state.l2_pages[..self.state.l2_len].to_vec(),
                l2_cursor: self.state.l2_cursor,
                l2_len: self.state.l2_len,
                l2_seen: self.state.l2_seen,
                last_compressed_turn_id: self.state.last_compressed_turn_id,
                allow_hyperbolic_cache: self.state.allow_hyperbolic_cache,
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
        if s_state.persistent_capacity != PERSISTENT_CAPACITY {
            return Err(invalid(format!(
                "corrupted persistent capacity: expected {}, found {}",
                PERSISTENT_CAPACITY, s_state.persistent_capacity
            )));
        }
        if s_state.persistent_keys.len() > PERSISTENT_CAPACITY {
            return Err(invalid(format!(
                "persistent slot count {} exceeds capacity {}",
                s_state.persistent_keys.len(),
                PERSISTENT_CAPACITY
            )));
        }
        if s_state.persistent_values.len() != s_state.persistent_keys.len() {
            return Err(invalid(format!(
                "corrupted persistent values count: expected {} (matching keys), found {}",
                s_state.persistent_keys.len(),
                s_state.persistent_values.len()
            )));
        }
        if s_state.persistent_tokens.len() != s_state.persistent_keys.len() {
            return Err(invalid(format!(
                "corrupted persistent tokens count: expected {} (matching keys), found {}",
                s_state.persistent_keys.len(),
                s_state.persistent_tokens.len()
            )));
        }
        if s_state.dialogue_capacity != DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "corrupted dialogue capacity: expected {}, found {}",
                DIALOGUE_CAPACITY, s_state.dialogue_capacity
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
        if s_state.dialogue_tokens.len() != DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "corrupted dialogue tokens: expected {} slots, found {}",
                DIALOGUE_CAPACITY,
                s_state.dialogue_tokens.len()
            )));
        }
        if s_state.dialogue_sequences.len() != DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "corrupted dialogue sequences: expected {} slots, found {}",
                DIALOGUE_CAPACITY,
                s_state.dialogue_sequences.len()
            )));
        }
        if s_state.dialogue_turn_ids.len() != DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "corrupted dialogue turn IDs: expected {} slots, found {}",
                DIALOGUE_CAPACITY,
                s_state.dialogue_turn_ids.len()
            )));
        }
        if s_state.dialogue_cursor >= DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "dialogue cursor {} out of bounds [0, {})",
                s_state.dialogue_cursor, DIALOGUE_CAPACITY
            )));
        }
        if s_state.dialogue_len > DIALOGUE_CAPACITY {
            return Err(invalid(format!(
                "dialogue len {} exceeds capacity {}",
                s_state.dialogue_len, DIALOGUE_CAPACITY
            )));
        }

        for (i, key) in s_state.dialogue_keys.iter().enumerate() {
            if key.len() != KEY_DIM {
                return Err(invalid(format!(
                    "corrupted dialogue key at slot {i}: expected dimension {KEY_DIM}, found {}",
                    key.len()
                )));
            }
            for (d, &coord) in key.iter().enumerate() {
                if !(-32767..=32767).contains(&coord) {
                    return Err(invalid(format!(
                        "corrupted dialogue key coordinate at slot {i}, dim {d}: {coord} outside [-32767, 32767]"
                    )));
                }
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
            for (d, &coord) in key.iter().enumerate() {
                if !(-32767..=32767).contains(&coord) {
                    return Err(invalid(format!(
                        "corrupted persistent key coordinate at slot {i}, dim {d}: {coord} outside [-32767, 32767]"
                    )));
                }
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

        if s_state.l2_pages.len() > L2_PAGE_CAPACITY {
            return Err(invalid(format!(
                "l2 page count {} exceeds capacity {}",
                s_state.l2_pages.len(),
                L2_PAGE_CAPACITY
            )));
        }
        if s_state.l2_cursor >= L2_PAGE_CAPACITY {
            return Err(invalid(format!(
                "l2 cursor {} out of bounds [0, {})",
                s_state.l2_cursor, L2_PAGE_CAPACITY
            )));
        }
        if s_state.l2_len > L2_PAGE_CAPACITY {
            return Err(invalid(format!(
                "l2 len {} exceeds capacity {}",
                s_state.l2_len, L2_PAGE_CAPACITY
            )));
        }
        if s_state.l2_pages.len() < s_state.l2_len {
            return Err(invalid(format!(
                "l2 pages count {} is less than l2_len {}",
                s_state.l2_pages.len(),
                s_state.l2_len
            )));
        }

        let mut l2_pages: Box<[L2PrimePage; L2_PAGE_CAPACITY]> =
            vec![L2PrimePage::default(); L2_PAGE_CAPACITY]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| invalid("l2_pages size mismatch"))?;
        for (i, page) in s_state.l2_pages.into_iter().enumerate() {
            if i < L2_PAGE_CAPACITY {
                for (d, &coord) in page.key.iter().enumerate() {
                    if !(-32767..=32767).contains(&coord) {
                        return Err(invalid(format!(
                            "corrupted L2 page key coordinate at page {i}, dim {d}: {coord} outside [-32767, 32767]"
                        )));
                    }
                }
                l2_pages[i] = page;
            }
        }

        let is_lorentz = bundle.model().is_lorentz();
        let allow_hyperbolic_cache = s_state.allow_hyperbolic_cache || is_lorentz;

        let mut persistent_key_norms = Vec::with_capacity(persistent_keys.len());
        if is_lorentz {
            for key in &persistent_keys {
                persistent_key_norms.push(crate::lorentz::squared_norm(key)?);
            }
        } else {
            persistent_key_norms.resize(persistent_keys.len(), 0i128);
        }

        let mut dialogue_key_norms: Box<[i128; DIALOGUE_CAPACITY]> = vec![0i128; DIALOGUE_CAPACITY]
            .into_boxed_slice()
            .try_into()
            .map_err(|_| invalid("dialogue_key_norms size mismatch"))?;
        if is_lorentz {
            for i in 0..DIALOGUE_CAPACITY {
                dialogue_key_norms[i] = crate::lorentz::squared_norm(&dialogue_keys[i])?;
            }
        }

        let mut l2_page_norms: Box<[i128; L2_PAGE_CAPACITY]> = vec![0i128; L2_PAGE_CAPACITY]
            .into_boxed_slice()
            .try_into()
            .map_err(|_| invalid("l2_page_norms size mismatch"))?;
        if is_lorentz {
            for i in 0..L2_PAGE_CAPACITY {
                l2_page_norms[i] = crate::lorentz::squared_norm(&l2_pages[i].key)?;
            }
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
            l2_pages,
            l2_cursor: s_state.l2_cursor,
            l2_len: s_state.l2_len,
            l2_seen: s_state.l2_seen,
            last_compressed_turn_id: s_state.last_compressed_turn_id,
            zeta_state: s_state.zeta_state,
            hopf_state: s_state.hopf_state,
            cumulative_holonomy_q30: s_state.cumulative_holonomy_q30,
            age_horizon_clamp: s_state.age_horizon_clamp,
            persistent_key_norms,
            dialogue_key_norms,
            l2_page_norms,
            allow_hyperbolic_cache,
            scratch_products: vec![[0i64; 16]; 512],
            copy_scratch: vec![0u64; 4096],
            last_probabilities: vec![0u64; 4096]
                .into_boxed_slice()
                .try_into()
                .map_err(|_| invalid("probabilities size mismatch"))?,
            last_read_masses: Vec::with_capacity(TOTAL_MEMORY_CANDIDATES),
            last_no_read_mass: 0,
            last_copy_gate: 0,
            has_step: false,
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

    fn force_prediction(session: &mut ChatSession<'_>, token: u32) {
        session.state.last_probabilities.fill(0);
        session.state.last_probabilities[token as usize] = crate::PROBABILITY_TOTAL;
        session.state.has_step = true;
    }

    #[test]
    fn chat_stream_closure_errors_are_terminal() -> Result<()> {
        let bundle = crate::bundle::create_test_bundle_with_byte_vocab();
        let text_token = *bundle
            .tokenizer()
            .encode("x")
            .first()
            .ok_or_else(|| invalid("test tokenizer lacks x"))?;

        // A sampled EOS used to report success even when its model write failed.
        let mut session = ChatSession::new(&bundle, None, 17)?;
        let eos = session.roles.eos_id;
        force_prediction(&mut session, eos);
        session.state.identity = "injected identity mismatch".into();
        let mut stream = ChatTokenStream::new(&mut session, 8, &[]);
        assert_eq!(stream.decoder.push_bytes(&[0xc3]), None);
        assert_eq!(stream.next(), Some("\u{fffd}".into()));
        assert_eq!(stream.stop_reason(), Some(StreamStopReason::ModelError));
        assert!(stream
            .error()
            .is_some_and(|error| error.to_string().contains("identity")));
        assert!(stream.generated_tokens().is_empty());
        assert_eq!(stream.next(), None);

        // Cap closure fails before any generated token or sampling operation.
        let mut session = ChatSession::new(&bundle, None, 17)?;
        session.roles.turn_end_id = bundle.model().config().vocab_size as u32;
        let sampler_before = session.sampler.state();
        let mut stream = ChatTokenStream::new(&mut session, 0, &[]);
        assert_eq!(stream.next(), None);
        assert_eq!(stream.stop_reason(), Some(StreamStopReason::ModelError));
        assert!(stream
            .error()
            .is_some_and(|error| error.to_string().contains("bounds")));
        assert_eq!(stream.session.state.dialogue_seen, 0);
        assert_eq!(stream.session.sampler.state(), sampler_before);
        assert_eq!(stream.next(), None);

        // The third ordinary token is successfully written; only the inserted
        // turn-end write fails. Preserve those generated IDs and report failure.
        let mut session = ChatSession::new(&bundle, None, 17)?;
        session.roles.turn_end_id = bundle.model().config().vocab_size as u32;
        let mut stream = ChatTokenStream::new(&mut session, 8, &[]);
        for _ in 0..2 {
            force_prediction(stream.session, text_token);
            assert_eq!(stream.next(), Some("x".into()));
        }
        force_prediction(stream.session, text_token);
        assert_eq!(stream.next(), None);
        assert_eq!(stream.stop_reason(), Some(StreamStopReason::ModelError));
        assert!(stream
            .error()
            .is_some_and(|error| error.to_string().contains("bounds")));
        assert_eq!(stream.generated_tokens(), &[text_token; 3]);
        assert_eq!(stream.session.state.dialogue_seen, 3);
        assert_eq!(stream.next(), None);
        Ok(())
    }

    #[test]
    fn chat_stream_successful_closures_preserve_history_and_sampler() -> Result<()> {
        let bundle = crate::bundle::create_test_bundle_with_byte_vocab();
        let text_token = *bundle
            .tokenizer()
            .encode("x")
            .first()
            .ok_or_else(|| invalid("test tokenizer lacks x"))?;
        for case in 0..5 {
            let mut session = ChatSession::new(&bundle, None, 17)?;
            let mut expected = ChatSession::new(&bundle, None, 17)?;
            let (selected, count, cap, stops, reason, policy) = match case {
                0 => (
                    session.roles.eos_id,
                    1,
                    8,
                    vec![],
                    StreamStopReason::Eos {
                        token_id: session.roles.eos_id,
                    },
                    SamplePolicy::Categorical { top_k: 0 },
                ),
                1 => (
                    session.roles.turn_end_id,
                    1,
                    8,
                    vec![],
                    StreamStopReason::TurnEnd {
                        token_id: session.roles.turn_end_id,
                    },
                    SamplePolicy::Greedy,
                ),
                2 => (
                    text_token,
                    1,
                    8,
                    vec![text_token],
                    StreamStopReason::CustomStop {
                        token_id: text_token,
                    },
                    SamplePolicy::Categorical { top_k: 0 },
                ),
                3 => (
                    text_token,
                    1,
                    1,
                    vec![],
                    StreamStopReason::MaxTokens { count: 1 },
                    SamplePolicy::Categorical { top_k: 0 },
                ),
                _ => (
                    text_token,
                    3,
                    8,
                    vec![],
                    StreamStopReason::CycleDetected { period: 1 },
                    SamplePolicy::Categorical { top_k: 0 },
                ),
            };
            session.set_policy(policy);
            expected.set_policy(policy);
            let turn_end = session.roles.turn_end_id;
            let mut stream = ChatTokenStream::new(&mut session, cap, &stops).with_policy(policy);
            let mut visible = String::new();
            for _ in 0..count {
                force_prediction(stream.session, selected);
                force_prediction(&mut expected, selected);
                let sampled = expected
                    .sampler
                    .select(&expected.state.last_probabilities[..], policy)
                    .map_err(|error| invalid(error.to_string()))?;
                assert_eq!(sampled, selected as usize);
                expected.step_token(selected, SlotTarget::Dialogue)?;
                if let Some(chunk) = stream.next() {
                    visible.push_str(&chunk);
                }
            }
            if case >= 3 {
                expected.step_token(turn_end, SlotTarget::Dialogue)?;
            }
            // The cap closure occurs on the next iterator call after its text.
            assert_eq!(stream.next(), None);
            assert_eq!(stream.stop_reason(), Some(reason));
            assert!(stream.error().is_none());
            let generated = if case < 3 {
                vec![]
            } else {
                vec![text_token; count]
            };
            assert_eq!(stream.generated_tokens(), generated);
            assert_eq!(
                visible,
                match case {
                    3 => "x",
                    4 => "xx",
                    _ => "",
                }
            );
            drop(stream);
            assert_eq!(
                serde_json::to_value(session.to_serialized(None))?,
                serde_json::to_value(expected.to_serialized(None))?
            );
            assert_eq!(
                session.last_step().map(|step| &step.probabilities),
                expected.last_step().map(|step| &step.probabilities)
            );
        }
        Ok(())
    }

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
