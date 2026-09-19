//! Set the VSA token-code mode on a geometric prose artifact.
//!
//! Card P7 / D2 follow-up. The 2026-09-19 routing measurement showed that codes derived from the
//! learned 120-root assignment (`root`) beat the fixed token-id hash (`fixed`) on every routing
//! metric — routed recall 8.4–8.8 % → 10.7–11.2 %, served-path shortlist BPB 2.72–2.83 → 2.67–2.78
//! — replicated across two disjoint slices. `decisions/DECISIONS.md` D2 requires that a measured
//! improvement not stay an evaluation-only switch, so the mode must be declarable **in the
//! artifact** rather than chosen by a test harness.
//!
//! The mode is stored in header byte 26, which was `_reserved[0]`. That keeps the header size and
//! the overall section layout unchanged: artifacts written before the field existed read back as
//! mode `0`, and `blake3` integrity still covers the whole payload.
//!
//! # The coherence requirement this tool enforces
//!
//! `HierarchicalCodebook` centroids are bundles of the token vectors used **at export time**.
//! Selecting a different code space without rebuilding those centroids would leave the router
//! comparing a query vector against centroids from another space. `prepare_vsa_code_mode` performs
//! the rebuild, and this tool verifies that the rebuild happened: for mode `1` the sector and
//! bucket *structure* must be unchanged while the anchor *contents* change.
//!
//! Offline artifact tool. No training, no inference beyond structural verification.

use std::path::PathBuf;
use std::process::ExitCode;

use uor_r4_core::native_geometric::learner::{
    build_root_codebook, ExportedGeometricModel, MmapGeometricModel,
};
use uor_r4_core::native_geometric::vsa::{Codebook, HierarchicalCodebook};

const DEFAULT_MODEL: &str = "native_geometric_prose_model.rgm";

fn usage() -> String {
    format!(
        "set-vsa-code-mode -- declare the VSA token-code mode in a .rgm artifact\n\
         \n\
         USAGE:\n  set-vsa-code-mode --mode <fixed|root> [--model <in.rgm>] [--out <out.rgm>]\n\
         \n\
         OPTIONS:\n\
         \x20 --mode <fixed|root>  REQUIRED. fixed = token-id hash (0); root = codes from the\n\
         \x20                     learned 120-root assignment (1)\n\
         \x20 --model <path>      input artifact (.rgm or .json)  [default: {DEFAULT_MODEL}]\n\
         \x20 --out <path>        output .rgm  [default: <model>_<mode>.rgm]\n\
         \x20 --help\n"
    )
}

struct Args {
    model: PathBuf,
    out: PathBuf,
    mode: u8,
    mode_name: &'static str,
}

fn parse_args() -> Result<Args, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut model = PathBuf::from(DEFAULT_MODEL);
    let mut out: Option<PathBuf> = None;
    let mut mode: Option<(u8, &'static str)> = None;
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
            "--mode" => {
                let v = argv
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --mode".to_string())?
                    .to_ascii_lowercase();
                mode = Some(match v.as_str() {
                    "fixed" | "0" => (0, "fixed"),
                    "root" | "1" => (1, "root"),
                    other => {
                        return Err(format!("unknown --mode '{other}'; known: fixed, root"));
                    }
                });
                i += 2;
            }
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
    }
    let (mode, mode_name) = mode.ok_or_else(|| format!("--mode is required\n\n{}", usage()))?;
    let out = match out {
        Some(p) => p,
        None => {
            let stem = model
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "model".to_string());
            let mut p = model.clone();
            p.set_file_name(format!("{stem}_{mode_name}.rgm"));
            p
        }
    };
    Ok(Args {
        model,
        out,
        mode,
        mode_name,
    })
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

/// Summarise a hierarchical codebook's structure and anchor contents, so the two can be compared
/// independently: a correct rebuild changes the contents and leaves the structure alone.
fn summarise(h: &HierarchicalCodebook<64>) -> (usize, usize, Vec<u64>) {
    let non_empty_sectors = h.sectors.iter().filter(|s| !s.is_empty()).count();
    let buckets: usize = h.sectors.iter().map(|s| s.len()).sum();
    let mut anchor_words = Vec::with_capacity(120 * 8);
    for a in h.root_anchors.iter() {
        anchor_words.extend_from_slice(&a.data[..8.min(a.data.len())]);
    }
    (non_empty_sectors, buckets, anchor_words)
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
    println!(
        "before: vsa_code_mode={} (artifact {} bytes)",
        model.vsa_code_mode,
        bytes.len()
    );
    if model.vsa_code_mode == args.mode {
        println!(
            "artifact already declares mode {} ({})",
            args.mode, args.mode_name
        );
    }

    // Build both code spaces so the rebuild can be verified structurally.
    let fixed_codebook = Codebook::<64>::on_demand(model.vocab_size, model.vsa_seed);
    let root_codebook = build_root_codebook(model.vocab_size, &model.token_to_root, model.vsa_seed);
    let selected = if args.mode == 1 {
        &root_codebook
    } else {
        &fixed_codebook
    };
    let expect_h =
        HierarchicalCodebook::<64>::new(model.vocab_size, &model.token_to_root, selected);
    let baseline_h =
        HierarchicalCodebook::<64>::new(model.vocab_size, &model.token_to_root, &fixed_codebook);
    let (exp_sectors, exp_buckets, exp_anchors) = summarise(&expect_h);
    let (base_sectors, base_buckets, base_anchors) = summarise(&baseline_h);
    // The partition structure depends only on `token_to_root` and the leaf-bucket capacity, not on
    // the code space, so changing the code must not move a single token to another bucket.
    if (base_sectors, base_buckets) != (exp_sectors, exp_buckets) {
        return Err(format!(
            "the two code spaces disagree on structure: fixed ({base_sectors},{base_buckets}) vs selected ({exp_sectors},{exp_buckets})"
        )
        .into());
    }

    model.vsa_code_mode = args.mode;
    model
        .prepare_vsa_code_mode()
        .map_err(|e| format!("prepare_vsa_code_mode: {e}"))?;

    let out_bytes = model.to_binary()?;
    std::fs::write(&args.out, &out_bytes)?;
    println!(
        "wrote {} ({} bytes); vsa_code_mode={} ({})",
        args.out.display(),
        out_bytes.len(),
        args.mode,
        args.mode_name
    );
    if out_bytes.len() != bytes.len() {
        return Err(format!(
            "artifact size changed: {} -> {} bytes; the mode is supposed to be size-compatible",
            bytes.len(),
            out_bytes.len()
        )
        .into());
    }

    // Verification 1: the mode survives a round trip through both readers.
    let reloaded = ExportedGeometricModel::from_binary(&out_bytes)?;
    if reloaded.vsa_code_mode != args.mode {
        return Err("owned reader did not round-trip vsa_code_mode".into());
    }
    let mmap = MmapGeometricModel::open(&args.out)?;
    if mmap.vsa_code_mode() != args.mode {
        return Err("mmap reader did not round-trip vsa_code_mode".into());
    }

    // Verification 2: for `root`, the rebuild happened — structure identical, anchors changed.
    let actual_h = reloaded
        .hierarchical_codebook
        .as_ref()
        .ok_or("reloaded artifact lost the hierarchical codebook")?;
    let (act_sectors, act_buckets, act_anchors) = summarise(actual_h);
    if (act_sectors, act_buckets) != (exp_sectors, exp_buckets) {
        return Err(format!(
            "rebuild changed the structure: sectors/buckets ({act_sectors},{act_buckets}) != expected ({exp_sectors},{exp_buckets})"
        )
        .into());
    }
    let differing = act_anchors
        .iter()
        .zip(base_anchors.iter())
        .filter(|(a, b)| a != b)
        .count();
    println!(
        "verification: structure unchanged ({} sectors, {} buckets); anchors differing from the fixed-hash build: {} of {} words",
        act_sectors,
        act_buckets,
        differing,
        base_anchors.len()
    );
    if args.mode == 1 && differing == 0 {
        return Err("mode 1 rebuilt a codebook identical to the fixed-hash build".into());
    }
    if args.mode == 0 && differing != 0 {
        return Err("mode 0 was expected to keep the export-time codebook unchanged".into());
    }
    if args.mode == 1 && exp_anchors == base_anchors {
        return Err("expected the root code space to differ from the fixed hash".into());
    }
    println!("artifact blake3: {}", blake3::hash(&out_bytes).to_hex());
    Ok(())
}
