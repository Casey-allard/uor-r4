use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, io::Write, path::Path};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub type Span = [usize; 2];
pub const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub row: u32,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle: Option<[usize; 4]>,
}

pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn write_manifest(directory: &Path, output: &str, files: &[&str]) -> Result<()> {
    let mut manifest = fs::File::create_new(directory.join(output))?;
    for &file in files {
        writeln!(
            manifest,
            "{}  {file}",
            sha(&fs::read(directory.join(file))?)
        )?;
    }
    Ok(())
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}
