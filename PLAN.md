# PLAN

**The one plan (D77).** Every lab picks its next unit from this file. Each unit takes the
step that unblocks the most items on a milestone path. CHARTER is the rule book, the item files in
`docs/roadmap/items/` are the work units with their dependencies, and STATE.md and HANDOFF.md
are rendered from them, never edited. A route or premise change needs a decision row in
`docs/integration/DECISIONS.md`. This file is written by PR. Newest first inside each table.

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
| The 16-root E8 orbit with Cl(0,6) frames can carry sequence position | Finite controls answered 10/10 at `b03d08e85` (#2843 comment `6380719988`); this is not an order-effect or language result. The registered order test (H_pos vs matched endpoint control + shuffled null) remains PENDING (#2843) | closed | 2026-10-24 | the parameter-matched endpoint control ties prefix holonomy on the registered panel |
| Scale-matched orbit-root snapping holds multi-stage residual blocks lawful | Finite controls answered 13/13 at `5844d911c` (#2842 comment `6381825466`); this is not a trunk or language result. The registered N_snap vs C_unscaled/C_raw trunk test remains PENDING (#2842) | closed | 2026-10-24 | the un-scaled or raw control matches N_snap on the registered panel |
| Exact addressed memory can serve durable conversation facts | Exact arithmetic controls answered 4/4 at `9aaaf13bb` (#2833 comment `6382217138`); this is not durable-conversation capability. The five-arm object-store oracle passed at `590ace9e`: true address 80/80, no-memory 0/80, deterministic sieve 20/80, with true-vs-sieve gains of 20/0 current and 20/0 abstention and D11 parity exact on 14,287 rows; FIT and D67 are recorded on #2833 | closed | 2026-10-26 | the true-address oracle gains nothing over the sieve and baseline arms |
| Number-aware tokenization is decidable and fixable | `cargo test -p uor-r4-tokenizer --test training number_aware_three_arm_comparison_and_digit_piece_census` — the 3-arm census (#2872, merged): Digits(false)+ByteLevel forms all 100 two-digit pieces with zero space-prefixed absorption | closed | 2026-10-22 | — (proven on both vocab sizes, digests pinned) |
| The R14 reply instrument is non-vacuous | `cargo test -p uor-r4-training --bin chat-grade --features native-rung0` — the v2 grader's constant/copy controls fail, the selective oracle passes, the closest archived run lands just under (PM ruling, #2029 6339245936) | closed | 2026-10-22 | — (ruled non-vacuous; the reply bar stays suspended with no target) |
| Reasoning comes from intermediate geometry and exact memory, not surface templates | Four-arm ablation instrument controls answered 3/3 at `9aaaf13bb` (#2866 comment `6382231845`); this is not reasoning evidence. The frozen-panel factorial-drop versus memorized-template-flatness question remains PENDING (#2866) | closed | 2026-10-24 | KnockoutBoth leaves panel accuracy at the count-table ceiling |

## Owner directions

Directions from the owner, recorded verbatim with the time (the same day). Status: `open`,
`done`, `superseded`. Newest first.

| Direction | Status | Date | Note |
|---|---|---|---|
| "we may have to pause for a moment and try to refocus on exactly what mechanisms we need to make this work, what we have, what order they need to be in, what geometry/mathematical domain they have to be in for downstream so that the mechanisms works cleanly from first trained "token" (or whatever we end up using) to last emitted work in the answer to a user's question" | open | 2026-10-27 | Claude's chat. The full text, with the goal (GPT-4-class, trained below datacenter scale, served on an M1-class laptop), is in item `pipeline-map`, which carries the map (claude-147). |
| "please reign in the project sprawl and get us back to a workable central source of truth we can run off of" | done | 2026-10-24 | Claude's chat, by 2026-10-25T00:03:59Z. This file is the one plan, and the held work is released (D77, claude-82). `loop next` ranking by unblock count follows as its own unit. |
| "please dont let us get locked down in CI hell, if something doesn't matter to the project it can be removed or skipped" | done | 2026-10-24 | Owner chose "Yes, change them (Recommended)". Six bookkeeping rules are advisory and banned presence scans added files (D76, #3170). The docs-workflow steps follow in a separate PR. |
| #2828: "Close it on its result (Recommended)" | done | 2026-10-24 | D78. Effort moves to the discrete block apply on the M1 path. |
| Trunk: "M1 closes without it (Recommended)" | done | 2026-10-24 | D79. `m1-trunk-assembly-and-training` is the first M2 item. |
| Outside contributors: answer their questions; no keys or paid resources; public HF links are fine | done | 2026-10-24 | Posted on #2655, #2036 and pointers on #2031, #2033–#2035. |
| End D59 now | done | 2026-10-22 | 13:56:44Z in claude's chat; switch merged as dffff0ed5 (#2973); setup PRs take any other-vendor review |
| Fix the loop harness errors that park labs stale without timers | done | 2026-10-22 | 13:50:03Z in claude's chat; no-idle-stops merged as 2ec16d513 (#2975); scope for it and the D59 switch by claude-29 (#2971) |
| PR 8 (notes migration) assigned to kimi | done | 2026-10-22 | "ok well I approve of #1", ~16:47Z in the kimi chat; ORDER kimi-9 carries it |

Pending owner *answers* are not directions and never enter this table. Since D65–D69 no step
waits for one: an unanswered ask becomes the lab's own decision review, logged for the owner.

## The path to M1, by unblock count

The M1 bar is CHARTER §4.8 (D39). Through the D11 export, on each of two seeds, integer top-1 must
beat both the unigram table (1,226/14,287) and the T2 successor table (6,300/14,287) on the same
DEV panel, with tokens/s reported. M1 closes on the saved-model path (D79).

| Step | Item (issue) | Needs | Unblocks | Kind |
|---|---|---|---|---|
| 1 | `m1-w2-capacity-capture` (#2845) | nothing; FIT done | Stage T, `m1-address-key-full-vector` (#2834), `m1-w4-float-credit-sheet` (#2846; with step 4, and only if W2 gives H_law and W3 fails) | GPU run, pre-run review |
| 2 | `m1-a8-finite-cuda-qualification` (#2830) | runbook staged (#3167) | `m1-a8-full-dose-rerun` (#2831) | GPU run, pre-run review |
| 3 | `m1-bvg-k-block-apply-c2` (#2835) | C2 controls met | Stage T, with step 1 | the one-page Stage T sheet and its FIT |
| 4 | `m1-bvg-k-stage-t-training` (#2836) | steps 1, 3 and 5, `m1-w2-code-utilization` (#3179), and `m1-address-key-full-vector` (#2834; claude-107's D68 card) | the M1 close; the trunk (M2) | GPU run, two seeds, about $12 |
| 5 | `m1-a8-full-dose-rerun` (#2831) | step 2 | the M1 close | GPU run |
| 6 | `m1-export-gap-and-toks` (#2839) | the exporter (done, #2838); its measured run needs a trained model from step 4 | `m1-fresh-panel-transfer` (#2841), the M1 close | export + tokens/s |
| 7 | `m1-fresh-panel-transfer` (#2841) | step 6 | the M1 close | GPU scoring |
| 8 | `m1-close-saved-model` (#2837) | steps 4–7 | M1 (#2029) | the §4.8 bar |

**Next to run on the side lines.** These are not on the close path.
- `m1-position-mechanism` (#2843): its next unit begins with a device-vs-host check of modes 1–3 and a stack-cost measurement (#2843 comment 6391161531).
- `m1-normalization-mechanism` (#2842): its order and norm tests.

Both feed the trunk.

## The route M2–M8

M2 (complete correct replies on DEV) starts with `m1-trunk-assembly-and-training` (#2844). That
needs position, normalization and Stage T, and D41's donor pass, which still needs the owner at
dispatch. Then the routes run in order:
- M3: durable memory qualified (provenance, correction, abstention, reset, restart).
- M4: the accepted UORLUT01 export, its served score and tokens/s.
- M5: the full-path laptop cost of that served model.
- M6: reasoning and coding on the same model.
- M7: the API, WASM and Studio.
- M8: one labelled release.

Work on a later milestone runs only when the step it needs is done, or when nothing on an
earlier path is free.

## Approved sequences

- **The W-sequence.**
  - W2 read-code capacity capture is step 1 above. It runs on its own pre-run review (D67) and no longer waits for a lease.
  - W3 BVG-K discrete batched-coordinate law: C1 qualified on 18 Oct, C2 controls met, Stage T is step 4.
  - W4 is one float-credit sheet, run only if W2 gives H_law and W3 fails (conditional, D37).
- **The K16 outcome follow-ups** stay pending with fresh UNAVAILABLE_NOT_RUN (sealed).
- **The loop.** PRs 2–12 are merged. The reset is D76 (gates) and D77 (this plan). Next, `loop next` ranks by unblock count.

## Not doing

- No softmax, squashing or normalized exponential anywhere, including renamed or table forms (D23).
- No dense matrix multiplication or GEMM in training or serving, including wrappers and hand loops (D25/D30).
- No transformer comparators (D28); no mesh as a coordination concept (L-021); codex keeps full access (P-5 dropped).
- Research compute is GPU-only (D24/§3.5); float training is offline-only in a separately scanned module with an export gap reported (D37); serving stays D11 integer (D11).
- The pooled-ranking loss (banned forms) does not return; a reply rerun needs a compliant objective.
- Nothing is deleted: negatives, unique source and other labs' work move byte-identical to archive/ (§4.7, L-015).
- No new infrastructure, documentation-clarification or record-hygiene unit unless a milestone item depends on it (D77). The loop's own fixes ride inside the unit that needs them.
- No certification chase on a closed configuration: the exact solve of the old saved case closed on its result (D78).
