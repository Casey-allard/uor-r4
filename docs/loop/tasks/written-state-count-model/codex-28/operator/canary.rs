use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::PathBuf};
use uor_r4_integer::{
    all_starts::{
        original_packet, Bank, OnsetMode, OnsetProfile, BANK_BYTES, ORIGINAL_CANDIDATE_WORDS,
        ORIGINAL_DECISION_WORDS,
    },
    assertion_value::AssertionValue,
};

type Row = [u32; ORIGINAL_CANDIDATE_WORDS];
type Failure = Box<dyn Error>;
const NIL: u32 = u32::MAX;

fn bytes(words: impl IntoIterator<Item = u32>) -> Vec<u8> {
    words.into_iter().flat_map(u32::to_le_bytes).collect()
}

fn fixture_bank(single: &Row, early: &Row) -> Result<(Bank, Vec<u8>), Failure> {
    let mut records = vec![0x31425355, 0, 4, 256];
    for candidate in [&single[6..16], &early[6..16]] {
        let shape = candidate[2].min(3) | ((candidate[1] - candidate[0]) << 8);
        for class in 0..=1 {
            records.extend([shape | class << 16, candidate[4], 1280]);
            records.extend(&candidate[5..10]);
            if class == 1 {
                let first_root = records.len() - 5;
                records[first_root] ^= 1;
            }
        }
    }
    records.resize(4 + 256 * 8, 0);
    let raw = bytes(records);
    let mut sealed = bytes([0x32425355, 6, raw.len() as u32, 0]);
    for hash in [
        [0; 32],
        [1; 32],
        [2; 32],
        [3; 32],
        [4; 32],
        Sha256::digest(&raw).into(),
    ] {
        sealed.extend(hash);
    }
    sealed.extend(&raw);
    assert_eq!(sealed.len(), BANK_BYTES);
    let profile = OnsetProfile {
        mode: OnsetMode::OrderedWholeWord,
        train_sha256: "03".repeat(32),
        labels_sha256: "04".repeat(32),
    };
    Ok((
        Bank::from_profile(&sealed, [0; 32], [1; 32], [2; 32], &profile)?,
        raw,
    ))
}

fn main() -> Result<(), Failure> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let [output] = args.as_slice() else {
        return Err("usage: d28-onset-canary NEW_DIRECTORY".into());
    };
    let output = PathBuf::from(output);
    let mut roots: Vec<u32> = (0..4096).collect();
    let mut classes = vec![1; 4096];
    for id in [0, 16, 201] {
        classes[id] = 0;
    }
    for id in [41, 43, 45, 47, 49, 51] {
        classes[id] = 2;
        roots[id] = 8;
    }
    for id in [42, 44, 46, 48] {
        roots[id] = 10;
    }
    let packet = |ids: &[u32], fence: Option<u32>, old_mask| {
        let mut frozen = AssertionValue {
            fence,
            ..Default::default()
        };
        frozen.value.eligible_mask = old_mask;
        frozen.value.word_start_mask = old_mask;
        original_packet(ids, &roots, &classes, 1, ids.len() - 1, frozen)
    };
    let single_ids = [0, 12, 40, 41, 42, 43, 44, 45, 16, 12];
    let single = packet(&single_ids, Some(8), 0)?;
    let early_only = packet(&[0, 12, 40, 41, 42, 43, 16, 12], Some(6), 2)?;
    let (bank, raw) = fixture_bank(&single, &early_only)?;
    let rows = [
        ("long connector", single, [0, 6], &[3][..]),
        ("early onset", early_only, [0, 4], &[3]),
        (
            "competing early and later onset",
            packet(&single_ids, Some(8), 2)?,
            [0, 4],
            &[3, 3],
        ),
        (
            "tied later onsets",
            packet(
                &[0, 12, 40, 41, 42, 43, 44, 45, 46, 47, 16, 12],
                Some(10),
                0,
            )?,
            [3, NIL],
            &[3, 3],
        ),
        (
            "unknown rival vetoes known",
            packet(&[0, 12, 40, 41, 42, 43, 44, 45, 46, 16, 12], Some(9), 0)?,
            [2, NIL],
            &[3, 2],
        ),
        (
            "unknown rival precedes known",
            packet(
                &[0, 12, 40, 41, 42, 43, 46, 44, 45, 48, 49, 44, 45, 16, 12],
                Some(13),
                0,
            )?,
            [2, NIL],
            &[2, 2, 2, 3],
        ),
        (
            "nine-piece rival inaccessible",
            packet(
                &[0, 12, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 201, 12],
                Some(13),
                2,
            )?,
            [2, NIL],
            &[1, 3, 3, 3, 2],
        ),
        (
            "no assertion stop",
            packet(&[0, 12, 40, 41, 42, 43, 44, 45, 12], None, 0)?,
            [1, NIL],
            &[],
        ),
    ];

    let mut input = bytes([
        0x49383244,
        rows.len() as u32,
        ORIGINAL_CANDIDATE_WORDS as u32,
        0,
    ]);
    let mut expected = bytes([
        0x4f383244,
        rows.len() as u32,
        ORIGINAL_DECISION_WORDS as u32,
        0,
    ]);
    for (name, row, outcome, expected_stages) in rows {
        let score = bank.score_original_row(&row)?;
        assert_eq!(&score[..2], &outcome, "{name}");
        let stages: Vec<_> = score[2..]
            .chunks_exact(10)
            .take(row[5] as usize)
            .map(|candidate| candidate[4])
            .collect();
        assert_eq!(stages, expected_stages, "{name}");
        eprintln!(
            "{name}: status={} source={} candidates={}",
            score[0], score[1], row[5]
        );
        input.extend(bytes(row));
        expected.extend(bytes(score));
    }
    fs::create_dir(&output)?;
    fs::write(output.join("canary.d28i"), input)?;
    fs::write(output.join("ordered.raw"), raw)?;
    fs::write(output.join("canary.oracle.d28o"), expected)?;
    Ok(())
}
