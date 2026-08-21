# #836 4c — segment-lane causal-utility verdict (deployed learned-table path)

**Scope.** #836 lowered the #835 segment lane onto the deployed serving path
(`R4Engine`): the R4G1 `PSTATE` format (#876/#877), the P-4 hot-path ring
(#878/#879), the engine wiring (#880/#882/#883), the compiler emission
(#881/#883), and — this increment chain — the **learned content→teacher-argmax
residual table**: the engine consumes it (4c-i, #883) and the compiler fits and
emits it (4c-ii). This document is 4c-iii: it records, under the repository's
long-run discipline, whether the now-fully-lowered lane may be **promoted** into
the serving default. It does not launch an hours-long causal run whose outcome
is already fixed by arithmetic.

## 1. Metric and current deployed value

- **Primary metric:** paired `causal-influence-delta` (‰) = full-context
  top-1-vs-teacher minus suffix-only top-1-vs-teacher, on the frozen #833
  canonical bundle, EXCT-disabled (`causal_prompt_run_834`, SUFFIX_K=2,
  N≈24k held-out).
- **Recorded current value (deployed, `docs/causal_run_834_result.json`):**
  **−1.6‰**, paired 95% CI **[−2.4, −0.8]**. The deployed model is suffix-local;
  full context does not beat the suffix on the serving path. Minimal-pairs
  follow 0/1460.
- **Pre-registered floor for a SELECT/PROMOTE:** **20.0‰**
  (`CAUSAL_FLOOR_PERMILLE`), frozen before any arm was built.

## 2. Reachability ceiling (arithmetic)

- The **maximum** whole-prompt-content signal measured anywhere is the #834 §6.2
  reference arm — the **learned content-token→teacher-argmax table, top-64 per
  key**: **Ψ +17.5‰, 95% CI [+15.9, +19.0]** over the suffix baseline, produced
  off the serving path and unquantized.
- **Ceiling < floor.** The *optimistic* upper bound of the whole-prompt effect
  (**19.0‰**) is **below** the 20.0‰ floor. The deployed packed lane can only be
  **≤** that ceiling, because it is strictly more constrained than the reference
  measurement:
  1. **Quantization** of the table to integer `ScoreQ` at a fixed
     `RATE_SCALE_SHIFT` (vs the reference's unquantized rates).
  2. **Top-8 candidate bound:** the deployed re-rank acts only within the served
     top-8 (`STEP_TOP_CANDIDATES`), vs the reference's full-vocab argmax.
- Therefore the reachable deployed ceiling ≤ 19.0‰ **< 20.0‰ floor**, with 1–2
  pushing it strictly lower. The conclusion is unconditional; no run can change
  it.

## 3. What changed since the original 4c contract: the mechanism gap is closed

The pre-4c contract (`/docs` history, superseded here) named a **mechanism
gap** as its first revision path: the deployed lane lowered only the #835-spec
**recurrence** rule (boost prompt-present candidates), whereas the +17.5‰ came
from a **learned** content→argmax table — a stronger, different mechanism the
serving path did not compute and the converter did not emit.

That gap is now **closed**:

- **4c-i (#883):** `R4Engine` reads the residual table from `PSTATE` and, when
  present, scores each candidate by the summed learned contribution of the live
  prompt-content tokens (`table_contribution` / `segment_adjusted_token_with_table`)
  — the faithful lowering of the §6.2 mechanism, not the recurrence proxy.
- **4c-ii:** `segment_fit::fit_segment_table` produces that table with the same
  content→argmax co-occurrence tally, top-K cap, and TRAIN-only fit as §6.2,
  quantized to the P-4 serving weights; `convert_with_segment_table` emits it.

So the lane the repository now ships **is** the §6.2 mechanism, end-to-end. The
verdict below is therefore **not** blocked on an un-lowered mechanism — it rests
purely on the ceiling arithmetic of §2, which the closed gap does not move
(quantization and the top-8 bound keep the deployed lane at or below the same
19.0‰ ceiling).

## 4. Predeclared exit rule and cost avoided

- **Launch a full causal run iff** the reachable ceiling ≥ floor. It is not
  (19.0 < 20.0). → **Do not launch.** A run over ≈24k held-out positions × 3
  passes (full / suffix / swap) on the 360m teacher bundle — tens of minutes to
  hours of wall-clock plus model load — for a foregone-conclusion negative is
  not spent.
- Positive branch (would have been): `delta_lo ≥ 20.0‰` on ≥1 domain → PROMOTE.
- Negative branch (**taken, by arithmetic**): ceiling below floor → **REVISE**.

## 5. Verdict: **REVISE** — mechanism fully lowered and safe, effect sub-floor, lane kept dormant

The segment lane is lowered end-to-end and is **safe by construction**: the
`PSTATE` section is optional, every existing artifact carries none and is
byte-identical under the new code (absent-section identity), and the re-rank
only ever reorders the already-decided top-8 (it never fabricates a token and
never turns a served decision into an abstention). But the whole-prompt content
**signal is real yet sub-floor** (+17.5‰, CI upper 19.0‰ < 20.0‰ floor), and the
deployed lane is strictly weaker than that ceiling. **A PROMOTE is not warranted
and is not claimed.**

**Ledger / claim.** The capability stays **dormant**: no `model/ids.toml`
serving row, `CONFORMANCE.md` unchanged, no public quality claim. The activation
gate is unchanged: *re-clears the 20‰ causal floor on the packed path.*

**Open revision paths (tracked as new issues, not abandonment):**

1. **Widen the candidate set the lane can act on** (candidate-support expansion,
   #835 family): the top-8 bound is one of the two constraints holding the
   deployed lane below the 19.0‰ ceiling. Lifting it is the only lever that could
   move the *deployed* effect toward the reference ceiling — though the ceiling
   itself (19.0‰) still sits below 20‰, so this alone cannot clear the floor.
2. **Bounded empirical spot-check of the deployed learned-table lane** on the
   real #833/#834 bundle (the 10/4,722 minimal pairs the reference arm resolved),
   to confirm the quantized deployed path tracks the +17.5‰ reference within its
   ceiling — seconds of compute, not the full run — as evidence for/against
   path 1 before investing in it.
3. **Governance review of the 20‰ floor** for a re-ranking lane whose honest
   effect is ~+17.5‰: whether that is the right bar. This is a
   pre-registration/governance question and must not be resolved by moving the
   floor post hoc to force a pass.

These are filed in implementation order; #836 closes REVISE with this document as
its evidence.
