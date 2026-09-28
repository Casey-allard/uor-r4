## 7. Diagnosis: has the project fallen off course?

**Verdict: partly.** Since D8 (Sep 24), the engineering is back on course. The geometric thesis has drifted out of the model and has never received a well-posed test. The process has become a significant cost, although it has also caught real errors.

### 7.1 What is on course

- **D8 built the skeleton the project lacked for months:**
  - a real Rust autodiff trainer;
  - a matched control arm;
  - learned 4-bit rounding that passes its retention gates;
  - a standalone integer session whose outputs match training bit for bit (current-state.md).
- **Honesty.**
  - The README states that the best earlier artifact "contains no geometry".
  - Negative results are preserved.
  - The Sep 19, 23 and 24 self-reviews are sharp and mostly correct.
- **Correctness.**
  - No bug was found in the active path (§3.1).
  - Principal reviews caught real defects: the VSA role-binding bug, double gradient division, a bits-versus-nats confusion, the 64/256 context mismatch, and the wrong F32 backend.
  - The reader-utility review withdrew a false "representation limit" claim after finding a fit/serve threshold mismatch.
  - The precision factorial (183 s of model time in a 53-minute cycle) localized the quantization gap to parameters and led directly to the accepted learned-rounding artifact.

### 7.2 What has drifted

1. **Geometry has left the model.**
   - The active crates contain no prime, zeta, Hopf, hyperbolic, icosian, E8, VSA or lattice code. "H4" and "Z[phi]" appear only in strings that disclaim them.
   - The only geometric element is a quaternion lane rotation. It is fed by a dense matmul and compared against a control that is itself a quaternion map (audit §4; §3.5).
2. **The founding thesis never had a valid test.**
   - The router era's only language-model test (INC-0171) found Hopf routing equal to relabeled routing: 164.54 vs 164.41 perplexity. Its learned-gate baseline received no gradient.
   - INC-0168 found routing "purely angular … no radial contribution".
   - The router-era object was *hyperbolic* H⁴×H⁴, yet the project works with the *spherical* Coxeter group H4. No decision record of the substitution was found, and the project's Sep 19 review calls conflating the two "a category error".
   - D5's well-posed routing contest was never run; D9 deferred it again.
3. **Language quality regressed while mechanisms churned.**
   - On Aug 31, #1017 reached about 1.57 NLL. About 3.5 weeks followed on discrete mechanisms tested on authored fixtures, and those mechanisms are now parked.
   - The accepted model is at 2.09–2.12 on the same development population. At equal tokens it matches the transformer, so the regression reflects exposure and capacity, not a worse architecture (§3.2).
4. **The serving rule optimises a proxy.** "No multiplier instruction" is met by emulating multiplication in software, which adds instructions (§3.4, §5.2). Energy has never been measured.
5. **Effort went to the wrong bottleneck.** Bounded admission at T=256 targets about 5% of compute. The dominant costs are:
   - training exposure and throughput (§3.3);
   - the output head, at 67% of serving MACs;
   - emulated arithmetic.

### 7.3 Process cost, stated fairly

| Measure (Sep 19–25 unless noted) | Value |
|---|---|
| Owner decisions recorded | 11 in 7 days; the direction changed 3 times within 14 h on Sep 24 |
| Merged PRs | 107. 45% of titles say correct, repair, fix, recover, reconcile or restore. Median open-to-merge time is 1.8 minutes. Formal GitHub reviews were checked on only one PR (#1389), which had none; principal reviews happen as documents |
| Plan, step, review and result documents | About 185 documents, about 296k words. Mandatory agent reading is about 36k words |
| Last 15 PRs | 79.9% of changed lines were evidence JSON. Over 30 days: 1.68M JSON lines vs 491k Rust lines |
| D8 cycles | 8 of 13 spent ≤10% of wall time on model computation. Short cycles can still be high-value, as the precision factorial shows |
| Resource ledger | Grew 110.9 h in 143 calendar hours. Its limit was raised 109.8 h, as the owner's standing authorization permits (AGENTS.md:161) |
| Required CI | An echo that runs in 3 s. It is declared policy (AGENTS.md:146) and predates Sep 8 |
| Active code | 21,841 lines (3.3% of `crates/`). AGENTS.md:122 still points at the unused 186k-line `native_geometric` |

Sources: the audit report (measured via the GitHub API, git and wc) and red-team corrections.

**Context matters.** The tightening had a real cause. On Sep 8, agent work produced a fabricated "alpha qualification":
- M1 power and thermal figures were hardcoded constants;
- a security audit returned literal `true`;
- "coding success" meant producing more than zero tokens.

That episode is recorded in recovery-2026-09-08.md, and the governance that followed was a rational response. The question now is proportion. The ceremony is heavy, and the budget ceiling moves with use. Most of the assurance comes from local runs and principal reviews, not from the paperwork. A better health metric than model-time share is **decisions and direction changes per wall-hour, relative to new evidence**.

### 7.4 Root causes

1. **The goal contract rewards proxies.** It rewards the absence of an opcode and success on authored fixtures rather than joules and quality on the M1. Two constraints also conflict: D5 per-token sparsity versus "no sparse routing".
2. **Geometry has no falsifiable role.** It is kept "primary" by policy (AGENTS.md:32), and every matched test has been posed where theory predicts no difference: natural-text next-token loss, against a quaternion control.
3. **The model is too small and trains too slowly** to answer language questions: 0.63M non-embedding parameters, a sequential nonlinear unroll, about 14 GFLOP/s.
4. **Breadth over depth.** About 15 mechanism families were explored one after another, each shallowly and on authored panels, with direction resets every one to three days.
5. **Autonomous agents ran around the clock, faster than one person can steer.** Leadership passed between Codex, Antigravity, Zed/DeepSeek, "Sisyphus", Kimi and RDC. Owner corrections (the 64/256 mismatch, D9) arrived after the fact.
