//! Exact-token, response-uniform episodes for the paired dialogue-prefix study.
//!
//! The caller binds token/mask/manifest/tokenizer identities and verifies the
//! prepared corpus's zero literal-special-token count before indexing. This is
//! not a parser for arbitrary tokenized dialogue: BOS and response-mask runs
//! have their original serializer meanings under that bound corpus contract.
//! No tokenizer, model, optimizer, filesystem or loss normalization runs here.

use serde::{Deserialize, Serialize};

use crate::{invalid, Result};

pub const EPISODE_CONTEXT: usize = 256;
pub const SAMPLER_ID: &str = "uor-r4.dialogue-response-uniform/splitmix64-counter-rejection-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefixPolicy {
    FullPrefix,
    RoleOnly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeContract {
    pub context: usize,
    pub vocab_size: usize,
    pub bos_id: u32,
    pub eos_id: u32,
    pub unk_id: u32,
    /// In-vocabulary filler used only after the last real EOS target.
    pub padding_id: u32,
    /// Bound encoder.encode_assistant_prefix(&[]).tokens[1..], after the caller
    /// verifies the omitted first ID is the bound BOS. No newline is included.
    pub assistant_marker_ids: Vec<u32>,
}

impl EpisodeContract {
    fn validate(&self) -> Result<()> {
        let specials = [self.bos_id, self.eos_id, self.unk_id];
        if self.context != EPISODE_CONTEXT
            || self.vocab_size == 0
            || self.vocab_size > usize::from(u16::MAX) + 1
            || specials.iter().any(|&id| id as usize >= self.vocab_size)
            || self.padding_id as usize >= self.vocab_size
            || self.bos_id == self.eos_id
            || self.bos_id == self.unk_id
            || self.eos_id == self.unk_id
            || self.assistant_marker_ids.is_empty()
            || self.assistant_marker_ids.len() > self.context - 2
            || self
                .assistant_marker_ids
                .iter()
                .any(|&id| id as usize >= self.vocab_size || specials.contains(&id))
        {
            return Err(invalid("dialogue episode protocol/256-ID context contract"));
        }
        Ok(())
    }
}

/// Ordered prepared-manifest source ranges, covering the complete token store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub label: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EpisodeSpan {
    /// Stable zero-based index in the ordered eligible-response inventory.
    pub response_id: usize,
    /// Zero-based response ordinal before the length eligibility filter.
    pub corpus_response_index: usize,
    pub source_index: usize,
    pub document_start: usize,
    pub response_start: usize,
    pub response_end: usize,
    pub identical_prefix: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct SourcePopulation {
    pub source_index: usize,
    pub label: String,
    pub documents: usize,
    pub response_runs: usize,
    pub response_tokens: usize,
    pub eligible_responses: usize,
    pub eligible_response_tokens: usize,
    pub excluded_over_context: usize,
    pub identical_prefixes: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct EpisodePopulation {
    pub corpus_tokens: usize,
    pub documents: usize,
    pub response_runs: usize,
    pub response_tokens: usize,
    pub eligible_responses: usize,
    pub eligible_response_tokens: usize,
    pub excluded_over_context: usize,
    pub identical_prefixes: usize,
    pub sources: Vec<SourcePopulation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct EpisodeCounts {
    pub response_visits: usize,
    /// Real observed input IDs, excluding the selected response's final EOS.
    pub real_input_positions: usize,
    /// Real input IDs preceding the selected response, including BOS/marker.
    pub prefix_positions: usize,
    pub padded_positions: usize,
    /// Selected response content and genuine EOS labels; no prior reply loss.
    pub supervised_target_count: usize,
    pub eos_targets: usize,
    pub identical_prefix_visits: usize,
}

impl EpisodeCounts {
    fn add(&mut self, other: &Self) {
        self.response_visits += other.response_visits;
        self.real_input_positions += other.real_input_positions;
        self.prefix_positions += other.prefix_positions;
        self.padded_positions += other.padded_positions;
        self.supervised_target_count += other.supervised_target_count;
        self.eos_targets += other.eos_targets;
        self.identical_prefix_visits += other.identical_prefix_visits;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EpisodeRow {
    pub response_id: usize,
    pub corpus_response_index: usize,
    pub source_index: usize,
    pub document_start: usize,
    pub response_start: usize,
    pub response_end: usize,
    pub counts: EpisodeCounts,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceVisits {
    pub source_index: usize,
    pub label: String,
    pub counts: EpisodeCounts,
}

#[derive(Clone, Debug)]
pub struct EpisodeBatch {
    pub policy: PrefixPolicy,
    pub batch: usize,
    pub time: usize,
    /// Batch-major rectangular arrays, right-padded only after real labels.
    pub inputs: Vec<u32>,
    pub targets: Vec<u32>,
    pub weights: Vec<f32>,
    pub response_ids: Vec<usize>,
    /// Selected response token IDs including EOS, concatenated in lane order.
    /// This sequence is identical across the two prefix policies.
    pub selected_target_ids: Vec<u32>,
    pub rows: Vec<EpisodeRow>,
    pub counts: EpisodeCounts,
    pub source_visits: Vec<SourceVisits>,
}

/// Borrows the exact validated token array, preventing later materialization
/// against a different token store. Mask validation happens once at creation.
pub struct EpisodeIndex<'a> {
    tokens: &'a [u16],
    contract: EpisodeContract,
    sources: Vec<SourceSpan>,
    episodes: Vec<EpisodeSpan>,
    population: EpisodePopulation,
}

impl<'a> EpisodeIndex<'a> {
    pub fn new(
        tokens: &'a [u16],
        mask: &[u8],
        contract: EpisodeContract,
        sources: &[SourceSpan],
    ) -> Result<Self> {
        contract.validate()?;
        if tokens.is_empty() || tokens.len() != mask.len() || sources.is_empty() {
            return Err(invalid("dialogue episode token/mask/source shape"));
        }
        let mut next_start = 0;
        for source in sources {
            if source.label.trim().is_empty()
                || source.start != next_start
                || source.start >= source.end
                || source.end > tokens.len()
                || u32::from(tokens[source.start]) != contract.bos_id
            {
                return Err(invalid(
                    "dialogue source ranges must partition the corpus at BOS",
                ));
            }
            next_start = source.end;
        }
        if next_start != tokens.len() || u32::from(tokens[tokens.len() - 1]) != contract.eos_id {
            return Err(invalid("dialogue source coverage or terminal EOS"));
        }
        let population = EpisodePopulation {
            corpus_tokens: tokens.len(),
            sources: sources
                .iter()
                .enumerate()
                .map(|(source_index, source)| SourcePopulation {
                    source_index,
                    label: source.label.clone(),
                    ..SourcePopulation::default()
                })
                .collect(),
            ..EpisodePopulation::default()
        };
        let mut index = Self {
            tokens,
            contract,
            sources: sources.to_vec(),
            episodes: Vec::new(),
            population,
        };
        let mut document = 0;
        let mut source_index = 0;
        let mut response = None;
        for (position, (&token, &selected)) in tokens.iter().zip(mask).enumerate() {
            let token = u32::from(token);
            if token as usize >= index.contract.vocab_size
                || token == index.contract.unk_id
                || selected > 1
            {
                return Err(invalid(
                    "dialogue corpus vocabulary/binary mask/zero-special contract",
                ));
            }
            // Close a response before changing the document/source at its
            // following BOS. Its final EOS still belongs to the previous one.
            if selected == 0 {
                if let Some(start) = response.take() {
                    index.record_response(mask, document, start, position, source_index)?;
                }
            }
            if position == index.sources[source_index].end {
                source_index += 1;
            }
            if token == index.contract.bos_id {
                if selected != 0
                    || (position != 0 && u32::from(tokens[position - 1]) != index.contract.eos_id)
                {
                    return Err(invalid(
                        "dialogue BOS must be unmasked and follow document EOS",
                    ));
                }
                document = position;
                index.population.documents += 1;
                index.population.sources[source_index].documents += 1;
            }
            if token == index.contract.eos_id
                && selected == 0
                && position + 1 < tokens.len()
                && u32::from(tokens[position + 1]) != index.contract.bos_id
            {
                return Err(invalid("unmasked dialogue EOS must terminate its document"));
            }
            if selected == 1 && response.is_none() {
                response = Some(position);
            }
        }
        if let Some(start) = response {
            index.record_response(mask, document, start, tokens.len(), source_index)?;
        }
        if index.episodes.is_empty() {
            return Err(invalid(
                "dialogue corpus has no complete response fitting 256 total IDs",
            ));
        }
        Ok(index)
    }

    fn record_response(
        &mut self,
        mask: &[u8],
        document: usize,
        start: usize,
        end: usize,
        source_index: usize,
    ) -> Result<()> {
        let marker_len = self.contract.assistant_marker_ids.len();
        let marker_start = start
            .checked_sub(marker_len)
            .ok_or_else(|| invalid("dialogue response has no complete assistant marker"))?;
        if start >= end
            || marker_start <= document
            || u32::from(self.tokens[end - 1]) != self.contract.eos_id
            || self.tokens[start..end - 1]
                .iter()
                .any(|&id| u32::from(id) == self.contract.eos_id)
            || mask[marker_start..start].iter().any(|&value| value != 0)
            || !self.tokens[marker_start..start]
                .iter()
                .map(|&id| u32::from(id))
                .eq(self.contract.assistant_marker_ids.iter().copied())
        {
            return Err(invalid(
                "dialogue response must retain exact unmasked marker and genuine terminal EOS",
            ));
        }
        let corpus_response_index = self.population.response_runs;
        let response_tokens = end - start;
        self.population.response_runs += 1;
        self.population.response_tokens += response_tokens;
        let source = &mut self.population.sources[source_index];
        source.response_runs += 1;
        source.response_tokens += response_tokens;
        if end - document > self.contract.context {
            self.population.excluded_over_context += 1;
            source.excluded_over_context += 1;
            return Ok(());
        }
        // BOS plus exact marker is the complete empty-history role prefix.
        let identical_prefix = marker_start == document + 1;
        self.population.eligible_responses += 1;
        self.population.eligible_response_tokens += response_tokens;
        self.population.identical_prefixes += usize::from(identical_prefix);
        source.eligible_responses += 1;
        source.eligible_response_tokens += response_tokens;
        source.identical_prefixes += usize::from(identical_prefix);
        self.episodes.push(EpisodeSpan {
            response_id: self.episodes.len(),
            corpus_response_index,
            source_index,
            document_start: document,
            response_start: start,
            response_end: end,
            identical_prefix,
        });
        Ok(())
    }

    pub fn contract(&self) -> &EpisodeContract {
        &self.contract
    }

    pub fn sources(&self) -> &[SourceSpan] {
        &self.sources
    }

    pub fn episodes(&self) -> &[EpisodeSpan] {
        &self.episodes
    }

    pub fn population(&self) -> &EpisodePopulation {
        &self.population
    }

    /// Uniform response selection with replacement. Prefix policy does not
    /// enter this schedule. Rejection removes modulo bias without reseeding.
    pub fn sample_ids(&self, seed: u64, step: u64, batch: usize) -> Result<Vec<usize>> {
        if !(1..=64).contains(&batch) {
            return Err(invalid("dialogue episode batch must be 1..64"));
        }
        let count = self.episodes.len() as u64;
        let threshold = count.wrapping_neg() % count;
        let mut result = Vec::with_capacity(batch);
        for lane in 0..batch {
            let mut counter = seed
                ^ step.wrapping_mul(0xd1342543de82ef95)
                ^ (lane as u64).wrapping_mul(0x9e3779b97f4a7c15);
            loop {
                let draw = splitmix(counter);
                if draw >= threshold {
                    result.push((draw % count) as usize);
                    break;
                }
                counter = counter.wrapping_add(0x9e3779b97f4a7c15);
            }
        }
        Ok(result)
    }

    pub fn materialize(&self, ids: &[usize], policy: PrefixPolicy) -> Result<EpisodeBatch> {
        if ids.is_empty() || ids.len() > 64 || ids.iter().any(|&id| id >= self.episodes.len()) {
            return Err(invalid("dialogue episode batch/response ID"));
        }
        let time = self.contract.context;
        let mut batch = EpisodeBatch {
            policy,
            batch: ids.len(),
            time,
            inputs: Vec::with_capacity(ids.len() * time),
            targets: Vec::with_capacity(ids.len() * time),
            weights: Vec::with_capacity(ids.len() * time),
            response_ids: ids.to_vec(),
            selected_target_ids: Vec::new(),
            rows: Vec::with_capacity(ids.len()),
            counts: EpisodeCounts::default(),
            source_visits: self
                .sources
                .iter()
                .enumerate()
                .map(|(source_index, source)| SourceVisits {
                    source_index,
                    label: source.label.clone(),
                    counts: EpisodeCounts::default(),
                })
                .collect(),
        };
        for &id in ids {
            let span = &self.episodes[id];
            let r = span.response_start;
            let e = span.response_end;
            let mut episode = Vec::with_capacity(e - span.document_start);
            match policy {
                PrefixPolicy::FullPrefix => episode.extend(
                    self.tokens[span.document_start..r]
                        .iter()
                        .map(|&token| u32::from(token)),
                ),
                PrefixPolicy::RoleOnly => {
                    episode.push(u32::from(self.tokens[span.document_start]));
                    let marker_start = r - self.contract.assistant_marker_ids.len();
                    episode.extend(
                        self.tokens[marker_start..r]
                            .iter()
                            .map(|&token| u32::from(token)),
                    );
                }
            }
            let prefix_positions = episode.len();
            episode.extend(self.tokens[r..e].iter().map(|&token| u32::from(token)));
            batch
                .selected_target_ids
                .extend(self.tokens[r..e].iter().map(|&token| u32::from(token)));
            let real_input_positions = episode.len() - 1;
            for position in 0..time {
                if position < real_input_positions {
                    batch.inputs.push(episode[position]);
                    batch.targets.push(episode[position + 1]);
                    batch
                        .weights
                        .push(f32::from(u8::from(position + 1 >= prefix_positions)));
                } else {
                    batch.inputs.push(self.contract.padding_id);
                    batch.targets.push(self.contract.padding_id);
                    batch.weights.push(0.0);
                }
            }
            let counts = EpisodeCounts {
                response_visits: 1,
                real_input_positions,
                prefix_positions,
                padded_positions: time - real_input_positions,
                supervised_target_count: e - r,
                eos_targets: 1,
                identical_prefix_visits: usize::from(span.identical_prefix),
            };
            batch.counts.add(&counts);
            batch.source_visits[span.source_index].counts.add(&counts);
            batch.rows.push(EpisodeRow {
                response_id: id,
                corpus_response_index: span.corpus_response_index,
                source_index: span.source_index,
                document_start: span.document_start,
                response_start: r,
                response_end: e,
                counts,
            });
        }
        Ok(batch)
    }
}

fn splitmix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> EpisodeContract {
        EpisodeContract {
            context: EPISODE_CONTEXT,
            vocab_size: 64,
            bos_id: 0,
            eos_id: 1,
            unk_id: 2,
            padding_id: 2,
            assistant_marker_ids: vec![7, 8],
        }
    }

    fn source(end: usize) -> Vec<SourceSpan> {
        vec![SourceSpan {
            label: "fixture.jsonl".into(),
            start: 0,
            end,
        }]
    }

    fn two_responses() -> (Vec<u16>, Vec<u8>) {
        // Opaque BPE IDs are copied exactly, including the independently
        // encoded marker, response tokens, separator and previous real EOS.
        (
            vec![0, 10, 11, 7, 8, 20, 21, 1, 9, 10, 7, 8, 22, 1],
            vec![0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1],
        )
    }

    fn supervised(batch: &EpisodeBatch, lane: usize) -> Vec<u32> {
        let start = lane * batch.time;
        batch.targets[start..start + batch.time]
            .iter()
            .zip(&batch.weights[start..start + batch.time])
            .filter_map(|(&id, &weight)| (weight == 1.0).then_some(id))
            .collect()
    }

    #[test]
    fn dialogue_episodes_preserve_exact_ids_and_only_selected_response_loss() -> Result<()> {
        let (tokens, mask) = two_responses();
        let index = EpisodeIndex::new(&tokens, &mask, contract(), &source(tokens.len()))?;
        let full = index.materialize(&[0, 1], PrefixPolicy::FullPrefix)?;
        let role = index.materialize(&[0, 1], PrefixPolicy::RoleOnly)?;
        assert_eq!(&full.inputs[..7], &[0, 10, 11, 7, 8, 20, 21]);
        assert_eq!(&full.targets[..7], &[10, 11, 7, 8, 20, 21, 1]);
        assert_eq!(&role.inputs[..5], &[0, 7, 8, 20, 21]);
        assert_eq!(&role.targets[..5], &[7, 8, 20, 21, 1]);
        assert_eq!(
            &full.inputs[256..269],
            &tokens[..13]
                .iter()
                .map(|&x| u32::from(x))
                .collect::<Vec<_>>()
        );
        assert!(full.weights[256..267].iter().all(|&x| x == 0.0));
        for batch in [&full, &role] {
            assert_eq!(supervised(batch, 0), [20, 21, 1]);
            assert_eq!(supervised(batch, 1), [22, 1]);
            assert_eq!(batch.selected_target_ids, [20, 21, 1, 22, 1]);
            assert_eq!(batch.counts.supervised_target_count, 5);
            assert_eq!(batch.counts.eos_targets, 2);
            for (lane, row) in batch.rows.iter().enumerate() {
                let last = lane * batch.time + row.counts.real_input_positions - 1;
                assert_eq!((batch.targets[last], batch.weights[last]), (1, 1.0));
                let padding = last + 1..(lane + 1) * batch.time;
                assert!(batch.inputs[padding.clone()].iter().all(|&id| id == 2));
                assert!(batch.targets[padding.clone()].iter().all(|&id| id == 2));
                assert!(batch.weights[padding].iter().all(|&weight| weight == 0.0));
            }
            assert_eq!(
                batch.counts.real_input_positions + batch.counts.padded_positions,
                512
            );
        }
        assert_eq!(full.counts.prefix_positions, 17);
        assert_eq!(role.counts.prefix_positions, 6);
        Ok(())
    }

    #[test]
    fn dialogue_episodes_enforce_256_total_ids_and_preserve_identical_prefix_rows() -> Result<()> {
        let document = |len: usize| {
            let mut tokens = vec![11u16; len];
            tokens[0] = 0;
            tokens[len - 4..].copy_from_slice(&[7, 8, 22, 1]);
            let mut mask = vec![0u8; len];
            mask[len - 2..].fill(1);
            (tokens, mask)
        };
        let (mut tokens, mut mask) = document(256);
        let (long, long_mask) = document(257);
        tokens.extend(long);
        mask.extend(long_mask);
        tokens.extend([0, 7, 8, 23, 1]);
        mask.extend([0, 0, 0, 1, 1]);
        let sources = [
            SourceSpan {
                label: "first".into(),
                start: 0,
                end: 256,
            },
            SourceSpan {
                label: "second".into(),
                start: 256,
                end: tokens.len(),
            },
        ];
        let index = EpisodeIndex::new(&tokens, &mask, contract(), &sources)?;
        assert_eq!(index.population().response_runs, 3);
        assert_eq!(index.population().eligible_responses, 2);
        assert_eq!(index.population().excluded_over_context, 1);
        assert_eq!(index.population().identical_prefixes, 1);
        assert_eq!(index.population().sources[1].documents, 2);
        assert_eq!(index.population().sources[1].eligible_responses, 1);
        assert_eq!(index.episodes()[1].corpus_response_index, 2);
        let full = index.materialize(&[0, 1], PrefixPolicy::FullPrefix)?;
        let role = index.materialize(&[0, 1], PrefixPolicy::RoleOnly)?;
        assert_eq!(full.rows[0].counts.real_input_positions, 255);
        assert_eq!(full.rows[0].counts.padded_positions, 1);
        assert_eq!(full.inputs[256..], role.inputs[256..]);
        assert_eq!(full.targets[256..], role.targets[256..]);
        assert_eq!(full.weights[256..], role.weights[256..]);
        assert_eq!(full.counts.identical_prefix_visits, 1);
        assert_eq!(full.source_visits[0].counts.response_visits, 1);
        assert_eq!(full.source_visits[1].counts.response_visits, 1);
        Ok(())
    }

    #[test]
    fn dialogue_episodes_reject_broken_marker_eos_masks_and_source_boundaries() -> Result<()> {
        let (tokens, mask) = two_responses();
        let mut changed = tokens.clone();
        changed[4] = 9;
        assert!(EpisodeIndex::new(&changed, &mask, contract(), &source(tokens.len())).is_err());
        changed = tokens.clone();
        changed[7] = 9;
        assert!(EpisodeIndex::new(&changed, &mask, contract(), &source(tokens.len())).is_err());
        changed = tokens.clone();
        changed[5] = 1;
        assert!(EpisodeIndex::new(&changed, &mask, contract(), &source(tokens.len())).is_err());
        let mut changed_mask = mask.clone();
        changed_mask[4] = 1;
        assert!(
            EpisodeIndex::new(&tokens, &changed_mask, contract(), &source(tokens.len())).is_err()
        );
        changed_mask = mask.clone();
        changed_mask[4] = 2;
        assert!(
            EpisodeIndex::new(&tokens, &changed_mask, contract(), &source(tokens.len())).is_err()
        );
        let wrong_sources = [
            SourceSpan {
                label: "a".into(),
                start: 0,
                end: 8,
            },
            SourceSpan {
                label: "b".into(),
                start: 8,
                end: tokens.len(),
            },
        ];
        assert!(EpisodeIndex::new(&tokens, &mask, contract(), &wrong_sources).is_err());
        Ok(())
    }

    #[test]
    fn dialogue_episodes_pair_full_response_targets_under_one_counter_schedule() -> Result<()> {
        let (tokens, mask) = two_responses();
        let index = EpisodeIndex::new(&tokens, &mask, contract(), &source(tokens.len()))?;
        let ids = index.sample_ids(240927, 17, 16)?;
        assert_eq!(ids, index.sample_ids(240927, 17, 16)?);
        let full = index.materialize(&ids, PrefixPolicy::FullPrefix)?;
        let role = index.materialize(&ids, PrefixPolicy::RoleOnly)?;
        assert_eq!(full.response_ids, role.response_ids);
        assert_eq!(full.selected_target_ids, role.selected_target_ids);
        assert_eq!(
            full.counts.supervised_target_count,
            role.counts.supervised_target_count
        );
        assert_eq!(full.counts.eos_targets, 16);
        for lane in 0..16 {
            assert_eq!(supervised(&full, lane), supervised(&role, lane));
        }
        assert!(index.sample_ids(0, 0, 0).is_err());
        assert!(index.materialize(&[2], PrefixPolicy::FullPrefix).is_err());
        assert_eq!(
            serde_json::to_value(PrefixPolicy::FullPrefix)?,
            "full_prefix"
        );
        assert_eq!(serde_json::to_value(PrefixPolicy::RoleOnly)?, "role_only");
        Ok(())
    }
}
