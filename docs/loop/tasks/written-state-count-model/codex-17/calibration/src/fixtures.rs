use crate::oracle::{
    compare_candidates, fit_synthetic, Candidate, Choice, Control, Descriptor, Example, Shape,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const MISSING: u32 = u32::MAX;
pub const PROMPT: [u32; 14] = [0, 55, 42, 10, 11, 13, 15, 14, 45, 16, 46, 47, 42, 201];

pub const RIGHT_CURSOR: [[u32; 8]; 5] = [
    [1, 5, 6, 14, 13, 13, 1, 0],
    [1, 5, 7, 14, 15, 15, 1, 0],
    [1, 5, 8, 14, 14, 14, 1, 0],
    [1, 5, 9, 14, 45, 45, 1, 0],
    [0, 5, 9, 14, 1, MISSING, 0, 1],
];

pub const WRONG_CURSOR: [[u32; 8]; 6] = [
    [1, 4, 5, 14, 11, 11, 1, 0],
    [1, 4, 6, 14, 13, 13, 1, 0],
    [1, 4, 7, 14, 15, 15, 1, 0],
    [1, 4, 8, 14, 14, 14, 1, 0],
    [1, 4, 9, 14, 45, 45, 1, 0],
    [0, 4, 9, 14, 1, MISSING, 0, 1],
];

pub const NO_COPY_CURSOR: [[u32; 8]; 2] = [
    [0, MISSING, MISSING, 14, 301, MISSING, 0, 0],
    [0, MISSING, MISSING, 14, 1, MISSING, 0, 0],
];

const fn pack([a, b, c, d]: [u8; 4]) -> u16 {
    u16::from_le_bytes([a + 16 * b, c + 16 * d])
}

pub const ROOT_4: u16 = pack([2, 0, 0, 0]);
pub const ROOT_8: u16 = pack([0, 1, 0, 0]);
pub const ROOT_12: u16 = pack([0, 1, 2, 0]);
pub const ROOT_20: u16 = pack([1, 0, 1, 2]);

pub fn candidates() -> [Candidate; 3] {
    [
        ([1, 1, 0, 0, 0, 0, 0, 0, 0, 0], 1, 1),
        ([1, 1, 1, 0, 0, 0, 0, 0, 0, 0], 2, 1),
        ([1, 1, 1, 1, 2, 0, 0, 0, 0, 0], 3, 2),
    ]
    .map(|(classes, connector_slots, word_slots)| Candidate {
        original_start: 3 + u32::from(connector_slots),
        descriptor: Descriptor {
            shape: Shape {
                connector_slots,
                word_slots,
                classes,
            },
            roots: [0; 10],
        },
    })
}

pub fn examples(non_onset: [u16; 3], omit_onsets_for: Option<u32>) -> Vec<Example> {
    let mut examples = Vec::new();
    for (candidate, root) in candidates().into_iter().zip(non_onset) {
        if Some(candidate.original_start) != omit_onsets_for {
            examples.extend([1, 2].map(|wording_family| Example {
                descriptor: candidate.descriptor,
                onset: true,
                wording_family,
            }));
        }
        let mut contrast = candidate.descriptor;
        contrast.roots[0] = root;
        examples.push(Example {
            descriptor: contrast,
            onset: false,
            wording_family: 3,
        });
    }
    examples
}

/// Emit just the independent finite oracle. All rows and supervision are
/// authored here and have no input-data dependency.
pub fn witness() -> Result<Value, crate::oracle::Error> {
    let planted = candidates();
    let cases = [
        (
            "known_right",
            [ROOT_4, ROOT_8, ROOT_8],
            None,
            [4, 8, 8],
            Choice::Source(5),
            &RIGHT_CURSOR[..],
        ),
        (
            "planted_wrong",
            [ROOT_8, ROOT_8, ROOT_8],
            None,
            [8, 8, 8],
            Choice::Source(4),
            &WRONG_CURSOR[..],
        ),
        (
            "exact_tie",
            [ROOT_8, ROOT_12, ROOT_20],
            None,
            [8, 12, 20],
            Choice::Tie,
            &NO_COPY_CURSOR[..],
        ),
        (
            "unseen_shape",
            [ROOT_4, ROOT_8, ROOT_8],
            Some(5),
            [4, 8, 8],
            Choice::Unsupported,
            &NO_COPY_CURSOR[..],
        ),
    ];
    let mut witnesses = Vec::new();
    for (name, roots, omit, authored_other_distances, authored_choice, cursor) in cases {
        let examples = examples(roots, omit);
        let mut arms = Vec::new();
        for control in [Control::Ordered, Control::WithinSegmentClassMultiset] {
            let bank = fit_synthetic(&examples, control)?;
            let comparison = compare_candidates(&bank, &planted)?;
            arms.push(json!({
                "control": control, "shapes_and_frozen_records": bank,
                "comparison": comparison
            }));
        }
        let independent = json!({"candidates":planted,"authored_examples":examples,"derived":arms});
        let receipt = Sha256::digest(serde_json::to_vec(&independent).expect("fixed fixture JSON"));
        witnesses.push(json!({
            "case":name, "synthetic_snapshot_sha256":format!("{receipt:x}"),
            "frame":independent,
            "authored_expected_non_onset_distances":authored_other_distances,
            "authored_expected_choice":authored_choice, "authored_expected_cursor":cursor,
            "authored_expected_ids": cursor.iter().map(|words| words[4]).collect::<Vec<_>>()
        }));
    }
    Ok(json!({
        "schema":"uor-r4.value-rank-independent-synthetic-oracle/1",
        "prompt_original_ids":PROMPT,
        "fact_end_inclusive":2,"requested_start":12,"assertion_fence":9,
        "legacy_mask":7,"word_start_mask":7,
        "extended_only":[8],"expected_target_ids":[13,15,14,45,1],
        "root_word_examples_le": [ROOT_4.to_le_bytes(),ROOT_8.to_le_bytes(),
                                  ROOT_12.to_le_bytes(),ROOT_20.to_le_bytes()],
        "cases":witnesses
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::Error;
    use uor_r4_integer::discrete_d11::{ExactCopyState, ROLE_COPY_STOPS};

    #[test]
    fn planted_cross_products_and_cursor_match_independent_literal_words() {
        let rows = witness().unwrap();
        assert_eq!(
            rows["root_word_examples_le"],
            json!([[2, 0], [16, 0], [16, 2], [1, 33]])
        );
        for case in rows["cases"].as_array().unwrap() {
            for arm in case["frame"]["derived"].as_array().unwrap() {
                let comparison = &arm["comparison"];
                let authored = &case["authored_expected_choice"];
                assert_eq!(&comparison["choice"], authored);
                let got: Vec<_> = comparison["observations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["non_onset_distance"].as_u64().unwrap())
                    .collect();
                assert_eq!(json!(got), case["authored_expected_non_onset_distances"]);
                let mut prefix = PROMPT.to_vec();
                let mut writer =
                    ExactCopyState::new(authored["source"].as_u64().map(|x| x as u32), 14);
                let ordinary = if authored["source"].is_null() {
                    vec![301, 1]
                } else {
                    vec![901; 6]
                };
                let mut actual = Vec::new();
                for word in ordinary {
                    let step = writer
                        .write(&prefix, word, ROLE_COPY_STOPS)
                        .unwrap()
                        .finish_with_eos(true, 1);
                    writer = step.state;
                    actual.push(step.words());
                    if step.emitted == 1 {
                        break;
                    }
                    prefix.push(step.emitted);
                }
                assert_eq!(json!(actual), case["authored_expected_cursor"]);
            }
        }
    }

    #[test]
    fn leave_family_out_radius_keeps_duplicates_but_omits_family() {
        let candidate = candidates()[0];
        let mut variation = candidate.descriptor;
        variation.roots[0] = ROOT_8;
        let mut nearby = candidate.descriptor;
        nearby.roots[0] = ROOT_4;
        let mut ex = [
            Example {
                descriptor: candidate.descriptor,
                onset: true,
                wording_family: 1,
            },
            Example {
                descriptor: nearby,
                onset: true,
                wording_family: 1,
            },
            Example {
                descriptor: variation,
                onset: true,
                wording_family: 2,
            },
            Example {
                descriptor: candidate.descriptor,
                onset: true,
                wording_family: 1,
            },
        ]
        .to_vec();
        let mut other_side = candidate.descriptor;
        other_side.roots[0] = ROOT_20;
        ex.push(Example {
            descriptor: other_side,
            onset: false,
            wording_family: 3,
        });
        let bank = fit_synthetic(&ex, Control::Ordered).unwrap();
        let as_wire = serde_json::to_value(&bank).unwrap();
        assert_eq!(as_wire["records"].as_array().unwrap().len(), ex.len());
        // Nibble 0 differs by 4 and nibble 1 by 8 between these two
        // particular families. The maximum is 12 even with duplicate zeros.
        assert_eq!(as_wire["radii"][0]["leave_family_out"], 12);

        let single = ex
            .iter()
            .copied()
            .filter(|x| x.wording_family != 2)
            .collect::<Vec<_>>();
        let bank = fit_synthetic(&single, Control::Ordered).unwrap();
        let off = compare_candidates(&bank, &[candidate]).unwrap();
        assert_eq!(off.choice, Choice::Unsupported);
        assert_eq!(off.observations[0].onset_distance, Some(0));
        assert_eq!(off.observations[0].radius, None);
    }

    #[test]
    fn supported_negative_margin_still_abstains() {
        let candidate = candidates()[0];
        let old = candidate.descriptor;
        let mut four = old;
        four.roots[0] = ROOT_4;
        let mut eight = old;
        eight.roots[0] = ROOT_8;
        let bank = fit_synthetic(
            &[
                Example {
                    descriptor: four,
                    onset: true,
                    wording_family: 1,
                },
                Example {
                    descriptor: eight,
                    onset: true,
                    wording_family: 2,
                },
                Example {
                    descriptor: old,
                    onset: false,
                    wording_family: 3,
                },
            ],
            Control::Ordered,
        )
        .unwrap();
        let tested = compare_candidates(&bank, &[candidate]).unwrap();
        assert_eq!(tested.observations[0].onset_distance, Some(4));
        assert_eq!(tested.observations[0].non_onset_distance, Some(0));
        assert_eq!(tested.observations[0].radius, Some(12));
        assert_eq!(tested.observations[0].margin_numerator, Some(-4));
        assert_eq!(tested.choice, Choice::Nonpositive);
    }

    #[test]
    fn strict_radius_admission_and_control_collision_are_separate_failures() {
        let ex = examples([ROOT_4, ROOT_8, ROOT_8], None);
        let bank = fit_synthetic(&ex, Control::Ordered).unwrap();
        let mut forged = candidates();
        forged[1].descriptor.roots[0] = ROOT_4;
        let scored = compare_candidates(&bank, &forged).unwrap();
        assert_eq!(scored.choice, Choice::Unsupported);
        assert_eq!(scored.observations[1].onset_distance, Some(4));
        assert_eq!(scored.observations[1].radius, Some(0));
        assert_eq!(scored.observations[1].margin_numerator, None);
        forged.swap(0, 1);
        assert_eq!(
            compare_candidates(&bank, &forged),
            Err(Error::InvalidCandidateOrder)
        );

        let mut a = candidates()[1].descriptor;
        a.roots[0] = ROOT_4;
        let mut b = a;
        b.roots.swap(0, 1);
        let opposed = [
            Example {
                descriptor: a,
                onset: true,
                wording_family: 1,
            },
            Example {
                descriptor: b,
                onset: false,
                wording_family: 2,
            },
        ];
        assert!(fit_synthetic(&opposed, Control::Ordered).is_ok());
        assert_eq!(
            fit_synthetic(&opposed, Control::WithinSegmentClassMultiset),
            Err(Error::OpposedDescriptor)
        );
    }
}
