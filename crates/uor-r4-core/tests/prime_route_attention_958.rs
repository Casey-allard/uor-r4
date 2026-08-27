use std::collections::BTreeSet;
use std::num::{NonZeroU16, NonZeroUsize};

use uor_r4_core::prime_route_attention::{
    compile_spin_manifest, ordered_sentence_key, zeta_grid_kappa, zeta_phase_delta,
    CompiledSpinManifest, GeometricAddress, ManifestProvenance, OrderedPrimeRoute,
    OrderedSentenceRouteState, PhaseQ29, PrimeAtom, PrimeRegistry, PrimeRouteCompilation,
    PrimeRouteError, RouteIntervention, RouteSentence, SemanticAtom, SemiprimeExpert,
    SpinTorsionState, TinyCanaryDimension, UnitS3Q30, ZPhi, ZeroPowerBridge, ZetaGridBinding,
    CANONICAL_MANIFEST_BODY_MAX_BYTES, CANONICAL_MANIFEST_MAX_BYTES, MANIFEST_MAX_ADDRESSES,
    MANIFEST_MAX_CANDIDATES_PER_ROW, MANIFEST_MAX_I1_ROWS, MANIFEST_MAX_I2_ROWS,
    MANIFEST_MAX_IS_ROWS, MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES, MANIFEST_MAX_TOTAL_ROWS,
    PHASE_FRACTION_BITS, PHASE_INTERVAL, PRIME_REGISTRY_DOMAIN, PRIME_REGISTRY_SCHEMA,
    PRIME_ROUTE_MANIFEST_DOMAIN, PRIME_ROUTE_MANIFEST_SCHEMA, QUANTIZATION_CHART_DOMAIN,
    QUANTIZATION_CHART_SCHEMA, RADIAL_RING, S3_S2_FRACTION_BITS, SPIN_CHART,
    TINY_CANARY_MAX_IDENTIFIER_BYTES, TINY_CANARY_MAX_OCCURRENCES,
    TINY_CANARY_MAX_ROUTES_PER_SENTENCE, TINY_CANARY_MAX_SENTENCES,
    TINY_CANARY_MAX_TOTAL_IDENTIFIER_BYTES, TINY_CANARY_MAX_TOTAL_ROUTES,
    TINY_CANARY_MAX_TRANSITIONS, ZETA_GRID_KAPPA_REFERENCE, ZETA_GRID_REVISION,
};
use uor_r4_core::zeta_zeros::ZETA_ZEROS;

const FIXTURE_MANIFEST_KAPPA_REFERENCE: &str =
    "blake3:48de73271c002b7f550c8459087270d0f34ad5ec0ca1fcf2d116684ded84ac63";

fn deterministic_label(seed: &str) -> String {
    format!("blake3:{}", blake3::hash(seed.as_bytes()).to_hex())
}

fn assert_canonical_blake3_label(label: &str) {
    let digest = label.strip_prefix("blake3:").expect("blake3 prefix");
    assert_eq!(digest.len(), 64);
    assert!(digest
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
}

fn json_value_kappa(value: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap();
    uor_addr::json::address_blake3(&bytes)
        .unwrap()
        .address
        .to_string()
}

fn rebind_outer_manifest_kappa(value: &mut serde_json::Value) {
    let kappa = json_value_kappa(&value["body"]);
    value["manifest_kappa"] = serde_json::Value::String(kappa);
}

fn tampered_manifest_bytes(
    compiled: &PrimeRouteCompilation,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> Vec<u8> {
    let mut value =
        serde_json::from_slice::<serde_json::Value>(&compiled.manifest.canonical_bytes().unwrap())
            .unwrap();
    mutate(&mut value);
    rebind_outer_manifest_kappa(&mut value);
    serde_json::to_vec(&value).unwrap()
}

fn assert_decode_rejects(bytes: &[u8], expected: &str) {
    let error = CompiledSpinManifest::decode_canonical(bytes).unwrap_err();
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?}, got {error}"
    );
}

fn semantic_atoms(payload_suffix: &str) -> Vec<SemanticAtom> {
    ["alpha", "beta", "gamma", "delta", "epsilon", "zeta"]
        .into_iter()
        .map(|id| SemanticAtom {
            semantic_atom_id: id.to_owned(),
            payload_cid: deterministic_label(&format!("{id}-{payload_suffix}")),
        })
        .collect()
}

fn address(registry: &PrimeRegistry, id: &str, lane: usize) -> GeometricAddress {
    let binding = registry.binding_for_id(id).expect("fixture binding");
    let r4 = match lane % 6 {
        0 => [1.0, 0.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0, 0.0],
        2 => [0.0, 0.0, 1.0, 0.0],
        3 => [0.0, 0.0, 0.0, 1.0],
        4 => [0.5, 0.5, 0.5, 0.5],
        _ => [0.5, -0.5, 0.5, -0.5],
    };
    let spin = SpinTorsionState::new(
        UnitS3Q30::from_r4(r4).expect("unit spin"),
        PhaseQ29::from_radians(lane as f64 * 0.125).expect("fiber"),
        PhaseQ29::from_radians(lane as f64 * -0.0625).expect("torsion"),
    )
    .expect("spin state");
    GeometricAddress {
        atom: binding.atom,
        spin,
        radial: ZPhi::new(lane as i64 + 1, lane as i64),
        payload_cid: binding.payload_cid.clone(),
    }
}

fn provenance() -> ManifestProvenance {
    ManifestProvenance {
        tokenizer_cid: deterministic_label("tokenizer"),
        corpus_cid: deterministic_label("corpus"),
        compiler_cid: deterministic_label("compiler"),
        cost_profile_cid: deterministic_label("m1-cost-profile"),
    }
}

fn fixture_sentences(registry: &PrimeRegistry) -> Vec<RouteSentence> {
    let a = address(registry, "alpha", 0);
    let b = address(registry, "beta", 1);
    let c = address(registry, "gamma", 2);
    let d = address(registry, "delta", 3);
    let e = address(registry, "epsilon", 4);
    vec![
        RouteSentence {
            sentence_id: "sentence-4".to_owned(),
            routes: vec![c.clone(), b.clone(), d.clone()],
        },
        RouteSentence {
            sentence_id: "sentence-2".to_owned(),
            routes: vec![a.clone(), b.clone(), d],
        },
        RouteSentence {
            sentence_id: "sentence-1".to_owned(),
            routes: vec![a.clone(), b.clone(), c.clone()],
        },
        RouteSentence {
            sentence_id: "sentence-5".to_owned(),
            routes: vec![e, b.clone(), c.clone()],
        },
        RouteSentence {
            sentence_id: "sentence-3".to_owned(),
            routes: vec![a, b, c],
        },
    ]
}

fn compile_custom(
    registry: &PrimeRegistry,
    sentences: &[RouteSentence],
    manifest_provenance: ManifestProvenance,
) -> PrimeRouteCompilation {
    compile_spin_manifest(
        sentences,
        registry.clone(),
        ZeroPowerBridge::ContinuousNull,
        manifest_provenance,
        NonZeroU16::new(8).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap()
}

fn compile_lengths(
    route_lengths: &[usize],
    workers: usize,
) -> Result<PrimeRouteCompilation, PrimeRouteError> {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1"))?;
    let a = address(&registry, "alpha", 0);
    let b = address(&registry, "beta", 1);
    let sentences = route_lengths
        .iter()
        .enumerate()
        .map(|(sentence_index, &route_length)| RouteSentence {
            sentence_id: format!("sentence-{sentence_index:04}"),
            routes: (0..route_length)
                .map(|route_index| {
                    if route_index.is_multiple_of(2) {
                        a.clone()
                    } else {
                        b.clone()
                    }
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    compile_spin_manifest(
        &sentences,
        registry,
        ZeroPowerBridge::ContinuousNull,
        provenance(),
        NonZeroU16::new(8).unwrap(),
        NonZeroUsize::new(workers).unwrap(),
    )
}

fn fixture(
    bridge: ZeroPowerBridge,
    workers: usize,
    maximum_candidates: u16,
    payload_suffix: &str,
) -> uor_r4_core::prime_route_attention::PrimeRouteCompilation {
    let registry = PrimeRegistry::compile(&semantic_atoms(payload_suffix)).expect("registry");
    let sentences = fixture_sentences(&registry);
    compile_spin_manifest(
        &sentences,
        registry,
        bridge,
        provenance(),
        NonZeroU16::new(maximum_candidates).expect("nonzero candidate bound"),
        NonZeroUsize::new(workers).expect("nonzero workers"),
    )
    .expect("compile fixture")
}

#[test]
fn typed_bridge_and_zphi_round_trip_are_exact() {
    assert_eq!(ZeroPowerBridge::ContinuousNull.value(), 0);
    assert_eq!(ZeroPowerBridge::DiscreteEmptyProduct.value(), 1);

    for value in [ZPhi::new(1, 0), ZPhi::new(2, 3), ZPhi::new(-5, 8)] {
        assert_eq!(
            value.times_phi().unwrap().times_phi_inverse().unwrap(),
            value
        );
        assert_eq!(
            value.times_phi_inverse().unwrap().times_phi().unwrap(),
            value
        );
    }
    assert!(ZPhi::new(i64::MAX, 1).times_phi().is_err());
    assert!(ZPhi::new(i64::MIN, 1).times_phi_inverse().is_err());
}

#[test]
fn six_prime_carriers_form_fifteen_square_free_semiprime_experts() {
    let primes = [5, 7, 11, 13, 17, 19].map(|value| PrimeAtom::new(value).unwrap());
    let mut products = BTreeSet::new();
    for left in 0..primes.len() {
        for right in left + 1..primes.len() {
            products.insert(
                SemiprimeExpert::new(primes[left], primes[right])
                    .unwrap()
                    .product(),
            );
        }
    }
    assert_eq!(products.len(), 15);
    let repeated = SemiprimeExpert::new(primes[0], primes[0]).unwrap();
    assert_eq!(repeated.factors(), [primes[0], primes[0]]);
    assert_eq!(repeated.product(), 25);

    let first = SemiprimeExpert::new(primes[0], primes[1]).unwrap();
    let second = SemiprimeExpert::new(primes[1], primes[2]).unwrap();
    assert_eq!(first.handoff(second), Some(primes[1]));
    assert_eq!(first.handoff(first), None);
}

#[test]
fn adjacent_repeated_route_atoms_are_retained_as_prime_square_experts() {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-repeat")).unwrap();
    let a = address(&registry, "alpha", 0);
    let b = address(&registry, "beta", 1);
    let sentences = vec![RouteSentence {
        sentence_id: "repeated-route".to_owned(),
        routes: vec![a.clone(), a, b],
    }];

    let compiled = compile_custom(&registry, &sentences, provenance());
    assert_eq!(
        compiled
            .manifest
            .experts
            .iter()
            .map(|record| {
                (
                    [record.factors[0].value(), record.factors[1].value()],
                    record.product,
                    record.occurrence_count,
                )
            })
            .collect::<Vec<_>>(),
        vec![([5, 5], 25, 1), ([5, 7], 35, 1)]
    );
    let bytes = compiled.manifest.canonical_bytes().unwrap();
    assert_eq!(
        CompiledSpinManifest::decode_canonical(&bytes).unwrap(),
        compiled.manifest
    );
}

#[test]
fn ordered_nlets_preserve_direction_and_factor_overlap_without_saturation() {
    let p5 = PrimeAtom::new(5).unwrap();
    let p7 = PrimeAtom::new(7).unwrap();
    let p11 = PrimeAtom::new(11).unwrap();
    let p13 = PrimeAtom::new(13).unwrap();
    let first = OrderedPrimeRoute::new(vec![p5, p7, p11]).unwrap();
    let reordered = OrderedPrimeRoute::new(vec![p11, p7, p5]).unwrap();
    let shifted = OrderedPrimeRoute::new(vec![p7, p11, p13]).unwrap();
    assert_eq!(first.factors(), reordered.factors());
    assert_ne!(
        first.ordered_kappa().unwrap(),
        reordered.ordered_kappa().unwrap()
    );
    assert_eq!(first.factor_overlap(&shifted), vec![p7, p11]);

    let largest = PrimeAtom::new(4_294_967_291).expect("largest u32 prime");
    let overflowing = OrderedPrimeRoute::new(vec![largest; 5]).unwrap();
    assert_eq!(overflowing.checked_product_u128(), None);
    assert_eq!(overflowing.factors(), &[largest; 5]);
}

#[test]
fn incremental_sentence_route_keys_match_the_diagnostic_full_history_wrapper() {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1")).unwrap();
    let history = [
        address(&registry, "alpha", 0),
        address(&registry, "beta", 1),
        address(&registry, "gamma", 2),
        address(&registry, "delta", 3),
    ];
    let mut state = OrderedSentenceRouteState::new().unwrap();
    assert_eq!(state.route_count(), 0);
    assert!(state.key().is_none());
    for (index, route) in history.iter().enumerate() {
        state.append(route).unwrap();
        assert_eq!(state.route_count(), u32::try_from(index + 1).unwrap());
        assert_eq!(
            state.key().unwrap(),
            &ordered_sentence_key(&history[..=index]).unwrap()
        );
    }

    let reversed = history.iter().rev().cloned().collect::<Vec<_>>();
    assert_ne!(
        state.key().unwrap(),
        &ordered_sentence_key(&reversed).unwrap()
    );
}

#[test]
fn incremental_sentence_route_rejects_an_inconsistent_hopf_observation_before_hashing() {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1")).unwrap();
    let mut inconsistent = address(&registry, "alpha", 0);
    let alternate = SpinTorsionState::new(
        UnitS3Q30::from_r4([0.5, 0.5, 0.5, -0.5]).unwrap(),
        inconsistent.spin.fiber,
        inconsistent.spin.torsion,
    )
    .unwrap();
    assert_ne!(inconsistent.spin.hopf, alternate.hopf);
    inconsistent.spin.hopf = alternate.hopf;

    let mut state = OrderedSentenceRouteState::new().unwrap();
    let error = state.append(&inconsistent).unwrap_err();
    assert!(error.to_string().contains("inconsistent Hopf observation"));
    assert_eq!(state.route_count(), 0);
    assert!(state.key().is_none());
    assert!(ordered_sentence_key(&[inconsistent]).is_err());
}

#[test]
fn all_payload_provenance_and_route_identity_labels_are_canonical_blake3() {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1")).unwrap();
    assert_canonical_blake3_label(&registry.registry_kappa);
    for binding in &registry.bindings {
        assert_canonical_blake3_label(&binding.payload_cid);
    }
    let route =
        OrderedPrimeRoute::new(vec![PrimeAtom::new(5).unwrap(), PrimeAtom::new(7).unwrap()])
            .unwrap();
    assert_canonical_blake3_label(&route.ordered_kappa().unwrap());
    let history = vec![
        address(&registry, "alpha", 0),
        address(&registry, "beta", 1),
    ];
    assert_canonical_blake3_label(ordered_sentence_key(&history).unwrap().as_str());
    let mut invalid_history = history.clone();
    invalid_history[0].payload_cid = "blake3:short".to_owned();
    assert!(ordered_sentence_key(&invalid_history).is_err());

    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v1");
    assert_canonical_blake3_label(&compiled.manifest.manifest_kappa);
    assert_canonical_blake3_label(&compiled.manifest.zeta_grid.grid_kappa);
    for value in [
        compiled.manifest.provenance.tokenizer_cid.as_str(),
        compiled.manifest.provenance.corpus_cid.as_str(),
        compiled.manifest.provenance.compiler_cid.as_str(),
        compiled.manifest.provenance.cost_profile_cid.as_str(),
    ] {
        assert_canonical_blake3_label(value);
    }
    let json =
        serde_json::from_slice::<serde_json::Value>(&compiled.manifest.canonical_bytes().unwrap())
            .unwrap();
    for row in json["body"]["sentence"].as_array().unwrap() {
        assert_canonical_blake3_label(row["key"].as_str().unwrap());
    }

    let mut malformed = semantic_atoms("payload-v1");
    malformed[0].payload_cid = format!("blake3:{}", "A".repeat(64));
    assert!(PrimeRegistry::compile(&malformed).is_err());

    let mut bad_provenance = provenance();
    bad_provenance.tokenizer_cid = "blake3:short".to_owned();
    assert!(compile_spin_manifest(
        &fixture_sentences(&registry),
        registry,
        ZeroPowerBridge::ContinuousNull,
        bad_provenance,
        NonZeroU16::new(8).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    )
    .is_err());
}

#[test]
fn fixed_zeta_delta_and_quantized_hopf_keep_the_declared_information() {
    let p5 = PrimeAtom::new(5).unwrap();
    let p7 = PrimeAtom::new(7).unwrap();
    assert_eq!(zeta_phase_delta(0, p5, p5).unwrap(), PhaseQ29::ZERO);
    let wrapped_log_ratio = PhaseQ29::from_radians(ZETA_ZEROS[0] * libm::log(7.0 / 5.0))
        .expect("wrapped zeta log-ratio reference");
    assert_eq!(wrapped_log_ratio.raw(), -819_932_194);
    assert_eq!(zeta_phase_delta(0, p5, p7).unwrap(), wrapped_log_ratio);
    let forward = zeta_phase_delta(3, p5, p7).unwrap();
    let reverse = zeta_phase_delta(3, p7, p5).unwrap();
    assert!((forward.raw() + reverse.raw()).abs() <= 1);
    assert!(zeta_phase_delta(512, p5, p7).is_err());
    assert_eq!(zeta_grid_kappa().unwrap(), ZETA_GRID_KAPPA_REFERENCE);
    let fixed_grid = ZetaGridBinding::fixed().unwrap();
    assert_eq!(fixed_grid.revision, ZETA_GRID_REVISION);
    assert_eq!(fixed_grid.channels, 512);
    assert_eq!(fixed_grid.grid_kappa, ZETA_GRID_KAPPA_REFERENCE);

    let spin = UnitS3Q30::from_r4([0.5, 0.5, 0.5, -0.5]).unwrap();
    let antipode = UnitS3Q30::from_r4([-0.5, -0.5, -0.5, 0.5]).unwrap();
    assert_ne!(spin, antipode);
    assert_eq!(spin.hopf().unwrap(), antipode.hopf().unwrap());
    let quarter_turn = PhaseQ29::from_radians(std::f64::consts::FRAC_PI_2).unwrap();
    assert_eq!(
        spin.hopf().unwrap(),
        spin.rotate_common_fiber(quarter_turn)
            .unwrap()
            .hopf()
            .unwrap()
    );
    let hopf_norm = spin
        .hopf()
        .unwrap()
        .to_r3()
        .into_iter()
        .map(|value| value * value)
        .sum::<f64>();
    assert!((hopf_norm - 1.0).abs() < 1.0e-8);
    assert!(UnitS3Q30::from_r4([0.0; 4]).is_err());
}

#[test]
fn canonical_manifest_is_worker_independent_and_strictly_round_trips() {
    let one = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v1");
    let four = fixture(ZeroPowerBridge::ContinuousNull, 4, 8, "payload-v1");
    let one_bytes = one.manifest.canonical_bytes().unwrap();
    let four_bytes = four.manifest.canonical_bytes().unwrap();
    assert_eq!(one_bytes, four_bytes);
    assert_eq!(one.manifest.manifest_kappa, four.manifest.manifest_kappa);
    assert_eq!(
        one.manifest.manifest_kappa,
        FIXTURE_MANIFEST_KAPPA_REFERENCE
    );
    let canonical_value = serde_json::from_slice::<serde_json::Value>(&one_bytes).unwrap();
    let body_bytes = serde_json::to_vec(&canonical_value["body"]).unwrap();
    assert!(body_bytes.len() <= CANONICAL_MANIFEST_BODY_MAX_BYTES);
    assert!(one_bytes.len() <= CANONICAL_MANIFEST_MAX_BYTES);
    assert_eq!(one.metadata.requested_workers, 1);
    assert_eq!(four.metadata.requested_workers, 4);
    assert_eq!(four.metadata.used_workers, 4);
    assert_eq!(four.metadata.causal_transitions, 10);
    assert_eq!(four.metadata.index_occurrences, 25);
    assert!((1..=four.metadata.used_workers).contains(&four.metadata.peak_active_workers));
    assert_eq!(four.metadata.worker_reports.len(), 4);
    assert_eq!(
        four.metadata
            .worker_reports
            .iter()
            .map(|report| report.partition_id)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(four
        .metadata
        .worker_reports
        .iter()
        .all(|report| report.sentence_count > 0
            && report.assigned_transitions > 0
            && report.assigned_transitions == report.completed_transitions));
    assert_eq!(
        four.metadata
            .worker_reports
            .iter()
            .map(|report| report.sentence_count)
            .sum::<usize>(),
        5
    );

    let decoded = CompiledSpinManifest::decode_canonical(&one_bytes).unwrap();
    assert_eq!(decoded, one.manifest);
    let four_decoded = CompiledSpinManifest::decode_canonical(&four_bytes).unwrap();
    assert_eq!(four_decoded, four.manifest);
    assert_eq!(four_decoded.canonical_bytes().unwrap(), four_bytes);
    let pretty = serde_json::to_vec_pretty(
        &serde_json::from_slice::<serde_json::Value>(&one_bytes).unwrap(),
    )
    .unwrap();
    assert!(CompiledSpinManifest::decode_canonical(&pretty).is_err());

    let discrete = fixture(ZeroPowerBridge::DiscreteEmptyProduct, 1, 8, "payload-v1");
    let payload_changed = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v2");
    assert_ne!(
        one.manifest.manifest_kappa,
        discrete.manifest.manifest_kappa
    );
    assert_ne!(
        one.manifest.manifest_kappa,
        payload_changed.manifest.manifest_kappa
    );
}

#[test]
fn schema_two_manifest_explicitly_binds_profile_experts_nlets_and_rebuild_witnesses() {
    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 4, 8, "payload-v1");
    let manifest = &compiled.manifest;
    assert_eq!(manifest.schema, PRIME_ROUTE_MANIFEST_SCHEMA);
    assert_eq!(PRIME_ROUTE_MANIFEST_SCHEMA, 2);
    assert_eq!(
        PRIME_ROUTE_MANIFEST_DOMAIN,
        "uor-r4.prime-route-spin-manifest/2"
    );

    let profile = &manifest.quantization_chart;
    assert_eq!(profile.schema, QUANTIZATION_CHART_SCHEMA);
    assert_eq!(profile.domain, QUANTIZATION_CHART_DOMAIN);
    assert_eq!(profile.phase_fraction_bits, PHASE_FRACTION_BITS);
    assert_eq!(profile.phase_interval, PHASE_INTERVAL);
    assert_eq!(profile.s3_fraction_bits, S3_S2_FRACTION_BITS);
    assert_eq!(profile.s2_fraction_bits, S3_S2_FRACTION_BITS);
    assert_eq!(profile.spin_chart, SPIN_CHART);
    assert_eq!(profile.radial_ring, RADIAL_RING);
    assert_eq!(profile.zeta_grid_revision, manifest.zeta_grid.revision);
    assert_eq!(profile.zeta_channels, manifest.zeta_grid.channels);
    assert_eq!(profile.zeta_grid_kappa, manifest.zeta_grid.grid_kappa);

    let experts = manifest
        .experts
        .iter()
        .map(|record| {
            (
                [record.factors[0].value(), record.factors[1].value()],
                record.product,
                record.occurrence_count,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        experts,
        vec![
            ([5, 7], 35, 3),
            ([7, 11], 77, 2),
            ([7, 13], 91, 1),
            ([7, 17], 119, 4),
        ]
    );
    assert_eq!(
        manifest
            .experts
            .iter()
            .map(|record| usize::try_from(record.occurrence_count).unwrap())
            .sum::<usize>(),
        compiled.metadata.causal_transitions
    );

    let nlets = manifest
        .nlets
        .iter()
        .map(|record| {
            (
                record.sentence_id.as_str(),
                record
                    .ordered_primes
                    .iter()
                    .map(|prime| prime.value())
                    .collect::<Vec<_>>(),
                record
                    .factor_multiset
                    .iter()
                    .map(|prime| prime.value())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        nlets,
        vec![
            ("sentence-1", vec![5, 7, 17], vec![5, 7, 17]),
            ("sentence-2", vec![5, 7, 11], vec![5, 7, 11]),
            ("sentence-3", vec![5, 7, 17], vec![5, 7, 17]),
            ("sentence-4", vec![17, 7, 11], vec![7, 11, 17]),
            ("sentence-5", vec![13, 7, 17], vec![7, 13, 17]),
        ]
    );
    assert_eq!(
        manifest
            .rebuild_witnesses
            .iter()
            .map(|witness| (
                witness.sentence_id.as_str(),
                witness.address_indices.clone()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("sentence-1", vec![0, 1, 4]),
            ("sentence-2", vec![0, 1, 2]),
            ("sentence-3", vec![0, 1, 4]),
            ("sentence-4", vec![4, 1, 2]),
            ("sentence-5", vec![3, 1, 4]),
        ]
    );

    let bytes = manifest.canonical_bytes().unwrap();
    let decoded = CompiledSpinManifest::decode_canonical(&bytes).unwrap();
    assert_eq!(decoded.indexes, manifest.indexes);
    assert_eq!(decoded.experts, manifest.experts);
    assert_eq!(decoded.nlets, manifest.nlets);
    assert_eq!(decoded.rebuild_witnesses, manifest.rebuild_witnesses);
}

#[test]
fn ordered_nlet_and_manifest_kappas_are_sensitive_to_order_and_repetition() {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1")).unwrap();
    let a = address(&registry, "alpha", 0);
    let b = address(&registry, "beta", 1);
    let c = address(&registry, "gamma", 2);
    let variants = [
        vec![a.clone(), b.clone(), c.clone(), a.clone()],
        vec![a.clone(), c.clone(), b.clone(), a.clone()],
        vec![a.clone(), b.clone(), c, a, b],
    ];
    let compiled = variants
        .iter()
        .map(|routes| {
            compile_custom(
                &registry,
                &[RouteSentence {
                    sentence_id: "s".to_owned(),
                    routes: routes.clone(),
                }],
                provenance(),
            )
        })
        .collect::<Vec<_>>();
    let manifest_kappas = compiled
        .iter()
        .map(|value| value.manifest.manifest_kappa.as_str())
        .collect::<BTreeSet<_>>();
    let nlet_kappas = compiled
        .iter()
        .map(|value| value.manifest.nlets[0].ordered_kappa.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(manifest_kappas.len(), 3);
    assert_eq!(nlet_kappas.len(), 3);
    assert_eq!(
        compiled[0].manifest.nlets[0].factor_multiset,
        compiled[1].manifest.nlets[0].factor_multiset
    );
    assert_ne!(
        compiled[0].manifest.nlets[0].ordered_primes,
        compiled[1].manifest.nlets[0].ordered_primes
    );
    assert_eq!(
        compiled[2].manifest.nlets[0]
            .ordered_primes
            .iter()
            .map(|prime| prime.value())
            .collect::<Vec<_>>(),
        vec![5, 7, 17, 5, 7]
    );
}

#[test]
fn route_order_spin_torsion_and_provenance_independently_change_manifest_kappa() {
    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1")).unwrap();
    let sentences = fixture_sentences(&registry);
    let baseline = compile_custom(&registry, &sentences, provenance());

    let mut route_order_sentences = sentences.clone();
    route_order_sentences[0].routes.swap(0, 1);
    let route_order = compile_custom(&registry, &route_order_sentences, provenance());

    let mut spin_sentences = sentences.clone();
    let original = spin_sentences[0].routes[0].spin;
    spin_sentences[0].routes[0].spin = SpinTorsionState::new(
        UnitS3Q30::from_r4([0.5, 0.5, -0.5, 0.5]).unwrap(),
        original.fiber,
        original.torsion,
    )
    .unwrap();
    let spin = compile_custom(&registry, &spin_sentences, provenance());

    let mut torsion_sentences = sentences.clone();
    torsion_sentences[0].routes[0].spin = torsion_sentences[0].routes[0]
        .spin
        .shift_torsion(PhaseQ29::from_radians(0.375).unwrap())
        .unwrap();
    let torsion = compile_custom(&registry, &torsion_sentences, provenance());

    let mut changed_provenance = provenance();
    changed_provenance.compiler_cid = deterministic_label("compiler-v2");
    let provenance_changed = compile_custom(&registry, &sentences, changed_provenance);

    let kappas = [
        baseline.manifest.manifest_kappa,
        route_order.manifest.manifest_kappa,
        spin.manifest.manifest_kappa,
        torsion.manifest.manifest_kappa,
        provenance_changed.manifest.manifest_kappa,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(kappas.len(), 5);
}

#[test]
fn basis_label_evidence_and_prime_registry_tampering_are_rejected_after_rekappa() {
    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v1");

    let basis = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["zeta_grid"]["revision"] =
            serde_json::Value::String("unregistered-zeta-basis".to_owned());
    });
    assert_decode_rejects(&basis, "immutable fixed grid");

    let invalid_is_key = tampered_manifest_bytes(&compiled, |value| {
        let key = value["body"]["sentence"][0]["key"]
            .as_str()
            .unwrap()
            .to_uppercase();
        value["body"]["sentence"][0]["key"] = serde_json::Value::String(key);
    });
    assert_decode_rejects(&invalid_is_key, "canonical lowercase");

    let excessive_evidence = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["last_one"][0]["candidates"][0]["count"] =
            serde_json::Value::from(u64::try_from(TINY_CANARY_MAX_TRANSITIONS + 1).unwrap());
    });
    assert_decode_rejects(&excessive_evidence, "candidate-row evidence");

    let canonical_prime_sequence = tampered_manifest_bytes(&compiled, |value| {
        let bindings = value["body"]["prime_bindings"].as_array_mut().unwrap();
        bindings.last_mut().unwrap()["prime"] = serde_json::Value::from(23u64);
        let registry_wire = serde_json::json!({
            "schema": PRIME_REGISTRY_SCHEMA,
            "domain": PRIME_REGISTRY_DOMAIN,
            "bindings": bindings.clone(),
        });
        value["body"]["prime_registry_kappa"] =
            serde_json::Value::String(json_value_kappa(&registry_wire));
    });
    assert_decode_rejects(&canonical_prime_sequence, "canonical sequential assignment");

    let mut envelope =
        serde_json::from_slice::<serde_json::Value>(&compiled.manifest.canonical_bytes().unwrap())
            .unwrap();
    envelope["manifest_kappa"] = serde_json::Value::String(format!("blake3:{}", "A".repeat(64)));
    assert_decode_rejects(
        &serde_json::to_vec(&envelope).unwrap(),
        "canonical lowercase",
    );
}

#[test]
fn profile_expert_nlet_witness_order_and_index_tampering_fail_after_rekappa() {
    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v1");

    let profile = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["quantization_chart"]["phase_fraction_bits"] =
            serde_json::Value::from(u64::from(PHASE_FRACTION_BITS - 1));
    });
    assert_decode_rejects(&profile, "immutable fixed profile");

    let expert = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["experts"][0]["product"] = serde_json::Value::from(385u64);
    });
    assert_decode_rejects(&expert, "semiprime-expert record is noncanonical");

    let expert_count = tampered_manifest_bytes(&compiled, |value| {
        let count = value["body"]["experts"][0]["occurrence_count"]
            .as_u64()
            .unwrap();
        value["body"]["experts"][0]["occurrence_count"] = serde_json::Value::from(count + 1);
    });
    assert_decode_rejects(&expert_count, "do not cover every witnessed transition");

    let nlet = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["nlets"][0]["ordered_primes"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
    });
    assert_decode_rejects(&nlet, "ordered n-lets do not reproduce");

    let nlet_order = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["nlets"].as_array_mut().unwrap().swap(0, 1);
    });
    assert_decode_rejects(&nlet_order, "ordered n-let shape");

    let witness_order = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["rebuild_witnesses"][0]["address_indices"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
    });
    assert_decode_rejects(&witness_order, "do not reproduce from the rebuild witness");

    let index = tampered_manifest_bytes(&compiled, |value| {
        let count = value["body"]["last_one"][0]["candidates"][0]["count"]
            .as_u64()
            .unwrap();
        value["body"]["last_one"][0]["candidates"][0]["count"] = serde_json::Value::from(count + 1);
    });
    assert_decode_rejects(&index, "indexes do not reproduce from the rebuild witness");

    let witness_index = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["rebuild_witnesses"][0]["address_indices"][0] =
            serde_json::Value::from(u64::try_from(MANIFEST_MAX_ADDRESSES).unwrap());
    });
    assert_decode_rejects(&witness_index, "address index is out of range");
}

#[test]
fn typed_manifest_canonicalization_rejects_rekappad_reconstruction_mismatch() {
    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v1");
    let tampered_bytes = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["nlets"][0]["ordered_primes"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
    });
    let envelope = serde_json::from_slice::<serde_json::Value>(&tampered_bytes).unwrap();

    let mut typed = compiled.manifest.clone();
    typed.nlets[0].ordered_primes.swap(0, 1);
    typed.manifest_kappa = envelope["manifest_kappa"].as_str().unwrap().to_owned();

    let error = typed.canonical_bytes().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("ordered n-lets do not reproduce from the rebuild witness"),
        "unexpected typed-manifest error: {error}"
    );
}

#[test]
fn manifest_decode_rejects_oversize_empty_and_over_ceiling_shapes_before_conversion() {
    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 1, 8, "payload-v1");

    let oversized = vec![b' '; CANONICAL_MANIFEST_MAX_BYTES + 1];
    assert_decode_rejects(&oversized, "byte ceiling");

    let empty_i1 = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["last_one"] = serde_json::Value::Array(Vec::new());
    });
    assert_decode_rejects(&empty_i1, "causal row population");

    let empty_is = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["sentence"] = serde_json::Value::Array(Vec::new());
    });
    assert_decode_rejects(&empty_is, "causal row population");

    let too_many_addresses = tampered_manifest_bytes(&compiled, |value| {
        let template = value["body"]["addresses"][0].clone();
        value["body"]["addresses"] =
            serde_json::Value::Array(vec![template; MANIFEST_MAX_ADDRESSES + 1]);
    });
    assert_decode_rejects(&too_many_addresses, "binding/address population");

    let too_many_i2_rows = tampered_manifest_bytes(&compiled, |value| {
        let template = value["body"]["last_two"][0].clone();
        value["body"]["last_two"] =
            serde_json::Value::Array(vec![template; MANIFEST_MAX_I2_ROWS + 1]);
    });
    assert_decode_rejects(&too_many_i2_rows, "per-index ceiling");

    let too_many_total_rows = tampered_manifest_bytes(&compiled, |value| {
        let i1 = value["body"]["last_one"][0].clone();
        let i2 = value["body"]["last_two"][0].clone();
        let sentence = value["body"]["sentence"][0].clone();
        let sentence_rows =
            MANIFEST_MAX_TOTAL_ROWS + 1 - MANIFEST_MAX_I1_ROWS - MANIFEST_MAX_I2_ROWS;
        assert!(sentence_rows <= MANIFEST_MAX_IS_ROWS);
        value["body"]["last_one"] = serde_json::Value::Array(vec![i1; MANIFEST_MAX_I1_ROWS]);
        value["body"]["last_two"] = serde_json::Value::Array(vec![i2; MANIFEST_MAX_I2_ROWS]);
        value["body"]["sentence"] = serde_json::Value::Array(vec![sentence; sentence_rows]);
    });
    assert_decode_rejects(&too_many_total_rows, "row population");

    let excessive_declared_cap = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["maximum_candidates"] =
            serde_json::Value::from(u64::from(MANIFEST_MAX_CANDIDATES_PER_ROW) + 1);
    });
    assert_decode_rejects(&excessive_declared_cap, "candidate bound");

    let empty_row = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["last_one"][0]["candidates"] = serde_json::Value::Array(Vec::new());
    });
    assert_decode_rejects(&empty_row, "candidate row");

    let zero_count = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["last_one"][0]["candidates"][0]["count"] = serde_json::Value::from(0u64);
    });
    assert_decode_rejects(&zero_count, "candidate row");

    let duplicate_next_with_different_counts = tampered_manifest_bytes(&compiled, |value| {
        let mut higher = value["body"]["last_one"][0]["candidates"][0].clone();
        higher["count"] = serde_json::Value::from(2u64);
        let mut lower = higher.clone();
        lower["count"] = serde_json::Value::from(1u64);
        value["body"]["last_one"][0]["candidates"] = serde_json::Value::Array(vec![higher, lower]);
    });
    assert_decode_rejects(
        &duplicate_next_with_different_counts,
        "repeats a next address",
    );

    let too_many_candidates_in_one_row = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["maximum_candidates"] =
            serde_json::Value::from(u64::from(MANIFEST_MAX_CANDIDATES_PER_ROW));
        let candidate = value["body"]["last_one"][0]["candidates"][0].clone();
        value["body"]["last_one"][0]["candidates"] =
            serde_json::Value::Array(vec![
                candidate;
                usize::from(MANIFEST_MAX_CANDIDATES_PER_ROW) + 1
            ]);
    });
    assert_decode_rejects(&too_many_candidates_in_one_row, "candidate row");

    let too_many_retained_candidates = tampered_manifest_bytes(&compiled, |value| {
        value["body"]["maximum_candidates"] =
            serde_json::Value::from(u64::from(MANIFEST_MAX_CANDIDATES_PER_ROW));
        let candidate = value["body"]["last_two"][0]["candidates"][0].clone();
        let mut row = value["body"]["last_two"][0].clone();
        row["candidates"] =
            serde_json::Value::Array(vec![
                candidate;
                usize::from(MANIFEST_MAX_CANDIDATES_PER_ROW)
            ]);
        let rows = MANIFEST_MAX_RETAINED_CANDIDATE_ENTRIES
            / usize::from(MANIFEST_MAX_CANDIDATES_PER_ROW)
            + 1;
        value["body"]["last_two"] = serde_json::Value::Array(vec![row; rows]);
    });
    assert_decode_rejects(&too_many_retained_candidates, "retained candidates");
}

#[test]
fn whole_sentence_partitions_balance_transition_weight_and_report_completion() {
    let compiled = compile_lengths(&[9, 5, 4, 3, 2], 4).unwrap();
    assert_eq!(compiled.metadata.sentences, 5);
    assert_eq!(compiled.metadata.route_steps, 23);
    assert_eq!(compiled.metadata.causal_transitions, 18);
    assert_eq!(compiled.metadata.index_occurrences, 49);
    assert!((1..=compiled.metadata.used_workers).contains(&compiled.metadata.peak_active_workers));
    assert_eq!(
        compiled
            .metadata
            .worker_reports
            .iter()
            .map(|report| report.sentence_count)
            .collect::<Vec<_>>(),
        vec![1, 1, 1, 2]
    );
    assert_eq!(
        compiled
            .metadata
            .worker_reports
            .iter()
            .map(|report| report.assigned_transitions)
            .collect::<Vec<_>>(),
        vec![8, 4, 3, 3]
    );
    assert!(compiled
        .metadata
        .worker_reports
        .iter()
        .all(|report| report.assigned_transitions > 0
            && report.completed_transitions == report.assigned_transitions));
}

#[test]
fn zero_transition_sentences_do_not_receive_dedicated_workers() {
    let one = compile_lengths(&[1, 1, 4, 1, 3], 1).unwrap();
    let requested_four = compile_lengths(&[1, 1, 4, 1, 3], 4).unwrap();

    assert_eq!(requested_four.metadata.requested_workers, 4);
    assert_eq!(requested_four.metadata.used_workers, 2);
    assert_eq!(requested_four.metadata.sentences, 5);
    assert_eq!(requested_four.metadata.causal_transitions, 5);
    assert_eq!(requested_four.metadata.worker_reports.len(), 2);
    assert_eq!(
        requested_four
            .metadata
            .worker_reports
            .iter()
            .map(|report| report.sentence_count)
            .sum::<usize>(),
        5
    );
    assert!(requested_four
        .metadata
        .worker_reports
        .iter()
        .all(|report| report.assigned_transitions > 0
            && report.assigned_transitions == report.completed_transitions));
    assert!((1..=2).contains(&requested_four.metadata.peak_active_workers));
    assert_eq!(
        one.manifest.canonical_bytes().unwrap(),
        requested_four.manifest.canonical_bytes().unwrap()
    );
}

#[test]
fn tiny_canary_limits_reject_corpus_shaped_or_noncausal_inputs_before_index_build() {
    let cases = [
        (
            vec![2; TINY_CANARY_MAX_SENTENCES + 1],
            TinyCanaryDimension::Sentences,
            TINY_CANARY_MAX_SENTENCES,
        ),
        (
            vec![TINY_CANARY_MAX_ROUTES_PER_SENTENCE + 1],
            TinyCanaryDimension::RoutesPerSentence,
            TINY_CANARY_MAX_ROUTES_PER_SENTENCE,
        ),
        (
            vec![
                TINY_CANARY_MAX_ROUTES_PER_SENTENCE;
                TINY_CANARY_MAX_TOTAL_ROUTES / TINY_CANARY_MAX_ROUTES_PER_SENTENCE + 1
            ],
            TinyCanaryDimension::TotalRoutes,
            TINY_CANARY_MAX_TOTAL_ROUTES,
        ),
        (
            vec![
                TINY_CANARY_MAX_ROUTES_PER_SENTENCE;
                TINY_CANARY_MAX_TOTAL_ROUTES / TINY_CANARY_MAX_ROUTES_PER_SENTENCE
            ],
            TinyCanaryDimension::Transitions,
            TINY_CANARY_MAX_TRANSITIONS,
        ),
        (
            vec![
                TINY_CANARY_MAX_ROUTES_PER_SENTENCE - 1;
                TINY_CANARY_MAX_TOTAL_ROUTES / TINY_CANARY_MAX_ROUTES_PER_SENTENCE
            ],
            TinyCanaryDimension::Occurrences,
            TINY_CANARY_MAX_OCCURRENCES,
        ),
    ];
    for (route_lengths, expected_dimension, expected_maximum) in cases {
        match compile_lengths(&route_lengths, 1) {
            Err(PrimeRouteError::TinyCanaryLimitExceeded {
                dimension,
                observed,
                maximum,
            }) => {
                assert_eq!(dimension, expected_dimension);
                assert!(observed > maximum);
                assert_eq!(maximum, expected_maximum);
            }
            other => panic!("expected {expected_dimension:?} limit rejection, got {other:?}"),
        }
    }

    let noncausal = compile_lengths(&[1, 1], 1).unwrap_err();
    assert!(noncausal
        .to_string()
        .contains("requires at least one causal transition"));
}

#[test]
fn semantic_and_sentence_identifiers_are_bounded_before_manifest_construction() {
    let oversized_atom = vec![SemanticAtom {
        semantic_atom_id: "x".repeat(TINY_CANARY_MAX_IDENTIFIER_BYTES + 1),
        payload_cid: deterministic_label("oversized-semantic-id"),
    }];
    let per_string_error = PrimeRegistry::compile(&oversized_atom).unwrap_err();
    assert!(per_string_error
        .to_string()
        .contains("semantic atom ID exceeds"));

    let aggregate_atoms = (0..TINY_CANARY_MAX_TOTAL_IDENTIFIER_BYTES
        / TINY_CANARY_MAX_IDENTIFIER_BYTES)
        .map(|index| {
            let prefix = format!("{index:04}-");
            SemanticAtom {
                semantic_atom_id: format!(
                    "{prefix}{}",
                    "x".repeat(TINY_CANARY_MAX_IDENTIFIER_BYTES - prefix.len())
                ),
                payload_cid: deterministic_label(&format!("aggregate-{index}")),
            }
        })
        .collect::<Vec<_>>();
    let aggregate_registry = PrimeRegistry::compile(&aggregate_atoms).unwrap();
    let aggregate_sentence = vec![RouteSentence {
        sentence_id: "s".to_owned(),
        routes: vec![
            address(&aggregate_registry, &aggregate_atoms[0].semantic_atom_id, 0),
            address(&aggregate_registry, &aggregate_atoms[1].semantic_atom_id, 1),
        ],
    }];
    match compile_spin_manifest(
        &aggregate_sentence,
        aggregate_registry,
        ZeroPowerBridge::ContinuousNull,
        provenance(),
        NonZeroU16::new(8).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    ) {
        Err(PrimeRouteError::TinyCanaryLimitExceeded {
            dimension,
            observed,
            maximum,
        }) => {
            assert_eq!(dimension, TinyCanaryDimension::IdentifierBytes);
            assert!(observed > maximum);
            assert_eq!(maximum, TINY_CANARY_MAX_TOTAL_IDENTIFIER_BYTES);
        }
        other => panic!("expected aggregate identifier limit rejection, got {other:?}"),
    }

    let registry = PrimeRegistry::compile(&semantic_atoms("payload-v1")).unwrap();
    let routes = vec![
        address(&registry, "alpha", 0),
        address(&registry, "beta", 1),
    ];
    let oversized_sentence = vec![RouteSentence {
        sentence_id: "s".repeat(TINY_CANARY_MAX_IDENTIFIER_BYTES + 1),
        routes,
    }];
    let sentence_error = compile_spin_manifest(
        &oversized_sentence,
        registry,
        ZeroPowerBridge::ContinuousNull,
        provenance(),
        NonZeroU16::new(8).unwrap(),
        NonZeroUsize::new(1).unwrap(),
    )
    .unwrap_err();
    assert!(sentence_error.to_string().contains("sentence ID exceeds"));
}

#[test]
fn i1_i2_and_sentence_indexes_are_bounded_direct_and_intervention_sensitive() {
    let compiled = fixture(ZeroPowerBridge::ContinuousNull, 4, 1, "payload-v1");
    let registry = &compiled.manifest.prime_registry;
    let a = address(registry, "alpha", 0);
    let b = address(registry, "beta", 1);
    let c = address(registry, "gamma", 2);

    let history = vec![a.clone(), b.clone()];
    let mut maintained = OrderedSentenceRouteState::new().unwrap();
    maintained.append(&a).unwrap();
    maintained.append(&b).unwrap();
    let maintained_key = maintained.key().unwrap();
    assert!(compiled.manifest.indexes.last_one(&b).is_some());
    assert!(compiled.manifest.indexes.last_two(&a, &b).is_some());
    assert!(compiled.manifest.indexes.sentence(&history).is_some());
    assert!(compiled
        .manifest
        .indexes
        .sentence_precomputed(maintained_key)
        .is_some());
    assert!(compiled
        .manifest
        .indexes
        .sentence(&[a.clone(), b.clone(), c])
        .is_none());

    let real = compiled
        .manifest
        .indexes
        .lookup(
            &history,
            compiled.manifest.maximum_candidates,
            &RouteIntervention::None,
        )
        .unwrap();
    assert_eq!(real.rows_read, [true, true, true]);
    assert!(real.candidate_entries_read <= 3);
    assert_eq!(real.candidates.len(), 1);
    let precomputed = compiled
        .manifest
        .indexes
        .lookup_precomputed(
            Some(&a),
            &b,
            maintained_key,
            compiled.manifest.maximum_candidates,
        )
        .unwrap();
    assert_eq!(precomputed, real);

    let last_two_permuted = compiled
        .manifest
        .indexes
        .lookup(
            &history,
            compiled.manifest.maximum_candidates,
            &RouteIntervention::LastTwo(b.clone(), a.clone()),
        )
        .unwrap();
    assert_eq!(last_two_permuted.rows_read, [true, false, true]);

    let sentence_permuted = compiled
        .manifest
        .indexes
        .lookup(
            &history,
            compiled.manifest.maximum_candidates,
            &RouteIntervention::Sentence(vec![b.clone(), a]),
        )
        .unwrap();
    assert_eq!(sentence_permuted.rows_read, [true, true, false]);

    let torsion_shifted = compiled
        .manifest
        .indexes
        .lookup(
            &history,
            compiled.manifest.maximum_candidates,
            &RouteIntervention::ShiftLastTorsion(PhaseQ29::from_radians(0.25).unwrap()),
        )
        .unwrap();
    assert_eq!(torsion_shifted.rows_read, [false, false, false]);
    assert!(torsion_shifted.candidates.is_empty());
}
