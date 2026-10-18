# NOW — what UOR-R4 is doing, on one page

**Updated 2026-10-18 (post-D39).** This page is the current plan. When it disagrees
with a longer document, this page is right; fix the longer document. History and the
long-form plan: `project-track.md`. Decisions: `DECISIONS.md` (D0–D39).

## Objective

One native Rust autoregressive geometric language model with exact addressed memory,
useful for conversation, memory, coding and reasoning, served integer-only (D11) on a
consumer laptop. **Pre-alpha: 0 of 8 milestones complete; no model holds a useful
conversation yet.** Milestones M1–M8 are GitHub milestones on issues #2029–#2036;
#2028 is the roadmap tracker.

## Active line — M1 (#2029)

K16-A vs K16-R admission redirect on the `rows_v1` counterfactual path
(sheet `docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`, FIT
`6199263447`, seeds {1,2}, V4096/B4/C192/K16, `ReadSpec::Route`,
`CanonicalVectorThenReadV1`, 16,404 batches). M1 closing bar: D39 item 1.

## Critical path, in order

| # | Step | Owner | State (2026-10-18) |
|---|---|---|---|
| 1 | One-session qualification at `21224b446`+: 25/25 controls + `admit_ptx` PASS, C-R1..C-R6 PASS, G-Pre PASS (4 cells), two checkpoint-0 A/R 100+1 timing records, sealed manifests | Codex | **running** |
| 2 | Consumer verifier repair (Codex findings against `af1dec4e`) | Antigravity | in progress |
| 3 | Packet acceptance (sole seat, owner `6251237195`) | Claude | pending |
| 4 | Four `rows_v1` configs + measured D-Packet | Codex | pending |
| 5 | Training dispatch — needs PM dispatch + owner release (`6201365250` chain) | Kimi PM + owner | held |
| 6 | W2 read-code capacity capture (already FIT'd, same lease) | Codex | queued |
| 7 | W3 BVG-K discrete batched-coordinate law (registered before any float sheet) | TBD | queued |
| 8 | W4 one float-credit sheet, only if W2 = H_law and W3 fails | TBD | conditional |

## Current instruments and gates

- **G-Cap (D39 item 2):** a run whose DEV-row alias ceiling cannot reach Bar (i) is
  `INSTRUMENT_UNREACHABLE`, not a negative. W0 census (2026-10-18): Bar (i) is
  reachable; the binding constraint is the admitted-cell ceiling, which this redirect
  targets. Coverage metrics (distinct vectors, alias groups, tie mass) are required
  in every qualification packet.
- **Interim M1 headline (D39 item 6):** injective codebook + Q4 admission ≥ 4,894.
  The 116/232 reply bar is BLOCKED under R14 and is not a closing condition.
- **Export-gap rule (D39 item 5):** a float candidate is disqualified when its export
  gap exceeds the two-seed band on the same evaluator, fixed per sheet at FIT time.

## Holds and hard rules

Training, M2 and Stage B are HELD; Q4 timing cells are waived (owner `6251027125`).
No softmax/softplus/sigmoid/tanh/GELU anywhere (D23/D25); no matmul/GEMM anywhere
(D25/D30); model compute is GPU-only (D24); banned mechanisms may not exist in any
compiled module (D29); Rust-first (D27); no transformer comparators (D28). D37:
float is legal in **offline training only**, in a separate scanned module; serving
stays D11 integer with the export boundary. D38: D5 is split into D5-a residency and
D5-b sparsity. External compute only via `scripts/pod/uor-pod` (≤ 4 pods, ≤ $8/h).

## How labs work now (D39 item 7)

Pick work from this page or the milestone queues, post a one-line intent note to the
owning issue, and keep going unless the PM (Kimi) redirects. No sitting dormant
waiting for handoff. Heartbeat every 5 minutes (`uor-harness heartbeat -m m1`).

## Glossary (retired as load-bearing terms)

| Term | Means | Status |
|---|---|---|
| Paired Relation/frozen-zero dose (sheet `m1-learned-attention-link-2026-10-17`) | the FIT'd integer sheet | run owner-stopped at 904/1,065 batches |
| `rows_v1` successor ("Dose 2") | `20d158ae`, packet `01833fa7` | stopped at Q1: configured admission collapse, both arms (2,034 → 225/205); instrument finding, not a negative |
| K16-A / K16-R redirect | `m1-discrete-rung0-admission-redirect-2026-10-15` | **the live line** |
