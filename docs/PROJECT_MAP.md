# UOR-R4 Geometric Language Model — project map

This map connects the current Rust model, product interfaces, research history
and retained evidence. It is navigation, not a new capability assessment or an
instruction to run every historical experiment.

## Read the authorities in this order

| Question | Source | Authority and scope |
| --- | --- | --- |
| What are we building? | [Root README](../README.md), [canonical project plan](integration/project-track.md) | The owner-directed goal, architectural constraints and development sequence. |
| What actually executes now? | [Current state](integration/current-state.md) | Accepted artifact, latest positive/negative results, actual remaining work and resource pointers. An implemented candidate can remain unpromoted. |
| What should the next implementation achieve? | [Model direction](integration/model-direction-2026-09.md), [roadmap](../ROADMAP.md) | Current synthesis and capability milestones; the plan remains the sequencing authority. |
| How should an agent work? | [AGENTS.md](../AGENTS.md), [machine policy](integration/agent-execution-policy.json), [continuation guide](integration/CONTINUE.md) | Rust lifecycle, scoped checks, complete cumulative resource projections, preservation and protected delivery. Owner instructions supersede historical process rules. |
| What supports a claim? | [Research ledger](RESEARCH.md), the named issue record and its [evidence](evidence/) | Artifact/data/operator/control-specific measurements. Compilation, a proof of another component, or a queue acknowledgement is not model quality. |
| Where is the broader research? | [Architecture audit](integration/architecture-2026-09/README.md), [research archive](../research/README.md) | Mechanism discovery with explicit reviewed source and evidence scope. Dated “current” and “next” statements remain historical. |

Live GitHub determines issue/PR delivery status; local source, artifact identity
and exact receipts determine what was built or executed. Read those together
before resuming expensive work. A cached knowledge record or an archived README
does not override the current owner direction.

## Current development path

The [canonical plan](integration/project-track.md#current-implementation-sequence--owner-adopted-september-12) prioritizes the shared trainable geometric language core. The [deep geometric-attention synthesis](integration/geometric-attention-research-2026-09/README.md) connects mathematical foundations, actual source, local research and primary literature with explicit coverage. The current experimental `native_geometric/hamming_refinement/` and `hamming_policy/` paths connect bounded reads, selected exact content, state updates and byte/EOS decisions; the policy is initialized and untrained. The research selects a structured learner with deliberate relation/phase/payload dependencies; its first compatibility boundary is implemented below. The [first-step design](integration/shared-geometric-core-2026-09.md) and `shared_core/` preserve earlier experiments. [Current state](integration/current-state.md) owns the actual outcome and retained `15baec48` identity.

The [learned typed attention result](native_geometric_relational_attention_973.md) and [relational_attention source](../crates/uor-r4-core/src/native_geometric/relational_attention/mod.rs) establish the first compatibility/byte-EOS learning boundary on supplied records. The [dependent query result](native_geometric_dependent_attention_973.md) and [dependent_attention source](../crates/uor-r4-core/src/native_geometric/dependent_attention/mod.rs) learn a query update between two actual reads. The [adaptive loop result](native_geometric_adaptive_attention_973.md) and [adaptive_attention source](../crates/uor-r4-core/src/native_geometric/adaptive_attention/mod.rs) learn content-controlled read/emit scheduling at the typed byte scope. The [grounded text result](native_geometric_text_attention_973.md) and [text_attention source](../crates/uor-r4-core/src/native_geometric/text_attention/mod.rs) add learned byte/EOS and cursor advancement for variable-length supplied spans. Within that text-only module, the query remains fixed. The combined recurrence below addresses dependent multi-source text; joint language learning remains unfinished. None of these results promotes a prose model.

The [recurrent text result](native_geometric_recurrent_text_973.md) and [recurrent_text source](../crates/uor-r4-core/src/native_geometric/recurrent_text/mod.rs) integrate the learned read/update/write primitives under one shared action policy and retain the earlier controls through that same path. The [ordered-state result](native_geometric_ordered_state_973.md) adds signed prefix history, while common primitive learning remains unfinished; byte-XOR address composition is not semantic transport.

The [language relation result](native_geometric_language_relation_973.md) and [language_relation source](../crates/uor-r4-core/src/native_geometric/language_relation/mod.rs) learn shared occurrence selection from raw four-word language inputs and answer text. They reuse signed word-prefix geometry and the recurrent writer; fixed syntax, frozen encoding and equal-performing exact-identity controls limit the claim. The [relative binding result](native_geometric_relative_language_973.md) and [relative_language source](../crates/uor-r4-core/src/native_geometric/relative_language/mod.rs) now learn shared match-order/boundary selection across mixed lengths and new active-component combinations, using the same lexical recurrence. The [dependent language result](native_geometric_dependent_language_973.md) and [dependent_language source](../crates/uor-r4-core/src/native_geometric/dependent_language/mod.rs) connect two actual raw-language reads with a final-output-learned one-word query substitution through the shared recurrent state. The two-question schedule is still explicit. The [shared scheduling result](native_geometric_language_scheduling_973.md) and [scheduling source](../crates/uor-r4-core/src/native_geometric/dependent_language/scheduling.rs) replace fixed second-read dispatch with an output-trained shared Read/Emit/Stop loop across direct and dependent questions. Punctuation and frozen primitives remain explicit. The [shared output-credit result](native_geometric_language_credit_973.md) and [joint learner](../crates/uor-r4-core/src/native_geometric/dependent_language/joint_learning.rs) now train reader/updater rules through actual suffix generation under the frozen scheduler/writer. Four blocks recover prior rules without incremental mixed-credit accuracy gain. The [third-read result](native_geometric_language_depth_973.md) and [depth data](../crates/uor-r4-core/src/native_geometric/dependent_language/depth_data.rs) qualify three-source sequential composition under unchanged parameters with scoped second-update controls. The [completion result](native_geometric_language_completion_973.md) and [completion runtime](../crates/uor-r4-core/src/native_geometric/dependent_language/completion.rs) learn an explicit terminal distinction for unresolved required continuations while preserving successful core traces. Current state owns the next bounded multiword-answer work.

The accumulated memory repair is parked and preserved. Its historical records remain navigation and evidence, not active instructions. Local task notes stay in the established `.uor-handoff/2026-09-12-codex-v7/` handoff, linked from the shared-core worktree; original artifacts and sealed reports stay at their existing paths.

## The active model and interfaces

The accepted native model lives in
[crates/uor-r4-core/src/native_geometric](../crates/uor-r4-core/src/native_geometric).
It implements learned bounded attention/source selection, causal inference,
exact retained and computed values, and selected response generation. General
prose completion, broad reasoning and frontier capability remain unqualified.
The objective is useful consumer-laptop language intelligence with lower
execution energy and compute demand. Offline Rust training may use matmul;
final serving must execute no matrix products or transformer backbone.

| Source | Implemented role and next place to read |
| --- | --- |
| [mod.rs](../crates/uor-r4-core/src/native_geometric/mod.rs), [training.rs](../crates/uor-r4-core/src/native_geometric/training.rs), [runtime.rs](../crates/uor-r4-core/src/native_geometric/runtime.rs) | Model/artifact schema and validation, corpus construction and training, bounded causal session prediction. |
| [memory_runtime.rs](../crates/uor-r4-core/src/native_geometric/memory_runtime.rs), [memory_training/resumable.rs](../crates/uor-r4-core/src/native_geometric/memory_training/resumable.rs) | Retained occurrence memory, geometric read features, offline fitting and resumable construction. Historical `/5` state fits are not automatically the accepted artifact. |
| [typed_routing.rs](../crates/uor-r4-core/src/native_geometric/typed_routing.rs), [value_runtime.rs](../crates/uor-r4-core/src/native_geometric/value_runtime.rs), [value_lexemes.rs](../crates/uor-r4-core/src/native_geometric/value_lexemes.rs), [numeral.rs](../crates/uor-r4-core/src/native_geometric/numeral.rs) | Query/operand provenance, learned Copy/Add/NoOperation selection, exact literal capture, exact selected arithmetic and numeral emission. |
| [source_routing.rs](../crates/uor-r4-core/src/native_geometric/source_routing.rs), [role_read.rs](../crates/uor-r4-core/src/native_geometric/role_read.rs), [joint_admission.rs](../crates/uor-r4-core/src/native_geometric/joint_admission.rs), [literal_refinement.rs](../crates/uor-r4-core/src/native_geometric/literal_refinement.rs) | Signed-H4 source/action scoring, source/NoRead decisions, literal numeric admission and offline literal selection continuation. Exact addresses identify features; their magnitudes are not semantic distances. |
| [word_copy_runtime.rs](../crates/uor-r4-core/src/native_geometric/word_copy_runtime.rs), [source_span.rs](../crates/uor-r4-core/src/native_geometric/source_span.rs) | Exact retained word copying and bounded source-span continuation. Source evidence distinguishes payload availability, admitted support and learned ranking. |
| [word_emission.rs](../crates/uor-r4-core/src/native_geometric/word_emission.rs), [lexical_emission.rs](../crates/uor-r4-core/src/native_geometric/lexical_emission.rs) | Shared signed-H4 byte/EOS and Base/defer selection from exact numeric or completed word/span context; bounded authored emission results, not general prose. |
| [relation.rs](../crates/uor-r4-core/src/native_geometric/relation.rs), [relation_admission.rs](../crates/uor-r4-core/src/native_geometric/relation_admission.rs), [dependent_read.rs](../crates/uor-r4-core/src/native_geometric/dependent_read.rs), [relation_span.rs](../crates/uor-r4-core/src/native_geometric/relation_span.rs) | Bounded learned relation writes/admission, retained versions, dependent reads and exact multiword values. |
| [relation_start.rs](../crates/uor-r4-core/src/native_geometric/relation_start.rs), [relation_start_training.rs](../crates/uor-r4-core/src/native_geometric/relation_start_training.rs) | Implemented candidate phrase-start learner. The current-state record identifies its transfer negative and retains the preceding model; source existence is not promotion. |
| [response_entry_runtime.rs](../crates/uor-r4-core/src/native_geometric/response_entry_runtime.rs), [response_runtime.rs](../crates/uor-r4-core/src/native_geometric/response_runtime.rs), [completion_runtime.rs](../crates/uor-r4-core/src/native_geometric/completion_runtime.rs), [snapshot.rs](../crates/uor-r4-core/src/native_geometric/snapshot.rs) | Causal response coordination, bounded completion, observation-only commitment and checkpoint restoration. |
| [Native CLI](../src/native_geometric_cli.rs), [native service](../src/native_geometric_service.rs), [API re-export](../crates/uor-r4-api/src/lib.rs) | `r4 geometric`, local model/session service and the same core exposed as a library. Existing historical graph APIs in this crate are separate interfaces. |
| [Probe example](../crates/uor-r4-core/examples/native_geometric_value_probe.rs), [probe modules](../crates/uor-r4-core/examples/native_geometric_value_probe/), [actual-artifact allocation checks](../crates/uor-r4-core/tests/native_geometric_allocations.rs) | Existing small fit/evaluation drivers and selected causal/allocation checks. Use the record's exact commands and artifact, not an indiscriminate full campaign. |

Use the [native workflow](native_geometric_workflow.md) for command examples and
[configuration](CONFIGURATION.md) for declared bounds. Each component's
`*_training.rs`, `*_snapshot.rs` and focused tests stay beside its runtime.
Source changes require compiling and exercising the changed path; documentation
navigation changes do not imply another model run.

## Geometry, imported mechanisms and preserved engines

The [source inventory](integration/architecture-2026-09/source-inventory.json)
binds reviewed source entries. The [mathematics supplement](integration/architecture-2026-09/mathematics.md),
[dependency/contributor supplement](integration/architecture-2026-09/imports.md)
and [engine/history supplement](integration/architecture-2026-09/engines.md)
explain implemented operations, useful reuse, negative results and missing bridges.
They do not claim every line or theorem in the combined archives was verified.

| Family | Source navigation | Role and boundary |
| --- | --- | --- |
| Prime routes, ordered n-lets, fixed zeta phases, signed geometric transport | [prime_route_attention.rs](../crates/uor-r4-core/src/prime_route_attention.rs), [prime_route_geometric_attention.rs](../crates/uor-r4-core/src/prime_route_geometric_attention.rs), [SpiralCore operator](../crates/uor-r4-core/src/spiralcore_operator.rs), [native mechanism map](native_geometric_mechanism_map_973.md) | Reusable exact state/address/transport and bounded attention mechanisms. Preserve order, chirality, polarity, fiber and declared frames; geometry naming alone is not predictive advantage. |
| E8/paired H4, exact `Z[phi]`, Hopf/fibers, trigonometry, vector calculus and RH work | [Mathematics audit](integration/architecture-2026-09/mathematics.md), [Prime Analysis](../research/prime-analysis/README.md), [Riemann/Lean archive](../research/riemann-lean/README.md) | Locate concrete code, proof obligations and hypotheses separately. Hopf S3→S2 observation retains an S1 fiber; projected coordinates are not a lossless replacement for full state. Fixed zeta constants do not require a proof of classical RH. |
| Original angular/prime and train-soft/infer-hard experiments | [AI-Research](../research/ai-research/README.md), [router-research](../research/ai-research/ai-router/router-research/README.md), [archived sandboxes](../research/archives/README.md) | Preserve partition, chart, transport, locality and training lessons. Earlier transformer/MoE/dense heads remain historical experiments, not the serving target. Duplicate snapshots are not independent replications. |
| `uor-addr`, UOR-Framework, `uor-prism`, `uor-matmul` | [Cargo.toml](../Cargo.toml), [Cargo.lock](../Cargo.lock), [dependency audit](integration/uor-source-audit.md), [import decisions](integration/architecture-2026-09/imports.md) | Active versions are the manifest/lock pins, not legacy `uor_standards/` copies. Identity and finite typed operators are reusable. A table implementation of a mathematical matrix product remains excluded from serving. |
| NEMESIS (`n3mesis`), W33 (`w33`), GoldSnnail (owner's “goldworm”), GNAF and contributor repositories | [Contributor decisions](integration/architecture-2026-09/imports.md), [external survey](integration/external-research-audit.md), [NEMESIS/W33](integration/nemesis-w33-relevance.md), [ecosystem follow-up](integration/afflom-ecosystem-followup.md), [source catalog](integration/afflom-ecosystem-sources.json), [GNAF import provenance](gnaf_import_provenance.md) | Cached source/pin/license and review limits are explicit. Reuse finite transition, exact state, immutable page/DAG and typed witness ideas when the current model needs them. No entire contributor repository is silently a working replacement LM. |
| Exploratory Rust router | [uor-r4-router](../crates/uor-r4-router/README.md) | Floating-point retrieval, Markov and dashboard mechanisms; distinct from the accepted native core. |
| Historical TLA/R4G1 compiler and runtime | [Core history](../crates/uor-r4-core/README.md), [graph compiler](../crates/uor-r4-graph-compiler/README.md), [graph format](../crates/uor-r4-graph-format/README.md), [graph runtime](../crates/uor-r4-graph-runtime/src/), [model ledger](../model/ledger.toml) | Preserve packed storage, deterministic identities, bounded operators and recorded limitations. Their no-multiply kernel and graph compilation do not establish native model language quality. |
| Dense/softmax references and older learned language tracks | [Historical trainer](../tools/r4-softmax-trainer/README.md), [model lifecycle](MODEL_LIFECYCLE.md), [engine/history audit](integration/architecture-2026-09/engines.md) | Reuse curricula, binding losses, numerical checks and preserved positives/negatives. Python/dense references do not become product dependencies. The user's “softmax tree” identification remains qualified by the audit. |
| Proof, conformance and wire compatibility | [Proof model](../crates/uor-r4-proof-model/README.md), [historical suites](../features/suites/README.md), [OpenAI profile](../profiles/openai/README.md), [formal vocabulary](formal_vocabulary.md) | Scoped obligations and protocols, not proofs of general reasoning or endpoint quality. Protected queue acknowledgements are not execution of these checks. |

## Where work and evidence are stored

| Location | What to preserve and how to recover it |
| --- | --- |
| [research/](../research/README.md) | Tracked research snapshots and original attribution. Some large media/data are Git LFS objects; a pointer file does not establish local payload availability. The archive index names retained history refs. |
| [docs/RESEARCH.md](RESEARCH.md), issue-numbered records, [docs/evidence/](evidence/) | Append-only measured results, exact input/artifact bindings, controls, negatives and `NOT_RUN`/`UNAVAILABLE` boundaries. Do not overwrite historical outcomes to simplify the story. |
| Repository-local ignored `.uor-models/` | Bulk corpus/model artifacts, retained parents/candidates, training/evaluation inputs, reports and command/resource receipts. These are local material, not guaranteed by cloning Git. Current-state and its evidence records identify exact paths/CIDs. |
| Current local native store: `/Users/casey.allard/uor-r4/.uor-models/native-typed-value-2026-09-05/` | Retained native lineage and bounded model-development receipts. This is the recorded host path, not a portable default; verify it on another host. |
| Current local cumulative ledger: `/Users/casey.allard/uor-r4/.uor-models/native-joint-learning-2026-09-04/model-time.json` | Shared consumed/remaining model allowance across fits, preparation, evaluation, retries and resumes. Refresh this ledger and storage before projecting the next complete execution; do not reset it because a new issue/task starts. |
| [Local project knowledge package](../tools/uor-knowledge/README.md) | SQLite source snapshots and relationships, normally `~/.local/share/uor-r4/knowledge/knowledge.sqlite3`. Sources have revisions, visibility and evidence status; retrieved text is evidence, not instructions. The index does not replace or author the model's memory/response. |
| Contributor audit caches named in the [import supplement](integration/architecture-2026-09/imports.md) | Source snapshots can be outside Git under the local knowledge audit directory or temporary extraction paths. Use recorded upstream revisions and provenance when a cache is absent; do not infer exhaustive live access. |
| Worktrees and Cargo targets | Keep user source edits, research and every unique result. Cargo output may be recreatable, but cleanup is a separate reviewed preservation decision. A warm cache and available disk must be checked, not assumed. |

Resource projection covers preparation, builds, fitting, controls, generated
evaluation, retries, artifacts, RAM and retained storage before execution.
Training time tolerance does not eliminate cumulative limits. Report serving
latency with the measured boundary; isolated warm steps are not whole-request
latency or an energy measurement.

## Studio and eventual browser deployment

The companion [uor-r4-wasm-chat repository](https://github.com/Casey-allard/uor-r4-wasm-chat)
contains the separately developed Studio surface. The previous source audit
records its local checkout as `/Users/casey.allard/Downloads/uor-r4-project`;
verify its current state before work there. See the
[engine/Studio audit](integration/architecture-2026-09/engines.md) for its
inspected revision and limits.

The intended order is useful language/coding/reasoning through one complete
native capability API, then the same accepted artifact executing in a WASM
worker behind the GitHub Pages Studio. Pages serves static assets. Other model
backends, reference responses, UI readiness and mocked output cannot establish
that the accepted native artifact runs in the browser. Carry artifact/backend
identity, actual generated bytes, context/state behavior and measured resources
through that integration.

## README coverage and maintenance

The [README inventory](integration/readme-inventory-2026-09.json) enumerates all
127 tracked paths whose basename contains `readme` case-insensitively at its
recorded source revision, including `README_HANDOFF.md` and `Download README.md`.
Each entry records the source Git blob, classification and disposition without
duplicating its body. First-party front doors receive current navigation;
historical/imported/vendor READMEs preserve provenance and are reached through
the current archive/audit maps. This is a complete path inventory, not a claim
that all 127 documents or their linked repositories were audited line by line.

Update the living plan/current-state and source map when the implementation
changes. Keep original package/import names intact. A historical graph/dense
command can remain documented while clearly scoped to its original lane; it
must not become the newcomer quickstart for the native model.
