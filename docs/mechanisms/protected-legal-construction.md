# Protected / discrete legal construction (native learner)
**The idea (first principles):** The served model must run with integers, tables and exact geometry (D11, D0-b). Ordinary float gradient steps produce weights that must later be rounded, and rounding can break what was learned. Legal construction builds each parameter move *already in the served form*. Inside a protected Prefix/Generate transaction (no change commits unless the protected behaviour still holds), it picks among the finite set of **legal destinations** (Q4 codes) for every coordinate at once. A mixed-integer solver enforces guards ("keep these 17 references correct") and asks for a descending move. Discrete feedback replaces the float gradient as the signal. If it works, every accepted step is exactly servable and provably preserves the guarded behaviours.
**What it replaces and why that matters:** It replaces train-in-float-then-quantize, plus manual regression checks. The point is learning that never leaves the legal, multiplier-free set, so nothing is lost at export.
**Where it lives in code:** The Codex native learner (M2). The exact constructor source and its frozen-map fitting examples are named in the PRs below and in their records. The records are `docs/labs/direct-legal-construction-2026-10-09/` and the sibling `docs/labs/*-2026-10-09/` records. The superseded absolute encoding and the first diagnostic labels are archived as patches in `docs/history/branch-archive/INDEX.md`.
**What has been tried, honestly:**
- #2079, #2084, #2088, #2101, #2109, #2117 and #2121: protected joint learning, Prefix transactions and discrete/residual feedback. Residual feedback never offered a protected descending update.
- #2125: direct joint construction over 15 legal destinations (1,960 variables, 440 constraints, 4,096-node branch bound). Both encodings stopped with `Singular matrix` before any proposal.
- #2132: the failure is localized to near-zero LU pivots (~9e-11) at branch refactorization.
- #2134: a scale-aware pivot rule verifies 148 of 150 fresh factors, then rejects.

**None of this reached panel scoring.** The mechanism was never judged. It was blocked by solver numerics, on one saved input (input245) at small scale. D21 §3 archived the line. [D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened) §5 reopened it, because a numerical blocker is not a negative.
**What success looks like:** Own: the constructor returns a legal, guard-preserving, loss-descending move, and repeated moves keep descending. Headline: M2 #2030 complete correct replies on the 512 panel (8/512 at D21; 437/512 development after #2162; fresh 0/128 in #2164).
**What failure looks like:** With exact or rational arithmetic (no LU drift), legal moves exist but give no descent over float-then-quantize at equal compute. Or the guard set is infeasible.
**A fair test:**
- Solve the same instance in exact rational arithmetic, or with a numerically robust solver, so numerics are removed.
- Then run N construction steps on the saved learner, scored on the frozen 512 panel.
- Compare against a float-trained, quantized control at matched compute, with ≥2 seeds.

**Open design questions:**
- Use Z[phi]/integer-exact pivoting instead of float LU?
- Decompose per coordinate group to shrink the basis?
- Relax guards to slack penalties?
- A greedy legal coordinate search as a baseline before full MIP?
