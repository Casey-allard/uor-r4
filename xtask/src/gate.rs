//! The activation-gate registry (`cargo xtask gate <claim-id>` / `gates`).
//!
//! #515 preserve-and-gate keeps every ruled-out or dormant mechanism registered
//! in `model/ledger.toml` as an `open` claim, and every `open` claim names an
//! *activation gate*: the pre-declared measurement that, once cleared, would
//! promote the mechanism off the shelf and back onto the serving path. This
//! module turns that prose register into a runnable one.
//!
//! `gates` lists every open claim, extracts its activation gate, classifies the
//! kind of measurement the gate demands, and reports its status. `gate <id>`
//! does the same for one claim. Every open claim is by definition NOT CLEARED —
//! an `open` claim is measured and reported, never asserted (R2); the moment a
//! gate is cleared the mechanism is activated and the claim leaves `open`. So
//! the honest verdict here is always `NOT CLEARED (dormant)`, and the value of
//! the command is that it prints *exactly what would clear each gate*, in one
//! place, re-runnable, so a later run knows precisely what to measure.
//!
//! The per-claim measurement runners are the extension point: as a mechanism's
//! measurement harness is wired (against the Gate C fixtures under
//! `crates/uor-r4-core/tests/fixtures`), its runner attaches here keyed by claim
//! id and this command begins reporting a live `CLEARED` / `NOT CLEARED` verdict
//! for it instead of the declared-bar reminder.

use std::path::Path;

use repo_model::{registry::Level, Model};

use crate::Fail;

/// What kind of measurement a claim's activation gate demands. The distinction
/// matters because the two are cleared by different things: a mechanism gate is
/// cleared by *running the mechanism* and beating a baseline, a governance gate
/// by a *decision* to adopt a metric as binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GateKind {
    /// "… scoring at or above the shipped path on a pre-declared held-out
    /// slice", "a measured lift clearing the pre-declared bar", "a positive
    /// contribution over a pre-declared null". Cleared by activating the dormant
    /// mechanism and measuring it against the shipped baseline.
    MechanismComparison,
    /// "… adopted as a required gate on a pre-declared bar". Cleared by a
    /// governance decision to promote a measurement to a binding conformance
    /// gate, not by running the mechanism.
    MetricAdoption,
}

impl GateKind {
    fn label(self) -> &'static str {
        match self {
            GateKind::MechanismComparison => "mechanism-comparison",
            GateKind::MetricAdoption => "metric-adoption",
        }
    }
}

/// Pull the activation-gate clause out of a claim statement. Claims phrase it as
/// "Activation gate: …", "whose activation gate is …", "Its activation gate is
/// …", or "behind its activation gate". Returns the text from the phrase to the
/// end of the statement, trimmed; `None` if the statement names no gate.
fn activation_gate(statement: &str) -> Option<&str> {
    let lower = statement.to_ascii_lowercase();
    // Prefer the LAST mention: statements that reference the gate in passing
    // ("behind its activation gate (#159): …") before stating it explicitly
    // ("… Activation gate: <the bar>") should report the explicit final clause.
    let anchor = lower.rfind("activation gate")?;
    let tail = &statement[anchor..];
    // Explicit "Activation gate:" form — the clause is everything after the
    // colon that immediately follows the phrase (allowing a short parenthetical).
    if let Some(colon) = tail.find(':') {
        // Only treat the colon as the gate separator if it is close to the
        // phrase (same clause), not a colon sentences away.
        if colon <= "activation gate".len() + 24 {
            let clause = tail[colon + 1..].trim();
            if !clause.is_empty() {
                return Some(clause);
            }
        }
    }
    // Otherwise take from the anchor to the end (the "… gate is X" prose form).
    Some(tail.trim())
}

/// Classify a gate clause. A gate that talks about scoring against the shipped
/// path, a measured lift, an improvement, or a positive contribution is a
/// mechanism comparison; a gate that talks about *adopting* a metric as a
/// required/conformance gate is a governance decision. When a clause reads both
/// ways, the mechanism reading wins — it is the stronger, runnable bar.
fn classify(gate_clause: &str) -> GateKind {
    let g = gate_clause.to_ascii_lowercase();
    let mechanism = g.contains("at or above")
        || g.contains("scoring at")
        || g.contains("measured lift")
        || g.contains("improves")
        || g.contains("improve top")
        || g.contains("positive")
        || g.contains("clearing gate")
        || g.contains("clears gate")
        || g.contains("recall result clearing")
        || g.contains("j(c) result clearing");
    if mechanism {
        GateKind::MechanismComparison
    } else if g.contains("adopt") {
        GateKind::MetricAdoption
    } else {
        // No adoption language and no explicit comparison verb: treat as a
        // mechanism comparison, the bar that must actually be run.
        GateKind::MechanismComparison
    }
}

/// One line of the registry for one open claim.
fn report_claim(id: &str, statement: &str) {
    println!("claim: {id}");
    match activation_gate(statement) {
        Some(clause) => {
            let kind = classify(clause);
            println!("  kind:   {}", kind.label());
            println!("  status: NOT CLEARED (dormant — no activation record)");
            println!("  gate:   {clause}");
            match kind {
                GateKind::MechanismComparison => println!(
                    "  clears: activate the mechanism and measure it against the \
                     shipped baseline on the pre-declared slice; the bar above must hold."
                ),
                GateKind::MetricAdoption => println!(
                    "  clears: a governance decision to adopt this measurement as a \
                     binding conformance gate at the pre-declared bar."
                ),
            }
        }
        None => {
            println!("  kind:   unspecified");
            println!("  status: NOT CLEARED (dormant — statement names no activation gate)");
            println!("  gate:   <none declared>");
        }
    }
}

/// `cargo xtask gate [<claim-id>]` / `gates`.
///
/// With `id = None` (`gates`) report every open claim. With `id = Some(x)`
/// (`gate x`) report just that claim, erroring if no open claim carries the id.
/// Reporting succeeds (exit 0) — this is a status register, not a gate that
/// fails CI; only an unknown id is an error.
pub fn gate(root: &Path, id: Option<&str>) -> Result<(), Fail> {
    let model = Model::load(&root.join("model"))?;
    let open: Vec<&repo_model::registry::Claim> = model
        .ledger
        .claim
        .iter()
        .filter(|c| c.level == Level::Open)
        .collect();

    match id {
        Some(want) => {
            let Some(c) = open.iter().find(|c| c.id == want) else {
                return Err(format!(
                    "no open claim with id `{want}`.\n\
                     Run `cargo xtask gates` to list the {} registered activation gates.",
                    open.len()
                )
                .into());
            };
            report_claim(&c.id, &c.statement);
        }
        None => {
            println!(
                "activation-gate registry (#515): {} open claims, all dormant \
                 until their gate is cleared.\n",
                open.len()
            );
            for c in &open {
                report_claim(&c.id, &c.statement);
                println!();
            }
            let mechanism = open
                .iter()
                .filter(|c| {
                    activation_gate(&c.statement)
                        .map(|g| classify(g) == GateKind::MechanismComparison)
                        .unwrap_or(false)
                })
                .count();
            let adoption = open
                .iter()
                .filter(|c| {
                    activation_gate(&c.statement)
                        .map(|g| classify(g) == GateKind::MetricAdoption)
                        .unwrap_or(false)
                })
                .count();
            println!(
                "summary: {} open claim(s) — {mechanism} mechanism-comparison, \
                 {adoption} metric-adoption. None cleared (an open claim is \
                 dormant by definition).",
                open.len()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_explicit_gate_clause() {
        let s = "A dormant thing is retained. Activation gate: a widget scoring \
                 at or above the shipped store on a pre-declared held-out slice.";
        let clause = activation_gate(s).expect("gate present");
        assert!(clause.starts_with("a widget scoring at or above"));
        assert_eq!(classify(clause), GateKind::MechanismComparison);
    }

    #[test]
    fn extracts_prose_gate_form() {
        let s = "The operator is kept dormant; its activation gate is a measured \
                 lift clearing the pre-declared bar.";
        let clause = activation_gate(s).expect("gate present");
        assert!(clause.to_ascii_lowercase().contains("activation gate is"));
        assert_eq!(classify(clause), GateKind::MechanismComparison);
    }

    #[test]
    fn classifies_metric_adoption() {
        let g = "a divergence metric adopted as a required conformance gate on a \
                 pre-declared threshold.";
        assert_eq!(classify(g), GateKind::MetricAdoption);
    }

    #[test]
    fn no_gate_declared() {
        assert!(activation_gate("A claim with no gate sentence at all.").is_none());
    }
}
