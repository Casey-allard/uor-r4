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
scope: [crates/uor-r4-cpu-train/**, crates/uor-r4-integer/Cargo.toml, crates/uor-r4-integer/README.md, crates/uor-r4-integer/src/bin/uor-chat.rs, crates/uor-r4-integer/src/openai.rs, crates/uor-r4-integer/src/session_codec.rs, crates/uor-r4-integer/src/stack/chat.rs, crates/uor-r4-integer/src/stack/content_readout.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/tests/openai_loopback.rs, crates/uor-r4-integer/tests/session_codec_interop_fixture.rs, crates/uor-r4-core/Cargo.toml, crates/uor-r4-core/src/native_support.rs, crates/uor-r4-discrete/Cargo.toml, Cargo.toml, Cargo.lock, docs/harness/claims.json, docs/harness/prep/cpu-training-pilot*, docs/integration/current-state.md, docs/labs/cpu-training-pilot-*/resource-ledger.json, docs/labs/cpu-training-pilot-content-serving-2026-11-04/**, docs/loop/tasks/cpu-training-pilot/**, docs/research/cpu-training-program.md, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-training-pilot.md]
---

## Why it matters

Component speed does not show whether CPU training can improve a model at useful scale. The workstation must be measured under a bounded, pre-registered pilot with the same evaluator as the GPU path.

## Readiness as of 3 November 2026

The bounded public internal-development pilot has executed and failed its registered ranking bar. The full item remains open because genuinely fresh complete-reply inputs and a configuration that clears a registered bar are still unavailable. Public 64→256 interval restart is now deterministic on both seeds.

| Prerequisite | Current state |
|---|---|
| Shared-vector forward and exact serving arithmetic | Supported for the recurrence-only/no-pointer FBS mode, including snapped and raw gate/transition combinations; synthetic and public trained-artifact CPU/serving parity controls pass. The CPU Hamming port now also executes copy key 2 with the integer session's ordered two-token query/key and reports the key in the audit manifest. See [deepseek-87](../../loop/tasks/cpu-training-pilot/deepseek-87/REPORT.md). |
| Exact served-forward contract and workload audit | Delivered for the same supported FBS mode. `cpu-served-forward-audit` binds the artifact, reports active mechanisms, compares every Q12 query/token/kept-cell decision, separates modelled work from measured CPU timing, and inventories float-only gradient treatments. Both public raw seeds pass a 64-position complete-decision control. A follow-up corrected the contract's Hamming-range narrative and added independent integer/CPU/float boundary tests; the kernels were already causal and correct. A composed control exercises Hamming stream/readout reads, opening and reply memory and point-key fallback with exact parity. [Deepseek-91](../../loop/tasks/cpu-training-pilot/deepseek-91/REPORT.md) now trains the writer, selects a Hamming margin by exact served score, derives opening/reply tables and a point row from examples, writes a 536,516-byte full-path artifact, reads it back and passes 704 Q12 coordinates plus 11 complete decisions with zero mismatches. [Deepseek-92](../../loop/tasks/cpu-training-pilot/deepseek-92/REPORT.md) adds full, memory-off, point-off, Hamming-off and stripped arms; every arm preserves exact 704-coordinate/11-decision/88-cell parity while removing its named records. [Deepseek-93](../../loop/tasks/cpu-training-pilot/deepseek-93/REPORT.md) adds a Q24 coordinate surrogate that trains opening and reply movement weights by reparsing the exported artifact and scoring complete served queries. Both finite arms lowered target distance with exact parity, but top-1 stayed unchanged. The initial trunk and training sample are finite; corpus-scale held-out comparison remains unavailable. |
| Generated-content serving override | `IntegerStackModel::serve_step` can consult an optional artifact-bound `uor-r4.stack-content-readout/1` sidecar after the native writer or point-key fallback. Two public seeds matched all **3,296** generated decisions with zero token, fallback, signature, distance, firing or decision mismatches; the deciding attempt-5 run exercised prompt-ID, one-shot, REPL and OpenAI-loopback paths and reports the exact sidecar path/hash in JSON. The selector is allocation-free after session construction and retains **94.44%** of no-readout single-thread generation throughput. Raw `step` and `step_native`, the `UORLUT01` format and the positive content quality are unchanged. See [deepseek-140](../../loop/tasks/cpu-training-pilot/deepseek-140/REPORT.md). |
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

The next point comparison prospectively fixed a two-row target representation:
the observed within-target L1 medoid plus the remaining observed point with
maximum minimum L1 distance. On the same 60/60 unseen-key panel, this reduced
48 raw rows to 25 prototype rows. Seed 1 tied point-off at 0/60 and lost to the
shuffled-target prototype control at 3/60. Seed 2 reached 3/60 and beat both
controls by three, below the required paired margin of four. Every arm kept
exact CPU/integer parity and all treatment openings changed. This closes the
fixed target-balanced `k=2` construction, not target-local point geometry
generally. See
[deepseek-130](../../loop/tasks/cpu-training-pilot/deepseek-130/REPORT.md).

The next termination unit trained the final gain itself, rather than deciding
after the writer query. At the final reply-content row it aimed decoder BOS at
top-1; every other row protected content and rejected added BOS. Both seeds
accepted all eight one-code moves, changed the held-out query distance and kept
exact CPU/integer parity. BOS still stayed outside top-8 and reached 0/103
held-out terminal top-1; seed 2 also lost two nonterminal content rows. Every
free reply hit the 16-token cap. This closes the fixed final-gain coordinate
schedule; it does not reject geometric termination or a direct serving-level
BOS candidate. See
[deepseek-131](../../loop/tasks/cpu-training-pilot/deepseek-131/REPORT.md).

A different termination unit bypassed writer admission with a CPU-only
two-class Hamming classifier over the complete 256-coordinate writer query.
Eight observed prototypes per class and a training-fitted margin recognized
held-out reference endings at 97/103 on both seeds with 8/1,085 false stops,
and beat the no-gate and shuffled-label gates by 97 paired rows. Free
generation did not transfer: seed 1 stopped all 103 replies prematurely and
seed 2 stopped 43/103 prematurely, with no exact replies. This is promising
evidence that the query separates reference terminal rows, but not a usable
stop mechanism or serving change. It requires a generated-trajectory
discriminator and later coordinated integer parity. See
[deepseek-132](../../loop/tasks/cpu-training-pilot/deepseek-132/REPORT.md).

The admission follow-up ranked every target cell across all 256 writer cells
and ran independent K=8 and K=16 generated trajectories. K=16 preserved every
table payload byte and exact CPU/integer parity, but seed 1 fell from 30 to 17
positive-margin training rows and seed 2 rose only from 32 to 52, with 6 and
17 same-cell pairs against a 64-pair readiness gate. Held-out admission added
32 and 49 against 128; the equal-budget random-cell control was no weaker.
This closes the existing 8→16 admission-budget configuration and makes direct
generated-query/admission training the next mechanism. See
[deepseek-134](../../loop/tasks/cpu-training-pilot/deepseek-134/REPORT.md).

Direct final-gain admission training then moved the query on actual generated
trajectories instead of relying on post-admission ranking. The fixed
32-coordinate, ±32, 12-move objective accepted only one move on seed 1 and two
on seed 2. Training eligibility reached 36 and 49 rows against 640; held-out
admission fell 53→47 and rose 48→55; teacher-forced top-1 fell on both seeds.
Cross-cell permutation and equal-move random controls were not beaten. This
closes the fixed final-gain schedule, not direct query geometry generally. See
[deepseek-135](../../loop/tasks/cpu-training-pilot/deepseek-135/REPORT.md).

The next pivot bypassed writer admission with an exact nearest-example readout
over the complete generated 256-Q12 query. The true readout raised aligned
top-1 to 67 and 42 against writer 22 and 7 and a behaviorally distinct
shuffled control 4 and 18; matching-prefix depth rose to 60 and 33. Both seeds
met the prompt-paired bars. The zero-distance threshold covered 99% of
training rows, but leave-one-out target accuracy was only 199/894 and 201/894,
and the readout fired on just 103 and 117 generated decisions. Every reply hit
the 16-token cap with zero exact or BOS-complete results. This is a supported
sparse geometric content override and a serving-contract candidate, not useful
chat. See
[deepseek-136](../../loop/tasks/cpu-training-pilot/deepseek-136/REPORT.md).

The follow-up compared the sign code with a 2,048-bit modulo-256 code over the
same generated states. The code reduced ambiguous leave-one-out rows from 892
to 277 and 879 to 262 and raised hits from 199 to 300 and 201 to 284. Generated
top-1 rose from sign/writer 67/22 to 73 on seed 1 and from 42/7 to 58 on seed
2; matching depth rose from 60 to 64 and 33 to 39. Every code-to-writer,
code-to-shuffle and code-to-sign paired comparison passed. All replies still
hit the cap with zero exact or BOS-complete outputs, so the code is a stronger
serving-contract candidate but not completion or chat evidence. See
[deepseek-137](../../loop/tasks/cpu-training-pilot/deepseek-137/REPORT.md).

The next readout variant added BOS only for generated training states at the
normalized reference endpoint. The augmented table raised leave-one-out hits
from 300 to 325 and 284 to 315, but selected BOS on none of the held-out
replies. Seed 1 also lost content, 73→60 top-1 and 64→46 depth; seed 2 was
unchanged at 58/39. Every true stop arm still hit the 16-token cap, while the
shuffled endpoint controls stopped 2 and 27 replies prematurely. This closes
shared content-table endpoint rows. A separate two-class generated-state stop
gate was the next termination direction; the positive content code remained
intact. See
[deepseek-138](../../loop/tasks/cpu-training-pilot/deepseek-138/REPORT.md).

The separate stop gate trained terminal and nonterminal code-signature
prototypes without changing content targets. It stayed inside the 1% false-stop
limit but recalled only 8/84 and 9/84 training endpoints. The true arms stopped
10 and 11 replies, none exact; seed 1 had 4 shorter, 2 at-reference and 4
longer stops, while seed 2 had 5 shorter, 4 at-reference and 2 longer. The
shuffled controls stopped 9 and 21 replies. Corrected control-only paired
thresholds made seed 1 pass at 10 versus 9 and seed 2 fail at 11 versus 21, so
the shuffled gain did not replicate. Content stayed exactly 73/64 and 58/39
with zero ungated token mismatches. A wider prototype preflight did not
approach the training bar. This closes the fixed eight-prototype
code-signature gate at the 1% false-stop limit; another termination
representation or continuation objective is required. See
[deepseek-139](../../loop/tasks/cpu-training-pilot/deepseek-139/REPORT.md).

The positive code readout now has an artifact-bound serving sidecar. The
`uor-r4.stack-content-readout/1` file carries the 894-row, 207-target table and
binds the complete serving artifact SHA-256. Only `serve_step` consults it,
after the native writer and point-key fallback; raw `step` and `step_native`
remain unchanged. The content driver compared all 1,648 generated decisions on
each public seed and found zero token or decision mismatches, including
chosen token, fallback, packed signature, nearest distance and fired state.
The deciding attempt-5 run at implementation pin `486932e24` loaded the same
files through prompt-ID, one-shot, REPL and 18 OpenAI-loopback tests,
readout-bound sessions carry the canonical identity in `UR4SESS2`, and the
selector allocated no heap on a token step. Three 32-token throughput pairs
reached 4,529.714 sidecar and 4,796.190 no-readout generated tokens/s at one
thread, retaining 94.44%. This closes the content-readout serving-integration
gap, not completion or useful chat: the content arm remains 73/64 and 58/39
with no exact replies. See
[deepseek-140](../../loop/tasks/cpu-training-pilot/deepseek-140/REPORT.md).

The next full-row density gate retained all 1,568 generated training states
and conditioned terminal/nonterminal neighbour comparisons on the generated
reply offset. Deterministic leave-one-out fitting selected k=8 and a margin
under the 1% nonterminal false-stop limit. Seed 1 recalled only 19/84 terminal
rows against the registered 26/84 floor, with 9/1,484 false stops; its
class-preserving shuffled control also recalled 19/84, with 11/1,484 false
stops. The preflight therefore stopped the run before seed 2, free generation
or the worker-count repeat. This closes the fixed full-row offset-conditioned
K<=8 density representation and leaves the next termination attempt to a
continuation objective or a different terminal-state representation. See
[deepseek-142](../../loop/tasks/cpu-training-pilot/deepseek-142/REPORT.md).

The following continuation-horizon unit changed the objective to signed
remaining reference length rather than terminal membership. Exact-offset
Hamming neighbours predicted a lower-median horizon, with k fitted by
leave-one-out and a within-offset horizon permutation as control. Seed 1
retained the exact 1,568-row split and recalled 40/84 endpoints, passing the
26/84 training floor; true horizon error was 7,184 against 9,525 for the
control. However, 66/1,484 nonterminal rows stopped against the 14-row 1%
limit, so the registered preflight stopped the run before seed 2, free
generation or the repeat. This closes the fixed exact-offset lower-median
horizon configuration while preserving the signal that ordinal horizon
prediction carries information. A later attempt needs a different trajectory
representation or a nonterminal-aware continuation objective. See
[deepseek-143](../../loop/tasks/cpu-training-pilot/deepseek-143/REPORT.md).

The following content unit added the first-order generated-state transition to
the positive modulo-256 code: current code followed by current XOR previous.
The 4,096-bit trajectory readout reduced leave-one-out ambiguity from 277 to
254 and changed 632/633 effective assignments under the target control, but
top-1 fell from 300 to 297 against code. The registered preflight therefore
stopped before seed 2 or generation. This closes the fixed current-plus-XOR
concatenation; higher-order transition history or learned transition geometry
remains open. See
[deepseek-144](../../loop/tasks/cpu-training-pilot/deepseek-144/REPORT.md).

An exact-head review then found that the first trajectory implementation's
full-generation fallback check compared reference-aligned decisions with all
generated decisions. The repair added a separate generated-decision count and
a capped-horizon regression, then reproduced the seed-1 preflight report and
structured failure byte-for-byte from the corrected source. The same 300
versus 297 top-1 result still stops before generation. See
[deepseek-145](../../loop/tasks/cpu-training-pilot/deepseek-145/REPORT.md).

The next termination unit formed a phase-aligned transition by rotating the
current modulo-256 code by the previous code's popcount phase and XORing the
previous code. The existing offset-conditioned density rule retained all 1,568
rows but recalled only 14/84 endpoints against the 26/84 floor; false stops
were 7/1,484. Its non-vacuous shuffled control changed 48 firing decisions but
recalled 16/84. The preflight stopped before generation. This closes the fixed
phase-alignment/density configuration, not transition geometry generally. See
[deepseek-146](../../loop/tasks/cpu-training-pilot/deepseek-146/REPORT.md).

The next writer unit tested training on the model's own generated prefixes
instead of reference prefixes. Its exact scan fed self-generated tokens through
FBS, Hamming and opening/reply memory, aligned live queries with reference
content at the same offset, and prepared reference, static/dynamic and
same-cell shuffled controls. The registered preflight stopped before any
update: seed 1 found only 30 true / 33 shuffled eligible admitted decisions and
seed 2 found 32 / 32, against 640; same-cell pairs were only 11-12 against 64.
Prompt and cell coverage were otherwise ample. This is an insufficiency result,
not a writer-training negative: the next content mechanism must expose far more
admitted generated-state targets before ranking can be tested. See
[deepseek-133](../../loop/tasks/cpu-training-pilot/deepseek-133/REPORT.md).

The public 19-table M1 trunk is now additively trainable and writable on CPU,
rather than read-only. One bounded run added 128 exact teacher-forced rows to
the unigram and all 12 sparse sufficient-statistic tables, exported and
re-imported all 19 tables, independently reconstructed every decoded count, and
preserved exact CPU/integer parity for 512 document-disjoint evaluation rows.
The counts changed but the held-out decisions did not, so the writeback gap is
closed while the need for a materially larger bounded dose remains. See
[deepseek-124](../../loop/tasks/cpu-training-pilot/deepseek-124/REPORT.md).

The next fixed dose raised additive count training from 128 to 4,096 rows on
the same public artifact and 512-row document-disjoint panel. Every count table
gained exactly 4,096 entries and writeback/parity remained exact. The exact
update changed four held-out decisions, but admitted/top-1/top-8 each fell by
one while the shuffled control changed nothing, so the registered comparison
is negative. This closes the fixed 4,096-row additive dose, not other count
schedules. See [deepseek-125](../../loop/tasks/cpu-training-pilot/deepseek-125/REPORT.md).

The count driver now uses one explicit local Rayon pool for ordered training-row
derivation and held-out CPU/integer evaluation. Eight fresh runs in mirrored
worker order reproduced every deepseek-125 artifact, table, decision and
checksum exactly while recording a strict positive peak RSS. The best mean
speedup was **1.065x** at 16 workers, below the pre-registered 1.5x bar; 32
workers was slightly slower. Flat wall time and roughly one-core process
utilization suggest the two parallel loops are a small share of total driver
work; stage timing is required before naming the remaining bottleneck. This is
a performance/instrument negative, not a model result. See
[deepseek-126](../../loop/tasks/cpu-training-pilot/deepseek-126/REPORT.md).

The count driver now reports and validates a complete 21-stage profile, failing
closed on duplicate, missing or extra stages. Four mirrored runs at one and
sixteen workers reproduced every deepseek-125/126 artifact and non-stage report
field while accounting for all measured wall time. `exact_verify` was the
largest stage at both worker counts, at **26.70%/28.27%**, and exact plus
shuffled verification consumed **45.32%/48.05%** of wall time. Held-out
evaluation consumed another **32.20%/32.02%**. The measured next target is to
remove repeated full-trunk decoding from verification without weakening its
independent full-table check. This is diagnostic evidence, not a speed or
capability gain. See
[deepseek-127](../../loop/tasks/cpu-training-pilot/deepseek-127/REPORT.md).

The next CPU implementation comparison removed six of the ten full-trunk
imports in exact and shuffled verification. The source table set is moved into
exact expected-count reconstruction, each exported arm is decoded once, and
the shuffled expectation still uses a separate fresh source import. Eight
mirrored runs at one and sixteen workers reduced mean wall time by
**19.77%/22.28%** and exact-plus-shuffled verification time by
**49.94%/50.46%**. Every artifact, table, decision, checksum and normalized
report remained byte-identical, and peak RSS stayed below 1.546 GiB. This is a
CPU implementation speedup, not a model or capability result. See
[deepseek-128](../../loop/tasks/cpu-training-pilot/deepseek-128/REPORT.md).

The next CPU implementation step reused the integer model already loaded for
input validation and the CPU trunk reader already loaded for teacher-forced
row derivation when evaluating the unchanged arm. Exact and shuffled arms
continue to reload independently. Eight mirrored runs at one and sixteen
workers reduced mean wall time by **16.01%/18.36%** and the no-update
evaluation stage by **92.68%/99.42%**, with every artifact, table, decision,
checksum and normalized report identical. Peak RSS remained below 1.55 GiB of
the strict 2-GiB stop. This is CPU implementation performance evidence, not a
capability result. See
[deepseek-129](../../loop/tasks/cpu-training-pilot/deepseek-129/REPORT.md).

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
