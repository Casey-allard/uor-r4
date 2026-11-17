<!-- freshness: sources=crates/uor-r4-discrete/src/d11_export.rs,crates/uor-r4-integer,crates/uor-r4-integer/src/discrete_d11.rs,crates/uor-r4-integer/src/stack/format.rs checked=abfad060460503bbd468f5b8c74b892b2aaca800 -->
# Serving and export

A finished model must leave the lab and run as an integer artifact on an ordinary device. D11 bans
floating-point and transcendental arithmetic and excludes dense matmul; D80 and D81 allow the
hardware integer multiply and divide. **The saved discrete M1 pair and a separate FBS trunk have
now been exported and served.** That is not useful-chat qualification or an M4/M7 product release.

![Historical float-training and export route. The D85 default instead carries one native geometric state through training and serving.](../figures/model/integer-export.svg)

*Historical figure. Under D85, the integer artifact is a lossless serialization of the state training updated.*

Labels: **proof**, **measurement**, **hypothesis**. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md) and the [glossary](../start/glossary.md).

<!-- native-space-contract: Native default: one persistent native geometric state uses one declared integer or fixed-point representation for token addresses, vector coordinates, carry, routes, memory keys and reads, learned state, gradients and optimizer updates; training and serving use the same typed operators; route choices, carry state, read choices, memory addresses and token ids are bit-identical on the same prompt and starting state; the artifact losslessly serializes that state instead of an F32 master model or an export gap; floating point remains reference or diagnostic code only. -->

**D85 status:** the persistent native state/operator implementation is pending; all float-to-integer measurements on this page are historical/export evidence.

## D85: one state, one operator path

D85 requires a lossless serialization of the integer geometric state updated by training. A Q31 fixture matched CPU and CUDA after each of 64 updates. Acceptance still requires an inventory of the shared operators, raw-word replay across every required field and a recorded planted fault. The FBS export measurements below describe the earlier float-to-integer boundary. [Q31 fixture](../loop/tasks/native-space-state-contract/codex-5/REPORT.md) · [Contract requirements](../roadmap/items/native-space-state-contract.md) · [Written-state design](../loop/tasks/written-state-count-model/claude-409/DECISION.md)

## Historical two halves with one hard border

**Offline training** runs on GPUs. The discrete path is integer end to end. D37 permits float only
in a separately scanned offline module, with no softmax/squashing or dense matmul and a mandatory
export-gap measurement.

**Serving** runs on the user's device. Its five D11 rules are:

| Rule | Plain meaning |
|---|---|
| R1 | No floating point or maths library |
| R2 | Integer arithmetic; hardware integer multiply and divide are allowed since D80/D81 |
| R3 | Reading every stored weight per token is an interim stage, not the D5 end state |
| R4 | No transformer backbone |
| R5 | Energy is claimed only when measured |

Serving an exported artifact on a client device, including browser WebAssembly, is the product and
is allowed ([charter](../../CHARTER.md), section 3.6).

## The M1 integer export

The M1 closing model is a 19-table `uor-r4.lut-discrete-trunk/1` artifact saved at two seeds.
`d11_export.rs` and `trunk.rs` build it, and `discrete_d11.rs` evaluates it in integer arithmetic.
It reproduces the reference on all 14,287 DEV rows and passes the planted-fault B4 control.

| Measurement | Seed 1 | Seed 2 |
|---|---:|---:|
| DEV integer top-1 | 7,002 / 14,287 | 7,001 / 14,287 |
| Fresh V2 integer top-1 | 7,215 / 14,479 | 7,228 / 14,479 |
| p-f held-out phrasing | 7,850 / 15,953 | 7,872 / 15,953 |
| Apple M1 decode, median | **10,876 tok/s** | **10,566 tok/s** |

The throughput protocol uses a 192-token prefill and 64-token decode, with about 2.67M prefill
tok/s. This is native Apple M1 serving, not a browser measurement and not a claim of chat quality. On p-f, 62.5% of the reference replies are verbatim TRAIN replies: reference-data overlap, not a generated-copy rate or useful-chat qualification
([M1 close](../labs/m1-close-saved-model/README.md), [Fresh V2](../labs/m1-fresh-panel-transfer/README.md), [bar-f report](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md)).

## The separate FBS export

The float-trained FBS trunk exports under `lut-stack/5`, which keeps rotations exact and puts the
export gap inside its bar on both seeds at DEV scope. Earlier `lut-stack/4` snapped variants missed
that bar; `/5` is the passing contract at its measured scope
([claude-181](../loop/tasks/d11-exporter-float-trunk/claude-181/REPORT.md), [claude-182](../loop/tasks/d11-exporter-float-trunk/claude-182/REPORT.md)). `/6` carries additional Hamming-read hooks; the joined served session remains open.

The FBS native rates are different measurements from the M1 pair's: 4,217–4,320 tok/s under `/5`
([claude-182](../loop/tasks/d11-exporter-float-trunk/claude-182/REPORT.md)), then 4,491/4,504 after the integer square root
([claude-186](../loop/tasks/fbs-serving-vector-kernel/claude-186/REPORT.md)); and 4,829 snapped under claude-177. Do not substitute them for the M1 pair's 10,876/10,566.

## Container and computation

`UORLUT01` starts with an 8-byte magic string and JSON header, then aligned integer data sections.
The reader validates offsets and lengths and computes SHA-256 as it loads. Table-native schemas
include `lut-discrete/1` and `lut-discrete-trunk/1`. The separate FBS `lut-stack/5` is a 4-bit
float-to-integer stack with exact rotations, not a table-native discrete schema. `lut-stack/12`
adds the exact 48-byte native Q31 recurrence state to a SpiralCore/FBS stack: the bytes live in
the `q31.state` table (twelve little-endian `u32` words), the header binds their SHA-256 and the
trainer's epoch, step shift and version, and loading attaches that same state to the real
recurrence with no conversion. An engine that does not serve the state refuses the artifact by
its schema instead of dropping it. Older dense `lut-stack/1`, `/2` and `/3` families are
historical and do not describe the M1 artifact
([`format.rs`](../../crates/uor-r4-integer/src/stack/format.rs)).

Exact geometry can be tables: the 120-element icosian Cayley table, Lorentz arcosh values and Q30
Hopf observations. `native_geometry.rs` supplies typed H4 geometry and checked integer arithmetic;
`discrete_d11.rs` loads table-native discrete artifacts with zero 4-bit matrix sections. Session
state uses `UR4SESS1` for byte-identical save/restore. `uor-chat --discrete` also offers a raw-token
finite serving path, not text chat or discrete session persistence.

## The serving audit

The audit disassembles serving functions and counts multiply, divide, float, vector and matrix
instructions. Since claude-175, the FBS stack's audit allows scalar integer multiply and divide and
reports their counts, while still refusing float and dense/vector/matrix forms. Other engines keep
their declared audit contracts.

- Native: 44 symbol ranges pass; the stack's step uses the allowed integer operations with no float.
- WebAssembly: 7 of 7 serving functions pass, with integer operations reported rather than mistaken
  for decimal instructions.
- Planted faults and parity checks test that the audit can fail and that the integer computation
  matches its reference.

The audit proves absence of refused instructions, not model quality. The WebAssembly SIMD kernels
remain unintegrated fixtures; the serving path uses scalar table kernels.

## Export gaps, old and current

The export gap is the served score minus the float score on the same evaluator and positions.
Today's discrete M1 export has zero row gap on the 14,287-row DEV panel at both seeds; the FBS `/5` gap is inside its
registered band at DEV. These are different contracts and should not be averaged.

Older float geometric stacks had larger gaps. Matched 512-window measurements were about 0.009 BPB,
mostly from weight rounding, and three 29M artifacts served at 1.20–1.24 BPB against a 0.90 bar. The older 0.877 float / 0.933 served pair used 512 and 64 windows respectively, so it is not a like-for-like comparison; the separate 19.9M reading of 0.886838 at 512 windows used a different protocol.
Those results are historical and do not negate the M1 export or the FBS `/5` result. A dense model
streamed through tables is still dense, which is why D5 asks for selected addressed reads rather
than assuming table lookup implies sparsity.

## What remains open

- **M4:** select the accepted model and qualify the full quality, CLI, session and audit path. A
  lawful likelihood (`PRLL`) is built and registered, but GPU held-out scoring remains open.
- **Joined FBS serving:** memory, readout and text/chat distribution are not all in one integer session.
- **M5:** full-path memory traffic and energy on the accepted model.
- **M7:** library/CLI/browser token and session parity on the accepted model, not only the synthetic stand-in.
- **Addressed/product-key memory:** the older exporter refuses it. The real point-key export and p-x run happened, but claude-229 was row 0's instrument fault, not a usable bar result. Claude-231 built the nearest readout on native and browser paths; [claude-232](../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) missed greedy parity in every cell. The newer pair rows, addressed by the turn's token pairs, are refused by the export too, so they are not served ([claude-370](../loop/tasks/one-model-end-to-end/claude-370/REPORT.md)).
- **One native train/serve state (D85, 13 Nov):** the default route is one persistent integer geometric state, trained and served by the same typed operators with bit-identical replay, so the artifact carries that state instead of an export gap. Codex trained one Q31 vector state on CUDA as a finite contract test, not language-model training ([codex-5](../loop/tasks/native-space-state-contract/codex-5/REPORT.md)). Since 14 Nov the integer crate restores that trained 48-byte state with no numerical conversion and runs the same shared transition: after 64 trained steps, the restored state and its transition results match as raw words. No token generation is claimed ([codex-8](../loop/tasks/native-space-state-contract/codex-8/REPORT.md)). Since 15 Nov the artifact itself carries the state under `lut-stack/12`, and parsing attaches it without a side-load ([codex-1](../loop/tasks/m4-export-uorlut01-cli/codex-1/REPORT.md)).

The web client is connected to the current integer worker. DeepSeek's #3442/#3444 rebuilt and
published the Studio; #3445 then wired the publicly released M1 trunk for top-8 next-token
prediction, and #3446 served `/7` integer turn memory on synthetic fixtures; trained-artifact parity and memory-session restore remain open. The M1 model is a language base, not a chat assistant;
accepted-model parity remains open. The mdBook workflow publishes preview on main and production
only by manual dispatch. This documentation unit publishes neither.

## Go deeper

- Mechanism: [D11 integer table serving](../mechanisms/lut-serving.md) and [discrete learning](../mechanisms/discrete-learning.md).
- Code: [`crates/uor-r4-integer`](../../crates/uor-r4-integer/), [`d11_export.rs`](../../crates/uor-r4-discrete/src/d11_export.rs), and [`discrete_d11.rs`](../../crates/uor-r4-integer/src/discrete_d11.rs).
- Decisions: [D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded), [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised), [D38](../integration/DECISIONS.md).
- Records: [M1 close](../labs/m1-close-saved-model/README.md), [bar-f](../loop/tasks/m1-heldout-phrasing-transfer/claude-211/REPORT.md), and the [WebAssembly runtime](../labs/wasm-serving-runtime-2026-10-18/README.md).
- Next: [data and tokenizer](./data-and-tokenizer.md), [GPU and CUDA](./gpu-and-cuda.md), and [architecture](./architecture.md).
