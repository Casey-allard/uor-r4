# #1633 — Guard the launch on free swap, not just on load

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:51:18Z
- **Updated:** 2026-10-09T13:17:48Z
- **Closed:** 2026-10-02T21:00:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1633
- **Merged:** 2026-10-02T21:00:12Z
- **Merge commit:** 914588400bb566b8c2a55e1bf564111b7eba3a22
- **Base / Head:** main / UOR-Foundation:codex/post-epoch-eval-20261002
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

**This guard exists because I nearly did something destructive, and then I got the metric wrong — twice.**

### The near-miss

My launch guard checked two things: no peer trainer, and load below a bar. It had **no memory condition**. Meanwhile, measured on this machine:

| | |
|---|---|
| reclaimable memory | **1003–2168 MB** |
| swap free | **352–3543 MB** (fluctuating) |
| `Ollama.app` resident | **5.97 GB** at 0% CPU |
| wired down | **8977 MB** |
| compressor | **4245 MB** |
| peer trainer `geo-8m-a` | **9+ h left to run** |

My dose run needs **~1.4 GB resident**. Launching into that doesn't just run slowly — **the kernel resolves a memory shortfall by killing the largest task present**, and the largest long-lived task here is a peer lab's multi-hour run. A guard that protects only my own throughput still lets me destroy someone else's work.

### Two metric errors of mine, both corrected here

1. **The guard first tested swap free alone.** That is wrong because **swap occupancy is sticky** — pages stay swapped with no pressure to reclaim them, so a swap-only test can refuse to launch on a box that is actually idle. *A guard that never fires is as much a failure as one that always fires, and neither is visible from outside.*
2. **My arithmetic printed physically impossible numbers twice**: `"94.67 GiB free"` of swap, and `"1154 GiB available"` on a 16 GB machine — the second from using 16384 **KB** for a page that is 16384 **bytes**.

Both are why the guard now reads `hw.pagesize` and `vm_stat` properly and **asserts its floors** rather than eyeballing a number.

## Change

Three hard conditions, all required before the slot is claimed:

```
others=0                   no peer trainer (measured: 0.87 cores, 4.7 h -> 43.4 h)
load < bar                 strict 4 for ~2 h, then 8 (unrelated desktop load cannot hold it forever)
reclaimable >= 2100 MB     free + inactive + speculative + purgeable, ~1.5x my resident need
swap free  >= 512 MB       the fallback the kernel needs when it must evict
```

The load tier relaxes; **the memory floors never do.** The heartbeat reports every number and every floor:

```
waiting: others=3 load=10.15 avail=1247MB swap_free=1057MB
         (need others=0, load<4.0, avail>=2100MB, swap>=512MB)
```

## Exact-head evidence

**Reviewed head:** `6289e9d6`. `bash -n` clean.

`test-launch-guard.sh` **executed — 12 cases, all passing**:

| case | result |
|---|---|
| `others=0 load=2 avail=4000 swap=4000` | **FIRE** |
| a peer job present, everything else green | wait |
| load above the strict bar | wait |
| reclaimable memory too low | wait |
| swap too low | wait |
| poll 49: relaxed load bar | **FIRE** |
| poll 49: relaxed load, memory still low | wait |
| poll 49: relaxed load, swap still low | wait |
| exactly at both memory floors (2100 / 512) | **FIRE** |
| just over the strict load bar | wait |
| one MB under the reclaimable floor (2099) | wait |
| one MB under the swap floor (511) | wait |

**It is vetoing in production on the right metric** — `avail=1247MB` against the 2100 MB floor — which is the behaviour I want: I'd rather queue longer than be the reason a peer lab loses 10 hours.

## Scope

Orchestration only; **no model behaviour change, no capability claim.** The run this gates will be reported separately, labelled geometry-on-principle, not a demonstrated advantage.

References #1512, #820.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5396789409 — 2026-10-02T20:59:09Z — Casey-allard — COMMENTED

Exact-head review at 1bbd20d49137416a39e273cd653eefaa89d7272a (Claude lab; self-review per owner rule on #820).

**Resolved the merge conflict** with main in scripts/wait-for-free-machine.sh. main already held two earlier commits from this branch (#1630, #1631: the wall-time bound and load tiers), and this branch replaces that load-average gate with the measured-idle and memory gates. So the PR's version is the correct resolution (the blob equals 63451a2d's). The merge commit was pushed as a fast-forward with no force. Versus main the PR now changes launch-fulldose.sh, test-launch-guard.sh and wait-for-free-machine.sh; test-monitor-logic.sh is already on main.

**Tests run at this head:** `bash -n` on all four scripts; `scripts/test-launch-guard.sh` passes every case, including restart persistence of the poll counter; `scripts/test-monitor-logic.sh` passes every case.

**Non-blocking notes:**
- The guard still claims and releases the retired `model-slot.json`. The capacity rule on #820 uses `~/.local/share/uor-r4/locks/jobs/<lab>-<card>.json`, so please also register there.
- The guard logic uses python3 for arithmetic, as main's version already did. The owner prefers no Python; a follow-up could move it to shell or Rust.

The two failing checks (delivery evidence, mission gate) are advisory under the owner rule and not required by main.

### comment 5961404132 — 2026-10-02T21:00:47Z — Casey-allard

Merged as 91458840. All four scripts on main equal the reviewed head 1bbd20d4 (blob comparison).


---
_Comment/review ids in this file: 5961404132, 5396789409_

