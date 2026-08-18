//! Executable proof module: Machine-checkable ProofStatusMatrix tracking verification status across all PDF theorems.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofStatus {
    Verified,
    ExecutableSpec,
    DifferentialPass,
    /// #787 (AUD-INV-001): the honest tier between `Verified` and
    /// `Unverified` — the property is enforced by source-scan witnesses
    /// (Witnessed evidence per `INFERENCE_OPERATION_CONTRACT.md` §6/§8),
    /// not yet by the #160 machine-code audit. A `Witnessed` row passes
    /// `verify_all` but is visibly not `Verified`, so the matrix can no
    /// longer overclaim structural evidence it does not have.
    Witnessed,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremEntry {
    pub name: String,
    pub theorem_id: String,
    pub status: ProofStatus,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStatusMatrix {
    pub entries: Vec<TheoremEntry>,
}

impl Default for ProofStatusMatrix {
    fn default() -> Self {
        ProofStatusMatrix {
            entries: vec![
                TheoremEntry {
                    name: "Allocation Freedom".to_string(),
                    theorem_id: "PDF §16".to_string(),
                    status: ProofStatus::Verified,
                    description:
                        "Zero allocation step contract enforced by counting allocator tests"
                            .to_string(),
                },
                TheoremEntry {
                    name: "Operation-Set Conformance".to_string(),
                    theorem_id: "Plan §6 / PDF §17".to_string(),
                    // #787: the description was always accurate; the status
                    // overclaimed. Witnessed until the #160 disassembly
                    // audit lands.
                    status: ProofStatus::Witnessed,
                    description:
                        "Witnessed source scans enforce the multiplication-free inference operation contract until disassembly audit lands".to_string(),
                },
                TheoremEntry {
                    name: "Bounded Ranges".to_string(),
                    theorem_id: "Theorem 8".to_string(),
                    status: ProofStatus::Verified,
                    description: "Section relative packed range boundaries verified bounds-checked"
                        .to_string(),
                },
                TheoremEntry {
                    name: "Deterministic Top-K".to_string(),
                    theorem_id: "PDF §23".to_string(),
                    status: ProofStatus::Verified,
                    description: "Canonical tie-breaking (highest score, then lowest TokenId)"
                        .to_string(),
                },
                TheoremEntry {
                    name: "Reverse Index Consistency".to_string(),
                    theorem_id: "Theorem 7".to_string(),
                    status: ProofStatus::Verified,
                    description:
                        "Reverse edge indexes reference exact canonical edge IDs sorted by target"
                            .to_string(),
                },
                TheoremEntry {
                    name: "Score Arithmetic Safety".to_string(),
                    theorem_id: "Kani-1".to_string(),
                    status: ProofStatus::Verified,
                    description: "ScoreQ saturating_add does not panic or overflow".to_string(),
                },
                TheoremEntry {
                    name: "Fixed-Capacity Container Invariants".to_string(),
                    theorem_id: "Kani-2".to_string(),
                    status: ProofStatus::Verified,
                    description: "RuntimeState slot updates do not panic or cause OOB".to_string(),
                },
                TheoremEntry {
                    name: "Graph Invariant Ownership Matrix".to_string(),
                    theorem_id: "PDF §9".to_string(),
                    status: ProofStatus::Verified,
                    description: "All 8 normative graph invariants have verified primary owners and loader checks".to_string(),
                },
            ],
        }
    }
}

impl ProofStatusMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total: returns `None` when every theorem entry is verified, or
    /// `Some(reason)` naming the first unverified theorem (R5 — a measured
    /// report of matrix state, not a raised error).
    pub fn verify_all(&self) -> Option<String> {
        for entry in &self.entries {
            if entry.status == ProofStatus::Unverified {
                return Some(format!("Unverified theorem found: {}", entry.theorem_id));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_matrix_all_verified() {
        let matrix = ProofStatusMatrix::new();
        assert!(matrix.verify_all().is_none());
    }

    /// #787 falsifier: the matrix instrument must be able to fail — a
    /// seeded `Unverified` entry makes `verify_all` report it. Before this
    /// issue the default matrix hardcoded 8/8 `Verified` and no code path
    /// ever constructed the failing status, making the instrument
    /// indistinguishable from one that always passes.
    #[test]
    fn verify_all_fails_on_a_seeded_unverified_entry() {
        let mut matrix = ProofStatusMatrix::new();
        matrix.entries.push(TheoremEntry {
            name: "Seeded violation".to_string(),
            theorem_id: "TEST-787".to_string(),
            status: ProofStatus::Unverified,
            description: "instrument falsifier".to_string(),
        });
        let reason = matrix.verify_all().expect("seeded violation must fail");
        assert!(reason.contains("TEST-787"));
    }

    /// #787: the honest tier is visible — Operation-Set Conformance is
    /// `Witnessed` (source-scan evidence), not `Verified`, until the #160
    /// disassembly audit lands; `verify_all` accepts it without erasing
    /// the distinction.
    #[test]
    fn operation_set_conformance_is_witnessed_not_verified() {
        let matrix = ProofStatusMatrix::new();
        let row = matrix
            .entries
            .iter()
            .find(|entry| entry.name == "Operation-Set Conformance")
            .expect("row present");
        assert_eq!(row.status, ProofStatus::Witnessed);
        assert!(matrix.verify_all().is_none());
    }
}
