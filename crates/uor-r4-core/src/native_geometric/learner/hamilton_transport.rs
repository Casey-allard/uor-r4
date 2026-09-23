//! Exact Q8 left actions on vectors, not quaternion dense neural attention.
//! For a pure unit axis K, K^T=-K and K^2=-I. The complex operator H=iK is
//! Hermitian; exp(-i*pi*H/2)=K. Serving uses the finite signed permutation only.
#![forbid(unsafe_code)]
pub fn apply(action: u8, x: [i32; 4]) -> Result<[i32; 4], String> {
    if action >= 8 {
        return Err("Q8 action outside 0..8".into());
    }
    let (indices, signs) = match action >> 1 {
        0 => ([0, 1, 2, 3], [false, false, false, false]),
        1 => ([1, 0, 3, 2], [true, false, true, false]),
        2 => ([2, 3, 0, 1], [true, false, false, true]),
        _ => ([3, 2, 1, 0], [true, true, false, false]),
    };
    let mut y = [0; 4];
    for j in 0..4 {
        let a = x[indices[j]];
        y[j] = if signs[j] ^ (action & 1 != 0) {
            a.checked_neg().ok_or("signed transport overflow")?
        } else {
            a
        };
    }
    Ok(y)
}
pub fn inverse(action: u8) -> Result<u8, String> {
    if action >= 8 {
        return Err("Q8 action outside 0..8".into());
    }
    Ok(if action < 2 { action } else { action ^ 1 })
}
/// Query-relative value transport; frame identities remain external exact data.
pub fn relative(query: u8, key: u8, value: [i32; 4]) -> Result<[i32; 4], String> {
    apply(inverse(query)?, apply(key, value)?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imaginary_actions_square_to_minus_identity() {
        let x = [1, 2, 3, 4];
        for a in [2, 4, 6] {
            assert_eq!(apply(a, apply(a, x).unwrap()).unwrap(), [-1, -2, -3, -4]);
        }
    }
    #[test]
    fn left_actions_do_not_commute() {
        let x = [1, 0, 0, 0];
        assert_ne!(
            apply(2, apply(4, x).unwrap()).unwrap(),
            apply(4, apply(2, x).unwrap()).unwrap()
        );
    }
    #[test]
    fn reject_overflow_before_sign_change() {
        assert!(apply(1, [i32::MIN, 0, 0, 0]).is_err());
        assert!(apply(8, [0; 4]).is_err());
    }
    #[test]
    fn inverses_and_norms_hold_for_all_actions() {
        for a in 0..8 {
            for n in -64..64 {
                let x = [n, 2 * n + 1, 3 * n - 2, -n];
                let y = apply(a, x).unwrap();
                assert_eq!(apply(inverse(a).unwrap(), y).unwrap(), x);
                assert_eq!(
                    x.iter().map(|x| i64::from(*x).pow(2)).sum::<i64>(),
                    y.iter().map(|x| i64::from(*x).pow(2)).sum::<i64>()
                );
            }
        }
    }
    #[test]
    fn common_rotation_does_not_create_attention_scores() {
        let q = [1, 2, 3, 4];
        let k = [4, 0, -2, 1];
        let dot = |x: [i32; 4], y: [i32; 4]| x.iter().zip(y).map(|(a, b)| a * b).sum::<i32>();
        for a in 0..8 {
            assert_eq!(dot(q, k), dot(apply(a, q).unwrap(), apply(a, k).unwrap()));
        }
    }
    #[test]
    fn query_relative_operation_uses_both_frames() {
        let v = [1, 2, 3, 4];
        assert_eq!(relative(2, 2, v).unwrap(), v);
        assert_ne!(relative(2, 4, v).unwrap(), relative(4, 2, v).unwrap());
    }
}
