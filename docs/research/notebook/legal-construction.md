# Legal construction — research notebook

The line asks whether a legal-construction step (each parameter change picked straight from values the integer model can hold) can be scored without a forbidden transform. One row per result; verdict words follow [`README.md`](./README.md).

| Date | What ran | Number | Bar | Verdict | Artifact |
|---|---|---|---|---|---|
| 2026-10-22 | Exact host instrument on the saved case `input245`: mapping settled from commit `ebd6ecaa` (`m = prefix+generate` masters, `g = prefix+generate` gradients, 380 `protected-margin-NNN` guard rows); loader + exact zero-move check ran; exact dense-tableau root LP attempted under a 120 s cap (this record makes no claim about any other lab's solver; see the lab-record correction) | `n = 1,920`, `guards = 380`, `fractional = 20`; load `0.13 s`; zero move exact-legal and guarded (min guard value `0/1`, objective `0/1`); 31-term f32 gradient sums bit-identical to aggregates; root LP returned no result in 120 s | none registered (diagnostic/instrument check, not a scoring run) | blocked (dense exact simplex does not scale at this size; no optimality or infeasibility claim) | [lab record](../../labs/legal-construction-exact-solve/README.md#unit-3--case-mapping-and-the-dense-lp-blocker-deepseek-33-2026-10-22), [`solve-case`](../../labs/legal-construction-exact-solve/solver/src/bin/solve-case.rs) |
