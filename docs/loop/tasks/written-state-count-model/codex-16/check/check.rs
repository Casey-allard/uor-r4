mod observed;
// Independent grading witnesses over original unmarked text and immutable token tapes.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, fs, io::Write, path::Path};
use uor_r4_tokenizer::ByteBpeTokenizer;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn read(p: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn save(p: &str, v: &Value) -> Result<()> {
    let mut file = fs::File::create_new(Path::new(p))?;
    file.write_all(&serde_json::to_vec(v)?)?;
    file.sync_all()?;
    Ok(())
}
fn ids(v: &Value) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(v.clone())?)
}
fn valid(actual: &[u32], expected: &[u32], answer: &str, t: &ByteBpeTokenizer) -> bool {
    let Some((&1, body)) = actual.split_last() else {
        return false;
    };
    !body.contains(&1)
        && actual == expected
        && t.decode_bytes(body) == format!(" {answer}").as_bytes()
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    if let Some((cmd, rest)) = args.split_first() {
        if cmd == "observed" {
            return observed::score(rest);
        }
    }
    let [tokenizer, prompts, grades, readings, out] = args.as_slice() else {
        return Err("TOKENIZER BLIND GRADES INDEPENDENT_READER NEW_REPORT".into());
    };
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tokenizer)?)
        .ok_or("bad tokenizer")?;
    let protocol = uor_r4_tokenizer::dialogue::DialogueProtocol::literal_roles_v2(&t)?;
    let encoder = protocol.bind(&t)?;
    let p = read(prompts)?;
    let g = read(grades)?;
    let a = read(readings)?;
    let rows = p.as_array().ok_or("not a panel")?;
    if rows.len() != 32
        || g.as_array().map(Vec::len) != Some(rows.len())
        || a.as_array().map(Vec::len) != Some(rows.len())
    {
        return Err("unbound questionnaires".into());
    }
    let mut records = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let grade = &g[i];
        let reader = &a[i];
        if reader["index"] != i || row.as_object().ok_or("bad prompt")?.len() != 2 {
            return Err("row order or blind fields".into());
        }
        let tape = ids(&row["prefix"])?;
        let raw = row["text"].as_str().ok_or("no raw user request")?;
        let frame = encoder.encode_assistant_prefix(&[uor_r4_tokenizer::dialogue::Message {
            role: "user",
            content: raw,
        }]);
        let mut separate = vec![0];
        for segment in [
            "User:".to_owned(),
            format!(" {raw}"),
            "\n".to_owned(),
            "Assistant:".to_owned(),
        ] {
            separate.extend(t.encode(&segment));
        }
        if frame.tokens != tape || separate != tape {
            return Err("original unmarked framing differs".into());
        }
        let pos = |span: &Value| -> Result<(usize, usize)> {
            let [a, b]: [usize; 2] = serde_json::from_value(span.clone())?;
            if a >= b || b > tape.len() {
                return Err("invalid original span".into());
            }
            Ok((a, b))
        };
        let (qs, qe) = pos(&grade["requested"])?;
        let requested = reader["requested_name"]
            .as_str()
            .ok_or("missing independently read name")?;
        if t.decode(&tape[qs..qe]).trim() != requested || grade["requested_name"] != requested {
            return Err("query identity differs".into());
        }
        let mut supports = 0;
        for fact in grade["facts"].as_array().ok_or("invalid facts")? {
            let (a, b) = pos(&fact["name_span"])?;
            let exact = t.decode(&tape[a..b]);
            if Some(exact.trim()) != fact["name"].as_str() {
                return Err("fact original name mismatch".into());
            }
            supports += usize::from(exact.trim() == requested);
        }
        if reader["support_count"] != supports {
            return Err("independent support count differs".into());
        }
        let mut h = Sha256::new();
        for id in &tape {
            h.update(id.to_le_bytes());
        }
        if grade["prefix_sha256"] != format!("{:x}", h.finalize()) {
            return Err("wrong prefix".into());
        }
        let status = match grade["stratum"].as_str().ok_or("bad stratum")? {
            "supported" => "supported",
            "unmentioned" => "absent",
            "conflict" => "conflict",
            _ => return Err("unknown stratum".into()),
        };
        if reader["status"] != status {
            return Err(format!("semantic mismatch at {i}: {}", reader["status"]).into());
        }
        if status != "supported" {
            if !grade["expected"].is_null() || !reader["answer"].is_null() {
                return Err("unsupported row carries expected literal".into());
            }
            records.push(json!({"index":i,"semantics":status}));
            continue;
        }
        let answer = reader["answer"]
            .as_str()
            .ok_or("missing independently recovered answer")?;
        let mut expected = t.encode(&format!(" {answer}"));
        expected.push(1);
        let factual = grade["facts"].as_array().ok_or("missing facts")?;
        let gold: usize = serde_json::from_value(grade["gold_assertion_index"].clone())?;
        let f = factual.get(gold).ok_or("bad fact")?;
        let [start, end]: [usize; 2] = serde_json::from_value(f["value_span"].clone())?;
        let rival: usize = serde_json::from_value(grade["rival_start"].clone())?;
        if f["name"] != reader["requested_name"]
            || f["value_span"] != grade["gold_value_span"]
            || grade["gold_start"] != start
            || f["rival_span"][0] != rival
            || f["delimiter"] != json!([end, 16])
            || end >= qs
        {
            return Err("independent fact or intervention address mismatch".into());
        }
        if start >= end
            || end >= tape.len()
            || rival >= end
            || rival == start
            || tape[end] != 16
            || end - start > 7
        {
            return Err("invalid original fence".into());
        }
        if tape[start..end]
            .iter()
            .chain(&tape[rival..end])
            .any(|x| [0, 1, 16, 201].contains(x))
        {
            return Err("interior cursor stop".into());
        }
        let mut literal = tape[start..end].to_vec();
        literal.push(1);
        if grade["answer_text"] != answer
            || ids(&grade["expected"])? != expected
            || !valid(&literal, &expected, answer, &t)
        {
            return Err(format!("two independent grading channels differ at {i}").into());
        }
        let second = factual
            .iter()
            .find(|q| q["name"] != f["name"])
            .ok_or("missing other entity")?["value"]
            .as_str()
            .ok_or("bad other literal")?;
        let mut other = t.encode(&format!(" {second}"));
        other.push(1);
        let mut rival_ids = tape[rival..end].to_vec();
        rival_ids.push(1);
        let word = answer
            .split_once(' ')
            .ok_or("no full-word deletion mutant")?
            .1;
        let mut dropped = t.encode(&format!(" {word}"));
        dropped.push(1);
        let mut wrong_stop = expected.clone();
        wrong_stop.insert(1, 1);
        let mut extra = t.encode(&format!(" {answer} again"));
        extra.push(1);
        let mut failures = Vec::new();
        for (tag, trial) in [
            ("rival_source", rival_ids),
            ("wrong_entity", other),
            ("drop_first_piece", expected[1..].to_vec()),
            ("drop_first_word", dropped),
            ("omit_eos", expected[..expected.len() - 1].to_vec()),
            ("interior_eos", wrong_stop),
            ("extra_word", extra),
        ] {
            if valid(&trial, &expected, answer, &t) {
                return Err(format!("scorer accepted {tag} at {i}").into());
            }
            failures.push(tag);
        }
        records.push(json!({"index":i,"semantics":status,"accepted_planted":true,"rejected":failures,"gold_copy_length":end-start,"rival_copy_length":end-rival,"rival_inside_seven":end-rival<=7}));
    }
    save(
        out,
        &json!({"schema":"uor-r4.fresh-validity-independent-grader/1","rows":records}),
    )
}
