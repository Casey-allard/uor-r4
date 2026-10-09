# Skrit-Bit assessment for UOR-R4

Status: research assessment and proposed integration experiment, not adopted architecture or executed model evidence. Reviewed 2026-09-30. External repository pinned to `e5d8c871daec411c3b47af56244e1a2fe70f9314`; UOR source inspected at `8e90aa33b8ef941d3ac5de57ad448859e20ad76b` (model files unchanged from main `61bbd85a0a49e1bb72c71b4e02752d575e3bee4b`). No external project code was executed, installed, or imported into UOR. Production compute hold remains active.

## Judgment

There is a useful architectural idea here: make semantic operations, context inheritance, constraints, and derivation steps explicit enough to be checked and executed cheaply. UOR can use that idea as an intermediate representation between learned interpretation and exact memory/typed operators. It could reduce the amount of detail that must be carried and repeatedly inferred in a continuous hidden state. It could also improve error localization and provide training feedback about structural failures.

The supplied implementation does not yet establish a dependently typed language, a theorem prover, sound ownership, general semantic understanding, or measured efficiency. The sensible research decision is to isolate a small mechanism that can improve the existing retrieval/session path. Replacing UOR with a new programming-language project is not warranted by this evidence.

## Materials and inspection

- Three-page Architectural Specification: all pages extracted and visually inspected. SHA256 `2741db768ab1f19dda98e8a71516623287588f8be516dd5d62b13942dceab34e`.
- Fifteen-page Systems Programming slide PDF: all pages rendered and visually inspected; text extraction was effectively empty. SHA256 `6367ea35d6b1440922b9a43147bf30676cf0c785df908acbca4048dab2a188f7`.
- Firecrawl retrieved the live GitHub repository page. GitHub API and source checkout bound the inspection to the commit above. Independent source, formal-methods, and UOR-integration reviewers examined complementary boundaries.
- Wolfram evaluated finite bitmask and dispatch examples and a rewriting counterexample. The checks are illustrations with explicit domains, not a proof of the external compiler or UOR.
- Primary references: Lean kernel/elaboration documentation; LLVM loop/SSA documentation; Perceus; Synchromesh; COGS/compositional semantic parsing; DreamCoder.

## What is implemented

The actual browser application executes a small TypeScript pipeline: lexical classification, parsing, bytecode generation, and a stack VM. The AST has numeric literals, variables, arrays, assignment and addition. Context propagation inherits a single active variable for an incomplete addition. The opcode set is Constant, Add, StoreVar, LoadVar and Return. This is a useful demonstrator of context shorthand and array updates.

The live engine directly connects lexer, parser, compiler and VM. It does not pass through a dependent-type checker, proof-term checker, pattern-subsumption solver or ownership checker. The Rust implementation is embedded as downloadable/displayed source strings rather than a built Rust workspace. Independent review identifies malformed multi-codepoint Rust character literals and mismatched TypeScript/Rust memory semantics. These are source-derived findings; no build or runtime reproduction was performed during this review.

Source anchors: [engine.ts](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/engine.ts), [types.ts](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/types.ts), [ast.ts](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/ast.ts), [compiler.ts](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/compiler.ts), [vm.ts](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/vm.ts), [rustSources.ts](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/rustSources.ts).

## Mechanisms worth testing

| Mechanism | Useful UOR interpretation | Boundary |
|---|---|---|
| Context propagation (Anuvritti) | A learned partial action is elaborated against an explicitly scoped, versioned context; inherited arguments record their source. | A single last-variable heuristic is not general discourse understanding. Ambiguous antecedents require alternatives/refusal; quoted, hypothetical and cross-scope text must not silently become assertions. |
| Roots plus constructors | Factor operations into a small library of typed primitives and compositional arguments instead of independent labels for every complete action. | An ADT is not automatically a dependent type. The available constructors, their semantic contracts and learning/generalization must be specified. |
| Specificity and explicit intersections | An exact, bounded legal-action layer can reject overlapping rules with no unique most-specific choice. Geometry ranks within legal candidates. | Predicate inclusion must be decidable in the chosen fragment or backed by checked certificates. Syntax depth alone cannot prove semantic inclusion. |
| Bitmask categories | Fixed operator/type/effect sets can use compact AND/OR/lookup checks without served floating-point matmul. | These masks represent declared sets, not arbitrary meaning, token order, or a universal ontology. Scope/domain size and all preparation/dispatch costs matter. |
| Proof/derivation objects | Retain a small checkable trace of sources, selected versions and typed operations; use failure locations as diagnostic supervision. | Valid inference from stated premises does not establish that those premises are true or that natural language was parsed correctly. |
| Functional reuse | Reduce temporary copying where exclusive ownership or sound reuse analysis proves it safe. | Preserve immutable evidence, past memory versions and aliases. Reuse must respect actual ownership, not a hand-maintained display counter. |

## Mathematical corrections

**Bitsets:** the source's vowel mask is 511 (`0x1ff`); its remaining mask covers 33 positions, making a disjoint partition of 42 positions. Wolfram checked these identities. Fixed-width category membership is constant work after a symbol has been decoded and mapped. Reading/tokenizing an arbitrary length-n input remains at least linear in n. The implementation loops over characters, branches and uses regular expressions for whitespace/ASCII fallback; JavaScript BigInt is not a measured single CPU instruction. Neither PDF establishes SIMD throughput or total energy savings.

**Dispatch:** let each rule denote a decidable set of inputs. For an input x, select a matching set minimal under inclusion. If multiple incomparable minima remain, do not choose by source order. An explicit intersection rule can resolve the overlap. On the finite domain 0..11, even and divisible-by-three rules overlap at 0 and 6. With a default rule covering 0..11, the even rule, the divisible-by-three rule, and their exact intersection, Wolfram checked one most-specific result for every input under all 24 declaration orders. A finite family closed under the relevant nonempty intersections, with equivalent predicates/actions handled consistently, supports this useful discipline. This does not establish confluence of an arbitrary rewrite system: `f(a) -> b` and `a -> c` can take `f(a)` to different normal forms `b` and `f(c)`. Nor does it prove termination. Closure can grow the rule family substantially; precomputation and storage need measurement.

**Proof:** proof-step names, examples and a five-part narrative do not replace inference rules, binding/substitution, equality, universes and a sound trusted checker. Lean explicitly elaborates syntax/tactics to terms checked by its kernel. A checker can reject an invalid formal argument; it cannot make an unsupported factual premise true. Source-bound provenance must therefore remain separate from logical validity. [Lean documentation](https://lean-lang.org/doc/reference/latest/Elaboration-and-Compilation/).

**SSA and termination:** a straight-line stack instruction stream with mutable variable stores is not demonstrated SSA. SSA can contain loops; tail-call lowering does not eliminate nontermination. The slide claim that it does is false. A tiny straight-line language may terminate because its instruction set excludes backward control flow, a much narrower property. [LLVM loop documentation](https://llvm.org/docs/LoopTerminology.html).

**Memory reuse:** the architecture PDF describes zero references while slide13 describes one. Uniquely owned live objects can permit in-place updates under correct alias/reuse analysis; zero references normally concerns reclamation. Perceus supplies an actual formal resource calculus and implementation precedent. Its guarantees do not transfer to Skrit-Bit's handwritten counter. [Perceus](https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/).

These corrections leave the central idea useful. The proposal's Sanskrit-derived vocabulary is a design language; its correctness comes from the definitions, implementation and evidence supplied for each mechanism.

## Fit with current UOR source

The closest current integration seam is `crates/uor-r4-training/src/stack_store.rs`: it bridges the stack's token addresses to the existing store, supplies source-bound `write_from` and pinned-view `read_at`, and packs four statuses with four history views into 16 codes. Its contract explicitly excludes a learned read/write, wiring into the stack forward pass, and D11 serving qualification. It currently allocates on lookup and uses JSON persistence. That existing seam should carry a proposed action adapter; the project does not need another arithmetic VM or duplicate memory implementation. [Pinned stack-store source](https://github.com/UOR-Foundation/uor-r4/blob/61bbd85a0a49e1bb72c71b4e02752d575e3bee4b/crates/uor-r4-training/src/stack_store.rs).

UOR already separates learned observation from exact scoped, versioned memory in `native_geometric/learner/scoped_memory.rs`; retains typed Read/Apply/Continue/Emit/Stop actions; distinguishes current, previous assertion and previous distinct value; and records same-value reassertions as distinct events. `typed_routing.rs` separates geometric metadata selection from exact payloads and contains bounded lineage/provenance operations. These are retained mechanisms, not evidence that the present geometric stack has integrated or qualified all of them. The current integration task must trace the actual session caller.

The offline `answer_oracle.rs` is evaluator infrastructure only. It must never become a serving shortcut or provide gold intents to the learned path. A new structural layer should expose a model-produced action and its selected evidence to a checker, keeping evaluator answers inaccessible.

Proposed flow:

Observed text -> learned geometric interpretation/retrieval -> explicit typed action with source/occurrence/version references -> bounded constraint check -> existing exact memory/operator -> answer with preserved evidence.

For example, an update can explicitly bind entity, relation, scope, polarity, event identity and selected version. A follow-up query can reuse only a uniquely resolved antecedent in that scope. Exact storage preserves original text and update occurrences; shared semantic structure can be interned separately. Active/passive paraphrases may share a semantic structure, but swapping agent/recipient, negation, tense, hypothetical scope or assertion identity must not collapse them.

Potential savings are fewer valid actions to score, shorter explicit action sequences, reused subexpressions and fewer copies. Costs include learned parsing, ambiguity handling, lookup, constraint checking, rule/index storage, failed proposals, fallback and output realization. No memory/energy reduction or model-size equivalence is yet measured. A bitmask cannot recover distinctions discarded by a representation.

## Smallest decision-bearing next experiment

Keep the current retrieval/session priority. After host recovery and ordinary admission, evaluate a Rust-native typed context/action boundary using existing exact-memory and operator machinery. Do not build a new language, borrow checker or general ITP first.

1. Fix a compact action vocabulary and source-reference schema, plus scope, inheritance, reset, hypothetical isolation and ambiguity semantics. Bound actions, depth and candidate count. Freeze the learned input and evaluator separation.
2. Use one matched task family containing explicit and omitted subjects, scope changes, corrections, same-value reassertions, previous/current queries, wrong antecedents, and unseen names/values/paraphrases. Pair factual and hypothetical statements. Keep natural-language order/roles and exact occurrence identity intact.
3. Compare the existing path, the same learned proposer with the structural checks, and an explicit-arguments control without shorthand. Add an oracle-parse diagnostic solely to locate the bottleneck; it is not a model result. If geometric rankings are being credited, compare an ordinary information/capacity/access-matched ranker. Use source edits and NoRead interventions.
4. Measure loaded, reloaded end-to-end answers and action correctness, false acceptances/refusals, scope leakage, invalid versus well-typed-wrong actions, and complete resident memory/latency/bytes accessed. Record all failed proposal cost. Separate valid programs from intended programs and correct factual answers.
5. If oracle parsing works and learned parsing fails, improve learning/grounding. If both fail, repair semantics/integration. If checks only improve syntax with no answer/cost benefit, retain them as diagnostics rather than claiming capability. Promote only a measured end-to-end benefit with independent review. No unchanged selector-tuning loop.

This investigation can inform the next existing work card; no new compute or programme-wide policy is authorized by this report. The implementation card must project complete actual resource bounds before execution.

## Research context

Synchromesh demonstrates the practical value of constraining code generation with syntax, scope and semantic constraints; it does not show that every valid program matches the user's intent. Compositional semantic parsing offers a relevant evaluation discipline: unseen combinations and bindings matter, and serialization artifacts must not masquerade as semantic failures. DreamCoder is a precedent for learned reusable program libraries, not evidence that UOR should replace its model or that the proposed compiler solves language. These references motivate a bounded experiment, not capability transfer.

- [Synchromesh](https://arxiv.org/abs/2201.11227)
- [COGS](https://aclanthology.org/2020.emnlp-main.731/)
- [Compositional generalization with a broad-coverage semantic parser](https://aclanthology.org/2022.starsem-1.4/)
- [ReCOGS](https://direct.mit.edu/tacl/article/doi/10.1162/tacl_a_00623/118855/ReCOGS-How-Incidental-Details-of-a-Logical-Form)
- [DreamCoder](https://arxiv.org/abs/2006.08381)

Recommendation: preserve Skrit-Bit as an external design lead and test a small checked semantic-action mechanism within UOR's existing programme. Do not import its advertised proof/safety/performance claims or replace the current roadmap on this evidence.
