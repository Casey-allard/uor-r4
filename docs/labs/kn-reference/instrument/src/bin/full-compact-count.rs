//! D31 complete UORKNC02 count materialization and independent readback.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use uor_r4_discrete::{
    kn_ceiling::CompactCountLookup,
    ngram_count::{
        plan_partitions, NgramOrderDigest, PartitionRange, NGRAM_MAX_ORDER, NGRAM_VOCABULARY,
    },
};

#[cfg(any(feature = "cuda", test))]
#[path = "full-compact-count/materialize.rs"]
mod materialize;
#[path = "full-compact-count/production.rs"]
mod production;

pub type Fail = Box<dyn std::error::Error>;
pub const N: u32 = NGRAM_MAX_ORDER;
pub const WIDTH: usize = N as usize + 1;
#[cfg(feature = "cuda")]
const BASE: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";

#[derive(Clone, Copy)]
pub struct Dose {
    name: &'static str,
    training: &'static str,
    census: &'static str,
    original_training: &'static str,
    tokens: usize,
    records: usize,
    bytes: u64,
    known_artifact: Option<&'static str>,
    known_tape: Option<&'static str>,
    archive_aliases_raw_top_hashes: bool,
}

impl Dose {
    fn parse(name: &str) -> Result<Self, Fail> {
        match name {
            "c128" => Ok(Self {
                name: "c128",
                training: "1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6",
                census: "bd03c2da8a64c6150bd71f2d13a5cfe922dd7389de4b9ce0dc0cf060a031f7d1",
                original_training:
                    "1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6",
                tokens: 111_671,
                records: 128,
                bytes: 16_459_406,
                known_artifact: Some(
                    "eea859b5d7272ef8d4be3a4ee623c92c2b11da866478b05765a7b20557b3788e",
                ),
                known_tape: Some(
                    "7dcd116dae4ebef8d7c96f5c17916cec55deb75d9d09cd9c276b52e078febd9e",
                ),
                archive_aliases_raw_top_hashes: false,
            }),
            "c1" => Ok(Self {
                name: "c1",
                training: "8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030",
                census: "c1c6fe526b99dbc3939ca2fa8927cdebe82eb47f23ebfe83499f35e035fdc959",
                original_training:
                    "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a",
                tokens: 20_754_301,
                records: 5_402,
                bytes: 2_229_740_835,
                known_artifact: None,
                known_tape: None,
                archive_aliases_raw_top_hashes: true,
            }),
            "c" => Ok(Self {
                name: "c",
                training: "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a",
                census: "28d06be01c22439fb715d6ba516845c8ee5a0530c0b0c09ec9d062461d4a4a53",
                original_training:
                    "cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a",
                tokens: 2_075_608_357,
                records: 515_520,
                bytes: 148_096_362_864,
                known_artifact: None,
                known_tape: None,
                archive_aliases_raw_top_hashes: true,
            }),
            _ => Err("dose must be c128, c1 or c".into()),
        }
    }
}

pub type Census = Vec<[NgramOrderDigest; 2]>;

pub fn digest_json(digest: &NgramOrderDigest) -> Value {
    json!({"order": digest.order, "targets": digest.targets, "grams": digest.grams,
        "contexts": digest.contexts, "count_of_counts": digest.count_of_counts,
        "triple_sha256": digest.triple_sha256, "context_sha256": digest.context_sha256})
}

fn read_census(dose: Dose, path: &Path) -> Result<Census, Fail> {
    let bytes = fs::read(path)?;
    if format!("{:x}", Sha256::digest(&bytes)) != dose.census {
        return Err("unregistered independent census digest".into());
    }
    let source: Value = serde_json::from_slice(&bytes)?;
    let (raw, adjusted, train_sha, tokens) = if dose.archive_aliases_raw_top_hashes {
        (
            &source["raw_orders"],
            &source["orders"],
            &source["inputs"]["train_sha256"],
            &source["inputs"]["train_tokens"],
        )
    } else {
        (
            &source["raw"],
            &source["adjusted"],
            &source["train_sha256"],
            &source["train_tokens"],
        )
    };
    if train_sha != dose.original_training || tokens != dose.tokens {
        return Err("independent census names another TRAIN dose".into());
    }
    let orders = |array: &Value| -> Result<Vec<NgramOrderDigest>, Fail> {
        let rows = array.as_array().ok_or("census rows must be an array")?;
        if rows.len() != WIDTH {
            return Err("independent census has the wrong order width".into());
        }
        rows.iter()
            .enumerate()
            .map(|(k, row)| {
                let integer = |name: &str| row[name].as_u64().ok_or("missing census integer");
                let hash = |name: &str| {
                    row[name]
                        .as_str()
                        .map(str::to_owned)
                        .ok_or("missing census hash")
                };
                if integer("order")? != k as u64 {
                    return Err("independent census orders are out of sequence".into());
                }
                Ok(NgramOrderDigest {
                    order: k as u32,
                    targets: integer("targets")?,
                    grams: integer("grams")?,
                    contexts: integer("contexts")?,
                    count_of_counts: serde_json::from_value(row["count_of_counts"].clone())?,
                    triple_sha256: hash("triple_sha256")?,
                    context_sha256: hash("context_sha256")?,
                })
            })
            .collect()
    };
    let pairs: Census = orders(raw)?
        .into_iter()
        .zip(orders(adjusted)?)
        .map(|(raw, adjusted)| [raw, adjusted])
        .collect();
    if pairs
        .iter()
        .any(|[raw, adjusted]| raw.grams != adjusted.grams || raw.contexts != adjusted.contexts)
    {
        return Err("independent raw and adjusted cardinalities differ".into());
    }
    if dose.archive_aliases_raw_top_hashes && pairs[N as usize][0] != pairs[N as usize][1] {
        return Err("the historical highest-order adjusted digest alias differs".into());
    }
    Ok(pairs)
}

fn verify_digests(
    actual: &[[NgramOrderDigest; 2]],
    expected: &[[NgramOrderDigest; 2]],
    archive_aliases_raw_top_hashes: bool,
) -> Result<(), Fail> {
    if actual.len() != expected.len() {
        return Err("wrong count census order width".into());
    }
    for ([raw, adjusted], [expected_raw, expected_adjusted]) in actual.iter().zip(expected) {
        for (name, value, prior) in [
            ("raw", raw, expected_raw),
            ("adjusted", adjusted, expected_adjusted),
        ] {
            let mut comparable = value.clone();
            if archive_aliases_raw_top_hashes && name == "raw" && value.order == N {
                // claude-439 emits its top adjusted-domain hashes in both
                // lists. Compare the rest of that row and audit the actual
                // raw-domain hashes against the original production words.
                comparable.triple_sha256.clone_from(&adjusted.triple_sha256);
                comparable
                    .context_sha256
                    .clone_from(&adjusted.context_sha256);
            }
            if &comparable != prior {
                return Err(format!(
                    "order {} {name} differs from independent census: {} expected {}",
                    value.order,
                    digest_json(value),
                    digest_json(prior)
                )
                .into());
            }
        }
    }
    Ok(())
}

fn read_training(dose: Dose, path: &Path) -> Result<(Vec<u16>, [u8; 32]), Fail> {
    production::read_training(dose, path, true).map(|training| (training.tokens, training.sha))
}

pub struct Probe {
    original_at: Option<usize>,
    target_at: usize,
}

pub fn probe_query(dose: Dose, tokens: &[u16]) -> (Vec<u16>, Vec<Probe>) {
    let mut query = Vec::new();
    let mut probes = Vec::new();
    let mut add = |prefix: &[u16], original_at| {
        // A delimiter prevents the neighboring probe from supplying an
        // admitted context. Full-width probes only examine their own suffix.
        query.push(0);
        query.extend_from_slice(prefix);
        probes.push(Probe {
            original_at,
            target_at: query.len() - 1,
        });
    };
    let step = if dose.name == "c128" { 13 } else { 2047 };
    for (at, &token) in tokens.iter().enumerate() {
        let boundary = dose.name == "c1"
            && (token == 1 || at.checked_sub(1).is_some_and(|prior| tokens[prior] == 0));
        if token != 0 && (at % step == 0 || boundary) {
            add(&tokens[at.saturating_sub(N as usize)..=at], Some(at));
        }
    }
    let mut state = 0xb13dd5204ce843a5u64;
    for i in 0..4096 {
        let mut history = [0u16; WIDTH];
        let at = i % WIDTH;
        for token in &mut history[1..] {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *token = 1 + (state % 4095) as u16;
        }
        if at == 0 {
            history[0] = 1 + (state % 4095) as u16;
        }
        add(&history[..=at], None);
    }
    (query, probes)
}

pub fn replay(
    lookup: &CompactCountLookup<'_>,
    dose: Dose,
    tokens: &[u16],
    output: &Path,
    reference: impl Fn(&[u16], usize, &Probe, &[u32], &[u64], u64) -> Result<(), Fail>,
) -> Result<Value, Fail> {
    let (query, probes) = probe_query(dose, tokens);
    let mut file = new_writer(output)?;
    let mut hash = Sha256::new();
    let mut original = 0;
    for (i, probe) in probes.iter().enumerate() {
        let (mut counts, mut totals) = ([0; WIDTH], [0; WIDTH]);
        let (stream, at) = match probe.original_at {
            Some(at) => {
                original += 1;
                (tokens, at)
            }
            None => {
                let at = (i - original) % WIDTH;
                (&query[probe.target_at - at..=probe.target_at], at)
            }
        };
        let mass = lookup.lookup(stream, at, &mut counts, &mut totals)?;
        reference(stream, at, probe, &counts, &totals, mass)?;
        for (count, total) in counts.into_iter().zip(totals) {
            hash.update(count.to_le_bytes());
            hash.update(total.to_le_bytes());
        }
        hash.update(mass.to_le_bytes());
        serde_json::to_writer(
            &mut file,
            &json!({"probe": i, "original_train_position": probe.original_at,
            "target": stream[at], "raw_count": counts, "raw_total": totals, "kn_q48": mass}),
        )?;
        file.write_all(b"\n")?;
    }
    let tape = format!("{:x}", hash.finalize());
    if dose.known_tape.is_some_and(|expected| tape != expected) {
        return Err("C128 lookup tape differs from D30".into());
    }
    seal(file)?;
    Ok(
        json!({"lookup_tape_sha256": tape, "lookup_targets": probes.len(),
        "original_train_targets": original, "generated_histories": probes.len() - original}),
    )
}

pub fn new_writer(path: &Path) -> Result<BufWriter<File>, Fail> {
    let file = File::options().write(true).create_new(true).open(path)?;
    Ok(BufWriter::with_capacity(1 << 20, file))
}

pub fn seal(mut writer: BufWriter<File>) -> Result<(), Fail> {
    writer.flush()?;
    writer.get_ref().sync_all()?;
    Ok(())
}

fn open_artifact(dose: Dose, path: &Path, pinned: &str) -> Result<(Vec<u8>, [u8; 32]), Fail> {
    if fs::metadata(path)?.len() != dose.bytes {
        return Err("compact count file has the wrong exact length".into());
    }
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    if format!("{digest:x}") != pinned || dose.known_artifact.is_some_and(|known| pinned != known) {
        return Err("compact count artifact digest differs from registered readback".into());
    }
    Ok((bytes, digest.into()))
}

fn main() -> Result<(), Fail> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, dose, train, census] if cmd == "plan" => {
            let dose = Dose::parse(dose)?;
            let census = read_census(dose, Path::new(census))?;
            let training = production::read_training(dose, Path::new(train), false)?;
            println!("{}", production::plan(dose, &training, &census)?.receipt);
        }
        #[cfg(feature = "cuda")]
        [cmd, dose, train, census, output] if cmd == "spool" => {
            let dose = Dose::parse(dose)?;
            let census = read_census(dose, Path::new(census))?;
            let training = production::read_training(dose, Path::new(train), true)?;
            let plan = production::plan(dose, &training, &census)?;
            let receipt = materialize::spool(dose, training, &census, plan, Path::new(output))?;
            println!("{receipt}");
        }
        [cmd, dose, train, census, artifact, sha, evidence] if cmd == "verify" => {
            let dose = Dose::parse(dose)?;
            if dose.name == "c" {
                return Err("use the streaming pruner to verify the full-C spool".into());
            }
            let (tokens, training) = read_training(dose, Path::new(train))?;
            let expected = read_census(dose, Path::new(census))?;
            let (bytes, pinned) = open_artifact(dose, Path::new(artifact), sha)?;
            let lookup = CompactCountLookup::from_bytes(&bytes, training, pinned)?;
            verify_digests(
                &lookup.canonical_digests()?,
                &expected,
                dose.archive_aliases_raw_top_hashes,
            )?;
            let replay = replay(&lookup, dose, &tokens, Path::new(evidence), |_, _, _, _, _, _| Ok(()))?;
            println!("{replay}");
        }
        #[cfg(feature = "cuda")]
        [cmd, dose, train, census, trunk, output] if cmd == "count" => {
            let started = std::time::Instant::now();
            let dose = Dose::parse(dose)?;
            if dose.name == "c" {
                return Err("use spool for full-C count production".into());
            }
            let (tokens, training) = read_training(dose, Path::new(train))?;
            let expected = read_census(dose, Path::new(census))?;
            let trunk = fs::read(trunk)?;
            if format!("{:x}", Sha256::digest(&trunk)) != BASE {
                return Err("unregistered A1 D11 trunk identity".into());
            }
            uor_r4_integer::discrete_d11::IntegerDiscreteModel::from_bytes(&trunk)?;
            let trunk_len = trunk.len();
            drop(trunk);
            let output = Path::new(output);
            fs::create_dir(output)?;
            let receipt = materialize::count(dose, &tokens, training, expected, output)?;
            let mut receipt_file = new_writer(&output.join("receipt.json"))?;
            let receipt = json!({"dose": dose.name, "train_sha256": dose.training,
                "train_tokens": dose.tokens, "train_records": dose.records, "census_sha256": dose.census,
                "base_trunk_sha256": BASE, "base_trunk_bytes": trunk_len,
                "count_seconds": started.elapsed().as_secs(), "counts": receipt});
            serde_json::to_writer_pretty(&mut receipt_file, &receipt)?;
            seal(receipt_file)?;
            println!("{receipt}");
        }
        _ => return Err("usage: full-compact-count plan DOSE TRAIN CENSUS | spool DOSE TRAIN CENSUS NEW-DIR (CUDA build) | count DOSE TRAIN CENSUS TRUNK NEW-DIR (CUDA build; c128/c1) | verify DOSE TRAIN CENSUS ARTIFACT SHA NEW-EVIDENCE (c128/c1)".into()),
    }
    Ok(())
}
