//! Time a SpiralCore/FBS stack artifact's integer session on this machine
//! (d11-exporter-float-trunk Stage 4b, claude-126).
//!
//! ```text
//! fbs-toks ARTIFACT [--prompt ID,ID,...] [--stages]
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

use std::process::{Command, ExitCode};
use std::time::Instant;

use serde_json::json;
use uor_r4_integer::stack::IntegerStackModel;

const USAGE: &str = "usage: fbs-toks ARTIFACT [--prompt ID,ID,...] [--stages]";
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

/// The protocol's runs again through `measure_fbs_step_stages`
/// (fbs-cost-profile, claude-158): every timed run's decode steps, each as its
/// stage times, and the reply, which every run must repeat.
fn stage_protocol(
    model: &IntegerStackModel,
    prompt: &[u32],
) -> Result<(Vec<Vec<u64>>, Vec<u32>), String> {
    let mut rows = Vec::with_capacity(RUNS * DECODE);
    let mut reply: Option<Vec<u32>> = None;
    let mut ns = Vec::new();
    for run in 0..WARMUP + RUNS {
        let mut session = model.session();
        let mut step = |token: u32, ns: &mut Vec<u64>| -> Result<u32, String> {
            session
                .measure_fbs_step_stages(token, ns)
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

/// The stages `measure_fbs_step_stages` times, in its order, for `layers` layers.
fn stage_names(layers: usize) -> Vec<String> {
    let mut names = vec!["embed".to_owned()];
    for l in 0..layers {
        names.push(format!("layer{l}.recurrence"));
        names.push(format!("layer{l}.mlp"));
    }
    names.extend(["final_norm", "writer", "bookkeeping"].map(str::to_owned));
    names
}

/// The stage profile of the decode steps in `rows`: each stage's median, their
/// sum against the median step (per mille), the whole-table nearest's median,
/// and the step against the plain protocol's per-token decode time. Their
/// difference, `loop_overhead_ns`, is the cost outside the step: the loop, and
/// `step_native`'s copies of the kept cells and the query into its `NativeStep`.
fn stage_report(
    rows: &[Vec<u64>],
    plain_decode_ns: u64,
    plain_reply: &[u32],
    staged_reply: &[u32],
) -> Result<serde_json::Value, String> {
    let width = rows.first().map_or(0, Vec::len);
    if width < 6 || (width - 6) % 2 != 0 || rows.iter().any(|r| r.len() != width) {
        return Err(format!(
            "stage rows of {width} times do not fit the stage order"
        ));
    }
    let column = |i: usize| {
        let mut values: Vec<u64> = rows.iter().map(|r| r[i]).collect();
        values.sort_unstable();
        median(&values)
    };
    let names = stage_names((width - 6) / 2);
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
    let stages = args.iter().any(|a| a == "--stages");
    let args: Vec<String> = args.into_iter().filter(|a| a != "--stages").collect();
    let (path, prompt_arg) = match args.as_slice() {
        [path] => (path, None),
        [path, flag, ids] if flag == "--prompt" => (path, Some(ids)),
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
    let timing = protocol(|| model.session(), &prompt)?;
    let mut report = json!({
        "schema": "uor-r4.fbs-toks/1",
        "artifact_sha256": model.artifact_sha256(),
        "prompt": prompt,
        "timing": timing,
        "hardware": hardware(),
    });
    if stages {
        let (rows, staged_reply) = stage_protocol(&model, &prompt)?;
        let plain_decode = report["timing"]["decode"]["median_ns"]
            .as_u64()
            .ok_or("the plain protocol wrote no decode median")?;
        let plain_reply: Vec<u32> = serde_json::from_value(report["timing"]["reply"].clone())
            .map_err(|e| format!("the plain protocol's reply: {e}"))?;
        report["stages"] = stage_report(&rows, plain_decode, &plain_reply, &staged_reply)?;
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
        let report = stage_report(&rows, 64 * 250, &[1, 2], &[1, 2]).expect("report");
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
        let other = stage_report(&rows, 64 * 250, &[1, 2], &[1, 3]).expect("report");
        assert_eq!(other["replies_match"], false);
        // Rows that do not fit the stage order are refused.
        assert!(stage_report(&[vec![1, 2, 3, 4, 5, 6, 7]], 1, &[], &[]).is_err());
        assert!(stage_report(&[], 1, &[], &[]).is_err());
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
}
