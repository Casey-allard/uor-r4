use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    ordered_role::RoleFraming,
    paired_role::{role_candidates, PairedRoleState, ROLE_ROOTS},
    span_selector::{CONT, START},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    let [_, assets, packed] = args.as_slice() else {
        return Err("usage: author-clause-fixtures MODEL_ASSET_DIR FROZEN_ROLE_DIR".into());
    };
    let assets = Path::new(assets);
    let packed = Path::new(packed);
    let mut fixtures = Vec::new();
    for seed in [1, 2] {
        let tokenizer_path = assets.join(format!("tokenizer-seed{seed}.json"));
        let parent_path = assets.join(format!("seed{seed}-paired.d11"));
        let state_path = packed.join(format!("seed{seed}.role"));
        let tokenizer_bytes = fs::read(&tokenizer_path)?;
        let parent = IntegerDiscreteModel::load(&parent_path)?;
        let native = parent.frozen_role_replay()?;
        let state_bytes = fs::read(&state_path)?;
        let state = PairedRoleState::from_bytes(&state_bytes)?;
        assert_eq!(native.roles.to_bytes(), state_bytes);
        assert_eq!(
            state.bindings()[1],
            Sha256::digest(&tokenizer_bytes).as_slice()
        );
        let tokenizer =
            ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("bad tokenizer")?;
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
        let encoder = protocol.bind(&tokenizer)?;
        let framing = RoleFraming::from_tokenizer(&tokenizer)?;
        let semicolon = tokenizer.encode(";");
        assert_eq!(semicolon.len(), 1);
        let mut rows = Vec::new();
        for (family, tail) in [("far-cue", " the exact item wording for"), ("near-cue", "")] {
            for reverse in [false, true] {
                for target in [0, 1] {
                    let names = ["Rosalind", "Benedict"];
                    let make_clause = |person| {
                        let cue = if person == target { "keep" } else { "drop" };
                        let name = names[person];
                        format!("please {cue}{tail} {name}")
                    };
                    let order = if reverse { [1, 0] } else { [0, 1] };
                    let text = format!(
                        "Facts. Rosalind owns glass jars. Benedict owns wooden brushes. {}; {}.",
                        make_clause(order[0]),
                        make_clause(order[1])
                    );
                    let ids = encoder
                        .encode_assistant_prefix(&[Message {
                            role: "user",
                            content: &text,
                        }])
                        .tokens;
                    let content = framing.content(&ids)?;
                    assert_eq!(
                        tokenizer.decode(&ids),
                        format!("<|bos|>User: {text}\nAssistant:")
                    );
                    let terminal = content.end - 1;
                    assert!(native.delimiters.contains(&ids[terminal]));
                    let boundary = (content.start..terminal)
                        .rfind(|&pos| native.delimiters.contains(&ids[pos]))
                        .ok_or("query boundary")?;
                    let separator = (boundary + 1..terminal)
                        .find(|&pos| ids[pos] == semicolon[0])
                        .ok_or("clause boundary")?;
                    let bounds = [boundary + 1..separator, separator + 1..terminal];
                    let mut candidates = Vec::new();
                    let legal_candidates: Vec<_> = role_candidates(state.sites(), &ids)?.collect();
                    let supporting_head = |span: uor_r4_integer::paired_role::RoleSpan| {
                        (content.start.max(1)..boundary).find_map(|start| {
                            if native.assertion_classes[ids[start] as usize] != START
                                || !native.delimiters.contains(&ids[start - 1])
                            {
                                return None;
                            }
                            let end = start
                                + 1
                                + ids[start + 1..]
                                    .iter()
                                    .take_while(|&&id| {
                                        native.assertion_classes[id as usize] == CONT
                                    })
                                    .count();
                            (end <= boundary && ids[start..end] == ids[span.start..span.end])
                                .then_some([start, end])
                        })
                    };
                    assert_eq!(
                        legal_candidates
                            .iter()
                            .filter(|(span, _)| {
                                span.start > boundary
                                    && span.end <= content.end
                                    && supporting_head(*span).is_some()
                            })
                            .count(),
                        2
                    );
                    for (person, name) in names.iter().enumerate() {
                        let (span, full_keys) = legal_candidates
                            .iter()
                            .find(|(span, _)| {
                                span.start > boundary
                                    && tokenizer.decode(&ids[span.start..span.end])
                                        == format!(" {name}")
                            })
                            .ok_or("missing query candidate")?;
                        let clause = bounds
                            .iter()
                            .find(|r| r.contains(&span.start))
                            .ok_or("candidate outside clause")?;
                        assert_eq!(span.end, clause.end);
                        let supporting_head =
                            supporting_head(*span).ok_or("missing exact assertion head")?;
                        let positions = [
                            span.start.wrapping_sub(3),
                            span.start.wrapping_sub(2),
                            span.start.wrapping_sub(1),
                            span.end,
                            span.end + 1,
                            span.end + 2,
                        ];
                        let keys = positions.map(|p| usize::from(!clause.contains(&p)));
                        let clipped_keys = std::array::from_fn::<_, 6, _>(|i| {
                            if keys[i] == 0 {
                                full_keys[i]
                            } else {
                                (i * ROLE_ROOTS + ROLE_ROOTS - 1) as u32
                            }
                        });
                        let live_ids = positions.map(|p| clause.contains(&p).then(|| ids[p]));
                        let clause_left_roots: Vec<_> = ids[clause.start..span.start]
                            .iter()
                            .map(|&id| state.sites()[id as usize][0])
                            .collect();
                        candidates.push(json!({
                            "person": name, "requested": person == target,
                            "span": [span.start, span.end], "clause": [clause.start, clause.end],
                            "supporting_head": supporting_head, "mention_ids": ids[span.start..span.end],
                            "clause_left_ids": ids[clause.start..span.start], "clause_left_roots": clause_left_roots,
                            "slot_ids": live_ids, "keys": clipped_keys, "full_keys": full_keys,
                        }));
                    }
                    rows.push(json!({
                        "family": family, "reversed": reverse, "requested_person": names[target],
                        "expected_requested_span": candidates[target]["span"],
                        "text": text, "prefix": ids, "content": [content.start, content.end],
                        "query": [boundary + 1, terminal], "clause_separator": separator,
                        "candidates": candidates
                    }));
                }
            }
        }
        for (start, collision) in [(0, true), (2, true), (4, false), (6, false)] {
            let a = &rows[start]["candidates"];
            let b = &rows[start + 1]["candidates"];
            for person in [0, 1] {
                assert_eq!(a[person]["clause"], b[person]["clause"]);
                assert_eq!(a[person]["span"], b[person]["span"]);
                assert_eq!(a[person]["supporting_head"], b[person]["supporting_head"]);
                assert_ne!(
                    a[person]["clause_left_roots"],
                    b[person]["clause_left_roots"]
                );
                assert_eq!(a[person]["keys"] == b[person]["keys"], collision);
                assert_eq!(a[person]["slot_ids"] == b[person]["slot_ids"], collision);
            }
        }
        for row in [0, 1, 4, 5] {
            // Reversing clause order with a fixed requested person must preserve both local feature packets.
            for person in [0, 1] {
                assert_eq!(
                    rows[row]["candidates"][person]["keys"],
                    rows[row + 2]["candidates"][person]["keys"]
                );
            }
        }
        fixtures.push(json!({
            "seed": seed, "tokenizer": tokenizer_path, "tokenizer_sha256": sha(&tokenizer_bytes),
            "paired_parent": parent_path, "paired_parent_sha256": sha(&fs::read(&parent_path)?),
            "role_state": state_path, "role_state_sha256": sha(&state_bytes),
            "sentence_delimiters": native.delimiters, "semicolon_id": semicolon[0], "rows": rows
        }));
    }
    let output: Value =
        json!({"schema":"uor-r4.clipped-role-feature-fixtures/1","fixtures":fixtures});
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
