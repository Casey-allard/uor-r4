use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::{
    all_starts::{
        Bank, OnsetMode, OnsetProfile, BANK_BYTES, ORIGINAL_CANDIDATE_WORDS,
        ORIGINAL_DECISION_WORDS,
    },
    discrete_d11::pack_rival_admission,
};

mod finite;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const INPUT: u32 = 0x49383244;
const OUTPUT: u32 = 0x4f383244;

fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::File::create_new(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn from_words(words: impl IntoIterator<Item = u32>) -> Vec<u8> {
    words.into_iter().flat_map(u32::to_le_bytes).collect()
}

fn bank(bytes: &[u8]) -> Result<Bank> {
    if bytes.len() != BANK_BYTES {
        return Err("invalid sealed ordered bank size".into());
    }
    let profile = OnsetProfile {
        mode: OnsetMode::OrderedWholeWord,
        train_sha256: hex::encode(&bytes[112..144]),
        labels_sha256: hex::encode(&bytes[144..176]),
    };
    Ok(Bank::from_profile(
        bytes,
        bytes[16..48].try_into()?,
        bytes[48..80].try_into()?,
        bytes[80..112].try_into()?,
        &profile,
    )?)
}

fn score(input: &[u8], bank: &Bank) -> Result<Vec<u8>> {
    if input.len() % 4 != 0 {
        return Err("partial original onset word".into());
    }
    let words: Vec<_> = input
        .chunks_exact(4)
        .map(|part| u32::from_le_bytes(part.try_into().unwrap()))
        .collect();
    let [magic, count, width, reserved] = *words.get(..4).ok_or("missing input header")? else {
        unreachable!()
    };
    if magic != INPUT
        || !(1..=8192).contains(&count)
        || reserved != 0
        || width != ORIGINAL_CANDIDATE_WORDS as u32
        || words.len() != 4 + count as usize * ORIGINAL_CANDIDATE_WORDS
    {
        return Err("invalid original onset input envelope".into());
    }
    let mut output = from_words([OUTPUT, count, ORIGINAL_DECISION_WORDS as u32, 0]);
    for row in words[4..].chunks_exact(ORIGINAL_CANDIDATE_WORDS) {
        output.extend(from_words(bank.score_original_row(row.try_into()?)?));
    }
    Ok(output)
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, parent, training, calibration, out] if mode == "pack" => {
            let parent = fs::read(parent)?;
            let training = fs::read(training)?;
            let calibration = fs::read(calibration)?;
            save(Path::new(out), &pack_rival_admission(&parent, Sha256::digest(training).into(), &calibration)?)
        }
        [mode, input, fitted, calibration, out] if mode == "score" => {
            let bank = bank(&fs::read(fitted)?)?.with_negative_calibration(&fs::read(calibration)?)?;
            save(Path::new(out), &score(&fs::read(input)?, &bank)?)
        }
        [mode, fitted, calibration, out] if mode == "shuffle" => {
            let bank = bank(&fs::read(fitted)?)?.with_negative_calibration(&fs::read(calibration)?)?;
            let (control, changed) = bank.shuffle_negative_calibration()?;
            save(Path::new(out), &control)?;
            println!("changed_signatures={changed} informative={}", changed != 0);
            Ok(())
        }
        [mode, out] if mode == "finite" => finite::generate(Path::new(out)),
        _ => Err("expected pack PARENT TRAIN CALIBRATION OUT, score INPUT FITTED.bank CALIBRATION OUT, shuffle FITTED.bank CALIBRATION OUT, or finite NEW_DIR".into()),
    }
}
