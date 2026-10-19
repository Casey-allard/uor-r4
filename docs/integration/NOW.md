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
| 1 | One-session qualification at `21224b446`+: 25/25 controls + `admit_ptx` PASS, C-R1..C-R6 PASS, G-Pre PASS (4 cells), two checkpoint-0 A/R 100+1 timing records, sealed manifests | Codex | **complete; DATA and late operator full readbacks verified** |
| 2 | Consumer verifier source repair, helper `5c3bc562` / README `410fa4de` (`6254038861`) and off-host `FilePin` manifest binding `fa981782` / `fc5a765b` (`6256900961`); W2 v5.1 consumer `db02488e` / `2b6f61e2` / `c21f6e27` (`6256723035`, `6257268086`) | Antigravity | complete; source findings closed |
| 3 | Packet acceptance (sole seat, owner `6251237195`), sealed packet request `6254373730` | Claude | **ACCEPT `6254417234`** |
| 4 | Four `rows_v1` configs + measured D-Packet | Codex | accepted packet includes DOCUMENT_ONLY readiness; no machine capability |
| 5 | Four unchanged A/R×seed1/2 trajectories: owner's relayed gated release `6254435225` | Kimi PM + Claude | PM dispatch `6254457591` ISSUED_GATED_START; four-init census `6254480425` POSTED_PASS (raw/net1,986); census report custody verified; four owner/admission pairs created and ratified (D40); **K16-A/seed1 COMPLETED** 16,404/407, exit 0 at 20:58:35Z (report fa863063, final 6936cf99); **K16-R-seed1 COMPLETED** 16,404/130, exit 0 at 22:02:38Z (report 2c0ccad5, eval bb6885e8; DEV 14,287 rows / 12,089 admitted / 101 top1 / 515 top8), COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION; **A-seed2 COMPLETED** 16,404/410, exit 0 at 22:39:16Z (report 3d4d17fe, final eval d7976a0b; DEV 14,287/3,167/463/1,295), COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION; fresh UNAVAILABLE_NOT_RUN; **R-seed2 COMPLETED** 16,404/129, exit 0 at 23:16:56Z (report 6ed5d05d, final eval 0dfd6fa5); registered paired interpretation pending |
| 6 | W2 read-code capacity capture (already FIT'd, same lease) | Codex | queued |
| 7 | W3 BVG-K discrete batched-coordinate law (registered before any float sheet) | claude | Stage C1 **qualified** 2026-10-18: K-coordinate scoring exact (5,130/5,130), sabotage FAILs as required, K=16 scores 16 coordinates per launch at 16.25× throughput. Stage C2 (block apply) next; Stage T held for the K16 outcome |
| 8 | W4 one float-credit sheet, only if W2 = H_law and W3 fails | TBD | conditional |

The frozen qualification packet retains source `21224b446`; new injective-lineage
work does not replace its source or receipts. The same lease is retained; normal
close is not claimed. Claude's four-init census `6254480425` is POSTED_PASS:
raw/net1,986 on each role >1,941,16 vectors,16 alias groups×256, tie mass4,096
tokens/12,301 DEV rows; cell metrics UNAVAILABLE. The scientific start gate is
cleared. Report SHA `19c053e9779d6ab1dd7b02a5403b6c10910131c5829b51dd7ca361d512f050cf`;
the3,119-byte report is retrieved/hash-verified. Durable object
`claude-k16-cp0-coverage-census-20261018.tar`:16,384bytes, MD5
`6687ae6ad16d9a1b56836ec414a57467` (root native fetch83459/exit0). No further
census wait remains. Native transport produced four owner/admission pairs, ratified by the owner (D40); K16-A/seed1 completed 16,404 batches / 407 accepted (exit 0 at 20:58:35Z, report fa863063, final 6936cf99); R-seed1 completed 16,404/130 (exit 0 22:02:38Z, report 2c0ccad5, eval bb6885e8; DEV 14,287/12,089/101/515), COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION; A-seed2 completed 16,404/410 (exit 0 22:39:16Z, report 3d4d17fe, final eval d7976a0b; DEV 14,287/3,167/463/1,295), COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION; fresh UNAVAILABLE_NOT_RUN; R-seed2 COMPLETED 16,404/129, exit 0 at 23:16:56Z (report 6ed5d05d, final eval 0dfd6fa5). Registered paired interpretation pending; no result claimed.
The gated release covers no W2/W3/W4 work or changed arms, bars or fresh seal.

Final four-role execution and custody are recorded in the [canonical result](../labs/k16-four-role-execution-2026-10-18/README.md): all four native jobs exited 0, and three final DATA shards plus two late operator payload objects and metadata-last passed full cloud readback. Claude's registered two-seed/Amendment7 decision and conditional fresh disposition remain pending; fresh is `UNAVAILABLE_NOT_RUN`. Normal close and final resource accounting are not yet recorded. These archives are additional to the prior qualification custody.

**Registered outcome call (Claude, 2026-10-19 00:58Z, #2029 6257335075; [record](../labs/k16-outcome-call-2026-10-19/README.md)):** `OCCUPANCY_COLLAPSE_TRIGGERED`. R triggers G-Occ on both seeds (19–20 occupied cells), K16-A fails G-Adm on both seeds (2,237 and 3,167), and every role fails Bars (i) and (ii). There is no KEEP, and fresh transfer is NOT_TRIGGERED (seal kept). Admission-arm selection: A. **Follow-up:** the injective pre-flight stopped at B3_init 127/3,847 < 500 and net G-Cap 127 < 500, so the run was not dispatched — `INSTRUMENT_UNREACHABLE`, not a mechanism negative; nothing trained. The owner selected the **address-structured injective-cell** successor as a **placement-only 256×16 pre-flight DRAFT** (`docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`); the 32-cell compact form is parked as a fallback. Source items (a/b/c), registration, a costed decision, and a typed D40 owner release at dispatch remain before any run; no compute ([pre-flight record](../labs/injective-preflight-and-c2-qualify-2026-10-19/README.md)).

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

Training outside an explicitly released scope remains HELD, including M2 and
Stage B. The completed K16 Route writer-objective experiment used its own scoped
release; its [registered outcome](../labs/k16-outcome-call-2026-10-19/README.md)
is `OCCUPANCY_COLLAPSE_TRIGGERED`, with no KEEP and fresh `NOT_TRIGGERED` (sealed).
Each unreleased successor needs its own applicable authorization and gates;
completed releases and FIT reviews do not authorize future training.

**Separate D41 donor exception:** the owner-ratified one-time offline donor corpus (`distill-d1`) generation is not student training; it runs only after the driver math optimization, under the D41 decision and conditions, and releases no student training hold. Its execution gate (release/PM-dispatch wording) remains a separate owner/PM item.

Q4 timing cells are waived (owner `6251027125`).
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
