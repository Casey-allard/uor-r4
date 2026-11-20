use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::{
    all_starts::{Bank, OnsetMode, OnsetProfile, NEGATIVE_CALIBRATION_BYTES},
    discrete_d11::pack_rival_admission,
};
#[path = "../../codex-28/writer/replay.rs"]
mod replay;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PARENT: &str = "72f962ebc0d5c7ddba7e9b0f4643b3733c42181f5b35b0ae8620d11006c0936d";
const FITTED: &str = "dc478bd042d1134606c81506349771c6e4cd8093c8f51612f25139a83888abc8";
fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    let [carrier, baseline, bank_file, panel, client, out] = &args[..] else {
        return Err(
            "expected CARRIER BASELINE FITTED.bank PANEL CLIENT_DIRECTORY_OR_DASH NEW_OUT".into(),
        );
    };
    let parent = fs::read(baseline)?;
    let fitted = fs::read(bank_file)?;
    if hex::encode(Sha256::digest(&parent)) != PARENT
        || hex::encode(Sha256::digest(&fitted)) != FITTED
    {
        return Err("wrong frozen parent or bank".into());
    }
    replay::grade_with(
        Path::new(carrier),
        Path::new(panel),
        (client != "-").then(|| Path::new(client)),
        Path::new(out),
        |bytes, mode| {
            if mode != OnsetMode::OrderedWholeWord {
                return Err("wrong value policy".into());
            }
            let bank = Bank::from_profile(
                &fitted,
                fitted[16..48].try_into()?,
                fitted[48..80].try_into()?,
                fitted[80..112].try_into()?,
                &OnsetProfile {
                    mode,
                    train_sha256: hex::encode(&fitted[112..144]),
                    labels_sha256: hex::encode(&fitted[144..176]),
                },
            )?;
            let bank = if bytes == parent {
                bank
            } else {
                let tail = bytes
                    .strip_suffix(parent.as_slice())
                    .ok_or("carrier lacks required parent")?;
                let offset = tail
                    .len()
                    .checked_sub(NEGATIVE_CALIBRATION_BYTES)
                    .ok_or("short calibration")?;
                let cal = &tail[offset..];
                if pack_rival_admission(&parent, fitted[112..144].try_into()?, cal)? != bytes {
                    return Err("calibrated carrier reconstruction failed".into());
                }
                bank.with_negative_calibration(cal)?
            };
            Ok((bank, fitted[208..].to_vec()))
        },
        true,
    )
}
