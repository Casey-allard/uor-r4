//! Inert one-time prediction-stream runner for issue #973 Phase III.
//!
//! This temporary integration test is ignored by default. It reads only the
//! protected construction and input freezes, builds two independent models,
//! and emits deterministic prediction bits. It contains no scoring oracle.

use uor_r4_core::direct_causal_geometric_attention::{
    CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY, CONNECTION_GAUGE_COVARIANCE_V4_POLICY,
    CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE,
    CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE, ConnectionGaugeCovarianceV4,
    ConnectionGaugeCovarianceV4Arm, ConnectionGaugeCovarianceV4Intervention,
    ConnectionGaugeCovarianceV4Trace, DirectCausalGeometricAttentionConfig,
};
use uor_r4_core::geometric_gated_delta_retention::{
    GeometricRetentionConstructionSequence, GeometricRetentionConstructionStep,
    GeometricRetentionSupportBinding,
};

const PHASE_I_PROTECTED_MERGE_SHA: &str = "b054197acb92e3dd23d88d81bd859379ea8fac67";
const PHASE_II_PROTECTED_MERGE_SHA: &str = "a567edd43ec4840c0bce495339c6416777c4c883";
const MAXIMUM_TOKEN_ID: u32 = 12;
const QUERY_TOKEN: u32 = 1;
const SUPPORT: [u32; 2] = [5, 6];
const EXPECTED_CONSTRUCTION_EVENT_COUNT: u64 = 116;
const EXPECTED_PARAMETER_COUNT_PER_ARM: usize = 13 * 4 * 3;
const REQUIRED_SCORE_GAP: f64 = 1.0e-8;
const STREAM_DOMAIN: &str = "uor-r4.cgcv-v4.prediction-stream/1";
const CASE_ID_DOMAIN: &str = "uor-r4.cgcv-v4.case-id/1";

const FROZEN_CONFIG: DirectCausalGeometricAttentionConfig = DirectCausalGeometricAttentionConfig {
    epochs: 80,
    learning_rate: 0.04,
    temperature: 0.30,
};

const FROZEN_PHASE_I_PREFLIGHT_CID: &str =
    "blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e";
const FROZEN_PHASE_I_CORE_CID: &str =
    "blake3:4c7c33d8de40dd6bd7424c9e6360183f672d55453c257a83fa554e045b6b1d1a";
const FROZEN_PHASE_I_ARTIFACT_CID: &str =
    "blake3:0ed7bf62074857df80045ac3b8bee13ee5f367be4b2b971748631b606ab5985a";
const FROZEN_PHASE_I_INITIALIZATION_CID: &str =
    "blake3:8f91f8d05cbde422593860cffdc3153007fb5b3b2946217ef0015668d3ac34d0";
const FROZEN_PHASE_I_CONSTRUCTION_KAPPA: &str =
    "blake3:446e4f16c9aff5b5dee4c342bf45847e6e8332d6bed8d4a9a21bfc99f82dbe39";
const FROZEN_PHASE_I_FRAME_CID: &str =
    "blake3:205ee0d1b9aebbee2475d97de3b95d359ff2ee8220334995cfe4c7a71ead5920";
const FROZEN_GENERATOR_POLICY_CID: &str =
    "blake3:73b4233b0b91ba85ffb6cd8c3d86132a954e4fbda5c7ec57510cc30bd9fb5dca";
const FROZEN_FORBIDDEN_ROOT: &str =
    "blake3:877707eff60857b9c790cfb0e8a2a5a12bbcadb51d3448c9bd7119d5b86b6c42";
const FROZEN_VALIDATION_PREFIX_ROOT: &str =
    "blake3:a3321b13d808d553d7588997f8fb7951be33e254724d45a1223460dd775a3ad8";
const FROZEN_VALIDATION_INPUT_CID: &str =
    "blake3:5a17c5526d866f2862b042750cb70f5183f6a8fc09ab53a067d79d28d1c989d1";
const FROZEN_COMMITMENT_CID: &str =
    "blake3:9773355914ed171f0d14950a4db554f5f543252804c703e8e0bbbbf17fe7b602";
const FROZEN_PRELABEL_CID: &str =
    "blake3:170419cfcf80b2b0e48cc74faff13c9791dd9106045a1ff59a82efe4f6b205aa";

#[derive(Clone, Copy)]
struct FrozenCase {
    case_id: &'static str,
    prefix: &'static [u32],
}

const FROZEN_CASES: [FrozenCase; 24] = [
    FrozenCase {
        case_id: "blake3:1a3d8f6b50f3dbf5b23572858b9e69c4bb7ee58318ce03ecd0d78f8f31844679",
        prefix: &[8, 4, 10, 11, 12, 1, 5, 2, 9, 6, 7, 3, 1],
    },
    FrozenCase {
        case_id: "blake3:fe99823d4f6c78d1e906a4690fad055eb48ac1eef34323862ff4b8f30e6db13c",
        prefix: &[8, 4, 10, 11, 12, 1, 6, 2, 9, 5, 7, 3, 1],
    },
    FrozenCase {
        case_id: "blake3:215cf21bd7ed218a7903f6285674fb7c54e83bb14490d624a215d5e670f36848",
        prefix: &[6, 7, 4, 11, 10, 8, 9, 1, 5, 3, 2, 12, 1],
    },
    FrozenCase {
        case_id: "blake3:3dbbb505382718dfe267670829df3cc4a28b0607c855b364fb17f062a8a000d7",
        prefix: &[5, 7, 4, 11, 10, 8, 9, 1, 6, 3, 2, 12, 1],
    },
    FrozenCase {
        case_id: "blake3:13f215def7d469a42abae814d3cbffdca0be193315dd582ce910860968b55586",
        prefix: &[12, 8, 7, 3, 9, 6, 2, 11, 1, 5, 10, 4, 1],
    },
    FrozenCase {
        case_id: "blake3:e5778dd9fbcdeb9a20892602ef49e5ea2f6c46690b2ff951adffbf377fa08a79",
        prefix: &[12, 8, 7, 3, 9, 5, 2, 11, 1, 6, 10, 4, 1],
    },
    FrozenCase {
        case_id: "blake3:c43b48eb2dfe758b9da850f0fb5ee21e6b750fcbdc89e650e9b3844fe989d63c",
        prefix: &[2, 4, 7, 1, 5, 3, 12, 6, 9, 8, 11, 10, 1],
    },
    FrozenCase {
        case_id: "blake3:d16ff997fdc4cdfad24e0aab1aa15f28093f96cab8b79995cc1301b8e10d0cbb",
        prefix: &[2, 4, 7, 1, 6, 3, 12, 5, 9, 8, 11, 10, 1],
    },
    FrozenCase {
        case_id: "blake3:e173d1b98d789493cb5c71db7689b35c76c8e383690f6132e344dd860da54860",
        prefix: &[4, 9, 8, 6, 7, 1, 5, 3, 10, 11, 2, 12, 1],
    },
    FrozenCase {
        case_id: "blake3:2fceb1c887b8f6faf412c94f5aebe0a7668fad11be6f83de1ddb30410e04374c",
        prefix: &[4, 9, 8, 5, 7, 1, 6, 3, 10, 11, 2, 12, 1],
    },
    FrozenCase {
        case_id: "blake3:b9557a9e58757b3411206813e8dec75a677d7a438af1947ef386a8f73ba768d6",
        prefix: &[11, 12, 1, 5, 6, 2, 4, 8, 3, 9, 7, 10, 1],
    },
    FrozenCase {
        case_id: "blake3:53153c2dd34b4473bfa483ba091645b8b5de3f296dcc79ca699cb01631a242aa",
        prefix: &[11, 12, 1, 6, 5, 2, 4, 8, 3, 9, 7, 10, 1],
    },
    FrozenCase {
        case_id: "blake3:f3a6625df4f281a5cef71f4e8020657ae61ec89d24e24bd1946c144a5d33b491",
        prefix: &[2, 1, 5, 9, 11, 10, 3, 8, 12, 4, 7, 6, 1],
    },
    FrozenCase {
        case_id: "blake3:ff92d321bb0497c13d5b28d978e2055dab20b02accb08888bea4119dcee5b6d0",
        prefix: &[2, 1, 6, 9, 11, 10, 3, 8, 12, 4, 7, 5, 1],
    },
    FrozenCase {
        case_id: "blake3:cb6c3aeaaedb0a696f08009f664cf8abae4e3b313ae3d7181711d28035211b42",
        prefix: &[10, 9, 12, 8, 2, 11, 7, 1, 5, 4, 3, 6, 1],
    },
    FrozenCase {
        case_id: "blake3:3753d93a9ff3bd9f1b2b6092307f68e3f7d9fc3436eed26f6c25788e59c52ab9",
        prefix: &[10, 9, 12, 8, 2, 11, 7, 1, 6, 4, 3, 5, 1],
    },
    FrozenCase {
        case_id: "blake3:67839ab0dc2ee3cf90fb3c1524eb77dd8fb7b0348d04730e191c7b104d640789",
        prefix: &[8, 6, 3, 7, 2, 1, 5, 12, 10, 4, 11, 9, 1],
    },
    FrozenCase {
        case_id: "blake3:b1846c2774efb974e6841abec35c37c240577deda45afb12b72afaed42b80b80",
        prefix: &[8, 5, 3, 7, 2, 1, 6, 12, 10, 4, 11, 9, 1],
    },
    FrozenCase {
        case_id: "blake3:95c68874f3c7fc81fc8cb0c082dbbee865f12046b0fd989a8390926d1452edba",
        prefix: &[3, 9, 2, 8, 12, 11, 7, 4, 1, 5, 6, 10, 1],
    },
    FrozenCase {
        case_id: "blake3:e00a31827260f55169d7ce998437588539a33c8c3ef6f804e3e45e6293591e7f",
        prefix: &[3, 9, 2, 8, 12, 11, 7, 4, 1, 6, 5, 10, 1],
    },
    FrozenCase {
        case_id: "blake3:a8e1c0288b571bb48bc2f04f3a5141586e79722bf86c30ad4cd9249051c64009",
        prefix: &[1, 5, 2, 4, 11, 9, 10, 7, 3, 12, 8, 6, 1],
    },
    FrozenCase {
        case_id: "blake3:e08e57eff49dfd07c89609edad36c4b1257cd4035ef4c9ac6f28374ed1062bf8",
        prefix: &[1, 6, 2, 4, 11, 9, 10, 7, 3, 12, 8, 5, 1],
    },
    FrozenCase {
        case_id: "blake3:64500c16993cf62b0a4e920f8fd171527fcdd978d488619f1f6f37d9cb3e51b5",
        prefix: &[12, 8, 3, 4, 6, 9, 1, 5, 7, 11, 10, 2, 1],
    },
    FrozenCase {
        case_id: "blake3:637591501c3d0d3909a9c734d092a5f70739c3360c833326a74f9452c4596999",
        prefix: &[12, 8, 3, 4, 5, 9, 1, 6, 7, 11, 10, 2, 1],
    },
];

#[derive(Clone, Copy)]
struct FrozenBinding {
    name: &'static str,
    arm_name: &'static str,
    intervention_name: &'static str,
    arm: ConnectionGaugeCovarianceV4Arm,
    intervention: ConnectionGaugeCovarianceV4Intervention,
    current_only: bool,
}

const FROZEN_BINDINGS: [FrozenBinding; 7] = [
    FrozenBinding {
        name: "baseline",
        arm_name: "h4_compatible",
        intervention_name: "none",
        arm: ConnectionGaugeCovarianceV4Arm::H4Compatible,
        intervention: ConnectionGaugeCovarianceV4Intervention::None,
        current_only: false,
    },
    FrozenBinding {
        name: "alternative",
        arm_name: "alternative_tangent",
        intervention_name: "none",
        arm: ConnectionGaugeCovarianceV4Arm::AlternativeTangent,
        intervention: ConnectionGaugeCovarianceV4Intervention::None,
        current_only: false,
    },
    FrozenBinding {
        name: "plain",
        arm_name: "plain_fixed",
        intervention_name: "none",
        arm: ConnectionGaugeCovarianceV4Arm::PlainFixed,
        intervention: ConnectionGaugeCovarianceV4Intervention::None,
        current_only: false,
    },
    FrozenBinding {
        name: "current_only",
        arm_name: "current_token_only",
        intervention_name: "none",
        arm: ConnectionGaugeCovarianceV4Arm::CurrentTokenOnly,
        intervention: ConnectionGaugeCovarianceV4Intervention::None,
        current_only: true,
    },
    FrozenBinding {
        name: "order_shuffled",
        arm_name: "h4_compatible",
        intervention_name: "order_shuffled",
        arm: ConnectionGaugeCovarianceV4Arm::H4Compatible,
        intervention: ConnectionGaugeCovarianceV4Intervention::OrderShuffled,
        current_only: false,
    },
    FrozenBinding {
        name: "value_permuted",
        arm_name: "h4_compatible",
        intervention_name: "value_permuted",
        arm: ConnectionGaugeCovarianceV4Arm::H4Compatible,
        intervention: ConnectionGaugeCovarianceV4Intervention::ValuePermuted,
        current_only: false,
    },
    FrozenBinding {
        name: "source_gauge_mismatch",
        arm_name: "h4_compatible",
        intervention_name: "source_gauge_mismatched",
        arm: ConnectionGaugeCovarianceV4Arm::H4Compatible,
        intervention: ConnectionGaugeCovarianceV4Intervention::SourceGaugeMismatched,
        current_only: false,
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
struct PredictionRow {
    case_index: u16,
    case_id: &'static str,
    binding_name: &'static str,
    selected_token: u32,
    score_5_bits: u64,
    score_6_bits: u64,
    absolute_score_gap_bits: u64,
}

fn tag(domain: &str) -> Vec<u8> {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn push_lp64(destination: &mut Vec<u8>, bytes: &[u8]) {
    destination.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    destination.extend_from_slice(bytes);
}

fn token_bytes(tokens: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + tokens.len() * 4);
    bytes.extend_from_slice(&(tokens.len() as u64).to_le_bytes());
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    bytes
}

fn parse_raw_cid(value: &str) -> [u8; 32] {
    let encoded = value.as_bytes();
    assert!(encoded.len() == 71 && encoded.starts_with(b"blake3:"));
    let mut raw = [0_u8; 32];
    for (index, pair) in encoded[7..].chunks_exact(2).enumerate() {
        let nibble = |byte| match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            _ => panic!("non-canonical CID"),
        };
        raw[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    raw
}

fn case_id(prefix: &[u32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&tag(CASE_ID_DOMAIN));
    let canonical = token_bytes(prefix);
    let mut framed = Vec::new();
    push_lp64(&mut framed, &canonical);
    hasher.update(&framed);
    *hasher.finalize().as_bytes()
}

fn fixture_binding() -> GeometricRetentionSupportBinding {
    GeometricRetentionSupportBinding::new(
        format!(
            "blake3:{}",
            blake3::hash(b"cgcv-973-binding-table-v4").to_hex()
        ),
        format!(
            "blake3:{}",
            blake3::hash(b"cgcv-973-binding-overlay-v4").to_hex()
        ),
        "cgcv-973-construction-only/4",
    )
    .expect("frozen support binding")
}

fn construction_document(
    document_id: &str,
    causal_prefix: &[u32],
    observed_recall_token: u32,
) -> GeometricRetentionConstructionSequence {
    assert!(causal_prefix.len() >= 2);
    assert!(causal_prefix.last() == Some(&QUERY_TOKEN));
    assert!(SUPPORT.binary_search(&observed_recall_token).is_ok());
    let mut steps = causal_prefix[1..]
        .iter()
        .copied()
        .map(|observed_token| GeometricRetentionConstructionStep {
            admitted_support: vec![observed_token],
            observed_token,
        })
        .collect::<Vec<_>>();
    steps.push(GeometricRetentionConstructionStep {
        admitted_support: SUPPORT.to_vec(),
        observed_token: observed_recall_token,
    });
    GeometricRetentionConstructionSequence {
        document_id: document_id.to_owned(),
        initial_token: causal_prefix[0],
        steps,
    }
}

fn construction_split() -> Vec<GeometricRetentionConstructionSequence> {
    vec![
        construction_document("construction-01", &[1, 5, 2, 6, 9, 1], 5),
        construction_document("construction-02", &[1, 6, 2, 5, 9, 1], 6),
        construction_document("construction-03", &[2, 6, 1, 5, 8, 1], 5),
        construction_document("construction-04", &[2, 5, 1, 6, 8, 1], 6),
        construction_document("construction-05", &[3, 1, 5, 4, 2, 6, 9, 1], 5),
        construction_document("construction-06", &[4, 1, 6, 3, 2, 5, 9, 1], 6),
        construction_document("construction-07", &[2, 6, 7, 1, 5, 8, 1], 5),
        construction_document("construction-08", &[2, 5, 7, 1, 6, 8, 1], 6),
        construction_document("construction-09", &[1, 5, 3, 2, 6, 4, 1], 5),
        construction_document("construction-10", &[1, 6, 3, 2, 5, 4, 1], 6),
        construction_document("construction-11", &[2, 6, 3, 1, 5, 7, 8, 1], 5),
        construction_document("construction-12", &[2, 5, 3, 1, 6, 7, 8, 1], 6),
        construction_document("construction-13", &[10, 1, 5, 11, 2, 6, 12, 1], 5),
        construction_document("construction-14", &[10, 1, 6, 11, 2, 5, 12, 1], 6),
        construction_document("construction-15", &[2, 6, 4, 1, 5, 3, 7, 1], 5),
        construction_document("construction-16", &[2, 5, 4, 1, 6, 3, 7, 1], 6),
    ]
}

fn compile_model() -> ConnectionGaugeCovarianceV4 {
    ConnectionGaugeCovarianceV4::compile(
        MAXIMUM_TOKEN_ID,
        &construction_split(),
        FROZEN_CONFIG,
        fixture_binding(),
    )
    .expect("frozen construction model compiles")
}

fn score(trace: &ConnectionGaugeCovarianceV4Trace, token: u32) -> f64 {
    trace
        .scores
        .iter()
        .find(|candidate| candidate.token == token)
        .expect("frozen support candidate exists")
        .score
}

fn close(left: f64, right: f64) -> bool {
    let scale = left.abs().max(right.abs());
    (left - right).abs()
        <= CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_ABSOLUTE_TOLERANCE
            + CONNECTION_GAUGE_COVARIANCE_V4_SCALAR_RELATIVE_TOLERANCE * scale
}

fn assert_main_parity(
    reference: &ConnectionGaugeCovarianceV4Trace,
    actual: &ConnectionGaugeCovarianceV4Trace,
) {
    assert!(actual.selected_token == reference.selected_token);
    assert!(actual.query_token == reference.query_token);
    assert!(actual.admitted_support == reference.admitted_support);
    assert!(actual.positions.len() == reference.positions.len());
    assert!(actual.scores.len() == reference.scores.len());
    assert!(actual.input_position_count == reference.input_position_count);
    assert!(actual.query_position == reference.query_position);
    assert!(actual.causal_prefix_position_count == reference.causal_prefix_position_count);
    assert!(actual.masked_future_position_count == reference.masked_future_position_count);
    assert!(actual.maximum_position_read == reference.maximum_position_read);
    assert!(actual.future_token_reads == reference.future_token_reads);
    assert!(actual.causal_token_value_reads == reference.causal_token_value_reads);
    assert!(actual.q_projections == reference.q_projections);
    assert!(actual.k_projections == reference.k_projections);
    assert!(actual.v_projections == reference.v_projections);
    assert!(actual.o_projections == reference.o_projections);
    assert!(actual.key_transports == reference.key_transports);
    assert!(actual.value_transports == reference.value_transports);
    assert!(actual.output_transports == reference.output_transports);
    assert!(actual.stored_scalar_parameter_count == reference.stored_scalar_parameter_count);
    assert!(actual.learned_effective_degree_count == reference.learned_effective_degree_count);
    assert!(close(
        actual.softmax_weight_sum,
        reference.softmax_weight_sum
    ));
    for (left, right) in actual.positions.iter().zip(&reference.positions) {
        assert!(left.attended_position == right.attended_position);
        assert!(left.observed_token == right.observed_token);
        assert!(left.key_source_token == right.key_source_token);
        assert!(left.value_source_token == right.value_source_token);
        assert!(close(left.attention_logit, right.attention_logit));
        assert!(close(left.attention_weight, right.attention_weight));
        for (left_value, right_value) in left
            .key_theta
            .coefficients
            .into_iter()
            .zip(right.key_theta.coefficients)
        {
            assert!(close(left_value, right_value));
        }
        for (left_value, right_value) in left
            .value_theta
            .coefficients
            .into_iter()
            .zip(right.value_theta.coefficients)
        {
            assert!(close(left_value, right_value));
        }
    }
    for (left, right) in actual.scores.iter().zip(&reference.scores) {
        assert!(left.token == right.token);
        assert!(close(left.score, right.score));
        for (left_value, right_value) in left
            .output_theta
            .coefficients
            .into_iter()
            .zip(right.output_theta.coefficients)
        {
            assert!(close(left_value, right_value));
        }
    }
    for (left, right) in actual
        .query_theta
        .coefficients
        .into_iter()
        .zip(reference.query_theta.coefficients)
    {
        assert!(close(left, right));
    }
    for (left, right) in actual
        .aggregate_local_coordinates
        .into_iter()
        .zip(reference.aggregate_local_coordinates)
    {
        assert!(close(left, right));
    }
}

fn assert_causal_work_and_finite(
    trace: &ConnectionGaugeCovarianceV4Trace,
    prefix_len: usize,
    current_only: bool,
) {
    let attended = if current_only { 1 } else { prefix_len };
    assert!(trace.input_position_count == prefix_len);
    assert!(trace.query_position + 1 == prefix_len);
    assert!(trace.causal_prefix_position_count == prefix_len);
    assert!(trace.masked_future_position_count == 0);
    assert!(trace.maximum_position_read == trace.query_position);
    assert!(trace.future_token_reads == 0);
    assert!(trace.causal_token_value_reads == attended as u64);
    assert!(trace.positions.len() == attended);
    assert!(trace.q_projections == 1);
    assert!(trace.k_projections == attended as u64);
    assert!(trace.v_projections == attended as u64);
    assert!(trace.o_projections == SUPPORT.len() as u64);
    assert!(trace.key_transports == attended as u64);
    assert!(trace.value_transports == attended as u64);
    assert!(trace.output_transports == SUPPORT.len() as u64);
    assert!(trace.stored_scalar_parameter_count == EXPECTED_PARAMETER_COUNT_PER_ARM);
    assert!(trace.learned_effective_degree_count == EXPECTED_PARAMETER_COUNT_PER_ARM);
    assert!(trace.query_token == QUERY_TOKEN);
    assert!(trace.admitted_support == SUPPORT);
    assert!(trace.query_tangent_residual.is_finite());
    assert!(trace.query_tangent_residual <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE);
    assert!(trace.softmax_weight_sum.is_finite());
    assert!(
        (trace.softmax_weight_sum - 1.0).abs()
            <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
    );
    assert!(trace.aggregate_value.into_iter().all(f64::is_finite));
    assert!(
        trace
            .aggregate_local_coordinates
            .into_iter()
            .all(f64::is_finite)
    );
    assert!(trace.positions.iter().all(|position| {
        position.attention_logit.is_finite()
            && position.attention_weight.is_finite()
            && position.transported_key_tangent_residual.is_finite()
            && position.transported_key_tangent_residual
                <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
            && position.transported_value_tangent_residual.is_finite()
            && position.transported_value_tangent_residual
                <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
    }));
    assert!(trace.scores.iter().all(|candidate| {
        candidate.score.is_finite()
            && candidate.output_tangent_residual.is_finite()
            && candidate.output_tangent_residual
                <= CONNECTION_GAUGE_COVARIANCE_V4_STRUCTURAL_TOLERANCE
    }));
}

fn assert_model_identity(model: &ConnectionGaugeCovarianceV4) {
    assert!(model.maximum_token_id() == MAXIMUM_TOKEN_ID);
    assert!(model.construction_event_count() == EXPECTED_CONSTRUCTION_EVENT_COUNT);
    assert!(model.policy_identity() == CONNECTION_GAUGE_COVARIANCE_V4_POLICY);
    assert!(model.generator_policy_identity() == CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY);
    assert!(model.artifact_cid() == FROZEN_PHASE_I_ARTIFACT_CID);
    assert!(model.core_freeze_cid() == FROZEN_PHASE_I_CORE_CID);
    assert!(model.construction_population_kappa() == FROZEN_PHASE_I_CONSTRUCTION_KAPPA);
    assert!(model.canonical_frame_manifest_cid() == FROZEN_PHASE_I_FRAME_CID);
    assert!(
        model.learning_update_counts_for_arm(ConnectionGaugeCovarianceV4Arm::H4Compatible)
            == [105, 105, 105, 105]
    );
    assert!(
        model.learning_update_counts_for_arm(ConnectionGaugeCovarianceV4Arm::AlternativeTangent)
            == [105, 105, 105, 105]
    );
    assert!(
        model.learning_update_counts_for_arm(ConnectionGaugeCovarianceV4Arm::PlainFixed)
            == [105, 105, 105, 105]
    );
    assert!(
        model.learning_update_counts_for_arm(ConnectionGaugeCovarianceV4Arm::CurrentTokenOnly)
            == [1280, 1280, 1280, 1280]
    );
    for arm in FROZEN_BINDINGS.map(|binding| binding.arm) {
        assert!(model.initialization_cid(arm) == FROZEN_PHASE_I_INITIALIZATION_CID);
    }
}

fn stream_header(model: &ConnectionGaugeCovarianceV4) -> Vec<u8> {
    assert_model_identity(model);
    let mut bytes = tag(STREAM_DOMAIN);
    push_lp64(&mut bytes, PHASE_I_PROTECTED_MERGE_SHA.as_bytes());
    push_lp64(&mut bytes, PHASE_II_PROTECTED_MERGE_SHA.as_bytes());
    for cid in [
        FROZEN_PHASE_I_PREFLIGHT_CID,
        FROZEN_PHASE_I_CORE_CID,
        FROZEN_PHASE_I_ARTIFACT_CID,
        FROZEN_PHASE_I_INITIALIZATION_CID,
        FROZEN_PHASE_I_CONSTRUCTION_KAPPA,
        FROZEN_PHASE_I_FRAME_CID,
        FROZEN_GENERATOR_POLICY_CID,
        FROZEN_FORBIDDEN_ROOT,
        FROZEN_VALIDATION_PREFIX_ROOT,
        FROZEN_VALIDATION_INPUT_CID,
        FROZEN_COMMITMENT_CID,
        FROZEN_PRELABEL_CID,
    ] {
        bytes.extend_from_slice(&parse_raw_cid(cid));
    }
    push_lp64(&mut bytes, CONNECTION_GAUGE_COVARIANCE_V4_POLICY.as_bytes());
    push_lp64(
        &mut bytes,
        CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY.as_bytes(),
    );
    bytes.extend_from_slice(&MAXIMUM_TOKEN_ID.to_le_bytes());
    bytes.extend_from_slice(&FROZEN_CONFIG.epochs.to_le_bytes());
    bytes.extend_from_slice(&FROZEN_CONFIG.learning_rate.to_bits().to_le_bytes());
    bytes.extend_from_slice(&FROZEN_CONFIG.temperature.to_bits().to_le_bytes());
    bytes.extend_from_slice(&REQUIRED_SCORE_GAP.to_bits().to_le_bytes());
    bytes.extend_from_slice(&token_bytes(&SUPPORT));
    bytes.extend_from_slice(&(FROZEN_CASES.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(FROZEN_BINDINGS.len() as u64).to_le_bytes());
    for binding in FROZEN_BINDINGS {
        push_lp64(&mut bytes, binding.name.as_bytes());
        push_lp64(&mut bytes, binding.arm_name.as_bytes());
        push_lp64(&mut bytes, binding.intervention_name.as_bytes());
    }
    bytes
}

fn prediction_stream(model: &ConnectionGaugeCovarianceV4) -> (Vec<u8>, Vec<PredictionRow>) {
    let mut bytes = stream_header(model);
    let mut rows = Vec::with_capacity(FROZEN_CASES.len() * FROZEN_BINDINGS.len());
    for (case_index, case) in FROZEN_CASES.iter().enumerate() {
        let case_index = u16::try_from(case_index).expect("case index fits u16");
        let case_raw = parse_raw_cid(case.case_id);
        assert!(case_id(case.prefix) == case_raw);
        bytes.extend_from_slice(&case_index.to_le_bytes());
        push_lp64(&mut bytes, &case_raw);
        let prefix_bytes = token_bytes(case.prefix);
        push_lp64(&mut bytes, &prefix_bytes);

        let traces = FROZEN_BINDINGS.map(|binding| {
            let trace = model
                .predict_prefix(case.prefix, &SUPPORT, binding.arm, binding.intervention)
                .expect("frozen prediction succeeds");
            assert!(trace.arm == binding.arm);
            assert!(trace.intervention == binding.intervention);
            assert_causal_work_and_finite(&trace, case.prefix.len(), binding.current_only);
            let score_5 = score(&trace, SUPPORT[0]);
            let score_6 = score(&trace, SUPPORT[1]);
            assert!((score_5 - score_6).abs() >= REQUIRED_SCORE_GAP);
            trace
        });
        assert_main_parity(&traces[0], &traces[1]);
        assert_main_parity(&traces[0], &traces[2]);
        assert!(
            traces[4]
                .positions
                .iter()
                .zip(&traces[0].positions)
                .any(|(control, baseline)| {
                    control.observed_token != baseline.observed_token
                        || control.key_source_token != baseline.key_source_token
                        || !close(control.attention_logit, baseline.attention_logit)
                })
        );
        assert!(
            traces[5]
                .positions
                .iter()
                .zip(&traces[0].positions)
                .any(|(control, baseline)| {
                    control.value_source_token != baseline.value_source_token
                })
        );
        assert!(
            traces[5]
                .positions
                .iter()
                .zip(&traces[0].positions)
                .all(|(control, baseline)| {
                    control.attention_logit.to_bits() == baseline.attention_logit.to_bits()
                })
        );
        assert!(
            traces[6]
                .positions
                .iter()
                .zip(&traces[0].positions)
                .any(|(control, baseline)| {
                    !close(control.attention_logit, baseline.attention_logit)
                })
        );

        for (binding, trace) in FROZEN_BINDINGS.into_iter().zip(traces) {
            let score_5 = score(&trace, SUPPORT[0]);
            let score_6 = score(&trace, SUPPORT[1]);
            let row = PredictionRow {
                case_index,
                case_id: case.case_id,
                binding_name: binding.name,
                selected_token: trace.selected_token,
                score_5_bits: score_5.to_bits(),
                score_6_bits: score_6.to_bits(),
                absolute_score_gap_bits: (score_5 - score_6).abs().to_bits(),
            };
            push_lp64(&mut bytes, binding.name.as_bytes());
            bytes.extend_from_slice(&row.selected_token.to_le_bytes());
            bytes.extend_from_slice(&row.score_5_bits.to_le_bytes());
            bytes.extend_from_slice(&row.score_6_bits.to_le_bytes());
            bytes.extend_from_slice(&row.absolute_score_gap_bits.to_le_bytes());
            let complete_trace =
                serde_json::to_vec(&trace).expect("finite frozen trace serializes canonically");
            push_lp64(&mut bytes, &complete_trace);
            rows.push(row);
        }
    }
    (bytes, rows)
}

#[test]
#[ignore = "one-time Phase-III runner; execute only from the protected Phase-II merge"]
fn produce_canonical_prediction_stream_once() {
    assert!(PHASE_I_PROTECTED_MERGE_SHA.len() == 40);
    assert!(PHASE_II_PROTECTED_MERGE_SHA.len() == 40);
    assert!(
        PHASE_I_PROTECTED_MERGE_SHA
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert!(
        PHASE_II_PROTECTED_MERGE_SHA
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert!(
        format!(
            "blake3:{}",
            blake3::hash(CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY.as_bytes()).to_hex()
        ) == FROZEN_GENERATOR_POLICY_CID
    );

    let first = compile_model();
    let second = compile_model();
    assert!(first.to_bytes() == second.to_bytes());
    let (first_bytes, first_rows) = prediction_stream(&first);
    let (second_bytes, second_rows) = prediction_stream(&second);
    assert!(first_bytes == second_bytes);
    assert!(first_rows == second_rows);
    assert!(first_rows.len() == FROZEN_CASES.len() * FROZEN_BINDINGS.len());

    let stream_cid = format!("blake3:{}", blake3::hash(&first_bytes).to_hex());
    println!("CGCV_973_PHASE_III_PREDICTION_STREAM_CID={stream_cid}");
    for row in first_rows {
        println!(
            "CGCV_973_PHASE_III_PREDICTION case_index={} case_id={} binding={} selected_token={} token_5_score_bits=0x{:016x} token_6_score_bits=0x{:016x} absolute_score_gap_bits=0x{:016x}",
            row.case_index,
            row.case_id,
            row.binding_name,
            row.selected_token,
            row.score_5_bits,
            row.score_6_bits,
            row.absolute_score_gap_bits,
        );
    }
}
