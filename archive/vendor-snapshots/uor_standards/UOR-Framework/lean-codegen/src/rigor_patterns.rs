//! v0.2.1 Phase 8b.6: single source of truth for Lean rigor bans.
//!
//! Every banned primitive in the published Lean surface is listed here.
//! Both the codegen-time sanitizer ([`crate::emit::sanitize_lean_line`])
//! and the conformance-time validator
//! (`conformance/src/validators/lean4/structure.rs::audit_sorry`) read
//! from this table, so the two enforcement layers cannot drift.
//!
//! To propose a new exception, edit this file and justify the change in
//! the commit message. Adding a new entry automatically tightens both
//! the codegen sanitizer and the conformance validator.

/// Banned textual primitives. Each entry is `(pattern_substring, reason)`.
/// Matching is naïve byte-level substring; line comments are stripped
/// before matching so `-- TODO: avoid sorry` doesn't false-positive.
pub const BANNED_PATTERNS: &[(&str, &str)] = &[
    (" sorry", "`sorry` leaves a hole in the proof"),
    ("\tsorry", "`sorry` leaves a hole in the proof"),
    (":= sorry", "`sorry` leaves a hole in the proof"),
    ("partial def", "`partial def` is non-reducible"),
    ("native_decide", "`native_decide` trusts native code"),
    (" unsafe ", "`unsafe` is banned in the published surface"),
    ("\tunsafe ", "`unsafe` is banned in the published surface"),
    ("@[extern", "`@[extern]` delegates to a native symbol"),
    (
        "@[implemented_by",
        "`@[implemented_by]` substitutes runtime code",
    ),
];

/// The single whitelisted `axiom` identifier — Phase 7g.3 sealed provenance.
/// Every other `axiom` declaration is rejected by both enforcement layers.
pub const ALLOWED_AXIOM: &str = "UOR_SEALED_PROVENANCE";
