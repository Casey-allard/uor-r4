#![forbid(unsafe_code)]

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, io::Write, path::Path};
use uor_r4_count_runtime::{code_pick, FrozenPickMixture, PickClassRule};
use uor_r4_integer::discrete_d11::count_deployment::{write_deployment, DiscreteCountModel};
use uor_r4_tokenizer::{dialogue::DialogueProtocol, ByteBpeTokenizer};

type Failure = Box<dyn Error>;
const TRUNK: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
const TRAIN: &str = "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a";
const R: &str = "5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a";
const COUNT: &str = "064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8";
const COUNT_BYTES: usize = 3_912_334_481;
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn ensure(ok: bool, message: impl Into<String>) -> Result<(), Failure> {
    if ok {
        Ok(())
    } else {
        Err(message.into().into())
    }
}

fn json_file(path: &Path, value: &Value) -> Result<(), Failure> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    Ok(())
}

fn word(bytes: &[u8]) -> Result<u64, Failure> {
    Ok(u64::from_le_bytes(bytes.try_into()?))
}

fn pack(
    fit: &Path,
    arm: &str,
    trunk_path: &Path,
    count_path: &Path,
    tokenizer: &Path,
    out: &Path,
) -> Result<(), Failure> {
    let rule = match arm {
        "a1" => PickClassRule::Three,
        "a1s" => PickClassRule::Five,
        _ => return Err("unknown fit arm".into()),
    };
    let source = |suffix: &str| fit.join(format!("{arm}.{suffix}"));
    let fit_bytes = fs::read(source("manifest.json"))?;
    let fit_digest = digest(&fit_bytes);
    let fit_manifest: Value = serde_json::from_slice(&fit_bytes)?;
    let prepared = &fit_manifest["prepared"];
    let prep_bytes = fs::read(fit.join("prepare.json"))?;
    let raw_probes = fs::read(fit.join("probes.json"))?;
    let device_ordinals = fs::read(fit.join("probe-ordinals.u64"))?;
    let device_probes = fs::read(source("probe.u64"))?;
    let trace = fs::read(source("trace"))?;
    let weights = fs::read(source("weights"))?;
    let frozen = FrozenPickMixture::from_bytes(&weights, Sha256::digest(&fit_bytes).into())?;
    ensure(
        fit_manifest["schema"] == "uor-r4.d37.r-fit/1"
            && fit_manifest["arm"] == arm
            && fit_manifest["rows"] == 15_947_584
            && fit_manifest["passes"] == 96
            && fit_manifest["pick_classes"] == rule.classes()
            && fit_manifest["weight_format"] == frozen.wire_tag()
            && fit_manifest["q32_weight_words"] == frozen.weights().len()
            && frozen.rule() == rule
            && prepared["format"] == "uor-r4.d37.r-preparation/1"
            && prepared["pick_mode"] == "identity"
            && prepared["c_sha256"] == TRAIN
            && prepared["r_sha256"] == R
            && prepared["trunk_sha256"] == TRUNK
            && prepared["compact_sha256"] == COUNT
            && prepared == &serde_json::from_slice::<Value>(&prep_bytes)?
            && fit_manifest["preparation_sha256"] == digest(&prep_bytes)
            && fit_manifest["cuda_trace_sha256"] == digest(&trace)
            && fit_manifest["cuda_probes_sha256"] == digest(&device_probes)
            && prepared["probes_sha256"] == digest(&raw_probes)
            && prepared["probe_ordinals_sha256"] == digest(&device_ordinals),
        "fit, preparation or CUDA identity mismatch",
    )?;
    let stride = 8 * (frozen.weights().len() + 1);
    ensure(
        trace.len() == 24 + 97 * stride
            && &trace[..8] == b"R37TR001"
            && word(&trace[8..16])? == 15_947_584
            && word(&trace[16..24])? == u64::from(rule.classes()) * 176,
        "incorrect CUDA trace carrier or shape",
    )?;
    let final_bytes = &trace[trace.len() - stride..];
    ensure(
        fit_manifest["final_fit_bits_q12"] == word(&final_bytes[..8])?
            && final_bytes[8..]
                .chunks_exact(8)
                .zip(frozen.weights())
                .all(|(bytes, &weight)| word(bytes).is_ok_and(|value| value == weight)),
        "weights differ from final CUDA state",
    )?;
    let raw: Vec<Value> = serde_json::from_slice(&raw_probes)?;
    let scored: Vec<Value> = serde_json::from_slice(&fs::read(source("probes-scored.json"))?)?;
    ensure(
        (64..=1024).contains(&raw.len())
            && raw.len() == scored.len()
            && raw.len() * 8 == device_ordinals.len()
            && raw.len() * 16 == device_probes.len()
            && fit_manifest["cuda_probe_count"] == raw.len(),
        "invalid ordered probe tape length",
    )?;

    let trunk = fs::read(trunk_path)?;
    ensure(digest(&trunk) == TRUNK, "wrong ordinary D11 trunk")?;
    ensure(
        fs::metadata(count_path)?.len() == COUNT_BYTES as u64,
        "wrong count byte length",
    )?;
    let mut state = Vec::with_capacity(48 + fit_bytes.len() + weights.len() + COUNT_BYTES);
    state.extend(b"UORP1A01");
    state.extend(u32::try_from(fit_bytes.len())?.to_le_bytes());
    state.extend(u32::try_from(weights.len())?.to_le_bytes());
    state.extend(Sha256::digest(&weights));
    state.extend(&fit_bytes);
    state.extend(&weights);
    std::io::copy(&mut fs::File::open(count_path)?, &mut state)?;
    let d11 = out.join("artifact.d11");
    let state_len = state.len();
    let state_digest = digest(&state);
    write_deployment(&trunk, &state, &d11)?;
    drop(state);

    let bytes = fs::read(d11)?;
    let deployment = DiscreteCountModel::load(&bytes, None)?;
    let identity = deployment
        .identity
        .ok_or("missing embedded count identity")?;
    let engine = deployment.state.ok_or("missing embedded count state")?;
    let components = json!({ "count_sha256": COUNT, "weights_sha256": digest(&weights),
        "fit_manifest_sha256": fit_digest });
    ensure(
        identity.trunk_bytes == trunk.len()
            && bytes[identity.header_bytes..identity.header_bytes + identity.trunk_bytes] == trunk
            && identity.state_bytes == state_len
            && engine.artifact_sha256 == state_digest
            && engine.component_digests() == components,
        "ordinary reader changed embedded component identity",
    )?;
    let codes = deployment.model.identity_pick_codes()?;
    let mut expected = Vec::with_capacity(raw.len());
    let mut changed_distance = None;
    let mut previous = None;
    for (index, (probe, sealed)) in raw.iter().zip(&scored).enumerate() {
        let ordinal = word(&device_ordinals[index * 8..][..8])?;
        let cuda_row = &device_probes[index * 16..][..16];
        let cuda_bucket = word(&cuda_row[..8])?;
        let cuda_mass = word(&cuda_row[8..])?;
        let mut expected_seal = probe.clone();
        expected_seal["bucket"] = json!(cuda_bucket);
        expected_seal["mixed_q48"] = json!(cuda_mass);
        ensure(
            &expected_seal == sealed
                && probe["ordinal"] == ordinal
                && previous.is_none_or(|last| last < ordinal),
            format!("sealed CUDA probe provenance differs at ordinal {ordinal}"),
        )?;
        previous = Some(ordinal);
        let source_position: u64 = serde_json::from_value(probe["position"].clone())?;
        // The absolute UORT word index includes BOS. The scored ordinal does not.
        let record = source_position
            .checked_sub(ordinal)
            .filter(|record| (1..=4_060).contains(record))
            .ok_or("invalid source record index")?;
        let prefix: Vec<u16> = serde_json::from_value(probe["prefix"].clone())?;
        let target: u16 = serde_json::from_value(probe["target"].clone())?;
        let picked = code_pick(&codes, &prefix);
        let choice = picked.map(|(position, distance, _)| (prefix[position + 1], distance));
        let class = rule.classify(choice.map(|(_, distance)| distance));
        ensure(
            probe["distance"] == json!(choice.map(|(_, distance)| distance))
                && probe["successor"] == json!(choice.map(|(successor, _)| successor))
                && probe["predecessor"] == json!(picked.map(|(position, _, _)| position))
                && probe["hit"] == json!(choice.is_some_and(|(successor, _)| successor == target))
                && cuda_bucket / 176 == u64::from(class),
            format!("native Pick provenance differs at ordinal {ordinal}"),
        )?;
        let diagnostic = deployment
            .model
            .observe_identity_pick(&prefix.iter().copied().map(u32::from).collect::<Vec<_>>())?;
        ensure(
            probe["blocks"] == json!(diagnostic.block_words),
            format!("GPU and native route words differ at ordinal {ordinal}"),
        )?;

        let mut masses = Sha256::new();
        let mut width = 0;
        let mut target_words = None;
        let (mut winner, mut bucket, mut maximum, mut ties) = (0, 0, 0, 0);
        engine
            .counts()
            .visit_next_masses(&prefix, &frozen, choice, |id, route, mass| {
                width += 1;
                masses.update(mass.to_le_bytes());
                if id == target {
                    target_words = Some((u64::from(route), mass));
                }
                if mass > maximum {
                    (winner, bucket, maximum, ties) = (id, route, mass, 1);
                } else if mass == maximum {
                    ties += 1;
                }
            })?;
        ensure(
            width == 4095 && target_words == Some((cuda_bucket, cuda_mass)),
            format!("complete native vector differs at CUDA target ordinal {ordinal}"),
        )?;
        let mut prefix_hash = Sha256::new();
        for token in &prefix {
            prefix_hash.update(token.to_le_bytes());
        }
        expected.push(json!({
            "ordinal": ordinal, "source_position": source_position, "record": record,
            "prefix": prefix, "read_class": class,
            "target": target, "cuda_bucket": cuda_bucket, "cuda_mass_q48": cuda_mass,
            "step": {
                "token": winner, "bucket": bucket, "mass_q48": maximum, "top_ties": ties,
                "mass_sha256": format!("{:x}", masses.finalize()),
                "prefix_sha256": format!("{:x}", prefix_hash.finalize()),
                "context_tokens": prefix.len(),
                "pick_successor": choice.map(|(id, _)| id),
                "pick_words": picked.map(|(p, d, t)| [p as u32, d, t]),
            },
        }));

        if let Some((successor, distance)) = choice.filter(|_| changed_distance.is_none()) {
            let changed = u32::from(distance == 0);
            let changed_class = rule.classify(Some(changed));
            let mut modified = None;
            engine.counts().visit_next_masses(
                &prefix,
                &frozen,
                Some((successor, changed)),
                |id, bucket, mass| {
                    if id == target {
                        modified = Some((bucket, mass));
                    }
                },
            )?;
            let (modified_bucket, modified_mass) =
                modified.ok_or("missing changed-distance target")?;
            ensure(
                changed_class != class && u64::from(modified_bucket) != cuda_bucket,
                "changed-distance negative failed to change the raw bucket",
            )?;
            changed_distance = Some(json!({"ordinal": ordinal, "original_distance": distance,
                "injected_distance": changed, "original_bucket": cuda_bucket,
                "injected_bucket": modified_bucket, "original_mass_q48": cuda_mass,
                "injected_mass_q48": modified_mass,
                "injected_result_matches_cuda": false}));
        }
    }
    let negative = changed_distance.ok_or("no fired probe for changed-distance negative")?;
    ensure(
        expected.iter().any(|p| p["target"] == 1),
        "no original EOS target probe",
    )?;
    let tokenizer_len = fs::metadata(tokenizer)?.len();
    let receipt = json!({
        "schema": "uor-r4.d37-ordinary-client-readback/1",
        "arm": arm, "artifact_sha256": identity.artifact_sha256,
        "trunk_sha256": TRUNK, "state_sha256": state_digest, "components": components,
        "cuda_probes_sha256": digest(&device_probes), "cuda_ordinals_sha256": digest(&device_ordinals),
        "tokenizer_sha256": digest(&fs::read(tokenizer)?),
        "bytes": {
            "count": COUNT_BYTES, "manifest": fit_bytes.len(), "weights": weights.len(),
            "p1_header": 48, "p1_state": identity.state_bytes,
            "d11_header": identity.header_bytes, "trunk": identity.trunk_bytes,
            "d11": bytes.len(), "tokenizer": tokenizer_len,
            "deployment": bytes.len() as u64 + tokenizer_len,
        },
        "negative": negative, "probes": expected,
    });
    json_file(&out.join("readback.json"), &receipt)
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let [command, tokenizer, answer] = args.as_slice() {
        ensure(command == "gold", "expected gold TOKENIZER ANSWER")?;
        let bytes = fs::read(tokenizer)?;
        ensure(digest(&bytes) == TOKENIZER, "gold tokenizer identity differs")?;
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
            .ok_or("invalid gold tokenizer")?;
        let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
        let answer = answer.to_str().ok_or("non-UTF8 gold answer")?;
        ensure(!answer.is_empty() && !answer.chars().any(char::is_whitespace),
            "gold must contain exactly one nonempty word")?;
        let text = format!(" {answer}");
        let mut ids = tokenizer.encode(&text);
        ensure(!ids.is_empty() && tokenizer.decode(&ids) == text, "gold token round trip differs")?;
        ids.push(protocol.eos_id);
        println!("{}", json!({"ids": ids, "eos": protocol.eos_id, "tokenizer_sha256": TOKENIZER}));
        return Ok(());
    }
    let [fit, arm, trunk, counts, tokenizer, out] = args.as_slice() else {
        return Err("usage: d37-client-check FIT ARM TRUNK COUNTS TOKENIZER NEW-ARM-OUTPUT | gold TOKENIZER ANSWER".into());
    };
    let out = Path::new(out);
    fs::create_dir(out)?;
    pack(
        Path::new(fit),
        arm.to_str().ok_or("non-UTF8 arm")?,
        Path::new(trunk),
        Path::new(counts),
        Path::new(tokenizer),
        out,
    )
}
