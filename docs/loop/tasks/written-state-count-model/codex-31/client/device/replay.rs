use crate::{
    fail,
    pick_device::{load_trunk, PickDevice, ABSENT},
    Failure,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    process::Command,
};
use uor_r4_discrete::{
    kn_ceiling::{
        encode_compact_lookup, CeilingTable, CompactCountLookup, FrozenPickMixture, PickTarget,
        WEIGHT_ONE,
    },
    kn_lookup_device::{compiled_kn_lookup_image, KnLookup},
    kn_score_device::{compiled_kn_score_image, KnScorer},
    ngram_count::{
        partition_from_triples, NgramAssembler, NgramOrderDigest, NgramTriple, PrimeRouteKey,
    },
};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

const N: u32 = 10;
const WIDTH: usize = 11;
const EXPERTS: usize = 14;
const TARGETS: usize = 4095;
const CONTEXT: usize = 192;
const STEPS: usize = 12;
const C1_SHA: &str = "8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030";
const R_SHA: &str = "5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a";
const REAL_ROWS: usize = 15_947_584;
const PANEL_SHA: &str = "e50d7812f4a4dcc3016793e07cf2efb55fa7524d880bf1253324ec255b4b8b11";
const PACKET_SHA: &str = "a4fb823cdd5031cc7ffdb1c402f7d7df2c149002b1335a102a8a012371f081dc";
const CENSUS: &[u8] = include_bytes!("../../../claude-439/run2/pick-p10.json");
const CENSUS_SHA: &str = "c1c6fe526b99dbc3939ca2fa8927cdebe82eb47f23ebfe83499f35e035fdc959";

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn binding(text: &str) -> Result<[u8; 32], Failure> {
    let mut out = [0u8; 32];
    hex::decode_to_slice(text, &mut out)?;
    Ok(out)
}
fn new(path: &Path) -> Result<BufWriter<File>, Failure> {
    Ok(BufWriter::new(
        File::options().write(true).create_new(true).open(path)?,
    ))
}
fn seal(mut file: BufWriter<File>) -> Result<(), Failure> {
    file.flush()?;
    file.get_ref().sync_all()?;
    Ok(())
}
fn order(row: &Value) -> Result<NgramOrderDigest, Failure> {
    let integer = |name: &str| {
        row[name]
            .as_u64()
            .ok_or_else(|| fail(format!("missing census {name}")))
    };
    let hash = |name: &str| {
        row[name]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| fail(format!("missing census {name}")))
    };
    Ok(NgramOrderDigest {
        order: integer("order")?.try_into()?,
        targets: integer("targets")?,
        grams: integer("grams")?,
        contexts: integer("contexts")?,
        count_of_counts: serde_json::from_value(row["count_of_counts"].clone())?,
        triple_sha256: hash("triple_sha256")?,
        context_sha256: hash("context_sha256")?,
    })
}
fn census() -> Result<Vec<[NgramOrderDigest; 2]>, Failure> {
    if digest(CENSUS) != CENSUS_SHA {
        return Err(fail("unexpected independent full order census"));
    }
    let value: Value = serde_json::from_slice(CENSUS)?;
    (0..WIDTH)
        .map(|k| Ok([order(&value["raw_orders"][k])?, order(&value["orders"][k])?]))
        .collect()
}

// Keep the existing GPU lookup kernel entrypoints and retain the only
// relevant target of each expanded suffix. The sparse tables contain every
// follower under every live suffix, including each original context total.
fn candidate_query(prefix: &[u16]) -> (Vec<u16>, Vec<usize>) {
    let suffix = &prefix[prefix.len().saturating_sub(N as usize)..];
    let preceding = suffix.iter().filter(|&&id| id != 0).count();
    let mut query = Vec::with_capacity((suffix.len() + 2) * TARGETS);
    let mut positions = Vec::with_capacity(TARGETS);
    for id in 1..=TARGETS {
        query.push(0);
        query.extend_from_slice(suffix);
        query.push(id as u16);
        positions.push(id * (preceding + 1) - 1);
    }
    (query, positions)
}

fn observed_pick(pick: &PickDevice, prefix: &[u16]) -> Result<([u32; 3], Option<u16>), Failure> {
    let mut stream = Vec::with_capacity(CONTEXT + 2);
    if prefix[0] != 0 {
        stream.push(0);
    }
    stream.extend_from_slice(prefix);
    stream.push(1);
    let ordinal = stream.iter().filter(|&&id| id != 0).count() - 1;
    let tape = pick.rows(&stream, CONTEXT, &|at| at == ordinal)?;
    let word = tape
        .words
        .first()
        .filter(|word| word.ordinal == ordinal)
        .ok_or_else(|| fail("missing device Pick trace"))?;
    if tape.words.len() != 1
        || word
            .blocks
            .iter()
            .any(|block| block[..3] != word.blocks[0][..3])
    {
        return Err(fail(
            "device Pick blocks disagree on predecessor, distance or ties",
        ));
    }
    let words: [u32; 3] = word.blocks[0][..3].try_into()?;
    if words[0] == ABSENT {
        if words != [ABSENT, ABSENT, 0] {
            return Err(fail("unexpected closed Pick trace"));
        }
        return Ok((words, None));
    }
    let successor = prefix
        .get(words[0] as usize + 1)
        .copied()
        .filter(|&id| id != 0)
        .ok_or_else(|| fail("device Pick successor is outside current causal prefix"))?;
    Ok((words, Some(successor)))
}

struct Step {
    token: u16,
    bucket: u16,
    mass: u64,
    ties: u32,
    words: Option<[u32; 3]>,
    successor: Option<u16>,
    mass_sha: String,
}

struct Replay<'a> {
    lookup: CompactCountLookup<'a>,
    census: Vec<[NgramOrderDigest; 2]>,
    pick: PickDevice,
    mixer: &'a Path,
    weights: &'a Path,
    fit: FrozenPickMixture,
}

impl Replay<'_> {
    fn step(
        &self,
        prefix: &[u16],
        enabled: bool,
        out: &Path,
        finite: bool,
    ) -> Result<Step, Failure> {
        let (raw, adjusted) = self.lookup.device_replay_tables(prefix, &self.census)?;
        let (query, positions) = candidate_query(prefix);
        let (counts, totals) = KnLookup::new(0, &raw)?.lookups(&query)?;
        let kn = KnScorer::new(0, &adjusted)?.score(&query)?;
        if counts.len() != kn.len() * WIDTH
            || totals.len() != counts.len()
            || positions.last().copied() != kn.len().checked_sub(1)
        {
            return Err(fail("wrong expanded CUDA lookup shape"));
        }
        let (words, successor) = if enabled {
            observed_pick(&self.pick, prefix)?
        } else {
            ([ABSENT, ABSENT, 0], None)
        };
        let class = match successor {
            None => 0,
            Some(_) if words[1] == 0 => 1,
            Some(_) => 2,
        };
        let mut probe_kn = Vec::with_capacity(TARGETS);
        let mut probe_counts = Vec::with_capacity(TARGETS * WIDTH);
        let mut probe_totals = Vec::with_capacity(TARGETS * WIDTH);
        let mut picks = Vec::with_capacity(TARGETS);
        for (i, &at) in positions.iter().enumerate() {
            let span = at * WIDTH..(at + 1) * WIDTH;
            probe_kn.push(kn[at]);
            probe_counts.extend_from_slice(&counts[span.clone()]);
            probe_totals.extend_from_slice(&totals[span]);
            picks.push(PickTarget {
                class,
                hit: successor == Some((i + 1) as u16),
            });
        }
        let candidates = CeilingTable::from_lookups_with_pick(
            N,
            &probe_kn,
            &probe_counts,
            &probe_totals,
            &picks,
        )?;
        let rows_path = out.join("candidate.rows");
        let words_path = out.join("candidate-masses.u64");
        let mut tape = new(&rows_path)?;
        tape.write_all(b"A1EM0001")?;
        tape.write_all(&(TARGETS as u64).to_le_bytes())?;
        for (&bucket, masses) in candidates
            .buckets()
            .iter()
            .zip(candidates.masses().chunks_exact(EXPERTS))
        {
            tape.write_all(&u64::from(bucket).to_le_bytes())?;
            for mass in masses {
                tape.write_all(&mass.to_le_bytes())?;
            }
        }
        seal(tape)?;
        let status = Command::new(self.mixer)
            .arg(&rows_path)
            .arg(self.weights)
            .arg(&words_path)
            .status()?;
        if !status.success() {
            return Err(fail(format!("CUDA fixed-weight scoring failed: {status}")));
        }
        let masses = fs::read(words_path)?;
        if masses.len() != TARGETS * 8 {
            return Err(fail("wrong CUDA mass tape length"));
        }
        let mut maximum = Step {
            token: 0,
            bucket: 0,
            mass: 0,
            ties: 0,
            words: successor.map(|_| words),
            successor,
            mass_sha: digest(&masses),
        };
        let mut original = prefix.to_vec();
        original.push(1);
        for (i, mass) in masses.chunks_exact(8).enumerate() {
            let q48 = u64::from_le_bytes(mass.try_into()?);
            let id = (i + 1) as u16;
            let bucket = candidates.buckets()[i];
            if finite {
                original[prefix.len()] = id;
                let (mut raw, mut total) = ([0; WIDTH], [0; WIDTH]);
                let kn = self
                    .lookup
                    .lookup(&original, prefix.len(), &mut raw, &mut total)?;
                let span = i * WIDTH..(i + 1) * WIDTH;
                let want = self.fit.predict_mass(kn, &raw, &total, picks[i])?;
                if (kn, raw.as_slice(), total.as_slice(), want)
                    != (
                        probe_kn[i],
                        &probe_counts[span.clone()],
                        &probe_totals[span],
                        (bucket, q48),
                    )
                {
                    return Err(fail(format!(
                        "finite native words differ at original target {id}"
                    )));
                }
            }
            if q48 > maximum.mass {
                (maximum.token, maximum.bucket, maximum.mass, maximum.ties) = (id, bucket, q48, 1);
            } else if q48 == maximum.mass {
                maximum.ties += 1;
            }
        }
        if maximum.token == 0 {
            return Err(fail("no positive legal CUDA candidate"));
        }
        Ok(maximum)
    }

    fn sequence(
        &self,
        mut prefix: Vec<u16>,
        id: u32,
        enabled: bool,
        limit: usize,
        out: &Path,
        trace: &mut impl Write,
        finite: bool,
    ) -> Result<Vec<Step>, Failure> {
        let mut steps = Vec::with_capacity(limit);
        for step in 0..limit {
            let at = out.join(format!("{id}-{}-{step:02}", u8::from(enabled)));
            fs::create_dir(&at)?;
            let ranked = self.step(&prefix, enabled, &at, finite)?;
            let digest = Sha256::new()
                .chain_update(
                    prefix
                        .iter()
                        .flat_map(|id| id.to_le_bytes())
                        .collect::<Vec<_>>(),
                )
                .finalize();
            serde_json::to_writer(
                &mut *trace,
                &json!({"id": id, "read_enabled": enabled, "step": step,
                "token":ranked.token,"bucket":ranked.bucket,"mass_q48":ranked.mass,"top_ties":ranked.ties,
                "pick_words":ranked.words,"pick_successor":ranked.successor,"mass_sha256":ranked.mass_sha,
                "prefix_sha256":format!("{digest:x}"), "context_tokens":prefix.len(),
                "stop":if ranked.token == 1 { Some("eos") } else if step + 1 == limit { Some("max_tokens") } else { None }}),
            )?;
            trace.write_all(b"\n")?;
            let token = ranked.token;
            steps.push(ranked);
            if token == 1 {
                break;
            }
            if prefix.len() == CONTEXT {
                prefix.remove(0);
            }
            prefix.push(token);
        }
        Ok(steps)
    }
}

fn synthetic() -> Result<(Vec<u8>, Vec<[NgramOrderDigest; 2]>, FrozenPickMixture), Failure> {
    let history = [0, 5, 6, 7, 5, 6, 8, 5, 6];
    let mut raw = Vec::new();
    let mut adjusted = Vec::new();
    let mut full = Vec::new();
    for k in 0..=N {
        let cells = if k == 0 {
            (1..=TARGETS)
                .map(|id| NgramTriple {
                    key: 0,
                    next: id as u16,
                    count: 1,
                })
                .collect()
        } else if k as usize <= history.len() {
            let key = PrimeRouteKey::from_context(&history[history.len() - k as usize..])?.word();
            vec![
                NgramTriple {
                    key,
                    next: 8,
                    count: 7,
                },
                NgramTriple {
                    key,
                    next: 4095,
                    count: 2,
                },
            ]
        } else {
            Vec::new()
        };
        let words = partition_from_triples(cells);
        let mut a = NgramAssembler::new(k, true)?;
        let mut b = NgramAssembler::new_adjusted(k, true)?;
        a.push(&words)?;
        b.push_owned(words)?;
        let a = a.finish();
        let b = b.finish();
        full.push([a.0.clone(), b.0.clone()]);
        raw.push(a);
        adjusted.push(b);
    }
    let mut weights = vec![0; 528 * EXPERTS];
    for (i, row) in weights.chunks_exact_mut(EXPERTS).enumerate() {
        row[if i < 176 { 12 } else { 13 }] = WEIGHT_ONE;
    }
    Ok((
        encode_compact_lookup(N, [0; 32], &raw, &adjusted)?,
        full,
        FrozenPickMixture::new(N, [0; 32], weights)?,
    ))
}

pub fn finite(mixer: &Path, trunk: &Path, out: &Path) -> Result<(), Failure> {
    let model = load_trunk(trunk)?;
    let (bytes, census, fit) = synthetic()?;
    let lookup = CompactCountLookup::from_bytes(&bytes, [0; 32], Sha256::digest(&bytes).into())?;
    fs::create_dir(out)?;
    let weight_file = out.join("finite.weights");
    let mut file = new(&weight_file)?;
    file.write_all(&fit.to_bytes())?;
    seal(file)?;
    let replay = Replay {
        lookup,
        census,
        pick: PickDevice::new(&model, None)?,
        mixer,
        weights: &weight_file,
        fit,
    };
    let mut trace = new(&out.join("finite.jsonl"))?;
    let history = vec![0, 5, 6, 7, 5, 6, 8, 5, 6];
    let denied = replay.sequence(history.clone(), 1, false, STEPS, out, &mut trace, true)?;
    let opened = replay.sequence(history.clone(), 1, true, STEPS, out, &mut trace, true)?;
    let short = replay.sequence(vec![0], 2, true, STEPS, out, &mut trace, true)?;
    let last_id = replay.sequence(vec![0, 5, 6, 4095, 5, 6], 3, true, 1, out, &mut trace, true)?;
    let mut boundary = vec![5; CONTEXT];
    boundary[0] = 0;
    let sliding = replay.sequence(boundary, 4, true, 2, out, &mut trace, true)?;
    let history32 = history.iter().map(|&id| u32::from(id)).collect::<Vec<_>>();
    let reference = model.observe_identity_pick(&history32)?;
    let expected: [u32; 3] = reference.block_words[0][..3].try_into()?;
    if denied.len() != 1
        || denied[0].token != 1
        || denied[0].ties != TARGETS as u32
        || denied[0].successor.is_some()
        || opened.len() != STEPS
        || opened[0].words != Some(expected)
        || expected != [5, 0, 2]
        || opened[0].token != 8
        || opened[0].successor != Some(8)
        || short.len() != 1
        || short[0].token != 1
        || short[0].words.is_some()
        || last_id[0].token != 4095
        || sliding.len() != 2
    {
        return Err(fail(
            "finite CUDA latest tie, gate suppression, EOS or max-token control failed",
        ));
    }
    seal(trace)?;
    println!("finite CUDA: 17 steps, 4095 targets per step, exact scalar words, Pick tie, forced winner switch, EOS, max ID, cap and sliding window");
    Ok(())
}

fn panel(bytes: &[u8]) -> Result<Vec<(u32, Vec<u16>)>, Failure> {
    if digest(bytes) != PACKET_SHA {
        return Err(fail("unregistered original-prefix packet"));
    }
    let value: Value = serde_json::from_slice(bytes)?;
    if value["panel_sha256"] != PANEL_SHA || value["protocol"] != 2 || value["max_tokens"] != STEPS
    {
        return Err(fail("wrong panel, protocol or generation limit"));
    }
    let rows = value["rows"]
        .as_array()
        .ok_or_else(|| fail("missing original prefix rows"))?;
    if rows.len() != 8 {
        return Err(fail("wrong number of prefix rows"));
    }
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let id = 31001 + i as u32;
            let prefix: Vec<u16> = serde_json::from_value(row["prefix"].clone())?;
            if row["id"] != id
                || prefix.first() != Some(&0)
                || prefix.len() > CONTEXT
                || prefix.iter().skip(1).any(|&id| !(2..=4095).contains(&id))
            {
                return Err(fail(format!("invalid original open prefix for row {id}")));
            }
            Ok((id, prefix))
        })
        .collect()
}

pub fn run(
    mixer: &Path,
    trunk: &Path,
    counts: &Path,
    manifest: &Path,
    weights: &Path,
    weights_sha: &Path,
    packet: &Path,
    out: &Path,
) -> Result<(), Failure> {
    let model: IntegerDiscreteModel = load_trunk(trunk)?;
    let manifest_bytes = fs::read(manifest)?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    let prepared = &manifest["prepared"];
    if manifest["schema"] != "uor-r4.d31.a1-fit/1"
        || manifest["historical_q12_pass_sums_equal"] != 97
        || manifest["historical_q32_pick_words_equal"] != 306
        || manifest["rows"] != REAL_ROWS
        || prepared["rows"] != REAL_ROWS
        || prepared["r_sha256"] != R_SHA
        || prepared["c_sha256"] != C1_SHA
        || prepared["trunk_sha256"] != model.artifact_sha256()
    {
        return Err(fail("unverified D31 fit, trunk, count origin or panel"));
    }
    let compact_sha = prepared["compact_sha256"]
        .as_str()
        .ok_or_else(|| fail("missing full count artifact binding"))?;
    let packed = fs::read(counts)?;
    if packed.len() != 2_229_740_835 {
        return Err(fail("wrong complete C1 count length"));
    }
    let lookup = CompactCountLookup::from_bytes(&packed, binding(C1_SHA)?, binding(compact_sha)?)?;
    let weight_bytes = fs::read(weights)?;
    let weight_hash = digest(&weight_bytes);
    if weights_sha.to_str() != Some(weight_hash.as_str()) || weight_bytes.len() != 59_180 {
        return Err(fail("unverified fitted weight bytes"));
    }
    let fit = FrozenPickMixture::from_bytes(&weight_bytes, Sha256::digest(&manifest_bytes).into())?;
    let packet_bytes = fs::read(packet)?;
    let prefixes = panel(&packet_bytes)?;
    let replay = Replay {
        lookup,
        census: census()?,
        pick: PickDevice::new(&model, None)?,
        mixer,
        weights,
        fit,
    };
    fs::create_dir(out)?;
    let mut trace = new(&out.join("steps.jsonl"))?;
    let mut step_count = 0;
    for (id, prefix) in prefixes {
        for read in [true, false] {
            step_count += replay
                .sequence(prefix.clone(), id, read, STEPS, out, &mut trace, false)?
                .len();
        }
    }
    seal(trace)?;
    let mut receipt = new(&out.join("receipt.json"))?;
    serde_json::to_writer_pretty(
        &mut receipt,
        &json!({"schema":"uor-r4.d31.cuda-client-oracle/1",
        "panel_sha256": PANEL_SHA,"prefix_packet_sha256":digest(&packet_bytes),"trunk_sha256":model.artifact_sha256(),
        "count_sha256":compact_sha,"weights_sha256":weight_hash,"fit_manifest_sha256":digest(&manifest_bytes),
        "kn_image":compiled_kn_score_image(),"raw_image":compiled_kn_lookup_image(),"steps":step_count,
        "candidates_per_step":TARGETS,"steps_sha256":digest(&fs::read(out.join("steps.jsonl"))?)}),
    )?;
    seal(receipt)?;
    Ok(())
}
