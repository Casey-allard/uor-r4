# REPORT: infra-loop-harness, deepseek-65 (GPU collection record)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2904, setup row 10. Base: `7395b468e`. `References #2904`.

## What did not work, first

- **The previous unit's ORDER missed the setup row and its review did not count.** #3123's counted review was rejected with "the ORDER names no valid setup row" because this is a loop-setup item; that unit merged without a counted record. This ORDER names **Setup row: 10** up front, and I will not enqueue until `loop review --check` prints a counted pass. The miss is recorded on #2028.
- **The record's home is a design decision, not a given.** The docs said "a collection record" without saying where or what fields. I kept it local and gitignored (`local/<lab>/gpu-collections/<id>.json`), consistent with `PICKUP` and the supervisor's session record, so a failed run never commits stray pod data; the ORDER states the fields and the atomic write.
- **The watcher's stop reasons are string-matched.** `watch_lease_until_renew_fails` reports a lease end as a note containing "lease ended" and a bound as "test bound"; the record maps those to `finished`/`stopped` rather than changing the existing watcher contract. If that wording changes the mapping must change with it (the test pins both).
- **Malformed records are skipped silently.** A corrupt `.json` under the collection directory is ignored rather than turning `loop next` into a failure; the ORDER records that choice.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | `GpuCollection` record; `collection_dir`/`collection_id`/`collection_path`; atomic `write_collection`; `read_collections` (skips malformed/foreign files); `collect_collection`; `watch_lease_collect`; CLI `--watch --lease` now writes/updates the record, and `--collect <id> [--artifact <path>]` marks it collected |
| `crates/uor-r4-harness/src/loopnext.rs` | `gpu-collections` is built: finished uncollected record -> `GO gpu-collection <id>`; running -> wait; none -> none; read error -> not-built with the reason |
| `crates/uor-r4-harness/tests/loop_drive.rs` | two tests: record lifecycle (`running` -> stopped/finished, collect idempotent) and malformed-file/id safety |
| `crates/uor-r4-harness/tests/loop_next_sources.rs` | one test: a finished record is offered, a collected one is not |
| `docs/harness/README.md`, `docs/loop/README.md` | the source is no longer described as not built; they name the record, the offer and `loop drive --collect` |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| `cargo test -p uor-r4-harness --test loop_drive` | 26 passed, 0 failed |
| `cargo test -p uor-r4-harness --test loop_next_sources` | 10 passed, 0 failed |
| `cargo test -p uor-r4-harness` (all targets) | see closing line |
| planted fault | dropping the `!record.collected` filter in `loopnext.rs` turns `a_finished_gpu_collection_is_offered_and_a_collected_one_is_not` red (verified by mutation, restored) |
| `loop next` on the head | `gpu-collections: no GPU collection record is waiting` (built, not "not built") |
| GPU/model/bar/hold | none touched |

## Checks run at the head

`cargo fmt --check`; `loop report --check`; `loop pr-check --local`; `loop gates --full` — see the closing line. No record is committed (the collection directory is gitignored).

## Skills used

- `solution-integrity`: the source is built rather than faked; the test fails with the filter set aside; the record is local and atomic so a crashed watcher cannot leave a half-written offer.
- `scope-guard`: only the two harness sources, the two gated test files, the two shared registration docs and this folder.

## Still open

- **No real GPU collection has been recorded yet** (no GPU run in this unit); the next real leased run will write the first record.
- **A collected record keeps its record** (so the history is visible in `local/<lab>/gpu-collections`); there is no automatic prune, and a future unit may add one.
- **The supervisor's step-15 sentence** still says "a watcher for a leased GPU run"; with the record built, a follow-up may state that a finished collection is offered through `loop next` (this unit updates both registration docs, not the step table).

## Plain summary

- The loop can now record a leased GPU run and offer its collection as work: the watcher writes a local record, `loop next` goes on a finished uncollected one, `loop drive --collect` closes it. Tests, a planted fault and both registration docs cover it.
