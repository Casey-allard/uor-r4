# #1629 — Add the dose launcher, the free-machine waiter, and its logic test

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:42:36Z
- **Updated:** 2026-10-02T17:43:56Z
- **Closed:** 2026-10-02T17:43:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1629
- **Merged:** 2026-10-02T17:43:55Z
- **Merge commit:** e7961fab39acf33958e64c3b0fc78ba112a97127
- **Base / Head:** main / UOR-Foundation:codex/post-epoch-eval-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

Three scripts that make an unattended, sequenced training run **auditable rather than faith-based**. They were developed outside the repo; keeping them only there meant the OS could clear them before the run they exist to start.

**`launch-fulldose.sh`** — the dose experiment. One chat-v0 LM phase **from the prose init** to a full pass over the store (**20,149 updates = 82.5M tokens, 11.5 tokens/parameter** against the previous arm's 1.28), with a dense eval cadence so the dose-response is readable, then one response phase at 8× the previous dose.

**Two earlier designs were wrong, and the reasons are recorded in the file so they are not repeated:**

1. **Staging the run from scratch costs the SUM.** Three from-scratch stages of 5k/10k/20,149 updates total **8.2–12.4 h**, not the 4.7–7.1 h of one full epoch — a 1.75× overshoot, against an objective whose point was to reduce time.
2. **From scratch also removes the prose init**, which confounds *"more dose"* with *"lost the base that produced the winning arm."* A from-scratch model scores ~ln(4096)=**8.32** at step 0; the prose base scores **5.739238081358119**. The launcher holds the init fixed, and I verified it loads by the step-0 dev NLL matching that value **exactly**.

**`wait-for-free-machine.sh`** — waits for **both** no other `geometric-stack` process and load below 4, then claims the slot noclobber and launches. Both conditions are required and the reasoning is in the file: a peer job that has not yet spun up its threads is still a job that will contend.

**Why waiting is worth it, measured:** a second concurrent trainer received **0.87 cores** and turned a **4.7 h** projection into **43.4 h**. Waiting is ~9× cheaper than contending.

Two robustness fixes over the version I was running: the launcher is **resolved beside the script** rather than hard-coded to `/tmp` (which the OS may clear), and the waiter **heartbeats** every ~30 min, because logging only state changes is indistinguishable from a hung monitor.

**`test-monitor-logic.sh`** — unit-tests the trigger **at both boundaries**, so a never-firing condition is caught now rather than after hours of waiting.

## Exact-head evidence

**Reviewed head:** `6453163f`. `bash -n` clean on all three.

Executed — the waiter's logic against synthetic states:

| state | result | expected |
|---|---|---|
| other=2, load=10.85 | wait | ✓ |
| other=2, **load=2.0** | wait | ✓ (another job, regardless of load) |
| other=0, load=12.0 | wait | ✓ |
| other=0, **load=1.5** | **FIRE** | ✓ |
| other=0, **load=3.99** | **FIRE** | ✓ |
| other=0, load=4.01 | wait | ✓ |

All six, both boundaries inclusive. The `load<4` bar is also **attainable**, not aspirational: the app harness's own floor is ~load 1.5.

The launcher's `init=` was verified to load — **step-0 dev NLL 5.739238081358119, byte-identical to the known value** — and its arguments match the wrapper that produced the winning arm.

## Scope

Orchestration only: **no model behaviour changes, no capability claim.** These run an experiment; the result will be reported separately and labelled **geometry-on-principle, not a demonstrated advantage** (the matched measurement is parity: 1.9981 geometric vs 2.0111 transformer, Δ −0.0130, inside the ±0.021 seed spread).

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

