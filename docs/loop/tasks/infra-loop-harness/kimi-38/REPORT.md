# REPORT: infra-loop-harness, kimi-38 (within-cell writer roadmap item)

Lab: kimi. Order: [ORDER.md](ORDER.md).
References #2904, #3128 and #3129; this PR closes none of them.

## What did not work, first

- `loop claim infra-loop-harness --lab-topic within-cell-item` refused because #3169 holds the umbrella. The probe confirmed no file overlap, so the run file's narrow hand-written-ORDER exception was used. No active scope was bypassed.
- The first roadmap check failed on two item-format errors: an 85-character title and a missing `## Sources` section. The title was shortened and the sources added; the next check passed.
- The first writer design failed independent builder and architecture review: it had an identity/loadability contradiction, an unbounded relation accumulation, an under-specified frame-source hypothesis and confounded controls. The roadmap item now requires bounded arithmetic, legacy replay, a zero-update B0, simpler frame controls and a strict paired floor before any training.
- The first exact-head PR review found that the ORDER lacked a setup row, four claim boundaries were missing from the item, the probe lacked idle-draft evidence, and the sync stop rule omitted the expected checklist update. `loop review` reproduced the setup-row refusal before the fix; all four findings were repaired for re-review.
- No GPU run or mechanism code was started, and #3128/#3129 are not closed by this bookkeeping unit.

## What changed

- Added `m1-within-cell-writer` as a ready M1 mechanism item. Its discrete scope remains unavailable while #3258 is open.
- The item requires error decomposition, a zero-update frame-source comparison, capped relation arithmetic, identity-safe replay, matched lookup/reranker controls and the strict paired floor before training.
- Regenerated `docs/roadmap/README.md`; added the ORDER, cancel probe and this REPORT.
- Size at the review-repair commit: 5 files, 187 additions and 2 deletions. Task documentation (ORDER/probe/REPORT) 122 lines, roadmap item 60, generated view 5 additions and 2 deletions; no code or test files. The ORDER set no numeric estimate, so an estimate ratio is not applicable.

## Measurements

- `roadmap sync` dry run proposed exactly one new issue and the expected #2029 checklist change; it was not applied before merge.
- `roadmap check`: 94 items OK; ready 20, held 15, decision 1, done 58.
- No model compute ran and no capability was measured.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap sync`: expected dry-run plan only.
- `... roadmap render`: generated the 94-item view.
- `... roadmap check`: 94 items OK.
- `uor-harness loop gates --quick`: GREEN at the item commit.
- `uor-harness loop gates --full`: 11/11 GREEN at `2ce92df23ea93c744af4483eb15d34f7d34a99c0`.
- `git diff --check origin/main`: clean.
- `uor-harness loop report --check --order docs/loop/tasks/infra-loop-harness/kimi-38/ORDER.md`: see final head.
- `uor-harness loop pr-check --local`: see final head.

## Skills used

- `uor-research-architect`: whole-project mechanism and evidence review.
- `superpowers:brainstorming`, `writing-plans`, `executing-plans`, `using-git-worktrees`, `test-driven-development`: staged design and delivery.
- `holding-the-goal`, `solution-integrity`, `scope-guard`: the item targets the observed writer bottleneck without crossing active scopes or promoting an untested law.

## Still open

- The item issue is created only after merge by `roadmap sync --apply`, and only if the dry run is unchanged.
- #3258 still blocks the discrete mechanism unit. B0, training and D11 have not run.
- #3128 and #3129 remain open until their own acceptance conditions are met.

## Plain summary

This adds a work item for the real problem: the model finds the right small group of tokens but often picks the wrong token inside it. It does not change the model. It requires a small, controlled read experiment before anyone pays for training, and it keeps older models and current labs' work untouched.
