# #1791 — lab-runner: add Action::Abandon to break the orphaned-reservation deadlock

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:43:14Z
- **Updated:** 2026-10-09T13:21:24Z
- **Closed:** 2026-10-06T04:46:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1791
- **Merged:** 2026-10-06T04:46:20Z
- **Merge commit:** c5282a6288b25ce453265458b59c0091a7086e63
- **Base / Head:** main / UOR-Foundation:deepseek/orphan-abandon
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Problem

The coordination store is pinned at `ba266ad4` (215 commits behind main), so `require_current_policy` refuses **every** `Claim`. `Action::AdoptPolicy` is the intended repair, but it refuses while any attempt is non-finalized — and exactly one is: `opencode-b0-flock-g1a2-20260930-g2` (issue #1512, task_epoch 6), reserved against `/Volumes/UOR-Workspace`, cancelled in the queue and never started. `FinishAttempt` needs a live claim (`owned()`), and re-claiming needs the failing policy guard. Three-way deadlock; no lab can claim work.

## Fix

Add an explicit, auditable `Action::Abandon` for a reservation that provably never produced compute. `FinishAttempt` is unchanged for live work.

- `Action::Abandon { attempt, session, reason, receipt }` finalizes a reserved attempt with **no `owned()` call**. Only the reserving session may abandon its own reservation, the reservation must still be open, and `reason` is non-empty and bounded (it remains auditable in the durable event log).
- `verify_exit_evidence(..., mode: EvidenceMode)` = `Finalize | Abandon`.
  - `Abandon` keeps every existing check: `receipt_binding`, this-host, canonical non-`/Volumes` runner root, receipt path inside `done(root)/<attempt>/`, regular bounded nonempty file, sha256, exit contents, and the `never_started` launch-state branch (queue/running/consumed/`attempt.json`/`process.json` absent, `cancel.json` present).
  - It **additionally requires** `exit.process_state == "never_started"`; `confirmed_stopped` or `unknown` evidence is refused (those cases keep the reviewed reconciliation paths).
  - It **replaces** the reserved-spec digest equality with a **job-identity** binding: the durable spec's `id` and `coordination.attempt_id` must both be the attempt, and the receipt must bind those exact durable bytes (durable digest + full coordination claim), while the reservation's issue/session/epoch must match. Digest and work-card drift between the reservation and the durable queued spec *is* the orphan condition, so binding identity rather than digest is the honest relaxation; it admits no compute.
- `Finalize` mode preserves today's behaviour exactly; existing assertions are unchanged (only the added mode argument at their call sites).

## Evidence

- `cargo test -p lab-runner --offline`: **108 passed / 0 failed before -> 114 passed / 0 failed after** (lib 84->90, cli 6, delivery 7, host_recovery 7, maintenance 4, doc 0).
- Six new tests: abandon succeeds for a reserved orphan with no live claim (and `AdoptPolicy` then drains); `confirmed_stopped`/`unknown` receipts rejected with the never-started reason; only the reserving session may abandon (unknown attempt, non-reserved attempt, empty/oversized reason also rejected); any launch state (queue/running/`attempt.json`/`process.json`, missing `cancel.json`) rejected; `FinishAttempt` still rejects the expired claim (and still finalizes under a live claim); operator event JSON round-trips through the real envelope parser.
- Read-only probe against the live store and the real DONE receipt (`exit.json` sha256 `a282d22b...`): `FinishAttempt`/Finalize -> `durable job spec differs from reserved spec`; `Abandon` -> `Ok`, attempt finalizes in memory, `AdoptPolicy` transition then succeeds. No store or runner state was written.

## Operator invocation (after review)

`lab-runner coord apply STORE EVENT_JSON` with `"op": "abandon"` (exact event in the review thread).

References #1512. Not queued for merge; review first.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6009505674 — 2026-10-06T04:44:38Z — Casey-allard

## Local validation at head `9202fdf7` (not queued; review first)

`cargo test -p lab-runner --offline --no-fail-fast` in the isolated worktree with `CARGO_TARGET_DIR=$HOME/.cache/uor-r4-orphan`:

| target | before | after |
| --- | --- | --- |
| lib (`coord`, `daemon`, `ledger`, ...) | 84 passed / 0 failed | **90 passed / 0 failed** |
| bin `lab-runner` | 0 | 0 |
| `tests/cli.rs` | 6 | 6 |
| `tests/delivery.rs` | 7 | 7 |
| `tests/host_recovery.rs` | 7 | 7 |
| `tests/maintenance.rs` | 4 | 4 |
| doc-tests | 0 | 0 |
| **total** | **108 passed / 0 failed** | **114 passed / 0 failed** |

`cargo fmt -p lab-runner -- --check` clean. One pre-existing flake observed once in the unmodified baseline (`cancel_finalizes_a_running_job`: `supervisor identity changed before stop`); it passed on re-run and in both full runs.

New tests: `abandon_finalizes_a_reserved_orphan_without_a_live_claim`, `abandon_rejects_stopped_or_unknown_process_evidence`, `abandon_requires_the_reserving_session_and_an_open_reservation`, `abandon_rejects_any_launch_state_or_missing_cancel_marker`, `abandon_event_round_trips_through_the_operator_json_shape`, `finish_attempt_still_requires_a_live_unexpired_claim`.

### Read-only probe against the live orphan (no store or runner state written)

Real store state (`seq 901`, `observed_server_time 1791261070`) and real DONE receipt (`exit.json` sha256 `a282d22bf0a52951cc024a776fb4599c2e59e42f868d13f774bb2386bc84bddf`, copied to a scratch root so `job_lock` could not write live runner state):

```
stored attempt: phase=reserved session=opencode-deepseek-20260930 epoch=6 spec_sha256=4147ddc9...
live root canonical=true under_volumes=false;  receipt host == this host: true
durable spec canonical digest=3d4fc50ff9e08a1ddb392600e6acfdee60dc62f10c413a2d45cc04c8c8ee8b43
FINALIZE -> Err(Invalid("durable job spec differs from reserved spec"))
ABANDON  -> Ok(())
after abandon: attempt phase=finalized receipt_present=true
ADOPT -> policy adopted
```

Note for review: the reservation and the durable queued spec also disagree on `coordination.work_card` (`9dd6655b...` vs `d65fe7ae...`), so the reserved-digest-bound work-card check in `validate_exit_contents` would also have refused this receipt. `Abandon` mode therefore binds the durable job identity (durable `id`/`coordination.attempt_id` plus the receipt's exact durable digest and full coordination claim) and re-checks the reservation's issue/session/epoch, while deliberately not re-binding the drifted digest or work card.

### Operator invocation (after review)

```
# build the fixed control-plane binary from this branch
cd ~/uor-r4-worktrees/orphan-abandon
CARGO_TARGET_DIR=$HOME/.cache/uor-r4-orphan ~/.cargo/bin/cargo build -p lab-runner --offline --release

# apply one abandon event to the live coordination store
$HOME/.cache/uor-r4-orphan/release/lab-runner coord apply \
  /Users/casey.allard/.local/share/uor-r4/coord.git \
  /tmp/orphan-abandon-1512.json
```

with `/tmp/orphan-abandon-1512.json` exactly:

```json
{
  "event_id": "orphan-abandon-1512-20261006",
  "op": "abandon",
  "attempt": "opencode-b0-flock-g1a2-20260930-g2",
  "session": "opencode-deepseek-20260930",
  "reason": "queued cancellation orphaned by the /Volumes/UOR-Workspace drive failure; never started",
  "receipt": {
    "host": "host-284b64fb2f70567aee465dc4941430076137e580d19d9568b15726d14b157cec",
    "job_id": "opencode-b0-flock-g1a2-20260930-g2",
    "attempt_id": "opencode-b0-flock-g1a2-20260930-g2",
    "path": "/Users/casey.allard/.local/share/uor-r4/runner/done/opencode-b0-flock-g1a2-20260930-g2/exit.json",
    "sha256": "a282d22bf0a52951cc024a776fb4599c2e59e42f868d13f774bb2386bc84bddf"
  }
}
```

The CLI parses this envelope (verified with the built binary: valid JSON reaches the store step, an envelope missing `session` is rejected as JSON). After it commits, `Action::AdoptPolicy` can drain every finalized attempt, and claims resume once the store policy is current.



---
_Comment/review ids in this file: 6009505674_

