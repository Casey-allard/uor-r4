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
use std::sync::atomic::{AtomicU64, Ordering};

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
    DuplicatePatchToken(u32),
    InvalidPatchPosition(usize),
    StalePatch,
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

/// Exact served fields only; raw diagnostic ranks are not cached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GeneratePatchSummary {
    pub reference_q24: i64,
    pub total_weight_q31: u64,
    pub chosen_token_id: u32,
    pub chosen_weight_q31: u64,
    pub used_full_reduction: bool,
    pub scanned_maximum: bool,
    pub scanned_winner: bool,
}
/// Owned, target-free offline pool. Only authenticated preparation creates one.
/// This cache is not a serving allocation or performance claim.
pub struct GeneratePatchCache {
    binding: SourceActionBinding,
    exp_sha256: String,
    identity: u64,
    revision: u64,
    generate_q24: Vec<i64>,
    copy_ids: Vec<u32>,
    copy_q24: Vec<i64>,
    generate_weights: Vec<u64>,
    masses: Vec<u64>,
    maximum_count: usize,
    summary: GeneratePatchSummary,
}
impl GeneratePatchCache {
    pub fn summary(&self) -> GeneratePatchSummary {
        self.summary
    }
    pub fn generate_scores(&self) -> &[i64] {
        &self.generate_q24
    }
    pub fn token_masses(&self) -> &[u64] {
        &self.masses
    }
    pub fn copy_token_ids(&self) -> &[u32] {
        &self.copy_ids
    }
    pub fn copy_scores(&self) -> &[i64] {
        &self.copy_q24
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
struct GeneratePatchAtom {
    token: usize,
    raw: i64,
    weight: u64,
    mass: u64,
}
enum GeneratePatchStorage {
    Sparse(Vec<GeneratePatchAtom>),
    Full {
        scores: Vec<i64>,
        weights: Vec<u64>,
        masses: Vec<u64>,
    },
}
/// Revision-bound staged result. Not Clone; batch commit consumes it once.
pub struct PendingGeneratePatch {
    identity: u64,
    revision: u64,
    next_revision: u64,
    maximum_count: usize,
    summary: GeneratePatchSummary,
    storage: GeneratePatchStorage,
}
impl PendingGeneratePatch {
    pub fn summary(&self) -> GeneratePatchSummary {
        self.summary
    }
    /// Exact staged pooled mass, bound to the unmodified incumbent revision.
    /// Targets are attached by the offline evaluator after full pool admission.
    pub fn token_mass(&self, cache: &GeneratePatchCache, token: u32) -> Result<u64> {
        if self.identity != cache.identity || self.revision != cache.revision {
            return Err(VocabularyActionError::StalePatch);
        }
        if cache.binding.validate_tokens(&[token]).is_err() {
            return Err(VocabularyActionError::InvalidTargetToken(token));
        }
        match &self.storage {
            GeneratePatchStorage::Sparse(atoms) => Ok(atoms
                .binary_search_by_key(&(token as usize), |v| v.token)
                .map(|i| atoms[i].mass)
                .unwrap_or(cache.masses[token as usize])),
            GeneratePatchStorage::Full { masses, .. } => Ok(masses[token as usize]),
        }
    }
}
static PATCH_CACHE_ID: AtomicU64 = AtomicU64::new(1);

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

    /// Admit all physical actions before attaching any preservation objective.
    pub fn prepare_generate_patch_cache(
        &mut self,
        gen: Vec<i64>,
        copy_ids: Vec<u32>,
        copy_scores: Vec<i64>,
    ) -> Result<GeneratePatchCache> {
        let count = self.action_count(copy_ids.len())?;
        let mut weights = vec![0; count];
        let mut masses = vec![0; self.vocab_size()];
        let baseline =
            self.reduce_into(&gen, &copy_ids, &copy_scores, &mut weights, &mut masses)?;
        let mut generate_weights = vec![0; self.vocab_size()];
        for (offset, &id) in self.legal_ids.iter().enumerate() {
            generate_weights[id as usize] = weights[offset];
        }
        let maximum_count = self
            .legal_ids
            .iter()
            .map(|&id| gen[id as usize])
            .chain(copy_scores.iter().copied())
            .filter(|&raw| raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24) == baseline.max_score_q24)
            .count();
        let identity = PATCH_CACHE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| VocabularyActionError::Overflow)?;
        Ok(GeneratePatchCache {
            binding: self.binding.clone(),
            exp_sha256: self.exp_sha256.clone(),
            identity,
            revision: 0,
            generate_q24: gen,
            copy_ids,
            copy_q24: copy_scores,
            generate_weights,
            masses,
            maximum_count,
            summary: GeneratePatchSummary {
                reference_q24: baseline.max_score_q24,
                total_weight_q31: baseline.total_weight_q31,
                chosen_token_id: baseline.chosen_token_id,
                chosen_weight_q31: baseline.chosen_weight_q31,
                used_full_reduction: false,
                scanned_maximum: false,
                scanned_winner: false,
            },
        })
    }
    fn validate_patch_cache(&self, cache: &GeneratePatchCache) -> Result<()> {
        if cache.binding != self.binding || cache.exp_sha256 != self.exp_sha256 {
            return Err(VocabularyActionError::CacheBindingMismatch);
        }
        Ok(())
    }
    /// Simultaneous absolute raw-score replacement against the current revision.
    /// Evaluation never mutates the incumbent, including on arithmetic failure.
    pub fn evaluate_generate_patch(
        &mut self,
        cache: &GeneratePatchCache,
        changes: &[(u32, i64)],
    ) -> Result<PendingGeneratePatch> {
        self.validate_patch_cache(cache)?;
        let next_revision = cache
            .revision
            .checked_add(1)
            .ok_or(VocabularyActionError::Overflow)?;
        let mut changes = changes.to_vec();
        changes.sort_unstable_by_key(|v| v.0);
        for (i, &(id, _)) in changes.iter().enumerate() {
            if !self.legal_mask.get(id as usize).copied().unwrap_or(false) {
                return Err(VocabularyActionError::InvalidGenerateToken(id));
            }
            if i > 0 && changes[i - 1].0 == id {
                return Err(VocabularyActionError::DuplicatePatchToken(id));
            }
        }
        let old_reference = cache.summary.reference_q24;
        let removed = changes
            .iter()
            .filter(|&&(id, _)| {
                cache.generate_q24[id as usize].clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24)
                    == old_reference
            })
            .count();
        let surviving = cache
            .maximum_count
            .checked_sub(removed)
            .ok_or(VocabularyActionError::Overflow)?;
        let changed_max = changes
            .iter()
            .map(|&(_, raw)| raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24))
            .max();
        let scanned_maximum = surviving == 0;
        let score_at = |id: u32| {
            changes
                .binary_search_by_key(&id, |v| v.0)
                .map(|i| changes[i].1)
                .unwrap_or(cache.generate_q24[id as usize])
        };
        let reference = if surviving > 0 {
            changed_max.map_or(old_reference, |v| v.max(old_reference))
        } else {
            self.legal_ids
                .iter()
                .map(|&id| score_at(id))
                .chain(cache.copy_q24.iter().copied())
                .map(|v| v.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24))
                .max()
                .ok_or(VocabularyActionError::MissingLegalTerminal)?
        };
        if reference != old_reference {
            let mut scores = cache.generate_q24.clone();
            for &(id, raw) in &changes {
                scores[id as usize] = raw;
            }
            let mut weights = vec![0; self.action_count(cache.copy_ids.len())?];
            let mut masses = vec![0; self.vocab_size()];
            let full = self.reduce_into(
                &scores,
                &cache.copy_ids,
                &cache.copy_q24,
                &mut weights,
                &mut masses,
            )?;
            let mut generate_weights = vec![0; self.vocab_size()];
            for (offset, &id) in self.legal_ids.iter().enumerate() {
                generate_weights[id as usize] = weights[offset];
            }
            let maximum_count = self
                .legal_ids
                .iter()
                .map(|&id| scores[id as usize])
                .chain(cache.copy_q24.iter().copied())
                .filter(|&raw| raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24) == reference)
                .count();
            return Ok(PendingGeneratePatch {
                identity: cache.identity,
                revision: cache.revision,
                next_revision,
                maximum_count,
                summary: GeneratePatchSummary {
                    reference_q24: reference,
                    total_weight_q31: full.total_weight_q31,
                    chosen_token_id: full.chosen_token_id,
                    chosen_weight_q31: full.chosen_weight_q31,
                    used_full_reduction: true,
                    scanned_maximum,
                    scanned_winner: true,
                },
                storage: GeneratePatchStorage::Full {
                    scores,
                    weights: generate_weights,
                    masses,
                },
            });
        }
        let maximum_count = surviving
            .checked_add(
                changes
                    .iter()
                    .filter(|&&(_, raw)| raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24) == reference)
                    .count(),
            )
            .ok_or(VocabularyActionError::Overflow)?;
        let mut atoms = Vec::with_capacity(changes.len());
        let mut removed_weight = 0u64;
        let mut added_weight = 0u64;
        for &(id, raw) in &changes {
            let token = id as usize;
            let clipped = raw.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24);
            let weight = stack_exp_neg(reference - clipped, -24, &self.exp, EXP_STEP_LOG2);
            if weight == 0 {
                return Err(VocabularyActionError::NonpositiveWeight);
            }
            let replace = |v: u64| {
                v.checked_sub(cache.generate_weights[token])
                    .and_then(|v| v.checked_add(weight))
                    .ok_or(VocabularyActionError::Overflow)
            };
            removed_weight = removed_weight
                .checked_add(cache.generate_weights[token])
                .ok_or(VocabularyActionError::Overflow)?;
            added_weight = added_weight
                .checked_add(weight)
                .ok_or(VocabularyActionError::Overflow)?;
            atoms.push(GeneratePatchAtom {
                token,
                raw,
                weight,
                mass: replace(cache.masses[token])?,
            });
        }
        let total = cache
            .summary
            .total_weight_q31
            .checked_sub(removed_weight)
            .and_then(|v| v.checked_add(added_weight))
            .ok_or(VocabularyActionError::Overflow)?;
        let mass_at = |id: u32| {
            atoms
                .binary_search_by_key(&(id as usize), |v| v.token)
                .map(|i| atoms[i].mass)
                .unwrap_or(cache.masses[id as usize])
        };
        let incumbent = cache.summary.chosen_token_id;
        let scanned_winner = mass_at(incumbent) < cache.summary.chosen_weight_q31;
        let mut winner = incumbent;
        let mut winner_mass = mass_at(winner);
        let consider = |id: u32, winner: &mut u32, winner_mass: &mut u64| {
            let mass = mass_at(id);
            if mass > *winner_mass || (mass == *winner_mass && id < *winner) {
                *winner = id;
                *winner_mass = mass;
            }
        };
        if scanned_winner {
            for &id in &self.legal_ids {
                consider(id, &mut winner, &mut winner_mass);
            }
        } else {
            for atom in &atoms {
                consider(atom.token as u32, &mut winner, &mut winner_mass);
            }
        }
        Ok(PendingGeneratePatch {
            identity: cache.identity,
            revision: cache.revision,
            next_revision,
            maximum_count,
            summary: GeneratePatchSummary {
                reference_q24: reference,
                total_weight_q31: total,
                chosen_token_id: winner,
                chosen_weight_q31: winner_mass,
                used_full_reduction: false,
                scanned_maximum,
                scanned_winner,
            },
            storage: GeneratePatchStorage::Sparse(atoms),
        })
    }
    /// Validate every position before any mutation; application is infallible.
    /// An error consumes staged patches but leaves all caches unchanged.
    pub fn commit_generate_patch_batch(
        &self,
        caches: &mut [GeneratePatchCache],
        patches: Vec<(usize, PendingGeneratePatch)>,
    ) -> Result<()> {
        let mut positions = patches.iter().map(|v| v.0).collect::<Vec<_>>();
        positions.sort_unstable();
        if let Some(pair) = positions.windows(2).find(|p| p[0] == p[1]) {
            return Err(VocabularyActionError::InvalidPatchPosition(pair[0]));
        }
        for (index, patch) in &patches {
            let cache = caches
                .get(*index)
                .ok_or(VocabularyActionError::InvalidPatchPosition(*index))?;
            self.validate_patch_cache(cache)?;
            if patch.identity != cache.identity || patch.revision != cache.revision {
                return Err(VocabularyActionError::StalePatch);
            }
        }
        for (index, patch) in patches {
            let cache = &mut caches[index];
            match patch.storage {
                GeneratePatchStorage::Sparse(atoms) => {
                    for atom in atoms {
                        cache.generate_q24[atom.token] = atom.raw;
                        cache.generate_weights[atom.token] = atom.weight;
                        cache.masses[atom.token] = atom.mass;
                    }
                }
                GeneratePatchStorage::Full {
                    scores,
                    weights,
                    masses,
                } => {
                    cache.generate_q24 = scores;
                    cache.generate_weights = weights;
                    cache.masses = masses;
                }
            }
            cache.maximum_count = patch.maximum_count;
            cache.summary = patch.summary;
            cache.revision = patch.next_revision;
        }
        Ok(())
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
    fn assert_patch_pool(
        reducer: &mut NativeVocabularyActions,
        cache: &GeneratePatchCache,
    ) -> Result<()> {
        let trace =
            reducer.reduce_trace(cache.generate_scores(), &cache.copy_ids, &cache.copy_q24)?;
        assert_eq!(cache.summary.reference_q24, trace.summary.max_score_q24);
        assert_eq!(
            cache.summary.total_weight_q31,
            trace.summary.total_weight_q31
        );
        assert_eq!(cache.summary.chosen_token_id, trace.summary.chosen_token_id);
        assert_eq!(
            cache.summary.chosen_weight_q31,
            trace.summary.chosen_weight_q31
        );
        for token in &trace.token_masses {
            assert_eq!(cache.masses[token.token_id as usize], token.weight_q31);
        }
        Ok(())
    }
    #[test]
    fn patch_cache_simultaneous_aliases_clips_references_and_consecutive_commits(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, false)?;
        let mut gen = vec![0; 12];
        gen[4] = SCORE_CLIP_Q24;
        let mut caches =
            vec![reducer.prepare_generate_patch_cache(gen, vec![5, 5, 4], vec![0, 0, 0])?];
        let trials = vec![
            vec![(5, 1 << 24), (6, 2 << 24)],   // sparse, alias mass changes
            vec![(4, 0), (6, -1 << 24)],        // unique maximum removed
            vec![(7, i64::MAX), (5, i64::MIN)], // new clipped maximum
            vec![(4, i64::MAX), (7, i64::MIN)], // removed max replaced in same transaction
            vec![(4, i64::MIN), (6, i64::MIN)], // Copy becomes the maximum
            vec![(5, 0), (6, 0), (7, 0)],       // pooled ties
        ];
        let mut saw_fast = false;
        let mut saw_full = false;
        let mut saw_scan = false;
        for trial in trials {
            let before = caches[0].generate_scores().to_vec();
            let patch = reducer.evaluate_generate_patch(&caches[0], &trial)?;
            assert_eq!(caches[0].generate_scores(), before);
            saw_fast |= !patch.summary().used_full_reduction;
            saw_full |= patch.summary().used_full_reduction;
            saw_scan |= patch.summary().scanned_maximum;
            let mut expected = before;
            for &(id, raw) in &trial {
                expected[id as usize] = raw;
            }
            let full = reducer.reduce_trace(&expected, &caches[0].copy_ids, &caches[0].copy_q24)?;
            assert_eq!(
                patch.summary().chosen_token_id,
                full.summary.chosen_token_id
            );
            assert_eq!(
                patch.summary().chosen_weight_q31,
                full.summary.chosen_weight_q31
            );
            assert_eq!(
                patch.summary().total_weight_q31,
                full.summary.total_weight_q31
            );
            reducer.commit_generate_patch_batch(&mut caches, vec![(0, patch)])?;
            assert_patch_pool(&mut reducer, &caches[0])?;
        }
        assert!(saw_fast && saw_full && saw_scan);
        assert_eq!(caches[0].revision(), 6);
        Ok(())
    }
    #[test]
    fn patch_cache_smallest_id_ties_and_incumbent_decrease_scan(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, false)?;
        let mut gen = vec![0; 12];
        gen[5] = 2 << 24;
        let mut caches = vec![reducer.prepare_generate_patch_cache(gen, vec![], vec![])?];
        let patch = reducer.evaluate_generate_patch(&caches[0], &[(4, 2 << 24)])?;
        assert!(!patch.summary().used_full_reduction);
        assert_eq!(patch.summary().chosen_token_id, 4);
        reducer.commit_generate_patch_batch(&mut caches, vec![(0, patch)])?;
        let patch = reducer.evaluate_generate_patch(&caches[0], &[(4, 0)])?;
        assert!(patch.summary().scanned_winner);
        assert!(!patch.summary().used_full_reduction);
        assert_eq!(patch.summary().chosen_token_id, 5);
        reducer.commit_generate_patch_batch(&mut caches, vec![(0, patch)])?;
        assert_patch_pool(&mut reducer, &caches[0])?;
        Ok(())
    }
    #[test]
    fn patch_cache_invalid_stale_wrong_row_and_late_errors_are_atomic(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, true)?;
        let mut caches = vec![
            reducer.prepare_generate_patch_cache(vec![0; 12], vec![4], vec![0])?,
            reducer.prepare_generate_patch_cache(vec![0; 12], vec![5], vec![0])?,
        ];
        assert!(matches!(
            reducer.evaluate_generate_patch(&caches[0], &[(4, 1), (4, 2)]),
            Err(VocabularyActionError::DuplicatePatchToken(4))
        ));
        assert!(matches!(
            reducer.evaluate_generate_patch(&caches[0], &[(7, 0)]),
            Err(VocabularyActionError::InvalidGenerateToken(7))
        ));
        let wrong = reducer.evaluate_generate_patch(&caches[0], &[(4, 1 << 24)])?;
        assert!(matches!(
            reducer.commit_generate_patch_batch(&mut caches, vec![(1, wrong)]),
            Err(VocabularyActionError::StalePatch)
        ));
        let stale = reducer.evaluate_generate_patch(&caches[1], &[(5, 1 << 24)])?;
        let advance = reducer.evaluate_generate_patch(&caches[1], &[(6, 1 << 24)])?;
        reducer.commit_generate_patch_batch(&mut caches, vec![(1, advance)])?;
        let before = caches
            .iter()
            .map(|c| (c.generate_q24.clone(), c.masses.clone(), c.revision))
            .collect::<Vec<_>>();
        let valid = reducer.evaluate_generate_patch(&caches[0], &[(4, 1 << 24)])?;
        assert!(matches!(
            reducer.commit_generate_patch_batch(&mut caches, vec![(0, valid), (1, stale)]),
            Err(VocabularyActionError::StalePatch)
        ));
        assert_eq!(
            before,
            caches
                .iter()
                .map(|c| (c.generate_q24.clone(), c.masses.clone(), c.revision))
                .collect::<Vec<_>>()
        );
        let a = reducer.evaluate_generate_patch(&caches[0], &[(4, 1)])?;
        let b = reducer.evaluate_generate_patch(&caches[0], &[(5, 1)])?;
        assert!(matches!(
            reducer.commit_generate_patch_batch(&mut caches, vec![(0, a), (0, b)]),
            Err(VocabularyActionError::InvalidPatchPosition(0))
        ));
        let mut other = fixture(12, false)?;
        assert!(matches!(
            other.evaluate_generate_patch(&caches[0], &[]),
            Err(VocabularyActionError::CacheBindingMismatch)
        ));
        caches[0].revision = u64::MAX;
        assert!(matches!(
            reducer.evaluate_generate_patch(&caches[0], &[]),
            Err(VocabularyActionError::Overflow)
        ));
        Ok(())
    }
    #[test]
    fn pending_patch_target_mass_is_exact_revision_bound_and_nonmutating(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, true)?;
        let mut gen = vec![0; 12];
        gen[4] = SCORE_CLIP_Q24;
        let mut caches = vec![
            reducer.prepare_generate_patch_cache(gen.clone(), vec![5, 5], vec![0, 0])?,
            reducer.prepare_generate_patch_cache(gen, vec![], vec![])?,
        ];
        for trial in [vec![(5, 2 << 24)], vec![(4, 0), (5, 2 << 24)]] {
            let patch = reducer.evaluate_generate_patch(&caches[0], &trial)?;
            let before = caches[0].generate_scores().to_vec();
            let mut actual = before.clone();
            for &(id, score) in &trial {
                actual[id as usize] = score;
            }
            let full = reducer.reduce_trace(&actual, &caches[0].copy_ids, &caches[0].copy_q24)?;
            for token in &full.token_masses {
                assert_eq!(
                    patch.token_mass(&caches[0], token.token_id)?,
                    token.weight_q31
                );
            }
            assert!(matches!(
                patch.token_mass(&caches[0], 7),
                Err(VocabularyActionError::InvalidTargetToken(7))
            ));
            assert!(matches!(
                patch.token_mass(&caches[1], 5),
                Err(VocabularyActionError::StalePatch)
            ));
            assert_eq!(before, caches[0].generate_scores());
        }
        let stale = reducer.evaluate_generate_patch(&caches[0], &[(5, 1 << 24)])?;
        let current = reducer.evaluate_generate_patch(&caches[0], &[(6, 1 << 24)])?;
        reducer.commit_generate_patch_batch(&mut caches, vec![(0, current)])?;
        assert!(matches!(
            stale.token_mass(&caches[0], 5),
            Err(VocabularyActionError::StalePatch)
        ));
        Ok(())
    }
    #[test]
    fn patch_cache_checked_mass_overflow_leaves_incumbent_unchanged(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, false)?;
        let mut gen = vec![0; 12];
        gen[4] = SCORE_CLIP_Q24;
        let mut cache = reducer.prepare_generate_patch_cache(gen, vec![], vec![])?;
        // Internal fault injection verifies checked arithmetic, not a public loader.
        cache.summary.total_weight_q31 = u64::MAX;
        let before = (
            cache.generate_q24.clone(),
            cache.masses.clone(),
            cache.summary,
        );
        assert!(matches!(
            reducer.evaluate_generate_patch(&cache, &[(5, 2 << 24)]),
            Err(VocabularyActionError::Overflow)
        ));
        assert_eq!(
            before,
            (
                cache.generate_q24.clone(),
                cache.masses.clone(),
                cache.summary
            )
        );
        Ok(())
    }
    #[test]
    fn patch_cache_discarded_late_infeasible_row_preserves_every_incumbent(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reducer = fixture(12, false)?;
        let mut gen = vec![0; 12];
        gen[4] = 2 << 24;
        let caches = vec![
            reducer.prepare_generate_patch_cache(gen.clone(), vec![], vec![])?,
            reducer.prepare_generate_patch_cache(gen, vec![], vec![])?,
        ];
        let before = caches
            .iter()
            .map(|c| (c.generate_q24.clone(), c.masses.clone(), c.revision))
            .collect::<Vec<_>>();
        let first = reducer.evaluate_generate_patch(&caches[0], &[(5, 1 << 24)])?;
        let late = reducer.evaluate_generate_patch(&caches[1], &[(5, 4 << 24)])?;
        assert_eq!(first.summary().chosen_token_id, 4);
        assert_ne!(late.summary().chosen_token_id, 4);
        drop((first, late));
        assert_eq!(
            before,
            caches
                .iter()
                .map(|c| (c.generate_q24.clone(), c.masses.clone(), c.revision))
                .collect::<Vec<_>>()
        );
        Ok(())
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
