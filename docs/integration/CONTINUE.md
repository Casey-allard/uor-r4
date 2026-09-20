# Continue UOR-R4 geometric model research

Read the [frozen S attribution result](s-attribution-result-2026-09-20.md) and its
[evidence receipt](../evidence/native_geometric_s_attribution_2026-09-20.txt), then refresh
origin/main, canonical plan/current state, live issues and resources.

## Where the programme stands

The [matched query read](query-read-result-2026-09-20.md) is a valid negative (Q loses to its
separable control S). The **attribution of S is now executed and decisive**, evaluation-only, from
base `8bbdb63d`, sealed at `.uor-models/realtext-prior-2026-09-20/s-attribution-3`
(30 files, 4,390,694 bytes; two earlier roots preserved sealed with identical numbers).

On `Z_S = E + u(q) + u(b)`, with `E` frozen at 7.170816 bits/target on the old panel:

| Condition | old micro (all) | new micro (all) |
| --- | ---: | ---: |
| E | 7.170816 | 7.048239 |
| S11 = E+u(q)+u(b) | 7.032228 | 6.919775 |
| Qonly = E+u(b) | **7.022581** | **6.912660** |
| Honly = E+u(q) | 7.171048 | 7.045576 |
| M01 fit-average history (offline) | 6.984779 | 6.894588 |

Matched older-present positions, paired 2,000-draw bootstrap: **M01−S11 = −0.018695
[−0.022344,−0.015019]** (new) / −0.023622 [−0.026746,−0.020352] (old); **Qonly−S11 = −0.007348
[−0.012050,−0.002327]** (new) / −0.009978 (old); Honly−S11 = +0.129926 / +0.143589. The integer
identity `Z11+Z00=Z10+Z01` passes with **0 failures**; E/S01/S00/Qonly are *exactly* invariant to
the donor and reversal maps; all 16 retained old vectors reproduce with a **0.0** maximum delta.

**Conclusion: S's measured improvement is a current-token query/emission calibration.** The older
reader row alone is indistinguishable from E; dropping it *improves* CE on both panels; a
content-free, length-conditioned fit average beats the individual state on both panels. The
exact-tail donor map is silent because it holds the length-conditioned offset fixed — the only part
of the history term with a measurable effect. **No candidate is promoted; the 120-state group
product is not carrying usable older content at this dose.**

## Components repaired with this step

- Production export **requires** the real raw 32-byte tokenizer identity; `ZERO_DIGEST` is refused.
  The legacy zero-digest files load only through `QueryHard::import_legacy`, and the run verifies the
  production loader rejects them. Corrected descendants change **exactly 32 bytes inside 73..105**.
- `inference_rows` / `rows_from_older` separate inference row selection from training-chain
  construction: S folds history 62 times instead of 124, **L 3 instead of 65**, with row equality
  against the training path at every position.
- `validate_tokens` / `generate_checked` reject out-of-vocabulary tokens and empty prompts.
- `bin/query-read-attribution.rs` is an evaluation-only entry point that cannot fall into fitting.
- 18 `query_read` module tests pass.

## The one next task

**Give the state a mechanism that can make older content *addressable* rather than merely
present**, and qualify it against fresh behavioural criteria:

1. **Bounded selective write/reset** over the exact 2I state — a four-bit
   `Continue(gamma_k)` / `ResetTo(gamma_k)` translation-plus-constant monoid is one coherent
   candidate. It is **not implemented, fitted or measured in this run** and adds no register
   capacity or keyed memory.
2. **Or exact occurrence/version access**, reusing the retained native-memory contracts, so a
   selected record rather than a lossy 120-state product carries the content.

Do **not** re-fit S, widen head/width/precision, fetch a corpus, run a projection campaign, or treat
the offline `M01` comparator as a serving candidate. **CPQK production continuation must be repaired
before any new fit**: it currently serializes masters/moments/ages but does not bind or recover the
dose permutation/cursor or full data/table/tokenizer identities, and the retained mini test resumed
in-memory bytes in one process.

Selective write/reset, exact memory and shared composition remain separate obligations, as do useful
conversation, executed Rust behaviour on one accepted artifact, and complete consumer-machine cost.
D0-b, the geometric priority and the prime/zeta/R4/S3/H4/icosian roles are unchanged.
`#973`/`#820`/`#963`/`#964` remain open.

## Resources

The standing-authorized **+2400000 ms** extension to **180500000 ms** was recorded in the live ledger
**before** consumption. This step charges **2400000 ms** once (compile/test cycles, five harness
executions including two probes and one superseded complete run, and a documentation/delivery
allocation). **New cumulative 178538565 / 180500000 ms, remaining 1961435 ms.** Retained: 4,390,694
bytes for `s-attribution-3` plus the preserved superseded `s-attribution-1` and `s-attribution-2`;
whole-run peak RSS 722,681,856 B; physical energy UNAVAILABLE; the 128 MiB storage stop margin is intact; no deletion,
corpus download or paid/external compute. Reuse the retained artifacts and vectors for derived
reporting rather than re-running the evaluation.

Deliver through a protected PR, verify the actual merged tree, and update the owning issues and any
existing project items.
