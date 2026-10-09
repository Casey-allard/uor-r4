//! Saved-score relaxation diagnostic. No training, proposal or trajectory replay.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::{
        geometric_continuation_field::{ContinuationReadCounts, NativeContinuationField},
        geometric_generate::{GenerateReadCounts, NativeGeometricGenerate},
    },
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::{NativeVocabularyActions, SCORE_CLIP_Q24},
    h4_tables::H4Code,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REPORT: &str = "a4d9f86ed218494b6aedafb13a32fc7b7c923224686722e10545bb78fd4f3b1f";
const MANIFEST: &str = "e2028f55f68c0fb061fc604356f5524ea42cc8c3f5bd51f8d739e23b3b93463e";
const UNIT: i64 = 1 << 20;
fn require(ok: bool, why: &str) -> Result<()> {
    if !ok {
        return Err(io::Error::new(io::ErrorKind::InvalidData, why).into());
    }
    Ok(())
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_reader(io::BufReader::new(
        fs::File::open(p)?,
    ))?)
}
fn hash(p: &Path) -> Result<String> {
    let mut h = Sha256::new();
    let mut f = io::BufReader::new(fs::File::open(p)?);
    let mut buf = [0u8; 65536];
    loop {
        let n = io::Read::read(&mut f, &mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn vec_i64(v: &Value) -> Result<Vec<i64>> {
    v.as_array()
        .ok_or("integer array missing")?
        .iter()
        .map(|x| x.as_i64().ok_or_else(|| "integer missing".into()))
        .collect()
}
fn codes(v: &Value) -> Result<Vec<H4Code>> {
    v.as_array()
        .ok_or("H4 array missing")?
        .iter()
        .map(|x| {
            let n = x.as_u64().ok_or("H4 integer missing")?;
            require(n < 120, "invalid H4")?;
            Ok(H4Code::try_from(n as u8)?)
        })
        .collect()
}
fn clip(x: i64) -> i64 {
    x.clamp(-SCORE_CLIP_Q24, SCORE_CLIP_Q24)
}
fn grids(bases: &[i64], radius: i64) -> Result<Vec<Vec<i64>>> {
    bases
        .iter()
        .map(|&base| {
            (-radius..=radius)
                .map(|s| {
                    Ok(clip(
                        base.checked_add(s.checked_mul(UNIT).ok_or("grid overflow")?)
                            .ok_or("grid overflow")?,
                    ))
                })
                .collect()
        })
        .collect()
}
/// Every true clipped maximum is either Copymax or one legal Generate grid
/// score at least Copymax. Shared coefficient equality is deliberately relaxed.
fn references(grid: &[Vec<i64>], copies: &[i64]) -> Result<BTreeSet<i64>> {
    require(!grid.is_empty(), "legal Generate grid missing")?;
    let floor = copies
        .iter()
        .map(|&x| clip(x))
        .max()
        .unwrap_or(-SCORE_CLIP_Q24);
    let mut refs = BTreeSet::new();
    if !copies.is_empty() {
        refs.insert(floor);
    }
    for scores in grid {
        for &score in scores {
            if score >= floor {
                refs.insert(score);
            }
        }
    }
    require(!refs.is_empty(), "reference superset empty")?;
    Ok(refs)
}
fn bound_at<F>(
    reference: i64,
    target_max: i64,
    ids: &[u32],
    copies: &[i64],
    target: u32,
    rival: u32,
    mut weight: F,
) -> Result<(u64, u64)>
where
    F: FnMut(i64, i64) -> Result<u64>,
{
    require(ids.len() == copies.len(), "physical alias shape")?;
    let mut upper = weight(target_max.min(reference), reference)?;
    let mut lower = 0u64;
    for (&id, &raw) in ids.iter().zip(copies) {
        if id == target {
            upper = upper
                .checked_add(weight(raw, reference)?)
                .ok_or("target overflow")?;
        }
        if id == rival {
            lower = lower
                .checked_add(weight(raw, reference)?)
                .ok_or("rival overflow")?;
        }
    }
    Ok((upper, lower))
}
fn analyze(run: &Path) -> Result<Value> {
    let start = Instant::now();
    require(
        hash(&run.join("report.json"))? == REPORT && hash(&run.join("manifest.json"))? == MANIFEST,
        "pinned run identity",
    )?;
    report_output::verify(run)?;
    let report = read(&run.join("report.json"))?;
    require(
        report["status"] == "COMPLETED"
            && report["mode"] == "generate_pair_episode_learning"
            && report["candidate_native_steps"] == 391
            && report["selected_model"] == false
            && report["final_gate"]["passed"] == false,
        "completed391 unselected pair-negative scope",
    )?;
    let population = read(&run.join("generate-population.json"))?;
    require(
        population["rows"].as_array().map(Vec::len) == Some(391) && population["guards"] == 380,
        "population scope",
    )?;
    let row = population["objective_row_map"][4]
        .as_u64()
        .ok_or("focus map missing")? as usize;
    let p = &population["rows"][row];
    require(
        p["input_index"] == 245 && p["position"] == 4 && p["target_label_only"] == 267,
        "focus identity",
    )?;
    let snapshot = run.join("candidate-row-0245-position-04.json");
    let f = read(&snapshot)?;
    for key in [
        "input_index",
        "position",
        "id",
        "actual_prefix_ids",
        "target_label_only",
    ] {
        require(f[key] == p[key], "snapshot/population identity")?;
    }
    let n = &f["native"];
    let post = codes(&n["post_state"])?;
    let ustate = codes(&n["continuation"]["state_codes"])?;
    let cp = run.join("checkpoint-0001");
    let binding = SourceActionBinding::new(&fs::read(cp.join("native/tokenizer.json"))?)?;
    let g = NativeGeometricGenerate::from_bytes(&fs::read(cp.join("generate.bin"))?, &binding)?;
    let source_metadata = cp.join("native/metadata.json");
    let source_binding = NativeArtifactBinding {
        metadata_sha256: hash(&source_metadata)?,
        identity: serde_json::from_value(read(&source_metadata)?["identity"].clone())?,
    };
    let field = NativeContinuationField::from_bytes(
        &fs::read(cp.join("continuation-field.bin"))?,
        &source_binding,
        &g,
    )?;
    let mut reducer =
        NativeVocabularyActions::new(binding, &fs::read(cp.join("native/consumer/exp-q31.bin"))?)?;
    require(
        g.lanes() == 8
            && post.len() == 8
            && ustate.len() == 8
            && g.energy().edges().len() == 4
            && reducer.legal_token_ids().iter().copied().eq(0..4096)
            && field.applies_to_copy(),
        "H4/eight lanes/four edges/full legal4096/sharedU",
    )?;
    let mut raw = vec![0; 4096];
    g.score_into(&post, &mut raw, &mut GenerateReadCounts::default())?;
    let mut u = vec![0; 4096];
    field.score_delta_into(&ustate, &g, &mut u, &mut ContinuationReadCounts::default())?;
    require(
        u == vec_i64(&n["continuation"]["delta_scores_q24"])?,
        "all4096 U parity",
    )?;
    let full = raw
        .iter()
        .zip(&u)
        .map(|(&a, &b)| a.checked_add(b).ok_or_else(|| "Generate/U overflow".into()))
        .collect::<Result<Vec<_>>>()?;
    require(
        full == vec_i64(&n["generate_q24"])?,
        "all4096 Generate+U parity",
    )?;
    let ids = n["copy_ids"]
        .as_array()
        .ok_or("Copy IDs missing")?
        .iter()
        .map(|x| {
            x.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| "Copy ID invalid".into())
        })
        .collect::<Result<Vec<_>>>()?;
    let copies = vec_i64(&n["copy_q24"])?;
    let trace = reducer.reduce_trace(&full, &ids, &copies)?;
    require(
        serde_json::to_value(&trace.summary)? == n["pool"]["summary"]
            && serde_json::to_value(&trace.token_masses)? == n["pool"]["token_masses"],
        "complete native pool/mass parity",
    )?;
    for atom in &trace.actions {
        require(
            reducer.atom_weight_at_reference(atom.raw_score_q24, trace.summary.max_score_q24)?
                == atom.weight_q31,
            "native atom weight parity",
        )?;
    }
    let mut bases = Vec::with_capacity(4096);
    let mut current_pairs = Vec::with_capacity(4096);
    let mut counts = GenerateReadCounts::default();
    let mut key = [0u32; 12];
    for token in 0..4096 {
        g.factor_incidence_into(&post, token, &mut key, &mut counts)?;
        let mut sum = 0i64;
        for e in 0..4 {
            let k = usize::try_from(key[8 + e])?
                .checked_sub(960 + e * 14400)
                .ok_or("pair key offset")?;
            require(k < 14400, "pair key domain")?;
            sum += i64::from(g.energy().get_pair(e, (k / 120) as u8, (k % 120) as u8)?);
        }
        current_pairs.push(sum);
        bases.push(
            full[token]
                .checked_sub(sum * UNIT)
                .ok_or("pair subtraction overflow")?,
        );
    }
    let grid = grids(&bases, 28)?;
    let refs = references(&grid, &copies)?;
    require(
        refs.contains(&trace.summary.max_score_q24),
        "actual reference included",
    )?;
    for token in 0..4096 {
        require(
            grid[token].contains(&clip(full[token])) && (-28..=28).contains(&current_pairs[token]),
            "current pair assignment/grid parity",
        )?;
    }
    let target = 267u32;
    let rival = 307u32;
    let target_max = *grid[target as usize].iter().max().ok_or("target grid")?;
    let mut bounds = Vec::with_capacity(refs.len());
    let mut survivors = 0usize;
    let mut equality = 0usize;
    for &r in &refs {
        let (upper, lower) = bound_at(r, target_max, &ids, &copies, target, rival, |s, r| {
            Ok(reducer.atom_weight_at_reference(s, r)?)
        })?;
        survivors += usize::from(upper >= lower);
        equality += usize::from(upper == lower);
        bounds.push([r, i64::try_from(upper)?, i64::try_from(lower)?]);
    }
    let actual = bound_at(
        trace.summary.max_score_q24,
        target_max,
        &ids,
        &copies,
        target,
        rival,
        |s, r| Ok(reducer.atom_weight_at_reference(s, r)?),
    )?;
    require(
        actual.0 >= trace.token_masses[target as usize].weight_q31
            && actual.1 <= trace.token_masses[rival as usize].weight_q31,
        "actual target/rival masses bounded",
    )?;
    Ok(
        json!({"schema":"uor-r4.native-pair-copy-bound/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"input_report_sha256":REPORT,"input_manifest_sha256":MANIFEST,"snapshot_sha256":hash(&snapshot)?,"generate_sha256":hash(&cp.join("generate.bin"))?,"continuation_sha256":hash(&cp.join("continuation-field.bin"))?,"focus":p,"target":target,"rival":rival,"legal_generate_tokens":4096,"pair_edges":4,"grid_sum_min":-28,"grid_sum_max":28,"grid_unit_q24":UNIT,"frozen_base_includes_U":true,"target_fixed_copy_aliases":ids.iter().filter(|&&x|x==target).count(),"rival_fixed_copy_aliases":ids.iter().filter(|&&x|x==rival).count(),"physical_copy_ids":ids,"physical_copy_scores_q24":copies,"target_fixed_base_q24":bases[target as usize],"target_generate_clipped_max_q24":target_max,"reference_count":refs.len(),"reference_bounds_columns":["reference_q24","target_mass_upper_q31","rival_copy_mass_lower_q31"],"reference_bounds":bounds,"surviving_references":survivors,"equality_references":equality,"excluded_by_relaxed_bound":survivors==0,"actual_reference_q24":trace.summary.max_score_q24,"actual_target_mass":trace.token_masses[target as usize].weight_q31,"actual_rival_mass":trace.token_masses[rival as usize].weight_q31,"interpretation":if survivors==0 {"Strict upper<lower at every possible reference excludes target victory under frozen Copy/unary/bias/prototypes/U with any legal shared pair coefficients."}else{"Bound survives: this does not establish any shared-coefficient assignment, objective/guard feasibility or target victory."},"relaxation":"All legal token scores independently use four-code sums; shared coefficient constraints dropped. Rival Generate mass omitted. Equality cannot exclude target267 because it wins an exact tie against307.","new_gradients":0,"new_proposals":0,"new_model_steps":0,"saved_score_reconstructions":1,"elapsed_seconds":start.elapsed().as_secs_f64()}),
    )
}
fn main() -> Result<()> {
    let argv = std::env::args_os().skip(1).collect::<Vec<_>>();
    require(argv.len() == 2, "usage: native-pair-copy-bound RUN OUTPUT")?;
    let run = fs::canonicalize(PathBuf::from(&argv[0]))?;
    let out = PathBuf::from(&argv[1]);
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let absolute = fs::canonicalize(parent)?.join(out.file_name().ok_or("output leaf missing")?);
    require(
        !absolute.starts_with(&run) && !run.starts_with(&absolute),
        "input/output ancestry",
    )?;
    report_output::claim(&out)?;
    let result = analyze(&run);
    let value = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-pair-copy-bound/1","status":"FAILED","error":e.to_string(),"new_gradients":0,"new_proposals":0,"new_model_steps":0})
        }
    };
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&value)?)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn possible_references_cover_exhaustive_toy_scores_offgrid_copy_and_clipping() -> Result<()> {
        let bases = [SCORE_CLIP_Q24 - UNIT, -SCORE_CLIP_Q24];
        let copies = [12345];
        let grid = grids(&bases, 4)?;
        let refs = references(&grid, &copies)?;
        for a in &grid[0] {
            for b in &grid[1] {
                assert!(refs.contains(&(*a).max(*b).max(12345)));
            }
        }
        assert!(refs.contains(&SCORE_CLIP_Q24));
        assert!(refs.contains(&12345));
        let empty = references(&grid, &[])?;
        for a in &grid[0] {
            for b in &grid[1] {
                assert!(empty.contains(&(*a).max(*b)));
            }
        }
        Ok(())
    }
    #[test]
    fn duplicate_aliases_and_equality_do_not_exclude_smaller_target() -> Result<()> {
        let weight = |s: i64, r: i64| -> Result<u64> {
            require(r >= clip(s), "bad toy reference")?;
            Ok((100 - (r - clip(s)).min(99)) as u64)
        };
        let (upper, lower) =
            bound_at(0, -50, &[267, 307, 307], &[-50, -50, -50], 267, 307, weight)?;
        assert_eq!(upper, lower);
        assert!(!(upper < lower));
        let (upper, lower) = bound_at(0, -50, &[], &[], 267, 307, weight)?;
        assert!(upper > lower);
        Ok(())
    }
    #[test]
    fn invalid_h4_and_mismatched_aliases_fail() -> Result<()> {
        assert!(codes(&json!([120])).is_err());
        assert!(bound_at(0, 0, &[267], &[], 267, 307, |_, _| Ok(1)).is_err());
        Ok(())
    }
}
