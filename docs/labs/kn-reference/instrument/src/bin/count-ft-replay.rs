//! Paired ft/dev top-1 on the exact M1 rows using a reloaded native count model and CUDA tape.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};
use uor_r4_integer::{discrete_d11::IntegerDiscreteModel, lawful_likelihood::bits_q12};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const N: usize = 42_856;
const ROWS: usize = 14_287;
const ROW_DIGEST: &str = "d933c533b8b95b4ed6cb54cf96fe83c25a221e3e3226f9a212f52c7bfc1a7074";
const REF_DIGEST: &str = "9c88ee8d20518a0063f1a8eabb0b37ed84861d52cce561604093404840c8caac";
const TOKEN_DIGEST: &str = "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
const MASK_DIGEST: &str = "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn field(row: &Value, name: &str) -> Result<usize> {
    Ok(usize::try_from(
        row[name].as_u64().ok_or("missing unsigned row field")?,
    )?)
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [path, database, state, gpu] = args.as_slice() else {
        return Err("count-ft-replay MODEL.d11 FIXTURE_DIR RELOADED.words STRIDE1.gpu".into());
    };
    let dir = Path::new(database);
    let file = fs::read(dir.join("tokens.u16"))?;
    let reference = fs::read(dir.join("baseline.jsonl"))?;
    let mask = fs::read(dir.join("response_mask.u8"))?;
    let metadata: Value = serde_json::from_slice(&fs::read(dir.join("rows.json"))?)?;
    if metadata["rows_sha256"] != ROW_DIGEST
        || metadata["rows"] != ROWS
        || hash(&reference) != REF_DIGEST
        || hash(&file) != TOKEN_DIGEST
        || hash(&mask) != MASK_DIGEST
        || mask.len() != N
        || mask.iter().filter(|&&mark| mark == 1).count() != ROWS
        || file.len() != 64 + N * 2
        || file[64..] != fs::read(dir.join("ft.raw.u16"))?
    {
        return Err("ft/dev or baseline identity mismatch".into());
    }
    let ids: Vec<_> = file[64..]
        .chunks_exact(2)
        .map(|r| u16::from_le_bytes([r[0], r[1]]))
        .collect();
    let model = IntegerDiscreteModel::load(path.as_ref())?;
    let count = model.count_mixer().ok_or("missing D11 count state")?;
    if count.words() != fs::read(state)? {
        return Err("device input differs from D11 native state".into());
    }
    let artifact_bytes = fs::metadata(path)?.len();
    let tape = fs::read(gpu)?;
    if ids.len() != N || tape.len() != N * (13 + 48) {
        return Err("incorrect stride-one GPU tape dimensions".into());
    }
    let words = |b: &[u8], i: usize| u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
    let bits = &tape[N * 8..N * 12];
    let greedy = &tape[N * 13..];
    let mut by_prefix = BTreeMap::new();
    let mut canon = Sha256::new();
    let (mut hits, mut base_hits, mut gains, mut losses) = (0u32, 0u32, 0u32, 0u32);
    let mut last = None;
    let mut examples = vec![];
    let rows = metadata["row_metadata"]
        .as_array()
        .ok_or("missing row metadata")?;
    let reference = std::str::from_utf8(&reference)?;
    if rows.len() != ROWS || reference.lines().count() != ROWS {
        return Err("incorrect reference count".into());
    }
    for (i, (meta, line)) in rows.iter().zip(reference.lines()).enumerate() {
        let base: Value = serde_json::from_str(line)?;
        let at = field(meta, "target_position")?;
        let n = field(meta, "prefix_len")?;
        let target = field(meta, "target_token")?;
        if at >= N
            || n > at
            || n == 0
            || last.is_some_and(|previous| previous >= at)
            || mask[at] != 1
            || field(meta, "ordinal")? != i
            || field(&base, "ordinal")? != i
            || field(&base, "status")? != 0
            || field(&base, "target")? != target
            || target != usize::from(ids[at])
        {
            return Err("misaligned ft/dev row".into());
        }
        last = Some(at);
        let prefix = &ids[at - n..at];
        canon.update((i as u64).to_le_bytes());
        canon.update((at as u64).to_le_bytes());
        canon.update((target as u32).to_le_bytes());
        canon.update((n as u32).to_le_bytes());
        for &t in prefix {
            canon.update(u32::from(t).to_le_bytes());
        }
        let key = &prefix[n.saturating_sub(count.context_order())..];
        let prediction = *by_prefix
            .entry(key)
            .or_insert_with(|| count.top1_for_prefix(prefix));
        let mass = count.mass_q32_for_prefix(prefix, ids[at]);
        let log_word = u32::from_le_bytes(bits[at * 4..at * 4 + 4].try_into()?);
        let routes = count.context_routes(prefix);
        if u64::from(prediction) != words(greedy, at * 6)
            || count.mass_q32_for_prefix(prefix, prediction) != words(greedy, at * 6 + 1)
            || mass != words(&tape, at)
            || bits_q12(mass << 16)? != u64::from(log_word)
            || routes
                .iter()
                .enumerate()
                .any(|(depth, &key)| key != words(greedy, at * 6 + 2 + depth))
        {
            return Err(format!("CUDA/D11 original winner or Q32 mismatch at row {i}").into());
        }
        let old = field(&base, "prediction")?;
        let right = prediction == ids[at];
        let was_right = old == target;
        hits += u32::from(right);
        base_hits += u32::from(was_right);
        gains += u32::from(right && !was_right);
        losses += u32::from(!right && was_right);
        if right != was_right && examples.len() < 8 {
            examples.push(json!({"ordinal":i,"position":at,"previous":key,"target":target,"trunk":old,"count":prediction}));
        }
    }
    if format!("{:x}", canon.finalize()) != ROW_DIGEST || base_hits != 7002 {
        return Err("canonical prefix or historical baseline mismatch".into());
    }
    let diff = hits.abs_diff(base_hits);
    let net = i64::from(hits) - i64::from(base_hits);
    let four_discordant = 4 * (gains + losses);
    let root = four_discordant.isqrt();
    let paired_band = root + u32::from(root * root < four_discordant);
    println!(
        "{}",
        json!({"model":model.artifact_sha256(),"artifact_bytes":artifact_bytes,"depth":count.context_order(),
        "native_state_sha256":hash(count.words()),"native_bytes":count.words().len(),"rows":ROWS,"exact_cuda_d11_rows":ROWS,
        "correct":hits,"base_correct":base_hits,"gained":gains,"lost":losses,"net":net,"absolute_floor_met":hits>=7002,
        "paired_band_ceiling_twice_sqrt":paired_band,"below_paired_band": net < -i64::from(paired_band),
        "squared_difference":u64::from(diff)*u64::from(diff),"four_times_discordant":four_discordant,
        "distinct_prefix_keys":by_prefix.len(),"gpu_tape":hash(&tape),"panel":ROW_DIGEST,
        "tokens_sha256":TOKEN_DIGEST,"response_mask_sha256":MASK_DIGEST,"historical_predictions_sha256":REF_DIGEST,"examples":examples})
    );
    Ok(())
}
