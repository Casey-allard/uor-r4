//! Capability API for native UOR-R4 integer serving.
//!
//! A lifetime-free session container over one loaded [`Bundle`] for host
//! applications: bundle metadata, a scoped capability status matrix, session
//! creation with an optional persistent system prompt, user-turn ingestion,
//! token generation and session save/restore. Safe Rust under the crate's
//! `#![forbid(unsafe_code)]`.
//!
//! The metadata declares the numerical serving contract; it does not certify
//! it. Qualification belongs to the instruction audit of the delivered release
//! binary and to measured, sealed reports, never to constants in this module.

use crate::bundle::Bundle;
use crate::config::ReadMode;
use crate::model::{SessionState, SlotTarget, PERSISTENT_CAPACITY};
use crate::sampling::{SamplePolicy, Sampler};
use crate::session::{ChatSession, RoleTokens, SerializedChatSession};
use crate::{invalid, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

pub const CAPABILITY_API_SCHEMA: &str = "uor-r4.integer-capability-api/1";

/// Numerical serving contract the integer runtime is written to (owner decision D11).
pub const DECLARED_NUMERICAL_CONTRACT: &str = "D11";

/// What a declaration of [`DECLARED_NUMERICAL_CONTRACT`] does and does not establish.
pub const CONTRACT_QUALIFICATION_NOTE: &str = "Declared, not certified by this API. \
Qualification requires the ARM64 instruction audit of the delivered release binary \
(scripts/audit_zero_matmul_serving.py) and a measured, sealed report.";

/// Capability status statements, each scoped to what is implemented or
/// established; unestablished capabilities are marked UNQUALIFIED.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityTruthMatrix {
    pub language_prose: String,
    pub causal_attention: String,
    pub multi_step_reasoning: String,
    pub executable_coding: String,
    pub durable_memory: String,
    pub serving_guarantees: String,
    pub m1_performance_profile: String,
    pub general_ai_disavowal: String,
}

impl Default for CapabilityTruthMatrix {
    fn default() -> Self {
        Self {
            language_prose: "UNQUALIFIED: general prose is not established. Replay parity with a recorded dialogue panel is a runtime check, not a language-quality result.".into(),
            causal_attention: "Implemented: causal integer-softmax reads over bounded addressed memory (persistent slots, dialogue ring, L2 pages). A structural description, not a measured capability.".into(),
            multi_step_reasoning: "UNQUALIFIED: general reasoning is not established.".into(),
            executable_coding: "UNQUALIFIED: general coding is not established.".into(),
            durable_memory: "Implemented: bounded session memory with save and restore through uor-r4.integer-session/1. That schema omits the width-576 value stores, so a restored width-576 session does not continue identically. Durable-memory capability is not established.".into(),
            serving_guarantees: format!("Declared contract {DECLARED_NUMERICAL_CONTRACT}: no floating point and no multiply or divide instruction in served kernels. {CONTRACT_QUALIFICATION_NOTE}"),
            m1_performance_profile: "UNQUALIFIED: no sealed M1 cost report is bound to this API.".into(),
            general_ai_disavowal: "Pre-alpha: general prose, general reasoning and frontier capability remain unproven.".into(),
        }
    }
}

/// Metadata describing the loaded native integer model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilityMetadata {
    pub api_schema_version: String,
    pub bundle_identity: String,
    pub width: usize,
    pub vocab_size: usize,
    pub context_capacity: usize,
    pub read_geometry: String,
    /// The contract the runtime is written to; see `contract_qualification`.
    pub declared_numerical_contract: String,
    /// What the declaration does not establish on its own.
    pub contract_qualification: String,
    pub truth_matrix: CapabilityTruthMatrix,
}

/// Configuration for creating a managed integer chat session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    pub system_prompt: Option<String>,
    pub seed: u64,
    pub read_mode: ReadMode,
    pub policy: SamplePolicy,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            system_prompt: None,
            seed: 0,
            read_mode: ReadMode::Enabled,
            policy: SamplePolicy::Greedy,
        }
    }
}

/// Session telemetry snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTelemetry {
    pub session_id: u32,
    pub active_tokens: usize,
    pub context_capacity: usize,
    pub read_mode: ReadMode,
    pub policy: SamplePolicy,
    pub prng_state: u64,
}

/// Owned conversational session state managed by the API.
struct ManagedSession {
    state: SessionState,
    roles: RoleTokens,
    sampler: Sampler,
    read_mode: ReadMode,
    policy: SamplePolicy,
}

/// Capability API container managing bundle lifetime and session state.
pub struct IntegerCapabilityApi {
    bundle: Bundle,
    sessions: Mutex<HashMap<u32, ManagedSession>>,
    next_session_id: AtomicU32,
}

impl IntegerCapabilityApi {
    /// Create a new Capability API instance over a loaded bundle.
    pub fn new(bundle: Bundle) -> Self {
        Self {
            bundle,
            sessions: Mutex::new(HashMap::new()),
            next_session_id: AtomicU32::new(1),
        }
    }

    /// Bundle identity, configuration and the declared (uncertified) contract.
    pub fn metadata(&self) -> ModelCapabilityMetadata {
        let config = self.bundle.model().config();
        ModelCapabilityMetadata {
            api_schema_version: CAPABILITY_API_SCHEMA.into(),
            bundle_identity: self.bundle.identity().into(),
            width: config.width,
            vocab_size: config.vocab_size,
            context_capacity: config.context,
            read_geometry: format!("{:?}", config.read_geometry),
            declared_numerical_contract: DECLARED_NUMERICAL_CONTRACT.into(),
            contract_qualification: CONTRACT_QUALIFICATION_NOTE.into(),
            truth_matrix: CapabilityTruthMatrix::default(),
        }
    }

    /// Instantiate a new managed conversational session with configuration.
    pub fn create_session(&self, config: SessionConfig) -> Result<u32> {
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        let roles = RoleTokens::from_tokenizer(self.bundle.tokenizer())?;
        let mut state = self.bundle.model().new_conversational_session();
        if self.bundle.model().is_lorentz() {
            state.enable_hyperbolic_cache();
        }

        let mut managed = ManagedSession {
            state,
            roles,
            sampler: Sampler::new(config.seed),
            read_mode: config.read_mode,
            policy: config.policy,
        };

        if let Some(prompt) = config.system_prompt {
            if !prompt.trim().is_empty() {
                let text_tokens = self.bundle.tokenizer().encode(&prompt);
                let mut tokens = Vec::with_capacity(text_tokens.len() + 2);
                tokens.push(managed.roles.system_id);
                tokens.extend(text_tokens);
                tokens.push(managed.roles.turn_end_id);

                if tokens.len() > PERSISTENT_CAPACITY {
                    return Err(invalid(format!(
                        "system prompt length {} exceeds persistent slot capacity {}",
                        tokens.len(),
                        PERSISTENT_CAPACITY
                    )));
                }
                for &token in &tokens {
                    self.bundle.model().step_conversational_into(
                        &mut managed.state,
                        token,
                        SlotTarget::Persistent,
                        ReadMode::Enabled,
                    )?;
                }
                managed.state.seal_persistent();
            }
        }

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        lock.insert(session_id, managed);
        Ok(session_id)
    }

    /// Ingest a user prompt into the specified session. An empty prompt is
    /// rejected before any session state changes.
    pub fn ingest_user_turn(&self, session_id: u32, prompt: &str) -> Result<usize> {
        if prompt.trim().is_empty() {
            return Err(invalid("empty user turn prompt"));
        }
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get_mut(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        let user_tokens = self.bundle.tokenizer().encode(prompt);
        let mut tokens = Vec::with_capacity(user_tokens.len() + 3);
        tokens.push(session.roles.user_id);
        tokens.extend(user_tokens);
        tokens.push(session.roles.turn_end_id);
        tokens.push(session.roles.assistant_id);

        session.state.start_turn();
        for &token in &tokens {
            self.bundle.model().step_conversational_into(
                &mut session.state,
                token,
                SlotTarget::Dialogue,
                session.read_mode,
            )?;
        }
        Ok(tokens.len())
    }

    /// Execute a single step with a token ID and return the next selected token.
    pub fn step_token(&self, session_id: u32, token_id: u32) -> Result<u32> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get_mut(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        self.bundle.model().step_conversational_into(
            &mut session.state,
            token_id,
            SlotTarget::Dialogue,
            session.read_mode,
        )?;

        let vocab_size = self.bundle.model().config().vocab_size;
        let probs = &session.state.last_probabilities[..vocab_size];
        let selected = session
            .sampler
            .select(probs, session.policy)
            .map_err(|e| invalid(format!("sampling error: {e}")))?;
        Ok(selected as u32)
    }

    /// Generate an assistant response turn up to `max_tokens`. A selected
    /// turn-end or EOS token is committed to the session before returning; a
    /// failed commit is returned as an error.
    pub fn generate_text(&self, session_id: u32, max_tokens: usize) -> Result<String> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get_mut(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        let vocab_size = self.bundle.model().config().vocab_size;
        let turn_end_id = session.roles.turn_end_id;
        let eos_id = session.roles.eos_id;

        let mut generated_tokens = Vec::with_capacity(max_tokens);
        for _ in 0..max_tokens {
            let probs = &session.state.last_probabilities[..vocab_size];
            let next_tok = session
                .sampler
                .select(probs, session.policy)
                .map_err(|e| invalid(format!("sampling error: {e}")))?
                as u32;

            self.bundle.model().step_conversational_into(
                &mut session.state,
                next_tok,
                SlotTarget::Dialogue,
                session.read_mode,
            )?;
            if next_tok == turn_end_id || next_tok == eos_id {
                break;
            }
            generated_tokens.push(next_tok);
        }

        Ok(self.bundle.tokenizer().decode(&generated_tokens))
    }

    /// Save session state, sampler state, sampling policy and read mode.
    pub fn save_session(&self, session_id: u32) -> Result<SerializedChatSession> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;

        let temp_session = ChatSession::from_parts(
            &self.bundle,
            session.state.clone(),
            session.roles,
            session.sampler.clone(),
            session.read_mode,
            session.policy,
        );
        Ok(temp_session.to_serialized(Some(self.bundle.identity())))
    }

    /// Restore a saved session as a new session, keeping its saved sampler
    /// state, sampling policy and read mode. `uor-r4.integer-session/1` does
    /// not carry the width-576 value stores, so a restored width-576 session
    /// reads empty values and does not continue identically.
    pub fn restore_session(&self, serialized: SerializedChatSession) -> Result<u32> {
        let restored =
            ChatSession::from_serialized(&self.bundle, serialized, self.bundle.identity())?;
        let roles = restored.roles();
        let sampler = restored.sampler().clone();
        let read_mode = restored.read_mode();
        let policy = restored.policy();
        let managed = ManagedSession {
            state: restored.into_state(),
            roles,
            sampler,
            read_mode,
            policy,
        };

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session_id = self.next_session_id.fetch_add(1, Ordering::SeqCst);
        lock.insert(session_id, managed);
        Ok(session_id)
    }

    /// Inspect session telemetry.
    pub fn get_telemetry(&self, session_id: u32) -> Result<SessionTelemetry> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| invalid("session lock poisoned"))?;
        let session = lock
            .get(&session_id)
            .ok_or_else(|| invalid(format!("session {session_id} not found")))?;
        Ok(SessionTelemetry {
            session_id,
            active_tokens: session.state.dialogue_len(),
            context_capacity: self.bundle.model().config().context,
            read_mode: session.read_mode,
            policy: session.policy,
            prng_state: session.sampler.state(),
        })
    }

    /// Free a session handle and release memory.
    pub fn close_session(&self, session_id: u32) -> bool {
        if let Ok(mut lock) = self.sessions.lock() {
            lock.remove(&session_id).is_some()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::create_test_bundle_with_byte_vocab;

    #[test]
    fn generate_text_returns_a_failed_turn_end_commit() -> Result<()> {
        let api = IntegerCapabilityApi::new(create_test_bundle_with_byte_vocab());
        let id = api.create_session(SessionConfig::default())?;
        api.ingest_user_turn(id, "hello")?;
        {
            let mut lock = api
                .sessions
                .lock()
                .map_err(|_| invalid("session lock poisoned"))?;
            let session = lock
                .get_mut(&id)
                .ok_or_else(|| invalid("session missing"))?;
            // Force EOS as the only candidate and make its commit step fail.
            let eos = session.roles.eos_id as usize;
            session.state.last_probabilities.fill(0);
            session.state.last_probabilities[eos] = crate::PROBABILITY_TOTAL;
            session.state.identity = "injected identity mismatch".into();
        }
        let error = match api.generate_text(id, 4) {
            Ok(text) => return Err(invalid(format!("expected an error, got {text:?}"))),
            Err(error) => error,
        };
        assert!(error.to_string().contains("identity"), "{error}");
        Ok(())
    }
}
