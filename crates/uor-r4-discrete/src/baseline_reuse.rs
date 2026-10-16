//! Exact audit and AOT image metadata for the opt-in baseline reuse experiment.
//! These parsers validate observations; they never implement a model reference.
use crate::decomposition::KernelResources;
use serde::{Deserialize, Serialize};

pub const ROW_CAPACITY: usize = 64;
pub const LANES: usize = 16;
pub const LANE_WORDS: usize = 6;
pub const WORDS: usize = ROW_CAPACITY + LANES * LANE_WORDS;
pub const AUDIT_BYTES: usize = WORDS * 8;
pub const SHARED_BYTES: u64 = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditError(pub String);
impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "baseline reuse audit: {}", self.0)
    }
}
impl std::error::Error for AuditError {}
impl From<AuditError> for String {
    fn from(error: AuditError) -> Self {
        error.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineReuseImage {
    pub schema: String,
    pub format: String,
    pub target: String,
    pub source_sha256: String,
    pub ptx_sha256: String,
    pub cubin_sha256: String,
    pub report_sha256: String,
}
impl BaselineReuseImage {
    pub fn validate(&self) -> Result<(), String> {
        let target = self.target.strip_prefix("sm_");
        if self.schema != "uor-r4.baseline-reuse-image/1"
            || self.format != "cubin"
            || !target.is_some_and(|digits| {
                !digits.is_empty()
                    && digits.bytes().all(|b| b.is_ascii_digit())
                    && digits
                        .parse::<u32>()
                        .is_ok_and(|n| n > 0 && n.to_string() == digits)
            })
        {
            return Err(
                "baseline reuse image schema/format/target is not an exact AOT binding".into(),
            );
        }
        for hash in [
            &self.source_sha256,
            &self.ptx_sha256,
            &self.cubin_sha256,
            &self.report_sha256,
        ] {
            if hash.len() != 64
                || !hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || !hash.bytes().any(|b| b != b'0')
            {
                return Err(
                    "baseline reuse image requires nonzero lowercase SHA256 identities".into(),
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RowRecord {
    pub status: u32,
    pub eligible: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LaneRecord {
    pub candidate: usize,
    pub producer_mask: u64,
    pub producer_call_count: u64,
    pub consumer_visited_mask: u64,
    pub consumer_eligible_mask: u64,
    pub virtual_visited_mask: u64,
    pub terminal_status: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Audit {
    pub rows: Vec<RowRecord>,
    pub lanes: Vec<LaneRecord>,
}

pub fn row_mask(rows: usize) -> Result<u64, AuditError> {
    match rows {
        1..=63 => Ok((1u64 << rows) - 1),
        64 => Ok(u64::MAX),
        _ => Err(AuditError("row count must be 1..64".into())),
    }
}

/// First64 words pack status in low32 and eligible in high32. The following
/// 16*6 words are producer mask/count, consumer visited/eligible, virtual
/// visited and terminal status. Unused row slots must have been reset to zero.
pub fn decode(raw: &[u64], rows: usize) -> Result<Audit, AuditError> {
    decode_words(raw, rows).map_err(AuditError)
}
fn decode_words(raw: &[u64], count: usize) -> Result<Audit, String> {
    let full = row_mask(count).map_err(|e| e.to_string())?;
    if raw.len() != WORDS || raw[count..ROW_CAPACITY].iter().any(|&word| word != 0) {
        return Err("inventory must be 160 words with zero unused rows".into());
    }
    let mut eligible_mask = 0u64;
    let mut fault_mask = 0u64;
    let mut rows = Vec::with_capacity(count);
    for (row, &word) in raw[..count].iter().enumerate() {
        let status = word as u32;
        let eligible = word >> 32;
        if !matches!(status, 0 | 2 | 8 | 12) || eligible > 1 || (status != 0 && eligible != 0) {
            return Err(format!("invalid producer status/eligibility at row {row}"));
        }
        if eligible == 1 {
            eligible_mask |= 1 << row;
        }
        if status != 0 {
            fault_mask |= 1 << row;
        }
        rows.push(RowRecord {
            status,
            eligible: eligible != 0,
        });
    }
    let mut union = 0u64;
    let mut total_calls = 0u64;
    let mut lanes = Vec::with_capacity(LANES);
    for (candidate, words) in raw[ROW_CAPACITY..].chunks_exact(LANE_WORDS).enumerate() {
        let [producer, calls, visited, eligible, virtual_mask, terminal]: [u64; 6] =
            words.try_into().map_err(|_| "incomplete lane words")?;
        let expected_producer = (candidate..count)
            .step_by(LANES)
            .fold(0u64, |mask, row| mask | (1 << row));
        if producer != expected_producer
            || producer & !full != 0
            || producer & union != 0
            || calls != u64::from(producer.count_ones())
        {
            return Err(format!(
                "producer partition/count mismatch in lane {candidate}"
            ));
        }
        union |= producer;
        total_calls += calls;
        // Consumers always visit an increasing prefix. wrapping_add handles
        // the explicit full64 case without shifting by64 or overflowing.
        if visited == 0
            || visited & !full != 0
            || visited & visited.wrapping_add(1) != 0
            || eligible != visited & eligible_mask
            || virtual_mask != eligible
            || !matches!(terminal, 0 | 2 | 8 | 12)
        {
            return Err(format!(
                "consumer masks/status mismatch in lane {candidate}"
            ));
        }
        let reached_faults = visited & fault_mask;
        if reached_faults != 0 {
            let last = 63usize - visited.leading_zeros() as usize;
            if reached_faults != 1u64 << last || u64::from(rows[last].status) != terminal {
                return Err(format!(
                    "producer fault was not deferred to its reached row in lane {candidate}"
                ));
            }
        } else if terminal == 8 && (eligible != 0 || visited != full) {
            return Err(format!(
                "all-ineligible refusal is inconsistent in lane {candidate}"
            ));
        } else if terminal == 0 && eligible == 0 {
            return Err(format!("successful lane {candidate} has no eligible row"));
        }
        lanes.push(LaneRecord {
            candidate,
            producer_mask: producer,
            producer_call_count: calls,
            consumer_visited_mask: visited,
            consumer_eligible_mask: eligible,
            virtual_visited_mask: virtual_mask,
            terminal_status: terminal as u32,
        });
    }
    if union != full || total_calls != count as u64 {
        return Err("producer coverage does not equal admitted rows".into());
    }
    Ok(Audit { rows, lanes })
}

/// New U/R contract: shared0/512, own-AOT equality, zero spills and block16.
/// Registers/local/stack deltas are observations, not inherited U/D ceilings.
pub fn resource_parity(u: &KernelResources, r: &KernelResources) -> bool {
    let consistent = |k: &KernelResources| {
        k.registers == k.ptxas.registers
            && k.local_bytes == k.ptxas.stack_bytes
            && k.max_threads_per_block >= LANES as u64
            && k.ptxas.spill_loads == 0
            && k.ptxas.spill_stores == 0
    };
    consistent(u) && consistent(r) && u.shared_bytes == 0 && r.shared_bytes == SHARED_BYTES
}

/// Extract static shared bytes from the selected entry's own ptxas Used line.
/// An otherwise complete Used line without smem denotes zero static shared.
pub fn report_shared_bytes(report: &str, entry: &str) -> Result<u64, String> {
    let mut active = false;
    let mut found = None;
    for line in report.lines() {
        if line.contains("Compiling entry function") || line.contains("Function properties for") {
            active = line.contains(&format!("'{entry}'"))
                || line.trim_end().ends_with(&format!("for {entry}"));
        }
        if active && line.contains("Used ") && line.contains(" registers") {
            let shared = if let Some((prefix, _)) = line.split_once(" bytes smem") {
                prefix
                    .split(|c: char| !c.is_ascii_digit())
                    .filter(|s| !s.is_empty())
                    .next_back()
                    .ok_or("missing static shared byte count")?
                    .parse::<u64>()
                    .map_err(|_| "invalid static shared byte count")?
            } else {
                0
            };
            if found.replace(shared).is_some() {
                return Err("duplicate ptxas Used line".into());
            }
        }
    }
    found.ok_or_else(|| "missing selected ptxas Used line".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn literal_audit(rows: usize) -> Vec<u64> {
        let full = row_mask(rows).expect("test row bound");
        let mut raw = vec![0; WORDS];
        raw[..rows].fill(1u64 << 32);
        for lane in 0..LANES {
            let offset = ROW_CAPACITY + lane * LANE_WORDS;
            let producer = (lane..rows)
                .step_by(LANES)
                .fold(0u64, |mask, row| mask | (1 << row));
            raw[offset..offset + LANE_WORDS].copy_from_slice(&[
                producer,
                u64::from(producer.count_ones()),
                full,
                full,
                full,
                0,
            ]);
        }
        raw
    }
    #[test]
    fn metadata_partition_bounds_and_fault_deferral() {
        for rows in [1, 2, 16, 17, 64] {
            let mut raw = literal_audit(rows);
            assert!(decode(&raw, rows).is_ok());
            raw[ROW_CAPACITY + 1] += 1;
            assert!(decode(&raw, rows).is_err());
        }
        let mut raw = literal_audit(2);
        raw[1] = 12;
        for lane in 0..LANES {
            let offset = ROW_CAPACITY + lane * LANE_WORDS;
            raw[offset + 2..offset + 6].copy_from_slice(&[1, 1, 1, 0]);
        }
        assert!(decode(&raw, 2).is_ok()); // speculative row2 is not reached
        raw[ROW_CAPACITY + 5] = 12;
        // A virtual arithmetic failure is separately legal, so explicit reached
        // row mismatch is the discriminating metadata failure here.
        raw[ROW_CAPACITY + 2] = 3;
        raw[ROW_CAPACITY + 5] = 2;
        assert!(decode(&raw, 2).is_err());
        assert_eq!(row_mask(64).expect("full mask"), u64::MAX);
        assert!(row_mask(0).is_err());
        assert!(row_mask(65).is_err());
    }
    #[test]
    fn image_and_report_metadata_refuse_stale_or_incomplete_bindings() {
        let mut image = BaselineReuseImage {
            schema: "uor-r4.baseline-reuse-image/1".into(),
            format: "cubin".into(),
            target: "sm_120".into(),
            source_sha256: "1".repeat(64),
            ptx_sha256: "2".repeat(64),
            cubin_sha256: "3".repeat(64),
            report_sha256: "4".repeat(64),
        };
        assert!(image.validate().is_ok());
        image.target = "sm_0120".into();
        assert!(image.validate().is_err());
        let report = "ptxas info : Function properties for reuse\n0 bytes stack frame, 0 bytes spill stores, 0 bytes spill loads\nptxas info : Used 88 registers, 512 bytes smem, 392 bytes cmem[0]\n";
        assert_eq!(report_shared_bytes(report, "reuse"), Ok(512));
        assert!(report_shared_bytes(report, "other").is_err());
    }
}
