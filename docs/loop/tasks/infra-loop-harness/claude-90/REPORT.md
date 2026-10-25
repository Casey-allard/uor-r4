# REPORT: infra-loop-harness, claude-90

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904
References #2835
References #2833

## What did not work, first

- **`loop claim infra-loop-harness` refused.** It said "already claimed by PR #3169 (codex-1)". #3169 is Codex's draft, idle since 01:07Z and conflicting with main. Because this umbrella item covers every loop-harness unit, one inactive draft locks them all for every lab.
  - This unit shares no file with #3169. So this ORDER was written by hand, as claude-88's was.
  - The lock design is listed under Still open.
- **The two defects came from my own reset (#3171).** It gave every M1 item the default scope `crates/uor-r4-discrete/**`.
  - For C2, whose crate work was already done, that default made the claim collide with Antigravity's A8 order for no file reason.
  - Separately, `m1-address-as-read` was marked ready with no dependencies, even though its done-when needs the trunk's checkpoint.

## What changed

- **`docs/roadmap/items/m1-bvg-k-block-apply-c2.md`.**
  - The scope is now its docs only: the C2 and Stage T prep sheets (`m1-bvgk-stage-t*`, the name its step 4 gives, and `m1-bvgk-staget*`, the existing draft), its lab and task folders, and the M1 notebook page. The fresh review caught that the first glob missed the sheet name and the notebook row, and that two older owner-release phrases remained; all three are fixed.
  - Two sentences change from "typed owner release, training hold" to the current rule: a FIT, then the lab's own pre-run review (D67, D77).
- **`docs/roadmap/items/m1-address-as-read.md`.**
  - It now has `depends_on: [m1-trunk-assembly-and-training]`.
  - Its how-to-start step 5 now names the trunk checkpoint, and its approval wording is D67's.
- **The decision record:** `[claude] owner log:` notes on #2835 and #2833 (D69). A first draft put it in `DECISIONS.md` as D80, but `loop pr-check` refused that, because `DECISIONS.md` is an owner path. The owner log is where a lab's own decision goes.
- **`docs/roadmap/README.md`:** re-rendered by `cargo run -p xtask -- roadmap render`, with the new dependency edge.

## Measurements

None. This unit edits the roadmap only.

## Checks run at the head

- `uor-harness check` exits 0, with warnings only: the M2 registry-versus-card headline, and stale `team claim` records.
  - I released my own stale claim, `claude/loop-probe-results-20261021`, whose work is already on main.
  - The Codex ones are left for Codex.
- `uor-harness loop report --check` and `loop pr-check --local` were run before the PR went ready (see the PR).

## Skills used

None.

## Still open

- **The umbrella-item lock.** `loop claim` holds a lock per item, so an umbrella item such as `infra-loop-harness` lets only one open order exist across all labs. A stale draft then blocks everyone. A follow-up should either key the claim lock on scope overlap alone for umbrella items, or ask the PM to nudge #3169 once it passes 24 hours idle.
- **C2's Stage T sheet** is the next unit (claim `m1-bvg-k-block-apply-c2` after this merges).

## Plain summary

Two roadmap entries were wrong in ways that wasted lab time. One blocked Claude from writing the Stage T sheet while Antigravity works on A8, although the two touch different files. The other offered an item that cannot be finished until the M2 trunk exists. Both are fixed, and the reasons are in the owner log on both issues.
