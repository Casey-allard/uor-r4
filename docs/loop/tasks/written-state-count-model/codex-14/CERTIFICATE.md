# Full-count ceiling identity and proof

## Frozen input and output words

Earlier receipt: main commit dd7a4b6b0, docs/loop/tasks/written-state-count-model/claude-433/run1/c433p/ceiling-p1000.json. The DEV-tune identity is also in claude-422/run1/sealed/fix2/sweep-10.json. The existing copies under Wick ~/uor-codex/20261118-diagnostics/inputs/c420 matched all four hashes by sha256sum during drafting.

| Input | SHA-256 |
|---|---|
| TRAIN train.tokens.u16 | c6cba29ead4c745fb47b3f87007d982e72defc9f04102a268709ba0d34d51460 |
| DEV-tune dev-tune.tokens.u16 | 75c6d1ac479bf120bac3aa60ac160536e8146f8052aa49c002209500799b7d0a |
| dev-tune.novel.u8 | 7df42437260bb767e2b1bdfd0cc55a242bd6e0b632a5bcd754852a7ab8aca6af |
| id-bytes.u8 | befb22f6908751ae522a80336afa584eb2c24e36a10b90ad3105e39271a48ac9 |

R*: ae8518d552bc42d8f1c3602680bd77586d0cd9c237962fbf5e2b36704b6bb2fd SHA-256 over DEV-tune Q48 KN masses in target order. Overall and novel tallies are pinned in SHEET.md. A mass-table digest for the expanded experts is new evidence. An overall raw lookup digest was unavailable in the earlier full receipt, so a repeat confirms its stability and the existing 1‰ host lookup is the independent semantic check; there is no full lookups digest to match against the old run. Whole-device memory figures include concurrent allocations unless marked exclusive. The local advisory flock only names the Codex execution family. At draft time no process held it. The observed CUDA job used 3,456 MiB in another checkout, while the card had 20,202 MiB free and MemAvailable was about 35 GiB. Recheck at dispatch.

## Bound for the integer objective

Use W = 2^32, M = 2^48, positive integer masses p when an expert has support and zero otherwise. For each bucket b, the allowed Q32 coefficient vector w is nonnegative, sums to W, and is zero for absent lanes. The exact on-grid mixed target mass is m_t(w) = max(1, floor(x_t(w))), where x_t(w) = sum_e w_e p_te / W. The floor of 1 applied to present coefficient lanes during EM is a search choice. The bound covers the entire allowed simplex grid, including pure KN.

For an arbitrary feasible anchor a, use s_t = m_t(a) in [1,M] and define q_t(w) = x_t(w) + 1. Then m_t(w) <= q_t(w), including the zero case. Since log z <= z-1 for natural log, for every w:

    -log2(m_t(w)/M) >= -log2(s_t/M) - ((q_t(w)-s_t)/s_t)/ln 2.

Within a bucket the sum of q/s is linear in w/W. Its maximum over the relaxed simplex is at an expert vertex. Define

    D_b = max_{e present in b} sum_{t in b} (p_te+1)/s_t - n_b.

Here D_b >= 0 because q_t(a) >= s_t. Thus a bound on the largest possible reduction from the anchor is (sum_b D_b) / ln 2, prior to rounding allowance. The maximum is over all present lanes, including ones whose EM weight is tiny or zero. It also identifies a useful candidate lane for the registered optimizer escape when the directional residual is materially positive, though an actual transfer must be measured by exact Q12 score.

No floating-point or approximate reduction is required to verify a conservative lower bound. For B = 2^32, sum U_be = sum_t ceil(B(p_te+1)/s_t) in checked integers. Let C = sum_b (max_e U_be - n_b B). Use 1/ln 2 < 3/2, since the first two nonzero terms of 2(1/3 + (1/3)^3/3 + ...) already exceed 2/3. The bound in Q12 units is

    L(a) = max(0, sum_t bits_q12(s_t) - 4N - ceil(6144 C / B)).

Reserve two Q12 units per target for the anchor's log approximation and two for the unknown optimum's approximation. This uses the documented absolute error of the 12-round integer routine without having to certify a one-sided version of it. All operands fit u128 for this manifest. The per-term numerator is at most B(M+1), and with N < 2^24 both C and 6144 C fit; use checked operations and reject noncanonical masses, malformed bucket assignments, incomplete streams and missing experts. Negative computed C is invalid, not clamped. Keep bucket-expert sums and maximizing lane with IDs in the device receipt so the verifier can reconstruct every arithmetic decision.

Take L as the maximum verified lower bound over registered anchors (pure KN and the best fitted point from both starts). H is the minimum feasible exact Q12 objective over KN, both starts and the restricted KN+uniform control. A two-fold fit chooses the weights using record parity in addition to bucket, so it is an overfit diagnostic and cannot supply H for this family.

A companion elementary witness is the per-row maximum mass: m_t(w) <= max(1,max_e p_te). Monotonicity of the implementation's bit score then gives another conservative L, useful as a sanity check. It may be weaker because it lets each target choose its own expert. Register it only if implemented without a further full collection. Report the observed fits as observations even when [L,H] resolves the branch. A numerical plateau or agreement between starts supplies no global lower bound. The certificate covers this exact count family and corpus, with the full-support floor and code-length rounding, and says nothing about a larger nonlinear or geometric family.

## Arithmetic controls

On a toy weight grid enumerate every admissible vector and verify L <= exact minimum <= H. Include a floor-to-one case, absent lanes, exact simplex boundary, tie at a bucket vertex, within-bucket association where sum of row maxima is unachievable, and literal numerator and carry cases. Use an independent Rust finite oracle for the direction and ceil operations and deliberately reverse the ceil, drop a bucket, swap a lane, and substitute the wrong mass tape in the verifier fixtures. For any mutated input that preserves a vacuous lower bound, require the identity controls to catch it. Reproduce the checked lower bound independently from the bucket receipts and the RHS arithmetic; fit updates or optimizer selection never affect certificate validity.

The draft arithmetic was checked separately using exact integer operations against 35 small fixture sets with a 561-vector coefficient grid and four anchors each. All 140 bounds enclosed the exhaustively scored grid minima. This finite check does not substitute for the new Rust/CUDA controls or any corpus evidence.
