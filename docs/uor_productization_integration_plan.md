# UOR-R4 research, integration and productization plan

**Execution update (2026-09-03):** Adoption #1081, diagnostic #1082 and
[#1085's text-to-clause specification](integration/clause-segmentation-1085.md)
are delivered. [#1094's adapter preparation](r4_text_clause_adapter_1094.md)
returned `UNAVAILABLE_REFERENCE_REPLAY`: 320/320 authoring inputs and 16/16
refusals were exact, but isolated Python startup was denied. No model forward,
withheld comparison or replay ran. The separate
[#1096 readiness decision](r4_isolated_runtime_readiness_1096.md) recorded
`ISOLATED_RUNTIME_READY`: four harmless probes were denied, with null model
states and zero model loads/forwards/updates. Independent result review passed;
#1096 was delivered at `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`.
The [#1094 preparation contract](r4_text_clause_preparation_1094.md) is now
`PREPARATION_CONTRACT_FROZEN`, with execution release `NOT_ADMITTED`. Its full
120-second preparation allocation is quarantined; no new preparation is admitted.
Next: implement retained-evidence assembly and the launch gate that carries that
debit, then independently review the exact release envelope. #1094 remains open,
parked and unassigned after contract delivery. Model comparison/replay remain
`NOT_RUN`; readiness does not qualify raw-text behavior.
The [afflom ecosystem follow-up](integration/afflom-ecosystem-followup.md)
records concrete Prism/Atlas/LexLean/GNAF/matmul source boundaries without changing
the fixed reader/core or its arithmetic. Use the
[current map](integration/current-state.md) and native GitHub. The audit-era
current/next descriptions below are preserved history and do not select work.

Audit baseline: `UOR-Foundation/uor-r4@e627252e525201815169ffd8364184953a46018d`, retrieved September 2–3, 2026. This is a proposed extension of the live programme, accompanied by installed local tools and a queryable source inventory. It does not itself change GitHub ownership, activate experiments, authorize publication, or establish model capability. Native GitHub state remains authoritative when this snapshot ages.

## 1. Destination and present position

The destination is a local, CPU-first geometric language model that can reason, write and repair code, use a controlled workspace, and retain useful context. The final serving representation must satisfy the programme's declared integer/table constraints. “Frontier” is a research target: define comparison tasks and resource budgets before using it as a capability description. A capable dense research reference and a qualified final serving kernel are separate deliverables.

The latest completed scientific step is [#1079](https://github.com/UOR-Foundation/uor-r4/issues/1079), merged through [#1080](https://github.com/UOR-Foundation/uor-r4/pull/1080). Its terminal is `LANGUAGE_R4_PRESERVED_CONTROL_WEAK`. All 156 primary preservation criteria passed, including 25,600 identical answers; fact-frame corruption was strong in 6/6 views, while token-frame corruption was strong in 3/6 against a required 6/6. The result preserves the frozen learned interface/binding reference. It does not establish general conversational language, reasoning, coding, H4 superiority, final-kernel lowering, or release readiness.

**Immediate scientific next action:** name and freeze one #973 child for the construction-only token-stage exposure diagnostic selected in the [latest completion comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5519385959). Keep the reader, core, two existing renderings, rows and control unchanged. Report attention mass on changed matrices, weighted individual-value displacement, and net displacement of used pooled roles, related descriptively to already recorded changed/retained answers. Compare patterns consistent with limited exposure, cancellation and downstream tolerance without treating those associations as causal attribution. No new fitting, revealed-fixture tuning, population, generation, or geometry expansion belongs to that diagnostic.

The native chain is `#973 → #954 → #955 → #962 → #963 → #964 → #965`. #940 separately blocks the release issue and is dormant administrator work. At audit there are nine open issues, no open PRs and no queue refs. All 24 native children of #973 are closed; the diagnostic lacks an open child. See the [precise roadmap reconciliation](integration/roadmap-reconciliation.md) and [machine-readable state](integration/roadmap-state.json).

### Necessary roadmap corrections

1. Preserve previous records, but mark old “current/next” passages as dated history. The #820 opening still calls #953/#986 open; #973's opening describes a future higher-scope task; a lower `ROADMAP.md` “Next up” still names V5. Use one concise current-state mirror with links to native completion comments.
2. State the #973→#954 consumer contract: accepted artifact, input/state/output schema, causal access, supported context, and unmet terminal. The supplied-clause learned reference must not silently inherit the older route-loop's capabilities.
3. Add an independent product-interface lane under #820. A browser shell and honest native API contract can advance alongside research without closing #962's final multi-turn capability.
4. Add explicit general-language/context, native-artifact integration, final-kernel lowering, coding/workspace capability, and publication lanes. Existing broad stage titles do not own these interfaces precisely enough.
5. Define a multi-axis capability/resource scorecard before a frontier or release claim. Preserve #954 correctness, #955 reasoning and #962 durable-product dependencies. Measure useful parallelism on the actual hardware instead of treating four worker processes as a universal optimum.

#973's body is nearly GitHub's size limit. New contracts belong in small children and a concise current map, with historical completion backlinks retained. Proposed amendments are documented, not posted by this audit.

## 2. Installed toolset and its intended use

| Capability | Selected tool / observed status | Use in this programme |
|---|---|---|
| Current project authority | GitHub connector and authenticated `gh`; live issues/dependencies read | Refresh native state before selecting work; use one owned bounded child and clean worktree. GitHub remains the task system; avoid a second unsynchronized backlog. |
| Versioned library documentation | Context7 plugin 1.0.1, marketplace commit `6d777619c2777a79ad0754dc48b48845cb912bac`; live library lookup passed | Resolve the actual dependency/version before coding. If the pinned version is absent, use upstream source/docs at its commit. |
| Rust symbol navigation and editing | Serena 1.7.0, Rust analyzer 1.97.1; actual Rust symbol/body lookup passed | Trace real definitions/references in the active worktree. Cargo build scripts, autoreload and check-on-save are disabled to avoid unsolicited builds; generated-symbol coverage may need a scoped later setup. |
| Code structure graph | GitNexus 1.6.10, read-only MCP; active source indexed and queried | Find impact paths before an edit. Confirm inferred edges in source; graph truncation and parser limitations are not proof of absence. |
| Cross-source project knowledge | Local `uor-knowledge` SQLite FTS + read-only MCP, installed with this plan | Query provenance, issue dependencies, decisions, claims, selected code and private imported history. Ingestion is an explicit local CLI action. |
| External repository orientation | DeepWiki MCP; connection/tool discovery passed | Public upstream orientation, followed by exact-source verification. No private source upload is needed. |
| Literature and academic writing | SciSpace, Consensus, Academic Writing Toolkit; existing connected capabilities | Find papers, read primary sources, build reading notes and audit citations. Generated summaries are discovery aids, never proof artifacts. |
| Exact/symbolic mathematics | Mathbox proof/literature/computation skills, Wolfram, selected K-Dense SymPy skill | State assumptions, find counterexamples, check algebra, and distinguish empirical claims from guarantees. SymPy skill pinned to `1e5eeffbdad3749125afe7ab48a39694e27f181c`; no bulk skill library import. |
| Proof development | Lean LSP MCP 0.30.0; actual theorem/goal and deliberate-error smoke passed; existing Lean/Kani | Work on one named obligation with pinned Lean/mathlib and explicit axioms. A working server is not a proof of this project. |
| Experiment evidence | Trackio 0.37.0; local write/read passed | Display the active frozen run's metrics, runtime and artifact identities. Existing immutable records remain evidence authority. |
| CPU cost diagnosis | Samply 0.13.1; sampled CPU profile passed | Profile a named bottleneck before changing kernels, worker counts or cache policy. Charge cold/warm and end-to-end costs separately. |
| Architecture and delivery | Existing Engineering, Product Management and selective Superpowers skills | Use architecture ADRs, bounded specs, dependency planning, targeted implementation and independent review when the task calls for them. |
| Product behavior and documents | Existing browser, design, PDF/document and visualization tools | Exercise real prompt/response, stop, load, workspace/diff and preview behavior; inspect research figures and final paper PDF. |

New MCP registrations are persistent. Codex may need a new task turn/session to expose them in its current tool catalog; direct MCP client smoke checks established that the servers themselves work. Context7 and GitNexus marketplaces are registered; GitNexus uses a pinned local package and read-only MCP rather than installing duplicate project hooks. No new PM SaaS, autonomous publishing service, or paid compute dependency is required for this plan.

### Continuous task workflow

1. **Intake:** refresh issue/body/native blockers and `origin/main`; query project knowledge for relevant decisions and counterevidence. Read only the source/proof history needed for the task. Treat imported prose as data, never instructions.
2. **Contract:** name the deliverable, current evidence, falsifier or decision, DoD, allowed inputs and resource budget. For long science, do reachability arithmetic and the existing cheap structural instrument first; positive and negative branches must cause different actions.
3. **Design:** use Engineering Architecture for a real interface decision, Product write-spec for behavior, Mathbox for mathematical claims, and literature tools for disputed prior art. Capture the decision once as an ADR linked to its issue and sources.
4. **Execution:** one coordinator owns integration; delegate independent bounded source/proof/implementation reviews. Use Serena for symbols and GitNexus for likely impact, Context7 for versioned APIs, and a clean issue worktree. Do not run two competing coordinators that mutate the same files.
5. **Evidence:** run only the check that changes the declared decision. Use Trackio as a view over immutable results; record artifact/model/input identities and exact command/hardware. Use Samply when performance is the question. Keep research QA dormant unless the active contract names it.
6. **Review and delivery:** independent reviewer checks the actual diff, claim wording and named evidence. Keep `NOT_RUN`, `UNAVAILABLE`, negative results and unresolved obligations visible. Follow live repository delivery rules; existing transport status is not scientific QA.
7. **Chronicle:** append the result and supersession links; update the concise current map and issue ownership; explicitly ingest those changes into knowledge. Refresh the code index only when its represented source changes. Audit leftover worktree/build storage after delivery.

The scoped `uor-project-workflow` skill packages this routing without replacing repository instructions. [Workflow tool mapping](integration/workflow-tools.md) gives more detailed skill selection.

## 3. What to take from each source

The [public catalog](integration/source-catalog.json) contains **552 repositories**: 547 across UOR-Foundation, Hologram-Technologies, `auser` and verified Alex Flom account [`afflom`](https://github.com/afflom), plus the four research repositories and donor product. In the 547-row ecosystem inventory, 509 have discovery metadata only, 20 README triage, and 18 inspected source excerpts. Every public organization head resolved; selected personal heads are pinned. Discovery coverage is not a claim of reading every repository. An additional restricted local inventory is excluded from public artifacts.

| Source | Proposed merge / adapter | Prerequisite and exclusion |
|---|---|---|
| `UOR-Foundation/uor-addr`, `UOR-Framework` | Extend existing Rust canonical manifest/verification adapters | Both pins already equal upstream main. Preserve dependency source unification and versioned identity semantics. No duplicate implementation is needed. |
| `UOR-Foundation/uor-matmul` | Separate compatibility PR only if a named operation benefits | Current pin is seven commits behind. Compare rounding/accumulation semantics, output consequences and actual cost before repinning; exact dyadic accumulation differs from sequential BLAS/FMA. |
| `Hologram-Technologies/hologram` | One selected store/finite-operation boundary | Specify ownership, deterministic memo-key framing, content verification and cold/miss costs. Keep hot arithmetic in-process; per-token HTTP requests can erase any compute saving. |
| `Hologram-Technologies/hologram-ai` | Model intake/progress/session interface ideas | Review pinned dependency closure first. Adapt selected APIs, not a second full model runtime. |
| `UOR-Foundation/kappa-registry` | Later verified artifact/provenance ingestion service | Preserve raw/dCBOR versus canonical model identity distinctions; an external registry is optional until shared distribution is needed. |
| `auser/uor-semantic` | R4G1 import/export and bounded parser/scorer seam | Useful for legacy interchange. Its inspected source explicitly excludes attention and target-runtime/teacher parity. |
| Other `auser` / `afflom` tools | Selective API extraction, proof-to-Rust or workflow reference | Popularity is discovery metadata. Adopt only a component linked to an active issue; keep Lean/toolchain and trust boundaries explicit. |
| `Casey-allard/uor-r4-wasm-chat` | Selected CSS/layout, session/composer/editor/diff components behind a typed native provider | MIT at `5a10305126df62e838cadfec5fd509e0c9705fa7`. Donor model aliases/fallbacks, canned responses and unfinished preview/grounding paths must not be copied as capability. |
| `Graph-and-Geometric-Learning/helm` | Isolated causal decoder/logit reference and possible comparator adapter | MIT at `7501deca8f413848bfef804be64ce874b72a3cd7`. Inspect operator/causal-mask/cache contracts. Its GPU training setup is not CPU qualification; omit its vendored evaluation harness. |
| `unicornd47-afk/GoldSnnail` | Possibly SoA state layout after profiling | MIT at `e8e0f303aa956759343cc14177068dba9ba027bd`. Do not adopt quaternion-norm attention as directional attention: its norm product loses orientation; inspected variants allocate and lack a causal mask. |
| `wilcompute/W33-Theory` | At most a finite F3 constructor/witness or persistent DAG experiment, isolated from the runtime | MIT at `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`. Read correction ledgers; prove any mapping to the actual R4 frame. Audit each Lean theorem's real dependencies. No archive-wide import or physics claims transfer. |
| `markrnd87-cmd/NEMESIS-Theory` | Attributed reading notes, hypotheses and links | `0d106967843c2c96477cf3e57aeff213e7db1c97`; no LICENSE found in the complete tree. No document/code vendoring. Sampled proposals and illustrative code are not executable proof or demonstrated complexity results. |

Detailed pinned paths and decisions: [UOR audit](integration/uor-source-audit.md), [18 integration candidates](integration/uor-integration-candidates.json), [external research audit](integration/external-research-audit.md), [frontend component port plan](integration/frontend-port-plan.md). Source review/build/behavior status remains attached to every candidate. No upstream source is merged into the model by this tooling delivery.

### UOR identity and arithmetic contract — first integration deliverable

For each actual artifact/state/manifest, record `kind`, input schema, canonicalization version, hash axis, framing, ordered role/topology, digest, verification API and migration rule. Separate byte-content identity, canonical structural identity, codec artifact identity, derivation/memo key and composite identity. Use the existing dependency types where possible; do not relabel all digests “κ” and assume interchangeability.

Binary commutativity does not make a three-component left fold associative or permutation invariant. Hologram archive's unsorted fold and hologram-ai's sorted fold have different declared behavior; graph wiring and operand roles still require framing. A derivation key must not be presented as a result-bytes hash.

An arithmetic adapter must name its domain. `Z/256Z`, the Boolean product ring with XOR/AND, and polynomial `GF(256)` are different algebras. For CRT, declare pairwise coprime moduli and reconstruction range: signed recovery of a dot product bounded by `KAB` needs product modulus `M > 2KAB`, including epilogue growth. Powers of two are not independent coprime channels. Byte encoding alone does not preserve real softmax, normalization or arbitrary tensor operations. Each selected replacement needs a precise encode/operate/decode statement or error bound and a measured cost model. The audit did not establish a reusable general CRT API in the inspected excerpts.

## 4. Product architecture and implementation sequence

**One Rust service owns model/artifact identity, capability discovery, loading, cancellation, inference, workspace access and persistence, and serves the web assets from the same origin.** A typed `NativeProvider` calls that service; an optional `BrowserTeacherProvider` is explicitly labeled with its real model. Model names, selected IDs and error states must match the loaded artifact; no silent model substitution.

Port the donor shell in bounded slices: style/layout → conversations/composer → lifecycle/progress → editor/diff/save → sandboxed preview → deliberate Git/PR workflow. Its current `index.html` sends inference to a Transformers.js worker and does not discover `/v1/models` or `/v1/chat/completions`. Source tracing found incorrect model labeling, progress scaling, detached grounding, and preview handlers without their intended elements. These are code-audit findings; fresh browser behavior was not rerun during this audit.

The first end-to-end operation must be something the native backend actually supports, such as a qualified raw-continuation route or a later typed binding operation. Do not turn a bounded binding experiment into a chat response template. Lifecycle checks must distinguish cold load, warm cache, actual response completion, stop/cancel, and failed load. Workspace behavior means a real read → proposed diff → review → save → reopen cycle, with explicit file identity and isolated preview.

| Work package | Dependency / owner at activation | Concrete done condition |
|---|---|---|
| P0 Tooling + knowledge + current-state map | This local delivery; coordinator | Servers work, plan/source records searchable, coverage truthful, project skill installed. |
| R1 Frozen token exposure diagnostic | New #973 child, assign executor when active | Construction-only report measures exposure and displacement, and compares possible cancellation/tolerance patterns without changing the candidate or choosing a new winner. |
| U1 Typed UOR identity/arithmetic ADR | Parallel bounded integration issue | Actual types mapped; one selected adapter and its semantic/cost obligations are reviewable. |
| P1 Native API/web-shell contract | Parallel interface child under #820 | Real capability discovery and one honest native operation; frontend assets and errors have a single owner. |
| R2 Language/context generalization | Follows R1 interpretation and explicit #973 contract | Remove one interface restriction at a time: clause segmentation, role ambiguity, new combinations, variable fact count/context and temporal updates. Independent holdouts, frozen decisions. |
| N1 Native reference export/loader | Accepted model/input/state schema | Artifact identity and the actual supported behavior survive export/load; no handcrafted answers or hidden fallback. |
| L1 Final-kernel lowering | Accepted reference and a stated lowering decision | Required runtime operations, representation, dtype/error and resource bounds met on the actual path. A failed mapping remains an open scientific result. |
| C1 Correctness and reasoning | #954 then #955, accepted consumer artifact | Grounding, contradiction, abstention and checkable composition work on the declared population; tiny entry probes do not establish frontier ability. |
| P2 Workspace and coding | Interface harness can precede model capability; competent coding depends on C1 | Independently checkable code tasks, multi-file repair and controlled tool iterations with real executable feedback. |
| M1 Durable model/product memory | #962 after its native blockers | Identity-scoped persistence, retrieval/update/forget behavior and multi-turn use are actually exercised. Imported project history is separate provenance. |
| E1 Cost/proof/release | #963/#964/#965 against implemented product | Resource scorecard, current proof obligations, installation/API/rollback and adopted release criteria satisfied; #940 governance resolved when required. |
| W1 Paper evidence and writing | Parallel documentation lane from now | A focused contribution, trustworthy related work, claim/evidence links, honest limitations, reviewed manuscript and reproducible source package. Submission is a distinct final action. |

Geometry expansion is deferred. It becomes useful only when a diagnosed capacity limitation predicts a discriminating improvement and the added geometry participates in the actual decision path. More coordinates or frames alone do not establish attention, language capacity or coding ability.

## 5. Formal mathematics and research-paper programme

Start the paper's evidence structure now, while the central contribution is still being discovered. The initial [claim ledger](integration/claim-ledger.json) records IDs, exact statements/quantifiers, assumptions, proof or experiment links, counterevidence and code/artifact identities. Keep the normative statement role (`Definition`, `Objective`, `Guarantee`, `Assumption`, `Empirical Criterion`) separate from evidence status (`Structural`, `Witnessed`, `Empirical`, `Assumed`, `Unproven`) and the measured or pending outcome. Reuse the repository's normative vocabulary and current proof matrix; historical R4G1 obligations do not automatically apply to the learned reference.

The formal lane should proceed through dependencies:

1. Specify bit-vector width, wrapping/overflow and shift behavior, and correspondence with `ZMod (2^w)` for the operations actually used.
2. Prove the exact finite algebra and frame/transport laws for the actual registered matrices. Match the repository's basis orientation and composition order; define encoding before proving an identity.
3. Prove weighted pooling bounds, zero cases and causal-prefix access for the specified reference computation.
4. Separately state floating-point refinement assumptions: finite inputs, rounding, reduction order and orthogonality error. An exact real identity is not bitwise BLAS equivalence.
5. Connect serialized artifacts, decoder/encoder and Rust operations to the specification; record Lean/mathlib commits, checked declarations and allowed axioms. A hash string or no-`sorry` grep is not an implementation correspondence proof.
6. Preserve empirical language/coding/performance claims as empirical. Formal correctness of a primitive does not prove learned capability or frontier quality.

Use Mathbox for an adversarial mathematical review, Wolfram/SymPy for independently derived checks and counterexamples, and Lean LSP for actual kernel feedback. Have an independent reviewer reconstruct the central argument from definitions and audit the theorem-to-code link. Do not require formalizing every incidental expression before a useful empirical paper can be written; identify exactly which claimed guarantees require proofs.

The manuscript should center one defensible contribution: the problem and prior art, precise construction, theorem statements with assumptions, executable algorithm, controlled results, limitations/negative results, and reproducibility. The eventual broad product goal need not be the paper's headline claim. A bounded result can be publishable research when its contribution is clear; this plan makes no prediction about a particular moderation decision.

Maintain reading notes and bibliography records with original authors, title, DOI/arXiv version, source URL, pages/equations supporting the used claim, checked quotation and correction/retraction status. Academic Writing Toolkit audits citation use; SciSpace/Consensus/Hugging Face discovery leads back to the primary paper. Generated bibliographies must be checked against the publisher/arXiv record. Keep sole authorship, coauthorship, acknowledgments and AI assistance accurate and distinct.

The user reports that both previous submissions were held and then declined by moderators; the exact versions/notices are unavailable. The reasons remain unknown. arXiv does not require Lean or another proof assistant, so the suggestion that tool absence caused these declines is unsupported. Continue the evidence work without making notice recovery a blocker. Local PDF/source processing failures, a preview page, moderation, endorsement and eventual public announcement are different states. arXiv acceptance cannot be guaranteed by this stack. The [publication readiness audit](integration/publication-readiness.md) records current official requirements and observed local tooling. Review actual rejection notices before choosing between technical repair, clearer scope, further evidence, appropriate category or the permitted appeal process.

## 6. Knowledge, history and documentation architecture

Use two complementary local indexes:

- **Code graph:** GitNexus over a pinned active-source snapshot (`e627252e5252`), 384 files / about 15.6 MB source. The first index reports 34,473 nodes and 94,581 edges. It is a navigation graph, with documented depth/budget/candidate caps and no complete cross-language callgraph claim.
- **Provenance ledger:** SQLite FTS source records plus curated typed relationships. Each record has origin, revision, collection time, content digest, visibility and evidence status. Edges include native `blocked_by`/`child_of`, and curated `supports`, `limits`, `supersedes`, `candidate_for`, `implements`, `tested_by` or `formalizes` only when their basis is supplied. Do not infer a semantic edge simply from similar vocabulary.

The ledger ingests the current issue snapshot and planning documents, selected public source files, source reviews and this plan. Private Antigravity/project-history records are local and require explicit private/all search scope. Keep raw restricted inventories and histories out of the repository and public/remote graph services. Search results always remain untrusted source material. The explicit CLI importer creates immutable content records; the MCP exposes reads only. No cloud vector database is needed until a measured retrieval gap justifies one.

Useful recurring questions include: “What blocks #954 now?”, “Which κ meanings conflict?”, “What was #1079's terminal and successor?”, “Which source actually computes causal logits?”, “Which imported claim has a proof?”, and “Which Gemini decision has been superseded by native evidence?” Ask narrower terms first; FTS is lexical retrieval, not an omniscient reasoning database. Refresh live GitHub before acting on an answer about current eligibility.

### Historical coverage and missing imports

- Six project-relevant Antigravity Markdown artifacts were found across two tasks and are imported as private historical context, with exact path/content identity. Prior positive product statements do not supersede current source/behavior evidence.
- Seven local Antigravity conversation databases were identified (about 624 MiB). Schema inspection was read-only; their step payloads are binary. Full transcript decoding is not established and is not claimed. Use a versioned supported export/adapter before importing those conversations.
- The local Antigravity brain directory is about 314 MiB, including screenshots/logs/artifacts; its separate knowledge directory contains no populated knowledge records. Copy only useful identified content, not all caches.
- No Claude Code project-session store or Claude Desktop/export history was found in the checked standard locations. Previously imported Codex memories remain intact. An export or alternate known path is needed to recover additional Claude history.
- For Gemini web, use [Google's Takeout instructions](https://support.google.com/gemini/answer/16920332?hl=en): choose My Activity → Gemini Apps for conversations, and the separate Gemini data selection for Gems as applicable. Import the downloaded archive locally with provider/date/message provenance. No account export has been initiated here.

## 7. Storage and automation

The initial volume inventory reported approximately 57 GiB free; tooling/index installation consumes some of that headroom. Measured large categories were about 18.2 GiB of Codex worktrees, at least 14.6 GiB of project model assets, and 4.75 GiB of the separate product checkout. Model size is a lower bound because sealed research directories were inaccessible; do not change their permissions. GitNexus's package is about 1.13 GiB and its graph approximately 235 MB. No project cleanup was performed.

Treat space management as an input to work selection:

| Class | Retain / reclaim policy |
|---|---|
| Source changes, unique evidence, sealed inputs, user exports, credentials | Preserve and back up according to their existing access scope. Never evict on age alone. |
| Pinned source snapshots and code indexes | Record revision/paths/size; retain active and cited snapshots. Rebuildable superseded indexes can be proposed for eviction. |
| Models/tokenizers/corpora | Shared content manifest; keep unique, active and cited artifacts. Deduplicate only after verifying identity and references. |
| Cargo/build caches | Inventory actual target directories; use package/manifest-scoped `cargo clean` after review. Account for baked worktree paths. |
| Worktrees | Remove only after checking clean state, merged/reachable commits, unique artifacts and evidence references. Never broad-clean the original mixed checkout. |
| Downloads and temporary exports | Check provenance/import receipt and backup before a named deletion. Avoid LFS/model downloads during repository discovery. |

Before each large clone/build/model run, estimate growth and reserve space. A proposed starting policy is a free-space reserve of max(20 GiB, 15% of the volume), adjustable to actual workloads; exceeding it triggers a reclaim proposal rather than automatic deletion. Budget source, build, model, evidence and index bytes separately. `git clean -ndX` is an inventory aid, not authorization for `git clean -fdX`.

Automation should follow events and named decisions:

| Trigger | Automatic read/prepare work | Mutating step |
|---|---|---|
| New active issue | Fetch native graph, query history, prepare bounded task contract | Assignment/implementation follows the user's active task authorization. |
| Changed upstream head | Compare selected paths/license/API, record drift | No automatic dependency upgrade or movement of frozen inputs. |
| Accepted commit/merge | Prepare new plan/source records and refresh affected index | Explicit local ingest; never silently rewrite historical evidence. |
| Long-run completion | Harvest declared metrics, artifacts and resource use | Update the owned issue only within its authorized delivery. |
| Storage threshold / periodic inventory | Report measured candidates and recovery cost | Delete only specific reviewed disposable artifacts. |
| Paper milestone | Check claim/evidence/citation/package completeness | Human mathematical review and explicit submission decision. |

These are workflow templates, not newly activated recurring schedules. No background task will upgrade dependencies, launch experiments, delete storage or submit a paper solely because this document exists. When a recurring monitor is requested, use the existing Codex heartbeat mechanism and notify only on a meaningful change, failure, completion or required action.

## 8. Reviewable delivery and next action

This delivery supplies installed tools, source inventories/reviews, a local queryable knowledge system, a scoped project skill and proposed repository documentation. The separate audit cache preserves raw source provenance locally; public repository artifacts contain curated public information only. The original mixed checkout is preserved. No scientific run, upstream product-code merge, issue rewrite or arXiv submission is performed by installing the tools.

The next active scientific task remains the frozen token exposure diagnostic under a new #973 child. Alongside it, the first engineering contracts are U1's typed UOR identity/arithmetic mapping and P1's native API/browser boundary. The first publication task is to establish the contribution/claim/evidence ledger around the actual current mechanism; incorporate the old notices if they become available. These three lanes make the next work more effective without treating tooling, a frontend or a proof assistant as model progress by itself.
