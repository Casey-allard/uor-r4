//! Remove the coarse lattice tier from a geometric prose artifact.
//!
//! Card P7 / D2 action. The 2026-09-19 ablation measured the coarse root-trigram tier as
//! BPB-NEGLIGIBLE while it occupies 64.2 % of the artifact and flips 9.5 % of decisions.
//! Removing it is a bytes-per-token (I2) decision, taken on resource cost rather than on an
//! accuracy gain, and it needs no retraining: the tier is a count table stored in the
//! artifact, and every other mechanism (fine clusters, lanes, readout, JEPA, engram,
//! codebook) is preserved byte-for-byte.
//!
//! Because `HierarchicalLatticeTables::coarse_score` returns `0` for an index beyond the
//! table, an artifact written with an absent coarse block scores exactly as one whose coarse
//! table is all zeros. This tool therefore produces the same scores as the `lattice_coarse`
//! ablation, at 64 % fewer bytes.
//!
//! This is an offline artifact tool. It performs no training and no model inference beyond a
//! deterministic scoring self-check.

use std::path::PathBuf;
use std::process::ExitCode;

use uor_r4_core::native_geometric::learner::{ExportedGeometricModel, MmapGeometricModel};

const DEFAULT_MODEL: &str = "native_geometric_prose_model.rgm";

fn usage() -> String {
    format!(
        "strip-coarse -- remove the coarse lattice tier from a .rgm artifact\n\
         \n\
         USAGE:\n  strip-coarse [--model <in.rgm>] [--out <out.rgm>]\n\
         \n\
         OPTIONS:\n\
         \x20 --model <path>   input artifact (.rgm or .json)  [default: {DEFAULT_MODEL}]\n\
         \x20 --out <path>     output .rgm                     [default: <model> with _nocoarse]\n\
         \x20 --help\n"
    )
}

struct Args {
    model: PathBuf,
    out: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut model = PathBuf::from(DEFAULT_MODEL);
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "--help" | "-h" => return Err(usage()),
            "--model" => {
                model = PathBuf::from(
                    argv.get(i + 1)
                        .ok_or_else(|| "missing value for --model".to_string())?,
                );
                i += 2;
            }
            "--out" => {
                out = Some(PathBuf::from(
                    argv.get(i + 1)
                        .ok_or_else(|| "missing value for --out".to_string())?,
                ));
                i += 2;
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    let out = match out {
        Some(p) => p,
        None => {
            let stem = model
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "model".to_string());
            let mut p = model.clone();
            p.set_file_name(format!("{stem}_nocoarse.rgm"));
            p
        }
    };
    Ok(Args { model, out })
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return Ok(());
        }
    };

    let bytes = std::fs::read(&args.model)?;
    let mut model = if bytes.len() >= 4 && &bytes[..4] == b"RGM1" {
        ExportedGeometricModel::from_binary(&bytes)?
    } else {
        serde_json::from_slice::<ExportedGeometricModel>(&bytes)?
    };

    let (removed, num_clusters, fine_len) = {
        let lattice = model
            .hierarchical_lattice
            .as_mut()
            .ok_or("artifact has no hierarchical lattice to strip")?;
        let removed = lattice.coarse_trigram.len();
        let num_clusters = lattice.num_clusters;
        let fine_len = lattice.fine_residual.len();
        lattice.coarse_trigram = Vec::new();
        (removed, num_clusters, fine_len)
    };
    if removed == 0 {
        println!("coarse tier is already absent; nothing written");
        return Ok(());
    }
    println!(
        "before: num_clusters={num_clusters} coarse_i8={removed} fine_i16={fine_len} \
         (artifact {} bytes)",
        bytes.len()
    );

    let out_bytes = model.to_binary()?;
    std::fs::write(&args.out, &out_bytes)?;
    let saved = bytes.len().saturating_sub(out_bytes.len());
    println!(
        "wrote {} ({} bytes); removed {} bytes ({:.1} % of the artifact)",
        args.out.display(),
        out_bytes.len(),
        saved,
        100.0 * saved as f64 / bytes.len() as f64
    );

    // Verification 1: the serialized artifact reloads through the owned reader with the
    // coarse tier absent and every other lattice field byte-identical.
    let reloaded = ExportedGeometricModel::from_binary(&out_bytes)?;
    let rl = reloaded
        .hierarchical_lattice
        .as_ref()
        .ok_or("reloaded artifact lost the lattice section")?;
    let ml = model
        .hierarchical_lattice
        .as_ref()
        .ok_or("stripped model lost the lattice section")?;
    if !rl.coarse_trigram.is_empty() {
        return Err("reloaded artifact still carries a coarse tier".into());
    }
    if rl.fine_residual != ml.fine_residual || rl.token_to_cluster != ml.token_to_cluster {
        return Err("reloaded artifact changed the fine tier or the cluster map".into());
    }
    if rl.num_clusters != ml.num_clusters {
        return Err("reloaded artifact changed num_clusters".into());
    }

    // Verification 2: scoring is exactly the fine-only value, compared against the
    // in-memory stripped model over a deterministic sample, for both self-transition branches.
    let mut checked = 0usize;
    let mut mismatches = 0usize;
    for r_prev in [0usize, 7, 61, 119] {
        for r_curr in [0usize, 13, 88, 119] {
            for r_cand in [0usize, 40, 119] {
                for c_curr in [0usize, 5, 239.min(num_clusters.saturating_sub(1))] {
                    for c_cand in [0usize, 9, 239.min(num_clusters.saturating_sub(1))] {
                        for is_self in [false, true] {
                            let a = rl.score_token(r_prev, r_curr, r_cand, c_curr, c_cand, is_self);
                            let b = ml.score_token(r_prev, r_curr, r_cand, c_curr, c_cand, is_self);
                            checked += 1;
                            if a != b {
                                mismatches += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    println!("scoring self-check: {checked} comparisons, {mismatches} mismatches");
    if mismatches != 0 {
        return Err("stripped artifact does not score identically to the stripped model".into());
    }

    // Verification 3: the zero-copy mmap reader agrees that the coarse tier is absent and the
    // fine tier is present, and reads the same length the owned reader does.
    let mmap = MmapGeometricModel::open(&args.out)?;
    if mmap.coarse_trigram().is_some() {
        return Err("mmap reader still exposes a coarse tier".into());
    }
    match mmap.fine_residual() {
        Some(f) if f.len() == fine_len => {}
        _ => return Err("mmap reader disagrees about the fine tier".into()),
    }
    println!("mmap verification: coarse absent, fine tier present and the expected length");

    Ok(())
}
