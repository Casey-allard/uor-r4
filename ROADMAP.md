# UOR-R4 Geometric Language Model — roadmap

**PR #1344, principal reviewed:** retain learned extraction across the tested ordinary forms, exact lexical joins across different BPE tokenizations and dependent copying. Both arms retain **48/48 development,40/40 exposed and24/24 final-composition** answers. Root7 first evaluated the post-selection composition panel; root8 and the principal correction replay it. Vocabulary/forms and all question strings overlap training; unfamiliar-entity and novel-syntax transfer remain unqualified. [Principal review](docs/integration/ordinary-form-binding-review-2026-09-22.md); [independent audit](docs/evidence/ordinary-form-binding-principal-audit-2026-09-22.json).

**Corrections and comparison:** source edits now preserve trailing text and later arguments; structured arithmetic/token bounds and independent decode/update tests are repaired. The separately executed lexical-membership control scores **32/40**, alongside historical token membership38/40, read-cap30/40 and NoRead0/40. [Executed checks](docs/evidence/ordinary-form-binding-principal-checks-2026-09-22.json) own corrected artifact and checkpoint counts. H4 potential weights are learned, but descriptor codes remain initialized: strict code search at96/96 cannot improve. Equal authored-task answers establish a quality tie, not learned-code or cost equivalence. Whole-path D0-b, energy and general language remain unqualified. Live GitHub owns protected delivery status.

**Next complete milestone:** [correction-aware conversation memory over unfamiliar entities](docs/integration/deepseek-scoped-correction-memory-step-2026-09-22.md), with learned assertion/correction/query intent, exact scope/entity/relation/version records, a pinned dependent read view and actual save/reload. Define current/previous/initial semantics, including same-value reassertions and historical downstream reads. Then consume one Q8-derived result through the same state; E/S language prediction, source-separated prose/conversation and executed Rust follow. Structural banks, Hopf fiber, signed H4/Spin and conditional E8/S7/harmonic functions remain tools for witnessed needs.

**Research leadership and autonomy:** Codex owns holistic mathematics/ML/systems review and roadmap revision; DeepSeek owns substantive implementation/diagnostic choices and complete lifecycle delivery. Necessary local extensions remain prospectively authorized, with cumulative debit and storage reserve preserved. No arbitrary short timer or retry quota replaces judgment. Preserve failed attempts and distinguish exact bookkeeping from learned interpretation.








## Capability direction

The [canonical project plan](docs/integration/project-track.md) owns the ordered roadmap and acceptance. Live [#820](https://github.com/UOR-Foundation/uor-r4/issues/820) is the programme tracker; [current-state](docs/integration/current-state.md) owns retained artifacts and changing results. This file is the short navigation view of that plan.

| Order | Responsibility / live issue | Why it belongs here |
|---|---|---|
| 01 | [#1139](https://github.com/UOR-Foundation/uor-r4/issues/1139) — Learn contextual phrase and role binding | Preserve bounded binding evidence; broader role transfer remains open. Current active core research proceeds under #973. |
| 02 | [#1140](https://github.com/UOR-Foundation/uor-r4/issues/1140) — Learn shared state transitions and compositional emission | Reliable binding must become reusable computation and language construction. A growing list of answer-family heads cannot establish a general model. |
| 03 | [#973](https://github.com/UOR-Foundation/uor-r4/issues/973) — Integrate the geometric model and learn general prose | Bounded copy attention and correct short answers do not establish a language model capable of sustained original prose. This issue owns both the integrated native architecture and the missing broad linguistic learner. |
| 04 | [#962](https://github.com/UOR-Foundation/uor-r4/issues/962) — Develop conversation and identity-scoped durable memory | Useful local assistance needs meaning to survive turns, corrections and restarts, with explicit user/project isolation. |
| 05 | [#954](https://github.com/UOR-Foundation/uor-r4/issues/954) — Qualify grounded correctness, conflict handling and abstention | Fluent output must distinguish supported claims from missing or contradictory evidence before it can support reliable reasoning. |
| 06 | [#955](https://github.com/UOR-Foundation/uor-r4/issues/955) — Qualify generalized multi-step reasoning | Executing a familiar arithmetic operator is not flexible reasoning. The model must compose accepted operations and preserve constraints on genuinely changed problems. |
| 07 | [#1088](https://github.com/UOR-Foundation/uor-r4/issues/1088) — Develop executable Rust coding and controlled workspace use | The coding goal requires working programs and repairs in real context, beyond familiar code-shaped responses. |
| 08 | [#963](https://github.com/UOR-Foundation/uor-r4/issues/963) — Scale quality with complete-path M1 latency, energy and memory | The project exists to reduce energy and wasted compute. D0-b multiplier-free low-bit execution is an architectural constraint, not itself evidence of lower energy or useful speed. |
| 09 | [#964](https://github.com/UOR-Foundation/uor-r4/issues/964) — Establish scoped serving, geometry and artifact guarantees | Serving claims need contracts for the operations actually executed, while mathematical proof must remain separate from language capability. |
| 10 | [#1172](https://github.com/UOR-Foundation/uor-r4/issues/1172) — Complete the native capability API and WASM model runtime | One coherent model must expose its actual abilities to applications before Studio integration can be meaningful. |
| 11 | [#1173](https://github.com/UOR-Foundation/uor-r4/issues/1173) — Run the native geometric model in GitHub Pages AI Studio | The final user-facing goal is the already developed Studio running our own local geometric model in the browser. |
| 12 | [#965](https://github.com/UOR-Foundation/uor-r4/issues/965) — Qualify, release and iteratively improve the local model | Alpha requires integrated conversation/memory and coding/reasoning on the delivered model; frontier capability is a longer-term evidence-driven objective. |

Current active research is #973: learned contextual query/source descriptions and directed H4 reads over exact BPE occurrences, against a calibrated recurrence comparator. Subsequent role/scope persistence and optional higher-geometric relation-state comparisons follow the canonical plan, with matched capability/cost evidence before adoption. #1140 is closed at its bounded scope; the other listed capability responsibilities remain open. #1139 retains binding obligations but is not the active-next issue. The numbered order guides implementation and qualification; it does not prevent necessary correctness, integration, cost or interface work when its inputs exist. Each issue explains what, why, source/evidence, implementation, acceptance, dependencies and resources.

Native bounded inference and contextual/copy attention exist; general prose and reasoning, broad coding, frontier quality and full-task energy advantage remain unqualified. See [the capability/direction assessment](docs/integration/model-direction-2026-09.md). Shared contextual transitions/emission must carry us beyond copied-answer patterns; model scaling follows measured quality/cost needs. API/WASM integration precedes claiming that the Pages Studio runs this model.

The owner-requested issue reconciliation inspected all 422 pre-existing issue records, reviewed all 11 open scopes, preserved all 411 closed statuses, refreshed all 11 open issues and added #1172/#1173. No issue was closed as complete. [The full census and prior bodies](docs/integration/issue-reconciliation-2026-09.json) preserve the cleanup history. Metadata census is not a claim to have re-audited every closed experiment.

## Historical roadmap

The [complete previous roadmap](https://github.com/UOR-Foundation/uor-r4/blob/bc03f2d7ffde99608da370808eca542360e54508/ROADMAP.md) retains original release/cloud/hologram/transformerless tracks and the exact beliefs at that revision. [Research records](docs/RESEARCH.md), [the architecture audit](docs/integration/architecture-2026-09/README.md) and [project map](docs/PROJECT_MAP.md) locate their source and outcomes. Their old next steps do not override the current owner-directed plan.
