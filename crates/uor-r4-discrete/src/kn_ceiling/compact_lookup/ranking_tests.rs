use super::*;
use crate::kn_ceiling::{bucket_count, expert_count, WEIGHT_ONE};
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
    let mut weights = vec![0; 3 * stride * experts];
    for (bucket, row) in weights.chunks_exact_mut(experts).enumerate() {
        row[0] = WEIGHT_ONE / 2;
        row[1] = WEIGHT_ONE / 4;
        if bucket < stride {
            row[experts - 2] = WEIGHT_ONE / 4;
        } else {
            row[experts - 1] = WEIGHT_ONE / 4;
        }
    }
    let fit = FrozenPickMixture::new(order, [19; 32], weights)?;
    for (history, picked) in [
        (&[0][..], None),
        (&[0, 31, 17, 31][..], Some((4095, 0))),
        (&[7, 1, 0, 4095][..], Some((17, 5))),
        (&[2, 3, 4, 5][..], None),
    ] {
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
            tokens[at] = id;
            let kn = lookup.lookup(&tokens, at, &mut counts, &mut totals)?;
            let target = PickTarget {
                class: match picked {
                    None => 0,
                    Some((_, 0)) => 1,
                    _ => 2,
                },
                hit: picked.is_some_and(|(token, _)| token == id),
            };
            assert_eq!(
                (bucket, mass),
                fit.predict_mass(kn, &counts, &totals, target)?
            );
        }
    }
    Ok(())
}
