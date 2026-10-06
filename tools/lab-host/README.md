# Lab host tools

Host-local operational tooling for durable labs (D14). These are not part of the
Rust workspace and are not built by CI. They exist because two failure modes went
unnoticed for days while every lab reasoned past them.

## The problem these solve

**1. Labs expire silently.** `tools/lab-runner/src/coord.rs:14` sets `TTL = 1200`
— **twenty minutes** — and `:492` expires a lab whose heartbeat is stale. **No
MANUAL client can renew inside that window.** Every lab adapter records this in its
own words (`"MANUAL client; no heartbeat automation"`), so a manual lab goes dark
roughly twenty minutes after its session idles, with no signal to anyone.

Measured 2026-10-06: **four of five labs had expired heartbeats**, stale by 3.7 to
5.9 days, while every lab believed it was coordinating.

**2. Local policy drifts from the machine and from the repo, unobserved.** The
local `runner/host-policy.json` (mtime 2026-09-30) still declared two volumes
destroyed when a drive failed and the workflow moved to iCloud. The runner's
`admissions-held.json` was holding admission on a stale free-space measurement that
nothing re-evaluated. Both were findable only by reading three files that nothing
compared.

## `lab-heartbeat <session> [coord.git]`

Renews one lab's coordination heartbeat. Unique event id per beat, bounded log,
exit non-zero on failure.

Install it as a LaunchAgent so it survives session end. `StartInterval` must be
**well under `TTL` (1200s)** — 600s gives a 2x margin, so one missed run (sleep,
network blip) does not expire the lab. A template lives in this directory; replace
the session id and paths for your lab:

```
sed -e 's/opencode-deepseek-20260930/<your-session>/g' \
    -e "s|/Users/casey.allard|$HOME|g" \
    org.uor.lab-heartbeat.plist.template > ~/Library/LaunchAgents/org.uor.<lab>-heartbeat.plist
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/org.uor.<lab>-heartbeat.plist
```

Verify it actually fired rather than assuming: the script appends to
`runner/heartbeat-<session>.log`, `launchctl list | grep uor` shows the agent, and
`coord status` shows the sequence advance.

## `host-policy-check`

Read-only. Compares four things that nothing was comparing:

1. local policy age;
2. whether every volume the policy declares **actually exists** (catches dead hardware);
3. free space against each declared **reserve** (catches a hold that should be active);
4. the repo's `agent-execution-policy.json` storage watermarks against the local reserve;
5. whether the admission hold is **stale** — it re-evaluates the hold's own claim
   against current free space, because a hold is written once and never re-read;
6. **every lab's heartbeat against the TTL** — this is the check that found the
   expired labs.

Exits **1** on any drift or blocking condition, so it can gate a session start.

```
host-policy-check          # clean exit 0; drift exit 1
REPO=/path/to/worktree host-policy-check
```

## Known disagreements left for the owner or council

- **The repo policy declares a three-volume topology that no longer exists.** It
  declares watermarks for `internal`, `inner_workspace` (120/60/30 GiB) and
  `outer_ssd` (240/120/60 GiB); this machine has no `workspace` or `backing`
  volume, since that hardware was destroyed and the workflow moved to iCloud.
  Whether the *policy* should drop those watermarks, or retain them for machines
  that still have the hardware, is an owner/council call — `host-policy-check`
  reports it as drift rather than deciding.
- **Stale admission holds do not self-clear.** Whether a hold should be
  re-measured on read, or expire, is unresolved. The current behaviour is that a
  hold written once blocks admission until a human notices.
