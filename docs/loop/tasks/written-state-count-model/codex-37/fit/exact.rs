use super::*;
use crate::pick_device::windows;
use uor_r4_discrete::{kn_pick::exact_pair_pick, ngram_count::read_uort_tokens};

const CONTEXT: usize = 192;
const MAGIC: &[u8; 8] = b"R37C2010";

pub fn read(path: &Path, rows: usize, expected: &str) -> Result<Vec<u32>, Failure> {
    let size = 16 + rows * 4;
    if fs::metadata(path)?.len() != size as u64 {
        return Err(fail("wrong C2 tape length"));
    }
    let bytes = fs::read(path)?;
    if bytes.len() != size
        || &bytes[..8] != MAGIC
        || word(&bytes[8..16])? != rows as u64
        || digest(&bytes) != expected
    {
        return Err(fail("wrong C2 tape header, count, or SHA-256"));
    }
    bytes[16..]
        .chunks_exact(4)
        .map(|v| Ok(u32::from_le_bytes(v.try_into()?)))
        .collect()
}

// The fixture forces empty history, repeated pairs with a later matching
// predecessor, a sliding window that evicts an older match, EOS and BOS reset.
fn fixture() -> Vec<u16> {
    let mut ids = vec![0, 1, 0, 2, 3, 5, 2, 3, 9, 2, 3, 7, 1, 0, 2, 3];
    ids.extend(20..230);
    ids.extend([2, 3, 10, 2, 3, 7, 1, 0, 2, 3, 11, 1]);
    ids
}

pub fn finite(path: &Path, verify_tape: Option<&Path>) -> Result<(), Failure> {
    let ids = fixture();
    let mut header = [0; 64];
    header[..4].copy_from_slice(b"UORT");
    header[4..8].copy_from_slice(&1u32.to_le_bytes());
    header[8..16].copy_from_slice(&(ids.len() as u64).to_le_bytes());
    header[16..20].copy_from_slice(&4096u32.to_le_bytes());
    let mut encoded = header.to_vec();
    encoded.extend(ids.iter().flat_map(|id| id.to_le_bytes()));
    match verify_tape {
        None => write_new(path, &encoded),
        Some(tape) => {
            let disk = fs::read(path)?;
            if disk != encoded || read_uort_tokens(&disk)? != ids {
                return Err(fail("C2 finite fixture identity differs"));
            }
            let windows = windows(&ids, CONTEXT)?;
            let picks = read(tape, windows.len(), &file_digest(tape)?)?;
            for (ordinal, ((_, window), picked)) in windows.iter().zip(&picks).enumerate() {
                let want = exact_pair_pick(window)
                    .map(|p| p as u32)
                    .unwrap_or(u32::MAX);
                if *picked != want {
                    return Err(fail(format!("C2 finite mismatch at ordinal {ordinal}")));
                }
            }
            if picks[9] != 5 || picks[225] != u32::MAX {
                return Err(fail(
                    "C2 finite tie or eviction control did not exercise its branch",
                ));
            }
            println!(
                "C2 finite original-offset equality for {} targets",
                picks.len()
            );
            Ok(())
        }
    }
}
