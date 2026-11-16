//! claude-395: the native trainer of the Q31 read's gates (prep sheet
//! `one-model-end-to-end-native-read-train-2026-11-15`, "Built in the run unit" item 2).
//! It reads a train and an eval `Q31ReadDump` with their targets, starts from identity
//! gates or a saved read, runs `train_pick_step` over 64-window batches in index order
//! (cycling epochs), writes one JSON line per eval step and saves the final read with
//! `Q31FirstPickRead::to_bytes` and its eval picks as little-endian u32.
//! The device step is the default; `--host` runs the Rust oracle instead.
//! Run with:
//! `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 cargo run --release -p
//! uor-r4-discrete --features cuda --example native_read_train -- --train-dump F
//! --train-targets F --train-buckets F --eval-dump F --eval-targets F --eval-other F
//! --eval-buckets F --start identity|dump|FILE --margin 0 --step-shift 4 --steps 512 --batch 64
//! --eval-at 0,64,128,256,384,512 --out-jsonl F --save-read F --save-picks F
//! [--check-host-steps 2] [--train-other F] [--host]`

use std::io::Write;
use uor_r4_discrete::native_vector::{
    Q31FirstPickRead, Q31Quaternion, Q31ReadDump, Q31ReadTrainStep,
};
use uor_r4_discrete::{Error, Result};

/// The identity gate: the unit quaternion (1, 0, 0, 0) in Q31.
const IDENTITY_GATE: Q31Quaternion = [1 << 31, 0, 0, 0];

fn invalid(message: String) -> Error {
    Error::Invalid(message)
}

/// Every option of one run.
#[derive(Clone, Debug, Default)]
struct Args {
    train_dump: String,
    train_targets: String,
    train_buckets: String,
    train_other: Option<String>,
    eval_dump: String,
    eval_targets: String,
    eval_other: String,
    eval_buckets: String,
    start: String,
    margin: u64,
    step_shift: u32,
    steps: usize,
    batch: usize,
    eval_at: Vec<usize>,
    out_jsonl: String,
    save_read: String,
    save_picks: String,
    check_host_steps: usize,
    host: bool,
}

fn number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T> {
    value
        .parse()
        .map_err(|_| invalid(format!("{flag} needs a whole number, got {value}")))
}

fn parse_args(argv: &[String]) -> Result<Args> {
    let mut args = Args {
        steps: 512,
        batch: 64,
        eval_at: vec![0, 64, 128, 256, 384, 512],
        ..Args::default()
    };
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].as_str();
        if flag == "--host" {
            args.host = true;
            i += 1;
            continue;
        }
        let value = argv
            .get(i + 1)
            .ok_or_else(|| invalid(format!("{flag} needs a value")))?
            .clone();
        match flag {
            "--train-dump" => args.train_dump = value,
            "--train-targets" => args.train_targets = value,
            "--train-buckets" => args.train_buckets = value,
            "--train-other" => args.train_other = Some(value),
            "--eval-dump" => args.eval_dump = value,
            "--eval-targets" => args.eval_targets = value,
            "--eval-other" => args.eval_other = value,
            "--eval-buckets" => args.eval_buckets = value,
            "--start" => args.start = value,
            "--margin" => args.margin = number(flag, &value)?,
            "--step-shift" => args.step_shift = number(flag, &value)?,
            "--steps" => args.steps = number(flag, &value)?,
            "--batch" => args.batch = number(flag, &value)?,
            "--eval-at" => {
                args.eval_at = value
                    .split(',')
                    .map(|v| number(flag, v))
                    .collect::<Result<_>>()?
            }
            "--out-jsonl" => args.out_jsonl = value,
            "--save-read" => args.save_read = value,
            "--save-picks" => args.save_picks = value,
            "--check-host-steps" => args.check_host_steps = number(flag, &value)?,
            _ => return Err(invalid(format!("unknown option {flag}"))),
        }
        i += 2;
    }
    for (flag, value) in [
        ("--train-dump", &args.train_dump),
        ("--train-targets", &args.train_targets),
        ("--train-buckets", &args.train_buckets),
        ("--eval-dump", &args.eval_dump),
        ("--eval-targets", &args.eval_targets),
        ("--eval-other", &args.eval_other),
        ("--eval-buckets", &args.eval_buckets),
        ("--start", &args.start),
        ("--out-jsonl", &args.out_jsonl),
        ("--save-read", &args.save_read),
        ("--save-picks", &args.save_picks),
    ] {
        if value.is_empty() {
            return Err(invalid(format!("{flag} is required")));
        }
    }
    Ok(args)
}

fn read_file(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path).map_err(|e| invalid(format!("reading {path}: {e}")))
}

/// Writes a new file; an existing one is refused, never written over (a replay must not
/// replace the read or picks it is compared with).
fn write_file(path: &str, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    std::fs::File::create_new(path)
        .and_then(|mut f| f.write_all(bytes))
        .map_err(|e| invalid(format!("writing {path}: {e}")))
}

/// Little-endian u32 words, one per row of a dump of `rows` rows.
fn read_words(path: &str, rows: usize) -> Result<Vec<u32>> {
    let bytes = read_file(path)?;
    if bytes.len() != rows * 4 {
        return Err(invalid(format!(
            "{path} holds {} bytes; its dump needs {rows} u32 words",
            bytes.len()
        )));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().expect("four bytes")))
        .collect())
}

/// One u8 bucket per row of a dump of `rows` rows.
fn read_buckets(path: &str, rows: usize) -> Result<Vec<u8>> {
    let bytes = read_file(path)?;
    if bytes.len() != rows {
        return Err(invalid(format!(
            "{path} holds {} bytes; its dump needs {rows} bucket bytes",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// The targets rule of `train_pick_step`: a target is `s + 1` for an earlier row `s`.
fn check_targets(path: &str, words: &[u32], time: usize) -> Result<()> {
    for (row, &word) in words.iter().enumerate() {
        if word as usize > row % time {
            return Err(invalid(format!(
                "{path}: row {row} names {word}, not an earlier row of its window"
            )));
        }
    }
    Ok(())
}

fn load_dump(path: &str) -> Result<Q31ReadDump> {
    Q31ReadDump::from_bytes(&read_file(path)?).map_err(Error::from)
}

/// One split: its dump, targets, optional other targets and buckets.
struct Split {
    dump: Q31ReadDump,
    targets: Vec<u32>,
    other: Option<Vec<u32>>,
    buckets: Vec<u8>,
}

fn load_split(dump: &str, targets: &str, other: Option<&str>, buckets: &str) -> Result<Split> {
    let dump = load_dump(dump)?;
    let rows = dump.batch * dump.time;
    let targets_words = read_words(targets, rows)?;
    check_targets(targets, &targets_words, dump.time)?;
    let other = match other {
        Some(path) => {
            let words = read_words(path, rows)?;
            check_targets(path, &words, dump.time)?;
            Some(words)
        }
        None => None,
    };
    let buckets = read_buckets(buckets, rows)?;
    Ok(Split {
        dump,
        targets: targets_words,
        other,
        buckets,
    })
}

/// Where the step and the picks run.
enum Backend {
    Host,
    #[cfg(feature = "cuda")]
    Device(uor_r4_discrete::native_vector_device::NativeVectorEngine),
}

impl Backend {
    fn picks(&self, read: &Q31FirstPickRead, dump: &Q31ReadDump) -> Result<Vec<u32>> {
        let (s, i, t) = (&dump.stream, &dump.ids, &dump.table);
        match self {
            Backend::Host => Ok(read.apply(s, i, t, dump.batch, dump.time)?.picks),
            #[cfg(feature = "cuda")]
            Backend::Device(engine) => Ok(engine
                .first_pick_read(read, s, i, t, dump.batch, dump.time)?
                .picks),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step(
        &self,
        read: &Q31FirstPickRead,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
        targets: &[u32],
        margin: u64,
        step_shift: u32,
    ) -> Result<Q31ReadTrainStep> {
        match self {
            Backend::Host => Ok(
                read.train_pick_step(stream, ids, table, batch, time, targets, margin, step_shift)?
            ),
            #[cfg(feature = "cuda")]
            Backend::Device(engine) => engine.read_train_pick_step(
                read, stream, ids, table, batch, time, targets, margin, step_shift,
            ),
        }
    }
}

fn backend(host: bool) -> Result<Backend> {
    if host {
        return Ok(Backend::Host);
    }
    #[cfg(feature = "cuda")]
    {
        Ok(Backend::Device(
            uor_r4_discrete::native_vector_device::NativeVectorEngine::new(0)?,
        ))
    }
    #[cfg(not(feature = "cuda"))]
    Err(invalid(
        "the device step needs --features cuda; pass --host for the Rust oracle".into(),
    ))
}

/// Identity gates over the dump read's own weights and gain shift, the dump's own converted
/// read (`dump`: claude-389's converted float gates, for C1 and C3), or a saved read.
fn start_read(start: &str, dump: &Q31ReadDump) -> Result<Q31FirstPickRead> {
    let read = if start == "dump" {
        dump.read.clone()
    } else if start == "identity" {
        let gates = dump.read.width() / 4;
        Q31FirstPickRead::new(
            vec![IDENTITY_GATE; gates],
            vec![IDENTITY_GATE; gates],
            dump.read.weights().to_vec(),
            dump.read.gain_shift(),
        )?
    } else {
        Q31FirstPickRead::from_bytes(&read_file(start)?)?
    };
    if read.width() != dump.read.width() {
        return Err(invalid(format!(
            "the start read has width {}, the dump {}",
            read.width(),
            dump.read.width()
        )));
    }
    Ok(read)
}

/// Per far bucket (1 and 2): rows whose pick is the target, the other target, and the count.
fn bucket_counts(split: &Split, picks: &[u32]) -> String {
    let mut parts = Vec::new();
    for bucket in [1u8, 2] {
        let (mut target, mut other, mut n) = (0usize, 0usize, 0usize);
        for (row, &b) in split.buckets.iter().enumerate() {
            if b != bucket {
                continue;
            }
            n += 1;
            if split.targets[row] != 0 && picks[row] == split.targets[row] {
                target += 1;
            }
            if let Some(words) = &split.other {
                if words[row] != 0 && picks[row] == words[row] {
                    other += 1;
                }
            }
        }
        let other = if split.other.is_some() {
            other.to_string()
        } else {
            "null".to_string()
        };
        parts.push(format!(
            "\"{bucket}\":{{\"chose_target\":{target},\"chose_other\":{other},\"n\":{n}}}"
        ));
    }
    format!("{{{}}}", parts.join(","))
}

/// Train, write the readings, save the read and the eval picks.
fn run(args: &Args) -> Result<()> {
    let train = load_split(
        &args.train_dump,
        &args.train_targets,
        args.train_other.as_deref(),
        &args.train_buckets,
    )?;
    let eval = load_split(
        &args.eval_dump,
        &args.eval_targets,
        Some(&args.eval_other),
        &args.eval_buckets,
    )?;
    let (windows, time) = (train.dump.batch, train.dump.time);
    let width = train.dump.read.width();
    if args.batch == 0 || windows % args.batch != 0 {
        return Err(invalid(format!(
            "{windows} train windows are not a whole number of {}-window batches",
            args.batch
        )));
    }
    if eval.dump.read.width() != width || eval.dump.time != time {
        return Err(invalid("the eval dump's width or time differs".into()));
    }
    if eval.dump.read.to_bytes() != train.dump.read.to_bytes() {
        return Err(invalid(
            "the eval dump's read differs from the train dump's: not the same state".into(),
        ));
    }
    for path in [&args.save_read, &args.save_picks] {
        if std::path::Path::new(path).exists() {
            return Err(invalid(format!(
                "{path} exists; the trainer never writes over one"
            )));
        }
    }
    let backend = backend(args.host)?;
    let mut read = start_read(&args.start, &train.dump)?;
    let mut out = std::fs::File::create_new(&args.out_jsonl)
        .map_err(|e| invalid(format!("creating {}: {e}", args.out_jsonl)))?;
    let batches = windows / args.batch;
    let (mut accepted, mut last_loss) = (0usize, None::<u128>);
    for step in 0..=args.steps {
        if args.eval_at.contains(&step) {
            let eval_picks = backend.picks(&read, &eval.dump)?;
            let train_picks = backend.picks(&read, &train.dump)?;
            let loss = last_loss.map_or("null".to_string(), |l| format!("\"{l}\""));
            let line = format!(
                "{{\"step\":{step},\"step_shift\":{},\"margin\":{},\"train_loss\":{loss},\
                 \"accepted_steps\":{accepted},\"eval\":{},\"train\":{}}}",
                args.step_shift,
                args.margin,
                bucket_counts(&eval, &eval_picks),
                bucket_counts(&train, &train_picks),
            );
            writeln!(out, "{line}").map_err(|e| invalid(format!("writing jsonl: {e}")))?;
        }
        if step == args.steps {
            break;
        }
        let first = (step % batches) * args.batch;
        let (rows, end) = (first * time, (first + args.batch) * time);
        let stream = &train.dump.stream[rows * width..end * width];
        let (ids, targets) = (&train.dump.ids[rows..end], &train.targets[rows..end]);
        let table = &train.dump.table;
        let result = backend.step(
            &read,
            stream,
            ids,
            table,
            args.batch,
            time,
            targets,
            args.margin,
            args.step_shift,
        )?;
        if step < args.check_host_steps {
            let host = read.train_pick_step(
                stream,
                ids,
                table,
                args.batch,
                time,
                targets,
                args.margin,
                args.step_shift,
            )?;
            if host != result {
                return Err(invalid(format!("host and device differ at step {step}")));
            }
        }
        if result.accepted {
            accepted += 1;
        }
        last_loss = Some(if result.accepted {
            result.loss_after
        } else {
            result.loss_before
        });
        read = result.read;
    }
    write_file(&args.save_read, &read.to_bytes())?;
    let picks = backend.picks(&read, &eval.dump)?;
    let bytes: Vec<u8> = picks.iter().flat_map(|p| p.to_le_bytes()).collect();
    write_file(&args.save_picks, &bytes)
}

fn main() -> Result<()> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    run(&parse_args(&argv)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The served residual's scale at the read (claude-390): about 2^18.
    const S: i64 = 1 << 18;

    fn row(lane0: i64, lane1: i64) -> Vec<i64> {
        let mut row = vec![0; 16];
        row[0] = lane0;
        row[1] = lane1;
        row
    }

    /// claude-391's fixture, repeated over `windows` windows: row 2's nearest earlier
    /// key is row 0's (pick 1); its target is row 1 (pick 2).
    fn dump(windows: usize) -> Q31ReadDump {
        let window = [row(S, 0), row(0, S), row(3 * S / 4, S / 2)].concat();
        let gates = vec![[1 << 31, 0, 0, 0]; 4];
        Q31ReadDump {
            read: Q31FirstPickRead::new(gates.clone(), gates, vec![0; 16], 4).unwrap(),
            batch: windows,
            time: 3,
            stream: window.repeat(windows),
            ids: [0, 1, 2].repeat(windows),
            table: vec![0; 3 * 16],
        }
    }

    fn words(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    /// A fresh scratch folder under the system temp dir, unique to this test.
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("native-read-train-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Writes the fixture files and returns the args of one host run.
    fn host_args(dir: &std::path::Path, shift: u32, tag: &str) -> Args {
        let p = |f: &str| dir.join(f).to_string_lossy().into_owned();
        let (train, eval) = (dump(4), dump(2));
        std::fs::write(p("train.dump"), train.to_bytes().unwrap()).unwrap();
        std::fs::write(p("eval.dump"), eval.to_bytes().unwrap()).unwrap();
        std::fs::write(p("train.targets"), words(&[0, 0, 2].repeat(4))).unwrap();
        std::fs::write(p("train.buckets"), [0u8, 0, 1].repeat(4)).unwrap();
        std::fs::write(p("eval.targets"), words(&[0, 0, 2].repeat(2))).unwrap();
        std::fs::write(p("eval.other"), words(&[0, 0, 1].repeat(2))).unwrap();
        std::fs::write(p("eval.buckets"), [0u8, 0, 1, 0, 0, 2]).unwrap();
        let argv: Vec<String> = [
            "--train-dump",
            &p("train.dump"),
            "--train-targets",
            &p("train.targets"),
            "--train-buckets",
            &p("train.buckets"),
            "--eval-dump",
            &p("eval.dump"),
            "--eval-targets",
            &p("eval.targets"),
            "--eval-other",
            &p("eval.other"),
            "--eval-buckets",
            &p("eval.buckets"),
            "--start",
            "identity",
            "--margin",
            "0",
            "--step-shift",
            &shift.to_string(),
            "--steps",
            "8",
            "--batch",
            "2",
            "--eval-at",
            "0,4,8",
            "--out-jsonl",
            &p(&format!("{tag}.jsonl")),
            "--save-read",
            &p(&format!("{tag}.read")),
            "--save-picks",
            &p(&format!("{tag}.picks")),
            "--check-host-steps",
            "2",
            "--host",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        parse_args(&argv).unwrap()
    }

    #[test]
    fn host_runs_write_parseable_readings_and_learn_the_target() {
        let dir = scratch("readings");
        let args = host_args(&dir, 4, "a");
        run(&args).unwrap();
        let text = std::fs::read_to_string(&args.out_jsonl).unwrap();
        let lines: Vec<serde_json::Value> = text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["step"], 0);
        assert!(lines[0]["train_loss"].is_null());
        // Identity gates pick row 0 at the query row: the other target.
        assert_eq!(lines[0]["eval"]["1"]["chose_other"], 1);
        assert_eq!(lines[0]["eval"]["1"]["chose_target"], 0);
        assert_eq!(lines[0]["eval"]["2"]["n"], 1);
        assert!(lines[0]["train"]["1"]["chose_other"].is_null());
        assert_eq!(lines[0]["train"]["1"]["n"], 4);
        // Two windows a batch double the credit: two steps settle what took four on one.
        assert_eq!(lines[2]["step"], 8);
        assert_eq!(lines[2]["accepted_steps"], 2);
        assert_eq!(lines[2]["eval"]["1"]["chose_target"], 1);
        assert_eq!(lines[2]["eval"]["2"]["chose_target"], 1);
        assert_eq!(lines[2]["train"]["1"]["chose_target"], 4);
        assert_eq!(lines[2]["train_loss"], "0");
        let picks = std::fs::read(&args.save_picks).unwrap();
        assert_eq!(picks, words(&[0, 1, 2, 0, 1, 2]), "batch * time eval picks");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn host_mode_is_deterministic() {
        let dir = scratch("replay");
        let first = host_args(&dir, 4, "a");
        let second = host_args(&dir, 4, "b");
        run(&first).unwrap();
        run(&second).unwrap();
        let read = std::fs::read(&first.save_read).unwrap();
        assert_eq!(read, std::fs::read(&second.save_read).unwrap());
        let picks = std::fs::read(&first.save_picks).unwrap();
        assert_eq!(picks.len(), 6 * 4);
        assert_eq!(picks, std::fs::read(&second.save_picks).unwrap());
        assert_eq!(
            std::fs::read(&first.out_jsonl).unwrap(),
            std::fs::read(&second.out_jsonl).unwrap()
        );
        // A saved read restarts training from where it stopped.
        let mut resumed = host_args(&dir, 4, "c");
        resumed.start = first.save_read.clone();
        run(&resumed).unwrap();
        assert_eq!(read, std::fs::read(&resumed.save_read).unwrap());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// C1 and C3: `--start dump` begins from the train dump's own converted read, so zero steps
    /// save that read unchanged; and a readings file is never written over.
    #[test]
    fn the_dump_start_keeps_the_dump_read() {
        let dir = scratch("dump-start");
        let mut args = host_args(&dir, 4, "d");
        args.start = "dump".into();
        args.steps = 0;
        args.eval_at = vec![0];
        run(&args).unwrap();
        let saved = std::fs::read(&args.save_read).unwrap();
        assert_eq!(saved, dump(4).read.to_bytes());
        assert!(run(&args).is_err(), "an existing readings file is refused");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// claude-395 review: a run never writes over a saved read or picks file (a replay with a
    /// new readings file but the compared run's paths is refused before it trains), and an
    /// eval dump from another state (another read) is refused.
    #[test]
    fn saved_files_and_other_states_are_refused() {
        let dir = scratch("saved");
        let mut args = host_args(&dir, 4, "a");
        run(&args).unwrap();
        let read = std::fs::read(&args.save_read).unwrap();
        args.out_jsonl = dir.join("again.jsonl").to_string_lossy().into_owned();
        assert!(matches!(run(&args), Err(Error::Invalid(m)) if m.contains("never writes over")));
        assert!(
            !std::path::Path::new(&args.out_jsonl).exists(),
            "refused before it trains"
        );
        assert_eq!(read, std::fs::read(&args.save_read).unwrap());
        let other = host_args(&dir, 4, "b");
        let mut eval = dump(2);
        let gates = vec![[1 << 31, 0, 0, 0]; 4];
        eval.read = Q31FirstPickRead::new(gates.clone(), gates, vec![1; 16], 4).unwrap();
        std::fs::write(&other.eval_dump, eval.to_bytes().unwrap()).unwrap();
        assert!(matches!(run(&other), Err(Error::Invalid(m)) if m.contains("not the same state")));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mismatched_files_are_refused() {
        let dir = scratch("refuse");
        let args = host_args(&dir, 4, "a");
        std::fs::write(&args.eval_targets, words(&[0, 0, 2])).unwrap();
        assert!(matches!(run(&args), Err(Error::Invalid(m)) if m.contains("u32 words")));
        std::fs::write(&args.eval_targets, words(&[0, 0, 3].repeat(2))).unwrap();
        assert!(matches!(run(&args), Err(Error::Invalid(m)) if m.contains("earlier row")));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
