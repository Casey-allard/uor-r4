# A3 result: integer routing learns admission; the language endpoint still fails

September 24, 2026. References #973 and #820. Continues adopted D7 Milestone A and open protected PR #1387.

## Decision

**Retain A3 as partial hard-admission progress, not a capable language successor.** Direct integer learning changes the served coarse address and raises the correct source's admission from **1/24 to 17/24** on the same new-world development first decisions. Both loaded arms rank the source first on **2/24**, select it through the gate on **0/24**, and produce **0/12 complete correct read-enabled answers**. Whole-development likelihood regresses against corrected A2. The application default is unchanged; Milestone A remains open.

The next integrated mechanism is **one learned candidate-or-NoRead action**, with integer fine-code/relative-energy credit and the same evidence-conditioned language head. The measured failure crosses ranking, gate calibration and realization. Improving only a ranking counter is insufficient. The source-separated development data remains exposed; this result introduces no fresh final acceptance draw.

## What was implemented and executed

The [prospective A3 plan](integrated-attention-a3-plan-2026-09-24.md) reuses A2's causal context representation, exact occurrence records, V4096 tokenizer, pinned natural prose/Rust and paired correction worlds. A fit-only collector replays raw prefixes through `Session`, records the delayed positive Key, every causally committed Key in order, and explicitly annotated different-entity value anchors. Other committed records impose page pressure without receiving fabricated semantic-negative labels. Ninety-six value-edit pairs cover 24 fit entities across styles; none were skipped.

The new offline optimizer edits only dedicated lane-zero context coefficients in the signed-four-bit range. It minimizes a declared combination of integer margins, wrong-entity collisions and actual source insertion/admission pressure. One latent bucket is shared across each entity's fit styles and value variants. Each accepted coordinate edit strictly reduces the objective over **all affected groups**; this is bounded greedy search, with no global-optimum guarantee. Exact source labels remain offline. Serving, artifact envelope version 2 and the bounded page lookup are unchanged.

The accepted coarse bank is checkpointed before later checks and then frozen during eight epochs of the existing joint language fit. Fine lanes, State, empirical relative energy, output and utility gates remain trainable. Each arm consumes **1,124,776 training tokens**; both complete the same logical workload. Source-bound export/reload precedes the first generated answer. Separate-process replay reproduces all eight compared behavior/component files per arm, including every one of the **56 generated rows** across the pair.

The [primary discrete-hashing paper](https://www.cv-foundation.org/openaccess/content_cvpr_2015/papers/Shen_Supervised_Discrete_Hashing_2015_CVPR_paper.pdf) motivates optimizing a discrete decision rather than assuming that a relaxed optimum survives quantization. A3's 120-way shared-row objective is different from that paper's binary classifier problem; no theorem or empirical result is transferred to this model.

## Measured results

All first-decision counts below use the same 24 new authored development prefixes, including both source-value worlds. Complete-answer counts use the retained 12 read-enabled original/changed-source generations per arm. Lower bits/token is better.

| Measure | Corrected A2 C120 | A3 C120 | Corrected A2 2I | A3 2I |
|---|---:|---:|---:|---:|
| Source admitted, first decision | 1/24 | **17/24** | 1/24 | **17/24** |
| Correct source ranked first | 0/24 | 2/24 | 0/24 | 2/24 |
| Correct source selected after gate | 0/24 | 0/24 | 0/24 | 0/24 |
| Complete correct read-enabled answers | 0/12 | 0/12 | 0/12 | 0/12 |
| Development bits/token, reads enabled | 6.854878 | 6.929851 | 6.798124 | 6.909304 |
| Development bits/token, NoRead | 7.155441 | 7.156333 | 7.101787 | 7.136158 |

Hard routing takes 188 accepted edits, changes 174 distinct exported coefficients and lowers its objective from **26,393 to 3,665** in both arms. It uses 2,178 proposals and 4,998,000 latent-bucket evaluations, stopping at the declared candidate-evaluation limit. Fit admission rises from **6/192 to 167/192**. After joint fitting, fit source rank is 39/192 C120 and 35/192 2I; correct fit source selection is zero in both. Fine address lanes have **zero net exported coefficient changes**, despite 4,414 query / 4,414 Key gradient calls in C120 and 4,406 query / 4,409 Key calls in 2I. The floating master trajectory is not retained; subquantization, cancellation and saturation are possible explanations rather than separately measured causes.

The admission improvement preserves A2's one successful new first-decision row and adds sixteen. Retained inherited answer-token probes are mixed: admission rises 12/28 to 13/28 through three gains and **two losses**, both at zero-based answer positions 10 and 11 of `dev:correction:orion-allowed`. Thus the aggregate increase is not full retention. Actual answer-token NLL also has regressions: on those 28 inherited rows, 12 improve and 16 worsen per arm. Every comparison row and generated output is retained in the [review evidence](../evidence/integrated-attention-a3-result-2026-09-24.json).

Example actual loaded first outputs remain `:::::::::::::::::::::::::Qu32000` (C120) and `:::::::::::::::::::::::::u100000` (2I). The two natural prose/Rust continuation endpoints also produce punctuation/code fragments. Changed output under a source edit is not an appropriate consequence or a complete answer. None of these continuations qualifies prose, coding or reasoning.

### Page concentration remains a limitation

Development queries use only coarse codes 47 and 71, twelve cases each. Maximum requested same-code load reaches 87 on fit and 83 on development; actual stored postings remain capped at 64. Development search reports incompleteness on 4/24 cases and fit on 35/192. `search_incomplete` combines index and search-bound conditions, so it does not identify which particular page caused the flag.

The optimizer's `overflow_worlds = 0` means **the annotated positive had fewer than 64 earlier same-code committed keys at insertion**. It does not mean every subsequent posting was retained or that the pages are uncongested. A3 learns useful broad admission on this panel; entity-specific coarse partitioning and long-stream behavior are not established.

## Loaded diagnosis: why admission does not become language

The read-only inspector is separately source-bound. Its exhaustive vocabulary scan and forced-source probes are offline analysis, not serving operations.

1. **Fine ranking is weak but not uniformly indistinguishable.** Fifteen of the seventeen admitted positives have unique relative codes in their actual candidate sets, per arm. Two positives collide with other records. Existing energy coefficients cannot separate identical relative codes; distinct codes alone do not guarantee that the shared unary/pair score can learn a semantic ordering. More geometry dimensions are not yet justified by these counts.
2. **The independent gate suppresses useful ranked sources.** Both C120 correctly ranked rows have gate score `-1`: bias `-1` plus six zero active weights. The corresponding 2I scores are `-1` and `-2`. All four reject the source despite a lower gold-token NLL with that source. The gate sees the previous token, four State codes and candidate token, with no Key/relative-code or occurrence/version features. The fit performs 1,003,290 gate updates per arm, of which at most 1,536 are authored first-answer decisions. C120's breakdown is 743,933 natural and 259,357 authored updates, the latter including prompt/body tokens. This imbalance is observed; it does not by itself prove a unique cause of every learned weight.
3. **The output head is also insufficient.** On all 24 new prefixes, actual and independent NoRead surface MAP is colon. Forcing the correct source lowers first-token NLL on all 24, but yields the target as true surface MAP on only **6/24 C120 and 2/24 2I**. The current bounded greedy binary decoder emits that target on only **1/24 and 0/24**, respectively. Better selection alone therefore cannot qualify this candidate. The difference between MAP and the chosen greedy path must remain visible; a full-vocabulary diagnostic cannot be silently adopted as sparse serving.

Independent mathematics, systems and evidence/language reviews agree on these boundaries. The exact integer gate/collision findings replace the earlier source-only hypotheses in those reviews.

## Next: a single evidence-use decision coupled to realization

Continue the same integrated model and retained data lineage. The next implementation should replace energy ranking followed by a separate scalar gate with a bounded action set

`{NoRead} ∪ {Read(record) for record in the actually admitted page}`.

Use shared low-bit scores over causal query state, relative geometric code, candidate value and explicitly available source descriptors. Retain exact record/occurrence identity for ownership and supervision; do not turn an arbitrary ID into semantic distance. Any new typed descriptor must come from observed text and preserve exact byte/BPE identity. The geometric and ordinary arms receive the same information and access limits.

Train the **exported integer decision**, including competing candidates and NoRead, against source provenance and downstream token utility. Source annotations and future targets are training labels only. Couple that decision to the existing evidence-conditioned output head, with explicit sampling/weight accounting for the rare grounded consequence and for ordinary prose/code preservation. Record integer fine-code and selector changes; a floating loss decrease is insufficient. Any decoder adjustment must have a prospective bounded-access budget and be evaluated on actual complete output.

Before choosing fit settings or a new evaluation draw, retain these decision conditions:

- Improve actual loaded source ranking/selection on the available admitted cases, including value edits; report admission losses, collisions and all inherited rows separately.
- Require an appropriate **uncopied** consequence from changed evidence and an improved complete answer against NoRead. Forced gold-source NLL and copied payloads are intermediate observations.
- If integer learning operates but source identity still cannot be separated, integrate the retained ordered span/role and version descriptors at that measured boundary. Do not extend a learning-rate sweep over a representation collision.
- If correct source selection improves but uncopied output remains degenerate, move the active effort to structured evidence-to-output learning and bounded decoding. Do not keep optimizing retrieval counts.
- Keep the broader Milestone A language/code endpoint and later B–E session, scale and laptop-cost work intact. The known saturated-page/ring-eviction restore defect must be repaired before persistent-session qualification.

This is the next implementation decision, **not an executed A4 result**. It is a consequence of the joined A3 evidence rather than a new architecture survey.

## Identities, verification and cost

| Item | Identity |
|---|---|
| Model build source | `626421e739e9aa4572496d3d9287ace7034e7483` |
| Fit/replay executable SHA-256 | `d2c0c40753f71adfe788d2cb52f568904f57bd510a554e59d6af4d49744a9045` |
| C120 `model.ia2` SHA-256 | `dea0356528763649b8d8212212ac718f33d71e4d64631063b81e78d7728f3faf` |
| 2I `model.ia2` SHA-256 | `53675902636cdbbd5d59bfffc93a9a1a77953bb490ec9018d38778d0ad11c1a6` |
| Inspector source | `438280454fb1e633f85f85808ea17f0b13034e8a` |
| Retained root | `/Users/casey.allard/uor-r4-investigations/integrated-attention-a3-20260924` |

The tokenizer, data hash, both route checkpoints, report manifests, full generation rows, source probes and row-by-row corrected-A2 comparisons are in the [evidence record](../evidence/integrated-attention-a3-result-2026-09-24.json). All six Rust report roots were exclusively claimed, sealed and verified by their producing executables. Both replay comparisons include evaluation, generation, source diagnosis, fit/dev routing, first-loaded generation and component digests. Snapshot parity applies only to the tested short cases, not saturation plus ring eviction or training resume.

Validation: **31 focused integrated-attention tests passed**, 1,129 unrelated tests filtered; the source-bound optimized model and inspector builds passed; both fits, reloads, replays and inspections executed. Formatting, claim-wording and diff checks are recorded in the [cycle closeout](../evidence/integrated-attention-a3-closeout-2026-09-24.json). GitHub queue acknowledgements do not execute these checks.

Each model stores **42,135,302 parameter bytes**. The two complete fit processes take 54.24 and 53.65 seconds including their immediate evaluation; peak measured fit RSS is **501,383,168 bytes**. Across individual generation rows, mean learned-coefficient accesses per generated step range from 35,858 to 36,249.75. These are instrumented logical reads, including repeated accesses; they are not machine-code, wall-time or electrical-energy qualification. Full-vocabulary offline inspection is excluded from served cost and included in cycle accounting.

The prospective local extension, complete-cycle charge, storage measurement and limitations are linked in the [budget](../evidence/integrated-attention-a3-budget-2026-09-24.json) and [closeout](../evidence/integrated-attention-a3-closeout-2026-09-24.json). Unique artifacts, research and worktrees are preserved. No paid compute or cleanup was used.

The serving mechanism remains a finite contextual code and empirical relative-group energy. Coarse learning is algebra-independent here; the matched coarse gains do not demonstrate a 2I advantage. Coefficients are fixed after load, with mutable state/query/records choosing their entries. No physical Hamiltonian flow, complete-path multiplier audit, general-language result, unbounded exact history or energy saving is established.
