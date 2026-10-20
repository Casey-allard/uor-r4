# Shared packed coefficients for the native integer runtime

References #973. **Status: implementation and focused checks complete; actual
loaded-output and cost comparison pending.** Keep this delivery as a draft until
that comparison is reviewed. No model is promoted and no speed, RSS, energy or
general-language improvement is established by this preparation.

The live [complete-prefix dialogue study](dialogue-prefix-study-2026-09-27.md)
continues unchanged. This independent serving work removes unnecessary retained
coefficient expansion while preserving the supported learned computation. It
does not widen the integer loader to the separate width-576 offline learner.
The [source-bound packet](../evidence/packed-integer-preparation-2026-09-27.json)
contains review, build, instruction-range and resource receipts.

## Change and architectural scope

The artifact decoder already validates a low-nibble-first signed4 format but
returned expanded `i16` vectors. The integer model then retained those vectors
in its parameter map and cloned them into cached hot-parameter fields. The new
private `packed_rows` module compiles decoded signed4 arrays once into shared
`Arc<[u8]>`; signed16 arrays use shared `Arc<[i16]>`. Loader expansion remains a
temporary operation. Existing public decoder, artifact identities, metadata,
scales and accepted width guards remain unchanged.

Embedding and normalization read signed codes in place. Vocabulary, recurrent,
read and update matrix kernels consume packed bytes directly, including the
existing eight-row width-256 and four-row width-512 blocking. Supported model
widths remain 128/256; width512 is the concatenated update input. No per-row
expanded buffer is introduced. The ordinary `IntegerSession` and separate
partitioned `SessionState` reach these shared kernels without changing their
memory, attention or session policies.

`IntegerModel::coefficient_storage()` reports each shared parameter payload once:
signed4 coefficient count/packed bytes and signed16 coefficient count/bytes.
It excludes scales, caches, allocator metadata, loading temporaries and session
state. It is neither an RSS nor a physical memory-traffic measurement.

Each signed4 coefficient occupies a nibble instead of an expanded two-byte
element, with the artifact's zero padding for odd lengths. Cached references now
share allocations. This still accesses dense parameter rows and leaves their
coefficient count unchanged. It is an interim D0-b implementation improvement,
not D5 selected parameter access or evidence of geometric predictive advantage.

## Executed preparation

Runtime source is pinned at
[`d0844ba0c1e7a22803c18767796336bfb8ab376e`](https://github.com/UOR-Foundation/uor-r4/commit/d0844ba0c1e7a22803c18767796336bfb8ab376e),
based on merged `0ac3df46`. The unchanged baseline is `9d78ebad`; its complete
tree equals that merge, and the integer source equality was rechecked before
building. Both use Rust 1.97.1 on `aarch64-apple-darwin`, release optimization and
the same local dependency cache. Frozen baseline and candidate binaries for
`uor-r4-integer`, `verify-integer` and `uor-chat` are bound by SHA256 in the packet.

Independent arithmetic and integration reviews found no concrete numerical
defect. They inspected signed nibble order, forbidden/reserved codes, row bounds,
accumulation range, preserved scaling/rounding, all parameter consumers and the
actual shared allocations. Four focused release tests pass:

- Sign decoding, low/high order, odd padding, bounds and shared storage.
- Extreme signed inputs and independent decoded scalar accumulation.
- All admitted matrix dispatches, non-block-aligned row tails, scales/biases,
  embedding rows and vocabulary projection.
- Eight-row width-256 accumulation against an independent scalar reference.

The reference multiplies exist only in tests. Rustfmt on the changed numerical
files and the whitespace check pass. The release library and all three CLI
binaries compile. This work ran **zero additional learned-model forwards,
training updates or generated replies** while the dialogue pair occupied the
single owned model slot.

The review also repaired the instruction auditor's moved-symbol matcher and
included emitted callers of the inline block kernels. Actual matched symbol
names, instruction counts and missing ranges remain explicit:

| Frozen artifact | Matched ranges | Forbidden instructions | Mandatory missing |
|---|---:|---:|---:|
| Baseline ordinary CLI | 16 | 0 | 3 |
| Candidate ordinary CLI | 16 | 0 | 3 |
| Baseline conversational CLI | 28 | 0 | 0 |
| Candidate conversational CLI | 28 | 0 | 0 |
| Candidate library archive | 30 | 0 | 0 |

The three ordinary-CLI omissions are unchanged conversational helper symbols;
its global mandatory-symbol audit therefore remains a coverage failure. The
conversational executable and archive include `project_vocab_with_products_into`,
`matrix_work_direct_into`, `affine_direct_into` and `step_conversational_into`.
All matched ranges have nonzero instruction counts. No multiply, divide or
floating-point opcode was detected in them. Absent optional ranges are reported
as not independently audited; this does not certify a transitive call graph or
whole-process multiplier freedom.

Baseline compilation took 16.376 seconds plus 4.098 seconds for the conversational
caller. Candidate focused checks/build took 31.079 seconds, with peak child RSS
343,310,336 bytes and maximum sampled process-group RSS 365,232,128 bytes. These
are build costs, not inference latency. The packet retains the complete overlapping
elapsed charge through 09:32:54 UTC: owned 32,390,265 ms, shared 680,545,845 ms.
The 18-hour owned ceiling and verified 722,400,000 ms shared allowance remain;
the foreign ledger's larger reported limit is preserved but not adopted.
The source/build preparation projection allows four hours and 1,800 seconds of
build/check work, subject to the parent dialogue phase limits, one Cargo process,
two build jobs, 4 GiB compile RSS, 64 MiB new internal preparation material,
512 MiB shared SSD growth and the existing physical reserve plus 128 MiB margin.
Overlapping fit, research and build time is charged once. Delivery time follows
this snapshot and remains in the cumulative ledger.

## Next decision, after the dialogue pair

Use a separate prospective replay/cost projection when the owned model slot is
free. Reuse the existing accepted quaternion and Householder-pair bundles, their
retained full-context replay inputs and source/continuation requests. Compare
the frozen baseline and candidate on exact probability/state hashes, generated
token decisions and RNG state, with the same windows and decoder settings.
Keep historical baseline discrepancies separate from this optimization.

The existing `uor-chat` stdin and save interface can additionally compare two
actual turns, saved partitioned state, exact history and sampler cursor. It
does not expose per-step distributions; state/output equality there must not be
reported as full distribution equality. No new broad evaluation framework is
needed for that check.

Record model-step and complete-workload cost on the same input and explain
whether reduced coefficient storage improves actual serving. If outputs change,
diagnose the first divergent operation. If outputs agree but the packed layout
slows the workload, retain the negative and decide whether its storage benefit
justifies the tradeoff. Four unit checks alone do not authorize promotion. Do
not change the running dialogue study, repeat its dose, or turn unrelated
historical failures into this serving task.
