use std::collections::{BTreeMap, BTreeSet};
use std::num::{NonZeroU16, NonZeroUsize};

use uor_r4_core::prime_route_attention::{
    compile_spin_manifest, GeometricAddress, ManifestProvenance, PhaseQ29, PrimeRegistry,
    RouteSentence, SemanticAtom, SpinTorsionState, UnitS3Q30, ZPhi, ZeroPowerBridge,
};
use uor_r4_core::prime_route_geometric_attention::{
    AttentionCandidateTrace, AttentionControl, AttentionGeometryIntervention, AttentionQueryPolicy,
    AttentionRowKey, AttentionRowSource, CausalAttentionState, GeometricAttentionArtifact,
    ATTENTION_ADJACENT_SPIN_ROWS, ATTENTION_MAX_CANDIDATE_ENTRIES_PER_QUERY,
    ATTENTION_ROWS_PER_QUERY, ATTENTION_ZETA_CHANNELS,
};

fn label(seed: &str) -> String {
    format!("blake3:{}", blake3::hash(seed.as_bytes()).to_hex())
}

fn provenance() -> ManifestProvenance {
    ManifestProvenance {
        tokenizer_cid: label("attention-tokenizer"),
        corpus_cid: label("attention-corpus"),
        compiler_cid: label("attention-compiler"),
        cost_profile_cid: label("attention-cost-profile"),
    }
}

fn registry() -> PrimeRegistry {
    PrimeRegistry::compile(
        &["a", "b", "c", "d", "e", "p", "q", "x", "y", "z"]
            .into_iter()
            .map(|id| SemanticAtom {
                semantic_atom_id: id.to_owned(),
                payload_cid: label(&format!("payload-{id}")),
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn address(
    registry: &PrimeRegistry,
    id: &str,
    torsion: f64,
    spin_lane: usize,
    radial_lane: i64,
) -> GeometricAddress {
    let binding = registry.binding_for_id(id).unwrap();
    let r4 = match spin_lane % 4 {
        0 => [1.0, 0.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0, 0.0],
        2 => [0.0, 0.0, 1.0, 0.0],
        _ => [0.0, 0.0, 0.0, 1.0],
    };
    GeometricAddress {
        atom: binding.atom,
        spin: SpinTorsionState::new(
            UnitS3Q30::from_r4(r4).unwrap(),
            PhaseQ29::from_radians(0.0).unwrap(),
            PhaseQ29::from_radians(torsion).unwrap(),
        )
        .unwrap(),
        radial: ZPhi::new(radial_lane, radial_lane + 1),
        payload_cid: binding.payload_cid.clone(),
    }
}

#[derive(Clone)]
struct FixtureRoutes {
    a: GeometricAddress,
    b: GeometricAddress,
    c0: GeometricAddress,
    c1: GeometricAddress,
    d: GeometricAddress,
    e: GeometricAddress,
    p: GeometricAddress,
    q: GeometricAddress,
    x: GeometricAddress,
    x_alt: GeometricAddress,
    y: GeometricAddress,
    z: GeometricAddress,
}

fn fixture(maximum_candidates: u16) -> (GeometricAttentionArtifact, FixtureRoutes) {
    let registry = registry();
    let routes = FixtureRoutes {
        a: address(&registry, "a", 0.0, 0, 1),
        b: address(&registry, "b", 0.05, 0, 2),
        c0: address(&registry, "c", 0.1, 0, 3),
        c1: address(&registry, "c", 0.6, 0, 4),
        d: address(&registry, "d", -0.05, 1, 5),
        e: address(&registry, "e", -0.1, 1, 6),
        p: address(&registry, "p", 0.15, 0, 7),
        q: address(&registry, "q", -0.15, 1, 8),
        x: address(&registry, "x", 0.2, 0, 9),
        x_alt: address(&registry, "x", 1.0, 0, 90),
        y: address(&registry, "y", -0.2, 2, 10),
        z: address(&registry, "z", 0.3, 3, 11),
    };
    let sentences = vec![
        RouteSentence {
            sentence_id: "ordered-a".to_owned(),
            routes: vec![
                routes.a.clone(),
                routes.b.clone(),
                routes.c0.clone(),
                routes.x.clone(),
            ],
        },
        RouteSentence {
            sentence_id: "ordered-d".to_owned(),
            routes: vec![
                routes.d.clone(),
                routes.b.clone(),
                routes.c0.clone(),
                routes.y.clone(),
            ],
        },
        RouteSentence {
            sentence_id: "last-two-e".to_owned(),
            routes: vec![routes.e.clone(), routes.c0.clone(), routes.z.clone()],
        },
        RouteSentence {
            sentence_id: "torsion-c1".to_owned(),
            routes: vec![routes.a.clone(), routes.c1.clone(), routes.x.clone()],
        },
        RouteSentence {
            sentence_id: "torsion-alt".to_owned(),
            routes: vec![routes.a.clone(), routes.c0.clone(), routes.x_alt.clone()],
        },
        RouteSentence {
            sentence_id: "last-one-p".to_owned(),
            routes: vec![routes.p.clone(), routes.x.clone()],
        },
        RouteSentence {
            sentence_id: "last-one-q".to_owned(),
            routes: vec![routes.q.clone(), routes.y.clone()],
        },
    ];
    let compilation = compile_spin_manifest(
        &sentences,
        registry,
        ZeroPowerBridge::ContinuousNull,
        provenance(),
        NonZeroU16::new(maximum_candidates).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap();
    let attention =
        GeometricAttentionArtifact::compile_from_manifest_witnesses(&compilation.manifest).unwrap();
    assert_eq!(
        attention.manifest_kappa(),
        compilation.manifest.manifest_kappa
    );
    (attention, routes)
}

fn state(
    attention: &GeometricAttentionArtifact,
    history: &[&GeometricAddress],
) -> CausalAttentionState {
    attention
        .causal_state_from_history(
            &history
                .iter()
                .map(|address| (*address).clone())
                .collect::<Vec<_>>(),
        )
        .unwrap()
}

fn candidate<'a>(
    trace: &'a uor_r4_core::prime_route_geometric_attention::GeometricAttentionTrace,
    next: &GeometricAddress,
) -> &'a AttentionCandidateTrace {
    trace
        .candidates
        .iter()
        .find(|candidate| candidate.next == *next)
        .expect("fixture candidate is reachable")
}

fn support(
    trace: &uor_r4_core::prime_route_geometric_attention::GeometricAttentionTrace,
) -> BTreeSet<GeometricAddress> {
    trace
        .candidates
        .iter()
        .map(|candidate| candidate.next.clone())
        .collect()
}

#[test]
fn lookup_is_strictly_row_and_candidate_bounded() {
    let (attention, routes) = fixture(3);
    let trace = attention
        .query(
            &state(&attention, &[&routes.a, &routes.b, &routes.c0]),
            AttentionControl::RealGeometry,
        )
        .unwrap();
    let bounds = attention.lookup_bounds();

    let policy = AttentionQueryPolicy::PrimaryThenAdjacentSpinFallbackV1;
    assert_eq!(trace.query_policy, policy);
    assert_eq!(trace.query_policy_kappa, policy.identity_kappa());
    assert_eq!(trace.manifest_kappa, attention.manifest_kappa());
    assert_eq!(trace.rows_read.len(), ATTENTION_ROWS_PER_QUERY);
    assert_eq!(bounds.rows_per_query, ATTENTION_ROWS_PER_QUERY);
    assert_eq!(
        trace.candidate_entry_ceiling,
        bounds.candidate_entries_per_query
    );
    assert_eq!(
        trace.candidate_ceiling,
        bounds.unique_candidates_after_ceiling
    );
    assert_eq!(
        trace
            .rows_read
            .iter()
            .filter(|row| row.source == AttentionRowSource::AdjacentSpin)
            .count(),
        ATTENTION_ADJACENT_SPIN_ROWS
    );
    assert!(trace.rows_read.iter().enumerate().all(|(slot_index, row)| {
        row.slot_index == slot_index
            && row.hit == row.physical_row_present
            && row.candidate_entries_available <= 3
            && row.candidate_entries_examined <= row.candidate_entries_available
            && row.candidate_entries_admitted <= row.candidate_entries_examined
    }));
    assert_eq!(
        trace.candidate_entries_available,
        trace
            .rows_read
            .iter()
            .map(|row| row.candidate_entries_available)
            .sum::<usize>()
    );
    assert_eq!(
        trace.candidate_entries_examined,
        trace
            .rows_read
            .iter()
            .map(|row| row.candidate_entries_examined)
            .sum::<usize>()
    );
    assert_eq!(
        trace.candidate_entries_admitted,
        trace
            .rows_read
            .iter()
            .map(|row| row.candidate_entries_admitted)
            .sum::<usize>()
    );
    assert!(trace.candidate_entries_available <= bounds.candidate_entries_per_query);
    assert!(trace.candidate_entries_examined <= bounds.candidate_entries_per_query);
    assert!(trace.candidate_entries_admitted <= bounds.candidate_entries_per_query);
    assert!(trace.candidate_entries_available <= ATTENTION_MAX_CANDIDATE_ENTRIES_PER_QUERY);
    assert!(trace.candidate_entries_examined <= ATTENTION_MAX_CANDIDATE_ENTRIES_PER_QUERY);
    assert!(trace.candidate_entries_admitted <= ATTENTION_MAX_CANDIDATE_ENTRIES_PER_QUERY);
    assert!(trace.candidates.len() <= bounds.unique_candidates_after_ceiling);
    assert_eq!(trace.geometry_evaluations, trace.candidates.len());
    assert_eq!(trace.selected.as_ref(), trace.candidates.first());

    let sources = trace
        .rows_read
        .iter()
        .map(|row| row.source)
        .collect::<Vec<_>>();
    assert_eq!(
        sources,
        vec![
            AttentionRowSource::LastOne,
            AttentionRowSource::LastTwo,
            AttentionRowSource::OrderedSentence,
            AttentionRowSource::Divisor,
            AttentionRowSource::AdjacentSpin,
            AttentionRowSource::AdjacentSpin,
            AttentionRowSource::AdjacentSpin,
        ]
    );
}

#[test]
fn compilation_and_lookup_are_deterministic_and_manifest_bound() {
    let (first, routes) = fixture(8);
    let (second, _) = fixture(8);
    let causal = state(&first, &[&routes.a, &routes.b, &routes.c0]);
    let first_trace = first
        .query(&causal, AttentionControl::RealGeometry)
        .unwrap();
    let repeated_trace = first
        .query(&causal, AttentionControl::RealGeometry)
        .unwrap();
    let second_trace = second
        .query(&causal, AttentionControl::RealGeometry)
        .unwrap();

    assert_eq!(first.manifest_kappa(), second.manifest_kappa());
    assert_eq!(first.compile_stats(), second.compile_stats());
    assert_eq!(first_trace, repeated_trace);
    assert_eq!(first_trace, second_trace);
    assert_eq!(first_trace.manifest_kappa, first.manifest_kappa());
    assert_eq!(ATTENTION_ZETA_CHANNELS, [0, 1, 2, 3, 5, 8, 13, 21]);
}

#[test]
fn real_permuted_and_count_controls_have_identical_support_and_budget() {
    let (attention, routes) = fixture(8);
    let causal = state(&attention, &[&routes.a, &routes.b, &routes.c0]);
    let real = attention
        .query(&causal, AttentionControl::RealGeometry)
        .unwrap();
    let permuted = attention
        .query(&causal, AttentionControl::PermutedGeometry)
        .unwrap();
    let count_only = attention
        .query(&causal, AttentionControl::CountOnly)
        .unwrap();

    assert_eq!(support(&real), support(&permuted));
    assert_eq!(support(&real), support(&count_only));
    assert_eq!(real.rows_read, permuted.rows_read);
    assert_eq!(real.rows_read, count_only.rows_read);
    assert_eq!(
        real.candidate_entries_examined,
        permuted.candidate_entries_examined
    );
    assert_eq!(
        real.candidate_entries_examined,
        count_only.candidate_entries_examined
    );
    assert_eq!(real.geometry_evaluations, permuted.geometry_evaluations);
    assert_eq!(real.geometry_evaluations, count_only.geometry_evaluations);
    assert!(permuted
        .candidates
        .iter()
        .any(|candidate| candidate.geometry_source_next != candidate.next));
    assert!(count_only
        .candidates
        .iter()
        .all(|candidate| candidate.ranking_energy == Default::default()));
}

#[test]
fn last_one_last_two_and_ordered_sentence_interventions_are_observable() {
    let (attention, routes) = fixture(8);

    let from_p = attention
        .query(
            &state(&attention, &[&routes.p]),
            AttentionControl::CountOnly,
        )
        .unwrap();
    let from_q = attention
        .query(
            &state(&attention, &[&routes.q]),
            AttentionControl::CountOnly,
        )
        .unwrap();
    assert!(candidate(&from_p, &routes.x).source_counts.last_one > 0);
    assert!(!support(&from_p).contains(&routes.y));
    assert!(candidate(&from_q, &routes.y).source_counts.last_one > 0);
    assert!(!support(&from_q).contains(&routes.x));

    let from_b_c = attention
        .query(
            &state(&attention, &[&routes.b, &routes.c0]),
            AttentionControl::CountOnly,
        )
        .unwrap();
    let from_e_c = attention
        .query(
            &state(&attention, &[&routes.e, &routes.c0]),
            AttentionControl::CountOnly,
        )
        .unwrap();
    assert!(candidate(&from_b_c, &routes.x).source_counts.last_two > 0);
    assert_eq!(candidate(&from_e_c, &routes.x).source_counts.last_two, 0);
    assert!(candidate(&from_e_c, &routes.z).source_counts.last_two > 0);
    assert_eq!(candidate(&from_b_c, &routes.z).source_counts.last_two, 0);

    let ordered_a = attention
        .query(
            &state(&attention, &[&routes.a, &routes.b, &routes.c0]),
            AttentionControl::CountOnly,
        )
        .unwrap();
    let ordered_d = attention
        .query(
            &state(&attention, &[&routes.d, &routes.b, &routes.c0]),
            AttentionControl::CountOnly,
        )
        .unwrap();
    assert!(
        candidate(&ordered_a, &routes.x)
            .source_counts
            .ordered_sentence
            > 0
    );
    assert_eq!(
        candidate(&ordered_a, &routes.y)
            .source_counts
            .ordered_sentence,
        0
    );
    assert!(
        candidate(&ordered_d, &routes.y)
            .source_counts
            .ordered_sentence
            > 0
    );
    assert_eq!(
        candidate(&ordered_d, &routes.x)
            .source_counts
            .ordered_sentence,
        0
    );
}

#[test]
fn isolated_phase_and_torsion_interventions_change_final_ordering() {
    let (attention, routes) = fixture(8);
    let phase_state = state(&attention, &[&routes.b, &routes.c0]);
    let phase_baseline = attention
        .query(&phase_state, AttentionControl::RealGeometry)
        .unwrap();
    let phase_shifted = attention
        .query_with_intervention(
            &phase_state,
            AttentionControl::RealGeometry,
            AttentionGeometryIntervention::PhaseDeltaOffset(PhaseQ29::from_radians(2.0).unwrap()),
        )
        .unwrap();
    assert_eq!(phase_baseline.rows_read, phase_shifted.rows_read);
    assert_eq!(support(&phase_baseline), support(&phase_shifted));
    assert_eq!(
        phase_baseline
            .candidates
            .iter()
            .map(|candidate| (candidate.next.clone(), candidate.source_counts))
            .collect::<BTreeMap<_, _>>(),
        phase_shifted
            .candidates
            .iter()
            .map(|candidate| (candidate.next.clone(), candidate.source_counts))
            .collect::<BTreeMap<_, _>>()
    );
    assert_ne!(
        phase_baseline
            .candidates
            .iter()
            .map(|candidate| candidate.next.clone())
            .collect::<Vec<_>>(),
        phase_shifted
            .candidates
            .iter()
            .map(|candidate| candidate.next.clone())
            .collect::<Vec<_>>(),
        "isolated phase offset must change final ordering"
    );

    let torsion_state = state(&attention, &[&routes.a, &routes.c0]);
    let torsion_baseline = attention
        .query(&torsion_state, AttentionControl::RealGeometry)
        .unwrap();
    let torsion_shifted = attention
        .query_with_intervention(
            &torsion_state,
            AttentionControl::RealGeometry,
            AttentionGeometryIntervention::TorsionDeltaOffset(PhaseQ29::from_radians(0.8).unwrap()),
        )
        .unwrap();
    assert_eq!(torsion_baseline.rows_read, torsion_shifted.rows_read);
    assert_eq!(support(&torsion_baseline), support(&torsion_shifted));
    let baseline_x = torsion_baseline
        .candidates
        .iter()
        .position(|candidate| candidate.next == routes.x)
        .unwrap();
    let baseline_x_alt = torsion_baseline
        .candidates
        .iter()
        .position(|candidate| candidate.next == routes.x_alt)
        .unwrap();
    let shifted_x = torsion_shifted
        .candidates
        .iter()
        .position(|candidate| candidate.next == routes.x)
        .unwrap();
    let shifted_x_alt = torsion_shifted
        .candidates
        .iter()
        .position(|candidate| candidate.next == routes.x_alt)
        .unwrap();
    assert!(baseline_x < baseline_x_alt);
    assert!(shifted_x_alt < shifted_x);
    assert_eq!(
        candidate(&torsion_baseline, &routes.x)
            .measured_energy
            .torsion,
        0
    );
    assert!(
        candidate(&torsion_shifted, &routes.x)
            .measured_energy
            .torsion
            > 0
    );
}

#[test]
fn unseen_global_prefixes_collapse_when_suffix_matches_and_sentence_row_misses() {
    let (attention, routes) = fixture(8);
    let mut from_p = attention
        .query(
            &state(&attention, &[&routes.p, &routes.b, &routes.c0]),
            AttentionControl::RealGeometry,
        )
        .unwrap();
    let mut from_q = attention
        .query(
            &state(&attention, &[&routes.q, &routes.b, &routes.c0]),
            AttentionControl::RealGeometry,
        )
        .unwrap();

    assert_eq!(
        from_p.rows_read[2].source,
        AttentionRowSource::OrderedSentence
    );
    assert_eq!(
        from_q.rows_read[2].source,
        AttentionRowSource::OrderedSentence
    );
    assert!(!from_p.rows_read[2].hit);
    assert!(!from_q.rows_read[2].hit);
    assert_ne!(from_p.rows_read[2].key, from_q.rows_read[2].key);
    assert_eq!(from_p.candidates, from_q.candidates);
    assert_eq!(from_p.selected, from_q.selected);

    // The distinct causal sentence kappas are the only trace difference. The
    // current bounded layer therefore does not establish held-out global-
    // prefix differentiation when exact IS recall misses.
    from_p.rows_read[2].key = AttentionRowKey::OrderedSentence("unseen".to_owned());
    from_q.rows_read[2].key = AttentionRowKey::OrderedSentence("unseen".to_owned());
    assert_eq!(from_p, from_q);
}

#[test]
fn trace_names_every_row_key_source_count_energy_and_selected_support() {
    let (attention, routes) = fixture(8);
    let trace = attention
        .query(
            &state(&attention, &[&routes.a, &routes.b, &routes.c0]),
            AttentionControl::RealGeometry,
        )
        .unwrap();

    assert!(matches!(
        trace.rows_read[0].key,
        AttentionRowKey::LastOne(_)
    ));
    assert!(matches!(
        trace.rows_read[1].key,
        AttentionRowKey::LastTwo { .. }
    ));
    assert!(matches!(
        trace.rows_read[2].key,
        AttentionRowKey::OrderedSentence(_)
    ));
    assert!(matches!(
        trace.rows_read[3].key,
        AttentionRowKey::Divisor(_)
    ));
    assert!(trace.rows_read[4..]
        .iter()
        .all(|row| matches!(row.key, AttentionRowKey::AdjacentSpin(_))));
    assert!(trace.candidates.iter().all(|candidate| {
        candidate.source_counts.source_breadth() > 0
            && candidate.source_counts.total() > 0
            && candidate.geometry_source_next == candidate.next
    }));
    let selected = trace.selected.unwrap();
    assert!(selected.source_counts.total() > 0);
    assert!(trace.tie_break_stages.len() >= 8);
}

#[test]
fn source_reachability_scan_excludes_future_input_learned_qk_and_population_scan() {
    let source = include_str!("../src/prime_route_geometric_attention.rs");
    for forbidden in [
        "softmax(",
        "learned_query",
        "learned_key",
        "query_matrix",
        "key_matrix",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden source token {forbidden}"
        );
    }

    let signature = source
        .split("pub fn query(")
        .nth(1)
        .unwrap()
        .split('{')
        .next()
        .unwrap();
    assert!(signature.contains("&CausalAttentionState"));
    assert!(signature.contains("AttentionControl"));
    assert!(!signature.contains("GeometricAddress"));
    assert!(!signature.contains("RouteSentence"));

    let lookup = source
        .split("// BEGIN GEOMETRIC_ATTENTION_BOUNDED_LOOKUP")
        .nth(1)
        .unwrap()
        .split("// END GEOMETRIC_ATTENTION_BOUNDED_LOOKUP")
        .next()
        .unwrap();
    for forbidden in [
        "f64",
        "libm",
        "zeta_phase_delta",
        "rebuild_witnesses",
        "RouteSentence",
        ".addresses",
    ] {
        assert!(
            !lookup.contains(forbidden),
            "lookup path contains forbidden population/float token {forbidden}"
        );
    }
}
