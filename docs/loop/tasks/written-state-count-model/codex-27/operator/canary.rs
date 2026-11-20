//! Planted D27 instrument checks on synthetic integer token tapes.
use std::{env, fs::OpenOptions, io, path::PathBuf};

use uor_r4_integer::{
    paired_role::{RoleSpan, ROLE_VOCAB},
    relation_retrieval::{write_device_preamble, Arm, Outcome, Plan},
    span_selector::{CONT, START},
};

#[derive(Clone, Copy)]
struct Expected<'a>(Outcome, &'a [usize]);

struct Case<'a> {
    label: &'a str,
    facts: &'a [u32],
    words: &'a [u32],
    exact: Expected<'a>,
    geometry: Expected<'a>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).map(PathBuf::from).collect();
    let [input, oracle] = args.as_slice() else {
        return Err("usage: d27-relation-canary INPUT ORACLE".into());
    };
    let mut input = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(input)?;
    let mut oracle = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(oracle)?;
    write_device_preamble(&mut input)?;

    let mut classes = vec![START; ROLE_VOCAB];
    classes[30] = CONT;
    classes[31] = CONT;
    classes[4] = 0;
    let mut roots = vec![0x1111; ROLE_VOCAB];
    for (id, root) in [
        (10, 2),
        (12, 2),
        (13, 15),
        (14, 0),
        (15, 1),
        (30, 0),
        (31, 0),
    ] {
        roots[id] = root;
    }

    use Outcome::{Collision, Empty, Selected, Tie};
    let start = [1];
    let two = [1, 5];
    let second = [5];
    let cases = [
        Case {
            label: "literal beats geometric neighbor",
            facts: &[3, 10, 19, 4, 3, 13, 18, 4],
            words: &[10, 14],
            exact: Expected(Selected, &start),
            geometry: Expected(Selected, &start),
        },
        Case {
            label: "one nonzero root step",
            facts: &[3, 10, 19, 4, 3, 11, 18, 4],
            words: &[14],
            exact: Expected(Empty, &[]),
            geometry: Expected(Selected, &start),
        },
        Case {
            label: "outside squared radius",
            facts: &[3, 14, 19, 4, 3, 11, 18, 4],
            words: &[15],
            exact: Expected(Empty, &[]),
            geometry: Expected(Empty, &[]),
        },
        Case {
            label: "distance normalized by complete token-piece count",
            facts: &[3, 10, 19, 4, 3, 13, 30, 18, 4],
            words: &[14, 14, 31],
            exact: Expected(Empty, &[]),
            geometry: Expected(Selected, &second),
        },
        Case {
            label: "distinct literal words share a root",
            facts: &[3, 10, 19, 4, 3, 11, 18, 4],
            words: &[12],
            exact: Expected(Empty, &[]),
            geometry: Expected(Collision, &[]),
        },
        Case {
            label: "equal geometric evidence",
            facts: &[3, 10, 19, 4, 3, 13, 18, 4],
            words: &[14],
            exact: Expected(Empty, &[]),
            geometry: Expected(Tie, &[]),
        },
        Case {
            label: "distinct relations literally present",
            facts: &[3, 10, 19, 4, 3, 13, 18, 4],
            words: &[10, 13],
            exact: Expected(Tie, &[]),
            geometry: Expected(Tie, &[]),
        },
        Case {
            label: "duplicate relation, independent value consensus",
            facts: &[3, 10, 19, 4, 3, 10, 18, 4],
            words: &[10],
            exact: Expected(Selected, &two),
            geometry: Expected(Selected, &two),
        },
        Case {
            label: "empty cue after person masks",
            facts: &[3, 10, 19, 4],
            words: &[6, 3],
            exact: Expected(Empty, &[]),
            geometry: Expected(Empty, &[]),
        },
        Case {
            label: "no earlier matches",
            facts: &[6, 10, 19, 4],
            words: &[10],
            exact: Expected(Empty, &[]),
            geometry: Expected(Empty, &[]),
        },
        Case {
            label: "misleading value and competing person",
            facts: &[3, 11, 10, 4, 6, 10, 18, 4],
            words: &[10, 6],
            exact: Expected(Empty, &[]),
            geometry: Expected(Empty, &[]),
        },
        Case {
            label: "complete split-word literal",
            facts: &[3, 10, 30, 19, 4],
            words: &[10, 30],
            exact: Expected(Selected, &start),
            geometry: Expected(Selected, &start),
        },
        Case {
            label: "split-word continuation collision",
            facts: &[3, 10, 30, 19, 4],
            words: &[10, 31],
            exact: Expected(Empty, &[]),
            geometry: Expected(Collision, &[]),
        },
        Case {
            label: "split-word length mismatch",
            facts: &[3, 10, 30, 19, 4],
            words: &[10],
            exact: Expected(Empty, &[]),
            geometry: Expected(Empty, &[]),
        },
        Case {
            label: "structural break prevents split word",
            facts: &[3, 10, 30, 19, 4],
            words: &[10, 4, 30],
            exact: Expected(Empty, &[]),
            geometry: Expected(Empty, &[]),
        },
    ];

    for case in &cases {
        let mut ids = vec![0];
        ids.extend_from_slice(case.facts);
        let query_start = ids.len();
        ids.push(3);
        ids.extend_from_slice(case.words);
        let excluded = ids[query_start..]
            .iter()
            .position(|&id| id == 6)
            .map(|i| RoleSpan {
                start: query_start + i,
                end: query_start + i + 1,
            });
        let plan = Plan::new(
            &ids,
            &roots,
            &classes,
            1..ids.len(),
            query_start..ids.len(),
            RoleSpan {
                start: query_start,
                end: query_start + 1,
            },
            excluded,
            &[0, 4],
            &[4],
        )?;
        for (arm, Expected(outcome, starts)) in
            [(Arm::Exact, case.exact), (Arm::Geometry, case.geometry)]
        {
            let decision = plan.select(arm);
            let obtained: Vec<_> = decision
                .selected_facts
                .iter()
                .map(|span| span.start)
                .collect();
            if decision.outcome != outcome || obtained != starts {
                return Err(io::Error::other(format!(
                    "{} / {arm:?}: expected {outcome:?} {starts:?}, got {decision:?}",
                    case.label
                ))
                .into());
            }
        }
        plan.write_device_row(&mut input)?;
        plan.write_device_oracle(&mut oracle)?;
    }
    eprintln!("{} planted host checks passed", cases.len());
    Ok(())
}
