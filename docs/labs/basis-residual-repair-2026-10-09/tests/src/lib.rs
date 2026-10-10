#[cfg(test)]
mod tests {
    use microlp::repair::qualify_saved_basis;
    #[test]
    fn deliberate_wrong_factors_rejected() {
        microlp::repair::unacceptable_factor_fixture().unwrap();
    }
    #[test]
    fn small_pivot_rescaling_and_all_solve_paths() {
        for s in [1.0, 1e-90, 1e90] {
            let v = qualify_saved_basis(&[vec![(0, s)], vec![(1, s * 1e-11)]]).unwrap();
            assert_eq!(v.dimension, 2);
            assert_eq!(v.rhs_checked_per_direction, 2);
            for e in [
                v.max_normal_error,
                v.max_transpose_error,
                v.max_sparse_normal_error,
                v.max_sparse_transpose_error,
            ] {
                assert!(e <= v.tolerance)
            }
        }
    }
    #[test]
    fn nonsymmetric_transpose_and_sparse_zero_solution() {
        let v = qualify_saved_basis(&[vec![(1, 2.0)], vec![(0, 3.0), (1, 4.0)]]).unwrap();
        for e in [
            v.max_normal_error,
            v.max_transpose_error,
            v.max_sparse_normal_error,
            v.max_sparse_transpose_error,
        ] {
            assert!(e <= v.tolerance)
        }
    }
    #[test]
    fn nonfinite_solve_and_denominator_arithmetic_rejected() {
        assert!(qualify_saved_basis(&[vec![(0, 1e-320)]]).is_err());
        assert!(qualify_saved_basis(&[vec![(0, 1e308)], vec![(1, 1e-308)]]).is_err());
    }
    #[test]
    fn singular_malformed_nonfinite_rejected() {
        assert!(qualify_saved_basis(&[vec![(0, 1.0)], vec![(0, 1.0)]]).is_err());
        assert!(qualify_saved_basis(&[vec![(0, f64::NAN)]]).is_err());
        assert!(qualify_saved_basis(&[vec![(1, 1.0)]]).is_err());
        assert!(qualify_saved_basis(&[vec![(0, 1.0), (0, 2.0)]]).is_err());
    }
}
