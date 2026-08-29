//! Generator-only chronology gate for `ConnectionGaugeCovarianceV4` Phase II.
//!
//! This public first checkpoint makes the Phase-I generator literal
//! executable without freezing or exposing any selected counter, prefix,
//! case ID, validation root, target row, nonce, or prediction.

use std::collections::BTreeSet;

use uor_r4_core::direct_causal_geometric_attention::CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY;

const PHASE_I_PROTECTED_MERGE_SHA: &str = "b054197acb92e3dd23d88d81bd859379ea8fac67";
const FROZEN_GENERATOR_POLICY_CID: &str =
    "blake3:73b4233b0b91ba85ffb6cd8c3d86132a954e4fbda5c7ec57510cc30bd9fb5dca";
const QUERY_TOKEN: u32 = 1;
const SUPPORT: [u32; 2] = [5, 6];
const REQUIRED_PAIR_COUNT: usize = 12;
const REQUIRED_CASE_COUNT: usize = REQUIRED_PAIR_COUNT * 2;
const PREDICTION_COUNT: u64 = 0;
const SCORING_LABEL_JOIN_COUNT: u64 = 0;

const PAIR_ORDER_DOMAIN: &str = "uor-r4.cgcv-v4.pair-order/1";
const UNIT_ORDER_DOMAIN: &str = "uor-r4.cgcv-v4.unit-order/1";
const CASE_ID_DOMAIN: &str = "uor-r4.cgcv-v4.case-id/1";

type Pair = (u16, Vec<u32>, Vec<u32>);

fn tag(domain: &str) -> Vec<u8> {
    let mut bytes = domain.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

fn lp64(bytes: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + bytes.len());
    encoded.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    encoded.extend_from_slice(bytes);
    encoded
}

fn tokens(values: &[u32]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + values.len() * 4);
    encoded.extend_from_slice(&(values.len() as u64).to_le_bytes());
    for value in values {
        encoded.extend_from_slice(&value.to_le_bytes());
    }
    encoded
}

fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

fn pair_order_key(seed: &[u8], counter: u16) -> [u8; 32] {
    digest(&[&tag(PAIR_ORDER_DOMAIN), &lp64(seed), &counter.to_le_bytes()])
}

fn unit_order_key(seed: &[u8], counter: u16, index: u16, unit: &[u32]) -> [u8; 32] {
    digest(&[
        &tag(UNIT_ORDER_DOMAIN),
        &lp64(seed),
        &counter.to_le_bytes(),
        &index.to_le_bytes(),
        &tokens(unit),
    ])
}

fn units() -> [Vec<u32>; 11] {
    [
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

fn candidate_prefix(seed: &[u8], counter: u16) -> Vec<u32> {
    let units = units();
    let mut ordered = units
        .iter()
        .enumerate()
        .map(|(index, unit)| {
            let index = u16::try_from(index).expect("eleven indexes fit u16");
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

fn antichain(left: &[u32], right: &[u32]) -> bool {
    left != right
        && !(left.len() < right.len() && right.starts_with(left))
        && !(right.len() < left.len() && left.starts_with(right))
}

fn case_id(prefix: &[u32]) -> [u8; 32] {
    digest(&[&tag(CASE_ID_DOMAIN), &lp64(&tokens(prefix))])
}

fn legacy_prefixes() -> Vec<Vec<u32>> {
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
        vec![3, 2, 6, 4, 1, 5, 7, 8, 1],
        vec![8, 2, 5, 3, 4, 1, 6, 7, 1],
        vec![10, 1, 5, 3, 4, 2, 6, 11, 12, 1],
        vec![11, 2, 5, 7, 1, 6, 3, 4, 1],
        vec![4, 2, 6, 8, 3, 1, 5, 10, 1],
        vec![12, 1, 6, 7, 3, 2, 5, 8, 4, 1],
        vec![2, 6, 10, 11, 1, 5, 3, 7, 8, 1],
        vec![3, 1, 6, 10, 2, 5, 11, 4, 7, 1],
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

fn eligible(prefix: &[u32], paired: &[u32], forbidden: &[Vec<u32>]) -> bool {
    prefix != paired
        && prefix.last() == Some(&QUERY_TOKEN)
        && paired.last() == Some(&QUERY_TOKEN)
        && prefix[..prefix.len() - 1]
            .iter()
            .filter(|token| **token == QUERY_TOKEN)
            .count()
            == 1
        && paired[..paired.len() - 1]
            .iter()
            .filter(|token| **token == QUERY_TOKEN)
            .count()
            == 1
        && antichain(prefix, paired)
        && [prefix, paired]
            .into_iter()
            .all(|candidate| forbidden.iter().all(|prior| antichain(candidate, prior)))
}

fn generate(seed: &[u8], base_forbidden: &[Vec<u32>]) -> (Vec<Pair>, usize) {
    let mut order = (u16::MIN..=u16::MAX)
        .map(|counter| (pair_order_key(seed, counter), counter))
        .collect::<Vec<_>>();
    order.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    assert_eq!(order.len(), u16::MAX as usize + 1);
    assert_eq!(
        order
            .iter()
            .map(|(_, counter)| *counter)
            .collect::<BTreeSet<_>>()
            .len(),
        u16::MAX as usize + 1
    );

    let mut forbidden = base_forbidden.to_vec();
    let mut selected = Vec::with_capacity(REQUIRED_PAIR_COUNT);
    let mut visited = 0;
    for (_, counter) in order {
        visited += 1;
        let prefix = candidate_prefix(seed, counter);
        let paired = mate(&prefix);
        if !eligible(&prefix, &paired, &forbidden) {
            continue;
        }
        forbidden.push(prefix.clone());
        forbidden.push(paired.clone());
        selected.push((counter, prefix, paired));
        if selected.len() == REQUIRED_PAIR_COUNT {
            break;
        }
    }
    (selected, visited)
}

fn structural_binding(prefix: &[u32]) -> Option<u32> {
    let earlier_query = prefix[..prefix.len().checked_sub(1)?]
        .iter()
        .position(|token| *token == QUERY_TOKEN)?;
    prefix.get(earlier_query + 1).copied()
}

#[test]
fn generator_only_checkpoint_is_deterministic_disjoint_and_unscored() {
    assert_eq!(PHASE_I_PROTECTED_MERGE_SHA.len(), 40);
    assert!(PHASE_I_PROTECTED_MERGE_SHA
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    let policy_cid = format!(
        "blake3:{}",
        blake3::hash(CONNECTION_GAUGE_COVARIANCE_V4_GENERATOR_POLICY.as_bytes()).to_hex()
    );
    assert_eq!(policy_cid, FROZEN_GENERATOR_POLICY_CID);

    let forbidden = legacy_prefixes();
    assert_eq!(forbidden.len(), 36);
    assert_eq!(forbidden.iter().cloned().collect::<BTreeSet<_>>().len(), 36);
    let (left, visited) = generate(PHASE_I_PROTECTED_MERGE_SHA.as_bytes(), &forbidden);
    let (right, replay_visited) = generate(PHASE_I_PROTECTED_MERGE_SHA.as_bytes(), &forbidden);
    assert_eq!(left, right);
    assert_eq!(visited, replay_visited);
    assert_eq!(left.len(), REQUIRED_PAIR_COUNT);

    let cases = left
        .iter()
        .flat_map(|(_, prefix, paired)| [prefix, paired])
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), REQUIRED_CASE_COUNT);
    assert_eq!(
        cases.iter().cloned().collect::<BTreeSet<_>>().len(),
        REQUIRED_CASE_COUNT
    );
    assert_eq!(
        cases
            .iter()
            .map(|prefix| case_id(prefix))
            .collect::<BTreeSet<_>>()
            .len(),
        REQUIRED_CASE_COUNT
    );

    for (_, prefix, paired) in &left {
        assert_eq!(mate(prefix), *paired);
        assert_eq!(mate(paired), *prefix);
        let mut first = prefix.to_vec();
        let mut second = paired.to_vec();
        first.sort_unstable();
        second.sort_unstable();
        assert_eq!(first, second);
    }
    for (index, prefix) in cases.iter().enumerate() {
        assert_eq!(prefix.len(), 13);
        assert_eq!(prefix.last(), Some(&QUERY_TOKEN));
        assert_eq!(
            prefix[..prefix.len() - 1]
                .iter()
                .filter(|token| **token == QUERY_TOKEN)
                .count(),
            1
        );
        assert!(forbidden.iter().all(|prior| antichain(prefix, prior)));
        for other in cases.iter().skip(index + 1) {
            assert!(antichain(prefix, other));
        }
    }

    let count_five = cases
        .iter()
        .filter(|prefix| structural_binding(prefix) == Some(SUPPORT[0]))
        .count();
    let count_six = cases
        .iter()
        .filter(|prefix| structural_binding(prefix) == Some(SUPPORT[1]))
        .count();
    assert_eq!((count_five, count_six), (12, 12));
    assert_eq!(PREDICTION_COUNT, 0);
    assert_eq!(SCORING_LABEL_JOIN_COUNT, 0);
    eprintln!(
        "CGCV_973_PHASE_II_GENERATOR_ONLY policy={policy_cid} pairs={} cases={} balance={count_five}/{count_six} predictions={PREDICTION_COUNT} scoring_label_joins={SCORING_LABEL_JOIN_COUNT}",
        left.len(),
        cases.len()
    );
}
