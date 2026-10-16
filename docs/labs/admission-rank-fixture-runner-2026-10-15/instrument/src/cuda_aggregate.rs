//! GPU kernel-level verification of `discrete_evaluation_aggregate` status-12
//! bounds and `u64` overflow checks (`C-R5` Fixture 5B, `objective.rs:110-118`).

use crate::fixtures::sha256_hex;
use cudarc::driver::{CudaContext, LaunchConfig, Ptx, PushKernelArg};
use serde::Serialize;
use uor_r4_discrete::objective::{AGGREGATE_WORDS, ROW_WORDS};

const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/discrete.ptx"));

#[derive(Debug, Clone, Serialize)]
pub struct Status12CaseReceipt {
    pub name: String,
    pub summaries_mode: u32,
    pub count: u32,
    pub width: u32,
    pub observed_status: u32,
    pub expected_status: u32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CudaAggregateOverflowReport {
    pub ptx_sha256: String,
    pub matches_device_compiled_kernel_sha256: bool,
    pub baseline_valid_row_status: u32,
    pub cases: Vec<Status12CaseReceipt>,
    pub all_passed: bool,
}

fn valid_baseline_row() -> [u64; ROW_WORDS] {
    // width = 32: T = 10, A = 4, R = 2, tr = 3, ar = 2, top1 = 0, hit8 = 1,
    // admitted_cells = 2, target_cell_rank = 4, occupied_cells = 5,
    // admitted_token_count = 3, loss_ta = 8*10 + 32*4 = 208,
    // loss_tr = 8*10 + 32*2 = 144, objective_loss = 144.
    [10, 4, 208, 3, 2, 0, 1, 2, 2, 4, 5, 3, 144, 144]
}

fn launch_aggregate(
    words: &[u64],
    count: u32,
    width: u32,
    summaries: u32,
) -> Result<(u32, Vec<u64>), String> {
    let context = CudaContext::new(0).map_err(|e| format!("CUDA context: {e}"))?;
    let stream = context.default_stream();
    let module = context
        .load_module(Ptx::from_src(PTX))
        .map_err(|e| format!("load PTX: {e}"))?;
    let func = module
        .load_function("discrete_evaluation_aggregate")
        .map_err(|e| format!("load discrete_evaluation_aggregate: {e}"))?;
    let metrics = stream
        .clone_htod(words)
        .map_err(|e| format!("htod metrics: {e}"))?;
    let mut out = stream
        .alloc_zeros::<u64>(AGGREGATE_WORDS)
        .map_err(|e| format!("alloc out: {e}"))?;
    let mut status = stream
        .alloc_zeros::<u32>(1)
        .map_err(|e| format!("alloc status: {e}"))?;
    let mut launch = stream.launch_builder(&func);
    launch
        .arg(&metrics)
        .arg(&count)
        .arg(&width)
        .arg(&summaries)
        .arg(&mut out)
        .arg(&mut status);
    let cfg = LaunchConfig {
        grid_dim: (1, 1, 1),
        block_dim: (1, 1, 1),
        shared_mem_bytes: 0,
    };
    // SAFETY: exact discrete_evaluation_aggregate ABI `(const W*, U, U, U, W*, U*)`,
    // single-thread launch `(1,1,1)`, and checked device buffer lengths.
    unsafe { launch.launch(cfg) }.map_err(|e| format!("launch aggregate: {e}"))?;
    let code = stream
        .clone_dtoh(&status)
        .map_err(|e| format!("dtoh status: {e}"))?
        .first()
        .copied()
        .ok_or_else(|| "missing status word".to_string())?;
    let out_words = stream
        .clone_dtoh(&out)
        .map_err(|e| format!("dtoh out: {e}"))?;
    Ok((code, out_words))
}

pub fn run_status12_overflow_checks() -> Result<CudaAggregateOverflowReport, String> {
    let ptx_sha256 = sha256_hex(PTX.as_bytes());
    let compiled = uor_r4_discrete::device::compiled_identity();
    if ptx_sha256 != compiled.kernel_sha256 {
        return Err(format!(
            "PTX SHA-256 mismatch between runner ({ptx_sha256}) and uor-r4-discrete ({})",
            compiled.kernel_sha256
        ));
    }

    let base_row = valid_baseline_row();
    let (base_status, base_summary) = launch_aggregate(&base_row, 1, 32, 0)?;
    if base_status != 0 {
        return Err(format!(
            "valid baseline row unexpectedly returned CUDA status {base_status}"
        ));
    }

    let mut cases = Vec::new();
    let mut check_case =
        |name: &str, words: &[u64], count: u32, width: u32, summaries: u32| -> Result<(), String> {
            let (observed, _) = launch_aggregate(words, count, width, summaries)?;
            let passed = observed == 12;
            if !passed {
                return Err(format!(
                    "C-R5 status-12 case '{name}' returned status {observed} (expected 12)"
                ));
            }
            cases.push(Status12CaseReceipt {
                name: name.into(),
                summaries_mode: summaries,
                count,
                width,
                observed_status: observed,
                expected_status: 12,
                passed,
            });
            Ok(())
        };

    // 1. T = 641 > 640 (raw_row_bounds.T)
    let mut r_t = base_row;
    r_t[0] = 641;
    r_t[2] = 8 * 641 + 32 * r_t[1];
    r_t[12] = 8 * 641 + 32 * r_t[8];
    r_t[13] = r_t[12];
    check_case("raw_row_bounds_T_gt_640", &r_t, 1, 32, 0)?;

    // 2. A = 73 > 72 (raw_row_bounds.A)
    let mut r_a = base_row;
    r_a[1] = 73;
    r_a[2] = 8 * r_a[0] + 32 * 73;
    check_case("raw_row_bounds_A_gt_72", &r_a, 1, 32, 0)?;

    // 3. R = 256 > 255 (raw_row_bounds.R)
    let mut r_r = base_row;
    r_r[8] = 256;
    check_case("raw_row_bounds_R_gt_255", &r_r, 1, 32, 0)?;

    // 4. target_cell_rank = 0 and 257
    let mut r_rank0 = base_row;
    r_rank0[9] = 0;
    check_case("target_cell_rank_zero", &r_rank0, 1, 32, 0)?;
    let mut r_rank257 = base_row;
    r_rank257[9] = 257;
    check_case("target_cell_rank_gt_256", &r_rank257, 1, 32, 0)?;

    // 5. occupied_cells = 0 and 257
    let mut r_occ0 = base_row;
    r_occ0[10] = 0;
    check_case("occupied_cells_zero", &r_occ0, 1, 32, 0)?;
    let mut r_occ257 = base_row;
    r_occ257[10] = 257;
    check_case("occupied_cells_gt_256", &r_occ257, 1, 32, 0)?;

    // 6. admitted_cells = 0 and 17
    let mut r_adm0 = base_row;
    r_adm0[7] = 0;
    check_case("admitted_cells_zero", &r_adm0, 1, 32, 0)?;
    let mut r_adm17 = base_row;
    r_adm17[7] = 17;
    check_case("admitted_cells_gt_16", &r_adm17, 1, 32, 0)?;

    // 7. loss_ta / loss_tr formula mismatch
    let mut r_lta = base_row;
    r_lta[2] += 1;
    check_case("loss_ta_formula_mismatch", &r_lta, 1, 32, 0)?;
    let mut r_ltr = base_row;
    r_ltr[12] += 1;
    r_ltr[13] = r_ltr[12];
    check_case("loss_tr_formula_mismatch", &r_ltr, 1, 32, 0)?;

    // 8. cross-row occupied_cells mismatch
    let mut two_rows = Vec::with_capacity(ROW_WORDS * 2);
    two_rows.extend_from_slice(&base_row);
    let mut second_row = base_row;
    second_row[10] = 6;
    two_rows.extend_from_slice(&second_row);
    check_case("cross_row_occupied_cells_mismatch", &two_rows, 2, 32, 0)?;

    // 9. "u64::MAX plus 1" (objective.rs:116)
    let mut sum_u64_max = Vec::with_capacity(AGGREGATE_WORDS * 2);
    sum_u64_max.extend_from_slice(&base_summary);
    let mut overflow_summary = base_summary.clone();
    overflow_summary[5] = u64::MAX; // total_loss_numerator_sum: base_summary[5] + u64::MAX overflows
    sum_u64_max.extend_from_slice(&overflow_summary);
    check_case("u64::MAX plus 1", &sum_u64_max, 2, 32, 1)?;

    // 10. "weighted R overflow" (objective.rs:116)
    let mut weighted_r_overflow = Vec::with_capacity(AGGREGATE_WORDS * 2);
    let mut wr_a = base_summary.clone();
    let mut wr_b = base_summary.clone();
    wr_a[20] = (u64::MAX / 2) + 16; // weighted_cell_rank_hinge_sum
    wr_b[20] = (u64::MAX / 2) + 16;
    weighted_r_overflow.extend_from_slice(&wr_a);
    weighted_r_overflow.extend_from_slice(&wr_b);
    check_case("weighted R overflow", &weighted_r_overflow, 2, 32, 1)?;

    // 11. "quantile threshold overflow" (objective.rs:116)
    let mut q_overflow = vec![0u64; AGGREGATE_WORDS];
    let huge_rows = (u64::MAX / 50) + 10;
    q_overflow[0] = huge_rows;
    q_overflow[16] = 32 * 8;
    q_overflow[23] = 5;
    q_overflow[29] = huge_rows;
    q_overflow[670] = huge_rows;
    q_overflow[743] = huge_rows;
    check_case("quantile threshold overflow", &q_overflow, 1, 32, 1)?;

    Ok(CudaAggregateOverflowReport {
        ptx_sha256,
        matches_device_compiled_kernel_sha256: true,
        baseline_valid_row_status: base_status,
        cases,
        all_passed: true,
    })
}
