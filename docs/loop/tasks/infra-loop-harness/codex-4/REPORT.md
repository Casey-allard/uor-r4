# REPORT: infra-loop-harness, codex-4

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim infra-loop-harness --lab-topic drive-etxtbsy` was refused with exit 2 because the
  item's own scope lists the shared registration file `docs/harness/gates.json`, which no item
  scope may list. The work continued under the owner's direction.
- **Records incident:** the first ORDER write used the `codex-1` path, which already held an
  older Codex unit, and truncated it. The original file was restored byte-for-byte from
  `origin/main` before any code changed, and this unit moved to the next free number, `codex-4`
  (`codex-1`, `-2` and `-3` already exist). No other record was touched.
- The first `cargo fmt --check -p uor-r4-harness` wanted `std::io::Error::other` wrapped; the
  line is formatted.
- The first draft (PR #3995) was closed because `test-freeze` requires a `TEST-CHANGE:` line on the
  commit that modifies the existing test file, and its branch could not be rewritten without a
  force-push. This fresh branch applies the same code and test and carries that line; the
  unreachable trailing `Err` was also removed.
- The retry cannot prove the filesystem's transient behavior on every host; it bounds the wait
  and still fails a persistent `ETXTBSY`.

## What changed

- `crates/uor-r4-harness/src/loopdrive.rs`: the one `gh` spawn in `post_note_with` moved into
  `run_note_gh`, which retries up to five times with a 10 ms doubling backoff **only** when the
  OS returns `ETXTBSY` (errno 26), and returns every other error unchanged. `post_note` and the
  test use the same helper.
- `crates/uor-r4-harness/tests/loop_drive.rs`: a new regression test holds the fake `gh` open for
  writing while `post_note_with` runs; the first exec is busy, the helper waits for the writer to
  close, and the note posts. Without the retry the test fails.

Size: one helper, one call site, one regression test and the loop records.

## Measurements

- `cargo test -p uor-r4-harness --test loop_drive` — `29` passed (28 before, plus the retry test).
- `cargo test -p uor-r4-harness` — all suites pass.
- The retry test deterministically produces `ETXTBSY` (the script is open for writing) and passes
  once the writer closes after 30 ms.
- No model, capability or milestone result is measured.

## Checks run at the head

- `cargo test --locked -p uor-r4-harness --test loop_drive` — `29` passed.
- `cargo test --locked -p uor-r4-harness` — passed.
- `cargo fmt --check -p uor-r4-harness` — passed.
- `test_freeze.py check --range origin/main..HEAD` — GREEN (the test-modifying commit carries
  `TEST-CHANGE:`).
- `cargo clippy --locked --no-deps -p uor-r4-harness --lib --tests` — clean in the touched files.
- `git diff --check` — passed.
- `uor-harness check --only harness.native_space_contract` — 1 rule, 0 failed.
- `uor-harness loop report --check`, `loop pr-check --local` and the exact-head review — at the head.

## Skills used

- `two-strikes`: the same check had failed twice in fresh-main gates; this unit fixes the
  instrument instead of re-running a third time.
- `solution-integrity`: bounded the retry to the one transient errno and kept every other error,
  so a real failure is not hidden.
- `test-freeze`: added a regression test; changed or deleted none.
- `working-discipline`: recorded the overwritten-record repair and restored it before proceeding.

## Still open

- The underlying filesystem timing is mitigated, not removed; a persistently busy script still
  fails after the bounded attempts.
- The `infra-loop-harness` item's scope still lists the shared `docs/harness/gates.json`, so
  `loop claim` on it remains refused. Fixing the item is a separate scope PR.
- The real one-space language-model objective remains the next model unit.

## Plain summary

The loop's note-posting step now waits out a brief "text file busy" from a just-written helper
script instead of failing the whole gate. A regression test holds the script open to force that
condition. Persistent errors still fail.
