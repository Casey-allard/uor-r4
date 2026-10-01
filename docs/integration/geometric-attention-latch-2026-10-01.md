# Geometric attention: learned identity retention across gaps, October 1

This continues the [identity-alignment study](geometric-attention-binding-2026-10-01.md) on the existing Rust native geometric stack and #1512. The successful fixed predecessor carry remains preserved. Its adjacency assumption motivates this experiment; the earlier negative does not retire geometric attention or the Lorentz reader. All new fits use the internal drive.

## Implemented mechanism

The opt-in identity input uses gained normalized read input `u_t`, zero initial state and zero initial predecessor:

```
g_t = sigmoid(W_gate [u_t, u_(t-1)] + b_gate)
Held:  h_t = (1 - g_t) h_(t-1) + g_t u_t
Local: h_t = g_t u_t
q_t = Wq_current u_t + Wq_identity h_(t-1)
k_t = Wk_current u_t + Wk_identity h_(t-1)
```

The read receives the prior state before the current update. Current role/payload information is retained separately from identity. Both modes have identical active parameters and initialization; Held versus Local changes accumulation, not parameter capacity. The scalar gate starts with zero weights and bias -2. Identity projections copy the current q/k projections without additional random draws. Values, NoRead, age, full causal candidate admission and the Lorentz scorer retain their meanings. No candidate mask supplies the source.

This is a learned local retention input to the geometric read, not a newly discovered geometric operator, semantic metric, expert gate or transformer backbone. It implements capture/hold/rebind from an initially zero state; it has no independent CLEAR/reset-to-zero action. The state is not normalized to a unit direction. It remains a floating-point research mode: integer export, grid reference, served mode and the checkpoint envelope reject it until a corresponding representation exists. Direct model save/load binds versioned mode/operation metadata to config and weights and validates tensor identity/shapes.

## Data, controls and measured scope

All arms have 35,591 parameters, width 32, two heads, `rra`, MLP 64, Lorentz read, rotation enabled and no pointer, candidate selector or memory extension. Two initialization/data seeds, 320 requested updates, batches 16, learning rate0.003, no decay and clipping1. Local/Held and objective pairs use the same initial parameters and generated data stream for a given seed.

The grammar is `[BOS, FACT, key, (NOISE, distractor_key)*, VALUE, value, ..., QUERY, key, (NOISE, distractor_key)*, ANSWER]`. Keys and values are independent; values can repeat. Distractors use the same key pool. Write and query gaps are independently drawn uniformly from zero through four noise pairs. Training alternates two/four facts. Only the final ANSWER position has language-loss weight. Right padding occurs strictly after it, with explicit query/source indices; padding does not manufacture compensating gaps.

Context ceiling 128, grammar ceilings of 60 training and 100 stress positions, and vector width 32 are distinct. The retained original and stress draws actually reach 50 and 76 input positions, respectively. Evaluation has 32 base groups and four correlated rows per group: base, changed query, swapped values and reversed order (with each fact's noise block). The original draw uses gaps0..4; a separate postfit draw uses0..8. These are open development diagnostics, not a certified final holdout. Source mass above0.5 is a sufficient unique-top-source condition, not complete source argmax accuracy. Report both heads.

The stress draw includes familiar gaps. Disjoint cohorts use the queried fact's `target_write_gap` and `query_gap`: neither>4, write-only, query-only, both. An unrelated fact's longest gap is not the target's retention distance. Each intervention uses existing saved rows; equal-valued answer pairs cannot demonstrate answer sensitivity.

## Answer-only result

| Mode / seed | Answers /128 | Identity disabled /128 | Stress /128 | Correct-source majorities, heads 0/1 | Distinct-answer query pairs both correct /29 |
|---|---:|---:|---:|---:|---:|
| Local /1 | 65 | 65 | 63 | 0 /0 | 0 |
| Held /1 | 59 | 57 | 60 | 1 /5 | 0 |
| Local /2 | 59 | 58 | 57 | 0 /1 | 0 |
| Held /2 | 59 | 59 | 55 | 0 /0 | 0 |

All four finish320 updates and independently reloaded logits match exactly on the first fixed evaluation episode. Recorded ordinary-answer gradients are finite and nonzero for gate weights/bias and both identity maps. Connectivity is established on those observations; useful capture is not.

Local seed 1 has identical128 argmax predictions after identity ablation, changes none of32 base/query predictions and answers none of29 distinct-answer pairs jointly. Held seed 1 emits a supplied value on127/128 rows but changes only1/32 query-pair predictions. Identity ablation changes25 predictions, losing9 correct rows and gaining7. Thus its channel affects behavior without reliable query binding. Its write/query key gates average approximately0.00317/0.00307, similar to noise; marker/value roles update more. Role means explain a hypothesis, not a prescribed parser-gate acceptance condition or proof of missing identity.

The observed failure is query-to-occurrence binding under this representation/credit/data/dose. It is not evidence that the emitter cannot produce source values, that Lorentz geometry lacks capacity, or that more identical training necessarily repairs the failure.

## Direct occurrence credit through the latch

The measured negative justifies a different causal experiment: keep the four fresh matched arms and add the existing head 0 mean negative log correct-occurrence mass with coefficient1 to ordinary answer loss. Head1 remains unconstrained. Gold source labels shape training through the same read; prediction receives neither sources nor capture-action labels. Pure-answer gradient norms are measured separately from the language component before the joint optimizer update. The earlier no-carry credit negative and successful fixed-carry credit result retain their exact scopes.

The fixed decision is whether direct credit improves source selection and query-sensitive answers. Reliable query-specific source selection without correct answers would motivate reader/emitter coupling. Failure to select a queried source instead motivates inspecting role/identity formation and learned capture, preserving representation and optimization explanations rather than choosing another scorer or unchanged dose. This revisits credit after a causal retained-state change. All four credit arms completed 320 updates and exact reload on the first fixed episode:

| Mode / seed | Answers /128 | Identity disabled /128 | Stress /128 | Correct-source majorities, heads 0/1 | Distinct-answer query pairs both correct /29 |
|---|---:|---:|---:|---:|---:|
| Local /1 | 53 | 50 | 55 | 27 /7 | 0 |
| Held /1 | 45 | 49 | 51 | 30 /0 | 0 |
| Local /2 | 62 | 63 | 56 | 31 /5 | 0 |
| Held /2 | 41 | 42 | 41 | 31 /0 | 0 |

None changes its answer in any of the 32 base/query pairs. Head-zero queried-source mass averages 0.487–0.496 for two facts and 0.243–0.257 for four facts: approximately 1/N value-role allocation. This is not reliable query-specific routing. Row-by-row comparison with answer-only loses/gains respectively 27/15, 30/16, 16/19 and 35/17 correct rows. Higher totals in one arm do not hide regressions or establish addressing.

Local credit gates open roughly 0.99 on both real keys and noise. Held credit gates open roughly 0.01–0.02 on both, while FACT/QUERY markers update more. Identity ablation changes 23/49/12/45 predictions respectively; the channel affects behavior but still lacks association. This is a capture/content learning failure under the tested conditions, not a demonstrated expressivity obstruction or a learned key that simply leaked over a long hold. Do not harden these failed weights.

## Training-only capture-event credit

The next causal experiment gives the existing gate direct CAPTURE/NO_CAPTURE credit with coefficient one, alongside answer and head-zero occurrence losses. CAPTURE labels are the real write/query-key positions; other real positions are NO_CAPTURE. Post-answer padding receives zero weight. Class-balanced stable two-class cross-entropy on `[0, gate_logit]` gives half the mean capture loss plus half the mean no-capture loss. Labels choose training targets, never actual gate values or prediction features. Inference remains predicted soft gates over token inputs only.

Both modes retain the same active parameters. NO_CAPTURE holds state in Held but zeroes the next identity in Local; a Held advantage would establish the value of accumulation with explicitly supervised event learning, not spontaneous capture discovery. Gate classification alone is insufficient: test query-conditioned source selection and answers, both-head observations, identity ablation and longer-gap cohorts. A successful learned capture with uniform routing would instead locate the next question at address formation/use. No new attention metric or unchanged dose sweep is justified.

The existing stable `logits_cross_entropy` is reused rather than Candle's naive sigmoid/log BCE. The public gate-logit getter shares the actual read's preactivation helper, preserving the operator, parameters and format. At zero-initialized gate weights, action loss initially reaches gate parameters; its trunk gradient appears after a gate update. Ordinary answer/source losses already reach the trunk. The sparse pure-answer gradient report remains separately differentiated from the language component.

## Capture-credit result and decision

All four capture arms complete 320 updates. The two Held models solve every original and stress row, whereas the same-parameter Local controls remain query-insensitive:

| Mode / seed | Answers /128 | Identity disabled /128 | Stress /128 | Correct-source majorities, heads 0/1, original | Distinct-answer query pairs both correct /29 |
|---|---:|---:|---:|---:|---:|
| Local /1 | 47 | 49 | 59 | 23 /8 | 0 |
| Held /1 | 128 | 45 | 128 | 128 /0 | 29 |
| Local /2 | 54 | 53 | 49 | 29 /0 | 0 |
| Held /2 | 128 | 49 | 128 | 128 /0 | 29 |

Both Held models also select the correct occurrence with head-zero majority on every stress row and answer both sides of all 27 answer-changing stress query pairs. The three original and five stress equal-answer query groups still select each distinct requested occurrence; this is not only matching a value. Saved original and stress inputs are byte-identical across all three rungs. All twelve models independently reload with exactly matching logits on the first fixed episode; no all-episode logit-equality claim is made.

| Disjoint stress cohort | Rows / related base groups | Held s1 correct | Held s2 correct |
|---|---:|---:|---:|
| Neither target gap >4 | 45 /15 | 45 | 45 |
| Query gap only >4 | 32 /10 | 32 | 32 |
| Target write gap only >4 | 23 /11 | 23 | 23 |
| Both >4 | 28 /9 | 28 | 28 |

A base group can contribute to different write-gap cells because changing the query changes the target fact. These group denominators do not add to 32; the rows are correlated. The compact evidence records the exact subgroup and intervention counts.

Head-zero source minimum/mean is 0.990440/0.999055 and 0.997171/0.999450 on the original draw, then 0.981923/0.998267 and 0.986661/0.998803 on stress for seeds one/two. Head one has no majority claim. Identity ablation loses 83/79 original correct answers. Compared row by row with matching answer-only Held parents, capture credit gains 69/69 original and 68/73 stress rows without losing any previously correct row; against occurrence-credit Held it gains 83/87 original and 77/87 stress rows, again with zero losses. These are retained comparative observations, not broader preservation qualification.

For seed one, original key gates exceed 0.99987; noise-key gates stay below 0.0000471 and noise/value-marker gates below 0.000596. The recurrence unrolling `alpha = g_key * product(1-g_between)` gives conservative original-row input coefficients at least 0.996748 (write-to-value) and 0.997339 (query-to-ANSWER). These are algebraic bounds from logged gate extrema, not measured stress coefficients, exact rounded states or total causal derivatives. Local also classifies the original capture schedule correctly but cannot hold the captured input through subsequent no-capture positions. Functional routing and answer interventions remain the primary evidence.

The successful mechanism is **explicit capture-event credit -> learned capture decisions -> designed retained identity -> query-specific selection by the same Lorentz read -> correct supplied-value answers**. Capture labels are training-only. The result establishes accumulation usefulness under this schedule/alignment, not superiority over all recurrent designs or a matched ordinary scorer. Capture-plus-answer without occurrence credit was not tested, so the necessity of binding loss when capture credit is present remains unresolved. It does not establish spontaneous event discovery, multi-token semantic addressing, natural-language parsing or language ability.

Preserve these successful soft models before lowering them. Next, compare a hard HOLD/CAPTURE transition with the saved soft path; near-binary observed gates motivate the intervention but do not establish its fidelity. Train a hard/relaxed bridge only if needed, then port scales/operations with the existing arithmetic constraints. In parallel, extend learned capture to spans/commit events in the same language-to-memory interface. Bounded candidate/index access remains a separate obligation: this success still scans the full prefix.

An independent architect identified contextual-content contamination and role/identity summation as possible failure causes, recommending a shared local embedding view only if healthy capture still failed routing. The actual Held success resolves that current concern; do not bundle that change, new map tying or a new score into the next experiment. Keep the conditional design for a later demonstrated failure. A drafted retention probe was preserved as `retention-probe-not-run.patch` but not compiled/run or made a prerequisite.

## Integer and geometric continuation boundary

In exact arithmetic, gained RMS inputs obey coordinate bound `|u_j| <= sqrt(d) |gamma_j|`; the zero-initialized convex state retains it. A port needs explicit rounding and accumulator bounds. The existing integer normalizer supplies mantissas and a per-token exponent. Hold must preserve both: interpreting retained bits at the current token's exponent changes the operator. Gate input halves and current/held q/k contributions require scale alignment or separate wide accumulations. Cached keys preserve their write-time identity; previous input advances every token, including holds.

For fixed diagonal gain `Gamma`, scalar gating commutes with gain: `h_gained = Gamma h_ungained`. Storing ungained state requires folding gain into both gate halves and both identity maps, as well as current-role maps. Existing old-map gain folding alone is insufficient. Preserve initial zero, signed orientation and radius; unit normalization or a direction codebook can erase partial capture, cancellation and relative current/identity strength. Exact paired-H4/icosian `Z[phi]` invariants and their error contract remain distinct from this float prototype.

Hard HOLD/WRITE can support copying and reuse of identity projections, but it changes the soft model. Train/measure its hard/relaxed bridge after useful behavior is observed. Intermediate gate use could instead require the existing integer table-sigmoid/product primitives with explicit fidelity and cost. Full-prefix scans and other dense maps remain: neither possibility establishes bounded indexed routing, multiplier-free complete serving or energy savings.

Gated DeltaNet distinguishes global forgetting from targeted associative updates; its matrix-valued memory is not adopted here. The relevant caution is that uniform decay and semantic editing are different operations, and scalar retention is not an exact addressed memory. [Primary paper](https://arxiv.org/html/2412.06464v3).

## Source, execution and preservation

Production source `15de5ac7b361cd234805659ed8ae55b115a672b3`; answer-only example `0947a3dd` (full identity in retained report); credit example `1e7eb6bcd551ba3f1412319cb6eb65eb081fe01d`; capture-credit example `e274835187a5388ec0c9c1c35cb1def9bc2b24f9` (getter source `fb7cc3b4e531f051c4db6063aeb75f4b42e71796`). Six focused executed tests passed: reference recurrence/gradients/causality, matched initialization/exclusivity, metadata reload/refusal, export/grid refusal checkpoint refusal, and live gate-logit/sigmoid equivalence with causal capture gradients and stable extreme-logit credit. Their finite-difference objective contains binding plus0.1 language; the actual fit histories separately record pure-answer gradients. Formatting/diff checks passed. Principal engineer and mathematician reviewed the source and actual first-seed failures.

Retained internal root `/Users/casey.allard/.local/share/uor-r4/research/attention-latch-20261001/` contains distinct claimed/sealed/verified runs, all learned models, row outputs, source/executable identities and compressed exact executables. No sealed attempt is amended. Run1 fit/evaluation/save/reload took664.886 seconds, maximum RSS51,183,616 bytes, two Rayon threads. This unoptimized research executable supplies no serving-speed result. Complete preparation/build/review/delivery is separately charged to the cumulative ledger. Initial focused compile 641 seconds; example build 164 seconds; objective-only rebuild 10.43 seconds; the six-test getter check rebuilt in 17.44 seconds and ran in 0.29 seconds. An example role-observation ownership error was repaired before its successful 10.56-second rebuild, not counted as a model failure.

Direct credit used 709.374 seconds (maximum RSS 51,838,976 bytes), within the original remaining 835-second cap. The two runs total 1,374.260 seconds. The measured capture failure justified a recorded standing-owner extension of cumulative fit/evaluation/save allowance from 1,500 to 2,700 seconds before use, with a 1,200-second cooperative cap for the new four-arm capture experiment; no timer reset. Complete card 120 minutes, disposable cache 6 GiB, new retained allowance 48 MiB,128MiB storage stop margin. Capture credit took 1,015.025 seconds, maximum RSS 52,101,120 bytes. Total twelve-arm fit/evaluation/save/reload cost is 2,389.285 seconds (39 minutes 49 seconds), 3,840 optimizer updates, 1,967,652 unpadded and 2,559,744 padded training positions. It stays within the extended 2,700-second cumulative ceiling. All models, positives and negatives, are retained. This lab removes its clean worktree/cache after protected delivery while preserving unique results and source branches. No external or SSD compute.

The [compact evidence](../evidence/attention-latch-2026-10-01.json) binds per-arm counts, model hashes, actual source/executable identities, histories in retained attempts and executed checks. References #1512. General-language attention, natural language, geometry superiority, integer serving, complete-path energy and frontier capability remain unqualified.
