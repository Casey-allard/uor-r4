//! B3 admission tallies and the Q1 diagnostic stop, using recorded metadata only.
//! No slot identities, forward computation or mechanism verdict is inferred here.
use super::invalid;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Classify recorded token identities only. The caller supplies an already
/// document-bounded, C192-truncated window whose last token is the query.
/// Only the nearest earlier query occurrence determines its copy successor.
pub fn is_copy_successor(window: &[crate::Identity], target: u32) -> bool {
    let Some(query) = window.last() else {
        return false;
    };
    window[..window.len() - 1]
        .iter()
        .rposition(|identity| identity.token == query.token)
        .is_some_and(|index| window[index + 1].token == target)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionRow {
    pub row_ordinal: u64,
    pub target_position: u64,
    pub target_token: u32,
    pub is_psucc: bool,
    pub target_admitted: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct B3Count {
    pub rows: u64,
    pub target_admitted: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct B3Summary {
    pub rows: Vec<AdmissionRow>,
    pub full: B3Count,
    pub psucc: B3Count,
    pub complement: B3Count,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Q1Decision {
    pub diagnostic_stop: bool,
    pub psucc_below_500: bool,
    pub psucc_less_than_half_init: bool,
    pub no_combine_or_relation_sensitivity: bool,
    pub init_psucc_target_admitted: u64,
    pub q1_psucc_target_admitted: u64,
    pub combine_sensitive: u64,
    pub relation_sensitive: u64,
}

fn increment(count: &mut B3Count, admitted: bool) -> Result<()> {
    count.rows = count
        .rows
        .checked_add(1)
        .ok_or_else(|| invalid("attention B3 row count overflow"))?;
    count.target_admitted = count
        .target_admitted
        .checked_add(u64::from(admitted))
        .ok_or_else(|| invalid("attention B3 admitted count overflow"))?;
    Ok(())
}

fn tally(rows: &[AdmissionRow]) -> Result<(B3Count, B3Count, B3Count)> {
    let (mut full, mut psucc, mut complement) =
        (B3Count::default(), B3Count::default(), B3Count::default());
    let mut previous = None;
    for row in rows {
        if row.row_ordinal != full.rows
            || previous.is_some_and(|position| row.target_position <= position)
        {
            return Err(invalid(
                "attention B3 rows require contiguous ordinals and strictly ascending target positions",
            ));
        }
        previous = Some(row.target_position);
        increment(&mut full, row.target_admitted)?;
        increment(
            if row.is_psucc {
                &mut psucc
            } else {
                &mut complement
            },
            row.target_admitted,
        )?;
    }
    Ok((full, psucc, complement))
}

fn u64_field(row: &Value, field: &str, ordinal: usize) -> Result<u64> {
    row.get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("attention B3 row {ordinal} requires u64 {field}")))
}

/// Bind each observed row to the caller's ordered (position, token, P_succ)
/// identities. Evaluation rows report counts: admitted_cells - address_rivals
/// is exactly the target-admission bit, checked in 0..=1 at K8. This does not
/// reconstruct Forward slots or an intersection of admitted cell identities.
pub fn summarize(evaluation: &Value, expected: &[(u64, u32, bool)]) -> Result<B3Summary> {
    if expected.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(invalid(
            "attention B3 expected target positions must be strictly ascending and unique",
        ));
    }
    let batches = evaluation
        .get("batches")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("attention B3 evaluation requires batches array"))?;
    let mut rows = Vec::with_capacity(expected.len());
    for batch in batches {
        let observed = batch
            .get("rows")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid("attention B3 batch requires rows array"))?;
        for row in observed {
            let ordinal = rows.len();
            let &(position, token, is_psucc) = expected
                .get(ordinal)
                .ok_or_else(|| invalid("attention B3 evaluation has extra rows"))?;
            let target_position = u64_field(row, "target_position", ordinal)?;
            let target_token =
                u32::try_from(u64_field(row, "target_token", ordinal)?).map_err(|_| {
                    invalid(format!(
                        "attention B3 row {ordinal} target token exceeds u32"
                    ))
                })?;
            if target_position != position || target_token != token {
                return Err(invalid(format!(
                    "attention B3 row {ordinal} differs from expected ordered target identity"
                )));
            }
            let admitted = u64_field(row, "admitted_cells", ordinal)?;
            let rivals = u64_field(row, "address_rivals", ordinal)?;
            if admitted > 8 || rivals > 8 {
                return Err(invalid(format!(
                    "attention B3 row {ordinal} admission/rival counts exceed K8"
                )));
            }
            let target_admitted = match admitted.checked_sub(rivals) {
                Some(0) => false,
                Some(1) => true,
                _ => {
                    return Err(invalid(format!(
                        "attention B3 row {ordinal} admitted_cells - address_rivals must be 0 or 1"
                    )))
                }
            };
            rows.push(AdmissionRow {
                row_ordinal: u64::try_from(ordinal)
                    .map_err(|_| invalid("attention B3 ordinal exceeds u64"))?,
                target_position,
                target_token,
                is_psucc,
                target_admitted,
            });
        }
    }
    if rows.len() != expected.len() {
        return Err(invalid("attention B3 evaluation is missing expected rows"));
    }
    let (full, psucc, complement) = tally(&rows)?;
    Ok(B3Summary {
        rows,
        full,
        psucc,
        complement,
    })
}

fn validate_summary(summary: &B3Summary) -> Result<()> {
    if tally(&summary.rows)? != (summary.full, summary.psucc, summary.complement) {
        return Err(invalid(
            "attention B3 summary counts disagree with its rows",
        ));
    }
    Ok(())
}

fn decide_counts(
    init_psucc: u64,
    q1_psucc: u64,
    combine_sensitive: u64,
    relation_sensitive: u64,
) -> Result<Q1Decision> {
    let doubled = q1_psucc
        .checked_mul(2)
        .ok_or_else(|| invalid("attention Q1 doubled P_succ admission count overflow"))?;
    let psucc_below_500 = q1_psucc < 500;
    let psucc_less_than_half_init = doubled < init_psucc;
    let no_combine_or_relation_sensitivity = combine_sensitive == 0 && relation_sensitive == 0;
    Ok(Q1Decision {
        diagnostic_stop: psucc_below_500
            || psucc_less_than_half_init
            || no_combine_or_relation_sensitivity,
        psucc_below_500,
        psucc_less_than_half_init,
        no_combine_or_relation_sensitivity,
        init_psucc_target_admitted: init_psucc,
        q1_psucc_target_admitted: q1_psucc,
        combine_sensitive,
        relation_sensitive,
    })
}

/// A stop requests diagnosis only; it is neither a mechanism negative nor a
/// training release. A non-stop adds no permission or arbitrary runtime budget.
pub fn q1_decision(
    init: &B3Summary,
    q1: &B3Summary,
    combine_sensitive: u64,
    relation_sensitive: u64,
) -> Result<Q1Decision> {
    validate_summary(init)?;
    validate_summary(q1)?;
    if init.rows.len() != q1.rows.len()
        || init.rows.iter().zip(&q1.rows).any(|(left, right)| {
            left.row_ordinal != right.row_ordinal
                || left.target_position != right.target_position
                || left.target_token != right.target_token
                || left.is_psucc != right.is_psucc
        })
    {
        return Err(invalid(
            "attention Q1 requires identical ordered init/Q1 row identities and P_succ membership",
        ));
    }
    decide_counts(
        init.psucc.target_admitted,
        q1.psucc.target_admitted,
        combine_sensitive,
        relation_sensitive,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(position: u64, token: u32, admitted: u64, rivals: u64) -> Value {
        json!({"target_position":position,"target_token":token,"admitted_cells":admitted,"address_rivals":rivals})
    }

    fn identities(tokens: &[u32]) -> Vec<crate::Identity> {
        tokens
            .iter()
            .enumerate()
            .map(|(index, &token)| crate::Identity {
                token,
                occurrence: index as u64,
                record: 17,
                version: 1,
            })
            .collect()
    }

    #[test]
    fn copy_successor_uses_nearest_occurrence_despite_older_conflict() {
        let window = identities(&[7, 11, 7, 13, 7]);
        assert!(is_copy_successor(&window, 13));
        assert!(!is_copy_successor(&window, 11));
        assert!(!is_copy_successor(&window, 7));
    }

    #[test]
    fn copy_successor_short_windows_distance_one_and_absent_query() {
        assert!(!is_copy_successor(&[], 7));
        assert!(!is_copy_successor(&identities(&[7]), 7));
        assert!(is_copy_successor(&identities(&[7, 7]), 7));
        assert!(is_copy_successor(&identities(&[5, 7, 7]), 7));
        assert!(!is_copy_successor(&identities(&[5, 7, 7]), 5));
        assert!(!is_copy_successor(&identities(&[1, 2, 7]), 7));
        assert!(!is_copy_successor(&identities(&[1, 2, 7]), 2));
    }

    #[test]
    fn counts_bind_ordered_rows_across_batches_and_partition_psucc() -> Result<()> {
        let value =
            json!({"batches":[{"rows":[row(3,7,8,7),row(5,9,8,8)]},{"rows":[row(9,11,1,0)]}]});
        let summary = summarize(&value, &[(3, 7, true), (5, 9, true), (9, 11, false)])?;
        assert_eq!(
            summary.full,
            B3Count {
                rows: 3,
                target_admitted: 2
            }
        );
        assert_eq!(
            summary.psucc,
            B3Count {
                rows: 2,
                target_admitted: 1
            }
        );
        assert_eq!(
            summary.complement,
            B3Count {
                rows: 1,
                target_admitted: 1
            }
        );
        assert_eq!(summary.rows[2].row_ordinal, 2);
        assert!(summary.rows[0].target_admitted);
        assert!(!summary.rows[1].target_admitted);
        let zero = summarize(
            &json!({"batches":[{"rows":[row(3,7,0,0)]}]}),
            &[(3, 7, true)],
        )?;
        assert_eq!(zero.full.target_admitted, 0);
        Ok(())
    }

    #[test]
    fn malformed_counts_and_missing_metadata_are_refused() {
        for (admitted, rivals) in [(9, 8), (8, 9), (0, 1), (8, 6), (u64::MAX, 0)] {
            assert!(summarize(
                &json!({"batches":[{"rows":[row(3,7,admitted,rivals)]}]}),
                &[(3, 7, true)]
            )
            .is_err());
        }
        for field in [
            "admitted_cells",
            "address_rivals",
            "target_position",
            "target_token",
        ] {
            for invalid_value in [json!(-1), json!([]), json!("1"), Value::Null, json!(true)] {
                let mut observed = row(3, 7, 1, 0);
                observed[field] = invalid_value;
                assert!(
                    summarize(&json!({"batches":[{"rows":[observed]}]}), &[(3, 7, true)]).is_err()
                );
            }
        }
        for malformed in [
            json!({}),
            json!({"batches":{}}),
            json!({"batches":[{}]}),
            json!({"batches":[{"rows":null}]}),
            json!({"batches":[{"rows":[{}]}]}),
        ] {
            assert!(summarize(&malformed, &[(3, 7, true)]).is_err());
        }
        let mut too_large_token = row(3, 7, 1, 0);
        too_large_token["target_token"] = json!(u64::from(u32::MAX) + 1);
        assert!(summarize(
            &json!({"batches":[{"rows":[too_large_token]}]}),
            &[(3, 7, true)]
        )
        .is_err());
    }

    #[test]
    fn duplicate_missing_extra_reordered_or_wrong_targets_are_refused() {
        let expected = [(3, 7, true), (5, 9, false)];
        for rows in [
            vec![row(3, 7, 1, 0)],
            vec![row(3, 7, 1, 0), row(5, 9, 1, 0), row(7, 11, 1, 0)],
            vec![row(5, 9, 1, 0), row(3, 7, 1, 0)],
            vec![row(3, 7, 1, 0), row(3, 9, 1, 0)],
            vec![row(3, 7, 1, 0), row(5, 8, 1, 0)],
        ] {
            assert!(summarize(&json!({"batches":[{"rows":rows}]}), &expected).is_err());
        }
        for expected in [[(3, 7, true), (3, 9, false)], [(5, 7, true), (3, 9, false)]] {
            assert!(summarize(&json!({"batches":[]}), &expected).is_err());
        }
    }

    #[test]
    fn q1_floor_half_and_sensitivity_boundaries_are_exact() -> Result<()> {
        let below = decide_counts(998, 499, 1, 0)?;
        assert!(below.diagnostic_stop && below.psucc_below_500);
        assert!(!below.psucc_less_than_half_init);
        assert!(!decide_counts(1000, 500, 1, 0)?.diagnostic_stop);
        assert!(!decide_counts(1000, 500, 0, 1)?.diagnostic_stop);
        let half = decide_counts(1001, 500, 1, 1)?;
        assert!(half.diagnostic_stop && half.psucc_less_than_half_init);
        assert!(!half.psucc_below_500);
        let insensitive = decide_counts(1000, 500, 0, 0)?;
        assert!(insensitive.diagnostic_stop && insensitive.no_combine_or_relation_sensitivity);
        assert!(!decide_counts(500, 500, u64::MAX, u64::MAX)?.diagnostic_stop);
        assert!(decide_counts(0, u64::MAX, 1, 1).is_err());
        Ok(())
    }

    #[test]
    fn q1_revalidates_stored_counts_and_every_ordered_identity() -> Result<()> {
        let init = summarize(
            &json!({"batches":[{"rows":[row(3,7,1,0),row(5,9,1,0)]}]}),
            &[(3, 7, true), (5, 9, false)],
        )?;
        assert!(q1_decision(&init, &init, 1, 0)?.diagnostic_stop);
        for field in 0..4 {
            let mut q1 = init.clone();
            match field {
                0 => q1.rows[0].target_position = 2,
                1 => q1.rows[0].target_token = 8,
                2 => q1.rows[0].is_psucc = false,
                _ => q1.rows[0].row_ordinal = 1,
            }
            if field == 2 {
                (q1.full, q1.psucc, q1.complement) = tally(&q1.rows)?;
            }
            assert!(q1_decision(&init, &q1, 1, 0).is_err());
        }
        let mut corrupt = init.clone();
        corrupt.psucc.target_admitted = u64::MAX;
        assert!(q1_decision(&init, &corrupt, 1, 0).is_err());
        let mut fewer = init.clone();
        fewer.rows.pop();
        (fewer.full, fewer.psucc, fewer.complement) = tally(&fewer.rows)?;
        assert!(q1_decision(&init, &fewer, 1, 0).is_err());
        Ok(())
    }

    #[test]
    fn stored_types_reject_unknown_fields() -> Result<()> {
        let summary = summarize(
            &json!({"batches":[{"rows":[row(3,7,1,0)]}]}),
            &[(3, 7, true)],
        )?;
        let mut summary_value =
            serde_json::to_value(&summary).map_err(|e| invalid(e.to_string()))?;
        summary_value["unexpected"] = json!(true);
        assert!(serde_json::from_value::<B3Summary>(summary_value).is_err());
        let mut row_value =
            serde_json::to_value(&summary.rows[0]).map_err(|e| invalid(e.to_string()))?;
        row_value["unexpected"] = json!(true);
        assert!(serde_json::from_value::<AdmissionRow>(row_value).is_err());
        assert!(serde_json::from_value::<B3Count>(
            json!({"rows":1,"target_admitted":1,"unexpected":true})
        )
        .is_err());
        let mut decision_value = serde_json::to_value(decide_counts(1000, 500, 1, 0)?)
            .map_err(|e| invalid(e.to_string()))?;
        decision_value["unexpected"] = json!(true);
        assert!(serde_json::from_value::<Q1Decision>(decision_value).is_err());
        Ok(())
    }
}
