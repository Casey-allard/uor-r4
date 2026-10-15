# #1765 — uor-pod: robust lease time parsing (fail-safe), preflight helpers, DC fallback, bounded build-lock wait (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T22:13:14Z
- **Updated:** 2026-10-09T13:09:07Z
- **Closed:** 2026-10-05T22:15:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1765
- **Merged:** 2026-10-05T22:15:04Z
- **Merge commit:** 258e1c66984ac063aa082280fd232fa7e4be39fc
- **Base / Head:** main / UOR-Foundation:claude/uor-pod-fixes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Follows #1758 (the tool) and #1761 (hand-written timestamps no longer abort it).

Fixes four defects in `scripts/pod/uor-pod` found in real use on 5 October.

## 1. Lease timestamps (critical)

A lease written by another lab had `"expires": "2026-10-05T22:21:35.875296+00:00"`. #1761 kept that file from aborting the tool, but only by reading every fractional+offset time as unparseable. So an **expired** lease in that form stayed live forever (never taken over, never reaped), and non-UTC offsets were not applied.

- `uor-pod` (jq `ts`/`norm_lease`) and the pod-side reaper (Python) now parse any RFC 3339 / ISO 8601 time with an explicit offset: fractional seconds, `Z`/`z`, `+HH:MM`, `+HHMM`, `+HH`, `T` or a space. They also accept an epoch number.
- **Fail safe:** some lease files are treated as **live**:
  - an unparseable `expires`, including a time with no offset, whose zone is ambiguous;
  - a `gpus` field that is not a list of indices;
  - a file that is not JSON.

  Such a lease holds the GPUs it names, or all of them when they are unknown. It is never read as expired. `status` prints a `MALFORMED LEASE FILE` block naming each file and its reason, marks the lease line `[MALFORMED: treated as LIVE]`, and repeats the warning on stderr.
- `uor-pod` writes only `YYYY-MM-DDTHH:MM:SSZ`. `renew` rewrites a hand-written `started` value in that form.
- The free-GPU count in `up` is guarded against a missing `gpuCount` or a failed lease read. Before #1761, a failed lease read produced the "integer expression expected" error and the free-GPU check was skipped.

## 2. Preflight of helper files

Before listing stock or creating anything, `up` checks every file it will upload:
- `uor-pod-bootstrap.sh` and `uor-reaper.sh`, unless `--no-bootstrap`;
- `hot-set.txt`.

Each must be present, readable and non-empty, and `bash -n` must pass on the scripts. `bootstrap` and `seed` repeat the check in a subshell, so a failure after creation goes through `up_failed` (the pod is deleted) rather than exiting with a pod left behind.

## 3. Placement fallback on create-time stock errors

Stock can change between the listing and `pod create`. When the create in one datacenter fails with Runpod's out-of-stock error ("no longer any instances available", plus a few equivalent phrasings), `up` tries the next stocked datacenter in policy order: EUR-NO-1 → EU-RO-1 → EUR-IS-1. Only after all of them refuse does it report no stock or enter the `--wait` retries; each retry makes a full pass again. Any other create error stops at once. The create's stdout is now parsed on its own, so a warning on stderr cannot hide a new pod id.

## 4. Bounded build-lock wait in the bootstrap

A pod no longer waits up to 45 minutes for another pod's build of the same commit.
- The lock holder refreshes `…lock.holder` every 20 s with its phase and target file count.
- A waiter gives up after at most **3 minutes** (`UOR_BUILD_WAIT_S`). It stops sooner if the heartbeat is more than 90 s old or shows no progress for 90 s, and it takes the lock if the lock comes free.
- It then builds on its own container disk into a private staging directory and publishes with a single `rename(2)` onto `bin/<sha>-sm<cap>/`. The rename never merges into an existing directory: the first parity-PASS build wins and a later one is discarded.
- A cached build without parity PASS is moved aside (`…failed-<UTC>-<pod>`) only while holding a short publish lock. Without that lock the new build stays private (`…private-<pod>-<UTC>`), and the cache is never overwritten.
- The heartbeat loop closes the lock descriptor and is stopped by an EXIT trap if the build fails.

`docs/labs/compute.md` describes all four changes.

## Verification (at this head)

- `scripts/pod/tests/uor-pod-dryrun.sh`: **123 passed, 0 failed** (was 64). The 59 new checks cover:
  - timestamps: expired and live fractional `+00:00`, `-05:00` and `+05:30` offsets applied, epoch numbers, garbage, an offset-less time, a non-JSON file, the status warnings, canonical writes and `renew` rewriting, and the reaper's parser;
  - preflight: missing, empty and syntax-broken helpers, each refused before any API call, and `--no-bootstrap`;
  - DC fallback: EUR-NO-1 refuses and the pod lands in EU-RO-1 on its volume; all three refuse; `--wait` makes repeated passes; a non-stock error stops at once;
  - build lock: a stalled holder, a progressing holder (wait bounded), the cache filling during the wait, a stale heartbeat, both pods finishing (first PASS wins, cache untouched), a failed build moved aside, and the publish lock unavailable (build kept private).
- The same suite run against main's `uor-pod`: 28 of the new checks fail. So they detect the bugs; they do not just pass.
- The placement-fallback checks run **non-dry against the fake `runpodctl` only**. The build-lock checks source the bootstrap's cache functions (`UOR_BOOTSTRAP_LIB=1`) with a stub build and a fake `flock`. No real pod, volume or GitHub call was made.
- `shellcheck` is clean on `uor-pod`, `uor-pod-bootstrap.sh`, `uor-reaper.sh` and the test. `python3 scripts/check_claim_wording.py` passes.
- The reported timestamp and the two live lease files on this laptop parse to the expected epochs (read-only `jq` check; no state written).

Not exercised: a real `pod create` stock error, and a real two-pod race on the network volume. Only the logic is tested here, against fakes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6004204356 — 2026-10-05T22:14:11Z — Casey-allard

Exact-head check at cc01b043 (Claude lab): shellcheck clean on all four scripts; dry-run suite 123 passed / 0 failed (28 of the new checks fail against main, so they cover the bugs); check_claim_wording exit 0. Lease parsing now reads RFC 3339 with offsets/fractional seconds and treats anything unreadable as LIVE (fail-safe) with a MALFORMED warning; preflight before any API call; DC fallback only on the stock error; bounded build-lock wait with atomic publish. Approve at this head.


---
_Comment/review ids in this file: 6004204356_

