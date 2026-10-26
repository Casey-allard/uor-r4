# REPORT: infra-loop-harness, claude-121 (narrow `d11-exporter-float-trunk`'s scope)

Lab: claude. Order: [ORDER.md](ORDER.md). References #2820.

## What did not work, first

- **The claim of the released item was refused.** `loop claim d11-exporter-float-trunk` refused right after claude-120 merged: `docs/mechanisms/lut-serving.md` is also in `infra-loop-harness`'s scope, which Codex's draft #3169 holds. That draft has been idle since 2026-10-24T23:53Z and never touches the page. The claim lock compares declared scopes, not the files a claim actually changes. That is why this second small unit was needed.
- **The mechanism page can now go stale.** `lut-serving.md` says the serving path stays READY "until a lawful trained model is exported through it". Without the page in scope, the exporter's results cannot update it in the same PR. So the item now names who does that: an infra-loop-harness item-maintenance unit once the umbrella is unclaimed. Every exporter REPORT lists it as open.
- **`m1-export-gap-and-toks` will hit the same refusal.** It also lists `docs/mechanisms/lut-serving.md` in its scope. This is recorded on #2820, but not changed here, because it is outside this ORDER.

## What changed

Size: docs 1 file (+3/−1), plus this task folder. The ORDER gave no estimate.

- **`docs/roadmap/items/d11-exporter-float-trunk.md`:** `docs/mechanisms/lut-serving.md` is gone from `scope:`. A dated scope note in "What is missing" says where the results are recorded and who updates the mechanism page.
- **`docs/roadmap/README.md`:** unchanged. Scope is not rendered, and a fresh render wrote it identically.
- **The decision review record** is the D68 owner log on #2820.

## Measurements

None. The decision's evidence:
- the refusal text;
- the umbrella's 38 scope globs, of which only this one is shared;
- #3169's files and last commit;
- the harness sources, which check that a mechanism page exists but never require one to change with its code (`prep.rs:845`, `briefingrules.rs:65`).

## Checks run at the head

See the exact-head review record on the PR: `xtask roadmap check` (93 items OK), `loop report --check` and `loop pr-check --local`.

## Skills used

- **holding-the-goal.** The refusal was read as the lock working, so the fix narrows this item's scope rather than overriding or touching Codex's claim.

## Still open

- **The exporter unit,** claimed next as `d11-exporter-float-trunk`.
- **The `lut-serving.md` row and status for the exporter's result,** once the umbrella is unclaimed.
- **The same overlap on `m1-export-gap-and-toks`.**

## Plain summary

The project lets only one lab work on a set of files at a time, judged by the files each task says it might touch. The exporter task listed one documentation page that a much larger maintenance task, held by another lab that has been idle for over a day, also lists. That one shared page blocked the start. This change removes the page from the exporter task's list and writes down who updates it later, so the exporter work can start without touching the other lab's claim.
