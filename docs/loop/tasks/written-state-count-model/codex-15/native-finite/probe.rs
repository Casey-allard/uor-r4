use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Instant,
};
use uor_r4_integer::{
    discrete_d11::{
        pack_ordered_role_artifact, ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS,
    },
    ordered_role::{RoleBank, RoleFraming, RoleMode, RolePrototypes},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const TEXT: &str = "Aster kept cobalt. Dahlia kept ivory. Which item belongs to Aster?";
const TRAIN: [u8; 32] = [13; 32];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn save(path: &Path, data: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(data)?;
    fs::write(path, &bytes)?;
    Ok(digest(&bytes))
}
fn load(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn ok(child: Output) -> Result<Value> {
    if !child.status.success() {
        return Err(String::from_utf8(child.stderr)?.into());
    }
    Ok(serde_json::from_slice(&child.stdout)?)
}
fn asset(root: &Path, seed: u8, name: &str) -> PathBuf {
    root.join("assets").join(format!("{name}-seed{seed}.json"))
}

fn prepare(root: &Path, area: &Path, trunk: &Path, seed: u8) -> Result<()> {
    let token = fs::read(asset(root, seed, "tokenizer"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&token).ok_or("bad tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoded = protocol
        .bind(&tokenizer)?
        .encode_assistant_prefix(&[Message {
            role: "user",
            content: TEXT,
        }]);
    let prefix = encoded.tokens;
    let framing = RoleFraming::from_tokenizer(&tokenizer)?;
    let content = framing.content(&prefix)?;
    let source_ids = tokenizer.encode(" cobalt");
    let source = prefix
        .windows(source_ids.len() + 1)
        .position(|w| w[..source_ids.len()] == source_ids && w.last() == Some(&16))
        .ok_or("finite copy source is missing")? as u32;
    let legacy = fs::read(root.join("assets").join(format!("seed{seed}-paired.d11")))?;
    let start = Instant::now();
    let parent = IntegerDiscreteModel::from_bytes(&legacy)?;
    let original_load_ns = start.elapsed().as_nanos();
    let bindings = parent.ordered_role_bindings(&framing, TRAIN)?;
    let model = fs::read(trunk)?;
    assert_eq!(Sha256::digest(&model).as_slice(), bindings[0]);
    let model_digest = digest(&model);
    let prefix_hash = digest(
        &prefix
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect::<Vec<_>>(),
    );
    let mut receipts = Vec::new();
    for mode in [RoleMode::Identity, RoleMode::Positional] {
        let bank = RoleBank::new(mode, RolePrototypes::default(), bindings);
        let start = Instant::now();
        let artifact = pack_ordered_role_artifact(&legacy, &token, &bank, TRAIN)?;
        let pack_and_verify_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let reloaded = IntegerDiscreteModel::from_bytes(&artifact)?;
        let reload_ns = start.elapsed().as_nanos();
        assert_eq!(reloaded.ordered_role_bank(), Some(&bank));
        assert_eq!(reloaded.ordered_role_framing(), Some(&framing));
        let raw_len = u64::from_le_bytes(artifact[8..16].try_into()?) as usize;
        let raw_start = (16 + raw_len).next_multiple_of(64);
        assert_eq!(&artifact[raw_start..raw_start + 400], bank.to_bytes());
        assert_eq!(&artifact[raw_start + 400 + 4096..], model);
        let path = area.join(format!("finite-s{seed}-{}.d11", mode as u8));
        fs::write(&path, &artifact)?;
        receipts.push(json!({
            "mode":mode, "artifact_sha256":digest(&artifact), "record_sha256":digest(&bank.to_bytes()),
            "artifact_bytes":artifact.len(), "carrier_padding_bytes":raw_start - 16 - raw_len,
            "container_prefix_bytes":raw_start, "record_bytes":400, "learned_bytes":192,
            "packed_classes_bytes":4096, "pack_and_verify_ns":pack_and_verify_ns, "from_bytes_ns":reload_ns,
        }));
        let mut corrupt = artifact;
        corrupt[raw_start + 400 + 7] ^= 1;
        assert!(IntegerDiscreteModel::from_bytes(&corrupt).is_err());
    }
    assert_eq!(receipts[0]["artifact_bytes"], receipts[1]["artifact_bytes"]);
    for size in [1, 32] {
        let base = area.join(format!("s{seed}-{size}"));
        fs::create_dir(&base)?;
        let prompts = vec![json!({"prefix":prefix}); size];
        let bare_digest = save(&base.join("bare.json"), &json!(prompts))?;
        let rows: Vec<_> = (0..size)
            .map(|i| {
                let source = (i % 2 == 0).then_some(source);
                json!({
                    "prefix_sha256":prefix_hash,
                    "continuation":{"source":source,"reason":0,"connector_mask":0,"onset_mask":0}
                })
            })
            .collect();
        let picks_digest = save(
            &base.join("picks.json"),
            &json!({
                "schema":"uor-r4.entity-pair-chain/1", "arm":"action",
                "artifact_sha256":model_digest,"bare_sha256":bare_digest,"rows":rows,
            }),
        )?;
        save(
            &base.join("manifest.json"),
            &json!([{"model":model_digest,"bare":bare_digest,"picks":picks_digest}]),
        )?;
    }
    save(
        &area.join(format!("s{seed}-pack.json")),
        &json!({
            "seed":seed, "raw_text":TEXT, "content":[content.start,content.end], "prefix":prefix,
            "expected_source_ids":source_ids,"source":source,"trunk_sha256":model_digest,
            "tokenizer_sha256":digest(&token),"original_from_bytes_ns":original_load_ns,"arms":receipts
        }),
    )?;
    Ok(())
}

fn public(root: &Path, area: &Path, chat: &Path, seed: u8, mode: u8, size: usize) -> Result<()> {
    let token = asset(root, seed, "tokenizer");
    let model = area.join(format!("finite-s{seed}-{mode}.d11"));
    let ref_row = load(&area.join(format!("s{seed}-pack.json")))?;
    let mut observations = Vec::new();
    let mut run_ns = Vec::new();
    let run = |trace: bool, read: &str| -> Result<Value> {
        let mut child = Command::new(chat);
        child
            .arg("--discrete")
            .arg(&model)
            .arg("--tokenizer")
            .arg(&token)
            .args([
                "--protocol",
                "2",
                "--max-tokens",
                "16",
                "--temperature",
                "0",
                "--read-mode",
                read,
                "--say",
                TEXT,
            ]);
        if trace {
            child.arg("--ordered-role-trace");
        }
        ok(child.output()?)
    };
    for _ in 0..size {
        let time = Instant::now();
        let record = run(false, "enabled")?;
        run_ns.push(time.elapsed().as_nanos());
        assert_eq!(record["prompt_ids"], ref_row["prefix"]);
        assert_eq!(record["selected"]["source"], Value::Null);
        observations.push(record);
    }
    assert!(observations.windows(2).all(|w| w[0] == w[1]));
    let first = &observations[0];
    let mut debug = run(true, "enabled")?;
    let detail = debug
        .as_object_mut()
        .ok_or("missing public JSON record")?
        .remove("ordered_role_trace")
        .ok_or("missing opt-in trace")?;
    assert_eq!(debug, *first);
    let no_read = run(false, "no_read")?;
    assert_eq!(first["generated_ids"], no_read["generated_ids"]);
    assert_eq!(first["copy_step_words"], no_read["copy_step_words"]);
    save(
        &area.join(format!("s{seed}-m{mode}-public-{size}.json")),
        &json!({
            "invocation_ns":run_ns, "sum_invocation_ns":run_ns.iter().sum::<u128>(),
            "rows":size, "canonical":first, "opt_in":detail, "no_read":no_read
        }),
    )?;
    Ok(())
}

fn verify(area: &Path, seed: u8, size: usize) -> Result<()> {
    let ref_row = load(&area.join(format!("s{seed}-pack.json")))?;
    let prefix: Vec<u32> = serde_json::from_value(ref_row["prefix"].clone())?;
    let model = IntegerDiscreteModel::load(&area.join(format!("finite-s{seed}-1.d11")))?;
    let dir = area.join(format!("s{seed}-{size}"));
    let output = load(&dir.join("writer.json"))?;
    let rows = output["rows"]
        .as_array()
        .ok_or("missing historical writer rows")?;
    assert_eq!(rows.len(), size);
    for (i, row) in rows.iter().enumerate() {
        let source: Option<u32> = serde_json::from_value(row["source"].clone())?;
        let ids: Vec<u32> = serde_json::from_value(row["generated"].clone())?;
        let steps = row["steps"]
            .as_array()
            .ok_or("missing historical writer steps")?;
        assert_eq!(ids.len(), steps.len());
        let mut cursor = ExactCopyState::new(source, prefix.len() as u32);
        let mut context = prefix.clone();
        for (&expected, prior) in ids.iter().zip(steps) {
            let ordinary = model.evaluate_prefix(&context)?.prediction;
            let step = cursor
                .write(&prefix, ordinary, ROLE_COPY_STOPS)?
                .finish_with_eos(true, 1);
            assert_eq!(step.emitted, expected);
            assert_eq!(prior["native_id"], json!(ordinary));
            assert_eq!(prior["release"], json!(step.release));
            assert_eq!(prior["after"], json!(step.state.words()));
            assert_eq!(prior["copied"], json!(step.copied));
            cursor = step.state;
            context.push(expected);
        }
        if i % 2 == 0 {
            let mut exact: Vec<u32> =
                serde_json::from_value(ref_row["expected_source_ids"].clone())?;
            exact.push(1);
            assert_eq!(ids, exact);
        } else {
            let public = load(&area.join(format!("s{seed}-m1-public-{size}.json")))?;
            assert_eq!(json!(ids), public["canonical"]["generated_ids"]);
        }
    }
    save(
        &dir.join("parity.json"),
        &json!({"rows":size,"copy_rows":size.div_ceil(2),"fallback_rows":size/2,"parity":"exact"}),
    )?;
    Ok(())
}

// Manually instantiate one finite bank with a prototype at the fixture target.
// No update, corpus pass or held prompt is involved.
fn positive(root: &Path, area: &Path, chat: &Path, seed: u8) -> Result<()> {
    let ref_row = load(&area.join(format!("s{seed}-pack.json")))?;
    let prefix: Vec<u32> = serde_json::from_value(ref_row["prefix"].clone())?;
    let expected_source: u32 = serde_json::from_value(ref_row["source"].clone())?;
    let mut expected: Vec<u32> = serde_json::from_value(ref_row["expected_source_ids"].clone())?;
    expected.push(1);
    let token = asset(root, seed, "tokenizer");
    let token_bytes = fs::read(&token)?;
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&token_bytes).ok_or("bad tokenizer")?;
    let framing = RoleFraming::from_tokenizer(&tokenizer)?;
    let bytes = fs::read(root.join("assets").join(format!("seed{seed}-paired.d11")))?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)?;
    let sites = model.ordered_role_sites()?;
    let prompt = sites.prompt(&prefix, framing.content(&prefix)?)?;
    let requested = prompt
        .candidates()
        .filter(|span| tokenizer.decode(&prefix[span.start..span.end]).trim() == "Aster")
        .last()
        .ok_or("requested finite word failed to enumerate")?;
    let bindings = model.ordered_role_bindings(&framing, TRAIN)?;
    let extreme = core::array::from_fn(|axis| if axis % 2 == 0 { 48 } else { -48 });
    for mode in [RoleMode::Identity, RoleMode::Positional] {
        let feature = prompt.feature(mode, requested, 0..prefix.len())?;
        let prototypes = RolePrototypes::new([feature, extreme, extreme])?;
        let bank = RoleBank::new(mode, prototypes, bindings);
        let path = area.join(format!("positive-s{seed}-{}.d11", mode as u8));
        fs::write(
            &path,
            pack_ordered_role_artifact(&bytes, &token_bytes, &bank, TRAIN)?,
        )?;
        let reloaded = IntegerDiscreteModel::load(&path)?;
        let decision = reloaded
            .paired_copy_decision(&prefix, true)?
            .ok_or("missing role decision")?;
        assert_eq!(decision.choice.source, Some(expected_source));
        let record = ok(Command::new(chat)
            .arg("--discrete")
            .arg(&path)
            .arg("--tokenizer")
            .arg(&token)
            .args([
                "--protocol",
                "2",
                "--max-tokens",
                "16",
                "--temperature",
                "0",
                "--read-mode",
                "enabled",
                "--say",
                TEXT,
                "--ordered-role-trace",
            ])
            .output()?)?;
        assert_eq!(record["generated_ids"], json!(expected));
        assert_eq!(record["stop"], "eos");
        assert_eq!(record["selected"]["source"], expected_source);
        assert_eq!(record["paired"], serde_json::to_value(decision)?);
        let (_, direct) = reloaded.paired_copy_witness(
            &prefix,
            prompt.predict(mode, &bank.prototypes, 0..prefix.len())?,
            None,
        )?;
        assert_eq!(
            record["ordered_role_trace"]["assertions"],
            serde_json::to_value(direct)?
        );
        save(
            &area.join(format!("positive-s{seed}-{}.json", mode as u8)),
            &record,
        )?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, root, area, trunk, seed] if cmd == "prepare" => prepare(
            Path::new(root),
            Path::new(area),
            Path::new(trunk),
            seed.parse()?,
        ),
        [cmd, root, area, chat, seed, mode, size] if cmd == "public" => public(
            Path::new(root),
            Path::new(area),
            Path::new(chat),
            seed.parse()?,
            mode.parse()?,
            size.parse()?,
        ),
        [cmd, root, area, chat, seed] if cmd == "positive" => positive(
            Path::new(root),
            Path::new(area),
            Path::new(chat),
            seed.parse()?,
        ),
        [cmd, area, seed, size] if cmd == "verify" => {
            verify(Path::new(area), seed.parse()?, size.parse()?)
        }
        _ => Err("finite runner arguments".into()),
    }
}
