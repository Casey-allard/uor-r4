# Softmax-free reads for the served stack: rank-table and Hamming-rank flock reads — October 10

Lab: claude. Milestone: [M4 #2032](https://github.com/UOR-Foundation/uor-r4/issues/2032), acceptance item 0 (no softmax at runtime). Pre-registration: [#2032 comment 6093484433](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6093484433) (arms A–D: 18:29Z and 20:35Z cards of 9 October).

**Result: REJECT for arm group 1 (B and D).** Both softmax-free reads, trained with the fixed `1/(r+1)` rank table, miss the pre-registered 0.01 BPB bar by a wide margin: rank by +0.040 BPB, Hamming-rank by +0.259 BPB. The served model keeps its softmax read, so acceptance item 0 is not met. Per the pre-registration the target is not dropped: the next step is to redesign the read, with rank tables learned in training (pivot card on #2032).

## The question

The served stack reads still weight their sources with an exp-table softmax. Can a read whose weights come from a fixed rank table (no exponential) replace it without a measurable loss?

## The mechanism (trainer, `geometric_stack.rs`)

`StackModel::set_read_weighting(ReadWeighting)`, saved as `config.json` field `read_weighting`, absent for softmax:

- **`rank` (arm B):** each read row keeps the flock support of `select=flock:W:K`: the sink at position 0, the last `W` positions, and the top `K` of the rest by score (the shared `flock_select`).
  - The NoRead slot is ranked among the kept sources and loses ties.
  - With `m` = kept + 1, the weights are `w_r = (1/(r+1)) / Σ_{i<m} 1/(i+1)` by rank. Unkept sources get exactly 0.
  - This is the starling rule: each source is weighted by its rank among the nearest few, not by an exponential of its score.
- **Gradient:** straight-through. The query, key and auxiliary gradients are exactly those of the flock-softmax read on the same scores. The value gradient uses the forward rank weights. Both passes use the same selection.
- **`hamming_rank` (arm D):** the projected query and key are binarized by a straight-through sign (±1, `sign(0) = +1`) before the read, then weighted by rank.
  - For ±1 vectors the L2 read's squared distance is `4 · popcount(q ⊕ k)` (the Lorentz excess is `2 · popcount`), so the score is a Hamming rank.
  - The served kernel can therefore compute it with XOR and popcount.
- **Where it runs:** CPU, and on CUDA through the host fallback, which every flock-selected read already uses.
- **What it refuses:** bf16; no flock; and a geometric address, latch, lineage or span, which replace the ordinary read.
- **CLI:** `geometric-stack train` and `dialogue-train` accept `select=flock:W:K` and `read_weighting=softmax|rank|hamming_rank`. They record both in the report and lineage, refuse a resume with a different weighting, and copy the weighting to data-parallel replicas.

## The run this prepares (arm group 1)

- **Recipe:** the M4 chat stack's recipe, the argv of `chat-served2-20261008/model-a1`: `train`, rrarrarr, w512 h8 ctx384, read l2, 19.93M params, chat-v0-p2 train, batch 32, lr 4e-4, warmup 500, 12,207 steps.
- **Arms:** seeds 1 and 2 for each of A (softmax), B (`select=flock:8:8 read_weighting=rank`) and D (`... hamming_rank`).
- **Bar:** B or D within 0.01 BPB and 2 v4 points of A, at equal or lower served cost.
- **Arm A** ran at `main` 5b241d89c (softmax path unchanged).
  - Float held-out NLL at 512 windows, 196,608 targets: **s1 1.7250754 (0.877118 BPB), s2 1.7253881 (0.877277 BPB)**, on the stream basis 2.837427 B/token.
  - s1 reproduces the recorded original run's 1.7250755 to 1e-7.

## Serving (preparation 2): the D11 engine serves both reads

- **Artifact:** schema `uor-r4.lut-stack/3`. `/2` already means a pointer stack, and the frozen D10 engine's exact-schema check refuses `/3`, so no older engine can serve a rank artifact as softmax. Its `shape` adds `read_select {window, k}` (the sink is always position 0), `read_weights "rank"` and `read_binary`.
  - The exporter writes `/3` only for a flock model with a rank or hamming-rank weighting. Every other refusal still applies: flock with softmax, a non-zero sink, U(1) transport (a review finding, pinned by a test).
- **Rank read:** per head, the engine's own scores plus age go through the allocation-free integer flock selector, now a bounded insertion top-k. The library `select_nth_unstable_by` and `sort_unstable_by` compiled to `madd`/`mul`, which the audit flagged as reachable callees.
  - NoRead is ranked among the kept sources and loses ties. The weights are the Q31 `1/(r+1)` tables precomputed at load for each support size.
  - Each read touches at most `window + k + 1` value rows (17 for `flock:8:8`), against `t + 1` for the softmax read.
- **Hamming-rank read:** `BitCode` is a sign-bit code of up to 256 lanes with XOR binding. Its Hamming distance is a shift-and-add SWAR popcount: `count_ones` lowers to NEON `cnt` plus `fmov` on arm64, and to a multiply on x86 without POPCNT.
  - A per-head table indexed by `h` holds the engine's own dense score of ±ONE vectors (ONE = 2^16) that differ in `h` coordinates. A score is one XOR, a popcount per 64 lanes and a table read, plus age.
  - BitCode is a similarity code. It is not an exact identity: BLAKE3 and prime addresses are.
- **Checks at the PR head:**
  - `uor-r4-integer` lib: 282 passed.
  - Cross-engine oracle `stack_d11_oracle`: 13/13 (schema /1 and /2 serve bit-identically).
  - `stack_softmax_free_export`: 3 passed. Random 4-bit weights, D11 against float: rank −0.039 nats, hamming −0.012 nats, top-1 agreement 1.0.
  - `stack_export`: 17 passed. Frozen `uor-r4-lut`: 25 passed.
  - `audit_zero_matmul_serving.py --stack`: **FULL PASS**, 91 reachable functions. A negative gate with the library sorts restored fails on exactly those callees.
- **Evaluation:** `geometric-stack d11-evaluate ... reference=none model=ROOT/model` scores D11 against the float model on the same windows, for artifacts D10 cannot read.

## Result (arm group 1): REJECT for B and D

All numbers are on held-out `heldout.u16` e5f400b0, 512 windows × 384 = 196,608 targets, the stream basis 2.837427 B/token.
- **Served column:** the D11 engine (`d11-evaluate reference=none model=…`, the same windows as the float column). A's served cell equals D10 `lut-evaluate` to 1e-7 (s2's is the D10 value).
- **v4:** the frozen-check `multi_turn_memory` `check_pass` of 40 (`chat-grade reply` then `grade-replies`, qwen2.5:7b).

| arm (seed) | float NLL | float BPB | served BPB | top-1 D11/float | v4 memory |
|---|---:|---:|---:|---:|---:|
| A softmax (s1) | 1.7250755 | 0.877118 | 0.886817 | 0.9297 | 6/40 |
| A softmax (s2) | 1.7253881 | 0.877277 | 0.886702 | — | 3/40 |
| B rank `flock:8:8` (s1) | 1.8035471 | 0.917017 | 0.925884 | 0.9242 | 3/40 |
| B rank (s2) | 1.8035873 | 0.917037 | 0.925945 | 0.9238 | 6/40 |
| D hamming_rank (s1) | 2.2865490 | 1.162600 | 1.173532 | 0.8954 | 0/40 |
| D hamming_rank (s2) | 2.1807848 | 1.108824 | 1.120014 | 0.9003 | 0/40 |

**Against the pre-registered bar** (seed means; B or D within 0.01 BPB and 2 v4 points of A, at equal or lower served cost):

| arm | Δ float BPB vs A | Δ served BPB vs A | Δ v4 | served speed (D11, 8 threads, 32 windows, run one after another) | verdict |
|---|---:|---:|---:|---|---|
| B rank | **+0.0398** | +0.0392 | 0.0 | 146.0 vs A 151.8 tok/s | **REJECT**: BPB misses by about 4× |
| D hamming_rank | **+0.2585** | +0.2600 | −4.5 | 148.2 vs A 151.8 tok/s | **REJECT**: BPB and v4 both miss |

**What the numbers say:**
- **The quantization cost is unchanged by the read:** float → served is +0.0097 BPB for A and +0.0089 for B. The gap is the trained model's, not the serving path's.
- **The two B seeds agree to 2e-5 BPB.** The gap from A is the read's, not the seed.
- **The served cost is equal at this context.** Each read touches at most 17 value rows per head, against 192.5 on average for the softmax read over a 384 window. But each token reads all 19.93M dense weights, which dominates, and the selection costs about what it saves. So the bar's cost clause neither rescues nor sinks B.
- **The Hamming-rank read loses most:** binarizing q and k to sign bits erases the magnitude information the L2 score ranks on. Its seeds also diverge (1.163 against 1.109).
- **Training cost:** B and D compute the flock on the host. That's 26k tok/s for B on 2×5090 and about 16k for D on 2×4090, against A's 163k, so the next run needs a GPU flock path or a smaller selection overhead.

**What this does not establish.** It is one window/k setting (`flock:8:8`), a fixed table, and a 19.9M chat stack at context 384. It does not show that softmax-free reads cannot work. It shows that a fixed `1/(r+1)` table costs about 0.04 BPB here.

**Decision.** REJECT B and D. The served M4 model keeps the softmax read, and acceptance item 0 stays open. Line "softmax-free served read" reaches **3/3**, so a pivot card follows on #2032. Per the pre-registration the read is redesigned, not dropped: the one decisive run is rank weights learned in training, served as a constant table, so still no softmax at runtime.

**Provenance and cost:**
- **Code:** A was trained at `main` 5b241d89c. B was trained at f46ec1964 (code-equal to #2140's merge 5a389aed). D was trained at 5a389aed. Exports and D11 evaluation ran at #2144's head.
- **Pods:** `gl992wcbgkrfhd` (2×5090, about 2.2 h) and `s4cx32npvlhhfd` (2×4090, about 2.7 h; two 5090 hosts in EU-RO-1 stalled at start-up), both deleted, about **$10** in total.
- **Laptop:** CPU for the replies, grading, exports and D11 evaluation.
- **Results:** run directories on the EU-RO-1 volume `rfsx702p68` under `uor-r4/claude/softmax-free-20261010`; checkpoints, artifacts, replies, grades and evaluations in cloud-store `claude/softmax-free-20261010` (566,367,232 bytes, MD5 `d43d071d1596657c68f625829c1a3677`).

## The decisive run (pivot at 3/3): rank tables learned in training — mechanism

Pivot card: [#2032 comment 6096272058](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6096272058). This is the one preparation PR the card allows.

- **Trainer, `read_weighting=learned_rank`:** each read layer gets a variable `read.rank_logits` `[heads, window + k + 2]`, initialized to `ln(1/(r+1))`, so a fresh model equals the fixed `rank` read.
  - For a row whose support (flock-kept sources plus NoRead) has size `m`, the weights are `softmax(ℓ[..m])` by rank, built as a table with tensor ops so the logits get an exact gradient.
  - q, k and aux keep the straight-through flock-softmax gradient. The logits are not weight-decayed.
  - Saved with the model, refused under bf16, carried by `train` and `dialogue-train`.
- **Serving:** schema `/3` with `read_weights: "learned"` and one u32 table `read_rank.<layer>` of `heads × M × M` Q31 weights. Each row is rounded down and the remainder given to the largest fractional parts, so every row sums to exactly 2^31.
  - The D11 engine reads the row for (head, m) by rank. The table is a sealed constant, so there is no exponential or softmax at runtime.
- **Checks:**
  - Trainer `read_weighting` 9: a fresh learned read equals the fixed one; finite-difference gradient to the logits; trained logits change; save/load.
  - Export 4: learned D11 against float −0.061 nats, top-1 1.0 on random weights, and served differently from the fixed table.
  - Integer lib 284: table rows by rank; malformed tables refused. Oracle 13/13; frozen D10 25.
  - D11 stack audit FULL PASS (91 functions). Negative gates on the forward table, the engine table and the exported mode.
- **The run (arm L):** started from this branch's trainer head `383e49410` on pod `583vvwhk05yy1p`. Seeds 1 and 2, `select=flock:8:8`, the M4 recipe. It has 19,929,424 parameters, which is A's plus 288 rank logits. The result is added by the result PR.
