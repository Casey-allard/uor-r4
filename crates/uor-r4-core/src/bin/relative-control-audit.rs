//! Stronger same-information comparator for the finite Q8 selection experiment.
use serde_json::json;
use std::{fs, path::PathBuf};
use uor_r4_core::native_geometric::learner::{
    hamilton_transport as ht, relative_action_learning as ra,
};
use uor_r4_core::report_output::{claim, seal, verify};
fn next(r: &mut u64) -> u64 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    *r
}
fn distance(a: [i32; 4], b: [i32; 4]) -> u64 {
    a.iter()
        .zip(b)
        .map(|(&a, b)| (i64::from(a) - i64::from(b)).unsigned_abs())
        .sum()
}
#[derive(Clone, Copy)]
struct Perm {
    index: [usize; 4],
    sign: u8,
}
fn apply(p: Perm, x: [i32; 4]) -> [i32; 4] {
    std::array::from_fn(|i| {
        if p.sign & (1 << i) != 0 {
            -x[p.index[i]]
        } else {
            x[p.index[i]]
        }
    })
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err("usage: relative-control-audit NEW_REPORT_ROOT".into());
    }
    let root = PathBuf::from(&args[1]);
    claim(&root).map_err(|e| e.to_string())?;
    let truth = [6u8, 3, 0, 5, 2, 7, 1, 4];
    let mut rng = 0x20260924u64;
    let mut training = Vec::new();
    for relation in 0..8 {
        for _ in 0..8 {
            let keys: Vec<[i32; 4]> = (0..8)
                .map(|_| std::array::from_fn(|_| (next(&mut rng) % 63) as i32 - 31))
                .collect();
            let target = (next(&mut rng) % 8) as usize;
            let query = ht::apply(truth[relation], keys[target])?;
            training.push(ra::ActionExample {
                relation,
                query,
                keys,
                target,
            });
        }
    }
    let model = ra::RelativeActionModel::fit(8, &training)?;
    let mut candidates = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    if [a, b, c, d]
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == 4
                    {
                        for sign in 0..16 {
                            candidates.push(Perm {
                                index: [a, b, c, d],
                                sign,
                            });
                        }
                    }
                }
            }
        }
    }
    let mut weak = Vec::new();
    let mut strong = Vec::new();
    let mut ties = Vec::new();
    for relation in 0..8 {
        let mut scores = Vec::new();
        for (i, &p) in candidates.iter().enumerate() {
            let (mut hinge, mut exact) = (0u64, 0u64);
            for ex in training.iter().filter(|e| e.relation == relation) {
                let positive = distance(ex.query, apply(p, ex.keys[ex.target]));
                let negative = ex
                    .keys
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != ex.target)
                    .map(|(_, k)| distance(ex.query, apply(p, *k)))
                    .min()
                    .ok_or("missing competitor")?;
                hinge += (positive + 1).saturating_sub(negative);
                exact += positive;
            }
            scores.push((hinge, exact, i));
        }
        let w = scores
            .iter()
            .min_by_key(|x| (x.0, x.2))
            .ok_or("empty candidates")?;
        let s = scores.iter().min().ok_or("empty candidates")?;
        weak.push(candidates[w.2]);
        strong.push(candidates[s.2]);
        ties.push(json!({"relation":relation,"min_hinge":w.0,"hinge_minimizers":scores.iter().filter(|x|x.0==w.0).count(),"weak_target_distance":w.1,"strong_target_distance":s.1}));
    }
    let mut correct = [0usize; 3];
    let mut rows = Vec::new();
    for test in 0..1024 {
        let keys: Vec<[i32; 4]> = (0..8)
            .map(|_| std::array::from_fn(|_| (next(&mut rng) % 2047) as i32 - 1023))
            .collect();
        let target = (next(&mut rng) % 8) as usize;
        let path: Vec<usize> = (0..1 + test % 6)
            .map(|_| (next(&mut rng) % 8) as usize)
            .collect();
        let mut query = keys[target];
        for &r in &path {
            query = ht::apply(truth[r], query)?;
        }
        let pick = |parameters: &[Perm]| {
            (0..8)
                .min_by_key(|&i| {
                    let mut v = keys[i];
                    for &r in &path {
                        v = apply(parameters[r], v);
                    }
                    distance(query, v)
                })
                .unwrap_or(0)
        };
        let result = [
            model.select(&path, query, &keys)?,
            pick(&weak),
            pick(&strong),
        ];
        for i in 0..3 {
            correct[i] += usize::from(result[i] == target);
        }
        rows.push(json!({"target":target,"predictions":result,"length":path.len()}));
    }
    if correct[..2] != [1024, 864] {
        return Err(format!("original fixture replication failed: {correct:?}"));
    }
    let report = json!({"schema":"uor-r4.relative-control-audit/1","seed":0x20260924u64,"training_examples":64,"test_examples":1024,"arms":["learned_Q8","ordinary_hinge_only","ordinary_hinge_then_observed_target_distance"],"correct":correct,"fit_ties":ties,"additional_supervision":false,"control_change":"Break ranking-loss ties using distance between the supplied query vector and the transformed supplied positive key. No additional labels, hidden operator IDs, test examples or fitted parameters.","interpretation":"A weak ordinary optimization objective can underidentify the generating operation. Compare against the strengthened same-information learner before claiming a geometric predictive advantage.","scope":"Exploratory comparator repair after the initial geometric result; same predeclared synthetic population, not natural-language attention."});
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        root.join("rows.json"),
        serde_json::to_vec(&rows).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    seal(&root).map_err(|e| e.to_string())?;
    let errors = verify(&root).map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("report verification {errors:?}"));
    }
    println!("{}", report);
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
