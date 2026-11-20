use super::{open_bank, parts, save, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::Path};
use uor_r4_integer::{
    all_starts::{original_packet, OnsetMode, ORIGINAL_CANDIDATE_WORDS, ORIGINAL_DECISION_WORDS},
    discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS},
    paired_role::RoleSpan,
};

#[derive(Deserialize)]
struct Blind {
    row: u32,
    ids: Vec<u32>,
}
#[derive(Deserialize)]
struct Person {
    row: u32,
    requested: [usize; 2],
}
#[derive(Deserialize)]
struct Head {
    person: [usize; 2],
}
#[derive(Deserialize)]
struct Key {
    row: u32,
    stratum: String,
    assertions: Vec<Head>,
    selected: Vec<usize>,
    selected_value_spans: Vec<[usize; 2]>,
    expected_tape: Option<Vec<u32>>,
    wrong_source: Option<u32>,
}
fn records<T: for<'a> Deserialize<'a>>(p: &Path) -> Result<Vec<T>> {
    fs::read_to_string(p)?
        .lines()
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}
fn span([start, end]: [usize; 2]) -> RoleSpan {
    RoleSpan { start, end }
}

fn generate(
    model: &IntegerDiscreteModel,
    ids: &[u32],
    source: Option<u32>,
) -> Result<(Vec<u32>, Vec<[u32; 8]>)> {
    let mut state = ExactCopyState::new(source, ids.len().try_into()?);
    let mut ctx = ids.to_vec();
    let mut tape = Vec::new();
    let mut cursor = Vec::new();
    for _ in 0..16 {
        let initial = state
            .write(ids, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, 1);
        let step = if initial.copied != 0 || initial.completed_span() {
            initial
        } else {
            state
                .write(
                    ids,
                    model.evaluate_prefix(&ctx)?.prediction,
                    ROLE_COPY_STOPS,
                )?
                .finish_with_eos(true, 1)
        };
        tape.push(step.emitted);
        cursor.push(step.words());
        if step.emitted == 1 {
            break;
        }
        state = step.state;
        if ctx.len() == model.shape().context as usize {
            ctx.remove(0);
        }
        ctx.push(step.emitted);
    }
    Ok((tape, cursor))
}
fn words(bytes: &mut Vec<u8>, values: &[u32]) {
    bytes.extend(values.iter().flat_map(|v| v.to_le_bytes()));
}

pub fn grade(model_file: &Path, panel: &Path, clients: Option<&Path>, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    let bytes = fs::read(model_file)?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    let mode = model.onset_mode().ok_or("missing onset policy")?;
    let frozen = parts(&bytes)?;
    let bank = open_bank(frozen.bank, mode)?;
    let roots = model.packed_codebook_roots();
    let classes = model.frozen_role_replay()?.assertion_classes;
    if let Some(clients) = clients {
        if fs::read_to_string(clients.join("artifact.sha256"))?.trim() != model.artifact_sha256() {
            return Err("client manifest differs".into());
        }
    }
    let blind = records::<Blind>(&panel.join("held.blind.jsonl"))?;
    let people = records::<Person>(&panel.join("held.person.jsonl"))?;
    let keys = records::<Key>(&panel.join("held.gold.jsonl"))?;
    if blind.len() != keys.len() || keys.len() != people.len() || keys.is_empty() {
        return Err("incomplete input".into());
    }
    let mut inputs = vec![0; 16];
    let mut expected = vec![0; 16];
    let mut rows = Vec::new();
    let mut visits = Vec::new();
    for ((b, p), k) in blind.into_iter().zip(people).zip(keys) {
        if b.row != k.row || p.row != k.row {
            return Err("crossed prompt rows".into());
        }
        let ordinary = model.paired_roles_diagnostic(&b.ids, 0..b.ids.len(), false)?;
        let gold_person = span(p.requested);
        let mut supplied = ordinary;
        supplied.requested = Some(gold_person);
        let heads = k
            .selected
            .iter()
            .map(|&i| {
                k.assertions
                    .get(i)
                    .map(|a| span(a.person))
                    .ok_or("invalid head")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut jobs = Vec::new();
        for (lane, role, selected) in [
            ("ordinary", ordinary, None),
            ("person", supplied, None),
            ("assertions", supplied, Some(heads.as_slice())),
        ] {
            let (decision, observations) = match selected {
                Some(heads) => model.paired_copy_selected_witness(&b.ids, role, heads)?,
                None => model.paired_copy_witness(&b.ids, role, None)?,
            };
            let mut stages = Vec::new();
            for w in observations {
                let query = role.requested.ok_or("assertion without query")?;
                let mut packet = original_packet(
                    &b.ids,
                    &roots,
                    classes,
                    w.fact_span[1] - 1,
                    query.start,
                    w.assertion,
                )?;
                if mode == OnsetMode::Ordered {
                    let early = packet[6..]
                        .chunks_exact(10)
                        .take(packet[5] as usize)
                        .take_while(|c| c[2] <= 3)
                        .count();
                    packet[5] = early as u32;
                    packet[6 + 10 * early..].fill(0);
                }
                let trace = bank.score_original_row(&packet)?;
                let onset = w.onset.ok_or("unscored assertion")?;
                if onset.status != trace[0] || onset.source.unwrap_or(u32::MAX) != trace[1] {
                    return Err(
                        format!("native packet differential failed {} {lane}", k.row).into(),
                    );
                }
                words(&mut inputs, &packet);
                words(&mut expected, &trace);
                let candidates = &trace[2..2 + 10 * packet[5] as usize];
                let stage = json!({"visit":visits.len(),"head":w.fact_span,"query":p.requested,"actual_query":[query.start,query.end],"fence":w.assertion.fence,"continuation":w.assertion.continuation.source,"onset":onset,"completion":w.completed_source_range,"candidate_words":candidates.chunks_exact(10).collect::<Vec<_>>()});
                visits.push(json!({"row":k.row,"lane":lane,"stage":stage}));
                stages.push(stage);
            }
            jobs.push((
                lane,
                role.requested,
                decision.choice.source,
                Some(serde_json::to_value(decision)?),
                stages,
            ));
        }
        let Some(clients) = clients else { continue };
        if let Some(tape) = &k.expected_tape {
            let [start, end] = *k
                .selected_value_spans
                .first()
                .ok_or("missing supported source")?;
            if b.ids.get(start..end) != tape.strip_suffix(&[1]) {
                return Err("original source and full expected IDs differ".into());
            }
            jobs.push((
                "gold_value",
                Some(gold_person),
                Some(start.try_into()?),
                None,
                vec![],
            ));
            // Duplicate-agreement controls have no opposing object; a later complete
            // suffix within the source is a legal wrong start if one exists.
            let competitor = k.wrong_source.or_else(|| {
                (start + 1..end)
                    .find(|&s| classes[b.ids[s] as usize] == 1)
                    .map(|s| s as u32)
            });
            if let Some(s) = competitor {
                jobs.push(("opposing", Some(gold_person), Some(s), None, vec![]));
            }
        }
        jobs.push(("disabled", None, None, None, vec![]));
        let mut cache = std::collections::BTreeMap::new();
        for (lane, person, source, decision, stages) in jobs {
            if let std::collections::btree_map::Entry::Vacant(entry) = cache.entry(source) {
                entry.insert(generate(&model, &b.ids, source)?);
            }
            let (tape, cursor) = &cache[&source];
            let exact = k.expected_tape.as_ref().map(|target| target == tape);
            let first = k.expected_tape.as_ref().and_then(|target| {
                (0..target.len().max(tape.len()))
                    .find(|&i| target.get(i) != tape.get(i))
                    .map(|i| json!({"offset":i,"expected":target.get(i),"actual":tape.get(i)}))
            });
            if (lane == "gold_value" && exact != Some(true))
                || (lane == "opposing" && exact != Some(false))
            {
                return Err("writer oracle failed".into());
            }
            if lane == "ordinary" {
                let c: Value =
                    serde_json::from_slice(&fs::read(clients.join(format!("{}.json", k.row)))?)?;
                if c["artifact_sha256"] != model.artifact_sha256()
                    || c["tokenizer_sha256"]
                        != model.tokenizer_sha256().ok_or("absent tokenizer")?
                    || c["read_enabled"] != true
                    || c["prompt_ids"] != json!(b.ids)
                    || c["generated_ids"] != json!(tape)
                    || c["copy_step_words"] != json!(cursor)
                    || Some(&c["paired"]) != decision.as_ref()
                    || c["stop"]
                        != if tape.last() == Some(&1) {
                            "eos"
                        } else {
                            "max_tokens"
                        }
                {
                    return Err(format!("normal client differs on {}", k.row).into());
                }
            }
            rows.push(json!({"row":k.row,"stratum":k.stratum,"lane":lane,"person":person.map(|s|[s.start,s.end]),"person_correct":person==Some(gold_person),"source":source,"value_spans":k.selected_value_spans,"source_correct":k.expected_tape.is_some()&&source.is_some_and(|s|k.selected_value_spans.iter().any(|p|p[0]==s as usize)),"expected":k.expected_tape,"tape":tape,"first":first,"exact":exact,"decision":decision,"stages":stages}));
        }
    }
    let count: u32 = visits.len().try_into()?;
    if count == 0 {
        return Err("no assertion visits".into());
    }
    for (buffer, magic, width) in [
        (&mut inputs, 0x49383244, ORIGINAL_CANDIDATE_WORDS),
        (&mut expected, 0x4f383244, ORIGINAL_DECISION_WORDS),
    ] {
        for (slot, word) in buffer[..16]
            .chunks_exact_mut(4)
            .zip([magic, count, width as u32, 0])
        {
            slot.copy_from_slice(&word.to_le_bytes());
        }
    }
    save(&out.join("input.bin"), &inputs)?;
    save(&out.join("expected.bin"), &expected)?;
    save(&out.join("bank.raw"), &frozen.bank[super::BINDINGS..])?;
    save(&out.join("visits.json"), &serde_json::to_vec(&visits)?)?;
    save(&out.join("rows.json"), &serde_json::to_vec(&rows)?)
}
