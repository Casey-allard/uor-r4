use super::common::{sha, write_json, write_manifest, Input, Result, Span, TOKENIZER};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::Command,
    thread,
};

const ARMS: [&str; 5] = [
    "baseline",
    "fact_true",
    "fact_reversed",
    "candidate_true",
    "candidate_reversed",
];
const WIDTH: usize = 32;
const MAX_ROWS: usize = 4096;
const MAX_CANDIDATES: usize = 192;
const MAX_PAIRS: usize = 8192;
const NONE: u32 = u32::MAX;
const WORKERS: usize = 6;
const MODEL: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";

fn parallel<T: Send>(len: usize, work: impl Fn(usize) -> Result<T> + Sync) -> Result<Vec<T>> {
    thread::scope(|scope| {
        let chunk = len.div_ceil(WORKERS).max(1);
        let mut jobs = Vec::new();
        for from in (0..len).step_by(chunk) {
            let work = &work;
            jobs.push(scope.spawn(move || {
                (from..(from + chunk).min(len))
                    .map(|i| work(i).map_err(|e| e.to_string()))
                    .collect::<std::result::Result<Vec<_>, _>>()
            }));
        }
        let groups = jobs
            .into_iter()
            .map(|job| job.join().map_err(|_| "panel worker failed")?)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(groups.into_iter().flatten().collect())
    })
}

fn read_inputs(path: &Path, oracle_allowed: bool) -> Result<Vec<Input>> {
    let entries = BufReader::new(fs::File::open(path)?)
        .lines()
        .map(|line| Ok(serde_json::from_str::<Input>(&line?)?))
        .collect::<Result<Vec<_>>>()?;
    if entries.is_empty()
        || entries.windows(2).any(|p| p[0].row >= p[1].row)
        || entries
            .iter()
            .any(|row| row.source.is_empty() || (!oracle_allowed && row.oracle.is_some()))
    {
        return Err(format!("invalid blind input {}", path.display()).into());
    }
    Ok(entries)
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
struct Candidate {
    span: CandidateSpan,
    narrow: bool,
    local: [i16; WIDTH],
    contextual: [i32; WIDTH],
    completed_source_range: [u32; 2],
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
struct CandidateSpan {
    start: u32,
    end: u32,
}

#[derive(Deserialize)]
struct LegacyScore {
    source: u32,
    eligible: bool,
}

#[derive(Deserialize)]
struct LegacySelection {
    source: Option<u32>,
}

#[derive(Deserialize)]
struct LegacyPool {
    fact_span: Span,
    selection: LegacySelection,
    candidates: Vec<(Candidate, LegacyScore)>,
}

#[derive(Deserialize)]
struct RankPool {
    fact_span: Span,
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Replay {
    contextual_sources: Vec<LegacyPool>,
    #[serde(default)]
    pair_rank_candidates: Vec<RankPool>,
}

#[derive(Deserialize)]
struct Projection {
    prompt_ids: Vec<u32>,
    artifact_sha256: String,
    tokenizer_sha256: String,
    generated_ids: Vec<u32>,
    role_replay: Replay,
}

#[derive(Serialize)]
struct IndexedPool {
    packet_row: usize,
    row: u32,
    fact: Option<Span>,
    candidates: Vec<[u32; 2]>,
}

struct Pool {
    fact: Option<Span>,
    baseline_selection: Option<u32>,
    candidates: Vec<Candidate>,
}

struct Runner<'a> {
    chat: &'a Path,
    tokenizer: &'a Path,
}

impl Runner<'_> {
    fn invoke(&self, artifact: &Path, row: &Input, extra: &[&str]) -> Result<Vec<u8>> {
        let mut command = Command::new(self.chat);
        command
            .arg("--discrete")
            .arg(artifact)
            .arg("--tokenizer")
            .arg(self.tokenizer)
            .args([
                "--protocol",
                "2",
                "--temperature",
                "0",
                "--max-tokens",
                "16",
                "--read-mode",
                "enabled",
                "--role-replay",
                "full-detached",
            ])
            .args(extra)
            .arg("--say")
            .arg(&row.source);
        if let Some([qs, qe, fs, fe]) = row.oracle {
            command
                .arg("--pair-rank-oracle")
                .arg(format!("{qs}:{qe}:{fs}:{fe}"));
        }
        let process = command.output()?;
        if !process.status.success() {
            return Err(format!(
                "row {} on {} {extra:?}: {}",
                row.row,
                artifact.display(),
                String::from_utf8_lossy(&process.stderr)
            )
            .into());
        }
        Ok(process.stdout)
    }
}

fn projections(
    runner: &Runner<'_>,
    parent: &Path,
    row: &Input,
    candidate: bool,
    output: &Path,
) -> Result<(Vec<u32>, Vec<Pool>)> {
    let flags = if candidate {
        ["--pair-rank-project-candidate"]
    } else {
        ["--pair-rank-project"]
    };
    let data = runner.invoke(parent, row, &flags)?;
    let projection: Projection = serde_json::from_slice(&data)?;
    if projection.artifact_sha256 != MODEL
        || projection.tokenizer_sha256 != TOKENIZER
        || !projection.generated_ids.is_empty()
        || projection.prompt_ids.is_empty()
    {
        return Err(format!("row {} projection identity differs or emitted IDs", row.row).into());
    }
    let mut candidates_by_fact: BTreeMap<_, _> = projection
        .role_replay
        .pair_rank_candidates
        .into_iter()
        .map(|pool| (pool.fact_span, pool.candidates))
        .collect();
    let mut pools = Vec::new();
    for old in projection.role_replay.contextual_sources {
        if old
            .candidates
            .iter()
            .any(|(c, score)| c.span.start != score.source)
        {
            return Err(format!(
                "row {} has mismatched candidate and score identities",
                row.row
            )
            .into());
        }
        let candidates = if candidate {
            let new = candidates_by_fact
                .remove(&old.fact_span)
                .ok_or("candidate-prefix fact missing")?;
            if !old.candidates.iter().zip(&new).all(|((a, _), b)| {
                a.span == b.span
                    && a.narrow == b.narrow
                    && a.local == b.local
                    && a.completed_source_range == b.completed_source_range
            }) || old.candidates.len() != new.len()
            {
                return Err(format!(
                    "row {} changed membership, local feature, or fence",
                    row.row
                )
                .into());
            }
            new.into_iter()
                .zip(old.candidates)
                .filter_map(|(new, (_, score))| score.eligible.then_some(new))
                .collect()
        } else {
            old.candidates
                .into_iter()
                .filter_map(|(c, score)| score.eligible.then_some(c))
                .collect()
        };
        pools.push(Pool {
            fact: Some(old.fact_span),
            baseline_selection: old.selection.source,
            candidates,
        });
    }
    if !candidates_by_fact.is_empty() && candidate {
        return Err(format!("row {} has unmatched candidate-prefix facts", row.row).into());
    }
    fs::write(output.join(format!("{}.json", row.row)), data)?;
    Ok((projection.prompt_ids, pools))
}

fn checked_file(path: &Path, expected: &str) -> Result<()> {
    if sha(&fs::read(path)?) != expected {
        return Err(format!("SHA-256 mismatch for {}", path.display()).into());
    }
    Ok(())
}

fn require_wick() -> Result<()> {
    let output = Command::new("hostname").arg("-s").output()?;
    if output.status.success() && std::str::from_utf8(&output.stdout)?.trim() == "wick" {
        Ok(())
    } else {
        Err("run D35 projection and capture on Wick".into())
    }
}

fn verify_manifest(root: &Path, entries: &[&str]) -> Result<()> {
    let text = fs::read_to_string(root.join("manifest.sha256"))?;
    let paths = text
        .lines()
        .map(|line| {
            let (digest, file) = line.split_once("  ").ok_or("bad manifest row")?;
            if !entries.contains(&file) {
                return Err(format!("unexpected manifest entry {file}").into());
            }
            checked_file(&root.join(file), digest)?;
            Ok(file)
        })
        .collect::<Result<Vec<_>>>()?;
    if paths != entries {
        return Err(format!("manifest entries differ for {}", root.display()).into());
    }
    Ok(())
}

fn go_receipt(
    runner: &Runner<'_>,
    path: &Path,
    phase: &str,
    inputs: &[(&Path, &str)],
) -> Result<Vec<u8>> {
    let receipt_raw = fs::read(path)?;
    let receipt: Value = serde_json::from_slice(&receipt_raw)?;
    if receipt["decision"] != "GO" || receipt["phase"] != phase || receipt["host"] != "wick" {
        return Err(format!("requires {phase} GO on Wick").into());
    }
    for (file, field) in [(runner.chat, "native_sha256")]
        .into_iter()
        .chain(inputs.iter().copied())
    {
        checked_file(file, receipt[field].as_str().ok_or("missing GO digest")?)?;
    }
    Ok(receipt_raw)
}

struct Packet {
    words: Vec<u32>,
    index: Vec<IndexedPool>,
    training: bool,
    pairs: usize,
}

impl Packet {
    fn new(training: bool) -> Self {
        Self {
            words: vec![0x3135_3344, if training { 1 } else { 2 }, 0],
            index: Vec::new(),
            training,
            pairs: 0,
        }
    }

    fn append(
        &mut self,
        row: &Input,
        pool: Pool,
        expected: Option<Span>,
        ids: usize,
    ) -> Result<()> {
        let candidates = &pool.candidates;
        if candidates.len() > MAX_CANDIDATES
            || self.index.len() == MAX_ROWS
            || candidates
                .windows(2)
                .any(|p| p[0].span.start >= p[1].span.start)
            || candidates.iter().any(|c| {
                c.span.start >= c.span.end
                    || c.span.end as usize > ids
                    || c.completed_source_range[0] != c.span.start
                    || c.completed_source_range[1] as usize >= ids
            })
        {
            return Err(format!("invalid candidate geometry for row {}", row.row).into());
        }
        let reference = if self.training {
            let [start, end] = expected.ok_or("missing supervised TRAIN span")?;
            let completed = [start.try_into()?, end.try_into()?];
            if candidates.len() < 2
                || !candidates
                    .iter()
                    .any(|c| c.completed_source_range == completed)
            {
                return Err(format!(
                    "TRAIN row {} lacks its gold candidate or rival: {:?} {:?}",
                    row.row,
                    pool.fact,
                    candidates
                        .iter()
                        .map(|c| c.completed_source_range)
                        .collect::<Vec<_>>()
                )
                .into());
            }
            self.pairs += candidates.len() - 1;
            if self.pairs > MAX_PAIRS {
                return Err("TRAIN pair limit exceeded".into());
            }
            completed[0]
        } else {
            match candidates.as_slice() {
                [only] if pool.baseline_selection == Some(only.span.start) => only.span.start,
                _ => NONE,
            }
        };
        self.words.push(candidates.len().try_into()?);
        self.words.push(reference);
        let mut coords = Vec::new();
        for c in candidates {
            self.words.extend([c.span.start, c.span.end]);
            self.words
                .extend(c.local.iter().map(|&x| i32::from(x) as u32));
            self.words.extend(c.contextual.iter().map(|&x| x as u32));
            coords.push([c.span.start, c.span.end]);
        }
        self.index.push(IndexedPool {
            packet_row: self.index.len(),
            row: row.row,
            fact: pool.fact,
            candidates: coords,
        });
        Ok(())
    }

    fn finish(mut self, root: &Path, name: &str) -> Result<()> {
        self.words[2] = self.index.len().try_into()?;
        let bytes: Vec<_> = self.words.into_iter().flat_map(u32::to_le_bytes).collect();
        fs::write(root.join(format!("{name}.bin")), bytes)?;
        write_json(&root.join(format!("{name}.index.json")), &self.index)?;
        println!(
            "{name}: {} device rows, {} TRAIN pair records",
            self.index.len(),
            self.pairs
        );
        Ok(())
    }
}

fn project(
    runner: &Runner<'_>,
    parent: &Path,
    frozen: &Path,
    training: Option<&Path>,
    go: &Path,
    output: &Path,
) -> Result<()> {
    require_wick()?;
    checked_file(runner.tokenizer, TOKENIZER)?;
    checked_file(parent, MODEL)?;
    let train_only = training.is_none();
    if train_only {
        verify_manifest(&frozen.join("train"), &["labels.json", "supplied.jsonl"])?;
    }
    let train_manifest = frozen.join("train/manifest.sha256");
    let mut pins: Vec<(&Path, &str)> = vec![(&train_manifest, "train_manifest_sha256")];
    let held_manifest = frozen.join("held/manifest.sha256");
    if !train_only {
        verify_manifest(&frozen.join("held"), &["ordinary.jsonl", "supplied.jsonl"])?;
        pins.push((&held_manifest, "held_manifest_sha256"));
    }
    let receipt = go_receipt(
        runner,
        go,
        if train_only {
            "train-project"
        } else {
            "project-held"
        },
        &pins,
    )?;
    let (train, ordinary, supplied, labels) = if train_only {
        (
            read_inputs(&frozen.join("train/supplied.jsonl"), true)?,
            Vec::new(),
            Vec::new(),
            serde_json::from_slice::<Vec<(u32, Span)>>(&fs::read(
                frozen.join("train/labels.json"),
            )?)?,
        )
    } else {
        (
            Vec::new(),
            read_inputs(&frozen.join("held/ordinary.jsonl"), false)?,
            read_inputs(&frozen.join("held/supplied.jsonl"), true)?,
            Vec::new(),
        )
    };
    let labels: BTreeMap<_, _> = labels.into_iter().collect();
    if train.len() != labels.len()
        || train
            .iter()
            .any(|r| r.oracle.is_none() || !labels.contains_key(&r.row))
        || ordinary.len() != supplied.len()
        || ordinary
            .iter()
            .zip(&supplied)
            .any(|(a, b)| a.row != b.row || a.source != b.source)
    {
        return Err("train/held input mismatch".into());
    }
    fs::create_dir(output)?;
    fs::write(output.join("go.json"), receipt)?;
    for (geometry, candidate) in [("fact-prefix", false), ("candidate-prefix", true)] {
        let directory = output.join(geometry);
        fs::create_dir(&directory)?;
        if let Some(training) = training {
            for file in ["train.bin", "train.index.json", "train.omitted.json"] {
                fs::copy(training.join(geometry).join(file), directory.join(file))?;
            }
        }
        for (name, rows, is_train) in [
            ("train", &train, true),
            ("ordinary", &ordinary, false),
            ("diagnostic", &supplied, false),
        ] {
            if train_only != is_train {
                continue;
            }
            let rows: Vec<_> = rows
                .iter()
                .filter(|r| name != "diagnostic" || r.oracle.is_some())
                .collect();
            let captures = directory.join(format!("{name}.projection"));
            fs::create_dir(&captures)?;
            let mut packet = Packet::new(is_train);
            let mut omitted = Vec::new();
            let projected = parallel(rows.len(), |i| {
                projections(runner, parent, rows[i], candidate, &captures)
            })?;
            for (row, (ids, mut pools)) in rows.into_iter().zip(projected) {
                if let Some([_, _, start, end]) = row.oracle {
                    if pools.len() != 1 || pools[0].fact != Some([start, end]) {
                        return Err(format!(
                            "supplied row {} did not produce exactly its original fact span",
                            row.row
                        )
                        .into());
                    }
                }
                if pools.is_empty() {
                    pools.push(Pool {
                        fact: None,
                        baseline_selection: None,
                        candidates: Vec::new(),
                    });
                }
                for pool in pools {
                    let label = is_train.then(|| labels[&row.row]);
                    if let Some([start, end]) = label {
                        let admitted =
                            pool.candidates.iter().any(|c| {
                                c.completed_source_range.map(|x| x as usize) == [start, end]
                            }) && pool.candidates.len() >= 2;
                        if !admitted {
                            omitted.push(serde_json::json!({
                                "row": row.row, "gold_source": [start,end],
                                "candidate_source_ranges": pool.candidates.iter()
                                    .map(|c| c.completed_source_range).collect::<Vec<_>>()
                            }));
                            continue;
                        }
                    }
                    packet.append(row, pool, label, ids.len())?;
                }
            }
            if is_train {
                write_json(&directory.join("train.omitted.json"), &omitted)?;
            }
            packet.finish(&directory, name)?;
        }
        if train_only {
            write_manifest(&directory, "packets.sha256", &["train.bin"])?;
            write_manifest(
                &directory,
                "indexes.sha256",
                &["train.index.json", "train.omitted.json"],
            )?;
            continue;
        }
        write_manifest(
            &directory,
            "packets.sha256",
            &["train.bin", "ordinary.bin", "diagnostic.bin"],
        )?;
        write_manifest(
            &directory,
            "indexes.sha256",
            &[
                "train.index.json",
                "train.omitted.json",
                "ordinary.index.json",
                "diagnostic.index.json",
            ],
        )?;
    }
    for path in [
        "train.index.json",
        "train.omitted.json",
        "ordinary.index.json",
        "diagnostic.index.json",
    ] {
        if train_only && !path.starts_with("train.") {
            break;
        }
        if fs::read(output.join("fact-prefix").join(path))?
            != fs::read(output.join("candidate-prefix").join(path))?
        {
            return Err(format!("candidate geometry changes packet index {path}").into());
        }
    }
    Ok(())
}

fn capture(
    runner: &Runner<'_>,
    parent: &Path,
    learned: Option<(&Path, &Path)>,
    held: &Path,
    go: &Path,
    output: &Path,
    baseline_input: Option<&Path>,
) -> Result<()> {
    require_wick()?;
    checked_file(runner.tokenizer, TOKENIZER)?;
    checked_file(parent, MODEL)?;
    verify_manifest(held, &["ordinary.jsonl", "supplied.jsonl"])?;
    let held_manifest = held.join("manifest.sha256");
    let baseline_manifest = baseline_input.map(|p| p.join("captures.sha256"));
    let mut pins: Vec<(&Path, &str)> = vec![(&held_manifest, "held_manifest_sha256")];
    let mut arms = vec![(ARMS[0], parent, false)];
    if let Some((fact, candidate)) = learned {
        pins.extend([
            (fact, "fact_carrier_sha256"),
            (candidate, "candidate_carrier_sha256"),
        ]);
        arms.extend([
            (ARMS[1], fact, false),
            (ARMS[2], fact, true),
            (ARMS[3], candidate, false),
            (ARMS[4], candidate, true),
        ]);
    }
    if let Some(path) = &baseline_manifest {
        pins.push((path, "baseline_manifest_sha256"));
    }
    let receipt_raw = go_receipt(
        runner,
        go,
        if learned.is_some() {
            "serve"
        } else {
            "baseline"
        },
        &pins,
    )?;
    let baseline_hashes: BTreeMap<String, String> = if let Some(path) = &baseline_manifest {
        fs::read_to_string(path)?
            .lines()
            .map(|line| {
                let (hash, file) = line.split_once("  ").ok_or("bad baseline checksum row")?;
                Ok((file.to_string(), hash.to_string()))
            })
            .collect::<Result<_>>()?
    } else {
        BTreeMap::new()
    };
    fs::create_dir(output)?;
    fs::write(output.join("go.json"), receipt_raw)?;
    let mut hashes = fs::File::create_new(output.join("captures.sha256"))?;
    for mode in ["ordinary", "supplied"] {
        let path = output.join(mode);
        fs::create_dir(&path)?;
        let rows: Vec<_> = read_inputs(&held.join(format!("{mode}.jsonl")), mode == "supplied")?
            .into_iter()
            .filter(|row| mode == "ordinary" || row.oracle.is_some())
            .collect();
        for &(arm, artifact, reversed) in &arms {
            let dir = path.join(arm);
            fs::create_dir(&dir)?;
            let captures = parallel(rows.len(), |i| {
                let row = &rows[i];
                let flags = if reversed {
                    ["--pair-rank-reversed"].as_slice()
                } else {
                    [].as_slice()
                };
                let relative = format!("{mode}/{arm}/{}.json", row.row);
                let bytes = match baseline_input {
                    Some(base) if arm == "baseline" => {
                        let bytes = fs::read(base.join(&relative))?;
                        if baseline_hashes.get(&relative) != Some(&sha(&bytes)) {
                            return Err(format!(
                                "baseline capture checksum differs for {relative}"
                            )
                            .into());
                        }
                        bytes
                    }
                    _ => runner.invoke(artifact, row, flags)?,
                };
                let mut file = fs::File::create_new(output.join(&relative))?;
                file.write_all(&bytes)?;
                Ok(format!("{}  {relative}", sha(&bytes)))
            })?;
            for checksum in captures {
                writeln!(hashes, "{checksum}")?;
            }
        }
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<()> {
    let paths: Vec<PathBuf> = args.iter().skip(1).map(PathBuf::from).collect();
    match (args.first().map(String::as_str), paths.as_slice()) {
        (Some("project-train"), [chat, parent, tokenizer, frozen, go, output]) => {
            project(&Runner { chat, tokenizer }, parent, frozen, None, go, output)
        }
        (Some("project-held"), [chat, parent, tokenizer, frozen, train, go, output]) => {
            project(&Runner { chat, tokenizer }, parent, frozen, Some(train), go, output)
        }
        (Some("capture"), [chat, parent, fact, candidate, tokenizer, held, go, output]) => {
            capture(&Runner { chat, tokenizer }, parent, Some((fact, candidate)), held, go, output, None)
        }
        (Some("capture"), [chat, parent, fact, candidate, tokenizer, held, go, output, baseline]) => {
            capture(&Runner { chat, tokenizer }, parent, Some((fact, candidate)), held, go, output, Some(baseline))
        }
        (Some("baseline"), [chat, parent, tokenizer, held, go, output]) => {
            capture(&Runner { chat, tokenizer }, parent, None, held, go, output, None)
        }
        _ => Err("usage: d35-panel project-train CHAT D26 TOKENIZER FROZEN GO NEW_DIR | project-held CHAT D26 TOKENIZER FROZEN TRAIN_PROJECTION GO NEW_DIR | baseline CHAT D26 TOKENIZER HELD GO NEW_DIR | capture CHAT D26 FACT_D35 CANDIDATE_D35 TOKENIZER HELD GO NEW_DIR [BASELINE_CAPTURES]".into()),
    }
}
