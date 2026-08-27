# Proof and Certificate

> **Preserved evidence scope:** the propositions and measurements below apply
> to the named transformerless TLA runtime, artifacts, and fixtures. They do
> not prove the current route-native attention hierarchy, source-free
> generation, correctness, reasoning, or chat. See the
> [documentation map](../README.md) before carrying a claim forward.

> **Migrated (2026-07-18):** this document moved from the `transformerless`
> repository into uor-r4, where it is integrated as
> `uor_r4_core::transformerless`. Command invocations use the
> `transformerless` binary. Measurements below are the pre-migration certificate
> (Linux container) unless otherwise noted; see the platform note in P3.

The transformerless claim decomposes into four propositions. Each is proven
in the strongest sense available to it — by construction, by witness, or by
measurement — and nothing is asserted beyond what its proof form supports.

## P1 — The runtime performs no multiplication. (By construction, measured.)

Every arithmetic operation on the inference path goes through `OpKernel`
(crates/uor-r4-core/src/transformerless/runtime.rs), whose complete method set is:

    add(i64, i64)   shl(i64, u32)   xor(u8, u8)   lt(i64, i64)
    table_u8(&[u8], u8)   table_i32(&[i32], usize)

Multiplication is absent from the interface; there is no method to call and
no census field to increment. The claim is hardened at three levels:

1. **Interface**: the runtime funnels the row decode, the bundle, the
   signature, the Hamming assignment, and the argmax through this kernel;
   the store is a B-tree, so key lookup is lexicographic byte comparison
   rather than hashing arithmetic.
2. **Source, machine-checked**: witness P-4 embeds `runtime.rs` at compile
   time and scans it on every `cargo test` — no multiplication, division,
   or modulo operator on any value. The scanned file contains the COMPLETE
   runtime arithmetic surface: the `OpKernel` itself, the popcount-table
   derivation, Hamming, the sign signature, decode, bundle, assignment,
   probe, and argmax all live inside it, so nothing on the inference path
   sits outside the scan. Rotation offsets, signature geometry, and the
   split point are compiled tables and constants derived in the compiler
   module; strides are walked by slice iterators, never computed as index
   products. (Delimitation:
   hardware address generation inside slice indexing and the B-tree is not
   program arithmetic; the popcount instruction used by the bulk path is a
   fused form of the kernel's xor + table + add loop, and their equality is
   itself witnessed.)
3. **Census, measured** (512-token sample, per-token averages, the
   shipped fixed-point decode-on-demand path included):

       add 59,571 | xor 36,864 | shift 22,707 | compare 1,323 | table-read 54,976
       multiply: 0 — no such operation exists in the kernel

(Era note, 2026-08-01 #327 re-pin: the tabulated census is the
TLA5-era record. With the shift-add dot assignment (TLA6) and the TLA7
residual wiring, the 500k-era certificate measures add 319,253 | xor 0 |
shift 318,932 | compare 1,360 | table-read 314,176 | multiply 0 per token
— the xor Hamming loop became shift-add table arithmetic; the multiply
count is unchanged because no such operation exists.)

Against the source transformer's ≈30 MFLOP per token, the runtime performs
≈1.8 × 10^5 integer operations per token — none of them multiplications,
memory-bound on table reads.

The bulk word-popcount path computes the identical functions for
throughput; equality with the kernel path is witnessed three stages deep on
every certification run — bundles, class codes, AND predictions,
bit-identical on 512/512 sampled positions. That witness earned its place:
it caught a tie-breaking divergence between the two argmax implementations
that the earlier codes-only witness could not see, now resolved by a
canonical rule (highest count, ties to the smallest token id) applied
identically in both paths and in the certifier's evaluator.

## P2 — The compiled artifact is a member of the source's behavioral
## equivalence class, with a measured residual. (By measurement.)

Source: stories15M llama2.c checkpoint, κ
`blake3:0ae7339518cb124bb9a8fcef88e5dfb615d9e56c55a118e55817dd077c77455d`
(60,816,028 bytes), its forward pass exactness-witnessed against a
single-threaded C oracle in the parent research crate (greedy tokens and
logit bytes identical). Evaluation stream: 150,000 teacher-labeled tokens,
757 stories, 30,192 held-out; teacher floor 1.5960 bits/token, teacher
ceiling (teacher-argmax vs sampled token) 70.4%.

Certificate (all rows use the runtime's prediction semantics —
deepest-populated-class argmax with backoff — for like-for-like comparison;
Witten–Bell bits are certifier instrumentation):

| Design | mul-free? | top-1 vs actual | agreement w/ teacher argmax | WB bits/token | store keys |
|---|---|---|---|---|---|
| A-f32 (nearest-centroid assignment) | assignment multiplies | 31.5% | 34.7% | 6.3214 | 86,574 |
| **A-binary (shipped runtime)** | **yes** | **28.9%** | **31.7%** | **6.5427** | **89,200** |
| B bit-prefix (no codebook classes) | yes | 26.0% | 28.6% | 7.6969 | 323,937 |

(The store is built by the runtime's own code path, so store keys and query
keys are one function by construction rather than by sampling.) The
measured price of full multiplication freedom at the assignment step is
3.0 agreement points (34.7% → 31.7%) and 0.22 bits/token. The codebook-free
bit-prefix coordinate — the purist "a vector at each bit" construction —
carries the concept at a further 3.1-point cost at this store scale: the
compiled classes structure the space better than raw dimension-ordered bit
prefixes, which is itself a measured design fact, not a prior.

Context for the residual: the parent report's scaling measurements (§3.10,
§3.11) show every one of these numbers still climbing with store size with
no saturation at 10^5 entries, the resolution knee migrating outward as
log_K of entry count, and table-native encoders converging on the source's
own context representation as the store grows. The residual is a property
of (artifact, store size, distribution), bounded above by the teacher
ceiling, and it shrinks with entries — which multiplies cannot buy and
storage can.

## P3 — Every compiled artifact is pinned; the compilation is
## reproducible. (By witness.)

Deterministic construction under fixed seeds, every frozen output
blake3-κ-labeled and reproduced bit-identically across independent
binaries (the token codebook κs below have now reproduced in four):

    source checkpoint      blake3:0ae73395…
    token codebook stages  blake3:6a6a6593… 43fc7c6c… 05177802… 5a49c745…
    stage books (i8, e=[7,9,9,9], shifts=[2,0,0,0])
                           blake3:54815061… 65e445d1… fbfa464c… 8f1bedf8…
    token codes            blake3:e3ad935b…
    threshold vector       blake3:0fdc1a2c…
    context codebook       blake3:a8375646… e13f7dfc… 314f4ebf… 7ff65bcc…
    class signatures       blake3:774419a7… 19537955… da2fec0f… fa266671…
    artifact container     blake3:be366bd6… (1,641,736 bytes, TLA3)

(The threshold, context-codebook, and class-signature κs changed when the
fixed-point decode landed — the P5 hardening — because the bundle
arithmetic they derive from changed; an earlier draft of this document
carried the stale pre-fixed-point labels, caught in audit. The token-side
κs are unchanged by construction: the codebook derivation precedes the
fixed-point step.)

**Platform note (2026-07-18, post-migration).** The pins above are the
Linux-container record. Reproduced on macOS arm64: every token-side pin
(codebook stages, stage books, token codes) is bit-identical; the
threshold, context-codebook, class-signature, and container κs — and with
them the corpus statistics (754 stories, 30,036 held-out) and certificate
rows (A-binary: 28.3% top-1, 31.5% agreement, 6.5634 WB bits/token,
92,464 keys) — differ, because corpus sampling runs through libm
`exp`/`ln`, which is not bit-stable across platforms. The macOS pins are
captured in full in `crates/uor-r4-core/tests/fixtures/baseline_kappa.json`
and asserted bit-identically by the κ-reproduction test
(`cargo test -p uor-r4-core --release --test kappa_reproduction -- --ignored`),
which is the migration's acceptance proof.

**Era note (2026-08-01, #327 re-pin, maintainer decision).** The
evaluation stream is now **500,000 teacher-labeled tokens, 2,507 stories,
100,306 held-out** (scaled 150k → 500k), and the artifact carries the
#318 Phase B residual wiring (TLA7 container, 1,346,836 bytes, κ
`blake3:ef6a20f3…`; the token-side pins are byte-identical to the prior
era by construction). Fresh full-certificate record on macOS arm64:
teacher floor 1.4260 bits/token, teacher ceiling 70.5%; **A (shipped,
single-key store + query-beam): top-1 34.7%, agreement 39.0%, WB 8.0249
bits/token, 179,068 keys**; A-f32 ablation 35.7% / 40.3% / 8.1063 /
187,644 keys; B bit-prefix 27.1% / 30.6% / 11.0356 / 1,150,027 keys; op
census per token add 319,253 / xor 0 / shift 318,932 / compare 1,360 /
table-read 314,176 / multiply 0. The new pins are the
`baseline_kappa.json` record; the 150k-era figures above are retained as
the historical record of the original certificate.

**Era note (2026-08-04, #407 fifth re-pin, maintainer decision).** The
context-codebook sample size `CTX_SAMPLE` moved 6,000 → 50,000 (attribution
sweep PR #411), and the era artifact was re-pinned under canonical
deterministic mode (`TLESS_CANONICAL_DETERMINISTIC=1`, the #265 D2 policy):
container κ moved `blake3:ef6a20f3…` → `blake3:8fbf3f68…` (1,346,836 bytes,
size unchanged). Token-side pins and the threshold vector are byte-identical
to the prior era by construction; the pins are the current
`baseline_kappa.json` record.

Library witnesses (cargo test): P-1 the popcount table matches its
definition on all 256 bytes and carries the stratum partition C(8,k);
P-2 kernel Hamming equals the direct definition with exact op counts;
P-3 sign signatures agree with the direct definition bit for bit. The TLA7
residual-wired path adds a persisted-container witness: after save/parse,
bundle and code outputs agree between kernel and plain paths on 512/512
sampled positions; the same witness is independently replayed by
`kappa_reproduction::tla7_resid_kernel_plain_witness` when the checkpoint is
present.

**Phase C adoption record (2026-08-01, #335).** On the 500k/TLA7 corpus,
write-time fan-out measured 20.3% top-1 / 22.7% teacher agreement / 8.1473
WB bits/token / 817,683 keys versus the selected single-key/query-beam shape
at 34.7% / 39.0% / 8.0249 / 179,068. The cpy8 and cpy16 rows were 35.3% /
39.6% / 8.5247 / 195,650 and 35.3% / 39.5% / 8.5205 / 196,220,
respectively; cpy8 is retained. The one-bit 1.5× mantissa refinement was
34.6% / 38.9% / 8.3097 / 162,119 against the coarse fold's 34.6% / 38.8% /
8.4241 / 167,799. It improves distribution shape but not top-1, so it is
not enabled in the runtime.

## P4 — The compilation is architecture-generic. (By construction,
## type-enforced; instantiated for one family.)

The two-surface claim is enforced by the type system, not by inspection:
`TeacherOracle` (crates/uor-r4-core/src/transformerless/teacher.rs) exposes exactly the embedding surface
(`embedding`) and the behavior surface (`reset`/`step`), plus geometry and
κ accessors, and the compiler, certifier, comparator, and scenario suite
are all written against `dyn TeacherOracle`. No consumer can reach an
attention variant, a gating choice, or a normalization detail, because the
trait does not carry them. The κ-reproduction check closes the refactor:
compiling through the trait reproduces every artifact κ bit-identically
against the direct-parse pipeline it replaced. This crate ships the
llama-family adapter (`LlamaOracle`); a qwen- or phi-class source
implements the trait — checkpoint reader plus forward pass — and nothing
downstream changes, by construction. The other adapters remain unbuilt and
are marked as such; what is closed is the assumption that generality
depended on discipline rather than structure.

## P5 — Compression is proven, load-bearing, and measured. (By witness
## and by measurement.)

The artifact ships the COMPRESSED token representation — 4 code bytes per
token plus four i8 stage books with power-of-two fixed-point scales — and
the runtime decodes rows on demand by table reads, shifts, and adds. The
expanded form never ships; compression is on the inference path, not a
storage option.

**(a) Container, witnessed.** save → load → save is byte-identical and
κ-stable, asserted every certification run. Era note (#327, maintainer
decision 2026-08-01): the default emission is the TLA7 container (#318
Phase B integer residual sections; 500k-corpus era) — witness
1,346,836 bytes, `blake3:ef6a20f3…` (2026-08-01 full-certify run; since the
2026-08-04 #407 fifth re-pin the current container κ is `blake3:8fbf3f68…`,
same byte length — see the P3 era note). The
residual sections and the new corpus change the serialization and the
bundle-derived κs; the token-side κ pins are byte-identical to the TLA5
era by construction, and prior-era containers (1-term TLA6 1,051,916
`blake3:cf64fe74…`, 2026-07-30 run; TLA5 462,092 `blake3:81db4406…`;
the original TLA3 1,641,736 `blake3:be366bd6…`) remain valid addresses
of their own bytes.

**(b) Representation rate–distortion, measured.** The exact bytes and
shifts the runtime reads, truncated at each prefix depth, against the
source's centered, normalized embedding rows (36,864,000 bytes f32):

| depth | bytes (codes + books) | ratio vs source | mean cosine |
|---|---|---|---|
| 1 | 105,728 | 348.7× | 0.9580 |
| 2 | 211,456 | 174.3× | 0.9636 |
| 3 | 317,184 | 116.2× | 0.9666 |
| 4 | 422,912 | **87.2×** | **0.9692** |

Monotone in depth — prefix refinement operating on the compressed form —
and within 0.0008 of the f32 codebook ceiling (0.9700) at full depth, so
the i8 fixed-point quantization is measurably near-free. This table is
itself a hardening artifact: the first implementation summed per-stage i8
codes under mismatched scales and produced a DECREASING curve
(0.9581 → 0.9443); the certifier exposed it, and the fix was shift-aligned
fixed point (per-stage power-of-two exponents, decode by `<< shift`),
which is kernel-legal and restored the monotonicity the construction
claims. The implementation was hardened to meet the prose; the prose was
not weakened to meet the implementation. (Era note, 2026-08-01 #327
re-pin: the byte/ratio columns are unchanged — the token codebooks are
byte-identical pins — while the fresh 500k-era certificate measures mean
cosines 0.9565 / 0.9613 / 0.9647 / 0.9676 at depths 1–4, monotone as
claimed; the tabulated values are the original Linux-certificate record.)

**(c) End-to-end artifact, measured.** Runtime tables 462,080 bytes +
store ≈ 1,704,201 bytes = 2,166,281 bytes against the 60,816,028-byte
source checkpoint: **28.1× smaller**, at the behavioral residual certified
in P2. This is compression of a behavior, priced by its certificate — the
number and the residual travel together or not at all. (Era note,
2026-08-01 #327 re-pin: with the 500k corpus the graded store grows to
≈8,342,205 bytes, so the 500k-era figure is 462,080 + 8,342,205 =
8,804,285 bytes, **6.9× smaller** than the source, at the improved
residual in the P2 era note — the store size is the scaling knob the P2
context paragraph describes; 28.1× is the 150k-era record.)

## Delimitations

Multiplication is confined to the compiler (offline, once, outputs frozen
and pinned) and the certifier (instrumentation, never at inference). The
certificate is same-distribution at ~10^5 store entries against a
15M-parameter source; convergence beyond that scale and on distributions
where long-range structure dominates is the open measurement, stated as
such here and in the parent report.

## Scope note appended 2026-08-18 (baseline audit)

P1's witness-scope sentence above ("the scanned file contains the COMPLETE
runtime arithmetic surface") is narrower in fact: `runtime.rs` delegates to
`transformerless::simd.rs` at five call sites (`hamming_distance_36`,
`dot_argmax`, `DotTables::from_packed`), and `simd.rs` is outside the P-4 scan
list. The SIMD Hamming path carries a scalar-equivalence witness
(`test_simd_hamming_equivalence`); the `dot_argmax` path currently does not.
The #160 machine-code (disassembly) audit has not landed, so operation-set
conformance remains **Witnessed** (source scan + census + construction), not
Structural — matching `INFERENCE_OPERATION_CONTRACT.md` §6/§8. Findings
AUD-INV-001/-002 in `docs/project_baseline_audit_2026_08_18.md`.
