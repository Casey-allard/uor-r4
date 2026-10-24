# PLAN

The route from today to done, kept by the PM by PR (the loop page, notes table). A route or
premise change needs a decision row in docs/integration/DECISIONS.md. STATE.md and HANDOFF.md
are rendered, never edited; this file is written. Newest first inside each table.

## Goal

One native Rust autoregressive geometric language model with exact addressed memory, useful for
conversation, memory, coding and reasoning, served integer-only (D11) on a consumer laptop.
Pre-alpha: 0 of 8 milestones complete; no model holds a useful conversation yet. Milestones
M1–M8 are issues #2029–#2036; #2028 is the shared board.

## Premises

The load-bearing assumptions, each with the probe that tests it and the line that cancels it.
A premise row's Status is `open` until its probe has run; `closed` means the probe's answer is in.

| Premise | Probe | Status | Date | Cancel if |
|---|---|---|---|---|
| Exact integer/geometric reads can carry a language model at all | The finite D11 stack suite is answered (3/3 at `127632bc3` and independently confirmed). Next evidence is the registered M1 closing-bar run (CHARTER §4 row 8): D11-export integer top-1 beats unigram (1,226/14,287) and T2 (6,300/14,287) on DEV, two seeds, tokens/s reported | held | 2026-10-23 | both seeds fail to beat the T2 table at full registered scale after the instrument class is ruled out |
| The 16-root E8 orbit with Cl(0,6) frames can carry sequence position | `cargo test -p uor-r4-discrete --lib position_mechanism` — the merged finite controls; the order test (H_pos vs matched endpoint control + shuffled null) rides sheet PENDING (#2843) | open | 2026-10-22 | the parameter-matched endpoint control ties prefix holonomy on the registered panel |
| Scale-matched orbit-root snapping holds multi-stage residual blocks lawful | `cargo test -p uor-r4-discrete --lib normalization_mechanism` — the merged N_snap vs C_unscaled/C_raw controls; the trunk test rides sheet PENDING (#2842) | open | 2026-10-22 | the un-scaled or raw control matches N_snap on the registered panel |
| Exact addressed memory can serve durable conversation facts | `cargo test -p uor-r4-integer --lib m6_exact_arithmetic` — the exact-memory admit/rank/version suite; then the address-as-read oracle sheet's FIT (deepseek, #2979: NOT_FIT until panel/checkpoint/bar/consumer are pinned) and its GPU run (#2833) | open | 2026-10-22 | the true-address oracle gains nothing over the sieve and baseline arms |
| Number-aware tokenization is decidable and fixable | `cargo test -p uor-r4-tokenizer --test training number_aware_three_arm_comparison_and_digit_piece_census` — the 3-arm census (#2872, merged): Digits(false)+ByteLevel forms all 100 two-digit pieces with zero space-prefixed absorption | closed | 2026-10-22 | — (proven on both vocab sizes, digests pinned) |
| The R14 reply instrument is non-vacuous | `cargo test -p uor-r4-training --bin chat-grade --features native-rung0` — the v2 grader's constant/copy controls fail, the selective oracle passes, the closest archived run lands just under (PM ruling, #2029 6339245936) | closed | 2026-10-22 | — (ruled non-vacuous; the reply bar stays suspended with no target) |
| Reasoning comes from intermediate geometry and exact memory, not surface templates | `cargo test -p uor-r4-integer --lib m6_ablation` — the four-arm ablation instrument (#2863, merged); the run rides the frozen panel (#2866, bar card with the owner): factorial drop vs memorized-template flatness | open | 2026-10-22 | KnockoutBoth leaves panel accuracy at the count-table ceiling |

## Owner directions

Directions from the owner, recorded verbatim with the time (the same day). Status: `open`,
`done`, `superseded`. Newest first.

| Direction | Status | Date | Note |
|---|---|---|---|
| End D59 now | done | 2026-10-22 | 13:56:44Z in claude's chat; switch merged as dffff0ed5 (#2973); setup PRs take any other-vendor review |
| Fix the loop harness errors that park labs stale without timers | done | 2026-10-22 | 13:50:03Z in claude's chat; no-idle-stops merged as 2ec16d513 (#2975); scope for it and the D59 switch by claude-29 (#2971) |
| PR 8 (notes migration) assigned to kimi | done | 2026-10-22 | "ok well I approve of #1", ~16:47Z in the kimi chat; ORDER kimi-9 carries it |

Pending owner *answers* are not directions and never enter this table; they are the decision
cards on #2028 (seven bar cards as of 22 Oct) until he answers in the owning lab's chat.

## The route, written backwards

M8 (one labelled release on the same native path) needs M7 (the model served through the API,
WASM and Studio), M6 (reasoning and coding qualified on the same native model) and M5 (the
full-path laptop cost measured on that served model); M7 needs M4 (the accepted UORLUT01
export, the served evaluation score and tokens/s on the held-out stream, the export-gap rule),
which M5 measures and M6 ablates; M4 needs M3 (durable memory qualified: provenance,
contradiction/correction, abstention, reset, restart, #1718 non-overwrite) needs M2 (complete
correct replies on DEV, HELD for release) needs M1 (the language base: the §4.8 closing bar) —
and M1 needs, in order:
the trunk assembled and trained (`m1-trunk-assembly-and-training`, #2845, HELD); the position
and normalization mechanisms' order/norm tests (sheets PENDING, controls merged); the address
memory oracle FIT then run (NOT_FIT today, #2979); the number-aware vocabulary registered
(census proven today, the definition card with the owner); the M6 ablation instrument (merged
today) on the frozen panel (in flight); and the reply headline unblocked (R14 ruled non-vacuous
today; the numeric reply bar is a separate owner card, unsent).

## Approved sequences

- The W-sequence from the retired NOW.md: W2 read-code capacity capture (FIT'd, queued on a
  lease), W3 BVG-K discrete batched-coordinate law (Stage C1 qualified 2026-10-18; Stage C2
  next; Stage T held for the K16 outcome), W4 one float-credit sheet only if W2 = H_law and W3
  fails (conditional, D37).
- The K16 outcome follow-ups: the registered two-seed/Amendment-7 interpretation and the
  conditional fresh disposition stay pending with fresh UNAVAILABLE_NOT_RUN (sealed).
- The loop build plan: PRs 2–12 merged; PR 8 (this migration) is in flight; the umbrella's
  wrap-up (onboarding receipts + one end-to-end task per lab) follows.

## Not doing

- No softmax, squashing or normalized exponential anywhere, including renamed or table forms (D23).
- No dense matrix multiplication or GEMM in training or serving, including wrappers and hand loops (D25/D30).
- No transformer comparators (D28); no mesh as a coordination concept (L-021); codex keeps full access (P-5 dropped).
- Research compute is GPU-only (D24/§3.5); float training is offline-only in a separately scanned module with an export gap reported (D37); serving stays D11 integer (D11).
- The pooled-ranking loss (banned forms) does not return; a reply rerun needs a compliant objective.
- Nothing is deleted: negatives, unique source and other labs' work move byte-identical to archive/ (§4.7, L-015).
