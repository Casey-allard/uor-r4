**Completeness critique of the final assessment and synthesis (spot-checked at `71b54adf`)**

**1. Claims that are unverified or overstated**
- **"Recall becomes robust."** It rests on few seeds. In #1701 every arm ran on seed 1 only. #1704 and the #820 correction (2026-10-05T00:36Z) add seed 2. Two seeds on a 1.36M-parameter synthetic bench do not establish robustness, and the assessment's own rule asks for at least 3 seeds per bench verdict.
- **"`rararr` = 1.000 with no shift."** The #820 correction table I read shows only `rrarra` (seed 1 fails, seed 2 0.9995) and all-read. Cite the exact #1704 row and seed count, or drop it.
- **"The production conv route already supplies the same signal."** This is H, not M. No probe has shown an `r`-layer conv output carrying k_{t−1} into a downstream `a` key. The 96M pattern `rrarrarrarrarr` is not `rararr`/`rrarra`, and the 8M rung was `rrarra` with a Lorentz read (#820, 2026-10-02T16:09Z). The bench result does not transfer to the rungs without a probe at each rung.
- **Unverified citations.** The #1691 shares (weight maps 84.7%, read about 10%), arXiv:2512.17351 (Canon), arXiv:2609.15545 and arXiv:2609.16183 were never checked. Step 0d's routing of D5 depends on the #1691 figure.
- **Code ownership conflict.** AGENTS.md says `crates/uor-r4-core/src/native_geometric/` is the current model implementation. The synthesis names `uor-r4-training/src/geometric_stack.rs`. This is an unflagged authority conflict.

**2. A mechanistic point the plan misses (query-side lineage)**
- The #1701 probe shows that arm A's heads put weight 0.23 on the **value** position (q−d+1) and about 0.02 on the key position. That is the induction-head signature (Olsson et al., arXiv:2209.11895): find the token whose predecessor matches the current token.
- In D19 the query ends in "{k} is". The predicting position holds "is", not k. Retrieval therefore needs predecessor content on the **query** side as well, or a two-hop match.
- #1069 tried query-side lineage only. F2 is key-side only. The plan has no arm with both: q_t + j⁻¹q_{t−1} together with k_t + j·k_{t−1}.
- Step 2's gap arms (g ∈ {0..3}) will confound "key lag" with "query lag" unless that arm is added.

**3. External prior art not considered**
- RWKV token-shift (arXiv:2305.13048) mixes x_t with x_{t−1} before K/V. It is the nearest prior art to F2 and should be the learned control, not only W_prev.
- Short convolutions on q/k/v: Based (arXiv:2402.18668) and Mamba's conv1d before B/C (arXiv:2312.00752).
- Zoology's MQAR analysis (arXiv:2312.04927) gives a theory of how dimension scales with the number of KV pairs. It predicts where all-read L2 fails at d200.
- Delta-rule and gated-DeltaNet state recall (arXiv:2406.06484, arXiv:2412.06464). #999 retired gated-delta on tiny panels, and nothing re-tests it with lineage keys.
- Each of these predicts that F2 is a known mechanism. Without them, the "canonical" framing has no novelty baseline.

**4. Repository mechanisms the assessment ignores**
- The Codex causal geometric bank-attention line: #1699, #1700, #1702, #1703, plus #1670 ("Learn geometric context and isolate its native reply regression"). Only #1705 is dismissed.
- The `PrimeRoute` pointer and the 2I transport snap, as arms rather than footnotes.
- The 2026-10-05T02:04Z #820 review of Muse's prime-gcd index. It is directly relevant to the Step 6 "interned admission" choice.
- The `uor-r4-router` crate and the frozen TLA/R4G1 kernel. Neither appears in the ledger.

**5. Decision rules that cannot decide or that collide**
- **Step 1.** F2@high = 15 with off@high ≤ 5 meets both "promote" (≥ 10 gap) and "kill" (both ≤ 15). Gaps of 6–9 have no defined action. McNemar is applied to two seeds without saying whether they are pooled or tested per seed.
- **Step 2.** "none ≥ 0.95 → stop lineage" and "F2 ≥ 0.95 → sufficient" can both fire, and no order is given. The learned-conv-within-0.03 retraction is uninformative when "none" also passes.
- **Step 0a and 0c.** The 15% threshold has no stated basis. "If (ii) is high" has no number. The taxonomy is labelled by qwen2.5:7b, the same judge the panel uses, with no human-agreement check, although §6 names this exact risk.
- **M1 (≥ 30/109) against Step 1's kill (≤ 15).** The band from 16 to 29 has no action.

**6. Parity and scope gaps**
- The bench context is 512 while training and serving use 384, and `session.rs:1433` refuses positions beyond the trained context. The parity checklist should add "bench context ≤ served context".
- Under D11, F2 and GPTQ were not checked against `save_state`/rollback with KV-cache truncation. Shift-on-read needs k_{s−1} at s = 0 after a rollback.
- The value side was never tested. Copying v needs the token after the matched position, a lag of +1 on the value.

**7. Goal coverage**
- Coding/reasoning has no instrument at all. The Rust-code corpus tie (1.998 vs 2.011, #1437) exists and could serve as a cheap held-out NLL plus a code-recall panel.
- R5 energy is only a "baseline". No milestone tests a J/token claim.
- The owner's hypothesis is that lineage is a geometric primitive (ordered n-lets, ADR-0003). Answering it properly needs an arm where lineage is used for exact n-let addressing, i.e. lineage-hashed admission, rather than only as a soft key. The plan defers this to Step 6 and never tests it against F2.

**8. Process**
- GPU authorization appears only as an "owner records" item. Step 1 spends pod compute before M0. Per AGENTS.md, the cumulative ledger projection should come first.