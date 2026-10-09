# Skrit-bit and UOR-R4: integration assessment

Reviewer: `/root/repair_execution_council`, non-author of either implementation. Date: 2026-09-30. Read-only source analysis; **no builds, execution, training, or evaluation performed**. Proposed integration and all performance/capability benefits below are **NOT RUN**.

Scope: supplied `itp.txt`; Skrit source commit `e5d8c871daec411c3b47af56244e1a2fe70f9314`; UOR main `61bbd85a0a49e1bb72c71b4e02752d575e3bee4b`. Inspected through worktree HEAD `8e90aa33b8ef941d3ac5de57ad448859e20ad76b`, whose changes relative to that main are runner-only. No scientific source was modified. The supplied document is a specification, not executed proof of implementation.

## Verdict

**Useful design inspiration for a small, checked context/action interface; no evidence to replace UOR's current model, tokenizer, memory, or compiler with Skrit-bit.** Its context inheritance, compact finite categories, and explicit operation trace could help expose and test the boundary between learned interpretation and exact execution. UOR already has richer exact identity, version, scope, and typed-operation contracts. The integration should reuse those contracts rather than weaken them to Skrit's single active variable or mutable variable table.

The smallest useful task is an optional Rust action-trace adapter in the existing retrieval/session integration lane, with gold-program execution strictly separated from learned text-to-program behavior. Success on the gold path would establish adapter semantics only. Useful natural-language reasoning still requires the learned path, unseen phrasings/values, interventions, and the existing integrated qualification.

## Source facts and actual seams

### Skrit is a small demonstrator at this pin

- `src/skrit/types.ts:27-35` has five expression forms: number, variable, array, assignment, and addition. Its bytecode at lines 44-51 is Constant/Add/StoreVar/LoadVar/Return. It has no dependent function type, proof-term, equality-elimination, theorem, or type-checking expression forms.
- `src/skrit/engine.ts:33-89` runs lexer → parser → compiler → VM. No independent type/proof checker is called. This is a statement about the inspected executable path, not an impossibility claim about the larger specification.
- `src/skrit/ast.ts:54-131` propagates a single `activeVar` into an omitted left operand. It does not implement a nested scope stack, a declared scope boundary or isolated context fork. Malformed/unhandled top-level tokens can be skipped in `parseAll` instead of producing a complete rejected parse.
- `src/skrit/compiler.ts:138-189` recursively visits expression operands and emits stack operations. StoreVar writes the same variable name repeatedly. Calling this SSA does not make it SSA; the representation shown has no unique value definitions, block parameters, or phi nodes. There are no function calls/recursion constructs for the asserted universal recursion elimination.
- `src/skrit/lexer.ts` maps a finite alphabet to a bit index and tests BigInt masks, but performs a switch to obtain the index, regex whitespace recognition, and an input-length scan. Constant-time category membership is not constant-time whole-input lexing or branch-free parsing. The emitted token value retains the original character: same classification bit alone does not canonically identify Latin and Devanagari variable names.
- `src/skrit/vm.ts` uses JS numeric values and a mutable variable map. Its array ownership counter is application-maintained. Embedded Rust source strings in `src/skrit/rustSources.ts` use `Rc::make_mut`; this provides a more concrete copy-on-write primitive, but those strings are not a compiled/proven UOR integration. Unique-reference mutation is relevant when an object has one owner; a zero-reference object cannot be the live value being mutated.

### Current UOR stack has the best integration seam already

`crates/uor-r4-training/src/stack_store.rs` is explicitly the I1 bridge from the geometric stack to the retained exact store. Its module contract is particularly important:

- Lines 12-32 distinguish current, previous assertion, previous distinct value, initial occurrence, absent, evicted, and NoHistory. Same-value reassertions append a record; they are never removed by deduplication.
- Lines 34-78 explain injective token-address encoding, exact value identity versus owned payload, source identity, pinned views, serialization, and limits.
- Lines 80-85 explicitly say the bridge is host infrastructure: lookup allocates, persistence is JSON, it is not audited as a D11 steady-state serving kernel, and it is **not wired into the stack forward pass**.
- `StackStore::write_from` at 400 preserves a source ID; `read_at` at 440 selects against a pinned commit; `StoreValue` at 208 returns record and commit identities; `StoreSignal` at 184 packs four statuses and four history views into 16 codes using shift/mask operations. These 16-code status semantics are already implemented, so a proposed bitmask encoding is not a new capability by itself.
- `from_bytes` at 529 checks lineage and token encoding. It is a ready semantic boundary for reload checks, not proof of current conversational memory competence.

`crates/uor-r4-core/src/native_geometric/learner/scoped_memory.rs` provides the reused semantics:

- `SessionAction` at 103 is Read/Apply/Continue/Emit/Stop.
- `IntentModel` at 214 learns statement and question intent from observed token cues; there are separate assert/correct/nonasserting/compute intents.
- `HistoryView` at 183 separates previous assertion from previous distinct value. The latter is not part of all existing learned question interfaces; the adapter must not pretend all views are already learned.
- `Record` at 1215 carries scope, entity, relation, exact value, owned payload hash, predecessor, commit, source, action, conflict, continuation, and eviction. Parse score never decides version order.
- `Memory::lookup` at 1312 and `Memory::write` at 1392 supply exact pinned-view lookup and append-only occurrence/version semantics. Capacity releases payload while retaining tombstone identity. That is not an erasure guarantee, nor a bound on total addresses/tombstones.

The earlier `learner/integrated_attention/memory.rs` separately offers raw event/source/turn/byte identity, `ExactKey::from_parts`, learned-code candidate pages, explicit incomplete-search status, and causal cutoff. Its finite product codes propose candidates; they never establish exact identity or prove absence. It is a reusable research reference, not authorization to revive an older local-selector programme as the next mainline task.

`native_geometric/value_types.rs:14-19,128-147` already defines Copy/Add/Sub/Mul and derivations containing operand record IDs and exact values. `typed_routing.rs` keeps learned metadata selection separate from numeric payload and tracks comparisons/logical bytes/lineage. A new Skrit arithmetic VM would duplicate this surface. Reuse scoped supported operations, and independently audit serving arithmetic before claiming D11 compliance; the name `Mul` alone proves neither use nor absence of a multiplier instruction.

`answer_oracle.rs:1-7,13-38,59-61` is explicitly an **offline** answer oracle. It constructs accepted answers once from typed case intent; evaluation is frozen-list membership. It must never supply runtime parsing, a missing memory key, a reasoning step, or the answer to the model.

The graph compiler/runtime/certifier are separate contracts. `uor-r4-graph-runtime/src/lib.rs` is no_std/forbid(unsafe_code); `graph-certify/src/certify.rs:1-20` describes numerical equivalence/operation-cost witnesses, not a general proof checker for arbitrary programs. `reference_compiler_ir.rs` is explicitly a floating-point research IR. None is a ready Skrit dependent-type compiler merely because it is called a compiler or certifier.

## Smallest decision-bearing Rust experiment

Keep this within the existing retrieval/session task and source ownership. It is a **proposal**, not an admitted compute job or new programme.

1. Define a small versioned `ContextAction` interface around the existing `StackStore`, with bounded fields: scope/lineage, source event or token span, entity token span, relation token, requested history view/pinned commit, and operation. Initially support Assert/Correct/Read/Copy/Stop only. Do not add arbitrary code execution, functions, a theorem language, or another model.
2. Represent inherited fields with witnesses identifying the preceding field, originating event, and scope epoch. An explicit boundary clears eligibility; an explicitly isolated sub-context cannot write its parent's frame. Ambiguity, stale provenance, illegal scope, and nonexistent/evicted operands produce distinct typed outcomes. A valid-looking field is not accepted merely because it is the last mentioned variable.
3. Lower checked actions to `StackStore::write_from` and `read_at`, preserving exact record/commit identities. A read returns StoreSignal plus owned value tokens and a trace. This adapter should have a small ordinary Rust interpreter/reference implementation. Reuse existing serialization and store semantics.
4. Connect the current stack's learned output/read interface to these actions only as a prospectively specified successor artifact. Gold semantic fields belong solely to training labels and diagnostic evaluation arms. The first source-only deliverable is the adapter/trace contract and paired evaluation generation, not a claimed trained parser.
5. Freeze train/development/final splits, same model/data/exposure/optimization budgets, and full preparation/build/storage cost before any fit. Execute only after the shared admission hold is legitimately resolved. No new run is authorized by this report.

### Required comparisons

| Arm | Inputs | Purpose and permitted claim |
|---|---|---|
| Gold explicit action program → same exact executor | Authored typed fields, conspicuously marked oracle | Backend/serialization/trace upper bound only; no language claim |
| Learned explicit-field actions → same executor | Raw observed language and prior permitted context; no gold fields at inference | Isolates learned parsing/addressing from exact execution |
| Learned context-inheriting actions → same executor | Same text/information, capacity and training budget as explicit-field arm | Tests whether inheritance helps transfer or reduces work, versus merely adding exact memory |
| Current stack/session baseline | Same task examples and declared access budget | Whole-system reference; disclose added structure and costs rather than calling it an information-matched mechanism control if its memory access differs |

For the two learned-action arms, a simple canonical explicit action encoding is the ordinary competent control. The special claim must be about compact context inheritance, not about giving one arm an external oracle or more exact memory. An apparent compactness advantage that only reduces target tokens is not automatically better learned reasoning; report both training-token exposures and source-example exposures, and account for interpreter/schema storage.

Use source interventions and NoRead/wrong-address controls to establish dependence on the selected source. Separate failures into parse/subject/scope/history-view selection, failed write, failed read, execution/type rejection, copying/realization, and restore failure. A correct executor with a wrong inferred premise gives a precisely wrong answer; type safety cannot establish semantic truth.

### Minimal discriminating panel

Reuse the current open-value conversation/relation-update development cases rather than inventing another closed-vocabulary arithmetic scoreboard. Add only needed causal pairs:

- Omitted subject with one eligible antecedent versus two competing antecedents.
- Explicit scope reset and a fork that must not mutate the parent.
- Same-value reassertion followed by previous-assertion versus previous-distinct requests.
- A correction versus a quotation or nonasserting hypothetical.
- Identical names in two scopes; identical values from different occurrences.
- Save/reload between assertion and query; replay with changed source value.
- True absence versus NoHistory versus proved capacity eviction; candidate truncation is not absence.
- A composition that must retain its exact operand occurrence IDs, not just equal payload bytes.

Declare support in advance. An unsupported ambiguity may return a typed rejection; do not award it a correct factual answer or broaden the existing chat acceptance rule after seeing results.

### Decisions this can change

- Gold actions fail: fix the adapter/store contract before spending training compute.
- Gold succeeds, learned actions fail: the bottleneck is interpretation/selection/learning; a theorem checker or bitmask cannot replace that work.
- Learned explicit actions succeed, inherited actions regress: use explicit actions; retain the inheritance negative at its scope.
- Inheritance improves source-separated transfer at matched information/budget: retain it as a bounded learned interface improvement, then test the existing integrated conversation milestone.
- Both action paths work but saved discrete serving loses behavior: numerical/serialization integration becomes the next task.

## Potential shortcuts and their real costs

1. **Finite role/status masks.** Constant-size membership can use one or a few word operations after symbol-to-category mapping. Whole input decoding remains proportional to input size; larger categories need several words. UOR already packs StoreSignal into four bits. Benchmark only if classification/dispatch is a material measured cost.
2. **Compile fixed legality/precedence rules offline.** For a restricted finite rule algebra, mask subset/intersection checks can reject overlap and construct a dispatch table. This can move repeated matching out of serving. Pairwise analysis can be quadratic in rule count; tables take memory and require versioning. Overlap of arbitrary predicates is not solved by naming it a partial-order graph. Legal dispatch is not semantic disambiguation of natural language.
3. **Reuse a context frame.** Shared explicit scope and antecedent handles can avoid repeatedly materializing the same fields. Savings trade off against frame lifetime checks, dependencies, invalidation, and errors under topic changes. Small syntax can hide substantial implicit state. Count all retained context and source bytes.
4. **Exact typed execution.** Once the learned model selects the right operation and operands, exact arithmetic/copy can avoid generating intermediate token arithmetic. Include parse, retrieval, operator, checking, serialization and output costs. This is an existing UOR architectural direction, not a result established by Skrit.
5. **Copy-on-write scratch storage.** Unique transient buffers can be mutated to avoid a full copy. Shared buffers must clone or preserve indirection. Persistent conversation/version records cannot be overwritten simply because their current payload looks equal. Rust ownership and references are preferable to importing the demo's hand-maintained counts.
6. **Deduplicate immutable structure, not events.** Intern rule schemas or equal immutable payload blocks with retained occurrence/version references. Same-value assertions still create new events. Hash comparison is not an identity proof without exact collision handling; count the index and reference metadata.

No measured energy reduction follows from these source observations. Any benefit must include full-path latency, bytes accessed, resident and retained memory, instructions/allocations, and physical energy where relevant. A new language front end does not reduce the dominant model cost if it runs only during preparation.

## Recommendation

Adopt the **separation of learned interpretation, explicit context, checked action, exact execution and trace** as a useful formulation of the existing programme. Use Skrit's terminology as optional inspiration, not evidence of a new proof system or a reason to divert effort from retrieval and saved-session integration. Prefer a small original Rust adapter over importing the TS demo or embedded unvalidated Rust strings. Any code reuse additionally requires provenance/license verification; this review establishes no license conclusion.

All integration, transfer, speed and energy claims remain hypotheses/NOT RUN. The reviewed sources support concrete interfaces and caveats; they do not establish learned natural-language competence or frontier reasoning.
