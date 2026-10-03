//! Integer joint normalization for source-copy occurrences, Period and Stop.
//!
//! This is an arithmetic wrapper, not a learned scorer or a language result.
//! It accepts no parent distribution, labels, answer length or scripted phase.
//! Heads contribute Q24 logits to the same actions: sum their logits first,
//! then normalize once. Every source occurrence and Period occupies one row of
//! the existing native reducer; Stop occupies its zero-payload null slot. The
//! payloads are all zero because only the shared Q31 masses are used here.
//!
//! Equal emitted tokens are aggregated *after* normalization, including source
//! aliases of Period and EOS. Greedy selection compares exact summed masses;
//! ties select the smallest token ID. Action offsets remain separate evidence.
//! A copy offset is an offset in the supplied source view, not necessarily an
//! original store-token offset; its caller retains that view's byte provenance.
//!
//! The reducer allocates its scratch at construction. This training-crate
//! wrapper allocates owned trace vectors on each successful call and performs
//! bounded token aggregation/sorting. It uses integer numerical arithmetic;
//! no compiled-opcode, allocation-free whole-path or native-chat claim follows.

use serde::Serialize;
use uor_r4_integer::geometric_read::NativeGeometricRead;
use uor_r4_tokenizer::{dialogue::DialogueProtocol, ByteBpeTokenizer};

use crate::{invalid, sha256_bytes, Result};

pub const POLICY: &str = "source-copy-period-stop/1;checked-sum-head-Q24-logits-before-one-native-Q31-normalization;copy-occurrences-and-period-rows;stop-null;aggregate-token-aliases-after-normalization;smallest-token-ID-ties;no-parent-label-or-phase";
pub const MAX_SOURCE_TOKENS: usize = 128;
pub const MAX_HEADS: usize = 2;
const MAX_OCCURRENCE_ROWS: usize = MAX_SOURCE_TOKENS + 1;

/// Admitted action-token identity. Reconstruct from the actual tokenizer bytes;
/// there is no unchecked deserializer or constructor accepting arbitrary IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceActionBinding {
    tokenizer_sha256: String,
    protocol: DialogueProtocol,
    period_token_id: u32,
    // Admission-only map also rejects holes among sparse added-token IDs.
    #[serde(skip)]
    token_byte_lengths: Vec<u32>,
}

impl SourceActionBinding {
    pub fn new(tokenizer_bytes: &[u8]) -> Result<Self> {
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(tokenizer_bytes)
            .ok_or_else(|| invalid("source actions require a valid byte-BPE tokenizer"))?;
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)
            .map_err(|e| invalid(format!("source action protocol: {e}")))?;
        let period = tokenizer.encode(".");
        let period_token_id = match period.as_slice() {
            [id] if tokenizer.decode_bytes(&[*id]) == b"." => *id,
            _ => {
                return Err(invalid(
                    "source actions require period to encode as one exact period token",
                ))
            }
        };
        if period_token_id == protocol.eos_id {
            return Err(invalid("source action Period and EOS IDs must differ"));
        }
        Ok(Self {
            tokenizer_sha256: sha256_bytes(tokenizer_bytes),
            protocol,
            period_token_id,
            token_byte_lengths: tokenizer.token_byte_lengths(),
        })
    }

    pub fn tokenizer_sha256(&self) -> &str {
        &self.tokenizer_sha256
    }
    pub fn protocol(&self) -> &DialogueProtocol {
        &self.protocol
    }
    pub fn period_token_id(&self) -> u32 {
        self.period_token_id
    }
    pub fn eos_token_id(&self) -> u32 {
        self.protocol.eos_id
    }
    pub fn vocab_size(&self) -> usize {
        self.token_byte_lengths.len()
    }

    fn admits_token(&self, id: u32) -> bool {
        usize::try_from(id)
            .ok()
            .and_then(|index| self.token_byte_lengths.get(index))
            .is_some_and(|&bytes| bytes != 0)
    }
}

/// All logits use the same absolute Q24 score unit/gauge. Copy order is the
/// caller's source-view occurrence order; no deduplication or masking occurs.
#[derive(Clone, Copy, Debug)]
pub struct ActionHeadScores<'a> {
    pub copy_q24: &'a [i64],
    pub period_q24: i64,
    pub stop_q24: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HeadActionTrace {
    pub copy_q24: Vec<i64>,
    pub period_q24: i64,
    pub stop_q24: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SourceAction {
    Copy { source_offset: usize },
    Period,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ActionMass {
    /// Copy offsets are 0..N; Period is N; Stop is N+1.
    pub action_offset: usize,
    pub action: SourceAction,
    pub token_id: u32,
    pub score_q24: i64,
    pub weight_q31: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TokenMass {
    pub token_id: u32,
    pub weight_q31: u64,
    /// Every contributing action offset, in original action order, including
    /// actions assigned zero mass by the admitted exp-table tail.
    pub action_offsets: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ActionTrace {
    pub policy: &'static str,
    pub tokenizer_sha256: String,
    pub protocol: DialogueProtocol,
    pub period_token_id: u32,
    pub eos_token_id: u32,
    pub head_scores: Vec<HeadActionTrace>,
    pub actions: Vec<ActionMass>,
    /// Sorted by token ID, with weights sharing `total_weight_q31`.
    pub token_masses: Vec<TokenMass>,
    pub max_score_q24: i64,
    pub total_weight_q31: u64,
    pub chosen_token_id: u32,
    pub chosen_weight_q31: u64,
}

/// Fixed bounded numerical scratch plus an admitted tokenizer/protocol binding.
/// Exp-table admission is inherited from NativeGeometricRead. The enclosing
/// artifact must bind the table's actual bytes to its declared producer; these
/// shape/order checks alone do not establish exponential approximation error.
pub struct NativeSourceActions {
    binding: SourceActionBinding,
    heads: usize,
    reducer: NativeGeometricRead,
    score_scratch: [i64; MAX_OCCURRENCE_ROWS],
    zero_age: [i64; MAX_OCCURRENCE_ROWS],
    zero_values: [i32; MAX_OCCURRENCE_ROWS],
}

impl NativeSourceActions {
    pub fn new(binding: SourceActionBinding, heads: usize, exp_q31: &[u32]) -> Result<Self> {
        if !(1..=MAX_HEADS).contains(&heads) {
            return Err(invalid("source action heads must be in 1..=2"));
        }
        let reducer = NativeGeometricRead::new(MAX_OCCURRENCE_ROWS, 1, exp_q31)
            .map_err(|e| invalid(format!("source action reducer: {e}")))?;
        Ok(Self {
            binding,
            heads,
            reducer,
            score_scratch: [0; MAX_OCCURRENCE_ROWS],
            zero_age: [0; MAX_OCCURRENCE_ROWS],
            zero_values: [0; MAX_OCCURRENCE_ROWS],
        })
    }

    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn heads(&self) -> usize {
        self.heads
    }

    /// Return an owned trace only on success. Invalid input may change private
    /// scratch, but cannot expose a partial result or alter a prior owned trace.
    /// Empty source is supported: Period and Stop still compete. Source aliases
    /// of either special token remain ordinary occurrence rows until aggregation.
    pub fn reduce(
        &mut self,
        source_ids: &[u32],
        head_scores: &[ActionHeadScores<'_>],
    ) -> Result<ActionTrace> {
        let count = source_ids.len();
        if count > MAX_SOURCE_TOKENS {
            return Err(invalid("source actions exceed 128 source occurrences"));
        }
        if head_scores.len() != self.heads
            || head_scores.iter().any(|head| head.copy_q24.len() != count)
        {
            return Err(invalid("source action head or copy-score shape differs"));
        }
        for &id in source_ids {
            if !self.binding.admits_token(id) {
                return Err(invalid("source action token is unassigned or out of range"));
            }
        }
        for offset in 0..count {
            let mut score = 0i64;
            for head in head_scores {
                score = score.checked_add(head.copy_q24[offset]).ok_or_else(|| {
                    invalid(format!(
                        "source action head-score overflow at copy {offset}"
                    ))
                })?;
            }
            self.score_scratch[offset] = score;
        }
        let mut period = 0i64;
        let mut stop = 0i64;
        for head in head_scores {
            period = period
                .checked_add(head.period_q24)
                .ok_or_else(|| invalid("source action head-score overflow at Period"))?;
            stop = stop
                .checked_add(head.stop_q24)
                .ok_or_else(|| invalid("source action head-score overflow at Stop"))?;
        }
        self.score_scratch[count] = period;
        let rows = count + 1;
        let reduction = self
            .reducer
            .reduce(
                &self.score_scratch[..rows],
                &self.zero_age[..rows],
                stop,
                &self.zero_values[..rows],
            )
            .map_err(|e| invalid(format!("source action reduction: {e}")))?;
        let mut actions = Vec::with_capacity(rows + 1);
        for (offset, &token_id) in source_ids.iter().enumerate() {
            actions.push(ActionMass {
                action_offset: offset,
                action: SourceAction::Copy {
                    source_offset: offset,
                },
                token_id,
                score_q24: self.score_scratch[offset],
                weight_q31: reduction.occurrence_weights_q31[offset],
            });
        }
        actions.push(ActionMass {
            action_offset: count,
            action: SourceAction::Period,
            token_id: self.binding.period_token_id(),
            score_q24: period,
            weight_q31: reduction.occurrence_weights_q31[count],
        });
        actions.push(ActionMass {
            action_offset: rows,
            action: SourceAction::Stop,
            token_id: self.binding.eos_token_id(),
            score_q24: stop,
            weight_q31: reduction.no_read_weight_q31,
        });
        let token_masses = aggregate_tokens(&actions)?;
        let mut mass_sum = 0u64;
        let mut chosen: Option<&TokenMass> = None;
        for token in &token_masses {
            mass_sum = mass_sum
                .checked_add(token.weight_q31)
                .ok_or_else(|| invalid("source action total token mass overflow"))?;
            // Sorted token IDs and strict greater-than preserve smallest-ID ties.
            if chosen.is_none_or(|old| token.weight_q31 > old.weight_q31) {
                chosen = Some(token);
            }
        }
        if mass_sum != reduction.total_weight_q31 {
            return Err(invalid("source action token mass differs from denominator"));
        }
        let chosen = chosen.ok_or_else(|| invalid("source actions have no token candidate"))?;
        let chosen_token_id = chosen.token_id;
        let chosen_weight_q31 = chosen.weight_q31;
        Ok(ActionTrace {
            policy: POLICY,
            tokenizer_sha256: self.binding.tokenizer_sha256.clone(),
            protocol: self.binding.protocol.clone(),
            period_token_id: self.binding.period_token_id(),
            eos_token_id: self.binding.eos_token_id(),
            head_scores: head_scores
                .iter()
                .map(|head| HeadActionTrace {
                    copy_q24: head.copy_q24.to_vec(),
                    period_q24: head.period_q24,
                    stop_q24: head.stop_q24,
                })
                .collect(),
            actions,
            token_masses,
            max_score_q24: reduction.max_score_q24,
            total_weight_q31: reduction.total_weight_q31,
            chosen_token_id,
            chosen_weight_q31,
        })
    }
}

fn aggregate_tokens(actions: &[ActionMass]) -> Result<Vec<TokenMass>> {
    let mut tokens: Vec<TokenMass> = Vec::with_capacity(actions.len());
    for action in actions {
        if let Some(token) = tokens
            .iter_mut()
            .find(|token| token.token_id == action.token_id)
        {
            token.weight_q31 = token
                .weight_q31
                .checked_add(action.weight_q31)
                .ok_or_else(|| invalid("source action alias mass overflow"))?;
            token.action_offsets.push(action.action_offset);
        } else {
            tokens.push(TokenMass {
                token_id: action.token_id,
                weight_q31: action.weight_q31,
                action_offsets: vec![action.action_offset],
            });
        }
    }
    tokens.sort_unstable_by_key(|token| token.token_id);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::geometric_read::{EXP_TABLE_LEN, WEIGHT_ONE};

    const TOKENIZER: &[u8] = br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"},{"id":9,"content":"<gap>"}]}"#;

    // Exact arithmetic fixture, not an exponential-approximation claim.
    fn table() -> Vec<u32> {
        let mut table = vec![0; EXP_TABLE_LEN];
        table[0] = WEIGHT_ONE as u32;
        table[1] = (WEIGHT_ONE >> 1) as u32;
        table[2] = (WEIGHT_ONE >> 2) as u32;
        table
    }
    fn reducer(heads: usize) -> Result<NativeSourceActions> {
        NativeSourceActions::new(SourceActionBinding::new(TOKENIZER)?, heads, &table())
    }

    #[test]
    fn source_actions_sum_logits_before_joint_normalization() -> Result<()> {
        let mut read = reducer(2)?;
        let step = 1i64 << 16;
        let out = read.reduce(
            &[4],
            &[
                ActionHeadScores {
                    copy_q24: &[0],
                    period_q24: -step,
                    stop_q24: -step,
                },
                ActionHeadScores {
                    copy_q24: &[-step],
                    period_q24: 0,
                    stop_q24: -step,
                },
            ],
        )?;
        assert_eq!(
            out.actions.iter().map(|a| a.score_q24).collect::<Vec<_>>(),
            [-step, -step, -2 * step]
        );
        assert_eq!(
            out.actions.iter().map(|a| a.weight_q31).collect::<Vec<_>>(),
            [WEIGHT_ONE, WEIGHT_ONE, WEIGHT_ONE >> 1]
        );
        assert_eq!(out.total_weight_q31, 2 * WEIGHT_ONE + (WEIGHT_ONE >> 1));
        assert_eq!(out.max_score_q24, -step);
        assert_eq!(out.chosen_token_id, 3); // Equal copy/period mass; smaller ID wins.
                                            // Averaging separately normalized heads would give 3/8 copy mass,
                                            // not this common 2/5. No floating normalization is used in the fixture.
        assert_ne!(out.actions[0].weight_q31 * 8, out.total_weight_q31 * 3);
        Ok(())
    }

    #[test]
    fn source_actions_preserve_occurrences_and_aggregate_period_eos_aliases() -> Result<()> {
        let mut read = reducer(1)?;
        let out = read.reduce(
            &[4, 4, 3, 1],
            &[ActionHeadScores {
                copy_q24: &[0; 4],
                period_q24: 0,
                stop_q24: 0,
            }],
        )?;
        assert_eq!(out.actions.len(), 6);
        assert_eq!(
            out.actions[2].action,
            SourceAction::Copy { source_offset: 2 }
        );
        assert_eq!(out.actions[4].action, SourceAction::Period);
        assert_eq!(out.actions[5].action, SourceAction::Stop);
        assert_eq!(out.total_weight_q31, 6 * WEIGHT_ONE);
        assert_eq!(
            out.token_masses,
            vec![
                TokenMass {
                    token_id: 1,
                    weight_q31: 2 * WEIGHT_ONE,
                    action_offsets: vec![3, 5]
                },
                TokenMass {
                    token_id: 3,
                    weight_q31: 2 * WEIGHT_ONE,
                    action_offsets: vec![2, 4]
                },
                TokenMass {
                    token_id: 4,
                    weight_q31: 2 * WEIGHT_ONE,
                    action_offsets: vec![0, 1]
                },
            ]
        );
        assert_eq!(out.chosen_token_id, 1);
        assert_eq!(out.tokenizer_sha256, sha256_bytes(TOKENIZER));
        assert_eq!(out.protocol.eos_id, 1);
        assert_eq!(out.period_token_id, 3);
        Ok(())
    }

    #[test]
    fn source_actions_refuse_invalid_bindings_shapes_and_overflow() -> Result<()> {
        assert!(SourceActionBinding::new(b"{}").is_err());
        let mut no_eos: serde_json::Value = serde_json::from_slice(TOKENIZER)?;
        no_eos["added_tokens"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture added tokens"))?
            .remove(1);
        assert!(SourceActionBinding::new(&serde_json::to_vec(&no_eos)?).is_err());
        let no_period = std::str::from_utf8(TOKENIZER)
            .map_err(|e| invalid(e.to_string()))?
            .replace("\".\":3", "\",\":3");
        assert!(SourceActionBinding::new(no_period.as_bytes()).is_err());
        assert!(reducer(0).is_err());
        assert!(reducer(3).is_err());
        let mut read = reducer(2)?;
        let valid = [ActionHeadScores {
            copy_q24: &[0],
            period_q24: 0,
            stop_q24: 0,
        }; 2];
        let before = read.reduce(&[4], &valid)?;
        assert!(read.reduce(&[4], &valid[..1]).is_err());
        assert!(read.reduce(&[], &valid).is_err());
        assert!(read.reduce(&[6], &valid).is_err()); // Hole below vocab_size 10.
        assert!(read.reduce(&[10], &valid).is_err());
        assert!(read.reduce(&[4; 129], &valid).is_err());
        for slot in 0..3 {
            let mut first = [0; 3];
            let mut second = [0; 3];
            first[slot] = i64::MAX;
            second[slot] = 1;
            assert!(read
                .reduce(
                    &[4],
                    &[
                        ActionHeadScores {
                            copy_q24: &first[..1],
                            period_q24: first[1],
                            stop_q24: first[2]
                        },
                        ActionHeadScores {
                            copy_q24: &second[..1],
                            period_q24: second[1],
                            stop_q24: second[2]
                        },
                    ]
                )
                .is_err());
        }
        assert!(read
            .reduce(
                &[4],
                &[
                    ActionHeadScores {
                        copy_q24: &[i64::MIN],
                        period_q24: i64::MAX,
                        stop_q24: 0
                    },
                    valid[1],
                ]
            )
            .is_err()); // Maximum-minus-score is also a checked i64 boundary.
        assert_eq!(read.reduce(&[4], &valid)?, before);
        // Public bounds keep true alias totals below 130*2^31; exercise the
        // defensive check directly without inventing a public oversized mode.
        let mut aliases = before.actions[..2].to_vec();
        aliases[0].weight_q31 = u64::MAX;
        aliases[1].token_id = aliases[0].token_id;
        aliases[1].weight_q31 = 1;
        assert!(aggregate_tokens(&aliases).is_err());
        Ok(())
    }

    #[test]
    fn source_actions_bound_empty_full_and_extreme_equal_scores() -> Result<()> {
        let mut read = reducer(2)?;
        for extreme in [i64::MIN, i64::MAX] {
            let scores = [extreme; MAX_SOURCE_TOKENS];
            let zeros = [0; MAX_SOURCE_TOKENS];
            let out = read.reduce(
                &[4; MAX_SOURCE_TOKENS],
                &[
                    ActionHeadScores {
                        copy_q24: &scores,
                        period_q24: extreme,
                        stop_q24: extreme,
                    },
                    ActionHeadScores {
                        copy_q24: &zeros,
                        period_q24: 0,
                        stop_q24: 0,
                    },
                ],
            )?;
            assert_eq!(out.max_score_q24, extreme);
            assert_eq!(out.total_weight_q31, 130 * WEIGHT_ONE);
            assert_eq!(out.chosen_token_id, 4);
            assert_eq!(out.chosen_weight_q31, 128 * WEIGHT_ONE);
            assert_eq!(out.actions[128].action, SourceAction::Period);
            assert_eq!(out.actions[129].action, SourceAction::Stop);
        }
        let empty = read.reduce(
            &[],
            &[ActionHeadScores {
                copy_q24: &[],
                period_q24: 0,
                stop_q24: 0,
            }; 2],
        )?;
        assert_eq!(empty.actions.len(), 2);
        assert_eq!(empty.total_weight_q31, 2 * WEIGHT_ONE);
        assert_eq!(empty.chosen_token_id, 1);
        // No parent distribution or target enters any constructor/reduce API.
        Ok(())
    }
}
