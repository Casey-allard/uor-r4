//! Tokenizer-bound persistent decoder turns for the geometric programme.
//!
//! The historical router corpus remains word-indexed for retrieval.  These
//! records add the exact source-token ids and memory-to-layer adapter identity
//! required by issue #950 without changing that corpus representation.

use serde::{Deserialize, Serialize};

use crate::{identity_key, UorR4Router};

pub const MAX_BOUND_TURN_TOKENS: usize = 256;

/// One user or assistant turn committed to the identity-scoped manifold.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenizerBoundMemorySpan {
    pub sequence: u64,
    pub role: String,
    pub text: String,
    pub token_ids: Vec<u32>,
    pub tokenizer_cid: String,
    pub adapter_identity: String,
    pub source_cid: String,
    pub route_window: u64,
    pub r4_coordinates: [f64; 4],
    pub route_uor_address: String,
    pub provenance: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecoderMemoryError {
    EmptyIdentity,
    EmptyRole,
    EmptyText,
    EmptyTokens,
    TokenLimit { requested: usize, maximum: usize },
    EmptyBinding,
    BindingMismatch,
    ArithmeticOverflow,
}

impl std::fmt::Display for DecoderMemoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "tokenizer-bound router memory unavailable: {self:?}"
        )
    }
}

impl std::error::Error for DecoderMemoryError {}

impl UorR4Router {
    /// Commit one exact-token turn to both the existing identity-scoped
    /// manifold and the new tokenizer/adapter-bound record stream.
    #[allow(clippy::too_many_arguments)]
    pub fn commit_tokenizer_bound_turn(
        &mut self,
        identity: &str,
        role: &str,
        text: &str,
        token_ids: &[u32],
        tokenizer_cid: &str,
        adapter_identity: &str,
        source_cid: &str,
    ) -> Result<TokenizerBoundMemorySpan, DecoderMemoryError> {
        let identity = identity.trim();
        let role = role.trim();
        let text = text.trim();
        if identity.is_empty() {
            return Err(DecoderMemoryError::EmptyIdentity);
        }
        if role.is_empty() {
            return Err(DecoderMemoryError::EmptyRole);
        }
        if text.is_empty() {
            return Err(DecoderMemoryError::EmptyText);
        }
        if token_ids.is_empty() {
            return Err(DecoderMemoryError::EmptyTokens);
        }
        if token_ids.len() > MAX_BOUND_TURN_TOKENS {
            return Err(DecoderMemoryError::TokenLimit {
                requested: token_ids.len(),
                maximum: MAX_BOUND_TURN_TOKENS,
            });
        }
        if tokenizer_cid.trim().is_empty()
            || adapter_identity.trim().is_empty()
            || source_cid.trim().is_empty()
        {
            return Err(DecoderMemoryError::EmptyBinding);
        }
        let scope = identity_key(identity);
        if self
            .decoder_memories_by_identity
            .get(&scope)
            .is_some_and(|records| {
                records.iter().any(|record| {
                    record.tokenizer_cid != tokenizer_cid
                        || record.adapter_identity != adapter_identity
                        || record.source_cid != source_cid
                })
            })
        {
            return Err(DecoderMemoryError::BindingMismatch);
        }

        // Advance the identity state with the turn, then derive the exact R4
        // coordinate the content-bearing corpus commit will inhabit.
        self.evolve_state(identity, text, 0.85);
        let (routing, hopf_input) =
            self.route_query_to_manifold_native_with_hopf_input(text, identity);
        let projected = self.get_state_4d_projection_native(&hopf_input);
        let mut r4_coordinates = [0.0f64; 4];
        for (target, source) in r4_coordinates.iter_mut().zip(projected) {
            *target = source;
        }
        self.index_sentence(text, identity);

        let sequence = u64::try_from(
            self.decoder_memories_by_identity
                .get(&scope)
                .map_or(0, Vec::len),
        )
        .map_err(|_| DecoderMemoryError::ArithmeticOverflow)?;
        let provenance = memory_provenance(
            identity,
            sequence,
            role,
            text,
            token_ids,
            tokenizer_cid,
            adapter_identity,
            source_cid,
            &routing.routed.uor_address,
            r4_coordinates,
        );
        let record = TokenizerBoundMemorySpan {
            sequence,
            role: role.to_owned(),
            text: text.to_owned(),
            token_ids: token_ids.to_vec(),
            tokenizer_cid: tokenizer_cid.to_owned(),
            adapter_identity: adapter_identity.to_owned(),
            source_cid: source_cid.to_owned(),
            route_window: routing.routed.window_index,
            r4_coordinates,
            route_uor_address: routing.routed.uor_address,
            provenance,
        };
        self.decoder_memories_by_identity
            .entry(scope)
            .or_default()
            .push(record.clone());
        Ok(record)
    }

    /// Retrieve the exact-bound stream for one identity.  Existing records
    /// with a different tokenizer/adapter/source fail closed; another identity
    /// simply has no stream.
    pub fn tokenizer_bound_turns(
        &self,
        identity: &str,
        tokenizer_cid: &str,
        adapter_identity: &str,
        source_cid: &str,
    ) -> Result<Vec<TokenizerBoundMemorySpan>, DecoderMemoryError> {
        let scope = identity_key(identity);
        let Some(records) = self.decoder_memories_by_identity.get(&scope) else {
            return Ok(Vec::new());
        };
        if records.iter().any(|record| {
            record.tokenizer_cid != tokenizer_cid
                || record.adapter_identity != adapter_identity
                || record.source_cid != source_cid
        }) {
            return Err(DecoderMemoryError::BindingMismatch);
        }
        Ok(records.clone())
    }

    /// Latest whole spans under explicit context bounds, returned in their
    /// original order.  A span is never silently split or re-tokenized.
    #[allow(clippy::too_many_arguments)]
    pub fn latest_tokenizer_bound_turns(
        &self,
        identity: &str,
        tokenizer_cid: &str,
        adapter_identity: &str,
        source_cid: &str,
        maximum_spans: usize,
        maximum_tokens: usize,
    ) -> Result<Vec<TokenizerBoundMemorySpan>, DecoderMemoryError> {
        let records =
            self.tokenizer_bound_turns(identity, tokenizer_cid, adapter_identity, source_cid)?;
        let mut selected = Vec::new();
        let mut tokens = 0usize;
        for record in records.iter().rev() {
            if selected.len() >= maximum_spans {
                break;
            }
            let Some(next_tokens) = tokens.checked_add(record.token_ids.len()) else {
                return Err(DecoderMemoryError::ArithmeticOverflow);
            };
            if next_tokens > maximum_tokens {
                continue;
            }
            tokens = next_tokens;
            selected.push(record.clone());
        }
        selected.reverse();
        Ok(selected)
    }

    /// Deterministic identity of the bound record stream for evidence and
    /// context provenance.  `None` means the identity has no such memory.
    pub fn tokenizer_bound_state_cid(&self, identity: &str) -> Option<String> {
        let records = self
            .decoder_memories_by_identity
            .get(&identity_key(identity))?;
        let bytes = serde_json::to_vec(records).ok()?;
        Some(format!("blake3:{}", blake3::hash(&bytes).to_hex()))
    }
}

#[allow(clippy::too_many_arguments)]
fn memory_provenance(
    identity: &str,
    sequence: u64,
    role: &str,
    text: &str,
    token_ids: &[u32],
    tokenizer_cid: &str,
    adapter_identity: &str,
    source_cid: &str,
    route_uor_address: &str,
    r4_coordinates: [f64; 4],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.tokenizer-bound-memory/1");
    for bytes in [
        identity.as_bytes(),
        role.as_bytes(),
        text.as_bytes(),
        tokenizer_cid.as_bytes(),
        adapter_identity.as_bytes(),
        source_cid.as_bytes(),
        route_uor_address.as_bytes(),
    ] {
        hasher.update(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(bytes);
    }
    hasher.update(&sequence.to_le_bytes());
    hasher.update(
        &u64::try_from(token_ids.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for token in token_ids {
        hasher.update(&token.to_le_bytes());
    }
    for coordinate in r4_coordinates {
        hasher.update(&coordinate.to_bits().to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}
