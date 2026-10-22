//! Exact legal-construction solver (#2828, unit 2).
//!
//! Every number is an arbitrary-precision rational. Binary floating point
//! appears only in `exact`, which converts saved input bits to rationals.
//!
//! The program: minimise `g·δ` subject to `J_j·δ >= 0` for every guard row,
//! where `δ_i = k_i/4` for an on-grid coefficient and
//! `δ_i = k_i/4 + (l_i + r_i)(q0_i/4 - m_i)` for a fractional one, with the
//! legal-code bounds and disjunction rows of the design note.

pub mod bb;
pub mod brute;
pub mod check;
pub mod exact;
pub mod lp;
pub mod problem;
