//! Canonical compiler-side H4 spin-frame sidecar for #973.
//!
//! The sidecar is a strict serialization bridge between the implementation-
//! owned exact H4 registry and the Python training process.  It does not
//! define a second coordinate system: root coordinates, products, inverses,
//! and frame matrices are all derived from the existing canonical registry.
//! Matrices are serialized by their `f64` bit patterns so content identity is
//! independent of JSON float spelling.

use serde::{Deserialize, Serialize};

use crate::bounded_global_exact_spin_attention::ExactSpinState;
use crate::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure, CanonicalLexicalError, OpaqueH4TableIndex,
};
use crate::helm_d_r4_attention::{canonical_registered_h4_spin_frames, HelmDR4AttentionError};
use crate::r4_group_addressed_retention::{
    R4GroupGeometryArtifactV1, R4GroupGeometryError, R4_GROUP_MAX_TOKEN_ID,
    R4_GROUP_SCRAMBLE_POLICY,
};

pub const H4_SPIN_FRAME_SIDECAR_SCHEMA: u32 = 1;
pub const H4_SPIN_FRAME_SIDECAR_DOMAIN: &str = "uor-r4.h4-spin-frame-sidecar/1";
pub const H4_SPIN_FRAME_COUNT: usize = 120;
pub const H4_SPIN_FRAME_WIDTH: usize = 4;
pub const H4_SPIN_FRAME_PRODUCT_COUNT: usize = H4_SPIN_FRAME_COUNT * H4_SPIN_FRAME_COUNT;
pub const H4_SPIN_FRAME_ROOT_TABLE_KAPPA: &str =
    "blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76";
pub const H4_SPIN_FRAME_PRODUCT_TABLE_KAPPA: &str =
    "blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759";
pub const H4_SPIN_FRAME_ROOT_COORDINATE_CONVENTION: &str = "fixed canonical H4 order; quaternion basis (1,i,j,k); each coordinate is (a,b) for (a+b*phi)/2";
pub const H4_SPIN_FRAME_MATRIX_CONVENTION: &str = "row-major left-quaternion decode matrix F: local R4 -> model R4; transport T(a->b)=transpose(F_b)*F_a";

type Matrix4 = [[f64; H4_SPIN_FRAME_WIDTH]; H4_SPIN_FRAME_WIDTH];
type Matrix4Bits = [[u64; H4_SPIN_FRAME_WIDTH]; H4_SPIN_FRAME_WIDTH];
type RootCoordinate = [[i64; 2]; H4_SPIN_FRAME_WIDTH];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum H4SpinFrameSidecarError {
    Invalid(String),
    Canonical(String),
    Frame(String),
    GroupGeometry(String),
    Serialization(String),
    ArithmeticOverflow,
}

impl std::fmt::Display for H4SpinFrameSidecarError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::Canonical(reason) => write!(formatter, "canonical H4 table: {reason}"),
            Self::Frame(reason) => write!(formatter, "registered H4 frame: {reason}"),
            Self::GroupGeometry(reason) => write!(formatter, "H4 control geometry: {reason}"),
            Self::Serialization(reason) => write!(formatter, "canonical JSON: {reason}"),
            Self::ArithmeticOverflow => formatter.write_str("H4 sidecar arithmetic overflow"),
        }
    }
}

impl std::error::Error for H4SpinFrameSidecarError {}

impl From<CanonicalLexicalError> for H4SpinFrameSidecarError {
    fn from(error: CanonicalLexicalError) -> Self {
        Self::Canonical(error.to_string())
    }
}

impl From<HelmDR4AttentionError> for H4SpinFrameSidecarError {
    fn from(error: HelmDR4AttentionError) -> Self {
        Self::Frame(error.to_string())
    }
}

impl From<R4GroupGeometryError> for H4SpinFrameSidecarError {
    fn from(error: R4GroupGeometryError) -> Self {
        Self::GroupGeometry(error.to_string())
    }
}

/// Exact, content-addressed bridge from Rust's registered H4 frames to a
/// compiler-side trainer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct H4SpinFrameSidecarV1 {
    pub schema: u32,
    pub domain: String,
    pub frame_count: u16,
    pub frame_width: u16,
    pub product_count: u32,
    pub root_coordinate_convention: String,
    pub matrix_convention: String,
    pub h4_root_table_kappa: String,
    pub h4_multiplication_table_kappa: String,
    pub identity_index: u16,
    /// Exact `(a,b)` coefficients for `(a + b*phi)/2`, in table order.
    pub root_coordinates: Vec<RootCoordinate>,
    /// Row-major registered decode matrices, serialized as exact `f64` bits.
    pub frame_matrix_f64_bits: Vec<Matrix4Bits>,
    pub inverse_indices: Vec<u16>,
    /// Row-major exact products: `left * 120 + right`.
    pub multiplication_indices: Vec<u16>,
    /// Existing identity-fixing, non-homomorphic #973 connection control.
    pub connection_control_policy: String,
    pub connection_control_source_cid: String,
    pub connection_control_permutation: Vec<u16>,
    /// BLAKE3 of this artifact serialized with this field empty.
    pub artifact_cid: String,
}

impl H4SpinFrameSidecarV1 {
    pub fn build() -> Result<Self, H4SpinFrameSidecarError> {
        let artifact = Self::build_unvalidated()?;
        artifact.validate()?;
        Ok(artifact)
    }

    fn build_unvalidated() -> Result<Self, H4SpinFrameSidecarError> {
        let table = validate_h4_binary_icosahedral_closure()?;
        if table.root_count != H4_SPIN_FRAME_COUNT
            || table.product_count != H4_SPIN_FRAME_PRODUCT_COUNT
            || table.h4_root_table_kappa != H4_SPIN_FRAME_ROOT_TABLE_KAPPA
            || table.multiplication_table_kappa != H4_SPIN_FRAME_PRODUCT_TABLE_KAPPA
        {
            return Err(H4SpinFrameSidecarError::Invalid(
                "canonical H4 registry differs from the frozen sidecar contract".to_owned(),
            ));
        }

        let frames = canonical_registered_h4_spin_frames()?;
        if frames.len() != table.root_count {
            return Err(H4SpinFrameSidecarError::Invalid(
                "registered frame count differs from the exact H4 root table".to_owned(),
            ));
        }

        let mut root_coordinates = Vec::with_capacity(table.root_count);
        let mut frame_matrix_f64_bits = Vec::with_capacity(table.root_count);
        for (offset, frame) in frames.into_iter().enumerate() {
            let offset_u16 =
                u16::try_from(offset).map_err(|_| H4SpinFrameSidecarError::ArithmeticOverflow)?;
            if frame.h4_table_offset() != offset_u16 {
                return Err(H4SpinFrameSidecarError::Invalid(
                    "registered frame order differs from canonical table order".to_owned(),
                ));
            }
            let index =
                OpaqueH4TableIndex::from_table_offset(offset_u16, &table).ok_or_else(|| {
                    H4SpinFrameSidecarError::Invalid(
                        "registered frame offset is outside the exact H4 table".to_owned(),
                    )
                })?;
            let state = ExactSpinState::from_table_index_and_phases(index, 0, 0, &table)
                .map_err(|error| H4SpinFrameSidecarError::Canonical(error.to_string()))?;
            root_coordinates.push(
                state
                    .root_coordinate(&table)
                    .map_err(|error| H4SpinFrameSidecarError::Canonical(error.to_string()))?
                    .scaled_zphi_quaternion,
            );
            frame_matrix_f64_bits.push(matrix_to_bits(decode_matrix(frame)?));
        }

        let control = R4GroupGeometryArtifactV1::build(R4_GROUP_MAX_TOKEN_ID)?;
        if control.identity_index != table.identity_index
            || control.h4_root_table_kappa != table.h4_root_table_kappa
            || control.h4_multiplication_table_kappa != table.multiplication_table_kappa
        {
            return Err(H4SpinFrameSidecarError::Invalid(
                "connection control is not bound to the same canonical H4 registry".to_owned(),
            ));
        }

        let mut artifact = Self {
            schema: H4_SPIN_FRAME_SIDECAR_SCHEMA,
            domain: H4_SPIN_FRAME_SIDECAR_DOMAIN.to_owned(),
            frame_count: u16::try_from(H4_SPIN_FRAME_COUNT)
                .map_err(|_| H4SpinFrameSidecarError::ArithmeticOverflow)?,
            frame_width: u16::try_from(H4_SPIN_FRAME_WIDTH)
                .map_err(|_| H4SpinFrameSidecarError::ArithmeticOverflow)?,
            product_count: u32::try_from(H4_SPIN_FRAME_PRODUCT_COUNT)
                .map_err(|_| H4SpinFrameSidecarError::ArithmeticOverflow)?,
            root_coordinate_convention: H4_SPIN_FRAME_ROOT_COORDINATE_CONVENTION.to_owned(),
            matrix_convention: H4_SPIN_FRAME_MATRIX_CONVENTION.to_owned(),
            h4_root_table_kappa: table.h4_root_table_kappa,
            h4_multiplication_table_kappa: table.multiplication_table_kappa,
            identity_index: table.identity_index,
            root_coordinates,
            frame_matrix_f64_bits,
            inverse_indices: table.inverse_indices,
            multiplication_indices: table.multiplication_indices,
            connection_control_policy: R4_GROUP_SCRAMBLE_POLICY.to_owned(),
            connection_control_source_cid: control.artifact_cid,
            connection_control_permutation: control.scramble.permutation,
            artifact_cid: String::new(),
        };
        artifact.artifact_cid = artifact.reproduce_artifact_cid()?;
        Ok(artifact)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, H4SpinFrameSidecarError> {
        serde_json::to_vec(self)
            .map_err(|error| H4SpinFrameSidecarError::Serialization(error.to_string()))
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, H4SpinFrameSidecarError> {
        let artifact: Self = serde_json::from_slice(bytes)
            .map_err(|error| H4SpinFrameSidecarError::Serialization(error.to_string()))?;
        artifact.validate()?;
        if artifact.canonical_bytes()? != bytes {
            return Err(H4SpinFrameSidecarError::Invalid(
                "spin-frame sidecar bytes are valid JSON but not canonical Rust JSON".to_owned(),
            ));
        }
        Ok(artifact)
    }

    pub fn reproduce_artifact_cid(&self) -> Result<String, H4SpinFrameSidecarError> {
        let mut seed = self.clone();
        seed.artifact_cid.clear();
        Ok(blake3_cid(&seed.canonical_bytes()?))
    }

    pub fn validate(&self) -> Result<(), H4SpinFrameSidecarError> {
        validate_shape_and_algebra(self)?;
        let expected = Self::build_unvalidated()?;
        if self != &expected {
            return Err(H4SpinFrameSidecarError::Invalid(
                "spin-frame sidecar does not reproduce the canonical registered artifact"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

fn decode_matrix(
    frame: crate::helm_d_r4_attention::R4RegisteredSpinFrame,
) -> Result<Matrix4, H4SpinFrameSidecarError> {
    let mut matrix = [[0.0; H4_SPIN_FRAME_WIDTH]; H4_SPIN_FRAME_WIDTH];
    for column in 0..H4_SPIN_FRAME_WIDTH {
        let mut basis = [0.0; H4_SPIN_FRAME_WIDTH];
        basis[column] = 1.0;
        let decoded = frame.decode_local_block(basis)?;
        for row in 0..H4_SPIN_FRAME_WIDTH {
            matrix[row][column] = decoded[row];
        }
    }
    Ok(matrix)
}

fn matrix_to_bits(matrix: Matrix4) -> Matrix4Bits {
    matrix.map(|row| row.map(f64::to_bits))
}

fn matrix_from_bits(bits: Matrix4Bits) -> Matrix4 {
    bits.map(|row| row.map(f64::from_bits))
}

fn validate_shape_and_algebra(
    artifact: &H4SpinFrameSidecarV1,
) -> Result<(), H4SpinFrameSidecarError> {
    if artifact.schema != H4_SPIN_FRAME_SIDECAR_SCHEMA
        || artifact.domain != H4_SPIN_FRAME_SIDECAR_DOMAIN
        || usize::from(artifact.frame_count) != H4_SPIN_FRAME_COUNT
        || usize::from(artifact.frame_width) != H4_SPIN_FRAME_WIDTH
        || usize::try_from(artifact.product_count).ok() != Some(H4_SPIN_FRAME_PRODUCT_COUNT)
        || artifact.root_coordinate_convention != H4_SPIN_FRAME_ROOT_COORDINATE_CONVENTION
        || artifact.matrix_convention != H4_SPIN_FRAME_MATRIX_CONVENTION
        || artifact.h4_root_table_kappa != H4_SPIN_FRAME_ROOT_TABLE_KAPPA
        || artifact.h4_multiplication_table_kappa != H4_SPIN_FRAME_PRODUCT_TABLE_KAPPA
        || artifact.root_coordinates.len() != H4_SPIN_FRAME_COUNT
        || artifact.frame_matrix_f64_bits.len() != H4_SPIN_FRAME_COUNT
        || artifact.inverse_indices.len() != H4_SPIN_FRAME_COUNT
        || artifact.multiplication_indices.len() != H4_SPIN_FRAME_PRODUCT_COUNT
        || artifact.connection_control_policy != R4_GROUP_SCRAMBLE_POLICY
        || artifact.connection_control_permutation.len() != H4_SPIN_FRAME_COUNT
        || usize::from(artifact.identity_index) >= H4_SPIN_FRAME_COUNT
    {
        return Err(H4SpinFrameSidecarError::Invalid(
            "spin-frame sidecar schema, provenance, or shape differs from the frozen contract"
                .to_owned(),
        ));
    }
    if artifact.artifact_cid != artifact.reproduce_artifact_cid()? {
        return Err(H4SpinFrameSidecarError::Invalid(
            "spin-frame sidecar content identity does not reproduce".to_owned(),
        ));
    }

    let identity = usize::from(artifact.identity_index);
    let expected_indices = (0_u16
        ..u16::try_from(H4_SPIN_FRAME_COUNT)
            .map_err(|_| H4SpinFrameSidecarError::ArithmeticOverflow)?)
        .collect::<Vec<_>>();
    let mut sorted_control = artifact.connection_control_permutation.clone();
    sorted_control.sort_unstable();
    if sorted_control != expected_indices
        || usize::from(artifact.connection_control_permutation[identity]) != identity
    {
        return Err(H4SpinFrameSidecarError::Invalid(
            "connection control is not an identity-fixing permutation".to_owned(),
        ));
    }

    let matrices = artifact
        .frame_matrix_f64_bits
        .iter()
        .copied()
        .map(matrix_from_bits)
        .collect::<Vec<_>>();
    for (index, matrix) in matrices.iter().enumerate() {
        if matrix.iter().flatten().any(|value| !value.is_finite()) {
            return Err(H4SpinFrameSidecarError::Invalid(format!(
                "registered H4 frame {index} contains a non-finite value"
            )));
        }
        require_matrix_close(
            matrix_multiply(transpose(*matrix), *matrix),
            identity_matrix(),
            1.0e-12,
            "registered H4 frame is not orthogonal",
        )?;
    }
    require_matrix_close(
        matrices[identity],
        identity_matrix(),
        0.0,
        "declared H4 identity frame is not the exact identity matrix",
    )?;

    let expected_row = expected_indices.as_slice();
    for left in 0..H4_SPIN_FRAME_COUNT {
        let row_start = left
            .checked_mul(H4_SPIN_FRAME_COUNT)
            .ok_or(H4SpinFrameSidecarError::ArithmeticOverflow)?;
        let row_end = row_start
            .checked_add(H4_SPIN_FRAME_COUNT)
            .ok_or(H4SpinFrameSidecarError::ArithmeticOverflow)?;
        let row = artifact
            .multiplication_indices
            .get(row_start..row_end)
            .ok_or_else(|| {
                H4SpinFrameSidecarError::Invalid(
                    "H4 multiplication row is outside the frozen table".to_owned(),
                )
            })?;
        let mut sorted_row = row.to_vec();
        sorted_row.sort_unstable();
        if sorted_row != expected_row {
            return Err(H4SpinFrameSidecarError::Invalid(format!(
                "H4 multiplication row {left} is not a permutation"
            )));
        }
        if usize::from(row[identity]) != left
            || usize::from(artifact.multiplication_indices[identity * H4_SPIN_FRAME_COUNT + left])
                != left
        {
            return Err(H4SpinFrameSidecarError::Invalid(
                "declared H4 identity does not act exactly".to_owned(),
            ));
        }
        let inverse = usize::from(artifact.inverse_indices[left]);
        if inverse >= H4_SPIN_FRAME_COUNT
            || usize::from(row[inverse]) != identity
            || usize::from(artifact.multiplication_indices[inverse * H4_SPIN_FRAME_COUNT + left])
                != identity
        {
            return Err(H4SpinFrameSidecarError::Invalid(format!(
                "declared H4 inverse fails at frame {left}"
            )));
        }
        for (right, &product) in row.iter().enumerate() {
            let product = usize::from(product);
            if product >= H4_SPIN_FRAME_COUNT {
                return Err(H4SpinFrameSidecarError::Invalid(
                    "H4 multiplication table contains an out-of-range product".to_owned(),
                ));
            }
            require_matrix_close(
                matrix_multiply(matrices[left], matrices[right]),
                matrices[product],
                2.0e-12,
                "registered H4 matrices do not realize an exact-table product",
            )?;
        }
    }

    let mut non_homomorphic = false;
    for left in 0..H4_SPIN_FRAME_COUNT {
        for right in 0..H4_SPIN_FRAME_COUNT {
            let product =
                usize::from(artifact.multiplication_indices[left * H4_SPIN_FRAME_COUNT + right]);
            let permuted_product = artifact.connection_control_permutation[product];
            let permuted_left = usize::from(artifact.connection_control_permutation[left]);
            let permuted_right = usize::from(artifact.connection_control_permutation[right]);
            let product_of_permuted = artifact.multiplication_indices
                [permuted_left * H4_SPIN_FRAME_COUNT + permuted_right];
            if permuted_product != product_of_permuted {
                non_homomorphic = true;
                break;
            }
        }
        if non_homomorphic {
            break;
        }
    }
    if !non_homomorphic {
        return Err(H4SpinFrameSidecarError::Invalid(
            "connection control is a homomorphism and cannot destroy the canonical connection"
                .to_owned(),
        ));
    }
    Ok(())
}

fn matrix_multiply(left: Matrix4, right: Matrix4) -> Matrix4 {
    let mut output = [[0.0; H4_SPIN_FRAME_WIDTH]; H4_SPIN_FRAME_WIDTH];
    for (row, output_row) in output.iter_mut().enumerate() {
        for (column, output_value) in output_row.iter_mut().enumerate() {
            for middle in 0..H4_SPIN_FRAME_WIDTH {
                *output_value += left[row][middle] * right[middle][column];
            }
        }
    }
    output
}

fn transpose(matrix: Matrix4) -> Matrix4 {
    let mut output = [[0.0; H4_SPIN_FRAME_WIDTH]; H4_SPIN_FRAME_WIDTH];
    for (row, output_row) in output.iter_mut().enumerate() {
        for (column, output_value) in output_row.iter_mut().enumerate() {
            *output_value = matrix[column][row];
        }
    }
    output
}

fn identity_matrix() -> Matrix4 {
    let mut matrix = [[0.0; H4_SPIN_FRAME_WIDTH]; H4_SPIN_FRAME_WIDTH];
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] = 1.0;
    }
    matrix
}

fn require_matrix_close(
    actual: Matrix4,
    expected: Matrix4,
    tolerance: f64,
    reason: &str,
) -> Result<(), H4SpinFrameSidecarError> {
    if actual
        .iter()
        .flatten()
        .zip(expected.iter().flatten())
        .any(|(actual, expected)| (*actual - *expected).abs() > tolerance)
    {
        return Err(H4SpinFrameSidecarError::Invalid(reason.to_owned()));
    }
    Ok(())
}

fn blake3_cid(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_sidecar_binds_every_registered_frame_and_product() {
        let artifact = H4SpinFrameSidecarV1::build().expect("canonical H4 sidecar");
        assert_eq!(artifact.root_coordinates.len(), 120);
        assert_eq!(artifact.frame_matrix_f64_bits.len(), 120);
        assert_eq!(artifact.multiplication_indices.len(), 14_400);
        assert_eq!(artifact.inverse_indices.len(), 120);
        assert_eq!(artifact.connection_control_permutation.len(), 120);
        assert_eq!(artifact.h4_root_table_kappa, H4_SPIN_FRAME_ROOT_TABLE_KAPPA);
        assert_eq!(
            artifact.h4_multiplication_table_kappa,
            H4_SPIN_FRAME_PRODUCT_TABLE_KAPPA
        );
        assert_eq!(
            artifact.artifact_cid,
            artifact.reproduce_artifact_cid().unwrap()
        );

        let bytes = artifact.canonical_bytes().expect("canonical bytes");
        let reparsed = H4SpinFrameSidecarV1::from_canonical_bytes(&bytes)
            .expect("strict canonical round trip");
        assert_eq!(reparsed, artifact);
    }

    #[test]
    fn canonical_sidecar_rejects_matrix_or_product_tampering() {
        let artifact = H4SpinFrameSidecarV1::build().expect("canonical H4 sidecar");

        let mut matrix_tamper = artifact.clone();
        matrix_tamper.frame_matrix_f64_bits[0][0][0] ^= 1;
        matrix_tamper.artifact_cid = matrix_tamper.reproduce_artifact_cid().unwrap();
        assert!(matrix_tamper.validate().is_err());

        let mut product_tamper = artifact;
        product_tamper.multiplication_indices[0] =
            (product_tamper.multiplication_indices[0] + 1) % 120;
        product_tamper.artifact_cid = product_tamper.reproduce_artifact_cid().unwrap();
        assert!(product_tamper.validate().is_err());
    }
}
