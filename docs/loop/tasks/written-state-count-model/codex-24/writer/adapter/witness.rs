//! Finite readback of the production binder and its original onset bank.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{env, error::Error, fs};
use uor_r4_integer::{
    all_starts::{Bank, OnsetProfile, BANK_BYTES, CANDIDATE_WORDS},
    discrete_d11::IntegerDiscreteModel,
    paired_role::{PairedRoleDecision, RoleSpan},
    span_selector::CONT,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MODEL_SHA256: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";

#[derive(Deserialize)]
struct Case {
    prompt_id: String,
    prefix: Vec<u32>,
    gold_person: [usize; 2],
    gold_onset: usize,
}

fn trace(model: &IntegerDiscreteModel, bank: &Bank, case: Case) -> Result<Value> {
    let [start, end] = case.gold_person;
    let span = RoleSpan { start, end };
    let empty = PairedRoleDecision {
        requested: None,
        excluded: None,
        requested_score: 0,
        excluded_score: 0,
        requested_runner_up: 0,
        excluded_runner_up: 0,
    };
    let (decision, witnesses) = model.paired_copy_witness(&case.prefix, empty, Some(span))?;
    let [witness] = witnesses.as_slice() else {
        return Err("expected a single earlier assertion".into());
    };
    let last = witness.fact_span[1] - 1;
    let fence = witness.assertion.fence.ok_or("missing assertion fence")? as usize;
    let onset = witness.onset.ok_or("missing onset bank witness")?;
    let value = witness.assertion.value;
    let roots = model.packed_codebook_roots();
    let replay = model.frozen_role_replay()?;
    let class = |id: u32| replay.assertion_classes[id as usize];
    let mut packet = [0; CANDIDATE_WORDS];
    packet[2..8].copy_from_slice(&[
        case.prefix.len() as u32,
        last as u32,
        start as u32,
        fence as u32,
        value.eligible_mask,
        value.word_start_mask,
    ]);
    let mut candidates = Vec::new();
    for offset in 0..3 {
        let record = &mut packet[8 + 9 * offset..17 + 9 * offset];
        let source = last + offset + 2;
        record[0] = u32::MAX;
        let bit = 1 << offset;
        let eligible = value.eligible_mask & bit != 0;
        let admitted = value.word_start_mask & bit != 0;
        candidates.push(json!({
            "relative":source-last,"source":source,"token":case.prefix[source],
            "class":class(case.prefix[source]),"eligible":eligible,"word_start":admitted,
        }));
        if !eligible || !admitted {
            continue;
        }
        let word_end = (source + 1..fence)
            .find(|&at| class(case.prefix[at]) != CONT)
            .unwrap_or(fence);
        record[..3].copy_from_slice(&[
            source as u32,
            word_end as u32,
            u32::from(fence - source < 8),
        ]);
        if record[2] != 0 {
            for (slot, &id) in case.prefix[last + 1..word_end].iter().enumerate() {
                record[3] |= class(id) << (2 * slot);
                record[4 + slot / 2] |= roots[id as usize] << (16 * (slot % 2));
            }
        }
    }
    let words = bank.score_legacy_row(&packet)?;
    if words[2] != onset.status || (words[3] != u32::MAX).then_some(words[3]) != onset.source {
        return Err("candidate score readback differs from production witness".into());
    }
    let known = |word: u32| (word != u32::MAX).then_some(word);
    for (offset, candidate) in candidates.iter_mut().enumerate() {
        if let [_, _, status, shape, signature, good, bad, radius, num, den, ..] =
            &words[8 + 20 * offset..28 + 20 * offset]
        {
            candidate["score"] = if *status == u32::MAX {
                Value::Null
            } else {
                json!({
                    "status":status,"shape":shape,"class_signature":signature,
                    "positive_distance":known(*good),"negative_distance":known(*bad),
                    "radius":known(*radius),"margin":(*den!=u32::MAX).then_some([*num as i32,*den as i32]),
                })
            };
        }
    }
    Ok(
        json!({"prompt_id":case.prompt_id,"gold_onset":case.gold_onset,"decision":decision,
        "witness":witness,"candidates":candidates}),
    )
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [model, cases] = args.as_slice() else {
        return Err("usage: clause-source-witness MODEL CASES".into());
    };
    let bytes = fs::read(model)?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    if model.artifact_sha256() != MODEL_SHA256 {
        return Err("unexpected writer model digest".into());
    }
    let header_end = 16 + usize::try_from(u64::from_le_bytes(bytes[8..16].try_into()?))?;
    let header: Value = serde_json::from_slice(&bytes[16..header_end])?;
    let bank_start = header_end.next_multiple_of(64);
    let raw = &bytes[bank_start..bank_start + BANK_BYTES];
    let profile: OnsetProfile = serde_json::from_value(header["profile"].clone())?;
    // The production loader already validated these bindings against the parent.
    let bank = Bank::from_profile(
        raw,
        raw[16..48].try_into()?,
        raw[48..80].try_into()?,
        raw[80..112].try_into()?,
        &profile,
    )?;
    let cases: Vec<Case> = serde_json::from_slice(&fs::read(cases)?)?;
    if cases.len() != 4 {
        return Err("expected the four prepared cases".into());
    }
    let rows = cases
        .into_iter()
        .map(|case| trace(&model, &bank, case))
        .collect::<Result<Vec<_>>>()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"model_sha256":MODEL_SHA256,
        "mode":bank.mode().name(),"rows":rows}))?
    );
    Ok(())
}
