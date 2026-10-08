//! Complete consumed Context path for one recorded adjacent native coefficient.
//! Reuses authenticated controls; captures only the two missing task frames.
use super::*;
use uor_r4_integer::{
    geometric_context::{ContextDecisionEvent, ContextDecisionFamily, ContextInvocation},
    geometric_context_q4::{
        basis_score_q24, unpack_coefficients, ContextQ4Config, NativeContextQ4,
    },
    h4_tables::{H4Code, HistoricalH4Tables},
};
const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
const GENERATE: &str = "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737";
const FIELD: &str = "82ae9daeb402b288e64492d5b299110b36849907019c952609a1cae6612673ee";
const MAX_EVENTS: usize = 20_000;
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_decomposition_root: PathBuf,
    pub retained_probe_root: PathBuf,
    pub retained_intermediate_root: PathBuf,
    pub retained_context_root: PathBuf,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.context_path_credit {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.prefix_context_credit.is_none()
                && a.readout_coadaptation.is_none()
                && a.reached_u.is_none()
                && a.prototype_compensation.is_none()
                && a.retained_context_root.is_none()
                && a.reference_replay.is_none()
                && !a.constrained_context_learning
                && !a.constrained_emission_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only
                && !a.native_code_proposals
                && !a.reached_frontier_objective,
            "Context path capture requires exclusive native diagnostic mode",
        )?;
        replay_require(
            fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(c.retained_intermediate_root.join("checkpoint-0001"))?,
            "Context path checkpoint differs",
        )?;
        replay_require(
            a.maximum_report_bytes <= 128 * 1024 * 1024,
            "Context path report admission exceeds128MiB",
        )?;
    }
    Ok(())
}
fn sealed(root: &Path, report: &str, seal: &str) -> Result<Value> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == report
            && sha256_file(&root.join("manifest.json"))? == seal,
        "Context path authority differs",
    )?;
    let r = read(&root.join("report.json"))?;
    replay_require(
        r["status"] == "COMPLETED",
        "Context path authority incomplete",
    )?;
    Ok(r)
}
fn num(v: &Value) -> Result<usize> {
    Ok(v.as_u64()
        .ok_or_else(|| bad("Context path integer absent"))?
        .try_into()?)
}
fn code(x: u8) -> Result<H4Code> {
    Ok(H4Code::try_from(x)?)
}
fn earliest(scores: &[i64]) -> Result<usize> {
    scores
        .iter()
        .enumerate()
        .fold(None, |best: Option<(usize, i64)>, (i, &x)| {
            if best.is_none_or(|(_, b)| x > b) {
                Some((i, x))
            } else {
                best
            }
        })
        .map(|x| x.0)
        .ok_or_else(|| bad("Context scores empty"))
}
fn snapshot(step: &NativeBankGenerateStep) -> Result<Value> {
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("Prefix Context U witness absent"))?;
    let bridge = step
        .bridge
        .as_ref()
        .ok_or_else(|| bad("Prefix Context bridge absent"))?;
    Ok(
        json!({"generate_q24":step.generate_raw_scores_q24,"copy_ids":step.copy_token_ids,"copy_q24":step.copy_raw_scores_q24,
        "post_state":step.post_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"pool":step.actions,
        "bridge":{"selected_ordinal":bridge.selected_ordinal,"selected_candidate":bridge.selected_candidate,"query_state":bridge.query_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"source_state":bridge.source_state.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_codes":bridge.action_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"action_scores_q24":bridge.action_scores_q24,"counts":bridge.counts},"continuation":{"query_tokens":u.query_tokens,"actual_prefix_tokens":u.actual_prefix_tokens,"state_codes":u.state_codes.iter().map(|c|c.index()).collect::<Vec<_>>(),"delta_scores_q24":u.delta_scores_q24,"encoding_coefficient_reads":u.encoding_coefficient_reads,"counts":u.counts},
        "bank_trace":step.bank_trace}),
    )
}
fn parity(v: &Value, saved: &Value) -> Result<()> {
    for key in ["generate_q24", "copy_ids", "copy_q24", "post_state", "pool"] {
        replay_require(
            v[key] == saved[key],
            &format!("Prefix Context saved {key} parity differs"),
        )?;
    }
    for key in [
        "selected_ordinal",
        "selected_candidate",
        "query_state",
        "source_state",
        "action_codes",
        "action_scores_q24",
        "counts",
    ] {
        replay_require(
            v["bridge"][key] == saved["bridge"][key],
            &format!("Prefix Context bridge {key} differs"),
        )?;
    }
    replay_require(
        v["continuation"] == saved["continuation"] && v["bank_trace"] == saved["bank_trace"],
        "Prefix Context complete U/bank trace parity differs",
    )
}
fn block_shape(block: &[ContextDecisionEvent], call_index: usize) -> Result<()> {
    let first = block.first().ok_or_else(|| bad("Context block empty"))?;
    let families: &[ContextDecisionFamily] = match first.invocation {
        ContextInvocation::Step => &[
            ContextDecisionFamily::Transition,
            ContextDecisionFamily::Root,
            ContextDecisionFamily::Category,
        ],
        ContextInvocation::ObserveStates => {
            &[ContextDecisionFamily::Root, ContextDecisionFamily::Category]
        }
    };
    replay_require(
        block.len() == 8 * families.len(),
        "Context block cardinality differs",
    )?;
    for (i, e) in block.iter().enumerate() {
        let flat = i / families.len();
        replay_require(
            e.call_index == call_index
                && e.invocation == first.invocation
                && e.token_id == first.token_id
                && e.flat_lane == flat
                && e.head == flat / 4
                && e.lane == flat % 4
                && e.family == families[i % families.len()],
            "Context capture family/lane/order incomplete",
        )?;
    }
    Ok(())
}
fn validate_events(events: &[ContextDecisionEvent], tables: &NativeContextQ4) -> Result<Value> {
    replay_require(
        !events.is_empty() && events.len() <= MAX_EVENTS,
        "Context call capture empty/overflow",
    )?;
    let mut at = 0;
    let mut calls = 0;
    let mut steps = 0;
    let mut observations = 0;
    while at < events.len() {
        let first = &events[at];
        let families: &[ContextDecisionFamily] = match first.invocation {
            ContextInvocation::Step => &[
                ContextDecisionFamily::Transition,
                ContextDecisionFamily::Root,
                ContextDecisionFamily::Category,
            ],
            ContextInvocation::ObserveStates => {
                &[ContextDecisionFamily::Root, ContextDecisionFamily::Category]
            }
        };
        let width = 8 * families.len();
        replay_require(
            at + width <= events.len() && first.call_index == calls,
            "Context capture call range/reset differs",
        )?;
        let block = &events[at..at + width];
        block_shape(block, calls)?;
        for (i, e) in block.iter().enumerate() {
            let flat = i / families.len();
            replay_require(
                e.call_index == calls
                    && e.invocation == first.invocation
                    && e.token_id == first.token_id
                    && e.flat_lane == flat
                    && e.head == flat / 4
                    && e.lane == flat % 4
                    && e.family == families[i % families.len()],
                "Context capture family/lane/order incomplete",
            )?;
            let (winner, scores) = tables.native().decision_scores(
                e.token_id,
                flat,
                e.family,
                code(e.own)?,
                code(e.neighbor)?,
            )?;
            replay_require(
                e.scores_q24 == scores
                    && e.winner == winner
                    && usize::from(winner) == earliest(&scores)?,
                "Context captured native scores/ties differ",
            )?;
            let sibling = flat / 4 * 4 + (flat % 4 + 1) % 4;
            let same_family = &block[sibling * families.len() + i % families.len()];
            replay_require(
                e.neighbor == same_family.own,
                "Context captured synchronous neighbor differs",
            )?;
        }
        if first.invocation == ContextInvocation::Step {
            let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
                "../../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
            ))?;
            for flat in 0..8 {
                let t = &block[flat * 3];
                let r = &block[flat * 3 + 1];
                let c = &block[flat * 3 + 2];
                replay_require(
                    r.own == c.own
                        && r.neighbor == c.neighbor
                        && geometry.compose(code(t.own)?, code(t.winner)?).index() == r.own,
                    "Context synchronous transition/observation state differs",
                )?;
            }
            steps += 1;
        } else {
            observations += 1;
        }
        at += width;
        calls += 1;
    }
    Ok(
        json!({"events":events.len(),"encoder_calls":calls,"step_calls":steps,"observe_calls":observations,"complete_all_families":true,"native_scores_and_earliest_ties":true}),
    )
}
fn margins(
    events: &[ContextDecisionEvent],
    family_offset: usize,
    q: &[i8],
    call: &Value,
) -> Result<Vec<Value>> {
    let start = family_offset + 3610 / 4 * 4;
    let old: [i8; 4] = q
        .get(start..start + 4)
        .ok_or_else(|| bad("Context basis row absent"))?
        .try_into()?;
    replay_require(old[2] == -1, "Recorded3610 parentcode differs")?;
    let mut new = old;
    new[2] = -2;
    let mut rows = Vec::new();
    for (event_index, e) in events.iter().enumerate() {
        if e.family != ContextDecisionFamily::Transition || e.flat_lane != 7 {
            continue;
        }
        let effect =
            i64::from(basis_score_q24(new, e.own)?) - i64::from(basis_score_q24(old, e.own)?);
        let mut shifted = e.scores_q24.clone();
        shifted[62] += effect;
        let next = earliest(&shifted)?;
        rows.push(json!({"scope":call,"event_index":event_index,"call_index":e.call_index,"token_id":e.token_id,"own":e.own,"neighbor":e.neighbor,"winner":e.winner,"class":62,"selected_class_gap_q24":e.scores_q24[usize::from(e.winner)]-e.scores_q24[62],"selected_class_effect_q24":effect,"hypothetical_winner":next,"frozen_input_action_changed":next!=usize::from(e.winner)}));
    }
    Ok(rows)
}
fn stream(a: &Args, file: &str, v: &impl serde::Serialize) -> Result<Value> {
    let path = a.out.join(file);
    let remaining = a
        .maximum_report_bytes
        .checked_sub(size(&a.out)? + 65536)
        .ok_or_else(|| bad("Context output cap exhausted"))?;
    let mut writer = constrained_context::BudgetWriter {
        inner: io::BufWriter::new(
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?,
        ),
        remaining,
    };
    serde_json::to_writer(&mut writer, v)?;
    io::Write::flush(&mut writer)?;
    Ok(json!({"file":file,"sha256":sha256_file(&path)?,"bytes":fs::metadata(path)?.len()}))
}
fn u_suffix(events: &[ContextDecisionEvent], saved: &Value, prefix: &[u32]) -> Result<Value> {
    let query: Vec<u32> = serde_json::from_value(
        saved["bank_trace"]["cue_bank"]["carrier"]["query"]["token_ids"].clone(),
    )?;
    replay_require(
        num(&saved["continuation"]["query_tokens"])? == query.len()
            && num(&saved["continuation"]["actual_prefix_tokens"])? == prefix.len(),
        "Context U query/prefix counts differ",
    )?;
    let tokens = query.iter().chain(prefix).copied().collect::<Vec<_>>();
    let count = tokens
        .len()
        .checked_mul(24)
        .ok_or_else(|| bad("U suffix size overflow"))?;
    let suffix = events
        .get(
            events
                .len()
                .checked_sub(count)
                .ok_or_else(|| bad("U suffix absent"))?..,
        )
        .ok_or_else(|| bad("U suffix range"))?;
    let geometry = HistoricalH4Tables::from_bytes(include_bytes!(
        "../../../uor-r4-integer/fixtures/historical-h4-tables-v1.bin"
    ))?;
    let mut state = vec![1u8; 8];
    for (&token, block) in tokens.iter().zip(suffix.chunks_exact(24)) {
        let mut next = state.clone();
        for flat in 0..8 {
            let e = &block[flat * 3];
            replay_require(
                e.invocation == ContextInvocation::Step
                    && e.family == ContextDecisionFamily::Transition
                    && e.token_id == token as usize
                    && e.own == state[flat]
                    && e.neighbor == state[flat / 4 * 4 + (flat % 4 + 1) % 4],
                "Context final U consumed-token suffix differs",
            )?;
            next[flat] = geometry
                .compose(code(state[flat])?, code(e.winner)?)
                .index();
        }
        state = next;
    }
    let final_state: Vec<u8> =
        serde_json::from_value(saved["continuation"]["state_codes"].clone())?;
    replay_require(
        state == final_state,
        "Context final U retained state differs",
    )?;
    Ok(
        json!({"query_ids":query,"actual_prefix_ids":prefix,"encoding_step_calls":tokens.len(),"first_event":events.len()-count,"final_state_codes":state,"scope":"algebraic saved action-chain witness; no encoder advance"}),
    )
}
pub(super) fn run(a: &Args, start: Instant) -> Result<Value> {
    validate_settings(a)?;
    let c = a
        .context_path_credit
        .as_ref()
        .ok_or_else(|| bad("Context path config absent"))?;
    let parent = sealed(
        &c.retained_intermediate_root,
        "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee",
        "de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5",
    )?;
    let probe = sealed(
        &c.retained_probe_root,
        "1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9",
        "c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332",
    )?;
    let derived = sealed(
        &c.retained_decomposition_root,
        "3a0c30dff9c8bd9b1f5c7a30251de432b90ef07b62f77ff79ab1d3b00bca0722",
        "a9b29208992b009fc55362116490d5ebe5746cd934f03ebd382b6b86c4b11b8b",
    )?;
    let oldreport = sealed(
        &c.retained_context_root,
        "fba3f4cbcdc475147062105bf175a82f0556e873c6a6b6918714d23c5aac57e4",
        "6be19fd174c0feb55b0bfa07815fb86dbac75739b2b86e7b2546b7bc5dcbc35f",
    )?;
    let p = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        parent["final_receipt"] == p.receipt
            && parent["mode"] == "readout_intermediate_candidate"
            && parent["selected_model"] == true
            && p.binding.metadata_sha256 == SOURCE
            && p.generate_sha256 == GENERATE
            && probe["parent_report_sha256"]
                == "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee"
            && derived["saved_run_report_sha256"]
                == "1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9",
        "Context path ancestry differs",
    )?;
    let candidate = derived["rankings"]["on_directed"]["top20_temporal_dot"][0].clone();
    replay_require(
        candidate["name"] == "consumer.context.self_transition"
            && candidate["index"] == 3610
            && candidate["before"] == -1
            && candidate["after"] == -2,
        "Context recorded3610 differs",
    )?;
    let oldroot = c.retained_context_root.join("native-candidate-00");
    let oldplan: ReferencePlan =
        serde_json::from_value(read(&oldroot.join("reference-plan.json"))?)?;
    let oldobjective = read(&oldroot.join("objective.json"))?;
    replay_require(
        oldobjective["source_binding"] == serde_json::to_value(&p.binding)?
            && oldplan.input_sha256 == INPUT_SHA
            && oldplan.labels_sha256 == LABEL_SHA,
        "Retained Context packet/source epoch differs",
    )?;
    let packed = fs::read(a.checkpoint.join("native/consumer/context-q4.bin"))?;
    replay_require(
        packed == fs::read(oldroot.join("checkpoint-0000/native/consumer/context-q4.bin"))?,
        "Retained/current Context packed payload differs",
    )?;
    let config = ContextQ4Config {
        vocab_size: 4096,
        heads: 2,
        lanes_per_head: 4,
    };
    let tables = NativeContextQ4::new(config, &packed)?;
    let q = unpack_coefficients(config.coefficient_count()?, &packed)?;
    let mut offset = 0;
    let mut self_offset = None;
    for (name, shape) in config.coefficient_shapes()? {
        if name == "self_transition" {
            self_offset = Some(offset);
        }
        offset += shape.iter().product::<usize>();
    }
    let self_offset = self_offset.ok_or_else(|| bad("Context self basis absent"))?;
    for (path, pin) in [
        (&a.training_inputs, INPUT_SHA),
        (&a.training_labels, LABEL_SHA),
    ] {
        report_output::verify(&seal_for(path)?)?;
        replay_require(sha256_file(path)? == pin, "Context panel authority differs")?;
    }
    replay_require(
        a.training_inputs == a.development_inputs && a.training_labels == a.development_labels,
        "Context panel paths differ",
    )?;
    let reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let legal = reducer
        .legal_token_ids()
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &p.integer,
        &p.tokenizer,
        &legal,
        512,
    )?;
    replay_require(eps.len() == 512, "Context panel length")?;
    let oldpath = oldroot.join("context-candidate-decisions.json");
    replay_require(
        sha256_file(&oldpath)?
            == "2b64b25b38b9dda869c7a4b5b2c030cb66d17aa72dfe6693cd5498a212418a12",
        "Retained full capture hash differs",
    )?;
    let numerical_projection = fs::metadata(&oldpath)?.len()
        + 4 * tables.stats().expanded_table_bytes as u64
        + 64 * 1024 * 1024
        + 32 * 1024 * 1024;
    replay_require(
        numerical_projection <= 512 * 1024 * 1024,
        "Context numerical cache projection exceeds512MiB",
    )?;
    let old: constrained_context::CapturedDecisions =
        serde_json::from_reader(io::BufReader::new(fs::File::open(&oldpath)?))?;
    replay_require(
        old.calls.len() == 92 && old.events.len() == 242496,
        "Retained complete capture population differs",
    )?;
    let mut endpoint = 0;
    for call in &old.calls {
        let first = num(&call["first"])?;
        let count = num(&call["count"])?;
        replay_require(
            first == endpoint && first + count <= old.events.len(),
            "Retained complete capture ranges differ",
        )?;
        endpoint += count;
    }
    replay_require(endpoint == old.events.len(), "Retained events unclaimed")?;
    let mut reused = Vec::new();
    let mut margin_rows = Vec::new();
    let mut frames = Vec::new();
    for input in CONTROL_INDICES {
        let row = oldplan
            .rows
            .get(input)
            .ok_or_else(|| bad("Retained packet row absent"))?;
        let e = eps.get(input).ok_or_else(|| bad("Context input absent"))?;
        replay_require(
            row.index == input
                && row.id == e.packet.id
                && row.packet_sha256 == sha256_bytes(&serde_json::to_vec(&e.packet)?),
            "Retained/current exact packet differs",
        )?;
        let admissions = old
            .calls
            .iter()
            .filter(|v| v["kind"] == "bank_admission" && v["index"] == input)
            .collect::<Vec<_>>();
        replay_require(
            admissions.len() == 1 && admissions[0]["count"] == 0,
            "Retained admission coverage differs",
        )?;
        reused.push(json!({"origin":"retained","call":admissions[0],"coverage":{"events":0,"reason":"bank admission compiler/tokenizer path performs no Context calls; guard reached before admission"}}));
        for position in [3, 4] {
            let path = c.retained_probe_root.join(format!(
                "baseline-row-{input:04}-position-{position:02}.json"
            ));
            let saved = read(&path)?;
            let prefix: Vec<u32> = serde_json::from_value(saved["actual_prefix_ids"].clone())?;
            replay_require(
                saved["id"] == row.id
                    && saved["input_index"] == input
                    && saved["position"] == position
                    && prefix.len() == position
                    && prefix == row.targets[..position],
                "Retained/current control prefix differs",
            )?;
            let found = old
                .calls
                .iter()
                .filter(|v| {
                    v["kind"] == "native_step" && v["index"] == input && v["position"] == position
                })
                .collect::<Vec<_>>();
            replay_require(
                found.len() == 1 && found[0]["prefix"] == json!(prefix),
                "Retained step call coverage differs",
            )?;
            let original = found[0];
            let first = num(&original["first"])?;
            let count = num(&original["count"])?;
            let events = &old.events[first..first + count];
            let term = oldobjective["terms"]
                .as_array()
                .ok_or_else(|| bad("Retained objective terms absent"))?
                .iter()
                .find(|t| t["term"]["index"] == input && t["term"]["position"] == position)
                .ok_or_else(|| bad("Retained exact term absent"))?;
            replay_require(
                term["term"]["parent_actual_prefix_ids"] == json!(prefix)
                    && term["continuation"]["query_tokens"]
                        == saved["native"]["continuation"]["query_tokens"]
                    && term["continuation"]["state_codes"]
                        == saved["native"]["continuation"]["state_codes"],
                "Retained/current control query/continuation inputs differ",
            )?;
            let scope = json!({"origin":"retained","input_index":input,"position":position,"original_call":original,"packet_sha256":row.packet_sha256,"baseline_file":path,"baseline_sha256":sha256_file(&path)?});
            let coverage = validate_events(events, &tables)?;
            let u = u_suffix(events, &saved["native"], &prefix)?;
            margin_rows.extend(margins(events, self_offset, &q, &scope)?);
            reused.push(json!({"scope":scope,"coverage":coverage,"u_suffix":u,"events_sha256":sha256_bytes(&serde_json::to_vec(events)?)}));
            frames.push(json!({"input_index":input,"position":position,"id":row.id,"origin":"retained","packet_sha256":row.packet_sha256,"actual_prefix_ids":prefix}));
        }
    }
    drop(old);
    let field = fs::read(a.checkpoint.join("continuation-field.bin"))?;
    replay_require(
        sha256_bytes(&field) == FIELD,
        "Context fixed U artifact differs",
    )?;
    let mut generator = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &field,
        sha256: FIELD,
    })?;
    let e = &eps[245];
    let guard = generator.capture_context_decisions(MAX_EVENTS)?;
    let bank = generator.admit_bank(continuation_snapshot(&e.packet)?)?;
    let admission = guard.finish()?;
    replay_require(
        admission.is_empty(),
        "Fresh admission unexpectedly consumes Context; scope expectation invalid",
    )?;
    let mut fresh_events = Vec::new();
    let mut fresh_calls = vec![json!({"kind":"bank_admission","index":245,"first":0,"count":0})];
    let mut fresh = vec![
        json!({"origin":"fresh","kind":"bank_admission","input_index":245,"coverage":{"events":0,"guard_finish":"PASS","scope":"guard enabled before admission; compiler/tokenizer admission consumes no Context decisions"}}),
    ];
    for position in [3, 4] {
        deadline(a, start)?;
        let path = c
            .retained_probe_root
            .join(format!("baseline-row-0245-position-{position:02}.json"));
        let saved = read(&path)?;
        let prefix: Vec<u32> = serde_json::from_value(saved["actual_prefix_ids"].clone())?;
        replay_require(
            saved["id"] == e.packet.id
                && saved["input_index"] == 245
                && saved["position"] == position
                && prefix.len() == position,
            "Fresh task frame identity differs",
        )?;
        let actual = read(
            &c.retained_intermediate_root
                .join("development-0001-row-0245.json"),
        )?;
        replay_require(
            actual["id"] == e.packet.id
                && actual["generation"][position]["actual_prefix_ids"] == json!(prefix)
                && prefix.iter().enumerate().all(|(i, t)| {
                    actual["generation"][i]["pool"]["summary"]["chosen_token_id"] == *t
                }),
            "Fresh task actual ancestry differs",
        )?;
        let guard = generator.capture_context_decisions(MAX_EVENTS - fresh_events.len())?;
        let step = generator.step(&bank, &prefix)?;
        let events = guard.finish()?;
        let native = snapshot(&step)?;
        parity(&native, &saved["native"])?;
        let scope = json!({"origin":"fresh","input_index":245,"position":position,"packet_sha256":sha256_bytes(&serde_json::to_vec(&e.packet)?),"baseline_file":path,"baseline_sha256":sha256_file(&path)?});
        let coverage = validate_events(&events, &tables)?;
        let u = u_suffix(&events, &native, &prefix)?;
        let native_snapshot = stream(
            a,
            &format!("fresh-row-0245-position-{position:02}.json"),
            &json!({"input_index":245,"position":position,"id":e.packet.id,"actual_prefix_ids":prefix,"native":native}),
        )?;
        margin_rows.extend(margins(&events, self_offset, &q, &scope)?);
        let first = fresh_events.len();
        fresh_events.extend(events);
        fresh_calls.push(json!({"kind":"native_step","index":245,"position":position,"prefix":prefix,"first":first,"count":fresh_events.len()-first}));
        fresh.push(json!({"scope":scope,"coverage":coverage,"u_suffix":u,"native_parity":true,"native_snapshot":native_snapshot}));
        frames.push(json!({"input_index":245,"position":position,"id":e.packet.id,"origin":"fresh","actual_prefix_ids":prefix}));
    }
    let capture = stream(
        a,
        "task-context-decisions.json",
        &json!({"calls":fresh_calls,"events":fresh_events}),
    )?;
    let margin_file = stream(a, "complete-path-margins.json", &margin_rows)?;
    let task_changes = margin_rows
        .iter()
        .filter(|x| x["scope"]["input_index"] == 245 && x["frozen_input_action_changed"] == true)
        .count();
    let control_changes = margin_rows
        .iter()
        .filter(|x| x["scope"]["input_index"] != 245 && x["frozen_input_action_changed"] == true)
        .count();
    Ok(
        json!({"schema":"uor-r4.context-path-credit/1","numerical_cache_projection_bytes":numerical_projection,"maximum_numerical_cache_bytes":536870912u64,"maximum_process_ram_bytes":4294967296u64,"maximum_temporary_bytes":536870912u64,"maximum_cpu_threads":2,"authority_scope":"original reference-plan Source identity remains historical; packet pins reused, actual candidate Context Source epoch from objective/checkpoint verified","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"candidate":candidate,"source_binding":p.binding,"context_packed_sha256":sha256_bytes(&packed),"generate_sha256":GENERATE,"continuation_sha256":FIELD,"authorities":{"decomposition":{"report":"3a0c30dff9c8bd9b1f5c7a30251de432b90ef07b62f77ff79ab1d3b00bca0722","seal":"a9b29208992b009fc55362116490d5ebe5746cd934f03ebd382b6b86c4b11b8b"},"probe":{"report":"1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9","seal":"c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332"},"parent":{"report":"c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee","seal":"de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5"},"retained":{"report":"fba3f4cbcdc475147062105bf175a82f0556e873c6a6b6918714d23c5aac57e4","seal":"6be19fd174c0feb55b0bfa07815fb86dbac75739b2b86e7b2546b7bc5dcbc35f"}},"retained_report_sha256":"fba3f4cbcdc475147062105bf175a82f0556e873c6a6b6918714d23c5aac57e4","retained_capture_file":oldpath,"retained_capture_sha256":"2b64b25b38b9dda869c7a4b5b2c030cb66d17aa72dfe6693cd5498a212418a12","retained_report_source":oldreport["source_commit"],"retained_reference_plan_sha256":sha256_file(&oldroot.join("reference-plan.json"))?,"retained_objective_sha256":sha256_file(&oldroot.join("objective.json"))?,"frames":frames,"reused_scopes":reused,"fresh_scopes":fresh,"fresh_capture":capture,"margins":margin_file,"transition_inputs":margin_rows.len(),"task_action_crossings":task_changes,"control_action_crossings":control_changes,"task_path_unchanged":task_changes==0,"all18_path_unchanged":task_changes==0 && control_changes==0,"first_control_crossing":margin_rows.iter().find(|x|x["scope"]["input_index"]!=245 && x["frozen_input_action_changed"]==true),"first_task_crossing":margin_rows.iter().find(|x|x["scope"]["input_index"]==245 && x["frozen_input_action_changed"]==true),"decision":if task_changes==0{"BOUNDED_COMPLETE_TASK_PATH_UNCHANGED"}else{"TASK_ACTION_CROSSING_REQUIRES_DECLARED_FINITE_CONTRAST"},"elapsed_seconds":start.elapsed().as_secs_f64(),"native_task_steps":2,"native_control_steps":0,"gradient":"NOT_RUN","reranking":"NOT_RUN","optimizer":"NOT_RUN","finite_artifact":"NOT_RUN","autoregressive_rollout":"NOT_RUN","coverage_scope":"all families at every captured encoder call; retained controls only after exact packet/prefix/Source/Context matching; changed readout pools are not a reuse criterion","limitations":["One native coefficient and fixed18 calls only; not family, capacity, gradient or general recurrent qualification","Hard path invariance depends on complete consumed-call coverage and frozen allothercoefficients; no new candidate/fullpool/CE/reply measurement"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_block_rejects_dropped_reordered_or_cross_call_events() -> Result<()> {
        let mut events = Vec::new();
        for flat in 0..8 {
            for family in [
                ContextDecisionFamily::Transition,
                ContextDecisionFamily::Root,
                ContextDecisionFamily::Category,
            ] {
                events.push(ContextDecisionEvent {
                    call_index: 0,
                    invocation: ContextInvocation::Step,
                    family,
                    token_id: 7,
                    head: flat / 4,
                    lane: flat % 4,
                    flat_lane: flat,
                    own: 1,
                    neighbor: 1,
                    winner: 0,
                    scores_q24: vec![],
                });
            }
        }
        block_shape(&events, 0)?;
        assert!(block_shape(&events[..23], 0).is_err());
        let mut changed = events.clone();
        changed.swap(1, 2);
        assert!(block_shape(&changed, 0).is_err());
        changed = events.clone();
        changed[23].call_index = 1;
        assert!(block_shape(&changed, 0).is_err());
        changed = events;
        changed[5].token_id = 9;
        assert!(block_shape(&changed, 0).is_err());
        Ok(())
    }
    #[test]
    fn earliest_tie_uses_class_zero_and_keeps_lower_class() -> Result<()> {
        assert_eq!(earliest(&[0, 0, 0])?, 0);
        assert_eq!(earliest(&[-2, 4, 4])?, 1);
        assert!(earliest(&[]).is_err());
        Ok(())
    }
}
