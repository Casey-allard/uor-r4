//! Finite CPU controls for all 24 kernels in the `uor-r4-float-train` extract (`#2823`).
//!
//! Each test exercises one or a paired forward/backward kernel family from the 24-kernel extract
//! (`22` in `src/cuda_stack.cu` + `2` in `src/spiralcore_fbs.cu`) at finite CPU control level.

use uor_r4_float_train::kernel_controls::{
    adam_update_cpu, hamilton_gate_bwd_cpu, hamilton_gate_fwd_cpu, margin_rank_grad_cpu,
    margin_rank_rows_cpu, poison_words_cpu, quaternion_scan_bwd_cpu, quaternion_scan_fwd_cpu,
    rate_select_bwd_cpu, rate_select_fwd_cpu, recurrence_bwd_params_cpu, recurrence_scan_fwd_cpu,
    recurrence_tile_carry_cpu, recurrence_tile_expand_cpu, recurrence_tile_fold_cpu,
    rms_norm_bwd_dx_cpu, rms_norm_dw_partial_cpu, rms_norm_dw_reduce_cpu, rms_norm_fwd_cpu,
    spiralcore_fbs_bwd_cpu, spiralcore_fbs_fwd_cpu, sq_lanes_cpu, sq_tree_cpu,
    straight_through_fwd_cpu, ScFbsHostTables,
};

#[test]
fn k01_straight_through_fwd_copies_exact_values() {
    let src = [0.25f32, -1.5, 3.75, 0.0, -0.125, 8.0];
    let dst = straight_through_fwd_cpu(&src);
    assert_eq!(dst, src);
}

#[test]
fn k02_k03_hamilton_gate_fwd_and_bwd_preserve_norm_and_match_finite_difference() {
    let gate = [0.6f32, -0.3, 0.5, 0.2, -0.4, 0.7, -0.1, 0.5];
    let up = [1.2f32, -0.5, 0.8, -0.3, 0.4, 0.9, -1.1, 0.2];
    let out = hamilton_gate_fwd_cpu(&gate, &up);
    assert_eq!(out.len(), 8);
    // Unit quaternion rotation preserves Euclidean norm of each 4D cell.
    for cell in 0..2 {
        let base = cell * 4;
        let in_norm: f32 = up[base..base + 4].iter().map(|x| x * x).sum::<f32>().sqrt();
        let out_norm: f32 = out[base..base + 4]
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        assert!(
            (in_norm - out_norm).abs() < 1e-5,
            "cell {cell} norm mismatch: {in_norm} vs {out_norm}"
        );
    }

    let grad = [0.3f32, -0.7, 0.2, 0.5, -0.6, 0.4, 0.1, -0.8];
    let (d_gate, d_up) = hamilton_gate_bwd_cpu(&gate, &up, &grad);
    let eps = 1e-3f32;
    for i in 0..8 {
        let mut g_plus = gate;
        let mut g_minus = gate;
        g_plus[i] += eps;
        g_minus[i] -= eps;
        let out_p = hamilton_gate_fwd_cpu(&g_plus, &up);
        let out_m = hamilton_gate_fwd_cpu(&g_minus, &up);
        let fd_g: f32 = out_p
            .iter()
            .zip(&out_m)
            .zip(&grad)
            .map(|((p, m), d)| (p - m) / (2.0 * eps) * d)
            .sum();
        assert!(
            (d_gate[i] - fd_g).abs() < 2e-3,
            "d_gate[{i}]={} vs fd={fd_g}",
            d_gate[i]
        );

        let mut u_plus = up;
        let mut u_minus = up;
        u_plus[i] += eps;
        u_minus[i] -= eps;
        let out_up = hamilton_gate_fwd_cpu(&gate, &u_plus);
        let out_um = hamilton_gate_fwd_cpu(&gate, &u_minus);
        let fd_u: f32 = out_up
            .iter()
            .zip(&out_um)
            .zip(&grad)
            .map(|((p, m), d)| (p - m) / (2.0 * eps) * d)
            .sum();
        assert!(
            (d_up[i] - fd_u).abs() < 2e-3,
            "d_up[{i}]={} vs fd={fd_u}",
            d_up[i]
        );
    }
}

#[test]
fn k04_k05_rate_select_fwd_and_bwd_dyadic_table_and_secant_credit() {
    let scores = [0.0f32, 0.15, 0.5, 0.99];
    let out = rate_select_fwd_cpu(&scores);
    assert_eq!(out.len(), 8);
    // Index 0 -> lambda = 0.5, keep = sqrt(1 - 0.25) = 0.8660254
    assert!((out[0] - 0.5).abs() < 1e-7);
    assert!((out[1] - 0.8660254).abs() < 1e-6);
    // Unit-circle invariant lambda^2 + keep^2 == 1 for early entries
    for k in 0..3 {
        let lam = out[2 * k];
        let keep = out[2 * k + 1];
        assert!((lam * lam + keep * keep - 1.0).abs() < 1e-5);
    }

    let grad = [1.0f32, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, -1.0];
    let d_score = rate_select_bwd_cpu(&scores, &grad);
    assert_eq!(d_score.len(), 4);
    assert!(d_score.iter().all(|v| v.is_finite()));
    // Increasing score index decreases lambda = 2^-(index+1), so positive d_lambda gives negative d_score
    assert!(d_score[0] < 0.0);
    assert!(d_score[1] < 0.0);
    // Increasing score index increases keep = sqrt(1 - lambda^2), so positive d_keep gives positive d_score
    assert!(d_score[2] > 0.0);
}

#[test]
fn k06_k07_k08_k09_rms_norm_fwd_bwd_dx_dw_partial_and_reduce() {
    let rows = 3usize;
    let width = 4usize;
    let x = [
        1.0f32, -2.0, 0.5, 1.5, // row 0
        -0.5, 0.25, 1.25, -1.0, // row 1
        0.75, 0.5, -0.25, 2.0, // row 2
    ];
    let w = [1.1f32, 0.9, 1.2, 0.8];
    let grad = [
        0.2f32, -0.3, 0.5, -0.1, // row 0
        -0.4, 0.6, 0.1, 0.3, // row 1
        0.3, -0.2, -0.5, 0.4, // row 2
    ];

    let out = rms_norm_fwd_cpu(&x, &w, rows, width);
    assert_eq!(out.len(), rows * width);
    assert!(out.iter().all(|v| v.is_finite()));
    assert!(out.iter().any(|&v| v.abs() > 0.1));

    let (dx, row_r) = rms_norm_bwd_dx_cpu(&x, &w, &grad, rows, width);
    assert_eq!(dx.len(), rows * width);
    assert_eq!(row_r.len(), rows);

    let chunk_rows = 2usize;
    let partials = rms_norm_dw_partial_cpu(&x, &grad, &row_r, rows, width, chunk_rows);
    let chunks = rows.div_ceil(chunk_rows);
    assert_eq!(partials.len(), chunks * width);
    let dw = rms_norm_dw_reduce_cpu(&partials, chunks, width);
    assert_eq!(dw.len(), width);

    // Finite-difference check on dx and dw
    let eps = 1e-3f32;
    for i in 0..x.len() {
        let mut xp = x;
        let mut xm = x;
        xp[i] += eps;
        xm[i] -= eps;
        let op = rms_norm_fwd_cpu(&xp, &w, rows, width);
        let om = rms_norm_fwd_cpu(&xm, &w, rows, width);
        let fd: f32 = op
            .iter()
            .zip(&om)
            .zip(&grad)
            .map(|((p, m), g)| (p - m) / (2.0 * eps) * g)
            .sum();
        assert!((dx[i] - fd).abs() < 2e-3, "dx[{i}]={} vs fd={fd}", dx[i]);
    }
    for col in 0..width {
        let mut wp = w;
        let mut wm = w;
        wp[col] += eps;
        wm[col] -= eps;
        let op = rms_norm_fwd_cpu(&x, &wp, rows, width);
        let om = rms_norm_fwd_cpu(&x, &wm, rows, width);
        let fd: f32 = op
            .iter()
            .zip(&om)
            .zip(&grad)
            .map(|((p, m), g)| (p - m) / (2.0 * eps) * g)
            .sum();
        assert!(
            (dw[col] - fd).abs() < 2e-3,
            "dw[{col}]={} vs fd={fd}",
            dw[col]
        );
    }
}

#[test]
fn k10_k11_quaternion_scan_fwd_and_bwd_adjoint_consistency() {
    let batch = 1usize;
    let time = 3usize;
    let lanes = 1usize;
    let transition = [
        0.8f32, 0.1, -0.2, 0.1, // t=0
        0.7, -0.2, 0.1, 0.3, // t=1
        0.6, 0.3, 0.2, -0.1, // t=2
    ];
    let drive = [
        0.5f32, -0.3, 0.2, 0.4, // t=0
        -0.2, 0.6, -0.1, 0.3, // t=1
        0.4, 0.1, -0.5, 0.2, // t=2
    ];
    let state = quaternion_scan_fwd_cpu(&transition, &drive, batch, time, lanes);
    assert_eq!(state.len(), 12);
    // At t=0, held starts at 0, so state[0..4] == drive[0..4]
    assert_eq!(&state[0..4], &drive[0..4]);

    let grad = [
        0.3f32, -0.2, 0.4, 0.1, -0.5, 0.3, 0.2, -0.4, 0.2, 0.6, -0.3, 0.5,
    ];
    let (dq, db) = quaternion_scan_bwd_cpu(&transition, &state, &grad, batch, time, lanes);
    // At t=0, dq[0..4] is zero because prior state at t=-1 is zero
    assert_eq!(&dq[0..4], &[0.0, 0.0, 0.0, 0.0]);

    let eps = 1e-3f32;
    for i in 0..12 {
        let mut tp = transition;
        let mut tm = transition;
        tp[i] += eps;
        tm[i] -= eps;
        let sp = quaternion_scan_fwd_cpu(&tp, &drive, batch, time, lanes);
        let sm = quaternion_scan_fwd_cpu(&tm, &drive, batch, time, lanes);
        let fd_q: f32 = sp
            .iter()
            .zip(&sm)
            .zip(&grad)
            .map(|((p, m), g)| (p - m) / (2.0 * eps) * g)
            .sum();
        assert!(
            (dq[i] - fd_q).abs() < 2e-3,
            "dq[{i}]={} vs fd={fd_q}",
            dq[i]
        );

        let mut dp = drive;
        let mut dm = drive;
        dp[i] += eps;
        dm[i] -= eps;
        let sdp = quaternion_scan_fwd_cpu(&transition, &dp, batch, time, lanes);
        let sdm = quaternion_scan_fwd_cpu(&transition, &dm, batch, time, lanes);
        let fd_b: f32 = sdp
            .iter()
            .zip(&sdm)
            .zip(&grad)
            .map(|((p, m), g)| (p - m) / (2.0 * eps) * g)
            .sum();
        assert!(
            (db[i] - fd_b).abs() < 2e-3,
            "db[{i}]={} vs fd={fd_b}",
            db[i]
        );
    }
}

#[test]
fn k12_k13_margin_rank_rows_and_grad_hinge_and_zero_sum_subgradient() {
    let vocab = 5usize;
    let rivals = 2usize;
    let logits = [
        1.0f32, 1.5, 0.2, 1.8,
        -0.5, // row 0, target 0 (zt=1.0; top-2 rivals: idx 3 (1.8), idx 1 (1.5))
        3.0, 0.5, 0.2, -1.0, 0.1, // row 1, target 0 (zt=3.0; all gaps 1 + z_r - zt <= 0)
    ];
    let targets = [0u32, 0u32];
    let losses = margin_rank_rows_cpu(&logits, &targets, vocab, rivals);
    assert_eq!(losses.len(), 2);
    // Row 0: gaps are (1 + 1.8 - 1.0) = 1.8 and (1 + 1.5 - 1.0) = 1.5 -> mean = 1.65
    assert!((losses[0] - 1.65).abs() < 1e-6, "row 0 loss={}", losses[0]);
    // Row 1: target exceeds all rivals by > 1.0 -> loss == 0.0
    assert_eq!(losses[1], 0.0);

    let scale = [1.0f64, 1.0f64];
    let grad = margin_rank_grad_cpu(&logits, &targets, &scale, vocab, rivals);
    assert_eq!(grad.len(), 10);
    // Each row's gradient sums to 0.0 (conservation of margin shift)
    let row0_sum: f32 = grad[0..5].iter().sum();
    let row1_sum: f32 = grad[5..10].iter().sum();
    assert!(row0_sum.abs() < 1e-6);
    assert_eq!(row1_sum, 0.0);
    assert!((grad[0] - (-1.0)).abs() < 1e-6);
    assert!((grad[1] - 0.5).abs() < 1e-6);
    assert!((grad[3] - 0.5).abs() < 1e-6);
}

#[test]
fn k14_k15_k16_k17_recurrence_scan_and_tiled_fold_carry_expand_agree() {
    let batch = 1usize;
    let time = 4usize;
    let lanes = 1usize;
    let width = 4usize;
    let q = [
        0.9f32, 0.1, 0.0, 0.0, // t=0
        0.8, 0.0, 0.2, 0.0, // t=1
        0.85, -0.1, 0.0, 0.1, // t=2
        0.75, 0.2, -0.1, 0.0, // t=3
    ];
    let drive = [
        0.4f32, -0.2, 0.1, 0.3, // t=0
        -0.1, 0.5, 0.2, -0.4, // t=1
        0.3, 0.1, -0.3, 0.2, // t=2
        0.2, -0.4, 0.5, 0.1, // t=3
    ];
    let keep = [0.8660254f32, 0.96824586, 0.8660254, 0.99215674];

    let serial = recurrence_scan_fwd_cpu(&q, &drive, &keep, batch, time, width, lanes);
    let tile = 2usize;
    let ntiles = 2usize;
    let (tile_q, tile_d) =
        recurrence_tile_fold_cpu(&q, &drive, &keep, batch, time, width, lanes, ntiles, tile);
    let entry = recurrence_tile_carry_cpu(&tile_q, &tile_d, batch, lanes, ntiles);
    let tiled = recurrence_tile_expand_cpu(
        &q, &drive, &keep, &entry, batch, time, width, lanes, ntiles, tile,
    );
    assert_eq!(serial.len(), tiled.len());
    for (i, (&s, &t)) in serial.iter().zip(&tiled).enumerate() {
        assert!(
            (s - t).abs() < 1e-5,
            "serial[{i}]={s} vs tiled[{i}]={t} diff={}",
            (s - t).abs()
        );
    }
}

#[test]
fn k18_recurrence_bwd_params_accumulates_causal_taps_and_bias() {
    let batch = 1usize;
    let time = 4usize;
    let width = 4usize;
    let lanes = 1usize;
    let branches: Vec<f32> = (0..(batch * time * 2 * width))
        .map(|i| ((i as f32) * 0.1) - 1.0)
        .collect();
    let params: Vec<f32> = (0..(4 * width)).map(|i| 0.05 * (i as f32 + 1.0)).collect();
    let dd: Vec<f32> = (0..(batch * time * width))
        .map(|i| 0.2 * ((i as f32) - 4.0))
        .collect();
    let log_term = [0.1f32, -0.2, 0.3, -0.05];
    let (d_branches, partials) = recurrence_bwd_params_cpu(
        &branches, &params, &dd, &log_term, batch, time, width, lanes,
    );
    assert_eq!(d_branches.len(), batch * time * 2 * width);
    assert_eq!(partials.len(), batch * (5 * width + lanes));
    assert!(d_branches.iter().all(|v| v.is_finite()));
    assert!(partials.iter().all(|v| v.is_finite()));
    // Check d_log_a sum at p[5 * width + 0]
    let expected_log_a: f64 = log_term.iter().rev().map(|&x| x as f64).sum();
    assert!((partials[5 * width] - expected_log_a).abs() < 1e-9);
}

#[test]
fn k19_poison_words_writes_all_ones_bit_pattern() {
    let buf = poison_words_cpu(8);
    assert_eq!(buf, vec![0xFFFF_FFFFu32; 8]);
    assert!(f32::from_bits(buf[0]).is_nan());
}

#[test]
fn k20_adam_update_moves_parameters_opposite_gradient() {
    let mut p = [1.0f32, -2.0, 0.5, 0.0];
    let mut m = [0.0f32; 4];
    let mut v = [0.0f32; 4];
    let g = [0.4f32, -0.6, 0.2, -0.1];
    // c = [grad_scale, beta1, 1-beta1, beta2, 1-beta2, step_num_scale, inv_bc2, eps, wd_keep, lr]
    let c = [
        1.0f32,
        0.9,
        0.1,
        0.999,
        0.001,
        1.0 / 0.1,
        1.0 / 0.001,
        1e-8,
        0.999,
        0.01,
    ];
    let p_before = p;
    adam_update_cpu(&mut p, &mut m, &mut v, &g, &c);
    for i in 0..4 {
        assert!(p[i].is_finite() && m[i].is_finite() && v[i].is_finite());
        assert!(v[i] > 0.0);
        if g[i] > 0.0 {
            assert!(p[i] < p_before[i]);
        } else {
            assert!(p[i] > p_before[i]);
        }
    }
}

#[test]
fn k21_k22_sq_lanes_and_sq_tree_reproduce_exact_squared_norm() {
    let x: Vec<f32> = (1..=20).map(|i| (i as f32) * 0.25).collect();
    let width = 8usize;
    let lanes = sq_lanes_cpu(&x, width, true);
    assert_eq!(lanes.len(), width);
    let segments = [0u32, width as u32];
    let tree_out = sq_tree_cpu(&lanes, &segments);
    assert_eq!(tree_out.len(), 1);
    let expected: f32 = x.iter().map(|v| v * v).sum();
    assert!((tree_out[0] - expected).abs() < 1e-5);
}

#[test]
fn k23_k24_spiralcore_fbs_fwd_and_bwd_signed_action_and_secant_pullback() {
    let tables = ScFbsHostTables::fixture_two_leaf_tree();
    let rows = 1usize;
    let outputs = 1usize;
    let inputs = 2usize;
    // Query close to root 0 selects leaf 0
    let mut queries = vec![0.0f32; tables.stages * 8];
    queries[0] = 0.9;
    let input: Vec<f32> = (0..(rows * inputs * 8))
        .map(|i| (i as f32 + 1.0) * 0.125)
        .collect();
    let (out, err_fwd) = spiralcore_fbs_fwd_cpu(&queries, &input, &tables, rows, outputs, inputs);
    assert_eq!(err_fwd, 0);
    assert_eq!(out.len(), rows * outputs * 8);
    assert!(out.iter().all(|v| v.is_finite()));

    let grad = [0.5f32, -0.25, 0.75, -0.5, 0.125, 0.375, -0.625, 0.25];
    let (d_queries, d_input, err_bwd) =
        spiralcore_fbs_bwd_cpu(&queries, &input, &grad, &tables, rows, outputs, inputs);
    assert_eq!(err_bwd, 0);
    assert_eq!(d_queries.len(), queries.len());
    assert_eq!(d_input.len(), input.len());
    assert!(d_queries.iter().all(|v| v.is_finite()));
    assert!(d_input.iter().all(|v| v.is_finite()));
    // Input adjoint is non-zero on selected atom 0 and zero on unselected atom 1
    assert!(d_input[0..8].iter().any(|&v| v != 0.0));
    assert!(d_input[8..16].iter().all(|&v| v == 0.0));
    // Secant query pullback is non-zero
    assert!(d_queries.iter().any(|&v| v != 0.0));
}
