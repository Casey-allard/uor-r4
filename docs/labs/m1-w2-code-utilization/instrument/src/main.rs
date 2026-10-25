//! `w2-utilization --capture <w2-out-b> --record <record.json> --out <dir> --discrete-commit <sha>`
//!
//! Runs the registered sheet once and writes `<dir>/record.json`. It refuses an
//! existing output, and it exits 2 on an instrument fault (no band is read).

use std::path::PathBuf;
use std::process::ExitCode;

use m1_w2_code_utilization_instrument as inst;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut capture = None;
    let mut record = None;
    let mut out = None;
    let mut commit = None;
    let mut i = 0;
    while i + 1 < args.len() {
        let value = args[i + 1].clone();
        match args[i].as_str() {
            "--capture" => capture = Some(PathBuf::from(value)),
            "--record" => record = Some(PathBuf::from(value)),
            "--out" => out = Some(PathBuf::from(value)),
            "--discrete-commit" => commit = Some(value),
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::from(1);
            }
        }
        i += 2;
    }
    let (Some(capture), Some(record), Some(out), Some(commit)) = (capture, record, out, commit)
    else {
        eprintln!(
            "usage: w2-utilization --capture DIR --record FILE --out DIR --discrete-commit SHA"
        );
        return ExitCode::from(1);
    };
    if commit.len() != 40 || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
        eprintln!("--discrete-commit must be a 40-hex commit");
        return ExitCode::from(1);
    }
    let target = out.join("record.json");
    if target.exists() {
        eprintln!("{} exists; refusing to overwrite", target.display());
        return ExitCode::from(1);
    }
    let result = inst::load(&capture, &record).and_then(|(inputs, rec)| {
        let first = inst::run(&inputs, &rec, &commit)?;
        let second = inst::run(&inputs, &rec, &commit)?;
        let a =
            serde_json::to_string_pretty(&first).map_err(|e| inst::Fault(vec![e.to_string()]))?;
        let b =
            serde_json::to_string_pretty(&second).map_err(|e| inst::Fault(vec![e.to_string()]))?;
        if a != b {
            return Err(inst::Fault(vec!["two runs differ (determinism)".into()]));
        }
        Ok((first, a))
    });
    match result {
        Err(fault) => {
            eprintln!("{fault}");
            ExitCode::from(2)
        }
        Ok((r, text)) => {
            if let Err(e) =
                std::fs::create_dir_all(&out).and_then(|()| std::fs::write(&target, &text))
            {
                eprintln!("{}: {e}", target.display());
                return ExitCode::from(1);
            }
            println!(
                "record {} sha256 {}",
                target.display(),
                inst::sha256_hex(text.as_bytes())
            );
            println!("precedence 0: {} checks OK", r.precedence_0.checks.len());
            println!(
                "QA {}: A_W {} A_T {} A_U {} A_G {} A_Omega {}; T-W {:?}; T-U {:?}",
                r.qa.band,
                r.qa.a_w,
                r.qa.a_t,
                r.qa.a_u,
                r.qa.a_g,
                r.qa.a_omega,
                r.qa.pair_t_w,
                r.qa.pair_t_u
            );
            println!(
                "QB {}: d_T {} T_proj {}; G1 {}",
                r.qb.result.band, r.qb.result.d_t, r.qb.result.t_proj, r.qb.g1
            );
            println!(
                "QC {}: J {} G {} joint-seen rows {}; J-G {:?}",
                r.qc.band, r.qc.j, r.qc.g, r.qc.joint_seen_rows, r.qc.pair_j_g
            );
            println!("arm R: {}", r.arm_r.status);
            ExitCode::SUCCESS
        }
    }
}
