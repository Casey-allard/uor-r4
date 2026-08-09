//! Executable proof module: Packed section range boundary verification.

/// Verify that packed section range `(start, len)` is bounded within section
/// length `total_len` (i.e., `start + len <= total_len`). Total: returns `None`
/// when the range is within bounds, or `Some(reason)` describing the overflow or
/// bounds violation (R5 — a failed bound is the absence of a valid range, a
/// measured report, not an error the model raises).
pub fn verify_range_bounds(
    start: usize,
    len: usize,
    total_len: usize,
    range_name: &str,
) -> Option<String> {
    let Some(end) = start.checked_add(len) else {
        return Some(format!("Range overflow in {range_name}"));
    };
    if end > total_len {
        Some(format!(
            "Range bounds violation in {range_name}: range [{start}..{end}] exceeds section length {total_len}"
        ))
    } else {
        None
    }
}
