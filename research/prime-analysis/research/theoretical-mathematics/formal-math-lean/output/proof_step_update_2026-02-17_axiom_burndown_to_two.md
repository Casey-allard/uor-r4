# Proof Step Update (2026-02-17): Axiom Burn-Down to Two

- Updated `research/formal/lean/PrimeRiemannBridge.lean` to remove global contract-import axioms:
  - removed `L1_contract_from_o1_o2_o4`
  - removed `L2_contract_from_o3`
- Rewired theorem chain to take explicit contract witnesses instead:
  - `L1_transfer_bound ... (k : L1ArtifactContract E H c1)`
  - `L2_bridge_bound ... (k : L2ArtifactContract H)`
  - `O5_final_implication ... (k1 : L1ArtifactContract E H c1) (k2 : L2ArtifactContract H)`
- Re-ran formal axiom audit:
  - blocker count reduced from 4 -> 2.
  - remaining blockers are only analytic side conditions:
    - `L0_log_sq_ge_one`
    - `L0_two_le_exp1`
