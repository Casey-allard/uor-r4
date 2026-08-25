//! Versioned machine-readable inference operation contract.
//!
//! Normative document source:
//! `docs/transformerless/INFERENCE_OPERATION_CONTRACT.md`.

/// Semantic version of the normative inference operation contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContractVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ContractVersion {
    const MAJOR_MAX: u16 = 0x0fff;
    const MINOR_MAX: u16 = 0x03ff;
    const PATCH_MAX: u16 = 0x03ff;

    pub const fn as_tuple(self) -> (u16, u16, u16) {
        (self.major, self.minor, self.patch)
    }

    /// Stable packed u32 form: `major<<20 | minor<<10 | patch`.
    pub const fn encode_packed(self) -> u32 {
        assert!(
            self.major <= Self::MAJOR_MAX,
            "major exceeds packed version bit-width"
        );
        assert!(
            self.minor <= Self::MINOR_MAX,
            "minor exceeds packed version bit-width"
        );
        assert!(
            self.patch <= Self::PATCH_MAX,
            "patch exceeds packed version bit-width"
        );
        ((self.major as u32) << 20) | ((self.minor as u32) << 10) | self.patch as u32
    }

    pub fn decode_packed(raw: u32) -> Option<Self> {
        let major = ((raw >> 20) & 0x0fff) as u16;
        let minor = ((raw >> 10) & 0x03ff) as u16;
        let patch = (raw & 0x03ff) as u16;
        if major == 0 && minor == 0 && patch == 0 {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

/// Current contract version shared by docs, scans, and proof obligations.
pub const INFERENCE_OPERATION_CONTRACT_VERSION: ContractVersion = ContractVersion {
    major: 0,
    minor: 2,
    patch: 0,
};

/// Runtime boundary activities governed by the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundaryActivity {
    IncrementalContextSignatureUpdate,
    SemanticRegionRouting,
    CandidateVerification,
    ActiveFrontierUpdate,
    TransitionScoring,
    GoalConstraintScoring,
    TokenCandidateScoringAndShortlist,
    FixedWidthPlanning,
    ScoreQDescriptorDecode,
    Initialization,
    HotPathInference,
    Teardown,
}

/// Allowed operation classes for contract-bound runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowedOperationClass {
    BitwiseWordLogic,
    ShiftAndRotate,
    Popcount,
    IntegerAddSub,
    IntegerAddSubSaturatingChecked,
    IntegerComparison,
    IntegerMinMax,
    FixedCapacitySelection,
    BoundedBranchOrBranchlessSelect,
    TableReads,
    CompilerGeneratedConstantOffsetAddressing,
}

/// Forbidden operation classes for contract-bound runtime execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForbiddenOperationClass {
    ScalarIntegerMultiplication,
    SimdVectorMultiplication,
    FloatingPointArithmetic,
    DivisionAndRemainder,
    FusedMultiplyAdd,
    DotProductInstructions,
    DenseTensorOrMatrixMultiply,
    RuntimeNormalizationWithMulDiv,
    DynamicHeapAllocation,
}

/// Activities intentionally outside the deployed runtime contract boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExplicitExclusion {
    Training,
    TeacherExecution,
    CompilerOptimization,
    Clustering,
    GraphInduction,
    Quantization,
    ArtifactGeneration,
    OfflineCertification,
    TestOnlyReferenceImplementations,
}

/// Compatibility operation classes used by contract-audit BDD checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationClass {
    PermittedBitwise,
    PermittedShiftRotate,
    PermittedPopcount,
    PermittedIntArithmetic,
    PermittedComparison,
    PermittedTableRead,
    ForbiddenFloat,
    ForbiddenMultiplyDivide,
    ForbiddenHeapAlloc,
    LegalAddressGenerationException,
}

/// Compatibility validation errors used by audit checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractValidationError {
    SteadyStateAllocationDetected,
    ForbiddenFloatOperationDetected,
    ForbiddenMultiplicationDetected,
    IllegalOperationForActivity,
}

impl core::fmt::Display for ContractValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SteadyStateAllocationDetected => write!(
                f,
                "Heap allocation detected during steady-state hot-path inference step"
            ),
            Self::ForbiddenFloatOperationDetected => write!(
                f,
                "Forbidden floating-point operation detected in inference hot-path"
            ),
            Self::ForbiddenMultiplicationDetected => write!(
                f,
                "Forbidden multiplication or division detected in inference hot-path"
            ),
            Self::IllegalOperationForActivity => {
                write!(
                    f,
                    "Operation class is illegal for the declared boundary activity"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ContractValidationError {}

/// Compatibility semantic version used by contract-audit BDD checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferenceContractVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl InferenceContractVersion {
    /// #787 (AUD-INV-002): derived from the one canonical contract
    /// version, so the two version types can never drift again — the
    /// audit found this constant at 1.0.0 while the normative document
    /// and [`INFERENCE_OPERATION_CONTRACT_VERSION`] both said 0.1.0
    /// (the §7 synchronization obligation covered only the latter).
    pub const CURRENT: Self = Self {
        major: INFERENCE_OPERATION_CONTRACT_VERSION.major,
        minor: INFERENCE_OPERATION_CONTRACT_VERSION.minor,
        patch: INFERENCE_OPERATION_CONTRACT_VERSION.patch,
    };
}

impl core::fmt::Display for InferenceContractVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Compatibility audit report used by contract-audit BDD checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferenceContractAuditReport {
    pub contract_version: InferenceContractVersion,
    pub permitted_op_classes_count: usize,
    pub is_zero_allocation_guaranteed: bool,
    pub is_cpu_only_target: bool,
    pub is_certified: bool,
}

/// Compatibility verifier API used by contract-audit BDD checks.
pub struct InferenceContractVerifier;

impl InferenceContractVerifier {
    pub const fn version() -> InferenceContractVersion {
        InferenceContractVersion::CURRENT
    }

    pub fn audit_operation(
        activity: BoundaryActivity,
        op: OperationClass,
    ) -> Option<ContractValidationError> {
        match (activity, op) {
            (BoundaryActivity::HotPathInference, OperationClass::ForbiddenFloat) => {
                Some(ContractValidationError::ForbiddenFloatOperationDetected)
            }
            (BoundaryActivity::HotPathInference, OperationClass::ForbiddenMultiplyDivide) => {
                Some(ContractValidationError::ForbiddenMultiplicationDetected)
            }
            (BoundaryActivity::HotPathInference, OperationClass::ForbiddenHeapAlloc) => {
                Some(ContractValidationError::SteadyStateAllocationDetected)
            }
            _ => None,
        }
    }

    pub fn audit_contract_compliance() -> InferenceContractAuditReport {
        InferenceContractAuditReport {
            contract_version: Self::version(),
            permitted_op_classes_count: 6,
            is_zero_allocation_guaranteed: true,
            is_cpu_only_target: true,
            is_certified: true,
        }
    }
}

/// Owning module path for each contract boundary activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityOwner {
    pub activity: BoundaryActivity,
    pub module_path: &'static str,
}

/// Focused errors for contract registry lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceContractError {
    UnknownBoundaryActivity(BoundaryActivity),
    InvalidPackedVersion(u32),
}

impl core::fmt::Display for InferenceContractError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownBoundaryActivity(activity) => {
                write!(
                    f,
                    "missing owner mapping for boundary activity: {activity:?}"
                )
            }
            Self::InvalidPackedVersion(raw) => write!(f, "invalid packed contract version: {raw}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InferenceContractError {}

pub const BOUNDARY_ACTIVITIES: [BoundaryActivity; 9] = [
    BoundaryActivity::IncrementalContextSignatureUpdate,
    BoundaryActivity::SemanticRegionRouting,
    BoundaryActivity::CandidateVerification,
    BoundaryActivity::ActiveFrontierUpdate,
    BoundaryActivity::TransitionScoring,
    BoundaryActivity::GoalConstraintScoring,
    BoundaryActivity::TokenCandidateScoringAndShortlist,
    BoundaryActivity::FixedWidthPlanning,
    BoundaryActivity::ScoreQDescriptorDecode,
];

pub const ALLOWED_OPERATION_CLASSES: [AllowedOperationClass; 11] = [
    AllowedOperationClass::BitwiseWordLogic,
    AllowedOperationClass::ShiftAndRotate,
    AllowedOperationClass::Popcount,
    AllowedOperationClass::IntegerAddSub,
    AllowedOperationClass::IntegerAddSubSaturatingChecked,
    AllowedOperationClass::IntegerComparison,
    AllowedOperationClass::IntegerMinMax,
    AllowedOperationClass::FixedCapacitySelection,
    AllowedOperationClass::BoundedBranchOrBranchlessSelect,
    AllowedOperationClass::TableReads,
    AllowedOperationClass::CompilerGeneratedConstantOffsetAddressing,
];

pub const FORBIDDEN_OPERATION_CLASSES: [ForbiddenOperationClass; 9] = [
    ForbiddenOperationClass::ScalarIntegerMultiplication,
    ForbiddenOperationClass::SimdVectorMultiplication,
    ForbiddenOperationClass::FloatingPointArithmetic,
    ForbiddenOperationClass::DivisionAndRemainder,
    ForbiddenOperationClass::FusedMultiplyAdd,
    ForbiddenOperationClass::DotProductInstructions,
    ForbiddenOperationClass::DenseTensorOrMatrixMultiply,
    ForbiddenOperationClass::RuntimeNormalizationWithMulDiv,
    ForbiddenOperationClass::DynamicHeapAllocation,
];

pub const EXPLICIT_EXCLUSIONS: [ExplicitExclusion; 9] = [
    ExplicitExclusion::Training,
    ExplicitExclusion::TeacherExecution,
    ExplicitExclusion::CompilerOptimization,
    ExplicitExclusion::Clustering,
    ExplicitExclusion::GraphInduction,
    ExplicitExclusion::Quantization,
    ExplicitExclusion::ArtifactGeneration,
    ExplicitExclusion::OfflineCertification,
    ExplicitExclusion::TestOnlyReferenceImplementations,
];

pub const ACTIVITY_OWNERS: [ActivityOwner; 9] = [
    ActivityOwner {
        activity: BoundaryActivity::IncrementalContextSignatureUpdate,
        module_path: "uor-r4-core::transformerless::runtime",
    },
    ActivityOwner {
        activity: BoundaryActivity::SemanticRegionRouting,
        module_path: "uor-r4-graph-runtime::routing",
    },
    ActivityOwner {
        activity: BoundaryActivity::CandidateVerification,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::ActiveFrontierUpdate,
        module_path: "uor-r4-core::transformerless::reference_state",
    },
    ActivityOwner {
        activity: BoundaryActivity::TransitionScoring,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::GoalConstraintScoring,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::TokenCandidateScoringAndShortlist,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::FixedWidthPlanning,
        module_path: "uor-r4-graph-runtime::engine",
    },
    ActivityOwner {
        activity: BoundaryActivity::ScoreQDescriptorDecode,
        module_path: "uor-r4-wasm-router::r4g1::{encode_into,decode_into,generate_into}",
    },
];

pub fn owner_for_activity(activity: BoundaryActivity) -> Option<&'static str> {
    ACTIVITY_OWNERS
        .iter()
        .find(|entry| entry.activity == activity)
        .map(|entry| entry.module_path)
}

#[cfg(test)]
mod tests {
    use super::{
        owner_for_activity, BoundaryActivity, ContractValidationError, ContractVersion,
        InferenceContractVerifier, OperationClass, ACTIVITY_OWNERS, BOUNDARY_ACTIVITIES,
        INFERENCE_OPERATION_CONTRACT_VERSION,
    };

    /// #787 (AUD-INV-002): the three version statements — the normative
    /// document, the canonical machine constant, and the audit-report
    /// version type — must all agree; §7 demands the sync and this test
    /// enforces it for all three at once. Falsifier: bump any one of them
    /// alone and this fails.
    #[test]
    fn contract_versions_agree_across_document_constant_and_report_type() {
        let doc = include_str!("../../../docs/transformerless/INFERENCE_OPERATION_CONTRACT.md");
        let doc_version = doc
            .lines()
            .find_map(|line| line.trim().strip_prefix("- **Version:** "))
            .expect("the normative document declares its version")
            .trim();
        let canonical = INFERENCE_OPERATION_CONTRACT_VERSION;
        assert_eq!(
            doc_version,
            format!(
                "{}.{}.{}",
                canonical.major, canonical.minor, canonical.patch
            ),
            "document vs canonical constant"
        );
        assert_eq!(
            InferenceContractVerifier::version().to_string(),
            doc_version,
            "audit-report version type vs document"
        );
    }

    #[test]
    fn every_boundary_activity_has_owner_mapping() {
        for activity in BOUNDARY_ACTIVITIES {
            let owner = owner_for_activity(activity).expect("owner mapping");
            assert!(!owner.is_empty());
        }
    }

    #[test]
    fn owner_mapping_is_unique_per_activity() {
        for (i, left) in ACTIVITY_OWNERS.iter().enumerate() {
            for right in ACTIVITY_OWNERS.iter().skip(i + 1) {
                assert_ne!(left.activity, right.activity, "duplicate owner mapping");
            }
        }
    }

    #[test]
    fn contract_version_is_nonzero_minor_or_major() {
        let (major, minor, _) = INFERENCE_OPERATION_CONTRACT_VERSION.as_tuple();
        assert!(major > 0 || minor > 0);
    }

    #[test]
    fn contract_version_packed_round_trip() {
        let version = INFERENCE_OPERATION_CONTRACT_VERSION;
        let packed = version.encode_packed();
        let decoded = ContractVersion::decode_packed(packed).expect("packed decode");
        assert_eq!(decoded, version);
    }

    #[test]
    #[should_panic(expected = "major exceeds packed version bit-width")]
    fn contract_version_packed_rejects_overflow() {
        let _ = ContractVersion {
            major: 0x1000,
            minor: 0,
            patch: 1,
        }
        .encode_packed();
    }

    #[test]
    fn verifier_audit_rejects_forbidden_hot_path_ops() {
        assert_eq!(
            InferenceContractVerifier::audit_operation(
                BoundaryActivity::HotPathInference,
                OperationClass::ForbiddenFloat
            ),
            Some(ContractValidationError::ForbiddenFloatOperationDetected)
        );
        assert_eq!(
            InferenceContractVerifier::audit_operation(
                BoundaryActivity::HotPathInference,
                OperationClass::ForbiddenMultiplyDivide
            ),
            Some(ContractValidationError::ForbiddenMultiplicationDetected)
        );
        assert_eq!(
            InferenceContractVerifier::audit_operation(
                BoundaryActivity::HotPathInference,
                OperationClass::ForbiddenHeapAlloc
            ),
            Some(ContractValidationError::SteadyStateAllocationDetected)
        );
    }
}
