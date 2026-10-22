# REPORT: infra-loop-harness, claude-26 (the PM's item edits for the bench, while Kimi is out)

Lab: claude. Base: `a8ff5690e`. Order: [ORDER.md](ORDER.md), committed first (`c6c6db8e8`). The cancel probe found that no open item's scope listed the bench feed's files or Kimi's REPORT, and that no item carried an `AGENTS.md`-only inbox rule.

## What did not work, first

- **The PM is out, so the PM's edits are done by Claude, at Casey's direction.** Kimi should check them on return. Casey's words: "Claude does them (Recommended)".
- **The copyright rephrase is not in this PR,** although Casey's answer said "Claude, in claude-26 (Recommended)". `loop pr-check` reads scope from the base commit, so `kimi-1/REPORT.md` has to be in the umbrella's scope on `main` before a PR may edit it. This PR adds that path, and claude-27 does the one-sentence rephrase.
- **My first wording of the new item reused the bench's status-marker phrase from CorvusWing** (2 matching 8-word runs). It is MIT-licensed, but D44 keeps kit text out of the repository, so I reworded it (`3c5219750`). Afterwards there were 0 matches against LoopKit and 0 against CorvusWing.

## What changed

| Commit | What |
|---|---|
| `035603b9d` | `infra-loop-harness`'s `scope:` gains `crates/uor-r4-harness/src/loopbench.rs`, `crates/uor-r4-harness/tests/loop_bench*.rs` and `docs/loop/tasks/infra-loop-harness/kimi-1/REPORT.md`; nothing else in that item changed. A new item, `infra-bench-inbox-rule`, has `scope: [AGENTS.md]` and states Casey's per-lab inbox decision as its done-when. `docs/roadmap/README.md` is re-rendered (88 items). |
| `3c5219750` | The new item's status markers are reworded in our own words. |
| this REPORT's commit | This REPORT. |

## Measurements

None. This is a roadmap record, with no model run.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK (ready 48, held 26, decision 4, done 10). `AGENTS.md` is now in three ready items' scopes (pr03-render, pr08-migration and the new one); item scopes may overlap, and only claims may not.
- `cargo xtask roadmap render` reproduces the committed README byte for byte.
- Claim-wording finds nothing in the changed files. `xtask links` finds 930 broken links repo-wide, as on `main`, and none in the changed files. The copyright check finds 0 of 823 8-word runs matching LoopKit and 0 matching CorvusWing.

## Skills used

scope-guard: the PR changes only the two item files, the shared roadmap page and its own task folder. The rephrase that would have reached outside the base scope is deferred to claude-27.

## Still open

- claude-27: rephrase the one sentence in `kimi-1/REPORT.md:37`.
- claude-25: the bench feed, which merges once this scope is on `main`.
- The inbox line in `AGENTS.md`, under `infra-bench-inbox-rule`.
- Kimi checks these edits on return.

## Plain summary

While Kimi is away, this does Kimi's bookkeeping so three of Casey's bench decisions can go ahead: the feed's new files are allowed, a task now exists for the inbox rule, and the copied sentence can be fixed next.
