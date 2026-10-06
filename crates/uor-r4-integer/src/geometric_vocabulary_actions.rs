//! Opt-in full-vocabulary Generate plus exact source-occurrence Copy reduction.
//! Scores are already summed across heads. One common final score clip bounds
//! every admitted action to +/-8 Q24 nats; the authenticated exponential table
//! therefore supplies strictly positive unnormalized masses for all legal IDs.
//! No Null/Period/Stop row is inserted: EOS and period each have one Generate
//! row, and ordinary Copy aliases are summed into the same emitted-token mass.
//! Construction and owned diagnostic tracing allocate. Successful reduce_into
//! uses preallocated scratch and caller output buffers, with no float arithmetic.
//! Raw-score ranking is an explicit tail-limited diagnostic, not the serving rule.
use crate::geometric_read::{EXP_STEP_LOG2, EXP_TAIL_Q24};
use crate::geometric_source_actions::{SourceActionBinding, MAX_SOURCE_TOKENS};
use crate::geometric_source_realizer::{canonical_exp, sha256_bytes};
use crate::stack::kernels::stack_exp_neg;
use serde::Serialize;
use std::fmt;

pub const MAX_VOCAB: usize = 4096;
pub const SCORE_CLIP_Q24: i64 = 8 << 24;
pub const POLICY: &str = "full-legal-vocabulary-Generate+occurrence-Copy/1;sum-head-inputs;common-final-clip+/-8Q24;canonical-positive-gap16-exp;exact-u64-unnormalized-token-aliases;smallest-token-ID-ties;no-extra-terminal-or-null";
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VocabularyActionError {
    InvalidVocabulary(usize),
    ExpAdmission(String),
    MissingLegalTerminal,
    CopyCount(usize),
    ScoreShape,
    OutputShape,
    InvalidCopyToken(u32),
    InvalidGenerateToken(u32),
    InvalidTargetToken(u32),
    CacheBindingMismatch,
    Overflow,
    NonpositiveWeight,
}
impl fmt::Display for VocabularyActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vocabulary actions: {self:?}")
    }
}
impl std::error::Error for VocabularyActionError {}
pub type Result<T> = std::result::Result<T, VocabularyActionError>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VocabularyReduction {
    pub legal_generate_actions: usize,
    pub copy_actions: usize,
    pub max_score_q24: i64,
    pub total_weight_q31: u64,
    pub chosen_token_id: u32,
    pub chosen_weight_q31: u64,
    pub generate_weight_q31: u64,
    pub copy_weight_q31: u64,
    pub chosen_generate_weight_q31: u64,
    pub chosen_copy_weight_q31: u64,
    pub clipped_low_actions: usize,
    pub clipped_high_actions: usize,
    pub raw_max_score_q24: i64,
    pub raw_total_weight_q31: u64,
    pub raw_chosen_token_id: u32,
    pub raw_chosen_weight_q31: u64,
    /// One plus number of legal tokens with strictly greater raw diagnostic mass.
    pub chosen_raw_mass_rank: usize,
    pub raw_chosen_clipped_mass_rank: usize,
    pub token_winner_changed_by_clip: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum VocabularyAction {
    Generate { token_id: u32 },
    Copy { source_offset: usize },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VocabularyActionMass {
    pub action: VocabularyAction,
    pub action_offset: usize,
    pub token_id: u32,
    pub raw_score_q24: i64,
    pub score_q24: i64,
    pub weight_q31: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VocabularyTokenMass {
    pub token_id: u32,
    pub weight_q31: u64,
    pub generate_weight_q31: u64,
    pub copy_weight_q31: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VocabularyActionTrace {
    pub policy: &'static str,
    pub tokenizer_sha256: String,
    pub period_token_id: u32,
    pub eos_token_id: u32,
    pub summary: VocabularyReduction,
    pub actions: Vec<VocabularyActionMass>,
    pub token_masses: Vec<VocabularyTokenMass>,
}
/// Offline fixed-position single-Generate-atom cache. Fields are opaque and
/// only an admitted reducer can prepare one; there is no mass/reference loader.
/// Owned input scores are the immutable incumbent across candidate evaluations.
/// Construction allocates; this is not a serving-path allocation claim.
pub struct GenerateSubstitutionCache {
    tokenizer_sha256: String,
    exp_sha256: String,
    generate_q24: Vec<i64>,
    copy_ids: Vec<u32>,
    copy_q24: Vec<i64>,
    selected: usize,
    gold: usize,
    old_weight_q31: u64,
    maximum_excluding_selected: i64,
    baseline: GenerateSubstitutionMass,
}

/// Exact served masses only: raw diagnostics and winner fields are deliberately
/// absent because their reference can change even on the clipped fast path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GenerateSubstitutionMass {
    pub total_weight_q31: u64,
    pub target_weight_q31: u64,
    pub used_full_reduction: bool,
    pub reference_q24: i64,
}
impl GenerateSubstitutionCache {
    pub fn incumbent_mass(&self) -> GenerateSubstitutionMass {
        self.baseline
    }
}

pub struct NativeVocabularyActions {
    binding: SourceActionBinding,
    legal_ids: Box<[u32]>,
    legal_mask: Box<[bool]>,
    exp: Box<[u32]>,
    exp_sha256: String,
    raw_masses: Box<[u64]>,
    generate_masses: Box<[u64]>,
    trace_weights: Box<[u64]>,
    trace_masses: Box<[u64]>,
}
impl NativeVocabularyActions {
    /// Authenticates the complete pinned table bytes, not only shape or order.
    pub fn new(binding: SourceActionBinding, exp_bytes: &[u8]) -> Result<Self> {
        let exp = canonical_exp(exp_bytes)
            .map_err(|e| VocabularyActionError::ExpAdmission(e.to_string()))?;
        Self::from_admitted(binding, exp)
    }
    fn from_admitted(binding: SourceActionBinding, exp: Vec<u32>) -> Result<Self> {
        let vocab = binding.vocab_size();
        if vocab == 0 || vocab > MAX_VOCAB {
            return Err(VocabularyActionError::InvalidVocabulary(vocab));
        }
        let legal_ids = (0..vocab)
            .filter(|&id| binding.validate_tokens(&[id as u32]).is_ok())
            .map(|id| id as u32)
            .collect::<Vec<_>>();
        let mut legal_mask = vec![false; vocab];
        for &id in &legal_ids {
            legal_mask[id as usize] = true;
        }
        if !legal_mask
            .get(binding.eos_token_id() as usize)
            .copied()
            .unwrap_or(false)
            || !legal_mask
                .get(binding.period_token_id() as usize)
                .copied()
                .unwrap_or(false)
        {
            return Err(VocabularyActionError::MissingLegalTerminal);
        }
        if stack_exp_neg(SCORE_CLIP_Q24 + SCORE_CLIP_Q24, -24, &exp, EXP_STEP_LOG2) == 0 {
            return Err(VocabularyActionError::NonpositiveWeight);
        }
        let capacity = legal_ids.len() + MAX_SOURCE_TOKENS;
        // Admission-only identity, never re-hashed in the numerical reducer or
        // per-candidate path. Public constructor authenticates canonical bytes.
        let exp_bytes = exp.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>();
        let exp_sha256 = sha256_bytes(&exp_bytes);
        Ok(Self {
            binding,
            legal_ids: legal_ids.into_boxed_slice(),
            legal_mask: legal_mask.into_boxed_slice(),
            exp: exp.into_boxed_slice(),
            exp_sha256,
            raw_masses: vec![0; vocab].into_boxed_slice(),
            generate_masses: vec![0; vocab].into_boxed_slice(),
            trace_weights: vec![0; capacity].into_boxed_slice(),
            trace_masses: vec![0; vocab].into_boxed_slice(),
        })
    }
    pub fn binding(&self) -> &SourceActionBinding {
        &self.binding
    }
    pub fn vocab_size(&self) -> usize {
        self.legal_mask.len()
    }
    pub fn legal_token_ids(&self) -> &[u32] {
        &self.legal_ids
    }
    pub fn action_count(&self, copies: usize) -> Result<usize> {
        if copies > MAX_SOURCE_TOKENS {
            return Err(VocabularyActionError::CopyCount(copies));
        }
        Ok(self.legal_ids.len() + copies)
    }
    fn validate(
        &self,
        gen: &[i64],
        ids: &[u32],
        copy: &[i64],
        weights: &[u64],
        masses: &[u64],
    ) -> Result<()> {
        if ids.len() > MAX_SOURCE_TOKENS {
            return Err(VocabularyActionError::CopyCount(ids.len()));
        }
        if gen.len() != self.vocab_size() || copy.len() != ids.len() {
            return Err(VocabularyActionError::ScoreShape);
        }
        if weights.len() != self.legal_ids.len() + ids.len() || masses.len() != self.vocab_size() {
            return Err(VocabularyActionError::OutputShape);
        }
        for &id in ids {
            if !self.legal_mask.get(id as usize).copied().unwrap_or(false) {
                return Err(VocabularyActionError::InvalidCopyToken(id));
            }
        }
        Ok(())
    }
    /// Output rows: ascending legal Generate IDs followed by every Copy in caller
    /// occurrence order. Holes in vocabulary-indexed scores are ignored and their
    /// output masses are zero. Invalid input leaves caller outputs unchanged.
    /// On arithmetic failure outputs can be partial; no valid summary is returned.
    pub fn reduce_into(
        &mut self,
        generate_q24: &[i64],
        copy_ids: &[u32],
        copy_q24: &[i64],
        action_weights: &mut [u64],
        token_masses: &mut [u64],
    ) -> Result<VocabularyReduction> {
        self.validate(
            generate_q24,
            copy_ids,
            copy_q24,
            action_weights,
            token_masses,
        )?;
        self.reduce_validated(
            generate_q24,
            copy_ids,
            copy_q24,
            action_weights,
            token_masses,
        )
    }
    /// Prepare an offline exact substitution cache from owned factual scores.
    /// All admission, baseline masses and reference are recomputed here.
    pub fn prepare_generate_substitution(
        &mut self,
        gen: Vec<i64>,
        copy_ids: Vec<u32>,
        copy_scores: Vec<i64>,
        selected_token: u32,
        gold_token: u32,
    ) -> Result<GenerateSubstitutionCache> {
        if !self
            .legal_mask
            .get(selected_token as usize)
            .copied()
            .unwrap_or(false)
        {
            return Err(VocabularyActionError::InvalidGenerateToken(selected_token));
        }
        if !self
            .legal_mask
            .get(gold_token as usize)
            .copied()
            .unwrap_or(false)
        {
            return Err(VocabularyActionError::InvalidTargetToken(gold_token));
        }
        let count = self.action_count(copy_ids.len())?;
        let selected_offset = self
            .legal_ids
            .iter()
            .position(|&id| id == selected_token)
            .ok_or(VocabularyActionError::InvalidGenerateToken(selected_token))?;
        // Scratch belongs to the admitted reducer, not every position cache.
        let mut weights = std::mem::take(&mut self.trace_weights);
        let mut masses = std::mem::take(&mut self.trace_masses);
        let result = self.reduce_into(
            &gen,
            &copy_ids,
            &copy_scores,
            &mut weights[..count],
            &mut masses,
        );
        let old_weight = weights[selected_offset];
        let target_weight = masses[gold_token as usize];
        self.trace_weights = weights;
        self.trace_masses = masses;
        let summary = result?;
        let maximum_excluding_selected = self
            .legal_ids
            .iter()
            .filter(|&&id| id != selected_token)
            .map(|&id| gen[id as usize].clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24))
            .chain(
                copy_scores
                    .iter()
                    .map(|&s| s.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24)),
            )
            .max()
            .ok_or(VocabularyActionError::MissingLegalTerminal)?;
        Ok(GenerateSubstitutionCache {
            tokenizer_sha256: self.binding.tokenizer_sha256().to_owned(),
            exp_sha256: self.exp_sha256.clone(),
            selected: selected_token as usize,
            gold: gold_token as usize,
            old_weight_q31: old_weight,
            maximum_excluding_selected,
            baseline: GenerateSubstitutionMass {
                total_weight_q31: summary.total_weight_q31,
                target_weight_q31: target_weight,
                reference_q24: summary.max_score_q24,
                used_full_reduction: false,
            },
            generate_q24: gen,
            copy_ids,
            copy_q24: copy_scores,
        })
    }

    /// Evaluate one candidate against the unchanged cache incumbent. Every
    /// position remains mandatory; target absence does not prune denominators.
    /// Changed clipped reference invokes the existing authoritative reducer.
    pub fn evaluate_generate_substitution(
        &mut self,
        cache: &mut GenerateSubstitutionCache,
        new_raw_score_q24: i64,
    ) -> Result<GenerateSubstitutionMass> {
        if cache.tokenizer_sha256 != self.binding.tokenizer_sha256()
            || cache.exp_sha256 != self.exp_sha256
        {
            return Err(VocabularyActionError::CacheBindingMismatch);
        }
        let clipped = new_raw_score_q24.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24);
        let reference = cache.maximum_excluding_selected.max(clipped);
        if reference == cache.baseline.reference_q24 {
            let gap = reference
                .checked_sub(clipped)
                .ok_or(VocabularyActionError::Overflow)?;
            let weight = stack_exp_neg(gap, -24, &self.exp, EXP_STEP_LOG2);
            if weight == 0 {
                return Err(VocabularyActionError::NonpositiveWeight);
            }
            let replace = |value: u64| {
                value
                    .checked_sub(cache.old_weight_q31)
                    .and_then(|v| v.checked_add(weight))
                    .ok_or(VocabularyActionError::Overflow)
            };
            let total = replace(cache.baseline.total_weight_q31)?;
            let target = if cache.selected == cache.gold {
                replace(cache.baseline.target_weight_q31)?
            } else {
                cache.baseline.target_weight_q31
            };
            return Ok(GenerateSubstitutionMass {
                total_weight_q31: total,
                target_weight_q31: target,
                reference_q24: reference,
                used_full_reduction: false,
            });
        }
        let count = self.action_count(cache.copy_ids.len())?;
        let mut weights = std::mem::take(&mut self.trace_weights);
        let mut masses = std::mem::take(&mut self.trace_masses);
        let old = cache.generate_q24[cache.selected];
        cache.generate_q24[cache.selected] = new_raw_score_q24;
        let result = self.reduce_into(
            &cache.generate_q24,
            &cache.copy_ids,
            &cache.copy_q24,
            &mut weights[..count],
            &mut masses,
        );
        let target = masses[cache.gold];
        // Restore input and scratch even on an error. A candidate never
        // advances the incumbent or owns a full-vocabulary scratch allocation.
        cache.generate_q24[cache.selected] = old;
        self.trace_weights = weights;
        self.trace_masses = masses;
        let summary = result?;
        Ok(GenerateSubstitutionMass {
            total_weight_q31: summary.total_weight_q31,
            target_weight_q31: target,
            reference_q24: summary.max_score_q24,
            used_full_reduction: true,
        })
    }

    fn reduce_validated(
        &mut self,
        gen: &[i64],
        ids: &[u32],
        copy: &[i64],
        weights: &mut [u64],
        masses: &mut [u64],
    ) -> Result<VocabularyReduction> {
        let raw_max = self
            .legal_ids
            .iter()
            .map(|&id| gen[id as usize])
            .chain(copy.iter().copied())
            .max()
            .ok_or(VocabularyActionError::MissingLegalTerminal)?;
        let max = raw_max.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24);
        masses.fill(0);
        self.raw_masses.fill(0);
        self.generate_masses.fill(0);
        let mut total = 0u64;
        let mut raw_total = 0u64;
        let mut gen_total = 0u64;
        let mut copy_total = 0u64;
        let mut low = 0;
        let mut high = 0;
        for offset in 0..weights.len() {
            let is_gen = offset < self.legal_ids.len();
            let (id, raw) = if is_gen {
                let id = self.legal_ids[offset];
                (id, gen[id as usize])
            } else {
                let i = offset - self.legal_ids.len();
                (ids[i], copy[i])
            };
            let score = raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24);
            low += usize::from(raw < -SCORE_CLIP_Q24);
            high += usize::from(raw > SCORE_CLIP_Q24);
            let weight = stack_exp_neg(max - score, -24, &self.exp, EXP_STEP_LOG2);
            if weight == 0 {
                return Err(VocabularyActionError::NonpositiveWeight);
            }
            weights[offset] = weight;
            total = total
                .checked_add(weight)
                .ok_or(VocabularyActionError::Overflow)?;
            masses[id as usize] = masses[id as usize]
                .checked_add(weight)
                .ok_or(VocabularyActionError::Overflow)?;
            if is_gen {
                gen_total = gen_total
                    .checked_add(weight)
                    .ok_or(VocabularyActionError::Overflow)?;
                self.generate_masses[id as usize] = weight;
            } else {
                copy_total = copy_total
                    .checked_add(weight)
                    .ok_or(VocabularyActionError::Overflow)?;
            }
            let gap = raw_max.saturating_sub(raw);
            let raw_weight = if gap >= EXP_TAIL_Q24 {
                0
            } else {
                stack_exp_neg(gap, -24, &self.exp, EXP_STEP_LOG2)
            };
            raw_total = raw_total
                .checked_add(raw_weight)
                .ok_or(VocabularyActionError::Overflow)?;
            self.raw_masses[id as usize] = self.raw_masses[id as usize]
                .checked_add(raw_weight)
                .ok_or(VocabularyActionError::Overflow)?;
        }
        let mut chosen = self.legal_ids[0];
        let mut raw_chosen = chosen;
        for &id in self.legal_ids.iter().skip(1) {
            if masses[id as usize] > masses[chosen as usize] {
                chosen = id;
            }
            if self.raw_masses[id as usize] > self.raw_masses[raw_chosen as usize] {
                raw_chosen = id;
            }
        }
        let chosen_mass = masses[chosen as usize];
        let chosen_gen = self.generate_masses[chosen as usize];
        Ok(VocabularyReduction {
            legal_generate_actions: self.legal_ids.len(),
            copy_actions: ids.len(),
            max_score_q24: max,
            total_weight_q31: total,
            chosen_token_id: chosen,
            chosen_weight_q31: chosen_mass,
            generate_weight_q31: gen_total,
            copy_weight_q31: copy_total,
            chosen_generate_weight_q31: chosen_gen,
            chosen_copy_weight_q31: chosen_mass - chosen_gen,
            clipped_low_actions: low,
            clipped_high_actions: high,
            raw_max_score_q24: raw_max,
            raw_total_weight_q31: raw_total,
            raw_chosen_token_id: raw_chosen,
            raw_chosen_weight_q31: self.raw_masses[raw_chosen as usize],
            chosen_raw_mass_rank: 1 + self
                .legal_ids
                .iter()
                .filter(|&&id| self.raw_masses[id as usize] > self.raw_masses[chosen as usize])
                .count(),
            raw_chosen_clipped_mass_rank: 1 + self
                .legal_ids
                .iter()
                .filter(|&&id| masses[id as usize] > masses[raw_chosen as usize])
                .count(),
            token_winner_changed_by_clip: chosen != raw_chosen,
        })
    }
    /// Allocating evidence adapter. The numerical reduce_into path is unchanged.
    pub fn reduce_trace(
        &mut self,
        gen: &[i64],
        ids: &[u32],
        copy: &[i64],
    ) -> Result<VocabularyActionTrace> {
        let n = self.action_count(ids.len())?;
        // Move preallocated buffers out temporarily to permit exclusive reducer
        // access without aliasing its internal fields; no scratch is constructed.
        let mut weights = std::mem::take(&mut self.trace_weights);
        let mut masses = std::mem::take(&mut self.trace_masses);
        let result = self.reduce_into(gen, ids, copy, &mut weights[..n], &mut masses);
        let trace = result.map(|summary| {
            let actions = (0..n)
                .map(|offset| {
                    let (action, id, raw) = if offset < self.legal_ids.len() {
                        let id = self.legal_ids[offset];
                        (
                            VocabularyAction::Generate { token_id: id },
                            id,
                            gen[id as usize],
                        )
                    } else {
                        let i = offset - self.legal_ids.len();
                        (VocabularyAction::Copy { source_offset: i }, ids[i], copy[i])
                    };
                    VocabularyActionMass {
                        action,
                        action_offset: offset,
                        token_id: id,
                        raw_score_q24: raw,
                        score_q24: raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24),
                        weight_q31: weights[offset],
                    }
                })
                .collect();
            let token_masses = self
                .legal_ids
                .iter()
                .map(|&id| VocabularyTokenMass {
                    token_id: id,
                    weight_q31: masses[id as usize],
                    generate_weight_q31: self.generate_masses[id as usize],
                    copy_weight_q31: masses[id as usize] - self.generate_masses[id as usize],
                })
                .collect();
            VocabularyActionTrace {
                policy: POLICY,
                tokenizer_sha256: self.binding.tokenizer_sha256().into(),
                period_token_id: self.binding.period_token_id(),
                eos_token_id: self.binding.eos_token_id(),
                summary,
                actions,
                token_masses,
            }
        });
        self.trace_weights = weights;
        self.trace_masses = masses;
        trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binding(
        vocab: usize,
        sparse: bool,
    ) -> std::result::Result<SourceActionBinding, Box<dyn std::error::Error>> {
        let mut map = serde_json::Map::new();
        let mut added = vec![
            serde_json::json!({"id":0,"content":"<|bos|>"}),
            serde_json::json!({"id":1,"content":"<|eos|>"}),
            serde_json::json!({"id":2,"content":"<|unk|>"}),
        ];
        for i in 0..vocab {
            if sparse && i == 7 {
                continue;
            }
            if sparse && i > 7 {
                // The BPE model vocabulary itself must be dense; sparse IDs
                // arise from added tokens beyond that model vocabulary.
                added.push(serde_json::json!({"id":i,"content":format!("x{i}")}));
                continue;
            }
            let name = match i {
                0 => "<|bos|>".to_string(),
                1 => "<|eos|>".to_string(),
                2 => "<|unk|>".to_string(),
                3 => ".".to_string(),
                _ => format!("x{i}"),
            };
            map.insert(name, serde_json::json!(i));
        }
        let bytes = serde_json::to_vec(
            &serde_json::json!({"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":map,"merges":[]},"added_tokens":added}),
        )?;
        Ok(SourceActionBinding::new(&bytes)?)
    }
    // Arithmetic-only synthetic monotone table. Public admission rejects these
    // bytes; canonical artifact authentication is separately tested below.
    fn fixture(
        vocab: usize,
        sparse: bool,
    ) -> std::result::Result<NativeVocabularyActions, Box<dyn std::error::Error>> {
        let mut table = (0..crate::geometric_read::EXP_TABLE_LEN)
            .map(|i| (1u32 << 31).checked_shr((i >> 8) as u32).unwrap_or(0))
            .collect::<Vec<_>>();
        if let Some(last) = table.last_mut() {
            *last = 0;
        }
        Ok(NativeVocabularyActions::from_admitted(
            binding(vocab, sparse)?,
            table,
        )?)
    }
    #[test]
    fn cached_single_generate_substitution_matches_full_reducer_all_references_and_aliases(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let candidates = [
            i64::MIN,
            -SCORE_CLIP_Q24 - 1,
            -SCORE_CLIP_Q24,
            -(3 << 24),
            -65_537,
            -65_536,
            -1,
            0,
            1,
            65_535,
            65_536,
            65_537,
            2 << 24,
            5 << 24,
            SCORE_CLIP_Q24,
            SCORE_CLIP_Q24 + 1,
            i64::MAX,
        ];
        let scenarios = [
            // Non-gold atom: every denominator changes, gold Copy duplicates stay.
            (4, 5, -(1 << 24), vec![5, 5, 4], vec![0, -(2 << 24), 0], 0),
            // Gold atom and multiple Copy aliases, including tied maximum.
            (5, 5, 5 << 24, vec![5, 5], vec![5 << 24, -(2 << 24)], 0),
            // Unique maximum drops or is raised, requiring full reduction.
            (4, 5, 5 << 24, vec![5, 5], vec![0, 1 << 24], 0),
            // Raw reference changes but clipped reference8 remains tied.
            (4, 4, i64::MAX, vec![4, 5], vec![i64::MAX, 0], 0),
            // Sole clipped maximum can disappear entirely.
            (4, 4, i64::MAX, vec![], vec![], 0),
            // All low-clipped atoms, low tie retained or candidate raises it.
            (
                4,
                5,
                i64::MIN,
                vec![5, 5],
                vec![i64::MIN, i64::MIN],
                i64::MIN,
            ),
        ];
        let mut saw_fast = false;
        let mut saw_full = false;
        for (selected, gold, old, copy_ids, copy_scores, other) in scenarios {
            let mut reducer = fixture(12, false)?;
            let mut full = fixture(12, false)?;
            let mut gen = vec![other; 12];
            gen[selected as usize] = old;
            let mut cache = reducer.prepare_generate_substitution(
                gen.clone(),
                copy_ids.clone(),
                copy_scores.clone(),
                selected,
                gold,
            )?;
            let baseline = cache.incumbent_mass();
            let excluding = gen
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != selected as usize)
                .map(|(_, &s)| s.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24))
                .chain(
                    copy_scores
                        .iter()
                        .map(|&s| s.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24)),
                )
                .max()
                .ok_or("no reference")?;
            for candidate in candidates {
                let actual = reducer.evaluate_generate_substitution(&mut cache, candidate)?;
                let mut changed = gen.clone();
                changed[selected as usize] = candidate;
                let trace = full.reduce_trace(&changed, &copy_ids, &copy_scores)?;
                let target = trace
                    .token_masses
                    .iter()
                    .find(|m| m.token_id == gold)
                    .ok_or("missing target")?;
                assert_eq!(actual.total_weight_q31, trace.summary.total_weight_q31);
                assert_eq!(actual.target_weight_q31, target.weight_q31);
                assert_eq!(actual.reference_q24, trace.summary.max_score_q24);
                assert_eq!(
                    actual.used_full_reduction,
                    excluding.max(candidate.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24))
                        != baseline.reference_q24
                );
                saw_full |= actual.used_full_reduction;
                saw_fast |= !actual.used_full_reduction;
                assert_eq!(cache.incumbent_mass(), baseline);
                assert_eq!(cache.generate_q24, gen);
                // Interleaved fallback/fast calls cannot advance the incumbent.
                let replay = reducer.evaluate_generate_substitution(&mut cache, old)?;
                assert_eq!(replay, baseline);
            }
        }
        assert!(saw_fast && saw_full);
        Ok(())
    }

    #[test]
    fn substitution_cache_rejects_sparse_tokens_shapes_and_foreign_admission(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, true)?;
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 12], vec![], vec![], 7, 5),
            Err(VocabularyActionError::InvalidGenerateToken(7))
        ));
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 12], vec![], vec![], 4, 7),
            Err(VocabularyActionError::InvalidTargetToken(7))
        ));
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 12], vec![], vec![], 12, 5),
            Err(VocabularyActionError::InvalidGenerateToken(12))
        ));
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 11], vec![], vec![], 4, 5),
            Err(VocabularyActionError::ScoreShape)
        ));
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 12], vec![7], vec![0], 4, 5),
            Err(VocabularyActionError::InvalidCopyToken(7))
        ));
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 12], vec![5], vec![], 4, 5),
            Err(VocabularyActionError::ScoreShape)
        ));
        assert!(matches!(
            reducer.prepare_generate_substitution(vec![0; 12], vec![5; 129], vec![0; 129], 4, 5),
            Err(VocabularyActionError::CopyCount(129))
        ));
        let mut gen = vec![0; 12];
        gen[7] = i64::MAX; // Hole cannot own maximum.
        let mut cache = reducer.prepare_generate_substitution(gen, vec![5, 5], vec![0, 0], 4, 5)?;
        assert_eq!(cache.incumbent_mass().reference_q24, 0);
        let mut foreign = fixture(12, false)?;
        assert_eq!(
            foreign.evaluate_generate_substitution(&mut cache, 0),
            Err(VocabularyActionError::CacheBindingMismatch)
        );
        // Canonical public constructors fix one table. The private arithmetic
        // fixture also checks table identity defensively rather than trusting it.
        let altered = reducer
            .exp
            .iter()
            .enumerate()
            .map(|(i, &v)| if i == 0 { v } else { v / 2 })
            .collect();
        let mut foreign = NativeVocabularyActions::from_admitted(binding(12, true)?, altered)?;
        assert_eq!(
            foreign.evaluate_generate_substitution(&mut cache, 0),
            Err(VocabularyActionError::CacheBindingMismatch)
        );
        let baseline = cache.incumbent_mass();
        assert_eq!(
            reducer.evaluate_generate_substitution(&mut cache, 0)?,
            baseline
        );
        Ok(())
    }

    #[test]
    fn full4096_zero_copy_and_extreme_scores_retain_positive_mass(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut r = fixture(4096, false)?;
        let mut scores = vec![i64::MIN; 4096];
        scores[4000] = i64::MAX;
        let t = r.reduce_trace(&scores, &[], &[])?;
        assert_eq!(t.actions.len(), 4096);
        assert_eq!(t.token_masses.len(), 4096);
        assert!(t.actions.iter().all(|a| a.weight_q31 > 0));
        assert_eq!(t.summary.chosen_token_id, 4000);
        assert_eq!(t.summary.clipped_low_actions, 4095);
        assert_eq!(t.summary.clipped_high_actions, 1);
        assert_eq!(t.summary.raw_chosen_token_id, 4000);
        assert!(t.token_masses.iter().all(|m| m.copy_weight_q31 == 0));
        let t = r.reduce_trace(&vec![0; 4096], &[], &[])?;
        assert_eq!(t.summary.chosen_token_id, 0);
        assert_eq!(t.summary.total_weight_q31, 4096u64 << 31);
        Ok(())
    }
    #[test]
    fn eos_period_aliases_are_generate_plus_occurrences_not_extra_terminals(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut r = fixture(10, false)?;
        let ids = [
            r.binding().eos_token_id(),
            r.binding().period_token_id(),
            r.binding().eos_token_id(),
        ];
        let t = r.reduce_trace(&[0; 10], &ids, &[0; 3])?;
        assert_eq!(t.actions.len(), 13);
        assert_eq!(t.summary.chosen_token_id, ids[0]);
        let eos = t
            .token_masses
            .iter()
            .find(|m| m.token_id == ids[0])
            .ok_or("missing EOS")?;
        assert_eq!(eos.generate_weight_q31, 1 << 31);
        assert_eq!(eos.copy_weight_q31, 2u64 << 31);
        assert_eq!(eos.weight_q31, 3u64 << 31);
        assert_eq!(
            t.actions
                .iter()
                .filter(|a| a.action == VocabularyAction::Generate { token_id: ids[0] })
                .count(),
            1
        );
        assert_eq!(t.summary.total_weight_q31, 13u64 << 31);
        assert_eq!(
            t.summary.generate_weight_q31 + t.summary.copy_weight_q31,
            t.summary.total_weight_q31
        );
        Ok(())
    }
    #[test]
    fn sparse_holes_padding_shapes_and_copy_limit_are_checked_without_partial_outputs(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut r = fixture(10, true)?;
        let mut gen = [0; 10];
        gen[7] = i64::MAX;
        let mut w = vec![99; r.action_count(0)?];
        let mut m = vec![99; 10];
        let s = r.reduce_into(&gen, &[], &[], &mut w, &mut m)?;
        assert_eq!(s.legal_generate_actions, 9);
        assert_eq!(s.chosen_token_id, 0);
        assert_eq!(m[7], 0);
        let mut w = vec![99; r.action_count(1)?];
        let mut m = vec![99; 10];
        assert_eq!(
            r.reduce_into(&gen, &[7], &[0], &mut w, &mut m),
            Err(VocabularyActionError::InvalidCopyToken(7))
        );
        assert!(w.iter().all(|&v| v == 99) && m.iter().all(|&v| v == 99));
        assert!(matches!(
            r.reduce_trace(&gen, &[10], &[0]),
            Err(VocabularyActionError::InvalidCopyToken(10))
        ));
        assert_eq!(
            r.action_count(129),
            Err(VocabularyActionError::CopyCount(129))
        );
        assert!(matches!(
            r.reduce_trace(&gen[..9], &[], &[]),
            Err(VocabularyActionError::ScoreShape)
        ));
        assert!(matches!(
            r.reduce_into(&gen, &[], &[], &mut [], &mut m),
            Err(VocabularyActionError::OutputShape)
        ));
        assert!(matches!(
            r.reduce_trace(&gen, &[1], &[]),
            Err(VocabularyActionError::ScoreShape)
        ));
        let t = r.reduce_trace(&gen, &[4; 128], &[0; 128])?;
        assert_eq!(t.actions.len(), 137);
        assert_eq!(t.summary.chosen_token_id, 4);
        Ok(())
    }
    #[test]
    fn clip_can_change_token_mass_winner_and_reports_rank(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut r = fixture(10, false)?;
        let mut gen = [0; 10];
        gen[5] = i64::MAX;
        gen[4] = SCORE_CLIP_Q24;
        let t = r.reduce_trace(&gen, &[4], &[SCORE_CLIP_Q24])?;
        assert_eq!(t.summary.raw_chosen_token_id, 5);
        assert_eq!(t.summary.chosen_token_id, 4);
        assert!(t.summary.token_winner_changed_by_clip);
        assert_eq!(t.summary.chosen_raw_mass_rank, 2);
        assert_eq!(t.summary.raw_chosen_clipped_mass_rank, 2);
        Ok(())
    }
    #[test]
    fn public_constructor_rejects_numerically_plausible_noncanonical_exp(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let fake = vec![0; crate::geometric_source_realizer::CANONICAL_EXP_BYTES];
        assert!(matches!(
            NativeVocabularyActions::new(binding(10, false)?, &fake),
            Err(VocabularyActionError::ExpAdmission(_))
        ));
        assert!(matches!(
            NativeVocabularyActions::new(binding(10, false)?, &fake[..fake.len() - 1]),
            Err(VocabularyActionError::ExpAdmission(_))
        ));
        assert!(matches!(fixture(4097, false), Err(_)));
        let mut exp = vec![0; crate::geometric_read::EXP_TABLE_LEN];
        exp[0] = 1 << 31;
        assert!(matches!(
            NativeVocabularyActions::from_admitted(binding(10, false)?, exp),
            Err(VocabularyActionError::NonpositiveWeight)
        ));
        Ok(())
    }
}
