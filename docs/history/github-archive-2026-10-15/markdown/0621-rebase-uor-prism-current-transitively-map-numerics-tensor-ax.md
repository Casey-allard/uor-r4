# #621 — rebase(uor-prism): current transitively — map numerics/tensor axes to compiler-side use; one-owner decision vs uor-matmul

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T22:54:01Z
- **Updated:** 2026-08-12T00:06:20Z
- **Closed:** 2026-08-12T00:06:20Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/621
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase, step 3: uor-prism).

## Audit result (measured 2026-08-11)

r4 carries `uor-prism 0.4.0` **transitively only** (via uor-addr), from crates.io; upstream v0.4.0 tag = `main` HEAD (2026-05-24). **Version verdict: current; nothing to rebase.** Subcrates in our lock today: `uor-prism`, `-crypto`, `-fhe`, `-numerics`, `-tensor` (+ `-verify` upstream, not in our graph).

## Feature-mining map (the actual work of this issue)

Prism is the Layer-3 standard library over the Framework axis system we already use (`uor_foundation_sdk::axis!` — r4 declares its own custom axis in `crates/uor-r4-router/src/lib.rs`). Surfaces relevant to r4:

1. **`prism-numerics`** — `BigIntAxis` (modular big-int over 2^(8N)), `FixedPointAxis` (Q-format on 64-bit containers, e.g. Q16.16/Q32.32), `FieldAxis`, `RingAxis`; all kernels `(input: &[u8], out: &mut [u8])`, bit-deterministic. Candidate to replace any hand-rolled fixed-point/modular arithmetic in the **compiler/certifier** (floats and allocation are legal there) with spec-grounded, deterministic axes. NOT for the deployed kernel (P-4 bans multiply/divide/float there).
2. **`prism-tensor`** — `TensorAxis` with `CpuI8MatmulSquare<DIM>` (i8×i8→i16 integer matmul, bit-deterministic) and `ActivationAxis`. Same placement rule: legal compiler/teacher/certifier-side, banned in the deployed kernel. Overlaps uor-matmul's territory — the uor-matmul sub-issue (step 4) should own the "which exact-matmul library where" decision so we adopt ONE, not both; this issue records the prism option as input to that decision.
3. **`prism-crypto`** — Sha256/blake3/sha3/keccak hashers already reach us through uor-addr's σ-projections. Note: prism-crypto's blake3 must stay compatible with our repo-wide `blake3 = "=1.5.0"` pin (it resolves in-lock today, so any future prism bump must be checked against the pin).
4. **`prism-fhe`** — no r4 use case identified; catalog only.
5. **`uor-prism-verify`** — not in our graph; the Framework-level `uor-foundation-verify` covers our replay-certification test. No action unless prism-level witness verification becomes wanted for router routes (would pair with the witness-proof work Casey named as a goal — record the option).

## Deliverables

- [ ] Inventory of compiler/certifier-side arithmetic that could sit on prism-numerics axes (file follow-up per site; adopt only where it simplifies, not for its own sake)
- [ ] Input recorded to the uor-matmul sub-issue: prism-tensor vs uor-matmul overlap, one-owner decision
- [ ] Direct-dependency decision: only add `uor-prism-*` as direct deps at first real use site (keep the graph honest; transitive presence costs nothing today)

## Verification

Any adoption PR: four local gates + full suite green; P-4 deployed-kernel scan untouched; determinism invariant (identical pinned inputs ⇒ identical artifact bytes) re-verified on any compiler-side arithmetic swap.


## Comments (1)

### comment 5260321410 — 2026-08-12T00:06:20Z — Casey-allard

## Deliverables complete — closing with record (2026-08-12)

**Version verdict (unchanged from audit):** uor-prism 0.4.0 transitive-only via uor-addr; v0.4.0 = upstream main HEAD; after #619 its uor-foundation/-sdk deps unify with ours. Nothing to rebase.

### Deliverable 1 — compiler/certifier arithmetic inventory (measured by source scan)

One genuine prism-numerics candidate found, evaluated, **not adopted — with reason**:

- `LoweredFixedPointScore::quantize_q88` (`crates/uor-r4-graph-compiler/src/lower_semantic_regions.rs`) — Q8.8 fixed-point on an **i16** container, with NaN→`None` (R5 absence semantics) and explicit saturation flags. Prism's `FixedPointAxis` (`FixedPointQNumeric<INT_BITS, FRAC_BITS>`) runs on a **64-bit container** with `(input: &[u8], out: &mut [u8])` kernel signatures — not a drop-in for the i16 program format, and the bespoke quantizer (~30 lines, own tests, R5-policy-entangled) is *smaller* than the axis harness it would sit in. Adopting would add machinery for zero simplification. Recorded; revisit only if a 64-bit Q-format need appears.

Remaining wrapping/`u128`/`i128` sites (observation hashing mixes, rate-distortion internals, certify `fmm.rs` exact accumulators, score_runtime) are deliberate exactness/hash patterns, not modular-bignum arithmetic wanting `BigIntAxis`. If any of them generalize into accumulation-over-coded-operands, the ecosystem answer is uor-matmul's accumulator/Limbs discipline — that belongs to #622, not here.

### Deliverable 2 — one-owner decision input (recorded on #622)

**uor-matmul owns exact matmul/accumulation for r4** (Casey's stated design intent, 2026-08-11); `prism-tensor`'s `TensorAxis`/`CpuI8MatmulSquare` is NOT adopted for that role. Rationale beyond the directive: uor-matmul is `no_std`+zero-alloc with the codec-tier system (Sign/Ternary/Book/E8) r4's stores actually use, ships its own κ-manifest identity, and covers the tropical semiring; prism-tensor is a square-i8 reference axis without the codec layer. Comment posted on #622.

### Deliverable 3 — direct-dependency decision

**No direct `uor-prism-*` dependency now.** Transitive presence costs nothing; first real use site (none exists after this inventory) is the trigger. `prism-crypto` hashers keep reaching us through uor-addr's σ-projections; blake3 `=1.5.0` pin compatibility holds in-lock.

No code changed under this issue; gates unaffected; zero-matmul runtime untouched. `uor-prism-verify` option (prism-level route-witness verification) remains recorded in the issue body for the future witness-proof work.


---
_Comment/review ids in this file: 5260321410_

