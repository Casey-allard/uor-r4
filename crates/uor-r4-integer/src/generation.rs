//! Stateful text sessions using integer predictions and integer token selection.
//! Supports `generate_stream` and incremental UTF-8 decoding with `pending_bytes`.
pub use crate::session::{ChatTokenStream, IncrementalUtf8Decoder};
use crate::{
    bundle::Bundle, invalid, IntegerError, IntegerSession, IntegerStep, ReadMode, Result,
    SamplePolicy, Sampler, PROBABILITY_TOTAL,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt, time::Instant};
use uor_r4_tokenizer::dialogue::{DialogueError, DialogueProtocol, Message};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    #[default]
    Greedy,
    Categorical {
        top_k: usize,
        seed: u64,
    },
}
fn read_enabled() -> ReadMode {
    ReadMode::Enabled
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub prompt: String,
    pub max_new_tokens: usize,
    #[serde(default)]
    pub selection: Selection,
    #[serde(default = "read_enabled")]
    pub read_mode: ReadMode,
    #[serde(default)]
    pub first_sentence: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Stop {
    Eos,
    TurnEnd,
    StopToken { token: u32 },
    FirstSentenceBoundary,
    ShortCycle { period: usize },
    MaximumNewTokens,
}
#[derive(Debug, Serialize)]
pub struct Decision {
    pub selected_token: u32,
    pub probability_q48: u64,
    pub probability_sum_q48: u64,
    pub probability_sha256_le_u64: String,
    pub exposed_causal_slots: usize,
    pub no_read_mass_q48: u64,
}
#[derive(Debug, Serialize)]
pub struct Generation {
    pub prompt: String,
    pub prompt_token_ids: Vec<u32>,
    pub generated_token_ids: Vec<u32>,
    pub response_text: String,
    pub raw_decoded: String,
    pub utf8_decodable: bool,
    pub stop: Stop,
    pub decisions: Vec<Decision>,
    pub selection: Selection,
    pub read_mode: ReadMode,
    pub context_capacity: usize,
    pub session_tokens_including_pending: usize,
    pub incremental_step_calls: usize,
    pub model_step_nanoseconds: u128,
    pub whole_generation_nanoseconds: u128,
    pub bundle_sha256: String,
    pub tokenizer_cid: String,
    /// Feed this value as the next per-call seed to continue a categorical stream.
    pub sampler_state_after: u64,
}

/// An explicit evaluation binding, not evidence that a bundle was chat-trained.
/// The exact segmented prompt is recorded in `generation.prompt_token_ids`;
/// `generation.prompt` is empty because decoding/re-encoding changes some BPE IDs.
#[derive(Debug, Serialize)]
pub struct DialogueGeneration {
    pub protocol: DialogueProtocol,
    pub protocol_identity: String,
    pub generation: Generation,
}

#[derive(Debug)]
pub enum DialogueSessionError {
    Protocol(DialogueError),
    Integer(IntegerError),
}
impl fmt::Display for DialogueSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => write!(f, "{error}"),
            Self::Integer(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for DialogueSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Protocol(error) => error,
            Self::Integer(error) => error,
        })
    }
}
impl From<DialogueError> for DialogueSessionError {
    fn from(error: DialogueError) -> Self {
        Self::Protocol(error)
    }
}
impl From<IntegerError> for DialogueSessionError {
    fn from(error: IntegerError) -> Self {
        Self::Integer(error)
    }
}

#[derive(Clone, Copy)]
struct StopTokens {
    eos: u32,
    turn_end: Option<u32>,
}
impl StopTokens {
    fn reason(self, token: u32) -> Option<Stop> {
        if token == self.eos {
            Some(Stop::Eos)
        } else if self.turn_end == Some(token) {
            Some(Stop::TurnEnd)
        } else {
            None
        }
    }
}

/// Retains exact occurrence history across calls. The last emitted token is
/// pending until another prediction/input is requested, matching autoregression.
/// Capacity exhaustion is an error: no silent eviction, truncation or reset.
pub struct TextSession<'a> {
    bundle: &'a Bundle,
    state: IntegerSession,
    current: Option<IntegerStep>,
    pending: Option<u32>,
    mode: ReadMode,
    observed: Vec<u32>,
    calls: usize,
    model_ns: u128,
    stop_tokens: StopTokens,
    dialogue_protocol: Option<DialogueProtocol>,
}
impl Bundle {
    pub fn text_session(&self, mode: ReadMode) -> Result<TextSession<'_>> {
        let mut session = TextSession {
            bundle: self,
            state: self.model().new_session(),
            current: None,
            pending: None,
            mode,
            observed: Vec::new(),
            calls: 0,
            model_ns: 0,
            stop_tokens: StopTokens {
                eos: 1,
                turn_end: self.tokenizer().token_id("<|turn_end|>"),
            },
            dialogue_protocol: None,
        };
        session.observe(0)?;
        Ok(session)
    }
    pub fn generate(&self, request: &Request) -> Result<Generation> {
        let clock = Instant::now();
        validate_budget(
            1,
            self.tokenizer().encode(&request.prompt).len(),
            request.max_new_tokens,
            self.model().config().context,
        )?;
        let mut session = self.text_session(request.read_mode)?;
        session.append(&request.prompt)?;
        let mut result = session.generate(
            request.max_new_tokens,
            request.selection,
            request.first_sentence,
        )?;
        result.prompt = request.prompt.clone();
        result.incremental_step_calls = session.calls;
        result.model_step_nanoseconds = session.model_ns;
        result.whole_generation_nanoseconds = clock.elapsed().as_nanos();
        Ok(result)
    }

    /// Encode a literal-role prefix without decoding/re-encoding its segments.
    /// Validate the binding, complete IDs and capacity before the first model step.
    /// The protocol's initial BOS is consumed once; explicit BOS/EOS in message
    /// content and completed assistant EOS tokens remain in the observed history.
    pub fn dialogue_session(
        &self,
        protocol: &DialogueProtocol,
        messages: &[Message<'_>],
        mode: ReadMode,
    ) -> std::result::Result<TextSession<'_>, DialogueSessionError> {
        let tokens = self.dialogue_prefix(protocol, messages)?;
        self.start_dialogue_session(protocol, &tokens, mode)
    }

    /// Generate under the explicitly bound literal-role EOS semantics. This
    /// adapter does not adopt the protocol into an existing sealed bundle.
    pub fn generate_dialogue(
        &self,
        protocol: &DialogueProtocol,
        messages: &[Message<'_>],
        max_new_tokens: usize,
        selection: Selection,
        read_mode: ReadMode,
        first_sentence: bool,
    ) -> std::result::Result<DialogueGeneration, DialogueSessionError> {
        let clock = Instant::now();
        let tokens = self.dialogue_prefix(protocol, messages)?;
        validate_generation_budget(tokens.len(), max_new_tokens, self.model().config().context)?;
        let protocol_identity = protocol.identity()?;
        let mut session = self.start_dialogue_session(protocol, &tokens, read_mode)?;
        let mut generation = session.generate(max_new_tokens, selection, first_sentence)?;
        generation.incremental_step_calls = session.calls;
        generation.model_step_nanoseconds = session.model_ns;
        generation.whole_generation_nanoseconds = clock.elapsed().as_nanos();
        Ok(DialogueGeneration {
            protocol: protocol.clone(),
            protocol_identity,
            generation,
        })
    }

    fn dialogue_prefix(
        &self,
        protocol: &DialogueProtocol,
        messages: &[Message<'_>],
    ) -> std::result::Result<Vec<u32>, DialogueSessionError> {
        let encoder = protocol.bind(self.tokenizer())?;
        // Bundle::from_parts can bypass the loaded bundle's BOS/EOS checks.
        if protocol.bos_id != 0 || protocol.eos_id != 1 {
            return Err(invalid("text sessions require protocol BOS0 and EOS1").into());
        }
        let tokens = encoder.encode_assistant_prefix(messages).tokens;
        if tokens.first() != Some(&protocol.bos_id) {
            return Err(invalid("dialogue prefix is missing its initial BOS").into());
        }
        validate_append_tokens(self, 0, &tokens)?;
        Ok(tokens)
    }

    fn start_dialogue_session(
        &self,
        protocol: &DialogueProtocol,
        tokens: &[u32],
        mode: ReadMode,
    ) -> std::result::Result<TextSession<'_>, DialogueSessionError> {
        let mut session = self.text_session(mode)?;
        session.stop_tokens = StopTokens {
            eos: protocol.eos_id,
            turn_end: None,
        };
        session.dialogue_protocol = Some(protocol.clone());
        session.append_tokens(&tokens[1..])?;
        Ok(session)
    }
}
impl<'a> TextSession<'a> {
    fn observe(&mut self, token: u32) -> Result<()> {
        let position = self.state.len();
        let clock = Instant::now();
        let step = self
            .bundle
            .model()
            .step(&mut self.state, token, self.mode)?;
        self.model_ns += clock.elapsed().as_nanos();
        let total = step
            .probabilities
            .iter()
            .try_fold(0u64, |a, &b| a.checked_add(b))
            .ok_or_else(|| invalid("probability total overflow"))?;
        let read_total = step
            .read_masses
            .iter()
            .try_fold(step.no_read_mass, |a, &b| a.checked_add(b))
            .ok_or_else(|| invalid("attention total overflow"))?;
        if step.read_masses.len() != position
            || total != PROBABILITY_TOTAL
            || read_total != PROBABILITY_TOTAL
            || (self.mode == ReadMode::NoRead
                && (step.no_read_mass != PROBABILITY_TOTAL
                    || step.read_masses.iter().any(|&m| m != 0)))
        {
            return Err(invalid(
                "integer session normalization or causal slots differ",
            ));
        }
        self.current = Some(step);
        self.observed.push(token);
        self.calls += 1;
        Ok(())
    }
    fn consume_pending(&mut self) -> Result<()> {
        if let Some(token) = self.pending {
            self.observe(token)?;
            self.pending = None;
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.state.len() + usize::from(self.pending.is_some())
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The explicit protocol binding for sessions constructed by `dialogue_session`.
    /// It describes input/termination semantics, not the model's training history.
    pub fn dialogue_protocol(&self) -> Option<&DialogueProtocol> {
        self.dialogue_protocol.as_ref()
    }

    pub fn append(&mut self, text: &str) -> Result<usize> {
        let tokens = self.bundle.tokenizer().encode(text);
        self.append_tokens(&tokens)
    }

    /// Append exact IDs without inserting BOS/EOS or re-tokenizing. An empty
    /// slice, any invalid ID or insufficient capacity fails before consuming a
    /// pending generated token or changing model state. Model execution errors
    /// after validation are propagated; this is not a rollback transaction.
    pub fn append_tokens(&mut self, tokens: &[u32]) -> Result<usize> {
        validate_append_tokens(self.bundle, self.len(), tokens)?;
        self.consume_pending()?;
        for &token in tokens {
            self.observe(token)?;
        }
        Ok(tokens.len())
    }
    /// The seed is explicit per call; use sampler_state_after to continue its stream.
    pub fn generate(
        &mut self,
        max_new_tokens: usize,
        selection: Selection,
        first_sentence: bool,
    ) -> Result<Generation> {
        let clock = Instant::now();
        let calls_at_entry = self.calls;
        let model_ns_at_entry = self.model_ns;
        validate_generation_budget(
            self.len(),
            max_new_tokens,
            self.bundle.model().config().context,
        )?;
        self.consume_pending()?;
        let prompt_ids = self.observed.clone();
        let (policy, seed) = match selection {
            Selection::Greedy => (SamplePolicy::Greedy, 0),
            Selection::Categorical { top_k, seed } => (SamplePolicy::Categorical { top_k }, seed),
        };
        let mut sampler = Sampler::new(seed);
        let mut generated = Vec::new();
        let mut decisions = Vec::new();
        let mut stop = Stop::MaximumNewTokens;
        for position in 0..max_new_tokens {
            self.consume_pending()?;
            let step = self
                .current
                .as_ref()
                .ok_or_else(|| invalid("missing session prediction"))?;
            let selected = sampler
                .select(&step.probabilities, policy)
                .map_err(|e| invalid(format!("integer sampling: {e}")))?;
            let mut hash = Sha256::new();
            for mass in &step.probabilities {
                hash.update(mass.to_le_bytes());
            }
            decisions.push(Decision {
                selected_token: selected as u32,
                probability_q48: step.probabilities[selected],
                probability_sum_q48: PROBABILITY_TOTAL,
                probability_sha256_le_u64: hex::encode(hash.finalize()),
                exposed_causal_slots: step.read_masses.len(),
                no_read_mass_q48: step.no_read_mass,
            });
            generated.push(selected as u32);
            self.pending = Some(selected as u32);
            if let Some(reason) = self.stop_tokens.reason(selected as u32) {
                stop = reason;
                break;
            }
            if first_sentence
                && self
                    .bundle
                    .tokenizer()
                    .decode_bytes(&generated)
                    .contains(&b'.')
            {
                stop = Stop::FirstSentenceBoundary;
                break;
            }
            if let Some(period) = short_cycle(&generated) {
                stop = Stop::ShortCycle { period };
                break;
            }
            if position + 1 == max_new_tokens {
                break;
            }
        }
        let end = generated
            .iter()
            .position(|&token| self.stop_tokens.reason(token).is_some())
            .unwrap_or(generated.len());
        let raw = self.bundle.tokenizer().decode_bytes(&generated);
        let bytes = self.bundle.tokenizer().decode_bytes(&generated[..end]);
        Ok(Generation {
            prompt: String::new(),
            prompt_token_ids: prompt_ids,
            generated_token_ids: generated,
            response_text: String::from_utf8_lossy(&bytes).trim().to_owned(),
            raw_decoded: String::from_utf8_lossy(&raw).into_owned(),
            utf8_decodable: std::str::from_utf8(&bytes).is_ok(),
            stop,
            decisions,
            selection,
            read_mode: self.mode,
            context_capacity: self.bundle.model().config().context,
            session_tokens_including_pending: self.len(),
            incremental_step_calls: self.calls - calls_at_entry,
            model_step_nanoseconds: self.model_ns - model_ns_at_entry,
            whole_generation_nanoseconds: clock.elapsed().as_nanos(),
            bundle_sha256: self.bundle.identity().to_owned(),
            tokenizer_cid: self.bundle.tokenizer().address(),
            sampler_state_after: sampler.state(),
        })
    }
}
fn validate_append_tokens(bundle: &Bundle, existing: usize, tokens: &[u32]) -> Result<()> {
    if tokens.is_empty()
        || existing
            .checked_add(tokens.len())
            .is_none_or(|n| n > bundle.model().config().context)
    {
        return Err(invalid("empty tokens or session context256 exhausted"));
    }
    for (position, &token) in tokens.iter().enumerate() {
        if token as usize >= bundle.model().config().vocab_size
            || token as usize >= bundle.tokenizer().vocab_size()
        {
            return Err(invalid(format!(
                "token {token} at append position {position} is outside model/tokenizer vocabulary"
            )));
        }
    }
    Ok(())
}
fn validate_budget(existing: usize, prompt: usize, generate: usize, capacity: usize) -> Result<()> {
    if prompt == 0 {
        return Err(invalid("empty prompt"));
    }
    validate_generation_budget(
        existing
            .checked_add(prompt)
            .ok_or_else(|| invalid("prompt length overflow"))?,
        generate,
        capacity,
    )
}
fn validate_generation_budget(existing: usize, generate: usize, capacity: usize) -> Result<()> {
    if generate == 0 || existing.checked_add(generate).is_none_or(|n| n > capacity) {
        return Err(invalid(
            "request exceeds full256 session budget; shorten the requested continuation explicitly",
        ));
    }
    Ok(())
}
fn short_cycle(tokens: &[u32]) -> Option<usize> {
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::IntegerModel;
    use serde_json::json;
    use uor_r4_tokenizer::ByteBpeTokenizer;

    fn small_bundle(turn_end: bool, swap_bos_eos: bool, vocab_size: usize) -> Bundle {
        let mut pieces = vec![
            "<|bos|>".to_owned(),
            "<|eos|>".to_owned(),
            "<|unk|>".to_owned(),
            "Assistant: ".to_owned(),
            "User: ".to_owned(),
            "Hi".to_owned(),
            if turn_end { "<|turn_end|>" } else { "$" }.to_owned(),
        ];
        if swap_bos_eos {
            pieces.swap(0, 1);
        }
        pieces.extend((pieces.len()..vocab_size).map(|id| format!("t{id}")));
        let vocab: serde_json::Map<String, serde_json::Value> = pieces
            .iter()
            .enumerate()
            .map(|(id, piece)| (piece.clone(), json!(id)))
            .collect();
        let added: Vec<_> = pieces[..6 + usize::from(turn_end)]
            .iter()
            .enumerate()
            .map(|(id, piece)| json!({"id":id,"content":piece}))
            .collect();
        let bytes = serde_json::to_vec(&json!({
            "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
            "model":{"type":"BPE","vocab":vocab,"merges":[]},
            "added_tokens":added
        }))
        .unwrap();
        Bundle::from_parts(
            IntegerModel::synthetic_for_test(),
            ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).unwrap(),
            "synthetic-token-session-boundary".to_owned(),
        )
    }

    // Force only the next selection to exercise API termination semantics;
    // these fixture predictions are not a learned-model behavior result.
    fn force_next(session: &mut TextSession<'_>, token: usize) {
        let probabilities = &mut session.current.as_mut().unwrap().probabilities;
        probabilities.fill(0);
        probabilities[token] = PROBABILITY_TOTAL;
    }

    #[test]
    fn token_session_preflight_preserves_pending_state() {
        let bundle = small_bundle(false, false, 7);
        let mut session = bundle.text_session(ReadMode::Enabled).unwrap();
        force_next(&mut session, 6);
        session.generate(1, Selection::Greedy, false).unwrap();
        let before = (
            session.len(),
            session.state.len(),
            session.observed.clone(),
            session.pending,
            session.calls,
            session.model_ns,
            session.current.as_ref().unwrap().state.clone(),
            session.current.as_ref().unwrap().probabilities.clone(),
        );
        let oversized = vec![5; bundle.model().config().context - session.len() + 1];
        for tokens in [&[][..], &[5, 7], &[5, 4096], oversized.as_slice()] {
            assert!(session.append_tokens(tokens).is_err());
            assert_eq!(
                before,
                (
                    session.len(),
                    session.state.len(),
                    session.observed.clone(),
                    session.pending,
                    session.calls,
                    session.model_ns,
                    session.current.as_ref().unwrap().state.clone(),
                    session.current.as_ref().unwrap().probabilities.clone(),
                )
            );
        }
        // A tokenizer-valid ID can still be invalid for the model.
        let wider = small_bundle(false, false, 4097);
        assert!(validate_append_tokens(&wider, 1, &[5, 4096]).is_err());
        assert_eq!(session.append_tokens(&[0, 5]).unwrap(), 2);
        assert_eq!(session.observed, [0, 6, 0, 5]);
        assert_eq!(session.pending, None);
    }

    #[test]
    fn token_session_dialogue_binding_and_prefix_are_exact() {
        let bundle = Bundle::create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let messages = [
            Message {
                role: "user",
                content: " \r\ncafé  \r\nz \r\n",
            },
            Message {
                role: "assistant",
                content: "<|bos|>ok",
            },
            Message {
                role: "user",
                content: "Hi",
            },
        ];
        // This fixture's ordinary byte IDs are 7 + byte, independently of the
        // serializer. Preserve the literal BOS and completed assistant EOS.
        let bytes = |s: &str| s.bytes().map(|b| 7 + u32::from(b)).collect::<Vec<_>>();
        let mut expected = vec![0];
        expected.extend(bytes("User: café  \nz\nAssistant: "));
        expected.push(0);
        expected.extend(bytes("ok"));
        expected.push(1);
        expected.extend(bytes("\nUser: Hi\nAssistant: "));
        assert_eq!(
            bundle.dialogue_prefix(&protocol, &messages).unwrap(),
            expected
        );

        let mut mismatch = protocol.clone();
        mismatch.tokenizer_cid.push('x');
        assert!(matches!(
            bundle.dialogue_session(&mismatch, &messages, ReadMode::Enabled),
            Err(DialogueSessionError::Protocol(
                DialogueError::TokenizerMismatch { .. }
            ))
        ));
        let long = "x".repeat(256);
        assert!(bundle
            .dialogue_session(
                &protocol,
                &[Message {
                    role: "user",
                    content: &long
                }],
                ReadMode::Enabled
            )
            .is_err());
        assert!(bundle
            .generate_dialogue(
                &protocol,
                &[],
                256,
                Selection::Greedy,
                ReadMode::Enabled,
                false
            )
            .is_err());
        let swapped = small_bundle(false, true, 7);
        let wrong_ids = DialogueProtocol::literal_roles_v1(swapped.tokenizer()).unwrap();
        assert!(matches!(
            swapped.dialogue_session(&wrong_ids, &[], ReadMode::Enabled),
            Err(DialogueSessionError::Integer(_))
        ));
    }

    #[test]
    fn token_session_stop_tokens_follow_the_bound_protocol() {
        let ordinary = small_bundle(false, false, 7);
        let mut text = ordinary.text_session(ReadMode::Enabled).unwrap();
        force_next(&mut text, 6);
        let dollar = text.generate(1, Selection::Greedy, false).unwrap();
        assert_eq!(dollar.stop, Stop::MaximumNewTokens);
        assert_eq!(dollar.response_text, "$");
        assert_eq!(dollar.generated_token_ids, [6]);

        let dedicated = small_bundle(true, false, 7);
        let mut text = dedicated.text_session(ReadMode::Enabled).unwrap();
        force_next(&mut text, 6);
        let turn_end = text.generate(1, Selection::Greedy, false).unwrap();
        assert_eq!(turn_end.stop, Stop::TurnEnd);
        assert_eq!(turn_end.response_text, "");
        assert_eq!(turn_end.raw_decoded, "<|turn_end|>");

        let protocol = DialogueProtocol::literal_roles_v1(dedicated.tokenizer()).unwrap();
        let mut dialogue = dedicated
            .dialogue_session(&protocol, &[], ReadMode::Enabled)
            .unwrap();
        assert_eq!(dialogue.dialogue_protocol(), Some(&protocol));
        assert_eq!(dialogue.observed, [0, 3]);
        force_next(&mut dialogue, 6);
        let literal = dialogue.generate(1, Selection::Greedy, false).unwrap();
        assert_eq!(literal.stop, Stop::MaximumNewTokens);
        assert_eq!(literal.response_text, "<|turn_end|>");

        let mut dialogue = dedicated
            .dialogue_session(&protocol, &[], ReadMode::Enabled)
            .unwrap();
        force_next(&mut dialogue, 1);
        let eos = dialogue.generate(1, Selection::Greedy, false).unwrap();
        assert_eq!(eos.stop, Stop::Eos);
        assert_eq!(eos.response_text, "");
        assert_eq!(eos.raw_decoded, "<|eos|>");
    }

    #[test]
    fn token_session_text_append_keeps_exact_id_trajectory() {
        let bundle = small_bundle(false, false, 7);
        let mut text = bundle.text_session(ReadMode::Enabled).unwrap();
        let mut ids = bundle.text_session(ReadMode::Enabled).unwrap();
        text.append("Hi").unwrap();
        ids.append_tokens(&bundle.tokenizer().encode("Hi")).unwrap();
        let selection = Selection::Categorical {
            top_k: 16,
            seed: 20260926,
        };
        let text = text.generate(1, selection, false).unwrap();
        let ids = ids.generate(1, selection, false).unwrap();
        assert_eq!(text.prompt_token_ids, ids.prompt_token_ids);
        assert_eq!(text.generated_token_ids, ids.generated_token_ids);
        assert_eq!(text.stop, ids.stop);
        assert_eq!(text.response_text, ids.response_text);
        assert_eq!(text.sampler_state_after, ids.sampler_state_after);
        assert_eq!(
            serde_json::to_value(text.decisions).unwrap(),
            serde_json::to_value(ids.decisions).unwrap()
        );
    }
    #[test]
    fn generation_budget_is_prospective_and_does_not_truncate() {
        assert!(validate_budget(1, 127, 128, 256).is_ok());
        assert!(validate_budget(1, 128, 128, 256).is_err());
        assert!(validate_budget(1, 0, 10, 256).is_err());
        assert!(validate_generation_budget(usize::MAX, 1, 256).is_err());
    }
    #[test]
    fn retained_cycle_stop_requires_three_complete_repeats() {
        assert_eq!(short_cycle(&[2, 3, 2, 3]), None);
        assert_eq!(short_cycle(&[2, 3, 2, 3, 2, 3]), Some(2));
    }
}
