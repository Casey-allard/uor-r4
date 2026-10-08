//! Saved-input integer action margins. No state advance or model displacement.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, io, path::Path};
use uor_r4_integer::{
    geometric_context::ContextDecisionFamily,
    geometric_context_q4::{
        basis_score_q24, unpack_coefficients, ContextQ4Config, NativeContextQ4,
    },
    h4_tables::{H4Code, HistoricalH4Tables},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn bad(s: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidData, s).into()
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn number(v: &Value) -> Result<usize> {
    Ok(v.as_u64()
        .ok_or_else(|| bad("missing integer witness"))?
        .try_into()?)
}
fn ids(v: &Value) -> Result<Vec<usize>> {
    v.as_array()
        .ok_or_else(|| bad("missing saved integer array"))?
        .iter()
        .map(number)
        .collect()
}
fn state(v: &Value) -> Result<Vec<usize>> {
    let s = ids(v)?;
    if s.len() != 8 || s.iter().any(|&x| x >= 120) {
        return Err(bad("saved state shape/domain"));
    }
    Ok(s)
}
fn code(x: usize) -> Result<H4Code> {
    Ok(H4Code::try_from(u8::try_from(x)?)?)
}
fn winner(s: &[i64]) -> Result<usize> {
    let mut w = 0;
    let mut best = i64::MIN;
    if s.len() != 120 {
        return Err(bad("transition score width"));
    }
    for (i, &x) in s.iter().enumerate() {
        if x > best {
            best = x;
            w = i;
        }
    }
    Ok(w)
}
fn neighbor(flat: usize) -> usize {
    flat / 4 * 4 + (flat % 4 + 1) % 4
}
struct Input {
    scope: String,
    token: usize,
    old: Vec<usize>,
}
fn inputs(run: &Path, input: usize, position: usize) -> Result<(Value, Vec<Input>)> {
    let frame = read(&run.join(format!(
        "baseline-row-{input:04}-position-{position:02}.json"
    )))?;
    if number(&frame["input_index"])? != input || number(&frame["position"])? != position {
        return Err(bad("frame identity"));
    }
    let p = &frame["native"]["bank_trace"]["prefix"];
    let mut rows = Vec::new();
    for s in p["sources"]
        .as_array()
        .ok_or_else(|| bad("saved sources absent"))?
    {
        let segment = number(&s["source_segment_index"])?;
        let tokens = ids(&s["token_ids"])?;
        let before = s["states_before"]
            .as_array()
            .ok_or_else(|| bad("saved source-before states absent"))?;
        if tokens.len() != before.len() {
            return Err(bad("source-before input cardinality"));
        }
        for (offset, (&token, v)) in tokens.iter().zip(before).enumerate() {
            let old = state(v)?;
            if offset == 0 && old != vec![1; 8] {
                return Err(bad("source identity start differs"));
            }
            // Prefix preparation deliberately never steps the final source
            // token: no candidate consumes that following state.
            if offset + 1 == tokens.len() {
                continue;
            }
            rows.push(Input {
                scope: format!("Source segment{segment} token{offset}"),
                token,
                old,
            });
        }
    }
    let mut response = "UNAVAILABLE preceding response state at position3; endpoint state only";
    if position == 4 {
        let prev = read(&run.join(format!("baseline-row-{input:04}-position-03.json")))?;
        let prefix = ids(&frame["actual_prefix_ids"])?;
        let prevprefix = ids(&prev["actual_prefix_ids"])?;
        if prefix.len() != 4
            || prevprefix != prefix[..3]
            || ids(&p["response"]["token_ids"])? != prefix
            || ids(&prev["native"]["bank_trace"]["prefix"]["response"]["token_ids"])? != prevprefix
        {
            return Err(bad("actual response prefix ancestry differs"));
        }
        rows.push(Input {
            scope: "Response last token at position4; old state from saved position3".into(),
            token: prefix[3],
            old: state(&prev["native"]["bank_trace"]["prefix"]["response"]["states"])?,
        });
        response =
            "PRESENT last response transition only; earlier response input states unavailable";
    }
    Ok((
        json!({"input_index":input,"position":position,"source_inputs":rows.iter().filter(|x|x.scope.starts_with("Source")).count(),"response_inputs_scope":response}),
        rows,
    ))
}
fn u_predecessor(prev: &Value, next: &Value, input: usize) -> Result<(Value, Input)> {
    if number(&prev["input_index"])? != input
        || number(&next["input_index"])? != input
        || number(&prev["position"])? != 3
        || number(&next["position"])? != 4
    {
        return Err(bad("U predecessor frame identity"));
    }
    let before = ids(&prev["actual_prefix_ids"])?;
    let after = ids(&next["actual_prefix_ids"])?;
    let query_path =
        |v: &Value| ids(&v["native"]["bank_trace"]["cue_bank"]["carrier"]["query"]["token_ids"]);
    let query = query_path(prev)?;
    let a = &prev["native"]["continuation"];
    let b = &next["native"]["continuation"];
    if !prev["id"].is_string()
        || prev["id"] != next["id"]
        || before.len() != 3
        || after.len() != 4
        || after[..3] != before
        || query_path(next)? != query
        || number(&a["query_tokens"])? != query.len()
        || number(&b["query_tokens"])? != query.len()
        || number(&a["actual_prefix_tokens"])? != 3
        || number(&b["actual_prefix_tokens"])? != 4
        || number(&prev["native"]["pool"]["summary"]["chosen_token_id"])? != after[3]
        || prev["native"]["bank_trace"]["prefix"]["metadata"]
            != next["native"]["bank_trace"]["prefix"]["metadata"]
    {
        return Err(bad(
            "U predecessor query/count/prefix/Source authority differs",
        ));
    }
    let old = state(&a["state_codes"])?;
    let next_state = state(&b["state_codes"])?;
    Ok((
        json!({"input_index":input,"position":4,"id":prev["id"],"old_state_codes":old,"next_state_codes":next_state,
        "query_token_ids":query,"query_tokens":query.len(),"actual_prefix_tokens_before":3,"actual_prefix_tokens_after":4,
        "actual_prefix_ids_before":before,"actual_prefix_ids_after":after,"context_authority":prev["native"]["bank_trace"]["prefix"]["metadata"]}),
        Input {
            scope: "U last actual-prefix transition at position4; old U state from saved position3"
                .into(),
            token: after[3],
            old,
        },
    ))
}
fn u_inputs(run: &Path, input: usize) -> Result<(Value, Vec<Input>)> {
    let prev = read(&run.join(format!("baseline-row-{input:04}-position-03.json")))?;
    let next = read(&run.join(format!("baseline-row-{input:04}-position-04.json")))?;
    let (receipt, visit) = u_predecessor(&prev, &next, input)?;
    Ok((receipt, vec![visit]))
}
pub fn analyze(run: &Path, parent: &Path, candidates: &Value) -> Result<Value> {
    analyze_mode(run, parent, candidates, false)
}
pub fn analyze_u(run: &Path, parent: &Path, candidates: &Value) -> Result<Value> {
    analyze_mode(run, parent, candidates, true)
}
fn analyze_mode(run: &Path, parent: &Path, candidates: &Value, u_only: bool) -> Result<Value> {
    let checkpoint = parent.join("checkpoint-0001");
    let packed = fs::read(checkpoint.join("native/consumer/context-q4.bin"))?;
    let hash = hex::encode(Sha256::digest(&packed));
    let first = read(&run.join("baseline-row-0245-position-04.json"))?;
    let meta = &first["native"]["bank_trace"]["prefix"]["metadata"];
    if meta["context_packed_sha256"] != hash
        || meta["parent_artifact"]["metadata_sha256"]
            != "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b"
    {
        return Err(bad("native Context Source authority differs"));
    }
    let config: ContextQ4Config = serde_json::from_value(meta["context"].clone())?;
    if config
        != (ContextQ4Config {
            vocab_size: 4096,
            heads: 2,
            lanes_per_head: 4,
        })
    {
        return Err(bad("fixed Context layout differs"));
    }
    let q = unpack_coefficients(config.coefficient_count()?, &packed)?;
    let tables = NativeContextQ4::new(config, &packed)?;
    let mut offsets = std::collections::BTreeMap::new();
    let mut at = 0;
    for (name, shape) in config.coefficient_shapes()? {
        offsets.insert(format!("consumer.context.{name}"), at);
        at += shape.iter().product::<usize>();
    }
    let candidates = candidates
        .as_array()
        .ok_or_else(|| bad("ranked candidates absent"))?;
    let mut cohort = Vec::new();
    let mut rows = Vec::new();
    for input in [245, 0, 1, 4, 5, 8, 9, 12, 13] {
        for &position in if u_only { &[4][..] } else { &[3, 4][..] } {
            let (receipt, visits) = if u_only {
                u_inputs(run, input)?
            } else {
                inputs(run, input, position)?
            };
            if u_only && receipt["context_authority"] != *meta {
                return Err(bad("U Context authority differs from admitted parent"));
            }
            cohort.push(receipt.clone());
            for c in candidates {
                let name = c["name"]
                    .as_str()
                    .ok_or_else(|| bad("candidate family absent"))?;
                if ![
                    "consumer.context.self_transition",
                    "consumer.context.neighbor_transition",
                ]
                .contains(&name)
                {
                    return Err(bad("margin candidate not transition basis"));
                }
                let index = number(&c["index"])?;
                if index >= 3840 {
                    return Err(bad("transition coordinate out of bounds"));
                }
                let flat = index / (120 * 4);
                let action = index / 4 % 120;
                let component = index % 4;
                let offset = *offsets
                    .get(name)
                    .ok_or_else(|| bad("packed family offset missing"))?;
                let start = offset + index / 4 * 4;
                let oldq: [i8; 4] = q[start..start + 4].try_into()?;
                if c["before"].as_i64() != Some(i64::from(oldq[component])) {
                    return Err(bad("ranked before code differs"));
                }
                let after: i8 = c["after"]
                    .as_i64()
                    .ok_or_else(|| bad("candidate after code missing"))?
                    .try_into()?;
                if !(-7..=7).contains(&after) || (after - oldq[component]).abs() != 1 {
                    return Err(bad("nonadjacent or illegal hypothetical code"));
                }
                let mut newq = oldq;
                newq[component] = after;
                for v in &visits {
                    let factor_state = if name.ends_with("self_transition") {
                        v.old[flat]
                    } else {
                        v.old[neighbor(flat)]
                    };
                    let (w, scores) = tables.native().decision_scores(
                        v.token,
                        flat,
                        ContextDecisionFamily::Transition,
                        code(v.old[flat])?,
                        code(v.old[neighbor(flat)])?,
                    )?;
                    if usize::from(w) != winner(&scores)? {
                        return Err(bad("native strict tie selection mismatch"));
                    }
                    let effect = i64::from(basis_score_q24(newq, factor_state.try_into()?)?)
                        - i64::from(basis_score_q24(oldq, factor_state.try_into()?)?);
                    let mut shifted = scores.clone();
                    shifted[action] += effect;
                    let newwinner = winner(&shifted)?;
                    let runner = scores
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != usize::from(w))
                        .map(|(_, x)| *x)
                        .max()
                        .ok_or_else(|| bad("missing runner"))?;
                    let mut row = json!({"name":name,"index":index,"input_index":input,"position":position,"scope":v.scope,"token_id":v.token,"head":flat/4,"lane":flat%4,"class":action,"component":component,"own":v.old[flat],"neighbor":v.old[neighbor(flat)],"factor_state":factor_state,"winner":w,"winner_margin_q24":scores[usize::from(w)]-runner,"selected_class_gap_q24":scores[usize::from(w)]-scores[action],"selected_class_effect_q24":effect,"hypothetical_winner":newwinner,"frozen_input_action_changed":newwinner!=usize::from(w)});
                    if u_only {
                        for key in [
                            "id",
                            "old_state_codes",
                            "next_state_codes",
                            "query_token_ids",
                            "query_tokens",
                            "actual_prefix_tokens_before",
                            "actual_prefix_tokens_after",
                            "actual_prefix_ids_before",
                            "actual_prefix_ids_after",
                        ] {
                            row[key] = receipt[key].clone();
                        }
                        let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
                            "../../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
                        ))?;
                        let next = number(&receipt["next_state_codes"][flat])?;
                        let composed = usize::from(
                            geometry
                                .compose(code(v.old[flat])?, code(usize::from(w))?)
                                .index(),
                        );
                        if composed != next {
                            return Err(bad("U saved next-state baseline action differs"));
                        }
                        row["baseline_next_code"] = json!(next);
                        row["baseline_composed_next_code"] = json!(composed);
                        row["baseline_action_consistent"] = json!(true);
                    }
                    rows.push(row);
                }
            }
        }
    }
    let changed = rows
        .iter()
        .filter(|x| x["frozen_input_action_changed"] == true)
        .count();
    let mut result = json!({"scope":"Exact regenerated native table scores at saved Source-before inputs and saved position3-to4 response input; no encoder advance, candidate artifact, model replay or new finite displacement","packed_sha256":hash,"compiled_table_bytes":tables.stats().expanded_table_bytes,"hypothetical_candidates":candidates.len(),"cohort":cohort,"local_input_rows":rows.len(),"local_action_changes":changed,"rows":rows,"limitations":["Hypothetical one-coefficient local score changes hold every visited input fixed; changed states would propagate differently in a real recurrence","These margins do not establish objective descent, autoregressive success, original-eight retention or complete512 improvement","No preceding response states before position3 retained; no earlier response transition margins fabricated"]});
    if u_only {
        result["scope"]=json!("Exact native transition scores at nine authenticated U position3-to4 predecessors; no encoder advance, candidate artifact or full model replay");
        result["limitations"]=json!(["Only last actual-prefix U transitions are retained; earlier query and prefix U inputs UNAVAILABLE","One-factor hypothetical margins hold old states fixed; no recurrent, fullpool, objective or reply qualification"]);
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn u_predecessor_rejects_query_counts_and_prefix_forgery() -> Result<()> {
        let prev = json!({"input_index":245,"position":3,"id":"fixture","actual_prefix_ids":[617,2097,315],"native":{"bank_trace":{"prefix":{"metadata":{"binding":"fixed"}},"cue_bank":{"carrier":{"query":{"token_ids":[7,8]}}}},"continuation":{"query_tokens":2,"actual_prefix_tokens":3,"state_codes":[1,1,1,1,1,1,1,1]},"pool":{"summary":{"chosen_token_id":1057}}}});
        let mut next = prev.clone();
        next["position"] = json!(4);
        next["actual_prefix_ids"] = json!([617, 2097, 315, 1057]);
        next["native"]["continuation"]["actual_prefix_tokens"] = json!(4);
        u_predecessor(&prev, &next, 245)?;
        let valid = next.clone();
        next["native"]["continuation"]["query_tokens"] = json!(3);
        assert!(u_predecessor(&prev, &next, 245).is_err());
        next = valid.clone();
        next["native"]["bank_trace"]["cue_bank"]["carrier"]["query"]["token_ids"] = json!([7, 9]);
        assert!(u_predecessor(&prev, &next, 245).is_err());
        next = valid.clone();
        next["actual_prefix_ids"] = json!([617, 2097, 315, 99]);
        assert!(u_predecessor(&prev, &next, 245).is_err());
        next = valid.clone();
        next["native"]["continuation"]["state_codes"][7] = json!(120);
        assert!(u_predecessor(&prev, &next, 245).is_err());
        next = valid;
        next["native"]["bank_trace"]["prefix"]["metadata"]["binding"] = json!("other");
        assert!(u_predecessor(&prev, &next, 245).is_err());
        Ok(())
    }
    #[test]
    fn strict_ties_and_neighbor_wrap() -> Result<()> {
        let mut scores = vec![0; 120];
        scores[3] = 7;
        scores[8] = 7;
        assert_eq!(winner(&scores)?, 3);
        assert_eq!(neighbor(3), 0);
        assert_eq!(neighbor(7), 4);
        Ok(())
    }
}
