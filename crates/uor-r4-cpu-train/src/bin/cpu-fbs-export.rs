use std::path::PathBuf;

use uor_r4_cpu_train::export_saved_checkpoint;
use uor_r4_integer::stack::FbsExportSnaps;

fn main() {
    if let Err(error) = run() {
        eprintln!("cpu-fbs-export: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut checkpoint = None;
    let mut run_config = None;
    let mut snapshot_out = None;
    let mut artifact_out = None;
    let mut snaps = FbsExportSnaps::SNAPPED;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--checkpoint" => checkpoint = Some(path(&mut args, "--checkpoint")?),
            "--run-config" => run_config = Some(path(&mut args, "--run-config")?),
            "--snapshot-out" => snapshot_out = Some(path(&mut args, "--snapshot-out")?),
            "--out" => artifact_out = Some(path(&mut args, "--out")?),
            "--raw-gates" if !snaps.gates => return Err("--raw-gates given twice".to_owned()),
            "--raw-gates" => snaps.gates = false,
            "--raw-transition" if !snaps.transition => {
                return Err("--raw-transition given twice".to_owned());
            }
            "--raw-transition" => snaps.transition = false,
            "--help" | "-h" => return Err(usage()),
            _ => return Err(format!("unknown argument {argument}\n{}", usage())),
        }
    }
    let checkpoint = checkpoint.ok_or_else(usage)?;
    let run_config = run_config.ok_or_else(usage)?;
    let snapshot_out = snapshot_out.ok_or_else(usage)?;
    let artifact_out = artifact_out.ok_or_else(usage)?;
    let (snapshot_sha256, artifact_sha256) = export_saved_checkpoint(
        &checkpoint,
        &run_config,
        &snapshot_out,
        &artifact_out,
        snaps,
    )?;
    println!(
        "{}",
        serde_json::json!({
            "schema": "uor-r4.cpu-fbs-export/1",
            "checkpoint": checkpoint,
            "run_config": run_config,
            "snapshot": snapshot_out,
            "snapshot_sha256": snapshot_sha256,
            "artifact": artifact_out,
            "artifact_sha256": artifact_sha256,
            "raw_gates": !snaps.gates,
            "raw_transition": !snaps.transition,
        })
    );
    Ok(())
}

fn path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} needs a value\n{}", usage()))
}

fn usage() -> String {
    "usage: cpu-fbs-export --checkpoint DIR --run-config FILE --snapshot-out FILE --out FILE [--raw-gates] [--raw-transition]".to_owned()
}
