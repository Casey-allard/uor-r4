use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::{
    all_starts::{Bank, OnsetMode, OnsetProfile, BANK_BYTES},
    discrete_d11::{pack_requested_occurrence_artifact, IntegerDiscreteModel},
    paired_role::RequestedContext,
};
mod replay;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PARENT: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";
const BINDINGS: usize = 208;

fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}
fn sha(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

struct Parts<'a> {
    bank: &'a [u8],
    span: &'a [u8],
    paired: &'a [u8],
}
fn parts(carrier: &[u8]) -> Result<Parts<'_>> {
    let len = u64::from_le_bytes(carrier.get(8..16).ok_or("missing D11 prefix")?.try_into()?);
    let end = 16usize
        .checked_add(len.try_into()?)
        .ok_or("oversized header")?;
    #[derive(Deserialize)]
    struct Envelope {
        span_bytes: usize,
        bank_sha256: String,
        span_sha256: String,
        parent_sha256: String,
    }
    let envelope: Envelope = serde_json::from_slice(carrier.get(16..end).ok_or("truncated D11")?)?;
    let padded = end
        .checked_next_multiple_of(64)
        .ok_or("invalid padded length")?;
    let data = carrier.get(padded..).ok_or("truncated payload")?;
    let (bank, rest) = data.split_at_checked(BANK_BYTES).ok_or("short bank")?;
    let (span, paired) = rest
        .split_at_checked(envelope.span_bytes)
        .ok_or("short span state")?;
    if [sha(bank), sha(span), sha(paired)]
        != [
            envelope.bank_sha256,
            envelope.span_sha256,
            envelope.parent_sha256,
        ]
    {
        return Err("payload checksum mismatch".into());
    }
    Ok(Parts { bank, span, paired })
}
fn profile(bank: &[u8], mode: OnsetMode) -> Result<OnsetProfile> {
    if bank.len() != BANK_BYTES {
        return Err("invalid sealed bank size".into());
    }
    Ok(OnsetProfile {
        mode,
        train_sha256: hex::encode(&bank[112..144]),
        labels_sha256: hex::encode(&bank[144..176]),
    })
}
fn open_bank(bank: &[u8], mode: OnsetMode) -> Result<Bank> {
    let p = profile(bank, mode)?;
    Ok(Bank::from_profile(
        bank,
        bank[16..48].try_into()?,
        bank[48..80].try_into()?,
        bank[80..112].try_into()?,
        &p,
    )?)
}
fn pack(
    parent: &Path,
    tokenizer: &Path,
    bank_file: &Path,
    mode: &str,
    output: &Path,
) -> Result<()> {
    let bytes = fs::read(parent)?;
    if sha(&bytes) != PARENT {
        return Err("frozen baseline differs".into());
    }
    let p = parts(&bytes)?;
    let mode = match mode {
        "legacy" => OnsetMode::Ordered,
        "whole" => OnsetMode::OrderedWholeWord,
        _ => return Err("unknown enumeration policy".into()),
    };
    let fitted = fs::read(bank_file)?;
    let profile = profile(&fitted, mode)?;
    let encoded = pack_requested_occurrence_artifact(
        p.paired,
        p.span,
        &fitted,
        &profile,
        &fs::read(tokenizer)?,
        RequestedContext::FullPrefix,
    )?;
    if mode == OnsetMode::Ordered && fitted == p.bank && encoded != bytes {
        return Err("baseline reconstruction failed".into());
    }
    if IntegerDiscreteModel::from_bytes(&encoded)?.onset_mode() != Some(mode) {
        return Err("reloaded policy differs".into());
    }
    save(output, &encoded)
}
fn augmented(old: &[u8], fitted: &[u8], raw: &[u8], output: &Path) -> Result<()> {
    open_bank(old, OnsetMode::Ordered)?;
    open_bank(fitted, OnsetMode::Ordered)?;
    if old[16..112] != fitted[16..112] || raw.len() != BANK_BYTES - BINDINGS {
        return Err("incompatible augmentation".into());
    }
    // The record prefix and the old per-shape radius are immutable. Rebuild the
    // exact registered append from the two fitted inputs as an independent check.
    let mut expected = old[BINDINGS..].to_vec();
    let used = u32::from_le_bytes(expected[8..12].try_into()?) as usize;
    let fitted_used = u32::from_le_bytes(fitted[216..220].try_into()?) as usize;
    let mut n = used;
    for add in fitted[BINDINGS + 16..].chunks_exact(32).take(fitted_used) {
        if n == 256 {
            break;
        }
        if add[2..4] != [1, 0] {
            continue;
        }
        let same_class = |r: &[u8]| r[..2] == add[..2] && r[4..8] == add[4..8];
        let original = &expected[16..16 + 32 * used];
        let radius = original
            .chunks_exact(32)
            .find(|r| same_class(r) && r[2..4] == [0, 0])
            .map(|r| <[u8; 4]>::try_from(&r[8..12]).unwrap());
        let Some(radius) = radius else { continue };
        if u32::from_le_bytes(radius) == 65535
            || original
                .chunks_exact(32)
                .any(|r| same_class(r) && r[2..4] == [1, 0])
            || expected[16..16 + 32 * n]
                .chunks_exact(32)
                .any(|r| same_class(r) && r[12..] == add[12..])
        {
            continue;
        }
        let dest = &mut expected[16 + 32 * n..16 + 32 * (n + 1)];
        dest.copy_from_slice(add);
        dest[8..12].copy_from_slice(&radius);
        n += 1;
    }
    expected[8..12].copy_from_slice(&(n as u32).to_le_bytes());
    if raw != expected {
        return Err("GPU and independent frozen augmentation differ".into());
    }
    let mut bank = old[..BINDINGS].to_vec();
    for (offset, tag) in [(112, b"training".as_slice()), (144, b"labels".as_slice())] {
        let mut h = Sha256::new();
        h.update(b"uor-r4/d28/missing-negatives/1");
        h.update(tag);
        h.update(&old[offset..offset + 32]);
        h.update(&fitted[offset..offset + 32]);
        h.update(Sha256::digest(old));
        h.update(Sha256::digest(fitted));
        bank[offset..offset + 32].copy_from_slice(&h.finalize());
    }
    bank[176..208].copy_from_slice(&Sha256::digest(raw));
    bank.extend_from_slice(raw);
    open_bank(&bank, OnsetMode::Ordered)?;
    save(output, &bank)?;
    println!(
        "retained {used} original records; appended {} TRAIN negatives",
        n - used
    );
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [cmd,p,out] if cmd=="extract" => {
            let bytes=fs::read(p)?;
            if sha(&bytes)!=PARENT {return Err("wrong original parent".into());}
            save(Path::new(out),parts(&bytes)?.bank)
        }
        [cmd,p,t,b,m,out] if cmd=="pack" => pack(p.as_ref(),t.as_ref(),b.as_ref(),m,out.as_ref()),
        [cmd,old,new,raw,out] if cmd=="augment" => augmented(&fs::read(old)?,&fs::read(new)?,&fs::read(raw)?,out.as_ref()),
        [cmd,model,panel,out] if cmd=="stage" => replay::grade(model.as_ref(),panel.as_ref(),None,out.as_ref()),
        [cmd,model,panel,clients,out] if cmd=="grade" => replay::grade(model.as_ref(),panel.as_ref(),Some(clients.as_ref()),out.as_ref()),
        _=>Err("usage: extract BASELINE BANK | pack BASELINE TOKENIZER BANK legacy|whole OUT | augment FROZEN FITTED RAW OUT | grade MODEL PANEL CLIENTS NEW_DIRECTORY".into()),
    }
}
