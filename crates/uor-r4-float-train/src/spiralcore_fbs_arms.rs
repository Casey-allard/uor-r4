//! The four registered arms of `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`
//! (spiralcore-fbs-offline-float-sheet, #2891, claude-119), as a thread-local hook on the stack's
//! SpiralCore/FBS projections. `StackModel::linear` reads the hook; nothing changes while it is
//! unset. The pinned `spiralcore_fbs.cu`, `spiralcore_fbs.rs` and `cuda_stack.cu` are untouched:
//! the identity-transport and two-path kernels live in `spiralcore_fbs_arms.cu`, built as their
//! own ahead-of-time PTX module (`build.rs`).
//!
//! - `Fbs_1Atom_Active`: the standard projection, `y_o = A_o x_sigma(o)`.
//! - `Fbs_2Atom_Mix`: two independent routed query paths per output block,
//!   `y_o = (A_1 x_sigma1(o) + A_2 x_sigma2(o)) / sqrt(2)`; its queries are `[outputs, 2 stages, 8]`
//!   (`StackModel::widen_fbs_queries_two_path`).
//! - `Fbs_IdentityTransport_Control`: the same tree selection, transport fixed to the identity,
//!   `y_o = x_sigma(o)`.
//! - `Fbs_FrozenQueries_Null`: the standard forward and input adjoint with zero query credit; the
//!   runner also keeps these queries out of both optimizer updates, so they stay at seed `s`.

use std::cell::Cell;
use std::sync::Arc;

use candle_core::{CpuStorage, CustomOp2, DType, Layout, Shape, Tensor};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::spiralcore_fbs::ProjectionPlan;
use crate::{invalid, Result};

/// One registered sheet arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FbsArm {
    OneAtomActive,
    TwoAtomMix,
    IdentityTransport,
    FrozenQueries,
}

impl FbsArm {
    pub const ALL: [FbsArm; 4] = [
        FbsArm::OneAtomActive,
        FbsArm::TwoAtomMix,
        FbsArm::IdentityTransport,
        FbsArm::FrozenQueries,
    ];

    /// The arm's name in the registered sheet.
    pub fn name(self) -> &'static str {
        match self {
            FbsArm::OneAtomActive => "Fbs_1Atom_Active",
            FbsArm::TwoAtomMix => "Fbs_2Atom_Mix",
            FbsArm::IdentityTransport => "Fbs_IdentityTransport_Control",
            FbsArm::FrozenQueries => "Fbs_FrozenQueries_Null",
        }
    }

    /// Routed query paths per output block.
    pub fn paths(self) -> usize {
        if self == FbsArm::TwoAtomMix {
            2
        } else {
            1
        }
    }

    /// Whether the arm's queries receive credit and optimizer updates.
    pub fn trains_queries(self) -> bool {
        self != FbsArm::FrozenQueries
    }

    /// The `arm` code of `spiralcore_fbs_arm_fwd`/`_bwd`, or `None` for an arm that uses the
    /// pinned kernels.
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    fn kernel_code(self) -> Option<u32> {
        match self {
            FbsArm::IdentityTransport => Some(1),
            FbsArm::TwoAtomMix => Some(2),
            FbsArm::OneAtomActive | FbsArm::FrozenQueries => None,
        }
    }
}

thread_local! {
    static ARM: Cell<Option<FbsArm>> = const { Cell::new(None) };
}

/// Sets (or clears) the arm every SpiralCore/FBS projection on this thread runs.
pub fn set_fbs_arm(arm: Option<FbsArm>) {
    ARM.with(|cell| cell.set(arm));
}

/// The arm set on this thread, if any.
pub fn fbs_arm() -> Option<FbsArm> {
    ARM.with(Cell::get)
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// sha256 of the arm kernel source (`spiralcore_fbs_arms.cu`), for the run receipt.
pub fn arms_kernel_sha256() -> String {
    sha256_hex(include_bytes!("spiralcore_fbs_arms.cu"))
}

/// sha256 of the arm module's prelude (`spiralcore_fbs_arms_prelude.cu`).
pub fn arms_prelude_sha256() -> String {
    sha256_hex(include_bytes!("spiralcore_fbs_arms_prelude.cu"))
}

/// sha256 of the arm module's admitted ahead-of-time PTX.
#[cfg(feature = "cuda")]
pub fn arms_ptx_sha256() -> String {
    sha256_hex(crate::geometric_stack::cuda_ops::SPIRALCORE_FBS_ARMS_PTX.as_bytes())
}

/// Runs one SpiralCore/FBS projection under `arm`. `Fbs_1Atom_Active` is the pinned
/// `ProjectionPlan::forward` unchanged.
pub fn project(
    plan: &Arc<ProjectionPlan>,
    queries: &Tensor,
    input: &Tensor,
    arm: FbsArm,
) -> Result<Tensor> {
    if arm == FbsArm::OneAtomActive {
        return plan.forward(queries, input);
    }
    if queries.device().is_cpu() || input.device().is_cpu() {
        return Err(invalid(
            "SpiralCore/FBS sheet arms are GPU-only; CPU execution is forbidden",
        ));
    }
    if queries.dtype() != DType::F32 || input.dtype() != DType::F32 {
        return Err(invalid("sheet arms run on F32 queries and F32 carriers"));
    }
    let [outputs, stages, eight] = plan.query_shape();
    if queries.dims() != [outputs, stages * arm.paths(), eight]
        || input.dims().last() != Some(&(8 * plan.input_blocks))
    {
        return Err(invalid(
            "sheet arm tensors disagree with the bound projection plan",
        ));
    }
    if input.elem_count() == 0 {
        return Err(invalid("empty SpiralCore/FBS carrier batch"));
    }
    Ok(queries.contiguous()?.apply_op2(
        &input.contiguous()?,
        ArmProjection {
            plan: Arc::clone(plan),
            arm,
        },
    )?)
}

#[cfg_attr(not(feature = "cuda"), allow(dead_code))]
struct ArmProjection {
    plan: Arc<ProjectionPlan>,
    arm: FbsArm,
}

impl CustomOp2 for ArmProjection {
    fn name(&self) -> &'static str {
        "spiralcore-fbs-sheet-arm-projection"
    }
    fn cpu_fwd(
        &self,
        _: &CpuStorage,
        _: &Layout,
        _: &CpuStorage,
        _: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        candle_core::bail!("CPU model projection is forbidden; no host fallback")
    }
    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        q: &candle_core::CudaStorage,
        ql: &Layout,
        x: &candle_core::CudaStorage,
        xl: &Layout,
    ) -> candle_core::Result<(candle_core::CudaStorage, Shape)> {
        use crate::geometric_stack::cuda_ops;
        match self.arm.kernel_code() {
            Some(code) => cuda_ops::spiralcore_fbs_arm_fwd(&self.plan, q, ql, x, xl, code),
            None => cuda_ops::spiralcore_fbs_fwd(&self.plan, q, ql, x, xl),
        }
    }
    fn bwd(
        &self,
        q: &Tensor,
        x: &Tensor,
        _: &Tensor,
        dy: &Tensor,
    ) -> candle_core::Result<(Option<Tensor>, Option<Tensor>)> {
        #[cfg(feature = "cuda")]
        if let candle_core::Device::Cuda(device) = q.device() {
            use crate::geometric_stack::cuda_ops;
            let (dq, dx) = match self.arm.kernel_code() {
                Some(code) => cuda_ops::spiralcore_fbs_arm_bwd(&self.plan, device, q, x, dy, code)?,
                None => cuda_ops::spiralcore_fbs_bwd(&self.plan, device, q, x, dy)?,
            };
            // The frozen-query arm keeps the exact input adjoint and gives its queries no credit.
            let dq = if self.arm.trains_queries() {
                dq
            } else {
                dq.zeros_like()?
            };
            return Ok((Some(dq), Some(dx)));
        }
        let _ = (q, x, dy, &self.plan);
        candle_core::bail!("SpiralCore/FBS sheet arm backward is GPU-only; no host adjoint")
    }
}

/// Finite GPU tests of the arm kernels, run on the pod (`--features cuda -- --ignored`). Each pins
/// an exact property that the arm would break if it were set aside.
#[cfg(all(test, feature = "cuda"))]
mod gpu_tests {
    use super::*;
    use candle_core::{Device, Var};

    const INPUT_ATOMS: usize = 32;
    const OUTPUT_ATOMS: usize = 48;
    const ROWS: usize = 6;

    fn device() -> Device {
        Device::new_cuda(0).expect("a CUDA device")
    }

    /// Deterministic values in [-1, 1) (fixture data only).
    fn values(seed: u64, n: usize) -> Vec<f32> {
        let mut s = seed | 1;
        (0..n)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                ((s >> 40) as f32 / (1u64 << 23) as f32) - 1.0
            })
            .collect()
    }

    fn setup(paths: usize, seed: u64) -> (Arc<ProjectionPlan>, Var, Tensor, Device) {
        let device = device();
        let plan = ProjectionPlan::new(INPUT_ATOMS * 8, OUTPUT_ATOMS * 8).expect("plan");
        let [outputs, stages, eight] = plan.query_shape();
        let q = Tensor::from_vec(
            values(seed, outputs * paths * stages * eight),
            (outputs, paths * stages, eight),
            &device,
        )
        .expect("queries");
        let x = Tensor::from_vec(
            values(seed ^ 0xabcd, ROWS * INPUT_ATOMS * 8),
            (ROWS, INPUT_ATOMS * 8),
            &device,
        )
        .expect("carriers");
        (plan, Var::from_tensor(&q).expect("var"), x, device)
    }

    /// Runs one projection and its backward against a fixed `dy`.
    fn run(
        plan: &Arc<ProjectionPlan>,
        q: &Var,
        x: &Tensor,
        arm: FbsArm,
        dy: &Tensor,
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let xv = Var::from_tensor(x).expect("x var");
        let y = project(plan, q.as_tensor(), xv.as_tensor(), arm).expect("forward");
        let loss = (&y * dy).expect("pair").sum_all().expect("sum");
        let grads = loss.backward().expect("backward");
        let flat = |t: &Tensor| t.flatten_all().unwrap().to_vec1::<f32>().unwrap();
        let dq = grads.get(q.as_tensor()).map(flat).unwrap_or_default();
        let dx = grads.get(xv.as_tensor()).map(flat).unwrap_or_default();
        (flat(&y), dq, dx)
    }

    fn dy_for(device: &Device, seed: u64) -> Tensor {
        Tensor::from_vec(
            values(seed, ROWS * OUTPUT_ATOMS * 8),
            (ROWS, OUTPUT_ATOMS * 8),
            device,
        )
        .expect("dy")
    }

    fn close(a: &[f32], b: &[f32], what: &str) {
        assert_eq!(a.len(), b.len(), "{what}: lengths");
        for (i, (p, q)) in a.iter().zip(b).enumerate() {
            assert!(
                (p - q).abs() <= 1e-5 * (1.0 + q.abs()),
                "{what}[{i}]: {p} vs {q}"
            );
        }
    }

    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn identity_transport_selects_the_same_atom_without_transport() {
        let (plan, q, _, device) = setup(1, 11);
        // Carriers coded by atom: atom a holds (a + 1) on axis 0 and zero elsewhere.
        let coded: Vec<f32> = (0..ROWS)
            .flat_map(|_| {
                (0..INPUT_ATOMS).flat_map(|a| {
                    let mut atom = [0.0f32; 8];
                    atom[0] = (a + 1) as f32;
                    atom
                })
            })
            .collect();
        let x = Tensor::from_vec(coded, (ROWS, INPUT_ATOMS * 8), &device).expect("coded");
        let dy = dy_for(&device, 3);
        let (standard, _, _) = run(&plan, &q, &x, FbsArm::OneAtomActive, &dy);
        let (identity, _, _) = run(&plan, &q, &x, FbsArm::IdentityTransport, &dy);
        for block in 0..ROWS * OUTPUT_ATOMS {
            let id = &identity[block * 8..block * 8 + 8];
            let st = &standard[block * 8..block * 8 + 8];
            assert!(
                id[1..].iter().all(|&v| v == 0.0),
                "identity transport moved axes: {id:?}"
            );
            let magnitude = st.iter().fold(0.0f32, |m, v| m.max(v.abs()));
            assert_eq!(
                id[0], magnitude,
                "block {block}: a different atom was selected"
            );
        }
    }

    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn two_paths_with_equal_queries_give_root_two_times_the_standard_projection() {
        let (plan, q1, x, device) = setup(1, 21);
        let [outputs, stages, eight] = plan.query_shape();
        let single = q1
            .as_tensor()
            .flatten_all()
            .unwrap()
            .to_vec1::<f32>()
            .unwrap();
        let row = stages * eight;
        let doubled: Vec<f32> = (0..outputs)
            .flat_map(|o| {
                let path = single[o * row..(o + 1) * row].to_vec();
                path.iter().chain(path.iter()).copied().collect::<Vec<_>>()
            })
            .collect();
        let q2 = Var::from_tensor(
            &Tensor::from_vec(doubled, (outputs, 2 * stages, eight), &device).unwrap(),
        )
        .unwrap();
        let dy = dy_for(&device, 5);
        let (y1, dq1, dx1) = run(&plan, &q1, &x, FbsArm::OneAtomActive, &dy);
        let (y2, dq2, dx2) = run(&plan, &q2, &x, FbsArm::TwoAtomMix, &dy);
        let root2 = std::f32::consts::SQRT_2;
        close(
            &y2,
            &y1.iter().map(|v| v * root2).collect::<Vec<_>>(),
            "forward",
        );
        close(
            &dx2,
            &dx1.iter().map(|v| v * root2).collect::<Vec<_>>(),
            "input adjoint",
        );
        for o in 0..outputs {
            for path in 0..2 {
                let got = &dq2[(o * 2 + path) * row..(o * 2 + path + 1) * row];
                let want: Vec<f32> = dq1[o * row..(o + 1) * row]
                    .iter()
                    .map(|v| v / root2)
                    .collect();
                close(got, &want, "query credit");
            }
        }
    }

    /// Distinct paths pin each path's own query slot: the mix is the 1/√2 sum of the two standard
    /// projections, its input adjoint the 1/√2 sum of theirs, and each path's query credit its own
    /// standard credit times 1/√2.
    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn two_distinct_paths_give_the_scaled_sum_and_each_their_own_query_credit() {
        let (plan, qa, x, device) = setup(1, 51);
        let [outputs, stages, eight] = plan.query_shape();
        let row = stages * eight;
        let qb = Var::from_tensor(
            &Tensor::from_vec(values(52, outputs * row), (outputs, stages, eight), &device)
                .unwrap(),
        )
        .unwrap();
        let host = |q: &Var| {
            q.as_tensor()
                .flatten_all()
                .unwrap()
                .to_vec1::<f32>()
                .unwrap()
        };
        let (a, b) = (host(&qa), host(&qb));
        let mixed: Vec<f32> = (0..outputs)
            .flat_map(|o| {
                let first = &a[o * row..(o + 1) * row];
                let second = &b[o * row..(o + 1) * row];
                first.iter().chain(second).copied().collect::<Vec<_>>()
            })
            .collect();
        let qm = Var::from_tensor(
            &Tensor::from_vec(mixed, (outputs, 2 * stages, eight), &device).unwrap(),
        )
        .unwrap();
        let dy = dy_for(&device, 13);
        let (ya, dqa, dxa) = run(&plan, &qa, &x, FbsArm::OneAtomActive, &dy);
        let (yb, dqb, dxb) = run(&plan, &qb, &x, FbsArm::OneAtomActive, &dy);
        let (ym, dqm, dxm) = run(&plan, &qm, &x, FbsArm::TwoAtomMix, &dy);
        assert_ne!(ya, yb, "the fixture's two paths route differently");
        let inv = std::f32::consts::FRAC_1_SQRT_2;
        let scaled_sum = |p: &[f32], q: &[f32]| {
            p.iter()
                .zip(q)
                .map(|(u, v)| (u + v) * inv)
                .collect::<Vec<_>>()
        };
        close(&ym, &scaled_sum(&ya, &yb), "forward");
        close(&dxm, &scaled_sum(&dxa, &dxb), "input adjoint");
        for o in 0..outputs {
            for (path, own) in [(0, &dqa), (1, &dqb)] {
                let got = &dqm[(o * 2 + path) * row..(o * 2 + path + 1) * row];
                let want: Vec<f32> = own[o * row..(o + 1) * row]
                    .iter()
                    .map(|v| v * inv)
                    .collect();
                close(got, &want, "query credit");
            }
        }
    }

    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn frozen_queries_get_no_credit_and_the_exact_input_adjoint() {
        let (plan, q, x, device) = setup(1, 31);
        let dy = dy_for(&device, 7);
        let (ys, dqs, dxs) = run(&plan, &q, &x, FbsArm::OneAtomActive, &dy);
        let (yf, dqf, dxf) = run(&plan, &q, &x, FbsArm::FrozenQueries, &dy);
        assert_eq!(
            yf, ys,
            "the frozen arm's forward is the standard projection"
        );
        // The input adjoint sums several output blocks into one atom with device atomics, so its
        // last bit depends on the order the threads land; compare it to rounding, not bitwise.
        close(&dxf, &dxs, "the frozen arm keeps the exact input adjoint");
        assert!(
            dqs.iter().any(|&v| v != 0.0),
            "the fixture has query credit to zero"
        );
        assert!(
            dqf.iter().all(|&v| v == 0.0),
            "frozen queries received credit"
        );
    }

    /// For fixed routes, `y` is linear in `x`, so `<dy, y(x)> = <dx(dy), x>` exactly up to
    /// rounding: the input adjoint is the transpose of the forward.
    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn the_arm_input_adjoints_are_the_transpose_of_their_forwards() {
        for (arm, paths) in [(FbsArm::IdentityTransport, 1), (FbsArm::TwoAtomMix, 2)] {
            let (plan, q, x, device) = setup(paths, 41 + paths as u64);
            let dy = dy_for(&device, 9);
            let (y, dq, dx) = run(&plan, &q, &x, arm, &dy);
            let dy_host = dy.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            let x_host = x.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            let left: f64 = dy_host.iter().zip(&y).map(|(a, b)| f64::from(a * b)).sum();
            let right: f64 = dx.iter().zip(&x_host).map(|(a, b)| f64::from(a * b)).sum();
            assert!(
                (left - right).abs() <= 1e-3 * (1.0 + left.abs()),
                "{}: {left} vs {right}",
                arm.name()
            );
            assert!(
                dq.iter().all(|v| v.is_finite()) && dq.iter().any(|&v| v != 0.0),
                "{}: query credit",
                arm.name()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arm_names_and_codes_follow_the_registered_sheet() {
        let names: Vec<&str> = FbsArm::ALL.iter().map(|arm| arm.name()).collect();
        assert_eq!(
            names,
            [
                "Fbs_1Atom_Active",
                "Fbs_2Atom_Mix",
                "Fbs_IdentityTransport_Control",
                "Fbs_FrozenQueries_Null"
            ]
        );
        assert_eq!(FbsArm::TwoAtomMix.paths(), 2);
        assert_eq!(FbsArm::OneAtomActive.paths(), 1);
        assert!(!FbsArm::FrozenQueries.trains_queries());
        assert_eq!(FbsArm::IdentityTransport.kernel_code(), Some(1));
        assert_eq!(FbsArm::TwoAtomMix.kernel_code(), Some(2));
        assert_eq!(FbsArm::FrozenQueries.kernel_code(), None);
    }

    #[test]
    fn the_hook_is_per_thread_and_clears() {
        set_fbs_arm(Some(FbsArm::IdentityTransport));
        assert_eq!(fbs_arm(), Some(FbsArm::IdentityTransport));
        std::thread::spawn(|| assert_eq!(fbs_arm(), None))
            .join()
            .expect("thread");
        set_fbs_arm(None);
        assert_eq!(fbs_arm(), None);
    }

    /// The arm module's kernels: exactly the two it declares, and none of the banned families
    /// (the same scan `tests/no_training_crate_dep.rs` runs on the 24-kernel extract).
    #[test]
    fn the_arm_module_declares_two_kernels_and_no_banned_function() {
        let source = format!(
            "{}\n{}",
            include_str!("spiralcore_fbs_arms_prelude.cu"),
            include_str!("spiralcore_fbs_arms.cu")
        );
        let kernels: Vec<String> = source
            .lines()
            .filter_map(|line| {
                line.trim_start()
                    .strip_prefix("extern \"C\" __global__ void ")
            })
            .map(|rest| {
                rest.chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect()
            })
            .collect();
        assert_eq!(
            kernels,
            ["spiralcore_fbs_arm_fwd", "spiralcore_fbs_arm_bwd"]
        );
        let lower = source.to_ascii_lowercase();
        for banned in [
            concat!("soft", "max"),
            concat!("sig", "moid"),
            concat!("ta", "nh"),
            concat!("ge", "lu"),
            concat!("soft", "plus"),
            concat!("ex", "p("),
            concat!("ex", "pf("),
            concat!("__ex", "pf"),
            concat!("mm", "a."),
            concat!("wm", "ma"),
        ] {
            assert!(!lower.contains(banned), "the arm module contains {banned}");
        }
    }
}
