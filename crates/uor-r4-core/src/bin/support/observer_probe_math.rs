//! Offline diagnostic arithmetic; not part of a served model.
pub fn peel(h: &[i32], embedding: &[i32], offset: &[i32]) -> Vec<i32> {
    assert_eq!(h.len(), embedding.len());
    assert_eq!(h.len(), offset.len());
    h.iter()
        .zip(embedding)
        .zip(offset)
        .map(|((&h, &e), &b)| h - e - b)
        .collect()
}
pub fn nearest(x: &[i32], dictionary: &[Vec<i32>]) -> usize {
    dictionary
        .iter()
        .enumerate()
        .min_by_key(|(_, row)| {
            assert_eq!(x.len(), row.len());
            x.iter()
                .zip(row.iter())
                .map(|(&a, &b)| {
                    let d = i64::from(a) - i64::from(b);
                    d * d
                })
                .sum::<i64>()
        })
        .map(|(i, _)| i)
        .expect("nonempty diagnostic dictionary")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observer_transport_removes_current_embedding_and_known_offset() {
        let residual = peel(&[11, 6], &[10, 0], &[1, 2]);
        assert_eq!(residual, vec![0, 4]);
        assert_eq!(nearest(&residual, &[vec![4, 0], vec![0, 4]]), 1);
    }
    #[test]
    fn observer_transport_nearest_ties_use_first_identity() {
        assert_eq!(nearest(&[0, 0], &[vec![1, 0], vec![-1, 0]]), 0);
    }
}
