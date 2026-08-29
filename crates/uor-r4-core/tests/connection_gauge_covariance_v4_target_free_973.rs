//! Phase-II target-free input freeze for `ConnectionGaugeCovarianceV4`.
//!
//! This file imports only the frozen generator-policy literal. It neither
//! constructs the attention mechanism nor loads any target row.

use std::collections::BTreeSet;

use uor_r4_core::direct_causal_geometric_attention::CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY;

const PHASE_I_PROTECTED_MERGE_SHA: &str = "b054197acb92e3dd23d88d81bd859379ea8fac67";
const QUERY_TOKEN: u32 = 1;
const SUPPORT: [u32; 2] = [5, 6];
const REQUIRED_PAIR_COUNT: usize = 12;
const REQUIRED_CASE_COUNT: usize = REQUIRED_PAIR_COUNT * 2;
const PREDICTION_COUNT: u64 = 0;
const SCORING_LABEL_JOIN_COUNT: u64 = 0;

const PAIR_ORDER_DOMAIN: &str = "uor-r4.cgcv-v4.pair-order/1";
const UNIT_ORDER_DOMAIN: &str = "uor-r4.cgcv-v4.unit-order/1";
const FORBIDDEN_ROOT_DOMAIN: &str = "uor-r4.cgcv-v4.forbidden-prefix-root/1";
const CASE_ID_DOMAIN: &str = "uor-r4.cgcv-v4.case-id/1";
const VALIDATION_PREFIX_ROOT_DOMAIN: &str = "uor-r4.cgcv-v4.validation-prefix-root/1";
const VALIDATION_INPUT_DOMAIN: &str = "uor-r4.cgcv-v4.validation-input/1";
const SALTED_LABEL_COMMITMENT_DOMAIN: &str = "uor-r4.cgcv-v4.salted-label-commitment/1";
const PRELABEL_FREEZE_DOMAIN: &str = "uor-r4.cgcv-v4.prelabel-freeze/1";
const PRELABEL_STATUS: &str = "inputs=FROZEN;labels=SEALED;predictions=NOT_RUN;scores=NOT_RUN";

const MAXIMUM_TOKEN_ID: u32 = 12;
const CONSTRUCTION_PREFIX_COUNT: u64 = 16;
const V2_PREFIX_COUNT: u64 = 8;
const V3_PREFIX_COUNT: u64 = 12;
const BASE_FORBIDDEN_COUNT: u64 = CONSTRUCTION_PREFIX_COUNT + V2_PREFIX_COUNT + V3_PREFIX_COUNT;

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
const FROZEN_PHASE_I_FRAME_MANIFEST_CID: &str =
    "blake3:205ee0d1b9aebbee2475d97de3b95d359ff2ee8220334995cfe4c7a71ead5920";
const FROZEN_V2_VALIDATION_INPUT_KAPPA: &str =
    "blake3:2b2448e51821b2c003ca5cdede0d667fd22def6880003da8f54c38c74a80c09c";
const FROZEN_V3_VALIDATION_INPUT_KAPPA: &str =
    "blake3:c6c5d6d3ec1af4aaa419ce1857bfe5e389d4a3e7a963d6a87b16d2161809829d";

const FROZEN_GENERATOR_POLICY_CID: &str =
    "blake3:73b4233b0b91ba85ffb6cd8c3d86132a954e4fbda5c7ec57510cc30bd9fb5dca";
const FROZEN_FORBIDDEN_ROOT: &str =
    "blake3:877707eff60857b9c790cfb0e8a2a5a12bbcadb51d3448c9bd7119d5b86b6c42";
const FROZEN_VALIDATION_PREFIX_ROOT: &str =
    "blake3:a3321b13d808d553d7588997f8fb7951be33e254724d45a1223460dd775a3ad8";
const FROZEN_VALIDATION_INPUT_CID: &str =
    "blake3:5a17c5526d866f2862b042750cb70f5183f6a8fc09ab53a067d79d28d1c989d1";
const FROZEN_SALTED_LABEL_COMMITMENT_CID: &str =
    "blake3:9773355914ed171f0d14950a4db554f5f543252804c703e8e0bbbbf17fe7b602";
const FROZEN_PRELABEL_FREEZE_CID: &str =
    "blake3:170419cfcf80b2b0e48cc74faff13c9791dd9106045a1ff59a82efe4f6b205aa";

// The protected Phase-II Git commit binds these comparator choices before
// reveal. They are deliberately not fields in the already-published prelabel
// byte contract; Phase III must assert these exact enum spellings and pairs.
const FROZEN_CONTROL_BINDINGS: [(&str, &str, &str); 7] = [
    ("baseline", "h4_compatible", "none"),
    ("alternative", "alternative_tangent", "none"),
    ("plain", "plain_fixed", "none"),
    ("current_only", "current_token_only", "none"),
    ("order_shuffled", "h4_compatible", "order_shuffled"),
    ("value_permuted", "h4_compatible", "value_permuted"),
    (
        "source_gauge_mismatch",
        "h4_compatible",
        "source_gauge_mismatched",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FrozenTargetFreeCase {
    pair_counter: u16,
    case_id: &'static str,
    prefix: &'static [u32],
}

// Canonical selected order: accepted-pair order, then unswapped endpoint and
// mate. The displayed case ID is `blake3:<lower-hex>`; the validation-input
// root length-prefixes the underlying raw 32-byte digest, never this text.
const FROZEN_TARGET_FREE_CASES: [FrozenTargetFreeCase; REQUIRED_CASE_COUNT] = [
    FrozenTargetFreeCase {
        pair_counter: 30149,
        case_id: "blake3:1a3d8f6b50f3dbf5b23572858b9e69c4bb7ee58318ce03ecd0d78f8f31844679",
        prefix: &[8, 4, 10, 11, 12, 1, 5, 2, 9, 6, 7, 3, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 30149,
        case_id: "blake3:fe99823d4f6c78d1e906a4690fad055eb48ac1eef34323862ff4b8f30e6db13c",
        prefix: &[8, 4, 10, 11, 12, 1, 6, 2, 9, 5, 7, 3, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 53145,
        case_id: "blake3:215cf21bd7ed218a7903f6285674fb7c54e83bb14490d624a215d5e670f36848",
        prefix: &[6, 7, 4, 11, 10, 8, 9, 1, 5, 3, 2, 12, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 53145,
        case_id: "blake3:3dbbb505382718dfe267670829df3cc4a28b0607c855b364fb17f062a8a000d7",
        prefix: &[5, 7, 4, 11, 10, 8, 9, 1, 6, 3, 2, 12, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 27994,
        case_id: "blake3:13f215def7d469a42abae814d3cbffdca0be193315dd582ce910860968b55586",
        prefix: &[12, 8, 7, 3, 9, 6, 2, 11, 1, 5, 10, 4, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 27994,
        case_id: "blake3:e5778dd9fbcdeb9a20892602ef49e5ea2f6c46690b2ff951adffbf377fa08a79",
        prefix: &[12, 8, 7, 3, 9, 5, 2, 11, 1, 6, 10, 4, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 21913,
        case_id: "blake3:c43b48eb2dfe758b9da850f0fb5ee21e6b750fcbdc89e650e9b3844fe989d63c",
        prefix: &[2, 4, 7, 1, 5, 3, 12, 6, 9, 8, 11, 10, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 21913,
        case_id: "blake3:d16ff997fdc4cdfad24e0aab1aa15f28093f96cab8b79995cc1301b8e10d0cbb",
        prefix: &[2, 4, 7, 1, 6, 3, 12, 5, 9, 8, 11, 10, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 21781,
        case_id: "blake3:e173d1b98d789493cb5c71db7689b35c76c8e383690f6132e344dd860da54860",
        prefix: &[4, 9, 8, 6, 7, 1, 5, 3, 10, 11, 2, 12, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 21781,
        case_id: "blake3:2fceb1c887b8f6faf412c94f5aebe0a7668fad11be6f83de1ddb30410e04374c",
        prefix: &[4, 9, 8, 5, 7, 1, 6, 3, 10, 11, 2, 12, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 64005,
        case_id: "blake3:b9557a9e58757b3411206813e8dec75a677d7a438af1947ef386a8f73ba768d6",
        prefix: &[11, 12, 1, 5, 6, 2, 4, 8, 3, 9, 7, 10, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 64005,
        case_id: "blake3:53153c2dd34b4473bfa483ba091645b8b5de3f296dcc79ca699cb01631a242aa",
        prefix: &[11, 12, 1, 6, 5, 2, 4, 8, 3, 9, 7, 10, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 53150,
        case_id: "blake3:f3a6625df4f281a5cef71f4e8020657ae61ec89d24e24bd1946c144a5d33b491",
        prefix: &[2, 1, 5, 9, 11, 10, 3, 8, 12, 4, 7, 6, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 53150,
        case_id: "blake3:ff92d321bb0497c13d5b28d978e2055dab20b02accb08888bea4119dcee5b6d0",
        prefix: &[2, 1, 6, 9, 11, 10, 3, 8, 12, 4, 7, 5, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 46433,
        case_id: "blake3:cb6c3aeaaedb0a696f08009f664cf8abae4e3b313ae3d7181711d28035211b42",
        prefix: &[10, 9, 12, 8, 2, 11, 7, 1, 5, 4, 3, 6, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 46433,
        case_id: "blake3:3753d93a9ff3bd9f1b2b6092307f68e3f7d9fc3436eed26f6c25788e59c52ab9",
        prefix: &[10, 9, 12, 8, 2, 11, 7, 1, 6, 4, 3, 5, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 44855,
        case_id: "blake3:67839ab0dc2ee3cf90fb3c1524eb77dd8fb7b0348d04730e191c7b104d640789",
        prefix: &[8, 6, 3, 7, 2, 1, 5, 12, 10, 4, 11, 9, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 44855,
        case_id: "blake3:b1846c2774efb974e6841abec35c37c240577deda45afb12b72afaed42b80b80",
        prefix: &[8, 5, 3, 7, 2, 1, 6, 12, 10, 4, 11, 9, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 31599,
        case_id: "blake3:95c68874f3c7fc81fc8cb0c082dbbee865f12046b0fd989a8390926d1452edba",
        prefix: &[3, 9, 2, 8, 12, 11, 7, 4, 1, 5, 6, 10, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 31599,
        case_id: "blake3:e00a31827260f55169d7ce998437588539a33c8c3ef6f804e3e45e6293591e7f",
        prefix: &[3, 9, 2, 8, 12, 11, 7, 4, 1, 6, 5, 10, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 62881,
        case_id: "blake3:a8e1c0288b571bb48bc2f04f3a5141586e79722bf86c30ad4cd9249051c64009",
        prefix: &[1, 5, 2, 4, 11, 9, 10, 7, 3, 12, 8, 6, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 62881,
        case_id: "blake3:e08e57eff49dfd07c89609edad36c4b1257cd4035ef4c9ac6f28374ed1062bf8",
        prefix: &[1, 6, 2, 4, 11, 9, 10, 7, 3, 12, 8, 5, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 20555,
        case_id: "blake3:64500c16993cf62b0a4e920f8fd171527fcdd978d488619f1f6f37d9cb3e51b5",
        prefix: &[12, 8, 3, 4, 6, 9, 1, 5, 7, 11, 10, 2, 1],
    },
    FrozenTargetFreeCase {
        pair_counter: 20555,
        case_id: "blake3:637591501c3d0d3909a9c734d092a5f70739c3360c833326a74f9452c4596999",
        prefix: &[12, 8, 3, 4, 5, 9, 1, 6, 7, 11, 10, 2, 1],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetFreeCase {
    case_id: [u8; 32],
    prefix: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetFreePair {
    counter: u16,
    unswapped: TargetFreeCase,
    mate: TargetFreeCase,
}

fn tag(domain: &str) -> Vec<u8> {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn length_prefixed(bytes: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + bytes.len());
    encoded.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    encoded.extend_from_slice(bytes);
    encoded
}

fn canonical_prefix_bytes(prefix: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + prefix.len() * 4);
    bytes.extend_from_slice(&(prefix.len() as u64).to_le_bytes());
    for token in prefix {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    bytes
}

fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

fn display_digest(value: [u8; 32]) -> String {
    format!("blake3:{}", blake3::Hash::from_bytes(value).to_hex())
}

fn parse_raw_cid(value: &str) -> Result<[u8; 32], &'static str> {
    let encoded = value.as_bytes();
    if encoded.len() != 71 || !encoded.starts_with(b"blake3:") {
        return Err("CID must be blake3: followed by exactly 64 lowercase hex bytes");
    }
    let mut raw = [0_u8; 32];
    for (index, pair) in encoded[7..].chunks_exact(2).enumerate() {
        let high = match pair[0] {
            b'0'..=b'9' => pair[0] - b'0',
            b'a'..=b'f' => pair[0] - b'a' + 10,
            _ => return Err("CID payload must be lowercase hexadecimal"),
        };
        let low = match pair[1] {
            b'0'..=b'9' => pair[1] - b'0',
            b'a'..=b'f' => pair[1] - b'a' + 10,
            _ => return Err("CID payload must be lowercase hexadecimal"),
        };
        raw[index] = (high << 4) | low;
    }
    Ok(raw)
}

fn required_raw_cid(value: &str) -> [u8; 32] {
    parse_raw_cid(value).expect("frozen CID must use the strict canonical text form")
}

fn pair_order_key(seed: &[u8], counter: u16) -> [u8; 32] {
    digest(&[
        &tag(PAIR_ORDER_DOMAIN),
        &length_prefixed(seed),
        &counter.to_le_bytes(),
    ])
}

fn units() -> Vec<Vec<u32>> {
    vec![
        vec![1, 5],
        vec![6],
        vec![2],
        vec![3],
        vec![4],
        vec![7],
        vec![8],
        vec![9],
        vec![10],
        vec![11],
        vec![12],
    ]
}

fn unit_order_key(seed: &[u8], counter: u16, index: u16, unit: &[u32]) -> [u8; 32] {
    digest(&[
        &tag(UNIT_ORDER_DOMAIN),
        &length_prefixed(seed),
        &counter.to_le_bytes(),
        &index.to_le_bytes(),
        &canonical_prefix_bytes(unit),
    ])
}

fn candidate_prefix(seed: &[u8], counter: u16) -> Vec<u32> {
    let units = units();
    let mut ordered = units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            let index = u16::try_from(index).expect("eleven unit indexes fit u16");
            (unit_order_key(seed, counter, index, unit), index, unit)
        })
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let mut prefix = ordered
        .into_iter()
        .flat_map(|(_, _, unit)| unit.iter().copied())
        .collect::<Vec<_>>();
    prefix.push(QUERY_TOKEN);
    prefix
}

fn mate(prefix: &[u32]) -> Vec<u32> {
    prefix
        .iter()
        .map(|token| match *token {
            5 => 6,
            6 => 5,
            other => other,
        })
        .collect()
}

fn is_proper_prefix(left: &[u32], right: &[u32]) -> bool {
    left.len() < right.len() && right.starts_with(left)
}

fn antichain(left: &[u32], right: &[u32]) -> bool {
    left != right && !is_proper_prefix(left, right) && !is_proper_prefix(right, left)
}

fn construction_prefixes() -> Vec<Vec<u32>> {
    vec![
        vec![1, 5, 2, 6, 9, 1],
        vec![1, 6, 2, 5, 9, 1],
        vec![2, 6, 1, 5, 8, 1],
        vec![2, 5, 1, 6, 8, 1],
        vec![3, 1, 5, 4, 2, 6, 9, 1],
        vec![4, 1, 6, 3, 2, 5, 9, 1],
        vec![2, 6, 7, 1, 5, 8, 1],
        vec![2, 5, 7, 1, 6, 8, 1],
        vec![1, 5, 3, 2, 6, 4, 1],
        vec![1, 6, 3, 2, 5, 4, 1],
        vec![2, 6, 3, 1, 5, 7, 8, 1],
        vec![2, 5, 3, 1, 6, 7, 8, 1],
        vec![10, 1, 5, 11, 2, 6, 12, 1],
        vec![10, 1, 6, 11, 2, 5, 12, 1],
        vec![2, 6, 4, 1, 5, 3, 7, 1],
        vec![2, 5, 4, 1, 6, 3, 7, 1],
    ]
}

fn v2_prefixes() -> Vec<Vec<u32>> {
    vec![
        vec![3, 2, 6, 4, 1, 5, 7, 8, 1],
        vec![8, 2, 5, 3, 4, 1, 6, 7, 1],
        vec![10, 1, 5, 3, 4, 2, 6, 11, 12, 1],
        vec![11, 2, 5, 7, 1, 6, 3, 4, 1],
        vec![4, 2, 6, 8, 3, 1, 5, 10, 1],
        vec![12, 1, 6, 7, 3, 2, 5, 8, 4, 1],
        vec![2, 6, 10, 11, 1, 5, 3, 7, 8, 1],
        vec![3, 1, 6, 10, 2, 5, 11, 4, 7, 1],
    ]
}

fn v3_prefixes() -> Vec<Vec<u32>> {
    vec![
        vec![7, 1, 5, 12, 4, 9, 2, 10, 6, 3, 8, 1],
        vec![5, 8, 1, 6, 11, 3, 9, 2, 12, 4, 7, 1],
        vec![11, 4, 2, 1, 5, 10, 6, 7, 3, 12, 1],
        vec![10, 3, 7, 1, 6, 12, 5, 8, 2, 11, 1],
        vec![6, 9, 3, 8, 1, 5, 12, 2, 4, 10, 7, 1],
        vec![4, 12, 9, 2, 1, 6, 8, 3, 10, 5, 7, 1],
        vec![12, 3, 7, 2, 9, 1, 5, 4, 11, 6, 8, 10, 1],
        vec![8, 5, 2, 11, 7, 1, 6, 3, 12, 4, 9, 10, 1],
        vec![4, 10, 1, 5, 8, 12, 3, 6, 11, 2, 9, 7, 1],
        vec![3, 9, 1, 6, 7, 11, 4, 5, 10, 2, 12, 8, 1],
        vec![9, 2, 11, 6, 4, 1, 5, 10, 3, 8, 12, 7, 1],
        vec![12, 7, 4, 10, 2, 1, 6, 9, 5, 11, 3, 8, 1],
    ]
}

fn base_forbidden_prefixes() -> Vec<Vec<u32>> {
    construction_prefixes()
        .into_iter()
        .chain(v2_prefixes())
        .chain(v3_prefixes())
        .collect()
}

fn forbidden_root(prefixes: &[Vec<u32>]) -> [u8; 32] {
    let mut canonical = prefixes
        .iter()
        .map(|prefix| canonical_prefix_bytes(prefix))
        .collect::<Vec<_>>();
    canonical.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&tag(FORBIDDEN_ROOT_DOMAIN));
    hasher.update(&(canonical.len() as u64).to_le_bytes());
    for prefix in canonical {
        hasher.update(&length_prefixed(&prefix));
    }
    *hasher.finalize().as_bytes()
}

fn case_id(prefix: &[u32]) -> [u8; 32] {
    digest(&[
        &tag(CASE_ID_DOMAIN),
        &length_prefixed(&canonical_prefix_bytes(prefix)),
    ])
}

fn eligible(prefix: &[u32], mate: &[u32], dynamic_forbidden: &[Vec<u32>]) -> bool {
    prefix != mate
        && prefix.last() == Some(&QUERY_TOKEN)
        && mate.last() == Some(&QUERY_TOKEN)
        && prefix[..prefix.len() - 1]
            .iter()
            .filter(|token| **token == QUERY_TOKEN)
            .count()
            == 1
        && mate[..mate.len() - 1]
            .iter()
            .filter(|token| **token == QUERY_TOKEN)
            .count()
            == 1
        && antichain(prefix, mate)
        && [prefix, mate].into_iter().all(|candidate| {
            dynamic_forbidden
                .iter()
                .all(|prior| antichain(candidate, prior))
        })
}

fn generate_pairs(seed: &[u8], forbidden: &[Vec<u32>]) -> Vec<TargetFreePair> {
    let mut candidate_order = (u16::MIN..=u16::MAX)
        .map(|counter| (pair_order_key(seed, counter), counter))
        .collect::<Vec<_>>();
    candidate_order.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let mut dynamic_forbidden = forbidden.to_vec();
    let mut selected = Vec::with_capacity(REQUIRED_PAIR_COUNT);
    for (_, counter) in candidate_order {
        let prefix = candidate_prefix(seed, counter);
        let paired = mate(&prefix);
        if !eligible(&prefix, &paired, &dynamic_forbidden) {
            continue;
        }
        let unswapped = TargetFreeCase {
            case_id: case_id(&prefix),
            prefix,
        };
        let mate = TargetFreeCase {
            case_id: case_id(&paired),
            prefix: paired,
        };
        dynamic_forbidden.push(unswapped.prefix.clone());
        dynamic_forbidden.push(mate.prefix.clone());
        selected.push(TargetFreePair {
            counter,
            unswapped,
            mate,
        });
        if selected.len() == REQUIRED_PAIR_COUNT {
            break;
        }
    }
    selected
}

fn validation_prefix_root(pairs: &[TargetFreePair]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&tag(VALIDATION_PREFIX_ROOT_DOMAIN));
    hasher.update(&(pairs.len() as u64 * 2).to_le_bytes());
    for case in pairs.iter().flat_map(|pair| [&pair.unswapped, &pair.mate]) {
        hasher.update(&length_prefixed(&case.case_id));
        hasher.update(&length_prefixed(&canonical_prefix_bytes(&case.prefix)));
    }
    *hasher.finalize().as_bytes()
}

fn validation_input_bytes(
    pairs: &[TargetFreePair],
    forbidden_root: [u8; 32],
    prefix_root: [u8; 32],
    structural_five_count: u64,
    structural_six_count: u64,
) -> Vec<u8> {
    let mut bytes = tag(VALIDATION_INPUT_DOMAIN);
    bytes.extend_from_slice(&length_prefixed(PHASE_I_PROTECTED_MERGE_SHA.as_bytes()));
    bytes.extend_from_slice(&length_prefixed(
        CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY.as_bytes(),
    ));
    for identity in [
        FROZEN_PHASE_I_PREFLIGHT_CID,
        FROZEN_PHASE_I_CORE_CID,
        FROZEN_PHASE_I_ARTIFACT_CID,
        FROZEN_PHASE_I_INITIALIZATION_CID,
        FROZEN_PHASE_I_CONSTRUCTION_KAPPA,
        FROZEN_PHASE_I_FRAME_MANIFEST_CID,
        FROZEN_V2_VALIDATION_INPUT_KAPPA,
        FROZEN_V3_VALIDATION_INPUT_KAPPA,
    ] {
        bytes.extend_from_slice(&required_raw_cid(identity));
    }
    bytes.extend_from_slice(&MAXIMUM_TOKEN_ID.to_le_bytes());
    bytes.extend_from_slice(&QUERY_TOKEN.to_le_bytes());
    bytes.extend_from_slice(&canonical_prefix_bytes(&SUPPORT));
    bytes.extend_from_slice(&CONSTRUCTION_PREFIX_COUNT.to_le_bytes());
    bytes.extend_from_slice(&V2_PREFIX_COUNT.to_le_bytes());
    bytes.extend_from_slice(&V3_PREFIX_COUNT.to_le_bytes());
    bytes.extend_from_slice(&BASE_FORBIDDEN_COUNT.to_le_bytes());
    bytes.extend_from_slice(&forbidden_root);
    bytes.extend_from_slice(&(pairs.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(pairs.len() as u64 * 2).to_le_bytes());
    for pair in pairs {
        bytes.extend_from_slice(&pair.counter.to_le_bytes());
    }
    bytes.extend_from_slice(&prefix_root);
    bytes.extend_from_slice(&structural_five_count.to_le_bytes());
    bytes.extend_from_slice(&structural_six_count.to_le_bytes());
    bytes
}

fn prelabel_freeze_bytes(
    forbidden_root: [u8; 32],
    prefix_root: [u8; 32],
    validation_input_cid: [u8; 32],
    label_commitment_cid: [u8; 32],
) -> Vec<u8> {
    let mut bytes = tag(PRELABEL_FREEZE_DOMAIN);
    bytes.extend_from_slice(&length_prefixed(PHASE_I_PROTECTED_MERGE_SHA.as_bytes()));
    for identity in [
        FROZEN_PHASE_I_PREFLIGHT_CID,
        FROZEN_PHASE_I_CORE_CID,
        FROZEN_PHASE_I_ARTIFACT_CID,
        FROZEN_PHASE_I_INITIALIZATION_CID,
        FROZEN_PHASE_I_CONSTRUCTION_KAPPA,
        FROZEN_PHASE_I_FRAME_MANIFEST_CID,
    ] {
        bytes.extend_from_slice(&required_raw_cid(identity));
    }
    bytes.extend_from_slice(&forbidden_root);
    bytes.extend_from_slice(&prefix_root);
    bytes.extend_from_slice(&validation_input_cid);
    bytes.extend_from_slice(&label_commitment_cid);
    bytes.extend_from_slice(&(REQUIRED_PAIR_COUNT as u64).to_le_bytes());
    bytes.extend_from_slice(&(REQUIRED_CASE_COUNT as u64).to_le_bytes());
    bytes.extend_from_slice(&12_u64.to_le_bytes());
    bytes.extend_from_slice(&12_u64.to_le_bytes());
    bytes.extend_from_slice(&PREDICTION_COUNT.to_le_bytes());
    bytes.extend_from_slice(&SCORING_LABEL_JOIN_COUNT.to_le_bytes());
    bytes.extend_from_slice(&length_prefixed(PRELABEL_STATUS.as_bytes()));
    bytes
}

fn salted_label_commitment_bytes(
    validation_input_cid: [u8; 32],
    nonce: [u8; 32],
    label_rows: &[([u8; 32], u32)],
) -> Result<Vec<u8>, &'static str> {
    if label_rows.len() != REQUIRED_CASE_COUNT {
        return Err("the V4 commitment preimage must contain exactly 24 label rows");
    }
    let mut bytes = tag(SALTED_LABEL_COMMITMENT_DOMAIN);
    bytes.extend_from_slice(&validation_input_cid);
    bytes.extend_from_slice(&32_u64.to_le_bytes());
    bytes.extend_from_slice(&nonce);
    bytes.extend_from_slice(&(label_rows.len() as u64).to_le_bytes());
    for (case_id, target) in label_rows {
        bytes.extend_from_slice(&length_prefixed(case_id));
        bytes.extend_from_slice(&target.to_le_bytes());
    }
    Ok(bytes)
}

fn structural_binding(prefix: &[u32]) -> Option<u32> {
    let earlier_query = prefix[..prefix.len().checked_sub(1)?]
        .iter()
        .position(|token| *token == QUERY_TOKEN)?;
    prefix.get(earlier_query + 1).copied()
}

#[test]
fn phase_ii_target_free_population_is_deterministic_disjoint_and_unscored() {
    assert_eq!(PHASE_I_PROTECTED_MERGE_SHA.len(), 40);
    assert!(PHASE_I_PROTECTED_MERGE_SHA
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    let seed = PHASE_I_PROTECTED_MERGE_SHA.as_bytes();
    let forbidden = base_forbidden_prefixes();
    assert_eq!(forbidden.len(), 36);
    assert_eq!(forbidden.iter().cloned().collect::<BTreeSet<_>>().len(), 36);

    let pairs = generate_pairs(seed, &forbidden);
    assert_eq!(pairs.len(), REQUIRED_PAIR_COUNT);
    assert_eq!(pairs, generate_pairs(seed, &forbidden));
    let cases = pairs
        .iter()
        .flat_map(|pair| [&pair.unswapped, &pair.mate])
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), REQUIRED_CASE_COUNT);
    let selected_prefixes = cases
        .iter()
        .map(|case| case.prefix.clone())
        .collect::<BTreeSet<_>>();
    let forbidden_set = forbidden.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(selected_prefixes.len(), REQUIRED_CASE_COUNT);
    assert!(selected_prefixes.is_disjoint(&forbidden_set));
    assert_eq!(
        cases
            .iter()
            .map(|case| case.case_id)
            .collect::<BTreeSet<_>>()
            .len(),
        REQUIRED_CASE_COUNT
    );
    let generated_fixture = pairs
        .iter()
        .flat_map(|pair| [(pair.counter, &pair.unswapped), (pair.counter, &pair.mate)])
        .collect::<Vec<_>>();
    for ((counter, actual), frozen) in generated_fixture.iter().zip(FROZEN_TARGET_FREE_CASES) {
        assert_eq!(*counter, frozen.pair_counter);
        assert_eq!(display_digest(actual.case_id), frozen.case_id);
        assert_eq!(actual.prefix, frozen.prefix);
    }
    for pair in &pairs {
        assert_eq!(mate(&pair.unswapped.prefix), pair.mate.prefix);
        assert_eq!(mate(&pair.mate.prefix), pair.unswapped.prefix);
        assert_eq!(pair.unswapped.prefix.len(), pair.mate.prefix.len());
        let mut left = pair.unswapped.prefix.clone();
        let mut right = pair.mate.prefix.clone();
        left.sort_unstable();
        right.sort_unstable();
        assert_eq!(left, right);
    }
    for (index, left) in cases.iter().enumerate() {
        assert_eq!(left.case_id, case_id(&left.prefix));
        assert_eq!(left.prefix.len(), 13);
        assert_eq!(left.prefix.last(), Some(&QUERY_TOKEN));
        assert_eq!(
            left.prefix[..left.prefix.len() - 1]
                .iter()
                .filter(|token| **token == QUERY_TOKEN)
                .count(),
            1
        );
        assert!(forbidden.iter().all(|prior| antichain(&left.prefix, prior)));
        for right in cases.iter().skip(index + 1) {
            assert!(antichain(&left.prefix, &right.prefix));
        }
    }

    let structural_five_count = cases
        .iter()
        .filter(|case| structural_binding(&case.prefix) == Some(SUPPORT[0]))
        .count();
    let structural_six_count = cases
        .iter()
        .filter(|case| structural_binding(&case.prefix) == Some(SUPPORT[1]))
        .count();
    assert_eq!((structural_five_count, structural_six_count), (12, 12));
    assert_eq!(PREDICTION_COUNT, 0);
    assert_eq!(SCORING_LABEL_JOIN_COUNT, 0);

    // This synthetic vector pins the commitment encoding without embedding the
    // real nonce or any real V4 label row in the Phase-II source.
    let synthetic_rows = (0..REQUIRED_CASE_COUNT)
        .map(|index| {
            (
                [u8::try_from(index).expect("24 indexes fit u8"); 32],
                if index % 2 == 0 { 5 } else { 6 },
            )
        })
        .collect::<Vec<_>>();
    assert!(salted_label_commitment_bytes([0x11; 32], [0x22; 32], &synthetic_rows[..23]).is_err());
    let mut too_many_rows = synthetic_rows.clone();
    too_many_rows.push(([0x55; 32], 5));
    assert!(salted_label_commitment_bytes([0x11; 32], [0x22; 32], &too_many_rows).is_err());
    let synthetic_preimage = salted_label_commitment_bytes([0x11; 32], [0x22; 32], &synthetic_rows)
        .expect("the synthetic vector has exactly 24 rows");
    let synthetic_commitment = display_digest(*blake3::hash(&synthetic_preimage).as_bytes());
    assert_eq!(
        synthetic_commitment,
        "blake3:0191c49d7eee35d1bc23fd62969ef6524b5ed99129dc221b51023f00c45029f7"
    );

    assert_eq!(
        FROZEN_CONTROL_BINDINGS,
        [
            ("baseline", "h4_compatible", "none"),
            ("alternative", "alternative_tangent", "none"),
            ("plain", "plain_fixed", "none"),
            ("current_only", "current_token_only", "none"),
            ("order_shuffled", "h4_compatible", "order_shuffled"),
            ("value_permuted", "h4_compatible", "value_permuted"),
            (
                "source_gauge_mismatch",
                "h4_compatible",
                "source_gauge_mismatched",
            ),
        ]
    );

    for identity in [
        FROZEN_GENERATOR_POLICY_CID,
        FROZEN_PHASE_I_PREFLIGHT_CID,
        FROZEN_PHASE_I_CORE_CID,
        FROZEN_PHASE_I_ARTIFACT_CID,
        FROZEN_PHASE_I_INITIALIZATION_CID,
        FROZEN_PHASE_I_CONSTRUCTION_KAPPA,
        FROZEN_PHASE_I_FRAME_MANIFEST_CID,
        FROZEN_V2_VALIDATION_INPUT_KAPPA,
        FROZEN_V3_VALIDATION_INPUT_KAPPA,
        FROZEN_FORBIDDEN_ROOT,
        FROZEN_VALIDATION_PREFIX_ROOT,
    ] {
        assert_eq!(display_digest(required_raw_cid(identity)), identity);
    }
    assert!(
        parse_raw_cid("be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e").is_err()
    );
    assert!(parse_raw_cid(
        "blake3:BE3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e"
    )
    .is_err());
    assert!(parse_raw_cid("blake3:be3772").is_err());
    assert!(parse_raw_cid(
        "BLAKE3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e"
    )
    .is_err());
    assert!(parse_raw_cid(
        "sha256:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e"
    )
    .is_err());
    assert!(parse_raw_cid(
        "blake3:ge3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e"
    )
    .is_err());
    assert!(parse_raw_cid(
        "blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e0"
    )
    .is_err());
    assert!(parse_raw_cid(
        "blake3:be3772f6d16ca2ae4e19559e4f44ebc60f389cadff2032b956fe12a31e1e725e\n"
    )
    .is_err());

    let policy_cid = display_digest(
        *blake3::hash(CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY.as_bytes()).as_bytes(),
    );
    let forbidden_root = forbidden_root(&forbidden);
    let prefix_root = validation_prefix_root(&pairs);
    let forbidden_cid = display_digest(forbidden_root);
    let prefix_cid = display_digest(prefix_root);
    let validation_input = validation_input_bytes(
        &pairs,
        forbidden_root,
        prefix_root,
        u64::try_from(structural_five_count).expect("structural count fits u64"),
        u64::try_from(structural_six_count).expect("structural count fits u64"),
    );
    let validation_input_raw = *blake3::hash(&validation_input).as_bytes();
    let validation_input_cid = display_digest(validation_input_raw);
    eprintln!("CGCV_973_PHASE_II_GENERATOR_POLICY_CID={policy_cid}");
    eprintln!("CGCV_973_PHASE_II_FORBIDDEN_ROOT={forbidden_cid}");
    eprintln!("CGCV_973_PHASE_II_VALIDATION_PREFIX_ROOT={prefix_cid}");
    eprintln!("CGCV_973_PHASE_II_VALIDATION_INPUT_CID={validation_input_cid}");
    eprintln!(
        "CGCV_973_PHASE_II_COUNTS pairs={} prefixes={} structural_balance={structural_five_count}/{structural_six_count} prediction_count={PREDICTION_COUNT} scoring_label_join_count={SCORING_LABEL_JOIN_COUNT}",
        pairs.len(),
        cases.len()
    );
    eprintln!(
        "CGCV_973_PHASE_II_SELECTED_COUNTERS={:?}",
        pairs.iter().map(|pair| pair.counter).collect::<Vec<_>>()
    );
    assert_eq!(policy_cid, FROZEN_GENERATOR_POLICY_CID);
    assert_eq!(forbidden_cid, FROZEN_FORBIDDEN_ROOT);
    assert_eq!(prefix_cid, FROZEN_VALIDATION_PREFIX_ROOT);

    assert_eq!(validation_input_cid, FROZEN_VALIDATION_INPUT_CID);
    assert_eq!(
        SALTED_LABEL_COMMITMENT_DOMAIN,
        "uor-r4.cgcv-v4.salted-label-commitment/1"
    );
    let commitment_cid = FROZEN_SALTED_LABEL_COMMITMENT_CID;
    let commitment_raw = required_raw_cid(commitment_cid);
    let prelabel_raw = *blake3::hash(&prelabel_freeze_bytes(
        forbidden_root,
        prefix_root,
        validation_input_raw,
        commitment_raw,
    ))
    .as_bytes();
    let prelabel_cid = display_digest(prelabel_raw);
    eprintln!("CGCV_973_PHASE_II_LABEL_COMMITMENT_CID={commitment_cid}");
    eprintln!("CGCV_973_PHASE_II_PRELABEL_FREEZE_CID={prelabel_cid}");
    assert_eq!(prelabel_cid, FROZEN_PRELABEL_FREEZE_CID);
}
