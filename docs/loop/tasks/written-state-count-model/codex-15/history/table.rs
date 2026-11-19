//! Inspect the packed roots in a sealed D11 asset, without executing the model.
use std::path::Path;
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = std::env::args().nth(1).ok_or("expected the frozen D11 path")?;
    let roots = IntegerDiscreteModel::load(Path::new(&model))?.packed_codebook_roots();
    if roots.len() != 4096 || roots.iter().any(|&root| root > 65535) {
        return Err("expected 4096 packed four-root words".into());
    }
    println!("{roots:?}");
    Ok(())
}
