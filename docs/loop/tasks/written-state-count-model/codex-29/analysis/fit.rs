use super::{require, sha, words, Result};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const NO_RADIUS: u32 = 65535;
pub type Signature = (u32, u32);

#[derive(Serialize)]
pub struct Stratum {
    pub shape: u32,
    pub signature: u32,
    pub positives: usize,
    pub negatives: usize,
    pub negative_families: BTreeSet<u32>,
    pub calibrated: u32,
    pub shuffled: u32,
}

pub struct Fit {
    pub sha256: [String; 3],
    pub strata: BTreeMap<Signature, Stratum>,
}

fn calibration(data: &[u8], n: usize) -> Result<Vec<u32>> {
    let table = words(data)?;
    require(
        table.len() == 260
            && table[..4] == [0x39324e52, n as u32, 256, 0]
            && table[4 + n..].iter().all(|&x| x == 0)
            && table[4..4 + n].iter().all(|&x| x == NO_RADIUS || x <= 1280),
        "invalid calibration header, radius or padding",
    )?;
    Ok(table[4..4 + n].to_vec())
}

pub fn inspect(train: &Path, calibrated: &Path, shuffled: &Path) -> Result<Fit> {
    let input = fs::read(train)?;
    let true_raw = fs::read(calibrated)?;
    let shuffle_raw = fs::read(shuffled)?;
    let packet = words(&input)?;
    require(
        sha(&input) == "dfcd22f0f4f75c6a0298c6b2f1363562d189b58508911b39d18877f41506bd9c"
            && packet.len() == 4 + 226 * 13
            && packet[..4] == [0x38324644, 226, 13, 0],
        "wrong frozen signed TRAIN packet",
    )?;
    let fitted = calibration(&true_raw, 226)?;
    let rotated = calibration(&shuffle_raw, 226)?;
    let mut strata: BTreeMap<Signature, Stratum> = BTreeMap::new();
    for (i, row) in packet[4..].chunks_exact(13).enumerate() {
        let (shape, signature) = (row[5], row[6]);
        require(
            (28000..28068).contains(&row[0]) && row[4] == (row[0] - 28000) / 4 && row[7] <= 1,
            "invalid signed TRAIN family",
        )?;
        let item = strata.entry((shape, signature)).or_insert(Stratum {
            shape,
            signature,
            positives: 0,
            negatives: 0,
            negative_families: BTreeSet::new(),
            calibrated: fitted[i],
            shuffled: rotated[i],
        });
        require(
            item.calibrated == fitted[i] && item.shuffled == rotated[i],
            "mixed radii inside one exact TRAIN signature",
        )?;
        if row[7] == 1 {
            item.negatives += 1;
            item.negative_families.insert(row[4]);
        } else {
            item.positives += 1;
        }
    }
    require(
        strata.values().map(|s| s.positives).sum::<usize>() == 68
            && strata.values().map(|s| s.negatives).sum::<usize>() == 158,
        "class totals differ",
    )?;
    for item in strata.values() {
        require(
            (item.calibrated != NO_RADIUS) == (item.negative_families.len() >= 2)
                && (item.calibrated != NO_RADIUS) == (item.shuffled != NO_RADIUS),
            "cross-family support or shuffle eligibility differs",
        )?;
    }
    let available: Vec<_> = strata
        .values()
        .filter(|s| s.calibrated != NO_RADIUS)
        .collect();
    require(
        available
            .iter()
            .zip(available.iter().cycle().skip(1))
            .all(|(s, next)| s.shuffled == next.calibrated),
        "shuffle is not the registered sorted-key rotation",
    )?;
    Ok(Fit {
        sha256: [sha(&input), sha(&true_raw), sha(&shuffle_raw)],
        strata,
    })
}
