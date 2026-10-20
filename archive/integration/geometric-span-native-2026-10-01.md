# Compiled geometric span actions — October 1

**Measured:** both saved ordered-span models retain the unchanged producer's behavior exactly through a source-bound token-action dictionary and integer span register. No training was performed. This advances geometric attention in the existing Rust stack; it does not qualify full integer serving or natural language.

## Mechanism and why it matters

The [learned ordered producer](geometric-span-actions-2026-10-01.md) removed the four q/k maps and learned static signed-H4 token actions plus a contextual OPEN/APPEND/COMMIT/HOLD controller. This continuation compiles those learned token actions once, then performs ordered right composition through the pinned exact group table. OPEN starts an empty working tuple, APPEND composes its action, and a nonempty COMMIT replaces the held tuple. Every occurrence reads old held state before the current update. A present identity product remains distinct from absence; empty/unopened commits preserve held state.

The dictionary is derived from actual saved F32 embedding bytes with the unchanged root classifier. Admission binds the model, configuration, embedding, explicit authored tokenizer registry, signed algebra and span policy. Reload recomputes the expected dictionary from the saved embedding: changing a code and rehashing metadata cannot authorize it. Live embedding changes also invalidate the compiled action artifact. Exact signed order, all120 inverses and all14,400 ordered products are checked against the existing pinned payload. Dictionary lookup and register update contain no source floating-point arithmetic, multiplication, division or steady-state allocation after construction.

Only the dictionary/register is the native numerical component. This replay still predicts events with the original floating-point controller, reconstructs canonical F32 coordinates at the reader boundary, and runs the surrounding floating-point reader/trunk/output. Trace storage and admission allocate. The complete model is not yet a compliant served runtime.

## Fixed retention measurement

Executed source: `251f0a796ee2015dd408fda9703e51c6aac4010e`.
Executable SHA256: `3c40bb21a08b3c79999dbd7562b644835544f94e8521239f45db3249d8657a7a`.
Original source: `cfdf535a800e343873699ca4693f7b11f9a2e32d`.
Both models, original128-row panel and stress128-row panel are hash-pinned to the original report. Actual predicted logits supply all controller events; source/answer labels are observers only. There are zero optimizer updates and no fresh draw, parameter tuning or candidate selection.

| Saved model | Panel | Retained answers /128 | Retained head0 source majorities /128 | Decision |
|---|---|---:|---:|---|
| Ordered s1 | Original |99|128|EXACT_FIXED_REPLAY|
| Ordered s1 | Stress |109|126|EXACT_FIXED_REPLAY|
| Ordered s2 | Original |128|127|EXACT_FIXED_REPLAY|
| Ordered s2 | Stress |128|128|EXACT_FIXED_REPLAY|

Across34,688 padded positions (28,976 unpadded), held presence and every present lane code agree exactly; reconstruction error is zero. All1,387,520 full-prefix F32 logits agree bit for bit. All512 answer-logit rows and both heads'1,024 source masses match both the unchanged producer and retained historical raw results. Each model uses a320-byte token dictionary (40 authored tokens ×8 lanes) and the shared15,601-byte pinned algebra payload; loader/session/offset metadata and surrounding model storage are additional. The explicit registry is not a general-language tokenizer.

The replay took69.359 seconds externally measured, maximum RSS96,763,904 bytes. This is retention evaluation timing, not optimized serving throughput or an energy result. Peak memory footprint was86,000,144 bytes. Completed chunk traces and outputs are sealed and verified at the retained root in the JSON receipt. The executed binary is preserved as a byte-identical gzip before disposable-cache cleanup.

Eleven focused checks passed: integer FSM/dictionary4, compiler/reload/replay4, stack integration1, original causal/interface1, evaluator1. Formatting and diff checks passed. After integrating main `f9c92692` (optional prime-ranked/n-gram pointer change), all17 span/interface/gradient/admission checks passed at `2fc1ec4f`; the measured models use no pointer, and no second model replay is claimed. Independent nonauthor source review approved the integration and fixed evaluator. Release AArch64 emitted dictionary-row, step and reset bodies contain no multiply/divide/floating-point arithmetic; table composition is inlined. External memcpy/memset and defensive unreachable panic callees were not disassembled, held is inlined, and closed-callgraph/full-binary instruction qualification remains unmeasured.

## Donor recompilation remains a supported route

Owner clarification retains offline recompilation of other learned weight sets. This result demonstrates conversion of our own learned static token actions, not transfer of a donor's reasoning. Token IDs alone do not contain a model's learned computation; donor embeddings can supply lexical representations, while contextual transitions, selection and output operators must also be transferred or learned into geometry. The [induction-head primary research](https://transformer-circuits.pub/2022/in-context-learning-and-induction-heads/index.html) provides a concrete precedent for predecessor-carry, matching and following-token retrieval residing in a multi-operation circuit. Its larger-model interpretation remains a hypothesis, and it is not evidence for our geometric mechanism.

Keep three transfer interfaces available: token-to-geometric actions, contextual read/write/transition behavior, and output distributions. Offline donor execution may inform training or compilation; served responses must come from the resulting geometric operators. Measure lexical retention, changed-context behavior and composed operations separately. Do not declare embeddings-only reasoning transfer, exact arbitrary-model recompilation, or geometric superiority.

## Decision and next causal deliverable

Retain both compiled producers and all prior errors. Correct occurrence selection and answer realization are distinct: exact conversion preserves seed1's output errors rather than repairing them. One held register also does not independently factor entity, role and scope or recover an arbitrary sequence from its finite product.

Next implement a saved-parameter bridge for the direct reader's signed-relative geometric potentials, preserving zero/radial/presence/orientation distinctions and separating the native event-controller obligation. Measure score/ranking/output retention before adopting it; do not resume runtime q/k maps or a metric sweep. Bounded candidate admission, learned natural span boundaries, typed scope/role state and native values/output remain subsequent attention/model obligations. Donor-informed learning can target the same interfaces. #1512 stays open for its wider language and serving acceptance.

[Machine receipt](geometric-span-native-2026-10-01.json) binds the measurement. Complete wall preparation/build/review/delivery/cleanup accounting is charged at closeout from the prior nonoverlapping cutoff; model time is included once, and concurrent specialist work is not doublecharged.
