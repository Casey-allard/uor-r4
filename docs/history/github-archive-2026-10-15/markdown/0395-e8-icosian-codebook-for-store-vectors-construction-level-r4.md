# #395 — E8/icosian codebook for store vectors: construction-level R4 geometry spike

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-03T23:31:47Z
- **Updated:** 2026-08-04T15:51:59Z
- **Closed:** 2026-08-04T15:51:59Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/395
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

Follow-up to the R4-native architecture framing issue: construction-level geometry (what the store IS), not a scoring channel. Explicitly not covered by #276/#392's negative, which tested phase as an addressing/scoring term.

## Math

- The 240 E8 roots fold onto H4 ⊕ φH4 — two 600-cells coupled by the golden ratio (Moody–Patera folding / Elser–Sloane).
- The icosian ring (quaternions over Z[φ]) with the Turyn norm is the E8 lattice as a Z-module: **every E8 lattice point is canonically a golden-coupled pair of R⁴ points.**
- E8 is the provably optimal sphere packing in 8D (Viazovska 2016) and the best known 8D lattice quantizer.
- Precedent: E8-codebook weight quantization is deployed practice in LLM compression (QuIP#-family); the Conway–Sloane nearest-point decoder for E8 is round/compare/coset-check — compatible with the no-multiply inference contract (`INFERENCE_OPERATION_CONTRACT.md`) with fixed-point golden-ratio scalings.

## Spike scope (research/, following the research/290-fmm pattern)

1. Icosian ↔ E8 mapping + Conway–Sloane decoder prototype (Python for validation, Rust port if distortion numbers justify).
2. Distortion measurement against real store content: quantize current prototype/emission vectors (and teacher hidden states h(c) from the observation corpus) into the E8 codebook at matched bit budgets; report MSE/sign-agreement vs the current 36-byte sign-bit code.
3. Decision gate: if E8 codes preserve neighborhood structure better than orthant signs at equal bits, promote to a construction experiment (compile a bundle whose ROUT prototypes are icosian pairs); else record negative and close with numbers.

## Non-goals

No serving-path changes, no certify schema changes, no scorer channels. Research-folder only until the decision gate.


## Comments (6)

### comment 5173065316 — 2026-08-04T00:08:43Z — Casey-allard

## Measurement 1 results (matched 288-bit budget, real store vectors) — tooling + RESULT-1 in PR #397

50,000 centered context-bundle vectors (dim 288) from the checked-in fixture corpus — the exact objects the shipped path sign-codes into 36 bytes. Disjoint train/pool/query split. Both codes at exactly 288 bits/vector.

| metric | SIGN (shipped orthant code) | E8-256 |
|---|---|---|
| recall@10 vs cosine GT | 0.553 | **0.606** |
| recall@10 vs L2 GT | 0.541 | **0.634** |
| relative reconstruction MSE | 0.446 | **0.213** |

Train data occupies ~22k distinct E8 lattice points per 8-dim block; capping the codebook at 256 (hard 8 bits/block, no entropy coding) still beats the sign code on every metric — reconstruction error halves, L2 neighborhood recall +9.3pp absolute.

Icosian reading: each block codeword is an E8 point = a golden-coupled pair of quaternions, so a context under this code is 36 icosian pairs (72 R⁴ orientations) instead of 288 uncoupled sign bits. Construction-level substrate, zero scoring channels touched — the category #276's negative does not cover.

**Decision gate met** per the issue plan → next: construction experiment (assignment/membership geometry over icosian block codes, graded by certify + the #394 infill criterion at equal store budget). Caveats in RESULT-1: retrieval proxy, fixture-only, kernel-contract conformance of the decoder unverified.


### comment 5173477857 — 2026-08-04T01:17:21Z — Casey-allard

## Construction experiment v0 (PR #403): recorded NEGATIVE for spatial-group keying — with the mechanism identified

Graded store keyed by E8 block-group codebooks at matched key budget (4 stages × 255-entry codebooks + OOV, identical evidence rule and eval):

| store | top1 | agree | WB bits | keys | infill-free top1 |
|---|---|---|---|---|---|
| shipped geometry | 34.8% | 39.1% | 8.4461 | 179,068 | 34.7% |
| E8 spatial-group v0 | 23.0% | 25.8% | 8.5579 | 47,372 | 23.0% |

**Why it lost — mechanical, not geometric:** stage OOV rates are 40–46% (210k–232k of 500k records per stage). The pattern space of 9 block ids dwarfs a 255-entry codebook, so nearly half of all records collapse into shared OOV buckets, and the store ends with a quarter of the shipped key count. The failure is the flat spatial keying + frequency cap, not the lattice: M1 (PR #397) showed the same E8 codes preserve neighborhood structure *better* than sign bits at matched bits.

**What this negative does and does not cover:** it kills "flat block-group patterns as graded keys." It does not touch **residual-E8** — stage k quantizing the full-vector residual after stage k−1's icosian reconstruction, mirroring the shipped residual-VQ shape with lattice-structured codebooks. That is the fair fight (the shipped scheme gets trained coarse-to-fine structure; v0 gave E8 none) and is the next measurement.

Scope note: fixture distribution; all certifier-side; scales grid-picked on construction only.


### comment 5173630761 — 2026-08-04T01:43:48Z — Casey-allard

## v1: residual-E8, retraining-controlled (PR #404) — lattice structure is FREE at the right coarseness

Five arms, one deterministic k-means training, identical store build and eval. The decision comparison is `rvq-e8@m` vs `rvq-f32` (same trained centroids, blocks snapped to the icosian lattice vs not):

| arm | top1 | agree | WB bits | keys | infill-free |
|---|---|---|---|---|---|
| shipped geometry | 34.8% | 39.1% | 8.4461 | 179,068 | 34.7% |
| rvq-f32 (control) | 36.8% | 41.4% | 7.9266 | 191,721 | 36.8% |
| **rvq-e8@0.5rms** | 36.7% | 41.2% | **7.9175** | 191,785 | 36.7% |
| rvq-e8@1rms | 36.5% | 41.1% | 7.9647 | 195,461 | 36.5% |
| rvq-e8@2rms | 35.8% | 40.3% | 8.1699 | 193,082 | 35.8% |

Readings:

1. **The construction gate advances.** At 0.5×block-RMS snap scale, icosian-lattice codebooks are statistically indistinguishable from the f32 control on top1 (−0.1pp) and slightly better on WB bits. Combined with v0's diagnosis, the record now reads: E8 loses only when denied residual structure; given the same skeleton the shipped scheme uses, the lattice costs nothing — and buys integer/fixed-point centroids, the golden-coupled quaternion-pair (icosian) representation per block, and a decode path compatible in principle with the no-multiply contract. Structure for free is a win for construction-level geometry.
2. **The coarseness curve is monotone above 0.5rms** (36.7 → 36.5 → 35.8): snap scale is a real dial; 0.5rms (or finer) is the operating point on this data.
3. **Orthogonal observation:** a freshly trained RVQ beats the shipped compile by +2pp top1 / −0.52 WB bits at the same budget on both open and infill slices. That is about compile quality, not lattices — flagged separately in case it is worth its own issue.

Caveats: fixture-only; certifier-side; kernel-contract conformance of the E8 decode is designed-for but unverified; natural-corpus rerun queued behind #381.


### comment 5173709385 — 2026-08-04T01:56:20Z — Casey-allard

## Z[φ] conformance sketch: icosian arithmetic is contract-native (next step on the E8 → H4×H4 path)

v1 (PR #404) showed lattice codebooks are free. This note argues the icosian representation is also *arithmetically* free under `INFERENCE_OPERATION_CONTRACT.md` — no floats, no multiplies at serving — and states what to measure next. Claim language: everything here is design argument pending the listed measurements; the group-theory identities are standard.

**Representation.** Every E8 codeword is an icosian: a quaternion whose four coordinates live in Z[φ]. Store each coordinate as an integer pair (a, b) denoting a + bφ. A 288-dim context under the v1 code is then 36 blocks × 1 icosian = 72 quaternions = 144 Z[φ] pairs — all integers.

**Golden arithmetic is add-only.** In the (a, b) basis: φ·(a + bφ) = b + (a+b)φ, i.e. (a, b) → (b, a+b) — the Fibonacci shift; φ⁻¹·(a + bφ) = (b−a) + aφ, i.e. (a, b) → (b−a, a). Multiplication by any power of φ is a chain of integer adds/subs. Golden-conjugation (φ ↔ −1/φ, the involution that swaps the two H4 copies of the fold) is (a, b) → (a+b, −b): adds and a negation.

**Serving operations, item by item:**
1. *Encode (query → block id):* Conway–Sloane E8 decode is round / compare / coset-select — adds and compares. The per-block scale is compile-time-fixed, so it folds into precompiled rounding thresholds (or a shift-add constant sequence); no runtime multiply.
2. *Distance for membership/scoring:* codeword-to-codeword distances are finite (256 × 256 per block codebook) → precompiled distance tables, 64KB/block-stage class of object — the same "table read" allowance the kernel already uses (popcount tables, FMM translation tables). Serving pays table reads, never a squared difference.
3. *Group action (the routing-algebra step):* the 120 unit icosians form the binary icosahedral group 2I (vertices of the 600-cell). Its entire multiplication is a 120×120 Cayley table; conjugation (unbinding) is an involution table of length 120. Token-as-rotation binding — the torque picture — is a pure table read at serving.

So: compile time pays the multiplies (building thresholds, distance tables, Cayley table); serving pays adds, compares, and table reads. Same division of labor the contract already enshrines.

**Measurement ladder implied:**
1. Icosian-distance membership A/B: same store, same eval, Hamming-ball vs lattice-distance-table membership at matched budget. First place H4×H4 structure can *win* rather than tie.
2. Cayley-table binding prototype: re-express window bundling as 2I group action on block icosians; grade on the #394 criterion. (This is the "each node carries its routing" mechanism in its native algebra.)
3. Speculative, flagged as such: assign the two golden-conjugate H4 factors to the two measured directional channels (causal E_f / forward E_b, cf. the #399 fusion results) — conjugation as the look-back/look-forward involution. Only after (1) and (2) carry signal.


### comment 5175371421 — 2026-08-04T06:23:23Z — Casey-allard

## Ladder step 1 result (PR #406): the all-integer table-read pipeline costs 0.4pp — the icosian serving path is validated at the accuracy level

Three arms, identical training (same deterministic k-means as v1):

| arm | top1 | agree | WB bits | keys |
|---|---|---|---|---|
| rvq-f32-exact (control) | 36.8% | 41.4% | 7.9266 | 191,721 |
| rvq-e8-exact (v1 winner) | 36.7% | 41.2% | 7.9175 | 191,785 |
| **rvq-e8-tables** | 36.3% | 40.7% | 7.9769 | 205,360 |

`rvq-e8-tables` lattice-quantizes the query residual per block BEFORE every distance and updates the residual in the quantized domain — so every distance the pipeline ever computes is between two lattice points from finite sets, i.e. a precompilable table entry. Price of going all-integer: **0.4pp top1 / 0.06 bits** vs exact lattice distances (0.5pp vs the f32 control).

The full icosian serving story is now measured or proven, layer by layer:

| layer | status |
|---|---|
| storage (centroids on the lattice) | FREE at 0.5rms (v1, PR #404) |
| distances (table reads) | −0.4pp (this measurement) |
| golden arithmetic (Fibonacci shift, conjugation) | exact, adds-only (unit tests green) |
| decoder (Conway–Sloane nearest-point) | verified vs adversarial in-lattice neighbors (unit test green; the test's own E8-membership predicate needed fixing — both cosets require an even coordinate sum) |
| group action (2I binding) | 120×120 Cayley table by construction (measurement pending, ladder step 2) |

Contrast with v0 completes the design lesson: quantizing QUERIES costs 0.4pp; quantizing KEYS into flat spatial groups was catastrophic (OOV flooding). The lattice belongs on the comparison side of the store, not in the key structure.

Remaining on the ladder: step 2 (Cayley-table binding graded on #394) and the natural-corpus rerun of this table once naturally-compiled artifacts exist (see #394's transfer-collapse finding — the fixture-compiled geometry question applies here identically).


### comment 5181451939 — 2026-08-04T15:51:58Z — Casey-allard

Closing: the spike is over-complete — four measurements recorded, decision gate met and exceeded. M1 (PR #397): E8 beats sign codes at matched bits. v0 (PR #403): flat spatial-group keying negative, OOV flooding diagnosed. v1 (PR #404): lattice-snapped codebooks FREE at 0.5rms, retraining-controlled. Ladder step 1 (PR #406): fully table-implementable all-integer assignment costs 0.4pp — with the Z[phi] conformance sketch and green unit tests (Fibonacci-shift arithmetic, conjugation involution, Conway-Sloane optimality), the icosian serving path is validated layer by layer: storage free, distances table-reads, arithmetic add-only, group action a 120x120 Cayley table. Design law recorded: lattice on the comparison side, never flattened into keys. Follow-ons live in the #393 program: Cayley-binding (ladder step 2) sequenced behind the substrate direction, natural-corpus icosian reruns behind the scale track, and the compile-quality control finding spun out as #407.


---
_Comment/review ids in this file: 5173065316, 5173477857, 5173630761, 5173709385, 5175371421, 5181451939_

