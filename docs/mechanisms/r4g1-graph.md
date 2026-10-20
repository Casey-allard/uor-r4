# R4G1 compiled lookup graph (teacher-to-tables)

**Status: PARKED.** Its gains were measured while the teacher's own text was supplied, and they never turned into coherent text the model wrote itself. Building the tables also needs a transformer teacher, which the project now bans. The packed file format and the frozen run-time program stay as a tested, multiplier-free reference. The owner decided on 19 October 2026 to repair the legacy crates around this mechanism so they build again; that restores the build, not the mechanism. See [what would reopen it](#what-would-reopen-it).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

This was the project's first approach, from June to August 2026. Let an ordinary language model, the "teacher", read a lot of text. Record what it prefers in each situation and store those preferences as huge lookup tables. To answer, a small program finds the matching table row and picks the best next word using only whole-number adds and comparisons, with no multiplying. On a held-out set of 72,130 positions its table choice agreed with the target word 29.52% of the time, against 28.12% for the earlier runtime. It never wrote coherent text on its own, and building the tables needs a transformer teacher, which the project now forbids. The file format and the run-time program are kept as a tested reference.

## What it replaces in a transformer

The whole engine, at run time. Instead of running attention and feed-forward layers, the program hashes the last few words into a table address and adds up stored scores. The price is that the knowledge is copied from a transformer, so the lookup tables can be no better than what the teacher showed them.

![Three panels. Left: a teacher's preferences copied into table rows, then one context hashed to a row and scored by additions, with the multiply sign crossed out. Middle: bars for the 72,130-position test: 26.07, 28.12 and 29.52 percent, and a scrambled-label control at 3.17. Right: two points for corpus size, 150 thousand and 500 thousand records.](../figures/mechanisms/r4g1-graph.svg)

*Left, how a teacher becomes a table and how a token is read from it. Middle, the main held-out result and its controls. Right, more text kept helping with no knee.*

## Its chapter in the project's history

This was Phase A of the project, from 10 June to 26 August 2026 (eras 1 to 10 of the project history). Dates below are the days issues or pull requests were opened.

- **June to July.** The first code was a router over sparse 512-dimension prime vectors (#1). The R4G1 file format, certificates and the rule that only serving must be multiply-free and allocation-free followed (#11, #157). This stretch was mostly structure.
- **23 July to 3 August.** The first quality chain started at 0.3 percent accuracy against a 31.7 percent baseline (#65). A metric that read zero had hidden a real loss (#102). A store built with shift-and-add arithmetic and zero multiplies matched and slightly beat a larger earlier one (#243).
- **3 to 12 August.** Geometry was tried as an index. E8 codebooks beat plain sign codes at equal bits (#395). Hopf projection and the zeta-grid routing headline failed (#430, #440). SmolLM2-360M became the pinned teacher (#509).
- **10 to 19 August.** Recompilation and serving. A record-ordering bug turned out to be a major cause of garbage output (#755).
- **19 to 26 August.** A staged plan (S0 to S7) set a truth baseline of 29.52 percent against 28.12 percent (#821). Calibrated abstention, free-running text and planning all ended at their stated limit, and one lane was inert (#946). On 25 to 26 August the maintainers reset the plan: coherent free-running generation had never been shown, so a learned geometric decoder became the goal (#948, #949).
- **Afterwards.** The no-multiply rule became the serving contract for the whole project (D11, see [lut-serving](./lut-serving.md)). Later decisions retired the float and softmax parts of the old compiler from the core, so the graph crates stopped building against it (D36).

## How it works

**Offline (compile).** A teacher transformer (SmolLM2 or GPT-2) runs over a corpus and its preferences are recorded. A compiler packs them into an R4G1 graph, a content-addressed file holding:

- records keyed by hashed context codes (recent-token n-grams and prefix regions);
- a cover of regions induced by entropy-gain splits;
- pre-rounded integer score tables for each region;
- optional extra lanes (skip-mix candidate injection, segment, trajectory);
- certificates comparing the graph with the teacher on held-out positions.

Compilation, certification and every teacher run are offline and unrestricted.

**Online (run).** To pick the next token the program:

1. hashes the recent tokens into a context code;
2. takes the exact-context record, or the best-matching region (nearest match over compact bit signatures by counting differing bits, scored with a shift-and-add dot product in place of a multiply);
3. adds each evidence source's stored integer scores, each counted once, saturating at the limits;
4. breaks ties by a fixed rule and emits the best token.

In symbols (definition), with $E$ the set of evidence sources found and $r_e[t]$ the stored score of token $t$ from source $e$ in fixed-point arithmetic ($v$ is stored as the integer nearest to $v\cdot 2^{16}$):

$$S(t)=\operatorname{sat}\Big(\sum_{e\in E} r_e[t]\Big),\qquad \hat t=\arg\max_t S(t).$$

The frozen kernel contract permits only XOR, AND, OR, shifts, rotates, bit counting, integer add and subtract, comparisons and table reads. There is no float, multiply, divide or allocation after setup ([inference contract](../inference_contract.md)).

**Geometry tried inside this frame.** Prime and zeta-phase routing keys, Hopf S3-to-S2 observation, E8 and 120-root codebooks, H4/icosian frames, max-plus route composition and Furey algebra modules. Only the E8 codebook (better than sign codes at matched bits) and the integer shift-and-add dot product kept measured value. The rest tied or lost, and the Furey modules were never wired into inference.

**Rule check.**

| Rule | Verdict |
|---|---|
| Integer serving ([D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded)) | The frozen kernel is D11's ancestor and is compliant as a kernel. No per-token parameter-access count was recorded, so the sparsity rule (D5-b) is unmeasured |
| No softmax ([D23](../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project)) | Run-time scoring is softmax-free by contract. The offline path used the teacher's probabilities, and the retired compiler and runtime modules carry float and softmax content that may not be restored |
| No matrix multiplication ([D30](../integration/DECISIONS.md)) | The frozen runtime has none. Compiling needs a transformer forward pass, which is banned anywhere, and no transformer teacher or comparator is allowed (D28) |
| Offline float ([D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)) | Float compiling would be legal in principle, but the teacher is separately banned. Nothing here is on the discrete GPU path, and the graph crates no longer build against the current core |

## What has been tried

"Top-1" means the table's first choice matched the target token. "Teacher-forced" means the teacher's own text was supplied as the context.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 23 Jul | Honest test of 16 candidate residual terms | Every candidate degraded held-out scoring (+0.64 to +2.25 bits per token). The fake metric was reverted | negative | [#102](../../archive/history/github-archive-2026-10-15/markdown/0102-main-carries-99-s-fake-interaction-residual-metric-and-80-s.md) |
| 28 Jul | Shift-and-add store with graded signatures | 30.7 / 34.3 top-1 / agreement, 47,476 keys, zero multiplies, against 28.2 / 31.9 for the earlier 134,733-key store | positive | [#243](../../archive/history/github-archive-2026-10-15/markdown/0243-f32-residual-vq-store-mul-free-graded-signature-query-assign.md) |
| 29 Jul | Graph file read through the graph path | 0.0% on held-out while the same bytes read 28.2% through the store path. A decoder-grammar defect, not a format verdict | negative (defect) | [#280](../../archive/history/github-archive-2026-10-15/markdown/0280-r4g1-graph-readout-predicts-0-0-on-held-out-while-the-same-s.md) |
| 1 Aug | Corpus size from 150k to 500k records | No knee. The integer runtime at 500k (33.4 / 37.3) beat the float ceiling at 150k (32.1 / 36.4); about +1.5 to +1.8 points per doubling | positive | [#327](../../archive/history/github-archive-2026-10-15/markdown/0327-decision-adopt-300k-token-corpus-as-next-pinned-era-1-8pp-me.md) |
| 1 Aug | SIMD version of the shift-and-add scan | 3.82 times faster alone (3 needed) but 1.3 to 1.56 times end to end (2 needed). Closed | inconclusive | [#330](../../archive/history/github-archive-2026-10-15/markdown/0330-design-simd-shift-add-dot-scan-contract-legal-serving-speedu.md) |
| 3 to 5 Aug | Phase-clock rows; router anchors; Hopf index | Phase rows 12.5% and 16.7% against 34.7%. Router anchors 7.6% against unigram 2.1% and shuffled 3.1% (true anchors 31.5%). Hopf index: 6.1 times occupancy spread but retrieval 0.0045 against 0.0743 | negative / positive | [#392](../../archive/history/github-archive-2026-10-15/markdown/0392-feat-certify-276-design-p-lane-b-rows-measured-recorded-nega.md), [#428](../../archive/history/github-archive-2026-10-15/markdown/0428-test-certify-421-m-r1-router-anchor-reconnection-positive-si.md), [#430](../../archive/history/github-archive-2026-10-15/markdown/0430-test-router-422-hopf-retrieval-quality-recorded-negative-occ.md) |
| 5 Aug | Band-matched controls for the zeta-grid headline | The 0.89 headline came from storage: full-width content vectors scored 0.8948 | negative | [#440](../../archive/history/github-archive-2026-10-15/markdown/0440-test-router-434-band-matched-controls-zeta-claim-re-attribut.md) |
| 6 Aug | Reconstruction against a scrambled null and a unigram floor | Real tables 0.0148 top-1; unigram floor 0.0636. Graph-only reconstruction sits below a unigram. More regions or a better codebook capped near +0.15 points | negative | [record](../../archive/reconstruction_null_456.md), [#460](../../archive/history/github-archive-2026-10-15/markdown/0460-cover-split-criterion-and-codebook-fit-the-two-capacity-leve.md) |
| 8 Aug | SmolLM2-360M observed on broad text | The ceiling was set by the teacher, not the store; controls recorded alongside | inconclusive | [#509](../../archive/history/github-archive-2026-10-15/markdown/0509-broad-corpus-program-observe-smollm2-360m-on-simple-wiki-doe.md) |
| 12 Aug | Max-plus multi-hop routes | 14.51 bits against a null at 8.57. The transition graph was dominated by self-loops | negative | [#626](../../archive/history/github-archive-2026-10-15/markdown/0626-router-tropical-max-plus-multi-hop-route-composition-p-4-leg.md) |
| 17 Aug | Recompile after fixing record order | Output became grammatical but ignored the prompt | positive | [#755](../../archive/history/github-archive-2026-10-15/markdown/0755-compiler-load-corpus-bytes-runtime-history-token-assume-per.md) |
| 19 Aug | S0 truth baseline, 72,130 held-out positions | 21,293 correct (29.52%) against 20,284 (28.12%) for the earlier runtime, paired difference +13.988 per thousand [11.057, 16.919]. Holds for that exact bundle only; not a general 30% | positive (exact scope) | [record](../../archive/attested_broad_baseline_833.md) |
| 19 to 21 Aug | Segment lane; calibrated abstention; skip-mix lane | Segment ceiling 19.0 per thousand against a 20 floor, 1 of 10 faithful. No calibrator met its gates. Skip-mix followed 41 of 87 pairs against 53 needed (fixed later: +28.45 per thousand, but only on a reference harness, off the serving path) | negative | [segment](../../archive/segment_lane_836_verdict.md), [calibration](../../archive/selective_calibration_837.md), [skip-mix](../../archive/skipmix_lane_897_spotcheck.md), [causal](../../archive/skipmix_endtoend_causal_908.md) |
| 21 Aug | How much of the free-running error a correction could reach | About 1 per thousand (3 of 3,200 steps) against 100 needed. The correction rounds were not launched | negative | [record](../../archive/free_running_reachability_840.md) |
| 22 Aug | Equal-budget test of geometric arms | No geometric advantage under equal budgets | negative | [#929](../../archive/history/github-archive-2026-10-15/markdown/0929-research-845-the-equal-budget-measurement-and-the-no-geometr.md) |
| 25 Aug | Trajectory-signature routing in the exact bundle | Reached in production on 20 of 20 canary positions, changed zero candidate lists and zero tokens | negative (inert) | [record](../../archive/trajectory_routing_946.md) |

**Reading these honestly.** The store worked as a lookup, and more text kept helping. But exact-context lookup dominated what was served, so extra regions and lanes had little room to matter (#460, #946), and the broad-text ceiling came from the teacher (#437). Nothing here shows the model writing coherent text unaided. Several failures were defects of the instrument or the conversion (#280, #102, #440), not of the format.

## What has never been tested

- A graph compiled from a non-transformer source, such as exact tables read out of the native discrete model.
- Free-running coherence of any compiled bundle. The correction rounds were never launched.
- Measured laptop tokens per second or energy of the frozen runtime as a serving engine on today's machine.
- Stages S3 to S7 of the completion plan. The reasoning stage is not established and its final grid was not run.
- Learned (rather than compiled) routing inside the graph.
- A collision screen on a registered real trace bundle. That one is unavailable, which is not a negative.

## What would reopen it

1. **Decided on 19 October 2026.** The owner chose to repair, not archive, `graph-compiler`, `graph-certify`, `api`, `proof-model` and the root crate on the current core ([charter](../../CHARTER.md), section 2). Repair restores the build only; this mechanism stays PARKED.
2. **A compile source that is not a banned transformer teacher**, such as tables read out of the native discrete model. The one-time donor corpus of D41 gives hard targets for training only, and not for graph compiling unless a new decision says so.
3. **A fresh pre-registration with a `TEST FITNESS: FIT` review**, at least two seeds, scored against count-table references on free-running generation, not on teacher-forced agreement (D22, D28).
4. **Adoption of the packed container and frozen kernel as the export container for the served model (milestone M4)**, with the export gap reported (D37).
5. **Any added lane must change served tokens at deployed reachability.** The trajectory lane changed none.

## Also known as

R4G1; TLA / TLA6 / TLA7 runtime; transformerless graph; graph compiler and certifier; the frozen `no_std` kernel; skip-mix lane (RF-31); segment lane; trajectory routing lane; Phase B shift-and-add store; the Geometric Intelligence Programme stages S0 to S7.

## Code

- [`graph-format`](../../crates/uor-r4-graph-format/src/lib.rs): the R4G1 container, two-stage validation and a canonical serializer (11,450 lines, 187 tests). Frozen. [`scoring_semantics.rs`](../../crates/uor-r4-graph-format/src/scoring_semantics.rs): the saturating fixed-point score rules.
- [`graph-runtime`](../../crates/uor-r4-graph-runtime/src/engine.rs): the multiplier-free table kernel `predict_step` (4,946 lines, 53 tests). Frozen.
- [`proof-model`](../../crates/uor-r4-proof-model/src/kani_proofs.rs): the executable proof specification for allocation, bounds and top-k determinism. To be repaired.
- [`graph-compiler`](../../crates/uor-r4-graph-compiler/src) (34,424 lines, 28 stages) and [`graph-certify`](../../crates/uor-r4-graph-certify/src) (25,799 lines): compile and verify graphs against the teacher. Historical; to be repaired.
- [`graph-cli`](../../crates/uor-r4-graph-cli/src) and [`api`](../../crates/uor-r4-api/src): the command-line library and the typed facade. [`model-source`](../../crates/uor-r4-model-source/src): the teacher side, kept as a comparator.
- [`transformerless/runtime.rs`](../../crates/uor-r4-core/src/transformerless/runtime.rs): the original in-core runtime, retired.
- [Inference contract](../inference_contract.md): the normative list of permitted operations.
- Evidence: [S0 baseline](../../archive/attested_broad_baseline_833.md), [baseline audit](../../archive/project_baseline_audit_2026_08_18.md), [matrix-operation census](../../archive/matrix_operation_census.md).

## Related mechanisms

[lut-serving](./lut-serving.md): the later integer runtime that carries the same no-multiply rule to learned stacks. [prime-zeta-codebook](./prime-zeta-codebook.md): prime and zeta routing first appeared here as graph keys. [route-read](./route-read.md): router anchors and the nearest-match tree were this line's early read. [e8-orbit-memory](./e8-orbit-memory.md): E8 codebooks beat sign codes at matched bits here. [hopf-observation](./hopf-observation.md): the Hopf index measured negative here. [native-geometric](./native-geometric.md): the from-scratch engine that replaced compiled tables with learned state. [geometric-stack](./geometric-stack.md): the learned float model that replaced the teacher-compiled graph.
