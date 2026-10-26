# ORDER: infra-loop-harness, claude-120 (release `d11-exporter-float-trunk` from `held`)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in claude-113 and claude-114). Base: `c54fdc121`.

## Goal

`d11-exporter-float-trunk` (#2820) is `held`. antigravity-75 (#3221) built the float-trunk UORLUT01 exporter, and its REPORT gives the hold reason. The Stage 4 export-gap run and the trained-model tokens/s "are blocked on upstream Stage 3 float-trunk training", the chain #2886 → #2890 → #2824 → #2891, which "has not yet produced trained two-seed Stage 3 float-trunk checkpoints".

That chain is now done; #2891 merged as `c54fdc121`. If a fresh D68 decision review confirms that the remaining work is the lab's own, this unit:
- sets the item to `ready`;
- adds `crates/uor-r4-float-train/**` to its scope;
- records in the item what is actually left.

The owner asked for this on 26 Oct, in Claude's chat: "Pick the best held item that doesn't touch crates/uor-r4-discrete, run your own pre-run/decision review per CHARTER §5.1, and start it."

What is left, as found before this ORDER (each to be confirmed by the cancel probe and the review):
- **No checkpoint exists.** None of the four Stage 3 runs saved a model to the store.
- **The exporter covers a different trunk.** It exports a dense-matrix trunk: recurrence `rec_in`/`rec_gate`/`rec_out`, MLP gate/up/down, and flock-read Q/K/V/out. The Stage 3 sheets trained six recurrence layers with SpiralCore/FBS projections and an 8-cell native vector writer head.
- **So the work left is code, then a cheap run:**
  - an FBS (atom, action) projection path and a vector-writer head in the export and integer runtime;
  - a snapshot writer in `crates/uor-r4-float-train`;
  - finite CPU controls;
  - then a two-seed retrain with saving, about 35 s per seed on one RTX 5090, under an amended Stage 4 sheet with a new FIT and a D67 pre-run review.

**Why the infra-loop-harness item.** Its scope holds `docs/roadmap/items/*.md`. Codex's idle #3169 holds the order but touches only `PLAN.md`, `docs/integration/DECISIONS.md` and its own task folder. So, as for claude-113 and claude-114, this ORDER is written by hand under the run file's umbrella exception. A held item cannot be claimed, and the guard reads scope from `origin/main`, so the release must merge before the exporter work is claimed.

## Cancel probe (run first)

Class: instrument. Run at `c54fdc121`; all must hold, or the unit stops:
- **The hold reason is the one quoted above,** in antigravity-75's REPORT and D68 log (#2820 comment `6399222146`).
- **The named chain is done:** `r-secant-surrogate-descent`, `recurrence-gates-six-recurrence-fit`, `hamilton-gate-offline-float-sheet` and `spiralcore-fbs-offline-float-sheet` all read `status: done` on `origin/main`.
- **No other hold reason exists:** the item and its issue name no other blocker, and no owner hold.
- **Nobody else is on it:** no open PR touches the item, `crates/uor-r4-integer` or `crates/uor-r4-float-train`.

## Files in scope

- `docs/roadmap/items/d11-exporter-float-trunk.md`
- `docs/roadmap/README.md` (the generated render only)
- This task folder

## Done when

- **The D68 review is recorded.** A fresh-context decision review has read the whole project chain and APPROVEd or rejected the release. Its record is on #2820 as `[claude] owner log:`.
- **If approved, the item is changed:**
  - it reads `status: ready`;
  - its scope adds `crates/uor-r4-float-train/**`;
  - its "What is missing" records the FBS/writer gap and the missing checkpoints;
  - the roadmap is re-rendered.
- **It is merged through the loop.**

## Stop and ask if

Stop and decide, never stop and wait.

- **The review finds another live blocker.** Then the item stays `held`, the finding is recorded on #2820, and the unit ends with a REPORT.
- **The release would also need a change to CHARTER §3 or a cap.** Then post one `[claude, for Casey]` note on #2028 and keep the item `held`.

## Cancel probe result

Run at `c54fdc121`. All four hold, so the unit goes ahead to the decision review:
- **The hold reason** in #2820 comment `6399222146` is the Stage 4 run "which requires two-seed trained Stage 3 float-trunk checkpoints from the upstream float-training chain (#2886 -> #2890 -> #2824 -> #2891)".
- **The chain is done:** `r-secant-surrogate-descent`, `recurrence-gates-six-recurrence-fit`, `hamilton-gate-offline-float-sheet` and `spiralcore-fbs-offline-float-sheet` all read `status: done`.
- **No other hold exists.** #2820 carries the labels `area:serving` and `roadmap:item` only, and no comment records an owner hold. The item's step 4 still says training "stays HELD until Casey types the owner release himself". That wording predates CHARTER §5.1's rule that held work goes ahead on the lab's own pre-run review (D67); the decision review weighs it.
- **Nobody else is on it:** no open PR touches the item, `crates/uor-r4-integer/` or `crates/uor-r4-float-train/` (open: #3239, #3169).
