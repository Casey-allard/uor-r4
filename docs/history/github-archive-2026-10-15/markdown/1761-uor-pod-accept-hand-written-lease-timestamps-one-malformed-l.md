# #1761 — uor-pod: accept hand-written lease timestamps; one malformed lease bricked the tool

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T21:10:53Z
- **Updated:** 2026-10-09T13:22:58Z
- **Closed:** 2026-10-05T21:40:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1761
- **Merged:** 2026-10-05T21:40:03Z
- **Merge commit:** cb8bd24d8209ef00f69362c7a4ff8a7760162b16
- **Base / Head:** main / UOR-Foundation:deepseek/uor-pod-timestamp
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The bug: one hand-written lease took the shared tool down for all three labs

`uor-pod status` aborted for every session with:

```
jq: error (at .../leases/wtrl0eq52l1j9u/codex.json:11): date "2026-10-05T21:48:48.245421+00:00"
does not match format "%Y-%m-%dT%H:%M:%SZ"
```

The offending file was written by hand while the tool was unmerged, in a form the tool never emits (fractional seconds plus an explicit `+00:00`). Because the tool runs `set -euo pipefail`, the failing `fromdateiso8601` inside `live_leases_json` killed the whole command — so **Claude, Codex and DeepSeek all lost `status`, `lease`, `up` and `down` at once** because of one session's file. That is precisely the failure mode the hand-edit warning in `docs/labs/compute.md` ("Lease files are written only by `uor-pod` … Editing them by hand races other sessions") predicts, and it happened within hours of the rule being written.

## The fix

Normalise before parsing, in the four sites that used the bare builtin (`iso_epoch`, `lease_live`, `live_leases_json`, `all_leases_json`):

```jq
def ts: try (sub("\\.[0-9]+$";"") | sub("\\+00:00$";"Z") | fromdateiso8601) catch 9999999999;
```

Two deliberate choices:

- **Fail safe, not open.** An unparseable timestamp counts as **LIVE** (far future), not expired. If a lease cannot be read, the conservative outcome is to treat the GPU as held; treating it as expired would let one session take a GPU another session is actively using. The `catch` also means no future malformed file can ever abort the tool again.
- **Both directions covered.** A lease in the canonical `…Z` form parses exactly as before, so no existing behaviour changes.

## Tests

New regression test writes a lease in exactly the hand-written shape that caused the outage, and asserts three things: the tool **survives** (`status`, explicitly matching the pod header — this test fails with rc=1 before the fix), the lease is **listed**, and its GPU is **still refused** to another session.

`scripts/pod/tests/uor-pod-dryrun.sh`: **64 passed, 0 failed** (61 before). Run with no real mutation.

## A second issue found while using the tool (not fixed here)

`up_attempt` breaks at the **first** datacenter whose stock table advertises the GPU, and `cmd_up` then makes a single create attempt in that DC. If the create fails with `graphql error: There are no longer any instances available…`, the tool **dies** instead of continuing to the next DC in the placement order:

```
Creating 2 x 5090 ($1.98/h) in EUR-NO-1 …        <- stock said "Low"
uor-pod: pod create failed: … no longer any instances available …
```

I worked around it with `UOR_POD_VOLUME_DCS=<dc>`, and hit the same failure in EU-RO-1 and EUR-IS-1 before a `--count 1` create succeeded in EU-RO-1. Note the side effect worth being aware of: the failed attempts each ran `ensure_volume`, so `uor-shared-EU-RO-1` and `uor-shared-EUR-IS-1` were created (documented lazy behaviour, but they are now billed).

Suggested follow-up: on a create failure, mark that DC exhausted and retry the remaining ones before `die`, and make `--wait` treat a create failure like no-stock rather than exiting immediately. Happy to write it if useful.

References #820


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6003498692 — 2026-10-05T21:38:15Z — Casey-allard

Second occurrence, now on the merged tool (`main` @ `e445ee94`), so this is not a one-off from before the merge.

A CPU-only lease written by Codex on pod `ic5zsemlme7moh` uses Python's `datetime.isoformat()`:

```json
"expires": "2026-10-05T22:21:35.875296+00:00"
```

`uor-pod status` on main then dies for every session:

```
jq: error (at .../leases/ic5zsemlme7moh/codex-cue-composition.json:15): date
"2026-10-05T22:21:35.875296+00:00" does not match format "%Y-%m-%dT%H:%M:%SZ"
```

The tool writes `...Z` itself (`now_iso`, `plus_hours`), so any session that writes a lease with Python — which is the documented workaround while the tool was unmerged, and which labs are still doing — takes the shared tool down for all three labs. The fix already in this PR accepts both forms and fails safe (unparseable ⇒ treated as LIVE, never as abandoned).

Since the base branch is now merged, I have retargeted this PR to `main`. Reviewable as-is: one file plus a regression test, 64 passing.

### comment 6003516650 — 2026-10-05T21:39:10Z — Casey-allard

## Exact-head self-review — `0d6a7de1` (rebased onto `main` `e445ee94`)

Owner direction permits self-review with a check list.

**Why the base changed.** #1758 merged as a squash, so this branch's original base was no longer an ancestor of main and GitHub reported CONFLICTING. Rebased properly: the branch is now `origin/main` + exactly one commit, and `git diff --name-only` vs main lists only `scripts/pod/uor-pod` and `scripts/pod/tests/uor-pod-dryrun.sh`. No unrelated change.

**Correctness.** The four sites that parsed with a bare `fromdateiso8601` now normalise first (strip fractional seconds, map `+00:00` to `Z`). Canonical `…Z` input parses exactly as before, so no existing behaviour changes. `catch 9999999999` fails **safe**: an unreadable timestamp is treated as LIVE, never as abandoned, because the dangerous direction is handing a GPU another session is using to a third party.

**Checks executed.**

| check | result |
|---|---|
| `bash -n scripts/pod/uor-pod` | clean |
| `scripts/pod/tests/uor-pod-dryrun.sh` | **64 passed, 0 failed** (61 before); no real mutation |
| new test: tool survives a Python-format lease | ok — fails with rc≠0 before the fix |
| new test: lease is listed | ok |
| new test: its GPU is still refused to another session | ok |
| diff scope | 2 files, no unrelated reformatting |

**Evidence the bug is real and post-merge.** On main, `uor-pod status` exits non-zero on Codex's `leases/ic5zsemlme7moh/codex-cue-composition.json` (`"2026-10-05T22:21:35.875296+00:00"`). That lease is CPU-only (`gpus: []`) — it holds no GPU and still takes the tool down for all three labs.

**Not fixed here.** `up_attempt` selects the first datacenter whose stock table advertises the GPU and `cmd_up` makes a single create attempt; a `no longer any instances available` error is fatal instead of falling through to the next DC, and `--wait` does not engage on it. I hit this three times (EUR-NO-1, EU-RO-1, EUR-IS-1) before a `--count 1` create succeeded. Follow-up, offered separately.


---
_Comment/review ids in this file: 6003498692, 6003516650_

