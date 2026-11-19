use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, fs, path::Path};

const MISSING: u32 = u32::MAX;
const ORDINARY: [&str; 3] = ["reference", "treatment", "no-read"];

fn bytes(root: &Path, name: &str) -> Vec<u8> {
    let path = root.join(name);
    fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read(root: &Path, name: &str) -> (Value, String) {
    let raw = bytes(root, name);
    (
        serde_json::from_slice(&raw).unwrap_or_else(|error| panic!("{name}: {error}")),
        digest(&raw),
    )
}

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("expected JSON array")
}

fn word(value: &Value) -> u32 {
    value
        .as_u64()
        .and_then(|word| word.try_into().ok())
        .expect("expected u32")
}

fn index(value: &Value) -> usize {
    word(value) as usize
}

fn prefix(value: &Value) -> String {
    digest(
        &array(value)
            .iter()
            .flat_map(|id| word(id).to_le_bytes())
            .collect::<Vec<_>>(),
    )
}

fn check_writers(root: &Path, expected_rows: usize) {
    let (receipt, _) = read(root, "writer/receipt.json");
    let (gpu, gpu_sha) = read(root, "writer/gpu.json");
    let (picks, picks_sha) = read(root, "writer/picks.json");
    let (bare, bare_sha) = read(root, "writer/bare.json");
    assert_eq!(receipt["gpu"], gpu_sha);
    assert_eq!(gpu["picks"], picks_sha);
    assert_eq!(gpu["bare"], bare_sha);
    assert_eq!(
        bytes(root, "writer/gpu.json"),
        bytes(root, "writer/nominees/chosen.json")
    );
    assert_eq!(receipt["model"], gpu["model"]);
    let trajectories = array(&gpu["rows"]);
    assert_eq!(trajectories.len(), index(&receipt["unique"]));
    assert_eq!(array(&bare).len(), trajectories.len());
    assert_eq!(array(&picks["rows"]).len(), trajectories.len());
    let mut verified_keys = BTreeSet::new();
    for (position, trajectory) in trajectories.iter().enumerate() {
        let pick = &picks["rows"][position];
        let ids = &bare[position]["prefix"];
        let source = &trajectory["source"];
        assert_eq!(
            trajectory["prefix_sha256"],
            prefix(ids),
            "bare row {position}"
        );
        assert_eq!(
            trajectory["prefix_sha256"], pick["prefix_sha256"],
            "pick row {position}"
        );
        assert_eq!(
            source, &pick["continuation"]["source"],
            "pick source row {position}"
        );
        assert!(
            verified_keys.insert((
                trajectory["prefix_sha256"].as_str().unwrap(),
                source.as_u64()
            )),
            "duplicate writer key at {position}"
        );
    }
    for arm in ORDINARY {
        let (selection, selection_sha) = read(root, &format!("arms/{arm}.json"));
        let (seal, _) = read(root, &format!("clients/{arm}/seal.json"));
        assert_eq!(selection["blind"], receipt["blind"], "{arm}");
        assert_eq!(seal["blind"], receipt["blind"], "{arm}");
        assert_eq!(seal["model"], selection["artifact"], "{arm}");
        let mappings = array(&receipt["arms"])
            .iter()
            .filter(|entry| entry["selection_sha256"] == selection_sha)
            .collect::<Vec<_>>();
        assert_eq!(mappings.len(), 1, "{arm}");
        let maps = array(&mappings[0]["rows"]);
        let rows = array(&selection["rows"]);
        assert_eq!(maps.len(), expected_rows, "{arm}");
        assert_eq!(rows.len(), expected_rows, "{arm}");
        assert_eq!(array(&seal["row_sha256"]).len(), expected_rows, "{arm}");
        let mut transitions = 0;
        let mut sourced = 0;
        let mut capped = Vec::new();
        for (ordinal, (mapping, selected)) in maps.iter().zip(rows).enumerate() {
            let trajectory_index = index(&mapping[1]);
            let trajectory = &trajectories[trajectory_index];
            let (client, client_sha) = read(root, &format!("clients/{arm}/row-{ordinal:04}.json"));
            let context = format!("{arm} row {ordinal}");
            assert_eq!(index(&mapping[0]), ordinal, "{context}");
            assert_eq!(index(&selected["index"]), ordinal, "{context}");
            assert_eq!(seal["row_sha256"][ordinal], client_sha, "{context}");
            assert_eq!(client["artifact_sha256"], seal["model"], "{context}");
            assert_eq!(client["tokenizer_sha256"], seal["tokenizer"], "{context}");
            assert_eq!(
                prefix(&client["prompt_ids"]),
                selected["prefix_sha256"],
                "{context}"
            );
            assert_eq!(
                bare[trajectory_index]["prefix"], client["prompt_ids"],
                "{context}"
            );
            assert_eq!(
                trajectory["prefix_sha256"], selected["prefix_sha256"],
                "{context}"
            );
            assert_eq!(client["paired"], selected["paired"], "{context}");
            assert_eq!(
                client["selected"], selected["paired"]["choice"],
                "{context}"
            );
            assert_eq!(
                client["selected"]["source"], trajectory["source"],
                "{context}"
            );
            assert_eq!(
                client["generated_ids"], trajectory["generated"],
                "{context}"
            );
            assert_eq!(client["read_enabled"], arm != "no-read", "{context}");
            let ids = array(&trajectory["generated"]);
            let eos = ids.last() == Some(&json!(1));
            assert_eq!(
                client["stop"],
                if eos { "eos" } else { "max_tokens" },
                "{context}"
            );
            if !eos {
                assert_eq!(arm, "no-read", "{context}");
                assert_eq!(ids.len(), 16, "{context}");
                capped.push(ordinal);
            }
            assert!(!ids[..ids.len() - 1].contains(&json!(1)), "{context}");
            let steps = array(&trajectory["steps"]);
            let client_words = array(&client["copy_step_words"]);
            assert_eq!(steps.len(), ids.len(), "{context}");
            assert_eq!(steps.len(), client_words.len(), "{context}");
            let original = array(&client["prompt_ids"]);
            let selected_source = trajectory["source"].as_u64().map(|source| source as u32);
            let cursor = selected_source.unwrap_or(MISSING);
            assert_eq!(
                steps[0]["before"],
                json!([
                    u32::from(selected_source.is_some()),
                    cursor,
                    cursor,
                    original.len() as u32
                ]),
                "{context} first cursor"
            );
            for (position, step) in steps.iter().enumerate() {
                let mut words = array(&step["after"]).iter().map(word).collect::<Vec<_>>();
                assert_eq!(words.len(), 4, "{context} step {position}");
                assert_eq!(step["emitted"], ids[position], "{context} step {position}");
                let source_id = &step["source_id"];
                if step["copied"] == 1 {
                    let old_cursor = index(&step["before"][2]);
                    assert_eq!(
                        step["emitted"], original[old_cursor],
                        "{context} step {position}"
                    );
                    assert_eq!(
                        source_id, &original[old_cursor],
                        "{context} step {position}"
                    );
                    assert_eq!(
                        index(&step["after"][2]),
                        old_cursor + 1,
                        "{context} step {position}"
                    );
                }
                words.extend([
                    word(&step["emitted"]),
                    if source_id.is_null() {
                        MISSING
                    } else {
                        word(source_id)
                    },
                    word(&step["copied"]),
                    word(&step["release"]),
                ]);
                assert_eq!(
                    client_words[position],
                    json!(words),
                    "{context} step {position}"
                );
                if position != 0 {
                    assert_eq!(
                        steps[position - 1]["after"],
                        step["before"],
                        "{context} step {position}"
                    );
                }
            }
            transitions += steps.len();
            sourced += usize::from(!trajectory["source"].is_null());
        }
        println!("{arm}: {expected_rows}/{expected_rows} rows, {transitions} cursor transitions, {sourced} sourced, capped prompts {capped:?}");
    }
}

fn frozen_words(raw: &[u8]) -> Vec<u32> {
    assert_eq!(raw.len() % 4, 0);
    raw.chunks_exact(4)
        .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|word| format!("{word:02x}")).collect()
}

fn check_veto(root: &Path, expected_prompts: usize) {
    let packet = bytes(root, "modes/supplied/input.packet");
    let supplied = bytes(root, "modes/supplied/score.trace");
    let (capture, capture_sha) = read(root, "capture/supplied/candidates.json");
    let (supplied_sources, _) = read(root, "modes/supplied/sources.json");
    assert_eq!(supplied_sources["blind_capture_sha256"], capture_sha);
    assert_eq!(capture["base_d11"], hex(&packet[16..48]));
    assert_eq!(capture["tokenizer_sha256"], hex(&packet[48..80]));
    assert_eq!(capture_sha, hex(&packet[112..144]));

    let has_veto = root.join("modes/supplied-veto").exists();
    let veto = if has_veto {
        assert_eq!(packet, bytes(root, "modes/supplied-veto/input.packet"));
        assert_eq!(supplied, bytes(root, "modes/supplied-veto/ordered.trace"));
        let veto = bytes(root, "modes/supplied-veto/score.trace");
        let (veto_sources, _) = read(root, "modes/supplied-veto/sources.json");
        assert_eq!(veto_sources["blind_capture_sha256"], capture_sha);
        assert_eq!(veto_sources["bank_sha256"], supplied_sources["bank_sha256"]);
        assert_eq!(veto_sources["input_packet_sha256"], digest(&packet));
        assert_eq!(veto_sources["trace_sha256"], digest(&veto));
        assert_eq!(veto_sources["mode"], "earlier-connector-v1");
        veto
    } else {
        supplied.clone()
    };
    assert_eq!(supplied_sources["input_packet_sha256"], digest(&packet));
    assert_eq!(supplied_sources["trace_sha256"], digest(&supplied));
    assert_eq!(supplied_sources["mode"], "ordered");

    let raw_packet = frozen_words(&packet);
    let raw_supplied = frozen_words(&supplied);
    let raw_veto = frozen_words(&veto);
    let count = raw_packet[1] as usize;
    assert_eq!(&raw_packet[..4], &[0x33415355, count as u32, 35, 0]);
    assert_eq!(&raw_supplied[..4], &[0x32545355, count as u32, 68, 0]);
    assert_eq!(
        &raw_veto[..4],
        &[0x32545355, count as u32, 68, if has_veto { 6 } else { 0 }]
    );
    assert_eq!(raw_packet.len(), 36 + 35 * count);
    assert_eq!(raw_supplied.len(), 4 + 68 * count);
    assert_eq!(raw_veto.len(), 4 + 68 * count);
    assert_eq!(array(&capture["rows"]).len(), expected_prompts);

    let mut changed = Vec::new();
    let mut unchanged = 0;
    let mut total_candidates = 0;
    for ((input, ordered), trial) in raw_packet[36..]
        .chunks_exact(35)
        .zip(raw_supplied[4..].chunks_exact(68))
        .zip(raw_veto[4..].chunks_exact(68))
    {
        let ordinal = input[0] as usize;
        let assertion = input[1] as usize;
        let captured = &capture["rows"][ordinal];
        assert_eq!(word(&captured["ordinal"]), input[0]);
        assert_eq!(word(&captured["original_tape_len"]), input[2]);
        assert_eq!(input[..2], ordered[..2]);
        assert_eq!(input[..2], trial[..2]);
        let context = format!("prompt {ordinal}, assertion {assertion}");
        if input[1] == MISSING {
            assert!(array(&captured["assertions"]).is_empty(), "{context}");
            assert_eq!(ordered[2..4], [1, MISSING], "{context}");
        } else {
            let fact = &captured["assertions"][assertion];
            assert_eq!(input[3], word(&fact["fact_span"][1]) - 1, "{context}");
            assert_eq!(input[4], word(&captured["requested_span"][0]), "{context}");
            assert_eq!(
                input[5],
                fact["fence"].as_u64().map_or(MISSING, |n| n as u32),
                "{context}"
            );
            assert_eq!(input[6], word(&fact["eligible_mask"]), "{context}");
            assert_eq!(input[7], word(&fact["word_start_mask"]), "{context}");
            let mut slots = 0;
            for candidate in array(&fact["legacy_candidates"]) {
                let depth = index(&candidate["connector_slots"]);
                let data = &input[8 + (depth - 1) * 9..8 + depth * 9];
                let scored = &ordered[8 + (depth - 1) * 20..8 + depth * 20];
                assert_eq!(data[0], word(&candidate["original_start"]), "{context}");
                assert_eq!(data[1], word(&candidate["first_word_end"]), "{context}");
                assert_eq!(data[..2], scored[..2], "{context}");
                let admitted = candidate["writer_reachable"].as_bool().expect("bool");
                assert_eq!(data[2], u32::from(admitted), "{context}");
                if admitted {
                    let roots = array(&candidate["packed_roots_u16"]);
                    assert_eq!(
                        roots.len(),
                        depth + (data[1] - data[0]) as usize,
                        "{context}"
                    );
                    let mut signature = 0;
                    for (slot_index, root) in roots.iter().enumerate() {
                        let packed = (data[4 + slot_index / 2] >> (16 * (slot_index % 2))) & 65535;
                        assert_eq!(packed, word(root), "{context}");
                        assert_eq!(scored[10 + slot_index], packed, "{context}");
                        signature |=
                            word(&candidate["slot_classes"][slot_index]) << (2 * slot_index);
                    }
                    assert_eq!(data[3], signature, "{context}");
                    assert_eq!(scored[4], signature, "{context}");
                    assert_eq!(
                        scored[3],
                        depth as u32 | ((data[1] - data[0]) << 8),
                        "{context}"
                    );
                    assert!([2, 3].contains(&scored[2]), "{context}");
                } else {
                    assert_eq!(scored[2], 1, "{context}");
                }
                slots += 1;
            }
            assert_eq!(
                slots,
                input[8..]
                    .chunks_exact(9)
                    .filter(|slot| slot[0] != MISSING)
                    .count(),
                "{context}"
            );
            total_candidates += slots;
        }
        let first = &ordered[8..28];
        let middle = &ordered[28..48];
        let last = &ordered[48..68];
        let right_signed_margin = |slot: &[u32], positive| {
            slot[2] == 3
                && slot[5] <= slot[7]
                && if positive {
                    slot[6] > slot[5]
                } else {
                    slot[6] < slot[5]
                }
        };
        let earliest_veto = first[2] == 2
            && first[5] != MISSING
            && first[6] != MISSING
            && first[7] != 65535
            && first[7] != MISSING
            && first[5] > first[7]
            && first[6] < first[5];
        let eligible = has_veto
            && earliest_veto
            && right_signed_margin(middle, false)
            && right_signed_margin(last, true)
            && first[1] == middle[0]
            && middle[1] == last[0];
        let differences = ordered
            .iter()
            .zip(trial)
            .enumerate()
            .filter_map(|(offset, (a, b))| (a != b).then_some(offset))
            .collect::<Vec<_>>();
        if eligible {
            assert_eq!(differences, &[2, 3, 10], "{context}");
            assert_eq!(&ordered[2..4], &[2, MISSING], "{context}");
            assert_eq!(&trial[2..4], &[0, last[0]], "{context}");
            assert_eq!(trial[10], 4, "{context}");
            changed.push((ordinal, assertion, first[0], middle[0], last[0]));
        } else {
            assert!(differences.is_empty(), "{context}");
            unchanged += 1;
        }
    }
    println!("supplied scorer: {count} assertion/sentinel rows, {total_candidates} original candidates, {unchanged} identical rows, {} scoped changes", changed.len());
    for (ordinal, assertion, first, middle, last) in changed {
        println!("prompt {ordinal}, assertion {assertion}: cleared {first}, kept {middle} negative, selected {last}");
    }
}

fn verify_manifest(root: &Path, manifest: &str) -> usize {
    let text = String::from_utf8(bytes(root, manifest)).unwrap();
    let mut paths = BTreeSet::new();
    for line in text.lines() {
        let (hash, path) = line.split_once("  ").expect("SHA-256 manifest format");
        assert!(paths.insert(path), "{manifest}: duplicate {path}");
        assert_eq!(hash, digest(&bytes(root, path)), "{manifest}: {path}");
    }
    paths.len()
}

fn compare_supplied_writer(root: &Path) {
    let (receipt, _) = read(root, "writer/receipt.json");
    let (gpu, _) = read(root, "writer/gpu.json");
    let (bare, _) = read(root, "writer/bare.json");
    let arms = ["supplied", "supplied-veto"];
    let mut combined = Vec::new();
    for arm in arms {
        let (selection, selection_sha) = read(root, &format!("arms/{arm}.json"));
        let (selection_blind, selection_blind_sha) = read(root, &format!("arms/{arm}-blind.json"));
        let (source_audit, _) = read(root, &format!("arms/{arm}-source-audit.json"));
        let (decoded, decoded_sha) = read(root, &format!("modes/{arm}/sources.json"));
        let (capture, capture_sha) = read(root, "capture/supplied/candidates.json");
        let traces = frozen_words(&bytes(root, &format!("modes/{arm}/score.trace")));
        let trace_rows = traces[4..].chunks_exact(68);
        assert_eq!(source_audit["candidate_capture_sha256"], capture_sha);
        assert_eq!(source_audit["selection_sha256"], selection_blind_sha);
        assert_eq!(
            selection["original_blind_selection_sha256"],
            selection_blind_sha
        );
        assert_eq!(source_audit["trace"], decoded_sha);
        assert_eq!(selection["blind"], receipt["blind"]);
        let matches = array(&receipt["arms"])
            .iter()
            .filter(|entry| entry["selection_sha256"] == selection_sha)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "{arm}");
        let mappings = array(&matches[0]["rows"]);
        let selected = array(&selection["rows"]);
        assert_eq!(mappings.len(), 96, "{arm}");
        assert_eq!(selected.len(), mappings.len(), "{arm}");
        let decoded_rows = array(&decoded["rows"]);
        assert_eq!(decoded_rows.len(), trace_rows.len(), "{arm}");
        let mut selected_sources = std::collections::BTreeMap::new();
        for (decoded, trace) in decoded_rows.iter().zip(trace_rows) {
            let raw_source = (trace[3] != MISSING).then_some(trace[3]);
            assert_eq!(decoded["source"], json!(raw_source), "{arm}");
            assert_eq!(decoded["ordinal"], trace[0], "{arm}");
            assert_eq!(
                decoded["assertion_index"],
                json!((trace[1] != MISSING).then_some(trace[1])),
                "{arm}"
            );
            assert!(
                selected_sources
                    .insert((trace[0], trace[1]), raw_source)
                    .is_none(),
                "{arm}"
            );
        }
        let mut controlled = Vec::new();
        for (map, row) in mappings.iter().zip(selected) {
            let ordinal = index(&map[0]);
            let replay_index = index(&map[1]);
            let trajectory = &gpu["rows"][replay_index];
            let original = &capture["rows"][ordinal];
            let audit = &source_audit["rows"][ordinal];
            let paired = &row["paired"];
            let original_ids = &bare[replay_index]["prefix"];
            assert_eq!(row["index"], ordinal, "{arm}");
            assert_eq!(
                row, &selection_blind["rows"][ordinal],
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                row["prefix_sha256"],
                prefix(original_ids),
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                row["prefix_sha256"], original["prefix_sha256"],
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                row["prefix_sha256"], audit["prefix_sha256"],
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                row["prefix_sha256"], trajectory["prefix_sha256"],
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                array(&paired["role_words"])[..2],
                array(&original["requested_span"])[..],
                "{arm} prompt {ordinal}"
            );
            let assertions = array(&original["assertions"]);
            let records = array(&audit["per_assertion"]);
            assert_eq!(records.len(), assertions.len(), "{arm} prompt {ordinal}");
            let mut values = Vec::new();
            for (j, (fact, record)) in assertions.iter().zip(records).enumerate() {
                let source = selected_sources[&(ordinal as u32, j as u32)];
                if let Some(source) = source {
                    assert_eq!(
                        record["source"], source,
                        "{arm} prompt {ordinal} assertion {j}"
                    );
                    let fence = index(&fact["fence"]);
                    let mut expected = array(original_ids)[source as usize..fence].to_vec();
                    expected.push(json!(1));
                    assert_eq!(
                        record["ids_through_eos"],
                        json!(expected),
                        "{arm} prompt {ordinal} assertion {j}"
                    );
                    values.push((word(&fact["fact_span"][0]), source, expected));
                } else {
                    assert!(record.is_null(), "{arm} prompt {ordinal} assertion {j}");
                }
            }
            let disagreement = values.windows(2).any(|pair| pair[0].2 != pair[1].2);
            let consensus = values
                .first()
                .filter(|_| values.len() == assertions.len() && !disagreement);
            assert_eq!(
                trajectory["source"],
                json!(consensus.map(|x| x.1)),
                "{arm} prompt {ordinal}"
            );
            if let Some((_, _, ids)) = consensus {
                assert_eq!(
                    trajectory["generated"],
                    json!(ids),
                    "{arm} prompt {ordinal}"
                );
            }
            assert_eq!(
                paired["choice"]["source"], trajectory["source"],
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                paired["choice"]["fact"],
                json!(consensus.map(|x| x.0)),
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                paired["supporting_facts"],
                assertions.len(),
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                paired["complete_values"],
                values.len(),
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                paired["values_disagree"], disagreement,
                "{arm} prompt {ordinal}"
            );
            assert_eq!(
                array(&trajectory["generated"]).last(),
                Some(&json!(1)),
                "{arm} prompt {ordinal}"
            );
            controlled.push((
                ordinal,
                paired["role_words"].clone(),
                trajectory["source"].clone(),
                trajectory["generated"].clone(),
            ));
        }
        combined.push(controlled);
    }
    let mut altered_sources = 0;
    let mut altered_tapes = 0;
    for (baseline, trial) in combined[0].iter().zip(&combined[1]) {
        assert_eq!(baseline.0, trial.0);
        assert_eq!(
            baseline.1, trial.1,
            "policy rewrote supplied roles on prompt {}",
            baseline.0
        );
        altered_sources += usize::from(baseline.2 != trial.2);
        altered_tapes += usize::from(baseline.3 != trial.3);
    }
    println!("supplied value writer: 96 paired rows with identical roles, {altered_sources} changed sources, {altered_tapes} changed token tapes");
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let expected_rows = args[0].parse().expect("expected row count");
    for name in &args[1..] {
        println!("{name}");
        let root = Path::new(name);
        check_writers(root, expected_rows);
        check_veto(root, expected_rows);
        if root.join("modes/supplied-veto").exists() {
            compare_supplied_writer(root);
        }
        for manifest in ["cold.sha256", "raw.sha256"] {
            println!(
                "{manifest}: {} hashed files verified",
                verify_manifest(root, manifest)
            );
        }
        if let Some(primary) = name.strip_suffix("-repeat") {
            let primary = format!("{primary}-primary");
            let primary = Path::new(&primary);
            assert_eq!(bytes(primary, "cold.sha256"), bytes(root, "cold.sha256"));
            assert_eq!(bytes(primary, "summary.json"), bytes(root, "summary.json"));
            println!("cold repeat equals primary on manifest and all covered bytes");
        }
    }
}
