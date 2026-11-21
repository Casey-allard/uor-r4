use super::*;
use crate::pick_device::{
    load_trunk, root_codes, windows, DeviceTape, PickDevice, C3_SEED, READ_BATCH,
};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use uor_r4_discrete::kn_pick::{
    code_pick, exact_pair_pick, permute_code_rows, row_permutation, PickRow,
};

const ARMS: [(&str, &str, &str, PickClassRule); 7] = [
    ("a0", "disabled", "a1", PickClassRule::Three),
    ("a1", "identity", "a1", PickClassRule::Three),
    ("a1s", "identity", "a1s", PickClassRule::Five),
    ("c1", "masked", "a1", PickClassRule::Three),
    ("c2", "exact", "a1", PickClassRule::Three),
    ("c3", "permuted", "a1", PickClassRule::Three),
    ("c3s", "permuted", "a1s", PickClassRule::Five),
];

fn file_sha(path: &Path) -> Result<String> {
    let mut digest = Sha256::new();
    std::io::copy(&mut fs::File::open(path)?, &mut digest)?;
    Ok(format!("{:x}", digest.finalize()))
}

struct CountTape {
    stream: BufReader<fs::File>,
    digest: Sha256,
    consumed: usize,
}
type CountRow = (u64, [u32; WIDTH], [u64; WIDTH]);

impl CountTape {
    fn open(path: &Path, tune: &Tune) -> Result<Self> {
        let file = fs::File::open(path)?;
        require(
            file.metadata()?.len() == (112 + 140 * ROWS) as u64,
            "wrong UORCBF01 byte length",
        )?;
        let mut stream = BufReader::with_capacity(1 << 20, file);
        let mut header = [0; 112];
        stream.read_exact(&mut header)?;
        require(
            &header[..8] == b"UORCBF01"
                && word(&header[8..16])? == ROWS as u64
                && header[16..48] == pin(C_SHA)?
                && header[48..80] == pin(COMPACT_SHA)?
                && header[80..] == pin(&sha(&tune.uort))?,
            "feature tape does not bind to deployed compact and DEV-tune",
        )?;
        let mut digest = Sha256::new();
        digest.update(header);
        Ok(Self {
            stream,
            digest,
            consumed: 0,
        })
    }

    fn next(&mut self) -> Result<CountRow> {
        let mut bytes = [0; 140];
        self.stream.read_exact(&mut bytes)?;
        self.digest.update(bytes);
        let mut counts = [0; WIDTH];
        let mut totals = [0; WIDTH];
        for i in 0..WIDTH {
            counts[i] = u32::from_le_bytes(bytes[4 * i..4 * (i + 1)].try_into()?);
            totals[i] = word(&bytes[44 + 8 * i..44 + 8 * (i + 1)])?;
        }
        let mass = word(&bytes[132..])?;
        if totals[0] == 0
            || !(1..=1 << 48).contains(&mass)
            || counts.iter().zip(totals).any(|(&c, t)| u64::from(c) > t)
        {
            return Err(fail(format!(
                "invalid count row at DEV-tune ordinal {}",
                self.consumed
            )));
        }
        self.consumed += 1;
        Ok((mass, counts, totals))
    }

    fn finish(mut self, expected: &str) -> Result<()> {
        require(
            self.consumed == ROWS
                && self.stream.read(&mut [0])? == 0
                && format!("{:x}", self.digest.finalize()) == expected,
            "consumed DEV-tune tape differs from GO",
        )
    }
}

struct DeviceScore {
    child: Child,
    input: BufWriter<ChildStdin>,
    output: BufReader<ChildStdout>,
}

impl DeviceScore {
    fn start(runner: &Path, state: &FrozenPickMixture) -> Result<Self> {
        let mut child = Command::new(runner)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let input =
            BufWriter::with_capacity(1 << 20, child.stdin.take().ok_or("missing CUDA stdin")?);
        let output =
            BufReader::with_capacity(1 << 18, child.stdout.take().ok_or("missing CUDA stdout")?);
        let mut device = Self {
            child,
            input,
            output,
        };
        device
            .input
            .write_all(&(176 * u64::from(state.rule().classes())).to_le_bytes())?;
        for &weight in state.weights() {
            device.input.write_all(&weight.to_le_bytes())?;
        }
        Ok(device)
    }

    fn batch(&mut self, table: &CeilingTable) -> Result<Vec<(u16, u64, u64)>> {
        self.input
            .write_all(&(table.buckets().len() as u64).to_le_bytes())?;
        for (&bucket, feature) in table
            .buckets()
            .iter()
            .zip(table.masses().chunks_exact(EXPERTS))
        {
            self.input.write_all(&u64::from(bucket).to_le_bytes())?;
            for &mass in feature {
                self.input.write_all(&mass.to_le_bytes())?;
            }
        }
        self.input.flush()?;
        let mut reply = vec![0; table.buckets().len() * 24];
        self.output.read_exact(&mut reply)?;
        reply
            .chunks_exact(24)
            .map(|row| {
                Ok((
                    word(&row[..8])?.try_into()?,
                    word(&row[8..16])?,
                    word(&row[16..])?,
                ))
            })
            .collect()
    }

    fn finish(mut self) -> Result<()> {
        self.input.write_all(&0u64.to_le_bytes())?;
        self.input.flush()?;
        drop(self.input);
        require(
            self.output.read(&mut [0])? == 0 && self.child.wait()?.success(),
            "CUDA frozen scorer failed",
        )
    }
}

struct State {
    frozen: FrozenPickMixture,
    weights_sha: String,
    manifest_sha: String,
    pick_tape_sha: String,
}

fn read_fit(root: &Path, go: &Value, arm: (&str, &str, &str, PickClassRule)) -> Result<State> {
    let (name, mode, fit_arm, rule) = arm;
    let parent = &go["arms"][name];
    let root = root.join(mode);
    let receipt = fs::read(root.join(format!("{fit_arm}.seal.json")))?;
    require(
        sha(&receipt) == string(&parent["seal_sha256"])?,
        "R-fit receipt differs from GO",
    )?;
    let receipt: Value = serde_json::from_slice(&receipt)?;
    let manifest = fs::read(root.join(format!("{fit_arm}.manifest.json")))?;
    let manifest_sha = sha(&manifest);
    let data: Value = serde_json::from_slice(&manifest)?;
    let prepared = &data["prepared"];
    let want_tag = match rule {
        PickClassRule::Three => "UORS5W01",
        PickClassRule::Five => "UORS5W02",
    };
    let weights = fs::read(root.join(format!("{fit_arm}.weights")))?;
    let weights_sha = sha(&weights);
    require(
        receipt["arm"] == fit_arm
            && receipt["pick_mode"] == mode
            && receipt["weight_sha256"] == weights_sha
            && receipt["manifest_sha256"] == manifest_sha
            && data["schema"] == "uor-r4.d37.r-fit/1"
            && data["arm"] == fit_arm
            && data["rows"] == 15_947_584
            && data["passes"] == 96
            && data["pick_classes"] == rule.classes()
            && data["weight_format"] == want_tag
            && data["q32_weight_words"] == 176 * usize::from(rule.classes()) * EXPERTS
            && prepared["r_sha256"] == R_SHA
            && prepared["c_sha256"] == C_SHA
            && prepared["trunk_sha256"] == TRUNK_SHA
            && prepared["compact_sha256"] == COMPACT_SHA
            && prepared["records"] == 4060
            && prepared["rows"] == 15_947_584
            && prepared["pick_mode"] == mode
            && prepared["count_tape_sha256"] == go["r_count_tape_sha256"]
            && (mode != "permuted" || prepared["permutation_seed"] == C3_SEED),
        &format!("wrong R-fit identity for {name}"),
    )?;
    let pick_tape_sha = string(&prepared["pick_tape_sha256"])?.to_owned();
    pin(&pick_tape_sha)?;
    let frozen = FrozenPickMixture::from_bytes(&weights, pin(&manifest_sha)?)?;
    require(
        frozen.rule() == rule
            && frozen.wire_tag() == want_tag
            && frozen.weights().len() == 176 * usize::from(rule.classes()) * EXPERTS
            && (mode != "masked"
                || frozen
                    .weights()
                    .chunks_exact(EXPERTS)
                    .all(|row| row[EXPERTS - 1] == 0)),
        "R-fit carrier has wrong class count or live masked Pick weights",
    )?;
    Ok(State {
        frozen,
        weights_sha,
        manifest_sha,
        pick_tape_sha,
    })
}

fn read_c2(path: &Path, digest: &str) -> Result<Vec<u32>> {
    let c2_bytes = pinned(path, digest, 16 + ROWS * 4)?;
    require(
        &c2_bytes[..8] == b"R37C2010" && word(&c2_bytes[8..16])? == ROWS as u64,
        "wrong C2 device tape header or cardinality",
    )?;
    Ok(c2_bytes[16..]
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes(word.try_into().expect("four-byte tape stride")))
        .collect())
}

pub fn verify_c2(root: &Path, tape: &Path, expected: &Path) -> Result<()> {
    let tune = Tune::load(root)?;
    let digest = expected.to_str().ok_or("invalid C2 digest")?;
    let words = read_c2(tape, digest)?;
    let wins = windows(&tune.ids, 192)?;
    require(
        wins.len() == words.len(),
        "C2/DEV-tune ordinal count differs",
    )?;
    let kept = keep(&tune, &wins);
    let mut probes = 0;
    for (ordinal, (&word, &(_, window))) in words.iter().zip(&wins).enumerate() {
        let pred = (word != u32::MAX).then_some(word as usize);
        if pred.is_some_and(|p| p == 0 || p + 1 >= window.len()) {
            return Err(fail(format!("invalid C2 predecessor at ordinal {ordinal}")));
        }
        if kept.get(probes) == Some(&ordinal) {
            if pred != exact_pair_pick(window) {
                return Err(fail(format!(
                    "native C2 differs at DEV-tune ordinal {ordinal}"
                )));
            }
            probes += 1;
        }
    }
    require(probes == kept.len() && probes >= 64, "incomplete C2 replay")?;
    println!(
        "{}",
        json!({"c2_tape_sha256": digest, "uort_sha256": sha(&tune.uort),
        "rows": ROWS, "native_probes": probes, "divergent_words": 0})
    );
    Ok(())
}

fn keep(tune: &Tune, wins: &[(usize, &[u16])]) -> Vec<usize> {
    let mut frame = 0;
    wins.iter()
        .enumerate()
        .filter_map(|(ordinal, &(at, prefix))| {
            frame += usize::from(prefix.len() == 1);
            let seam = [BATCH, READ_BATCH, CAPTURE_BATCH].iter().any(|width| {
                let offset = ordinal % width;
                offset == 0 || offset + 1 == *width
            });
            ((frame <= 64 && (prefix.len() <= 8 || tune.ids[at] == 1))
                || seam
                || ordinal % 4093 == 197
                || ordinal + 2 >= ROWS)
                .then_some(ordinal)
        })
        .collect()
}

fn verify_pick(
    expected: Option<(usize, u32, u32)>,
    actual: Option<(usize, u32, u32)>,
    ordinal: usize,
) -> Result<()> {
    if expected != actual
        || expected.map(|(_, d, _)| PickClassRule::Five.classify(Some(d)))
            != actual.map(|(_, d, _)| PickClassRule::Five.classify(Some(d)))
    {
        return Err(fail(format!(
            "DEV-tune Pick predecessor/distance/tie/class diverged at ordinal {ordinal}"
        )));
    }
    Ok(())
}

fn read_pick(
    trunk: &Path,
    tune: &Tune,
    wins: &[(usize, &[u16])],
    kept: &[usize],
    permuted: bool,
) -> Result<Vec<PickRow>> {
    let model = load_trunk(trunk)?;
    let perm = permuted.then(|| row_permutation(4096, C3_SEED));
    let dev = PickDevice::new(&model, perm.as_deref())?;
    let DeviceTape { rows, words } =
        dev.rows(&tune.ids, 192, &|i| kept.binary_search(&i).is_ok())?;
    require(
        rows.len() == ROWS && words.len() == kept.len(),
        "CUDA Pick missed rows or probes",
    )?;
    let perm_codes = if permuted {
        Some(permute_code_rows(
            &root_codes(&model.identity_pick_inventory()?.0)?,
            C3_SEED,
        ))
    } else {
        None
    };
    for (&ordinal, observed) in kept.iter().zip(words) {
        let (at, prefix) = wins[ordinal];
        let dev = rows[ordinal];
        let found = dev
            .pick
            .zip(dev.distance)
            .map(|(pick, d)| (pick as usize, d, observed.blocks[0][2]));
        if observed.ordinal != ordinal {
            return Err(fail(format!("DEV-tune Pick ordinal drift at {ordinal}")));
        }
        let expected = if let Some(codes) = &perm_codes {
            code_pick(codes, prefix)
        } else {
            let ids: Vec<_> = prefix.iter().map(|&id| u32::from(id)).collect();
            let native = model.observe_identity_pick(&ids)?;
            if native.block_words.as_slice() != observed.blocks.as_slice()
                || native.successor != found.map(|(p, _, _)| u32::from(prefix[p + 1]))
            {
                return Err(fail(format!(
                    "DEV-tune native Pick read/tie diverged at ordinal {ordinal}"
                )));
            }
            let block = native.block_words[0];
            if block[0] == u32::MAX {
                None
            } else {
                Some((block[0] as usize, block[1], block[2]))
            }
        };
        verify_pick(expected, found, ordinal)?;
        if dev.hit != found.is_some_and(|(p, _, _)| prefix.get(p + 1) == Some(&tune.ids[at])) {
            return Err(fail(format!(
                "DEV-tune Pick hit diverged at ordinal {ordinal}"
            )));
        }
    }
    Ok(rows)
}

#[derive(Default)]
struct Scores {
    records: Vec<[u128; 2]>,
    tallies: [Tally; 4],
    classes: [[u64; 2]; 5],
    compared: usize,
}

impl Scores {
    fn add(
        &mut self,
        tune: &Tune,
        at: usize,
        prefix: &[u16],
        pick: PickTarget,
        bits: u64,
    ) -> Result<()> {
        if prefix.len() == 1 {
            self.records.push([0; 2]);
        }
        let novel = tune.novel[at] == CLASS_NOVEL;
        let response = tune.mask[at] == 1;
        let len = tune.bytes[tune.ids[at] as usize];
        let record = self.records.last_mut().ok_or("missing DEV-tune record")?;
        record[0] += u128::from(bits);
        if novel {
            record[1] += u128::from(bits);
        }
        for (tally, include) in
            self.tallies
                .iter_mut()
                .zip([true, novel, response, novel && response])
        {
            if include {
                tally.add(bits, len);
            }
        }
        self.classes[pick.class as usize][0] += 1;
        self.classes[pick.class as usize][1] += u64::from(pick.hit);
        Ok(())
    }

    fn render(&self, state: &State) -> Value {
        json!({"all": self.tallies[0].json(), "novel": self.tallies[1].json(),
            "response": self.tallies[2].json(), "novel_response": self.tallies[3].json(),
            "by_class_targets_hits": self.classes, "native_count_and_mix_probes": self.compared,
            "manifest_sha256": state.manifest_sha, "weight_sha256": state.weights_sha})
    }
}

fn compare_record_band(a: &Scores, b: &Scores, index: usize) -> Result<[i128; 2]> {
    require(
        a.records.len() == RECORDS
            && b.records.len() == RECORDS
            && a.tallies[index].bytes == b.tallies[index].bytes,
        "paired records or denominator differ",
    )?;
    let x: Vec<_> = a.records.iter().map(|row| row[index]).collect();
    let y: Vec<_> = b.records.iter().map(|row| row[index]).collect();
    let (lo, hi) = pick_arms::paired_band(&x, &y, 1000, 4390)?;
    Ok([lo, hi])
}

fn report_band(a: &Scores, b: &Scores) -> Result<Value> {
    let axis = |i: usize| -> Result<Value> {
        let denom = i128::from(a.tallies[i].bytes) * 4096;
        require(denom > 0, "missing paired band denominator")?;
        let raw = compare_record_band(a, b, i)?;
        // Round outward to keep the reported interval conservative.
        let lo = (raw[0] * 1_000_000).div_euclid(denom);
        let hi = -(-raw[1] * 1_000_000).div_euclid(denom);
        Ok(
            json!({"q12": raw.map(|v| v.to_string()), "micro_bpb": [lo.to_string(), hi.to_string()],
            "strictly_below_zero": raw[1] < 0}),
        )
    };
    Ok(json!({"all": axis(0)?, "novel": axis(1)?}))
}

struct Inputs<'a> {
    tune: &'a Tune,
    windows: &'a [(usize, &'a [u16])],
    probes: &'a [usize],
    compact: &'a CompactCountLookup<'a>,
    c2: &'a [u32],
    tape: &'a Path,
    tape_sha: &'a str,
    runner: &'a Path,
}
impl Inputs<'_> {
    fn score(&self, state: &State, mode: &str, reads: &[PickRow]) -> Result<Scores> {
        let mut tape = CountTape::open(self.tape, self.tune)?;
        let mut device = DeviceScore::start(self.runner, &state.frozen)?;
        let rule = state.frozen.rule();
        let mut scores = Scores::default();
        for (batch, windows) in self.windows.chunks(BATCH).enumerate() {
            let first = batch * BATCH;
            let mut kn = Vec::with_capacity(windows.len());
            let mut counts = Vec::with_capacity(windows.len() * WIDTH);
            let mut totals = Vec::with_capacity(windows.len() * WIDTH);
            let mut picks = Vec::with_capacity(windows.len());
            for (i, &(at, prefix)) in windows.iter().enumerate() {
                let ordinal = first + i;
                let (mass, c, t) = tape.next()?;
                let (pred, distance, device_hit) = match mode {
                    "disabled" => (None, None, None),
                    "exact" => {
                        let word = self.c2[ordinal];
                        let pred = (word != u32::MAX).then_some(word as usize);
                        if self.probes.binary_search(&ordinal).is_ok()
                            && pred != exact_pair_pick(prefix)
                        {
                            return Err(fail(format!(
                                "C2 GPU/native exact-pair differs at DEV-tune ordinal {ordinal}"
                            )));
                        }
                        (pred, pred.map(|_| 0), None)
                    }
                    _ => {
                        let row = reads[ordinal];
                        (row.pick.map(|i| i as usize), row.distance, Some(row.hit))
                    }
                };
                let successor = pred
                    .and_then(|p| p.checked_add(1).and_then(|p| prefix.get(p)))
                    .filter(|_| pred != Some(0));
                let hit = successor == Some(&self.tune.ids[at]);
                if pred.is_some() != distance.is_some()
                    || pred.is_some() != successor.is_some()
                    || device_hit.is_some_and(|seen| seen != hit)
                {
                    return Err(fail(format!(
                        "noncausal Pick row at DEV-tune ordinal {ordinal}"
                    )));
                }
                if self.probes.binary_search(&ordinal).is_ok() {
                    let (mut native_c, mut native_t) = ([0; WIDTH], [0; WIDTH]);
                    let native =
                        self.compact
                            .lookup(&self.tune.ids, at, &mut native_c, &mut native_t)?;
                    if (native, native_c, native_t) != (mass, c, t) {
                        return Err(fail(format!(
                            "native compact parity failed at DEV-tune ordinal {ordinal}"
                        )));
                    }
                }
                kn.push(mass);
                counts.extend(c);
                totals.extend(t);
                picks.push(PickTarget {
                    class: rule.classify(distance),
                    hit: mode != "masked" && hit,
                });
            }
            let table = CeilingTable::from_lookups_with_pick_classes(
                10,
                &kn,
                &counts,
                &totals,
                &picks,
                rule.classes(),
            )?;
            for (i, (bucket, mass, bits)) in device.batch(&table)?.into_iter().enumerate() {
                let ordinal = first + i;
                if self.probes.get(scores.compared) == Some(&ordinal) {
                    let span = i * WIDTH..(i + 1) * WIDTH;
                    let expected = state.frozen.predict_mass(
                        kn[i],
                        &counts[span.clone()],
                        &totals[span],
                        picks[i],
                    )?;
                    if expected != (bucket, mass) || bits_q12(mass).ok() != Some(bits) {
                        return Err(fail(format!(
                            "native frozen score parity failed at DEV-tune ordinal {ordinal}"
                        )));
                    }
                    scores.compared += 1;
                }
                require(
                    bucket == table.buckets()[i]
                        && (1..=1 << 48).contains(&mass)
                        && bits <= 48 * 4096,
                    "CUDA frozen scorer returned an invalid bucket, mass or bit count",
                )?;
                let (at, prefix) = windows[i];
                scores.add(self.tune, at, prefix, picks[i], bits)?;
            }
        }
        tape.finish(self.tape_sha)?;
        device.finish()?;
        require(
            scores.records.len() == RECORDS
                && scores.tallies[0].targets == ROWS as u64
                && scores.compared == self.probes.len(),
            "missing DEV-tune record, row or replay probe",
        )?;
        Ok(scores)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    root: &Path,
    go_path: &Path,
    go_sha: &Path,
    compact: &Path,
    trunk: &Path,
    tape: &Path,
    c2_path: &Path,
    runner: &Path,
    fit_root: &Path,
    out: &Path,
) -> Result<()> {
    let go = fs::read(go_path)?;
    require(go_sha.to_str() == Some(&sha(&go)), "wrong parent GO digest")?;
    let go_digest = sha(&go);
    let go: Value = serde_json::from_slice(&go)?;
    let sheet = go_path.with_file_name("SHEET.md");
    let fit = go_path.with_file_name("FIT.md");
    require(
        go["schema"] == "uor-r4.d37.tune-go/1"
            && go["phase"] == "DEV-tune frozen S5 selection"
            && go["census_sha256"] == CENSUS_SHA
            && go["compact_sha256"] == COMPACT_SHA
            && go["fit_sha256"] == file_sha(&fit)?
            && go["sheet_sha256"] == file_sha(&sheet)?
            && go["scorer_sha256"] == file_sha(&env::current_exe()?)?
            && go["device_scorer_sha256"] == file_sha(runner)?
            && go["fit_verdict"] == "FIT"
            && go["out"] == out.to_str().ok_or("non-UTF8 output")?,
        "wrong DEV-tune admission, non-author FIT or executable identity",
    )?;
    let tape_sha = string(&go["dev_tune_tape_sha256"])?;
    pin(string(&go["r_count_tape_sha256"])?)?;
    require(file_sha(tape)? == tape_sha, "wrong sealed DEV-tune tape")?;
    let c2_sha = string(&go["dev_tune_c2_tape_sha256"])?;
    let c2 = read_c2(c2_path, c2_sha)?;
    let tune = Tune::load(root)?;
    let states = ARMS
        .iter()
        .map(|&arm| read_fit(fit_root, &go, arm))
        .collect::<Result<Vec<_>>>()?;
    require(
        states[1].pick_tape_sha == states[2].pick_tape_sha
            && states[1].pick_tape_sha == states[3].pick_tape_sha
            && states[5].pick_tape_sha == states[6].pick_tape_sha,
        "R paired arms used different causal read tapes",
    )?;
    require(
        fs::metadata(compact)?.len() == COMPACT_BYTES,
        "wrong compact byte length",
    )?;
    let wins = windows(&tune.ids, 192)?;
    require(
        wins.len() == ROWS,
        "incorrect original DEV-tune ordinal count",
    )?;
    let kept = keep(&tune, &wins);
    require(
        kept.len() >= 64,
        "native/device replay requires >=64 probes",
    )?;
    fs::create_dir(out)?;
    let identity = read_pick(trunk, &tune, &wins, &kept, false)?;
    let permuted = read_pick(trunk, &tune, &wins, &kept, true)?;
    let compact_bytes = fs::read(compact)?;
    let compact = CompactCountLookup::from_bytes(&compact_bytes, pin(C_SHA)?, pin(COMPACT_SHA)?)?;
    let inputs = Inputs {
        tune: &tune,
        windows: &wins,
        probes: &kept,
        compact: &compact,
        c2: &c2,
        tape,
        tape_sha,
        runner,
    };
    let mut scores = Vec::with_capacity(ARMS.len());
    let mut arms = serde_json::Map::new();
    for ((name, mode, _, _), state) in ARMS.into_iter().zip(&states) {
        let reads = if mode == "permuted" {
            &permuted
        } else {
            &identity
        };
        let score = inputs.score(state, mode, reads)?;
        arms.insert(name.into(), score.render(state));
        scores.push(score);
        eprintln!("DEV-tune frozen arm {name} complete");
    }
    let pairs = [
        (2, 1),
        (1, 0),
        (1, 3),
        (1, 4),
        (1, 5),
        (3, 0),
        (2, 4),
        (2, 6),
        (6, 5),
    ];
    let mut bands = serde_json::Map::new();
    for (a, b) in pairs {
        bands.insert(
            format!("{}_minus_{}", ARMS[a].0, ARMS[b].0),
            report_band(&scores[a], &scores[b])?,
        );
    }
    let selected_index = choose(compare_record_band(&scores[2], &scores[1], 0)?);
    let selected = &states[selected_index];
    let report = json!({"schema": "uor-r4.d37.tune-selection/1", "go_sha256": go_digest,
        "phase": "DEV-tune frozen S5 selection", "records": RECORDS, "rows": ROWS,
        "census_sha256": CENSUS_SHA, "dev_tune_uort_sha256": sha(&tune.uort),
        "compact_sha256": COMPACT_SHA, "dev_tune_tape_sha256": tape_sha, "dev_tune_c2_tape_sha256": c2_sha,
        "arms": arms, "bands": bands, "bootstrap": {"resamples":1000,"seed":4390},
        "selection": {"arm": ARMS[selected_index].0, "weight_sha256": selected.weights_sha,
            "manifest_sha256": selected.manifest_sha, "format": selected.frozen.wire_tag()}});
    let mut per_record = new(&out.join("per-record-q12.json"))?;
    serde_json::to_writer(
        &mut per_record,
        &scores
            .iter()
            .map(|score| &score.records)
            .collect::<Vec<_>>(),
    )?;
    per_record.sync_all()?;
    let mut file = new(&out.join("selection.json"))?;
    serde_json::to_writer_pretty(&mut file, &report)?;
    file.sync_all()?;
    println!("{}", serde_json::to_string(&report["selection"])?);
    Ok(())
}

fn choose(overall: [i128; 2]) -> usize {
    if overall[1] < 0 {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_requires_upper_endpoint_strictly_below_zero() {
        assert_eq!(choose([-7, -1]), 2);
        assert_eq!(choose([-7, 0]), 1);
        assert_eq!(choose([-7, 1]), 1);
        assert_eq!(choose([1, 9]), 1);
    }

    #[test]
    fn replay_rejects_a_planted_distance_crossing_and_tie_change() {
        let native = Some((8, 2, 2));
        assert!(verify_pick(native, native, 5).is_ok());
        assert!(verify_pick(native, Some((8, 3, 2)), 5).is_err());
        assert!(verify_pick(native, Some((8, 2, 1)), 5).is_err());
    }
}

/// Synthesizes only integer features and initialized native weights.
pub fn finite(runner: &Path) -> Result<()> {
    let rows = BATCH + 1;
    let mut kn = Vec::new();
    let mut counts = Vec::new();
    let mut totals = Vec::new();
    let mut distances = Vec::new();
    for i in 0..rows {
        kn.push(match i % 5 {
            0 => 1,
            1 => 1 << 48,
            // Keep the final batch-seam row inside the Q48 mass domain.
            _ => 1 + i as u64 * 65_537 * 524_271,
        });
        let order = i % WIDTH;
        distances.push([None, Some(0), Some(1), Some(2), Some(3), Some(16)][i % 6]);
        for k in 0..WIDTH {
            let total = if order == WIDTH - 1 {
                65_537
            } else if k <= order {
                (i + k) as u64 % 29 + 1
            } else {
                0
            };
            totals.push(total);
            counts.push((i as u64 % (total + 1)) as u32);
        }
    }
    for rule in [PickClassRule::Three, PickClassRule::Five] {
        let picks: Vec<_> = distances
            .iter()
            .enumerate()
            .map(|(i, &d)| PickTarget {
                class: rule.classify(d),
                hit: d.is_some() && (i % 2 == 0 || d == Some(2)),
            })
            .collect();
        let table = CeilingTable::from_lookups_with_pick_classes(
            10,
            &kn,
            &counts,
            &totals,
            &picks,
            rule.classes(),
        )?;
        let buckets = 176 * usize::from(rule.classes());
        let mut weights = vec![0; buckets * EXPERTS];
        for (bucket, state) in weights.chunks_exact_mut(EXPERTS).enumerate() {
            let pick = if bucket < 176 {
                0
            } else {
                (1u64 << 30) + (bucket as u64) * (1 << 14)
            };
            state[0] = (1u64 << 32) - pick;
            state[EXPERTS - 1] = pick;
        }
        let frozen = FrozenPickMixture::with_rule(10, [0; 32], weights, rule)?;
        let mut device = DeviceScore::start(runner, &frozen)?;
        let mut substantive = 0;
        for first in (0..rows).step_by(BATCH) {
            let end = (first + BATCH).min(rows);
            let words = first * WIDTH..end * WIDTH;
            let batch = CeilingTable::from_lookups_with_pick_classes(
                10,
                &kn[first..end],
                &counts[words.clone()],
                &totals[words],
                &picks[first..end],
                rule.classes(),
            )?;
            for (local, (bucket, mass, bits)) in device.batch(&batch)?.into_iter().enumerate() {
                let row = first + local;
                let words = row * WIDTH..(row + 1) * WIDTH;
                let expected = frozen.predict_mass(
                    kn[row],
                    &counts[words.clone()],
                    &totals[words],
                    picks[row],
                )?;
                substantive += usize::from(picks[row].hit && mass != kn[row]);
                require(
                    expected == (bucket, mass)
                        && bits_q12(mass).ok() == Some(bits)
                        && table.buckets()[row] == bucket,
                    &format!("finite parity failed at ordinal {row}"),
                )?;
            }
        }
        require(
            substantive > 0,
            "finite fixture failed to score a substantive Pick blend",
        )?;
        let row = distances
            .iter()
            .enumerate()
            .find(|&(row, distance)| *distance == Some(2) && kn[row] < 1 << 48)
            .map(|(row, _)| row);
        // A distance 2 -> 3 shift has a scoring effect only in the five-class arm.
        if rule == PickClassRule::Five {
            let row = row.ok_or("missing planted class fixture")?;
            let words = row * WIDTH..(row + 1) * WIDTH;
            let lookup = |distance| {
                frozen.predict_mass(
                    kn[row],
                    &counts[words.clone()],
                    &totals[words.clone()],
                    PickTarget {
                        class: rule.classify(Some(distance)),
                        hit: true,
                    },
                )
            };
            let (bucket, mass) = lookup(2)?;
            let (moved_bucket, moved_mass) = lookup(3)?;
            require(
                bucket != moved_bucket && mass != moved_mass,
                "distance injection did not move bucket and score",
            )?;
        }
        device.finish()?;
        require(
            table
                .buckets()
                .contains(&(176 * u16::from(rule.classes()) - 1)),
            "finite fixture did not visit final class bucket",
        )?;
    }
    println!(
        "{}",
        json!({"synthetic_rows_per_arm": rows, "arms": 2, "raw_words_compared": rows * 6, "divergent_words": 0})
    );
    Ok(())
}
