//! Convert a saved SpiralCore/FBS float trunk (`StackModel::save`) to the
//! integer crate's snapshot JSON (d11-exporter-float-trunk, claude-123).
//!
//! It runs on one GPU, because the routes are read back through the model's
//! CUDA projection (`fbs_export`). The serving build's `fbs-export` turns the
//! JSON into the `UORLUT01` artifact. The optional readouts, for the serving
//! build's `fbs-compare`, are the float trunk's own outputs, in order:
//! - `READOUT_JSON`: plain;
//! - `EXPORT_READOUT_JSON`: in the float sheets' D11 export mode;
//! - `D11_READOUT_JSON`: in the D11 mode, after rounding the loaded trunk's
//!   tables as exported (claude-125).
//!
//! ```text
//! fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]
//! ```
//!
//! It never replaces or leaves a partial output file, and it prints each
//! file's sha256.

use std::path::Path;

use candle_core::Device;
use sha2::{Digest, Sha256};
use uor_r4_float_train::fbs_export::{
    fbs_float_readout, fbs_trunk_snapshot, round_tables_as_exported,
};
use uor_r4_float_train::geometric_stack::StackModel;
use uor_r4_float_train::Result;
use uor_r4_integer::fbs_snapshot::{write_new_file, FbsReadoutMode};

fn write(path: &str, json: &[u8]) -> Result<()> {
    write_new_file(Path::new(path), json)?;
    println!("{}  {path}", hex::encode(Sha256::digest(json)));
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (model_dir, out, readouts) = match args.as_slice() {
        [model_dir, out, readouts @ ..] if readouts.len() <= 3 => (model_dir, out, readouts),
        _ => {
            eprintln!(
                "usage: fbs-snapshot MODEL_DIR OUT_JSON [READOUT_JSON [EXPORT_READOUT_JSON [D11_READOUT_JSON]]]"
            );
            std::process::exit(2);
        }
    };
    let device = Device::new_cuda(0)?;
    let model = StackModel::load(Path::new(model_dir), &device)?;
    write(out, &serde_json::to_vec(&fbs_trunk_snapshot(&model)?)?)?;
    for (path, mode) in readouts.iter().zip([
        FbsReadoutMode::Plain,
        FbsReadoutMode::Export,
        FbsReadoutMode::D11,
    ]) {
        if mode == FbsReadoutMode::D11 {
            round_tables_as_exported(&model)?;
        }
        write(
            path,
            &serde_json::to_vec(&fbs_float_readout(&model, mode)?)?,
        )?;
    }
    Ok(())
}
