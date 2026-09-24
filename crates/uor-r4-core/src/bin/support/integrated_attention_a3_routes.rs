//! Fit-only causal route curriculum and loaded hard-route diagnosis for A3.
//!
//! The annotation names an earlier exact source event. All encoder inputs are
//! collected by replaying the observed prefix through the native Session; no
//! answer token or annotation enters a serving Query or Key. A committed key
//! affects finite-page occupancy even when it is not a labeled negative.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::Value;
use uor_r4_core::native_geometric::learner::integrated_attention::{
    encoder::{EncoderInput, HardAddressPair, HardAddressWorld},
    training::TrainingEpisode,
    IntegratedModel, Session, ADDRESS_COMMIT_DELAY,
};

use crate::integrated_attention_a2_data::DataSet;

type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug, Serialize)]
pub struct CommittedKey {
    /// Zero-based location of the owned source token in this episode.
    pub source_position: usize,
    pub source_event_id: u64,
    pub record_id: u64,
    pub token: u16,
    pub key_input: EncoderInput,
    /// Hard product code at the instant this occurrence was committed.
    pub code: [u8; 16],
}

#[derive(Clone, Debug, Serialize)]
pub struct HardRouteDiagnostic {
    pub query_code: [u8; 16],
    pub positive_code: [u8; 16],
    pub positive_committed: bool,
    pub positive_coarse_rank: usize,
    pub positive_coarse_score: i16,
    pub query_coarse_score: i16,
    pub query_coarse_margin: i16,
    pub same_coarse_committed: usize,
    pub source_admitted: bool,
    pub source_ranked: bool,
    pub source_selected: bool,
    pub gate_open: bool,
    pub candidate_count: usize,
    pub search_incomplete: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RouteCase {
    pub episode_name: String,
    pub split: String,
    pub entity: String,
    pub style: Option<u8>,
    pub polarity: String,
    pub pair_id: String,
    pub source_id: u64,
    pub prompt_len: usize,
    pub target_first: u16,
    pub source_position: usize,
    pub source_token: u16,
    pub source_event_id: u64,
    pub positive_key_input: EncoderInput,
    /// Index into `committed`; absent if the actual write gate did not commit
    /// the annotated source. A3 cannot call such a source admitted.
    pub positive_committed_index: Option<usize>,
    /// Only exact distractor-value anchors for an explicitly different entity.
    /// Other records are unlabeled and contribute page occupancy only.
    pub negative_indices: Vec<usize>,
    /// Exact raw-token positions overlapping the authored other-entity value,
    /// whether or not the learned write gate committed those occurrences.
    pub wrong_entity_value_positions: Vec<usize>,
    pub wrong_entity: Option<String>,
    pub query: EncoderInput,
    pub committed: Vec<CommittedKey>,
    pub hard: HardRouteDiagnostic,
    /// Hash of the exact causal prefix and owner ID, independent of the model.
    pub frozen_prefix_blake3: String,
    /// Hash of the contextual address-token identities for Query and positive
    /// Key; masked positions remain explicit. This is independent of weights.
    pub frozen_address_context_blake3: String,
    pub source_manifest_sha256: String,
}

pub struct RouteCollection {
    pub cases: Vec<RouteCase>,
    pub pairs: Vec<HardAddressPair>,
    pub skipped_pair_ids: Vec<String>,
    pub skipped_pair_reasons: Vec<(String, String)>,
    /// Stable sorted fit entity names; its index is `entity_group` in pairs.
    pub entity_groups: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RouteSummary {
    pub cases: usize,
    pub positive_committed: usize,
    pub explicit_negative_committed_cases: usize,
    pub positive_coarse_top_one: usize,
    pub positive_coarse_rank_sum: usize,
    pub positive_coarse_rank_max: usize,
    pub positive_same_coarse: usize,
    pub source_admitted: usize,
    pub source_ranked: usize,
    pub source_selected: usize,
    pub search_incomplete: usize,
    pub same_coarse_committed_max: usize,
    pub query_coarse_histogram: BTreeMap<u8, usize>,
}

pub fn summarize(cases: &[RouteCase]) -> RouteSummary {
    let mut result = RouteSummary::default();
    for case in cases {
        result.cases += 1;
        result.positive_committed += usize::from(case.hard.positive_committed);
        result.explicit_negative_committed_cases += usize::from(!case.negative_indices.is_empty());
        result.positive_coarse_top_one += usize::from(case.hard.positive_coarse_rank == 1);
        result.positive_coarse_rank_sum += case.hard.positive_coarse_rank;
        result.positive_coarse_rank_max = result
            .positive_coarse_rank_max
            .max(case.hard.positive_coarse_rank);
        result.positive_same_coarse +=
            usize::from(case.hard.query_code[0] == case.hard.positive_code[0]);
        result.source_admitted += usize::from(case.hard.source_admitted);
        result.source_ranked += usize::from(case.hard.source_ranked);
        result.source_selected += usize::from(case.hard.source_selected);
        result.search_incomplete += usize::from(case.hard.search_incomplete);
        result.same_coarse_committed_max = result
            .same_coarse_committed_max
            .max(case.hard.same_coarse_committed);
        *result
            .query_coarse_histogram
            .entry(case.hard.query_code[0])
            .or_default() += 1;
    }
    result
}

struct CaseMetadata {
    split: String,
    entity: String,
    style: Option<u8>,
    polarity: String,
    pair_id: String,
    source_anchor_position: Option<usize>,
    source_manifest_sha256: String,
}

fn case_metadata(data: &DataSet, episode: &TrainingEpisode) -> AnyResult<CaseMetadata> {
    let fields: Vec<_> = episode.name.split(':').collect();
    if fields.len() < 3 || fields[1] != "correction" {
        return Err("A3 route case is not an authored correction".into());
    }
    let manifest = data
        .manifest
        .get("correction_cases")
        .and_then(Value::as_array)
        .ok_or("A2 correction manifest absent")?;
    let row = manifest
        .iter()
        .find(|row| row.get("name").and_then(Value::as_str) == Some(episode.name.as_str()))
        .ok_or("correction missing from pinned manifest")?;
    if row.get("source_id").and_then(Value::as_u64) != Some(episode.source_id)
        || row.get("prompt_tokens").and_then(Value::as_u64) != episode.prompt_len.map(|n| n as u64)
    {
        return Err("correction manifest source/prompt mismatch".into());
    }
    let new_style = fields.len() == 5 && fields[3].starts_with("style-");
    let (style, polarity, pair_id, source_anchor_position, source_manifest_sha256) = if new_style {
        let style: u8 = fields[3]["style-".len()..].parse()?;
        if !matches!(fields[4], "positive" | "negative") {
            return Err("invalid correction polarity".into());
        }
        (
            Some(style),
            fields[4].to_owned(),
            row.get("pair_id")
                .and_then(Value::as_str)
                .ok_or("new correction pair ID absent")?
                .to_owned(),
            Some(usize::try_from(
                row.get("source_anchor_position")
                    .and_then(Value::as_u64)
                    .ok_or("new correction source anchor absent")?,
            )?),
            row.get("sha256_full_text")
                .and_then(Value::as_str)
                .ok_or("new correction text hash absent")?
                .to_owned(),
        )
    } else {
        if fields.len() != 3 || fields[0] != "dev" {
            return Err("unexpected inherited correction name".into());
        }
        (
            None,
            "inherited".to_owned(),
            episode.name.clone(),
            None,
            row.get("sha256_tokens")
                .and_then(Value::as_str)
                .ok_or("inherited correction token hash absent")?
                .to_owned(),
        )
    };
    if row
        .get("split")
        .and_then(Value::as_str)
        .is_some_and(|split| split != fields[0])
    {
        return Err("correction manifest split mismatch".into());
    }
    Ok(CaseMetadata {
        split: fields[0].to_owned(),
        entity: fields[2].to_owned(),
        style,
        polarity,
        pair_id,
        source_anchor_position,
        source_manifest_sha256,
    })
}

fn wrong_entity_value_span(
    prompt: &str,
    entity: &str,
) -> AnyResult<Option<(String, usize, usize)>> {
    const PREFIX: &str = "A separate ledger lists Project ";
    const STATUS: &str = " with status ";
    const SUFFIX: &str = " for its own queue.";
    let Some(marker) = prompt.find(PREFIX) else {
        return Ok(None);
    };
    let name_start = marker + PREFIX.len();
    let status_offset = prompt[name_start..]
        .find(STATUS)
        .ok_or("authored distractor lacks status marker")?;
    let wrong_entity = &prompt[name_start..name_start + status_offset];
    if wrong_entity.is_empty() || wrong_entity == entity {
        return Err("distractor does not identify a different entity".into());
    }
    let value_start = name_start + status_offset + STATUS.len();
    let value_end = value_start
        + prompt[value_start..]
            .find(SUFFIX)
            .ok_or("authored distractor lacks status suffix")?;
    if !matches!(&prompt[value_start..value_end], "allowed" | "denied") {
        return Err("authored distractor status changed".into());
    }
    Ok(Some((wrong_entity.to_owned(), value_start, value_end)))
}

fn overlapping_prefix_positions(
    episode: &TrainingEpisode,
    prompt: &str,
    start: usize,
    end: usize,
) -> AnyResult<Vec<usize>> {
    let prefix_len = episode.prompt_len.ok_or("missing prompt length")?;
    let mut offset = 0usize;
    let mut overlap = Vec::new();
    let mut exact = Vec::with_capacity(prompt.len());
    for (position, token_bytes) in episode.token_bytes.iter().take(prefix_len).enumerate() {
        let next = offset
            .checked_add(token_bytes.len())
            .ok_or("token byte offset overflow")?;
        if offset < end && next > start {
            overlap.push(position);
        }
        exact.extend_from_slice(token_bytes);
        offset = next;
    }
    if exact != prompt.as_bytes() || offset != prompt.len() || overlap.is_empty() {
        return Err("pinned prompt/token alignment changed".into());
    }
    Ok(overlap)
}

pub fn collect_case(
    model: &IntegratedModel,
    episode: &TrainingEpisode,
    data: &DataSet,
) -> AnyResult<Option<RouteCase>> {
    let Some(prompt_len) = episode.prompt_len else {
        return Ok(None);
    };
    episode.validate(&model.config)?;
    let metadata = case_metadata(data, episode)?;
    let target_first = *episode
        .tokens
        .get(prompt_len)
        .ok_or("first answer token absent")?;
    let source_position = episode.source_targets[prompt_len]
        .ok_or("first answer lacks fit-only source annotation")?;
    if metadata
        .source_anchor_position
        .is_some_and(|position| position != source_position)
    {
        return Err("manifest and first-answer source anchor disagree".into());
    }
    let source_event_id = source_position as u64 + 1;
    let source_token = episode.tokens[source_position];
    let prompt_edit = data.prompt_edit(episode)?;
    let (wrong_entity, wrong_positions) = if metadata.style.is_some() {
        let edit = prompt_edit.ok_or("new correction prompt edit absent")?;
        let (name, start, end) = wrong_entity_value_span(&edit.original, &metadata.entity)?
            .ok_or("new correction distractor absent")?;
        let positions = overlapping_prefix_positions(episode, &edit.original, start, end)?;
        (Some(name), positions)
    } else {
        (None, Vec::new())
    };

    let mut session = Session::new(model.runtime(), episode.source_id)?;
    let mut committed = Vec::new();
    let mut positive_key_input = None;
    for position in 0..prompt_len {
        let read = session.read(model.runtime(), true)?;
        let observation = session.observe(
            model.runtime(),
            u32::from(episode.tokens[position]),
            &episode.token_bytes[position],
            read.selected_token(),
        )?;
        let delayed_source = observation
            .event_id
            .checked_sub(ADDRESS_COMMIT_DELAY)
            .filter(|&id| id > 0);
        if delayed_source == Some(source_event_id) {
            positive_key_input = Some(observation.key_input);
        }
        if let Some(record_id) = observation.record_id {
            let indexed_source = observation
                .indexed_source_event_id
                .ok_or("committed record lacks indexed source")?;
            if delayed_source != Some(indexed_source) {
                return Err("committed record has wrong delayed source".into());
            }
            let source_index = usize::try_from(indexed_source - 1)?;
            committed.push(CommittedKey {
                source_position: source_index,
                source_event_id: indexed_source,
                record_id,
                token: episode.tokens[source_index],
                key_input: observation.key_input,
                code: observation.key,
            });
        }
    }
    let positive_key_input = positive_key_input.ok_or("positive delayed key not observed")?;
    let positive_committed_index = committed
        .iter()
        .position(|entry| entry.source_event_id == source_event_id);
    let negative_indices: Vec<_> = committed
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            wrong_positions
                .contains(&entry.source_position)
                .then_some(index)
        })
        .collect();
    if negative_indices.contains(&positive_committed_index.unwrap_or(usize::MAX)) {
        return Err("positive source also labeled wrong-entity negative".into());
    }
    let read = session.read(model.runtime(), true)?;
    let positive_code = model.encoder.encode(positive_key_input)?.codes;
    let source_coarse = usize::from(positive_code[0]);
    let lane = model.encoder.score_lane(read.input, 0)?;
    if lane.best != read.query[0] {
        return Err("hard query code disagrees with selected-row score".into());
    }
    let positive_score = lane.scores[source_coarse];
    let positive_coarse_rank = 1 + lane
        .scores
        .iter()
        .enumerate()
        .filter(|(code, score)| {
            **score > positive_score || (**score == positive_score && *code < source_coarse)
        })
        .count();
    let positive_record_id = positive_committed_index.map(|index| committed[index].record_id);
    let candidates = &read.candidates[..read.candidate_count];
    let source_admitted = positive_record_id
        .is_some_and(|id| candidates.iter().any(|candidate| candidate.record_id == id));
    let source_ranked = positive_record_id.is_some_and(|id| {
        read.ranked
            .and_then(|index| candidates.get(index))
            .is_some_and(|candidate| candidate.record_id == id)
    });
    let source_selected = positive_record_id.is_some_and(|id| {
        read.selected
            .and_then(|index| candidates.get(index))
            .is_some_and(|candidate| candidate.record_id == id)
    });
    let same_coarse_committed = committed
        .iter()
        .filter(|entry| entry.code[0] == read.query[0])
        .count();
    let hard = HardRouteDiagnostic {
        query_code: read.query,
        positive_code,
        positive_committed: positive_committed_index.is_some(),
        positive_coarse_rank,
        positive_coarse_score: positive_score,
        query_coarse_score: lane.best_score,
        query_coarse_margin: lane.margin,
        same_coarse_committed,
        source_admitted,
        source_ranked,
        source_selected,
        gate_open: read.gate_enabled,
        candidate_count: read.candidate_count,
        search_incomplete: read.search_incomplete,
    };
    let frozen_prefix_blake3 = blake3::hash(&serde_json::to_vec(&(
        episode.source_id,
        &episode.tokens[..prompt_len],
        &episode.token_bytes[..prompt_len],
    ))?)
    .to_hex()
    .to_string();
    let frozen_address_context_blake3 = blake3::hash(&serde_json::to_vec(&(
        &read.input.context[..usize::from(read.input.context_len)],
        &positive_key_input.context[..usize::from(positive_key_input.context_len)],
    ))?)
    .to_hex()
    .to_string();
    Ok(Some(RouteCase {
        episode_name: episode.name.clone(),
        split: metadata.split,
        entity: metadata.entity,
        style: metadata.style,
        polarity: metadata.polarity,
        pair_id: metadata.pair_id,
        source_id: episode.source_id,
        prompt_len,
        target_first,
        source_position,
        source_token,
        source_event_id,
        positive_key_input,
        positive_committed_index,
        negative_indices,
        wrong_entity_value_positions: wrong_positions,
        wrong_entity,
        query: read.input,
        committed,
        hard,
        frozen_prefix_blake3,
        frozen_address_context_blake3,
        source_manifest_sha256: metadata.source_manifest_sha256,
    }))
}

pub fn collect_cases(
    model: &IntegratedModel,
    episodes: &[TrainingEpisode],
    data: &DataSet,
) -> AnyResult<Vec<RouteCase>> {
    let mut cases = Vec::new();
    for episode in episodes {
        if let Some(case) = collect_case(model, episode, data)? {
            cases.push(case);
        }
    }
    Ok(cases)
}

pub fn collect_fit_pairs(
    model: &IntegratedModel,
    episodes: &[TrainingEpisode],
    data: &DataSet,
) -> AnyResult<RouteCollection> {
    let cases = collect_cases(model, episodes, data)?;
    let mut entities = BTreeSet::new();
    let mut by_pair: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, case) in cases.iter().enumerate() {
        if case.split != "fit" || case.style.is_none() {
            return Err("A3 fit collection contains non-fit or inherited correction".into());
        }
        entities.insert(case.entity.clone());
        by_pair.entry(case.pair_id.clone()).or_default().push(index);
    }
    let entity_groups: Vec<_> = entities.into_iter().collect();
    let mut pairs = Vec::with_capacity(by_pair.len());
    let mut skipped_pair_ids = Vec::new();
    let mut skipped_pair_reasons = Vec::new();
    for (pair_id, indices) in by_pair {
        if indices.len() != 2 {
            return Err(format!("pair {pair_id} lacks two worlds").into());
        }
        let first = &cases[indices[0]];
        let second = &cases[indices[1]];
        if first.entity != second.entity
            || first.style != second.style
            || first.polarity == second.polarity
        {
            return Err(format!("pair {pair_id} has inconsistent worlds").into());
        }
        let (Some(first_positive), Some(second_positive)) = (
            first.positive_committed_index,
            second.positive_committed_index,
        ) else {
            skipped_pair_reasons.push((pair_id.clone(), "annotated positive not committed".into()));
            skipped_pair_ids.push(pair_id);
            continue;
        };
        if first.negative_indices.is_empty() || second.negative_indices.is_empty() {
            skipped_pair_reasons.push((
                pair_id.clone(),
                "explicit wrong-entity value not committed".into(),
            ));
            skipped_pair_ids.push(pair_id);
            continue;
        }
        let group = entity_groups
            .binary_search(&first.entity)
            .map_err(|_| "fit entity group absent")?;
        let entity_group = u16::try_from(group)?;
        let to_world = |case: &RouteCase, positive_index: usize| HardAddressWorld {
            query: case.query,
            positive_index,
            negative_indices: case.negative_indices.clone(),
            committed_keys: case.committed.iter().map(|key| key.key_input).collect(),
        };
        pairs.push(HardAddressPair {
            entity_group,
            worlds: [
                to_world(first, first_positive),
                to_world(second, second_positive),
            ],
        });
    }
    Ok(RouteCollection {
        cases,
        pairs,
        skipped_pair_ids,
        skipped_pair_reasons,
        entity_groups,
    })
}
