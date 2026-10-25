# ORDER: m2-pin-train-mask-sheet, claude-99

Lab: claude. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md) (#2851). Base: `53b40ecd6`.

## Goal

The item's done-when reads:

> A recorded answer from the PM or owner on whether the Amendment 6 sheet is live under D37 or superseded, and a one-page sheet that is ready for an owner hold-release decision: either the pins re-verified against current main and restated in a refreshed sheet, or the pins carried into the new discrete-law sheet, with a non-author FIT. The sheet itself runs nothing: no training or optimizer update is authorised by it.

## Cancel probe (run first)

**Result at `53b40ecd6`:** most of the done-when is already on `main`, but the FIT is blocked by other items.
- **The owner's answer is recorded.** On 24 Oct (#2028 comment `6379430093`, item 3) Casey said: "I want both because we do not know which we will ultimately need." So the D37-basis sheet `docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md` goes ahead as the successor to Amendment 6, and a discrete-law rung-0 sheet is also wanted.
- **The refreshed sheet exists, but its FIT is NOT_FIT** (`antigravity-33`), on four prerequisites:
  - (a) the owner's answer, now given;
  - (b) the export-gap statistic and band, which must be fixed in the sheet itself. On reading line 29 after the first ORDER commit, this turns out to be done already: the PM cover (deepseek) registered it on 24 Oct (`G_top1`, `G_recall8`, band `max(0.83 pp, two-seed spread)`);
  - (c) the separate D29-scanned float module (`float-training-module`, #2823) and the D11 float-trunk exporter (`d11-exporter-float-trunk`, #2820), neither built;
  - (d) the literal source SHAs (now pinned, lines 73-76) and a sealed fresh-transfer slice, which the sheet itself defers as a KEEP gate.
- **The pins re-verify today.**
  - The mask member `mw-balp/train/response_mask.u8`, extracted from the HF store's `data/ft-balp.tar` (sha256 `b1d22113…473f`), is 7,512,797 bytes with sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
  - `9e904a83448c451f738d5c1d45224dfdf73eb84e` is an ancestor of `main`.
  - The count reference `examples/geometric-stack/native_rung0.rs` is byte-identical between `2fd6358bb` and `main`.
- **Class:** instrument (the FIT is blocked on tooling other items build). This is not a mechanism result.

## What this unit does

1. **(b) needs no work:** it is already registered at the sheet's line 29.
2. **Record the 25 Oct pin re-verification** in the sheet, and state which prerequisites are cleared: (a), (b) and the SHAs in (d). (c) and the fresh slice remain.
3. **(Rejected by the decision review, and not done.)** Give the item `depends_on: [float-training-module, d11-exporter-float-trunk]`. The review found the done-when's discrete-law route needs neither, so the edge would gate discrete M2 behind the float path. The item keeps `depends_on: []`, and its note names both routes.

## Files in scope

- `docs/harness/prep/m2-pin-train-mask-sheet*`
- `docs/harness/milestones.json`
- `docs/roadmap/items/m2-pin-train-mask-sheet.md` (the item's own file) and the rendered roadmap
- this task folder

## Done when

This unit does not meet the item's done-when: the FIT waits on (c). It is done when:
- the sheet carries the 25 Oct re-verification and says which prerequisites are cleared;
- the item's text records the state and both routes;
- the decision review passed and is logged on #2851;
- `loop report --check` and `loop pr-check --local` pass, and a fresh-context review counts at the exact head;
- the REPORT says `References #2851` (not Closes).

## Stop and ask if

- The sheet's registered band (line 29) turns out not to match §3.7.
- The decision review finds that another lawful path would let the sheet reach FIT without (c).
