//! A learned grounded lexical realization policy for one owned answer.
//!
//! The retained scoped session could only copy an owned payload verbatim. A scalar copy boost changes
//! one token's logit and therefore cannot change the odds between two *uncopied* tokens; a learned
//! gate mixing a frozen local donor with a pure pointer has the same limitation outside pointer
//! support. This module supplies the missing interaction: a compact, finite, context-indexed table
//! that chooses, at every emission position, between
//!
//! * `Copy` the next exact token of the owned payload (or of the grounded derived result),
//! * `Insert` one learned vocabulary word from a shared small slot set, and
//! * `Stop`.
//!
//! The table is indexed by *causal* session state only: the observed relation, the requested history
//! position, whether the answer is a derived (computed) result, whether the address has a superseded
//! older committed value, the copy stage and
//! a bounded count of already-emitted vocabulary words. Correctness, expected payload, evaluator
//! family, target token, required answer length and gold route are never inputs.
//!
//! Because an owned-evidence feature participates in the key, changing older eligible evidence (or a
//! derived-result provenance flag) can change an *uncopied* lexical choice while the recent request and the
//! generated prefix up to that decision are held fixed. That is the property a scalar copy boost
//! cannot express. The payload class and emitted token identities do not index this prototype;
//! it demonstrates a flag-conditioned action table, not content-sensitive lexical meaning.
//!
//! Served execution is a keyed row lookup plus an integer argmax: table reads, additions,
//! comparisons, shifts and rotations, with no multiplier instruction and no floating point on the
//! served path. Fitting is integer counting over observed development text and is explicitly
//! offline.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Artifact format version.
pub const LR_VERSION: u8 = 1;
/// Declared maximum number of shared learned vocabulary slots.
pub const LR_MAX_SLOTS: usize = 8;
/// Declared maximum number of evidence classes.
pub const LR_MAX_EVIDENCE_BUCKETS: u16 = 64;

/// Row index of the `Copy` action.
pub const LR_COPY: usize = 0;
/// Row index of the `Stop` action.
pub const LR_STOP: usize = 1;
/// Row index of the first `Insert` action; slot `s` is `LR_INSERT_BASE + s`.
pub const LR_INSERT_BASE: usize = 2;

/// Salt separating this key space from any other.
const LR_KEY_SALT: u64 = 0x9e37_79b9_7f4a_7c15;

/// The causal, target-free context that indexes the realization table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealizationContext {
    /// The observed relation byte of the address.
    pub relation: u8,
    /// Requested history: 0 current, 1 previous assertion, 2 previous distinct, 3 initial.
    pub history: u8,
    /// The answer value came from consuming a computed result.
    pub derived: bool,
    /// An older committed value for the same address was superseded before this one.
    pub prior_differs: bool,
    /// Reported bounded class of the owned payload; deliberately omitted from the table key.
    pub evidence_class: u16,
    /// 0 before the payload, 1 inside it, 2 once it is complete.
    pub copy_stage: u8,
    /// `min(emitted vocabulary words, 3)`.
    pub emitted_bucket: u8,
}

/// One realized emission action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealizationAction {
    /// Emit the next exact token of the owned payload.
    Copy,
    /// Emit one learned vocabulary word from the named shared slot.
    Insert(u8),
    /// End the realized response.
    Stop,
}

/// The observed decision of one realized emission step, reported for measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizationDecision {
    pub action: RealizationAction,
    /// Whether the reported action was the table's own choice (`false` when an owned structural
    /// invariant overrode it).
    pub from_table: bool,
    /// The context key the decision used.
    pub key: u64,
    /// The bounded geometric class of the owned evidence at this step. Reported only; it is not a
    /// key field, so a decision can be attributed to the causal flags rather than to the class.
    pub evidence_class: u16,
}

/// A fitted realization policy: one compact finite context-indexed table over a shared slot set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealizationModel {
    pub version: u8,
    /// Lexical vocabulary the slot tokens must belong to.
    pub vocab: usize,
    /// Learned shared insert vocabulary, one token per slot.
    pub slots: Vec<u32>,
    /// Number of evidence classes used when building a key.
    pub evidence_buckets: u16,
    /// `(key, scores)` rows sorted by key. Row `i` of a score vector is action index `i`.
    pub table: Vec<(u64, Vec<i32>)>,
}

impl RealizationModel {
    /// Number of actions scored by every row.
    pub fn n_actions(&self) -> usize {
        LR_INSERT_BASE + self.slots.len()
    }

    /// The context key.
    ///
    /// Every input is causal session state; no target or evaluator field participates. The bounded
    /// geometric evidence class is deliberately *not* a key field: an exact-key lookup cannot
    /// generalise to a class it never observed, so a class term would replace a causal effect with a
    /// memorised per-value mapping. The class is still computed and reported alongside every decision
    /// so the geometric feature is measured rather than assumed.
    pub fn context_key(&self, ctx: &RealizationContext) -> u64 {
        let mut h = LR_KEY_SALT;
        // Multiply-free mixing: shifts, rotations, xors and additions only, so the served key
        // derivation cannot emit a multiplier instruction either.
        let mut mix = |v: u64| {
            h ^= v.wrapping_add(LR_KEY_SALT);
            h = h.wrapping_add(h << 13);
            h ^= h >> 7;
            h = h.wrapping_add(h << 17);
            h = h.rotate_left(29);
        };
        mix(u64::from(ctx.relation));
        mix(u64::from(ctx.history));
        mix(u64::from(ctx.derived));
        mix(u64::from(ctx.prior_differs));
        mix(u64::from(ctx.copy_stage));
        mix(u64::from(ctx.emitted_bucket));
        h
    }

    /// The learned score row for a context, if the table was fitted for that key.
    pub fn row(&self, key: u64) -> Option<&[i32]> {
        self.table
            .binary_search_by_key(&key, |(k, _)| *k)
            .ok()
            .map(|i| self.table[i].1.as_slice())
    }

    /// The table's own decision for a context, before any owned structural invariant.
    ///
    /// A key absent from the fitted table falls back to the declared default policy: complete the
    /// exact span, then stop. This is a stated default, not a learned behaviour.
    pub fn decide(&self, ctx: &RealizationContext) -> RealizationDecision {
        let key = self.context_key(ctx);
        let action = match self.row(key) {
            Some(scores) => {
                let mut best = LR_COPY;
                let mut best_score = i32::MIN;
                for (i, score) in scores.iter().enumerate() {
                    // Ties prefer the lower action index: Copy, then Stop, then inserts.
                    if *score > best_score {
                        best_score = *score;
                        best = i;
                    }
                }
                match best {
                    LR_COPY => RealizationAction::Copy,
                    LR_STOP => RealizationAction::Stop,
                    _ => RealizationAction::Insert((best - LR_INSERT_BASE) as u8),
                }
            }
            None => {
                if ctx.copy_stage < 2 {
                    RealizationAction::Copy
                } else {
                    RealizationAction::Stop
                }
            }
        };
        RealizationDecision {
            action,
            from_table: self.row(key).is_some(),
            key,
            evidence_class: ctx.evidence_class,
        }
    }

    /// Structural validation of a loaded or fitted artifact.
    pub fn validate(&self, max_vocab: usize) -> Result<(), String> {
        if self.version != LR_VERSION {
            return Err("unsupported realization artifact version".into());
        }
        if self.vocab == 0 || self.vocab > max_vocab {
            return Err("realization artifact vocabulary is out of range".into());
        }
        if self.slots.len() > LR_MAX_SLOTS {
            return Err("realization slot count exceeds the declared bound".into());
        }
        if self.evidence_buckets == 0
            || self.evidence_buckets > LR_MAX_EVIDENCE_BUCKETS
            || !self.evidence_buckets.is_power_of_two()
        {
            return Err("realization evidence bucket count must be a power of two in range".into());
        }
        if self.slots.iter().any(|t| *t as usize >= self.vocab) {
            return Err("realization slot token is outside the vocabulary".into());
        }
        let n = self.n_actions();
        let mut previous: Option<u64> = None;
        for (key, scores) in &self.table {
            if scores.len() != n {
                return Err("realization row width disagrees with the slot set".into());
            }
            if previous.is_some_and(|p| p >= *key) {
                return Err("realization table is not strictly sorted by key".into());
            }
            previous = Some(*key);
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }

    pub fn from_bytes(bytes: &[u8], max_vocab: usize) -> Result<Self, String> {
        let model: RealizationModel =
            serde_json::from_slice(bytes).map_err(|e| format!("realization artifact: {e}"))?;
        model.validate(max_vocab)?;
        Ok(model)
    }
}

/// One observed development example: a causal context and the realized action actually taken by the
/// declared development response at that position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RealizationExample {
    pub context: RealizationContext,
    pub action: RealizationAction,
}

/// Fit the compact table by integer counting over observed development examples.
///
/// Each example contributes one unit to its `(key, action)` cell. A row is emitted only for keys the
/// development data actually observed, so an unfitted context uses the declared default policy. The
/// action with the most observed units wins a row; ties fall to the lower action index.
pub fn fit_realization(
    examples: &[RealizationExample],
    vocab: usize,
    slots: Vec<u32>,
    evidence_buckets: u16,
) -> Result<RealizationModel, String> {
    let n = LR_INSERT_BASE + slots.len();
    let mut model = RealizationModel {
        version: LR_VERSION,
        vocab,
        slots,
        evidence_buckets,
        table: Vec::new(),
    };
    let mut counts: std::collections::BTreeMap<u64, Vec<i64>> = std::collections::BTreeMap::new();
    for example in examples {
        let key = model.context_key(&example.context);
        let index = match example.action {
            RealizationAction::Copy => LR_COPY,
            RealizationAction::Stop => LR_STOP,
            RealizationAction::Insert(slot) => {
                if slot as usize >= model.slots.len() {
                    return Err("development example names an undeclared slot".into());
                }
                LR_INSERT_BASE + slot as usize
            }
        };
        let row = counts.entry(key).or_insert_with(|| vec![0i64; n]);
        row[index] += 1;
    }
    model.table = counts
        .into_iter()
        .map(|(key, counts)| {
            let peak = counts.iter().copied().max().unwrap_or(0).max(1);
            let scores = counts
                .into_iter()
                .map(|c| ((c * 1000) / peak).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32)
                .collect();
            (key, scores)
        })
        .collect();
    model.validate(vocab)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(evidence: u16, prior: bool, copy_stage: u8, emitted: u8) -> RealizationContext {
        RealizationContext {
            relation: 0,
            history: 0,
            derived: false,
            prior_differs: prior,
            evidence_class: evidence,
            copy_stage,
            emitted_bucket: emitted,
        }
    }

    #[test]
    fn the_key_separates_older_evidence_with_everything_else_fixed() {
        let model = fit_realization(
            &[
                RealizationExample {
                    context: ctx(1, false, 0, 0),
                    action: RealizationAction::Insert(0),
                },
                RealizationExample {
                    context: ctx(1, true, 0, 0),
                    action: RealizationAction::Insert(1),
                },
            ],
            64,
            vec![7, 9],
            8,
        )
        .unwrap();
        // Identical prompt, identical empty generated prefix: only the older evidence flag differs.
        assert_eq!(
            model.decide(&ctx(1, false, 0, 0)).action,
            RealizationAction::Insert(0)
        );
        assert_eq!(
            model.decide(&ctx(1, true, 0, 0)).action,
            RealizationAction::Insert(1)
        );
    }

    #[test]
    fn the_key_separates_a_derived_result_from_a_direct_read() {
        let model = fit_realization(
            &[
                RealizationExample {
                    context: ctx(1, false, 0, 0),
                    action: RealizationAction::Insert(0),
                },
                RealizationExample {
                    context: RealizationContext {
                        derived: true,
                        ..ctx(1, false, 0, 0)
                    },
                    action: RealizationAction::Insert(1),
                },
            ],
            64,
            vec![7, 9],
            8,
        )
        .unwrap();
        let direct = model.decide(&ctx(1, false, 0, 0));
        let derived = model.decide(&RealizationContext {
            derived: true,
            ..ctx(1, false, 0, 0)
        });
        assert_ne!(direct.action, derived.action);
    }

    #[test]
    fn an_unseen_evidence_class_still_selects_the_learned_uncopied_word() {
        // The class is reported, not keyed, so a fresh evidence value generalises to the learned
        // causal rule instead of falling back to the default policy.
        let model = fit_realization(
            &[RealizationExample {
                context: ctx(2, true, 0, 0),
                action: RealizationAction::Insert(1),
            }],
            64,
            vec![7, 9],
            8,
        )
        .unwrap();
        let fresh = model.decide(&ctx(37, true, 0, 0));
        assert_eq!(fresh.action, RealizationAction::Insert(1));
        assert!(fresh.from_table);
        assert_eq!(fresh.evidence_class, 37);
    }

    #[test]
    fn an_unfitted_context_uses_the_declared_default_policy() {
        let model = fit_realization(&[], 64, vec![3], 8).unwrap();
        assert_eq!(model.table.len(), 0);
        assert_eq!(
            model.decide(&ctx(0, false, 0, 0)).action,
            RealizationAction::Copy
        );
        assert_eq!(
            model.decide(&ctx(0, false, 1, 0)).action,
            RealizationAction::Copy
        );
        assert_eq!(
            model.decide(&ctx(0, false, 2, 0)).action,
            RealizationAction::Stop
        );
        assert!(!model.decide(&ctx(0, false, 0, 0)).from_table);
    }

    #[test]
    fn the_table_round_trips_and_rejects_a_wide_or_unsorted_row() {
        let model = fit_realization(
            &[RealizationExample {
                context: ctx(1, false, 0, 0),
                action: RealizationAction::Copy,
            }],
            64,
            vec![5, 6],
            8,
        )
        .unwrap();
        let bytes = model.to_bytes().unwrap();
        assert_eq!(RealizationModel::from_bytes(&bytes, 64).unwrap(), model);

        let mut wide = model.clone();
        wide.table[0].1.push(0);
        assert!(wide.validate(64).is_err());

        let mut unsorted = model;
        unsorted.table.push((0, vec![0, 0, 0, 0]));
        assert!(unsorted.validate(64).is_err());
    }
}
