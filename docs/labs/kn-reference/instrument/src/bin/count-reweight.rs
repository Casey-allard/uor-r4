//! Cross a frozen source count state with a separate TRAIN-fitted conditional weight.
use serde_json::json;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use uor_r4_integer::count_mixer::{pack, VOCAB};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

fn main() -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [base, full, donor, new] = paths.as_slice() else {
        return Err(
            "usage: count-reweight BASE.d11 FULL.d11 WEIGHT_SOURCE.d11 NEW_MODEL.d11".into(),
        );
    };
    if base.as_os_str() == "--fixture-donor" {
        let carrier = fs::read(full)?;
        let source = IntegerDiscreteModel::load(donor)?;
        let fitted = source.count_mixer().ok_or("missing finite mixer")?;
        let mut words = fitted.words().to_vec();
        if fitted.conditional_weight() == 128 {
            return Err("fixture already has the fixed control weight".into());
        }
        words[0..4].copy_from_slice(&128u32.to_le_bytes());
        let packed = pack(&carrier, &words)?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(new)?;
        file.write_all(&packed)?;
        file.sync_all()?;
        return Ok(());
    }
    let base = fs::read(base)?;
    let source_bytes = fs::read(full)?;
    let source = IntegerDiscreteModel::from_bytes(&source_bytes)?;
    let donor_bytes = fs::read(donor)?;
    let donor = IntegerDiscreteModel::from_bytes(&donor_bytes)?;
    let full_mix = source.count_mixer().ok_or("expected count source")?;
    let donor_mix = donor.count_mixer().ok_or("expected count weight donor")?;
    let mut words = full_mix.words().to_vec();
    if pack(&base, &words)? != source_bytes || pack(&base, donor_mix.words())? != donor_bytes {
        return Err("carrier identity/canonical encoding mismatch".into());
    }
    if words[4..(VOCAB + 1) * 4] != donor_mix.words()[4..(VOCAB + 1) * 4] {
        return Err("different unigram or count data source".into());
    }
    if full_mix.conditional_weight() == donor_mix.conditional_weight() {
        return Err("uninformative crossed weight".into());
    }
    words[..4].copy_from_slice(&donor_mix.conditional_weight().to_le_bytes());
    let packed = pack(&base, &words)?;
    let model = IntegerDiscreteModel::from_bytes(&packed)?;
    let mix = model.count_mixer().ok_or("missing crossed count state")?;
    if packed.len() != source_bytes.len()
        || mix.words()[4..] != full_mix.words()[4..]
        || mix.conditional_weight() != donor_mix.conditional_weight()
        || model.tokenizer_sha256() != source.tokenizer_sha256()
    {
        return Err("unexpected change while crossing weight".into());
    }
    let mut out = OpenOptions::new().write(true).create_new(true).open(new)?;
    out.write_all(&packed)?;
    out.sync_all()?;
    println!(
        "{}",
        json!({"source":source.artifact_sha256(),"weight_donor":donor.artifact_sha256(),
        "original_weight":full_mix.conditional_weight(),"crossed_weight":mix.conditional_weight(),
        "output_sha256":model.artifact_sha256(),"source_and_output_bytes":packed.len(),
        "tokenizer_sha256":model.tokenizer_sha256(),"unchanged_native_bytes_after_weight":words.len()-4})
    );
    Ok(())
}
