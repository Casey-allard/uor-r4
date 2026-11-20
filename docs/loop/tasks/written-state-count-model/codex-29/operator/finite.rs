use super::{bank, from_words, save, score, Result, INPUT};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use uor_r4_integer::all_starts::{ORIGINAL_CANDIDATE_WORDS, ORIGINAL_DECISION_WORDS};

const NIL: u32 = u32::MAX;
const ABSENT: u32 = 65535;
const CAP: usize = 256;

#[derive(Clone, Copy)]
struct Exemplar {
    row: u32,
    label: u32,
    shape: u32,
    signature: u32,
    root: u32,
    positive: u32,
    negative: u32,
}

fn fit_packet(exemplars: &[Exemplar]) -> Vec<u8> {
    let mut words = vec![0x38324644, exemplars.len() as u32, 13, 0];
    for e in exemplars {
        words.extend([
            e.row,
            3,
            4,
            7,
            (e.row - 28000) / 4,
            e.shape,
            e.signature,
            e.label,
            e.root,
            0,
            0,
            0,
            0,
        ]);
    }
    from_words(words)
}

fn packet(candidates: &[(u32, u32, u32)]) -> [u32; ORIGINAL_CANDIDATE_WORDS] {
    let mut words = [0; ORIGINAL_CANDIDATE_WORDS];
    let fence = 7;
    let mask = candidates.iter().fold(0, |mask, &(source, _, _)| {
        if source <= 5 {
            mask | 1 << (source - 3)
        } else {
            mask
        }
    });
    words[..6].copy_from_slice(&[10, 1, 8, fence, mask, candidates.len() as u32]);
    for (slot, &(source, signature, root)) in candidates.iter().enumerate() {
        let index = 6 + 10 * slot;
        words[index..index + 6].copy_from_slice(&[
            source,
            source + 1,
            source - 2,
            fence - source,
            signature,
            root,
        ]);
    }
    words
}

pub(super) fn generate(out: &Path) -> Result<()> {
    let e = |row, label, shape, signature, root, positive, negative| Exemplar {
        row,
        label,
        shape,
        signature,
        root,
        positive,
        negative,
    };
    // Root 0=(2,2), 1=(-2,-2), 2=(-2,2), 15=(2,-2).
    // The first signature's two cross-family negatives are distance 4.
    // The depth-three signature's two negatives differ at two blocks, distance 16.
    // The depth-two signature has negatives only in one family, so its radius stays absent.
    let exemplars = [
        e(28000, 0, 257, 5, 0, 0, 4),
        e(28001, 1, 257, 5, 1, 0, 4),
        e(28004, 1, 257, 5, 2, 0, 4),
        e(28008, 0, 259, 85, 0, 0, 16),
        e(28009, 1, 259, 85, 1, 0, 16),
        e(28012, 1, 259, 85, 16, 0, 16),
        e(28016, 1, 258, 21, 0, ABSENT, ABSENT),
        e(28017, 1, 258, 21, 1, ABSENT, ABSENT),
        e(28020, 1, 259, 84, 1, ABSENT, 4),
        e(28024, 1, 259, 84, 2, ABSENT, 4),
    ];
    let fit = fit_packet(&exemplars);
    let mut raw = vec![0x31425355, 0, exemplars.len() as u32, CAP as u32];
    let mut table = vec![0x39324e52, exemplars.len() as u32, CAP as u32, 0];
    for e in &exemplars {
        raw.extend([
            e.shape | e.label << 16,
            e.signature,
            e.positive,
            e.root,
            0,
            0,
            0,
            0,
        ]);
        table.push(e.negative);
    }
    raw.resize(4 + CAP * 8, 0);
    table.resize(4 + CAP, 0);
    let raw = from_words(raw);
    let table = from_words(table);
    let mut sealed = from_words([0x32425355, 6, raw.len() as u32, 0]);
    for digest in [
        [0; 32],
        [1; 32],
        [2; 32],
        Sha256::digest(&fit).into(),
        [4; 32],
        Sha256::digest(&raw).into(),
    ] {
        sealed.extend(digest);
    }
    sealed.extend(&raw);

    let cases: &[(&str, &[(u32, u32, u32)], [u32; 2], &[u32])] = &[
        (
            "unresolved earlier rival clears",
            &[(3, 5, 1), (6, 85, 0)],
            [0, 6],
            &[4, 3],
        ),
        ("true early winner preserved", &[(3, 5, 0)], [0, 3], &[3]),
        (
            "tie between two admitted contenders",
            &[(3, 5, 0), (6, 85, 0)],
            [3, NIL],
            &[3, 3],
        ),
        (
            "strict class tie cannot clear",
            &[(3, 5, 15), (6, 85, 0)],
            [2, NIL],
            &[2, 3],
        ),
        (
            "out of negative radius stays unresolved",
            &[(3, 5, 17), (6, 85, 0)],
            [2, NIL],
            &[2, 3],
        ),
        (
            "both classes absent",
            &[(3, 4, 0), (6, 85, 0)],
            [2, NIL],
            &[2, 3],
        ),
        (
            "one TRAIN family does not clear",
            &[(4, 21, 0), (6, 85, 0)],
            [2, NIL],
            &[2, 3],
        ),
        (
            "known negative without positive is not a source",
            &[(6, 84, 1)],
            [1, NIL],
            &[4],
        ),
        (
            "clearing a late rival preserves the true early source",
            &[(3, 5, 0), (6, 85, 1)],
            [0, 3],
            &[3, 4],
        ),
    ];
    let mut input = from_words([
        INPUT,
        cases.len() as u32,
        ORIGINAL_CANDIDATE_WORDS as u32,
        0,
    ]);
    for &(_, candidates, _, _) in cases {
        input.extend(from_words(packet(candidates)));
    }
    let core = bank(&sealed)?;
    let calibrated = core.clone().with_negative_calibration(&table)?;
    let oracle = score(&input, &calibrated)?;
    for ((name, _, words, stages), trace) in cases
        .iter()
        .zip(oracle[16..].chunks_exact(4 * ORIGINAL_DECISION_WORDS))
    {
        let decoded: Vec<_> = trace
            .chunks_exact(4)
            .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
            .collect();
        assert_eq!(decoded[..2], *words, "{name}");
        for (index, stage) in stages.iter().enumerate() {
            assert_eq!(decoded[2 + 10 * index + 4], *stage, "{name}");
        }
    }
    let d28 = score(&input, &core)?;
    assert_eq!(u32::from_le_bytes(d28[16..20].try_into()?), 2);
    // The original true-early winner and genuine tie retain their complete
    // device words because neither row has unresolved candidates.
    let row_size = 4 * ORIGINAL_DECISION_WORDS;
    assert_eq!(
        d28[16 + row_size..16 + 3 * row_size],
        oracle[16 + row_size..16 + 3 * row_size]
    );
    let (shuffle, changed) = calibrated.shuffle_negative_calibration()?;
    assert_eq!(changed, 2);
    core.clone().with_negative_calibration(&shuffle)?;

    let mut bad_fit = fit.clone();
    bad_fit[16 + 8 * 4] ^= 1;
    let mut bad_table = table.clone();
    bad_table[16 + 4..16 + 8].copy_from_slice(&99u32.to_le_bytes());
    assert!(core.clone().with_negative_calibration(&bad_table).is_err());
    let mut bad_input = input.clone();
    bad_input[16 + (6 + 2) * 4] ^= 1;
    assert!(score(&bad_input, &calibrated).is_err());

    fs::create_dir(out)?;
    for (name, bytes) in [
        ("ordered.raw", raw),
        ("ordered.bank", sealed),
        ("train.packet", fit),
        ("expected-calibration.raw", table),
        ("score.d28i", input),
        ("expected-score.d28o", oracle),
        ("bad-train.packet", bad_fit),
        ("bad-calibration.raw", bad_table),
        ("bad-score.d28i", bad_input),
        ("shuffled-calibration.raw", shuffle),
    ] {
        save(&out.join(name), &bytes)?;
    }
    Ok(())
}
