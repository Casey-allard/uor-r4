# Pre-registration — M1 read-code capacity diagnostic

## 4.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat
- **Authority:** owner D22/D24/D32; [M1 admission sheet](m1-discrete-rung0-admission-2026-10-15.md); [D34](../../integration/DECISIONS.md:1642); [Kimi PM substrate review 2026-10-17 (PR #2572 thread)](https://github.com/UOR-Foundation/uor-r4/pull/2572), the math-falsifier review that produced this dispatch.
- **Line:** M1 qualification; count **0/3**, suspended headline **37/232**, unchanged. No mechanism negative, KEEP, acceptance or count reset.
- **Delivery:** **docs only**. No instrument, kernel, launch, allocator or model change in this PR. Every new measurement is **NOT_RUN**. This is the design submitted for PM-seat review.
- **Scope:** **diagnostic, not qualification / not a capability claim / not a fresh-transfer result**. Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**. FIT releases none of these holds. These are not D-Release-class decisions; no owner hold changes.

## 4.2 Question and smallest falsifiable hypothesis

**Question.** At the tested implementation scope — seed-1 Q4, `C192 / B4 / V4096`, `ReadSpec::Route`, `K=8`, frozen 14,287-row DEV panel — is the wall the **read-code representation** (at most `16^B = 65,536` codes) or the **learned output map / training law** (16,384 codebook vectors + 20 Route trits + K=8 admission, trained by one-coordinate integer-hinge descent)? The diagnostic compares a table keyed by the exact read code against **two co-primary ordinary references** — the model-matched masked-TRAIN unigram and the query-token bigram — and against the trained writer.

Define on the same 14,287 DEV rows:
- `T` = top-1 hits of the exact-code table built on the model-matched 1,049,856 TRAIN targets;
- `U` = top-1 hits of the model-matched unigram built on the same 1,049,856 targets;
- `G` = top-1 hits of the query-token bigram built on the same targets;
- `M = max(U, G)`;
- `b_X`, `c_X` = paired discordance against comparator X (`b` = X hit & table miss, `c` = X miss & table hit);
- `F_X = max(24, 2·sqrt(b_X + c_X))`, `F = max(F_U, F_G)`;
- `W = 149` (Q4 writer full-panel top-1), `H = max(24, 2·sqrt(b_W + c_W))`.

**H_rep (representation wall).** `W − H ≤ T < W + H` and `T ≤ M + F`: the exact-code table is not distinguishable from the trained writer and does not beat **both** co-primary ordinary references by their floors. The 16-bit code adds nothing beyond the trained map at this scope.

**H_law (learning-law wall).** `T > M + F`: the code table beats both `U` and `G` by the larger of the two paired floors, so the read code carries information the trained output map fails to extract.

**H_mixed.** `W + H ≤ T ≤ M + F`: the code table beats the trained writer but not both ordinary references. The code is partially informative; the next diagnostic is code utilization/capacity, not a learning-law change.

**Writer-dominates (descriptive).** `T < W − H` and `T ≤ M + F`: the trained geometric map extracts more than per-code counts; report the coverage curve and do not infer a representation wall.

**Disconfirming observation at this scope.** The four bands are disjoint and exhaustive: H_law is `T > M+F`; every other band is `T ≤ M+F` and partitions by `T` against `W+H` and `W−H`.
- H_law is **disconfirmed** if `T ≤ M + F`.
- H_mixed is **disconfirmed** if `T < W + H` or `T > M + F`.
- H_rep is **disconfirmed** if `T < W − H`, `T ≥ W + H`, or `T > M + F`.
- Writer-dominates is **disconfirmed** if `T ≥ W − H` or `T > M + F`.
- Every band is subject to the §4.6 evaluability rule; if it fails, the result is **UNDERPOWERED** and no hypothesis fires.

The upper boundary of each band is strict (`T = M + F` is never H_law); under the normal ordering (`M + F ≥ W + H`) `T = M + F` is H_mixed, `T = W + H` is H_mixed, and `T = W − H` is H_rep, so no outcome is double-assigned. If `M + F < W + H`, the H_mixed interval is empty; every `T` still maps to H_law, H_rep or writer-dominates.

**Pathological-ordering guard.** `M + F < W − H` is unreachable with the measured unigram/bigram rates (both are far above the writer); if it is ever observed, `T < W − H` is H_rep regardless of the `M + F` comparison, because the trained map then exceeds the code table by more than paired noise. Report the ordering as an analysis note.

## 4.3 Observable signature and per-row outputs

The diagnostic produces one lossless per-row record per DEV row (14,287) and a compact code/target stream for TRAIN (2,425,925 rows; 2 bytes code + 2 bytes target ≈ 9.8 MB). No free-running generation is loaded; the model is scored teacher-forced.

Per DEV row:

| Field | Type | Source |
| --- | --- | --- |
| `row_ordinal` | u32 | frozen panel order; `ordered_row_binding_sha256 = e4ef44fb…` |
| `doc_id`, `target_position` | u32 | pinned `ft/dev` tokens/mask |
| `query_token`, `target_token` | u16 | pinned corpus |
| `code` | u16 (4 nibbles) | 4 root ids from `Forward::vectors()` (`device.rs:325-327`) |
| `root_first`, `root_last` | u8 | code nibbles 0 and 3 |
| `admitted_cells[16]` | u16 | `Forward::admitted_cells` (`device.rs:307-308`, `:1217-1219`), 16 slots, `0xFFFF` = absent; cell id `255` is valid |
| `admitted_count` | u8 | number of non-`0xFFFF` slots (`0..=16`; K=8 at this scope) |
| `writer_prediction`, `writer_hit1`, `writer_hit8` | u16/bool | sealed Q4 receipt, re-verified; `Forward::predictions` on a fresh forward |
| `table_prediction`, `table_hit1`, `table_hit8` | u16/bool | model-matched count table |
| `table_prediction_admitted`, `table_hit1_admitted`, `table_hit8_admitted` | u16/bool | table restricted to the same `admitted_cells` |
| `table_sym_prediction`, `table_sym_hit1`, `table_sym_hit8` | u16/bool | secondary 64-action orbit-pooled table |
| `address_only_prediction`, `address_only_hit1` | u16/bool | 8-bit `(first,last)` table |
| `interior_only_prediction`, `interior_only_hit1` | u16/bool | 8-bit `(root1,root2)` table |
| `unigram_1049856_hit1`, `unigram_1049856_hit8` | bool | model-matched unigram (primary) |
| `unigram_full_hit1`, `unigram_full_hit8` | bool | full-TRAIN unigram (secondary; sealed reference `1,226 / 4,060`) |
| `bigram_prediction`, `bigram_hit1`, `bigram_hit8` | u16/bool | query-token → most frequent successor from the same 1,049,856 TRAIN rows |
| `code_train_count`, `code_majority_fraction` | u32/q16 | matched table |
| `seen_code`, `fallback_used` | bool | coverage flags |
| `source_sha256`, `checkpoint_sha256`, `panel_binding_sha256` | hex | identity |

`Forward::admitted_cells` is a `Vec<u32>` of 16 slots per row with sentinel `ABSENT = 0xffffffffu` (`discrete.cu:15`; `device.rs:307-308`, `:1217-1219`). The record re-encodes each slot as `u16` with `0xFFFF` as absent; cells are `0..=255`, so cell id `255` remains a valid cell and is not a sentinel. A presence bitmap is an equivalent encoding.

`unigram_1049856_hit1_admitted`, `unigram_1049856_hit8_admitted`, `bigram_hit1_admitted`, `bigram_hit8_admitted`, `table_sym_hit1_admitted`, and `table_sym_hit8_admitted` are derived aggregates computed from each row's `admitted_cells` and the corresponding prediction; they are not separate raw capture fields.

Aggregates are exact integer sums per predictor and per denominator (14,287; 1,920 admitted; 3,847 `P_succ`; 10,440 complement; 1,335 conflict), plus top-8 sums and the `b+c` discordance counts against each comparator.

## 4.4 Strongest simple controls and matching

1. **Co-primary ordinary control A — model-matched unigram (1,049,856).** Histogram of the exact 1,049,856 masked TRAIN target tokens the model consumed (16,404 batches × 64, in corpus order); top-1/top-8 on DEV. This is the primary marginal reference. The sealed **1,226/14,287 (8.58%) top-1 and 4,060 top-8** is the full-TRAIN (2,425,925) **secondary** reference; it is not the model-matched value and must not be substituted for `U`.
2. **Co-primary ordinary control B — query-token bigram (same 1,049,856 rows).** Sparse 4,096 × 4,096 counts of the query token `tokens[p-1]` to the target `tokens[p]` at each consumed TRAIN target position `p`; tie by lowest token id; model-matched unigram fallback for unseen query tokens. It ignores the 192-token window but uses token identity and adjacency. If the 16-bit code table cannot beat it, the code adds nothing beyond token identity.
3. **Same-representation comparator — the trained writer.** Q4 = 149/14,287 overall and 149/1,920 = 7.76% conditional on its admitted rows; sealed per-row hits (`evaluation-16404.json`, sha256 `4fff1d3c…`). This is the exact artifact the diagnostic tests.
4. **Admission-matched comparator.** The code table restricted to the writer's admitted cells, using the same rule as D34 §6.4 item 7: for each row, `cell(t) = root_id(codebook[t,0])*16 + root_id(codebook[t,B-1])` for every token `t`, and the table prediction is `argmax_{t : cell(t) ∈ admitted} count[code][t]`, tie by lowest token id, with the model-matched unigram fallback restricted to the admitted set; no host recomputation of admission. This is the decisive conditional comparison, because top-1 ≤ admitted rows (1,920 at K=8); the same restriction is applied to the writer, unigram, bigram and symmetry arms.
5. **Data-scaling arm (secondary).** A table and unigram built on the full 2,425,925 TRAIN targets, reported separately and never substituted for the model-matched arm. It tests whether more data alone would change the readout.
6. **Symmetry-pooled secondary control.** `table_sym` pools counts over the (at most) 64-element orbit of the code under the 64 signed SpiralCore actions applied uniformly to all four root blocks, canonicalized by the minimum resulting code id; any action output that is not an exact root is an instrument fault. Tie by lowest token id; unigram fallback. This control imposes an invariance the programme does not assume, so it is descriptive and cannot fire H_law/H_rep.
7. **Cost-matched.** Host counting (seconds to minutes); no new model parameters; no new training.

## 4.5 Design and dose

**Instrument and scope split.** The existing `Device::forward` exposes the read vector (`Forward::vectors()`, `device.rs:325-327`), the writer's predictions (`predictions`), and the admitted cells (`admitted_cells`). Admission is computed inside the writer (`discrete.cu:307-345`), so a writer-skipping capture cannot produce admitted cells. The instrument is therefore split:
- **TRAIN q-capture only.** A read-only variant calls the same `read_vector` (`discrete.cu:270-273`) and captures the 32-int q for each of the 2,425,925 masked targets, skipping only the writer. It makes no admission claim and is never used on DEV.
- **DEV production forward.** The unmodified production `forward` over 224 batches (223 × 64 + 15) yields `predictions`, `admitted_cells`, losses and per-row hits. No read-only path is used on DEV.

The read-only TRAIN variant changes `source_sha256`, so it requires:
- a reviewed evaluation-only source/checkpoint bridge (same pattern as the Stage-A instrument, `stage-a-admission-instrument-2026-10-15/README.md:13-18`);
- the production loader unchanged;
- a `uor-harness ratchet d23|d25|d32` pass on the new source;
- no training path change.

If the PM refuses a source change, the fallback is the existing production `forward` on the model-matched 1,049,856 rows only, with no full-TRAIN arm. That fallback is projected at 1–2 GPU-hours and biases against H_law (a smaller table is a weaker comparator); it must be labeled as such.

**D24 classification.** Building and scoring the count tables on the host is post-processing of GPU-produced read codes and targets, not model forward or scoring. It follows the accepted "CPU audits stored GPU observations" pattern (`stage-a-admission-instrument-2026-10-15/README.md:31`). This classification is flagged for PM/owner confirmation (assumption 7); if refused, the GPU `Device::count_reference` path computes only the unigram, and the code/bigram tables would require a GPU implementation before execution.

**Dose.**
1. Restore the four pinned checkpoints; authenticate each against `a5a22dce…`, `dde889aa…`, `102beed4…`, `4477eb5c…`.
2. Load Q4 (`4477eb5c…`, cursor 16,404 completed batches, **407 accepted updates, epoch 407**; `training.rs:778` enforces `model.epoch() == accepted_updates`) with `ReadSpec::Route`, `K=8`.
3. Enumerate TRAIN in pinned corpus order: for each of the 2,425,925 masked targets, build the C192 window exactly as `PreparedDriver::batch` does (`training.rs:265-272`); capture the 32-int read vector q and the target token. The first 1,049,856 rows form the model-matched table; all rows form the full table.
4. Enumerate DEV with the production `forward`: 14,287 rows in 224 batches (223 × 64 + 15), same order; capture q, admitted cells, writer prediction and writer top-8.
5. Host: decode each 8-int block to a root id by exact match (`root_id`, `discrete.cu:282-285`); any `ABSENT` is an instrument fault, not a finding. Build `count[code][token]` for both TRAIN regimes; score DEV with a fixed tie rule (highest count, then lowest token id) and a fixed fallback (model-matched unigram for the code and bigram tables). Build the address-only, interior-only, symmetry-pooled and unigram tables.
6. Join to the sealed Q4 per-row hits by ordinal; verify `ordered_row_binding_sha256 = e4ef44fb…` and the target identity on every row.
7. Emit the per-row record and exact aggregates, including `b+c` against each comparator and the coverage/multiplicity quantities in §4.6; run the instrument-validity checks.

**Zero-apply statement.** No `apply`, proposal, optimizer, epoch advance or checkpoint rewrite. `snapshot_unchanged: true` and `updates_applied: 0` before/after every batch. The checkpoints are read-only inputs. The capture is a forward pass only.

**No softmax / no matmul statement.** The design introduces no softmax, softplus, sigmoid, tanh, GELU or normalized exponential, and no matrix multiplication/GEMM. Counts are integer additions; scoring is exact integer comparison; the read is the existing integer `read_vector`; the writer is the existing integer `writer`. Any prohibited site in the new source is a blocker under D23/D25/D32, not a design choice.

## 4.6 Bars and uncertainty

- **Evaluability (precedence over every hypothesis).** Before the decision table is applied, the record must show:
  - **Coverage:** ≥90.00% of DEV rows (≥12,859/14,287) have a code with ≥1 count in the model-matched 1,049,856 table. Report the exact numerator/denominator and the coverage curve by count.
  - **Multiplicity:** ≥1,024 distinct codes have ≥2 TRAIN counts in the model-matched table. Report the exact count, the count histogram, and the fraction of DEV rows whose code has ≥2 counts.
  - **Paired discordance:** for each co-primary comparator, `b+c` is reported. If `b+c ≥ 100`, the normal bar `2·sqrt(b+c)` is used; if `b+c < 100`, the exact two-sided McNemar/binomial test is reported and the result is UNDERPOWERED unless it rejects at α = 0.05.
  If any check fails, the result is **UNDERPOWERED**; no hypothesis fires.
- **Full panel, n = 14,287.** For each comparator X: `b` = X hit & table miss, `c` = X miss & table hit; difference `c − b = T − top1(X)`; null SE `sqrt(b + c)`; per-comparator bar `max(24, 2·sqrt(b+c))`. The **24-row minimum** is a rounded 2-SE floor at 149/14,287 (2 SE = 24.3 rows); the paired `2·sqrt(b+c)` bar is operative and is used consistently for every full-panel top-1 comparison. The design's unpaired 2-SE figure for the sealed full-TRAIN 1,226/14,287 reference is 68 rows (exact 2 SE ≈ 67; the registered 68 is conservative); it is reported as a descriptive secondary figure, not used as the model-matched floor.
  - H_law requires `T > M + F` with `F = max(F_U, F_G)`; this implies each co-primary paired bar is met.
  - vs writer (149): `max(24, 2·sqrt(b_W+c_W))`.
- **Admitted conditional, n = 1,920.** Writer 149/1,920 (7.76%), SE ≈ 11.7; floor **24 rows** (rounded 2 SE) or the paired `2·sqrt(b+c)`, whichever is larger. This is the decisive writer comparison; the same floor applies to unigram/bigram restricted to the admitted set.
- **Sub-panel, n = 3,847.** Floor **39 rows**, the M4 sheet's `P_succ` floor (`m4-finite-relational-head-2026-10-15.md:30-31`). It is **top-8-derived** (from the Q1–Q3 `P_succ` top-8 rates and the 39-row seed floor) and is **conservative** for top-1 on 3,847 rows; it is used only for sub-panel top-8 statements, and top-1 sub-panel results are descriptive.
- **Top-8.** Report code, unigram, bigram and writer top-8 against the model top-8 **747/14,287** and the full-TRAIN unigram **4,060**; the top-8 floor is **63 rows** (the M4 B2 floor), with the paired `2·sqrt(b+c)` term where applicable.
- **Wilson 95% intervals** for every rate; report the interval, not just the point.
- **D22 power.** The full 14,287-row frozen panel exceeds the ≥200-row requirement; no training and no seed is involved, so seed noise is not applicable. This is a diagnostic, not a decisive learning KEEP; a capability claim would require a fresh ≥2,000-row draw under a separate pre-reg.

## 4.7 Decision rule

Apply in order; the first matching row fires. The bands are disjoint and exhaustive for every integer `T`.

| Precedence | Observation (model-matched, full panel) | Interpretation at this scope | Next decision |
| --- | --- | --- | --- |
| 0 | Root mismatch, TRAIN q-capture ≠ production q, hash/epoch change, count-sum mismatch, `ABSENT` root, symmetry action not a root, CUDA/status fault | INSTRUMENT FAULT | Diagnose/repair (D31); never a mechanism negative |
| 1 | Coverage <90.00%, or <1,024 codes with ≥2 counts, or a co-primary `b+c < 100` and the exact test cannot reject at α = 0.05 | UNDERPOWERED | Report coverage, multiplicity, `b+c`; next diagnostic is code utilization, not a capacity verdict |
| 2 | `T > M + F` | H_law: the code carries information the trained map does not extract | Learning-law decomposition (objective vs optimizer vs schedule vs codebook capacity), with the code table as the fixed readout; no substrate change |
| 3 | `T ≥ W + H` (and not row 2) | H_mixed: the code beats the writer but not both co-primary references | Code-utilization/capacity diagnostic; do not change the learning law yet |
| 4 | `T ≥ W − H` (and not rows 2–3) | H_rep: no bar-clearing win over **both** co-primary references; the code table is not distinguishable from the trained writer | Representation/code-utilization diagnostic (realized code count, code–target mutual information, route-learning coverage); no learning-law change yet |
| 5 | `T < W − H` (and not rows 2–4) | The geometric map extracts more than per-code counts; sparsity/coverage likely | Report the coverage curve; do not infer a representation wall |

**What changes the decision.** The single number that flips the programme is whether `T` crosses `M + F`. If it does, the next work is a learning-law decomposition; if it does not, the next work is a representation/code-utilization diagnostic. The admitted-conditional comparison against `149/1,920` localizes whether the output map or the admission channel is the immediate bottleneck.

Under the normal ordering (`M+F ≥ W+H`), the old design's `(173, 1,158)` and `(1,226, 1,294)` sub-intervals are now one closed `[W+H, M+F]` H_mixed band (row 3); the `[W−H, W+H)` band is H_rep (row 4). If the measured references invert that ordering, the ordered conditions still assign exactly one row. `T = M+F` is H_mixed, never H_law, and `T = W+H` is H_mixed, never H_rep, so neither boundary is double-assigned.

**Pathological-ordering exception.** If `M+F < W−H` (unreachable at the measured rates), row 4 (H_rep) takes precedence over row 2 for `T < W−H`, per the §4.2 guard.

## 4.8 Instrument validity

1. **Root round-trip.** Every captured block must match exactly one root; reconstruct the 32-int vector from the four root ids and compare bit-exactly to `Forward::vectors()` on a 1,024-row sample. Any mismatch is a fault.
2. **TRAIN capture agreement (q-only).** On a 1,024-row TRAIN sample scored with the production `forward`, the read-only capture's q must equal the production q exactly. The check does not compare admitted cells: admission is computed inside the writer (`discrete.cu:307-345`), so the writer-skipping TRAIN path makes no admission claim. DEV admission comes from the production `forward` and is checked by rows 3–4.
3. **Sealed-receipt agreement.** The Q4 per-row `top1_correct` and `top8_correct` must match the sealed `evaluation-16404.json` (sha256 `4fff1d3c…`) on all 14,287 rows; the aggregate must be **149/747/1920**.
4. **Count-table conservation.** The model-matched table's total counts = 1,049,856; the full table's total = 2,425,925; the bigram table's total = 1,049,856; no token ≥ 4,096; no negative counts.
5. **Zero-apply.** Snapshot/epoch unchanged before/after; `updates_applied: 0`; checkpoint file hashes unchanged.
6. **Determinism.** The same code stream on a repeat of the first 1,000 rows must be byte-identical.
7. **Coverage/multiplicity record.** The exact coverage numerator/denominator, count histogram, ≥2-count code count, and `b+c` per comparator are emitted; the UNDERPOWERED row must be evaluable from the record without re-running.
8. **D23/D25/D32 ratchets** on the new source; any prohibited site is a blocker.

**Which negatives only trigger diagnosis.** A root mismatch, a capture/production q disagreement, a count-sum mismatch, a hash/epoch change, or a CUDA status fault is an **instrument fault** and triggers repair, not a representation or learning-law conclusion. An underpowered coverage result triggers a code-utilization diagnostic, not retirement of the E8 read mechanism.

## 4.9 Loaded artifacts, lineage and resource projection

| Item | Value |
| --- | --- |
| Base | `origin/main = d325c3037b5a6cebdf9401636f9ac0cf91a259fe` |
| Q4 checkpoint | `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf`; cursor 16,404; **407 accepted updates; epoch 407** |
| Q1 / Q2 / Q3 | `a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd`, `dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822`, `102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982` |
| Q4 evaluation | `4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8` |
| DEV tokens / mask | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` / `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| TRAIN tokens / mask | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (`docs/labs/k16-qualification-restart4-2026-10-16/data-file-sha256.txt`) |
| Tokenizer | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| Ordered row binding | `e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446` |
| Geometry | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Source anchors | `discrete.cu:15` (`ABSENT`), `:41-77` (Route read), `:71` (root output), `:264` (Flock/Relation root output), `:270-273` (dispatch), `:282-290` (root_id/token_cell), `:307-345` (writer), `:471-481` (admitted-only ranking); `device.rs:307-308`, `:325-327`, `:1217-1219`, `1453-1505`; `training.rs:265-291`, `:778`, `859-897`; `lib.rs:47-65` |

**Job time.** 10–30 GPU-minutes for the read-only q-capture of 2,425,925 (2.43M) TRAIN rows plus the production DEV forward (224 batches); host counting 1–5 minutes; build/controls 10–20 minutes.

**Lease.** One RTX 5090 through `uor-pod` only; one **1-hour lease at $0.908/h = $0.908**, covering build, capture and host scoring. The job estimate is 10–30 GPU-minutes inside that hour. If the PM refuses the source change, the production-forward fallback on the 1,049,856 TRAIN rows is projected at **1–2 GPU-hours = $0.91–$1.82**, requiring a **2-hour lease ($1.816)**; label the smaller-table bias. <32 MiB derived records; $0 new GPU cost for this docs delivery. `df ≥ 30 GiB` before lease; caps ≤4 pods and ≤$8/h across labs.

**Stop conditions.** First root mismatch, capture/production q disagreement, pin/hash mismatch, snapshot/epoch change, count-sum mismatch, CUDA/status fault, or disk-floor fault stops the affected execution and seals partial evidence for PM disposition. No on-lease patch, extra pass, or unreviewed retry.

**Record schema.** `uor-r4.read-code-capacity/1`, with `applied_updates: 0`, `model_compute_executed: true`, `zero_apply: true`, `epoch: 407`, `source_sha256`, `checkpoint_sha256`, `panel_binding_sha256`, per-row records, exact aggregates, and the §4.6 coverage/multiplicity/`b+c` fields. Seal with the existing `report_output::claim`/`seal` discipline.

## 4.10 D22, claim discipline, FIT and assumptions

**D22.** A negative closes the `(C192 / B4 / V4096, Route, K=8, seed-1 Q4, this code/readout)` configuration. It does not close the E8 read mechanism, the learning law, or any wider substrate. Stopping a mechanism family needs the owner.

**Claim discipline.** No capability claim. This is a diagnostic measurement on a frozen panel; it is not fresh transfer, not a language result, and not a qualification. A KEEP-style follow-up would need a fresh ≥2,000-row draw under a separate pre-reg.

TEST FITNESS: FIT (design) — Kimi PM seat with independent DeepSeek architecture review, [PR #2572](https://github.com/UOR-Foundation/uor-r4/pull/2572) at `f53c5198a`, copyedits `b67567d01`. FIT authorizes no compute; execution releases are separate PM steps.

**FIT / execution release.** The FIT verdict applies to this design only. It authorizes no compute. Actual execution requires a separate PM release, exact source/device pins, the evaluation-only bridge, and (for any D-Release-class decision) owner release. These three diagnostics are not D-Release-class.

**Assumptions the lead must resolve.**
1. Authorize the read-only TRAIN q-capture plus production DEV forward, or accept the production-forward fallback on the 1,049,856 rows with the bias disclosure.
2. Confirm the model-matched 1,049,856-target regime as primary, with the full 2,425,925 as secondary.
3. Confirm the 90% coverage threshold, the ≥1,024-code multiplicity threshold, and the `b+c ≥ 100` / exact-test power rule.
4. Confirm the tie rule (highest count, then lowest token id) and the model-matched unigram fallback.
5. Confirm that a count-table readout is an acceptable **diagnostic** comparator under the native geometric programme, even though it is not a proposed serving mechanism.
6. Confirm the 24-row minimum full-panel floor, the paired `2·sqrt(b+c)` rule for each co-primary comparator, the 63-row top-8 floor, and the 39-row top-8-derived sub-panel floor.
7. Confirm the D24 classification: host table build/scoring is post-processing of GPU-produced observations, not model forward/scoring.
8. Confirm that the symmetry-pooled table is secondary/descriptive and cannot fire H_law/H_rep.
