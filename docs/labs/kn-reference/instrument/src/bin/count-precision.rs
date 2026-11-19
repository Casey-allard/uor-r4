//! Restore precision at a fixed D11 byte budget by joining two validated TRAIN states.
use serde_json::json;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use uor_r4_integer::count_mixer::{pack, CountMixer, VOCAB};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const FIRST_EDGE: usize = (1 + VOCAB) * 4;

fn key(row: &[u8]) -> (u16, u16) {
    (
        u16::from_le_bytes([row[0], row[1]]),
        u16::from_le_bytes([row[2], row[3]]),
    )
}

fn count(row: &[u8]) -> u32 {
    u32::from_le_bytes(row[4..8].try_into().unwrap())
}

fn restore(full: &CountMixer, pruned: &CountMixer) -> Result<(Vec<u8>, usize)> {
    if full.words()[4..FIRST_EDGE] != pruned.words()[4..FIRST_EDGE] {
        return Err("unigram counts differ".into());
    }
    let mut restored = pruned.words().to_vec();
    let mut full_rows = full.words()[FIRST_EDGE..].chunks_exact(8).peekable();
    let mut changed = 0;
    for row in restored[FIRST_EDGE..].chunks_exact_mut(8) {
        let address = key(row);
        while full_rows.next_if(|r| key(r) < address).is_some() {}
        let exact = full_rows
            .next_if(|r| key(r) == address)
            .ok_or("missing retained key")?;
        let value = count(exact);
        if count(row) != 1u32 << value.ilog2() {
            return Err("retained count does not match declared quantization".into());
        }
        if count(row) != value {
            row[4..8].copy_from_slice(&value.to_le_bytes());
            changed += 1;
        }
    }
    Ok((restored, changed))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [base_path, full_path, pruned_path, out_path] = args.as_slice() else {
        return Err("usage: count-precision BASE.d11 FULL.d11 PRUNED.d11 NEW.d11".into());
    };
    let base = fs::read(base_path)?;
    let full = IntegerDiscreteModel::load(full_path.as_ref())?;
    let pruned = IntegerDiscreteModel::load(pruned_path.as_ref())?;
    let full_mix = full.count_mixer().ok_or("missing full mixer")?;
    let pruned_mix = pruned.count_mixer().ok_or("missing pruned mixer")?;
    for (model, words) in [(&full, full_mix.words()), (&pruned, pruned_mix.words())] {
        if format!("{:x}", Sha256::digest(pack(&base, words)?)) != model.artifact_sha256() {
            return Err("source carrier or canonical encoding mismatch".into());
        }
    }
    let (words, changed) = restore(full_mix, pruned_mix)?;
    let packed = pack(&base, &words)?;
    let bytes = packed.len() as u64;
    if changed == 0 || bytes != fs::metadata(pruned_path)?.len() {
        return Err("uninformative comparison or changed deployment budget".into());
    }
    let loaded = IntegerDiscreteModel::from_bytes(&packed)?;
    if loaded
        .count_mixer()
        .ok_or("missing restored state")?
        .words()
        != words
    {
        return Err("count words changed on reload".into());
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out_path)?;
    output.write_all(&packed)?;
    output.sync_all()?;
    println!(
        "{}",
        json!({
            "full_source":full.artifact_sha256(), "pruned_source":pruned.artifact_sha256(),
            "restored":loaded.artifact_sha256(), "pruned_and_restored_bytes_each":bytes,
            "weight_of_256":pruned_mix.conditional_weight(), "changed_counts": changed,
            "retained_bigrams":(words.len()-FIRST_EDGE)/8,
        })
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(conditional: &[(u16, u16, u32)]) -> CountMixer {
        let mut words = vec![0u8; FIRST_EDGE];
        words[..4].copy_from_slice(&128u32.to_le_bytes());
        words[8..12].copy_from_slice(&21u32.to_le_bytes());
        for (p, t, c) in conditional {
            words.extend(p.to_le_bytes());
            words.extend(t.to_le_bytes());
            words.extend(c.to_le_bytes());
        }
        CountMixer::from_words(&words).unwrap()
    }
    #[test]
    fn restore_only_matching_retained_edges() {
        let full = fixture(&[(1, 3, 1), (5, 2, 7), (5, 4, 8)]);
        let pruned = fixture(&[(5, 2, 4), (5, 4, 8)]);
        let (result, changed) = restore(&full, &pruned).unwrap();
        assert_eq!(changed, 1);
        assert_eq!(result, fixture(&[(5, 2, 7), (5, 4, 8)]).words());
        assert!(restore(&full, &fixture(&[(5, 3, 4)])).is_err());
        assert!(restore(&full, &fixture(&[(5, 2, 2)])).is_err());
    }
}
