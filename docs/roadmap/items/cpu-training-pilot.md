---
id: cpu-training-pilot
title: Run and grade the bounded CPU training pilot
milestone: research
area: training
status: ready
open_to: lab
needs_gpu: false
depends_on: [cpu-discrete-training-path, cpu-shared-vector-train, cpu-quantization-exact-memory, cpu-whole-path-exact-updates]
notebook: docs/research/notebook/cpu-training.md
done_class: host
scope: [crates/uor-r4-cpu-train/**, Cargo.toml, Cargo.lock, docs/harness/prep/cpu-training-pilot*, docs/loop/tasks/cpu-training-pilot/**, docs/research/cpu-training-program.md, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-training-pilot.md]
---

## Why it matters

Component speed does not show whether CPU training can improve a model at useful scale. The workstation must be measured under a bounded, pre-registered pilot with the same evaluator as the GPU path.

## Readiness as of 3 November 2026

The bounded public internal-development pilot has executed and failed its registered ranking bar. The full item remains open because genuinely fresh complete-reply inputs and a configuration that clears a registered bar are still unavailable. Public 64→256 interval restart is now deterministic on both seeds.

| Prerequisite | Current state |
|---|---|
| Shared-vector forward and exact serving arithmetic | Supported for the recurrence-only/no-pointer FBS mode, including snapped and raw gate/transition combinations; synthetic and public trained-artifact CPU/serving parity controls pass. The CPU Hamming port now also executes copy key 2 with the integer session's ordered two-token query/key and reports the key in the audit manifest. See [deepseek-87](../../loop/tasks/cpu-training-pilot/deepseek-87/REPORT.md). |
| Exact served-forward contract and workload audit | Delivered for the same supported FBS mode. `cpu-served-forward-audit` binds the artifact, reports active mechanisms, compares every Q12 query/token/kept-cell decision, separates modelled work from measured CPU timing, and inventories float-only gradient treatments. Both public raw seeds pass a 64-position complete-decision control. A follow-up corrected the contract's Hamming-range narrative and added independent integer/CPU/float boundary tests; the kernels were already causal and correct. A composed control exercises Hamming stream/readout reads, opening and reply memory and point-key fallback with exact parity. [Deepseek-91](../../loop/tasks/cpu-training-pilot/deepseek-91/REPORT.md) now trains the writer, selects a Hamming margin by exact served score, derives opening/reply tables and a point row from examples, writes a 536,516-byte full-path artifact, reads it back and passes 704 Q12 coordinates plus 11 complete decisions with zero mismatches. [Deepseek-92](../../loop/tasks/cpu-training-pilot/deepseek-92/REPORT.md) adds full, memory-off, point-off, Hamming-off and stripped arms; every arm preserves exact 704-coordinate/11-decision/88-cell parity while removing its named records. [Deepseek-93](../../loop/tasks/cpu-training-pilot/deepseek-93/REPORT.md) adds a Q24 coordinate surrogate that trains opening and reply movement weights by reparsing the exported artifact and scoring complete served queries. Both finite arms lowered target distance with exact parity, but top-1 stayed unchanged. The initial trunk and training sample are finite; corpus-scale held-out comparison remains unavailable. |
| Candidate artifact gate | `scripts/audit_served_forward_gate.py` binds a `UORT` token slice and optional expected artifact hash, validates the complete deterministic audit, and atomically promotes only exact-passing artifacts. The exact CPU corpus driver now runs the same audit internally before final promotion, validates all workload and mismatch fields, persists `final.audit.json`, and reloads both outputs. Public seed 1 passed a zero-step 64-position smoke with zero mismatches over 16,384 Q12 coordinates. Studio trained-model staging now requires a locally validated receipt and publishes it beside the model; remote cards pin the artifact and receipt; the release-bundle packager requires adjacent receipts for every UORLUT. The synthetic demo and arbitrary public-URL slot remain outside the trained-release gate, and automatic workflow publication/receipt signing are still open. See [deepseek-86](../../loop/tasks/cpu-training-pilot/deepseek-86/REPORT.md) and [deepseek-94](../../loop/tasks/cpu-training-pilot/deepseek-94/REPORT.md). |
| Tied-table and lawful parameter-group trainer | Tied-token and BVG-K normalization/bias/decay/route groups delivered on finite fixtures. The exact CPU corpus driver binds rank-margin, coordinate-step, parameter-limit and coordinate-count settings into checkpoint identity. The two-seed 1/16/64-Q12 step comparison remained ranking-negative. The FIT-reviewed same-amplitude one-versus-four-coordinate pivot exercised 437/238 accepted four-coordinate blocks, reduced the margin objective on both seeds, and remained ranking-negative; seed 1 also lost admission and top-8. See [deepseek-9](../../loop/tasks/cpu-training-pilot/deepseek-9/REPORT.md) and [deepseek-82](../../loop/tasks/cpu-training-pilot/deepseek-82/REPORT.md). The repeated-key full-path memory comparison then trained eight coordinates with 16-thread candidate evaluation. Both seeds improved held-out target distance and top-1 over frozen and random controls with exact parity; the top-1 gains missed their three-sigma bars. See [deepseek-95](../../loop/tasks/cpu-training-pilot/deepseek-95/REPORT.md). A cached teacher-forced fast path then reproduced both runs exactly while parallelizing independent trace, evaluation and parity samples; walls fell to 8.774/8.397 s with zero artifact or reading changes. See [deepseek-96](../../loop/tasks/cpu-training-pilot/deepseek-96/REPORT.md). A final full-width gain trainer then accepted all eight updates on each seed and reduced held-out distance by 16,799,488/16,154,486 through the complete served query, but top-1 remained 291/243 and the registered ranking bar failed. See [deepseek-97](../../loop/tasks/cpu-training-pilot/deepseek-97/REPORT.md). |
| Counterfactual corpus comparison | A one-arm runner now applies the existing exact BVG-K counterfactual trainer to the public masked corpus and compares normalization, bias, decay and route groups with zero-update controls. On both seeds the fixed 128-step, four-coordinate, delta-one configurations had exact export but no replicated ranking gain: normalization lost admission on seed 1, decay accepted no update, bias stayed rank-neutral, and route sharply degraded the held-out panel. See [deepseek-11](../../loop/tasks/cpu-training-pilot/deepseek-11/REPORT.md). |
| Matched admission and exact checkpoint export control | Panel evaluator and gate delivered on finite fixtures. |
| Trained artifact plus internal query panel | Bound: public dual-c131 seeds 1 and 2 have exact CPU/serving parity; the strict corpus driver creates document-disjoint public development samples. The public 19-table M1 artifact now has a read-only CPU path through its trained Relation read and all 15 joint-pointer trunk tables: all 14,287 DEV rows pass with zero query, admitted-cell, top-8 or token mismatch and reproduce 9,012 admitted / 7,002 top-1 / 7,485 top-8. See [deepseek-90](../../loop/tasks/cpu-training-pilot/deepseek-90/REPORT.md). |
| Tested public 64→256 interval restart | Passed. A FIT-reviewed post-FIT control resumed hash-bound step-64 checkpoints in fresh processes to step 256 and produced byte-identical final artifact/checkpoint state on both public seeds. Other steps and crash consistency remain untested. |
| Sealed complete-reply and fresh-phrasing panels | Unavailable without credentials; `m2-fresh256` returns HTTP 401. A public 160-prompt panel supports only qualitative generation output, not correctness scoring. |
| Discrete reads/proposals | #3501 is done with finite parity and restart controls. |
| Discrete corpus driver and artifact writeback | A K=1 CPU driver now runs the public four-table artifacts over a strict masked corpus, preserves nonzero source epochs, emits reloadable artifacts and resumes byte-identically. A separate read-only CPU reader now covers the public 19-table M1 trunk with exact full-DEV parity, but training/writeback of its 15 extra tables remains unsupported. See [deepseek-4](../../loop/tasks/cpu-training-pilot/deepseek-4/REPORT.md) and [deepseek-90](../../loop/tasks/cpu-training-pilot/deepseek-90/REPORT.md). |
| Quantization-aware updates and trained float shadow | Finite integer/float shadow controls are delivered; a genuine trained FBS float snapshot is not published, so the trained comparison remains `NOT_RUN`. |
| FIT pre-registration and pre-run review | Complete. The public pilot sheet was FIT before compute and received a fresh non-author GO review. |

## Public pilot result

The pre-registered two-seed zero-versus-256-step run completed on the Ryzen 9 9950X. Exact CPU/serving export and target admission were preserved, but top-1 and top-8 did not move on either seed:

| Seed | Top-1 baseline → 256 steps | Top-8 baseline → 256 steps | Admitted baseline → 256 steps | Verdict |
|---|---:|---:|---:|---|
| 1 | 278 → 278 | 384 → 384 | 396 → 396 | fail registered gain bar |
| 2 | 224 → 224 | 285 → 285 | 292 → 292 | fail registered gain bar |

This is a negative for the tested public 256-step configuration, not a rejection of rank margin, FBS or CPU training generally.

The subsequent two-seed step-size comparison tested 1,024 attempts at 1, 16
and 64 Q12 units per accepted coordinate update. Unit-step seed 1 lost one
admitted target; step-16 lost one top-8; step-64 gained only 3 top-1 rows
against a 43-row bar. Seed 2 was unchanged through step-16, while step-64 lost
3 top-1 rows for a 1-row top-8 gain. Exact CPU/serving export stayed perfect
throughout. This is another bounded configuration negative and does not reject
all rank-margin or CPU credit assignments.

The same-amplitude BVG-K pivot then fixed every coordinate move at one Q12 unit
and compared one coordinate with four. K4 executed 437 four-coordinate blocks
on seed 1 and 238 on seed 2 with exact serving export. It reduced the held-out
margin-loss sum on both seeds, but top-1 did not move. Seed 1 lost one top-8 row
and one admitted target; seed 2 remained 224/285/292 top-1/top-8/admitted.
K4 therefore failed admission and ranking on seed 1 and did not exceed K1 on
either seed. This closes the tested coordinate-breadth configuration, not the
BVG-K mechanism generally.

The next materially different CPU comparison used the exact discrete
proposal/apply learner rather than rank margin. Baseline, K1 and K4 ran for 256
blocks on both public four-table seeds. Baseline read 38/108/361
top-1/top-8/admitted; K1 read 19/82/248 and K4 read 13/63/256. K4 committed 71
moves across 46 blocks, including fourteen blocks that committed more than one
coordinate, so the joint mechanism was exercised. Both trained settings lost
admission and ranking, and K4 trailed K1. This is a preserved negative for the
tested discrete K1/K4 budget, not a rejection of every proposal or credit
assignment. See [deepseek-88](../../loop/tasks/cpu-training-pilot/deepseek-88/REPORT.md).

The subsequent final served-gain comparison trained the full-width final FBS
normalization gain through the exact writer, memory and point path. Both seeds
accepted all eight updates and beat frozen and random distance with exact
five-arm parity, but top-1 stayed 291 and 243. This preserves a positive
bounded distance mechanism and a negative ranking/capability result; it closes
the tested one-code gain configuration rather than the geometric path.

## What is missing

The bounded public pilot supplies cost, exact-export, admission and ranking
evidence, but it is not a fresh capability test. The full item still needs
credentialed sealed complete-reply and fresh-phrasing grading, a configuration
that clears a registered ranking or capability bar, and a corpus-trained
full-path model rather than the finite synthetic-initial-trunk control.

## Done when

- The pre-registration exists before compute and names pass, fail and stop outcomes.
- The pilot runs on the named Ryzen 9 9950X and records its source, binary, hardware, OS and checksums.
- Training loss is never the only outcome; admission, top-1/top-8, exact export, complete replies and fresh phrasing are reported.
- Every historical training family has a final supported, superseded or rejected disposition linked to evidence.
- The REPORT states explicitly whether CPU training is useful for development, ready for a named qualification path, or rejected for the tested configurations.

## Cancel if

Any dependency still fails parity or deterministic resume. A fast but unverified trainer cannot enter the pilot.

## How to start

First collect the child REPORTs and rerun their parity/resume checks at the pilot head. Write the pre-registration with explicit pass, fail and stop outcomes and the machine identity. Import only the selected bounded corpus slice after the sheet is FIT.

## Rules that apply

- Pre-register before CPU research compute.
- Name the Ryzen 9 9950X and exact source/binary identity.
- Report capability and cost together; no loss-only success.

## Sources

- `docs/research/cpu-training-program.md`
- The six preceding CPU component items and their REPORTs
- The current evaluator and panel selected by the pre-registration
