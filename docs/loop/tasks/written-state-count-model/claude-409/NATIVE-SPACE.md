# Native-space items: written-state-count-model, claude-409 (D85)

The five items that `.agents/skills/native-space-cycle/SKILL.md` asks every model unit to write down before model code [claude]. They are taken from the PI review's DECISION §3 and §4 P0 item 2 ([review record](../../../../research/reviews/pi-review-2026-11-16/DECISION.md)) [claude]. They bind P1–P5 of the written-state track (the [decision card](DECISION.md)). No code implements them yet; each line names the phase that builds it [claude].

Representation: every field below is an integer word (u16 token ids, packed u64/u128 keys, u32 counts, i8 error cells, Q16 Brier errors, integer mixer weights). No field is floating point. `-log2` stays in the offline PRLL scorer, which is reference code under D85 [claude].

## 1. State schema (S0–S6)

One persistent integer state, extending the D11 discrete trunk's `TrunkTables` (`crates/uor-r4-discrete/src/trunk.rs:612`) [claude].

| Part | Carried fields | Representation | How it learns | Home | Phase |
|---|---|---|---|---|---|
| S0 tokens | V = 4,096 token ids; one prime q(t, j) per (token, slot), 4,096 × 12 = 49,152 primes, the largest ~6.0e5 (20 bits) [est]; one E8 code per token | u16 ids; prime table; E8 code words | fixed | tokenizer; prime-zeta codebook | P1 |
| S1 transport | the D11 discrete trunk: Hamilton gate, read and rec codes per position | the existing trunk words | fixed geometry; ≤ 1e5 scored transport coordinates in P5 | `TrunkTables` for `DiscreteHamiltonGateV1` | existing; P5 |
| S2 n-let rows | orders 1..N under `PrimeRouteKey`; value = (token, count) plus context type counts | packed key word, u16 token, rank-quantized count | counted; Brier error cells are a measured P3 arm | new table family in `TrunkTables` | P1 |
| S3 state rows | key = (E8-snapped transport code, optionally the last token); value = (token, count, error cell) | E8 product-key index (240 × 240 = 57,600 per scale), u16 token, count, i8 error | counted, then error-written | extends the read/rec/query tables (`trunk.rs:614-628`) | P2 |
| S4 conjunction rows | key = the squarefree prime product of two user-turn tokens (order-free by design); value = pointer (role or span offset), count, error cell | factor-bitset key, pointer word, count, i8 error | counted from TRAIN; error-written, with the hinge on pointer choice | new table family in `TrunkTables` | P3 |
| S5 session copy | the Pick read: the latest earlier position whose (p−1, p) block codes are nearest by XOR + popcount, and its gated successor | 16-bit block codes, popcount distance, gate trits | none (session carry) | `DiscreteReadMode::Pick` (`crates/uor-r4-integer/src/discrete_d11.rs:286-298`); `pick_device` (`crates/uor-r4-discrete/src/pick_read_cuda.rs:56`) | done (#4045) |
| S6 mixer | integer weights w[bucket, expert], ≤ 4,096 buckets × ≤ 12 experts (≤ 49k weights); a full-support unigram and a uniform floor f ≥ 1 | integer weights | integer EM, then Brier error-write | the skip-mix table-injection precedent | P1, P3 |

Bucket b = (longest order hit, Pick match length, E8 distance, expert agreement) [claude].

## 2. Shared operator paths

Each operator has one source definition. CUDA (nvcc PTX in its own module, built by `build.rs`, no NVRTC), the CPU oracle and the serving reader `discrete_d11` implement it behind that definition and are held equal by bit-identical differential tests [claude].

| Operator | Meaning | CUDA (wick) | CPU oracle | Serving (`discrete_d11`) |
|---|---|---|---|---|
| `PrimeRouteKey` | K_k = Π_{j≤k} q(t_{−j}, j); stored as the packed ordered ids (12 bits × k: u64 to k = 5, u128 to k = 10), a bijection by unique factorization; back-off K_{k−1} = K_k / q(t_{−k}, k) is dropping the last slot; shared-suffix gcd is slot-wise equality; hashing only for bucket placement, with the full key checked in the row | key generation on device from the resident u16 stream | `cpu-trunk-count-driver`, 1–10% slice: a host parity check in a separately scanned Rust module, as D84 allows for CPU parity; it qualifies no bar | key lookup in the frozen form |
| Counting kernel | radix sort-reduce of (key, token) per hash-range partition; prune to count ≥ 2 on device before copy-out; ties in top-k break by token id | yes | yes, on the slice | n/a (reads counts) |
| E8 snap | each 8-coordinate trunk block to its nearest E8 point (Conway–Sloane: integer rounding plus a parity fix); a miss is filled by a Starling hard read over the ~7 nearest of the 240 root neighbours | yes | yes | yes |
| Pick read | as claude-408: XOR + popcount over transported 16-bit block codes, least distance, ties to the latest position | `pick_device` | `discrete_route.rs` | `DiscreteReadMode::Pick` |
| Lawful mixer | score(t) = Σ_e w[b, e] · c_e(t) · L / tot_e + w_u · uni(t) + f; integer multiply-add over ≤ ~300 hit candidates plus the closed-form floor; PPM exclusion, inheritance and SEE as integer count ratios | yes | yes | yes (greedy decode needs no normalizer) |
| Error-write | the integer Brier error (1[t = target] − P(t)) at Q16, written into the addressed S3, S4 and S6 cells; LMS on a linear mixture | yes | yes, on the slice | n/a (reads the written cells) |
| PRLL scoring | the integer score vector (every entry ≥ f ≥ 1) through `mle_masses_from_counts` (`crates/uor-r4-integer/src/lawful_likelihood.rs:113-117`), then `bits_q12` and `bits_per_byte`; never `masses_from_scores` for count models, whose min-shift cancels the floor | GPU scoring is research compute (§3.6) | yes | n/a |

Banned on every path (§3.2, §3.3, DECISION §3.4 rule 4): logistic, stretch or logit mixing; stretch-domain SSE/APM; Hedge, exponentiated-gradient or multiplicative weights; maximum-entropy or log-linear smoothing; any vocab × feature or candidate × hidden weight table [claude].

## 3. Replay oracle

- **The canonical triple stream.** The written state is the sorted (key, token, count) triple stream, and its SHA-256 is the replay word [claude]. Two streams are hashed, and each gate names which one it uses [claude]:
  - **The exact stream** is the unpruned counts. Its SHA must be equal between the GPU writer and the CPU oracle, on the 1–10% slice and never the whole corpus (R2-m3).
  - **The declared stream** is the exact stream after the declared prune and rank-quantize step of §5. That step is a pure function of the exact stream, and the oracle applies it too. The SHA of the declared stream must be equal across the writer's output, the oracle's output and the `discrete_d11` loader's read-back of the frozen form (P1 gate (a)).
- **Carried fields.** On ≥ 64 prompts, including misses and back-offs, the raw words of every carried field are equal between wick (CUDA) and the M1 (CPU): addresses, reads, Pick choices, pointer choices and emitted token ids (P4 gate) [claude]. Comparisons are of raw integer words, never rounded values [claude].
- **Determinism.** Counts are commutative integer sums; top-k ties break by token id; error-writes apply in canonical key order. Replay is therefore exact by construction, and the oracle tests it [claude].

## 4. Update rule: count, then integer Brier error-write

1. **Count.** S2, S3 and S4 rows are initialized by exact integer counts over D87 TRAIN [claude].
2. **Fit the mixer.** S6 weights are initialized by integer EM (deleted interpolation, Jelinek–Mercer) on a held-out TRAIN slice: w_e ← Σ w_e p_e / Σ_e' w_e' p_e', a ratio of linear sums [claude].
3. **Error-write.** For 2–3 epochs on wick, the integer Brier error at Q16 is written into the addressed S3, S4 and S6 cells, in canonical key order [claude]. S2 error cells are a measured arm only, since they double S2's bytes [claude].
4. **Scored core (P5 only).** ≤ 1e5 S1 transport coordinates through rows-K (`score_rows_k`, `crates/uor-r4-discrete/src/rowsk_device.rs:56`) and `run_block_transitions`, with the integer Brier score of the mixed mass as the objective, against a re-seeded random-coordinate control [claude].

Every update lands in the same integer cells the server reads. There is no float master and no export step [claude].

## 5. Lossless format

- **Oracle form.** `SparseCountTable` (`BTreeMap<u32, BTreeMap<u16, u32>>`, `trunk.rs:510-513`), sized for 1,049,856 rows and keys up to order 2 [claude]. It stays the oracle form on fixtures and slices; it cannot hold 1e9 entries [claude].
- **Frozen form (the artifact).** Elias–Fano sorted tries (the tongrams algorithm, re-implemented in Rust) with rank-quantized counts, entropy-pruned to fit ≤ 4 GB resident [claude, est]. Its read-back re-emits the declared stream (§3), so its SHA equals the declared-stream SHA of the writer and the oracle, not the exact-stream SHA [claude].
- **Round trip.** A lossless round-trip between the frozen form and the oracle form on fixtures is a required test (DECISION §3.3) [claude]. Pruning is a declared step before freezing, measured by its BPB cost against unpruned R\* (P1 gate (b)); it is not a translation of a different model [claude].
- **Fallback.** If 4 GB forces more than a 3% pruning cost, orders above 6 move to an optional mmap'd suffix array of ~13 GB on disk, read only on long matches, with cold tok/s reported [claude, est].

## What this file does not claim

No item above is built or measured. Every size, speed and cost figure is an estimate from DECISION §3.5–§3.7 [claude, est]. The track's bars are in the [decision card](DECISION.md) and DECISION §2 [claude].
