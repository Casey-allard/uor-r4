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
scope: [crates/uor-r4-cpu-train/**, crates/uor-r4-integer/Cargo.toml, crates/uor-r4-integer/README.md, crates/uor-r4-integer/src/bin/uor-chat.rs, crates/uor-r4-integer/src/openai.rs, crates/uor-r4-integer/src/session_codec.rs, crates/uor-r4-integer/src/stack/chat.rs, crates/uor-r4-integer/src/stack/content_readout.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/tests/openai_loopback.rs, crates/uor-r4-integer/tests/session_codec_interop_fixture.rs, crates/uor-r4-core/Cargo.toml, crates/uor-r4-core/src/native_support.rs, crates/uor-r4-discrete/Cargo.toml, Cargo.toml, Cargo.lock, docs/harness/claims.json, docs/harness/prep/cpu-training-pilot*, docs/integration/current-state.md, docs/labs/cpu-training-pilot-*/resource-ledger.json, docs/labs/cpu-training-pilot-content-serving-2026-11-04/**, docs/labs/cpu-training-pilot-trajectory-density-2026-11-04/**, docs/labs/cpu-training-pilot-copy-key-depth-2026-11-05/**, docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**, docs/labs/cpu-training-pilot-copy-termination-oracle-2026-11-05/**, docs/labs/cpu-training-pilot-copy-generated-stop-2026-11-05/**, docs/loop/tasks/cpu-training-pilot/**, docs/research/cpu-training-program.md, docs/research/notebook/cpu-training.md, docs/roadmap/items/cpu-training-pilot.md]
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
| Candidate artifact gate | `scripts/audit_served_forward_gate.py` binds a `UORT` token slice and optional expected artifact hash, validates the complete deterministic audit, and atomically promotes only exact-passing artifacts. The exact CPU corpus driver now runs the same audit internally before final promotion, validates all workload and mismatch fields, persists `final.audit.json`, and reloads both outputs. Public seed 1 passed a zero-step 64-position smoke with zero mismatches over 16,384 Q12 coordinates. Studio trained-model staging now requires a locally validated receipt and publishes it beside the model; remote cards pin the artifact and receipt; the release-bundle packager requires adjacent receipts for every UORLUT. A separate read-only gate now checks that every token's two 8-coordinate writer ports survive Q12 export/read-back at its cell roots; a synthetic positive passes and both public unanchored artifacts are rejected as expected. The synthetic demo and arbitrary public-URL slot remain outside the trained-release gate, and automatic workflow publication/receipt signing are still open. See [deepseek-86](../../loop/tasks/cpu-training-pilot/deepseek-86/REPORT.md), [deepseek-94](../../loop/tasks/cpu-training-pilot/deepseek-94/REPORT.md) and [deepseek-181](../../loop/tasks/cpu-training-pilot/deepseek-181/REPORT.md). |
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

The following termination attempt concatenated the surviving current code with
that phase-aligned transition. The fixed 4,096-bit state selected k=2 and a 775
margin, changed 58 leave-one-out decisions and preserved the ungated content
prefix exactly over 1,145 decisions. Seed 1 still recalled only 16/84 endpoints
against the 26/84 floor; its 11/1,484 false stops stayed inside the 14-row
limit, while the shuffled control recalled 18/84 with 13/1,484 false stops. The
preflight stopped before seed 2 or generation. This closes the exact
current-code-plus-phase-transition density configuration. See
[deepseek-147](../../loop/tasks/cpu-training-pilot/deepseek-147/REPORT.md).

The following CPU read screen extended Claude's exact copy-key backoff from
depth two to depth four. Public DEV reproduced Claude-267's parity counts
exactly. On two new m-world-v2 panels, the longest-first four-to-one key beat
`k2>k1` by 9:1 and 8:0 paired discordant decisions and beat a random-depth
control by 32:7 and 33:5. The absolute gain was +8 correct copyable decisions
on each panel. This selects depths three and four for a trained-model test; it
is not a model, language or serving result. See
[deepseek-149](../../loop/tasks/cpu-training-pilot/deepseek-149/REPORT.md).

A follow-up fitted a 36-cell empirical precision table over suffix depth and
fixed candidate-age bins on new `train,train` seed 2601, then evaluated the
unchanged table on new `dev,dev` seed 2602. The router was right on
**913 / 1,655** copyable positions, eight below longest-first at **921**.
Although it beat `k2>k1` by 913 to 912 and random depth by 913 to 876, all three
pre-registered comparisons were required. Longest-first therefore remains the
selected exact-copy candidate; the negative closes this fitted depth/age table,
not learned copy reads generally. See
[deepseek-151](../../loop/tasks/cpu-training-pilot/deepseek-151/REPORT.md).

The complete-reply bridge then applied the five fixed selectors to generated
trajectories on two new panels and two public artifact seeds. Longest-first
exact copy beat `k2>k1`, fixed depth 1, random depth and the unmodified writer
on correct reference body positions in all four artifact/panel cells. However,
every decision was a copy decision, no writer fallback occurred, and every
reply hit the 64-token cap with zero BOS-complete or exact replies. Because the
prompt-derived copy domain excludes BOS, longest-first copy carries content
headroom but cannot terminate by itself. The result preserves depth-three/four
geometry for a combined copy-plus-termination read rather than selecting an
exact-copy decoder. See
[deepseek-153](../../loop/tasks/cpu-training-pilot/deepseek-153/REPORT.md).

An oracle-stop replay of those sealed traces then separated termination from
content. Truncating longest-first output at the reference length and inserting
BOS would complete **11/453** train and **13/449** dev replies on both artifact
seeds; `k2>k1` reached 8/8, random 0/2, and depth-1 and writer zero. Every
paired exact and body-position comparison favored longest-first. Most current
copy bodies still miss, so termination is the immediate blocker and
terminal-token accuracy is the next content target. See
[deepseek-155](../../loop/tasks/cpu-training-pilot/deepseek-155/REPORT.md).

The first generated-trajectory stop-gate attempt then replayed those trace
paths through the exact CPU forward and fitted eight terminal/eight nonterminal
sign prototypes on the train panel. The gate fit under the 1% training
false-stop cap but produced **0 exact replies** on both artifact seeds and both
panels. It fired at the reference position on some rows, but never on an exact
body, and training premature stops were 34/31 against the 4-reply limit. This
rejects all-reference-length terminal labeling for the fixed prototype gate.
See [deepseek-157](../../loop/tasks/cpu-training-pilot/deepseek-157/REPORT.md).

The exact-body follow-up then fitted the same gate only on generated
trajectories whose train bodies already reached the full reference. The gate
fit 7/11 terminal trajectories on both artifact seeds with no selected-row
false stops. A registered confirmation on an unopened seed-2722 dev panel had
458 reachable replies and a ten-reply oracle ceiling, but the gate produced
**0/10** exact replies on both seeds and 5/9 premature stops against a
four-stop limit. All shuffled, all-reference and no-gate controls returned zero
exact dev replies. Exact-body filtering is closed for this prototype family;
the next termination test needs a copy-boundary or continuation objective and
better terminal-token construction. See
[deepseek-159](../../loop/tasks/cpu-training-pilot/deepseek-159/REPORT.md).

The next content screen tested whether exact suffix matches deeper than four
add information. On a fresh 458-reply panel, longest-32 selected depth 32 on
most decisions and beat every non-nested shallower or randomized control, but
it chose the same token as nested longest-4 at every fresh-dev decision. Both arms reached
**300/4,758** fresh-dev and **409/4,661** train correct body positions; no arm
completed a reply. This closes exact depths eight through 32 and directs the
next content test to a non-exact geometric read rather than a longer exact
suffix. See
[deepseek-163](../../loop/tasks/cpu-training-pilot/deepseek-163/REPORT.md).

Static token-row geometry did not repair that gap. A second fresh screen ranked
the same exact-copy candidates by sign-Hamming distance between the current
Q12 writer query and each token's shared vector-table row. It changed
thousands of decisions and remained deterministic, but scored only **270 and
286/4,822** fresh body positions against **624** for longest-32, and **174 and
195/4,661** against **409** on train. It beat fixed depth 1 and writer but lost
longest-32, `k2>k1` and random-32. This closes raw per-token writer-row sign
proximity as an arbitration rule. The next geometric-copy read must learn or
derive candidate agreement from context instead of treating a static row as a
copy key. See
[deepseek-165](../../loop/tasks/cpu-training-pilot/deepseek-165/REPORT.md).

Learning from generated query codes did not escape that ceiling. A modulo-256
readout fitted on 4,661 longest-copy train states was restricted to exact-copy
candidates and evaluated on a fresh seed-2763 panel. It arbitrated 210 and 174
decisions but scored with longest-32 on seed 1 and gained one body position on
seed 2, below the paired threshold; the shuffled learned read also failed the
comparison. Leave-one-out training top-1 was only 116 and 125 of 1,024. This
closes the fixed row-level nearest-example copy read. The next candidate
representation must model multi-token continuation or candidate-specific
context rather than map one writer state to one token. See
[deepseek-167](../../loop/tasks/cpu-training-pilot/deepseek-167/REPORT.md).

One-step exact-state lookahead did not repair the copy signal. The selector
rolled each depth-1..4 candidate through a cloned forward and used the next
state's learned readout distance. On fresh seed 2764 it scored **194 and
191/4,799** body positions versus **393** for longest-32 and strongly favored
depth four. Shuffling the readout targets changed zero decisions because the
score used distance only, so the registered control failed closed. This closes
distance-only candidate lookahead; the next attempt must score predicted token
identity or multi-token continuation content with a new panel. See
[deepseek-169](../../loop/tasks/cpu-training-pilot/deepseek-169/REPORT.md).

Making lookahead continuation-sensitive repaired the control but not the
content signal. The new score compared each candidate-conditioned prediction
with the token that historically followed the candidate's retained source
occurrence. On fresh seed 2765, target shuffling changed 167 and 278 decisions,
yet continuation lookahead scored only **242 and 262/4,899** body positions
against **511** for both no-lookahead learned and longest-32. It found a
continuation match on only 231 and 209 of 30,784 evaluated decisions. This
closes this single-token learned lookahead configuration. Longest exact copy
remains the stronger content selector; the next completion work should target
an independent termination path rather than retune this scalar. See
[deepseek-171](../../loop/tasks/cpu-training-pilot/deepseek-171/REPORT.md).

The next candidate-ranking unit changed the learning target rather than another
scalar. It represented every exact-copy candidate by depth, sign agreement,
eight rotated-row agreements, recency, frequency and writer agreement and
defined an eight-pass integer pairwise perceptron with shuffled-target and
depth-only controls. The seed-2766 fit stopped before that perceptron was
called: longest-32-generated train panel 2711 produced only **74**
two-candidate groups with the reference next token present, against the
registered **640** floor. No ranker selection or reply evaluation ran. This is
an insufficiency result for the longest-32 trajectory source, not a negative
for pairwise candidate ranking. The next attempt must create materially more
candidate-labelled states, with a new seed and registration. See
[deepseek-173](../../loop/tasks/cpu-training-pilot/deepseek-173/REPORT.md).

Teacher-forced reference prefixes then tested whether generated-trajectory drift
caused that supervision shortage. It raised the candidate-group count only from
**74 to 85**, against the registered **640** floor, and stopped before
perceptron fitting or reply evaluation. This identifies the exact-copy
depth-1..8 candidate definition itself as the bottleneck, not longest-generated
drift. Generated inference remained unchanged. A later ranker must broaden the
candidate construction or use a denser objective, with a new seed and
registration. See
[deepseek-175](../../loop/tasks/cpu-training-pilot/deepseek-175/REPORT.md).

The next ranker broadened the exact-copy candidate set with the writer's top
128 non-BOS geometric candidates and a writer fallback, while retaining exact
depths through 32. On artifact seed 1 this produced **2,442** supervised groups
and 332 target tokens, a large increase over exact copy alone. Artifact seed 2
stopped at **1,847** groups against the fixed 2,048 floor, so the registered
two-seed comparison was not evaluated and the run is
**INSUFFICIENT_TRAINING_DATA**. Seed 1's diagnostic ranker also lost to
longest-32. The broader union is therefore promising for supervision but not
yet robust or strong enough; the next unit must clear both artifacts under a
new seed without lowering the floor. See
[deepseek-177](../../loop/tasks/cpu-training-pilot/deepseek-177/REPORT.md).

Widening the CPU-only geometric cell source supplied enough supervision but did
not improve arbitration. K12 stopped before evaluation because seed 2 had only
**2,043** target-present groups against 2,048; that selection attempt is
preserved as an instrument fault. K16 cleared every floor on both public
artifacts. On fresh seed 2770, its longest-32-anchored linear correction
overrode the baseline on all 27,264 decisions and scored only **13 / 4** body
positions, against **603 / 603** for longest-32, geometry-only **202 / 68**,
unanchored **218 / 603** and writer **47 / 48**. Both repeats matched and every
control differed from longest-32. This closes the fitted K16 linear correction,
not geometric candidate ranking generally. Future copy/body work needs a
different candidate-conditioned representation or objective; longest-32
remains the current body selector. See
[deepseek-179](../../loop/tasks/cpu-training-pilot/deepseek-179/REPORT.md).

The next CPU comparison trained the two query ports directly toward each
target's cell roots, starting from the table-anchored construction Claude-290
introduced. Eight moves were accepted on each public artifact with exact
CPU/serving parity. Seed 1 improved port distance over the table-anchor and
random controls but lost to the moved-count-matched shuffled-root control by
3,055 units. Seed 2 reduced port distance below all three controls, but lost
one admitted target and one top-1 against the table-anchor baseline. More
importantly, anchoring every token row's shared input/writer ports collapsed
admitted/top-1 from 245/181 to 120/55 on seed 1 and from 171/136 to 29/10 on
seed 2. This is a **REJECT** for the tested query-port gain objective and a
strong cost signal for table-wide port anchoring. The run does not separate
the input-code effect from direct writer-row mismatch; the next representation
must decouple writer addressing from the learned input code or train both with
an identity-preservation objective. See
[deepseek-183](../../loop/tasks/cpu-training-pilot/deepseek-183/REPORT.md).

The four-arm table-anchor isolation then separated that cost. On seed 1,
raw/input-only/writer-only/combined admitted/top-1/top-8 was
245/181/236, 120/55/89, 245/195/233 and 120/55/86. On seed 2 it was
171/136/164, 29/9/20, 171/137/162 and 29/10/20. Input-only reproduced combined
admission on both seeds and combined top-1 on seed 1; writer-only preserved
raw admission, kept cells and port distance exactly. The loss is therefore an
input-code effect, not direct writer-row mismatch. Root-addressed writer
scoring remains viable only if its address rows are separated from the learned
input rows or trained with input-code preservation. The hybrid table pairs are
not representable in the current one-table artifact and make no serving claim.
See [deepseek-185](../../loop/tasks/cpu-training-pilot/deepseek-185/REPORT.md).

The next CPU training comparison kept the shared table raw and moved only the
final-gain query ports, so every result remained an exportable one-table
artifact. Seed 1 accepted eight moves and seed 2 five, with exact 616-row
CPU/serving parity and retained admission/top-1/top-8. Target-port distance
improved over raw on both seeds (`29,882,444→29,613,365` and
`23,870,298→23,786,304`). However, the shuffled-cell arm was behaviorally
identical to trained on both seeds. The trained arm beat random gain
(`29,613,365 < 29,818,562` and `23,786,304 < 23,802,494`), but the result is a
**CONTROL_FAULT** with no promotion: the one-step cell-rotation control is
closed for this raw-conditioned configuration, so the objective still needs a
behaviorally distinct control. See
[deepseek-188](../../loop/tasks/cpu-training-pilot/deepseek-188/REPORT.md).

The exact serving path now has a cross-width scale preflight. Run 2 exercised
the current whole-table-nearest recipe. Synthetic projection and full
Hamming-plus-memory artifacts at widths 256, 512 and 1,024 and representative
contexts 16, 64 and 192 all exported, reloaded and matched
`IntegerStackSession` on every one of three complete repeated audits. The run
covered 4,896 token decisions, 2,924,544 Q12 coordinates and 39,168 kept cells
with zero mismatches. Projection artifacts grew 2,145,088 -> 4,280,384 ->
8,551,488 bytes; single-pass serving throughput fell from 1,750-1,801 to
936-940 and then 477-481 tokens/s. Full-path throughput fell from 1,721-1,763
to 910-927 and then 466-476 tokens/s. The run took 18.476 s and peaked at
76,388 KiB. This removes output-path uncertainty before Claude's scale run, but
does not establish quality growth, real-text training or GPU-memory
feasibility. See
[deepseek-190](../../loop/tasks/cpu-training-pilot/deepseek-190/REPORT.md).

The first exact reply-boundary experiment then tested termination as the next
symbol of an observed reply continuation. A TRAIN-only suffix table admitted
decoder BOS only when the longest known generated suffix had terminal
observations and no continuation; no-boundary and depth-preserving shuffled
controls ran beside the established `Longest32To1` body selector. The
TRAIN-only preflight exercised the mechanism: true boundary produced 11 exact
and 41 premature stops, while shuffled produced 1 exact and 9 premature. On the
fresh seed-2723 panel the oracle completed only 7 of 456 reachable replies,
below the registered 10-reply power floor. True/no-boundary/shuffled exact
replies were 0/0/0 on both seeds, while true and shuffled boundary stopped 14
and 3 replies with 346 correct body positions in every arm. The result is
**INSUFFICIENT_POWER**, not a mechanism negative: stopping was exercised, but
fresh body reconstruction left too few exact completions to assess. See
[deepseek-191](../../loop/tasks/cpu-training-pilot/deepseek-191/REPORT.md).

The next complete-reply screen trained a global discrete reply-suffix table
from TRAIN panel 2711. Every suffix of length one through 32 counted its next
reply token, with decoder BOS as the terminal target. At generation, the
longest known generated suffix selected its most frequent successor, ties to
the smaller token, with writer and true-boundary fallback. Fresh seed 2772
cleared the completion-power floor with 14 oracle-exact bodies.

The treatment still rejected decisively. Trained suffix-32 produced
**57/86** correct body positions on artifact seeds 1 and 2, against **514/514**
for local `Longest32To1`, and made **14/227** premature stops against the
four-stop ceiling. Depth-one produced **48/89**, the successor-multiset shuffle
**0/23**, and the writer **52/51**. The table changed 15,601 shuffled
associations and thousands of reachable decisions, so both mechanism and
control were active. The fixed global plurality construction displaced a
strong local copy signal and stopped too early on fresh data. The result
closes this table/objective, not suffix memory generally; content continuation
and stopping should be separated, confidence/fresh-support should gate an
override, and a materially larger reply corpus remains the scaling route. See
[deepseek-193](../../loop/tasks/cpu-training-pilot/deepseek-193/REPORT.md).

The follow-up removed BOS from that global table and preserved local exact copy:
a confident content suffix could act only when local `Longest32To1` had no
candidate. On the TRAIN preflight, local copy had a candidate at every
**25,202** reachable generated decisions. The treatment made zero fallback
decisions against the required 100 and was behaviorally identical to local.
Content-only was active with 12,684 suffix decisions but reached only **43**
body positions versus **409** for local. The run stopped before its fresh panel
as **INSUFFICIENT_FALLBACK_COVERAGE**, not a mechanism negative. The strict
local-absent gate is structurally unreachable in this path. The next design
must compare local and global suffix depth directly, allowing a deeper
confident global continuation to arbitrate against a shallow local match. See
[deepseek-196](../../loop/tasks/cpu-training-pilot/deepseek-196/REPORT.md).

The next unit aligned the memory surfaces: full prompt-plus-reply histories
became the suffix table, and global/local depth was compared on the same stream.
A confident global suffix replaced local only when deeper; ties stayed local.
On fresh seed 2774 and both artifacts, treatment produced **17/17** exact
replies and **847/844** body positions, against local **0/0** and **578/578**.
All five body comparisons and the exact comparison passed on both artifacts.

The combined configuration still rejected because the inherited boundary made
**85/104** premature stops against the four-stop ceiling. This is a
**content-positive, termination-limited** result. The full-sequence content
and depth arbiter should be retained; the next unit must retrain or re-key
termination on the same memory trajectory. See
[deepseek-198](../../loop/tasks/cpu-training-pilot/deepseek-198/REPORT.md).

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
