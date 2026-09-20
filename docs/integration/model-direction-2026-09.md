# UOR-R4 Geometric Language Model: direction and capability assessment

The [principal review after PR #1312](occurrence-reader-review-2026-09-20.md) preserves the verified frozen S attribution and selects **one learned BPE exact-occurrence reader over the frozen S-query-only local baseline**. Removing the individual-history contribution improves aggregate loss on both panels; this supports changing the mechanism, not a theorem that older content is absent. M01 is an offline comparator with a small terminal-position population defect; its gain does not identify length dependence as the useful cause. The reported submillisecond timings are warm cached evaluation-harness timings, not direct serving measurements. No broad S replay is selected.

**Next mechanism:** a bounded exact-occurrence reader with learned candidate-relative geometric selection, attached to frozen E plus the S-query-only local residual. [Execution prompt](deepseek-occurrence-reader-step-2026-09-20.md). Reset alone supplies forgetting, not keyed payload retention; retain it for a later demonstrated maintenance need. Validate continuation for the actual next trainer, not an unused legacy framework. Practical research screens may evolve prospectively with reasons and preserved outcomes; causal correctness and truthful claims remain required.

Owner-adopted shared-core direction, 2026-09-12, extending the September 7
source assessment. Plan adoption adds no model-quality result.
[Current state](current-state.md) owns the retained artifact;
[project-track](project-track.md) owns ordered delivery; live GitHub owns issues.

## Name, purpose and architecture

**UOR-R4 Geometric Language Model** is the product/research name. Technically it
is an **experimental autoregressive geometric state model with exact addressed
memory and learned typed operators**. Cargo package, binary and artifact-schema
names remain unchanged for compatibility. The owner may use “geo-transformer” as an aspiration; it does not establish a
Transformer architecture or a measured replacement capability. Historical R4/Spin models retaining dense
attention/feed-forward layers are still transformer references.

The goal is frontier-level local language capability on a consumer M1-class
laptop with lower energy and wasted computation. It is an objective, not a
capacity, speed or energy claim. Offline Rust training may use floating point,
gradients and matrix multiplication, including uor-matmul. Final serving follows owner-adopted D0-b: bounded <=4-bit additive/shift/lookup
linear maps are permitted, with no multiplier instruction in
the declared numerical kernel and no floating point in served computation. Geometric routing remains preferred; no transformer backbone
is adopted. Deterministic geometric address/page
selection is permitted. Shared typed operators are the present design; expert
gates remain conditional future work requiring a demonstrated need and full
laptop cost. Training duration is secondary to execution quality, latency and
energy, within explicitly authorized cumulative machine resources.

Attention is a contextual access mechanism, not the definition of a Transformer.
Autoregression means generating from earlier observations; it does not imply a
transformer. Offline training arithmetic likewise does not determine serving
architecture. See the original [attention paper](https://arxiv.org/abs/1409.0473)
and [Transformer paper](https://arxiv.org/abs/1706.03762) for this distinction.

## Capability assessment

| Capability | Evidence-supported status | Missing qualification |
|---|---|---|
| Inference | Implemented native artifact/session observe/predict/operator/generate path | Complete product API, current-artifact browser deployment and whole-path performance |
| Attention | Bounded learned geometric source choice, copy attention and causal context/intermediate controls | Broad context/role transfer and scalable multi-fact use; not a complete large attention-stack replacement |
| General prose | Not established in the native model; bounded answers/completion exist | Sustained novel coherent paragraphs, paraphrases and instruction following |
| Reasoning | Bounded selected arithmetic, dependent reads, exact intermediates and conflict/update behavior | Flexible novel compositions, multiple predicates, verification and error recovery |
| Coding | Bounded familiar Rust forms and some historical semantic tests | Novel program construction, meaningful tests and controlled repair/workspace use |
| Memory | Exact bounded versions, eviction survival and session mechanisms | Rich relation/discourse representation and product-scoped durable memory |
| Efficiency | Integer/table execution and scoped zero-allocation/timing results | Quality-matched full-task latency, memory traffic and energy on M1 |
| Studio | Separate reusable UI/worker surface | This accepted native model executing through WASM; other backends do not qualify it |

Relevant positives: retained source context improves fresh complete answers
24/32 to 32/32, with context disabled 24/32; context-sensitive span continuation
is 9/9 fresh versus 6/9 with its source-cue pair disabled. These are authored
familiar-template comparisons, not broad language benchmarks. Source/receipts:
[source context](../native_geometric_source_context_1139.md),
[span context](../native_geometric_span_context_1139.md),
[dependent operations](../native_geometric_dependent_source_1139.md).

## Current architectural decision

Learn recurrent state, contextual access and lexical emission together through one reusable geometric computation. The [canonical plan](project-track.md#current-implementation-sequence--owner-adopted-september-12) owns sequencing and replacement acceptance. The [first-step design](shared-geometric-core-2026-09.md) specifies an isolated H4 finite-table recurrence, bounded geometric reads and byte/EOS branch decisions. Its offline coordinate learner evaluates the same discrete forward path used for generation.

This separates the language-learning question from the accumulated memory repair, which is parked. Training cannot recover distinctions erased by representation or select a record absent from its candidate set. Repeated downstream fitting against a frozen erroneous writer is therefore not the default route to general language. Shared learned parameters, including biases where justified, are allowed; request-specific answer rules and an expanding set of narrow correction heads do not establish a general learner.

The new path must satisfy the owner-adopted D0-b serving-operation contract. The earlier blanket exclusion of lookup/add feature-weight contractions is superseded; a learned prior must still be explicitly identified and tested rather than hidden behind a geometric label. Hard geometric reads are a testable access design, not evidence that they suffice for broad language. Predictive improvement, generated continuations and matched context/state/geometry controls must support any advancement. Exact memory and typed operators are preserved but not connected to the first core experiment. Its use of H4 alone does not establish the roles or benefits of every prime/zeta/paired-H4 mechanism.

## Historical September 7 diagnosis

The following diagnosis and proposed phrase-start slice are retained as dated
evidence. Their artifact pointer, resource snapshot and next action are not current;
[current state](current-state.md) owns the live checkpoint.

PR #1171 candidate bb6b8ba4 fits construction 8/8 from parent 2/8 and reaches
open 6/8 from 2/8, preserving the selected earlier outputs. It fails selection;
321e990f remains retained. Fresh cases and subsequent diagnostics were not run.
Only predecessor-shape codes changed: `A ledger says Pearl Cove holds tesvi`
yields ` says Pearl Cove.\n`. Both candidate starts follow lowercase words;
the learned states tie and the longer start wins. See the
[result](../native_geometric_relation_start_1140.md) and
[evidence](../evidence/native_geometric_relation_start_1140.json).

There are two different limitations. Better construction contrasts can defeat
that particular learned shortcut. But shape-only features also admit true
feature aliases: if both starts are admitted in `the report says quiet river
holds mira`, `says quiet river` and `quiet river` can share every first/next/
predecessor shape, gap, shape-pair and non-singleton feature. Identical features
cannot receive distinct scores from this selector. This is a source-derived
conditional example, **not a new model execution**. More shape-only fitting
cannot resolve the general case.

## Historical proposed phrase-start implementation

Extend the existing phrase-start selector with **bounded ordered lexical and
role context**, reusing exact predecessor records, construction-bound prime
identity lookup, the writer's owner/value context and the existing signed-H4
code/landmark learner. Retain shape as one feature family. Preserve endpoint,
admission, payload, versions and parent behavior while measuring the new choice.
The successful next-word/original-cue pairing is the closest supported donor.
Prime values address learned codes; their numerical magnitude is not a semantic
metric. Start by exposing distinctions already present in retained source state.

Use newly authored construction contrasts: same predecessor shape with different
correct starts, lowercase values, multiword introductions, changed owners,
reversed order, distractors and one word in multiple roles. Keep new names and
payloads distinct from structural cues. Open failures are development evidence;
the reserved fresh panel must not be silently recycled as fit data. Compare
complete generated answers and EOS, exact parent preservation and a context-
disabled control. Inspect available support separately from ranking. A cheap
feature-alias check belongs in the same implementation, not a new proof campaign.

This is the first concrete slice of reusable contextual binding. Next, learn
shared contextual state transitions and lexical emission from bound records and
derived values, so the model constructs new sentences/programs. Do not accumulate
an unlimited collection of question parsers or answer-ending heads. Broad prose
must eventually test actual free generation rather than copied phrases alone.

## Mechanism decisions from all audited families

| Family | Role and reuse decision | Boundary to preserve |
|---|---|---|
| Prime/ordered n-lets/UOR-ADDR | Exact lexical, occurrence, ordered-context and artifact identities | Identity is not semantic distance; preserve order separately from commutative factors |
| Signed H4/Z[phi]/paired icosian representation | Finite ordered state, exact composition and orientation; primary implementation foundation | A 120-state root is lossy; a Galois companion is not independent learned capacity |
| Angular/Hopf/fibers/vector bundles/trigonometry | Typed local charts, retained orientation and finite transport; possible geometric page design | S3 Hopf observation on S2 loses S1 fiber information; no reversible universal S3→S2→S1 chain |
| Fixed zeta phases/prime combinatorics/spin | Structured coordinates and state-sufficiency lessons | No classical RH proof is required for supplied finite phase constants; future-derived spin words are not causal language state |
| SpiralCore | Exact finite signed actions/composition as an operator donor | Define and qualify the state/value/action bridge; 8D and R4/H4 are not interchangeable |
| R4G1/W33 | Packed bytes, bounded execution/planning, immutable pages and DAG sharing | Prior W33 mapping showed no advantage; integrate pages only for measured access scaling |
| Framework/Prism/NAF/GNAF | Typed contracts, values, identities and finite primitives | Ontology declarations are not a semantic learner; some operations are matrix products or proof obligations remain open |
| NEMESIS | Specify carried meaning and exact finite operator correspondence | Audited source does not supply a ready general language learner; preserve provenance/license/claim limits |
| GoldSnnail | State-layout, dynamic-system and finite-program donors | Audited chat/attention/dense recurrent paths do not establish our target capability |
| Softmax/HELM/dense R4 references | Offline losses, curricula, binding and gradient/frame controls | Their prose does not transfer automatically; exact recalled “softmax tree” identity remains unresolved |

The complete source-linked [audit index](architecture-2026-09/README.md),
[mathematics](architecture-2026-09/mathematics.md),
[imports](architecture-2026-09/imports.md) and
[engines](architecture-2026-09/engines.md) retain actual source roles and negative
results. Their dated next actions are superseded by this direction and live plan.
None of the audited imports supplies the missing general learner as a drop-in.

## Language-model development requirements

Develop text/byte coverage and separators; contextual roles/entities/predicates/
negation/time; learned nonlinear state changes; output construction beyond copy;
credit assignment across choices; generalization and calibrated abstention;
consolidation/updates/forgetting; compositional operations and verification;
artifact-compatible streaming/cancellation/errors; and quality-matched scaling.
These responsibilities have explicit ordered issue owners in project-track.

Measure energy and memory traffic **per useful completed task**, with latency,
quality, load/startup, ingestion, retrieval, operator execution, emission and
checkpoint costs separated. Tiny warm-kernel timings are not complete-model
submillisecond claims. Tables can trade arithmetic for memory traffic; avoid
Cartesian-product table growth or assuming every new coordinate adds information.

## Historical September 7 resource snapshot

No new model result accompanied that direction. The recorded cycle used 60.167 s
model and 276.856 s engineering commands. Cumulative model use is
4,387.536/4,410 s, leaving 22.464 s at this snapshot. Refresh the shared ledger
and storage receipt before execution; an issue or new session never resets them.
A complete successor requires an adequate projection/authorization, not blind
retry or a hidden budget increase. Keep source, every parent/negative artifact,
opened/held-out distinction and user material. The model remains pre-alpha;
frontier capability and energy savings remain goals.


**Standing owner authorization (2026-09-06):** necessary local model/time/storage allowance extensions are already authorized. Record the complete projection, reason, increment and updated cumulative limit before using each extension; retain cumulative charges and the 128 MiB storage stop margin. Do not ask the owner to approve the same class of necessary increase again. This authorizes neither destructive deletion nor paid/external compute, and does not require spending unused allowance.
