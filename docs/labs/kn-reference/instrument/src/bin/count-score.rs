//! Count, fit, persist and replay an order-one mixture on a complete named DEV panel.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
use uor_r4_discrete::{
    ngram_count::{NgramAssembler, NgramTriple, PrimeRouteKey},
    ngram_count_device::NgramCounter,
    ngram_count_oracle::oracle_count,
};
use uor_r4_integer::{
    count_mixer::{self, CountMixer, VOCAB},
    discrete_d11::IntegerDiscreteModel,
    lawful_likelihood::bits_q12,
};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
const TRAIN: &str = "c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460";
const BASE: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
fn sha(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn verified(path: &Path, expected: &str) -> Result<Vec<u8>> {
    let b = fs::read(path)?;
    if sha(&b) != expected {
        return Err(format!("identity mismatch: {}", path.display()).into());
    }
    Ok(b)
}
fn token_bytes(ids: &[u16]) -> Vec<u8> {
    ids.iter().flat_map(|&x| x.to_le_bytes()).collect()
}
fn sample(path: &Path) -> Result<(Vec<u16>, Vec<u16>)> {
    let mut f = BufReader::new(fs::File::open(path)?);
    let mut header = [0; 64];
    f.read_exact(&mut header)?;
    if &header[..4] != b"UORT" {
        return Err("bad TRAIN header".into());
    }
    let mut hash = Sha256::new();
    hash.update(header);
    let mut left = f.get_ref().metadata()?.len() - 64;
    let mut buf = vec![0; 1 << 20];
    let (mut train, mut fit) = (vec![], vec![]);
    let (mut records, mut prev, mut tokens) = (0u32, 1u16, 0u64);
    while left > 0 {
        let n = left.min(buf.len() as u64) as usize;
        f.read_exact(&mut buf[..n])?;
        hash.update(&buf[..n]);
        left -= n as u64;
        for x in buf[..n].chunks_exact(2) {
            let t = u16::from_le_bytes([x[0], x[1]]);
            if usize::from(t) >= VOCAB || (t == 0) != (prev == 1) {
                return Err("invalid TRAIN records".into());
            }
            if t == 0 {
                records += 1;
            }
            if (records - 1) % 100 == 0 {
                train.push(t);
            }
            if (records - 1) % 1000 == 1 {
                fit.push(t);
            }
            prev = t;
            tokens += 1;
        }
    }
    if format!("{:x}", hash.finalize()) != TRAIN
        || prev != 1
        || records != 519580
        || tokens != 2091560001
    {
        return Err("bad TRAIN identity".into());
    }
    Ok((train, fit))
}
struct Panel {
    ids: Vec<u16>,
    novel: Vec<u8>,
    bytes: Vec<u8>,
}
impl Panel {
    fn load(dir: &Path) -> Result<Self> {
        let id_words = verified(
            &dir.join("dev-tune.tokens.u16"),
            "75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a",
        )?;
        let ids = id_words[64..]
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .collect::<Vec<_>>();
        let novel = verified(
            &dir.join("dev-tune.novel.u8"),
            "7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af",
        )?;
        let bytes = verified(
            &dir.join("id-bytes.u8"),
            "befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9",
        )?;
        if ids.len() != 10810087 || novel.len() != ids.len() || bytes.len() != VOCAB {
            return Err("bad query shape".into());
        }
        Ok(Self { ids, novel, bytes })
    }
}
fn write_counts(uni: &[NgramTriple], bigram: &[NgramTriple]) -> Result<Vec<u8>> {
    let mut words = vec![0u8; (VOCAB + 1) * 4];
    words[..4].copy_from_slice(&128u32.to_le_bytes());
    for row in uni {
        let i = (usize::from(row.next) + 1) * 4;
        words[i..i + 4].copy_from_slice(&row.count.to_le_bytes());
    }
    for row in bigram {
        let p = PrimeRouteKey::from_word(row.key)?
            .slot(1)
            .ok_or("missing previous ID")?;
        words.extend(p.to_le_bytes());
        words.extend(row.next.to_le_bytes());
        words.extend(row.count.to_le_bytes());
    }
    CountMixer::from_words(&words)?;
    Ok(words)
}
fn count(tokens: &[u16]) -> Result<(Vec<u8>, Vec<u8>)> {
    let device = NgramCounter::new(0, tokens)?;
    let raw = |order| -> Result<Vec<NgramTriple>> {
        let mut assembler = NgramAssembler::new(order, true)?;
        device.count_order(order, 1_000_000, &mut assembler)?;
        let (a, gpu, _) = assembler.finish();
        let (b, cpu, _) = oracle_count(tokens, order)?;
        if a != b || gpu != cpu {
            return Err("raw count/oracle mismatch".into());
        }
        Ok(gpu)
    };
    let uni = raw(0)?;
    let bigram = raw(1)?;
    let pruned = device.freeze_s2(&bigram)?;
    let mut retained = vec![];
    for row in bigram.chunk_by(|a, b| a.key == b.key) {
        let mut sorted = row.to_vec();
        sorted.retain(|x| x.count >= 2);
        sorted.sort_unstable_by_key(|x| (std::cmp::Reverse(x.count), x.next));
        sorted.truncate(4);
        sorted.sort_unstable_by_key(|x| x.next);
        for mut x in sorted {
            x.count = 1 << x.count.ilog2();
            retained.push(x);
        }
    }
    if retained != pruned {
        return Err("pruning dose/oracle mismatch".into());
    }
    Ok((write_counts(&uni, &bigram)?, write_counts(&uni, &pruned)?))
}
fn call(device: &Path, mode: &str, state: &Path, ids: &Path, out: &Path, w: u32) -> Result<()> {
    if !Command::new(device)
        .arg(mode)
        .arg(state)
        .arg(ids)
        .arg(out)
        .arg(w.to_string())
        .status()?
        .success()
    {
        return Err(format!("GPU {mode} failed").into());
    }
    Ok(())
}
fn fit(device: &Path, out: &Path, dose: &str, words: &mut [u8], fit: &[u16]) -> Result<Vec<u32>> {
    let path = out.join(format!("{dose}-fit.words"));
    fs::write(&path, &*words)?;
    let updates = out.join(format!("{dose}-em.u32"));
    call(device, "fit", &path, &out.join("fit.u16"), &updates, 128)?;
    let em: Vec<_> = fs::read(updates)?
        .chunks_exact(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect();
    if em.len() != 4 {
        return Err("invalid device EM tape".into());
    }
    let model = CountMixer::from_words(words)?;
    let mut w = 128u32;
    for &device_weight in &em {
        let mut sum = 0u64;
        let mut n = 0u64;
        for pair in fit.windows(2).filter(|s| s[1] != 0) {
            let (u, c) = model.expert_masses(pair[0], pair[1]);
            let cw = u64::from(w) * u64::from(c);
            let mass = cw + u64::from(256 - w) * u64::from(u);
            sum += ((cw << 16) + mass / 2) / mass;
            n += 1;
        }
        w = ((sum + n * 128) / (n * 256)).clamp(1, 255) as u32;
        if w != device_weight {
            return Err("device fit/host replay disagreement".into());
        }
    }
    words[..4].copy_from_slice(&w.to_le_bytes());
    Ok(em)
}
fn get_u64(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap())
}
fn get_u32(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap())
}
fn score(
    device: &Path,
    out: &Path,
    name: &str,
    model: &CountMixer,
    w: u32,
    panel: &Panel,
) -> Result<Value> {
    let start = Instant::now();
    let state = out.join(format!("{name}.reload.words"));
    fs::write(&state, model.words())?;
    let raw_path = out.join(format!("{name}.gpu"));
    call(
        device,
        "score",
        &state,
        &out.join("query.u16"),
        &raw_path,
        w,
    )?;
    let raw = fs::read(raw_path)?;
    if raw.len() != VOCAB * 2 + panel.ids.len() * 12 {
        return Err("bad scored tape".into());
    }
    let (best, raw_mass) = raw.split_at(VOCAB * 2);
    let (mass, bits) = raw_mass.split_at(panel.ids.len() * 8);
    let choices: Vec<_> = best
        .chunks_exact(2)
        .map(|v| u16::from_le_bytes([v[0], v[1]]))
        .collect();
    let (mut track, mut novel) = ([0u64; 4], [0u64; 4]);
    let mut lowest = u64::MAX;
    let mut examples = vec![];
    let mut last = 0;
    for i in 0..panel.ids.len() {
        let t = panel.ids[i];
        let qm = get_u64(mass, i);
        let qb = get_u32(bits, i);
        if t == 0 {
            if (qm, qb, panel.novel[i]) != (0, 0, 3) {
                return Err("BOS scored".into());
            }
        } else {
            let (u, c) = model.expert_masses(last, t);
            let expected = u64::from(w) * u64::from(c) + u64::from(256 - w) * u64::from(u);
            if qm == 0 || qm != expected || u64::from(qb) != bits_q12(qm << 16)? {
                return Err(format!("device/D11 parity failed at {i}").into());
            }
            let choice = choices[usize::from(last)];
            let hit = u64::from(t == choice);
            let delta = [
                1,
                u64::from(panel.bytes[usize::from(t)]),
                u64::from(qb),
                hit,
            ];
            for j in 0..4 {
                track[j] += delta[j];
                if panel.novel[i] == 1 {
                    novel[j] += delta[j];
                }
            }
            lowest = lowest.min(qm);
            if examples.len() < 12 {
                let (tu, tc) = model.expert_masses(last, choice);
                examples.push(json!({"position":i,"previous":last,"target":t,"device_top1":choice,"target_q32":qm,"winner_q32":u64::from(w)*u64::from(tc)+u64::from(256-w)*u64::from(tu)}));
            }
        }
        last = t;
    }
    for p in 0..VOCAB {
        let native = if w == model.conditional_weight() {
            model.top1(p as u16)
        } else {
            let mut choice = 1;
            let mut best_mass = 0;
            for t in 1..VOCAB as u16 {
                let (u, c) = model.expert_masses(p as u16, t);
                let m = u64::from(256 - w) * u64::from(u) + u64::from(w) * u64::from(c);
                if m > best_mass {
                    best_mass = m;
                    choice = t;
                }
            }
            choice
        };
        if native != choices[p] {
            return Err(format!("D11 greedy mismatch for {p}").into());
        }
    }
    let bins = |v: [u64; 4]| json!({"targets":v[0],"bytes":v[1],"bits_q12":v[2],"correct":v[3]});
    Ok(
        json!({"arm":name,"weight":w,"overall":bins(track),"novel":bins(novel),"min_observed_mass_q32":lowest,"device_tape_sha256":sha(&raw),"greedy_contexts_checked":VOCAB,"wall_including_host_parity_ms":start.elapsed().as_millis(),"examples":examples}),
    )
}
fn run() -> Result<()> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 5 {
        return Err(
            "expected (fixture|run|eval) C420_INPUTS DEVICE BASE_OR_D11 NEW_OUTPUT_DIR".into(),
        );
    }
    let dir = PathBuf::from(&a[1]);
    let device = PathBuf::from(&a[2]);
    let base_path = PathBuf::from(&a[3]);
    let out = PathBuf::from(&a[4]);
    fs::create_dir(&out)?;
    if a[0] == "eval" {
        let model = IntegerDiscreteModel::load(&base_path)?;
        let m = model.count_mixer().ok_or("not mixer")?;
        let panel = Panel::load(&dir)?;
        fs::write(out.join("query.u16"), token_bytes(&panel.ids))?;
        let res = score(&device, &out, "loaded", m, m.conditional_weight(), &panel)?;
        println!(
            "{}",
            serde_json::to_string_pretty(
                &json!({"d11_sha256":model.artifact_sha256(),"score":res})
            )?
        );
        return Ok(());
    }
    let fixture = a[0] == "fixture";
    if !fixture && a[0] != "run" {
        return Err("unknown mode".into());
    }
    let base = verified(&base_path, BASE)?;
    let (train, held, panel) = if fixture {
        let mut train = vec![];
        for (t, n) in [(4, 7), (5, 5), (6, 3), (7, 3), (8, 3), (9, 1)] {
            for _ in 0..n {
                train.extend([0, 3, t, 1]);
            }
        }
        let held = vec![0, 3, 5, 1, 0, 4095, 3, 1];
        let p = Panel {
            ids: held.clone(),
            novel: vec![3, 2, 1, 1, 3, 2, 1, 1],
            bytes: vec![1; VOCAB],
        };
        (train, held, p)
    } else {
        let (t, f) = sample(&dir.join("train.tokens.u16"))?;
        (t, f, Panel::load(&dir)?)
    };
    fs::write(out.join("fit.u16"), token_bytes(&held))?;
    fs::write(out.join("query.u16"), token_bytes(&panel.ids))?;
    let (full, pruned) = count(&train)?;
    let mut doses = vec![];
    let mut scores = vec![];
    let mut full_weight = 0;
    for (dose, mut words) in [("full", full), ("pruned", pruned)] {
        let em = fit(&device, &out, dose, &mut words, &held)?;
        let packed = count_mixer::pack(&base, &words)?;
        let file = out.join(format!("{dose}.d11"));
        fs::write(&file, &packed)?;
        let loaded = IntegerDiscreteModel::load(&file)?;
        let m = loaded.count_mixer().ok_or("missing reloaded mixer")?;
        if m.words() != words || loaded.artifact_sha256() != sha(&packed) {
            return Err("artifact mismatch".into());
        }
        let weight = m.conditional_weight();
        if fixture {
            let mut bad = packed.clone();
            let i = bad.len() - base.len() - 1;
            bad[i] ^= 1;
            if IntegerDiscreteModel::from_bytes(&bad).is_ok() {
                return Err("corruption accepted".into());
            }
            let mut corrupt = words.clone();
            corrupt[0] = 0;
            if CountMixer::from_words(&corrupt).is_ok() {
                return Err("invalid fitted weight accepted".into());
            }
            for previous in [0, 3, 4095] {
                let total = (1..VOCAB as u16)
                    .map(|t| m.mass_q32(previous, t))
                    .sum::<u64>();
                if total != 1u64 << 32 || (1..VOCAB as u16).any(|t| m.mass_q32(previous, t) == 0) {
                    return Err("lack of conservation/support".into());
                }
            }
            let (fallback_unigram, fallback_conditional) = m.expert_masses(4095, 3);
            if fallback_unigram != fallback_conditional {
                return Err("missing-context fallback".into());
            }
            let (_, retained_conditional) = m.expert_masses(3, 7);
            let (_, discarded_conditional) = m.expert_masses(3, 8);
            if dose == "pruned" && (retained_conditional == 0 || discarded_conditional != 0) {
                return Err("prune tie failure".into());
            }
        }
        let min_uni = (1..VOCAB as u16)
            .map(|t| {
                let (unigram, _) = m.expert_masses(0, t);
                unigram
            })
            .min()
            .unwrap();
        doses.push(json!({"dose":dose,"bytes":packed.len(),"state_bytes":words.len(),"rows":(words.len()-(VOCAB+1)*4)/8,"d11_sha256":sha(&packed),"words_sha256":sha(&words),"em":em,"min_unigram_q24":min_uni,"guaranteed_floor_q32":u64::from(min_uni)*u64::from(256-weight)}));
        if dose == "full" {
            full_weight = weight;
            scores.push(score(&device, &out, "unigram", m, 0, &panel)?);
        }
        scores.push(score(
            &device,
            &out,
            &format!("{dose}-fixed"),
            m,
            128,
            &panel,
        )?);
        scores.push(score(
            &device,
            &out,
            &format!("{dose}-fit"),
            m,
            weight,
            &panel,
        )?);
        if dose == "pruned" {
            scores.push(score(
                &device,
                &out,
                "pruned-full-weight",
                m,
                full_weight,
                &panel,
            )?);
        }
    }
    let identity = |t: &[u16]| json!({"sha256":sha(&token_bytes(t)),"tokens":t.len(),"records":t.iter().filter(|&&v|v==0).count()});
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"fixture":fixture,"train":identity(&train),"fit":identity(&held),"query":identity(&panel.ids),"doses":doses,"scores":scores})
        )?
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
