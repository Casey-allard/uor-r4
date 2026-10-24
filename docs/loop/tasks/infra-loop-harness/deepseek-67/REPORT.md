# REPORT: infra-loop-harness, deepseek-67 (stale GPU collection records)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904, setup row 10. Base: `38bec04ba`. `References #2904`.

## What did not work, first

- **A killed watcher leaves no note at all.** #3124 fixed the error path (a watcher that returns an error now writes `stopped`), but `SIGKILL` or a host crash cannot write anything, so the record stays `running` and the source waits forever. This unit adds the bound Claude noted as optional: a running record older than six hours is stale and offered.
- **The rule is an age heuristic, not a heartbeat.** The honest alternative is a liveness heartbeat file written by the watcher each interval; that is a larger watcher-contract change and was left as a named follow-up, not smuggled in. Six hours is longer than any check-in interval and short enough to collect before a pod is usually reclaimed; the constant is public and documented so a future unit can tune it.
- **No fsync and no pruning remain from #3124's low findings** (stated there); this unit does not change the writer.
- **The stale record is offered, not auto-marked.** Marking it stopped would destroy evidence and could race a live watcher that is merely slow; a human/session decides whether to collect or close it.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopnext.rs` | `STALE_COLLECTION_SECS` (6 h, documented); the gpu-collections source treats an older running record as stale work with a stale reason, counts only recent running records as waits |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | new test: a stale running record is offered (reason names staleness), and a recently-running record still waits |
| `docs/harness/README.md`, `docs/loop/README.md` | both state the stale rule |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| stale record (started `now - 6h - 60s`) | `GO gpu-collection <id>` with "is stale running ... no live watcher"; test passes |
| fresh record (started now) | still waits: no GO, "GPU collection(s) running"; test passes |
| mutation: drop `\|\| stale(record)` | the stale test **FAILS**; restored it passes |
| `cargo test -p uor-r4-harness --test loop_next_sources` | 12 passed, 0 failed |
| `cargo test -p uor-r4-harness --test loop_next` | 33 passed, 0 failed |

## Checks run at the head

`cargo fmt --check`, `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`, `loop report --check`, `loop pr-check --local`, `loop gates --full` — see the closing line.

## Skills used

- `solution-integrity`: the review's optional finding became a bounded, tested rule; the heuristic and its alternative are stated; the test fails when the predicate is removed.
- `scope-guard`: one harness source, one gated test file, two shared registration docs.

## Still open

- **A real liveness heartbeat** (watcher writes a tick file each interval) would replace the age heuristic; named as a follow-up.
- **No real GPU collection has been recorded yet**; the first leased run will write one.
- **#3123's process miss** remains recorded on #2028; this ORDER names Setup row 10 and will not be enqueued before a counted pass.

## Plain summary

- A GPU collection whose watcher was killed can no longer block the loop forever: after six hours it is offered as stale work so a session can collect or close it, with tests and docs.
