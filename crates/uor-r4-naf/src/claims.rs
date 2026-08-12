//! GNAF claim-class vocabulary and dual status reporting (#623 adoption of
//! `uor-gnaf/1-draft.1` §12.4–§12.5 as r4 witness-label VOCABULARY).
//!
//! This module is discipline, not machinery: the enums exist so that r4
//! result records can name the strongest *honest* claim class instead of an
//! unscoped "optimal" (which §12.4 makes a nonconforming label), and so that
//! execution status and optimization status are reported independently — a
//! run that was requested global-optimal but delivered best-known reports
//! `Accepted` + `OptimizationIncomplete` + `BestKnown`, and MUST NOT report
//! the requested global claim as accepted (§12.5 worked example).
//!
//! Wiring these labels into `graph-certify` witness output is deliberately
//! NOT part of #623 — it would touch claim/ledger wording governance (#515);
//! adopt per-site with the team's word.

/// GNAF §12.4 claim classes, strongest-to-weakest ordering not implied.
/// `universal-global-optimal`, "optimal for all future additions", and
/// unscoped "optimal for arbitrary operations" are nonconforming labels and
/// deliberately have no variant here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimClass {
    Exact,
    NormalForm,
    Canonical,
    RepresentationMinimal,
    /// Scoped: exact observation, machine, candidate universe, resource
    /// envelope, and objective — never unscoped.
    GlobalOptimal,
    ParetoOptimal,
    FrontierComplete,
    FamilyOptimal,
    RestrictedUniverseOptimal,
    RevisionPreserved,
    BestKnown,
    MeasuredBestAmongTested,
    HeuristicSelected,
}

/// GNAF §12.5 execution status — reported independently of optimization
/// status; the pairing is the honesty mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    NotRun,
    Accepted,
    Invalid,
    Unresolved,
    Unsupported,
    Unadmitted,
    Incoherent,
    Unsealed,
}

/// GNAF §12.5 optimization status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationStatus {
    NotRequested,
    Certified,
    Infeasible,
    Unattained,
    OptimizationIncomplete,
}
