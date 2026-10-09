# Skrit-Bit formal-semantics audit

**Primary verdict: incomplete, with the smallest missing implication being a specified, independently checkable core calculus connecting the narrative proof/type/matching rules to the executed program.** A useful restricted dispatch theorem can be recovered immediately; the document does not establish a complete dependent theorem prover, confluent rewrite system, or verified optimizing compiler.

Audit date: 2026-09-30. Reviewer: `independent_systems_review`, non-author, same Codex provider/root launcher. Scope: read-only static analysis; **no builds, tests, model runs, source edits, or external contact with authors**. Small counterexamples below are mathematical derivations or source traces, not executed observations.

Source: [Skrit-Bit-0.1 revision e5d8c871daec411c3b47af56244e1a2fe70f9314](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/tree/e5d8c871daec411c3b47af56244e1a2fe70f9314), locally `source/` beside this report. `itp.txt` SHA-256: `0a92b65b7bb020af1fee6950558675c24cf474e659d86336cb35f673f6a62765`. The ITP text has three page markers. This audit does not independently audit the separately scanned Systems PDF. Source checkout was clean when inspected.

## 1. Normalize the actual claims

The ITP specification proposes:

1. Fixed-domain bitmasks for lexical categories.
2. Context propagation that elaborates incomplete statements using preceding statements.
3. Morphological surface syntax for algebraic and dependent types plus ownership/borrowing.
4. A partial order of pattern denotations: strict subsets override supersets; incomparable overlaps require explicit intersection rules.
5. Reflexivity and a five-part inference tactic checked by a compiler.
6. SSA/tail-call lowering plus reference-count-driven mutation that preserves functional meaning.

The main dependency graph is:

`surface syntax → explicit scoped core terms → well-formed types/patterns → checked derivations and deterministic dispatch → defined evaluation semantics → semantics-preserving lowering/erasure → safe runtime representation → measured execution cost`.

Names and analogies motivate these arrows; each arrow needs its own contract. The current implementation directly connects a small parser to a bytecode compiler and VM. `src/skrit/engine.ts:48–77` contains no intermediate type checker or proof checker. `types.ts:27–32` defines only numeric literals, variables, arrays, assignment and addition; `types.ts:45–50` defines five opcodes. The exported Rust workspace is string data in `rustSources.ts`, not a separate checked Rust implementation in this repository. It likewise has no pattern graph, dependent typing, proof terms, ownership checker, or theorem kernel.

This is a useful educational compiler prototype with a broader architectural proposal. It is not evidence that the proposed formal layer cannot be built.

## 2. Pattern specificity: a real theorem under explicit restrictions

Let a pure pattern denote a set `P ⊆ D` of inputs in a fixed domain. Write `P < Q` when `P ⊂ Q` (proper subset): P is more specific. A rule is `(P, action)`. For input x, let `A(x) = { P : x ∈ P }`. A most-specific rule is an inclusion-minimal member of this finite family.

**Restricted dispatch theorem.** Suppose:

- the rule family is finite;
- predicate membership has a fixed, side-effect-free interpretation;
- semantically equal predicates are identified, and each resulting predicate has one unambiguous action (or provably equivalent actions);
- every nonempty intersection of two predicates in the family is represented in the family;
- the dispatcher chooses an inclusion-minimal applicable predicate.

Then every covered input has exactly one most-specific predicate, independently of source enumeration order.

**Proof.** Finite nonempty `A(x)` has a minimal member. If P and Q were distinct minimal members, x lies in `P ∩ Q`. Its represented predicate is a subset of both. Minimality forces `P ∩ Q = P = Q`, contradicting distinctness. Existence and uniqueness follow. This proves order-independent dispatch for a fixed input; total coverage requires an additional default/exhaustiveness condition.

Full intersection closure is sufficient, not necessary. It is enough that each overlapping incomparable pair has lower-priority-resolving predicates whose union covers the entire overlap, with all newly introduced ambiguities checked. One arbitrary example from the overlap does not suffice.

Concrete obligations the short specification leaves open:

- **Equal sets:** `P={0} → red` and `Q={0} → blue` have no strict subset relationship. Adding their intersection adds the same set and does not resolve the conflict. Reject incompatible duplicates or establish action equivalence.
- **Incomplete resolver:** `P={0,1,2}`, `Q={1,2,3}`, and added `R={1}` still leave x=2 ambiguous. The resolver must cover the full overlap.
- **New conflicts:** an intersection rule introduced for one pair can conflict with a third rule. Recheck the resulting family, not just the original pairs.
- **Coverage:** unique choices on covered inputs say nothing about an input matching no rule.
- **Stateful predicates:** if predicates consult changing state or have side effects, even membership can depend on evaluation order. Snapshot state or exclude such predicates from the dispatch theorem.
- **Cost:** n independent Boolean conditions can generate `2^n−1` distinct nonempty conjunction sets under full intersection closure. Correctness is compatible with exponential representation growth. Lazy checking, restricted domains and shared decision diagrams are possible implementations, not automatic cost guarantees.

**Decidability boundary.** Arbitrary computable predicates do not admit a complete terminating overlap/subsumption solver. For a machine M, define the total predicate `P_M(n)` to mean that M halts within n steps. Deciding whether `P_M ∩ N` is empty decides whether M ever halts. Restrict the predicate language (finite constructors, bitsets, bounded integers, a declared decidable logic), require proof witnesses, or retain an explicit UNKNOWN result. A solver timeout must not mean disjointness.

**Direct UOR-R4 use:** treat this as a candidate design for typed operator applicability and exact address/memory access permissions. Bitset/interval predicates can compile to bounded integer tests. It can prevent two incompatible rules silently claiming the same state. It does not supply a semantic similarity metric, a learned selector, or a theorem about language ability.

## 3. Deterministic dispatch is different from confluence

A deterministic strategy chooses one next step. Confluence says *every* permitted reduction path from the same term can reach a common result. A priority convention can give deterministic behavior to a non-confluent relation.

Small counterexample, with a unary constructor f and distinct constants a,b,c:

```
f(a) → b
a    → c
```

The two root-pattern denotations are disjoint, so root-level intersection dispatch finds no ambiguity. But rewriting beneath constructors gives:

```
f(a) → b
f(a) → f(c)
```

Both b and f(c) are normal forms under these rules and are unequal. The conflict is a nested overlap. A fixed outermost strategy returns b; an innermost strategy returns f(c). Either can be specified deterministically, but the unqualified rewrite relation is not confluent.

Separately, `a → f(a)` gives a unique applicable rule while repeated rewriting below constructors can continue forever. Specificity ordering does not prove termination.

**Salvage:** choose the requirement actually needed. For a fixed inference engine, deterministic operational semantics and bounded fuel may suffice; fuel exhaustion is a result distinct from normal-form completion. For a normalization-based type checker, specify reduction, definitional equality and its termination/confluence obligations. Critical overlaps include nested positions and substitutions, not only set intersections of top-level subjects. This audit provides the elementary counterexample itself and does not claim to have proved a general completion theorem.

Context propagation is also intentionally order-sensitive. `k=1; b=2; +3` elaborates the final expression using b; `b=2; k=1; +3` uses k. Thus any order-independence theorem should apply **after explicit context elaboration**, to an appropriate rule collection, not to arbitrary permutations of source statements. `ast.ts:60–77,96–119` makes this dependence explicit. Preserve scope, binder identity and provenance when elaborating omitted operands; a global “most recent subject” is not a general solution to lexical scoping or capture avoidance.

## 4. Proof kernel versus a narrative tactic

The Nyāya structure can be a good human/agent interface for organizing a proof obligation. It does not become a sound inference rule merely because all five fields are present.

Countermodel to example-based universal inference: take `D={0,1}` and `P={0}`. P(0) is true; `∀x∈D.P(x)` is false. A five-part account containing a claim, P(0) as reason, 0 as example, a supposed application to arbitrary x, and a repeated universal conclusion is structurally complete but logically invalid. The missing item is the justified universal step. This is **not** a claim that Nyāya itself endorses that invalid inference; it isolates what a structural-only checker cannot establish.

A sound implementation needs propositions, proof terms or checked inference nodes, an explicit context, substitution, equality/conversion, and a kernel judgment such as `Γ ⊢ t : P`. A tactic may be heuristic, use a language model, or follow the five-part narrative; its output must elaborate to evidence accepted by that checker. Reflexivity is valid when the compared terms are definitionally equal under the stated reduction rules, not because the prose calls them self-evident. The document supplies no syntax or checking judgment with which to establish those conditions.

Lean provides a concrete reference boundary: proofs are terms of the proposition's type and the checker validates the term and its type. That is a stronger contract than checking a tactic narrative's shape. [Lean: Propositions and Proofs](https://lean-lang.org/theorem_proving_in_lean4/Propositions-and-Proofs/).

**UOR-R4 opportunity:** use the narrative as a structured research/proof work card while retaining evidence labels. For exact finite operator claims, a small Rust checker can validate explicit certificates independently of the learned proposal/search process. Scientific language-quality claims still require measurements; no kernel can promote a noisy hypothesis simply from narrative structure. A future kernel should first serve an existing finite operator or memory-address obligation rather than creating a separate large theorem-prover programme.

## 5. Dependent typing, ownership and erasure

Morphology can be an effective surface grammar for types. The concrete dependent content is a family such as `Vec A n`, together with introduction/elimination and index-equality rules. An implementation must define what makes a type well formed, how dependencies bind, how equality is checked, and which recursion/axioms are permitted. “Array size depends on a value” describes an intended use; it is not itself a type system.

“No hardcoded primitive types” is not inherently incoherent: inductive encodings can express natural numbers and products. Efficient lowering still needs an explicit correspondence between logical representations and machine representations, including overflow and bounds. Here the prototype actually hardcodes numeric/array variants (`types.ts:71–74`; Rust template `rustSources.ts:428–433`). It is not currently a morphological dependent-type implementation.

Borrow/move case markers can be syntax for a discipline. The discipline must ensure exclusive mutation, lifetime validity, and invalidation of moved values over all branches/callers. For example, accepting a moved x and then allowing a later read of x is invalid regardless of the suffix naming the move. A dependent proof of a bound and an ownership capability are different resources; one cannot substitute for the other.

Erasure likewise needs a criterion. Proof-only evidence may be removable after checking. A length/index needed to allocate or select a runtime branch remains computational data unless the compiler can recover it from retained data. Erasing the n from `make_vector(n)` while expecting a correctly sized runtime vector loses required information. Lean explicitly distinguishes proof-irrelevant propositions from data-producing noncomputable constructions in its compiled interpretation. [Lean: Axioms and Computation](https://lean-lang.org/theorem_proving_in_lean4/Axioms-and-Computation/).

**UOR-R4 opportunity:** compile already checked type/shape/operator constraints into compact bounds, tables and trusted loader checks. Keep exact occurrence/version identity, lengths needed for access, and learned content available. Removing proof payload is a compiler optimization; it does not compress arbitrary semantic history or turn mathematical validity into useful prediction.

## 6. Actual ownership counterexample and a precise repair

The text's “reference count zero → destructive update” is incorrect for ordinary reference-counted ownership. Zero normally means the owned value is destroyed; safe reuse requires a separately retained allocation/reuse capability. The prototype instead tests count ≤1, but the TypeScript count is not a count of all owners:

- `vm.ts:115–126` counts current variable bindings only when storing.
- `vm.ts:141–154` loads the same object onto the stack without incrementing.
- `vm.ts:192–203` mutates an object when this incomplete count is ≤1.
- `vm.ts:228–230` decrements the old count after copy-on-write although original variables remain bound.

Source-traced specimen accepted by both parsers' intended grammars:

```
k = [1]
+ 2
+ 3
```

Each plus elaborates to `Add(Var(k), Num(...))`, **not** a StoreVar/assignment. TypeScript's first plus mutates the object still bound to k, so the second plus predicts final `[1,2,3]`. Its simulated ownership has changed a still-observable prior value. The exported Rust VM clones the Rc when loading a variable and uses `Rc::make_mut`; its corresponding source semantics predict `[1,3]`, preserving original k. The constants pool also retains an Rc, so even the single-variable example is not uniquely owned in that template. These are static derivations; the Rust template was not compiled, and it contains a separate compile issue noted below.

A clearer undercount witness is:

```
k = [1]
b = k
+ 2
+ 3
```

Both variables still point at the original array. The first plus sees 2 and copies, then decrements the original to 1. The second plus now mutates the original even though k and b both retain it. A correct reference count or affine ownership analysis cannot authorize that mutation.

Rust's documented `Rc::make_mut` clones the value when other strong owners remain. Use that operation's actual contract, or an equivalent complete ownership proof, rather than reproducing a partial count. [Rust Rc::make_mut](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut).

**Repair choices:** (a) keep expressions pure and use actual clone-on-write ownership; (b) define explicit updates and compile them to assignment, with proper alias handling; or (c) consume the old variable at its proven last use and transfer sole ownership. Do not let the representation's incidental count decide whether source-language expressions have side effects.

Uniqueness also does not guarantee worst-case O(1) append. A full-capacity Vec may reallocate and copy; push is amortized O(1), with a linear growth step. For no-allocation UOR serving, use bounded capacity plus checked capacity proof or a fixed buffer, and account for retained bytes. [Rust Vec::push](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push).

The source's exported Rust lexer also writes multi-scalar Devanagari sequences such as `'च्'` as `char` literals (`rustSources.ts:109–130`), so the export cannot be assumed buildable. Fixing that lexical representation is separate from the ownership derivation; no compilation result is claimed here.

## 7. Lowering and small-domain shortcuts

The actual compiler is recursive postorder AST traversal to a stack bytecode (`compiler.ts:121–173`). It has no functions, calls, jump opcode or recursion-elimination transformation. Repeated assignments store under the same variable key. Straight-line stack temporaries can certainly be translated to SSA, and an SSA IR may still access mutable memory; neither fact proves the advertised SSA construction was implemented. There is no evidence here of SSA versioning, control-flow joins or tail-call conversion. LLVM's actual SSA representation and phi rules are a useful concrete reference. [LLVM Language Reference](https://llvm.org/docs/LangRef.html#introduction).

Eliminating runtime recursion generally requires appropriate tail positions or explicit continuation/stack transformations. An unbounded loop needs some revisited control state; “instructions only flow forward” cannot at the same time explain unrestricted recursion using only a finite straight-line sequence. A fixed bounded unrolling is legitimate but has an explicit bound and code-size cost.

The bitmask idea is valid in scope: membership in a fixed machine-word set is a constant-size operation. It does not make whole-input tokenization O(1), eliminate Unicode decoding, or establish SIMD execution. The actual lexer loops over characters, includes branches and regex tests, and accumulates numeric literals with multiplication (`lexer.ts:113–156,177–243`). For UOR, reuse finite bitset candidate filters after an exact index mapping; do not confuse them with semantic selection. A logical category index is also different from variable identity: the lexer preserves raw characters as names, so equal category bits alone do not make `k` and `क` identical ASTs/identifiers.

## 8. Obligation matrix and practical next decision

| Obligation | Finding | Smallest useful next check |
|---|---|---|
| Finite set membership via masks | Correct in stated fixed-width domain | Verify category-to-index mapping and reject unknown symbols |
| Order-independent specificity dispatch | Correct under explicit restrictions proved above; unimplemented here | Exhaustive finite-domain oracle over rule permutations, duplicate/equal/intersection cases |
| General predicate overlap solver | Impossible as a complete decision procedure for unrestricted computable predicates | Choose decidable fragment or explicit proof/UNKNOWN protocol |
| Confluence from dispatch ordering | Refuted by nested rewrite counterexample | Decide deterministic strategy versus normalization requirement |
| Five-step narrative implies theorem | Incomplete: checked inference/term missing | One proposition language with independently checked certificate and rejected invalid generalization |
| Morphological dependent typing | Not addressed by prototype | Specify one indexed datatype and one rejected ill-typed term |
| Functional meaning preserved by TS mutation | Refuted by static alias trace | Pure reference evaluator versus corrected ownership implementation on the specimens above |
| Rust exported implementation | Unverified as executable; lexical char representation has a concrete compile blocker | A separately authorized extraction/build, then cross-runtime agreement |
| SSA/tail-call elimination | Not implemented at advertised scope | State stack-bytecode scope, or supply one defined lowering with preservation check |
| No-allocation/O(1) append | Not implied by uniqueness | Fixed capacity and last-use proof, actual allocation/byte counters |
| Useful language-model capability or energy savings | Outside evidence provided | Only an integrated, matched task/cost measurement can decide this |

**Recommended transfer order:** first reuse the *contract* of explicit elaboration plus deterministic typed dispatch where UOR already has overlapping operator applicability or address rules; next use ownership/liveness to reduce copying at an observed memory bottleneck; use proof certificates only for a specific existing exact correctness obligation. None requires importing the UI, adopting Sanskrit identifiers, replacing the geometric model, or starting a new language implementation.

A future bounded development-only discriminator could compare an ordinary explicit-rule interpreter with a checked compiled rule table on the identical inputs, outputs and memory-access trace. Test duplicate rules, uncovered inputs, complete/incomplete intersections, declaration permutations after elaboration, and exact versioned address identity. The output should be a precise correctness/cost result for that mechanism. If it does not remove a measured integration blocker, retain the idea in the knowledge bank and continue the model programme.

Primary external documentation was read on 2026-09-30. Rust pages identify standard library 1.98.1; Lean/LLVM are official live documentation, not pinned theorem artifacts. No novelty priority claim or exhaustive literature survey was attempted. The dispatch proof, countermodels and source traces above are the audit's own derivations; cited documentation supports only the stated comparator/API facts.
