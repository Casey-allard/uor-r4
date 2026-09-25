# Standalone full-context integer serving

September 25, 2026. References #973 under #820. **Retain this serving implementation
and exact arithmetic optimization. Language quality is unchanged.**

## What now runs

[`uor-r4-integer`](../../crates/uor-r4-integer/README.md) packages and loads the
accepted learned-code quaternion and ordinary Householder models, tokenizes real
prompts, retains a text session, and generates tokens using integer greedy or
categorical selection. Its dependency graph excludes the training crate, Candle,
core, model-source and external model providers. The training evaluator reuses
this same numerical implementation. The existing byte-BPE engine and report-root
implementation are shared rather than forked.

Training / evaluation / session context stays **256 /256 /256**. Every causal
occurrence remains available: full admission reaches255 previous tokens. The
64-coordinate reader is a representation dimension. No weights, scale, numerical
table, admission policy or acceptance threshold changed; no fitting occurred.

Bundles bind the unchanged accepted parent files and table bytes, the original
tokenizer SHA256 and canonical identity, and BOS0/EOS1. Loading verifies the sealed
file set and hashes. Runtime source paths are provenance only. Text-session
continuation consumes each pending output once; capacity errors are prospective.
Appended text is tokenized in finalized segments, not arbitrary byte chunks.
Sessions are in-memory; report files are not restart checkpoints.

Categorical selection is a declared new temperature1 policy on Q48 masses, with
top-k, deterministic ties, explicit xorshift state and bounded rejection sampling.
It does not reproduce the old floating temperature sampler's seeded outputs.
The CLI accepts independent requests; the Rust API retains state across calls.

## Exact retained computation

The [independent saved-data comparison](../evidence/integer-serving-comparison-2026-09-25.json)
records zero differences for both the extracted and optimized binaries:

| Comparison | Result |
|---|---|
| Existing four256-token windows, Read/NoRead, both arms | All4,096 complete Q48 distribution hashes and greedy predictions match |
| Existing source panel, both modes and arms | All128 prompt/output token rows and341 decision hashes match |
| Exact complete source answers, quaternion /ordinary | 28/32 /24/32; zero previously correct rows lost |
| NoRead complete answers | 0/32 in each arm |
| Loaded multi-call session versus raw occurrence replay | Both pass, including pending-token consumption and capacity rejection without mutation |
| New integer categorical continuations before/after optimization | All10 outputs, distributions, stops and sampler states match; only timing excluded |

This is extraction and optimization verification on existing exposed material.
It is not another language population, training dose or fresh final holdout.
The exact normalized output and attention masses and all causal-slot counts are
checked during the executed paths. The saved comparison binds files by SHA256
and checks complete inventories/lengths; BLAKE3 seal verification is performed
by the Rust producer and loader.

## One measured optimization

A native three-second sample of the actual standalone quaternion continuation
places `low_bit_dot` at the top of1,847 of2,214 sampled stacks (**83.4%**).
The single change precomputes each input coordinate's small signed multiples
using shifts/add/subtract and reuses them across dense rows. The largest current
temporary table is64KiB. Accumulation fits i64 even at the declared extreme
inputs; no rounding or parameter changes are introduced. The compiler uses
multiple exact accumulators, whose regrouping preserves the bounded integer sum.

| Same workload, batch1; two arms concurrent | Quaternion before → after | Ordinary before → after |
|---|---:|---:|
| 64 source requests, whole CLI through JSONL | 52.973s →8.016s | 53.154s →8.006s |
| Five categorical continuations, whole CLI through JSONL | 11.934s →2.624s | 10.765s →2.141s |
| Continuation generated tokens, unchanged | 640 | 578 |
| Continuation model calls, including prompt ingest | 760 | 698 |
| Full-window Read model call average after optimization | 3.695ms | 3.698ms |

Observed complete-CLI speedups are **6.61–6.64x** on source requests and
**4.55–5.03x** on continuations. These are single ordered before/after observations
on this M1, not repeated benchmark confidence intervals. The quaternion
continuation is sampled by the profiler in both runs. Timing includes loading,
tokenization, model work, selection, decision hashing and JSONL writes, and
excludes final summary/sealing. External paired-process receipts include exit
and sealing: source53.819s→8.110s, continuation12.195s→3.053s,
replay32.573s→7.122s. Do not add nested model clocks to total wall time or divide
by output tokens while ignoring prompt ingest.

The full-window optimized integer step remains about6.5 times the historical
matched F32 step; that is a cross-run indication, not a newly controlled F32
benchmark. Dense parameter access and allocation remain. No physical energy or
D5 parameter-sparsity result follows from this speedup.

## Numerical instruction scope

The [final audit](../evidence/integer-serving-instruction-audit-2026-09-25.json)
inspects55 symbol ranges /12,756 ARM64 instructions in the exact optimized CLI:
model42 ranges, sampler9, generation4. It finds no floating arithmetic/conversion
and no hardware multiply/divide on model values, probability masses or PRNG
state. Product-table construction compiles to shifts/add/subtract/negation/stores.
One `fmov` transfers an integer SIMD result.

**Whole-process multiplier-free execution is not claimed.** Six model
shape/address operations, one top-k sort address operation and six nanosecond
timing conversions use multiply/divide. Metadata parsing, tokenizer, hashing,
allocation and system libraries remain outside the audited numerical boundary.
This implements the useful numerical part of D0-b; complete-path obligations
remain explicit.

## What this says about attention and language

The combined learned context path operates with full access and preserves its
source-sensitive outputs. R4 quaternion transport executes within the model.
The ordinary control also works. This does not establish geometric superiority,
far-distance retrieval, an isolated value-feedback benefit, Hamiltonian dynamics
or an exact invertible summary of history. The occurrence tape stores history;
the recurrent state is a learned finite representation.

Actual [continuations](integer-serving-outputs-2026-09-25.md) still confuse
entities, grammar and events. For example, the quaternion model says “We have
provide for us!” and ends a story mid-thought. The ordinary model also loses
roles and event consistency. Faster identical output cannot repair those errors.

The next integrated milestone is **coherent source-conditioned language
continuation through this same integer session**. Existing continuous learning
curves are still improving; capacity saturation or defective attention has not
been demonstrated. Resume meaningful full256 language learning with the same
architecture/objective and ordinary arm, then carry that candidate once through
the established export. Freeze the complete learning/export/evaluation budget
and a compact output-quality decision before launch. If likelihood improves
without useful output improvement, compare objective/data coverage and capacity
explicitly; do not automatically repeat exposure or return to selector sweeps.
The [current work card](current-state.md#active-execution-contract) owns this next
action. No new multi-hour fit is launched by this implementation result.

## Delivery evidence and resources

- Initial serving source: `20db6e49fe611d96e525c891831b9ff882f121f4`.
- Multi-call verifier: `bdbec26a` (full identity in the bound result).
- Optimized source: `be223fc5` (full identity in the bound result).
- Optimized CLI SHA256: `3811173d3e4d1b4415d41dfc363c772e22c994542e8fe2e05ef3ae089c65b182`.
- [Report summaries, artifacts, executable hashes and process receipts](../evidence/integer-serving-result-2026-09-25.json).
- [Prospective work card](../evidence/integer-serving-plan-2026-09-25.json),
  [DeepSeek review adjudication](../evidence/integer-serving-review-2026-09-25.json),
  [cumulative closeout](../evidence/integer-serving-closeout-2026-09-25.json).

RDC ran one substantive DeepSeek review and paired Rust processes on this same
Mac; it supplied concurrency, not additional hardware. Three independent agents
covered extraction, tokenization, arithmetic/audit, source review and comparison.
No Kimi model, paid external training or source/artifact deletion occurred.

Executed checks:24 integer plus3 tokenizer unit checks pass; the extended
signed4 arithmetic check passes; release bins build; training compatibility
check with original `metal,cpu-accelerate,reference-accelerate` features passes.
Two requested core integration targets implicitly compiled unrelated core bins
and were canceled before execution: **NOT_RUN**, not PASS. Loaded comparisons
cover the changed tokenizer/session boundary; no replacement blanket suite was
run. Queue compatibility acknowledgements do not constitute validation.
