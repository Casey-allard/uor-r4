//! Persistent literal-role conversations with exact generated-token history.
//!
//! Framing follows the shared training serializer. Generated IDs are retained
//! verbatim: they need not equal a re-encoding of normalized or decoded text.
//! Full context is bounded; this layer never evicts, truncates or replays it.

use super::{
    validate_append_tokens, validate_generation_budget, DialogueGeneration, Selection, Stop,
    StopTokens, TextSession,
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

impl DialogueConversation<'_> {
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
    /// Cursor for the next categorical selection, including across greedy turns.
    pub fn sampler_state(&self) -> Result<u64> {
        if self.poisoned {
            Err(ConversationError::Poisoned)
        } else {
            Ok(self.sampler_state)
        }
    }

    pub fn respond(&mut self, request: ConversationRequest<'_>) -> Result<ConversationTurn> {
        if self.poisoned {
            return Err(ConversationError::Poisoned);
        }
        let selection = match request.policy {
            SamplePolicy::Greedy => Selection::Greedy,
            SamplePolicy::Categorical { top_k } => Selection::Categorical {
                top_k,
                seed: self.sampler_state,
            },
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
        let result = self.execute(
            &appended,
            request.max_new_tokens,
            selection,
            request.first_sentence,
        );
        let mut generation = match result {
            Ok(value) => value,
            Err(error) => {
                self.poisoned = true;
                return Err(ConversationError::Execution(error));
            }
        };
        if let Some(session) = &self.session {
            generation.incremental_step_calls = session.calls - calls_before;
            generation.model_step_nanoseconds = session.model_ns - ns_before;
        }
        generation.whole_generation_nanoseconds = clock.elapsed().as_nanos();
        if matches!(request.policy, SamplePolicy::Categorical { .. }) {
            self.sampler_state = generation.sampler_state_after;
        }
        // The wrapper's receipt exposes its authoritative continuation cursor,
        // including on greedy turns which do not advance that cursor.
        generation.sampler_state_after = self.sampler_state;
        self.previous_stop = Some(generation.stop);
        Ok(ConversationTurn {
            boundary,
            appended_token_ids: appended,
            raw_generated_bytes: self
                .bundle
                .tokenizer()
                .decode_bytes(&generation.generated_token_ids),
            dialogue: DialogueGeneration {
                protocol: self.protocol.clone(),
                protocol_identity: self.protocol_identity.clone(),
                generation,
            },
        })
    }

    fn execute(
        &mut self,
        appended: &[u32],
        max_new_tokens: usize,
        selection: Selection,
        first_sentence: bool,
    ) -> crate::Result<super::Generation> {
        let first = self.session.is_none();
        if first {
            let mut session = self.bundle.text_session(self.mode)?;
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
        // text_session consumed exactly the first BOS. No later literal BOS is
        // stripped, and append_tokens consumes the previous pending ID once.
        let suffix = if first { &appended[1..] } else { appended };
        if !suffix.is_empty() {
            session.append_tokens(suffix)?;
        }
        session.generate(max_new_tokens, selection, first_sentence)
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
}
