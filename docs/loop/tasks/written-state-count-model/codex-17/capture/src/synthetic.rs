//! Planted raw frozen-selection and writer traces, independent of any panel.

use super::*;
use uor_r4_integer::{continuation_selector::Continuation, value_selector::Decisions};

const TAPE: [u32; 14] = [0, 55, 42, 10, 11, 13, 15, 14, 45, 16, 46, 47, 42, 201];
const QUERY: [usize; 2] = [12, 13];

fn tables() -> ([u32; ROLE_VOCAB], Vec<u32>) {
    let mut classes = [0; ROLE_VOCAB];
    for id in [10, 11, 13, 15, 41, 42, 45, 46, 47] {
        classes[id] = START;
    }
    classes[14] = CONT;
    let roots = (0..ROLE_VOCAB)
        .map(|id| ((id as u32) & 0xf) | 0x4320)
        .collect();
    (classes, roots)
}

fn planted(fence: u32) -> AssertionValue {
    AssertionValue {
        fence: Some(fence),
        value: Decisions {
            eligible_mask: 0b111,
            word_start_mask: 0b111,
            current: Some(6),
            word_start: Some(5),
            ..Decisions::default()
        },
        continuation: Continuation {
            source: Some(5),
            ..Continuation::default()
        },
        source: Some(5),
    }
}

fn raw(assertion: AssertionValue, fs: [usize; 2]) -> Value {
    json!({"paired":{"choice":{"source":5,"query":12,"fact":2},
        "supporting_facts":1,"complete_values":1,"values_disagree":false},
        "assertions":[{"fact_span":fs,"assertion":assertion,
            "completed_source_range":[5,assertion.fence]}]})
}

fn group(a: AssertionValue, fact: [usize; 2]) -> FixtureGroup {
    FixtureGroup {
        sites: vec![fact[0]],
        fact_spans: vec![fact],
        ranks: vec![a],
        selected: Some(5),
        selected_index: 0,
    }
}

fn absent() -> (Value, FixtureGroup) {
    (
        json!({"paired":{"choice":{"source":null,"query":12,"fact":null},
        "supporting_facts":0,"complete_values":0,"values_disagree":false},"assertions":[]}),
        FixtureGroup {
            sites: vec![],
            fact_spans: vec![],
            ranks: vec![],
            selected: None,
            selected_index: 0,
        },
    )
}

pub fn fixture() -> Result<Value> {
    let (classes, roots) = tables();
    let bound = planted(9);
    let normal = bind_assertions(
        &TAPE,
        Some(QUERY),
        &raw(bound, [2, 3]),
        &group(bound, [2, 3]),
        &classes,
        &roots,
    )?;
    let mut reversed = TAPE;
    reversed[12] = 41;
    let (wire, empty) = absent();
    let no_fact = bind_assertions(&reversed, Some(QUERY), &wire, &empty, &classes, &roots)?;
    let roots_le = codebook_bytes(&roots)?;
    Ok(json!({"schema":"uor-r4.legacy-assertion-candidates/1",
    "mode":"synthetic","codebook_sha256":sha(&roots_le),"copy_stops":ROLE_COPY_STOPS,
    "rows":[
      {"ordinal":0,"prefix_sha256":prefix_sha(&TAPE),"requested_span":QUERY,
       "excluded_span":null,"original_tape_len":TAPE.len(),"assertions":normal},
      {"ordinal":1,"prefix_sha256":prefix_sha(&reversed),"requested_span":QUERY,
       "excluded_span":null,"original_tape_len":reversed.len(),"assertions":no_fact}
    ]}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retains_all_legacy_masks_same_fence_and_packed_words() {
        let frame = fixture().unwrap();
        let a = &frame["rows"][0]["assertions"][0];
        assert_eq!(a["legacy_joint_mask"], 7);
        assert_eq!(a["fence"], 9);
        let starts = a["legacy_candidates"].as_array().unwrap();
        assert_eq!(
            starts
                .iter()
                .map(|x| x["original_start"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [4, 5, 6]
        );
        assert_eq!(starts[2]["slot_ids"], json!([10, 11, 13, 15, 14]));
        assert_eq!(starts[2]["slot_classes"], json!([1, 1, 1, 1, 2]));
        assert_eq!(
            starts[2]["packed_roots_u16"],
            json!([0x432a, 0x432b, 0x432d, 0x432f, 0x432e])
        );
        assert_eq!(
            starts[2]["packed_roots_le"],
            json!([0x2a, 0x43, 0x2b, 0x43, 0x2d, 0x43, 0x2f, 0x43, 0x2e, 0x43])
        );
        assert_eq!(a["outside_legacy_inventory"][0]["original_start"], 8);
        assert_ne!(
            frame["rows"][0]["prefix_sha256"],
            frame["rows"][1]["prefix_sha256"]
        );
        assert_eq!(frame["rows"][1]["assertions"], json!([]));
    }

    #[test]
    fn wrong_fact_stale_fence_and_reversed_request_refuse_spliced_trace() {
        let (classes, roots) = tables();
        let good = planted(9);
        assert!(bind_assertions(
            &TAPE,
            Some(QUERY),
            &raw(good, [2, 4]),
            &group(good, [2, 3]),
            &classes,
            &roots
        )
        .is_err());
        assert!(bind_assertions(
            &TAPE,
            Some(QUERY),
            &raw(good, [3, 4]),
            &group(good, [3, 4]),
            &classes,
            &roots
        )
        .is_err());
        let later = planted(10);
        assert!(bind_assertions(
            &TAPE,
            Some(QUERY),
            &raw(later, [2, 3]),
            &group(later, [2, 3]),
            &classes,
            &roots
        )
        .is_err());
        let mut reversed = TAPE;
        reversed[12] = 41;
        assert!(bind_assertions(
            &reversed,
            Some(QUERY),
            &raw(good, [2, 3]),
            &group(good, [2, 3]),
            &classes,
            &roots
        )
        .is_err());
        let (wire, empty) = absent();
        let packet =
            bind_assertions(&reversed, Some(QUERY), &wire, &empty, &classes, &roots).unwrap();
        assert!(packet.is_empty());
    }

    #[test]
    fn cap_rejected_masked_address_is_retained_as_inactive() {
        let (mut classes, roots) = tables();
        classes[12] = CONT;
        let ids = [
            0, 55, 42, 10, 11, 12, 13, 14, 15, 45, 46, 47, 16, 46, 47, 42, 201,
        ];
        let mut a = planted(12);
        a.value.word_start_mask = 0b101;
        a.continuation.source = Some(6);
        a.source = Some(6);
        let witness = Witness {
            fact_span: [2, 3],
            assertion: a,
            completed_source_range: Some([6, 12]),
        };
        let out = package_assertion(&ids, [15, 16], witness, &classes, &roots).unwrap();
        let c = out["legacy_candidates"].as_array().unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[0]["original_start"], 4);
        assert_eq!(c[0]["writer_reachable"], false);
        assert_eq!(c[1]["writer_reachable"], true);
    }

    #[test]
    fn frozen_no_fact_writer_trace_never_picks_other_writable_value() {
        let mut reversed = TAPE;
        reversed[12] = 41;
        let raw = json!({
        "source":null,"generated":[301,1],
        "steps":[
          {"before":[0,NIL,NIL,14],"after":[0,NIL,NIL,14],"native_id":301,
           "native_top8":[301,302,303,304,305,306,307,308],
           "emitted":301,"source_id":null,"copied":0,"release":0},
          {"before":[0,NIL,NIL,14],"after":[0,NIL,NIL,14],"native_id":1,
           "native_top8":[1,302,303,304,305,306,307,308],
           "emitted":1,"source_id":null,"copied":0,"release":0}
        ]});
        let steps = cursor_words(&reversed, &raw, None).unwrap();
        assert_eq!(
            steps,
            [
                [0, NIL, NIL, 14, 301, NIL, 0, 0],
                [0, NIL, NIL, 14, 1, NIL, 0, 0]
            ]
        );
        assert!(cursor_words(&reversed, &raw, Some(5)).is_err());
    }
}
