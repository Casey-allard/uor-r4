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
scope: [crates/uor-r4-cpu-train/**, crates/uor-r4-core/Cargo.toml, crates/uor-r4-core/src/native_support.rs, crates/uor-r4-discrete/Cargo.toml, Cargo.toml, Cargo.lock, docs/harness/prep/cpu-training-pilot*, docs/loop/tasks/cpu-training-pilot/**, docs/research/cpu-training-program.md, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-training-pilot.md]
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
| Ordered parallel evaluation | Panel rows and counterfactual candidate lanes now use bounded Rayon workers, with source-ordered integer reductions and serial coordinate/joint selection. Tests reproduce panel readings, accepted steps, artifacts and checkpoints exactly across 1, 2, 3, 5, 8, 16 and 32 workers. At 16 workers, the synthetic panel reached 90,010 rows/s (9.13x) and counterfactual groups reached 4,686-5,192 candidate evaluations/s (7.33-8.80x). Thirty-two workers were workload-dependent rather than uniformly faster. See [deepseek-98](../../loop/tasks/cpu-training-pilot/deepseek-98/REPORT.md). On the real public corpus, two reversed-order runs measured 62.15 s at one worker, 8.48 s at eight, 4.63 s at 16 and 3.53 s at 32, an average 17.62x gain with byte-identical artifacts, checkpoints, panels and audits. The update chain and recurrent audit are dependency-ordered; the gain is concentrated in exact panel evaluation. See [deepseek-100](../../loop/tasks/cpu-training-pilot/deepseek-100/REPORT.md) and its corrected [deepseek-101](../../loop/tasks/cpu-training-pilot/deepseek-101/REPORT.md) accounting. |
| CPU build graph | `uor-r4-discrete`'s build script now uses `uor-r4-core` with its default `integer-runtime` feature disabled. The build path therefore executes only the exact SpiralCore/prime-route metadata modules and does not compile a second feature set of `uor-r4-integer`. Ordinary `uor-r4-cpu-train` checks run in a shared target without the host/target `cdylib` collision or a fresh target directory. Normal core callers retain the full default API. See [deepseek-107](../../loop/tasks/cpu-training-pilot/deepseek-107/REPORT.md). |
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

The first unseen-exact-opening-key point transfer is now recorded. On both
public seeds, training-only point rows beat point-off, but shuffled targets
beat the learned table on seed 2. This closes the tested 60/60 split, 48-row
point table, Q24 query points and nearest-target rule as a bounded negative; it
does not reject every point-key representation. See
[deepseek-104](../../loop/tasks/cpu-training-pilot/deepseek-104/REPORT.md).

An eight-row majority vote over the same point rows then beat point-off, the
single nearest row and the eight-row shuffled-target vote on both seeds, at
`3/60` versus `1–2/60`. The gain was below the separate three-sigma threshold.
The rule is now artifact-bound: `vote_k` defaults to the byte-compatible
one-row behavior, and a synthetic plus two-seed public control proves exact
native-session, CPU-oracle and served-forward parity at `vote_k = 8`. Seed 2's
shuffled vote also reached `3/60`, so the mechanism remains subthreshold and
is not a language result. See
[deepseek-105](../../loop/tasks/cpu-training-pilot/deepseek-105/REPORT.md) and
[deepseek-106](../../loop/tasks/cpu-training-pilot/deepseek-106/REPORT.md).

A subsequent FIT free-generation screen tested whether the vote propagates
beyond the opening. On both public seeds the vote reached `3/60` correct
openings, but every arm produced zero exact-reference and zero BOS-complete
replies; all 60 prompts per arm hit the 16-token cap. Seed 1's vote also
produced fewer distinct replies and more repeated 4-grams than nearest. This
closes the fixed greedy point-vote reply configuration and moves the next work
to reply content and termination rather than another vote width. See
[deepseek-108](../../loop/tasks/cpu-training-pilot/deepseek-108/REPORT.md).

The memory-weight surrogate was then scaled from eight keys / 60 training / 56
evaluation examples to 16 keys / 102 training / 114 evaluation examples with
32 accepted Q24 movement updates. Trained top-1 rose `507→526` and `441→484`,
beating frozen and equal-move random on both seeds, while target distance fell
below both controls. The gains were `+19/+19` and `+43/+53`, still below the
registered three-sigma thresholds of 64 and 60. This is a stronger
repeated-key mechanism result and a ranking-signal negative. See
[deepseek-109](../../loop/tasks/cpu-training-pilot/deepseek-109/REPORT.md).

The expanded artifacts were then replayed through complete greedy replies,
together with a direct exact-memory target arm and a non-vacuous shuffled-target
control. Both seeds and a fresh seed-1 repeat hit the 16-token cap on every one
of the 114 prompts in every arm: zero exact-reference and zero BOS-complete
replies. Trained opening correctness moved from 0 to 14 on seed 1 but stayed at
4/4 on seed 2. Direct targets reached 50/114 openings on both seeds but did not
complete or terminate a reply; target-frozen and target-trained outputs were
identical. This closes the fixed memory-weight/target-arbitration complete-reply
configuration and moves the next work to continuation and termination rather
than more coordinate steps.
See [deepseek-110](../../loop/tasks/cpu-training-pilot/deepseek-110/REPORT.md).

The next CPU diagnostic measured why BOS never terminated those replies. At
1,824 decisions per arm, missing BOS-cell admission dominated: 1,772/1,824 for
seed-1 frozen and 1,717 for trained, with 1,710/1,824 in every seed-2 arm.
When admitted, BOS never reached top-8 or top-1; its best rank was 47-89.
Chosen BOS and decoder suppression were zero everywhere. This rules out the
stop contract as the active fault and prioritizes geometric termination-cell
admission, followed by a BOS movement objective for the minority of admitted
but distant steps. See
[deepseek-111](../../loop/tasks/cpu-training-pilot/deepseek-111/REPORT.md).

A direct training-derived termination-point gate then tested whether a local
L1 radius could replace cell admission. On both public seeds it retained 102
EOS terminal queries, derived radii of 98,313 and 220,730, and removed no
exact collisions. The trained radius never fired: every evaluation reply
remained byte-identical to no-gate and hit the 16-token cap. An equal-size
random nonterminal gate fired at the first decision on all 114 replies on both
seeds, but every stop was premature and no reference became exact. This closes
the fixed single-radius point-gate configuration, not geometric termination;
the next attempt must change the representation or compare terminal against
nonterminal density. See
[deepseek-118](../../loop/tasks/cpu-training-pilot/deepseek-118/REPORT.md).

A top-writer-cell majority gate then tested cell identity instead of point
distance. Each seed learned from 102 terminal and 1,192 nonterminal training
queries. Cell 1 occupied 92 terminal rows and was the dominant learned cell;
seed 2 also learned cells 145 and 191. The learned gate stopped 114/114 seed-1
replies and 48/114 seed-2 replies, but every BOS was premature and the exact
count stayed zero. Equal-size disjoint shuffled-cell controls never fired.
This resolves the missing-admission question but shows that cell identity alone
does not separate the final row from ordinary continuation. See
[deepseek-120](../../loop/tasks/cpu-training-pilot/deepseek-120/REPORT.md).

A finite full-path acceptance fixture now covers the 2,048-position boundary
Claude's long-window track targets. With mixed-key Hamming reads, opening/reply
memory and a point key active, the CPU oracle and integer serving agreed on
all 131,072 Q12 coordinates, 2,048 token decisions and 16,384 kept cells. A
copy read at position 1,024 retained early position 100, replay was
deterministic, and position 2,049 was refused as context exhaustion. This
prepares the existing `cpu-served-forward-audit` check for Claude's first
trained long-context export; it is not a long-context capability result. See
[deepseek-121](../../loop/tasks/cpu-training-pilot/deepseek-121/REPORT.md).

A cell-relative orbit-phase gate then tested whether terminal and continuation
rows sharing a writer cell separate by their position among the 16 orbit roots.
The 32-value signature was reached on both seeds, but every learned stop was
premature and both paired comparisons failed; the duplicate-safe label control
also made only premature stops. This closes the fixed two-block nearest-class
orbit-phase configuration. The next termination attempt needs a different
within-cell representation or continuation-state invariant rather than a
post-observation retune of this signature. See
[deepseek-122](../../loop/tasks/cpu-training-pilot/deepseek-122/REPORT.md).

A materially different point-training comparison then reduced each supervised
opening target to an observed geometric medoid by exact within-target L1 sum.
The medoid and a matched shuffled medoid ran against point-off, raw nearest and
the served raw-row vote on the fixed unseen-key split. Seed 1 tied both primary
controls at zero; seed 2's 4/60 opening result passed the point-off margin but
beat its shuffled medoid by only one against a threshold of two. This closes
the fixed 48-to-18 target-medoid construction, while leaving target-specific
local neighborhoods open. See
[deepseek-123](../../loop/tasks/cpu-training-pilot/deepseek-123/REPORT.md).

The public 19-table M1 trunk is now additively trainable and writable on CPU,
rather than read-only. One bounded run added 128 exact teacher-forced rows to
the unigram and all 12 sparse sufficient-statistic tables, exported and
re-imported all 19 tables, independently reconstructed every decoded count, and
preserved exact CPU/integer parity for 512 document-disjoint evaluation rows.
The counts changed but the held-out decisions did not, so the writeback gap is
closed while the need for a materially larger bounded dose remains. See
[deepseek-124](../../loop/tasks/cpu-training-pilot/deepseek-124/REPORT.md).

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
