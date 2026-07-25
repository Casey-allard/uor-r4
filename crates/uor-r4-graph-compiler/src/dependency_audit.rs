//! CPU-Only Compiler Dependency and Feature Auditor
//!
//! Specification: `docs/compiler_dependency_audit.md` (Issue #174).
//!
//! Audits workspace lockfiles, crate manifests, and default feature flags to verify
//! that no GPU, tensor, or BLAS accelerator dependencies enter the compiler tree.

use core::fmt;

/// Errors emitted during compiler dependency auditing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilerDependencyAuditError {
    /// Forbidden GPU or accelerator crate detected in dependency tree.
    ForbiddenCrateDetected {
        crate_name: String,
        matched_pattern: String,
    },
    /// Default GPU feature flag detected in workspace manifest.
    DefaultGpuFeatureDetected { feature_name: String },
}

impl fmt::Display for CompilerDependencyAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForbiddenCrateDetected {
                crate_name,
                matched_pattern,
            } => write!(
                f,
                "Forbidden crate '{crate_name}' detected matching denylist pattern '{matched_pattern}'"
            ),
            Self::DefaultGpuFeatureDetected { feature_name } => write!(
                f,
                "Forbidden default GPU feature '{feature_name}' detected in workspace manifest"
            ),
        }
    }
}

impl std::error::Error for CompilerDependencyAuditError {}

/// Audit report produced by `CompilerDependencyAuditor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerDependencyAuditReport {
    /// Total number of crates/dependencies audited.
    pub total_audited: usize,
    /// Verification status.
    pub is_clean: bool,
}

/// Compiler Dependency and Feature Auditor Engine.
pub struct CompilerDependencyAuditor;

impl CompilerDependencyAuditor {
    /// Exact forbidden crate names.
    pub const EXACT_FORBIDDEN_CRATES: &'static [&'static str] =
        &["ash", "ort", "torch", "metal", "rocm", "cuda", "opencl"];

    /// Substring / prefix forbidden crate patterns.
    pub const SUBSTRING_FORBIDDEN_PATTERNS: &'static [&'static str] = &[
        "cuda-",
        "cuda_",
        "cust",
        "cudnn",
        "nvml",
        "chainer-cuda",
        "hip-sys",
        "metal-sys",
        "opencl-",
        "cl-sys",
        "vulkan",
        "wgpu",
        "directml",
        "sycl",
        "tch",
        "candle-core",
        "onnxruntime",
        "openblas-sys",
        "intel-mkl-sys",
        "accelerate-src",
    ];

    /// Forbidden default feature patterns.
    pub const FORBIDDEN_FEATURE_PATTERNS: &'static [&'static str] =
        &["gpu", "cuda", "metal", "opencl", "vulkan", "wgpu", "sycl"];

    /// Check if a crate name matches any forbidden GPU/accelerator rule.
    pub fn matches_forbidden_rule(crate_name: &str) -> Option<&'static str> {
        let lower = crate_name.to_lowercase();
        if let Some(&exact) = Self::EXACT_FORBIDDEN_CRATES.iter().find(|&&e| lower == e) {
            return Some(exact);
        }
        if let Some(&pattern) = Self::SUBSTRING_FORBIDDEN_PATTERNS
            .iter()
            .find(|&&p| lower.contains(p))
        {
            return Some(pattern);
        }
        None
    }

    /// Audit a Cargo.lock string for forbidden GPU/tensor/BLAS crates.
    pub fn audit_lockfile_contents(
        lockfile_str: &str,
    ) -> Result<usize, CompilerDependencyAuditError> {
        let mut audited_count = 0;
        for line in lockfile_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name = ") {
                audited_count += 1;
                let crate_name = trimmed
                    .trim_start_matches("name = ")
                    .trim_matches('"')
                    .to_lowercase();

                if let Some(matched_pattern) = Self::matches_forbidden_rule(&crate_name) {
                    return Err(CompilerDependencyAuditError::ForbiddenCrateDetected {
                        crate_name,
                        matched_pattern: matched_pattern.to_string(),
                    });
                }
            }
        }
        Ok(audited_count)
    }

    /// Audit a Cargo.toml string for forbidden default features.
    pub fn audit_workspace_features(
        manifest_str: &str,
    ) -> Result<usize, CompilerDependencyAuditError> {
        let mut audited_count = 0;
        let mut in_default_features = false;

        for line in manifest_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[features]") {
                in_default_features = true;
                continue;
            }
            if trimmed.starts_with('[') {
                in_default_features = false;
            }

            if in_default_features && trimmed.starts_with("default =") {
                audited_count += 1;
                let lower = trimmed.to_lowercase();
                for &pattern in Self::FORBIDDEN_FEATURE_PATTERNS {
                    if lower.contains(pattern) {
                        return Err(CompilerDependencyAuditError::DefaultGpuFeatureDetected {
                            feature_name: pattern.to_string(),
                        });
                    }
                }
            }
        }
        Ok(audited_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_lockfile_passes_audit() {
        let sample_lockfile = r#"
[[package]]
name = "uor-r4-graph-compiler"
version = "0.1.0"

[[package]]
name = "rayon"
version = "1.10.0"

[[package]]
name = "serde"
version = "1.0.210"

[[package]]
name = "portable-atomic"
version = "1.6.0"
"#;
        let count = CompilerDependencyAuditor::audit_lockfile_contents(sample_lockfile).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_denylisted_cuda_crate_fails_audit() {
        let sample_lockfile = r#"
[[package]]
name = "uor-r4-graph-compiler"
version = "0.1.0"

[[package]]
name = "cust"
version = "0.3.0"
"#;
        let err = CompilerDependencyAuditor::audit_lockfile_contents(sample_lockfile).unwrap_err();
        assert!(matches!(
            err,
            CompilerDependencyAuditError::ForbiddenCrateDetected { .. }
        ));
    }

    #[test]
    fn test_denylisted_torch_crate_fails_audit() {
        let sample_lockfile = r#"
[[package]]
name = "tch"
version = "0.14.0"
"#;
        let err = CompilerDependencyAuditor::audit_lockfile_contents(sample_lockfile).unwrap_err();
        assert!(matches!(
            err,
            CompilerDependencyAuditError::ForbiddenCrateDetected { .. }
        ));
    }

    #[test]
    fn test_clean_workspace_features_pass() {
        let manifest = r#"
[features]
default = ["alloc"]
gpu-experimental = []
"#;
        let count = CompilerDependencyAuditor::audit_workspace_features(manifest).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_default_gpu_feature_fails_audit() {
        let manifest = r#"
[features]
default = ["gpu", "alloc"]
"#;
        let err = CompilerDependencyAuditor::audit_workspace_features(manifest).unwrap_err();
        assert!(matches!(
            err,
            CompilerDependencyAuditError::DefaultGpuFeatureDetected { .. }
        ));
    }
}
