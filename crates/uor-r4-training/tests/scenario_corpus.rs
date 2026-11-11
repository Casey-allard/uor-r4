use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};
use uor_r4_training::scenario_corpus::{
    check_corpus, generate_scenarios, manifest_path, write_corpus, ScenarioCorpusConfig,
    ScenarioSplit,
};

const SPLITS: [ScenarioSplit; 3] = [
    ScenarioSplit::Train,
    ScenarioSplit::Development,
    ScenarioSplit::Fresh,
];

fn root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "uor-scenario-corpus-{name}-{}-{nonce}",
        std::process::id()
    ))
}

fn byte_char(byte: u8) -> char {
    if (b'!'..=b'~').contains(&byte)
        || (0xA1..=0xAC).contains(&byte)
        || (0xAE..=0xFF).contains(&byte)
    {
        return char::from(byte);
    }
    let mut extra = 0u32;
    for candidate in 0u8..byte {
        if !((b'!'..=b'~').contains(&candidate)
            || (0xA1..=0xAC).contains(&candidate)
            || (0xAE..=0xFF).contains(&candidate))
        {
            extra += 1;
        }
    }
    char::from_u32(256 + extra).expect("byte-level codepoint")
}

fn write_tokenizer(path: &Path) {
    let mut vocab = Map::new();
    vocab.insert("<|bos|>".to_owned(), json!(0));
    vocab.insert("<|eos|>".to_owned(), json!(1));
    vocab.insert("<|unk|>".to_owned(), json!(2));
    for byte in 0u8..=255 {
        vocab.insert(byte_char(byte).to_string(), json!(usize::from(byte) + 3));
    }
    let value = json!({
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [
            {"id": 0, "content": "<|bos|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true},
            {"id": 1, "content": "<|eos|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true},
            {"id": 2, "content": "<|unk|>", "single_word": false, "lstrip": false, "rstrip": false, "normalized": false, "special": true}
        ],
        "normalizer": null,
        "pre_tokenizer": {
            "type": "ByteLevel",
            "add_prefix_space": false,
            "trim_offsets": true,
            "use_regex": true
        },
        "post_processor": null,
        "decoder": {
            "type": "ByteLevel",
            "add_prefix_space": true,
            "trim_offsets": true,
            "use_regex": true
        },
        "model": {
            "type": "BPE",
            "dropout": null,
            "unk_token": "<|unk|>",
            "continuing_subword_prefix": null,
            "end_of_word_suffix": null,
            "fuse_unk": false,
            "byte_fallback": false,
            "ignore_merges": false,
            "vocab": Value::Object(vocab),
            "merges": []
        }
    });
    fs::write(path, serde_json::to_vec(&value).expect("tokenizer JSON")).expect("write tokenizer");
}

fn config() -> ScenarioCorpusConfig {
    ScenarioCorpusConfig {
        seed: 17,
        train: 64,
        development: 64,
        fresh: 64,
        context: 512,
    }
}

#[test]
fn scenarios_are_deterministic_aligned_and_split_disjoint() {
    let config = config();
    let first = generate_scenarios(&config).expect("rows");
    let second = generate_scenarios(&config).expect("repeat");
    assert_eq!(first, second);

    let mut names = BTreeSet::new();
    let mut values = BTreeSet::new();
    let mut prompts = BTreeSet::new();
    let mut families = BTreeSet::new();
    for row in &first {
        assert!(names.insert((row.split, row.entity.clone())));
        families.insert((row.split, row.question_family.clone()));
        assert_eq!(
            row.turns.last().map(|turn| turn.role.as_str()),
            Some("assistant")
        );
        assert!(row.expected_eos);
        if let Some(value) = &row.value {
            assert!(values.insert((row.split, value.clone())));
            assert!(row.turns.iter().any(|turn| turn.content.contains(value)));
        }
        for turn in &row.turns {
            if turn.role == "user" {
                prompts.insert((row.split, turn.content.clone()));
            }
        }
    }
    for pair in [
        (ScenarioSplit::Train, ScenarioSplit::Development),
        (ScenarioSplit::Train, ScenarioSplit::Fresh),
        (ScenarioSplit::Development, ScenarioSplit::Fresh),
    ] {
        let left_names = names
            .iter()
            .filter(|(split, _)| *split == pair.0)
            .map(|(_, name)| name)
            .collect::<BTreeSet<_>>();
        let right_names = names
            .iter()
            .filter(|(split, _)| *split == pair.1)
            .map(|(_, name)| name)
            .collect::<BTreeSet<_>>();
        assert!(left_names.is_disjoint(&right_names));
        assert!(!prompts.iter().any(|(split, text)| {
            *split == pair.0
                && prompts
                    .iter()
                    .any(|(other, other_text)| *other == pair.1 && text == other_text)
        }));
    }
    assert_eq!(
        families
            .iter()
            .filter(|(split, _)| *split == ScenarioSplit::Train)
            .count(),
        3
    );
    assert_eq!(
        families
            .iter()
            .filter(|(split, _)| *split == ScenarioSplit::Development)
            .count(),
        2
    );
    assert_eq!(
        families
            .iter()
            .filter(|(split, _)| *split == ScenarioSplit::Fresh)
            .count(),
        2
    );
}

#[test]
fn every_generated_scenario_has_question_and_eos() {
    let rows = generate_scenarios(&config()).expect("rows");
    for split in SPLITS {
        let split_rows = rows
            .iter()
            .filter(|row| row.split == split)
            .collect::<Vec<_>>();
        let questions = split_rows.iter().filter(|row| row.scored_question).count();
        assert!(questions * 1000 / split_rows.len() >= 950);
        assert!(split_rows
            .iter()
            .all(|row| row.turns.last().map(|turn| turn.role.as_str()) == Some("assistant")));
    }
}

#[test]
fn corpus_round_trips_and_check_refuses_mutations() {
    let root = root("roundtrip");
    let tokenizer_path = root.join("tokenizer.json");
    fs::create_dir_all(&root).expect("root");
    write_tokenizer(&tokenizer_path);
    let corpus = root.join("corpus");
    write_corpus(&config(), &tokenizer_path, &corpus).expect("write corpus");
    check_corpus(&config(), &tokenizer_path, &corpus).expect("clean check");
    let manifest: Value =
        serde_json::from_slice(&fs::read(manifest_path(&corpus)).expect("manifest"))
            .expect("manifest JSON");
    assert_eq!(manifest["splits"].as_object().expect("splits").len(), 3);
    let tokenizer_sha256 =
        uor_r4_training::sha256_file(&tokenizer_path).expect("tokenizer SHA-256");
    for split in ["train", "development", "fresh"] {
        let split_manifest: Value = serde_json::from_slice(
            &fs::read(corpus.join(split).join("manifest.json")).expect("split manifest"),
        )
        .expect("split manifest JSON");
        assert_eq!(
            split_manifest["tokenizer"]["path"].as_str(),
            tokenizer_path.to_str()
        );
        assert_eq!(
            split_manifest["tokenizer"]["sha256"].as_str(),
            Some(tokenizer_sha256.as_str())
        );
    }

    for relative in [
        "train/tokens.u16",
        "train/response_mask.u8",
        "train/manifest.json",
        "train/scenarios.jsonl",
        "development/manifest.json",
        "fresh/scenarios.jsonl",
        "scenario-manifest.json",
    ] {
        let path = corpus.join(relative);
        let original = fs::read(&path).expect("original file");
        let mut changed = original.clone();
        changed.push(b'!');
        fs::write(&path, changed).expect("mutate file");
        let error = check_corpus(&config(), &tokenizer_path, &corpus).expect_err(relative);
        assert!(
            error.contains(relative),
            "unexpected error for {relative}: {error}"
        );
        fs::write(&path, original).expect("restore file");
    }
    let missing = corpus.join("fresh/manifest.json");
    let original = fs::read(&missing).expect("fresh manifest");
    fs::remove_file(&missing).expect("remove split file");
    let error = check_corpus(&config(), &tokenizer_path, &corpus).expect_err("missing split file");
    assert!(error.contains("fresh/manifest.json"));
    fs::write(&missing, original).expect("restore missing file");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn generator_accepts_only_small_safe_configs() {
    let mut config = config();
    config.context = 63;
    assert!(generate_scenarios(&config).is_err());
    config.context = 256;
    config.train = 31;
    assert!(generate_scenarios(&config).is_err());
}
