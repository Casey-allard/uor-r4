//! Diagnostic receipt parsing and exact accounting only. No host model/oracle.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstrumentError {
    Trace(String),
    Proposal(String),
    Resource(String),
    Routing(String),
}
impl std::fmt::Display for InstrumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (kind, message) = match self {
            Self::Trace(message) => ("trace", message),
            Self::Proposal(message) => ("proposal", message),
            Self::Resource(message) => ("resource", message),
            Self::Routing(message) => ("routing", message),
        };
        write!(f, "decomposition {kind}: {message}")
    }
}
impl std::error::Error for InstrumentError {}
impl From<InstrumentError> for String {
    fn from(error: InstrumentError) -> Self {
        error.to_string()
    }
}
pub type InstrumentResult<T> = Result<T, InstrumentError>;

/// Exact embedded AOT images and the resource reports from their assembly.
/// Validation checks host metadata only; it performs no CUDA/model operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CudaImages {
    pub schema: String,
    pub format: String,
    pub target: String,
    pub ordinary_sha256: String,
    pub decomposition_sha256: String,
    pub ordinary_report_sha256: String,
    pub decomposition_report_sha256: String,
}
impl CudaImages {
    pub fn validate(&self) -> Result<(), String> {
        let target = self.target.strip_prefix("sm_");
        let valid_target = target.is_some_and(|digits| {
            !digits.is_empty()
                && digits.bytes().all(|b| b.is_ascii_digit())
                && digits
                    .parse::<u32>()
                    .is_ok_and(|n| n > 0 && n.to_string() == digits)
        });
        if self.schema != "uor-r4.cuda-images/1" || self.format != "cubin" || !valid_target {
            return Err(
                "CUDA image schema/format/target differs from an exact AOT cubin binding".into(),
            );
        }
        for (name, hash) in [
            ("ordinary_sha256", &self.ordinary_sha256),
            ("decomposition_sha256", &self.decomposition_sha256),
            ("ordinary_report_sha256", &self.ordinary_report_sha256),
            (
                "decomposition_report_sha256",
                &self.decomposition_report_sha256,
            ),
        ] {
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || !hash.bytes().any(|b| b != b'0')
            {
                return Err(format!(
                    "CUDA image {name} requires a nonzero lowercase SHA256"
                ));
            }
        }
        Ok(())
    }
}

pub const LANES: usize = 16;
pub const WORDS: usize = 40;
pub const MAGIC: u64 = 0x434631;
pub const LIMITATION: &str = "no profiling-off control for the already-merged enclosing host/event instrument — E/S bounds only the incremental internal clocks.";
pub const DENOMINATOR: &str = "lane end-start cycles, including setup/tail/uncovered overhead; excludes final record stores which remain in enclosing CUDA event";
pub const WORD_NAMES: [&str; WORDS] = [
    "magic",
    "candidate",
    "terminal_record",
    "fault_code",
    "lane_cycles",
    "start_clock",
    "end_clock",
    "visited_rows",
    "visited_mask",
    "eligible_rows",
    "eligible_mask",
    "baseline_read_calls",
    "baseline_writer_calls",
    "virtual_read_calls",
    "virtual_writer_calls",
    "baseline_read_cycles",
    "baseline_writer_cycles",
    "eligibility_cycles",
    "virtual_read_cycles",
    "virtual_writer_cycles",
    "accumulation_cycles",
    "baseline_occupancy_cycles",
    "baseline_selection_member_cycles",
    "baseline_final_cycles",
    "baseline_selected_cells",
    "baseline_token_cell_calls",
    "baseline_member_distances",
    "virtual_occupancy_cycles",
    "virtual_selection_member_cycles",
    "virtual_final_cycles",
    "virtual_selected_cells",
    "virtual_token_cell_calls",
    "virtual_member_distances",
    "reserved33",
    "reserved34",
    "reserved35",
    "reserved36",
    "reserved37",
    "reserved38",
    "reserved39",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ratio {
    pub numerator: u128,
    pub denominator: u128,
}
impl Ratio {
    fn at_least_percent(&self, percent: u128) -> bool {
        self.denominator != 0 && self.numerator * 100 >= self.denominator * percent
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct LaneRecord {
    pub words: Vec<u64>,
    pub candidate: usize,
    pub fault_code: u64,
    pub visited_rows: u64,
    pub eligible_rows: u64,
    pub visited_mask: u64,
    pub eligible_mask: u64,
    pub lane_cycles: u64,
    pub accounted_phase_cycles: u64,
    pub uncovered_cycles: u64,
    pub writer_uncovered_cycles: [u64; 2],
    pub baseline_share: Ratio,
    pub writer_share: Ratio,
}

pub fn decode(raw: &[u64]) -> InstrumentResult<Vec<LaneRecord>> {
    decode_words(raw).map_err(InstrumentError::Trace)
}
fn decode_words(raw: &[u64]) -> Result<Vec<LaneRecord>, String> {
    if raw.len() != LANES * WORDS {
        return Err("diagnostic trace inventory must be 16 x 40 words".into());
    }
    raw.chunks_exact(WORDS)
        .enumerate()
        .map(|(candidate, t)| {
            let sum = |values: &[u64]| {
                values
                    .iter()
                    .try_fold(0u64, |sum, x| sum.checked_add(*x))
                    .ok_or("cycle/count overflow".to_owned())
            };
            if t[0] != MAGIC
                || t[1] != candidate as u64
                || t[2] != 1
                || t[6].checked_sub(t[5]) != Some(t[4])
                || t[4] == 0
                || t[7] != u64::from(t[8].count_ones())
                || t[9] != u64::from(t[10].count_ones())
                || t[10] & !t[8] != 0
                || t[7] > 64
                || t[11] != t[7]
                || t[12] > t[11]
                || t[13] > t[9]
                || t[14] > t[13]
                || t[33..].iter().any(|x| *x != 0)
            {
                return Err(format!(
                    "incomplete or inconsistent diagnostic lane {candidate}"
                ));
            }
            // Faulted lanes are retained as raw records and never counted as support.
            if t[3] == 0 && (t[11] != t[12] || t[13] != t[14] || t[13] != t[9]) {
                return Err(format!("successful lane {candidate} has incomplete calls"));
            }
            let accounted = sum(&t[15..21])?;
            let uncovered = t[4]
                .checked_sub(accounted)
                .ok_or("phase cycles exceed lane denominator")?;
            let nested = [sum(&t[21..24])?, sum(&t[27..30])?];
            let writer_uncovered = [
                t[16]
                    .checked_sub(nested[0])
                    .ok_or("baseline writer nested cycles exceed parent")?,
                t[19]
                    .checked_sub(nested[1])
                    .ok_or("virtual writer nested cycles exceed parent")?,
            ];
            let denominator = u128::from(t[4]);
            Ok(LaneRecord {
                words: t.to_vec(),
                candidate,
                fault_code: t[3],
                visited_rows: t[7],
                eligible_rows: t[9],
                visited_mask: t[8],
                eligible_mask: t[10],
                lane_cycles: t[4],
                accounted_phase_cycles: accounted,
                uncovered_cycles: uncovered,
                writer_uncovered_cycles: writer_uncovered,
                baseline_share: Ratio {
                    numerator: u128::from(sum(&t[15..18])?),
                    denominator,
                },
                writer_share: Ratio {
                    numerator: u128::from(sum(&[t[16], t[19]])?),
                    denominator,
                },
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProposalReceipt {
    pub losses: Vec<u64>,
    pub status: u32,
    pub proposal: [u64; 5],
}
impl ProposalReceipt {
    pub fn validate(&self) -> InstrumentResult<()> {
        self.validate_fields().map_err(InstrumentError::Proposal)
    }
    fn validate_fields(&self) -> Result<(), String> {
        if self.losses.len() != 16
            || self.status != 0
            || self.proposal[0] >= 16
            || self.proposal[3] != 0
            || self.proposal[4] > 1
        {
            return Err("incomplete/refused/nonzero-epoch proposal receipt".into());
        }
        // This is storage consistency, not a host choice or loss oracle.
        if self.proposal[1] != self.losses[0]
            || self.proposal[2] != self.losses[self.proposal[0] as usize]
        {
            return Err("proposal storage disagrees with loss storage".into());
        }
        Ok(())
    }
}
pub fn parity(u: &ProposalReceipt, d: &ProposalReceipt) -> InstrumentResult<()> {
    proposal_parity(u, d).map_err(InstrumentError::Proposal)
}
fn proposal_parity(u: &ProposalReceipt, d: &ProposalReceipt) -> Result<(), String> {
    u.validate()?;
    d.validate()?;
    if u != d {
        Err("U/D integer loss/status/proposal mismatch".into())
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PtxasResources {
    pub registers: u64,
    pub stack_bytes: u64,
    pub spill_stores: u64,
    pub spill_loads: u64,
}
fn number_before(line: &str, suffix: &str) -> Option<u64> {
    let before = line.split_once(suffix)?.0;
    before
        .split(|c: char| !c.is_ascii_digit())
        .rfind(|s| !s.is_empty())?
        .parse()
        .ok()
}
pub fn parse_ptxas(log: &str, entry: &str) -> InstrumentResult<PtxasResources> {
    parse_resource_log(log, entry).map_err(InstrumentError::Resource)
}
fn parse_resource_log(log: &str, entry: &str) -> Result<PtxasResources, String> {
    let mut active = false;
    let mut stack = None;
    let mut stores = None;
    let mut loads = None;
    let mut registers = None;
    for line in log.lines() {
        if line.contains("Compiling entry function") || line.contains("Function properties for") {
            active = line.contains(&format!("'{entry}'"))
                || line.trim_end().ends_with(&format!("for {entry}"));
        }
        if active {
            stack = number_before(line, " bytes stack frame").or(stack);
            stores = number_before(line, " bytes spill stores").or(stores);
            loads = number_before(line, " bytes spill loads").or(loads);
            registers = number_before(line, " registers").or(registers);
        }
    }
    Ok(PtxasResources {
        registers: registers.ok_or("missing ptxas register count")?,
        stack_bytes: stack.ok_or("missing ptxas stack count")?,
        spill_stores: stores.ok_or("missing ptxas spill stores")?,
        spill_loads: loads.ok_or("missing ptxas spill loads")?,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelResources {
    pub registers: u64,
    pub local_bytes: u64,
    pub shared_bytes: u64,
    pub max_threads_per_block: u64,
    pub ptxas: PtxasResources,
}
pub fn resource_parity(u: &KernelResources, d: &KernelResources) -> bool {
    // PM amendment6222308938 supersedes U+16 before any dose. Zero delta
    // between runtime and AOT registers/local is intentionally fail-closed:
    // a different JIT resource layout needs a new disposition, not attribution.
    // Launchability discloses support for the fixed host block of16; it is
    // not a register guard or an occupancy/performance measurement.
    let consistent = |r: &KernelResources| {
        r.registers == r.ptxas.registers && r.local_bytes == r.ptxas.stack_bytes
    };
    consistent(u)
        && consistent(d)
        && u.max_threads_per_block >= 16
        && d.max_threads_per_block >= 16
        && d.local_bytes <= u.local_bytes.saturating_add(1024)
        && d.ptxas.stack_bytes <= u.ptxas.stack_bytes.saturating_add(1024)
        && d.shared_bytes == u.shared_bytes
        && u.ptxas.spill_stores == 0
        && d.ptxas.spill_stores == 0
        && u.ptxas.spill_loads == 0
        && d.ptxas.spill_loads == 0
}

#[derive(Debug, Clone, Serialize)]
pub struct CellDecision {
    pub e_ns: u128,
    pub s_ns: u128,
    pub minimum_u_ns: u128,
    pub unresolved: bool,
    pub baseline_material: bool,
    pub writer_dominant: bool,
    pub reason: &'static str,
}
pub fn decide(
    samples: &[(u128, u128, Vec<LaneRecord>)],
    resources_match: bool,
) -> InstrumentResult<CellDecision> {
    decide_cell(samples, resources_match).map_err(InstrumentError::Routing)
}
fn decide_cell(
    samples: &[(u128, u128, Vec<LaneRecord>)],
    resources_match: bool,
) -> Result<CellDecision, String> {
    if samples.len() != 3 {
        return Err("three matched samples required per operation/policy cell".into());
    }
    let min_u = samples.iter().map(|s| s.0).min().ok_or("empty samples")?;
    let spread = |mode: usize| {
        let values: Vec<_> = samples
            .iter()
            .map(|s| if mode == 0 { s.0 } else { s.1 })
            .collect();
        values.iter().max().copied().unwrap_or(0) - values.iter().min().copied().unwrap_or(0)
    };
    let e = samples.iter().map(|s| s.0.abs_diff(s.1)).max().unwrap_or(0);
    let s = spread(0).max(spread(1));
    let mut baseline = [false; 2];
    let mut writer = [false; 2];
    let mut valid = resources_match && min_u != 0;
    // Retained means ALL 16 completed, nonfaulted lanes in EACH sample. No
    // successful subset can hide a refused lane; eligible masks must agree
    // on the intersection of visited rows, so legal early breaks stay visible.
    for (_, _, lanes) in samples {
        if lanes.len() != 16 {
            valid = false;
        }
        for a in lanes {
            let denominator = u128::from(a.lane_cycles);
            if denominator == 0
                || a.baseline_share.denominator != denominator
                || a.writer_share.denominator != denominator
                || a.baseline_share.numerator > denominator
                || a.writer_share.numerator > denominator
            {
                valid = false;
                continue;
            }
            if a.fault_code != 0 {
                valid = false;
            }
            baseline[usize::from(a.baseline_share.at_least_percent(20))] = true;
            writer[usize::from(a.writer_share.at_least_percent(80))] = true;
            for b in lanes {
                let common = a.visited_mask & b.visited_mask;
                if (a.eligible_mask ^ b.eligible_mask) & common != 0 {
                    valid = false;
                }
            }
        }
    }
    let perturbation = e.max(s).checked_mul(5).is_none_or(|x| x >= min_u);
    let unresolved = !valid || perturbation || baseline == [true, true] || writer == [true, true];
    Ok(CellDecision {
        e_ns: e,
        s_ns: s,
        minimum_u_ns: min_u,
        unresolved,
        baseline_material: !unresolved && baseline[1],
        writer_dominant: !unresolved && writer[1],
        reason: if unresolved {
            "UNRESOLVED: perturbation, resource, lane or phase-share boundary"
        } else {
            "descriptive routing only; H1 magnitude NOT_APPROVED"
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn images() -> CudaImages {
        CudaImages {
            schema: "uor-r4.cuda-images/1".into(),
            format: "cubin".into(),
            target: "sm_120".into(),
            ordinary_sha256: "1".repeat(64),
            decomposition_sha256: "2".repeat(64),
            ordinary_report_sha256: "3".repeat(64),
            decomposition_report_sha256: "4".repeat(64),
        }
    }
    #[test]
    fn cuda_image_metadata_requires_exact_format_and_canonical_target() {
        let binding = images();
        assert!(binding.validate().is_ok());
        for target in [
            "",
            "compute_120",
            "sm_0",
            "sm_0120",
            "sm_120a",
            "sm_4294967296",
        ] {
            let mut wrong = binding.clone();
            wrong.target = target.into();
            assert!(wrong.validate().is_err(), "accepted target {target}");
        }
        let mut wrong = binding.clone();
        wrong.format = "ptx".into();
        assert!(wrong.validate().is_err());
        let mut wrong = binding;
        wrong.schema = "uor-r4.cuda-images/2".into();
        assert!(wrong.validate().is_err());
    }
    #[test]
    fn cuda_image_metadata_refuses_pending_malformed_hashes_and_unknown_fields() {
        let value = serde_json::to_value(images()).unwrap();
        for field in [
            "ordinary_sha256",
            "decomposition_sha256",
            "ordinary_report_sha256",
            "decomposition_report_sha256",
        ] {
            for malformed in [
                "PENDING".into(),
                "0".repeat(64),
                "A".repeat(64),
                "1".repeat(63),
            ] {
                let mut wrong = value.clone();
                wrong[field] = serde_json::json!(malformed);
                let binding: CudaImages = serde_json::from_value(wrong).unwrap();
                assert!(binding.validate().is_err(), "accepted malformed {field}");
            }
        }
        let mut unknown = value;
        unknown["fallback"] = serde_json::json!("ptx");
        assert!(serde_json::from_value::<CudaImages>(unknown).is_err());
    }
    fn words() -> Vec<u64> {
        let mut out = vec![0; LANES * WORDS];
        for (i, t) in out.chunks_exact_mut(WORDS).enumerate() {
            t[0] = MAGIC;
            t[1] = i as u64;
            t[2] = 1;
            t[4] = 1000;
            t[5] = 10;
            t[6] = 1010;
            t[7] = 1;
            t[8] = 1;
            t[9] = 1;
            t[10] = 1;
            t[11..15].fill(1);
            t[15] = 10;
            t[16] = 440;
            t[17] = 10;
            t[18] = 10;
            t[19] = 440;
            t[20] = 10;
            t[21] = 100;
            t[22] = 300;
            t[23] = 20;
            t[27] = 100;
            t[28] = 300;
            t[29] = 20;
        }
        out
    }
    #[test]
    fn producer_shaped_trace_accounts_nested_cycles_without_double_count() {
        let lanes = decode(&words()).unwrap();
        assert_eq!(lanes[0].uncovered_cycles, 80);
        assert_eq!(lanes[0].writer_uncovered_cycles, [20, 20]);
        assert_eq!(
            lanes[0].baseline_share,
            Ratio {
                numerator: 460,
                denominator: 1000
            }
        );
        assert_eq!(
            lanes[0].writer_share,
            Ratio {
                numerator: 880,
                denominator: 1000
            }
        );
    }
    #[test]
    fn missing_terminal_fault_and_nested_overcount_cannot_support_routing() {
        let mut raw = words();
        raw[2] = 0;
        assert!(decode(&raw).is_err());
        raw = words();
        raw[21] = 1000;
        assert!(decode(&raw).is_err());
        raw = words();
        raw[3] = 8;
        let lanes = decode(&raw).unwrap();
        assert!(
            decide(
                &[
                    (100, 100, lanes.clone()),
                    (100, 100, lanes.clone()),
                    (100, 100, lanes)
                ],
                true
            )
            .unwrap()
            .unresolved
        );
    }
    #[test]
    fn every_candidate_loss_and_proposal_word_participates_in_parity() {
        let u = ProposalReceipt {
            losses: vec![7; 16],
            status: 0,
            proposal: [0, 7, 7, 0, 0],
        };
        for i in 0..16 {
            let mut d = u.clone();
            d.losses[i] = 8;
            assert!(parity(&u, &d).is_err());
        }
        let mut d = u.clone();
        d.proposal[4] = 1;
        assert!(parity(&u, &d).is_err());
        assert!(parity(&u, &u).is_ok());
    }
    #[test]
    fn resource_amendment_refuses_jit_drift_spills_and_launch_failure() {
        let u = KernelResources {
            registers: 88,
            local_bytes: 21536,
            shared_bytes: 0,
            max_threads_per_block: 1024,
            ptxas: PtxasResources {
                registers: 88,
                stack_bytes: 21536,
                spill_stores: 0,
                spill_loads: 0,
            },
        };
        let mut d = u.clone();
        d.registers = 126;
        d.ptxas.registers = 126;
        d.local_bytes = 22096;
        d.ptxas.stack_bytes = 22096;
        assert!(resource_parity(&u, &d));
        let admitted = d.clone();
        d.registers += 1;
        assert!(!resource_parity(&u, &d));
        d = admitted.clone();
        d.local_bytes += 1;
        assert!(!resource_parity(&u, &d));
        d = admitted.clone();
        d.ptxas.spill_stores = 56;
        d.ptxas.spill_loads = 56;
        assert!(!resource_parity(&u, &d));
        d = admitted.clone();
        d.max_threads_per_block = 15;
        assert!(!resource_parity(&u, &d));
        d = admitted.clone();
        d.shared_bytes = 1;
        assert!(!resource_parity(&u, &d));
        d = admitted;
        d.local_bytes = u.local_bytes + 1024;
        d.ptxas.stack_bytes = d.local_bytes;
        assert!(resource_parity(&u, &d));
        d.local_bytes += 1;
        d.ptxas.stack_bytes += 1;
        assert!(!resource_parity(&u, &d));
        let mut changed_u = u.clone();
        changed_u.ptxas.stack_bytes += 1;
        assert!(!resource_parity(&changed_u, &u));
        changed_u = u.clone();
        changed_u.registers += 1;
        assert!(!resource_parity(&changed_u, &u));
        changed_u = u.clone();
        changed_u.ptxas.spill_loads = 1;
        assert!(!resource_parity(&changed_u, &u));
    }
    #[test]
    fn resource_log_parser_is_entry_scoped_and_refuses_missing_data() {
        let log="ptxas info : Compiling entry function 'other' for 'sm_120'\nptxas info : Function properties for other\n  12 bytes stack frame, 7 bytes spill stores, 8 bytes spill loads\nptxas info : Used 20 registers\nptxas info : Compiling entry function 'discrete_counterfactual' for 'sm_120'\nptxas info : Function properties for discrete_counterfactual\n  21536 bytes stack frame, 0 bytes spill stores, 0 bytes spill loads\nptxas info : Used 32 registers\n";
        assert_eq!(
            parse_ptxas(log, "discrete_counterfactual").unwrap(),
            PtxasResources {
                registers: 32,
                stack_bytes: 21536,
                spill_stores: 0,
                spill_loads: 0
            }
        );
        assert!(parse_ptxas(log, "missing").is_err());
    }
    #[test]
    fn boundary_straddle_and_incremental_perturbation_are_unresolved() {
        let lanes = decode(&words()).unwrap();
        let a = decide(
            &[
                (100, 100, lanes.clone()),
                (100, 100, lanes.clone()),
                (100, 100, lanes.clone()),
            ],
            true,
        )
        .unwrap();
        assert!(!a.unresolved && a.baseline_material && a.writer_dominant);
        assert!(
            decide(
                &[
                    (100, 120, lanes.clone()),
                    (100, 100, lanes.clone()),
                    (100, 100, lanes.clone())
                ],
                true
            )
            .unwrap()
            .unresolved
        );
        let mut changed = lanes.clone();
        changed[0].baseline_share.numerator = 100;
        assert!(
            decide(
                &[
                    (100, 100, lanes.clone()),
                    (100, 100, lanes),
                    (100, 100, changed)
                ],
                true
            )
            .unwrap()
            .unresolved
        );
    }
}
