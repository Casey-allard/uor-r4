# #622 — rebase(uor-matmul): adopt as the exact-accumulation library — tropical route composition, E8 codebook unification, teacher-side exact GEMM

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-11T23:02:32Z
- **Updated:** 2026-08-12T01:07:21Z
- **Closed:** 2026-08-12T01:07:21Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/622
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent: #589 (ecosystem rebase, step 4: uor-matmul).

## Design intent (Casey, 2026-08-11 — supersedes the audit comment's "compile-side only" line)

uor-matmul is matrix multiplication reverse-engineered *for* the transformerless architecture — exact accumulation over coded operands, no real floating-point matmul — and **it is the library this project wants to use for the mechanisms that need it**. The standing prohibition is real FP matmul + transformer gating. r4's deployed-kernel invariant (P-4: no multiply/divide/float in the runtime kernel) is stricter and stays authoritative: placement per mechanism is the work of this issue.

## Audit result (measured 2026-08-11)

Not in r4's dependency graph at all. Upstream: no releases/tags; HEAD `b13c984` (2026-08-11 — actively developed); 9 crates; `no_std`, **zero heap allocation**, wasm + embedded (Cortex-M parity harness CB-11) — matches r4's C1-style constraints natively; rust-version 1.89 (AVX-512 VNNI kernels), covered by our 1.97.1 pin. License Apache-2.0 (deny.toml allowlist check at adoption). Dependency must be a **git rev pin** (nothing on crates.io).

## Surfaces mapped to r4 mechanisms

1. **Tropical semiring (`Trop<i8>`/`Trop<i64>`, `TropAcc`)** — max-plus accumulation: `max_p(a_p + b_p)` is integer add + compare ONLY, no multiply. Potentially legal **inside the deployed kernel under P-4 as written**. Route composition over additive costs (geodesic/tangent-space path composition) *is* tropical matmul. This is the runtime-relevant piece and the most direct "matmul mechanism without matmul" win.
2. **E8 codebook (`uor-matmul-codec::e8`)** — 256 codewords × 8 × i8 (2048 B, flash-resident, code 0 = origin, norm-sorted D8 shell + half-integer coset, self-verified by `e8_is_the_lattice_it_claims`). Convergence with #395's E8-256 store (measured recall@10 0.634 vs sign 0.541 at matched 288 bits): one canonical, verified table across the ecosystem instead of r4's hand-derived one. NOTE: swapping tables changes store bytes ⇒ κ re-pin with record, per measurement discipline.
3. **Codec tier family** (`Identity/Grid/Packed/Book/Sign/Ternary/Offset/Runs/Transcode`, one `Codec` trait, `MAX_BLOCK` unifies scalar/block) — r4's sign codes and E8 codes become *tiers*; tier transcode is provably decoded-stream-identical (CL-MM01, gated by CK-02/CK-05).
4. **Exact GEMM (`slice::gemm`, view API with arbitrary strides)** — i8→i32 etc., accumulator width a compile-time function of element type (overflow *unreachable*: i8→i128 … i64→Limbs<3>), same bytes on every route/host (CD-22), caller-supplied scratch (never allocates). Teacher/compiler/certifier-side exact arithmetic with r4's determinism invariant satisfied by construction. #616's batched teacher inference is the first candidate call site to examine.
5. **κ manifests (`uor-matmul-codec::kappa`)** — weight-artifact identity = κ of canonical manifest (not code bytes), sized inside uor-addr-1 ceilings; transcodes are visibly distinct artifacts with provably identical decoded streams. Bridges directly to our uor-addr dep and the kappa-registry sub-issue.

## Work items (ordered, cheap-check-first)

- [ ] 1. Deployed-kernel legality memo: confirm by source read + P-4-scan dry run that the tropical path compiles to add/compare/table-read only; if yes, file the tropical route-composition experiment as its own measured issue (predeclared criterion before running)
- [ ] 2. Add git-pinned dep to compiler/certify/model-source crates only; four gates + full suite; deny.toml license entry
- [ ] 3. E8 unification decision: adopt upstream table in the #395 store lineage (κ re-pin + record) or record why not
- [ ] 4. Teacher-side survey: which existing f32/hand-rolled accumulations (incl. #616 batching) become exact gemm; profile before optimize
- [ ] 5. One-owner decision (input from #621): recommend uor-matmul owns exact-matmul; prism-tensor not adopted for this role
- [ ] 6. Upstream is unreleased and moving (HEAD merged today): pin discipline + a re-audit note in the PR that bumps the rev

## Safeguards

Zero real-FP-matmul in the router runtime; P-4 scan untouched (any tropical runtime use must PASS it, not amend it — amending the invariant is a Casey/team governance decision, filed separately if ever needed); allocation census stays green; any κ-bearing artifact change ships with a re-pin record.


## Comments (3)

### comment 5260321300 — 2026-08-12T00:06:19Z — Casey-allard

One-owner decision recorded from #621 (2026-08-12): **uor-matmul owns exact matmul/accumulation for r4**; prism-tensor's TensorAxis/CpuI8MatmulSquare is not adopted for that role. Basis: Casey's design intent (uor-matmul is the library for the mechanisms that need it) plus fit — no_std/zero-alloc, the codec-tier system (Sign/Ternary/Book/E8) our stores actually use, κ-manifest artifact identity, and the tropical semiring path. Work item 5 of this issue is therefore resolved in advance.

### comment 5260339350 — 2026-08-12T00:08:54Z — Casey-allard

## Work items 1, 3, 4 — records (2026-08-12, source-audit level; no code changed)

### Item 1 — deployed-kernel legality memo: tropical path is P-4-compatible arithmetic; adoption route decided

Source audit of `uor-matmul-core/src/tropical.rs` (rev `b13c984`):

- `⊗` = `Trop::mac`: `acc.offer(x as i128 + y as i128)` — one **widening integer add**, exact for every representable pair; the `-inf` absorbing arms perform *no arithmetic at all* (deliberately, per their C6 note).
- `⊕` = `TropAcc::offer`/`combine`: **one comparison** (`if v > self.v`). No addition, no carry, no depth term — accumulator width is k-independent (their CA-04: `Trop<i8>` = 10 bits at any depth).
- No multiply, divide, remainder, or float anywhere on the path. **Every operation is in r4's legal runtime vocabulary** (int add/compare per the P-4 allowlist).

**Scan-authority nuance that decides the adoption route:** our P-4 gate is a *source scan over r4's own runtime modules* — it cannot see dependency source. Calling into uor-matmul from the deployed kernel would make the invariant hold by trust instead of by scan. Decision:

- **Route (a) — adopted:** implement the tropical kernel *in r4 runtime source* (it is ~a widening add + a comparison + the `-inf` guard; trivially small), so the P-4 scan keeps full authority. uor-matmul enters as a **dev-dependency only** for differential parity: r4's kernel must byte-agree with `uor_matmul_core::dot_tropical_ref` on the conformance corpus. Shipped runtime dep graph stays untouched.
- Route (b) — extending the scan to vendored dependency source — rejected as heavier and governance-adjacent, recorded so the option isn't lost.

Bonus surfaced by the audit, directly relevant to the witness-proof goal: tropical `⊕` is a **selection**, so the accumulator's value is *one of the offered values* and uor-matmul's `TropicalValue` trait + D-6 tie rule (smallest index) make the argmax **witness-bearing by construction** — a ring accumulator provably cannot do this (their own doc note). Max-plus route composition therefore yields the winning path *and its witness* from the same accumulation.

Experiment filed as its own measured issue (linked below) per measurement discipline — criterion to be pre-registered on that issue before any run.

### Item 3 — E8 record: no table swap; unification point identified

The two E8 objects are different in kind today: r4's #395 lineage (`graph-certify/tests/e8_*.rs`) uses **nearest-lattice-point quantization** to full E8 (best-of-D8-and-coset, ×2 scaling, computed on the fly, per-stage RVQ with dynamically built per-stage codebooks) and remains experiment-stage; uor-matmul ships a **static 256-codeword shell table** (norm-sorted, i8-scaled, code 0 = origin, self-verified by `e8_is_the_lattice_it_claims`). No deployed r4 store uses a static E8-256 codebook, so there is nothing to swap and no κ re-pin due. **Standing decision recorded:** the day a store design adopts a *static* E8-256 codebook, the canonical table is `uor_matmul_codec::e8` (+ its κ manifest), not a hand-derived copy.

### Item 4 — teacher-side survey (sites identified; swap deferred to a measured decision)

Real-FP-matmul sites, all teacher/compiler-side (legal there): `crates/uor-r4-model-source/src/lib.rs:190-345` — `matmul` / `matmul_fast` / `matmul_batched`, with the fast paths on **`cblas_sgemm`** (#615/#616's ~15× batched teacher inference). Trade identified: BLAS backends are not bit-deterministic across hosts, the same hazard class as the AGENTS.md Gate-E libm note; uor-matmul's `gemm_float`/`raw::sgemm` are signature-compatible and bit-deterministic by construction (their CD-22), at unmeasured cost vs cblas. **Deferred pending measurement** (profile-before-optimize): today teacher κs are pinned per-artifact so the hazard is managed; the swap becomes load-bearing if cross-host reproducibility of observation runs is ever required. Recorded here so the option isn't lost.

Items remaining on this issue: 2 (dependency addition — now scoped by item 1 to dev-dep for parity + real dep only at the first compiler-side use site), 6 (pin discipline note for the moving upstream). Item 5 was resolved via #621's one-owner comment.

Tropical experiment issue: https://github.com/UOR-Foundation/uor-r4/issues/626 (filed as sub-issue of this one).

### comment 5260881638 — 2026-08-12T01:07:20Z — Casey-allard

## All work items complete — closing with record (2026-08-12)

1. **Legality memo** (comment above): tropical path = widening add + compare, P-4-legal; adoption route (a) — runtime-source implementation with dev-only parity — decided and used by #626/#627.
2. **Dependency added as scoped**: uor-matmul-core rev `b13c984` is a dev-dependency of uor-r4-graph-cli only (PR #627, merged), parity-pinning our max-plus fold to `dot_tropical_ref`; deny.toml source allowlisted with a dev-only marker. No shipped crate depends on uor-matmul.
3. **E8**: no table swap (nearest-point quantizer vs static codebook — different objects); standing decision recorded: any future static E8-256 store adopts `uor_matmul_codec::e8` as canonical, with a κ re-pin record.
4. **Teacher-side survey**: cblas sites identified (model-source lib.rs:190-345); exact-gemm swap deferred pending measurement — becomes load-bearing only if cross-host reproducibility of observation runs is required (teacher κs are pinned per-artifact today).
5. **One-owner**: uor-matmul owns exact matmul; prism-tensor not adopted (#621's record).
6. **Pin discipline (item 6, closing note)**: upstream has no releases and moves fast (HEAD advanced the day of the audit). The dev-dep is rev-pinned; any rev bump PR must re-run the parity test AND re-read the tropical module for semantic drift before adopting the new rev — the parity test is the tripwire, and a parity failure on a bump is a STOP, not a test to fix.

Measured outcome of the mechanism this issue mapped: #626 NEGATIVE on the current cover (E_f seat empty of signal — table on #626; `tropical-route-dormant` in the ledger via #629). The integration surface (kernel legality, parity pin, codec-tier and κ-manifest bridges) is delivered and recorded; the *first live use* of uor-matmul now waits on a workload with real signal (post-#597/#603 artifacts or the #604/#605 operator line), exactly as the dormant-lane gate states.


---
_Comment/review ids in this file: 5260321300, 5260339350, 5260881638_

