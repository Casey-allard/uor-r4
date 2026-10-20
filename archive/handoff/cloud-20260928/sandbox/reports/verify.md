# Code verification report (agent: verify)

Scope: build/test of the active D8 path, multiplier-free claim, per-token access (D5), correctness
review of `joint_model.rs` + integer `model.rs`, x86 timing, 2I group table. Repo HEAD `413a32f`
(#1397). Box: x86_64 Intel Xeon @ 2.80 GHz, 4 cores shared with ~8 agents (load avg 5-12 during runs),
rustc/cargo 1.97.1. **Not an M1**; all timings below are x86 and indicative only. Repo tree left
clean (`git status --short` empty). Scratch artifacts: `exp/verify/`.

## 1. Executive verdict

- **All three crates build and pass on x86: integer 24/24, tokenizer 3/3, training 68/68; 0 failed,
  0 ignored.** None of these unit tests needs owner artifacts; all loaded-artifact checks (packed
  models, `tables-1`, serving bundles under `/Users/casey.allard/uor-r4-investigations/...`) cannot
  run here. [MEASURED]
- **No correctness bug found** in the training model or integer runtime: causality, write order,
  pointer copy, RMS norm, age indexing, NoRead slot, Hamilton product, Householder pair, STE,
  AdaRound, AdamW and shard averaging all check out. The integer runtime is **bit-identical** to a
  scratch re-implementation using hardware multiply/divide over 256 steps x 3 reps x 2 modes x 2
  arms. [SOURCE + MEASURED]
- **"Multiplier-free" is true at instruction level but is achieved by software shift-and-add
  multiplication (~97k activation products + ~9.5k long divisions per token).** On x86 the
  repo's step is **4.4x slower** than the same arithmetic with hardware `mul`/`div` (6.73 vs
  1.53 ms/step mean, bit-identical outputs). The repo itself reports the optimized integer step at
  ~6.5x the F32 step on M1 (`docs/integration/integer-serving-result-2026-09-25.md:82-84`).
  A per-section profile puts ~79% of step time in software mul/div, mostly the attention value mix
  and the vocabulary mixture; the 1e-8 uniform mixture alone costs 4,096 software multiplications and
  4,096 long divisions per token. The constraint, as implemented, costs time and very likely energy.
  [MEASURED x86; HYPOTHESIS for M1 energy]
- **The "signed4 product table" is a per-activation 16-entry multiples table (x·k, k∈[-7,7]),
  128 B per input coordinate, indexed by the weight nibble** — D0-b-compliant (built with
  shifts/adds, used by table reads), but it replaces exactly one multiply per lookup (it does not
  tabulate partial sums over groups of activations) and on x86 it is no faster than a plain scalar
  hardware MAC (1.27 vs 1.30 ms/step NoRead). [SOURCE + MEASURED]
- **Per-token access is 100% dense:** 1,672,960 four-bit weight reads/token (every 4-bit weight,
  plus the input row), of which **62.7% is the full 4096x256 tied output embedding**. 848 KB/token
  if packed; **3.36 MB/token as implemented** (codes held as `i16`). KV read at 255 events:
  326 KB (i32). D5 non-compliant, as the repo already states. [SOURCE + DERIVED]
- **The current model contains none of the prime/zeta/H4/2I/Z[phi]/Hopf machinery.** Its only
  geometric element is a per-4-lane unit-quaternion left multiplication; its matched control is a
  Householder-pair simple rotation. The 2I group table is correct (repo test source reviewed plus my
  independent recomputation; the repo's own core tests timed out here) but unused by the D8 path.
  [SOURCE + MEASURED]
- Parameter count confirmed: **1,678,466 total; 1,048,576 (62.5%) tied embedding; 629,890
  non-embedding.** [DERIVED from `crates/uor-r4-integer/src/config.rs:59-84`]

## 2. Findings

### F1. Build and test results [MEASURED]

| Command (all with `CARGO_TARGET_DIR=<scratchpad>/target`) | Result | Time |
|---|---|---|
| `cargo test -p uor-r4-integer --release -j2` | 24 passed, 0 failed, 0 ignored (bins/doc: 0 tests) | 31.0 s total (30.5 s build incl. git fetch) |
| `cargo test -p uor-r4-tokenizer -j2` | 3 passed, 0 failed, 0 ignored | 11.1 s |
| `cargo test -p uor-r4-training --lib --release -j2 -- --test-threads=2` (`RAYON_NUM_THREADS=1`) | **68 passed, 0 failed, 0 ignored** | build 8 m 45 s (525 s, incl. uor-r4-core release), tests 14.42 s |
| `timeout 1200 cargo test --release -p uor-r4-core --lib -j2 group_table -- --test-threads=2` | **NOT RUN**: uor-r4-core test-mode compile did not finish in 20 min (box load avg ~12); EXIT=124 | 1,200 s (killed) |

Warnings only (dead code in core `observed_text_session.rs:175`, `transferable_lexical.rs:1590`;
one `unused_mut` in a training test). Owner artifacts: `grep` finds `/Users/casey.allard/...` only
in docs/evidence, `crates/uor-r4-training/README.md:198` (#1017 snapshot) and core bins, e.g. packed models at
`.../learned-rounding-20260925/fit-*-rounding-3/packed-model`, tables at
`.../full-context-integer-20260925/tables-1`; `find / -name hard-model.json` returns nothing. So
`verify-integer` replay, `uor-r4-integer generate` and all language/NLL numbers are **not
reproducible here**; unit tests use synthetic fixtures. [MEASURED]

### F2. Where products are computed; the signed4 table [SOURCE]

- Weight x activation (dense maps): `model.rs:89-120` `low_bit_products` builds, per input
  coordinate x, `[0,x,2x,3x,4x,5x,6x,7x,0,-7x,...,-x]` (i64, from `<<`, `+`, `-`); `low_bit_dot`
  sums `multiples[(w as u16) & 15]`. Size 16x8 B = 128 B/coordinate (32 KiB for 256 inputs, 64 KiB
  for 512). Rebuilt on every `matrix_work` call: 10 calls/token, 3,328 coordinates, 426 KB/token.
  It replaced a branchy per-weight shift/add `match` (HEAD~1 `joint_integer.rs:89-107`); the
  reported 4.5-6.6x speedup therefore comes from removing per-weight branching/recomputation, not
  from a change of arithmetic class (mechanism [HYPOTHESIS]; the repo's profile put `low_bit_dot`
  at 83.4% of samples before the change, `integer-serving-result-2026-09-25.md:56-57`).
- Activation x activation: `model.rs:73-75` `product()` -> `math.rs:61-83` shift-and-add loop
  (up to 128 iterations over the smaller operand's bits, with overflow checks); `divide()` ->
  `math.rs:113-146` binary long division; `isqrt` radix-4. Used for: 3x RMS norms (768 squares +
  768 divisions), 2 blends (1,024), transport (1,280 quaternion / 768 Householder products, 256
  divisions), attention scores (64 x 255 = 16,320), attention read (256 x 255 = 65,280), output
  norm (256), vocabulary mixture (3 x 4,096 products + 4,096 divisions) and the two softmaxes
  (4,096 + 256 divisions). **~97k software multiplications and ~9.5k long divisions per token at
  255 events.** [DERIVED]
- Remaining source `*`,`/`,`%`: only `token_index * width` (`model.rs:288`), constant sizes, and
  loader-time decoding (`format.rs:228-229,329,337,347`). sampling.rs/generation.rs/tables.rs have
  none on values.
- D0-b status (`DECISIONS.md:96-102`): compliant with "no multiplier instruction" and "<=4-bit
  weights". D0-b's text does not address activation x activation products (16x16-bit attention and
  gates, 48x15-bit mixture); the implementation treats them as exact fixed-point computed by
  add/shift loops. The letter is satisfied; the energy rationale is not demonstrated (F4).

### F3. x86-64 disassembly census [MEASURED; scope: this x86 build only, static presence]

`objdump -d -C` (binutils 2.42) of repo-built `target/release/uor-r4-integer`
(sha256 `3b34599d...1590`), 76 step/sampler/generation symbols, 16,355 instructions:
6 `imul`/`mul`, 2 `div`, 1 `cvtss2sd`. Classification: 3 `imul` in `IntegerModel::step`
(token x width embedding-slice bounds), 1 `imul` in the `matrix_work` row closure (row offset),
2 `div` in `matrix_work` (`chunks_exact` count), 2 `mul` in `TextSession::{observe,generate}`
(Instant -> ns), `cvtss2sd` in serde parsing of `Selection` (request JSON, not model).
**Zero multiply/divide/float on model values**, agreeing with the M1 audit
(`docs/evidence/integer-serving-instruction-audit-2026-09-25.json`). The M1 binary is a
different artifact; this does not re-certify it.

### F4. Timing and the cost of software multiplication [MEASURED, x86, loaded shared box]

Harness `exp/verify/inttime` writes a random-weight artifact in the exact on-disk format
(plausible Glorot-scale exponents), loads it through `IntegerModel::load_with_tables`, runs 256
random tokens, best of 3. Variants: repo code; scratch copy with hardware `product`/`divide`;
scratch copy with hardware multiply plus direct MAC instead of the product table. All
distributions were bit-identical across variants in every run.

| ms/step | Read mean | Read pos 0-15 | Read pos 240-255 | NoRead mean |
|---|---:|---:|---:|---:|
| Quaternion, repo (software mul/div) | 6.73 | 4.26 | 8.25 | 4.25 |
| Quaternion, hardware mul/div | 1.53 | 1.30 | 1.53 | 1.27 |
| Quaternion, hw mul + direct MAC (no table) | 1.48 | 1.50 | 1.61 | 1.30 |
| Householder, repo | 6.62 | 4.23 | 8.14 | 4.30 |
| Householder, hardware mul/div | 1.43 | 1.32 | 1.53 | 1.39 |

Load (hash verification + decode) 32-36 ms. Attention at 255 events costs ~4.0 ms with software
products versus ~0.26 ms with hardware.

Per-section profile (scratch instrumented copies `exp/verify/{swprof,hwprof}`, quaternion, Read,
mean over 256 steps, best of 3; `exp/verify/profile_quaternion.log`):

| Section | software (repo) ms/step | % | hardware mul/div ms/step | % |
|---|---:|---:|---:|---:|
| attention read (mass x value) | 2.293 | 30.7 | 0.159 | 10.1 |
| copy + mixture + 1e-8 uniform + residual | 2.080 | 27.8 | 0.077 | 4.9 |
| output norm + tied logits (dense 4096x256) | 0.887 | 11.9 | 0.697 | 44.2 |
| vocabulary softmax (4,096 long divisions) | 0.887 | 11.9 | 0.150 | 9.5 |
| attention scores + softmax | 0.435 | 5.8 | 0.041 | 2.6 |
| transport + blend + rms | 0.264 | 3.5 | 0.034 | 2.1 |
| other dense maps (recurrent in/state, update, gates, key/value) | 0.624 | 8.4 | 0.419 | 26.4 |
| **total** | **7.48** | | **1.58** | |

About 79% of the repo's step time (5.9 of 7.48 ms, the difference from the hardware variant) is the
cost of software multiplication/division, concentrated in the attention value mix (2.1 ms) and the
vocabulary mixture (2.0 ms). The 1e-8 uniform mixture alone costs 4,096 software multiplications
by 99,999,999 and 4,096 long divisions per token for a 1e-8 effect. With hardware arithmetic, the dense
tied output layer dominates (44%). [MEASURED x86] Random weights, not the trained artifact; loop lengths of
the software multiply depend on operand magnitudes, so trained-model timings may differ. The owner's M1
reports 3.70 ms per full-window Read call.

### F5. Per-token parameter and memory access (Read, 255 prior events) [SOURCE + DERIVED]

| Tensor (shape) | Entries read/token | Access |
|---|---:|---|
| embedding input row [4096,256] | 256 | sparse (1/4096) |
| recurrent.input / .state [768,256] each | 196,608 + 196,608 | dense |
| read.query, read.key [64,256] | 16,384 + 16,384 | dense |
| read.value [256,256] | 65,536 | dense |
| update [256,512] | 131,072 | dense |
| read.no_read [1,256]; update.gate, copy.gate [1,512]; output.norm [256] | 256+512+512+256 | dense |
| **embedding as tied output [4096,256]** | **1,048,576** | **dense, full vocabulary** (`model.rs:367`) |
| 16-bit biases + full read.age (re-scaled each step) + output.bias | 5,762 | dense |

Totals: 1,672,960 four-bit reads (= 100.02% of 1,672,704 four-bit weights) + 5,762 sixteen-bit.
Bytes/token: packed 4-bit 836,480 + 11,524 = **848,004 B (828 KiB)**; as implemented (codes decoded
to `Vec<i16>`, `format.rs:334-355`) **3,357,444 B (3.2 MiB)** plus 426 KB of product tables.
KV: 255 x (64 key + 256 value) x 4 B = **326,400 B** as `Vec<Vec<i32>>` (163,200 B at 16-bit);
81,600 attention products. The whole model (3.2 MiB decoded) fits in an M1 P-cluster L2, so
per-token DRAM traffic after warm-up is likely small and compute (software multiplies) dominates
energy. [HYPOTHESIS]

Note: D5's text (`docs/integration/DECISIONS.md:253`) quotes 611,814 inspections/step for the Sept 24
served artifact; the current integer artifact reads 1,672,960 four-bit weights per step.

D5 tension (flag, not resolved): D5 requires reading a selected subset of rows per token; for
this model the cheapest large win is the output layer (62.7% of reads), which needs per-token row
selection, i.e. routing. That conflicts with the owner's "no sparse routing" unless deterministic
geometric address routing is explicitly distinguished from learned gating.

### F6. Correctness review of `joint_model.rs` and integer `model.rs` [SOURCE; checks MEASURED where noted]

Verified, no defect:
1. **Causality.** Sequential unroll; read uses `memory.keys` (events 0..t-1); the current key/value is
   appended after read/update (`joint_model.rs:1069-1077`); future read columns are zero-padded
   (`:824-833`); integer commits after the prediction (`model.rs:394-398`). Position t cannot read
   itself or the future. Repo tests `causal_normalized_read_and_no_read_distributions` and
   `precision_views_match_endpoints_sessions_and_causal_prefix` pass.
2. **Write order** read previous -> update -> write current: `joint_model.rs:1046-1077`, `model.rs:316-354`.
3. **Pointer copy** `copy[t,v] = sum_{j<t} a_{t,j}[x_j=v]` (`training_copy :1342`, `incremental_copy
   :1369`, integer `model.rs:375-378`); mixture `(1-g(1-a0))P_vocab + g*copy` sums to 1 because
   `sum_{j>=1} a_j = 1-a0` (`:1331-1339`, `model.rs:379-384`).
4. **RMS norm** `x/sqrt(mean x^2 + 1e-5)` (`:1690`); integer `normalize_state` algebra checked:
   output `x*2^14/sqrt(sum x^2 + 2^30*1e-5)` equals the Q11 -> Q10 form (`model.rs:413-428`).
5. **Age bias**: `age_order` reversed (`:282`) narrowed at `context-1-previous` (`:1023`) gives
   index `previous-1-j`; integer `age[previous-1-index]` (`model.rs:330`); max index 254 of 255.
6. **NoRead slot**: learned null logit from `rms(provisional)` competes in one softmax (`:1032-1033`);
   its mass reads a zero vector and lowers the copy fraction.
7. **Hamilton product**: numerically equals left multiplication `u*x` (max error 0 over 1,000
   random pairs; differs from right multiplication by up to 5.6). [MEASURED]
8. **Householder pair** equals `H(v)H(e0)x` (error 8.9e-16), det +1. Local-scale match
   `||Q-I||_F^2/||HH-I||_F^2 = 0.99999999905`. Structural difference: quaternion `R-I` has 4 equal
   singular values (isoclinic); Householder pair has 2 zero singular values (simple rotation).
   Both are 3-dof SO(4) families. [MEASURED]
9. **Integer constants**: `15,549,442,559,877 = round(2560*sqrt2*2^32)`; thresholds
   10,995,116 / 15,549,443 correspond to the 1e-6 minimum norm. [DERIVED]
10. **No detach in training** (`:1114`, `append_history :1677`); gradients reach query/key/value/
    recurrent/age/null (test passes).
11. **Loss** `-mean log p[target]`, targets = inputs shifted by one (`joint_campaign.rs:412-421`);
    uniform mixture `(1-1e-8)p + 1e-8/V` exact in both paths.
12. **STE** (`joint_quantization.rs:463-504`): forward exactly hard; gradient 1 inside the clip
    interval, 0 outside, `1-s` during ramp. **AdaRound** (`joint_rounding.rs:604-644`): rectified
    sigmoid `clip(1.2*sigma-0.1,0,1)`, penalty `1-|2d-1|^beta`, tie rule reproduces
    round-half-away. **AdamW** (`joint_optimizer.rs:369-392`): bias correction, global clip
    `g/max(||g||,1)` and decoupled decay are correct. **Shard averaging** (`joint_parallel.rs:117-150`)
    equals the full-batch mean.
13. **Integer arithmetic exactness at scale**: bit-identical to hardware mul/div (F4). [MEASURED]

Low-severity issues (performance/design; none changes outputs):
- L1 `format.rs:334-355`, `model.rs:28-31`: 4-bit codes held as `i16` (4x packed size); KV as `Vec<Vec<i32>>`.
- L2 `model.rs:319-320, 346-347, 353-354`: product tables rebuilt for identical inputs
  (query/no_read, update/update.gate, key/value).
- L3 `model.rs:214-225, 249-250, 321`: biases and the whole 255-entry age vector re-scaled each
  step; `format!` + `BTreeMap<String>` lookups and allocations in the hot path.
- L4 `joint_model.rs:908`, `model.rs:376`: the pointer can copy only `x_0..x_{t-1}`, never the
  current input `x_t` (design limitation).
- L5 `joint_optimizer.rs:389-391`: weight decay also applied to `output.norm.weight`, biases, `read.age`.
- L6 `joint_model.rs:1204-1211`: `selected_history`'s parameter named `training` actually receives
  `dense_batch_history` (misleading name; behaviour correct).
- L7 `generation.rs:241-249`: first-sentence stop fires on any `.` byte ("Mr.", decimals).

### F7. Geometry actually present in the served model [SOURCE]

`grep` of `crates/uor-r4-integer` and `crates/uor-r4-training` finds no use of group tables,
H4 roots, zeta phases, prime addresses, Z[phi] or Hopf maps. The contract string says "no hemisphere
folding or H4 codebook" and "not exact Z[phi]" (`config.rs:104,152`). The training crate uses
uor-r4-core only for report sealing, the BPE tokenizer and the answer oracle. Architecturally it is
a GRU-like recurrence with lane-wise rotation, a single-head soft read over up to 255 events with
age bias and a null slot, and a pointer-sentinel-style copy mixture.

### F8. Quaternion arm vs control [SOURCE: current-state; DERIVED]

Both arms apply a 3-parameter rotation per lane with matched local scale. The quaternion family is
a group (left multiplication is a homomorphism: `L(p)L(q)=L(pq)` checked numerically); the
Householder-pair family is not closed (0/200 random compositions `HH(a)HH(b)` stay in the family).
[MEASURED] Group closure, together with isoclinic versus simple rotation, is the only mathematical
distinction under test, and the control is slightly better on the reported tail NLL (2.085 vs
2.110). [SOURCE current-state]

### F9. 2I group table [SOURCE + MEASURED]

`crates/uor-r4-core/src/native_geometric/learner/group_table.rs`: 120x128 (stride-padded) u8 product table
+ inverse row, built once in f64 from `canonical_h4_roots_q30`, classified by signed nearest element. Repo
tests assert Latin square, associativity for all 1,728,000 triples, two-sided identity, right
inverses and residual < 1e-6. A monoid with right inverses is a group, so the claim is sound. My
independent numpy rebuild of the same construction (`embedding.rs:22-90`; all 12 listed
permutations are even): 120 distinct unit roots, closure residual 0.0, Latin square, associative for
all 1,728,000 triples, element-order census {1:1, 2:1, 3:20, 4:30, 5:24, 6:20, 10:24} = 2I.
**Confirmed by source review and independent recomputation.** The repo's own `group_table` tests were
NOT RUN here (core test build timed out, F1).

## 3. Strong vs weak

Strong: disciplined, exact integer semantics with overflow-checked arithmetic, artifact binding and
sealed I/O; training/serving numerical agreement; honest scope statements ("dense", "not
whole-process", "no energy"); the loss pipeline and its dependencies are correct; the 2I table is
a correct exact group implementation.

Weak or irrelevant to the goal: software multiplication buys instruction-level compliance at a 4.4x
time cost (x86) with no demonstrated energy benefit; the product table is a per-element multiply
replacement that does not amortize work across activations; the served model is dense (100% of weights per token) and 63% of the
reads are the output embedding; the geometric primitives the project is named for are absent from
the model being trained; and the only geometric component loses to its matched control.

## 4. Recommendations (ranked)

1. **Measure energy per token for three kernels on the M1**: repo software multiply, hardware
   multiply (my scratch variant; bit-identical) and a NEON int8/int4 SDOT kernel, using
   `powermetrics`. Cost: ~1 day. Falsifier: if software multiply is not lower in J/token, drop the
   shift-add requirement for activation products or re-scope D0-b to "no float; <=4-bit weights".
   Expected: software multiply loses by several times. [HYPOTHESIS]
2. **Remove pure overhead from the serving step**: apply the 1e-8 uniform floor analytically at
   selection time instead of 4,096 multiply/divide pairs per token (28% of the software step
   together with the copy mixture); pack 4-bit codes in RAM and store KV as i16; stop rebuilding
   duplicate product tables; hoist bias/age scaling out of `step`. Cost: hours. Falsifier: the
   distribution hashes of the existing replay (or this harness) must stay bit-identical except where
   the uniform floor is deliberately redefined. Expected: 4x fewer parameter bytes/token (3.36 MB to
   0.85 MB) and a large step-time cut. [MEASURED x86 for the cost shares]
3. **If D5 is to be pursued, target the output layer first** (62.7% of reads) with a two-level or
   address-routed vocabulary. Owner must first decide whether deterministic geometric row selection
   counts as forbidden "sparse routing". Falsify cheaply by measuring top-1/NLL loss with exact
   top-K recall on held-out text before any retraining.
4. **Stop describing the quaternion lane rotation as the project's geometry** until a geometric
   primitive (2I-quantized transport, H4 codebook, zeta phase) is actually in the trained model
   and beats its matched control on the D6 probe.
5. Fix L6/L7 naming/heuristic issues when the files are next touched (trivial).

## 5. Open questions

- M1 energy and M1 timing of the hardware-multiply variant (not measurable here).
- Whether the trained artifact's operand magnitudes change the software-multiply cost ratio
  (random weights used here).
- Whether the owner's "no sparse routing" excludes deterministic address routing (decides D5 feasibility).
- NLL/quality numbers cannot be reproduced without the owner's artifacts and data.

## 6. Resources used by this agent [MEASURED]

Compile: integer 31 s, tokenizer 11 s, training 525 s (+14 s tests), CLI 2 s, scratch harness/profiler
~115 s; core test build killed at 1,200 s (not run). Model runs ~1.5 min single-thread. Storage: shared
`<scratchpad>/target` ~1.1 GB (shared with other agents), `exp/verify` ~8 MB. No repo edits; no
network use beyond crates.io/GitHub dependency fetches by cargo.
