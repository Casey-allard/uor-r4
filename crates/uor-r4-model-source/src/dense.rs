//! Immutable source-dense operator identities.
//!
//! Dense arithmetic is execution provenance for the offline teacher. It does
//! not change the source snapshot kappa and is not a deployed-runtime
//! operation. A registry version names stable output semantics; proof-bound
//! refinements that preserve every output bit remain within that version.

use serde::{Deserialize, Serialize};

/// Typed, versioned record of one source-dense operator.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DenseOperatorSpec {
    /// Registry id of the operator.
    #[serde(default)]
    pub id: String,
    /// Immutable registry version.
    #[serde(default)]
    pub version: u32,
    /// Logical Conv1D weight orientation and traversal domain.
    #[serde(default)]
    pub conv1d_weight_layout: String,
    /// Dot-product semantics used by every Conv1D projection.
    #[serde(default)]
    pub conv1d_accumulation: String,
    /// Placement and rounding semantics of the Conv1D bias.
    #[serde(default)]
    pub conv1d_bias: String,
    /// Logical tied-lm-head weight orientation.
    #[serde(default)]
    pub lm_head_weight_layout: String,
    /// Dot-product and bias semantics of the tied lm-head.
    #[serde(default)]
    pub lm_head_accumulation: String,
    /// Owner that establishes the declared output bits.
    #[serde(default)]
    pub arithmetic_owner: String,
    /// Host-side operation class; this is not the deployed integer runtime.
    #[serde(default)]
    pub permitted_operation_class: String,
    /// `blake3:<hex>` over [`Self::canonical_bytes`].
    #[serde(default)]
    pub implementation_digest: String,
}

impl DenseOperatorSpec {
    /// GPT-2 source-dense registry id.
    pub const GPT2_ID: &'static str = "gpt2-source-dense";
    /// Historical conventional GPT-2 dense arithmetic.
    pub const GPT2_V1_VERSION: u32 = 1;
    /// Current correctly-rounded exact-real-dot GPT-2 dense arithmetic.
    pub const GPT2_V2_VERSION: u32 = 2;
    /// Current GPT-2 dense registry version.
    pub const GPT2_VERSION: u32 = Self::GPT2_V2_VERSION;

    /// Immutable historical `gpt2-source-dense/1` record.
    pub fn gpt2_v1() -> Self {
        let mut record = Self {
            id: Self::GPT2_ID.to_owned(),
            version: Self::GPT2_V1_VERSION,
            conv1d_weight_layout: "input-major-row-major-[in,out]".to_owned(),
            conv1d_accumulation:
                "input-index-ascending-binary32-product-add-left-fold-zero-input-skipped".to_owned(),
            conv1d_bias: "bias-seeds-fold-before-input-products".to_owned(),
            lm_head_weight_layout: "tied-wte-row-major-[vocab,in]".to_owned(),
            lm_head_accumulation:
                "zero-seeded-input-index-ascending-binary32-product-add-left-fold-no-bias"
                    .to_owned(),
            arithmetic_owner: "scalar-conventional-binary32".to_owned(),
            permitted_operation_class: "host-source-f32".to_owned(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Immutable current `gpt2-source-dense/2` record. The production owner
    /// establishes these bits with certified-native arithmetic and the pinned
    /// exact fallback. Those proof mechanics are intentionally noncanonical:
    /// a bit-preserving certificate refinement does not create v3.
    pub fn gpt2_v2() -> Self {
        let mut record = Self {
            id: Self::GPT2_ID.to_owned(),
            version: Self::GPT2_V2_VERSION,
            conv1d_weight_layout: "input-major-row-major-[in,out]".to_owned(),
            conv1d_accumulation: "correctly-rounded-binary32-exact-real-dot".to_owned(),
            conv1d_bias: "one-binary32-add-after-dot".to_owned(),
            lm_head_weight_layout: "tied-wte-row-major-[vocab,in]".to_owned(),
            lm_head_accumulation: "correctly-rounded-binary32-exact-real-dot-no-bias".to_owned(),
            arithmetic_owner: "correctly-rounded-binary32-exact-real-result".to_owned(),
            permitted_operation_class: "host-source-f32-f64-exact-real-result".to_owned(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Current GPT-2 source-dense record.
    pub fn gpt2_source_dense() -> Self {
        Self::gpt2_v2()
    }

    /// Fixed-order canonical serialization of the declared identity.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "uor-r4-dense-operator/1\n\
             id={}\n\
             version={}\n\
             conv1d_weight_layout={}\n\
             conv1d_accumulation={}\n\
             conv1d_bias={}\n\
             lm_head_weight_layout={}\n\
             lm_head_accumulation={}\n\
             arithmetic_owner={}\n\
             permitted_operation_class={}\n",
            self.id,
            self.version,
            self.conv1d_weight_layout,
            self.conv1d_accumulation,
            self.conv1d_bias,
            self.lm_head_weight_layout,
            self.lm_head_accumulation,
            self.arithmetic_owner,
            self.permitted_operation_class,
        )
        .into_bytes()
    }

    /// Digest implied by the canonical declared identity.
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// Versioned dense-operator registry. Unknown pairs fail closed.
#[cfg(not(target_arch = "wasm32"))]
pub fn operator_spec(
    id: &str,
    version: u32,
) -> Result<DenseOperatorSpec, crate::SourceUnavailable> {
    match (id, version) {
        (DenseOperatorSpec::GPT2_ID, DenseOperatorSpec::GPT2_V1_VERSION) => {
            Ok(DenseOperatorSpec::gpt2_v1())
        }
        (DenseOperatorSpec::GPT2_ID, DenseOperatorSpec::GPT2_V2_VERSION) => {
            Ok(DenseOperatorSpec::gpt2_source_dense())
        }
        _ => Err(crate::SourceIngestKind::UnknownDenseOperator {
            id: id.to_owned(),
            version,
        }
        .into()),
    }
}

/// Validate one jointly declared source-attention/source-dense identity.
///
/// Absence of a dense record preserves legacy and non-GPT-2 producers. Once
/// GPT-2 dense provenance is present, its version must form one registered
/// execution era with learned-absolute GPT-2 attention; an absent or Llama
/// attention record is never silently paired with it.
#[cfg(not(target_arch = "wasm32"))]
pub fn validate_source_execution_pair(
    attention: Option<&crate::attention::AttentionOperatorSpec>,
    dense: Option<&DenseOperatorSpec>,
) -> Result<(), crate::SourceUnavailable> {
    let Some(dense) = dense else {
        return Ok(());
    };
    let valid = attention.is_some_and(|attention| {
        matches!(
            (
                attention.id.as_str(),
                attention.version,
                dense.id.as_str(),
                dense.version,
            ),
            (
                crate::attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
                crate::attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_V1_VERSION,
                DenseOperatorSpec::GPT2_ID,
                DenseOperatorSpec::GPT2_V1_VERSION,
            ) | (
                crate::attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_ID,
                crate::attention::AttentionOperatorSpec::LEARNED_ABSOLUTE_V2_VERSION,
                DenseOperatorSpec::GPT2_ID,
                DenseOperatorSpec::GPT2_V2_VERSION,
            )
        )
    });
    if valid {
        Ok(())
    } else {
        let attention = attention.map_or_else(
            || "absent".to_owned(),
            |record| format!("{}/{}", record.id, record.version),
        );
        Err(crate::SourceUnavailable::new(format!(
            "source execution pair is not registered: attention={attention}, dense={}/{}",
            dense.id, dense.version
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_alias_and_registry_are_exact() {
        assert_eq!(
            DenseOperatorSpec::gpt2_source_dense(),
            DenseOperatorSpec::gpt2_v2()
        );
        assert_eq!(
            DenseOperatorSpec::GPT2_VERSION,
            DenseOperatorSpec::GPT2_V2_VERSION
        );
        assert_eq!(
            operator_spec(
                DenseOperatorSpec::GPT2_ID,
                DenseOperatorSpec::GPT2_V1_VERSION
            )
            .expect("registered v1"),
            DenseOperatorSpec::gpt2_v1()
        );
        assert_eq!(
            operator_spec(
                DenseOperatorSpec::GPT2_ID,
                DenseOperatorSpec::GPT2_V2_VERSION
            )
            .expect("registered v2"),
            DenseOperatorSpec::gpt2_v2()
        );
        assert!(operator_spec(DenseOperatorSpec::GPT2_ID, 3).is_err());
    }

    #[test]
    fn canonical_literals_and_digests_are_pinned() {
        let v1 = DenseOperatorSpec::gpt2_v1();
        assert_eq!(
            v1.canonical_bytes(),
            b"uor-r4-dense-operator/1\n\
id=gpt2-source-dense\n\
version=1\n\
conv1d_weight_layout=input-major-row-major-[in,out]\n\
conv1d_accumulation=input-index-ascending-binary32-product-add-left-fold-zero-input-skipped\n\
conv1d_bias=bias-seeds-fold-before-input-products\n\
lm_head_weight_layout=tied-wte-row-major-[vocab,in]\n\
lm_head_accumulation=zero-seeded-input-index-ascending-binary32-product-add-left-fold-no-bias\n\
arithmetic_owner=scalar-conventional-binary32\n\
permitted_operation_class=host-source-f32\n"
        );
        assert_eq!(
            v1.implementation_digest,
            "blake3:b16a2a7f14828f854a7784d33cea9b49631136dbda77491899f2171cec011033"
        );

        let v2 = DenseOperatorSpec::gpt2_v2();
        assert_eq!(
            v2.canonical_bytes(),
            b"uor-r4-dense-operator/1\n\
id=gpt2-source-dense\n\
version=2\n\
conv1d_weight_layout=input-major-row-major-[in,out]\n\
conv1d_accumulation=correctly-rounded-binary32-exact-real-dot\n\
conv1d_bias=one-binary32-add-after-dot\n\
lm_head_weight_layout=tied-wte-row-major-[vocab,in]\n\
lm_head_accumulation=correctly-rounded-binary32-exact-real-dot-no-bias\n\
arithmetic_owner=correctly-rounded-binary32-exact-real-result\n\
permitted_operation_class=host-source-f32-f64-exact-real-result\n"
        );
        assert_eq!(
            v2.implementation_digest,
            "blake3:3a61d92e61b2a322e086162767173aca8439dffd1ddc7443f1d8b44ee1b1eaf6"
        );
    }

    #[test]
    fn source_execution_pairs_fail_closed() {
        let learned_v1 = crate::attention::AttentionOperatorSpec::learned_absolute_v1();
        let learned_v2 = crate::attention::AttentionOperatorSpec::learned_absolute_v2();
        let standard = crate::attention::AttentionOperatorSpec::standard();
        let dense_v1 = DenseOperatorSpec::gpt2_v1();
        let dense_v2 = DenseOperatorSpec::gpt2_v2();

        assert!(validate_source_execution_pair(None, None).is_ok());
        assert!(validate_source_execution_pair(Some(&standard), None).is_ok());
        assert!(validate_source_execution_pair(Some(&learned_v1), Some(&dense_v1)).is_ok());
        assert!(validate_source_execution_pair(Some(&learned_v2), Some(&dense_v2)).is_ok());
        assert!(validate_source_execution_pair(None, Some(&dense_v2)).is_err());
        assert!(validate_source_execution_pair(Some(&standard), Some(&dense_v2)).is_err());
        assert!(validate_source_execution_pair(Some(&learned_v1), Some(&dense_v2)).is_err());
        assert!(validate_source_execution_pair(Some(&learned_v2), Some(&dense_v1)).is_err());
    }
}
