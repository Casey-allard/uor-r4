use serde_json::{json, Value};
use std::{env, fs};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn draw(seed: &mut u64) -> usize {
    *seed = seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut x = *seed;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((x ^ (x >> 31)) % 2670) as usize
}
fn main() -> Result<()> {
    let paths: Vec<_> = env::args().skip(1).collect();
    let [s, p] = paths.as_slice() else {
        return Err("selection.json per-record-q12.json".into());
    };
    let summary: Value = serde_json::from_slice(&fs::read(s)?)?;
    let raw: Vec<Vec<[i64; 2]>> = serde_json::from_slice(&fs::read(p)?)?;
    let names = ["a0", "a1", "a1s", "c1", "c2", "c3", "c3s"];
    if raw.len() != 7 || raw.iter().any(|arm| arm.len() != 2670) {
        return Err("incomplete record coverage".into());
    }
    let mut bands = Vec::new();
    for (i, &name) in names.iter().enumerate() {
        for (j, axis) in ["all", "novel"].iter().enumerate() {
            let total: i64 = raw[i].iter().map(|r| r[j]).sum();
            if summary["arms"][name][axis]["bits_q12"] != total.to_string() {
                return Err(format!("wrong total for {name}/{axis}").into());
            }
        }
    }
    for (pair, axis) in summary["bands"].as_object().ok_or("missing bands")? {
        let (a, b) = pair.split_once("_minus_").ok_or("pair name")?;
        let i = names
            .iter()
            .position(|&x| x == a)
            .ok_or("unrecognized arm")?;
        let k = names
            .iter()
            .position(|&x| x == b)
            .ok_or("unrecognized arm")?;
        let diffs: Vec<_> = raw[i]
            .iter()
            .zip(&raw[k])
            .map(|(r, s)| [r[0] - s[0], r[1] - s[1]])
            .collect();
        let mut all = Vec::new();
        let mut novel = Vec::new();
        let mut seed = 4390;
        for _ in 0..1000 {
            let mut totals = [0, 0];
            for _ in &diffs {
                let item = &diffs[draw(&mut seed)];
                totals[0] += item[0];
                totals[1] += item[1];
            }
            all.push(totals[0]);
            novel.push(totals[1]);
        }
        for (mut draws, cohort) in [(all, "all"), (novel, "novel")] {
            draws.sort_unstable();
            if axis[cohort]["q12"] != json!([draws[24].to_string(), draws[974].to_string()]) {
                return Err(format!("paired band mismatch for {pair} {cohort}").into());
            }
            bands.push(json!([pair, cohort, draws[24], draws[974]]));
        }
    }
    println!(
        "{}",
        json!({"records_each":2670,"arms":7,"paired_bands_verified":bands,"chosen":summary["selection"]})
    );
    Ok(())
}
