//! Focused schema-/4 local-path and occurrence-composition checks. Fitting and
//! generated-behavior tests exercise the same operator at their own boundary.
use super::*;
use crate::native_geometric::{Config, Document, Trainer};

fn fixture(context_tokens: usize, schema: &str) -> (Model, MemoryModel) {
    let documents = [Document {
        id: "occurrence-kernel-fixture".into(),
        text: "red green blue alpha beta gamma".into(),
    }];
    let mut trainer = Trainer::new(
        Config {
            context_tokens,
            ..Config::default()
        },
        &documents,
    )
    .unwrap();
    trainer.train_documents(&documents).unwrap();
    let model = trainer.compile().unwrap();
    let memory = MemoryModel {
        schema: schema.into(),
        baseline_artifact: model.artifact_cid().into(),
        cue_aliases: None,
        config: MemoryReadFitConfig {
            query_tokens: 4,
            source_offsets: 4,
            postings_per_address: 2,
            candidate_limit: 32,
            ..MemoryReadFitConfig::default()
        },
        source_shift: 2,
        posting_shift: 1,
        training: Vec::new(),
        rows: Vec::new(),
        fit_positions: 0,
        fit_schedule: None,
    };
    (model, memory)
}

fn observed(model: &Model, memory: &MemoryModel, tokens: &[u32]) -> MemoryState {
    let mut state = MemoryState::new(model, memory);
    for &token in tokens {
        state.observe(model, memory, token, &mut Work::default());
    }
    state.collect(model, memory, Control::Full, &mut Work::default());
    state
}

fn route(state: &MemoryState, sequence: u64, query: usize, source: usize) -> MemoryCandidate {
    state
        .candidates
        .iter()
        .find(|route| {
            route.sequence == sequence
                && route.features[1].value == ((query as u64) << 8) | source as u64
        })
        .copied()
        .expect("the declared cue route must be admitted")
}

fn product(model: &Model, left: u16, right: u16) -> u16 {
    model.geometry.products[model.geometry.row_bases[usize::from(left)] + usize::from(right)]
}

#[test]
fn occurrence_paths_compare_local_order_and_ignore_unrelated_prefix_and_gap() {
    let (model, memory) = fixture(32, OCCURRENCE_MEMORY_SCHEMA);
    let cue = 67;
    let first = 68;
    let value = 69;
    let before = observed(
        &model,
        &memory,
        &[BOS, cue, first, value, 70, 71, cue, first],
    );
    let after = observed(
        &model,
        &memory,
        &[BOS, 72, 73, cue, first, value, 74, 70, 71, 75, cue, first],
    );
    let left = route(&before, 3, 2, 2);
    let right = route(&after, 5, 2, 2);
    assert_eq!(&left.features[5..16], &right.features[5..16]);

    let source_path = product(
        &model,
        model.geometry.tokens[first as usize].leaf,
        model.geometry.tokens[value as usize].leaf,
    );
    let expected = product(
        &model,
        model.geometry.inverses[usize::from(source_path)],
        model.geometry.tokens[first as usize].leaf,
    );
    assert_eq!(left.features[5].value, u64::from(expected));
    assert_eq!(
        left.features[7].value,
        u64::from(model.geometry.orientation[usize::from(expected)])
    );
    for channel in 0..PHASE_CHANNELS {
        // Q contains `first`; S contains `first,value`. Full modular phase
        // subtraction therefore leaves -phase(value), before discretization.
        let expected_phase =
            0_u16.wrapping_sub(model.geometry.tokens[value as usize].phases[channel]);
        assert_eq!(
            left.features[8 + channel].value,
            u64::from(expected_phase >> 12)
        );
    }
}

#[test]
fn occurrence_local_paths_preserve_noncommuting_order_and_signed_state() {
    let (model, memory) = fixture(32, OCCURRENCE_MEMORY_SCHEMA);
    let (first, second) = (2..64_u32)
        .flat_map(|left| (2..64_u32).map(move |right| (left, right)))
        .find(|&(left, right)| {
            let left = model.geometry.tokens[left as usize].leaf;
            let right = model.geometry.tokens[right as usize].leaf;
            product(&model, left, right) != product(&model, right, left)
        })
        .unwrap();
    let cue = 200;
    let first_order = observed(&model, &memory, &[BOS, cue, first, second, 201, cue, 202]);
    let reverse_order = observed(&model, &memory, &[BOS, cue, second, first, 201, cue, 202]);
    let first_route = route(&first_order, 3, 2, 2);
    let reverse_route = route(&reverse_order, 3, 2, 2);
    assert_ne!(first_route.features[5], reverse_route.features[5]);
    assert_eq!(&first_route.features[8..16], &reverse_route.features[8..16]);
}

#[test]
fn occurrence_transport_and_phases_are_invariant_to_common_frame_changes() {
    let (model, memory) = fixture(32, OCCURRENCE_MEMORY_SCHEMA);
    let mut state = observed(&model, &memory, &[BOS, 67, 68, 69, 70, 71, 67, 68]);
    let expected: Vec<_> = state
        .candidates
        .iter()
        .map(|route| route.features)
        .collect();
    let gauge = model.geometry.tokens[103].leaf;
    state.pose = product(&model, gauge, state.pose);
    for phase in &mut state.phases {
        *phase = phase.wrapping_add(65_503);
    }
    for entry in &mut state.ring[..state.length] {
        entry.pose = product(&model, gauge, entry.pose);
        for phase in &mut entry.phases {
            *phase = phase.wrapping_add(65_503);
        }
    }
    state.collect(&model, &memory, Control::Full, &mut Work::default());
    assert_eq!(
        expected,
        state
            .candidates
            .iter()
            .map(|route| route.features)
            .collect::<Vec<_>>()
    );
}

#[test]
fn occurrence_union_counts_each_feature_and_prior_once_without_merging_equal_tokens() {
    let (mut model, mut memory) = fixture(32, OCCURRENCE_MEMORY_SCHEMA);
    let mut first = [MemoryFeature { kind: 0, value: 0 }; MEMORY_FEATURE_COUNT];
    for (index, feature) in first.iter_mut().enumerate() {
        *feature = MemoryFeature {
            kind: index as u8,
            value: index as u64,
        };
    }
    let mut second = first;
    second[1].value = 91;
    let mut independent = first;
    independent[2].value = 92;
    let token = 67;
    model.prior_scores[token as usize] = 101;
    let mut union: Vec<_> = first.into_iter().chain(second).chain(independent).collect();
    union.sort_unstable();
    union.dedup();
    memory.rows = union
        .into_iter()
        .map(|feature| MemoryWeight {
            feature,
            score: 1 + i32::from(feature.kind),
        })
        .collect();
    let mut state = MemoryState::new(&model, &memory);
    for (sequence, features) in [(10, first), (10, second), (10, first), (11, independent)] {
        state.candidates.push(MemoryCandidate {
            sequence,
            token,
            score: -999,
            features,
        });
    }
    let mut work = Work::default();
    state.compose_occurrences(&model, &memory, Control::Full, &mut work);
    assert_eq!(state.composed.len(), 2);
    assert_eq!(state.composed[0].sequence, 10);
    assert_eq!(state.composed[1].sequence, 11);
    assert_eq!(state.composed[0].feature_count, 19);
    assert_eq!(state.composed[1].feature_count, 18);
    assert_eq!(state.composed[0].score, 101 + (1..=18).sum::<i64>() + 2);
    assert_eq!(state.composed[1].score, 101 + (1..=18).sum::<i64>());
    assert_eq!(work.memory_composition_feature_offers, 72);
    assert_eq!(work.memory_composition_duplicate_features, 35);
    assert_eq!(work.memory_composed_candidates, 2);
    for occurrence in &state.composed {
        let features = &state.composition_features
            [occurrence.feature_start..occurrence.feature_start + occurrence.feature_count];
        assert!(features.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            features.iter().filter(|feature| feature.kind == 0).count(),
            1
        );
    }
    state.composed.clear();
    state.composition_features.clear();
    state.compose_occurrences(
        &model,
        &memory,
        Control::GeometryDisabled,
        &mut Work::default(),
    );
    let admitted = (1..=5).sum::<i64>() + 17 + 18;
    assert_eq!(state.composed[0].score, 101 + admitted + 2);
    assert_eq!(state.composed[1].score, 101 + admitted);

    // Exercise the complete collector, not only its reduction helper: fitted
    // row searches must occur exactly once per unique occurrence feature,
    // without first scoring every diagnostic flat route.
    let mut collected = observed(&model, &memory, &[BOS, 67, 68, 69, 70, 71, 67, 68]);
    let mut work = Work::default();
    collected.collect(&model, &memory, Control::Full, &mut work);
    assert!(!collected.composed.is_empty());
    assert_eq!(
        work.memory_score_lookups,
        collected.composition_features.len() as u64
    );
    assert!(work.memory_score_lookups <= work.memory_candidates * MEMORY_FEATURE_COUNT as u64);
    assert!(collected.candidates.iter().all(
        |candidate| candidate.score == i64::from(model.prior_scores[candidate.token as usize])
    ));
}

#[test]
fn occurrence_scratch_is_bounded_and_evicted_source_cues_are_rejected() {
    let (model, memory) = fixture(8, OCCURRENCE_MEMORY_SCHEMA);
    let mut state = MemoryState::new(&model, &memory);
    let route_capacity = state.candidates.capacity();
    let occurrence_capacity = state.composed.capacity();
    let feature_capacity = state.composition_features.capacity();
    let mut work = Work::default();
    for token in [BOS, 72, 73, 67, 68, 69, 74, 70, 71, 75, 67, 68] {
        state.observe(&model, &memory, token, &mut work);
        state.collect(&model, &memory, Control::Full, &mut work);
        assert_eq!(state.candidates.capacity(), route_capacity);
        assert_eq!(state.composed.capacity(), occurrence_capacity);
        assert_eq!(state.composition_features.capacity(), feature_capacity);
        assert!(state.composed.len() <= memory.config.candidate_limit);
        assert!(
            state.composition_features.len()
                <= memory.config.candidate_limit * MEMORY_FEATURE_COUNT
        );
    }
    assert!(!state
        .candidates
        .iter()
        .any(|route| { route.sequence == 5 && route.features[1].value == (2 << 8) | 2 }));
    assert!(work.memory_stale_rejections > 0);
    assert_eq!(
        state.view.composed_candidate_storage_bytes,
        occurrence_capacity * std::mem::size_of::<ComposedCandidate>()
    );
    assert_eq!(
        state.view.composition_feature_storage_bytes,
        feature_capacity * std::mem::size_of::<MemoryFeature>()
    );
}

#[test]
fn query_context_v3_keeps_global_relative_geometry_and_flat_routes() {
    let (model, memory) = fixture(32, QUERY_CONTEXT_MEMORY_SCHEMA);
    let mut state = observed(&model, &memory, &[BOS, 67, 68, 69, 70, 71, 67, 68]);
    let mut work = Work::default();
    state.collect(&model, &memory, Control::Full, &mut work);
    let candidate = route(&state, 3, 2, 2);
    let value = state.ring[3];
    let expected = product(
        &model,
        model.geometry.inverses[usize::from(value.pose)],
        state.pose,
    );
    assert_eq!(candidate.features[5].value, u64::from(expected));
    for channel in 0..PHASE_CHANNELS {
        assert_eq!(
            candidate.features[8 + channel].value,
            u64::from(state.phases[channel].wrapping_sub(value.phases[channel]) >> 12)
        );
    }
    assert!(state.composed.is_empty());
    assert!(state.composition_features.is_empty());
    assert_eq!(state.composed.capacity(), 0);
    assert_eq!(state.composition_features.capacity(), 0);
    assert_eq!(work.memory_composed_candidates, 0);
    assert_eq!(work.memory_composition_feature_offers, 0);
}
