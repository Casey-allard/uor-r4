//! Time a SpiralCore/FBS stack artifact's integer session on this machine
//! (d11-exporter-float-trunk Stage 4b, claude-126).
//!
//! ```text
//! fbs-toks ARTIFACT [--prompt ID,ID,...] [--stages] [--parts]
//! ```
//!
//! The protocol follows the step-6 one (`docs/labs/m1-d11-artifact-scorer`,
//! claude-116), fitted to a session that cannot slide its window: 1 warmup and
//! 20 timed runs, each in a fresh session, of a 128-token prefill (context)
//! and then 64 greedy decode steps, each feeding the token the writer chose.
//! The two together fill a 192-token context. It records the raw durations in
//! integer nanoseconds, the median (the integer mean of the middle two for an
//! even count), the nearest-rank p95 (rank ceil(0.95 n)), integer tokens per
//! second for each, the reply, and the hardware profile. It refuses an
//! artifact whose context is shorter than 192, a prompt of the wrong length,
//! and runs that write different replies.
//!
//! The served path is `IntegerStackSession::step_native`; argument parsing,
//! timing, statistics and JSON are outside it. It prints one JSON line.
//!
//! `--stages` (fbs-cost-profile, claude-158) runs the protocol a second time
//! through `IntegerStackSession::measure_fbs_step_stages` and adds `stages`:
//! each stage's median over the timed decode steps, their sum against the
//! step, a whole-table nearest search's median beside the writer, and the
//! step against the plain protocol's per-token time. The plain protocol's
//! numbers are taken first and unchanged.
//!
//! `--parts` (fbs-serving-kernels, claude-163) implies `--stages` and runs the
//! protocol a third time through `IntegerStackSession::measure_fbs_step_parts`,
//! which splits each layer's two blocks into nine parts. It adds `parts`: the
//! same profile by part, and under `against_blocks` each block's parts summed
//! against that block's median from the `--stages` run, per mille.
//!
//! The Hamming read's trace (integer-hamming-read, claude-235):
//! - `--hamming-trace D11_READOUT_JSON TRACE_JSON` steps a fresh session over each sequence of
//!   the float D11 readout (`fbs-snapshot`'s) and writes what each read chose
//!   (`uor-r4.hamming-trace/1`), as `fbs-snapshot --hamming-trace` writes the float's;
//! - `--hamming-trace-compare FIRST SECOND` holds two traces side by side
//!   (`compare_hamming_traces`) and prints the comparison;
//! - `--copy-codes FLOAT_TRACE INTEGER_TRACE OUT_JSON` (claude-237) compares every vocabulary
//!   entry's copy-head code three ways (`copy_code_check`): the CPU float control's and the
//!   session's on the artifact's own key rotations and writer table, and the two traces' at
//!   each position, matched by token, each trace also against the control with the rotations
//!   snapped to the icosians, as the GPU's D11 forward snaps them. It writes
//!   `uor-r4.copy-codes/1` new to `OUT_JSON`;
//! - `--context-sweep` runs the protocol with prefills of 32, 64, 96 and 128 tokens, each
//!   followed by the 64 decode steps, with the artifact's Hamming read and without it: decode
//!   tokens per second against context, the read's long-context cost;
//! - `--context-sweep --tokens TOKENS_U16` (claude-242) runs the same protocol at prefills of
//!   128, 512, 2,048, 8,192 and 32,704 tokens, over the first 32,704 ids of a `UORT` token
//!   file, so the contexts end at 192 to 32,768. It times the points the artifact's context holds
//!   and names those it skips (claude-288); it refuses one that holds none.
//!
//! `--forced PROMPTS_JSON --out FILE` (integer-nearest-readout, claude-244) reads the nearest
//! head teacher-forced: each panel prompt is stepped as `--replies` steps it, then its reference
//! is fed one token at a time, and each offset records the pick, the whole writer table's
//! runner-up and both scores at Q12. It refuses a point-key artifact and one whose head is not
//! the nearest one, and writes the rows new to `FILE`.
//!
//! Q5's trace, the integer side R (claude-249), both options of `--forced`:
//! - `--decision-trace TRACE_JSON` writes, new, every decision of claude-247's five kinds at each
//!   step of each row run (`uor-r4.decision-trace/1`, side `R`): each layer's rate buckets and
//!   the read after it, the readout read, the turn memory's lookup, and from the prompt's last
//!   step on the nearest, with R's Q12 query. The session refuses an artifact off R's path.
//! - `--force FORCE_JSON` (`uor-r4.decision-force/1`) runs only the rows it names, each under
//!   its own forced rate buckets and codes. The rows written keep `--forced`'s format.
//!
//! Q5's near-tie grader (claude-254), on the laptop, on traces any side writes. A table is the
//! side's own: R's writer table from R's artifact, T's the same divided by 4096, F's the `embed`
//! of its `fbs-snapshot` JSON.
//! - `--decision-deciles TRACE_JSON OUT_JSON` writes, new, G₁₀(φ, P) for every family of a
//!   reference side's (F or T) unforced trace of a cell (`uor-r4.decision-deciles/1`).
//! - `--decision-check TRACE_JSON --table TABLE --out OUT_JSON` checks a whole unforced trace
//!   as the grader checks each side: every entry is the one its score selects, every float code
//!   is its coordinates' signs, and the table reproduces every nearest's two scores
//!   (`uor-r4.decision-check/1`).
//! - `--decision-grade REFERENCE OTHER --deciles DECILES_JSON --g10 G --reference-table TABLE
//!   --other-table TABLE [--force FORCE_JSON] [--at POSITION:OFFSET]... --out OUT_JSON` grades
//!   one pass of the forced replay (`grade_pass`) at each chosen position, or, without `--at`,
//!   at every nearest of each of OTHER's windows, with each window's forced set from
//!   `FORCE_JSON` (none without it), and writes `uor-r4.decision-grade/1` new.
//! - `ARTIFACT --decision-replay REFERENCE --prompts PROMPTS_JSON --deciles DECILES_JSON --g10 G
//!   --reference-table TABLE [--at POSITION:OFFSET]... --out OUT_JSON` (claude-256) runs the
//!   whole forced replay (`replay_step`) of the arithmetic step (T, R), R from ARTIFACT as Q, at
//!   each chosen position, or, without `--at`, at every offset of REFERENCE's windows inside R's
//!   (the row's prompt and reference) where R's unforced pick is not the reference's, the rest
//!   counted equal. A float window reads on past the reference; its picks there are counted
//!   beyond R's window, not compared. Each pass steps the row's prompt and its reference up to
//!   the offset under Φ, as `--forced --force` steps a row. It
//!   writes `uor-r4.decision-replay/1` new: each replay's passes, Φ, and whether it accounts for
//!   its step.
//! - Both refuse an `--at` named twice and a deciles file that names no trace.

use std::process::{Command, ExitCode};
use std::time::Instant;

use serde::Deserialize;
use serde_json::json;
use uor_r4_integer::fbs_snapshot::{
    check_decision_trace, compare_hamming_traces, copy_code_check, copy_code_reads,
    decision_deciles, grade_pass, replay_step, write_new_file, DecisionDeciles, DecisionForce,
    DecisionForceWindow, DecisionNearest, DecisionSource, DecisionStep, DecisionTrace,
    DecisionWindow, FbsFloatReadout, FbsReadoutMode, GradeSide, HammingTrace, NearestTable,
    PassOutcome, StepReplay, COPY_CODES_SCHEMA, DECISION_CHECK_SCHEMA, DECISION_DECILES_SCHEMA,
    DECISION_FORCE_SCHEMA, DECISION_GRADE_SCHEMA, DECISION_REPLAY_SCHEMA, DECISION_TRACE_SCHEMA,
    HAMMING_TRACE_SCHEMA,
};
use uor_r4_integer::stack::{
    hamming_trace_window, panel_reply, read_hamming_uorlut01, served_copy_codes_uorlut01,
    IntegerStackModel, ReplyEnd,
};

const USAGE: &str = "usage: fbs-toks ARTIFACT [--prompt ID,ID,...] [--stages] [--parts]\n       fbs-toks ARTIFACT --replies PROMPTS_JSON --out FILE [--reply-tokens N]\n       fbs-toks ARTIFACT --forced PROMPTS_JSON --out FILE [--decision-trace TRACE_JSON] [--force FORCE_JSON]\n       fbs-toks ARTIFACT --hamming-trace D11_READOUT_JSON TRACE_JSON\n       fbs-toks --hamming-trace-compare FIRST_TRACE SECOND_TRACE\n       fbs-toks ARTIFACT --copy-codes FLOAT_TRACE INTEGER_TRACE OUT_JSON\n       fbs-toks ARTIFACT --context-sweep [--tokens TOKENS_U16]\n       fbs-toks --decision-deciles TRACE_JSON OUT_JSON\n       fbs-toks --decision-check TRACE_JSON --table TABLE --out OUT_JSON\n       fbs-toks --decision-grade REFERENCE_TRACE OTHER_TRACE --deciles DECILES_JSON --g10 G --reference-table TABLE --other-table TABLE [--force FORCE_JSON] [--at POSITION:OFFSET]... --out OUT_JSON\n       fbs-toks ARTIFACT --decision-replay REFERENCE_TRACE --prompts PROMPTS_JSON --deciles DECILES_JSON --g10 G --reference-table TABLE [--at POSITION:OFFSET]... --out OUT_JSON";

/// The prefills `--context-sweep` times, each followed by `DECODE` steps (claude-235).
const SWEEP_PREFILLS: [usize; 4] = [32, 64, 96, 128];

/// The prefills `--context-sweep --tokens` times (claude-242), each followed by `DECODE` steps,
/// so the contexts end at 192, 576, 2,112, 8,256 and 32,768.
const LONG_SWEEP_PREFILLS: [usize; 5] = [128, 512, 2_048, 8_192, 32_704];

/// `--hamming-trace` (claude-235): each read's choices in a fresh session over each sequence of
/// the float D11 readout at `readout`, written new to `out`; it prints the window count and the
/// file's sha256.
fn hamming_trace(
    model: &IntegerStackModel,
    bytes: &[u8],
    readout: &str,
    out: &str,
) -> Result<(), String> {
    let image = read_hamming_uorlut01(bytes)
        .map_err(|e| format!("the Hamming reads: {e}"))?
        .ok_or("the artifact has no Hamming read to trace")?;
    let text = std::fs::read(readout).map_err(|e| format!("{readout}: {e}"))?;
    let readout: FbsFloatReadout =
        serde_json::from_slice(&text).map_err(|e| format!("{readout}: {e}"))?;
    if readout.mode != FbsReadoutMode::D11 {
        return Err(format!(
            "the readout was read in the {:?} mode, not D11",
            readout.mode
        ));
    }
    let mut windows = Vec::with_capacity(readout.sequences.len());
    for (i, sequence) in readout.sequences.iter().enumerate() {
        let mut session = model.session();
        session.set_hamming_trace(true);
        for &token in &sequence.tokens {
            session
                .step_native(token)
                .map_err(|e| format!("sequence {i}: {e}"))?;
        }
        let steps = session.take_hamming_trace().ok_or("the trace was off")?;
        windows.push(
            hamming_trace_window(&sequence.tokens, &steps)
                .map_err(|e| format!("sequence {i}: {e}"))?,
        );
    }
    let trace = HammingTrace {
        schema: HAMMING_TRACE_SCHEMA.to_owned(),
        side: "integer".to_owned(),
        arm: "integer".to_owned(),
        heads: image.record.heads,
        kept: image.record.kept,
        windows,
    };
    let text = serde_json::to_vec(&trace).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "windows": trace.windows.len(), "out": out, "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&text)) })
    );
    Ok(())
}

/// `--hamming-trace-compare` (claude-235): the comparison of the trace at `first` with the one
/// at `second`, as one JSON line.
fn hamming_trace_compare(first: &str, second: &str) -> Result<(), String> {
    let read = |path: &str| -> Result<HammingTrace, String> {
        let text = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        serde_json::from_slice(&text).map_err(|e| format!("{path}: {e}"))
    };
    let (a, b) = (read(first)?, read(second)?);
    let comparison = compare_hamming_traces(&a, &b)?;
    println!(
        "{}",
        json!({
            "schema": "uor-r4.hamming-trace-compare/1",
            "first": { "path": first, "side": a.side, "arm": a.arm },
            "second": { "path": second, "side": b.side, "arm": b.arm },
            "comparison": comparison,
        })
    );
    Ok(())
}

/// A file's sha256, in hex.
fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(<sha2::Sha256 as sha2::Digest>::digest(bytes))
}

/// The decision trace at `path` and its file's sha256.
fn read_decision_trace(path: &str) -> Result<(DecisionTrace, String), String> {
    let text = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let trace = serde_json::from_slice(&text).map_err(|e| format!("{path}: {e}"))?;
    Ok((trace, sha256_hex(&text)))
}

/// A side's own nearest table, owned (claude-247): R's writer table at Q12 and T's, that table
/// divided by 4096, from R's artifact; F's float table, its snapshot's `embed`.
enum OwnedTable {
    Q12(Vec<i16>, usize),
    F32(Vec<f32>, usize),
}

impl OwnedTable {
    fn view(&self) -> NearestTable<'_> {
        match self {
            OwnedTable::Q12(table, width) => NearestTable::Q12 {
                table,
                width: *width,
            },
            OwnedTable::F32(table, width) => NearestTable::F32 {
                table,
                width: *width,
            },
        }
    }
}

/// Side `side`'s own table from the file at `path`, and the file's sha256.
fn load_table(side: &str, path: &str) -> Result<(OwnedTable, String), String> {
    if !["F", "R", "T"].contains(&side) {
        return Err(format!("{path}: side {side} has no table; F, R and T do"));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let sha = sha256_hex(&bytes);
    let table = if side == "F" {
        #[derive(Deserialize)]
        struct Embed {
            vocab: usize,
            width: usize,
            embed: Vec<f32>,
        }
        let snapshot: Embed = serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
        if snapshot.embed.len() != snapshot.vocab * snapshot.width {
            return Err(format!(
                "{path}: the embed is not {} rows of {}",
                snapshot.vocab, snapshot.width
            ));
        }
        OwnedTable::F32(snapshot.embed, snapshot.width)
    } else {
        let served = served_copy_codes_uorlut01(&bytes)
            .map_err(|e| format!("{path}: the Hamming reads: {e}"))?
            .ok_or_else(|| {
                format!("{path}: the artifact has no Hamming read to take its writer table from")
            })?;
        if served.table.len() != served.vocab * served.width {
            return Err(format!(
                "{path}: the writer table is not {} rows of {}",
                served.vocab, served.width
            ));
        }
        match side {
            "R" => OwnedTable::Q12(served.table, served.width),
            "T" => OwnedTable::F32(
                served
                    .table
                    .iter()
                    .map(|&v| f32::from(v) / 4096.0)
                    .collect(),
                served.width,
            ),
            _ => return Err(format!("{path}: side {side} has no table; F, R and T do")),
        }
    };
    Ok((table, sha))
}

/// `--decision-deciles` (claude-254): G₁₀(φ, P) for every family of the reference side's
/// unforced trace at `trace`, written new to `out`; it prints the trace's and the file's sha256.
fn decision_deciles_out(trace: &str, out: &str) -> Result<(), String> {
    let (decisions, sha) = read_decision_trace(trace)?;
    let mut deciles = decision_deciles(&decisions)?;
    deciles.trace = Some(DecisionSource {
        path: trace.to_owned(),
        sha256: sha.clone(),
    });
    let text = serde_json::to_vec_pretty(&deciles).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({
            "schema": DECISION_DECILES_SCHEMA,
            "trace": { "path": trace, "sha256": sha, "side": decisions.side },
            "families": deciles.families.len(),
            "out": out,
            "sha256": sha256_hex(&text),
        })
    );
    Ok(())
}

/// `--decision-check` (claude-254): `check_decision_trace` on the unforced trace at `trace`
/// with its side's own table at `table`, written new to `out`.
fn decision_check_out(trace: &str, table: &str, out: &str) -> Result<(), String> {
    let (decisions, sha) = read_decision_trace(trace)?;
    let (owned, table_sha) = load_table(&decisions.side, table)?;
    let check = check_decision_trace(&decisions, owned.view())?;
    let record = json!({
        "schema": DECISION_CHECK_SCHEMA,
        "trace": { "path": trace, "sha256": sha, "side": decisions.side },
        "table": { "path": table, "sha256": table_sha },
        "check": check,
    });
    let text = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "schema": DECISION_CHECK_SCHEMA, "check": check, "out": out, "sha256": sha256_hex(&text) })
    );
    Ok(())
}

/// The named flags with their values, and each `--at POSITION:OFFSET` in order (claude-256).
type DecisionFlags<'a> = (
    std::collections::BTreeMap<&'a str, &'a str>,
    Vec<(usize, usize)>,
);

/// The flags after `--decision-grade`'s or `--decision-replay`'s traces (claude-256): each of
/// `names` at most once with its value, and `--at POSITION:OFFSET` any number of times, none
/// named twice. Anything else is the usage.
fn decision_flags<'a>(rest: &'a [String], names: &[&str]) -> Result<DecisionFlags<'a>, String> {
    let (mut flags, mut at) = (std::collections::BTreeMap::new(), Vec::new());
    let mut rest = rest.iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or_else(|| USAGE.to_owned())?;
        if flag == "--at" {
            let parsed = value
                .split_once(':')
                .and_then(|(p, o)| Some((p.parse::<usize>().ok()?, o.parse::<usize>().ok()?)));
            at.push(parsed.ok_or_else(|| format!("--at {value}: not POSITION:OFFSET"))?);
        } else if !names.contains(&flag.as_str())
            || flags.insert(flag.as_str(), value.as_str()).is_some()
        {
            return Err(USAGE.to_owned());
        }
    }
    let mut named = at.clone();
    named.sort_unstable();
    if let Some(p) = named.windows(2).find(|p| p[0] == p[1]) {
        return Err(format!("--at {}:{} is named twice", p[0].0, p[0].1));
    }
    Ok((flags, at))
}

/// Flag `name`'s value in `flags`, which the command needs.
fn needed<'a>(
    flags: &std::collections::BTreeMap<&str, &'a str>,
    name: &str,
) -> Result<&'a str, String> {
    flags.get(name).copied().ok_or_else(|| USAGE.to_owned())
}

/// The deciles in the file at `path`, whose bytes are `text` (claude-254). A file that names no
/// trace its G₁₀s were read from is refused (claude-256): no grade could say whose they are.
fn parse_deciles(path: &str, text: &[u8]) -> Result<DecisionDeciles, String> {
    let deciles: DecisionDeciles =
        serde_json::from_slice(text).map_err(|e| format!("{path}: {e}"))?;
    if deciles.trace.is_none() {
        return Err(format!(
            "{path}: the deciles name no trace they were read from"
        ));
    }
    Ok(deciles)
}

/// The step at offset `offset` of `window`'s reply, refused past the window's steps.
fn offset_step(window: &DecisionWindow, offset: usize) -> Result<usize, String> {
    window
        .reply_start
        .checked_add(offset)
        .filter(|&s| s < window.steps.len())
        .ok_or_else(|| {
            format!(
                "position {}, offset {offset}: past the window's {} steps",
                window.position,
                window.steps.len()
            )
        })
}

/// `--decision-grade` (claude-254): one pass of the forced replay (`grade_pass`) at each chosen
/// position, written new as `uor-r4.decision-grade/1`. `args` are the whole command line.
fn decision_grade_out(args: &[String]) -> Result<(), String> {
    let [_, reference, other, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let names = [
        "--deciles",
        "--g10",
        "--reference-table",
        "--other-table",
        "--force",
        "--out",
    ];
    let (flags, at) = decision_flags(rest, &names)?;
    let (deciles_path, g10, reference_table, other_table, out) = (
        needed(&flags, "--deciles")?,
        needed(&flags, "--g10")?,
        needed(&flags, "--reference-table")?,
        needed(&flags, "--other-table")?,
        needed(&flags, "--out")?,
    );
    let force = flags.get("--force").copied();
    let g10: i64 = g10.parse().map_err(|e| format!("--g10 {g10}: {e}"))?;
    let (rt, reference_sha) = read_decision_trace(reference)?;
    let (ot, other_sha) = read_decision_trace(other)?;
    let deciles_text = std::fs::read(deciles_path).map_err(|e| format!("{deciles_path}: {e}"))?;
    let deciles = parse_deciles(deciles_path, &deciles_text)?;
    let (rtab, rtab_sha) = load_table(&rt.side, reference_table)?;
    let (otab, otab_sha) = load_table(&ot.side, other_table)?;
    let (phi, force_record) = match force {
        None => (Vec::new(), serde_json::Value::Null),
        Some(path) => {
            let text = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            let set: DecisionForce =
                serde_json::from_slice(&text).map_err(|e| format!("{path}: {e}"))?;
            if set.schema != DECISION_FORCE_SCHEMA {
                return Err(format!(
                    "{path}: the schema is {}, not {DECISION_FORCE_SCHEMA}",
                    set.schema
                ));
            }
            let mut positions: Vec<usize> = set.windows.iter().map(|w| w.position).collect();
            positions.sort_unstable();
            if positions.windows(2).any(|p| p[0] == p[1]) {
                return Err(format!("{path}: a position has two forced sets"));
            }
            (
                set.windows,
                json!({ "path": path, "sha256": sha256_hex(&text) }),
            )
        }
    };
    // Each chosen position, or every nearest of each of the other side's windows, as offsets.
    let chosen: Vec<(usize, usize)> = if at.is_empty() {
        ot.windows
            .iter()
            .flat_map(|w| {
                let steps = w.steps.iter().enumerate().skip(w.reply_start);
                steps
                    .filter(|(_, s)| s.nearest.is_some())
                    .map(|(t, _)| (w.position, t - w.reply_start))
            })
            .collect()
    } else {
        at
    };
    let sides = (
        GradeSide {
            trace: &rt,
            table: rtab.view(),
        },
        GradeSide {
            trace: &ot,
            table: otab.view(),
        },
    );
    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    let mut passes = Vec::with_capacity(chosen.len());
    for (position, offset) in chosen {
        let place = format!("position {position}, offset {offset}");
        let window = ot
            .windows
            .iter()
            .find(|w| w.position == position)
            .ok_or_else(|| format!("{place}: the other trace has no such window"))?;
        let step = offset_step(window, offset)?;
        let none = DecisionForceWindow {
            position,
            ..Default::default()
        };
        let forced = phi.iter().find(|f| f.position == position).unwrap_or(&none);
        let outcome = grade_pass(sides.0, sides.1, &deciles, position, step, forced, g10)
            .map_err(|e| format!("{place}: {e}"))?;
        let (kind, near_tie) = match &outcome {
            PassOutcome::Bucket { near_tie, .. } => ("bucket", *near_tie),
            PassOutcome::Code { near_tie, .. } => ("code", *near_tie),
            PassOutcome::Kept { .. } => ("kept", false),
            PassOutcome::Memory { .. } => ("memory", false),
            PassOutcome::Nearest {
                near_tie,
                reference_pick,
                other_pick,
                ..
            } => {
                if reference_pick == other_pick {
                    *counts.entry("nearest_equal").or_default() += 1;
                }
                ("nearest", *near_tie)
            }
        };
        *counts.entry(kind).or_default() += 1;
        *counts.entry("near_tie").or_default() += usize::from(near_tie);
        passes.push(json!({
            "position": position,
            "offset": offset,
            "step": step,
            "forced": forced.buckets.len() + forced.codes.len(),
            "outcome": outcome,
        }));
    }
    let record = json!({
        "schema": DECISION_GRADE_SCHEMA,
        "reference": { "path": reference, "sha256": reference_sha, "side": rt.side },
        "other": { "path": other, "sha256": other_sha, "side": ot.side },
        "reference_table": { "path": reference_table, "sha256": rtab_sha },
        "other_table": { "path": other_table, "sha256": otab_sha },
        // The cell's whole trace the deciles were read from, which may be more than `reference`.
        "deciles": { "path": deciles_path, "sha256": sha256_hex(&deciles_text), "trace": deciles.trace },
        "force": force_record,
        "g10": g10,
        "passes": passes.len(),
        "counts": counts,
        "graded": passes,
    });
    let text = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "schema": DECISION_GRADE_SCHEMA, "passes": record["passes"], "counts": record["counts"], "out": out, "sha256": sha256_hex(&text) })
    );
    Ok(())
}

/// `--copy-codes` (claude-237): every vocabulary entry's copy-head code three ways
/// (`copy_code_check`), on the artifact at `path` and the traces at `float_trace` and
/// `integer_trace`, written new to `out`. Everything is read and checked before anything is
/// compared or written. It prints one JSON line: the file's sha256 and, per read, the codes on
/// which the control and the session agree, the float trace's query codes equal to each and to
/// the snapped control, the integer trace's equal to its session, and each trace's
/// inconsistent pairs.
///
/// This checker is defined for copy key 1: its per-token control and trace reconstruction assumes
/// one token per copy code. A two-token key, or a key per head, is refused by its header before
/// any comparison.
fn copy_codes(path: &str, float_trace: &str, integer_trace: &str, out: &str) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    require_one_token_copy_codes(&bytes)?;
    IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
    let served = served_copy_codes_uorlut01(&bytes)
        .map_err(|e| format!("the Hamming reads: {e}"))?
        .ok_or("the artifact has no Hamming read to check")?;
    let (vocab, width, heads) = (served.vocab, served.width, served.image.record.heads);
    let read_trace = |path: &str| -> Result<(HammingTrace, Vec<u8>), String> {
        let text = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let trace = serde_json::from_slice(&text).map_err(|e| format!("{path}: {e}"))?;
        Ok((trace, text))
    };
    let (float, float_text) = read_trace(float_trace)?;
    let (integer, integer_text) = read_trace(integer_trace)?;
    let inputs: Vec<(usize, &[i32], Vec<u64>)> = served
        .image
        .reads
        .iter()
        .zip(&served.codes)
        .map(|(read, codes)| (read.depth, read.key_rotation.as_slice(), codes.clone()))
        .collect();
    let rotation_exp = served.image.record.rotation_exp;
    let reads = copy_code_reads(&inputs, rotation_exp, &served.table, vocab, width, heads)?;
    let bytes = bytes.as_slice();
    let check = copy_code_check(vocab, width, heads, &reads, &float, &integer)?;
    let sha = |b: &[u8]| hex::encode(<sha2::Sha256 as sha2::Digest>::digest(b));
    let record = json!({
        "schema": COPY_CODES_SCHEMA,
        "artifact": { "path": path, "sha256": sha(bytes) },
        "float_trace": { "path": float_trace, "sha256": sha(&float_text), "arm": float.arm },
        "integer_trace": { "path": integer_trace, "sha256": sha(&integer_text), "arm": integer.arm },
        "width": width,
        "check": check,
    });
    let text = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    let summary: Vec<_> = check
        .reads
        .iter()
        .map(|r| {
            json!({
                "depth": r.depth,
                "codes": r.control_vs_integer.codes,
                "control_equal_integer": r.control_vs_integer.equal,
                "float_query_equal_control": r.float.query.equal_control,
                "float_query_equal_integer": r.float.query.equal_integer,
                "float_query_equal_snapped": r.float.query.equal_snapped,
                "float_query_samples": r.float.query.samples,
                "integer_query_equal_integer": r.integer.query.equal_integer,
                "inconsistent": [r.float.inconsistent, r.integer.inconsistent],
            })
        })
        .collect();
    println!(
        "{}",
        json!({ "out": out, "sha256": sha(&text), "reads": summary })
    );
    Ok(())
}

/// The Hamming record in a `UORLUT01` header, `None` when the artifact has none. Only this header
/// is read, so the guards below also work with an artifact whose newer record shape is not yet
/// known to this binary.
fn artifact_hamming(
    bytes: &[u8],
) -> Result<Option<serde_json::Map<String, serde_json::Value>>, String> {
    if bytes.len() < 16 || &bytes[..8] != b"UORLUT01" {
        return Err("the artifact is not a UORLUT01 container".to_owned());
    }
    let len = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| "the artifact header length is truncated")?,
    );
    let len = usize::try_from(len).map_err(|_| "the artifact header is too large")?;
    let end = 16usize
        .checked_add(len)
        .filter(|&end| end <= bytes.len())
        .ok_or("the artifact header is truncated")?;
    let header: serde_json::Value =
        serde_json::from_slice(&bytes[16..end]).map_err(|e| format!("the artifact header: {e}"))?;
    let Some(hamming) = header.get("hamming") else {
        return Ok(None);
    };
    let hamming = hamming
        .as_object()
        .ok_or("the artifact's Hamming record is not an object")?;
    Ok(Some(hamming.clone()))
}

/// The copy-key number in a `UORLUT01` header, defaulting to 1 when a one-token artifact leaves
/// the optional field out.
fn artifact_copy_key(bytes: &[u8]) -> Result<usize, String> {
    let Some(hamming) = artifact_hamming(bytes)? else {
        return Ok(1);
    };
    match hamming.get("copy_key") {
        None => Ok(1),
        Some(value) => value
            .as_u64()
            .and_then(|key| usize::try_from(key).ok())
            .ok_or_else(|| {
                "the artifact's Hamming copy key is not a non-negative integer".to_owned()
            }),
    }
}

/// Refuse `--copy-codes` on any artifact whose copy heads do not all use the one-token key its
/// per-token reconstruction expects: a copy key other than 1, or a key per head (`copy_keys`,
/// written only when the heads' keys differ, so one of them is two-token; claude-269).
fn require_one_token_copy_codes(bytes: &[u8]) -> Result<(), String> {
    if let Some(keys) = artifact_hamming(bytes)?.and_then(|h| h.get("copy_keys").cloned()) {
        return Err(format!(
            "--copy-codes audits the one-token copy key only; the artifact records copy_keys {keys}"
        ));
    }
    let copy_key = artifact_copy_key(bytes)?;
    if copy_key != 1 {
        return Err(format!(
            "--copy-codes audits the one-token copy key only; the artifact records copy_key {copy_key}"
        ));
    }
    Ok(())
}

/// `--context-sweep` (claude-235): the protocol at each of `prefills` over the prompt's first
/// tokens, on `model` and on the same model without its Hamming read.
fn context_sweep(
    model: &IntegerStackModel,
    without: &IntegerStackModel,
    prompt: &[u32],
    prefills: &[usize],
) -> Result<serde_json::Value, String> {
    let mut points = Vec::with_capacity(prefills.len());
    for &prefill in prefills {
        let prompt = &prompt[..prefill];
        points.push(json!({
            "prefill": prefill,
            "context_end": prefill + DECODE,
            "with_read": protocol(|| model.session(), prompt)?,
            "without_read": protocol(|| without.session(), prompt)?,
        }));
    }
    Ok(json!(points))
}

/// The `LONG_SWEEP_PREFILLS` an artifact of `context` positions holds with their decode, and
/// those it cannot (claude-288): a carry export at 8,256 is swept to 8,256, not refused.
fn long_sweep_points(context: usize) -> (Vec<usize>, Vec<usize>) {
    LONG_SWEEP_PREFILLS
        .iter()
        .partition(|&&prefill| prefill + DECODE <= context)
}

/// The prompt `--context-sweep --tokens` sweeps (claude-242): a `UORT` token file's first ids, as
/// many as the largest of `LONG_SWEEP_PREFILLS` the artifact's `context` holds with its decode
/// (claude-288). The file is read as `fbs-export --point-key` reads one: a 64-byte header
/// starting `UORT`, whose count at bytes 8 to 16 must match the payload, then little-endian `u16`
/// ids. It refuses an artifact whose `context` holds no prefill and its decode, a file that is
/// not `UORT` or whose count disagrees, one with too few ids, and an id outside `vocab`.
fn long_sweep_prompt(bytes: &[u8], vocab: usize, context: usize) -> Result<Vec<u32>, String> {
    let Some(&largest) = long_sweep_points(context).0.last() else {
        return Err(format!(
            "the artifact's context {context} is shorter than {} + {DECODE}",
            LONG_SWEEP_PREFILLS[0]
        ));
    };
    if bytes.len() < 64 || &bytes[..4] != b"UORT" {
        return Err("the token file is not a UORT token file".to_owned());
    }
    let mut count = [0u8; 8];
    count.copy_from_slice(&bytes[8..16]);
    let count = u64::from_le_bytes(count);
    let payload = &bytes[64..];
    if usize::try_from(count).ok().and_then(|c| c.checked_mul(2)) != Some(payload.len()) {
        return Err(format!(
            "the token file's header counts {count} ids, its payload holds {} bytes",
            payload.len()
        ));
    }
    if payload.len() / 2 < largest {
        return Err(format!(
            "the token file holds {} ids, fewer than {largest}",
            payload.len() / 2
        ));
    }
    let prompt: Vec<u32> = payload[..2 * largest]
        .chunks_exact(2)
        .map(|b| u32::from(u16::from_le_bytes([b[0], b[1]])))
        .collect();
    if let Some((i, &t)) = prompt
        .iter()
        .enumerate()
        .find(|(_, &t)| t as usize >= vocab)
    {
        return Err(format!(
            "token {t} at {i} is outside the vocabulary {vocab}"
        ));
    }
    Ok(prompt)
}

/// One panel prompt for `--replies` (claude-224), as `train-in-space --panel-prompts` writes it:
/// the reply's corpus position, the prompt's ids, the reference reply, and optionally the float
/// side's served name at the opening.
#[derive(Deserialize)]
struct PanelPrompt {
    position: usize,
    prompt: Vec<u32>,
    reference: Vec<u32>,
    #[serde(default)]
    float_keyed: Option<u32>,
}

/// `--replies` (point-key-integer unit 2, claude-224): each prompt's greedy reply on this
/// artifact (`panel_reply`), written new to `out` as JSON. Each sample holds the opening's
/// point-key name and least distance at Q12, where the key served, and the float name's least
/// distance. It prints one JSON line: rows, keyed openings, capped replies and the file's sha256.
/// Each keyed opening matched the reference scan (claude-228): `panel_reply` stops the decode on
/// any that does not, so `reference_checked` is the keyed count of a decode that finished.
fn replies(
    model: &IntegerStackModel,
    prompts: &str,
    out: &str,
    reply_tokens: usize,
) -> Result<(), String> {
    let bytes = std::fs::read(prompts).map_err(|e| format!("{prompts}: {e}"))?;
    let rows: Vec<PanelPrompt> =
        serde_json::from_slice(&bytes).map_err(|e| format!("{prompts}: {e}"))?;
    let (mut samples, mut keyed, mut capped) = (Vec::with_capacity(rows.len()), 0usize, 0usize);
    for row in &rows {
        let reply = panel_reply(model, &row.prompt, reply_tokens, row.float_keyed)
            .map_err(|e| format!("position {}: {e}", row.position))?;
        keyed += usize::from(reply.keyed.is_some());
        capped += usize::from(reply.end == ReplyEnd::Context);
        samples.push(json!({
            "position": row.position,
            "prompt": row.prompt,
            "reply": reply.reply,
            "reference": row.reference,
            "keyed": reply.keyed.map(|k| k.0),
            "served_distance_q12": reply.keyed.map(|k| k.1),
            "float_keyed": row.float_keyed,
            "float_distance_q12": reply.float_distance,
            "end": format!("{:?}", reply.end).to_lowercase(),
        }));
    }
    let text = serde_json::to_vec(&samples).map_err(|e| e.to_string())?;
    uor_r4_integer::fbs_snapshot::write_new_file(std::path::Path::new(out), &text)
        .map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "rows": rows.len(), "keyed": keyed, "reference_checked": keyed, "capped": capped, "out": out, "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&text)) })
    );
    Ok(())
}

/// The artifact `--forced` reads (claude-244): one with no point key, whose head is the nearest
/// one, so that each step's pick is the whole writer table's nearest row.
fn forced_artifact(model: &IntegerStackModel) -> Result<(), String> {
    if model.point_key_rows().is_some() {
        return Err(
            "--forced reads the nearest head, and the artifact serves a point key".to_owned(),
        );
    }
    if !model.shape().head_nearest() {
        return Err(
            "--forced reads the nearest head, and the artifact's head is not it".to_owned(),
        );
    }
    Ok(())
}

/// One row's teacher-forced top two (claude-244): at each offset the pick, the runner-up and their
/// scores at Q12, over `n` offsets; `capped` when the context cut the reference short.
#[derive(Debug)]
struct Forced {
    picks: Vec<u32>,
    runner_up: Vec<u32>,
    score_q12: Vec<i64>,
    runner_score_q12: Vec<i64>,
    capped: bool,
    /// Under a decision trace (claude-249), each step's decisions, the nearest filled in from the
    /// prompt's last step on.
    steps: Option<Vec<DecisionStep>>,
}

/// `prompt` stepped as `panel_reply` steps it, then the `reference` fed one token at a time
/// (claude-244). Offset 0 reads the step after the prompt's last token; offset `i` the step after
/// `reference[i - 1]`. Each offset's pick must be the whole table's nearest row. Under `force`
/// the session records its decisions and forces those `force` names (claude-249).
fn forced_picks(
    model: &IntegerStackModel,
    prompt: &[u32],
    reference: &[u32],
    force: Option<&DecisionForceWindow>,
) -> Result<Forced, String> {
    let (vocab, context) = (model.shape().vocab, model.shape().context);
    if prompt.is_empty() || prompt.len() > context {
        return Err(format!(
            "the prompt has {} tokens, not 1 to {context}",
            prompt.len()
        ));
    }
    if reference.is_empty() {
        return Err("the reference is empty".to_owned());
    }
    let n = reference.len().min(context - prompt.len() + 1);
    let fed = prompt.iter().chain(&reference[..n - 1]);
    if let Some(&t) = fed.clone().find(|&&t| t as usize >= vocab) {
        return Err(format!("token {t} is outside the vocabulary {vocab}"));
    }
    let mut session = model.session();
    if let Some(force) = force {
        session
            .set_decision_trace(force)
            .map_err(|e| format!("the decision trace: {e}"))?;
    }
    let mut step = None;
    for (row, &token) in prompt.iter().enumerate() {
        if row + 1 == prompt.len() {
            session.mark_reply_start();
        }
        step = Some(session.step_native(token).map_err(|e| e.to_string())?);
    }
    let mut step = step.ok_or("the prompt took no step")?;
    let mut forced = Forced {
        picks: Vec::with_capacity(n),
        runner_up: Vec::with_capacity(n),
        score_q12: Vec::with_capacity(n),
        runner_score_q12: Vec::with_capacity(n),
        capped: reference.len() > n,
        steps: None,
    };
    let mut queries = Vec::with_capacity(if force.is_some() { n } else { 0 });
    for i in 0..n {
        if i > 0 {
            step = session
                .step_native(reference[i - 1])
                .map_err(|e| format!("offset {i}: {e}"))?;
        }
        let top = model
            .whole_table_nearest_two(&step.query_q12)
            .map_err(|e| format!("offset {i}: {e}"))?;
        if top[0].0 != step.token {
            return Err(format!(
                "offset {i}: the step picked {}, the whole table's nearest is {}",
                step.token, top[0].0
            ));
        }
        forced.picks.push(step.token);
        forced.runner_up.push(top[1].0);
        forced.score_q12.push(top[0].1);
        forced.runner_score_q12.push(top[1].1);
        if force.is_some() {
            queries.push(step.query_q12.clone());
        }
    }
    if force.is_some() {
        let mut steps = session
            .take_decision_trace()
            .ok_or("the decision trace is off")?;
        let first = prompt.len() - 1;
        if steps.len() != first + n {
            return Err(format!(
                "the decision trace holds {} steps, not {}",
                steps.len(),
                first + n
            ));
        }
        for (i, query_q12) in queries.into_iter().enumerate() {
            steps[first + i].nearest = Some(DecisionNearest {
                pick: forced.picks[i],
                score_q12: Some(forced.score_q12[i]),
                score_f64: None,
                runner_up: forced.runner_up[i],
                runner_score_q12: Some(forced.runner_score_q12[i]),
                runner_score_f64: None,
                query_q12,
                query_f32: Vec::new(),
            });
        }
        forced.steps = Some(steps);
    }
    Ok(forced)
}

/// `--force`'s file at `path` (claude-249), read by [`force_windows`].
fn read_force(
    path: &str,
    rows: &[PanelPrompt],
) -> Result<std::collections::BTreeMap<usize, DecisionForceWindow>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    force_windows(&bytes, rows).map_err(|e| format!("{path}: {e}"))
}

/// A `uor-r4.decision-force/1` file's forced sets by their rows' positions (claude-249).
/// Refused: a file of another schema, a position named twice, and one that names no row of
/// `rows`.
fn force_windows(
    bytes: &[u8],
    rows: &[PanelPrompt],
) -> Result<std::collections::BTreeMap<usize, DecisionForceWindow>, String> {
    let file: DecisionForce = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if file.schema != DECISION_FORCE_SCHEMA {
        return Err(format!(
            "the schema is {}, not {DECISION_FORCE_SCHEMA}",
            file.schema
        ));
    }
    let mut windows = std::collections::BTreeMap::new();
    for window in file.windows {
        let position = window.position;
        if !rows.iter().any(|r| r.position == position) {
            return Err(format!("position {position} names no prompt"));
        }
        if windows.insert(position, window).is_some() {
            return Err(format!("position {position} is named twice"));
        }
    }
    Ok(windows)
}

/// One `--forced` output row (claude-244).
fn forced_row(position: usize, forced: &Forced) -> serde_json::Value {
    json!({
        "position": position,
        "picks": forced.picks,
        "runner_up": forced.runner_up,
        "score_q12": forced.score_q12,
        "runner_score_q12": forced.runner_score_q12,
    })
}

/// The decision window of the row at `position` that `forced_picks` read under a decision trace
/// (claude-249): the prompt and the reference tokens fed, the reply from the prompt's last step.
fn forced_window(
    position: usize,
    prompt: &[u32],
    reference: &[u32],
    forced: Forced,
) -> Option<DecisionWindow> {
    let fed = &reference[..forced.picks.len() - 1];
    Some(DecisionWindow {
        position,
        ids: prompt.iter().chain(fed).copied().collect(),
        reply_start: prompt.len() - 1,
        steps: forced.steps?,
    })
}

/// R's decision trace of `windows` on `model` (claude-249), its reads' head and kept counts
/// from the artifact's bytes.
fn r_decision_trace(
    model: &IntegerStackModel,
    artifact: &[u8],
    windows: Vec<DecisionWindow>,
) -> Result<DecisionTrace, String> {
    let reads = read_hamming_uorlut01(artifact).map_err(|e| format!("the reads: {e}"))?;
    let (heads, kept) = reads.map_or((0, 0), |r| (r.record.heads, r.record.kept));
    Ok(DecisionTrace {
        schema: DECISION_TRACE_SCHEMA.to_owned(),
        side: "R".to_owned(),
        layers: model.shape().layers(),
        lanes: model.shape().lanes(),
        heads,
        kept,
        windows,
    })
}

/// `--forced` (integer-nearest-readout, claude-244): each panel prompt's teacher-forced top two on
/// a nearest-head artifact (`forced_picks`), written new to `out` as a JSON array of rows. Under
/// `trace` (claude-249) it also writes, new, each row's decisions as R's `uor-r4.decision-trace/1`;
/// under `force`, the `uor-r4.decision-force/1` file there, it runs only the rows that file
/// names, each under its own forced set. It prints one JSON line: rows, offsets read, capped rows,
/// the time per row in microseconds, and each file's sha256.
fn forced(
    model: &IntegerStackModel,
    artifact: &[u8],
    prompts: &str,
    out: &str,
    trace: Option<&str>,
    force: Option<&str>,
) -> Result<(), String> {
    forced_artifact(model)?;
    let bytes = std::fs::read(prompts).map_err(|e| format!("{prompts}: {e}"))?;
    let rows: Vec<PanelPrompt> =
        serde_json::from_slice(&bytes).map_err(|e| format!("{prompts}: {e}"))?;
    let force = force.map(|path| read_force(path, &rows)).transpose()?;
    let none = DecisionForceWindow::default();
    let (mut written, mut positions, mut capped) = (Vec::with_capacity(rows.len()), 0usize, 0usize);
    let mut windows = Vec::new();
    let started = Instant::now();
    for row in &rows {
        let window = match &force {
            Some(forced) => match forced.get(&row.position) {
                Some(window) => Some(window),
                None => continue,
            },
            None => trace.is_some().then_some(&none),
        };
        let forced = forced_picks(model, &row.prompt, &row.reference, window)
            .map_err(|e| format!("position {}: {e}", row.position))?;
        positions += forced.picks.len();
        capped += usize::from(forced.capped);
        written.push(forced_row(row.position, &forced));
        windows.extend(forced_window(
            row.position,
            &row.prompt,
            &row.reference,
            forced,
        ));
    }
    let per_row_us = started.elapsed().as_micros() / written.len().max(1) as u128;
    let text = serde_json::to_vec(&written).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    let mut line = json!({ "rows": written.len(), "positions": positions, "capped": capped, "per_row_us": per_row_us as u64, "out": out, "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&text)) });
    if let Some(path) = trace {
        let record = r_decision_trace(model, artifact, windows)?;
        let text = serde_json::to_vec(&record).map_err(|e| e.to_string())?;
        write_new_file(std::path::Path::new(path), &text).map_err(|e| format!("{path}: {e}"))?;
        line["decision_trace"] = json!(path);
        line["decision_trace_sha256"] =
            json!(hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&text)));
    }
    println!("{line}");
    Ok(())
}

/// What every replay of one `--decision-replay` run shares (claude-256): R from the artifact as
/// Q, R's trace header (no windows), the reference side, the deciles and the cell's G₁₀.
struct ReplayRun<'a> {
    model: &'a IntegerStackModel,
    header: DecisionTrace,
    reference: GradeSide<'a>,
    other_table: NearestTable<'a>,
    deciles: &'a DecisionDeciles,
    g10: i64,
}

impl ReplayRun<'_> {
    /// The forced replay (`replay_step`) of the step at `offset` of `row`'s reply, and that
    /// step. Each pass steps the prompt and the reference up to the offset under Φ
    /// (`forced_picks`): the forward is causal, so a pass cut at p reads p's prefix as the whole
    /// window does. Pass 0, with Φ empty, must pick `unforced`, R's unforced pick there
    /// (I-Rerun).
    fn replay(
        &self,
        row: &PanelPrompt,
        offset: usize,
        unforced: u32,
    ) -> Result<(usize, StepReplay), String> {
        let position = row.position;
        let window = self
            .reference
            .trace
            .windows
            .iter()
            .find(|w| w.position == position)
            .ok_or_else(|| format!("position {position}: the reference has no such window"))?;
        let step = offset_step(window, offset)?;
        let cut = row.reference.get(..=offset).ok_or_else(|| {
            format!(
                "position {position}, offset {offset}: past the reference's {} tokens",
                row.reference.len()
            )
        })?;
        let replay = replay_step(
            self.reference,
            self.other_table,
            self.deciles,
            position,
            step,
            self.g10,
            |phi| {
                let forced = forced_picks(self.model, &row.prompt, cut, Some(phi))?;
                let rerun = phi.buckets.is_empty() && phi.codes.is_empty();
                if rerun && forced.picks.last() != Some(&unforced) {
                    return Err(format!(
                        "unforced, R picks {:?} at offset {offset}, not its unforced {unforced} (I-Rerun)",
                        forced.picks.last()
                    ));
                }
                let window = forced_window(position, &row.prompt, cut, forced)
                    .ok_or("the decision trace is off")?;
                Ok(DecisionTrace {
                    windows: vec![window],
                    ..self.header.clone()
                })
            },
        )?;
        Ok((step, replay))
    }
}

/// The offsets of the reference's `window` inside R's window of `row` (its prompt and its
/// reference less the last token) where R's unforced picks `r_picks` differ (claude-256), and how
/// many of the window's picks (its steps with a nearest) lie past R's. A float side's window may
/// read on past the reference; the forward is causal, so its first offsets read R's tokens. A
/// window that does not start its reply at the prompt's last step, does not reach R's last
/// offset, or reads other tokens in R's window is refused (I-Align).
fn differing_offsets(
    window: &DecisionWindow,
    row: &PanelPrompt,
    r_picks: &[u32],
) -> Result<(Vec<usize>, usize), String> {
    let n = r_picks.len();
    let fed = &row.reference[..n.saturating_sub(1)];
    let ids: Vec<u32> = row.prompt.iter().chain(fed).copied().collect();
    let reply = window.steps.get(window.reply_start..).unwrap_or_default();
    if window.reply_start + 1 != row.prompt.len()
        || reply.len() < n
        || window.ids.get(..ids.len()) != Some(&ids[..])
    {
        return Err(format!(
            "position {}: the reference does not read R's {} tokens with its reply from the prompt's last step through R's {n} offsets (I-Align)",
            window.position,
            ids.len()
        ));
    }
    let mut differ = Vec::new();
    for (offset, (step, &r)) in reply.iter().zip(r_picks).enumerate() {
        let pick = step.nearest.as_ref().map(|n| n.pick).ok_or_else(|| {
            format!(
                "position {}, offset {offset}: the reference has no nearest",
                window.position
            )
        })?;
        if pick != r {
            differ.push(offset);
        }
    }
    let beyond = reply[n..].iter().filter(|s| s.nearest.is_some()).count();
    Ok((differ, beyond))
}

/// `--decision-replay` (claude-256): claude-247's forced replay of the arithmetic step (T, R),
/// R from the artifact at `path` as Q, at each chosen position, or, without `--at`, at every
/// offset inside R's window where R's unforced pick is not the reference's (`differing_offsets`,
/// which counts the reference's picks past R's window); written new
/// as `uor-r4.decision-replay/1`. `args` follow `--decision-replay` on the command line. It
/// prints one JSON line: the offsets compared and equal, the replays, their counts and the file's
/// sha256. Before any replay it refuses a reference that is not T's trace, holds two windows at
/// one position, or is not the trace the deciles were read from; a `--reference-table` that is
/// not the artifact itself; and a named offset past its window, its reference or R's offsets. A
/// refused replay is recorded with its refusal, the rest still run, and the command then fails.
fn decision_replay_out(path: &str, args: &[String]) -> Result<(), String> {
    let [reference, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let names = [
        "--prompts",
        "--deciles",
        "--g10",
        "--reference-table",
        "--out",
    ];
    let (flags, at) = decision_flags(rest, &names)?;
    let (prompts, deciles_path, g10, reference_table, out) = (
        needed(&flags, "--prompts")?,
        needed(&flags, "--deciles")?,
        needed(&flags, "--g10")?,
        needed(&flags, "--reference-table")?,
        needed(&flags, "--out")?,
    );
    let g10: i64 = g10.parse().map_err(|e| format!("--g10 {g10}: {e}"))?;
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
    forced_artifact(&model)?;
    let (rt, reference_sha) = read_decision_trace(reference)?;
    if rt.side != "T" {
        return Err(format!(
            "{reference}: side {}, but --decision-replay replays the arithmetic step (T, R)",
            rt.side
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    if let Some(w) = rt.windows.iter().find(|w| !seen.insert(w.position)) {
        return Err(format!(
            "{reference}: two windows at position {}",
            w.position
        ));
    }
    let deciles_text = std::fs::read(deciles_path).map_err(|e| format!("{deciles_path}: {e}"))?;
    let deciles = parse_deciles(deciles_path, &deciles_text)?;
    if deciles.trace.as_ref().map(|t| t.sha256.as_str()) != Some(reference_sha.as_str()) {
        return Err(format!(
            "{deciles_path}: the deciles were read from another trace than {reference}"
        ));
    }
    let prompts_text = std::fs::read(prompts).map_err(|e| format!("{prompts}: {e}"))?;
    let rows: Vec<PanelPrompt> =
        serde_json::from_slice(&prompts_text).map_err(|e| format!("{prompts}: {e}"))?;
    // T's table is R's writer table divided by 4096, so its file is R's artifact.
    let table_bytes =
        std::fs::read(reference_table).map_err(|e| format!("{reference_table}: {e}"))?;
    if table_bytes != bytes {
        return Err(format!(
            "{reference_table}: T's table is R's artifact divided by 4096, and this is not {path}"
        ));
    }
    let (rtab, rtab_sha) = load_table(&rt.side, reference_table)?;
    // R's own table: the writer table its nearest scans.
    let writer = model
        .fbs_writer_table()
        .ok_or("the artifact has no FBS writer table")?;
    let vocab = model.shape().vocab;
    let run = ReplayRun {
        model: &model,
        header: r_decision_trace(&model, &bytes, Vec::new())?,
        reference: GradeSide {
            trace: &rt,
            table: rtab.view(),
        },
        other_table: NearestTable::Q12 {
            table: writer,
            width: writer.len() / vocab,
        },
        deciles: &deciles,
        g10,
    };
    let row_at = |position: usize| {
        rows.iter()
            .find(|r| r.position == position)
            .ok_or_else(|| format!("position {position}: {prompts} has no such row"))
    };
    // The offsets named, or every one of the reference's inside R's window where R's unforced
    // pick differs, each with R's unforced pick there, and the reference's picks past R's
    // window.
    let (chosen, compared, beyond) = if at.is_empty() {
        let (mut chosen, mut compared, mut beyond) = (Vec::new(), 0usize, 0usize);
        for window in &rt.windows {
            let row = row_at(window.position)?;
            let unforced = forced_picks(&model, &row.prompt, &row.reference, None)
                .map_err(|e| format!("position {}: {e}", row.position))?;
            let (differ, past) = differing_offsets(window, row, &unforced.picks)?;
            compared += unforced.picks.len();
            beyond += past;
            chosen.extend(
                differ
                    .into_iter()
                    .map(|offset| (window.position, offset, unforced.picks[offset])),
            );
        }
        (chosen, Some(compared), Some(beyond))
    } else {
        // Each named offset is checked before any replay runs.
        let mut chosen = Vec::with_capacity(at.len());
        for (position, offset) in at {
            let row = row_at(position)?;
            let window = rt
                .windows
                .iter()
                .find(|w| w.position == position)
                .ok_or_else(|| format!("position {position}: the reference has no such window"))?;
            offset_step(window, offset)?;
            let cut = row.reference.get(..=offset).ok_or_else(|| {
                format!(
                    "position {position}, offset {offset}: past the reference's {} tokens",
                    row.reference.len()
                )
            })?;
            let unforced = forced_picks(&model, &row.prompt, cut, None)
                .map_err(|e| format!("position {position}: {e}"))?;
            let pick = unforced.picks.get(offset).copied().ok_or_else(|| {
                format!(
                    "position {position}, offset {offset}: past R's {} offsets",
                    unforced.picks.len()
                )
            })?;
            chosen.push((position, offset, pick));
        }
        (chosen, None, None)
    };
    let kinds = [
        "accounted",
        "unaccounted",
        "refused",
        "ended_kept",
        "ended_memory",
        "ended_nearest",
        "passes",
    ];
    let mut counts: std::collections::BTreeMap<&str, usize> =
        kinds.into_iter().map(|k| (k, 0)).collect();
    let mut replayed = Vec::with_capacity(chosen.len());
    let started = Instant::now();
    // Each replay is recorded as it ends; a refused one is recorded with its refusal, and the
    // run goes on.
    for (position, offset, unforced) in chosen {
        let (step, replay) = match run.replay(row_at(position)?, offset, unforced) {
            Ok(done) => done,
            Err(refused) => {
                *counts.entry("refused").or_default() += 1;
                replayed.push(json!({
                    "position": position,
                    "offset": offset,
                    "refused": refused,
                }));
                continue;
            }
        };
        let end = match replay.passes.last() {
            Some(PassOutcome::Kept { .. }) => "ended_kept",
            Some(PassOutcome::Memory { .. }) => "ended_memory",
            _ => "ended_nearest",
        };
        *counts.entry(end).or_default() += 1;
        *counts
            .entry(if replay.accounted {
                "accounted"
            } else {
                "unaccounted"
            })
            .or_default() += 1;
        *counts.entry("passes").or_default() += replay.passes.len();
        replayed.push(json!({
            "position": position,
            "offset": offset,
            "step": step,
            "replay": replay,
        }));
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let record = json!({
        "schema": DECISION_REPLAY_SCHEMA,
        "artifact": { "path": path, "sha256": sha256_hex(&bytes) },
        "reference": { "path": reference, "sha256": reference_sha, "side": rt.side },
        "prompts": { "path": prompts, "sha256": sha256_hex(&prompts_text) },
        "reference_table": { "path": reference_table, "sha256": rtab_sha },
        "other_table": { "path": path, "writer_rows": vocab, "width": writer.len() / vocab },
        "deciles": { "path": deciles_path, "sha256": sha256_hex(&deciles_text), "trace": deciles.trace },
        "g10": g10,
        "offsets_compared": compared,
        "picks_beyond_r": beyond,
        "equal": compared.map(|c| c - replayed.len()),
        "replays": replayed.len(),
        "counts": counts,
        "replay_ms": elapsed_ms,
        "replayed": replayed,
    });
    let text = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    write_new_file(std::path::Path::new(out), &text).map_err(|e| format!("{out}: {e}"))?;
    println!(
        "{}",
        json!({ "schema": DECISION_REPLAY_SCHEMA, "offsets_compared": record["offsets_compared"], "picks_beyond_r": record["picks_beyond_r"], "equal": record["equal"], "replays": record["replays"], "counts": record["counts"], "replay_ms": elapsed_ms, "out": out, "sha256": sha256_hex(&text) })
    );
    match counts["refused"] {
        0 => Ok(()),
        refused => Err(format!(
            "{refused} of {} replays were refused; each is recorded in {out}",
            replayed.len()
        )),
    }
}

const PREFILL: usize = 128;
const DECODE: usize = 64;
const WARMUP: usize = 1;
const RUNS: usize = 20;

/// One session step: the token fed in, the token the writer chose.
trait Stepper {
    fn step(&mut self, token: u32) -> Result<u32, String>;
}

impl Stepper for uor_r4_integer::stack::IntegerStackSession<'_> {
    fn step(&mut self, token: u32) -> Result<u32, String> {
        self.step_native(token)
            .map(|s| s.token)
            .map_err(|e| e.to_string())
    }
}

/// One timed run in a fresh session: the prefill over `prompt` and `decode`
/// greedy steps after it, as `(prefill_ns, decode_ns, reply)`. The reply is
/// the prefill's last choice followed by each decode step's.
fn timed_run<S: Stepper>(
    mut session: S,
    prompt: &[u32],
    decode: usize,
) -> Result<(u64, u64, Vec<u32>), String> {
    let start = Instant::now();
    let mut next = 0;
    for &t in prompt {
        next = session.step(t)?;
    }
    let prefill = start.elapsed();
    let mut reply = vec![next];
    let start = Instant::now();
    for _ in 0..decode {
        next = session.step(next)?;
        reply.push(next);
    }
    let decode = start.elapsed();
    let ns = |d: std::time::Duration| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX);
    Ok((ns(prefill), ns(decode), reply))
}

/// The median of integer durations: the integer mean of the middle two for an
/// even count.
fn median(sorted: &[u64]) -> u64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2
    }
}

/// The nearest-rank p95: the value at rank ceil(0.95 n), counting from 1.
fn p95(sorted: &[u64]) -> u64 {
    let rank = (sorted.len() * 95).div_ceil(100).max(1);
    sorted[rank - 1]
}

/// Integer tokens per second for `tokens` in `ns` nanoseconds.
fn tokens_per_second(tokens: usize, ns: u64) -> u64 {
    (tokens as u128 * 1_000_000_000 / u128::from(ns.max(1))) as u64
}

/// Run the protocol: `WARMUP` untimed runs, then `RUNS` timed ones, every
/// run's reply the same.
fn protocol<S: Stepper>(
    mut fresh: impl FnMut() -> S,
    prompt: &[u32],
) -> Result<serde_json::Value, String> {
    for _ in 0..WARMUP {
        timed_run(fresh(), prompt, DECODE)?;
    }
    let mut prefill = Vec::with_capacity(RUNS);
    let mut decode = Vec::with_capacity(RUNS);
    let mut reply: Option<Vec<u32>> = None;
    for _ in 0..RUNS {
        let (p, d, r) = timed_run(fresh(), prompt, DECODE)?;
        match &reply {
            Some(first) if *first != r => {
                return Err("timed runs wrote different replies".to_owned())
            }
            Some(_) => {}
            None => reply = Some(r),
        }
        prefill.push(p);
        decode.push(d);
    }
    let summary = |raw: &[u64], tokens: usize| {
        let mut sorted = raw.to_vec();
        sorted.sort_unstable();
        let (med, slow) = (median(&sorted), p95(&sorted));
        json!({
            "tokens": tokens,
            "raw_ns": raw,
            "median_ns": med,
            "p95_ns": slow,
            "median_tokens_per_second": tokens_per_second(tokens, med),
            "p95_tokens_per_second": tokens_per_second(tokens, slow),
        })
    };
    Ok(json!({
        "warmup_runs": WARMUP,
        "timed_runs": RUNS,
        "context_prefill": summary(&prefill, prompt.len()),
        "decode": summary(&decode, DECODE),
        "reply": reply.unwrap_or_default(),
    }))
}

/// The two blocks `measure_fbs_step_stages` times in each layer, in its order.
const BLOCKS: [&str; 2] = ["recurrence", "mlp"];
/// The nine parts `measure_fbs_step_parts` times in each layer, in its order;
/// each is named after the block it belongs to.
const PARTS: [&str; 9] = [
    "recurrence.norm",
    "recurrence.gather_in",
    "recurrence.core",
    "recurrence.gather_out",
    "mlp.norm",
    "mlp.gather_in",
    "mlp.product",
    "mlp.requant",
    "mlp.gather_out",
];

/// The protocol's runs again through `measure_fbs_step_stages`
/// (fbs-cost-profile, claude-158), or with `parts` through
/// `measure_fbs_step_parts` (fbs-serving-kernels, claude-163): every timed
/// run's decode steps, each as its stage times, and the reply, which every
/// run must repeat.
fn stage_protocol(
    model: &IntegerStackModel,
    prompt: &[u32],
    parts: bool,
) -> Result<(Vec<Vec<u64>>, Vec<u32>), String> {
    let mut rows = Vec::with_capacity(RUNS * DECODE);
    let mut reply: Option<Vec<u32>> = None;
    let mut ns = Vec::new();
    for run in 0..WARMUP + RUNS {
        let mut session = model.session();
        let mut step = |token: u32, ns: &mut Vec<u64>| -> Result<u32, String> {
            if parts {
                session.measure_fbs_step_parts(token, ns)
            } else {
                session.measure_fbs_step_stages(token, ns)
            }
            .map(|(s, _)| s.token)
            .map_err(|e| e.to_string())
        };
        let mut next = 0;
        for &t in prompt {
            next = step(t, &mut ns)?;
        }
        let mut written = vec![next];
        for _ in 0..DECODE {
            next = step(next, &mut ns)?;
            written.push(next);
            if run >= WARMUP {
                rows.push(ns.clone());
            }
        }
        match &reply {
            Some(first) if *first != written => {
                return Err("stage-timed runs wrote different replies".to_owned())
            }
            Some(_) => {}
            None => reply = Some(written),
        }
    }
    Ok((rows, reply.unwrap_or_default()))
}

/// The stages timed in order for `layers` layers, each layer's named by
/// `per_layer` ([`BLOCKS`] or [`PARTS`]).
fn stage_names(layers: usize, per_layer: &[&str]) -> Vec<String> {
    let mut names = vec!["embed".to_owned()];
    for l in 0..layers {
        names.extend(per_layer.iter().map(|p| format!("layer{l}.{p}")));
    }
    names.extend(["final_norm", "writer", "bookkeeping"].map(str::to_owned));
    names
}

/// The stage profile of the decode steps in `rows`, each layer's stages named
/// by `per_layer`: each stage's median, their sum against the median step (per
/// mille), the whole-table nearest's median, and the step against the plain
/// protocol's per-token decode time. Their difference, `loop_overhead_ns`, is
/// the cost outside the step: the loop, and `step_native`'s copies of the kept
/// cells and the query into its `NativeStep`.
fn stage_report(
    rows: &[Vec<u64>],
    per_layer: &[&str],
    plain_decode_ns: u64,
    plain_reply: &[u32],
    staged_reply: &[u32],
) -> Result<serde_json::Value, String> {
    let width = rows.first().map_or(0, Vec::len);
    if width < 6
        || !(width - 6).is_multiple_of(per_layer.len())
        || rows.iter().any(|r| r.len() != width)
    {
        return Err(format!(
            "stage rows of {width} times do not fit the stage order"
        ));
    }
    let column = |i: usize| {
        let mut values: Vec<u64> = rows.iter().map(|r| r[i]).collect();
        values.sort_unstable();
        median(&values)
    };
    let names = stage_names((width - 6) / per_layer.len(), per_layer);
    let stages: Vec<serde_json::Value> = names
        .iter()
        .enumerate()
        .map(|(i, name)| json!({ "name": name, "median_ns": column(i) }))
        .collect();
    let stage_sum: u64 = (0..names.len()).map(column).sum();
    let (step, nearest) = (column(width - 2), column(width - 1));
    let per_token = plain_decode_ns / DECODE as u64;
    Ok(json!({
        "decode_steps": rows.len(),
        "stages": stages,
        "stage_sum_ns": stage_sum,
        "step_median_ns": step,
        "stage_sum_per_mille_of_step": stage_sum * 1000 / step.max(1),
        "whole_table_nearest_median_ns": nearest,
        "plain_decode_per_token_ns": per_token,
        "step_per_mille_of_plain_token": step * 1000 / per_token.max(1),
        "loop_overhead_ns": per_token as i64 - step as i64,
        "replies_match": plain_reply == staged_reply,
    }))
}

/// Each layer's block against its parts (fbs-serving-kernels, claude-163): for
/// every block in `blocks` (a [`BLOCKS`] report), the medians of its parts in
/// `parts` (a [`PARTS`] report) summed, against the block's own median, per
/// mille, and whether every block lies within 10% (900 to 1,100 per mille).
fn blocks_against_parts(
    blocks: &serde_json::Value,
    parts: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let medians = |report: &serde_json::Value| -> Result<Vec<(String, u64)>, String> {
        report["stages"]
            .as_array()
            .ok_or("a stage report without stages")?
            .iter()
            .map(|s| match (s["name"].as_str(), s["median_ns"].as_u64()) {
                (Some(name), Some(ns)) => Ok((name.to_owned(), ns)),
                _ => Err("a stage without a name or a median".to_owned()),
            })
            .collect()
    };
    let (blocks, parts) = (medians(blocks)?, medians(parts)?);
    let mut rows = Vec::new();
    let mut within = true;
    for (block, block_ns) in blocks.iter().filter(|(name, _)| {
        name.starts_with("layer") && BLOCKS.iter().any(|b| name.ends_with(&format!(".{b}")))
    }) {
        let prefix = format!("{block}.");
        // The block's kind (`recurrence` or `mlp`) names how many parts it has.
        let kind = block.rsplit('.').next().unwrap_or_default();
        let expected = PARTS
            .iter()
            .filter(|p| p.starts_with(&format!("{kind}.")))
            .count();
        let found: Vec<u64> = parts
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, ns)| *ns)
            .collect();
        if found.len() != expected {
            return Err(format!(
                "{} of {block}'s {expected} parts were timed",
                found.len()
            ));
        }
        let summed: u64 = found.iter().sum();
        let per_mille = summed * 1000 / (*block_ns).max(1);
        within &= (900..=1100).contains(&per_mille);
        rows.push(json!({
            "block": block,
            "block_median_ns": block_ns,
            "parts_sum_ns": summed,
            "per_mille": per_mille,
        }));
    }
    if rows.is_empty() {
        return Err("the block report holds no layer blocks".to_owned());
    }
    Ok(json!({ "blocks": rows, "within_10_percent": within }))
}

/// One line of a command's output, or `None` when it does not run.
fn probe(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn hardware() -> serde_json::Value {
    let sysctl = |key: &str| probe("sysctl", &["-n", key]);
    json!({
        "os": probe("sw_vers", &["-productVersion"]),
        "cpu": sysctl("machdep.cpu.brand_string"),
        "performance_cores": sysctl("hw.perflevel0.physicalcpu"),
        "cores": sysctl("hw.ncpu"),
        "memory_bytes": sysctl("hw.memsize"),
        "power": probe("pmset", &["-g", "batt"]).and_then(|s| s.lines().next().map(str::to_owned)),
    })
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--replies") {
        let (path, prompts, out, reply_tokens) = match args.as_slice() {
            [path, r, prompts, o, out] if r == "--replies" && o == "--out" => {
                (path, prompts, out, DECODE)
            }
            [path, r, prompts, o, out, t, n]
                if r == "--replies" && o == "--out" && t == "--reply-tokens" =>
            {
                (
                    path,
                    prompts,
                    out,
                    n.parse()
                        .map_err(|_| format!("--reply-tokens {n}: {USAGE}"))?,
                )
            }
            _ => return Err(USAGE.to_owned()),
        };
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
        return replies(&model, prompts, out, reply_tokens);
    }
    if args.iter().any(|a| a == "--forced") {
        let (path, prompts, out, rest) = match args.as_slice() {
            [path, f, prompts, o, out, rest @ ..] if f == "--forced" && o == "--out" => {
                (path, prompts, out, rest)
            }
            _ => return Err(USAGE.to_owned()),
        };
        // claude-249's options, each at most once, in either order.
        let (mut trace, mut force) = (None, None);
        let mut rest = rest.iter();
        while let Some(flag) = rest.next() {
            let slot = match flag.as_str() {
                "--decision-trace" => &mut trace,
                "--force" => &mut force,
                _ => return Err(USAGE.to_owned()),
            };
            let value = rest.next().ok_or_else(|| USAGE.to_owned())?;
            if slot.replace(value.as_str()).is_some() {
                return Err(USAGE.to_owned());
            }
        }
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
        return forced(&model, &bytes, prompts, out, trace, force);
    }
    if args.first().is_some_and(|a| a == "--decision-grade") {
        return decision_grade_out(&args);
    }
    if let [path, flag, rest @ ..] = args.as_slice() {
        if flag == "--decision-replay" {
            return decision_replay_out(path, rest);
        }
    }
    match args.as_slice() {
        [flag, first, second] if flag == "--hamming-trace-compare" => {
            return hamming_trace_compare(first, second);
        }
        [flag, trace, out] if flag == "--decision-deciles" => {
            return decision_deciles_out(trace, out);
        }
        [flag, trace, t, table, o, out]
            if flag == "--decision-check" && t == "--table" && o == "--out" =>
        {
            return decision_check_out(trace, table, out);
        }
        [flag, ..] if flag == "--decision-deciles" || flag == "--decision-check" => {
            return Err(USAGE.to_owned());
        }
        [path, flag, readout, out] if flag == "--hamming-trace" => {
            let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
            return hamming_trace(&model, &bytes, readout, out);
        }
        [path, flag, float, integer, out] if flag == "--copy-codes" => {
            return copy_codes(path, float, integer, out);
        }
        _ => {}
    }
    let sweep = args.iter().any(|a| a == "--context-sweep");
    let args: Vec<String> = args
        .into_iter()
        .filter(|a| a != "--context-sweep")
        .collect();
    let parts = args.iter().any(|a| a == "--parts");
    let stages = parts || args.iter().any(|a| a == "--stages");
    let args: Vec<String> = args
        .into_iter()
        .filter(|a| a != "--stages" && a != "--parts")
        .collect();
    let (path, prompt_arg, tokens_arg) = match args.as_slice() {
        [path] => (path, None, None),
        [path, flag, ids] if flag == "--prompt" => (path, Some(ids), None),
        [path, flag, file] if flag == "--tokens" && sweep => (path, None, Some(file)),
        _ => return Err(USAGE.to_owned()),
    };
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| format!("parse: {e}"))?;
    let (vocab, context) = (model.shape().vocab, model.shape().context);
    if context < PREFILL + DECODE {
        return Err(format!(
            "the artifact's context {context} is shorter than {PREFILL} + {DECODE}"
        ));
    }
    let prompt: Vec<u32> = match prompt_arg {
        Some(ids) => ids
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<u32>()
                    .map_err(|e| format!("--prompt {s}: {e}"))
            })
            .collect::<Result<_, _>>()?,
        // A fixed deterministic prompt, as the readouts' sequences are built.
        None => (0..PREFILL)
            .map(|t| ((t * 11 + 1) % vocab) as u32)
            .collect(),
    };
    if prompt.len() != PREFILL {
        return Err(format!(
            "the prompt has {} tokens, not {PREFILL}",
            prompt.len()
        ));
    }
    if let Some(&t) = prompt.iter().find(|&&t| t as usize >= vocab) {
        return Err(format!(
            "prompt token {t} is outside the vocabulary {vocab}"
        ));
    }
    if sweep {
        if stages || prompt_arg.is_some() {
            return Err(format!("--context-sweep takes no other flag: {USAGE}"));
        }
        if read_hamming_uorlut01(&bytes)
            .map_err(|e| format!("the Hamming reads: {e}"))?
            .is_none()
        {
            return Err(
                "--context-sweep times the Hamming read, and the artifact has none".to_owned(),
            );
        }
        let (fit, skipped) = long_sweep_points(context);
        let (prompt, prefills, tokens) = match tokens_arg {
            None => (prompt, &SWEEP_PREFILLS[..], None),
            Some(file) => {
                let raw = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
                let prompt = long_sweep_prompt(&raw, vocab, context)?;
                for prefill in &skipped {
                    eprintln!(
                        "fbs-toks: skips the prefill {prefill}: the artifact's context {context} \
                         cannot hold it and its {DECODE} decode steps"
                    );
                }
                let tokens = json!({
                    "file": file,
                    "sha256": hex::encode(<sha2::Sha256 as sha2::Digest>::digest(&raw)),
                    "ids": (raw.len() - 64) / 2,
                });
                (prompt, &fit[..], Some(tokens))
            }
        };
        let without = IntegerStackModel::parse(&bytes)
            .map_err(|e| format!("parse: {e}"))?
            .without_hamming_read();
        let mut report = json!({
            "schema": "uor-r4.fbs-toks-context/1",
            "artifact_sha256": model.artifact_sha256(),
            "decode_tokens": DECODE,
            "points": context_sweep(&model, &without, &prompt, prefills)?,
            "hardware": hardware(),
        });
        if let Some(tokens) = tokens {
            report["context"] = json!(context);
            report["tokens"] = tokens;
            if !skipped.is_empty() {
                report["skipped_prefills"] = json!(skipped);
            }
        }
        println!("{report}");
        return Ok(());
    }
    let timing = protocol(|| model.session(), &prompt)?;
    let mut report = json!({
        "schema": "uor-r4.fbs-toks/1",
        "artifact_sha256": model.artifact_sha256(),
        "prompt": prompt,
        "timing": timing,
        "hardware": hardware(),
    });
    if stages {
        let (rows, staged_reply) = stage_protocol(&model, &prompt, false)?;
        let plain_decode = report["timing"]["decode"]["median_ns"]
            .as_u64()
            .ok_or("the plain protocol wrote no decode median")?;
        let plain_reply: Vec<u32> = serde_json::from_value(report["timing"]["reply"].clone())
            .map_err(|e| format!("the plain protocol's reply: {e}"))?;
        report["stages"] = stage_report(&rows, &BLOCKS, plain_decode, &plain_reply, &staged_reply)?;
        if parts {
            let (rows, parted_reply) = stage_protocol(&model, &prompt, true)?;
            let mut parted =
                stage_report(&rows, &PARTS, plain_decode, &plain_reply, &parted_reply)?;
            parted["against_blocks"] = blocks_against_parts(&report["stages"], &parted)?;
            report["parts"] = parted;
        }
    }
    println!("{report}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fbs-toks: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact_header(header: serde_json::Value) -> Vec<u8> {
        let header = serde_json::to_vec(&header).unwrap();
        let mut bytes = b"UORLUT01".to_vec();
        bytes.extend_from_slice(&(header.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&header);
        bytes
    }

    #[test]
    fn copy_codes_refuses_every_non_one_token_key_before_comparison() {
        let one = artifact_header(json!({"hamming": {"heads": 4}}));
        assert_eq!(artifact_copy_key(&one), Ok(1));
        assert_eq!(require_one_token_copy_codes(&one), Ok(()));

        for key in [0, 2, 3] {
            let bytes = artifact_header(json!({"hamming": {"heads": 4, "copy_key": key}}));
            assert_eq!(artifact_copy_key(&bytes), Ok(key));
            let error = require_one_token_copy_codes(&bytes).expect_err("non-one key");
            assert!(error.contains(&format!("copy_key {key}")), "{error}");
            assert!(error.contains("one-token copy key only"), "{error}");
        }

        // claude-269: a key per head leaves copy_key at 1 and is refused by its list.
        for keys in [[2, 1], [1, 2]] {
            let bytes = artifact_header(json!({"hamming": {"heads": 4, "copy_keys": keys}}));
            assert_eq!(artifact_copy_key(&bytes), Ok(1));
            let error = require_one_token_copy_codes(&bytes).expect_err("a key per head");
            let list = format!("copy_keys [{},{}]", keys[0], keys[1]);
            assert!(error.contains(&list), "{error}");
            assert!(error.contains("one-token copy key only"), "{error}");
        }

        let absent = artifact_header(json!({"shape": {}}));
        assert_eq!(artifact_copy_key(&absent), Ok(1));
        assert_eq!(
            artifact_copy_key(b"not a container"),
            Err("the artifact is not a UORLUT01 container".to_owned())
        );
        let malformed = artifact_header(json!({"hamming": {"copy_key": "two"}}));
        assert!(artifact_copy_key(&malformed)
            .expect_err("a non-integer key")
            .contains("not a non-negative integer"));
    }

    /// A session that answers each token with the next id, or, once `flip`
    /// is set, a different reply on every run.
    struct Fake {
        runs: std::rc::Rc<std::cell::Cell<u32>>,
        flip: bool,
    }

    impl Stepper for Fake {
        fn step(&mut self, token: u32) -> Result<u32, String> {
            Ok(if self.flip {
                token + self.runs.get()
            } else {
                token + 1
            })
        }
    }

    #[test]
    fn the_stage_report_takes_medians_and_reads_the_step_against_the_plain_token() {
        // One layer: embed, recurrence, mlp, final norm, writer, bookkeeping, step, nearest.
        let row = |scale: u64| -> Vec<u64> {
            [10, 40, 20, 5, 15, 2, 100, 300]
                .iter()
                .map(|v| v * scale)
                .collect()
        };
        let rows = [row(1), row(2), row(3)];
        let report = stage_report(&rows, &BLOCKS, 64 * 250, &[1, 2], &[1, 2]).expect("report");
        assert_eq!(report["decode_steps"], 3);
        let names: Vec<&str> = report["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "embed",
                "layer0.recurrence",
                "layer0.mlp",
                "final_norm",
                "writer",
                "bookkeeping"
            ]
        );
        // The middle row's stages: 20 + 80 + 40 + 10 + 30 + 4 = 184, against its step of 200.
        assert_eq!(report["stage_sum_ns"], 184);
        assert_eq!(report["step_median_ns"], 200);
        assert_eq!(report["stage_sum_per_mille_of_step"], 920);
        assert_eq!(report["whole_table_nearest_median_ns"], 600);
        // The plain protocol's decode: 250 ns a token, so the loop costs 50 beside the step.
        assert_eq!(report["plain_decode_per_token_ns"], 250);
        assert_eq!(report["step_per_mille_of_plain_token"], 800);
        assert_eq!(report["loop_overhead_ns"], 50);
        assert_eq!(report["replies_match"], true);
        let other = stage_report(&rows, &BLOCKS, 64 * 250, &[1, 2], &[1, 3]).expect("report");
        assert_eq!(other["replies_match"], false);
        // Rows that do not fit the stage order are refused.
        assert!(stage_report(&[vec![1, 2, 3, 4, 5, 6, 7]], &BLOCKS, 1, &[], &[]).is_err());
        assert!(stage_report(&[], &BLOCKS, 1, &[], &[]).is_err());
        // Nine parts a layer: a row of 6 + 9 fits, one of 6 + 2 does not.
        assert!(stage_report(&[vec![1; 15]], &PARTS, 1, &[], &[]).is_ok());
        assert!(stage_report(&[vec![1; 8]], &PARTS, 1, &[], &[]).is_err());
    }

    #[test]
    fn each_block_is_read_against_the_sum_of_its_parts() {
        // One layer. Blocks: recurrence 100, mlp 200.
        let blocks = stage_report(
            &[vec![5, 100, 200, 1, 1, 1, 400, 9]],
            &BLOCKS,
            64 * 500,
            &[],
            &[],
        )
        .expect("blocks");
        // Parts: the recurrence's four sum to 95 (950 per mille); the mlp's five to 230 (1,150).
        let row = vec![5, 10, 40, 30, 15, 20, 60, 100, 20, 30, 1, 1, 1, 420, 9];
        let parts = stage_report(&[row], &PARTS, 64 * 500, &[], &[]).expect("parts");
        let names: Vec<&str> = parts["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert_eq!(names[1], "layer0.recurrence.norm");
        assert_eq!(names[9], "layer0.mlp.gather_out");
        let read = blocks_against_parts(&blocks, &parts).expect("against");
        assert_eq!(read["blocks"][0]["block"], "layer0.recurrence");
        assert_eq!(read["blocks"][0]["parts_sum_ns"], 95);
        assert_eq!(read["blocks"][0]["per_mille"], 950);
        assert_eq!(read["blocks"][1]["block"], "layer0.mlp");
        assert_eq!(read["blocks"][1]["parts_sum_ns"], 230);
        assert_eq!(read["blocks"][1]["per_mille"], 1150);
        // The mlp's parts lie 15% above its block, so not every block is within 10%.
        assert_eq!(read["within_10_percent"], false);
        // Within 10% at both: the mlp's parts brought to 210 (1,050 per mille).
        let row = vec![5, 10, 40, 30, 15, 20, 60, 80, 20, 30, 1, 1, 1, 400, 9];
        let parts = stage_report(&[row], &PARTS, 64 * 500, &[], &[]).expect("parts");
        let read = blocks_against_parts(&blocks, &parts).expect("against");
        assert_eq!(read["blocks"][1]["per_mille"], 1050);
        assert_eq!(read["within_10_percent"], true);
        // A part report that names no part of a block is refused.
        assert!(blocks_against_parts(&blocks, &blocks).is_err());
        // So is one that holds only some of a block's parts: a second layer's
        // blocks against a one-layer part report.
        let two = stage_report(
            &[vec![5, 100, 200, 100, 200, 1, 1, 1, 700, 9]],
            &BLOCKS,
            64 * 500,
            &[],
            &[],
        )
        .expect("two layers");
        assert!(blocks_against_parts(&two, &parts).is_err());
        let mut short = parts.clone();
        short["stages"].as_array_mut().unwrap().remove(9);
        let err = blocks_against_parts(&blocks, &short).expect_err("a missing part");
        assert_eq!(err, "4 of layer0.mlp's 5 parts were timed");
    }

    #[test]
    fn the_statistics_follow_the_step_6_protocol() {
        assert_eq!(median(&[1, 2, 9]), 2);
        // The integer mean of the middle two for an even count, not the upper one.
        assert_eq!(median(&[1, 2, 4, 9]), 3);
        // Rank ceil(0.95 n): 19 of 20, 1 of 1.
        let twenty: Vec<u64> = (1..=20).collect();
        assert_eq!(p95(&twenty), 19);
        assert_eq!(p95(&[7]), 7);
        assert_eq!(tokens_per_second(64, 1_000_000), 64_000);
        assert_eq!(tokens_per_second(64, 0), 64_000_000_000);
    }

    #[test]
    fn the_long_sweep_reads_a_uort_file_and_refuses_what_it_cannot_sweep() {
        // The tokenless sweep keeps claude-235's prefills; the long one ends at 192 and 32,768.
        assert_eq!(SWEEP_PREFILLS, [32, 64, 96, 128]);
        let largest = LONG_SWEEP_PREFILLS[LONG_SWEEP_PREFILLS.len() - 1];
        let end = largest + DECODE;
        assert_eq!(LONG_SWEEP_PREFILLS[0] + DECODE, 192);
        assert_eq!(end, 32_768);
        // A UORT file: the magic, a version, the id count, then the ids low byte first.
        let uort = |ids: &[u16], count: u64| -> Vec<u8> {
            let mut b = vec![0u8; 64];
            b[..4].copy_from_slice(b"UORT");
            b[4] = 1;
            b[8..16].copy_from_slice(&count.to_le_bytes());
            b.extend(ids.iter().flat_map(|i| i.to_le_bytes()));
            b
        };
        let bytes_of = |ids: &[u16]| uort(ids, ids.len() as u64);
        // One id more than the largest prefill. The first reads 258 low byte first (513 if not).
        let mut ids: Vec<u16> = (0..=largest).map(|i| (i % 1000) as u16).collect();
        ids[0] = 0x0102;
        let bytes = bytes_of(&ids);
        let prompt = long_sweep_prompt(&bytes, 1000, end).expect("prompt");
        assert_eq!(prompt.len(), largest);
        assert_eq!(prompt[0], 258);
        assert_eq!(prompt[largest - 1], ((largest - 1) % 1000) as u32);
        // An artifact one position short of the last context sweeps the points it holds
        // (claude-288); one short of the first is refused.
        assert_eq!(
            long_sweep_prompt(&bytes, 1000, end - 1).expect("short context").len(),
            8_192
        );
        assert_eq!(
            long_sweep_prompt(&bytes, 1000, 191).expect_err("no point"),
            format!("the artifact's context 191 is shorter than 128 + {DECODE}")
        );
        // Raw ids with no header, a wrong magic, and a header shorter than 64 bytes.
        let raw: Vec<u8> = ids.iter().flat_map(|i| i.to_le_bytes()).collect();
        let not_uort = "the token file is not a UORT token file";
        assert_eq!(
            long_sweep_prompt(&raw, 1000, end).expect_err("raw"),
            not_uort
        );
        let mut magic = bytes.clone();
        magic[3] = b'X';
        assert_eq!(
            long_sweep_prompt(&magic, 1000, end).expect_err("magic"),
            not_uort
        );
        assert_eq!(
            long_sweep_prompt(&bytes[..63], 1000, end).expect_err("short"),
            not_uort
        );
        // A count that disagrees with the payload, and a payload of odd length.
        assert_eq!(
            long_sweep_prompt(&uort(&ids, ids.len() as u64 + 1), 1000, end).expect_err("count"),
            format!(
                "the token file's header counts {} ids, its payload holds {} bytes",
                ids.len() + 1,
                2 * ids.len()
            )
        );
        assert!(long_sweep_prompt(&bytes[..bytes.len() - 1], 1000, end)
            .expect_err("odd length")
            .contains("its payload holds"));
        // A file an id short of the largest prefill.
        assert_eq!(
            long_sweep_prompt(&bytes_of(&ids[..largest - 1]), 1000, end).expect_err("an id short"),
            format!(
                "the token file holds {} ids, fewer than {largest}",
                largest - 1
            )
        );
        // An id at the vocabulary's size in the prompt is refused; past the prompt it is not read.
        let mut inside = ids.clone();
        inside[largest - 1] = 1000;
        assert_eq!(
            long_sweep_prompt(&bytes_of(&inside), 1000, end).expect_err("outside the vocabulary"),
            format!(
                "token 1000 at {} is outside the vocabulary 1000",
                largest - 1
            )
        );
        let mut past = ids;
        past[largest] = 1000;
        assert!(long_sweep_prompt(&bytes_of(&past), 1000, end).is_ok());
    }

    #[test]
    fn the_long_sweep_times_the_points_a_carry_context_holds() {
        // claude-288: a carry export at 8,256 is swept to 8,256 and skips 32,704.
        assert_eq!(
            long_sweep_points(8_256),
            (vec![128, 512, 2_048, 8_192], vec![32_704])
        );
        assert_eq!(long_sweep_points(8_255), (vec![128, 512, 2_048], vec![8_192, 32_704]));
        assert_eq!(long_sweep_points(32_768), (LONG_SWEEP_PREFILLS.to_vec(), vec![]));
        assert_eq!(long_sweep_points(100), (vec![], LONG_SWEEP_PREFILLS.to_vec()));
    }

    /// `--forced` on the synthetic nearest-head artifact (claude-244), whose context is 8.
    #[cfg(feature = "fbs-synthetic")]
    mod forced {
        use super::super::*;
        use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, synthetic::synthetic_fbs_trunk};

        fn artifact(nearest: bool) -> IntegerStackModel {
            let mut snap = synthetic_fbs_trunk(7);
            snap.readout_nearest = nearest;
            let bytes = export_fbs_trunk_uorlut01(&snap).expect("export");
            IntegerStackModel::parse(&bytes).expect("parse")
        }

        /// Prompts of one to five tokens, spread over the vocabulary.
        fn prompts(vocab: usize) -> Vec<Vec<u32>> {
            (1..=5)
                .map(|len| {
                    (0..len)
                        .map(|i| ((i * 37 + len * 11 + 5) % vocab) as u32)
                        .collect()
                })
                .collect()
        }

        #[test]
        fn offset_zero_is_the_greedy_replys_first_token() {
            let model = artifact(true);
            forced_artifact(&model).expect("a nearest-head artifact");
            let mut checked = 0;
            for prompt in prompts(model.shape().vocab) {
                let reply = panel_reply(&model, &prompt, 64, None).expect("reply").reply;
                let forced = forced_picks(&model, &prompt, &[0], None).expect("forced");
                if let Some(&first) = reply.first() {
                    assert_eq!(forced.picks[0], first, "prompt {prompt:?}");
                    checked += 1;
                }
            }
            assert!(checked >= 2, "{checked}");
        }

        #[test]
        fn forced_on_the_greedy_reply_picks_the_greedy_reply() {
            let model = artifact(true);
            let (mut checked, mut long) = (0, 0);
            for prompt in prompts(model.shape().vocab) {
                let reply = panel_reply(&model, &prompt, 64, None).expect("reply").reply;
                if reply.is_empty() {
                    continue;
                }
                let forced = forced_picks(&model, &prompt, &reply, None).expect("forced");
                assert!(forced.picks.len() >= reply.len(), "prompt {prompt:?}");
                assert_eq!(
                    &forced.picks[..reply.len()],
                    &reply[..],
                    "prompt {prompt:?}"
                );
                checked += 1;
                long += usize::from(reply.len() >= 2);
            }
            assert!(checked >= 2 && long >= 1, "{checked} {long}");
        }

        #[test]
        fn each_offset_reads_the_step_after_the_reference_token() {
            let model = artifact(true);
            let vocab = model.shape().vocab;
            let mut checked = 0;
            for prompt in prompts(vocab).into_iter().filter(|p| p.len() <= 4) {
                let p0 = forced_picks(&model, &prompt, &[0], None)
                    .expect("forced")
                    .picks[0];
                let reference: Vec<u32> = (1..=3)
                    .map(|k| ((p0 as usize + k) % vocab) as u32)
                    .collect();
                let reply = panel_reply(&model, &prompt, 64, None).expect("reply").reply;
                assert_ne!(
                    reference.first(),
                    reply.first(),
                    "the reference is not the greedy reply"
                );
                let forced = forced_picks(&model, &prompt, &reference, None).expect("forced");
                assert_eq!(forced.picks.len(), 3, "prompt {prompt:?}");
                let mut session = model.session();
                for (row, &token) in prompt.iter().enumerate() {
                    if row + 1 == prompt.len() {
                        session.mark_reply_start();
                    }
                    session.step_native(token).expect("prompt step");
                }
                for offset in 1..=2 {
                    let step = session.step_native(reference[offset - 1]).expect("step");
                    let top = model
                        .whole_table_nearest_two(&step.query_q12)
                        .expect("top two");
                    assert_eq!(
                        forced.picks[offset], step.token,
                        "prompt {prompt:?} offset {offset}"
                    );
                    assert_eq!(
                        forced.runner_up[offset], top[1].0,
                        "prompt {prompt:?} offset {offset}"
                    );
                }
                checked += 1;
            }
            assert!(checked >= 3, "{checked}");
        }

        #[test]
        fn a_row_holds_its_offsets_and_the_context_caps_them() {
            let model = artifact(true);
            let vocab = model.shape().vocab;
            assert_eq!(model.shape().context, 8);
            let prompt: Vec<u32> = (0..3).map(|i| ((i * 5 + 2) % vocab) as u32).collect();
            let reference: Vec<u32> = (0..20).map(|i| ((i * 3 + 1) % vocab) as u32).collect();
            let forced = forced_picks(&model, &prompt, &reference, None).expect("forced");
            assert!(forced.capped);
            let row = forced_row(42, &forced);
            let keys: Vec<&str> = row
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            let mut want = vec![
                "position",
                "picks",
                "runner_up",
                "score_q12",
                "runner_score_q12",
            ];
            want.sort_unstable();
            let mut keys = keys;
            keys.sort_unstable();
            assert_eq!(keys, want);
            assert_eq!(row["position"], 42);
            for key in ["picks", "runner_up", "score_q12", "runner_score_q12"] {
                assert_eq!(row[key].as_array().map(Vec::len), Some(6), "{key}");
            }
            for (s, r) in forced.score_q12.iter().zip(&forced.runner_score_q12) {
                assert!(s >= r, "{s} {r}");
            }
            // A reference the context holds whole is not capped.
            let whole = forced_picks(&model, &prompt, &reference[..6], None).expect("forced");
            assert!(!whole.capped && whole.picks.len() == 6);
            // Refusals: an artifact whose head is not the nearest one, an empty reference, an
            // empty prompt, a prompt past the context, and a fed token outside the vocabulary.
            assert!(forced_artifact(&artifact(false))
                .expect_err("not the nearest head")
                .contains("is not it"));
            assert!(forced_picks(&model, &prompt, &[], None).is_err());
            assert!(forced_picks(&model, &[], &reference, None).is_err());
            assert!(forced_picks(&model, &[1; 9], &reference, None).is_err());
            assert!(forced_picks(&model, &prompt, &[vocab as u32, 1], None).is_err());
            // The last reference token is never fed, so it may lie outside the vocabulary.
            assert!(forced_picks(&model, &prompt, &[1, vocab as u32], None).is_ok());
        }

        /// claude-249: the synthetic trunk exported as R's artifact is: neither the gates nor
        /// the transition snapped, with the nearest head.
        fn raw_artifact() -> IntegerStackModel {
            IntegerStackModel::parse(&raw_bytes(false)).expect("parse")
        }

        /// [`raw_artifact`]'s bytes, `reads` adding four-head Hamming reads, as R's artifact
        /// has them (claude-256).
        fn raw_bytes(reads: bool) -> Vec<u8> {
            use uor_r4_integer::stack::synthetic::synthetic_hamming;
            use uor_r4_integer::stack::{export_fbs_trunk_uorlut01_with, FbsExportSnaps};
            let mut snap = synthetic_fbs_trunk(7);
            snap.readout_nearest = true;
            if reads {
                snap.hamming = Some(synthetic_hamming(snap.width, snap.layers.len(), 4, 188));
            }
            let raw = FbsExportSnaps {
                gates: false,
                transition: false,
            };
            export_fbs_trunk_uorlut01_with(&snap, raw).expect("export")
        }

        /// claude-256: R's trace as a float side T writes one, on R's table divided by 4096:
        /// each rate score q16 / 2^16, each code's coordinates ±1/2 by its bits, each nearest's
        /// query Q12 / 4096 with its two scores recomputed on `table`. Its decisions are R's.
        fn as_t(r: &DecisionTrace, table: NearestTable<'_>) -> DecisionTrace {
            let coordinates = |code: u64| -> Vec<u32> {
                (0..64)
                    .map(|i| if code >> i & 1 == 1 { 0.5f32 } else { -0.5 }.to_bits())
                    .collect()
            };
            let mut t = r.clone();
            t.side = "T".to_owned();
            for step in t.windows.iter_mut().flat_map(|w| &mut w.steps) {
                for layer in &mut step.layers {
                    let scores = layer.score_q16.drain(..);
                    layer.score_f32 = scores.map(|s| (s as f32 / 65536.0).to_bits()).collect();
                }
                let reads = step.layers.iter_mut().filter_map(|l| l.read.as_mut());
                for read in reads.chain(step.readout.as_mut()) {
                    for head in &mut read.heads {
                        head.query_f32 = coordinates(head.query);
                        head.key_f32 = coordinates(head.key);
                    }
                }
                if let Some(n) = step.nearest.as_mut() {
                    let query = n.query_q12.drain(..);
                    n.query_f32 = query.map(|q| (q as f32 / 4096.0).to_bits()).collect();
                    (n.score_q12, n.runner_score_q12) = (None, None);
                    let pick = table.score(n, n.pick).expect("a row");
                    let runner = table.score(n, n.runner_up).expect("a row");
                    (n.score_f64, n.runner_score_f64) =
                        (Some(pick.to_bits()), Some(runner.to_bits()));
                }
            }
            t
        }

        /// claude-256: a pass cut at p reads the picks and decisions up to p as the whole window
        /// does; `--decision-replay`'s replay of R against R's own trace as T is one pass at each
        /// offset; a pass 0 that does not pick R's unforced pick is refused (I-Rerun); and a T
        /// rate bucket moved into the next entry ends pass 0 there and joins Φ, which R takes in
        /// pass 1 (else the grade refuses it, I-Force).
        #[test]
        fn a_replay_on_r_cuts_its_window_at_p_and_forces_r() {
            use uor_r4_integer::fbs_snapshot::ForcedBucket;
            let bytes = raw_bytes(true);
            let model = IntegerStackModel::parse(&bytes).expect("parse");
            let writer = model.fbs_writer_table().expect("the FBS trunk");
            let t_table: Vec<f32> = writer.iter().map(|&v| f32::from(v) / 4096.0).collect();
            let width = writer.len() / model.shape().vocab;
            let t_view = NearestTable::F32 {
                table: &t_table,
                width,
            };
            let r_view = NearestTable::Q12 {
                table: writer,
                width,
            };
            let none = DecisionForceWindow::default();
            let prompt = prompts(model.shape().vocab)[2].clone();
            let reference = [3u32, 5, 7];
            let whole = forced_picks(&model, &prompt, &reference, Some(&none)).expect("whole");
            let whole_steps = whole.steps.clone().expect("a trace");
            let picks = whole.picks.clone();
            for offset in 0..reference.len() {
                let cut = forced_picks(&model, &prompt, &reference[..=offset], Some(&none))
                    .expect("cut at p");
                let p = prompt.len() - 1 + offset;
                assert_eq!(cut.picks[..], whole.picks[..=offset], "offset {offset}");
                assert_eq!(
                    cut.steps.as_deref(),
                    Some(&whole_steps[..=p]),
                    "offset {offset}"
                );
            }
            let row = PanelPrompt {
                position: 4,
                prompt: prompt.clone(),
                reference: reference.to_vec(),
                float_keyed: None,
            };
            let window = forced_window(row.position, &prompt, &reference, whole).expect("window");
            let r = r_decision_trace(&model, &bytes, vec![window]).expect("R's trace");
            let t = as_t(&r, t_view);
            check_decision_trace(&t, t_view).expect("T's trace checks");
            let replay_from = |t: &DecisionTrace, offset: usize, unforced: u32| {
                let deciles = decision_deciles(t).expect("deciles");
                let run = ReplayRun {
                    model: &model,
                    header: r_decision_trace(&model, &bytes, Vec::new()).expect("header"),
                    reference: GradeSide {
                        trace: t,
                        table: t_view,
                    },
                    other_table: r_view,
                    deciles: &deciles,
                    g10: 0,
                };
                run.replay(&row, offset, unforced)
            };
            let replay = |t: &DecisionTrace, offset: usize| replay_from(t, offset, picks[offset]);
            for offset in 0..reference.len() {
                let (step, same) = replay(&t, offset).expect("a replay");
                assert_eq!(step, prompt.len() - 1 + offset);
                assert_eq!((same.passes.len(), same.accounted), (1, true), "{same:?}");
            }
            let vocab = model.shape().vocab as u32;
            let rerun = replay_from(&t, 1, (picks[1] + 1) % vocab).expect_err("not R's pick");
            assert!(rerun.contains("(I-Rerun)"), "{rerun}");
            let mut moved = t.clone();
            let layer = &mut moved.windows[0].steps[0].layers[0];
            let entry = layer.entry[0];
            let next = if entry < 15 { entry + 1 } else { entry - 1 };
            layer.entry[0] = next;
            layer.score_f32[0] = ((f32::from(next) + 0.5) / 16.0).to_bits();
            let (_, forced) = replay(&moved, 2).expect("a replay R takes Φ in");
            let want = ForcedBucket {
                step: 0,
                layer: 0,
                lane: 0,
                entry: next,
            };
            assert!(
                matches!(&forced.passes[0], PassOutcome::Bucket { force, other_entry, .. } if *force == want && *other_entry == entry),
                "{:?}",
                forced.passes[0]
            );
            assert_eq!(forced.forced.buckets.first(), Some(&want));
            assert!(forced.passes.len() >= 2, "{forced:?}");
        }

        /// claude-256: the unforced comparison reads the reference's offsets inside R's window,
        /// and counts those past it, as a float window that reads on past the reference has
        /// them; a pick that differs there is chosen. A window that reads other tokens in R's
        /// window, stops short of R's last offset or starts its reply elsewhere is refused.
        #[test]
        fn the_unforced_comparison_reads_r_window_and_counts_the_rest() {
            let model = raw_artifact();
            let none = DecisionForceWindow::default();
            let prompt = prompts(model.shape().vocab)[2].clone();
            let reference = [3u32, 5, 7];
            let whole = forced_picks(&model, &prompt, &reference, Some(&none)).expect("whole");
            let picks = whole.picks.clone();
            let window = forced_window(4, &prompt, &reference, whole).expect("window");
            let row = PanelPrompt {
                position: 4,
                prompt: prompt.clone(),
                reference: reference.to_vec(),
                float_keyed: None,
            };
            assert_eq!(differing_offsets(&window, &row, &picks), Ok((vec![], 0)));
            let mut longer = window.clone();
            let last = longer.steps.last().cloned().expect("a step");
            let mut unread = last.clone();
            unread.nearest = None;
            longer.ids.extend([7, 9, 11]);
            longer.steps.extend([last.clone(), last, unread]);
            assert_eq!(differing_offsets(&longer, &row, &picks), Ok((vec![], 2)));
            let mut differs = longer.clone();
            let nearest = differs.steps[window.reply_start + 1]
                .nearest
                .as_mut()
                .expect("a nearest");
            nearest.pick = (nearest.pick + 1) % model.shape().vocab as u32;
            assert_eq!(differing_offsets(&differs, &row, &picks), Ok((vec![1], 2)));
            let refused = |w: &DecisionWindow| {
                let e = differing_offsets(w, &row, &picks).expect_err("refused");
                assert!(e.contains("(I-Align)"), "{e}");
            };
            let mut other = longer.clone();
            other.ids[prompt.len()] = (reference[0] + 1) % model.shape().vocab as u32;
            refused(&other);
            let mut short = window.clone();
            short.steps.pop();
            refused(&short);
            let mut moved = longer;
            moved.reply_start -= 1;
            refused(&moved);
        }

        /// claude-256, round 2: `--decision-replay` on files. T's and R's tables load from R's
        /// artifact; T's check is not circular (a T score off its table's is refused); R's reads
        /// have four heads. A clean run compares R's window and replays nothing; a refused
        /// replay is recorded with its refusal, and the command fails after writing its file.
        /// Refused before any replay, writing nothing, each by its own check alone: a trace named
        /// side R as the reference, two windows at one position, deciles read from another
        /// trace, another artifact as the table, and a named offset past the window but inside
        /// the reference.
        #[test]
        fn the_replay_command_records_each_replay_and_refuses_what_it_cannot_replay() {
            /// The test's own folder, removed when the test ends, a failing one too.
            struct Scratch(std::path::PathBuf);
            impl Drop for Scratch {
                fn drop(&mut self) {
                    let _ = std::fs::remove_dir_all(&self.0);
                }
            }
            let scratch = Scratch(
                std::env::temp_dir().join(format!("uor_fbs_toks_replay_{}", std::process::id())),
            );
            let dir = &scratch.0;
            let _ = std::fs::remove_dir_all(dir);
            std::fs::create_dir_all(dir).expect("a scratch folder");
            let path = |name: &str| dir.join(name).to_string_lossy().into_owned();
            let put = |name: &str, bytes: &[u8]| std::fs::write(path(name), bytes).expect("write");
            let bytes = raw_bytes(true);
            put("r.uorlut", &bytes);
            // Another artifact T's loader takes (snapped, with the same Hamming reads): only
            // the table's identity check can refuse it.
            let other = {
                use uor_r4_integer::stack::synthetic::synthetic_hamming;
                let mut snap = synthetic_fbs_trunk(7);
                snap.readout_nearest = true;
                snap.hamming = Some(synthetic_hamming(snap.width, snap.layers.len(), 4, 188));
                export_fbs_trunk_uorlut01(&snap).expect("export")
            };
            assert_ne!(other, bytes);
            put("other.uorlut", &other);
            load_table("T", &path("other.uorlut")).expect("T's loader takes it");
            let model = IntegerStackModel::parse(&bytes).expect("parse");
            let writer = model.fbs_writer_table().expect("the FBS trunk");
            let width = writer.len() / model.shape().vocab;
            let t_table: Vec<f32> = writer.iter().map(|&v| f32::from(v) / 4096.0).collect();
            let (r_owned, _) = load_table("R", &path("r.uorlut")).expect("R's table");
            let (t_owned, _) = load_table("T", &path("r.uorlut")).expect("T's table");
            assert!(
                matches!(&r_owned, OwnedTable::Q12(t, w) if t[..] == writer[..] && *w == width)
            );
            assert!(matches!(&t_owned, OwnedTable::F32(t, w) if *t == t_table && *w == width));
            let t_view = t_owned.view();
            let none = DecisionForceWindow::default();
            let prompt = prompts(model.shape().vocab)[2].clone();
            let reference = [3u32, 5, 7];
            let whole = forced_picks(&model, &prompt, &reference, Some(&none)).expect("whole");
            let window = forced_window(4, &prompt, &reference, whole).expect("window");
            let r = r_decision_trace(&model, &bytes, vec![window]).expect("R's trace");
            let t = as_t(&r, t_view);
            check_decision_trace(&t, t_view).expect("T's trace checks");
            let reads: Vec<_> = t.windows[0]
                .steps
                .iter()
                .flat_map(|s| s.layers.iter().filter_map(|l| l.read.as_ref()))
                .collect();
            assert!(!reads.is_empty() && reads.iter().all(|read| read.heads.len() == 4));
            let mut tampered = t.clone();
            let nearest = tampered.windows[0]
                .steps
                .iter_mut()
                .find_map(|s| s.nearest.as_mut())
                .expect("a nearest");
            let score = f64::from_bits(nearest.score_f64.expect("a score"));
            nearest.score_f64 = Some((score + 1.0).to_bits());
            assert!(check_decision_trace(&tampered, t_view).is_err());
            let json_of = |trace: &DecisionTrace| serde_json::to_vec(trace).expect("json");
            let deciles_of = |trace: &DecisionTrace, name: &str| {
                let mut deciles = decision_deciles(trace).expect("deciles");
                deciles.trace = Some(DecisionSource {
                    path: path(name),
                    sha256: sha256_hex(&json_of(trace)),
                });
                serde_json::to_vec(&deciles).expect("json")
            };
            let mut twice = t.clone();
            twice.windows.push(t.windows[0].clone());
            let mut moved = t.clone();
            let start = moved.windows[0].reply_start;
            let nearest = moved.windows[0].steps[start + 2]
                .nearest
                .as_mut()
                .expect("a nearest");
            nearest.pick = (nearest.pick + 1) % model.shape().vocab as u32;
            for (name, trace) in [("t", &t), ("twice", &twice), ("moved", &moved)] {
                put(&format!("{name}.json"), &json_of(trace));
                put(
                    &format!("d-{name}.json"),
                    &deciles_of(trace, &format!("{name}.json")),
                );
            }
            // T's trace named side R, with deciles that name it: only the side check can refuse
            // it (R's own trace has no deciles: G₁₀ is read on the reference side).
            let mut as_r = t.clone();
            as_r.side = "R".to_owned();
            put("as-r.json", &json_of(&as_r));
            let mut deciles = decision_deciles(&t).expect("deciles");
            deciles.trace = Some(DecisionSource {
                path: path("as-r.json"),
                sha256: sha256_hex(&json_of(&as_r)),
            });
            put("d-as-r.json", &serde_json::to_vec(&deciles).expect("json"));
            for (name, reference) in [("prompts", &reference[..]), ("long", &[3, 5, 7, 9])] {
                let rows = json!([{ "position": 4, "prompt": prompt, "reference": reference }]);
                put(
                    &format!("{name}.json"),
                    &serde_json::to_vec(&rows).expect("json"),
                );
            }
            let run_on =
                |prompts: &str, trace: &str, deciles: &str, table: &str, out: &str, at: &[&str]| {
                    let mut args = vec![path(trace)];
                    for (flag, value) in [
                        ("--prompts", path(prompts)),
                        ("--deciles", path(deciles)),
                        ("--g10", "0".to_owned()),
                        ("--reference-table", path(table)),
                        ("--out", path(out)),
                    ] {
                        args.extend([flag.to_owned(), value]);
                    }
                    for a in at {
                        args.extend(["--at".to_owned(), (*a).to_owned()]);
                    }
                    decision_replay_out(&path("r.uorlut"), &args)
                };
            let run = |trace: &str, deciles: &str, table: &str, out: &str, at: &[&str]| {
                run_on("prompts.json", trace, deciles, table, out, at)
            };
            let read = |name: &str| -> serde_json::Value {
                serde_json::from_slice(&std::fs::read(path(name)).expect("read")).expect("json")
            };
            run("t.json", "d-t.json", "r.uorlut", "same.json", &[]).expect("a clean run");
            let same = read("same.json");
            assert_eq!(
                [&same["offsets_compared"], &same["equal"], &same["replays"]],
                [&json!(3), &json!(3), &json!(0)]
            );
            let refused = run(
                "moved.json",
                "d-moved.json",
                "r.uorlut",
                "moved-out.json",
                &[],
            )
            .expect_err("one replay refused");
            assert_eq!(
                refused,
                format!(
                    "1 of 1 replays were refused; each is recorded in {}",
                    path("moved-out.json")
                )
            );
            let moved_out = read("moved-out.json");
            assert_eq!(moved_out["counts"]["refused"], 1);
            assert_eq!(moved_out["replayed"][0]["offset"], 2);
            // The moved pick is refused by pass 0's grade: T's table does not score it so.
            assert!(
                moved_out["replayed"][0]["refused"]
                    .as_str()
                    .is_some_and(|e| e.starts_with("pass 0: ")),
                "{moved_out}"
            );
            let refusal = |trace: &str, deciles: &str, table: &str, at: &[&str]| {
                run(trace, deciles, table, "never.json", at).expect_err("refused")
            };
            for (got, want) in [
                (
                    refusal("as-r.json", "d-as-r.json", "r.uorlut", &[]),
                    "side R, but --decision-replay replays the arithmetic step (T, R)",
                ),
                (
                    refusal("twice.json", "d-twice.json", "r.uorlut", &[]),
                    "two windows at position 4",
                ),
                (
                    refusal("t.json", "d-moved.json", "r.uorlut", &[]),
                    "the deciles were read from another trace than",
                ),
                (
                    refusal("t.json", "d-t.json", "other.uorlut", &[]),
                    "T's table is R's artifact divided by 4096, and this is not",
                ),
            ] {
                assert!(got.contains(want), "{got}");
            }
            // A reference that reads on past T's window: only the window's own check refuses
            // offset 3 before any replay.
            let past = run_on(
                "long.json",
                "t.json",
                "d-t.json",
                "r.uorlut",
                "never.json",
                &["4:3"],
            )
            .expect_err("refused");
            assert_eq!(
                past,
                format!(
                    "position 4, offset 3: past the window's {} steps",
                    t.windows[0].steps.len()
                )
            );
            assert!(!std::path::Path::new(&path("never.json")).exists());
        }

        /// claude-249: under the decision trace each row is the untraced row; the trace holds a
        /// step for each id fed and the nearest from the prompt's last step on, each the whole
        /// table's top two of its own query. Forced to its own buckets, the row and its trace are
        /// the same. A snapped artifact is refused.
        #[test]
        fn the_decision_trace_fills_the_nearest_and_changes_no_row() {
            use uor_r4_integer::fbs_snapshot::ForcedBucket;
            let model = raw_artifact();
            let none = DecisionForceWindow::default();
            let reference = [3u32, 5, 7];
            let mut checked = 0;
            for prompt in prompts(model.shape().vocab)
                .into_iter()
                .filter(|p| p.len() <= 4)
            {
                let plain = forced_picks(&model, &prompt, &reference, None).expect("forced");
                let traced =
                    forced_picks(&model, &prompt, &reference, Some(&none)).expect("traced");
                assert_eq!(
                    forced_row(1, &traced),
                    forced_row(1, &plain),
                    "prompt {prompt:?}"
                );
                assert_eq!(plain.steps, None);
                let steps = traced.steps.expect("the trace");
                let first = prompt.len() - 1;
                assert_eq!(
                    steps.len(),
                    first + plain.picks.len(),
                    "a step for each id fed"
                );
                for (t, step) in steps.iter().enumerate() {
                    let Some(nearest) = &step.nearest else {
                        assert!(
                            t < first,
                            "prompt {prompt:?} step {t}: the nearest from the prompt's last step"
                        );
                        continue;
                    };
                    let i = t - first;
                    let top = model
                        .whole_table_nearest_two(&nearest.query_q12)
                        .expect("top two");
                    assert_eq!(
                        (
                            nearest.pick,
                            nearest.score_q12,
                            nearest.runner_up,
                            nearest.runner_score_q12
                        ),
                        (top[0].0, Some(top[0].1), top[1].0, Some(top[1].1)),
                        "prompt {prompt:?} step {t}"
                    );
                    assert_eq!(
                        (nearest.pick, nearest.runner_up),
                        (plain.picks[i], plain.runner_up[i])
                    );
                }
                let mut own = DecisionForceWindow::default();
                for (step, s) in steps.iter().enumerate() {
                    for (layer, l) in s.layers.iter().enumerate() {
                        own.buckets
                            .extend(l.entry.iter().enumerate().map(|(lane, &entry)| {
                                ForcedBucket {
                                    step,
                                    layer,
                                    lane,
                                    entry,
                                }
                            }));
                    }
                }
                let forced = forced_picks(&model, &prompt, &reference, Some(&own))
                    .expect("forced to its own");
                assert_eq!(
                    forced_row(1, &forced),
                    forced_row(1, &plain),
                    "prompt {prompt:?}"
                );
                assert_eq!(forced.steps.as_ref(), Some(&steps), "prompt {prompt:?}");
                checked += 1;
            }
            assert!(checked >= 3, "{checked}");
            let refused =
                forced_picks(&artifact(true), &[1], &[2], Some(&none)).expect_err("snapped");
            assert!(refused.contains("the decision trace"), "{refused}");
        }

        /// claude-249: `--force`'s file names rows by position; another schema, a position named
        /// twice and one naming no prompt are refused.
        #[test]
        fn a_force_file_names_rows_by_position() {
            let rows: Vec<PanelPrompt> = [4usize, 9]
                .iter()
                .map(|&position| PanelPrompt {
                    position,
                    prompt: vec![1],
                    reference: vec![2],
                    float_keyed: None,
                })
                .collect();
            let file = |schema: &str, positions: &[usize]| {
                let windows: Vec<_> = positions
                    .iter()
                    .map(|p| json!({ "position": p, "buckets": [], "codes": [] }))
                    .collect();
                serde_json::to_vec(&json!({ "schema": schema, "windows": windows })).expect("json")
            };
            let windows =
                force_windows(&file(DECISION_FORCE_SCHEMA, &[9]), &rows).expect("one row");
            assert_eq!(windows.keys().copied().collect::<Vec<_>>(), [9]);
            assert!(force_windows(&file("uor-r4.decision-force/0", &[9]), &rows).is_err());
            let twice =
                force_windows(&file(DECISION_FORCE_SCHEMA, &[9, 9]), &rows).expect_err("twice");
            assert!(twice.contains("named twice"), "{twice}");
            let unknown =
                force_windows(&file(DECISION_FORCE_SCHEMA, &[5]), &rows).expect_err("no row");
            assert!(unknown.contains("names no prompt"), "{unknown}");
        }
    }

    #[test]
    fn a_run_prefills_then_decodes_greedily_and_replies_must_agree() {
        let runs = std::rc::Rc::new(std::cell::Cell::new(0));
        let prompt: Vec<u32> = (0..PREFILL as u32).collect();
        let fresh = || {
            runs.set(runs.get() + 1);
            Fake {
                runs: runs.clone(),
                flip: false,
            }
        };
        let report = protocol(fresh, &prompt).expect("protocol");
        // One warmup and 20 timed runs, each in a fresh session.
        assert_eq!(runs.get(), (WARMUP + RUNS) as u32);
        // The prefill's last choice, then 64 decode steps each feeding the last.
        let reply: Vec<u32> = (PREFILL as u32..=(PREFILL + DECODE) as u32).collect();
        assert_eq!(report["reply"], json!(reply));
        assert_eq!(report["decode"]["tokens"], DECODE);
        assert_eq!(report["context_prefill"]["tokens"], PREFILL);
        assert_eq!(
            report["decode"]["raw_ns"].as_array().map(Vec::len),
            Some(RUNS)
        );
        let flipping = || {
            runs.set(runs.get() + 1);
            Fake {
                runs: runs.clone(),
                flip: true,
            }
        };
        assert_eq!(
            protocol(flipping, &prompt).expect_err("different replies"),
            "timed runs wrote different replies"
        );
    }

    /// claude-256: the grader's and the replay's refusals that need no file: an `--at` named
    /// twice (before any file is read), a flag named twice or unknown, a side with no table, an
    /// offset past the window, and a deciles file that names no trace.
    #[test]
    fn the_decision_commands_refuse_what_they_cannot_grade() {
        let args = |s: &[&str]| s.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        let names = ["--deciles", "--out"];
        let flags = |s: &[&str]| decision_flags(&args(s), &names).map(|(_, at)| at);
        assert_eq!(
            flags(&["--at", "3:1", "--deciles", "d", "--at", "3:1"]),
            Err("--at 3:1 is named twice".to_owned())
        );
        assert_eq!(
            flags(&["--at", "3:1", "--at", "3:2", "--at", "3:1"]),
            Err("--at 3:1 is named twice".to_owned())
        );
        assert!(flags(&["--at", "3"])
            .unwrap_err()
            .contains("not POSITION:OFFSET"));
        for refused in [
            &["--out", "a", "--out", "b"][..],
            &["--force", "f"],
            &["--out"],
        ] {
            assert_eq!(flags(refused), Err(USAGE.to_owned()), "{refused:?}");
        }
        let given = args(&["--at", "3:1", "--out", "o", "--at", "3:2"]);
        let (named, at) = decision_flags(&given, &names).expect("flags");
        assert_eq!(at, [(3, 1), (3, 2)]);
        assert_eq!(needed(&named, "--out"), Ok("o"));
        assert_eq!(needed(&named, "--deciles"), Err(USAGE.to_owned()));
        let twice = ["--at", "1:0", "--at", "1:0"];
        let grade = args(
            &[
                &["--decision-grade", "/no/such/p", "/no/such/q"][..],
                &twice,
            ]
            .concat(),
        );
        assert_eq!(
            decision_grade_out(&grade),
            Err("--at 1:0 is named twice".to_owned())
        );
        let replay = args(&[&["/no/such/p"][..], &twice].concat());
        assert_eq!(
            decision_replay_out("/no/such/artifact", &replay),
            Err("--at 1:0 is named twice".to_owned())
        );
        assert_eq!(
            load_table("X", "/no/such/table").map(|_| ()),
            Err("/no/such/table: side X has no table; F, R and T do".to_owned())
        );
        let step = DecisionStep {
            layers: Vec::new(),
            readout: None,
            memory: None,
            nearest: None,
        };
        let window = DecisionWindow {
            position: 9,
            ids: vec![1, 2, 3],
            reply_start: 1,
            steps: vec![step; 3],
        };
        assert_eq!(offset_step(&window, 1), Ok(2));
        assert_eq!(
            offset_step(&window, 2),
            Err("position 9, offset 2: past the window's 3 steps".to_owned())
        );
        let deciles = |trace: serde_json::Value| {
            let mut file =
                json!({ "schema": DECISION_DECILES_SCHEMA, "side": "T", "families": [] });
            if !trace.is_null() {
                file["trace"] = trace;
            }
            serde_json::to_vec(&file).expect("json")
        };
        assert_eq!(
            parse_deciles("d.json", &deciles(serde_json::Value::Null)).map(|_| ()),
            Err("d.json: the deciles name no trace they were read from".to_owned())
        );
        let named = deciles(json!({ "path": "t.json", "sha256": "ab" }));
        assert!(parse_deciles("d.json", &named).is_ok());
    }
}
