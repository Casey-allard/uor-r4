# REPORT: m2-reply-grading-instrument-r14, kimi-1

Lab: kimi. Order: [ORDER.md](ORDER.md). References #2852. Closes #2852.

## What did not work, first

- The first attempt at this flip (#2962, kimi-9) named the setup umbrella item
  (`infra-loop-harness`) in its ORDER. The review checker ruled the ORDER invalid:
  an order under the setup umbrella must name a valid `Setup row:`, and either way
  the PR becomes a setup PR under D59 whose review only claude can give. The flip
  of a research item belongs under the research item's own folder, where the
  item's own status line is the always-allowed change and the doc-only same-lab
  fresh-context review path is valid. #2962 was closed unmerged and redone as this
  order; the lesson is recorded for every future PM flip.
- The item spent half a day `ready` after its substance completed because #2956
  said `References #2852` and no closing PR flipped the status — the same
  record-lag class as the pr03-render flip (kimi-8).

## What changed

One word in one file: `docs/roadmap/items/m2-reply-grading-instrument-r14.md`
`status: ready` → `status: done`. Docs class; size 1 line, ratio 1.0 against the
ORDER's estimate.

## Measurements

Nothing was measured; this is a metadata edit. The substance it records was
measured in the day's three review records: 26/26 `chat-grade` tests at
`494bad222` (kimi re-run), all 30 MANIFEST.sha256 files OK, ratchets d23/d25/d32
zero growth, `xtask links` back to 930 baseline at `378f4b3ab` (#2961 review).

## Checks run at the head

- Cancel probe at base `4797ca883`: item file read (`status: ready`); #2956,
  #2961, #2960 all MERGED; the R14 ruling posted on #2029 (6339245936, 11:07Z).
  Task stood.
- `uor-harness loop gates --quick` and `--full`, `uor-harness loop report --check`,
  `uor-harness loop pr-check --local`, `cargo run -q --locked -p xtask -- roadmap
  check` and `roadmap render` — results recorded before push.

## Skills used

None.

## Still open

- The M1 reply-quality headline stays SUSPENDED with no target (per the R14
  ruling); a numeric reply bar is a separate owner/PM card.
- `infra-loop-pr08-migration` (owner-gated) and `infra-loop-pr10-supervisor`
  (completes when #2949 lands) remain `ready`, correctly.

## Plain summary

The reply-grading instrument item was finished in substance — its design note is
on the M1 issue, the abstention test is built and reviewed by two labs, its link
defects are repaired, and the PM ruled it passes the R14 non-vacuity requirement —
but its roadmap line still said "ready". This PR changes that one word to "done"
so the work router and the status pages show the truth, and it does so under the
item's own task folder, which is the shape the review checker accepts for a
doc-only PM record fix.
