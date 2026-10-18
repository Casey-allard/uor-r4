# UOR-R4 Geometric Language Model — long-form plan

**The current plan, on one page, is [`NOW.md`](NOW.md).** Read that first. This page is
the long form behind it: the active line, each milestone's definition of done, and the
sequence that follows the active line. Where this page and `NOW.md` disagree, `NOW.md`
is right and this page is the thing to fix.

**Updated 2026-10-18 — D39 course-correction.** Until today this page carried **three
sections that each called themselves the active plan**, a D8 rung ladder written for a
substrate D32 froze, and a September 12 implementation sequence. All of it is preserved
verbatim in
[project-track-archive-2026-10-18](../history/project-track-archive-2026-10-18.md),
with five `Superseded` banners at the head of the sections they govern. Nothing was
deleted. Three headings that other documents link to by anchor are kept as stubs at the
bottom of this page, pointing into the archive.

## Objective

One native Rust autoregressive geometric language model with exact addressed memory,
useful for conversation, memory, coding and reasoning, served integer-only (D11) on a
consumer laptop. **Pre-alpha: 0 of 8 milestones complete; no model holds a useful
conversation yet.** Frontier capability on a laptop is the long-term research objective,
not an established feasibility or scaling result. Milestones M1–M8 are the GitHub
milestones on issues [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)–
[#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036);
[#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028) is the roadmap tracker.

## Active line — M1 `#2029`: the K16-A/K16-R admission redirect

The live work is a matched 2-arm × 2-seed contrast on the `rows_v1` counterfactual path:
**K16-A** control `8T+32A` against **K16-R** candidate `8T+32R`, at `V4096/B4/C192/K16`,
`ReadSpec::Route`, `CanonicalVectorThenReadV1`, 16,404 batches (1,049,856 targets),
fresh init, and a fresh transfer panel before any KEEP. Sheet:
[`m1-discrete-rung0-admission-redirect-2026-10-15`](../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md),
FIT `6199263447`, seeds {1, 2}. M1's closing bar is D39 item 1, below.

**Substrate.** D32 discrete (`crates/uor-r4-discrete`), GPU-only, integer. D37 re-admits
float **offline only**, in a separately scanned geometric module that reports its export
gap; softmax/squashing (D23/D25) and dense matrix multiplication (D25/D30) stay banned
everywhere; serving stays D11 integer (D37 item 3). D37 releases no hold, FIT or compute,
and every live M1 sheet is integer-only, so the float permission changes nothing on this
line today.

### Critical path, in order

| # | Step | Owner | State (2026-10-18) |
|---|---|---|---|
| 1 | Full one-session qualification at `21224b446`+: 25/25 finite controls + `admit_ptx`, C-R1..C-R6, G-Pre (checkpoint-0 `target_admitted_rows` ≥ 4,894, both seeds), active-R rows checks, two checkpoint-0 A/R 100+1 timing records, measured all-in budget, sealed manifests | Codex | **running** |
| 2 | Consumer-verifier corrections (two-cell timing amendment, rows-image 4-tuple, C-R5 constants, negative-witness identity, C-R6 four distinct roles, source/checkpoint identity) | Antigravity | in progress |
| 3 | Packet acceptance — sole seat, owner `6251237195` | Claude | pending |
| 4 | Regenerate four `rows_v1` configs + measured D-Packet | Codex | pending |
| 5 | **Training dispatch** — needs PM dispatch and an explicit owner release | Kimi PM + owner | held |
| 6 | W2 read-code capacity capture (already FIT'd, same lease) | Codex | queued |
| 7 | W3 BVG-K discrete batched-coordinate law (registered before any float sheet) | TBD | queued |
| 8 | W4 one float-credit sheet, only if W2 decides `H_law` and W3 fails | TBD | conditional |

**Outcome branches, pre-declared** in
[`m1-learned-attention-link-R-DRAFT-2026-10-18`](../harness/prep/m1-learned-attention-link-R-DRAFT-2026-10-18.md):
`K16-R` pass → register the Relation + R successor; `K16_R_NULL` with A keeping admission →
width-only successor; `ADMISSION_STILL_CAPPED_AT_K16` → full-vector address metric.

**Budget and holds.** Measured (Codex, 2026-10-18, [`#2029`](https://github.com/UOR-Foundation/uor-r4/issues/2029)):
the registered 100-sample timing units ran **26.022125399 s (A) / 26.023861995 s (R)**,
each containing carry plus both proposal/observation families, zero apply at checkpoint 0.
Applied to the registered 16,384-vector/20-route mix that is **8,528.79 s ≈ 2.37 GPU-hours
across four trajectories, proposal-only**; accepted apply, later occupancy, the 20 DEV
evaluations, conditional fresh, I/O and paid idle are explicitly unmeasured. The concrete
remaining proposal is **5–8 h on one 5090 ($4.54–$7.264 at $0.908/h)** with those allowances
unmeasured; no all-in measured training duration or empirical upper bound is claimed.
Pod quote $0.908/h on the retained `vast-56571294`. The earlier "1.2 h measured" phrase was
a rough projection, not a measurement — do not dispatch on it.
Training, M2 and Stage B are **HELD**; the two Q4 timing cells are waived by the owner
(`6251027125`); Q4 is `NOT_EXECUTABLE`. No step above releases a hold.

**Instrument gate (D39 item 2).** Every admission sheet carries a pre-run G-Cap gate: the
alias ceiling over DEV rows — rows whose target is the lowest-ID member of its alias group,
the only member that can win the evaluation tie-break — must exceed the sheet's Bar (i), net
of admission. Below it the run is `INSTRUMENT_UNREACHABLE` (D31), not a mechanism negative.
The W0 census ([alias-census report](../labs/course-correction-2026-10-18/alias-census-REPORT.md))
found Bar (i) reachable; the binding constraint is the admitted-cell ceiling, which this
redirect's ≥ 4,894 admission gate targets. Every qualification packet also reports coverage
metrics: distinct vectors, alias-group histogram and within-cell tie mass.

## Milestones M1–M8 — definitions of done

M1's bar is fixed by the owner in D39 item 1. M2–M8 quote the acceptance criteria already
frozen on their milestone issues; no bar here is new, and a change to any of them is an
owner decision, not a PM edit.

### M1 `#2029` — language base

M1 closes when **one saved discrete rung-0 artifact** passes:

- **(a)** `G-Pre` admission ceiling ≥ **4,894**;
- **(b)** `G-Adm` on **both seeds**;
- **(c)** `G-Occ` with **> 16 occupied cells and an actual R > 0 row**;
- **(d)** **≤ 0.90 BPB** on the chat held-out stream **through the D11 export**, with the
  export gap reported;
- **(e)** a **fresh, pre-registered transfer panel**.

The 116/232 reply bar is **BLOCKED under R14** (no instrument) and is **not a closing
condition**. **Interim headline (D39 item 6):** injective codebook + Q4 admission ≥ 4,894 —
a legibility instrument so D21 progress control can fire again, not a language claim.

### M2 `#2030` — grounded reply from exact memory

≥ **256 / 512** complete correct replies on the `m2-reply512` development panel **and**
≥ **40%** on a genuinely fresh held-out panel (D22), using **one** unified geometric model
path — no softmax/sigmoid/tanh (D23/D25), no matmul (D25), GPU-only model compute (D24);
learned relation/action/span extraction and emission through the shared geometric carrier,
with no hand-coded phrase tables or external teachers (D28); and a clean export to D11
integer serving with the export gap reported (D37).

### M3 `#2031` — durable conversation memory

On the selected M1/M2 artifact, `m3-memory` passes **provenance**, **contradiction and
correction**, **abstention** on unstated or retracted facts, **explicit reset** and
**process restart**; saved and restored sessions continue identically token-for-token
(`IntegerStackSession` + `UR4SESS1`); and non-memory conversational prose does not silently
overwrite stored facts (`#1718`).

### M4 `#2032` — one served model through D11 and a CLI

Both the native binary (`uor-chat` / `uor-r4-stack`) and the WASM serving binary pass the
D11 static call-graph audits with **0** float, **0** softmax/squashing, **0** matmul and
**0** hardware `mul`/`div`; the chosen M1/M2 model exports to `UORLUT01` and reports its
training-to-D11 export gap, meets the **≤ 0.90 BPB** held-out bar plus the M1/M2/M3 reply and
memory bars through `uor-chat`; and saving a live session then restoring it in a fresh
process or WASM worker produces bit-identical continuation tokens.

The audit criterion (1) is a **runtime** gate, green since #2655 (native, 44 symbol ranges)
and #2654 (WASM, 7/7). It is re-run on the shipped artifact; it is not a model bar and does
not close M4 on its own.

### M5 `#2033` — laptop cost (D5)

Full-path latency (`ms/tok`), peak RSS (`MB`), energy (`mJ/tok`) and `weights_per_token` /
total parameter ratio measured and reproducible on the accepted M4 `D11` artifact across
ingestion, routing, memory lookup and token emission; D5-a residency and D5-b selected
access reported as the separate halves D38 made them; no hidden host/CPU float fallback
(D24) and no uncounted daemon or service overhead.

### M6 `#2034` — reasoning and coding on the same model

The served M4 model passes the frozen `m6-reasoning` panel of unseen multi-step
compositions, altered constraints and executable coding tasks checked by deterministic
compile/test oracles (D28); and a causal intermediate-state ablation shows the result depends
on the model's geometric state transport and addressed memory rather than on memorized
templates.

### M7 `#2035` — distribution: capability API, WASM runtime, Pages Studio

The qualified M4 `UORLUT01` artifact produces bit-identical token completions and session
snapshots across the Rust library, the `uor-chat` CLI and the browser `serving-wasm` worker;
the compiled `.wasm` passes `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`
(7/7, `mul=div=float=0`); and Pages Studio runs entirely client-side in a Web Worker with
truthful capability reporting and zero remote model calls.

The .wasm audit criterion is already green at runtime (#2654, 7/7) and is re-run against the
shipped model bundle. What remains is the bundle, not the audit.

### M8 `#2036` — alpha release

One versioned, checksum-pinned `UORLUT01` artifact meets the frozen acceptance bars of
**M1–M7**; reproducible installation, release notes binding the exact measured scores and
known limitations, and independent verification on consumer hardware.

## After the K16 qualification — W1 to W4 (D39 item 4)

After the qualification concludes, in this order, each step needing its own
`TEST FITNESS: FIT` and an owner release wherever a hold applies:

1. **W1** — the K16-A/R run itself (proposal-only ≈ 2.37 GPU-h on the measured timing basis; concrete proposal 5–8 h on one 5090 with allowances unmeasured — see Budget and holds).
2. **W2** — the already-FIT'd read-code capacity capture, on the same lease.
3. **W3** — the BVG-K batched-coordinate discrete law; the matched discrete speed
   competitor, registered **before** any float sheet.
4. **W4** — a single float-credit sheet, only if W2 decides `H_law` **and** W3 fails.

D39 item 4 releases no hold.

## Rules that bind every row

- **D23 / D25 / D30.** No softmax, softplus, sigmoid, tanh or GELU anywhere, including
  renamed and table forms, in objectives, gradients, sampling, teachers or comparators; no
  dense matrix multiplication or GEMM anywhere, including wrapper laundering.
- **D24.** Model compute runs on the GPU, with no host fallback.
- **D37.** Float is legal in **offline training only**, in a separate scanned geometric
  module with its own allowlist entry added at pre-registration; the served artifact is D11
  integer, and every float-trained candidate reports its export gap.
- **D11.** Served computation has no floating point, no transcendental arithmetic and no
  multiplier instruction.
- **D39 item 5.** A float candidate is disqualified when its export gap exceeds the two-seed
  noise band on the same evaluator; the band is fixed per sheet at FIT time. Float scores are
  instrument readings, never decision metrics.
- **D22 / D31.** A negative closes a configuration, never a mechanism. A tooling or numerical
  failure is a blocker to fix, not a negative, and an unreachable instrument is
  `INSTRUMENT_UNREACHABLE`, not a mechanism result.
- **D21 progress control, restored by D39 item 6.** After three consecutive merged PRs on one
  line that do not move the owning milestone's headline, the next action is a pivot card on
  the milestone issue. M1's interim headline exists so this rule can fire again.
- **D39 item 7.** Labs pick work from `NOW.md` and the milestone queues, post a one-line
  intent note on the owning issue, and continue unless the PM redirects; every gate above is
  unchanged.
- **Delivery.** Named paths, protected PR, exact-head checks, and verification of the actual
  merge commit on a fresh `origin/main`. An open PR is pending, not merged.

## Superseded sections — original anchors preserved

These three headings are linked by anchor from other documents. Their bodies moved to
[project-track-archive-2026-10-18](../history/project-track-archive-2026-10-18.md) on
2026-10-18; the headings are kept here so the inbound links still resolve to a pointer
instead of a 404.

## Research dependencies and exit conditions

Archived 2026-10-18. Text:
[Research dependencies and exit conditions](../history/project-track-archive-2026-10-18.md#research-dependencies-and-exit-conditions).

## Historical implementation sequence — owner adopted September 12

Archived 2026-10-18. Text:
[Historical implementation sequence — owner adopted September 12](../history/project-track-archive-2026-10-18.md#historical-implementation-sequence--owner-adopted-september-12).

## Structural memory and geometric representation follow-up

Archived 2026-10-18. Text:
[Structural memory and geometric representation follow-up](../history/project-track-archive-2026-10-18.md#structural-memory-and-geometric-representation-follow-up).
