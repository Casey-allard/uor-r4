//! Stateful text sessions using integer predictions and integer token selection.
//! Supports `generate_stream` and incremental UTF-8 decoding with `pending_bytes`.
pub use crate::session::{ChatTokenStream, IncrementalUtf8Decoder};
use crate::{
    bundle::Bundle, invalid, IntegerSession, IntegerStep, ReadMode, Result, SamplePolicy, Sampler,
    PROBABILITY_TOTAL,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;

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
    pub fn append(&mut self, text: &str) -> Result<usize> {
        let tokens = self.bundle.tokenizer().encode(text);
        if tokens.is_empty()
            || self
                .len()
                .checked_add(tokens.len())
                .is_none_or(|n| n > self.bundle.model().config().context)
        {
            return Err(invalid("empty text or session context256 exhausted"));
        }
        self.consume_pending()?;
        for &token in &tokens {
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
            let turn_end_id = self
                .bundle
                .tokenizer()
                .token_id("<|turn_end|>")
                .unwrap_or(6);
            if selected == 1 {
                stop = Stop::Eos;
                break;
            }
            if selected == turn_end_id as usize {
                stop = Stop::TurnEnd;
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
        let turn_end_id = self
            .bundle
            .tokenizer()
            .token_id("<|turn_end|>")
            .unwrap_or(6);
        let end = generated
            .iter()
            .position(|&token| token == 1 || token == turn_end_id)
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
