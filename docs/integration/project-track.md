# UOR-R4 Geometric Language Model — long-form project track

**The one plan is [`PLAN.md`](../../PLAN.md).** Read that first for current work. This page is the
long-form record of the milestone definitions and current state behind it. Where they disagree,
`PLAN.md` and the live milestone issues are right, and this page is the thing to fix.

Older K16/A8 routes and the pre-charter implementation sequence are historical, not the active plan.
Their 18–19 October plan and negative evidence remains in [`current-state.md`](current-state.md)
and dated lab records, especially [K16's outcome](../labs/k16-outcome-call-2026-10-19/README.md)
and the [A8 dose](../labs/a8-dose-2026-10-19/README.md). The 18 October archive snapshot alone is
not presented as containing those later paragraphs.

## Objective

One native Rust autoregressive geometric language model with exact addressed memory, useful for
conversation, memory, coding and reasoning, served integer-only (D11) on a consumer laptop.
**Pre-alpha: 1 of 8 milestones is complete (M1, 30 Oct 2026); no model holds a useful conversation.**
M1–M8 are issues [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)–[#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036);
#2028 is the roadmap tracker.

## Current state: M1 closed, generation open

The saved discrete `trunk-joint-pointer-v1-d11` model passed all six bars at both seeds. Its
composition is Flock `FrozenZeroV1`, a discrete Hamilton gate, discrete quaternion recurrence,
`PositionHolonomyV1`, `OrbitRootSnapNormV1`, a four-block orbit-root causal pointer and post-rotation outer-L1 readout.

| Bar | Evidence at two seeds |
|---|---|
| (a) self-contained D11 | Two `UORLUT01` artifacts load in the clean-room reader |
| (b) DEV export/reference and fault control | 14,287 of 14,287 agree; B4 passes |
| (c) tokens per second | close-grade Apple M1 decode 11,026 / 10,573 |
| (d) DEV integer top-1 | 7,002 / 7,001 against unigram 1,226 and T2 6,300, above the 6,324 target (T2 + 24) |
| (e) Fresh V2 transfer | 7,215 / 7,228 against unigram 1,279 and T2 6,582, band 82 |
| (f) held-out phrasing | p-f 7,850 / 7,872 against unigram 1,409 and T2 7,375, band 81; separate Apple M1 remeasurement 10,876 / 10,566 tok/s |

The older close grade covers (a)–(e); claude-211's panel score supplies (f), and #2029 is closed.
The limit is explicit: 62.5% of p-f's reference replies are verbatim TRAIN replies. M1 reads new
prompt phrasings into known reply kinds, not new reply kinds or useful conversation
([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).

## Active line: new-phrasing reply opening and a reachable memory key

The separate float-trained FBS trunk is the current complete-reply research line. It is not the M1
artifact and does not share its table, trunk or served session.

Claude-217's cycle and echo steps end every registered loop, but every content decision is
underpowered and greedy remains the decode. On p-s:

- Teacher-forced reply openings are 17.4%, 13.7% and 18.6% at three seeds, against 80.8%, 80.2% and 79.9% on later
  tokens;
- exact-reference replies are only 3, 1 and 4 of 256, against 38–40 on DEV;
- exact turn memory keys on exact first token ids and adds only 14–16 of 1,400 openings on p-s,
  against 322–337 of 1,226 on DEV.

Claude-218 found trunk-state memory keys that improved reply openings offline.
[Claude-219](../loop/tasks/one-model-end-to-end/claude-219/REPORT.md) then served the point key
for the first reply token in free generation on fresh p-v: templates improve by +13/+8/+12 and
exact replies by +9/+8/+12 at three seeds. It reaches only 12–18% of the forced-opening lift; the
sign key is inconclusive at this panel size. [Claude-220](../loop/tasks/one-model-end-to-end/claude-220/REPORT.md) then served the point key inside one float model on fresh p-w: templates improve by +41/+40/+42 at three seeds with exact claude-219 parity, while recalling TRAIN replies rather than generalizing. [Claude-222](../loop/tasks/point-key-integer/claude-222/REPORT.md) has built the integer point table, search and `/8` record, with 380 library tests and a full serving audit. [Claude-224](../loop/tasks/point-key-integer/claude-224/REPORT.md) has built the panel decode and grade. [Claude-227](../loop/tasks/integer-retention-rates/claude-227/REPORT.md) passed retention GPU parity within the 3% bar; [Claude-228](../loop/tasks/point-key-integer/claude-228/REPORT.md) built the run's reference/read-back/BOS checks; [Claude-229](../loop/tasks/point-key-integer/claude-229/REPORT.md) found an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so no bar result or throughput is reported. [Claude-231](../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun. Useful chat remains unproven. Then every required part
must run in integer serving and one joined model must be scored on fresh prompts. See
[PLAN.md](../../PLAN.md) and the [pipeline map](../model/pipeline-map.md).

## Milestones M1–M8 — definitions of done

### M1 `#2029` — language base: complete

**Original D39 acceptance text, preserved at item start rather than rewritten:** one saved discrete rung-0 artifact passes:

- **(a)** `G-Pre` admission ceiling ≥ **4,894**;
- **(b)** `G-Adm` on **both seeds**;
- **(c)** `G-Occ` with **> 16 occupied cells and an actual R > 0 row**;
- **(d)** **≤ 0.90 BPB** on the chat held-out stream **through the D11 export**, with the export gap reported;
- **(e)** a **fresh, pre-registered transfer panel**.

The 116/232 reply bar is **BLOCKED under R14** (no instrument) and is **not a closing condition**. **Interim headline (D39 item 6):** injective codebook + Q4 admission ≥ 4,894 — a legibility instrument so D21 progress control can fire again, not a language claim.

**Dated close outcome:** D83 and [decision log 6515151261](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6515151261) record the pre-existing D68 whole-project lab decision closing M1 on 30 October 2026. The original acceptance text is retained as history; this PR writes no new acceptance bar.

### M2 `#2030` — grounded reply from exact memory: in progress

**Original M2 acceptance text, preserved at item start:** ≥ **256 / 512** complete correct replies on the `m2-reply512` development panel **and** ≥ **40%** on a genuinely fresh held-out panel (D22), using **one** unified geometric model path — no softmax/sigmoid/tanh (D23/D25), no matmul (D25), GPU-only model compute (D24); learned relation/action/span extraction and emission through the shared geometric carrier, with no hand-coded phrase tables or external teachers (D28); and a clean export to D11 integer serving with the export gap reported (D37). The old 437/512 seen-row score followed by 0/128 on fresh data is a panel-fitting lesson, not qualified current progress.

### M3 `#2031` — durable conversation memory: in progress

**Original M3 acceptance text, preserved at item start:** on the selected M1/M2 artifact, `m3-memory` passes **provenance**, **contradiction and correction**, **abstention** on unstated or retracted facts, **explicit reset** and **process restart**; saved and restored sessions continue identically token-for-token (`IntegerStackSession` + `UR4SESS1`); and non-memory conversational prose does not silently overwrite stored facts (`#1718`). The on-model run remains open; host controls and the true-address 80/80 oracle are not learned address formation or M3 completion.

### M4 `#2032` — one served model through D11 and a CLI: not complete; accepted model path not started

**Original M4 acceptance text, preserved at item start:** both the native binary (`uor-chat` / `uor-r4-stack`) and the WASM serving binary pass the D11 static call-graph audits with **0** float, **0** softmax/squashing, **0** matmul and **0** hardware `mul`/`div`; the chosen M1/M2 model exports to `UORLUT01` and reports its training-to-D11 export gap, meets the **≤ 0.90 BPB** held-out bar plus the M1/M2/M3 reply and memory bars through `uor-chat`; and saving a live session then restoring it in a fresh process or WASM worker produces bit-identical continuation tokens. D80/D81 later allow and count integer multiply/divide. `PRLL` is built and registered, but GPU held-out scoring and the accepted full path remain open; engine audits alone do not close M4.

### M5 `#2033` — laptop cost (D5): not started

**Original M5 acceptance text, preserved at item start:** full-path latency (`ms/tok`), peak RSS (`MB`), energy (`mJ/tok`) and `weights_per_token` / total parameter ratio measured and reproducible on the accepted M4 `D11` artifact across ingestion, routing, memory lookup and token emission; D5-a residency and D5-b selected access reported as the separate halves D38 made them; no hidden host/CPU float fallback (D24) and no uncounted daemon or service overhead.

### M6 `#2034` — reasoning and coding on the same model: not started

**Original M6 acceptance text, preserved at item start:** the served M4 model passes the frozen `m6-reasoning` panel of unseen multi-step compositions, altered constraints and executable coding tasks checked by deterministic compile/test oracles (D28); and a causal intermediate-state ablation shows the result depends on the model's geometric state transport and addressed memory rather than on memorized templates.

### M7 `#2035` — distribution: runtime delivered, accepted model open

**Original M7 acceptance text, preserved at item start:** the qualified M4 `UORLUT01` artifact produces bit-identical token completions and session snapshots across the Rust library, the `uor-chat` CLI and the browser `serving-wasm` worker; the compiled `.wasm` passes `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7, `mul=div=float=0`); and Pages Studio runs entirely client-side in a Web Worker with truthful capability reporting and zero remote model calls.

Current behavior is local by default, with only an explicit user-configured web API sending prompts to the chosen endpoint; current audits allow and count integer multiply/divide under D80/D81. Those are status/evidence updates, not a new acceptance condition. Existing synthetic-stand-in parity is an instrument result, not M7 completion.

### M8 `#2036` — alpha release: not started

**Original M8 acceptance text, preserved at item start:** one versioned, checksum-pinned `UORLUT01` artifact meets the frozen acceptance bars of **M1–M7**; reproducible installation, release notes binding the exact measured scores and known limitations, and independent verification on consumer hardware. M1 alone is not an alpha release.

## Rules that bind every row

- **Geometry only (§3.1–§3.3).** No softmax, squashing or normalized exponential, and no dense
  matrix multiplication, in training, evaluation, instruments, serving or tooling.
- **Research compute is GPU-first (§3.5).** GPU results qualify training and evaluation. The
  owner-authorized D84 CPU program runs development research only and cannot replace a GPU bar
  without a later owner decision. Client-device serving of the D11 artifact is the product and is
  allowed (§3.6).
- **One persistent native state (D85).** Train and serve the same integer/fixed-point geometric
  state; float is historical/reference or diagnostic work outside the default path. The historical
  export-gap route remains evidence, not the default model boundary.
- **D11 serving.** No float or transcendental arithmetic; D80/D81 allow integer multiply and divide.
- **D22/D31.** A negative closes a configuration, never a mechanism. Instrument defects are not
  mechanism negatives.
- **D67.** GPU/training runs go through the lab's own whole-project pre-run review; D41's donor pass
  is the named owner-at-dispatch exception. There is no blanket owner hold.
- **Delivery.** Named paths, protected PR, exact-head review and checks, merge queue, then
  verification of the actual merge on fresh `main`. Open PRs are pending, not merged.
- **Original D39 item 5, preserved:** a float candidate is disqualified when its export gap exceeds the two-seed noise band on the same evaluator; the band is fixed per sheet at FIT time. Float scores are instrument readings, never decision metrics.
- **Original D21 progress control, restored by D39 item 6 and preserved:** after three consecutive merged PRs on one line that do not move the owning milestone's headline, the next action is a pivot card on the milestone issue. M1's interim headline exists so this rule can fire again.

**Original binding rule text, preserved at item start as history:**
- **D23 / D25 / D30.** No softmax, softplus, sigmoid, tanh or GELU anywhere, including renamed and table forms, in objectives, gradients, sampling, teachers or comparators; no dense matrix multiplication or GEMM anywhere, including wrapper laundering.
- **D24.** Model compute runs on the GPU, with no host fallback.
- **D37.** Float is legal in **offline training only**, in a separate scanned geometric module with its own allowlist entry added at pre-registration; the served artifact is D11 integer, and every float-trained candidate reports its export gap.
- **D11.** Served computation has no floating point and no transcendental arithmetic. Since D80 and D81 (28 Oct) it may use the hardware integer multiply and divide.
- **D22 / D31.** A negative closes a configuration, never a mechanism. A tooling or numerical failure is a blocker to fix, not a negative, and an unreachable instrument is `INSTRUMENT_UNREACHABLE`, not a mechanism result.
- **D39 item 7.** Labs pick work from `NOW.md` and the milestone queues, post a one-line intent note on the owning issue, and continue unless the PM redirects; every gate above is unchanged.
- **Delivery.** Named paths, protected PR, exact-head checks/review, and verification of the actual merge commit on a fresh `origin/main`. An open PR is pending, not merged.

## Superseded sections — original anchors preserved

These headings remain as pointers so old inbound links resolve to history rather than a current plan.

## Post-dose optimization path

Original 19 October section, preserved at its recorded source:
[project-track at ec9a790fe](https://github.com/UOR-Foundation/uor-r4/blob/ec9a790fe5be73cd673f082eba86ced551b22d12/docs/integration/project-track.md#post-dose-optimization-path). The separate [18 October archive snapshot](../../archive/history/project-track-archive-2026-10-18.md#post-dose-optimization-path--owner-direction-october-18) contains the earlier text, not the later 19 October additions.

## Research dependencies and exit conditions

Archived 2026-10-18. Text:
[Research dependencies and exit conditions](../../archive/history/project-track-archive-2026-10-18.md#research-dependencies-and-exit-conditions).

## Historical implementation sequence — owner adopted September 12

Archived 2026-10-18. Text:
[Historical implementation sequence — owner adopted September 12](../../archive/history/project-track-archive-2026-10-18.md#historical-implementation-sequence--owner-adopted-september-12).

## Structural memory and geometric representation follow-up

Archived 2026-10-18. Text:
[Structural memory and geometric representation follow-up](../../archive/history/project-track-archive-2026-10-18.md#structural-memory-and-geometric-representation-follow-up).
