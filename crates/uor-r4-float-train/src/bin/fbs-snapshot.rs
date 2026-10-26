//! Convert a saved SpiralCore/FBS float trunk (`StackModel::save`) to the
//! integer crate's snapshot JSON (d11-exporter-float-trunk, claude-123).
//!
//! It runs on one GPU, because the routes are read back through the model's
//! CUDA projection (`fbs_export`). The serving build's `fbs-export` turns the
//! JSON into the `UORLUT01` artifact; with `READOUT_JSON`, it also writes the
//! float trunk's own single-token outputs for the serving build's
//! `fbs-compare`.
//!
//! ```text
//! fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON]
//! ```
//!
//! It never replaces or leaves a partial output file, and it prints each
//! file's sha256.

use std::path::Path;

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::{fbs_float_readout, fbs_trunk_snapshot};
use uor_r4_float_train::geometric_stack::StackModel;
use uor_r4_float_train::Result;
use uor_r4_integer::fbs_snapshot::write_new_file;

fn write(path: &str, json: &[u8]) -> Result<()> {
    write_new_file(Path::new(path), json)?;
    println!("{}  {path}", hex::encode(Sha256::digest(json)));
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (model_dir, out, readout) = match args.as_slice() {
        [model_dir, out] => (model_dir, out, None),
        [model_dir, out, readout] => (model_dir, out, Some(readout)),
        _ => {
            eprintln!("usage: fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON]");
            std::process::exit(2);
        }
    };
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(Path::new(model_dir), &device)?;
    write(out, &serde_json::to_vec(&fbs_trunk_snapshot(&model)?)?)?;
    if let Some(readout) = readout {
        write(readout, &serde_json::to_vec(&fbs_float_readout(&model)?)?)?;
    }
    Ok(())
}
