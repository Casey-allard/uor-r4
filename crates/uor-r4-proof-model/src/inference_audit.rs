//! Machine-Code and Dependency Audit ENGINE — reference fixtures only.
//!
//! Specification & Source: `docs/inference_contract.md`; `archive/scoring_semantics.md`;
//! `archive/hologram_formal_analysis_direction.md` PDF §13; GitHub Issue #160.
//!
//! **What this module IS (#787, honest scope):** the audit *engine* — a
//! mnemonic scanner over disassembly text and a denylist scanner over
//! dependency-name lists — exercised against reference fixtures so the
//! engine itself is tested and its falsifiers demonstrably fire.
//!
//! **What this module is NOT:** the #160 machine-code audit. Nothing here
//! disassembles the release binary, walks the real `cargo metadata`
//! graph, or hooks an allocator; until #160 lands, the deployed
//! operation-set guarantee rests on the P-4 source-scan witnesses
//! (`INFERENCE_OPERATION_CONTRACT.md` §6/§8 — **Witnessed**, not
//! Structural, mirrored by the proof matrix's `Witnessed` row). The real
//! allocation census lives in the runtime crates' counting-allocator
//! tests, not here.

use core::fmt;

/// Audit Verdict for Machine-Code and Dependency Compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditVerdict {
    /// Fully compliant with all inference contract constraints.
    Compliant,
    /// Forbidden instruction class detected in release disassembly.
    ForbiddenInstructionDetected,
    /// Forbidden GPU/tensor/BLAS dependency detected in manifest.
    ForbiddenDependencyDetected,
}

impl fmt::Display for AuditVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compliant => write!(f, "Compliant"),
            Self::ForbiddenInstructionDetected => write!(f, "Forbidden Instruction Detected"),
            Self::ForbiddenDependencyDetected => write!(f, "Forbidden Dependency Detected"),
        }
    }
}

/// Report produced by [`InferenceAuditVerifier::audit_reference_fixtures`].
///
/// #787: this reports on the ENGINE's reference fixtures only — it
/// carries no allocation figure (nothing here measures allocations; the
/// old `steady_state_allocations: 0` literal was removed as an
/// overclaim) and `fixtures_clean` certifies the fixtures, never the
/// release binary (#160's job).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceAuditReport {
    pub verdict: AuditVerdict,
    pub instructions_scanned: usize,
    pub dependencies_scanned: usize,
    pub fixtures_clean: bool,
}

/// Machine-code and dependency audit ENGINE (fixture-level; see the
/// module doc — the #160 release-binary audit has not landed).
pub struct InferenceAuditVerifier;

impl InferenceAuditVerifier {
    /// Forbidden instruction mnemonics for x86_64, AArch64, and PTX/SASS.
    pub const FORBIDDEN_MNEMONICS: &'static [&'static str] = &[
        // Floating point (x86_64 / AArch64 / PTX)
        "fadd",
        "fsub",
        "fmul",
        "fdiv",
        "vaddss",
        "vsubss",
        "vmulss",
        "vdivss",
        "vaddps",
        "vsubps",
        "vmulps",
        "vdivps",
        "fadd.s",
        "fmul.s",
        // Fused multiply-add & float conversions (x86_64 / AArch64 / PTX)
        "fma.",
        "fmadd",
        "fmsub",
        "fnmadd",
        "vfmadd",
        "vfmsub",
        "vfnmadd",
        "cvtsi2ss",
        "cvtsi2sd",
        "cvtss2si",
        "cvtsd2si",
        "vcvt",
        "scvtf",
        "ucvtf",
        "fcvtzs",
        "fcvtzu",
        // PTX transcendentals & reciprocal/rsqrt/sqrt approximations (D23/D25/D32)
        "ex2.",
        "lg2.",
        "sin.",
        "cos.",
        "tanh.",
        "rcp.",
        "rsqrt.",
        "sqrt.approx",
        "sqrt.rn",
        // Tensor-core / matrix-multiply-accumulate (PTX / SASS — D25/D32)
        "mma.",
        "wmma.",
        "wgmma",
        "tcgen05",
        "hmma",
        "imma",
        "qmma",
        // Multiplication & Division
        "mul",
        "imul",
        "div",
        "idiv",
        "mul.d",
        "div.d",
        // Heap Allocation
        "malloc",
        "free",
        "_zn5alloc",
    ];

    /// Forbidden dependency crate names (D11/D25/D32).
    pub const FORBIDDEN_DEPENDENCIES: &'static [&'static str] = &[
        "cuda",
        "rocm",
        "metal",
        "opencl",
        "webgpu",
        "vulkan",
        "directml",
        "oneapi",
        "torch",
        "tensorflow",
        "blas",
        "cublas",
        "cudnn",
        "cutlass",
        "cutensor",
        "cusolver",
        "cusparse",
        "nvrtc",
        "nccl",
        "candle",
        "burn",
        "tch",
        "ort",
        "onnx",
        "nalgebra",
        "ndarray",
        "faer",
        "gemm",
        "matrixmultiply",
    ];

    fn is_forbidden_ptx_float_cvt(line_lower: &str) -> bool {
        const PTX_FLOAT_TYPES: &[&str] =
            &[".f16", ".bf16", ".f32", ".f64", ".tf32", ".e4m3", ".e5m2"];
        line_lower.contains("cvt.") && PTX_FLOAT_TYPES.iter().any(|ty| line_lower.contains(ty))
    }

    fn dep_matches_forbidden(dep_lower: &str, forbidden: &str) -> bool {
        if dep_lower == forbidden
            || dep_lower
                .split(|c: char| c == '-' || c == '_')
                .any(|seg| seg == forbidden)
        {
            return true;
        }
        if matches!(forbidden, "ort" | "tch") {
            return false;
        }
        dep_lower.contains(forbidden)
    }

    /// Audit a disassembly snippet for forbidden instruction mnemonics. Total:
    /// returns `Some(count)` of scanned instructions when clean, or `None` when
    /// a forbidden mnemonic is present (R5 — a failed audit is a measured
    /// report; the single reportable condition is
    /// [`AuditVerdict::ForbiddenInstructionDetected`], surfaced by `audit_all`).
    pub fn audit_disassembly(disassembly: &str) -> Option<usize> {
        let mut count = 0;
        for line in disassembly.lines() {
            count += 1;
            let lower = line.to_lowercase();
            if Self::is_forbidden_ptx_float_cvt(&lower) {
                return None;
            }
            for forbidden in Self::FORBIDDEN_MNEMONICS {
                if lower.contains(forbidden) {
                    return None;
                }
            }
        }
        Some(count)
    }

    /// Audit a Cargo manifest dependency list for forbidden GPU/tensor/BLAS
    /// dependencies. Total: `Some(count)` when clean, `None` when a forbidden
    /// dependency is present (R5).
    pub fn audit_dependencies(dependencies: &[&str]) -> Option<usize> {
        let mut count = 0;
        for dep in dependencies {
            count += 1;
            let lower = dep.to_lowercase();
            for forbidden in Self::FORBIDDEN_DEPENDENCIES {
                if Self::dep_matches_forbidden(&lower, forbidden) {
                    return None;
                }
            }
        }
        Some(count)
    }

    /// Exercise the audit ENGINE against its reference fixtures. Total:
    /// always produces an [`InferenceAuditReport`]; `verdict` and
    /// `fixtures_clean` carry whether the fixture scans held (R5 — the
    /// audit reports its finding, it never raises).
    ///
    /// #787: this is an engine self-test, not the #160 machine-code
    /// audit — the fixtures are a hand-written disassembly snippet and a
    /// hand-written dependency list, named as such. When #160 lands, the
    /// real inputs (release-binary disassembly of the contract-owned
    /// functions, the actual `cargo metadata` graph) replace these
    /// fixtures and this doc contract changes with them.
    pub fn audit_reference_fixtures() -> InferenceAuditReport {
        let fixture_disassembly = "mov eax, [rsp+8]\nadd eax, ebx\nxor ecx, ecx\nret";
        let Some(instructions_scanned) = Self::audit_disassembly(fixture_disassembly) else {
            return InferenceAuditReport {
                verdict: AuditVerdict::ForbiddenInstructionDetected,
                instructions_scanned: 0,
                dependencies_scanned: 0,
                fixtures_clean: false,
            };
        };

        let fixture_dependencies = &["uor-r4-graph-format", "uor-r4-graph-runtime", "core"];
        let Some(dependencies_scanned) = Self::audit_dependencies(fixture_dependencies) else {
            return InferenceAuditReport {
                verdict: AuditVerdict::ForbiddenDependencyDetected,
                instructions_scanned,
                dependencies_scanned: 0,
                fixtures_clean: false,
            };
        };

        InferenceAuditReport {
            verdict: AuditVerdict::Compliant,
            instructions_scanned,
            dependencies_scanned,
            fixtures_clean: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassembly_audit_passes_clean_code() {
        let clean = "mov eax, [rsp+8]\nadd eax, ebx\nxor ecx, ecx\npopcnt edx, eax\ncvt.u64.u32 %rd1, %r1;\ncvt.s32.s16 %r2, %r3;\nret";
        assert!(InferenceAuditVerifier::audit_disassembly(clean).is_some());
    }

    #[test]
    fn test_disassembly_audit_rejects_floating_point() {
        let bad = "vaddss xmm0, xmm1, xmm2";
        assert!(InferenceAuditVerifier::audit_disassembly(bad).is_none());
    }

    #[test]
    fn test_disassembly_audit_rejects_fma_conversions_ptx_and_tensor_core() {
        for bad in [
            "vfmadd213ps ymm0, ymm1, ymm2",
            "scvtf s0, w0",
            "cvt.rn.f32.s32 %f1, %r1;",
            "cvt.rzi.s32.f32 %r1, %f1;",
            "cvt.f16.f32 %h1, %f1;",
            "ex2.approx.f32 %f2, %f1;",
            "tanh.approx.f32 %f3, %f1;",
            "rcp.approx.f32 %f4, %f1;",
            "sqrt.rn.f32 %f5, %f1;",
            "mma.sync.aligned.m16n8k8.row.col.f32.f16.f16.f32",
            "wgmma.mma_async.sync.aligned.m64n64k16.f32.f16.f16",
            "tcgen05.mma.cta_group::1.kind::f16",
        ] {
            assert!(
                InferenceAuditVerifier::audit_disassembly(bad).is_none(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn test_disassembly_audit_rejects_multiplication() {
        let bad = "imul eax, ebx";
        assert!(InferenceAuditVerifier::audit_disassembly(bad).is_none());
    }

    #[test]
    fn test_dependency_audit_rejects_gpu_and_tensor_deps() {
        let clean_deps = &["uor-r4-graph-runtime", "uor-r4-support", "uor-r4-transport"];
        assert!(InferenceAuditVerifier::audit_dependencies(clean_deps).is_some());

        for bad in [
            "cuda-sys",
            "candle-core",
            "candle-nn",
            "faer-core",
            "ndarray",
            "nalgebra",
            "matrixmultiply",
            "ort",
            "ort-sys",
            "tch",
            "tch-sys",
            "cudnn-sys",
        ] {
            let bad_deps = &["uor-r4-graph-runtime", bad];
            assert!(
                InferenceAuditVerifier::audit_dependencies(bad_deps).is_none(),
                "expected dependency {bad:?} to be rejected"
            );
        }
    }
}
