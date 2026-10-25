# REPORT: m2-pin-train-mask-sheet, claude-99

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2851. References #2030.

## What did not work, first

- **The decision review rejected this unit's main change, and it was right.** I had given the item `depends_on: [float-training-module, d11-exporter-float-trunk]`, on the premise that these were the only blockers to its FIT. But the item's done-when is an either/or: a refreshed D37 sheet, **or** the pins carried into a discrete-law sheet, each with a non-author FIT. The discrete-law route needs neither float item.
  - `m2-train-grounded-replies` depends on this item and keeps the student on the discrete path (D41). So the edge would have made discrete M2 wait on the float path, and on float training too: `d11-exporter-float-trunk` needs a trained float trunk.
  - That misreads Casey's "I want both because we do not know which we will ultimately need". It is a hedge across both routes, not a float-first order.
  - My ORDER's own stop rule (a lawful path to FIT without (c)) should have caught this. The edge is dropped, and the state note now names both routes.
- **The item's done-when is not met in this unit.** Neither sheet has a FIT yet. The D37 sheet waits on the float module and exporter. The discrete-law sheet, which this item's done-when allows instead, is not yet written.
- **My first ORDER listed the export-gap band as still to register.** It was already registered at the D37 sheet's line 29 by the PM cover on 24 Oct, so no band is added.
- **The ORDER promised a `milestones.json` refresh that was not needed.** The M2 `next` text already names the remaining blockers, so the file is unchanged.
- **An advisory scope WARN (D76).** `loop pr-check` allows only an item's status line by default, and this unit adds a dated note to the item's text and re-renders the roadmap page.
- **No notebook row.** This unit re-verifies pins and records state. It measures nothing, and the item names no `notebook:` page.

## What changed

- **`docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md`:** one line in its source pins records the 25 Oct re-verification at `53b40ecd6`:
  - the mask member `mw-balp/train/response_mask.u8`, extracted from the HF store's `data/ft-balp.tar` (sha256 `b1d22113…473f`), is 7,512,797 bytes with sha256 `b5e7352d…49e3`, matching the pin;
  - `9e904a83` is an ancestor of `main`;
  - the count reference `native_rung0.rs` is byte-identical between `2fd6358bb` and `main`.
- **`docs/roadmap/items/m2-pin-train-mask-sheet.md`:** a dated "State on 25 Oct" note at the top of "What is missing". It records the owner's answer, what the D37 sheet holds and what blocks its FIT, and that the discrete-law route needs neither float item. `depends_on` is unchanged (`[]`), and the original text is kept below the note.
- **`docs/roadmap/README.md`:** re-rendered.

## Measurements

None. The pin checks are hash and git checks on the laptop. Nothing was built, trained or run on a GPU.

## Checks run at the head

- **Mask member:** `tar -xOf data/ft-balp.tar mw-balp/train/response_mask.u8 | shasum -a 256` gives `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, and `wc -c` gives 7,512,797. The reviewer confirmed both, and the tar's size.
- `git merge-base --is-ancestor 9e904a83… origin/main` holds. `git diff 2fd6358bb origin/main -- …/native_rung0.rs` is empty.
- `cargo xtask roadmap check`: 92 items OK.
- **Review history:**
  - The first fresh review returned CHANGES, with DECISION REVIEW: REJECT on the dependency, and found no cycle. The fixes above follow it.
  - The round-2 review is in the PR's review record.

## Skills used

- holding-the-goal: the done-when has a discrete route, so the next step is that sheet, not a wait on the float path.

## Still open

- **The discrete-law rung-0 sheet.** The owner wants it, it closes this item without the float path, and it serves the discrete M2 route. It needs the discrete learning law's trainer, arms and bars, carrying this item's mask, count and source pins.
- **The D37 sheet** goes on to FIT once `float-training-module` (#2823) and `d11-exporter-float-trunk` (#2820) exist and its fresh-transfer slice is sealed.

## Plain summary

The M2 training sheet's open question was already answered by Casey ("both"), and its pins still check out. The float version waits on tooling that doesn't exist yet. The discrete version doesn't, so it's the way to finish this item.
