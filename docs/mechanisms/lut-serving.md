# D11 integer table serving (UORLUT01)

**Status: ACTIVE.** It is the end-of-pipeline contract: the finished model must ship as one integer artifact, and this engine runs it. The engine reproduces its reference answers exactly, and its zero-multiply audits are green. A table-native schema (`uor-r4.lut-discrete/1`) exports `DiscreteModel` snapshots into `UORLUT01` (`discrete.table`, `discrete.cells`, `discrete.routes`, `discrete.gates`) and evaluates `Route`, `Flock`, and `Relation` prefixes in `uor-r4-integer` with exact integer parity against the CUDA-qualified witnesses (#2838). No trained checkpoint built under today's rules has been exported and scored on the held-out evaluation stream yet (training is HELD under D40).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

This is how a finished model runs on an ordinary laptop or in a web browser using only whole numbers: adds, shifts and table look-ups. There is no decimal arithmetic and no multiply instruction. A trained model is exported to a compact file, mostly 4-bit numbers. A checker then reads the compiled program and counts machine instructions to prove that no multiply sneaks in. The engine reproduces its reference answers exactly, and the checks are green. But it has only served older models that fell short of their bars, and no model built under today's rules has been exported through it.

## What it replaces in a transformer

The arithmetic. A transformer runs on large matrix multiplications. Here the same weights are read through tables, so every product is a table read and an add. This changes how a model is computed, not what it is. The engine does not make a weak model better, and, as the history below shows, it still reads every stored weight for every token.

![Three panels. Left: one dot product done with table look-ups and a shift, with a counter reading zero multiplies. Middle: float and integer outputs on the diagonal, and a bar showing the real cost is weight rounding. Right: a strip of 44 native and 7 WebAssembly symbols, all counted zero multiply, divide and float instructions.](../figures/mechanisms/lut-serving.svg)

*Left, how a dot product needs no multiply. Middle, the engine is exact and the loss is in rounding the weights. Right, the audit counts instructions in the compiled program.*

## Its chapter in the project's history

- **July to August.** The first multiplier-free runtime came from the compiled-graph approach ([r4g1-graph](./r4g1-graph.md)). Its contract said only serving must avoid multiplies and allocation (#157).
- **25 September.** Learned attention, recurrence and rotation from two 1.68-million-parameter models ran with integer arithmetic and tables, and the standalone integer library reproduced the saved distributions (#1397).
- **26 to 29 September.** An interim "D10" engine allowed multiplies and a softmax table. On 27 September the owner replaced it: D11 made multiplier-free, transformer-free serving the one contract for every lab (#1443). A first energy test found table emulation of every product used about 4.3 times more energy than float (#1449).
- **8 October.** The first chat model was trained, exported and served end to end (#1893). On 10 October three 29-million-parameter models served at 1.20 to 1.24 bits per byte, and four softmax-free serving reads missed their bar.
- **12 to 13 October.** The bans on softmax and matrix multiplication moved the older multiplier-allowing engines out of every build (D29).
- **18 October.** D37 made the integer export the boundary: training may use float, serving may not. The native zero-multiply audit went green (#2655) and a contributor-built WebAssembly runtime followed (#2654).
- **22 October.** A table-native schema (`uor-r4.lut-discrete/1`) landed for `DiscreteModel` snapshots (#2838): [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs) packs and reconstructs `discrete.table`, `discrete.cells`, `discrete.routes`, and `discrete.gates` inside `UORLUT01`, and [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs) evaluates `Route`, `Flock`, and `Relation` prefixes in `uor-r4-integer` with exact integer parity against the CUDA-qualified witnesses.
- **Next.** Export one trained discrete model checkpoint through the container and measure its served evaluation score and prompt-reading / reply-writing throughput ([Stage 4 pre-registration sheet](../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md), [Stage 4 lab record](../labs/m1-export-gap-and-toks/README.md), milestone M1 #2839 / M4 #2032).

Under the [charter](../../CHARTER.md), serving this artifact on client devices (CPU, browser WebAssembly, Apple silicon) is the product and is allowed.

## How it works

**Export.** Training is offline and may use float. For serving the weights go into a `UORLUT01` container. Weights are 4-bit codes in groups of 32. Each group has a one-byte scale (a 4-bit mantissa and a 4-bit exponent). A few small integer tables cover the nonlinear steps (square root, decay rates, SiLU, arcosh, exp).

**One projection (definition).** For a weight matrix $W$ and an integer activation vector $x$, the output is $y_i=\sum_j w_{ij}x_j$, where each weight is a group scale times a 4-bit code, $w_{ij}=s_g\,q_{ij}$ with $s_g=m\,2^{e}$. The engine does not multiply $q_{ij}$ by $x_j$. For each activation it builds a 16-entry table $T_j[q]=q\,x_j$, which needs only additions to fill, and then reads $T_j[q_{ij}]$ for each weight. Four output rows are blocked to reuse the table. The scale $s_g$ is applied by shifts and adds.

**Products of two runtime values.** A score, a gate or a value mix needs a true product of two numbers known only at run time. The engine builds a table of the sixteen multiples of one operand by repeated addition, reads it at each base-16 digit of the other operand, and sums the reads with shifts: $ab=\sum_k 16^k\,T_a[d_k(b)]$, exact by the definition of base-16 digits (proof; `stack_mul_u64` and `stack_mul_u128` in `kernels.rs`). The quarter-square identity would also be allowed, but the serving engine does not use it. Normalisation uses an integer square root or a table. Sampling is integer min-p.

**Session state.** A conversation's state is saved and restored byte-for-byte (the `UR4SESS1` format).

**The five rules (D11).** R1: no floating point and no maths library. R2: no multiply or divide instruction. R3: dense per-token access to a learned store is interim, not the end state (D5). R4: no transformer backbone. R5: energy is claimed only when measured.

**The audit.** The release binary is disassembled, and the multiply, divide and float instructions are counted over the serving symbols, natively and in WebAssembly. The audit proves absence of those instructions. It says nothing about model quality, and the SIMD kernels in the WebAssembly build are unintegrated fixtures that the 7-of-7 audit never exercises ([record](../labs/wasm-serving-runtime-2026-10-18/README.md)).

**Geometry, as exact finite tables.** The 120-element icosian group's Cayley table (about 14 KiB), a 1,025-entry arcosh table for Lorentz read scores, icosian-pair weights and the Q30 Hopf observation are stored as tables. The current integer crate selects typed H4 geometry and checked integer arithmetic for the discrete model's own export ([`native_geometry.rs`](../../crates/uor-r4-integer/src/native_geometry.rs)), and [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs) loads the table-native `uor-r4.lut-discrete/1` (`UORLUT01`) container exported by [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs) (`discrete.table`, `discrete.cells`, `discrete.routes`, and `discrete.gates`, with zero 4-bit matrix sections) to evaluate `Route`, `Flock`, and `Relation` prefixes in integer arithmetic.

**Rule check.**

| Rule | Verdict |
|---|---|
| Integer serving ([D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded)) | This is D11. Exact against its quantised reference on every windowed check, with zero multiply, divide and float instructions on the audited symbols. The gap: about 30.2 million parameters are read per token for the 29M stack, which the sparsity rule ([D5-b](../integration/DECISIONS.md#d5--per-token-parameter-sparsity-is-the-terminal-serving-invariant)) calls non-compliant. Cache residency (D5-a) is unmeasured until hardware counters exist |
| No softmax ([D23](../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project)) | Partly non-compliant today. Schemas 1 and 2 serve softmax attention through exp-table masses. Schema 3 serves a softmax-free flock rank read, but the trained arms missed the bar. `uor-r4.lut-discrete/1` serves softmax-free `Route`, `Flock`, and `Relation` reads |
| No matrix multiplication ([D30](../integration/DECISIONS.md)) | Table-read matrix-vector products stream every weight each token, which D5 calls a dense multiply in disguise. The allowed target is selected addressed reads and exact geometry tables (`uor-r4.lut-discrete/1` carries zero matrix sections) |
| Offline float, with an export gap ([D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)) | Float training is legal offline. Every candidate must report its export gap on the same evaluator. The discrete path exports losslessly through `uor-r4.lut-discrete/1` (`#2838`) with zero rounding gap on the CUDA-qualified `Route`, `Flock`, and `Relation` witnesses; no trained checkpoint has been scored on the held-out stream yet |

## What has been tried

"Gap" means served minus float score. "Bits per byte" (BPB) is the model's loss on a held-out chat stream, lower is better; the milestone bar is at most 0.90. The D10 engine is the older engine that allowed multiplies. A "D11 minus D10" loss of 0.0 therefore means the multiplier-free engine reproduces the older one exactly.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 25 Sep | Integer execution, then a standalone library, of two 1.68M learned models | Probabilities sum exactly to $2^{48}$; all 4,096 distribution hashes match; 4.5 to 6.6 times faster after one fix. Language quality unproven | positive | [execution](../../archive/integration/integer-execution-result-2026-09-25.md), [serving](../../archive/integration/integer-serving-result-2026-09-25.md) |
| 25 Sep | Learned rounding between legal 4-bit codes; two quantisation-aware recipes | Learned rounding +0.0237 and +0.0465 nats against a 0.05 gate (passed). Both quantisation-aware recipes failed (+0.059 and +0.080) | positive / negative | [rounding](../../archive/integration/learned-rounding-result-2026-09-25.md), [QAT](../../archive/integration/quantized-recurrent-result-2026-09-25.md) |
| 28 Sep | Whole-system energy of table-emulated products against float | 0.00413 against 0.00096 joules per token (about 4.3 times worse), about 2.6 times slower. A dense-model result, not a verdict on selected geometry. Power counters were unavailable | negative | [#1449](../../archive/history/github-archive-2026-10-15/markdown/1449-roadmap-measured-energy-the-oversight-audit-reconciliation-i.md) |
| 28 Sep | Export of the 7.15M stack, and other codecs | +0.0362 (nearest) and +0.0257 (GPTQ) nats against a 0.02 gate. Integer arithmetic cost nothing measurable; the gap is weight rounding, mostly the output head. No codec arm met the gate; E8 at about 2 bits per weight lost 2.42 nats. 184.58 tokens/s on one thread was forward throughput, not decoding | negative | [export](../../archive/integration/s1-stack-serving-measurements-2026-09-28.md), [codecs](../../archive/integration/d4-e8-and-d11-cost-result-2026-09-28.md) |
| 29 Sep | Quantisation-aware training for 1,000 updates; the dialogue version | Gap +0.010637 nats against 0.02 (marked pending a re-run); integer arithmetic gap 7.8e-7. Dialogue version: greedy agreement 5 of 58 turns against at least 29 | inconclusive / negative | [stack](../../archive/integration/d4-geometric-s1-qat-result-2026-09-29.md), [dialogue](../../archive/integration/d4-s2-dialogue-qat-result-2026-09-29.md) |
| 8 Oct | 19.9M chat stack trained, exported, served end to end | Served 0.9550 (64 windows) against 4.3885 for the previous served artifact; the float figure 0.8770 is a 512-window number, so the two are not comparable (see the next rows for the matched gap). D11 minus D10 loss 0.0, maximum logit difference 0. The pipeline had never been run end to end before | positive | [#1893](../../archive/history/github-archive-2026-10-15/markdown/1893-a-chat-trained-geometric-stack-served-multiplier-free-0-9550.md) |
| 8 Oct | GPTQ against nearest rounding; a snapped-training route | GPTQ with 64 calibration windows best (0.93277, against 0.93653 and 0.93368). Snapped tuning served worse (0.95540 and 0.95150). Closes that one recipe | positive / negative | [GPTQ](../../archive/history/github-archive-2026-10-15/markdown/1904-gptq-beats-round-to-nearest-by-0-00376-bpb-64-calibration-wi.md), [snap](../../archive/history/github-archive-2026-10-15/markdown/1912-snapped-training-route-closed-quantisation-damage-is-a-basin.md) |
| 8 to 9 Oct | The gap on identical positions; then a matched float / reference / integer triple at 512 windows | Rounding cost 0.00883, then 0.009288 BPB; engine difference about 1e-6 and 2.7e-7. The earlier 0.877 / 0.933 pair mixed 512 and 64 windows. The 0.90 bar is neither met nor claimable on that evidence | inconclusive | [#1915](../../archive/history/github-archive-2026-10-15/markdown/1915-served-gap-settled-on-identical-positions-0-00883-bpb-engine.md), [pin](../../archive/labs/criterion2-protocol-pin-2026-10-09/README.md) |
| 10 Oct | Three 29M artifacts exported and served, 64 windows | Engine exact on all three. Served 1.20138, 1.24402 and 1.24233 BPB against the 0.90 bar. D11 ran at 68 to 80 tokens/s against 127 to 137 for D10 on 8 threads. The gap is the model, not the export | negative (quality) | [record](../../archive/labs/d11-serving-2026-10-10/README.md) |
| 10 Oct | Four softmax-free read arms, two seeds | All missed the 0.01 BPB bar (softmax 0.8772; extra cost +0.0172 to +0.2585). A learned rank table closed 54% of the gap. Closes these configurations | negative | [record](../../archive/labs/softmax-free-read-2026-10-10/README.md) |
| 11 Oct | Serve product-key memory; context 384, 1,024, 4,096; icosian activation snap | The integer path refuses product-key memory, so its speed case is unmeasured. D11 fell from 63.41 to 30.43 tokens/s across the contexts while its slowness against D10 shrank from 1.66 times to 1.24 times. Snapping after training cost +0.020 BPB | blocked / inconclusive / negative | [port](../../archive/labs/native-address-port-2026-10-11/README.md), [context](../../archive/labs/context-curve-2026-10-11/README.md), [snap](../../archive/labs/activation-snap-2026-10-11/README.md) |
| 18 Oct | WebAssembly runtime, hardened before merge; native audit | 7 of 7 WebAssembly serving symbols with zero multiply, divide and float; 165 tests pass; a saved session reloads identically. Native audit green across 44 symbol ranges. The demo is not runnable from a clean clone | positive | [record](../labs/wasm-serving-runtime-2026-10-18/README.md), [project track](../integration/project-track.md) |
| 22 Oct | Table-native `uor-r4.lut-discrete/1` exporter (`uor-r4-discrete`), container schema (`uor-r4-lut`), and integer evaluator (`uor-r4-integer`) across `Route`, `Flock`, and `Relation` (#2838) | Round-trip reconstructs `DiscreteModel` snapshots byte-for-byte (`import_d11(export_d11(&snapshot)) == snapshot`). `IntegerDiscreteModel` matches the CUDA-qualified `exact_writer`, `context_transport`, and `C2` copy-successor witnesses bit-for-bit (0 BPB rounding gap; losses 0/640 and 640 to 448). Native 44-symbol and WebAssembly 7-of-7 zero-mul audits remain green | positive (mechanism) | [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs), [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs) |

**Reading these honestly.** Two things are true at once. The engine is exact: every windowed comparison matched its reference to within about 1e-6 bits per byte (exactly, on the 64-window D11-minus-D10 checks), and the audits count zero forbidden instructions. And the models it served were weak, and it is not yet fast. The loss between float and integer is weight rounding, about 0.009 bits per byte on matched positions, not the engine. Table-based products on a dense model cost more energy and time than float. A dense model streamed through tables is still dense, which is why the project's sparsity rule asks for selected, addressed reads instead.

## What has never been tested

- Serving a model built under today's rules (discrete or lawful float) through the container. Its export gap is unmeasured.
- Hardware-counter measurement of memory traffic per token (D5-a) and of parameters touched per token (D5-b). Milestone M5 has not run.
- Whole-system energy per token with a validated sampler. The only earlier figures (5.7 and 21.0 millijoules per token) came from a truncated capture.
- Integer serving of a product-key or other addressed memory read. The kernel is not built.
- A comparison with the SIMD kernels on the serving hot path.
- Free-running generation speed. The 184.58 tokens/s figure was input-driven forward throughput.

## What stays frozen, and what would restore it

This mechanism is active, so nothing here needs reopening. Two sub-parts are held:

1. **The D10 engine and its SIMD crate** stay frozen, non-mission comparators. They return only if an owner decision restores a serving tier that allows multipliers, which D11 withdrew.
2. **Serving of product-key or addressed memory** needs an integer read kernel. The exporter refuses it today.

**Open questions.** What is the export gap of a discrete or allowed float model through the container, and does `UORLUT01` need a new schema for table-native models? How many of the 30.2 million parameters touched per token can selected reads remove before tokens per second move, given that arithmetic count was not time? Are the audit scripts, which are Python, to be ported to Rust and run in CI?

## Also known as

UORLUT01; the D11 integer engine; the D10 LUT engine (superseded); `uor-r4-lut`; `uor-r4-stack` and `uor-chat`; multiplier-free serving; the zero-mul audit; `serving-wasm`; the LUT-4 shortlist (retracted).

## Code

- [`kernels.rs`](../../crates/uor-r4-integer/src/stack/kernels.rs): table kernels, blocked product tables, radix-16 multiples-table products (`stack_mul_u64`, `stack_mul_u128`). [`session.rs`](../../crates/uor-r4-integer/src/stack/session.rs): the per-token step and exact session state.
- [`format.rs`](../../crates/uor-r4-integer/src/stack/format.rs): the container and its schemas. [`flock.rs`](../../crates/uor-r4-integer/src/stack/flock.rs): integer flock rank selection.
- [`simd.rs`](../../crates/uor-r4-integer/src/stack/simd.rs): WebAssembly SIMD fixtures, not on the hot path. [`wasm_serve.rs`](../../crates/uor-r4-integer/src/wasm_serve.rs): browser and Node entry points. [`session_codec.rs`](../../crates/uor-r4-integer/src/session_codec.rs): the exact session snapshot.
- [`native_geometry.rs`](../../crates/uor-r4-integer/src/native_geometry.rs) and [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs): integer geometry and `uor-r4.lut-discrete/1` loader/evaluator for the discrete model's export, paired with [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs) and [`crates/uor-r4-lut/src/format.rs`](../../crates/uor-r4-lut/src/format.rs). [`uor-r4-stack.rs`](../../crates/uor-r4-integer/src/bin/uor-r4-stack.rs): the command-line program the audit checks.
- [`audit_zero_matmul_serving.py`](../../scripts/audit_zero_matmul_serving.py) and [`audit_zero_matmul_wasm.py`](../../scripts/audit_zero_matmul_wasm.py): the native and WebAssembly instruction audits (existing Python scripts).
- [`uor-r4-lut`](../../crates/uor-r4-lut/src/lib.rs): container metadata only, with the dense engines excluded. [preserved D10 engine](../../archive/history/d29-excluded-lut-2026-10-13/src/stack.rs) and [`uor-r4-simd`](../../crates/uor-r4-simd/src/lib.rs): frozen. [`stack_export.rs`](../../crates/uor-r4-training/src/stack_export.rs): the old exporter, compiled out.
- Evidence: [served stacks](../../archive/labs/d11-serving-2026-10-10/README.md), [protocol pin](../../archive/labs/criterion2-protocol-pin-2026-10-09/README.md), [port design](../../archive/integration/stack-mission-port-design-2026-09-27.md).

## Related mechanisms

[geometric-stack](./geometric-stack.md): the float model this engine exported and served; the engine is exact and the stack's quality is the limit. [flock-read](./flock-read.md): the softmax-free read that schema 3 can serve; trained arms missed the bar. [exact-memory](./exact-memory.md): addressed memory the engine cannot yet serve. [spin-weights](./spin-weights.md): icosian-pair weights that make table serving of quaternion maps exact. [ternary-xor](./ternary-xor.md): the multiplier-free arithmetic substrate. [r4g1-graph](./r4g1-graph.md): the earlier frozen table runtime with the same no-multiply rule. [discrete-learning](./discrete-learning.md): the live model whose integer export is the next thing this must carry.
