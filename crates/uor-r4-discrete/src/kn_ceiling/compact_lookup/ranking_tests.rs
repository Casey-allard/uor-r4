use super::*;
use crate::kn_ceiling::{bucket_count, expert_count, PickClassRule, WEIGHT_ONE};
use crate::ngram_count_oracle::{oracle_adjusted, oracle_count};

#[test]
fn every_original_id_matches_scalar_score_after_a_causal_prefix() -> Result<()> {
    let train = [0, 4095, 31, 17, 31, 6, 1, 0, 17, 31, 1, 0, 31, 17, 31, 7, 1];
    let order = 3;
    let raw = (0..=order)
        .map(|n| oracle_count(&train, n))
        .collect::<Result<Vec<_>>>()?;
    let adjusted = (0..=order)
        .map(|n| oracle_adjusted(&train, n, order))
        .collect::<Result<Vec<_>>>()?;
    let wire = encode_compact_lookup(order, [3; 32], &raw, &adjusted)?;
    let lookup = CompactCountLookup::from_bytes(&wire, [3; 32], Sha256::digest(&wire).into())?;
    let width = order as usize + 1;
    let experts = expert_count(order) + 1;
    let stride = bucket_count(order);
    for (rule, expected_classes) in [
        (PickClassRule::Three, [0, 1, 2, 2, 2]),
        (PickClassRule::Five, [0, 1, 2, 3, 4]),
    ] {
        let mut weights = vec![0; usize::from(rule.classes()) * stride * experts];
        for (bucket, row) in weights.chunks_exact_mut(experts).enumerate() {
            row[1] = WEIGHT_ONE / 4;
            let class = bucket / stride;
            let specialist = if class == 0 { experts - 2 } else { experts - 1 };
            row[specialist] = WEIGHT_ONE / (4 + class as u64);
            row[0] = WEIGHT_ONE - row[1] - row[specialist];
        }
        let bytes = FrozenPickMixture::with_rule(order, [19; 32], weights, rule)?.to_bytes();
        let fit = FrozenPickMixture::from_bytes(&bytes, [19; 32])?;
        for ((history, picked), class) in [
            (&[0][..], None),
            (&[0, 31, 17, 31][..], Some((4095, 0))),
            (&[0, 17, 31, 17][..], Some((1, 1))),
            (&[7, 1, 0, 4095][..], Some((17, 2))),
            (&[2, 3, 4, 5][..], Some((17, 5))),
        ]
        .into_iter()
        .zip(expected_classes)
        {
            let mut ranked = Vec::new();
            lookup.visit_next_masses(history, &fit, picked, |id, bucket, m| {
                ranked.push((id, bucket, m))
            })?;
            let mut tokens = history.to_vec();
            let at = tokens.len();
            tokens.push(0);
            assert_eq!(ranked.len(), 4095);
            let (mut counts, mut totals) = (vec![0; width], vec![0; width]);
            for (i, (id, bucket, mass)) in ranked.into_iter().enumerate() {
                assert_eq!(usize::from(id), i + 1);
                assert_eq!(usize::from(bucket) / stride, usize::from(class));
                tokens[at] = id;
                let kn = lookup.lookup(&tokens, at, &mut counts, &mut totals)?;
                let target = PickTarget {
                    class,
                    hit: picked.is_some_and(|(token, _)| token == id),
                };
                assert_eq!(
                    (bucket, mass),
                    fit.predict_mass(kn, &counts, &totals, target)?
                );
            }
        }
    }
    Ok(())
}
