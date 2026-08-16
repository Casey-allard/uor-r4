#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let id = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "uor-r4-scale-sweep-729-{}-{id}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("create unique scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn write_meta(path: &Path, records: u64) {
    let mut bytes = Vec::with_capacity(25);
    bytes.extend_from_slice(&records.to_le_bytes());
    bytes.extend_from_slice(&10_u64.to_le_bytes());
    bytes.extend_from_slice(&7_u64.to_le_bytes());
    bytes.push(1);
    std::fs::write(path, bytes).expect("write finalized source metadata");
}

fn write_fake_r4(path: &Path) {
    std::fs::write(
        path,
        r#"#!/usr/bin/env python3
import json
import os
from pathlib import Path
import shutil
import struct
import sys

args = sys.argv[1:]
with open(os.environ["FAKE_LOG"], "a", encoding="utf-8") as log:
    log.write(" ".join(args) + "\n")

def value(flag):
    return args[args.index(flag) + 1]

operation = args[1]
if operation == "subsample-recorded-corpus":
    source_meta = Path(value("--src-meta"))
    source_n = struct.unpack_from("<Q", source_meta.read_bytes())[0]
    requested = int(value("--records"))
    actual = requested if requested == source_n else requested - 1
    out_meta = Path(value("--out-meta"))
    out_recs = Path(value("--out-recs"))
    out_meta.parent.mkdir(parents=True, exist_ok=True)
    out_meta.write_bytes(struct.pack("<QQQ", actual, 10, 7) + b"\x01")
    out_recs.write_bytes(b"")
elif operation == "compile-recorded":
    output = Path(value("--out"))
    output.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(value("--corpus-meta"), output / "corpus.meta")
    shutil.copyfile(value("--corpus-recs"), output / "corpus.records")
    (output / "tless_artifacts.bin").write_bytes(b"artifacts")
    (output / "attention_operator.json").write_text('{"schema":2}')
    if os.environ.get("FAKE_DENSE") == "1":
        (output / "dense_operator.json").write_text('{"schema":2}')
elif operation == "cover":
    output = Path(value("--out"))
    output.mkdir(parents=True, exist_ok=True)
    actual = struct.unpack_from("<Q", Path(value("--corpus-meta")).read_bytes())[0]
    train = actual * 4 // 5
    held = actual - train
    if os.environ.get("FAKE_BAD_COUNTS") == "1":
        held += 1
    report = {"inputs": {"train_observations": train, "held_out_observations": held}}
    (output / "cover_report.json").write_text(json.dumps(report))
    (output / "cover.r4g1").write_bytes(b"cover")
elif operation == "score":
    output = Path(value("--out"))
    output.mkdir(parents=True, exist_ok=True)
    cover_report = Path(value("--cover")).parent / "cover_report.json"
    held = json.loads(cover_report.read_text())["inputs"]["held_out_observations"]
    report = {
        "gate_c": {
            "held_out_population": held,
            "rule12_precedence": {"top1_agreement": 0.25},
        },
        "distribution": {"held_out_positions": held, "exct_miss_rate": 0.5},
    }
    (output / "score_report.json").write_text(json.dumps(report))
else:
    raise SystemExit(f"unexpected operation: {operation}")
"#,
    )
    .expect("write fake r4");
    let mut permissions = std::fs::metadata(path)
        .expect("fake r4 metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make fake r4 executable");
}

struct SweepOptions<'a> {
    targets: &'a [&'a str],
    dense: bool,
    bad_counts: bool,
}

fn run_sweep(
    source_meta: &Path,
    source_recs: &Path,
    fake_r4: &Path,
    work: &Path,
    log: &Path,
    options: SweepOptions<'_>,
) -> Output {
    let mut command = Command::new("bash");
    command
        .arg(workspace_root().join("scripts/scale_sweep.sh"))
        .arg(source_meta)
        .arg(source_recs)
        .arg("49152")
        .args(options.targets)
        .env("R4", fake_r4)
        .env("WORK", work)
        .env("FAKE_LOG", log);
    if options.dense {
        command.env("FAKE_DENSE", "1");
    }
    if options.bad_counts {
        command.env("FAKE_BAD_COUNTS", "1");
    }
    command.output().expect("run scale sweep")
}

fn fixture(scratch: &Scratch, records: u64) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let source_meta = scratch.0.join("source.meta");
    let source_recs = scratch.0.join("source.records");
    let fake_r4 = scratch.0.join("fake-r4");
    let log = scratch.0.join("calls.log");
    write_meta(&source_meta, records);
    std::fs::write(&source_recs, b"").expect("write source records fixture");
    write_fake_r4(&fake_r4);
    (source_meta, source_recs, fake_r4, log)
}

#[test]
fn fixed_partition_workflow_is_rerunnable_and_reports_requested_vs_actual() {
    let scratch = Scratch::new();
    let (meta, recs, fake_r4, log) = fixture(&scratch, 20);
    let work = scratch.0.join("work");

    let first = run_sweep(
        &meta,
        &recs,
        &fake_r4,
        &work,
        &log,
        SweepOptions {
            targets: &["10", "20"],
            dense: true,
            bad_counts: false,
        },
    );
    assert!(
        first.status.success(),
        "first sweep failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_stdout = String::from_utf8(first.stdout).expect("UTF-8 sweep output");
    let rows: Vec<Vec<&str>> = first_stdout
        .lines()
        .filter(|line| line.starts_with(|character: char| character.is_ascii_digit()))
        .map(|line| line.split_whitespace().collect())
        .collect();
    assert_eq!(rows[0][..4], ["10", "9", "7", "2"]);
    assert_eq!(rows[1][..4], ["20", "20", "16", "4"]);

    for target in ["10", "20"] {
        let case = work.join(format!("n-{target}"));
        for sibling in ["input", "compiled", "cover", "score"] {
            assert!(case.join(sibling).is_dir(), "missing {target}/{sibling}");
        }
    }
    let first_log = std::fs::read_to_string(&log).expect("read first invocation log");
    assert!(first_log.contains("subsample-recorded-corpus"));
    assert!(
        first_log.contains("--records 20"),
        "exact source was not sampled"
    );
    assert!(first_log.contains("--dense-operator"));
    assert!(first_log.contains("n-10/compiled"));
    assert!(first_log.contains("n-10/cover"));
    assert!(first_log.contains("n-10/score"));

    let second = run_sweep(
        &meta,
        &recs,
        &fake_r4,
        &work,
        &log,
        SweepOptions {
            targets: &["10", "20"],
            dense: true,
            bad_counts: false,
        },
    );
    assert!(second.status.success());
    assert_eq!(first_stdout.as_bytes(), second.stdout);
}

#[test]
fn all_targets_are_rejected_before_work_or_r4_mutation() {
    let scratch = Scratch::new();
    let (meta, recs, fake_r4, log) = fixture(&scratch, 20);
    let work = scratch.0.join("must-not-exist");

    let output = run_sweep(
        &meta,
        &recs,
        &fake_r4,
        &work,
        &log,
        SweepOptions {
            targets: &["10", "21"],
            dense: false,
            bad_counts: false,
        },
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("exceeds finalized source size"));
    assert!(!work.exists(), "preflight failure created WORK");
    assert!(!log.exists(), "preflight failure invoked r4");
}

#[test]
fn default_targets_are_bounded_candidates_plus_exact_source() {
    let scratch = Scratch::new();
    let (meta, recs, fake_r4, log) = fixture(&scratch, 900_001);
    let output = run_sweep(
        &meta,
        &recs,
        &fake_r4,
        &scratch.0.join("work"),
        &log,
        SweepOptions {
            targets: &[],
            dense: false,
            bad_counts: false,
        },
    );
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 sweep output");
    let requested: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with(|character: char| character.is_ascii_digit()))
        .map(|line| line.split_whitespace().next().expect("requested column"))
        .collect();
    assert_eq!(requested, ["50000", "200000", "800000", "900001"]);
}

#[test]
fn attention_only_cover_omits_the_dense_argument() {
    let scratch = Scratch::new();
    let (meta, recs, fake_r4, log) = fixture(&scratch, 20);
    let output = run_sweep(
        &meta,
        &recs,
        &fake_r4,
        &scratch.0.join("work"),
        &log,
        SweepOptions {
            targets: &["20"],
            dense: false,
            bad_counts: false,
        },
    );
    assert!(
        output.status.success(),
        "attention-only sweep failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let calls = std::fs::read_to_string(log).expect("read attention-only invocation log");
    let cover = calls
        .lines()
        .find(|line| line.starts_with("transformerless cover "))
        .expect("cover invocation");
    assert!(cover.contains("--attention-operator"));
    assert!(!cover.contains("--dense-operator"));
}

#[test]
fn inconsistent_cover_counts_are_terminal() {
    let scratch = Scratch::new();
    let (meta, recs, fake_r4, log) = fixture(&scratch, 20);
    let output = run_sweep(
        &meta,
        &recs,
        &fake_r4,
        &scratch.0.join("work"),
        &log,
        SweepOptions {
            targets: &["10"],
            dense: false,
            bad_counts: true,
        },
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cover partition mismatch"));
}

#[test]
fn retired_python_writer_never_touches_outputs() {
    let scratch = Scratch::new();
    let out_meta = scratch.0.join("sentinel.meta");
    let out_recs = scratch.0.join("sentinel.records");
    std::fs::write(&out_meta, b"meta sentinel").expect("write metadata sentinel");
    std::fs::write(&out_recs, b"records sentinel").expect("write records sentinel");

    let output = Command::new("python3")
        .arg(workspace_root().join("scripts/mc1_subsample_corpus.py"))
        .args([
            "--src-meta",
            "missing.meta",
            "--src-recs",
            "missing.records",
        ])
        .arg("--out-meta")
        .arg(&out_meta)
        .arg("--out-recs")
        .arg(&out_recs)
        .args(["--records", "10"])
        .output()
        .expect("run retired Python writer");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("is retired"));
    assert_eq!(
        std::fs::read(&out_meta).expect("read metadata sentinel"),
        b"meta sentinel"
    );
    assert_eq!(
        std::fs::read(&out_recs).expect("read records sentinel"),
        b"records sentinel"
    );
}
