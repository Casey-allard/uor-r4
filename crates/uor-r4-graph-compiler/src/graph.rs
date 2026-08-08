//! Graph construction (Phase 2 & 4) --- a dormant alternative entry point.
//!
//! This standalone builder (refinement edges, lateral neighbour edges, forward
//! transition edges) was never wired: the shipped compiler constructs graphs
//! through `induction`, not here. It is retained rather than deleted because a
//! distinct construction path is a mechanism worth being able to return to, and
//! kept honest by registration --- the `open` claim `graph-construction-dormant`
//! in model/ledger.toml declares it dormant behind its activation gate (R4 is
//! satisfied by registration, not by removal).

/// Dormant alternative graph-construction entry point (open:
/// graph-construction-dormant). Not on any shipped path; calling it is a
/// declared no-op-until-activated rather than a supported operation.
pub fn build_graph() {
    // The marker is sanctioned by the open claim cited on the same line (R4
    // reads per line, so the citation and the marker must stay together).
    unimplemented!("dormant - open: graph-construction-dormant")
}
