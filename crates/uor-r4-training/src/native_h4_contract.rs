//! Offline cross-crate binding of the standalone integer classifier's order.
//!
//! This checks exact coefficients, not floating classifier agreement or language.

use uor_r4_core::native_geometric::learner::prefix_artifact::historical_roots;
use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;

#[test]
fn historical_coefficients_match_exact_donor() {
    let donor = historical_roots();
    assert_eq!(H4_ROOT_COEFFICIENTS.len(), donor.len());
    for (index, (actual, expected)) in H4_ROOT_COEFFICIENTS.iter().zip(&donor).enumerate() {
        for coordinate in 0..4 {
            for coefficient in 0..2 {
                assert_eq!(
                    i64::from(actual[coordinate][coefficient]),
                    expected[coordinate][coefficient],
                    "historical root {index}, coordinate {coordinate}, coefficient {coefficient}"
                );
            }
        }
    }
    assert_eq!(H4_ROOT_COEFFICIENTS[1], [[2, 0], [0, 0], [0, 0], [0, 0]]);
}
