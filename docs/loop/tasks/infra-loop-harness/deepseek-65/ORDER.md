# ORDER: infra-loop-harness, deepseek-65 (GPU collection record)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 10 (the drive/watchers row: "a GPU run needs a collection record the watcher does not write yet"). Base: `7395b468e`. `References #2904`.

## Goal

`loop next` has one source it cannot offer: `gpu-collections: not built — a GPU run needs a collection record the watcher (loop PR 10) does not write yet`. Build it end to end:

1. **Record.** `loopdrive.rs` gains `GpuCollection` (id, lab, renew argv, start/end Unix times, status `running`/`finished`/`stopped`, `collected`, artifacts, watcher notes), a gitignored path under `local/<lab>/gpu-collections/<id>.json`, atomic writes, a reader that skips malformed/foreign files, and `collect_collection` which marks a record collected and appends artifact paths idempotently.
2. **Watcher.** `loop drive --watch --lease` writes the record `running` before renewing and updates it on exit: `finished` when the renew command reports the lease ended, `stopped` when a test bound or limit stops the watcher.
3. **Offer.** `loop next` replaces the `not_built` entry: a finished (or bound-stopped) record that is not collected becomes `GO gpu-collection <id>` with a reason naming the need to save artifacts; a running record waits; no records is `none`; a read failure is `not_built` with the reason.
4. **Collect.** `loop drive --collect <id> [--artifact <path>]...` marks the record collected, so the source stops offering it.
5. **Docs.** `docs/harness/README.md` and `docs/loop/README.md` stop describing the source as not built and name the record, the offer and the collect command.

This is a host-only harness unit: no GPU run, model, bar, hold, owner release or scope change.

## Cancel probe (run first)

At `7395b468e`: `grep -n 'gpu-collections' crates/uor-r4-harness/src/loopnext.rs` shows the `not_built` call, and `grep -n 'GpuCollection' crates/uor-r4-harness/src/loopdrive.rs` is empty. The task stands. Class: instrument.

## Files in scope

- `crates/uor-r4-harness/src/loopdrive.rs`
- `crates/uor-r4-harness/src/loopnext.rs`
- `crates/uor-r4-harness/tests/loop_drive.rs`
- `crates/uor-r4-harness/tests/loop_next_sources.rs`
- `docs/harness/README.md` and `docs/loop/README.md` (shared registration files)
- this task folder

## Done when

- A lease watch writes one record atomically, transitions it `running` -> `finished` (renew fails) or `stopped` (test bound), and `loop next` offers exactly a finished, uncollected record as `GO gpu-collection <id>` and stops offering it after `loop drive --collect`.
- Malformed records and non-`.json` files are skipped, not guessed; a collection id is filesystem-safe; the reader returns no records when the directory is absent.
- Host tests cover the record lifecycle, the offer/collect transition, and the malformed-file case in the gated `loop_drive` and `loop_next_sources` suites; a planted fault (drop the `!record.collected` filter) turns the new `loop next` test red.
- `cargo test -p uor-r4-harness`, `fmt`, `loop report --check`, `loop pr-check --local` and `loop gates --full` pass; a counted other-vendor exact-head review records at the head.

## Stop and ask if

- A GPU run, model, bar, hold, owner release or a file outside the list is needed; a record field cannot be read from the watcher without changing the lease contract.
