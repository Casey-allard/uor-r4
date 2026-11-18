//! A bounded, paired source-to-emission diagnostic on the released M1 trunks.
use m1_d11_artifact_scorer_instrument as scorer;
use scorer::device::DeviceScorer;
use scorer::kernel::{compare_row_with_target, pack_rows_with_targets};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::discrete_d11::{spiralcore_orbit_roots, IntegerDiscreteModel};

const PANEL: &str = "68357e20e1dd5e17854e6b124a1de20239508fa7371f73a7fb725a9fd80d9098";
const RELEASED: [&str; 2] = [
    "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d",
    "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de",
];
const EOS: u32 = 1;
const CAP: usize = 16;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    pair: String,
    role: String,
    prefix: Vec<u32>,
    asked_start: usize,
    asked_end: usize,
    other_start: usize,
    other_end: usize,
    expected: Vec<u32>,
}

impl Case {
    fn validate(&self, model: &IntegerDiscreteModel) -> Result<(), String> {
        let asked = self
            .prefix
            .get(self.asked_start..self.asked_end)
            .ok_or("bad asked span")?;
        let other = self
            .prefix
            .get(self.other_start..self.other_end)
            .ok_or("bad other span")?;
        if asked.is_empty()
            || other.is_empty()
            || asked == other
            || !(self.asked_end <= self.other_start || self.other_end <= self.asked_start)
            || self.expected.last() != Some(&EOS)
            || self.expected[..self.expected.len() - 1] != *asked
            || self.prefix.len() + CAP > model.shape().context as usize
            || self.prefix.iter().any(|&t| t >= model.shape().vocabulary)
        {
            return Err(format!(
                "invalid span, expected answer or context for {}",
                self.id
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Native,
    NoPointer,
    IdentityPick,
    AskedPointer,
    OtherPointer,
    AskedWriter,
    OtherWriter,
    ForceFirst,
    ForceSpan,
    ForceOther,
}
const ARMS: [Arm; 10] = [
    Arm::Native,
    Arm::NoPointer,
    Arm::IdentityPick,
    Arm::AskedPointer,
    Arm::OtherPointer,
    Arm::AskedWriter,
    Arm::OtherWriter,
    Arm::ForceFirst,
    Arm::ForceSpan,
    Arm::ForceOther,
];

impl Arm {
    fn intervention(self, case: &Case, step: usize) -> [i32; 2] {
        match self {
            Self::NoPointer => [4, -1],
            Self::IdentityPick => [1, -1],
            Self::AskedPointer | Self::AskedWriter if step < case.asked_end - case.asked_start => [
                if self == Self::AskedWriter { 3 } else { 2 },
                (case.asked_start + step) as i32,
            ],
            Self::OtherPointer | Self::OtherWriter if step < case.other_end - case.other_start => [
                if self == Self::OtherWriter { 3 } else { 2 },
                (case.other_start + step) as i32,
            ],
            _ => [0, -1],
        }
    }

    fn feed(self, case: &Case, step: usize, model_token: u32) -> u32 {
        let span = match self {
            Self::ForceFirst if step == 0 => case.prefix.get(case.asked_start..case.asked_end),
            Self::ForceSpan => case.prefix.get(case.asked_start..case.asked_end),
            Self::ForceOther => case.prefix.get(case.other_start..case.other_end),
            _ => None,
        };
        span.and_then(|v| v.get(step))
            .copied()
            .unwrap_or(model_token)
    }
}

// Independent selector oracle on the immutable packed codebook. The CUDA
// diagnostic computes L1/2 instead. A pick is the *successor* token position.
fn selector_sources(prefix: &[u32], codes: &[u32], signatures: &[u64]) -> [i32; 3] {
    let end = prefix.len() - 1;
    let query = codes[prefix[end] as usize];
    let natural = prefix[..end]
        .iter()
        .rposition(|&t| codes[t as usize] == query)
        .map_or(-1, |p| (p + 1) as i32);
    let mut source = -1;
    let mut distance = -1;
    for p in 1..end {
        let d = (signatures[prefix[p] as usize] ^ signatures[prefix[end] as usize]).count_ones()
            + (signatures[prefix[p - 1] as usize] ^ signatures[prefix[end - 1] as usize])
                .count_ones();
        if distance < 0 || d as i32 <= distance {
            distance = d as i32;
            source = (p + 1) as i32;
        }
    }
    [natural, source, distance]
}

fn signatures(codes: &[u32]) -> Vec<u64> {
    let roots = spiralcore_orbit_roots();
    codes
        .iter()
        .map(|&packed| {
            let mut sig = 0u64;
            for block in 0..4 {
                let root = roots[((packed >> (4 * block)) & 15) as usize];
                for (d, coordinate) in root.into_iter().enumerate() {
                    let sign = u64::from(coordinate > 0) | (u64::from(coordinate < 0) << 1);
                    sig |= sign << (16 * block + 2 * d);
                }
            }
            sig
        })
        .collect()
}

struct Rollout<'a> {
    case: &'a Case,
    arm: Arm,
    prefix: Vec<u32>,
    steps: Vec<Value>,
}

fn position(index: i32) -> Option<usize> {
    usize::try_from(index).ok()
}

pub fn run(artifact: &str, fixture: &str) -> Result<Value, String> {
    let bytes = fs::read(artifact).map_err(|e| e.to_string())?;
    let hash = sha256(&bytes);
    if !RELEASED.contains(&hash.as_str()) {
        return Err("unregistered artifact SHA-256".into());
    }
    let input = fs::read(fixture).map_err(|e| e.to_string())?;
    if sha256(&input) != PANEL {
        return Err("unregistered role-swap fixture".into());
    }
    let cases: Vec<Case> = serde_json::from_slice(&input).map_err(|e| e.to_string())?;
    let (tables, model) = scorer::artifact::ArtifactTables::from_bytes(&bytes)?;
    let ids: BTreeSet<_> = cases.iter().map(|c| &c.id).collect();
    if ids.len() != 16 || cases.len() != 16 || !model.is_trunk_d11() {
        return Err("unexpected fixture or trunk".into());
    }
    for case in &cases {
        case.validate(&model)?;
    }
    let device = DeviceScorer::new(0, &tables)?;
    let codes = model.packed_codebook_roots();
    let signs = signatures(&codes);
    let mut rollouts: Vec<_> = cases
        .iter()
        .flat_map(|case| {
            ARMS.map(|arm| Rollout {
                case,
                arm,
                prefix: case.prefix.clone(),
                steps: vec![],
            })
        })
        .collect();
    let mut baseline_comparisons = 0usize;
    let mut selector_comparisons = 0usize;
    let mut target_invariance_checks = 0usize;
    for step in 0..CAP {
        let active: Vec<usize> = rollouts
            .iter()
            .enumerate()
            .filter(|(_, r)| step == 0 || r.prefix.last() != Some(&EOS))
            .map(|(i, _)| i)
            .collect();
        if active.is_empty() {
            break;
        }
        let prefixes: Vec<_> = active
            .iter()
            .map(|&i| rollouts[i].prefix.as_slice())
            .collect();
        let targets: Vec<_> = active
            .iter()
            .map(|&i| rollouts[i].case.expected.get(step).copied().unwrap_or(EOS))
            .collect();
        let overrides: Vec<_> = active
            .iter()
            .map(|&i| rollouts[i].arm.intervention(rollouts[i].case, step))
            .collect();
        let batch = pack_rows_with_targets(&prefixes, &targets)?;
        let (scores, traces) = device.score_pointers(&batch, &overrides)?;
        // Original untraced production GPU entry, using alternate labels, checks
        // the predicted tokens at the exact same native prefixes independently.
        let ordinary: Vec<_> = active
            .iter()
            .enumerate()
            .filter(|(_, &i)| rollouts[i].arm == Arm::Native)
            .map(|(row, _)| row)
            .collect();
        let ordinary_prefixes: Vec<_> = ordinary.iter().map(|&r| prefixes[r]).collect();
        let alternate_labels: Vec<_> = ordinary
            .iter()
            .map(|&r| (targets[r] + 1) % model.shape().vocabulary)
            .collect();
        let control = device.score(&pack_rows_with_targets(
            &ordinary_prefixes,
            &alternate_labels,
        )?)?;
        for (&r, comp) in ordinary.iter().zip(control) {
            if comp.status != 0
                || scores[r].read_vector != comp.read_vector
                || scores[r].top8 != comp.top8
                || scores[r].admitted_cells != comp.admitted_cells
                || scores[r].prediction != comp.prediction
            {
                return Err("trace or altered scoring target changed the native forward".into());
            }
            target_invariance_checks += 1;
        }
        for (row, &i) in active.iter().enumerate() {
            let run = &mut rollouts[i];
            let case = run.case;
            let score = &scores[row];
            let target = targets[row];
            let [orbit, pair, pair_distance, used, read, rec, mixed, writer, winner_lo, winner_hi, target_lo, target_hi, in_pool] =
                traces[row];
            let score_words =
                |lo: i32, hi: i32| u64::from(lo as u32) | (u64::from(hi as u32) << 32);
            if score.status != 0 {
                return Err(format!(
                    "{} {:?} step {step}: kernel status {}",
                    case.id, run.arm, score.status
                ));
            }
            let reference = selector_sources(&run.prefix, &codes, &signs);
            if [orbit, pair, pair_distance] != reference {
                return Err("source index or distance mismatch".into());
            }
            let [mode, given] = overrides[row];
            let expected_source = match mode {
                0 | 3 => orbit,
                1 => pair,
                2 => given,
                4 => -1,
                _ => unreachable!(),
            };
            if used != expected_source
                || (mode == 3 && writer as u32 != run.prefix[given as usize])
                || (mode != 3 && writer != mixed)
            {
                return Err("intervention trace inconsistent with selected source".into());
            }
            selector_comparisons += 1;
            if mode == 0 {
                compare_row_with_target(&model, &run.prefix, target, score)?;
                baseline_comparisons += 1;
            }
            let source_token = position(used).map(|p| run.prefix[p]);
            let next = run.arm.feed(case, step, score.prediction);
            run.steps.push(json!({
                "step":step,"prefix_len":run.prefix.len(),"input_last":run.prefix.last(),"mode":mode,
                "orbit_key_position":position(orbit).map(|p| p-1),"orbit_successor_position":position(orbit),
                "orbit_successor_id":position(orbit).map(|p| run.prefix[p]),
                "pick_key_position":position(pair).map(|p| p-1),"pick_successor_position":position(pair),
                "pick_successor_id":position(pair).map(|p| run.prefix[p]),"pick_distance":position(pair_distance),
                "effective_source_position":position(used),"effective_source_id":source_token,
                "pointer_score_boost":if used >= 0 {256} else {0},
                "forced_writer_source_position":if mode == 3 {position(given)} else {None},
                "mixed_score":score_words(winner_lo,winner_hi),
                "scoring_target":target,"target_mixed_score":score_words(target_lo,target_hi),
                "target_in_mixer_pool":in_pool != 0,
                "read_code":read,"recurrence_code":rec,"mixed_token":mixed,"writer_input_token":writer,
                "final_query":score.read_vector,"admitted_cells":score.admitted_cells,"top8":score.top8,
                "expected":case.expected.get(step),"expected_cell":case.expected.get(step).map(|&t| model.cells()[t as usize]),
                "predicted":score.prediction,"fed":next
            }));
            run.prefix.push(next);
        }
    }
    let rows:Vec<Value> = rollouts.into_iter().map(|r| {
        let generated = &r.prefix[r.case.prefix.len()..];
        let first_error = generated.iter().zip(&r.case.expected).position(|(a,b)| a!=b)
            .or_else(|| (generated.len()!=r.case.expected.len()).then_some(generated.len().min(r.case.expected.len())));
        json!({"id":r.case.id,"role":r.case.role,"pair":r.case.pair,"arm":format!("{:?}",r.arm),
            "expected":r.case.expected,"generated":generated,"exact":generated==r.case.expected,
            "first_error":first_error,"stop":if generated.last()==Some(&EOS) {"eos"} else {"cap"},"steps":r.steps})
    }).collect();
    Ok(
        json!({"schema":"uor-r4.discrete-role-swap/1","artifact_sha256":hash,"fixture_sha256":PANEL,
        "shape":model.shape(),"eos":EOS,"cap":CAP,"cuda_target":scorer::device::IMAGE_TARGET,
        "ptx_sha256":scorer::device::ptx_sha256(),"cubin_sha256":scorer::device::cubin_sha256(),
        "ordinary_cuda_changed_target_comparisons":target_invariance_checks,
        "native_rust_full_decision_comparisons":baseline_comparisons,"independent_source_comparisons":selector_comparisons,
        "rows":rows}),
    )
}
