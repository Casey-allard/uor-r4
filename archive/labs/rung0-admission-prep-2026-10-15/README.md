# Rung-0 admission-objective successor — Stage A pre-registration (2026-10-15)

Canonical lab record for the pre-registration sheet
[`docs/harness/prep/m1-discrete-rung0-admission-2026-10-15.md`](../../harness/prep/m1-discrete-rung0-admission-2026-10-15.md).

## What this is

The seed-1 rung-0 completion run finished rc=0 at 16,404 batches and was PM-verified
EXACT (#2029 comment 6186067850): top-1 149 (bar 1,941), top-8 747 (bar 4,775), guard
105 (bar ≈525), target-admitted rows 1,920, B3 nesting exact, instrument valid. The
bars were arithmetically unreachable because top-1/top-8 ≤ target-admitted rows and
1,920 < 1,941 — an admission-limited miss: iteration evidence, not a mechanism
negative (D22).

The owner decided (#2029 comment 6187197075, 2026-10-15 ~07:55Z): seed-2 as registered
is HELD; the line iterates the admission objective; the admitted-row ceiling must
exceed the decisive bar counts before any run is released.

The sheet pre-registers **Stage A**: a zero-update kept-cells (K ∈ 8..16) sweep that
re-scores the four preserved seed-1 quarter checkpoints on the frozen 14,287-row dev
panel — an exact-integer ceiling measurement with a stored-expected-value
qualification at K=8 (it must reproduce the sealed quarter metrics exactly). Gate
G-A: some K reaches ≥ 4,894 target-admitted rows at checkpoint 16,404 (bar (ii) 4,775
+ one Wilson half-width 119). PASS → Stage-B training amendment at the smallest
passing K (fresh lineage, re-based guard, Claude FIT, owner release — training stays
HELD until both). FAIL → closes eval-time widening of the K=8-trained codebook
(D22 configuration closure); K>8-trained lineages stay open as a named successor;
redirect to the admission policy/objective sheet; no training under this sheet.

## Artifact pins

Pod `/workspace/uor-r4/rung0-completion-20261014/seed1-attempt-1/` (archive
`rung0-completion-seed1-20261015.tgz` sha256
`300246ab16a1e0adaba547d43590e08264912f59ee29f35d7e024d77591e63a1`):

| checkpoint | sha256 |
|---|---|
| 04101 | `a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd` |
| 08202 | `dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822` |
| 12303 | `102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982` |
| 16404 | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` |

## Chain

Sheet → Codex evaluation-only K-sweep diff (D-A) → Claude TEST FITNESS → Kimi
instrument packet → owner go for the sub-$1 pod minutes → run → sweep table + G-A
verdict on #2029. Training and M2 remain HELD; a FIT releases nothing.

Status: **TEST FITNESS: FIT (design, Stage A)**, confirmed by Claude
[6192687624](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192687624).
Kimi recorded the Stage-A-only owner GO at
[6192566334](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192566334)
(one RTX 5090, < $0.50 all-in). The [D-A source/runbook](../stage-a-admission-instrument-2026-10-15/README.md)
was implemented with host metadata checks; exact-source review and the bound
instrument packet preceded the executed run. Its existing evaluator also reads
masked TRAIN to reconstruct the count reference, alongside DEV evaluation;
this clarification is explicitly submitted to Claude with the source.
No compute had run under this sheet at registration; the Stage-A evaluation-only
sweep has since executed (2026-10-15, pod vast-56109378) and returned
**G-A FAIL** — see the [sweep record](../rung0-stage-a-sweep-2026-10-15/README.md).
No training has run under this sheet. Training and M2 remain HELD.
