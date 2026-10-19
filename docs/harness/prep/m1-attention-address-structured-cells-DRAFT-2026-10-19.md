# Address-structured injective cells for the learned attention read: pre-flight only — DRAFT

- **Status:** DRAFT, pre-flight only. **TEST FITNESS: FIT** (2026-10-19, second-round fresh-context adversarial review; see Review below).
  - **This sheet runs pre-flight only:** zero training updates on any arm. A training sheet follows only if an arm clears the gates below.
  - Training stays HELD. PM dispatch alone does not run this pre-flight. The owner gate is in **Registration bindings** below.
- **Line:** M1 learned attention (#2029). **Lab:** claude.
- **Owner decision:** Claude chat, 2026-10-19 — "draft an address-structured injective cell table sheet (with the K-scaled table as a bracketing control), pre-flight only first."
- **Parent and context:**
  - The injective sheet `m1-attention-injective-init-DRAFT-2026-10-18.md` (FIT).
  - Its executed pre-flight (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/`).
  - The successor options card (#2029 `6257846074`).
- **Rules:**
  - D24: GPU only for every model quantity. Admission is scored on the device; the host only builds tables from TRAIN counts.
  - D23/D25/D30: no softmax, squashing or matmul anywhere, including the construction.
  - D32: integer tables.
  - D22: a negative closes a configuration only.
  - D40: no owner phrase is involved in a zero-update pre-flight.

## Registration bindings (2026-10-19)

These answer DeepSeek's four registration items (#2029 6258174049) and bind Kimi's answers (6258190943). None changes a bar, an arm or a gate.

- **Source (item 2).**
  - (a) is merged: #2742, `2811c6e93e886552d34f26cccf031d67a004a649`. The FIT'd sheet is merged: #2746, `05b90b4e5d1ddbd00b70564c0edf1d0880d7ca9b`.
  - (b) and (c) need no code. The K = 16 arms run only `init-corpus` and `eval-corpus` (`main.rs:78-100`, `DriverConfig::validate`). They never reach the attention validator's K = 8 pin (`attention_learning.rs:169`), which DeepSeek's 6258208112 also locates. The `attention-*` commands run on the K = 8 arms only.
  - If `eval-corpus` refuses a K = 16 config at the pod, P16 and A16 are dropped as diagnostics. They decide nothing, and no gate on P8, A8 or S8 depends on them.
- **Source freeze.**
  - The battery runs at the `origin/main` commit named in the PM's costed plan. The `attention-build-identity` output at that commit is recorded with the receipt.
  - #2743 (row-proposal scratch reuse) is **neither required nor excluded**. By source, the battery calls no `propose` or `apply`:
    - `attention_preflight.rs` has no such call;
    - `init-corpus` writes checkpoint 0;
    - `eval-corpus` launches `Device::evaluate`'s own kernel (`device.rs:1493`).
  - So including or omitting #2743 cannot change an arm, and P8's exact reproduction of 427 / 127 checks the frozen commit either way.
- **Owner gate (item 3; Kimi 6258190943).**
  - PM dispatch does not run this pre-flight.
  - Kimi brings Casey the exact costed plan (his standing instruction 6258074111: a costed decision before any rental).
  - Casey's explicit approval in chat is the recorded owner authorization for the rental and the pre-flight.
  - Any owner-identity phrase in the runbook is typed by Casey personally (D40); no lab types one.
  - The "about $0.40 / under 30 min" figure in **Cost** is this sheet's estimate, not a quote; the PM's plan carries the quote.
- **D41 and Stage T (item 4).**
  - This pre-flight is separate from D41 donor work and from the Stage T learning-law draft.
  - It uses no donor and releases no training hold. Its result does not change D41's sequence.
- **Design (item 1).** Kimi confirmed placement-only 256 × 16 as drafted. The 32-cell compact form stays parked as the fallback if A8 fails its floor.

## Why

The injective PrimeZeta table removed aliasing: 4,096 distinct vectors, a ceiling of 14,287 and zero tie mass. But it spread tokens uniformly over all 256 cells. With K = 8 admitted cells, a target's cell was admitted in only 427 of 14,287 DEV rows (3.0% ≈ 8/256), and in 127 of 3,847 P_succ rows, below the 500 floor.

The read can only rank what it admits. This sheet tests whether **placing each token's cell where the read looks** raises admission without giving up injectivity.

## Mechanism facts this design rests on (from source, `origin/main`)

- **Cell of a token.** `token_cell = root_id(block 0) · 16 + root_id(block B−1)` (`discrete.cu:286-289`). Every block must be one of the 16 orbit roots (`discrete_validate`, status 6).
  - Any token can therefore be placed in any cell, and each cell has 256 distinct vectors available.
  - Blocks 1 and 2 are free, but they enter within-cell ranking (`token_distance`) and the read's payload.
- **Admission at checkpoint 0.** The writer admits the `kept_cells` = K occupied cells nearest the read output `q`, by L1 distance on (block 0, block B−1) roots, with ties to the lower cell (`writer`, `discrete.cu:307-318`).
  - At checkpoint 0 (ActiveLeafV1: leaf gates +1, every other gate 0), `q_b = snap_2(u_b + m_b)`. Here `u` is the **last context token**, and `m` is the snapped consensus of up to 8 earlier context leaves, ranked by frame-root proximity (`flock_vector`, `discrete.cu:125-268`).
  - So **admission is centred on the last token's address, pulled by the context consensus.** It is not the last token's cell alone.
- **Admission is a model quantity.** Predicting it means running the read and the writer. Under D24 it is measured only on the GPU, through the existing zero-update pre-flight tools. The host constructs candidate tables and never scores them.
- **K limits.** Writer K has a maximum of 16 (`Spec::validate`; 16-slot admitted buffers), and the attention validator pins K = 8. The **bracketing control therefore uses K = 16**, the largest supported, through the generic `init-corpus` / `eval-corpus` path.

## Construction (host data preparation from TRAIN only; deterministic; integer)

**`address_structured_v1`** builds an injective V4096 / B4 table in which a token's cell is chosen to sit near the cells of the tokens that **precede** it in TRAIN. It does this in four steps.
1. **Count.** Count TRAIN bigrams `(s → t)` at the driver's **eligible response-masked target positions** (`corpus.rs:243`, `eligible_targets`).
   - Each scored target at `position` gets the window `tokens[start..position)` (`corpus.rs:270-278`; policy `response-masked-targets/within-document-predecessor/…`, `corpus.rs:9`). So `s = tokens[position − 1]` is the **last context token** of that row.
   - The counts are integers. Their file is pinned by SHA-256.
2. **Cell geometry.** Each cell is a pair of root ids `(r0, r3)`. Distance is the same L1 on root coordinates that the writer uses.
3. **Greedy placement.** Tokens are taken in descending TRAIN target frequency, with ties to the lower token id. Each token goes to the cell minimising `Σ_s count(s→t) · L1(cell(s), cell)` over its already-placed predecessors.
   - The placement respects a **per-cell quota of 16**, so occupancy stays 256 cells × 16, exactly as for PrimeZeta. **The only change is where tokens sit, not how many cells are used.**
   - **Zero placed-predecessor mass.** This covers every token whose predecessors are not yet placed, not only the first. Such a token goes to its PrimeZeta cell if that cell has quota left; otherwise to the nearest non-full cell by L1, then by cell id. This prevents low-id packing.
   - The remaining ties go to the lowest cell id.
   - **Known limitation:** only `s → t` edges with `s` already placed pull a token. Edges toward later-placed successors never pull earlier tokens. This is stated, not corrected, in v1.
4. **Distinctness.** Within a cell, blocks 1 and 2 are assigned distinct root pairs in token-id order, so all 4,096 vectors are distinct. `distinct_vectors` is recounted on load.
5. **The proxy, stated.** The greedy optimises the L1 between a target's cell and its **predecessor's** cell. Admission is centred on the read output `q_b = snap_2(u_b + m_b)`, not on `cell(u)`. The frame for block 0 is block B−1 (`frame_block`, `discrete.cu:112`). So `cell(q) = cell(last token)` holds only partially.
   - **P8 reports, on the device, the distribution of `address_distance(q, cell(last token))`** over the DEV rows, **reported if an existing device output supplies it; not a gate.** The read trace already records the output and the root of `u`. That distribution states how much any predecessor-keyed placement can gain.
   - B3 ≥ 500 needs about 13% P_succ admission, against 3.3% today, roughly a 4× gain.

**Source work before any run:**
- **(a) Constructor and exporter — merged** (#2742, `2811c6e93e886552d34f26cccf031d67a004a649`, verified on `origin/main`; record `docs/labs/address-cells-codebook-2026-10-19/`): `src/address_cells.rs` and `address-cells-codebook`. The tables are pinned from the real TRAIN export, and the pod rebuild must reproduce them:

  | Arm table | `table_sha256` | file SHA-256 |
  |---|---|---|
  | A8 / A16 (`address_structured_v1`) | `4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74` | `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1` |
  | S8 (`address_shuffled_v1_seed_20261019`) | `d2fb8680a16e8017f6f0e6518df6810f6206023b0d14388dc076d4cfde2ebc5b` | `a185bdca07077fe7157e947ff1fa80919e6f99165a4dd5a04061a97873c92718` |

  The counts SHA is `755a5a81…`, shared by both tables.

  #2742 also adds two read-only accessors to `corpus.rs` (`eligible_positions()`, `tokens()`), so the driver hash changes. P8's exact reproduction of 427 / 127 under the new hash is the check that this change has no behavioural effect.
- **(b) Attention plumbing — no code needed, by source.**
  - The table loader checks no `mode` string.
  - `attention_learning::scope` keys on the table file SHA prefix.
  - `validate_injective_table` requires only `distinct_vectors == V`.

  The host build refuses every model command (D24), so this is **confirmed on the pod** by `attention-build-identity` on the A8 / S8 configs as the battery's first step. A refusal there is an instrument blocker.
- **(c) K = 16 path — no code needed, by source.** `init-corpus` / `eval-corpus` call only `DriverConfig::validate` (`main.rs:78-100`). That calls `ReadSpec::validate`, which caps the **read** kept count at 8 (`lib.rs:145-148`), and `Spec::validate`, which allows **writer** `kept_cells` up to 16. The attention-only K = 8 pin (`attention_learning.rs:169`) is not on this path.
- **Arm configs:** staged in `docs/labs/address-cells-preflight-configs-2026-10-19/evidence/` as P8, A8, S8, P16 and A16. Each differs from the injective treatment config (`docs/labs/injective-attention-runbook-2026-10-19/evidence/injective-relation-seed1.json`) only in the table pin and writer K.

The construction reads **TRAIN only** (pin `3dee858f…`). DEV is never used to choose cells.

## Arms (all checkpoint 0, zero updates, seed 1, Relation{8,1}, objective A, ActiveLeafV1, interleaved, `rows_v1`, Dose 1 pins)

| Arm | Table | Writer K | Role |
|---|---|---|---|
| P8 | PrimeZeta (`83c1380a…`) | 8 | **Control, and an exactness gate:** must reproduce 427 full-panel and 127 P_succ target-admitted rows exactly. Any difference is an instrument fault (D31), not a reading. |
| A8 | `address_structured_v1` | 8 | **Treatment** |
| S8 | `address_structured_v1` with the predecessor information removed: the same greedy, but each target's predecessor counts are reassigned to a deterministic seeded permutation of predecessors (a fixed, recorded seed), so frequency order and quota packing are kept | 8 | **Packing control.** It separates the predecessor relation from frequency packing and the low-id tie-breaks. |
| P16 | PrimeZeta | 16 | **Bracket:** admission from width alone. P_succ is predicted at about 254 / 3,847, still below 500 (K ≥ 34 would be needed), so it **decides nothing**. It shows only whether admission scales with K. |
| A16 | `address_structured_v1` | 16 | Bracket: the combined effect, reported only |

The P16 / A16 configs differ from P8 / A8 only in `kept_cells`.

## Bar (fixed before any measurement)

The gate below is the parent injective sheet's §3 battery, applied unchanged to the **A8** arm. It is what would promote A8 to a training sheet:
- **G-Init:** the table equals the pin word for word, both negative controls pass, and routes, gates and epoch equal the unpinned initializer's.
- **G-Occ:** `occupied_cells` > 16, with zero within-cell tie mass and no split alias group.
- **B3_init(P_succ) ≥ 500.**
- **Net G-Cap > max(500, top1_control(cp0) + 24)**, joined from the raw alias ceiling, the zero-update `B3` / `P_succ` admission and the control's checkpoint-0 top-1.

Readings:
- **Effect of placement** (one-sided, fixed before measurement). Paired discordance on the same 3,847 P_succ rows, with `b` = admitted by A8 only and `c` = admitted by P8 only. "Placement moved admission" requires `b − c > F = max(24, 2·sqrt(b+c))`, about 2σ of the null McNemar difference.
- **Attributed to the predecessor relation** only if, in addition, A8 against S8 clears the same one-sided floor. Otherwise the gain is recorded as frequency packing.
- **Proxy ceiling.** P8's `address_distance(q, cell(last token))` distribution is reported alongside.
- **Bracket.** P16 tests whether admission scales with K (expected about 2× P8 if uniform). A16 reports the combined effect. Neither bracket arm can promote anything by itself, because writer K = 16 is outside the attention validator.

## Base artifact

None. Every arm is a fresh checkpoint 0 (`init-corpus`, `new_model(0)`) from its pinned table and config. Each is bound by table file SHA, config SHA and lineage in the receipt.

## Data split

- **TRAIN** (`3dee858f…` / mask `b5e7352d…`) is used only by the host construction.
- **DEV** (`0919e90f…` / `a838bb1a…`, 14,287 rows; P_succ 3,847 by the panel's canonical selection) is used only for zero-update device scoring.
- **Fresh** slices are not touched.

## Steps

1. Source piece (a) is merged (#2742); (b) and (c) need no code. The A8 / S8 tables are pinned above, rebuilt on the pod and required to match.
2. FIT this sheet with a fresh-context adversarial reviewer subagent (owner direction 6255848646), posting every finding (done, #2746). Then follow the owner gate in **Registration bindings**: PM's costed plan, then Casey's explicit chat approval.
3. On one GPU via `uor-pod`, build, then run the following per arm:
   - `init-corpus` and `eval-corpus`;
   - `attention-dev-b3`, for the K = 8 arms;
   - `attention-export-token-cells` + `coverage-census --token-cells`;
   - G-Init with both negatives;
   - P8's `q`-distance distribution: reported if an existing device output supplies it; not a gate, and no new code is written for it.

   **The K = 16 join** is grading of stored outputs on the host (D24 permits this):
   - A row is target-admitted when `admitted_cells − address_rivals == 1`, read from `eval-corpus`'s per-row records (`batches[].rows[]`).
   - Rows join on `(target_position, target_token)` to the K = 8 `attention-panel` rows' `is_psucc`, through an exported file with a pinned SHA.
   - If `eval-corpus` does not accept a K = 16 Relation config, that is source item (c).
   - **Join precondition (an instrument gate; any miss is D31, never a negative):**
     - every `eval-corpus` row has both `target_position` and `target_token` (they are `Option`, `device.rs:67-69`);
     - the keys are unique;
     - there are 14,287 rows;
     - exactly 3,847 rows join to `is_psucc = true`.
   - **Formula check:** on P8's K = 8 rows, counting `admitted_cells − address_rivals == 1` must reproduce 427 overall and 127 on P_succ. Only then is the formula used for P16 / A16.
4. Post the receipt. If A8 clears every gate, draft the A8 training sheet (the parent's Relation versus FrozenZero contrast with only the table changed). Otherwise post the successor decision.

## Stop rule

- Any instrument fault stops for diagnosis (D31), never as a negative.
- An arm failing a gate closes that configuration only (D22).
- Owner caps and the 30 GiB disk floor are hard stops.

## Cost

Under 30 min on one RTX 5090. The build is about 10 min; each of the five arms' checkpoint-0, evaluation and export is under 1 min. That is about $0.40, the same instrument as the 2026-10-19 pre-flight.

## Minimum test power

- B3_init and G-Cap are exact counts on the full fixed DEV panel (14,287 rows; 3,847 P_succ). The 500 floor exceeds the panel's 2-SE row floor.
- The A8-against-P8 and A8-against-S8 effects use one-sided paired discordance on identical rows, with `F = max(24, 2·sqrt(b+c))`. They are reported regardless of the gate.
- P8's exact reproduction of 427 and 127 is an instrument gate.

Review: a fresh-context adversarial reviewer subagent returned FIT-WITH-FIXES on the first draft. All eight findings are applied here:
- the zero-mass placement rule;
- the S8 packing control;
- the stated proxy, with P8's q-distance;
- the row and predecessor citation (`corpus.rs:270-278`);
- the K = 16 join definition;
- the P8 exactness gate;
- the one-sided floor;
- the one-direction limitation.

A second fresh-context adversarial reviewer subagent read the revised sheet against `origin/main` and returned FIT-WITH-FIXES, with three wording fixes and no change to any bar. All three are applied:
- source piece (a) is stated as merged only after #2742 merged and was verified on `origin/main`; P8's exact 427 / 127 is named as the check that #2742's `corpus.rs` accessors change no behaviour under the new driver hash;
- the K = 16 join has a precondition: both keys present, unique, 14,287 rows and exactly 3,847 P_succ joins (any miss is D31), and the admission formula must reproduce P8's 427 / 127 at K = 8 before it is used at K = 16;
- P8's q-distance is reported if an existing device output supplies it, and is not a gate.

TEST FITNESS: FIT — fresh-context subagent review (owner direction 6255848646), fixes applied. FIT releases nothing: compute still needs PM dispatch and the owner's release (D40).
