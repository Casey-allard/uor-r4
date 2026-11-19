use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    ops::Range,
    path::Path,
    process::{Command, Output, Stdio},
    time::Instant,
};
use uor_r4_integer::{
    discrete_d11::{spiralcore_orbit_roots, spiralcore_signed_axes, IntegerDiscreteModel},
    ordered_role::{
        rank_roles, RoleBank, RoleClass, RoleFraming, RoleMode, RolePrototypes, RoleScore,
        RoleSites, RoleUpdate, ROLE_DIMENSIONS, ROLE_RADIUS,
    },
    paired_role::{RoleSpan, ROLE_CONTEXT, ROLE_NIL, ROLE_VOCAB},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const ABI: u32 = 0x324c_524f;
const MAX_ROWS: usize = 8192;
const CANDIDATE_WORDS: usize = 134;

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
enum Operation {
    Signature = 0,
    Train = 1,
    Public = 2,
    Bounded = 3,
    ClampFixture = 4,
}

impl Operation {
    fn fitting(self) -> bool {
        matches!(self, Self::Train | Self::ClampFixture)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Signature => "signature",
            Self::Train => "train",
            Self::Public => "public-full-prefix",
            Self::Bounded => "true-question",
            Self::ClampFixture => "clamp-fixture",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Blind {
    prefix: Vec<u32>,
    text: String,
}

#[derive(Clone, Default)]
struct Labels {
    requested: Option<[usize; 2]>,
    excluded: Option<[usize; 2]>,
}

#[derive(Deserialize)]
struct TrainGrade {
    prefix_sha256: String,
    #[serde(deserialize_with = "required_span")]
    requested: Option<[usize; 2]>,
    #[serde(deserialize_with = "required_span")]
    excluded: Option<[usize; 2]>,
}

fn required_span<'de, D: serde::Deserializer<'de>>(
    input: D,
) -> std::result::Result<Option<[usize; 2]>, D::Error> {
    Option::deserialize(input)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bound {
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct Row {
    ids: Vec<u32>,
    content: Range<usize>,
    region: Range<usize>,
    labels: Labels,
}

#[derive(Serialize)]
struct Candidate {
    span: [usize; 2],
    admitted: bool,
    ids: Option<[Option<u32>; 2 * ROLE_RADIUS]>,
    feature: Option<[i16; ROLE_DIMENSIONS]>,
    score: Option<RoleScore>,
    gold: Option<RoleClass>,
    update: Option<RoleUpdate>,
    margin: Option<i32>,
    updated_gold_and_rival: Option<[[i16; ROLE_DIMENSIONS]; 2]>,
}

#[derive(Serialize)]
struct Decision {
    requested: Option<[usize; 2]>,
    excluded: Option<[usize; 2]>,
    scores: [i32; 2],
    runners_up: [i32; 2],
    ties: [bool; 2],
    words: [u32; 8],
}

impl From<uor_r4_integer::paired_role::PairedRoleDecision> for Decision {
    fn from(decision: uor_r4_integer::paired_role::PairedRoleDecision) -> Self {
        Self {
            requested: decision.requested.map(|span| [span.start, span.end]),
            excluded: decision.excluded.map(|span| [span.start, span.end]),
            scores: [decision.requested_score, decision.excluded_score],
            runners_up: [decision.requested_runner_up, decision.excluded_runner_up],
            ties: [
                decision.requested_score != i32::MIN
                    && decision.requested_score == decision.requested_runner_up,
                decision.excluded_score != i32::MIN
                    && decision.excluded_score == decision.excluded_runner_up,
            ],
            words: decision.words(),
        }
    }
}

#[derive(Serialize)]
struct RowTrace {
    row: usize,
    pass: usize,
    prefix_sha256: String,
    region: [usize; 2],
    content: [usize; 2],
    decision: Decision,
    candidates: Vec<Candidate>,
}

#[derive(Serialize)]
struct RunTrace {
    schema: &'static str,
    mode: RoleMode,
    operation: &'static str,
    bare_sha256: String,
    bounds_sha256: Option<String>,
    #[serde(flatten)]
    state: Option<StateTrace>,
    traces: Vec<RowTrace>,
}

#[derive(Serialize)]
struct StateTrace {
    bindings: [String; 6],
    bank_sha256: String,
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(sha256(bytes))
}

fn words(words: impl IntoIterator<Item = u32>) -> Vec<u8> {
    words.into_iter().flat_map(u32::to_le_bytes).collect()
}

fn append_span(packet: &mut Vec<u8>, span: Option<[usize; 2]>) {
    packet.extend(words(
        span.map_or([ROLE_NIL; 2], |ends| ends.map(|x| x as u32)),
    ));
}

struct Tables {
    roots: Vec<u32>,
    classes: Vec<u32>,
}

impl Tables {
    fn sites(&self) -> Result<RoleSites<'_>> {
        Ok(RoleSites::new(&self.roots, &self.classes)?)
    }

    fn append(&self, packet: &mut Vec<u8>) {
        packet.extend(words(
            self.roots
                .iter()
                .zip(&self.classes)
                .flat_map(|(&r, &c)| [r, c]),
        ));
        packet.extend(spiralcore_orbit_roots().iter().flatten().map(|&c| c as u8));
        packet.extend(spiralcore_signed_axes());
    }
}

fn packet(
    tables: &Tables,
    mode: RoleMode,
    bank: &RolePrototypes,
    rows: &[Row],
    op: Operation,
) -> Result<Vec<u8>> {
    if rows.is_empty() || rows.len() > MAX_ROWS {
        return Err("unsupported tape row count".into());
    }
    let sites = tables.sites()?;
    let mut input = words([ABI, op as u32, mode as u32, rows.len() as u32]);
    tables.append(&mut input);
    input.extend(
        bank.coordinates()
            .iter()
            .flatten()
            .flat_map(|c| c.to_le_bytes()),
    );
    for row in rows {
        sites.prompt(&row.ids, row.content.clone())?;
        input.extend(words(row.ids.iter().copied()));
        input.resize(input.len() + (ROLE_CONTEXT - row.ids.len()) * 4, 0);
        input.extend(words([row.ids.len() as u32]));
        append_span(&mut input, Some([row.content.start, row.content.end]));
        append_span(&mut input, Some([row.region.start, row.region.end]));
        append_span(&mut input, row.labels.requested);
        append_span(&mut input, row.labels.excluded);
    }
    Ok(input)
}

fn execute(device: &str, input: &[u8]) -> Result<Output> {
    execute_args(device, &[], input)
}

fn execute_args(device: &str, args: &[&str], input: &[u8]) -> Result<Output> {
    let mut child = Command::new(device)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let write = child
        .stdin
        .take()
        .ok_or("CUDA stdin missing")?
        .write_all(input);
    let output = child.wait_with_output()?;
    if output.status.success() {
        write?;
    }
    Ok(output)
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N]> {
        let (value, rest) = self
            .0
            .split_at_checked(N)
            .ok_or("truncated CUDA response")?;
        self.0 = rest;
        Ok(value.try_into()?)
    }

    fn word(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take()?))
    }

    fn words<const N: usize>(&mut self) -> Result<[u32; N]> {
        let mut words = [0; N];
        for word in &mut words {
            *word = self.word()?;
        }
        Ok(words)
    }

    fn point(&mut self) -> Result<[i16; ROLE_DIMENSIONS]> {
        let mut point = [0; ROLE_DIMENSIONS];
        for c in &mut point {
            *c = i16::try_from(self.word()? as i32)?;
            if !(-48..=48).contains(c) {
                return Err("CUDA lattice coordinate out of range".into());
            }
        }
        Ok(point)
    }

    fn bank(&mut self) -> Result<RolePrototypes> {
        let mut saved = [[0; ROLE_DIMENSIONS]; 3];
        for word in saved.iter_mut().flatten() {
            *word = i16::from_le_bytes(self.take()?);
        }
        Ok(RolePrototypes::new(saved)?)
    }
}

fn class(span: [usize; 2], labels: &Labels) -> RoleClass {
    if Some(span) == labels.requested {
        RoleClass::Requested
    } else if Some(span) == labels.excluded {
        RoleClass::Excluded
    } else {
        RoleClass::Neither
    }
}

impl Candidate {
    fn raw_words(&self) -> Vec<u32> {
        let mut output = vec![
            self.span[0] as u32,
            self.span[1] as u32,
            u32::from(self.admitted),
        ];
        output.extend(
            self.ids
                .unwrap_or_default()
                .map(|id| id.unwrap_or(if self.admitted { ROLE_NIL } else { 0 })),
        );
        output.extend(self.feature.unwrap_or_default().map(|x| x as u32));
        if let Some(score) = self.score {
            output.extend(score.distances.map(u32::from));
            output.extend([
                score.winner.map_or(ROLE_NIL, |c| c as u32),
                u32::from(score.evidence),
            ]);
        } else {
            output.extend([0; 5]);
        }
        if let (Some(gold), Some(update), Some(margin)) = (self.gold, self.update, self.margin) {
            output.extend([
                gold as u32,
                update.rival as u32,
                u32::from(update.applied),
                u32::from(update.clamp_low),
                u32::from(update.clamp_high),
                margin as u32,
            ]);
        } else {
            output.extend([0; 6]);
        }
        output.extend(
            self.updated_gold_and_rival
                .unwrap_or_default()
                .into_iter()
                .flatten()
                .map(|x| x as u32),
        );
        debug_assert_eq!(output.len(), CANDIDATE_WORDS);
        output
    }

    fn ranking(&self) -> Option<(RoleSpan, RoleScore)> {
        self.score.map(|score| {
            (
                RoleSpan {
                    start: self.span[0],
                    end: self.span[1],
                },
                score,
            )
        })
    }
}

fn compare(
    output: &[u8],
    tables: &Tables,
    initial: &RolePrototypes,
    rows: &[Row],
    mode: RoleMode,
    op: Operation,
) -> Result<(RolePrototypes, Vec<RowTrace>)> {
    let mut reader = Reader(output);
    let saved = reader.bank()?;
    let mut bank = initial.clone();
    let sites = tables.sites()?;
    let mut traces = Vec::new();
    let passes = if op.fitting() { 2 } else { 1 };
    for pass in 0..passes {
        for (ordinal, row) in rows.iter().enumerate() {
            let mut actual_decision = [0; 8];
            for word in &mut actual_decision {
                *word = reader.word()?;
            }
            let count = reader.word()? as usize;
            let prompt = sites.prompt(&row.ids, row.content.clone())?;
            let candidates = prompt.candidates().collect::<Vec<_>>();
            if count != candidates.len() {
                return Err(format!("candidate count differs on pass {pass} row {ordinal}").into());
            }
            let mut traces_for_row = Vec::with_capacity(count);
            for span in candidates {
                let admitted = span.start >= row.region.start && span.end <= row.region.end;
                let mut candidate = Candidate {
                    span: [span.start, span.end],
                    admitted,
                    ids: None,
                    feature: None,
                    score: None,
                    gold: None,
                    update: None,
                    margin: None,
                    updated_gold_and_rival: None,
                };
                if admitted {
                    candidate.ids = Some(
                        prompt
                            .feature_positions(span, row.region.clone())?
                            .map(|slot| slot.map(|at| row.ids[at])),
                    );
                    let feature = prompt.feature(mode, span, row.region.clone())?;
                    candidate.feature = Some(feature);
                    candidate.score = Some(bank.score(&feature)?);
                    if op.fitting() {
                        let gold = class(candidate.span, &row.labels);
                        let update = bank.update(&feature, gold)?;
                        candidate.gold = Some(gold);
                        candidate.margin = Some(
                            i32::from(update.distances[update.rival as usize])
                                - i32::from(update.distances[gold as usize]),
                        );
                        candidate.update = Some(update);
                        candidate.updated_gold_and_rival = update.applied.then(|| {
                            [
                                bank.coordinates()[gold as usize],
                                bank.coordinates()[update.rival as usize],
                            ]
                        });
                    }
                }
                for (field, expected) in candidate.raw_words().into_iter().enumerate() {
                    let actual = reader.word()?;
                    if actual != expected {
                        return Err(format!(
                            "CUDA/Rust mismatch pass {pass} row {ordinal} span {:?} word {field}: {actual} vs {expected}",
                            candidate.span
                        ).into());
                    }
                }
                traces_for_row.push(candidate);
            }
            let decision = rank_roles(traces_for_row.iter().filter_map(Candidate::ranking));
            if actual_decision != decision.words()
                || (!op.fitting() && decision != prompt.predict(mode, &bank, row.region.clone())?)
            {
                return Err(format!("rank mismatch on pass {pass} row {ordinal}").into());
            }
            traces.push(RowTrace {
                row: ordinal,
                pass,
                prefix_sha256: digest(&words(row.ids.iter().copied())),
                region: [row.region.start, row.region.end],
                content: [row.content.start, row.content.end],
                decision: decision.into(),
                candidates: traces_for_row,
            });
        }
    }
    if !reader.0.is_empty() || bank != saved {
        return Err("extra CUDA response bytes or final coordinate mismatch".into());
    }
    Ok((saved, traces))
}

fn run_cuda(
    device: &str,
    tables: &Tables,
    initial: &RolePrototypes,
    rows: &[Row],
    mode: RoleMode,
    op: Operation,
) -> Result<Vec<u8>> {
    let input = packet(tables, mode, initial, rows, op)?;
    let output = execute(device, &input)?;
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(format!("CUDA process exited with {}", output.status).into());
    }
    println!(
        "{} mode={mode:?} rows={} CUDA response: {} bytes, SHA-256 {}",
        op.name(),
        rows.len(),
        output.stdout.len(),
        digest(&output.stdout)
    );
    Ok(output.stdout)
}

fn launch(
    device: &str,
    tables: &Tables,
    initial: &RolePrototypes,
    rows: &[Row],
    mode: RoleMode,
    op: Operation,
) -> Result<(RolePrototypes, Vec<RowTrace>)> {
    let output = run_cuda(device, tables, initial, rows, mode, op)?;
    compare(&output, tables, initial, rows, mode, op)
}

fn decode_class(word: u32) -> Result<RoleClass> {
    match word {
        0 => Ok(RoleClass::Requested),
        1 => Ok(RoleClass::Excluded),
        2 => Ok(RoleClass::Neither),
        _ => Err("invalid CUDA role class".into()),
    }
}

/// Decode the research fit without re-running updates or distance scoring on the host.
fn decode_train(
    output: &[u8],
    tables: &Tables,
    rows: &[Row],
) -> Result<(RolePrototypes, Vec<RowTrace>)> {
    let mut reader = Reader(output);
    let bank = reader.bank()?;
    let sites = tables.sites()?;
    let mut traces = Vec::with_capacity(2 * rows.len());
    for pass in 0..2 {
        for (ordinal, row) in rows.iter().enumerate() {
            let words = reader.words::<8>()?;
            let count = reader.word()? as usize;
            let prompt = sites.prompt(&row.ids, row.content.clone())?;
            let enumerated = prompt.candidates().collect::<Vec<_>>();
            if count != enumerated.len() {
                return Err(
                    format!("TRAIN candidate count differs on pass {pass} row {ordinal}").into(),
                );
            }
            let mut candidates = Vec::with_capacity(count);
            for expected in enumerated {
                let [start, end, admitted] = reader.words::<3>()?;
                let span = [start as usize, end as usize];
                if span != [expected.start, expected.end] || admitted != 1 {
                    return Err("CUDA TRAIN candidate span or admission differs".into());
                }
                let ids = reader
                    .words::<{ 2 * ROLE_RADIUS }>()?
                    .map(|id| (id != ROLE_NIL).then_some(id));
                let feature = reader.point()?;
                let raw_distances = reader.words::<3>()?;
                if raw_distances.iter().any(|&d| d > 3072) {
                    return Err("invalid CUDA lattice distance".into());
                }
                let distances = raw_distances.map(|d| d as u16);
                let winner_word = reader.word()?;
                let winner = (winner_word != ROLE_NIL)
                    .then(|| decode_class(winner_word))
                    .transpose()?;
                let evidence = u16::try_from(reader.word()?)?;
                let gold = decode_class(reader.word()?)?;
                let rival = decode_class(reader.word()?)?;
                let [applied, clamp_low, clamp_high, margin] = reader.words::<4>()?;
                let updated = [reader.point()?, reader.point()?];
                if gold != class(span, &row.labels)
                    || applied > 1
                    || clamp_low > 64
                    || clamp_high > 64
                    || rival == gold
                    || (applied == 0 && updated != [[0; ROLE_DIMENSIONS]; 2])
                {
                    return Err("invalid CUDA role update trace".into());
                }
                candidates.push(Candidate {
                    span,
                    admitted: true,
                    ids: Some(ids),
                    feature: Some(feature),
                    score: Some(RoleScore {
                        distances,
                        winner,
                        evidence,
                    }),
                    gold: Some(gold),
                    update: Some(RoleUpdate {
                        distances,
                        rival,
                        applied: applied == 1,
                        clamp_low: clamp_low as u16,
                        clamp_high: clamp_high as u16,
                    }),
                    margin: Some(margin as i32),
                    updated_gold_and_rival: (applied == 1).then_some(updated),
                });
            }
            let decision = rank_roles(candidates.iter().filter_map(Candidate::ranking));
            if decision.words() != words {
                return Err("CUDA TRAIN ranking differs from raw scored candidates".into());
            }
            traces.push(RowTrace {
                row: ordinal,
                pass,
                prefix_sha256: digest(&self::words(row.ids.iter().copied())),
                region: [row.region.start, row.region.end],
                content: [row.content.start, row.content.end],
                decision: decision.into(),
                candidates,
            });
        }
    }
    if !reader.0.is_empty() {
        return Err("extra CUDA TRAIN response bytes".into());
    }
    Ok((bank, traces))
}

fn read<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn clean_write(path: &Path, bytes: &[u8]) -> Result<()> {
    File::options()
        .create_new(true)
        .write(true)
        .open(path)?
        .write_all(bytes)?;
    Ok(())
}

fn trace_write(path: &Path, data: impl Serialize) -> Result<()> {
    let file = File::options().create_new(true).write(true).open(path)?;
    let mut out = BufWriter::new(file);
    serde_json::to_writer(&mut out, &data)?;
    writeln!(out)?;
    out.flush()?;
    Ok(())
}

struct Asset {
    model: IntegerDiscreteModel,
    tables: Tables,
    tokenizer: ByteBpeTokenizer,
    framing: RoleFraming,
}

impl Asset {
    fn load(model_path: &str, tokenizer_path: &str) -> Result<Self> {
        let start = Instant::now();
        let model = IntegerDiscreteModel::load(Path::new(model_path))?;
        let tokenizer_bytes = fs::read(tokenizer_path)?;
        if model.tokenizer_sha256() != Some(&digest(&tokenizer_bytes)) {
            return Err("model/tokenizer binding mismatch".into());
        }
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
            .ok_or("invalid tokenizer")?;
        let framing = RoleFraming::from_tokenizer(&tokenizer)?;
        let (roots, classes) = model.ordered_role_tables()?;
        let tables = Tables { roots, classes };
        tables.sites()?;
        eprintln!("model_and_framing_load_ns={}", start.elapsed().as_nanos());
        Ok(Self {
            model,
            tables,
            tokenizer,
            framing,
        })
    }

    fn rows(
        &self,
        tape: Vec<Blind>,
        bounds: Option<Vec<Bound>>,
        labels: Option<Vec<TrainGrade>>,
    ) -> Result<Vec<Row>> {
        if bounds
            .as_ref()
            .is_some_and(|bound| bound.len() != tape.len())
            || labels
                .as_ref()
                .is_some_and(|labels| labels.len() != tape.len())
        {
            return Err("row count differs from ordered tape".into());
        }
        let protocol = DialogueProtocol::literal_roles_v2(&self.tokenizer)?;
        let codec = protocol.bind(&self.tokenizer)?;
        tape.into_iter()
            .enumerate()
            .map(|(ordinal, row)| {
                let encoded = codec.encode_assistant_prefix(&[Message {
                    role: "user",
                    content: &row.text,
                }]);
                if encoded.tokens != row.prefix {
                    return Err(format!("protocol-2 public tape disagrees at row {ordinal}").into());
                }
                let content = self.framing.content(&row.prefix)?;
                let region = match &bounds {
                    Some(bounds) => {
                        let bound = &bounds[ordinal];
                        if bound.start > bound.end || bound.end > row.prefix.len() {
                            return Err(
                                format!("invalid diagnostic region at row {ordinal}").into()
                            );
                        }
                        bound.start..bound.end
                    }
                    None => 0..row.prefix.len(),
                };
                let labels = match &labels {
                    Some(grades) => {
                        let grade = &grades[ordinal];
                        if grade.prefix_sha256 != digest(&words(row.prefix.iter().copied())) {
                            return Err(
                                format!("TRAIN grade prefix differs at row {ordinal}").into()
                            );
                        }
                        let prompt = self.tables.sites()?.prompt(&row.prefix, content.clone())?;
                        for span in [grade.requested, grade.excluded].into_iter().flatten() {
                            if !prompt.candidates().any(|c| span == [c.start, c.end]) {
                                return Err(format!(
                                    "TRAIN label did not enumerate at row {ordinal}"
                                )
                                .into());
                            }
                        }
                        if grade.requested.is_some() && grade.requested == grade.excluded {
                            return Err(format!("TRAIN row {ordinal} repeats a role label").into());
                        }
                        Labels {
                            requested: grade.requested,
                            excluded: grade.excluded,
                        }
                    }
                    None => Labels::default(),
                };
                Ok(Row {
                    ids: row.prefix,
                    content,
                    region,
                    labels,
                })
            })
            .collect()
    }
}

fn mode(arg: &str) -> Result<RoleMode> {
    match arg {
        "identity" => Ok(RoleMode::Identity),
        "positional" => Ok(RoleMode::Positional),
        _ => Err("mode must be identity or positional".into()),
    }
}

fn fit(args: &[String]) -> Result<()> {
    let [device, arm, model, tokenizer, blind, grades, out] = args else {
        return Err("fit DEVICE MODE MODEL TOKENIZER TRAIN_BLIND TRAIN_GRADES NEW_OUT".into());
    };
    let asset = Asset::load(model, tokenizer)?;
    let blind = fs::read(blind)?;
    let grades = fs::read(grades)?;
    let receipt = format!(
        "uor-r4.ordered-role-train/1\nblind_sha256 {}\ngrades_sha256 {}\npasses 2\norder row_then_original_candidate_start\n",
        digest(&blind), digest(&grades)
    );
    let mode = mode(arm)?;
    let rows = asset.rows(
        serde_json::from_slice(&blind)?,
        None,
        Some(serde_json::from_slice(&grades)?),
    )?;
    let output = run_cuda(
        device,
        &asset.tables,
        &RolePrototypes::default(),
        &rows,
        mode,
        Operation::Train,
    )?;
    let (bank, traces) = decode_train(&output, &asset.tables, &rows)?;
    let start = Instant::now();
    let bindings = asset
        .model
        .ordered_role_bindings(&asset.framing, sha256(receipt.as_bytes()))?;
    let bank = RoleBank::new(mode, bank, bindings);
    let bytes = bank.to_bytes();
    let restored = RoleBank::from_bytes(&bytes)?;
    restored.verify_bindings(&bindings)?;
    if restored != bank || restored.to_bytes() != bytes {
        return Err("serialized bank reload differs".into());
    }
    eprintln!("bank_pack_verify_reload_ns={}", start.elapsed().as_nanos());
    let out = Path::new(out);
    fs::create_dir(out)?;
    clean_write(&out.join("bank.role"), &bytes)?;
    clean_write(&out.join("train.receipt"), receipt.as_bytes())?;
    trace_write(
        &out.join("train.json"),
        RunTrace {
            schema: "uor-r4.ordered-role-cuda/1",
            mode,
            operation: Operation::Train.name(),
            bare_sha256: digest(&blind),
            bounds_sha256: None,
            state: Some(StateTrace {
                bindings: bindings.map(hex::encode),
                bank_sha256: digest(&bytes),
            }),
            traces,
        },
    )
}

fn infer(args: &[String], bounded: bool) -> Result<()> {
    let [device, model, tokenizer, bank, receipt, blind, rest @ ..] = args else {
        return Err("public DEVICE MODEL TOKENIZER BANK TRAIN_RECEIPT BLIND OUT; bounded adds BOUNDS before OUT".into());
    };
    let (bounds, output) = match (bounded, rest) {
        (true, [bounds, output]) => (Some(fs::read(bounds)?), output),
        (false, [output]) => (None, output),
        _ => {
            return Err(
                "public requires no diagnostic bounds; bounded requires one bounds file".into(),
            )
        }
    };
    let asset = Asset::load(model, tokenizer)?;
    let receipt = fs::read(receipt)?;
    if !receipt.starts_with(b"uor-r4.ordered-role-train/1\n") {
        return Err("invalid ordered TRAIN receipt".into());
    }
    let start = Instant::now();
    let bytes = fs::read(bank)?;
    let bank = RoleBank::from_bytes(&bytes)?;
    let bindings = asset
        .model
        .ordered_role_bindings(&asset.framing, sha256(&receipt))?;
    bank.verify_bindings(&bindings)?;
    if bank.to_bytes().as_slice() != bytes {
        return Err("serialized bank reload differs".into());
    }
    eprintln!("bank_load_and_verify_ns={}", start.elapsed().as_nanos());
    let tape = fs::read(blind)?;
    let regions = bounds.as_deref().map(serde_json::from_slice).transpose()?;
    let rows = asset.rows(serde_json::from_slice(&tape)?, regions, None)?;
    let op = if bounded {
        Operation::Bounded
    } else {
        Operation::Public
    };
    let (_, traces) = launch(
        device,
        &asset.tables,
        &bank.prototypes,
        &rows,
        bank.mode,
        op,
    )?;
    trace_write(
        Path::new(output),
        RunTrace {
            schema: "uor-r4.ordered-role-cuda/1",
            mode: bank.mode,
            operation: op.name(),
            bare_sha256: digest(&tape),
            bounds_sha256: bounds.as_deref().map(digest),
            state: Some(StateTrace {
                bindings: bindings.map(hex::encode),
                bank_sha256: digest(&bytes),
            }),
            traces,
        },
    )
}

fn signatures(args: &[String]) -> Result<()> {
    let [device, arm, model, tokenizer, blind, bounds, out] = args else {
        return Err("signature DEVICE MODE MODEL TOKENIZER BLIND BOUNDS|all OUT".into());
    };
    let asset = Asset::load(model, tokenizer)?;
    let tape = fs::read(blind)?;
    let bounds = (bounds != "all").then(|| fs::read(bounds)).transpose()?;
    let region = bounds.as_deref().map(serde_json::from_slice).transpose()?;
    let rows = asset.rows(serde_json::from_slice(&tape)?, region, None)?;
    let mode = mode(arm)?;
    let (_, traces) = launch(
        device,
        &asset.tables,
        &RolePrototypes::default(),
        &rows,
        mode,
        Operation::Signature,
    )?;
    trace_write(
        Path::new(out),
        RunTrace {
            schema: "uor-r4.ordered-role-cuda/1",
            mode,
            operation: Operation::Signature.name(),
            bare_sha256: digest(&tape),
            bounds_sha256: bounds.as_deref().map(digest),
            state: None,
            traces,
        },
    )
}

#[derive(Deserialize)]
struct FixedMatch {
    family: String,
    pair: String,
    row_left: usize,
    row_right: usize,
    left: [Option<[usize; 2]>; 2],
    right: [Option<[usize; 2]>; 2],
    eligible: bool,
    reasons: Vec<String>,
}

#[derive(Deserialize)]
struct CapturedScore {
    distances: [u16; 3],
    winner: Option<String>,
    evidence: u16,
}

impl CapturedScore {
    fn role_score(&self) -> Result<RoleScore> {
        let winner = match self.winner.as_deref() {
            None => None,
            Some("Requested") => Some(RoleClass::Requested),
            Some("Excluded") => Some(RoleClass::Excluded),
            Some("Neither") => Some(RoleClass::Neither),
            _ => return Err("unknown captured role class".into()),
        };
        Ok(RoleScore {
            distances: self.distances,
            winner,
            evidence: self.evidence,
        })
    }
}

#[derive(Deserialize)]
struct CapturedCandidate {
    span: [usize; 2],
    admitted: bool,
    ids: Option<[Option<u32>; 2 * ROLE_RADIUS]>,
    feature: Option<[i16; ROLE_DIMENSIONS]>,
    score: Option<CapturedScore>,
}

#[derive(Deserialize)]
struct CapturedRow {
    row: usize,
    pass: usize,
    prefix_sha256: String,
    decision: serde_json::Value,
    candidates: Vec<CapturedCandidate>,
}

#[derive(Deserialize)]
struct Capture {
    schema: String,
    mode: String,
    operation: String,
    bank_sha256: String,
    bare_sha256: String,
    traces: Vec<CapturedRow>,
}

impl Capture {
    fn validate(&self, operation: Operation) -> Result<()> {
        if self.schema != "uor-r4.ordered-role-cuda/1"
            || self.operation != operation.name()
            || self
                .traces
                .iter()
                .enumerate()
                .any(|(i, row)| row.row != i || row.pass != 0)
        {
            return Err("diagnostic expects one in-order CUDA inference trace".into());
        }
        Ok(())
    }
}

type FixedScores = [(RoleSpan, RoleScore); 2];

fn fixed_scores(row: &CapturedRow, spans: [Option<[usize; 2]>; 2]) -> Result<Option<FixedScores>> {
    let [Some(left), Some(right)] = spans else {
        return Ok(None);
    };
    if left == right {
        return Err("fixed-competitor mapping repeats a span".into());
    }
    let mut scored = Vec::new();
    for span @ [start, end] in [left, right] {
        let Some(candidate) = row
            .candidates
            .iter()
            .find(|entry| entry.span == span && entry.admitted)
        else {
            return Ok(None);
        };
        let score = candidate
            .score
            .as_ref()
            .ok_or("admitted candidate lacks a score")?;
        scored.push((RoleSpan { start, end }, score.role_score()?));
    }
    Ok(Some(
        scored.try_into().map_err(|_| "wrong fixed score count")?,
    ))
}

fn verify_fixed_gpu(device: &str, scores: &[FixedScores]) -> Result<()> {
    if scores.is_empty() {
        return Ok(());
    }
    if scores.len() > MAX_ROWS {
        return Err("fixed-score diagnostic exceeds device row cap".into());
    }
    let mut request = words([ABI, scores.len() as u32]);
    for pair in scores {
        for (span, score) in pair {
            request.extend(words([
                span.start as u32,
                span.end as u32,
                score.winner.map_or(ROLE_NIL, |class| class as u32),
                u32::from(score.evidence),
            ]));
        }
    }
    let output = execute_args(device, &["--fixed"], &request)?;
    if !output.status.success() {
        return Err(format!(
            "fixed GPU rank failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let mut reader = Reader(&output.stdout);
    for (row, pair) in scores.iter().enumerate() {
        if reader.words::<8>()? != rank_roles(*pair).words() {
            return Err(format!("fixed GPU/Rust rank differs at slice row {row}").into());
        }
    }
    if !reader.0.is_empty() {
        return Err("extra fixed-rank CUDA response bytes".into());
    }
    println!(
        "{} fixed GPU/Rust rank rows, raw SHA-256 {}",
        scores.len(),
        digest(&output.stdout)
    );
    Ok(())
}

fn fixed(args: &[String]) -> Result<()> {
    let [device, bounded, matches, output] = args else {
        return Err("fixed DEVICE BOUNDED_TRACE HELD_MATCH OUT".into());
    };
    let capture: Capture = read(bounded)?;
    capture.validate(Operation::Bounded)?;
    let matches = fs::read(matches)?;
    let mappings: Vec<FixedMatch> = serde_json::from_slice(&matches)?;
    let mut fixed = Vec::new();
    let mut scores = Vec::new();
    for mapping in mappings {
        let left = capture
            .traces
            .get(mapping.row_left)
            .ok_or("left mapping row missing")?;
        let right = capture
            .traces
            .get(mapping.row_right)
            .ok_or("right mapping row missing")?;
        if left.row == right.row {
            return Err("fixed-competitor pair repeats a row".into());
        }
        let left_scores = fixed_scores(left, mapping.left)?;
        let right_scores = fixed_scores(right, mapping.right)?;
        scores.extend([left_scores, right_scores].into_iter().flatten());
        let left_fixed = left_scores.map(|pair| Decision::from(rank_roles(pair)));
        let right_fixed = right_scores.map(|pair| Decision::from(rank_roles(pair)));
        if mapping.eligible && (left_fixed.is_none() || right_fixed.is_none()) {
            return Err("sealed eligible pair has no fixed-competitor trace".into());
        }
        fixed.push(json!({
            "family":mapping.family, "pair":mapping.pair, "matched":mapping.eligible,
            "reasons":mapping.reasons,
            "left":{"row":left.row,"prefix_sha256":left.prefix_sha256,"spans":mapping.left,
                    "fixed":left_fixed,"global":left.decision},
            "right":{"row":right.row,"prefix_sha256":right.prefix_sha256,"spans":mapping.right,
                     "fixed":right_fixed,"global":right.decision},
        }));
    }
    verify_fixed_gpu(device, &scores)?;
    trace_write(
        Path::new(output),
        json!({
            "schema":"uor-r4.ordered-role-fixed/1", "mode":capture.mode, "bank_sha256":capture.bank_sha256,
            "bare_sha256":capture.bare_sha256, "mapping_sha256":digest(&matches), "pairs":fixed
        }),
    )
}

fn eligibility(args: &[String]) -> Result<()> {
    let [public, bounded, output] = args else {
        return Err("eligibility PUBLIC_TRACE BOUNDED_TRACE OUT".into());
    };
    let public: Capture = read(public)?;
    let bounded: Capture = read(bounded)?;
    public.validate(Operation::Public)?;
    bounded.validate(Operation::Bounded)?;
    if public.mode != bounded.mode
        || public.bank_sha256 != bounded.bank_sha256
        || public.bare_sha256 != bounded.bare_sha256
        || public.traces.len() != bounded.traces.len()
    {
        return Err("public and bounded captures use different banks or tapes".into());
    }
    let mut rows = Vec::new();
    for (full, question) in public.traces.iter().zip(&bounded.traces) {
        if full.prefix_sha256 != question.prefix_sha256
            || full.candidates.len() != question.candidates.len()
        {
            return Err("original candidate identities differ".into());
        }
        let mut excluded_by_bound = Vec::new();
        let mut context_changed = Vec::new();
        let mut feature_changed = Vec::new();
        for (before, after) in full.candidates.iter().zip(&question.candidates) {
            if before.span != after.span || !before.admitted {
                return Err("original complete-word candidates differ".into());
            }
            if !after.admitted {
                excluded_by_bound.push(after.span);
            } else {
                if before.ids != after.ids {
                    context_changed.push(after.span);
                }
                if before.feature != after.feature {
                    feature_changed.push(after.span);
                }
            }
        }
        rows.push(json!({
            "row":full.row,"prefix_sha256":full.prefix_sha256,"excluded_by_bound":excluded_by_bound,
            "shared_context_changed":context_changed,"shared_feature_changed":feature_changed,
            "decision_changed":full.decision != question.decision,
            "public":full.decision,"bounded":question.decision
        }));
    }
    trace_write(
        Path::new(output),
        json!({
            "schema":"uor-r4.ordered-role-eligibility/1", "mode":public.mode,
            "bank_sha256":public.bank_sha256,"bare_sha256":public.bare_sha256,"rows":rows
        }),
    )
}

fn fixture_tables() -> Tables {
    let roots = vec![0; ROLE_VOCAB];
    let mut classes = vec![0; ROLE_VOCAB];
    classes[3] = 1;
    classes[4] = 2;
    // Boundary slots deliberately use a word-start class in some fixtures.
    classes[9] = 1;
    Tables { roots, classes }
}

fn dense_row() -> Row {
    let mut ids = vec![3; ROLE_CONTEXT];
    ids[0] = 0;
    ids[ROLE_CONTEXT - 1] = 9;
    ids[ROLE_CONTEXT / 2] = 4;
    Row {
        ids,
        content: 1..ROLE_CONTEXT - 1,
        region: 0..ROLE_CONTEXT,
        labels: Labels::default(),
    }
}

fn finite(args: &[String]) -> Result<()> {
    let [device, out] = args else {
        return Err("finite DEVICE NEW_OUT".into());
    };
    let tables = fixture_tables();
    let full = Row {
        ids: vec![0, 9, 3, 4, 2, 3, 2, 9],
        content: 2..7,
        region: 0..8,
        labels: Labels::default(),
    };
    let bounds = [0..8, 2..4, 2..3, 3..4, 2..2, 5..7];
    let rows: Vec<_> = bounds
        .into_iter()
        .map(|region| Row {
            ids: full.ids.clone(),
            content: full.content.clone(),
            region,
            labels: Labels::default(),
        })
        .chain([dense_row()])
        .collect();
    let mut evidence = Vec::new();
    for mode in [RoleMode::Identity, RoleMode::Positional] {
        let (_, traces) = launch(
            device,
            &tables,
            &RolePrototypes::default(),
            &rows,
            mode,
            Operation::Signature,
        )?;
        if traces[0]
            .candidates
            .iter()
            .map(|candidate| candidate.span)
            .collect::<Vec<_>>()
            != [[2, 4], [5, 6]]
            || traces[1].candidates[0].feature != Some([0; ROLE_DIMENSIONS])
            || traces[2..5]
                .iter()
                .any(|row| row.candidates.iter().any(|c| c.admitted))
            || traces[5].candidates.iter().filter(|c| c.admitted).count() != 1
            || traces[0]
                .candidates
                .iter()
                .any(|c| c.score.is_none_or(|score| score.winner.is_some()))
            || traces[6]
                .candidates
                .iter()
                .filter_map(|c| c.feature)
                .flatten()
                .all(|x| x != 48)
                && mode == RoleMode::Identity
        {
            return Err(
                "finite multipart, clip, full width, structural slot or zero-bank tie check failed"
                    .into(),
            );
        }
        evidence.push(json!({"mode":mode,"test":"signature", "traces":traces}));

        let fit = Row {
            labels: Labels {
                requested: Some([2, 4]),
                excluded: Some([5, 6]),
            },
            ..full.clone()
        };
        let initial = RolePrototypes::default();
        let train_rows = [fit];
        let output = run_cuda(
            device,
            &tables,
            &initial,
            &train_rows,
            mode,
            Operation::Train,
        )?;
        let (prototype, traces) = compare(
            &output,
            &tables,
            &initial,
            &train_rows,
            mode,
            Operation::Train,
        )?;
        let (decoded_bank, decoded_traces) = decode_train(&output, &tables, &train_rows)?;
        if decoded_bank != prototype
            || serde_json::to_vec(&decoded_traces)? != serde_json::to_vec(&traces)?
        {
            return Err("TRAIN transcript decoder differs from the finite parity view".into());
        }
        if traces.len() != 2
            || traces
                .iter()
                .any(|row| row.candidates.iter().any(|c| c.update.is_none()))
        {
            return Err("finite two ordered passes or labels missing".into());
        }
        let bank = RoleBank::new(mode, prototype, [[0; 32]; 6]);
        let pack_start = Instant::now();
        let bytes = bank.to_bytes();
        if RoleBank::from_bytes(&bytes)?.to_bytes() != bytes {
            return Err("finite reload differs".into());
        }
        eprintln!(
            "finite_bank_pack_reload_ns={}",
            pack_start.elapsed().as_nanos()
        );
        let mut bad = bytes;
        bad[8] = 2;
        if RoleBank::from_bytes(&bad).is_ok() || RoleBank::from_bytes(&bytes[..399]).is_ok() {
            return Err("malformed arm or state length passed".into());
        }
        evidence.push(
            json!({"mode":mode,"test":"ordered_fit","bank_sha256":digest(&bytes),"traces":traces}),
        );

        let mut points = [
            [48; ROLE_DIMENSIONS],
            [0; ROLE_DIMENSIONS],
            [-48; ROLE_DIMENSIONS],
        ];
        [points[1][0], points[1][1]] = [48, -48];
        let clamp = Row {
            ids: vec![0, 2, 3, 2],
            content: 1..4,
            region: 0..4,
            labels: Labels {
                requested: Some([2, 3]),
                excluded: None,
            },
        };
        let (_, traces) = launch(
            device,
            &tables,
            &RolePrototypes::new(points)?,
            &[clamp],
            mode,
            Operation::ClampFixture,
        )?;
        let update = traces[0].candidates[0]
            .update
            .ok_or("missing clamp update")?;
        if !update.applied || update.clamp_low != 1 || update.clamp_high != 1 {
            return Err("both clamp directions must be exercised".into());
        }
        evidence.push(json!({"mode":mode,"test":"clamps","traces":traces}));
    }

    let tie = Row {
        ids: vec![0, 2, 3, 2, 3, 2, 9],
        content: 1..6,
        region: 0..7,
        labels: Labels::default(),
    };
    let initial = RolePrototypes::new([
        [4; ROLE_DIMENSIONS],
        [48; ROLE_DIMENSIONS],
        [-48; ROLE_DIMENSIONS],
    ])?;
    let (_, traces) = launch(
        device,
        &tables,
        &initial,
        &[tie],
        RoleMode::Identity,
        Operation::Public,
    )?;
    if traces[0].decision.requested.is_some() || !traces[0].decision.ties[0] {
        return Err("inter-candidate role tie did not abstain".into());
    }
    evidence.push(json!({"mode":RoleMode::Identity,"test":"inter_candidate_tie","traces":traces}));
    let distractor = Row {
        ids: vec![0, 2, 3, 4, 2, 3, 2, 3, 2, 9],
        content: 1..9,
        region: 1..9,
        labels: Labels::default(),
    };
    let mut wanted = [0; ROLE_DIMENSIONS];
    for block in wanted.chunks_exact_mut(8) {
        block[..2].fill(14);
    }
    let bank = RolePrototypes::new([wanted, [0; ROLE_DIMENSIONS], [-48; ROLE_DIMENSIONS]])?;
    let (_, traces) = launch(
        device,
        &tables,
        &bank,
        &[distractor],
        RoleMode::Identity,
        Operation::Bounded,
    )?;
    let capture = serde_json::from_value::<CapturedRow>(serde_json::to_value(&traces[0])?)?;
    let scores =
        fixed_scores(&capture, [Some([2, 4]), Some([5, 6])])?.ok_or("missing fixed competitor")?;
    verify_fixed_gpu(device, &[scores])?;
    let fixed = Decision::from(rank_roles(scores));
    if traces[0].decision.requested.is_some()
        || !traces[0].decision.ties[0]
        || fixed.requested != Some([5, 6])
        || fixed.ties[0]
    {
        return Err("fixed competitor did not rerank captured bounded features".into());
    }
    evidence.push(
        json!({"mode":RoleMode::Identity,"test":"fixed_competitor","traces":traces,"fixed":fixed}),
    );
    let mut wrong_bounds = full.clone();
    wrong_bounds.region = 2..7;
    let mut wrong_label = full.clone();
    wrong_label.labels.requested = Some([2, 3]);
    for (op, row, expected) in [
        (Operation::Public, wrong_bounds, "full tape eligibility"),
        (
            Operation::Public,
            wrong_label.clone(),
            "only accepted in fit",
        ),
        (Operation::Train, wrong_label, "label did not enumerate"),
    ] {
        let bytes = packet(
            &tables,
            RoleMode::Identity,
            &RolePrototypes::default(),
            &[row],
            op,
        )?;
        let output = execute(device, &bytes)?;
        if output.status.success()
            || !output.stdout.is_empty()
            || !String::from_utf8_lossy(&output.stderr).contains(expected)
        {
            return Err(format!("malformed finite packet did not fail closed: {expected}").into());
        }
    }
    trace_write(
        Path::new(out),
        json!({"schema":"uor-r4.ordered-role-finite/1","results":evidence}),
    )
}

fn timing(args: &[String]) -> Result<()> {
    let [device, count] = args else {
        return Err("timing DEVICE 1|32".into());
    };
    let count: usize = count.parse()?;
    if ![1, 32].contains(&count) {
        return Err("timing count must be one or 32".into());
    }
    let tables = fixture_tables();
    let rows = vec![dense_row(); count];
    for arm in [RoleMode::Identity, RoleMode::Positional] {
        for op in [Operation::Train, Operation::Public] {
            let start = Instant::now();
            launch(device, &tables, &RolePrototypes::default(), &rows, arm, op)?;
            eprintln!(
                "{} mode={arm:?} rows={count} pipeline_ns={}",
                op.name(),
                start.elapsed().as_nanos()
            );
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [command, args @ ..] = args.as_slice() else {
        return Err("expected signature, fit, public, bounded, fixed, eligibility, finite, timing or model-check".into());
    };
    match command.as_str() {
        "signature" => signatures(args),
        "fit" => fit(args),
        "public" => infer(args, false),
        "bounded" => infer(args, true),
        "fixed" => fixed(args),
        "eligibility" => eligibility(args),
        "finite" => finite(args),
        "timing" => timing(args),
        "model-check" => {
            let [model, tokenizer] = args else {
                return Err("model-check MODEL TOKENIZER".into());
            };
            Asset::load(model, tokenizer)?;
            Ok(())
        }
        _ => Err("unknown command".into()),
    }
}
