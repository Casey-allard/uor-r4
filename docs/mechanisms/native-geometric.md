# Native geometric engine (typed values, version chains, grounded replies)

**Status: PARKED.** Its passes were on question sets the project wrote itself and never reached free prose. Its best score, 437 of 512 on the practice set, fell to 0 of 128 on fresh questions. Its ranking objective used functions the project now bans, and its October kernels were quarantined. The idea of one shared writer lives on in the discrete path. The owner decided on 19 October 2026 to repair the root crate and other legacy crates so they build again; that restores the build, not this mechanism. See [what would reopen it](#what-would-reopen-it).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

This is a small language model written from scratch in Rust. It treats every word as an exact prime-number address and keeps its memory as rotations in four-dimensional geometry. It learned which earlier word or number to copy, add, or stop on, and which version of a changing fact to read, such as the first or the latest locker number. On questions the project wrote itself it answered exactly. It never wrote free prose. Its best score, 437 of 512 on the practice set it had been fitted to, fell to 0 of 128 on fresh questions.

## What it replaces in a transformer

The whole network. This is not a transformer variant. Exact address lookups, version chains and small learned selector tables take the place of attention and feed-forward layers, and a readout of score tables predicts the next piece of text.

![Three panels. Left: a fact edited three times as a chain of boxes, with a selector pointing at initial, previous or current. Middle: a staircase of scores on the practice set, 8 to 437 of 512, ending in a trapdoor to 0 of 128 on fresh rows. Right: a word becoming a prime, then a rotation on a sphere, with the exact sum and the eight zeta phases.](../figures/mechanisms/native-geometric.svg)

*Left, how a changing fact is read by version. Middle, the best score and what happened on fresh questions. Right, how words become exact addresses and rotations.*

## Its chapter in the project's history

- **3 to 8 September (build first).** After weeks of frozen ladders and bookkeeping, the owner ruled that a process which forbids running code produces nothing (#1111) and restored a single Rust model path (#1125). The first source-free native artifact continued prompts from a 35,655,288-byte table without opening the training text (#1112). A "model v0" followed, with typed Stop, Copy and Add selection and a 663-answer regression set (#1140). The project was named UOR-R4 Geometric Language Model (#1174), and an honesty pass withdrew fabricated scorecards (#1198).
- **8 to 17 September.** Exact versioned memory and rules for abstaining. The first shared-core test failed (#1229). Typed attention was exported to exact Boolean tables (#1254), but passes came on constructed panels only (#1264).
- **19 to 25 September.** A falsification sweep found the trained number overstated the shipped artifact by 0.57 bits per byte (#1284). No geometric mechanism had beaten a matched ordinary control (#1371). Continuous loss improved while actual prose scored 0 of 5 (#1398). The next stage of the plan moved to the layered stack ([geometric-stack](./geometric-stack.md)).
- **6 to 10 October (the October line).** The records call it Line B. Cross-state fields over categorical H4 banks raised complete correct replies on a 512-row development panel from 8 to 437 (#2152 to #2162). A single fresh draw scored 0 of 128 (#2164). The rules in D21 and D22 followed: stop a line that does not move its milestone number, but a negative closes a configuration, never a mechanism.
- **12 to 13 October.** The bans on softmax and squashing made the pooled-ranking loss non-compliant (D23, D25), and 15 kernels were quarantined after a hidden dense contraction was found (D29). The discrete path (`crates/uor-r4-discrete`) took over, and it keeps this line's single shared writer.

## How it works

**Addresses and phases (definition).** Each token has a full prime address, and ordered two-prime keys name contexts. Eight fixed zeta-zero phase channels, compiled to 16-bit turns, supply position-like phases.

**Window fold (definition).** A window of recent tokens is folded, in order, into exact H4 coordinates. H4 is the 4-dimensional symmetry of the 600-cell, whose 120 corners are the unit icosians. Sums are exact pairs $a+b\varphi$ with $\varphi^2=\varphi+1$. Products stay exact because

$$(a+b\varphi)(c+d\varphi)=(ac+bd)+(ad+bc+bd)\,\varphi \qquad\text{(proof, by } \varphi^2=\varphi+1).$$

The fold keeps signed chirality and evicts old entries with a bound.

**Readout.** Conditional score tables plus seven gated feature groups, with coefficients rounded to eighths and bounded shift-and-add arithmetic, predict the next piece.

**Selectors on top.** Small learned tables, each fitted on constructed families, choose: which source occurrence to read; whether to write an exact owner-value relation (sixteen versions kept); whether to Copy, Add or Stop on typed numeric literals; how deep to read; and which version intent (initial, previous or current) to follow along a validated chain, abstaining when the chain is truncated. A shared geometric writer emits the answer. Exact occurrence and version identity stay separate from the geometric addresses, and digests are never used as meaning.

**October cross-state form.** Trained 4-bit ("Q4") coefficients over categorical H4 banks (Context, Prefix, Generate), scored with exact integers. The training goal was a pooled-token ranking: for each token, push the correct answer above its strongest wrong rival, using

$$\ell=\operatorname{softplus}\!\Big(\log\frac{m_{\text{rival}}}{m_{\text{gold}}}\Big),$$

with $m$ the score masses. That function is banned. The allowed replacement is the hinge $\max(0,\ \delta-(s_{\text{gold}}-s_{\text{rival}}))$ of [rank-margin](./rank-margin.md).

**Training and serving.** Training used floating point. Serving executes the rounded integer scores. The preserved source is excluded from the current build by `cfg(any())` and the facade in `native_support.rs`.

**Rule check.**

| Rule | Verdict |
|---|---|
| Integer serving ([D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded)) | Partly. Scores are rounded integers with bounded shift-and-add, and an integer realizer ran with no source access (28 replies, 178 actions reproduced). Host tokenizing, fitting and reports use float, and no complete-binary instruction audit is recorded |
| No softmax or squashing ([D23](../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project)) | Broken in the October line. The pooled loss uses softplus, log-sum-exp episode weights and softmax token weights. The September sampler used a float softmax |
| No matrix multiplication ([D30](../integration/DECISIONS.md)) | Unclear for September: its fitting was table search on the host. D29 quarantined the 14 native-context kernels and 1 selected-potential kernel after a hidden dense contraction was found |
| Offline float, with an export gap ([D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)) | Float learning over Q4 coefficients would be legal once the objective is allowed and the module is separately scanned. The 4-bit grid builds in an export gap that was never reported against a float score |

## What has been tried

"Authored" means a panel the project wrote itself. "Exposed" means rows used while fitting, so a pass there is development evidence, not transfer. "Fresh" means rows the model never saw.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 4 Sep | First native model with a fixed geometric readout | 0.13 to 0.23% next-piece accuracy, against 21.5% with geometry off. A learned readout replaced it | negative | [record](../../archive/native_geometric_recovery_973.md) |
| 5 to 7 Sep | Relation writer shortcuts; typed numeric literals; literal selection and binding | Writer comparisons cut from 2,054,052 to 91,884 with identical writes. New numeric answers 12 of 16 but identifier copies slipped (62 to 60 of 62), so the parent was kept. Angular continuation 16 of 16 against 8 of 16 | positive / negative | [admission](../../archive/native_geometric_relation_admission_1139.md), [typed](../../archive/native_geometric_typed_value_973.md), [selection](../../archive/native_geometric_literal_selection_1139.md), [binding](../../archive/native_geometric_literal_binding_1139.md) |
| 8 to 10 Sep | Current-query scope (255 of 255 construction, 48 of 48 open cases); learned initial-or-previous version selector | Passed construction, development and one independent fresh draw, on authored data | positive | [query](../../archive/native_geometric_current_query_1140.md), [version](../../archive/native_geometric_historical_version_973.md) |
| 12 Sep | Fresh draw of learned neighbor-role rules | 2,016 of 2,304. Names with an interior "will" fail systematically (0 of 24 against 24 of 24) | negative | [record](../../archive/native_geometric_independent_neighbor_973.md) |
| 12 Sep | First shared-core experiment | Held-out loss gain 1.425% against more than 2% needed; byte and end-of-text accuracy 2 of 249 to 0 of 249 | negative | [#1229](../../archive/history/github-archive-2026-10-15/markdown/1229-adopt-shared-geometric-core-plan-and-record-failed-first-lea.md) |
| 12 to 13 Sep | Credit assignment; block learning; joint dependent reads; decoder refit | 0 of 8 proposed replacements helped; the joint fit failed transfer; 768 of 768 open development answers passed; 0 of 4 refit cells passed | negative / positive | [credit](../../archive/native_geometric_credit_assignment_973.md), [block](../../archive/native_geometric_block_learning_973.md), [dependent](../../archive/native_geometric_dependent_language_973.md), [refit](../../archive/native_geometric_context_decoder_refit_973.md) |
| 19 Sep | Per-mechanism ablation of the earlier lattice learner, 2,048 positions | Baseline 1.8055 bits per byte. Taking out the JEPA block, second readout or engram cost +0.3743, +0.3377 and +0.2228. Taking out VSA attention, the coarse lattice or the lanes changed it by −0.0007, −0.0066 and −0.0071, so those did not help. Trained 1.2372 against shipped 1.8055 | inconclusive | [#1284](../../archive/history/github-archive-2026-10-15/markdown/1284-stage-1-falsification-sweep-per-mechanism-ablation-attention.md) |
| 21 to 22 Sep | H4 shared transitions against a corrected ordinary table; a typed lexical artifact | H4 186 of 288, 48 of 128, 10 of 64 against the table's 288, 124, 64. Lexical artifact 31 of 31, but on an exposed panel and an interface result only | negative / inconclusive | [transitions](../../archive/integration/shared-transition-result-2026-09-21.md), [lexical](../../archive/integration/transferable-lexical-result-2026-09-22.md) |
| 2 to 4 Oct | Q4 value-source fit, two seeds, 256 exposed rows each; a source-free packed realizer | 479 of 512 against 451 projection and 467 unrestricted parent, with no matched extra-dose control. The realizer reproduced 4 reads, 28 generations and 178 steps with no source access | positive | [fit](../../archive/integration/geometric-value-q4-fit-2026-10-02.md), [realizer](../../archive/integration/native-source-realizer-runtime-2026-10-04.md) |
| 10 Oct | Cross-state field on the 512-row development panel, resumed and rival-aware | Complete correct replies 8, 22, 145, 175, 177 of 512, all earlier successes kept. All rows exposed | positive | [record](../../archive/labs/m2-cross-state-2026-10-10/README.md) |
| 10 Oct | Pooled-token ranking from the 177 model, 256 updates | 177 to 437 of 512 with 260 gains and no losses. All 512 rows were exposed in training, and the objective is now banned | positive (development) | [record](../../archive/labs/m2-pooled-rank-2026-10-10/README.md) |
| 10 Oct | One frozen fresh qualification of the 437 model, 128 unexposed combinations | 0 of 128 complete replies against 52 needed; 124 output-limit stops. Development replay reproduced 437 of 512. Recorded as panel fitting | negative | [record](../../archive/labs/m2-fresh-437-2026-10-10/README.md) |
| 10 Oct | Ordinary-gradient reply recipes on the 8-reply model | Reply-gradient kept 2 of 8; prototype fell from 8 to 4 of 512; joint reached 0 of 512; stratified kept 2 of 8; four more passes from 145 went to 139. All damaged working replies, which triggered the three-negatives pivot (D21) | negative | [record](../../archive/labs/m2-reply-gradient-2026-10-10/README.md) |

**Reading these honestly.** The pattern is the one the project's own rules warn about. Exact selectors, writers and version chains worked on authored panels, and a general language model never followed. Fixed geometry lost to an ordinary table given the same supervision (186 of 288 against 288 of 288). The 437 figure was fitting: the fresh draw reused 15 of the 24 literals the model had been fitted on, and still scored zero. The project's reading is that the practice panel was fit, which is a failure of that configuration and not a refutation of cross-state continuation (D22).

## What has never been tested

- The engine at matched capacity against an ordinary control. It was set aside at 1.68 million parameters.
- Free-running prose from this engine; continuous loss improved, generated prose scored 0 of 5.
- A genuinely fresh qualification of any October model after the exposed-panel critique. The one fresh draw scored 0 of 128, and no successor ran. The replacement 256-row fresh held-out panel (`m2-fresh256`, [`docs/harness/panels/m2-fresh256.json`](../harness/panels/m2-fresh256.json), [`docs/labs/m2-fresh-heldout-panel/README.md`](../labs/m2-fresh-heldout-panel/README.md)) is frozen at `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/` (`fb8c5e4fcd3568cc59be8be3073151295b98069c`, all four files read back byte-identically) with non-author `TEST FITNESS: FIT` for construction/overlap/control/seal; no candidate has been scored (`NOT_RUN`, owner-registered bar `pairs_both_complete_distinct >= 52/128` (`complete >= 104/256`) alongside `>= 256/512` on `m2-reply512`).
- A rank-margin or other allowed objective for the cross-state banks.
- GPU execution of the September engine. Its fitting was recorded as host Rust.
- Whether version-chain selectors transfer to free-form edited facts, not authored wording.
- A D11 instruction audit of the complete native binary.

**Open questions.** Which Line B ideas survive into the discrete path: the Copy/Add/Stop selector, version chains, or the exact-relation writer? Do exact chains need a learned selector at all once the D19 object store holds exact records? Can the cross-state field be rebuilt as a SpiralCore read with a hinge objective and score above zero on fresh rows? What did the 75 development replies (of 512) that still fail do wrong, beyond output limits?

## What would reopen it

1. **A new pre-registration with a `TEST FITNESS: FIT` review**, after the owner lifts the separate hold on this line's training and rental. It replaces the softplus ranking with an allowed hinge and runs on GPU in its own scanned module (D37).
2. **A genuinely unexposed decisive panel**: at least 200 fresh rows or at least 2 seeds (D22), not the 512-row practice panel. The 256-row `m2-fresh256` panel ([`docs/harness/panels/m2-fresh256.json`](../harness/panels/m2-fresh256.json)) is frozen (`inputs.json` `sha256:1de2fc28…`, `labels.json` `sha256:3b5c5477…`; non-author construction/overlap/control/seal `TEST FITNESS: FIT` and byte-identical upload/readback at `fb8c5e4f`). It remains `NOT_RUN` until a compliant candidate is owner-released.
3. **Rebuild and re-qualify the quarantined kernels** without the hidden dense contraction (D29).
4. **Join the line to the shared integer vector writer**, so the language path and the compile-and-emit path use one geometric representation.
5. **A result from the relation-read reranker** ([relation-read](./relation-read.md), D34) that needs exact multi-version identity memory. That is the natural route back for the version-chain selectors.

## Also known as

`native_geometric`; Line B native learner; `r4 geometric`; version-intent chain learner; Copy / Add / Stop typed selector; native H4/Q4 source realizer; cross-state continuation (M2); native pooled-token ranking; the native lattice learner (September ancestor).

## Code

- [`native_geometric/mod.rs`](../../crates/uor-r4-core/src/native_geometric/mod.rs): the preserved September engine (about 63,000 lines in the top-level files, 193,000 including subdirectories). [`native_support.rs`](../../crates/uor-r4-core/src/native_support.rs): the facade that excludes it from the selected build.
- In the same directory: [`relation.rs`](../../crates/uor-r4-core/src/native_geometric/relation.rs) (exact relation writes and reads), [`historical_version_intent.rs`](../../crates/uor-r4-core/src/native_geometric/historical_version_intent.rs) (the version selector), [`operation_transition.rs`](../../crates/uor-r4-core/src/native_geometric/operation_transition.rs) (Stop and Add), [`numeral.rs`](../../crates/uor-r4-core/src/native_geometric/numeral.rs) (digits to one typed value) and [`durable_memory.rs`](../../crates/uor-r4-core/src/native_geometric/durable_memory.rs).
- [`native_geometric_cli.rs`](../../crates/uor-r4-wasm-router/src/native_geometric_cli.rs): the root `r4 geometric` command. The root crate is among those to be repaired.
- October line, all compiled out or quarantined: [`geometric_read.rs`](../../crates/uor-r4-training/src/geometric_read.rs), [`native_geometric_cuda_kernels.rs`](../../crates/uor-r4-training/src/native_geometric_cuda_kernels.rs) and [`geometric_context_q4.rs`](../../crates/uor-r4-integer/src/geometric_context_q4.rs).
- [`native_vector_writer.rs`](../../crates/uor-r4-training/src/native_vector_writer.rs): the shared integer vector writer in the current training crate, which this line and the discrete path both rely on.
- Evidence: [workflow](../../archive/native_geometric_workflow.md), [mechanism map](../../archive/native_geometric_mechanism_map_973.md), [model direction](../../archive/integration/model-direction-2026-09.md), [module split](../../archive/labs/d29-module-split-2026-10-13/README.md).

## Related mechanisms

[prime-zeta-codebook](./prime-zeta-codebook.md): prime addresses and fixed zeta phases first appeared here and were rebuilt as a qualified construction. [relation-read](./relation-read.md): the exact-relation head is the reranker-first successor. [exact-memory](./exact-memory.md): version chains and relation writes are its ancestor. [object-store](./object-store.md): the exact store that takes over durable records. [carrier-credit](./carrier-credit.md): October's unified credit was built inside this line. [legal-construction](./legal-construction.md): protected discrete construction was this line's solver family. [rank-margin](./rank-margin.md): the allowed hinge that replaces the banned ranking. [pointer-key](./pointer-key.md) and [read-binding](./read-binding.md): copy-with-identity ideas that started here. [hopf-observation](./hopf-observation.md): the Hopf map with the lost fibre kept. [discrete-learning](./discrete-learning.md): the live path that keeps the single shared writer. [geometric-stack](./geometric-stack.md): the learned float model that took over after the September ladder. [r4g1-graph](./r4g1-graph.md): the compiled-table approach this replaced.
