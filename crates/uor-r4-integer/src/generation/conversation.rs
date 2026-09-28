//! Persistent literal-role conversations with exact generated-token history.
//!
//! Framing follows the shared training serializer. Generated IDs are retained
//! verbatim: they need not equal a re-encoding of normalized or decoded text.
//! Full context is bounded; this layer never evicts, truncates or replays it.

use super::{
    validate_append_tokens, validate_generation_budget, Decision, DialogueGeneration, Generation,
    IncrementalUtf8Decoder, Selection, Stop, StopTokens, TextSession,
};
use crate::{invalid, Bundle, IntegerError, ReadMode, SamplePolicy, Sampler};
use serde::Serialize;
use std::{fmt, time::Instant};
use uor_r4_tokenizer::dialogue::{DialogueError, DialogueProtocol, Message};

/// A new user turn may interrupt an unfinished assistant only explicitly.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnClosure {
    RequireModelEos,
    InterruptAssistant,
}

/// How the preceding turn was closed. CallerEos is input, never generated output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TurnBoundary {
    InitialHistory,
    ModelEos { token: u32 },
    CallerEos { token: u32, interrupted_stop: Stop },
}

#[derive(Clone, Copy, Debug)]
pub struct ConversationRequest<'a> {
    pub user: &'a str,
    pub max_new_tokens: usize,
    /// Only Greedy and Categorical are supported. Their existing top-k semantics
    /// are unchanged: zero or >= vocabulary size includes the whole vocabulary.
    pub policy: SamplePolicy,
    pub first_sentence: bool,
    pub closure: TurnClosure,
}

#[derive(Debug, Serialize)]
pub struct ConversationTurn {
    pub boundary: TurnBoundary,
    /// Newly supplied input IDs, excluding prior generated/pending IDs. The
    /// first turn includes BOS and initial history; later turns never add BOS.
    pub appended_token_ids: Vec<u32>,
    /// Exact bytes decoded from generated IDs, including any generated EOS.
    /// Unlike response_text, this preserves whitespace and incomplete UTF-8.
    pub raw_generated_bytes: Vec<u8>,
    pub dialogue: DialogueGeneration,
}

#[derive(Debug)]
pub enum ConversationError {
    Protocol(DialogueError),
    Invalid(IntegerError),
    EmptyUser,
    AssistantStillOpen {
        stop: Stop,
    },
    UnsupportedPolicy(SamplePolicy),
    Poisoned,
    /// Execution may have partially changed state. All later requests and
    /// sampler-state access are rejected; create a new conversation explicitly.
    Execution(IntegerError),
}
impl fmt::Display for ConversationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => write!(f, "{e}"),
            Self::Invalid(e) => write!(f, "{e}"),
            Self::EmptyUser => f.write_str("user message is empty after dialogue normalization"),
            Self::AssistantStillOpen { stop } => write!(
                f,
                "assistant stopped at {stop:?}; explicitly interrupt before a new user turn"
            ),
            Self::UnsupportedPolicy(policy) => write!(
                f,
                "conversation supports only Greedy/Categorical, not {policy:?}"
            ),
            Self::Poisoned => f.write_str(
                "conversation is poisoned by an execution failure; create a new conversation",
            ),
            Self::Execution(e) => write!(
                f,
                "conversation execution failed and poisoned the session: {e}"
            ),
        }
    }
}
impl std::error::Error for ConversationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Protocol(e) => Some(e),
            Self::Invalid(e) | Self::Execution(e) => Some(e),
            _ => None,
        }
    }
}
impl From<DialogueError> for ConversationError {
    fn from(error: DialogueError) -> Self {
        Self::Protocol(error)
    }
}
impl From<IntegerError> for ConversationError {
    fn from(error: IntegerError) -> Self {
        Self::Invalid(error)
    }
}
pub type Result<T> = std::result::Result<T, ConversationError>;

/// Protocol, initial persona/history and read mode are immutable. Initial
/// history follows V1's existing skip/normalization rules with no alternation
/// requirement: empty, system-only and user-ending history are supported. Each
/// respond call appends its own required nonempty user message.
///
/// Model state is created lazily, after the complete first request passes
/// preflight. Greedy turns preserve the categorical random cursor. Unsupported
/// selection/input errors preserve state; execution errors poison it instead.
pub struct DialogueConversation<'a> {
    bundle: &'a Bundle,
    protocol: DialogueProtocol,
    protocol_identity: String,
    initial_tokens: Vec<u32>,
    initial_has_history: bool,
    mode: ReadMode,
    session: Option<TextSession<'a>>,
    sampler_state: u64,
    previous_stop: Option<Stop>,
    poisoned: bool,
}

impl Bundle {
    pub fn dialogue_conversation(
        &self,
        protocol: &DialogueProtocol,
        initial_history: &[Message<'_>],
        seed: u64,
        mode: ReadMode,
    ) -> Result<DialogueConversation<'_>> {
        let encoder = protocol.bind(self.tokenizer())?;
        if protocol.bos_id != 0
            || protocol.eos_id != 1
            || self.tokenizer().vocab_size() != self.model().config().vocab_size
        {
            return Err(invalid(
                "conversation requires BOS0/EOS1 and equal model/tokenizer vocabularies",
            )
            .into());
        }
        let history = encoder.encode_open_history(initial_history);
        validate_append_tokens(self, 0, &history.tokens)?;
        Ok(DialogueConversation {
            bundle: self,
            protocol: protocol.clone(),
            protocol_identity: protocol.identity()?,
            initial_tokens: history.tokens,
            initial_has_history: history.emitted_turns != 0,
            mode,
            session: None,
            sampler_state: Sampler::new(seed).state(),
            previous_stop: None,
            poisoned: false,
        })
    }
}

impl<'a> DialogueConversation<'a> {
    pub fn protocol(&self) -> &DialogueProtocol {
        &self.protocol
    }
    pub fn protocol_identity(&self) -> &str {
        &self.protocol_identity
    }
    /// Actual observed/generated count, including the pending generated token.
    /// Zero before first respond; after poisoning this is diagnostic only.
    pub fn len(&self) -> usize {
        self.session.as_ref().map_or(0, TextSession::len)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Total successful observed model steps. After poisoning this is diagnostic only.
    pub fn step_calls(&self) -> usize {
        self.session.as_ref().map_or(0, |s| s.calls)
    }
    pub fn is_poisoned(&self) -> bool {
        self.poisoned
    }
    pub fn bundle(&self) -> &'a Bundle {
        self.bundle
    }
    pub fn read_mode(&self) -> ReadMode {
        self.mode
    }
    pub fn previous_stop(&self) -> Option<Stop> {
        self.previous_stop
    }
    pub fn initial_tokens(&self) -> &[u32] {
        &self.initial_tokens
    }
    pub fn initial_has_history(&self) -> bool {
        self.initial_has_history
    }
    pub fn session(&self) -> Option<&TextSession<'a>> {
        self.session.as_ref()
    }
    /// Cursor for the next categorical selection, including across greedy turns.
    pub fn sampler_state(&self) -> Result<u64> {
        if self.poisoned {
            Err(ConversationError::Poisoned)
        } else {
            Ok(self.sampler_state)
        }
    }

    pub fn respond_stream<'s>(
        &'s mut self,
        request: ConversationRequest<'_>,
    ) -> Result<DialogueConversationStream<'s, 'a>> {
        if self.poisoned {
            return Err(ConversationError::Poisoned);
        }
        let policy = match request.policy {
            SamplePolicy::Greedy => SamplePolicy::Greedy,
            SamplePolicy::Categorical { top_k } => SamplePolicy::Categorical { top_k },
            other => return Err(ConversationError::UnsupportedPolicy(other)),
        };
        let boundary = match self.previous_stop {
            None => TurnBoundary::InitialHistory,
            Some(Stop::Eos) => TurnBoundary::ModelEos {
                token: self.protocol.eos_id,
            },
            Some(stop) if request.closure == TurnClosure::InterruptAssistant => {
                TurnBoundary::CallerEos {
                    token: self.protocol.eos_id,
                    interrupted_stop: stop,
                }
            }
            Some(stop) => return Err(ConversationError::AssistantStillOpen { stop }),
        };
        let user = self
            .protocol
            .bind(self.bundle.tokenizer())?
            .encode_user_prefix(
                request.user,
                self.previous_stop.is_some() || self.initial_has_history,
            );
        if user.emitted_turns == 0 {
            return Err(ConversationError::EmptyUser);
        }
        let mut appended = if self.session.is_none() {
            self.initial_tokens.clone()
        } else {
            Vec::new()
        };
        if let TurnBoundary::CallerEos { token, .. } = boundary {
            appended.push(token);
        }
        appended.extend(user.tokens);
        // len includes pending. Validate every supplied ID, the complete user
        // suffix and any caller EOS, plus all requested output before mutation.
        validate_append_tokens(self.bundle, self.len(), &appended)?;
        let next_len = self
            .len()
            .checked_add(appended.len())
            .ok_or_else(|| invalid("conversation length overflow"))?;
        validate_generation_budget(
            next_len,
            request.max_new_tokens,
            self.bundle.model().config().context,
        )?;

        let clock = Instant::now();
        let (calls_before, ns_before) = self
            .session
            .as_ref()
            .map_or((0, 0), |s| (s.calls, s.model_ns));

        let starting_selection = match policy {
            SamplePolicy::Greedy => Selection::Greedy,
            SamplePolicy::Categorical { top_k } => Selection::Categorical {
                top_k,
                seed: self.sampler_state,
            },
            other => return Err(ConversationError::UnsupportedPolicy(other)),
        };

        let first = self.session.is_none();
        if first {
            let mut session = match self.bundle.text_session(self.mode) {
                Ok(s) => s,
                Err(error) => {
                    self.poisoned = true;
                    return Err(ConversationError::Execution(error));
                }
            };
            session.stop_tokens = StopTokens {
                eos: self.protocol.eos_id,
                turn_end: None,
            };
            session.dialogue_protocol = Some(self.protocol.clone());
            self.session = Some(session);
        }
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| invalid("conversation session missing"))?;
        let suffix = if first { &appended[1..] } else { &appended };
        if !suffix.is_empty() {
            if let Err(error) = session.append_tokens(suffix) {
                self.poisoned = true;
                return Err(ConversationError::Execution(error));
            }
        }
        let prompt_token_ids = session.observed.clone();
        let sampler = match policy {
            SamplePolicy::Greedy => Sampler::new(0),
            SamplePolicy::Categorical { .. } => Sampler::new(self.sampler_state),
            other => return Err(ConversationError::UnsupportedPolicy(other)),
        };

        Ok(DialogueConversationStream {
            conversation: self,
            boundary,
            appended,
            prompt_token_ids,
            max_new_tokens: request.max_new_tokens,
            policy,
            starting_selection,
            first_sentence: request.first_sentence,
            tokens_generated: 0,
            generated_tokens: Vec::new(),
            decisions: Vec::new(),
            sampler,
            decoder: IncrementalUtf8Decoder::new(),
            stopped: false,
            stop_reason: None,
            error: None,
            clock,
            calls_before,
            ns_before,
        })
    }

    pub fn respond(&mut self, request: ConversationRequest<'_>) -> Result<ConversationTurn> {
        let stream = self.respond_stream(request)?;
        stream.into_turn()
    }
}

/// Active streaming token iterator for a dialogue conversation.
pub struct DialogueConversationStream<'s, 'a> {
    conversation: &'s mut DialogueConversation<'a>,
    boundary: TurnBoundary,
    appended: Vec<u32>,
    prompt_token_ids: Vec<u32>,
    max_new_tokens: usize,
    policy: SamplePolicy,
    starting_selection: Selection,
    first_sentence: bool,
    tokens_generated: usize,
    generated_tokens: Vec<u32>,
    decisions: Vec<Decision>,
    sampler: Sampler,
    decoder: IncrementalUtf8Decoder,
    stopped: bool,
    stop_reason: Option<Stop>,
    error: Option<ConversationError>,
    clock: Instant,
    calls_before: usize,
    ns_before: u128,
}

impl<'s, 'a> DialogueConversationStream<'s, 'a> {
    pub fn tokens_generated(&self) -> usize {
        self.tokens_generated
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    pub fn stop_reason(&self) -> Option<Stop> {
        self.stop_reason
    }

    pub fn error(&self) -> Option<&ConversationError> {
        self.error.as_ref()
    }

    pub fn generated_tokens(&self) -> &[u32] {
        &self.generated_tokens
    }

    pub fn boundary(&self) -> TurnBoundary {
        self.boundary
    }

    fn finish_with_error(&mut self, error: ConversationError) {
        self.stopped = true;
        self.conversation.poisoned = true;
        self.error = Some(error);
    }

    fn finish_turn(&mut self, stop: Stop) {
        self.stopped = true;
        self.stop_reason = Some(stop);
        self.conversation.previous_stop = Some(stop);
        if matches!(self.policy, SamplePolicy::Categorical { .. }) {
            self.conversation.sampler_state = self.sampler.state();
        }
    }

    pub fn into_turn(mut self) -> Result<ConversationTurn> {
        if let Some(err) = self.error.take() {
            return Err(err);
        }
        if !self.stopped {
            while self.next().is_some() {}
            if let Some(err) = self.error.take() {
                return Err(err);
            }
        }
        let stop = self.stop_reason.unwrap_or(Stop::MaximumNewTokens);
        let end = self
            .generated_tokens
            .iter()
            .position(|&token| {
                self.conversation
                    .session
                    .as_ref()
                    .and_then(|s| s.stop_reason(token))
                    .is_some()
            })
            .unwrap_or(self.generated_tokens.len());
        let raw = self
            .conversation
            .bundle
            .tokenizer()
            .decode_bytes(&self.generated_tokens);
        let bytes = self
            .conversation
            .bundle
            .tokenizer()
            .decode_bytes(&self.generated_tokens[..end]);
        let (calls_after, ns_after) = self
            .conversation
            .session
            .as_ref()
            .map_or((0, 0), |s| (s.calls, s.model_ns));
        let incremental_step_calls = calls_after - self.calls_before;
        let model_step_nanoseconds = ns_after - self.ns_before;
        let whole_generation_nanoseconds = super::duration_nanos_exact(self.clock.elapsed());

        let selection = self.starting_selection;

        let generation = Generation {
            prompt: String::new(),
            prompt_token_ids: std::mem::take(&mut self.prompt_token_ids),
            generated_token_ids: std::mem::take(&mut self.generated_tokens),
            response_text: String::from_utf8_lossy(&bytes).trim().to_owned(),
            raw_decoded: String::from_utf8_lossy(&raw).into_owned(),
            utf8_decodable: std::str::from_utf8(&bytes).is_ok(),
            stop,
            decisions: std::mem::take(&mut self.decisions),
            selection,
            read_mode: self.conversation.mode,
            context_capacity: self.conversation.bundle.model().config().context,
            session_tokens_including_pending: self.conversation.len(),
            incremental_step_calls,
            model_step_nanoseconds,
            whole_generation_nanoseconds,
            bundle_sha256: self.conversation.bundle.identity().to_owned(),
            tokenizer_cid: self.conversation.bundle.tokenizer().address(),
            sampler_state_after: self.conversation.sampler_state,
        };

        Ok(ConversationTurn {
            boundary: self.boundary,
            appended_token_ids: std::mem::take(&mut self.appended),
            raw_generated_bytes: raw,
            dialogue: DialogueGeneration {
                protocol: self.conversation.protocol.clone(),
                protocol_identity: self.conversation.protocol_identity.clone(),
                generation,
            },
        })
    }
}

impl<'s, 'a> Iterator for DialogueConversationStream<'s, 'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stopped {
            return None;
        }

        while self.tokens_generated < self.max_new_tokens {
            let session = match self.conversation.session.as_mut() {
                Some(s) => s,
                None => {
                    self.finish_with_error(ConversationError::Execution(invalid(
                        "missing conversation session",
                    )));
                    return self.decoder.flush();
                }
            };

            let step_res = session.step_next_token(&mut self.sampler, self.policy);
            let (token, decision) = match step_res {
                Ok(pair) => pair,
                Err(err) => {
                    self.finish_with_error(ConversationError::Execution(err));
                    return self.decoder.flush();
                }
            };

            self.decisions.push(decision);
            self.generated_tokens.push(token);
            self.tokens_generated += 1;

            if let Some(reason) = session.stop_reason(token) {
                self.finish_turn(reason);
                return self.decoder.flush();
            }

            let token_bytes = self.conversation.bundle.tokenizer().decode_bytes(&[token]);
            let chunk = self.decoder.push_bytes(&token_bytes);

            if self.first_sentence
                && super::bytes_contain_byte(
                    &self
                        .conversation
                        .bundle
                        .tokenizer()
                        .decode_bytes(&self.generated_tokens),
                    b'.',
                )
            {
                self.finish_turn(Stop::FirstSentenceBoundary);
                let flushed = self.decoder.flush();
                return match (chunk, flushed) {
                    (Some(c), Some(f)) => Some(format!("{c}{f}")),
                    (Some(c), None) => Some(c),
                    (None, Some(f)) => Some(f),
                    (None, None) => None,
                };
            }

            if let Some(period) = super::short_cycle(&self.generated_tokens) {
                self.finish_turn(Stop::ShortCycle { period });
                let flushed = self.decoder.flush();
                return match (chunk, flushed) {
                    (Some(c), Some(f)) => Some(format!("{c}{f}")),
                    (Some(c), None) => Some(c),
                    (None, Some(f)) => Some(f),
                    (None, None) => None,
                };
            }

            if self.tokens_generated == self.max_new_tokens {
                self.finish_turn(Stop::MaximumNewTokens);
                let flushed = self.decoder.flush();
                return match (chunk, flushed) {
                    (Some(c), Some(f)) => Some(format!("{c}{f}")),
                    (Some(c), None) => Some(c),
                    (None, Some(f)) => Some(f),
                    (None, None) => None,
                };
            }

            if let Some(c) = chunk {
                return Some(c);
            }
        }

        self.finish_turn(Stop::MaximumNewTokens);
        self.decoder.flush()
    }
}

impl<'s, 'a> Drop for DialogueConversationStream<'s, 'a> {
    fn drop(&mut self) {
        if !self.stopped {
            self.finish_with_error(ConversationError::Execution(invalid(
                "dialogue conversation stream dropped before terminal stop token",
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntegerModel, PROBABILITY_TOTAL};
    use serde_json::json;
    use uor_r4_tokenizer::ByteBpeTokenizer;

    fn request(user: &str, policy: SamplePolicy) -> ConversationRequest<'_> {
        ConversationRequest {
            user,
            max_new_tokens: 2,
            policy,
            first_sentence: false,
            closure: TurnClosure::InterruptAssistant,
        }
    }
    fn byte_ids(text: &str) -> Vec<u32> {
        text.bytes().map(|b| 7 + u32::from(b)).collect()
    }
    fn diagnostic_snapshot(conversation: &DialogueConversation<'_>) -> serde_json::Value {
        json!({"len":conversation.len(),"calls":conversation.step_calls(),
            "cursor":conversation.sampler_state,"stop":conversation.previous_stop,
            "observed":conversation.session.as_ref().map(|s|&s.observed),
            "pending":conversation.session.as_ref().and_then(|s|s.pending),
            "probabilities":conversation.session.as_ref().and_then(|s|s.current.as_ref()).map(|p|&p.probabilities)})
    }

    #[test]
    fn conversation_preflight_keeps_lazy_and_pending_states_unchanged() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let mut conversation = bundle
            .dialogue_conversation(&protocol, &[], 73, ReadMode::Enabled)
            .unwrap();
        for after_generation in [false, true] {
            if after_generation {
                conversation
                    .respond(request("Hi", SamplePolicy::Greedy))
                    .unwrap();
            }
            let before = diagnostic_snapshot(&conversation);
            let mut oversized = request("Hi", SamplePolicy::Greedy);
            oversized.max_new_tokens = 256;
            let mut zero = request("Hi", SamplePolicy::Greedy);
            zero.max_new_tokens = 0;
            for invalid in [
                request(" \r\n\t", SamplePolicy::Greedy),
                oversized,
                zero,
                request(
                    "Hi",
                    SamplePolicy::MinP {
                        top_k: 40,
                        min_p_q16: 1,
                    },
                ),
            ] {
                assert!(conversation.respond(invalid).is_err());
                assert_eq!(diagnostic_snapshot(&conversation), before);
                assert!(!conversation.is_poisoned());
            }
        }
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(json!({
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2},"merges":[]},
            "added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]
        }).to_string().as_bytes()).unwrap();
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).unwrap();
        let mismatch = Bundle::from_parts(
            IntegerModel::synthetic_for_test(),
            tokenizer,
            "fixture".to_owned(),
        );
        assert!(matches!(
            mismatch.dialogue_conversation(&protocol, &[], 73, ReadMode::Enabled),
            Err(ConversationError::Invalid(_))
        ));
    }

    #[test]
    fn conversation_preserves_pending_eos_or_explicitly_closes_exact_bytes() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        for (token, stop) in [
            (1, Stop::Eos),
            (7 + 195, Stop::MaximumNewTokens),
            (7 + 195, Stop::ShortCycle { period: 1 }),
            (7 + 195, Stop::FirstSentenceBoundary),
        ] {
            let mut conversation = bundle
                .dialogue_conversation(&protocol, &[], 73, ReadMode::Enabled)
                .unwrap();
            // Fixture only: create a valid next prediction with a controlled
            // byte-fragment/EOS selection. This is not learned output evidence.
            let mut session = bundle
                .dialogue_session(
                    &protocol,
                    &[Message {
                        role: "user",
                        content: "Hi",
                    }],
                    ReadMode::Enabled,
                )
                .unwrap();
            let probabilities = &mut session.current.as_mut().unwrap().probabilities;
            probabilities.fill(0);
            probabilities[token] = PROBABILITY_TOTAL;
            let emitted = session.generate(1, Selection::Greedy, false).unwrap();
            conversation.session = Some(session);
            // Same open/closed state for all non-EOS stopping reasons. The
            // forced byte is intentionally invalid UTF-8 without a continuation.
            conversation.previous_stop = Some(stop);
            let mut expected = emitted.prompt_token_ids.clone();
            expected.extend(&emitted.generated_token_ids);
            let before = diagnostic_snapshot(&conversation);
            let mut next = request("Go<|eos|>", SamplePolicy::Greedy);
            next.max_new_tokens = 1;
            next.closure = TurnClosure::RequireModelEos;
            if stop != Stop::Eos {
                assert!(matches!(
                    conversation.respond(next),
                    Err(ConversationError::AssistantStillOpen { .. })
                ));
                assert_eq!(diagnostic_snapshot(&conversation), before);
                next.closure = TurnClosure::InterruptAssistant;
                expected.push(1);
                assert!(!emitted.utf8_decodable);
            }
            let mut suffix = byte_ids("\nUser: Go");
            suffix.push(1);
            suffix.extend(byte_ids("\nAssistant: "));
            expected.extend(&suffix);
            let calls_before = conversation.step_calls();
            let turn = conversation.respond(next).unwrap();
            assert_eq!(turn.dialogue.generation.prompt_token_ids, expected);
            assert_eq!(
                conversation.step_calls() - calls_before,
                1 + turn.appended_token_ids.len()
            );
            if stop == Stop::Eos {
                assert_eq!(turn.boundary, TurnBoundary::ModelEos { token: 1 });
                assert_eq!(turn.appended_token_ids, suffix);
            } else {
                assert_eq!(
                    turn.boundary,
                    TurnBoundary::CallerEos {
                        token: 1,
                        interrupted_stop: stop
                    }
                );
                assert_eq!(turn.appended_token_ids[0], 1);
                assert_eq!(turn.appended_token_ids[1..], suffix);
            }
            assert_eq!(
                turn.raw_generated_bytes,
                bundle
                    .tokenizer()
                    .decode_bytes(&turn.dialogue.generation.generated_token_ids)
            );
        }
    }

    #[test]
    fn conversation_categorical_cursor_survives_greedy_turn_without_replay() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let seed = 20260926;
        let mut conversation = bundle
            .dialogue_conversation(&protocol, &[], seed, ReadMode::Enabled)
            .unwrap();
        let mut reference = bundle.text_session(ReadMode::Enabled).unwrap();
        reference.stop_tokens.turn_end = None;
        let mut cursor = Sampler::new(seed).state();
        let mut previous_stop = None;
        for (index, (user, policy)) in [
            ("Hi", SamplePolicy::Categorical { top_k: 40 }),
            ("Go", SamplePolicy::Greedy),
            ("Again", SamplePolicy::Categorical { top_k: 40 }),
        ]
        .into_iter()
        .enumerate()
        {
            let mut suffix = Vec::new();
            if index != 0 {
                if previous_stop != Some(Stop::Eos) {
                    suffix.push(1);
                }
                suffix.extend(byte_ids("\n"));
            }
            suffix.extend(byte_ids("User: "));
            suffix.extend(byte_ids(user));
            suffix.extend(byte_ids("\nAssistant: "));
            reference.append_tokens(&suffix).unwrap();
            let selection = match policy {
                SamplePolicy::Greedy => Selection::Greedy,
                SamplePolicy::Categorical { top_k } => Selection::Categorical {
                    top_k,
                    seed: cursor,
                },
                _ => unreachable!(),
            };
            let expected = reference.generate(2, selection, false).unwrap();
            let turn = conversation.respond(request(user, policy)).unwrap();
            let actual = turn.dialogue.generation;
            assert_eq!(actual.selection, expected.selection);
            assert_eq!(actual.prompt_token_ids, expected.prompt_token_ids);
            assert_eq!(actual.generated_token_ids, expected.generated_token_ids);
            assert_eq!(
                serde_json::to_value(actual.decisions).unwrap(),
                serde_json::to_value(expected.decisions).unwrap()
            );
            assert_eq!(conversation.step_calls(), reference.calls);
            assert_eq!(conversation.len(), reference.len());
            if matches!(policy, SamplePolicy::Categorical { .. }) {
                cursor = expected.sampler_state_after;
            }
            assert_eq!(conversation.sampler_state().unwrap(), cursor);
            assert_eq!(actual.sampler_state_after, cursor);
            previous_stop = Some(expected.stop);
        }
        assert_ne!(cursor, Sampler::new(seed).state());
    }

    #[test]
    fn conversation_execution_error_poison_prevents_retry_or_cursor_reuse() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let mut conversation = bundle
            .dialogue_conversation(&protocol, &[], 73, ReadMode::Enabled)
            .unwrap();
        conversation
            .respond(request("Hi", SamplePolicy::Greedy))
            .unwrap();
        // Inject an impossible pending ID to emulate an unexpected model-step
        // failure after request preflight, without changing the production model.
        conversation.session.as_mut().unwrap().pending = Some(4096);
        assert!(matches!(
            conversation.respond(request("Go", SamplePolicy::Greedy)),
            Err(ConversationError::Execution(_))
        ));
        assert!(conversation.is_poisoned());
        let calls = conversation.step_calls();
        assert!(matches!(
            conversation.respond(request("Go", SamplePolicy::Greedy)),
            Err(ConversationError::Poisoned)
        ));
        assert!(matches!(
            conversation.sampler_state(),
            Err(ConversationError::Poisoned)
        ));
        assert_eq!(conversation.step_calls(), calls);
    }

    #[test]
    fn conversation_streaming_produces_identical_turn_to_batch() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let seed = 101;
        let mut conv_batch = bundle
            .dialogue_conversation(&protocol, &[], seed, ReadMode::Enabled)
            .unwrap();
        let mut conv_stream = bundle
            .dialogue_conversation(&protocol, &[], seed, ReadMode::Enabled)
            .unwrap();

        for (user, policy) in [
            ("Hello world", SamplePolicy::Categorical { top_k: 32 }),
            ("Continue here", SamplePolicy::Greedy),
            ("Final message", SamplePolicy::Categorical { top_k: 16 }),
        ] {
            let req_batch = ConversationRequest {
                user,
                max_new_tokens: 3,
                policy,
                first_sentence: false,
                closure: TurnClosure::InterruptAssistant,
            };
            let req_stream = ConversationRequest {
                user,
                max_new_tokens: 3,
                policy,
                first_sentence: false,
                closure: TurnClosure::InterruptAssistant,
            };

            let turn_batch = conv_batch.respond(req_batch).unwrap();
            let mut stream = conv_stream.respond_stream(req_stream).unwrap();
            let mut streamed_text = String::new();
            while let Some(chunk) = stream.next() {
                assert!(std::str::from_utf8(chunk.as_bytes()).is_ok());
                streamed_text.push_str(&chunk);
            }
            let turn_stream = stream.into_turn().unwrap();

            assert_eq!(turn_stream.boundary, turn_batch.boundary);
            assert_eq!(
                turn_stream.appended_token_ids,
                turn_batch.appended_token_ids
            );
            assert_eq!(
                turn_stream.dialogue.generation.prompt_token_ids,
                turn_batch.dialogue.generation.prompt_token_ids
            );
            assert_eq!(
                turn_stream.dialogue.generation.generated_token_ids,
                turn_batch.dialogue.generation.generated_token_ids
            );
            assert_eq!(
                turn_stream.dialogue.generation.response_text,
                turn_batch.dialogue.generation.response_text
            );
            assert_eq!(streamed_text, turn_batch.dialogue.generation.response_text);
            assert_eq!(
                turn_stream.dialogue.generation.stop,
                turn_batch.dialogue.generation.stop
            );
            assert_eq!(
                turn_stream.dialogue.generation.sampler_state_after,
                turn_batch.dialogue.generation.sampler_state_after
            );
            assert_eq!(
                conv_stream.sampler_state().unwrap(),
                conv_batch.sampler_state().unwrap()
            );
            assert_eq!(conv_stream.len(), conv_batch.len());
            assert_eq!(conv_stream.step_calls(), conv_batch.step_calls());
        }
    }

    #[test]
    fn conversation_stream_execution_error_poisons_session() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let mut conversation = bundle
            .dialogue_conversation(&protocol, &[], 42, ReadMode::Enabled)
            .unwrap();
        let mut stream = conversation
            .respond_stream(request("Hi", SamplePolicy::Greedy))
            .unwrap();
        let _ = stream.next();
        stream.conversation.session.as_mut().unwrap().pending = Some(4096);
        let _ = stream.next();
        assert!(stream.error().is_some());
        assert!(stream.conversation.is_poisoned());
        assert!(matches!(
            stream.into_turn(),
            Err(ConversationError::Execution(_))
        ));
        assert!(matches!(
            conversation.sampler_state(),
            Err(ConversationError::Poisoned)
        ));
    }

    #[test]
    fn conversation_stream_early_drop_poisons_session() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let mut conversation = bundle
            .dialogue_conversation(&protocol, &[], 42, ReadMode::Enabled)
            .unwrap();
        {
            let req = ConversationRequest {
                user: "Hello",
                max_new_tokens: 16,
                policy: SamplePolicy::Greedy,
                first_sentence: false,
                closure: TurnClosure::InterruptAssistant,
            };
            let mut stream = conversation.respond_stream(req).unwrap();
            let chunk = stream.next();
            assert!(chunk.is_some());
            assert!(!stream.is_stopped());
            // stream is dropped here before terminal stop token
        }
        assert!(conversation.is_poisoned());
        let next_req = ConversationRequest {
            user: "Second message",
            max_new_tokens: 4,
            policy: SamplePolicy::Greedy,
            first_sentence: false,
            closure: TurnClosure::InterruptAssistant,
        };
        assert!(matches!(
            conversation.respond(next_req),
            Err(ConversationError::Poisoned)
        ));
    }
}
