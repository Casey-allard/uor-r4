# Skrit-Bit 0.1: pinned-source audit

Date: 2026-09-30. Reviewer: Codex history/science subagent.

Source: `https://github.com/markrnd87-cmd/Skrit-Bit-0.1`, commit **e5d8c871daec411c3b47af56244e1a2fe70f9314**. Local checkout: `/Users/casey.allard/.local/state/uor-r4/research/20260930-skrit-bit/source`. `git rev-parse HEAD` matched the pin and `git status --short` was empty.

**Scope and evidence status:** SOURCE_REVIEWED; NOT_EXECUTED. Read all seven `src/skrit` files (1,867 lines, including the 717-line Rust-source string collection), inventoried the repository, and inspected the application dispatch and relevant UI claims/viewers. No install, application execution, Rust build, Cargo, model work, or production runner mutation. Counterexamples below are static traces of this pinned source, not captured execution results. PDF-level designs may describe mechanisms absent from this repository; this audit does not equate their absence here with a refutation of a separately specified design.

## Assessment

This repository contains a small **TypeScript compiler/interpreter demonstrator** with a genuine local lexer → parser → stack-bytecode → VM path. It is useful for illustrating finite bitset classification, carrying an implicit subject through a parser, stack-machine lowering, and ownership-inspired array updates. It does **not** implement dependent types, a proof kernel, rule subsumption, order-independent rewriting, a learned language model, or a geometric reasoning system. Its displayed Rust workspace is source text for copying, not the implementation that the interface executes; that text also contains definite Rust syntax errors and materially different semantics.

The strongest issue is not merely unfinished breadth: the TypeScript reference-count simulation can mutate a still-shared array, and the advertised dual-script normalization, SSA, constant-time/no-reallocation, and production-Rust claims exceed the code.

## What actually executes

- [`App.tsx:96–117`](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/App.tsx#L96-L117) calls TypeScript `runSkritBit` and constructs a TypeScript `VM` for stepping.
- [`engine.ts:36–84`](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/engine.ts#L36-L84) actually lexes, parses, emits a chunk, and runs the VM. Output is not simply a hard-coded answer table.
- The complete AST is `Num`, `Var`, numeric `Array`, `Assign`, and `Add`; the complete instruction set is constant/load/store/add/return. See [`types.ts:26–51`](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/types.ts#L26-L51).
- `package.json` provides Vite/TypeScript commands, with no test command. The tracked inventory contains no `.rs`, actual `Cargo.toml`, Rust crate tree, `.wasm` module, or tests. `@google/genai` and a Gemini capability label are present in package/metadata, but the inspected application execution path contains no model request.
- [`rustSources.ts:4–24`](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/rustSources.ts#L4-L24) stores filenames and contents as JavaScript data. [`RustWorkspaceModal.tsx:10–13,92–95`](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/components/RustWorkspaceModal.tsx#L10-L13) copies/displays this text. It does not compile or load it. The claim at lines 23–26 that these are production-ready crates is unsupported.
- The “Rust Debug Text” pane actually receives `JSON.stringify(ast, null, 2)` from `engine.ts:55`; see `AstInspector.tsx:56,132–134`. This is a presentation label, not Rust execution evidence.

## Claim-by-claim findings

| Claim or mechanism | What the pinned source establishes | Boundary |
|---|---|---|
| Bitmask categorization | Explicit character-to-index switch; fixed interval masks; bitwise membership | Useful finite classification, not semantic inference, proof, or whole-input O(1) tokenization |
| Anuvṛtti context | One parser field `activeVar`, updated sequentially; omitted left operand becomes `Var(activeVar)` | Parser desugaring, not dependent context/type inference or learned contextual reasoning |
| Dependent types / ITP / proof kernel | A token category named `Type` and a metadata ITP label | No type judgments, type terms, binders, universes, substitution, definitional equality, proof objects, or checker |
| Subsumption | No rule-set representation or subsumption procedure | A bitset subset test could be added, but is not present and would only prove set inclusion for the declared encoding |
| Order independence | Source-order parse, context update, compile, mutable variable store | Explicitly order dependent; no confluence check, dependency scheduler, priority conflict resolver, or normalization proof |
| SSA | Recursive post-order emission into a straight-line stack program | No SSA construction/renaming/verification; linear bytecode alone is not an SSA transformation |
| Ownership / Mokṣa | TypeScript manually maintained count, conditional array push or spread-copy | Count is not sound ownership accounting; Rust `Rc` shown separately is real standard COW in principle, but not executed here |
| Dual-script invariance | Some spellings share classification bits and digit values | Variable identity remains the raw character; AST/constant pools are not canonicalized |
| Hardware pointers / allocation savings | Artificial incrementing object IDs and mutation/copy counters | No physical address evidence, allocation benchmark, energy measurement, or whole-path serving cost |

### 1. The bitmask mechanism is real, but narrowly scoped

[`lexer.ts:10–60,66–94`](https://github.com/markrnd87-cmd/Skrit-Bit-0.1/blob/e5d8c871daec411c3b47af56244e1a2fe70f9314/src/skrit/lexer.ts#L10-L94) maps a bounded inventory to indexes and forms an interval mask as

`M[s,e) = (1 << e) - (1 << s)`.

For valid `0 <= s < e`, this sets precisely the interval's bits; `MASK_AC` is bits 0–8 and `MASK_HAL` bits 9–41. Classification uses `(1 << index) & mask` (`lexer.ts:202–225`). This is a compact implementation of a finite, authored predicate. It neither discovers categories nor turns the bit index into a semantic distance.

The whole lexer loops over input (`lexer.ts:120–248`) and accumulates multi-digit numbers (`147–174`), so its work is at least linear in input length. The TypeScript mask uses `bigint`, not a verified native `u64` kernel. The bounded index range permits bounded mask operations, but no timing or instruction audit has been performed.

The mapping is customized, not a complete canonical implementation of all Pāṇinian phonological classes: several characters lack ASCII counterparts, arbitrary Latin fallback characters share display bit 14 (`231–242`), and only six end markers exist. A membership mask can classify multiple spellings alike while still preserving distinct identities; that is exactly what this source does.

### 2. Dual-script identity and type claims do not follow from classification

Tokens retain `value: c` (`lexer.ts:207–225`); the parser retains that spelling as the variable name (`ast.ts:56–76,159–160`), and compiled constants retain it (`compiler.ts:146–164`). Therefore:

```text
k = [1]
क + 2
```

has matching classification bit 14 for both letters, but the VM looks up variable `क`, while only `k` was stored. The predicted result is an unbound-variable error (`vm.ts:147–150`). Entirely transliterated programs can have equivalent results after a consistent renaming, and their numeric opcode/index streams can match, but their AST strings and variable-name constant pools differ. The stronger identical-AST assertion in `App.tsx:54` is false for this source.

The token `Type` is assigned to vowels, including ASCII `a` (`lexer.ts:13,218–225`), but there is no type grammar or type checker. The shipped aliasing preset starts with `a = [1,2,3]` (`App.tsx:44–48`), so `a` is not an identifier and cannot establish the advertised alias binding. The parser's fallback skips unsupported tokens (`ast.ts:35–42,143`), rather than interpreting their type meaning.

### 3. Context propagation is deterministic desugaring and source-order dependent

`activeVar` is a single nullable name (`types.ts:34–42`), initialized empty and updated by explicit declarations (`ast.ts:64–76`) or explicit variable addition (`80–89`). An incomplete `+ value` becomes `Add(Var(activeVar), value)` (`95–123`). There are no scopes, cancellation syntax, multiple role bindings, or learned selector.

Reordering these declarations changes the final inherited subject:

```text
k = 1
y = 2
+ 0
```

predicts 2; swapping the first two statements predicts 1. This is intentional sequential state, not order independence.

The parser consumes only an atomic RHS (`ast.ts:149–198`). `x = 1 + 2 + 3` is therefore an assignment of 1, followed by two independent additions using `x`, predicting final 4 rather than 6. Likewise, the numeric cascade preset `x=10; +5; +20` (newlines) produces intermediate 15 and final 30; it does not update `x` to 15. The compiler emits no store for `Add` (`compiler.ts:168–173`). Array expressions behave differently because the VM may mutate the variable's referenced object.

Unsupported characters are silently discarded (`lexer.ts:230–247`): `1*2` predicts two literal expressions and final 2, and `x=-1` loses the minus. These are parser-language gaps, not a formal rejection of ill-typed programs.

### 4. Stack lowering is not evidence of SSA or a verified rewrite calculus

The actual compiler performs ordinary recursive post-order traversal (`compiler.ts:121–175`), with repeated `StoreVar` to raw names and `LoadVar` by name. There is no unique SSA version for assignments, no value-definition graph, no phi mechanism, no type-preserving transformation proof, and no subsumption or rewrite-rule scheduler. A compiler could subsequently translate this stack code into SSA, but that transformation is absent.

The claimed association between a grammatical metarule and post-order lowering is a design analogy; it does not establish compiler correctness, confluence, order independence, or language-learning capability. For comparison, the [LLVM language reference](https://llvm.org/docs/LangRef.html) specifies an SSA representation and its verification constraints; a sequential instruction array alone does not establish those properties. Mutable memory can coexist with an SSA IR, so the objection is the absent SSA representation/transformation, not merely the existence of mutable storage.

### 5. The TypeScript ownership simulation has a concrete aliasing failure

`StoreVar` counts matching variables and assigns `refs + 1` (`vm.ts:115–130`). `LoadVar` pushes the same object without increasing the count (`153–154`). When copying on an array append, the VM decrements the old object's count (`228–230`) even though it has not replaced either original variable binding. There is no general decrement on replacing a variable, no stack-liveness accounting, and no heap reclamation before reset.

Static counterexample:

```text
k = [1]
y = k
+ 2
+ 3
```

1. `k` and `y` both refer to array A, recorded count 2.
2. First `+2` clones A to B `[1,2]`, pushes B, and decrements A's recorded count to 1. Both variables still refer to A.
3. Second `+3` loads `y` → A. Seeing count 1, it mutates A to `[1,3]`.
4. Both `k` and `y` now observe `[1,3]`; the prior result B stays `[1,2]`. The claimed shared-alias immutability was not preserved.

This is a correctness failure, not just a missing optimization. `k=k` also counts its soon-to-be-replaced binding and inflates the count. All heap objects stay rooted in `memoryHeap`; counters and displayed hexadecimal IDs are not physical pointers (`vm.ts:16–17,35–45,217–234`). Compiler constant IDs start at 101 while COW IDs start at 201, so sufficiently many array constants can also collide with generated COW IDs; the two ID spaces are not separated.

### 6. Displayed Rust is neither buildable as written nor semantically equivalent

**Definite syntax defect:** `rustSources.ts:109–130` declares a `char` parameter, then uses literals such as `'ण्'` and `'च्'`, each composed of a consonant plus U+094D virama. Rust character literals represent one character scalar, not a two-scalar grapheme sequence. These literals cannot compile as written. This is a source-level conclusion, not an observed compiler run. See the [Rust literal grammar](https://doc.rust-lang.org/reference/tokens.html#character-and-string-literals).

Other material differences and gaps:

- Rust `start_idx` recognizes far fewer consonants, maps ASCII `x` to vowel index 4, and has no TypeScript Latin fallback (`rustSources.ts:82–105` versus `lexer.ts:56,231–242`). Rust parsing lacks the TypeScript explicit `k + 2` and top-level numeric-expression cases (`rustSources.ts:279–308`).
- Rust VM `Value` derives `Clone` and holds `Rc<Vec<i32>>` (`428–433`); constant loading clones that `Rc` (`517–521`), the variable table retains it (`523–527`), and variable loading clones it again (`530–535`). Thus even the advertised simple unique-array case has at least the constant-pool, variable-table, and operand ownership references when `Rc::make_mut` is called (`550–553`). The displayed count-1 claim is not true for that path.
- After correcting only the syntax, `k=[1]; +2; +3` (newlines) would append to separate COW results and leave the variable at `[1]`, predicting final `[1,3]`. TypeScript instead mutates its simulated unique object, predicting `[1,2,3]`. These are source-derived predictions, not executed parity results. [Rust's `Rc::make_mut` documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.make_mut) confirms cloning when other strong owners exist.
- Rust truncates constant indexes with `as u8` (`486–489`), so the 257th entry wraps to index zero; TypeScript uses unbounded `number[]` indexes (`compiler.ts:6–17`). No rejection or wide-operand encoding is implemented.
- Rust indexes instruction/constant arrays and unwraps stack values (`511–542`) and panics on bad opcodes (`455–463`); it is not a checked untrusted-bytecode interface.
- Neither path establishes the claimed worst-case O(1), no-reallocation append. [Rust `Vec::push`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push) is amortized O(1), with copying when capacity is exhausted. Both displayed Rust logging (`557–563`) and TypeScript logging (`210,240`) format the complete array, so the demonstrated append path includes O(n) output work even if the buffer append itself does not reallocate.

## Additional claim limits and useful next steps

The TypeScript VM uses JavaScript numbers for numeric parsing/addition, with `*10` during parsing, dynamic arrays/maps, string formatting, and per-instruction logs. It is not evidence for UOR-R4's final integer-only/no-multiplier/no-allocation kernel contract. Its 10,000-instruction ceiling exits the loop without an explicit limit error (`vm.ts:279–290`), which can return an empty result with no error while execution has not halted. There are no measured throughput, RAM, power, corpus, reasoning, correctness, or cross-implementation parity results in this repository.

The useful transferable ideas should be extracted at their actual scope:

1. **Compile finite, independently defined predicates into bitsets.** Suitable for operator capability masks, grammatical feature sets, and exact candidate admissibility. Establish the mapping and maximum width; preserve unknown-category rejection. A class-membership mask cannot replace content identity, occurrence/version tracking, geometry, or a learned semantic metric.
2. **Elaborate context-dependent shorthand into explicit typed IR.** An explicit elaboration trace could help inspect UOR routing or structured-tool decisions. Preserve source spans, entity identity, binding scopes, version and complete operator operands; a lone “last variable” is insufficient for conversation memory.
3. **Expose each boundary for debugging.** Token/AST/IR/output views are valuable if they identify the actual executing backend and immutable input/artifact identity. Do not label TypeScript JSON as Rust or counters as measured allocation/performance evidence.
4. **Use established ownership mechanisms for buffer reuse.** Move semantics, exclusive mutable references, preallocated arenas, or correctly scoped `Rc::make_mut` can reduce actual copies. Count all owners, define observation semantics, preserve constant artifacts, and evaluate worst-case capacity growth and retained memory. This is an engineering technique, not a new proof of functional purity.
5. **Separate a reasoning proposer from a small trusted checker.** If the PDF proposes a proof-oriented language, a minimal Rust checker with a specified core calculus could be independently useful. This repository supplies none of that checker; bitmask lexing and naming conventions cannot substitute for typing/equality/normalization rules. Any such implementation should answer a concrete UOR deliverable rather than become a new broad language-rewrite project.

**Recommendation:** retain Skrit-Bit as a source-linked design illustration and a source of small engineering patterns. Do not adopt its code as a sound proof system, ownership foundation, canonical SSA compiler, or existing Rust/WASM implementation. No new model experiment or broad compiler rewrite is justified by this audit alone. The next decision is whether one of the narrowly scoped mechanisms resolves an already identified UOR bottleneck under matched correctness and cost conditions.
