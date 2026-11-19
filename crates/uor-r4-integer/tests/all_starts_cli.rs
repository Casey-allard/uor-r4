//! Finite protocol-2 replay using a synthetic role carrier and planted onset bank.
#![cfg(feature = "serving-wasm")]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use uor_r4_integer::{
    all_starts::{OnsetMode, OnsetProfile, BANK_BYTES},
    discrete_d11::{
        pack_all_starts_artifact, spiralcore_orbit_roots, IntegerDiscreteModel,
        DISCRETE_ALL_STARTS_SCHEMA, DISCRETE_PAIRED_SCHEMA,
    },
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

fn path(key: &str) -> PathBuf {
    env::var_os(key)
        .unwrap_or_else(|| panic!("missing finite fixture path {key}"))
        .into()
}

fn output(model: &Path, tokenizer: &Path, text: &str, mode: &str) -> Value {
    let process = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--discrete")
        .arg(model)
        .arg("--tokenizer")
        .arg(tokenizer)
        .args([
            "--protocol",
            "2",
            "--temperature",
            "0",
            "--max-tokens",
            "16",
            "--read-mode",
            mode,
            "--say",
            text,
        ])
        .output()
        .unwrap();
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    serde_json::from_slice(&process.stdout).unwrap()
}

fn write_words(out: &mut Vec<u8>, words: impl IntoIterator<Item = u32>) {
    out.extend(words.into_iter().flat_map(u32::to_le_bytes));
}

#[test]
#[ignore = "requires pinned synthetic-onset fixture paths in ONSET_FIXTURE_*"]
fn frozen_role_source_intervention_reaches_normal_text_and_cold_writer() {
    let carrier = path("ONSET_FIXTURE_CARRIER");
    let tokenizer_path = path("ONSET_FIXTURE_TOKENIZER");
    let out_root = path("ONSET_FIXTURE_OUT");
    fs::create_dir(&out_root).unwrap();
    let span_bytes = fs::read(path("ONSET_FIXTURE_SPAN")).unwrap();
    let legacy = fs::read(&carrier).unwrap();
    let old = IntegerDiscreteModel::from_bytes(&legacy).unwrap();
    assert_eq!(old.artifact_schema(), DISCRETE_PAIRED_SCHEMA);
    assert_eq!(old.onset_mode(), None);
    let tok =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path).unwrap()).unwrap();
    let text = "Mira kept blue green. Nora kept red. Please give what Mira collected before closing, rather than Nora beside the entrance.";
    let protocol = DialogueProtocol::literal_roles_v2(&tok).unwrap();
    let ids = protocol
        .bind(&tok)
        .unwrap()
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: text,
        }])
        .tokens;
    let roles = old
        .paired_roles_diagnostic(&ids, 0..ids.len(), false)
        .unwrap();
    let requested = roles.requested.expect("finite planted role must nominate");
    assert_eq!(
        tok.decode(&ids[requested.start..requested.end]).trim(),
        "Mira"
    );
    let (_, assertions) = old.paired_copy_witness(&ids, roles, None).unwrap();
    assert_eq!(assertions.len(), 1);
    let assertion = assertions[0];
    let fact_end = assertion.fact_span[1] - 1;
    let fence = assertion.assertion.fence.unwrap() as usize;
    let masks = assertion.assertion.value;
    let legacy_mask = masks.eligible_mask & masks.word_start_mask;
    assert!(legacy_mask.count_ones() >= 2);

    let span: Value = serde_json::from_slice(&span_bytes).unwrap();
    let classes: Vec<u8> = serde_json::from_value(span["bank"]["classes"].clone()).unwrap();
    let roots = old.packed_codebook_roots();
    let root_bytes: Vec<_> = roots
        .iter()
        .flat_map(|&r| (r as u16).to_le_bytes())
        .collect();
    let mut site_hash = Sha256::new();
    site_hash.update(b"uor-r4/all-starts-sites/1");
    site_hash.update(hex::decode(old.artifact_sha256()).unwrap());
    site_hash.update(Sha256::digest(&root_bytes));
    site_hash.update(Sha256::digest(&classes));
    site_hash.update(Sha256::digest(&span_bytes));
    for anchor in spiralcore_orbit_roots() {
        site_hash.update(anchor.map(|x| x as u8));
    }
    let sites = site_hash.finalize();

    let train: [u8; 32] = Sha256::digest(b"finite synthetic profiles, no fitting").into();
    let labels: [u8; 32] = Sha256::digest(b"known literal offsets, same writer").into();
    let profile = OnsetProfile {
        mode: OnsetMode::Ordered,
        train_sha256: hex::encode(train),
        labels_sha256: hex::encode(labels),
    };
    let mut observations = Vec::new();
    for (arm, text_literal) in [("known-right", "blue green"), ("planted-wrong", "green")] {
        let literal = tok.encode(&format!(" {text_literal}"));
        assert_eq!(ids[fence - literal.len()..fence], literal);
        let target = fence - literal.len();
        let depth = target - fact_end - 1;
        assert!((1..=3).contains(&depth) && (legacy_mask & (1 << (depth - 1))) != 0);
        let mut records = Vec::new();
        for k in 0..3 {
            if legacy_mask & (1 << k) == 0 {
                continue;
            }
            let source = fact_end + 2 + k;
            assert!(fence - source < 8);
            let end = (source + 1..fence)
                .find(|&at| classes[ids[at] as usize] != 2)
                .unwrap_or(fence);
            let mut signature = 0u32;
            let mut packed = [0u32; 10];
            for (i, &id) in ids[fact_end + 1..end].iter().enumerate() {
                signature |= u32::from(classes[id as usize]) << (2 * i);
                packed[i] = roots[id as usize];
            }
            for class in 0..=1 {
                let mut witness = packed;
                if class != u32::from(source != target) {
                    witness[0] ^= 1;
                }
                let shape = (k as u32 + 1) | ((end - source) as u32) << 8;
                let geom = witness.chunks_exact(2).map(|r| r[0] | r[1] << 16);
                records.push(
                    [shape | class << 16, signature, 128]
                        .into_iter()
                        .chain(geom)
                        .collect::<Vec<_>>(),
                );
            }
        }
        let mut raw = Vec::new();
        write_words(&mut raw, [0x31425355, 0, records.len() as u32, 256]);
        for record in records {
            write_words(&mut raw, record);
        }
        raw.resize(16 + 256 * 32, 0);
        let mut bank = Vec::new();
        write_words(&mut bank, [0x32425355, 6, raw.len() as u32, 0]);
        for binding in [
            hex::decode(span["artifact_sha256"].as_str().unwrap()).unwrap(),
            hex::decode(old.tokenizer_sha256().unwrap()).unwrap(),
            sites.to_vec(),
            train.to_vec(),
            labels.to_vec(),
            Sha256::digest(&raw).to_vec(),
        ] {
            bank.extend(binding);
        }
        bank.extend(raw);
        assert_eq!(bank.len(), BANK_BYTES);
        let mut wrong_profile = profile.clone();
        wrong_profile.mode = OnsetMode::Neutral;
        assert!(pack_all_starts_artifact(&legacy, &span_bytes, &bank, &wrong_profile).is_err());
        wrong_profile = profile.clone();
        wrong_profile.labels_sha256 = "1".repeat(64);
        assert!(pack_all_starts_artifact(&legacy, &span_bytes, &bank, &wrong_profile).is_err());
        let mut wrong_span: Value = serde_json::from_slice(&span_bytes).unwrap();
        wrong_span["bank"]["classes"][0] = json!(9);
        assert!(pack_all_starts_artifact(
            &legacy,
            &serde_json::to_vec(&wrong_span).unwrap(),
            &bank,
            &profile
        )
        .is_err());
        let file = pack_all_starts_artifact(&legacy, &span_bytes, &bank, &profile).unwrap();
        let parsed = IntegerDiscreteModel::from_bytes(&file).unwrap();
        assert_eq!(parsed.artifact_schema(), DISCRETE_ALL_STARTS_SCHEMA);
        assert_eq!(parsed.onset_mode(), Some(OnsetMode::Ordered));
        let on_disk = out_root.join(format!("{arm}.d11"));
        fs::write(&on_disk, &file).unwrap();
        if arm == "known-right" {
            let bank_path = out_root.join("known-right.bank");
            let profile_path = out_root.join("profile.json");
            let cli_path = out_root.join("cli-packed.d11");
            fs::write(&bank_path, &bank).unwrap();
            fs::write(&profile_path, serde_json::to_vec(&profile).unwrap()).unwrap();
            let cli = Command::new(env!("CARGO_BIN_EXE_pack-all-starts"))
                .arg(&carrier)
                .arg(path("ONSET_FIXTURE_SPAN"))
                .arg(bank_path)
                .arg(profile_path)
                .arg(&cli_path)
                .output()
                .unwrap();
            assert!(
                cli.status.success(),
                "{}",
                String::from_utf8_lossy(&cli.stderr)
            );
            assert_eq!(fs::read(cli_path).unwrap(), file);
        }
        let first = output(&on_disk, &tokenizer_path, text, "enabled");
        assert_eq!(first, output(&on_disk, &tokenizer_path, text, "enabled"));
        assert_eq!(first["prompt_ids"], json!(ids));
        let expected: Vec<_> = literal.iter().copied().chain([1]).collect();
        assert_eq!(first["selected"]["source"], target);
        assert_eq!(first["generated_ids"], json!(expected));
        assert_eq!(first["stop"], "eos");
        let no_read = output(&on_disk, &tokenizer_path, text, "no_read");
        let legacy_no_read = output(&carrier, &tokenizer_path, text, "no_read");
        assert_eq!(no_read["generated_ids"], legacy_no_read["generated_ids"]);
        assert_eq!(
            no_read["copy_step_words"],
            legacy_no_read["copy_step_words"]
        );
        assert!(no_read["selected"]["source"].is_null());
        let (decision, checked) = parsed.paired_copy_witness(&ids, roles, None).unwrap();
        assert_eq!(decision.choice.source, Some(target as u32));
        assert_eq!(checked[0].onset.unwrap().source, Some(target as u32));
        assert_eq!(
            checked[0].completed_source_range,
            Some([target as u32, fence as u32])
        );
        if arm == "known-right" {
            let mut corrupt = file;
            let header_len = u64::from_le_bytes(corrupt[8..16].try_into().unwrap()) as usize;
            corrupt[(16 + header_len).next_multiple_of(64) + 225] ^= 1;
            assert!(IntegerDiscreteModel::from_bytes(&corrupt).is_err());
        }
        observations.push(json!({"arm":arm,"writer_source":decision.choice.source,"public":first,"no_read":no_read}));
    }
    assert_ne!(
        observations[0]["public"]["generated_ids"][0],
        observations[1]["public"]["generated_ids"][0]
    );
    fs::write(
        out_root.join("result.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
}

fn raw_words(bytes: &[u8]) -> impl Iterator<Item = u32> + '_ {
    assert_eq!(bytes.len() % 4, 0);
    bytes
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
}

#[test]
#[ignore = "requires synthetic-only Wick run-05 receipt in ONSET_SYNTHETIC_CUDA"]
fn product_scorer_matches_every_cold_synthetic_cuda_word() {
    use uor_r4_integer::all_starts::{Bank, CANDIDATE_WORDS, DECISION_WORDS};
    let root = path("ONSET_SYNTHETIC_CUDA");
    let cases = [
        (
            "right-ordered",
            "right-ordered",
            "infer",
            OnsetMode::Ordered,
        ),
        (
            "right-neutral",
            "right-neutral",
            "infer",
            OnsetMode::Neutral,
        ),
        (
            "right-position",
            "right-position",
            "infer",
            OnsetMode::Position,
        ),
        ("wrong", "wrong", "infer", OnsetMode::Ordered),
        ("unseen", "unseen", "infer", OnsetMode::Ordered),
        ("tie", "right-ordered", "tie-infer", OnsetMode::Ordered),
    ];
    let mut compared = 0usize;
    for cold in ["cold-1", "cold-2"] {
        for (trace_name, bank_name, packet_name, mode) in cases {
            let packet =
                fs::read(root.join("fixtures").join(format!("{packet_name}.packet"))).unwrap();
            let rows: Vec<_> = raw_words(&packet).collect();
            assert_eq!(&rows[..4], &[0x33415355, 1025, CANDIDATE_WORDS as u32, 0]);
            assert_eq!(rows.len(), 36 + 1025 * CANDIDATE_WORDS);
            let bytes = fs::read(root.join(cold).join(format!("{bank_name}.bank"))).unwrap();
            let profile = OnsetProfile {
                mode,
                train_sha256: hex::encode(&bytes[112..144]),
                labels_sha256: hex::encode(&bytes[144..176]),
            };
            let bank = Bank::from_profile(
                &bytes,
                packet[16..48].try_into().unwrap(),
                packet[48..80].try_into().unwrap(),
                packet[80..112].try_into().unwrap(),
                &profile,
            )
            .unwrap();
            let cuda = fs::read(root.join(cold).join(format!("{trace_name}.trace"))).unwrap();
            let trace: Vec<_> = raw_words(&cuda).collect();
            assert_eq!(
                &trace[..4],
                &[0x32545355, 1025, DECISION_WORDS as u32, mode as u32]
            );
            assert_eq!(trace.len(), 4 + 1025 * DECISION_WORDS);
            for (ordinal, (row, cooked)) in rows[36..]
                .chunks_exact(CANDIDATE_WORDS)
                .zip(trace[4..].chunks_exact(DECISION_WORDS))
                .enumerate()
            {
                let expected = bank.score_legacy_row(row.try_into().unwrap()).unwrap();
                assert_eq!(&expected, cooked, "{cold}/{trace_name} assertion {ordinal}");
                compared += DECISION_WORDS;
            }
        }
    }
    assert_eq!(compared, 836_400);
    println!(
        "production all_starts::Bank compared {compared} exact words in two cold synthetic stages"
    );
}
