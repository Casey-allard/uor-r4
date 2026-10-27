# Serving and export

A finished model has to leave the lab and run on an ordinary laptop or in a web browser. The project's rule is that the file it ships contains only whole numbers, and that the program running it uses only adds, shifts and table look-ups: no decimal (floating-point) arithmetic and no multiply instruction. The program that does this exists, and a checker has counted its machine instructions. But it has only ever run older models that fell short of their targets, plus a synthetic stand-in. No model built under today's rules has been exported to it yet, so nobody can chat with the project's model on a laptop today.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md) and the [glossary](../start/glossary.md).

![Four boxes left to right. A table of float weights, trained offline. A snap step that rounds each weight to a 4-bit code with a group scale. An integer table file (UORLUT01). A serving engine that answers each step with table look-ups, adds and shifts. A bracket over the first three boxes marks the export gap, the quality lost by snapping.](../figures/model/integer-export.svg)

*Training may use decimals; serving may not. The snap from one to the other is where quality can be lost, and that loss is the export gap.*

## Two halves with one hard border

**Offline half.** Training runs on rented GPUs and may use floating point. The owner's rule (D37, 18 October) allows float forward passes, gradients and optimizers offline, with two limits that stay: no matrix multiplication and no squashing functions such as softmax ([D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)). The project's active learning path, the discrete model in `crates/uor-r4-discrete`, is stricter still: it is integer from end to end ([GPU and CUDA](./gpu-and-cuda.md)).

**Served half.** What ships is "D11", the project's serving contract ([D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded), 27 September). It has five rules:

| Rule | Plain meaning |
|---|---|
| R1 | No floating point and no maths library |
| R2 | No multiply or divide instruction in any served kernel. Products of runtime values use tables or exact geometry |
| R3 | Reading every stored weight for every token is an interim stage, not the end state (D5) |
| R4 | No transformer backbone |
| R5 | Energy is claimed only when measured |

Serving on the user's own device, including browser WebAssembly, is the product and is allowed ([charter](../../CHARTER.md), section 3.6). The border between the halves is the export, and it is where a float-trained candidate is judged.

## How the engine works

**The container (definition).** `UORLUT01` stores weights as 4-bit codes in groups of 32. Each group has a one-byte scale, built from a 4-bit mantissa and a 4-bit exponent. A few small integer tables cover the nonlinear steps: square root, decay rates, exp, SiLU, GELU and arcosh (SiLU, GELU and exp are squashing-type tables; see the compliance note under "Where it stands") ([`format.rs`](../../crates/uor-r4-integer/src/stack/format.rs)).

**A projection with no multiply (proof by construction).** For a weight row $w_{ij}=s_g\,q_{ij}$ with 4-bit code $q_{ij}$ and group scale $s_g$, the output is

$$y_i=\sum_j s_{g(j)}\,q_{ij}\,x_j .$$

Instead of multiplying $q_{ij}$ by $x_j$, the engine builds once per activation a 16-entry table $T_j[q]=q\,x_j$ for $q=0,\dots,15$. Each entry comes from the one before by adding $x_j$. Each weight's code then selects an entry, and the group scale is applied with shifts and adds. Four output rows share each table ([`kernels.rs`](../../crates/uor-r4-integer/src/stack/kernels.rs)).

**Two runtime values multiplied together (proof).** A score, a gate or a value mix needs a true product of two numbers that only exist at run time. The engine reads it from a table of the sixteen multiples of one operand, built by repeated addition, indexed by the radix-16 digits (base-16 digits) of the other operand and summed with shifts (`stack_mul_u64` and `stack_mul_u128` in [`kernels.rs`](../../crates/uor-r4-integer/src/stack/kernels.rs)). Write $b=\sum_k 16^k d_k$ with each digit $d_k\in\{0,\dots,15\}$ and let $T_a[q]=q\,a$, built as $T_a[q+1]=T_a[q]+a$. Then

$$ab=\sum_k 16^k\,T_a[d_k(b)] ,$$

which is exact by the definition of base-16 digits, and each term is a table read followed by a shift. (An alternative that D11 rule R2 would also allow is the quarter-square identity $ab=\lfloor (a+b)^2/4\rfloor-\lfloor (a-b)^2/4\rfloor$, which holds for all integers; the training crate uses it in `addressing_arms.rs`, but the serving engine does not.) Normalisation uses an integer square root, computed digit by digit, and the sampler is an integer min-p rule. That sampler filters on probability mass read from the exp tables, so it is part of the compliance gap described under "Where it stands".

**Exact geometry as tables.** Where the model uses geometry, the tables are exact: the Cayley table of the 120-element icosian group (about 14 KiB), a 1,025-entry arcosh table for Lorentz read scores, and the Q30 Hopf observation. The integer crate's [`native_geometry.rs`](../../crates/uor-r4-integer/src/native_geometry.rs) provides the typed H4 geometry and checked integer arithmetic that the qualified native build uses, and [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs) loads the table-native `uor-r4.lut-discrete/1` (`UORLUT01`) container exported by [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs) (`discrete.table`, `discrete.cells`, `discrete.routes`, and `discrete.gates`, with zero 4-bit matrix sections) to evaluate `Route`, `Flock`, and `Relation` prefixes in integer arithmetic (#2838).

**A saved conversation (definition).** The `UR4SESS1` format saves a session's state and restores it byte for byte ([`session_codec.rs`](../../crates/uor-r4-integer/src/session_codec.rs)).

**Discrete raw-token CLI.** `uor-chat --discrete MODEL.lut --prompt-ids CSV` loads the table-native artifact through `IntegerDiscreteModel` and appends its integer top-1 prediction for a fixed generation budget. The initial prompt must fit the declared context, which then slides during decode. No tokenizer or sampler is used; text chat and discrete `UR4SESS1` sessions are not implemented. The finite control compares two fresh processes with direct `evaluate_prefix` output. This is deterministic artifact-reload evidence, not conversational persistence, trained-model quality, or machine-code coverage of the new CLI root. The existing audit symbol sets do not name that root; the gap and the remaining quality inputs are registered in the [M4 draft sheet](../labs/m4-export-uorlut01-cli/PRE-REGISTRATION.md).

## The zero-multiply audit

The audit disassembles the compiled release program and counts three kinds of instruction over the serving functions: Class I multiply, Class II divide, Class III floating point. The scripts are [`audit_zero_matmul_serving.py`](../../scripts/audit_zero_matmul_serving.py) (native) and [`audit_zero_matmul_wasm.py`](../../scripts/audit_zero_matmul_wasm.py) (WebAssembly, with a call-graph mode). They are existing Python scripts; porting them to Rust is an open question.

- **Native:** 44 symbol ranges audited, none with a forbidden instruction ([project track](../integration/project-track.md), milestone M4). **Measurement.**
- **WebAssembly:** 7 of 7 serving functions (not modules) with multiply, divide and float counts all 0, in the runtime merged as `UOR-Foundation/uor-r4#2654` ([record](../labs/wasm-serving-runtime-2026-10-18/README.md)). It was written by an outside contributor and hardened in review before merge. **Measurement.**

What the audit proves is absence: those instructions are not in the audited functions. It says nothing about model quality, and it does not cover every function in the binary. The record also lists three limits. First, the WebAssembly SIMD kernels are fixtures that nothing calls; the serving path still uses the scalar table kernel. Second, the demo in `docs/wasm-serving/` does not run from a clean clone until its bundle is built. Third, the `serving-wasm` build compiles the `model`, `sampling` and `tables` modules unconditionally. Compliance with the "presence counts" rule (D29) rests on the feature defaulting off plus link-time removal, an interpretation that `DECISIONS.md` does not record.

## The export gap

**Definition.** The export gap of a candidate is its served score minus its float score, on the same evaluator and the same positions. It is the price of snapping. The charter says a float candidate whose gap is above the two-seed noise band is disqualified ([charter](../../CHARTER.md), section 3.7).

Past gaps come from an older float model, the "geometric stack", not from today's discrete model. Bits per byte (BPB) and nats are different units, and lower is better in both.

| Date | What was exported | Gap | Source |
|---|---|---|---|
| 28 Sep | 7.15M stack, round-to-nearest 4-bit | +0.0362 nats (gate 0.02) | [record](../../archive/integration/s1-stack-serving-measurements-2026-09-28.md) |
| 28 Sep | Same stack, GPTQ rounding | +0.0257 nats (gate 0.02) | same record |
| 29 Sep | Same stack after 1,000 quantisation-aware updates | +0.010637 nats; marked pending a re-run | [record](../../archive/integration/d4-geometric-s1-qat-result-2026-09-29.md) |
| 8 Oct | 19.9M chat stack, identical positions | 0.00883 BPB; engine difference about 1e-6 | [#1915](../../archive/history/github-archive-2026-10-15/markdown/1915-served-gap-settled-on-identical-positions-0-00883-bpb-engine.md) |
| 9 Oct | Matched float, reference and integer runs at 512 windows | 0.009288 BPB; engine 2.7e-7 | [pin](../../archive/labs/criterion2-protocol-pin-2026-10-09/README.md) |

Two readings follow, both **measurements** on those older models. The integer engine adds almost nothing of its own: it reproduces its reference to about a millionth. And the whole gap is weight rounding, with the output layer the largest share in the 28 September record. Two cautions apply. The 8 October pair 0.877 float and 0.933 served was never like-for-like, because it mixed 512 and 64 evaluation windows. And the export of a model built under today's rules has not been measured, because none has been exported.

## Where it stands

| Item | State | Kind | Source |
|---|---|---|---|
| Native zero-multiply audit | Green, 44 symbol ranges | measurement | [project track](../integration/project-track.md) |
| WebAssembly runtime and audit | 7 of 7 serving functions clean; 165 tests pass; a saved session reloads identically on a synthetic stack | measurement | [`#2654` record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| Engine against its reference | No difference on every windowed check of the older models | measurement | [D11 serving](../../archive/labs/d11-serving-2026-10-10/README.md) |
| Quality of models it served | 1.20138, 1.24402 and 1.24233 BPB, against a 0.90 BPB bar. The record blames the model, not the export | measurement | same record |
| Speed of those models | 68 to 80 tokens/s on 8 threads for the 29M-parameter stack, which reads about 30.2M parameters per token. This is evaluation throughput over 64 scored (teacher-forced) windows, not free-running decode: no decode tokens/s exists for any D11 artifact | measurement | same record |
| Energy of table-emulated products | 0.00413 J/token against 0.00096 for float, about 4.3 times worse, on a dense model. The record notes power counters were unavailable on that OS version | measurement | [#1449](../../archive/history/github-archive-2026-10-15/markdown/1449-roadmap-measured-energy-the-oversight-audit-reconciliation-i.md) |
| Export of a model built under today's rules | Table-native `uor-r4.lut-discrete/1` / `uor-r4.lut-discrete-trunk/1` exporter ([`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs), [`trunk.rs`](../../crates/uor-r4-discrete/src/trunk.rs)) and integer evaluator ([`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs)) match the CUDA-qualified `Route`, `Flock`, and `Relation` witnesses and the two-seed M1 closing exports (`04652a3d…` and `1b49e8bd…`, `#2837`/`#2841`) with zero export gap (`14,287 / 14,287` on `ft/dev`, `14,479 / 14,479` on `Fresh V2`); the potential-ratio lawful likelihood (`PRLL`, [`lawful_likelihood.rs`](../../crates/uor-r4-integer/src/lawful_likelihood.rs), `#2861`) is registered (`TEST FITNESS: FIT`) for held-out stream scoring, which remains unrun until `m4-export-uorlut01-cli` (`#2859`) | proof / measurement | [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs), [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs), [`lawful_likelihood.rs`](../../crates/uor-r4-integer/src/lawful_likelihood.rs), [project track](../integration/project-track.md), M4 |
| Serving addressed (product-key) memory | Refused by the exporter; the kernel is not built | blocker | [record](../../archive/labs/native-address-port-2026-10-11/README.md) |
| Milestone M4 (one served model, bar 0.90 BPB) | Not met. 0 of 8 milestones are complete | status | [project track](../integration/project-track.md) |
| Web client | The Studio page still points at a retired engine; the Pages deploy is switched off (D36) and the live site is the 13 October build | status | [D36](../integration/DECISIONS.md) |

**Why reading every weight is not the end state.** A dense model streamed through tables is still dense. The table-emulation energy test above is the evidence, and D5 asks instead for selected, addressed reads that touch few parameters per token. **No existing schema fits today's rules as it stands.** Schemas /1 and /2 carry softmax-style reads through exp-table masses, which the no-softmax rule (D23) names. A third schema serves a softmax-free flock read, but its trained arms missed their bar ([record](../../archive/labs/softmax-free-read-2026-10-10/README.md)). All three carry SiLU and GELU tables, which the charter bans in renamed or table form (section 3.2), and all three stream dense 4-bit weight maps through a table-based matrix-vector product, which the charter excludes as a dense model disguised as lookup tables (sections 3.3 and 3.6). The min-p sampler also works on exp-table probability mass. D11 rule R3 ("reading every weight is an interim stage") is historical wording: dense table maps are no longer an allowed stepping stone for new exports. A new table-native schema, built on selected and addressed reads, is therefore likely required. These are the engine's known compliance gaps, and the [serving mechanism page](../mechanisms/lut-serving.md) tracks them.

**Next.** Export one trained discrete model checkpoint through the `uor-r4.lut-discrete/1` container and report its served score on the held-out stream (milestone M4).

## Go deeper

- Mechanism page: [D11 integer table serving](../mechanisms/lut-serving.md). Model and geometry: [discrete learning](../mechanisms/discrete-learning.md).
- Code: [`crates/uor-r4-integer`](../../crates/uor-r4-integer/) (engine, container, session codec, `uor-chat` and `uor-r4-stack` command-line programs behind the `serving-wasm` feature).
- Decisions: [D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded), [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised), [D38](../integration/DECISIONS.md) (the two halves of D5).
- Records: [context curve](../../archive/labs/context-curve-2026-10-11/README.md), [stack port design](../../archive/integration/stack-mission-port-design-2026-09-27.md).
- Next pages: [data and tokenizer](./data-and-tokenizer.md), [GPU and CUDA](./gpu-and-cuda.md). Model pages: [architecture](./architecture.md), [training](./training.md).
