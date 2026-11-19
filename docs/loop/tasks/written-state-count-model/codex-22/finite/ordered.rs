use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, ops::Range, path::Path};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    ordered_role::{RoleFraming, RoleMode, ROLE_DIMENSIONS, ROLE_RADIUS},
    paired_role::{role_candidates, RoleSpan, ROLE_ROOTS},
    span_selector::{CONT, START},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Vector = [i16; ROLE_DIMENSIONS];

const NAMES: [&str; 2] = ["Rosalind", "Benedict"];
const FACTS: &str = "Facts. Rosalind owns glass jars. Benedict owns wooden brushes.";
const PREREGISTRATION_SHA256: &str =
    "ae8c5362b91d24cdabc6a5e0f4fe131a4a979eb29971bd261d584a846072580f";
const PINS: [[&str; 3]; 2] = [
    [
        "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377",
        "b78c4ddc9dfca0b0c0f7f6fcab84ad6c09bdcb3c9c404a817dda7fa2548b3c33",
        "56610eab5073cc4b1c8edc7bbae9a9345cbe12876d47b9aec81f4862630087d0",
    ],
    [
        "f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e",
        "4d6427dd27fd2fde428f647d25ca3fa8c0446c9720a4f262a3cdee51ffbbaa9d",
        "1c386426e4b5395de9cfa0e4fd4e2d59b2867e55ce1c930bc1eff4ee534f9d44",
    ],
];

#[derive(Serialize)]
struct Candidate {
    person: &'static str,
    span: [usize; 2],
    assertion_head: [usize; 2],
    clause: Range<usize>,
    active_positions_nearest_first: Vec<usize>,
    active_ids_nearest_first: Vec<u32>,
    active_roots_nearest_first: Vec<u32>,
    sorted_root_multiset: Vec<u32>,
    six_keys: [u32; 6],
    identity: Vector,
    positional: Vector,
}

#[derive(Serialize)]
struct Row {
    id: String,
    requested_person: &'static str,
    prompt: String,
    prefix: Vec<u32>,
    candidates: Vec<Candidate>,
}

fn pinned(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    if digest != expected {
        return Err(format!("pin mismatch for {}", path.display()).into());
    }
    Ok(bytes)
}

fn l1(a: Vector, b: Vector) -> u32 {
    a.iter()
        .zip(b)
        .map(|(&x, y)| u32::from(x.abs_diff(y)))
        .sum()
}

fn wordings(family: &str, requested: usize, reversed: bool) -> String {
    let clause = |person: usize| {
        let name = NAMES[person];
        let (cue, other) = if person == requested {
            ("keep", "drop")
        } else {
            ("drop", "keep")
        };
        match family {
            "same-multiset" => format!("please {cue} rather than {other} {name}"),
            "far-cue" => format!("please {cue} the exact item wording for {name}"),
            "near-cue" => format!("please {cue} {name}"),
            _ => unreachable!(),
        }
    };
    let first = usize::from(reversed);
    format!("{FACTS} {}; {}.", clause(first), clause(1 - first))
}

fn compare(family: &str, rows: &[Row]) -> Vec<Value> {
    let mut comparisons = Vec::new();
    for (kind, pairs) in [
        ("polarity", [(0, 1), (2, 3)]),
        ("clause_order", [(0, 2), (1, 3)]),
    ] {
        for (left, right) in pairs {
            let (a, b) = (&rows[left], &rows[right]);
            let is_polarity = kind == "polarity";
            assert_eq!(a.requested_person != b.requested_person, is_polarity);
            for (candidate, other) in a.candidates.iter().zip(&b.candidates) {
                assert_eq!(candidate.person, other.person);
                assert_eq!(candidate.assertion_head, other.assertion_head);
                if is_polarity {
                    assert_eq!(candidate.span, other.span);
                    assert_eq!(candidate.clause, other.clause);
                }
                let same_multiset = candidate.sorted_root_multiset == other.sorted_root_multiset;
                if family == "same-multiset" || !is_polarity {
                    assert!(same_multiset);
                }
                let identity = l1(candidate.identity, other.identity);
                let positional = l1(candidate.positional, other.positional);
                let six_distinct = candidate.six_keys != other.six_keys;
                comparisons.push(json!({
                    "rows": [a.id, b.id], "kind": kind, "person": candidate.person,
                    "same_multiset": same_multiset, "six_keys_distinct": six_distinct,
                    "identity_l1": identity, "positional_l1": positional,
                }));
            }
        }
    }
    comparisons
}

fn run_seed(seed: usize, assets: &Path, state_dir: &Path) -> Result<Value> {
    let [tokenizer_pin, parent_pin, role_pin] = PINS[seed - 1];
    let tokenizer_bytes = pinned(
        &assets.join(format!("tokenizer-seed{seed}.json")),
        tokenizer_pin,
    )?;
    let parent_path = assets.join(format!("seed{seed}-paired.d11"));
    pinned(&parent_path, parent_pin)?;
    let role_bytes = pinned(&state_dir.join(format!("seed{seed}.role")), role_pin)?;
    let parent = IntegerDiscreteModel::load(&parent_path)?;
    let frozen = parent.frozen_role_replay()?;
    assert_eq!(frozen.roles.to_bytes(), role_bytes);
    let sites = parent.ordered_role_sites()?;
    let roots: Vec<_> = sites.packed_roots().collect();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("invalid pinned tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let framing = RoleFraming::from_tokenizer(&tokenizer)?;
    let semicolon = tokenizer.encode(";");
    assert_eq!(semicolon.len(), 1);

    let mut groups = Vec::new();
    for family in ["same-multiset", "far-cue", "near-cue"] {
        let mut rows = Vec::new();
        for reversed in [false, true] {
            for requested in [0, 1] {
                let text = wordings(family, requested, reversed);
                let ids = encoder
                    .encode_assistant_prefix(&[Message {
                        role: "user",
                        content: &text,
                    }])
                    .tokens;
                assert_eq!(
                    tokenizer.decode(&ids),
                    format!("<|bos|>User: {text}\nAssistant:")
                );
                let content = framing.content(&ids)?;
                let terminal = content.end - 1;
                assert!(frozen.delimiters.contains(&ids[terminal]));
                let boundary = (content.start..terminal)
                    .rfind(|&pos| frozen.delimiters.contains(&ids[pos]))
                    .ok_or("no query boundary")?;
                let separator = (boundary + 1..terminal)
                    .find(|&pos| ids[pos] == semicolon[0])
                    .ok_or("no clause boundary")?;
                let clauses = [boundary + 1..separator, separator + 1..terminal];
                let class = |position| frozen.assertion_classes[ids[position] as usize];
                let assertion_head = |span: RoleSpan| {
                    (content.start..boundary).find_map(|start| {
                        if class(start) != START || !frozen.delimiters.contains(&ids[start - 1]) {
                            return None;
                        }
                        let end = (start + 1..ids.len())
                            .find(|&at| class(at) != CONT)
                            .unwrap_or(ids.len());
                        (end <= boundary && ids[start..end] == ids[span.start..span.end])
                            .then_some([start, end])
                    })
                };
                let eligible: Vec<_> = role_candidates(frozen.roles.sites(), &ids)?
                    .filter(|(span, _)| span.start > boundary && span.end <= terminal)
                    .filter_map(|(span, full_keys)| {
                        assertion_head(span).map(|head| (span, full_keys, head))
                    })
                    .collect();
                assert_eq!(eligible.len(), 2);
                let mut candidates = Vec::new();
                for name in NAMES {
                    let (span, full_keys, head) = *eligible
                        .iter()
                        .find(|(span, _, _)| {
                            tokenizer.decode(&ids[span.start..span.end]) == format!(" {name}")
                        })
                        .ok_or("missing supported name")?;
                    let clause = clauses
                        .iter()
                        .find(|range| range.contains(&span.start))
                        .ok_or("missing candidate clause")?
                        .clone();
                    assert_eq!(span.end, clause.end);
                    let positions = [
                        span.start.wrapping_sub(3),
                        span.start.wrapping_sub(2),
                        span.start.wrapping_sub(1),
                        span.end,
                        span.end + 1,
                        span.end + 2,
                    ];
                    let six_keys = std::array::from_fn(|i| {
                        if clause.contains(&positions[i]) {
                            full_keys[i]
                        } else {
                            ((i + 1) * ROLE_ROOTS - 1) as u32
                        }
                    });
                    let prompt = sites.prompt(&ids, clause.clone())?;
                    let active_positions: Vec<_> = prompt
                        .feature_positions(span, clause.clone())?
                        .into_iter()
                        .flatten()
                        .collect();
                    let active_ids: Vec<_> = active_positions.iter().map(|&at| ids[at]).collect();
                    let active_roots: Vec<_> =
                        active_ids.iter().map(|&id| roots[id as usize]).collect();
                    let mut sorted_root_multiset = active_roots.clone();
                    sorted_root_multiset.sort_unstable();
                    let mut complete_clause_multiset: Vec<_> = ids[clause.start..span.start]
                        .iter()
                        .chain(&ids[span.end..clause.end])
                        .map(|&id| roots[id as usize])
                        .collect();
                    complete_clause_multiset.sort_unstable();
                    assert_eq!(sorted_root_multiset, complete_clause_multiset);
                    candidates.push(Candidate {
                        person: name,
                        span: [span.start, span.end],
                        assertion_head: head,
                        clause: clause.clone(),
                        active_positions_nearest_first: active_positions,
                        active_ids_nearest_first: active_ids,
                        active_roots_nearest_first: active_roots,
                        sorted_root_multiset,
                        six_keys,
                        identity: prompt.feature(RoleMode::Identity, span, clause.clone())?,
                        positional: prompt.feature(RoleMode::Positional, span, clause)?,
                    });
                }
                assert!(candidates[0].assertion_head[1] <= candidates[1].assertion_head[0]);
                rows.push(Row {
                    id: format!("{family}-{}", rows.len()),
                    requested_person: NAMES[requested],
                    prompt: text,
                    prefix: ids,
                    candidates,
                });
            }
        }
        groups.push(json!({"family":family, "comparisons":compare(family, &rows), "rows":rows}));
    }
    Ok(json!({
        "seed":seed, "tokenizer_sha256":tokenizer_pin, "paired_parent_sha256":parent_pin,
        "role_state_sha256":role_pin, "groups":groups,
    }))
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [assets, role_dir] = args.as_slice() else {
        return Err(
            "usage: clause-feature-proof TOKENIZER_AND_PARENT_DIR ORIGINAL_ROLE_STATE_DIR".into(),
        );
    };
    let assets = Path::new(assets);
    let role_dir = Path::new(role_dir);
    let evidence = json!({
        "schema":"uor-r4.clause-feature-proof/1",
        "preregistration_sha256":PREREGISTRATION_SHA256,
        "radius":ROLE_RADIUS,
        "dimensions":ROLE_DIMENSIONS,
        "seeds":[run_seed(1, assets, role_dir)?, run_seed(2, assets, role_dir)?]
    });
    println!("{}", serde_json::to_string_pretty(&evidence)?);
    Ok(())
}
