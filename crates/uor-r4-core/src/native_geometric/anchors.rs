//! Exact compiler-side anchors reused from the canonical H4/icosian profile.
//!
//! Class numbers are equality addresses for complete exact coefficient tuples.
//! Their numeric differences are not distances. The paired coefficients are a
//! typed, invertible representation of one H4 root; they do not add independent
//! state or establish an orthogonal Euclidean E8 root embedding. Every root has
//! the same radius. A varying window radius must be computed from the actual
//! accumulated coefficients, not fabricated by assigning different root radii.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::canonical_lexical_ingestion::{
    canonical_icosian_anchor_table, CanonicalLexicalError, H4BinaryIcosahedralClosure,
};
use crate::prime_route_attention::ZPhi;

const ANCHOR_DOMAIN: &str = "uor-r4.native-h4-icosian-anchors/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AnchorRow {
    pub root_index: u16,
    /// Coordinates are (a+b*phi)/2 in the ordered (1,i,j,k) basis.
    pub root_scaled_zphi: [[i64; 2]; 4],
    /// [a0,a1,a2,a3,b0,b1,b2,b3] in the existing integral coefficient basis.
    pub paired_coefficients: [i64; 8],
    /// Exact phi*Galois(a+b*phi), represented by (-b,a).
    pub phi_galois_companion: [[i64; 2]; 4],
    pub coordinate_kappa: String,
    /// Numerator of the root squared norm; denominator is four. Always (4,0).
    pub root_norm_squared: [i64; 2],
    /// Exact squared radius of the (q0,q1) projection, also divided by four.
    /// This is a projection magnitude, not a different full-root radial shell.
    pub projection_radius_squared: [i64; 2],
    /// Existing signed heatmap convention: sin=q0, cos=q1; activation=sin^2.
    pub activation_squared: [i64; 2],
    pub chirality: i8,
    pub cosine_polarity: i8,
    pub projection_is_null: bool,
    pub paired_class: u16,
    pub radial_class: u16,
    pub orientation_class: u16,
    pub projection_radius_class: u16,
    pub heatmap_class: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AnchorTable {
    pub domain: String,
    pub root_table_kappa: String,
    pub multiplication_table_kappa: String,
    pub icosian_profile_kappa: String,
    pub icosian_operator_table_kappa: String,
    pub coordinate_scale_denominator: u8,
    pub rows: Vec<AnchorRow>,
}

/// Derive all fixed rows and their exact equality classes before inference.
/// Existing canonical profile identities and inverse checks are reused; no
/// corpus ingestion, prime reassignment or new hash-derived geometry occurs.
pub(super) fn compile_anchor_table(
    table: &H4BinaryIcosahedralClosure,
) -> Result<AnchorTable, CanonicalLexicalError> {
    let canonical = canonical_icosian_anchor_table(table)?;
    let mut rows = Vec::with_capacity(canonical.rows.len());
    for root in canonical.rows {
        let sine = ZPhi::new(root.root_scaled_zphi[0][0], root.root_scaled_zphi[0][1]);
        let cosine = ZPhi::new(root.root_scaled_zphi[1][0], root.root_scaled_zphi[1][1]);
        let activation = sine.checked_mul(sine)?;
        let projection = activation.checked_add(cosine.checked_mul(cosine)?)?;
        let chirality = exact_sign(sine)?;
        let cosine_polarity = exact_sign(cosine)?;
        let orientation_class =
            u16::try_from((i16::from(chirality) + 1) * 3 + i16::from(cosine_polarity) + 1)
                .map_err(|_| CanonicalLexicalError::ArithmeticOverflow)?;
        rows.push(AnchorRow {
            root_index: root.root_index,
            root_scaled_zphi: root.root_scaled_zphi,
            paired_coefficients: root.paired_coefficients,
            phi_galois_companion: root.phi_galois_companion,
            coordinate_kappa: root.coordinate_kappa,
            root_norm_squared: root.root_norm_squared,
            projection_radius_squared: [projection.a, projection.b],
            activation_squared: [activation.a, activation.b],
            chirality,
            cosine_polarity,
            projection_is_null: sine == ZPhi::new(0, 0) && cosine == ZPhi::new(0, 0),
            paired_class: 0,
            radial_class: 0,
            orientation_class,
            projection_radius_class: 0,
            heatmap_class: 0,
        });
    }
    let paired = classes(
        rows.iter()
            .map(|row| (row.paired_coefficients, row.phi_galois_companion)),
    );
    let radial = classes(rows.iter().map(|row| row.root_norm_squared));
    let projected = classes(rows.iter().map(|row| row.projection_radius_squared));
    let heatmap = classes(
        rows.iter()
            .map(|row| (row.root_scaled_zphi[0], row.root_scaled_zphi[1])),
    );
    for row in &mut rows {
        row.paired_class = class_index(
            &paired,
            &(row.paired_coefficients, row.phi_galois_companion),
        )?;
        row.radial_class = class_index(&radial, &row.root_norm_squared)?;
        row.projection_radius_class = class_index(&projected, &row.projection_radius_squared)?;
        row.heatmap_class = class_index(
            &heatmap,
            &(row.root_scaled_zphi[0], row.root_scaled_zphi[1]),
        )?;
    }
    Ok(AnchorTable {
        domain: ANCHOR_DOMAIN.to_owned(),
        root_table_kappa: table.h4_root_table_kappa.clone(),
        multiplication_table_kappa: table.multiplication_table_kappa.clone(),
        icosian_profile_kappa: canonical.profile_kappa,
        icosian_operator_table_kappa: canonical.operator_table_kappa,
        coordinate_scale_denominator: 2,
        rows,
    })
}

fn classes<T: Ord>(values: impl Iterator<Item = T>) -> Vec<T> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}

fn class_index<T: Ord>(classes: &[T], value: &T) -> Result<u16, CanonicalLexicalError> {
    let index = classes
        .binary_search(value)
        .map_err(|_| CanonicalLexicalError::Invalid("exact anchor class is missing".to_owned()))?;
    u16::try_from(index).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)
}

/// Sign of a+b*phi using 2(a+b*phi)=(2a+b)+b*sqrt(5). The only
/// comparisons of unlike signs use checked squared integer magnitudes.
fn exact_sign(value: ZPhi) -> Result<i8, CanonicalLexicalError> {
    let a = i128::from(value.a);
    let b = i128::from(value.b);
    if a == 0 && b == 0 {
        return Ok(0);
    }
    let rational = a
        .checked_mul(2)
        .and_then(|v| v.checked_add(b))
        .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
    let sign = if rational == 0 {
        b.signum()
    } else if b == 0 || rational.signum() == b.signum() {
        rational.signum()
    } else {
        let rational_squared = rational
            .checked_mul(rational)
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        let irrational_squared = b
            .checked_mul(b)
            .and_then(|v| v.checked_mul(5))
            .ok_or(CanonicalLexicalError::ArithmeticOverflow)?;
        match rational_squared.cmp(&irrational_squared) {
            std::cmp::Ordering::Greater => rational.signum(),
            std::cmp::Ordering::Less => b.signum(),
            std::cmp::Ordering::Equal => {
                return Err(CanonicalLexicalError::Invalid(
                    "nonzero irrational anchor unexpectedly has zero sign".to_owned(),
                ))
            }
        }
    };
    i8::try_from(sign).map_err(|_| CanonicalLexicalError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical_lexical_ingestion::validate_h4_binary_icosahedral_closure;

    #[test]
    fn all_roots_retain_exact_paired_inverse_and_unit_radius() {
        let closure = validate_h4_binary_icosahedral_closure().unwrap();
        let anchors = compile_anchor_table(&closure).unwrap();
        assert_eq!(anchors.rows.len(), 120);
        assert_eq!(anchors.coordinate_scale_denominator, 2);
        let mut classes = BTreeSet::new();
        for row in &anchors.rows {
            classes.insert(row.paired_class);
            assert_eq!(row.root_norm_squared, [4, 0]);
            assert_eq!(row.radial_class, 0);
            for index in 0..4 {
                let [a, b] = row.root_scaled_zphi[index];
                let value = ZPhi::new(a, b);
                assert_eq!(row.paired_coefficients[index], a);
                assert_eq!(row.paired_coefficients[index + 4], b);
                assert_eq!(row.phi_galois_companion[index], [-b, a]);
                let companion = value.golden_conjugate().unwrap().times_phi().unwrap();
                assert_eq!(companion, ZPhi::new(-b, a));
                assert_eq!(
                    companion
                        .times_phi_inverse()
                        .unwrap()
                        .golden_conjugate()
                        .unwrap(),
                    value
                );
                assert_eq!(
                    value.times_phi().unwrap().times_phi_inverse().unwrap(),
                    value
                );
            }
        }
        assert_eq!(classes.len(), 120);
        assert!(anchors
            .rows
            .iter()
            .any(|row| row.root_scaled_zphi.iter().any(|v| v[1] != 0)));
    }

    #[test]
    fn exact_signed_heatmap_preserves_antipodes_and_projection_scope() {
        let closure = validate_h4_binary_icosahedral_closure().unwrap();
        let anchors = compile_anchor_table(&closure).unwrap();
        for row in &anchors.rows {
            let negative = row.root_scaled_zphi.map(|v| [-v[0], -v[1]]);
            let antipode = anchors
                .rows
                .iter()
                .find(|other| other.root_scaled_zphi == negative)
                .unwrap();
            assert_eq!(row.activation_squared, antipode.activation_squared);
            assert_eq!(
                row.projection_radius_squared,
                antipode.projection_radius_squared
            );
            assert_eq!(row.chirality, -antipode.chirality);
            assert_eq!(row.cosine_polarity, -antipode.cosine_polarity);
            assert_ne!(row.paired_class, antipode.paired_class);
        }
        assert!(anchors.rows.iter().any(|row| row.projection_is_null));
        assert!(
            anchors
                .rows
                .iter()
                .map(|row| row.projection_radius_class)
                .collect::<BTreeSet<_>>()
                .len()
                > 1
        );
        assert_eq!(
            anchors
                .rows
                .iter()
                .map(|row| row.heatmap_class)
                .collect::<BTreeSet<_>>()
                .len(),
            45
        );
    }

    #[test]
    fn anchor_classes_and_wire_roundtrip_are_deterministic_and_bound_to_root_order() {
        let mut closure = validate_h4_binary_icosahedral_closure().unwrap();
        let first = compile_anchor_table(&closure).unwrap();
        let second = compile_anchor_table(&closure).unwrap();
        let bytes = serde_json::to_vec(&first).unwrap();
        assert_eq!(bytes, serde_json::to_vec(&second).unwrap());
        assert_eq!(
            serde_json::from_slice::<AnchorTable>(&bytes).unwrap(),
            first
        );
        closure.h4_root_table_kappa.push('0');
        assert!(compile_anchor_table(&closure).is_err());
    }

    #[test]
    fn coefficient_sign_uses_exact_golden_comparison_and_checked_bounds() {
        for (value, expected) in [
            (ZPhi::new(0, 0), 0),
            (ZPhi::new(-1, 1), 1),
            (ZPhi::new(1, -1), -1),
            (ZPhi::new(2, -1), 1),
            (ZPhi::new(-2, 1), -1),
        ] {
            assert_eq!(exact_sign(value).unwrap(), expected);
        }
        assert!(exact_sign(ZPhi::new(i64::MAX, i64::MIN)).is_err());
    }
}
