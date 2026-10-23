# Prefix-holonomy position mechanism on the discrete M1 orbit (`PositionHolonomyV1`)

- **Status:** draft pre-registration, `TEST FITNESS: NOT_FIT` (non-author review `deepseek-36`, 2026-10-22; see the Verdict at the end). No compute is cleared. **Sheet update 2026-10-23 (deepseek-39):** the owner's standing-direction decision registered the §3 bar at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`; card `6337522273`), §5 now pins the full DEV pair from the code pin, and the order-sensitive slice rule is defined with its hash slot `PENDING` for the frozen subset.
  - Training and GPU evaluation remain HELD. A run needs a non-author `TEST FITNESS: FIT` review at a later head (CHARTER §4.5) from Claude or Kimi, the bar registered by the owner's standing-direction decision on `#2028` (`2026-10-23T02:36:36Z`, comment `6356034555`; card `6337522273`), the frozen order-sensitive slice hash, `m1-trunk-assembly-and-training`, PM dispatch, and Casey's typed D40 release.
- **Line:** M1 learned attention and sequence position (#2029, item `m1-position-mechanism` #2843). **Lab:** antigravity.
- **Parent:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](m1-attention-a8-address-cells-learning-2026-10-19.md) (the A8 address-structured cells sheet).
  - **What changes from the parent:** replaces the endpoint-only candidate transport in `Flock` / `Relation` (`discrete.cu` `flock_vector`) with causal prefix-holonomy transport `PositionHolonomyV1` (`crates/uor-r4-discrete/src/position_mechanism.rs`) and adds two matched controls (`EndpointControlV1` and shuffled-prefix evaluation `C_shuf`) to isolate sequence-order credit at equal parameter count.
- **Applicable owner decisions:** D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D40.
  - Geometry only (`Cl(0,6)` 64-frame signed-permutation group acting on the 16-root $E_8$ orbit $\mathcal{O}_{16}$).
  - Zero normalized exponentials or squashing functions (CHARTER §3.2, D23).
  - Zero dense matrix multiplications or GEMM (CHARTER §3.3, D25).
  - Zero floating-point types or literals on the discrete path (D32).
  - Exports to D11 as $16 \times B$ unsigned whole numbers (`u8` in `0..16`) plus fixed lookup tables (`COMPOSE`, `INVERSE`, `AXES`, `ROOTS`), requiring zero multiply instructions at serving time.

## 1. Why: endpoint-only transport is blind to intervening token order

In `crates/uor-r4-discrete/src/discrete.cu` (`flock_vector`), when an earlier position $s < t$ is carried into the query frame at $t = L - 1$, the transport action `g[BFS + frame[s]*16 + fq]` depends only on the two endpoint roots `frame[s]` and `fq`. Permuting the intervening tokens $(s, t)$ leaves that transport unchanged. Meanwhile, the historical D1 quaternion transport (`archive/integration/transport-attribution-d1-2026-09-28.md`) ran inside a float `rrarra` core that used banned squashing gates and dense layers and carried $+332{,}928$ more parameters than its identity arm.

`PositionHolonomyV1` (`crates/uor-r4-discrete/src/position_mechanism.rs`) supplies a lawful discrete position mechanism on the active M1 substrate:
- Each `(block, root)` slot owns a discrete step candidate $\tau[b, r] \in \{0, \dots, 15\}$ (`0` $\mapsto$ `IDENTITY`, `1..=15` $\mapsto$ `GENERATORS[c - 1]`), giving $u_{p,b} \in G_{64}$ at position $p$.
- Along the causal window $p = 0, \dots, L-1$, the cumulative frame updates by a single `COMPOSE` lookup:
  $$F_{0,b} = u_{0,b}, \qquad F_{p,b} = \mathrm{COMPOSE}[u_{p,b} \cdot 64 + F_{p-1,b}].$$
- For any candidate at position $s < t$, the relative path holonomy is formed by one `INVERSE` lookup and one `COMPOSE` lookup:
  $$H_{t \leftarrow s, b} = F_{t,b} \circ F_{s,b}^{-1} = \mathrm{COMPOSE}\bigl[F_{t,b} \cdot 64 + \mathrm{INVERSE}[F_{s,b}]\bigr] = u_{t,b} \circ u_{t-1,b} \circ \dots \circ u_{s+1,b}.$$
Because $G_{64}$ is non-commutative, interior token swaps in $(s, t)$ change $H_{t \leftarrow s, b}$ and alter both candidate frame ranking (`root_cost`) and transported payload consensus (`snap`).

## 2. Construction and matched controls

Three arms share the identical A8 initialization (`address_structured_v1`, file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`), `Spec` ($V=4096$, $B=4$, $C=192$, $K=8$), objective A (`NearestAddressMarginV1`), and proposal schedule:

1. **Treatment (`H_pos` — `PositionMode::PrefixHolonomyV1`):** Learns the $16 \times B = 64$ discrete step candidates $\tau[b, r] \in 0..16$ and applies the causal prefix holonomy $H_{t \leftarrow s, b} = F_{t,b} \circ F_{s,b}^{-1}$.
2. **Parameter-matched endpoint control (`C_match` — `PositionMode::EndpointControlV1`):** Learns the identical $16 \times B = 64$ discrete step table $\tau[b, r] \in 0..16$ on the same schedule and move space, but forms $\mathrm{COMPOSE}[u_{t,b} \cdot 64 + \mathrm{INVERSE}[u_{s,b}]]$ from the endpoints $(s, t)$ alone. Parameter count ($64$ position slots), candidate cardinality ($16$), and proposal visits are matched exactly.
3. **Shuffled-context-order null (`C_shuf`):** Evaluates the trained `H_pos` checkpoint on the identical evaluation windows after permuting the prefix positions $[0 \dots L-2]$ with `shuffle_prefix_roots(window, seed)` while holding the query position $L - 1$ and the prefix token multiset fixed.

## 3. Bar

- **Pre-run finite controls (CPU, landed in `crates/uor-r4-discrete/src/position_mechanism.rs`):**
  1. Identity table leaves all prefix frames at `IDENTITY` and maps all 16 roots to themselves (`identity_table_leaves_all_frames_and_roots_unchanged`).
  2. Prefix holonomy $F_t \circ F_s^{-1}$ matches direct step composition across $1..=243$ and composes transitively $H_{t \leftarrow m} \circ H_{m \leftarrow s} = H_{t \leftarrow s}$ (`prefix_holonomy_matches_direct_composition_and_transitivity`).
  3. All 64 frames $\times$ 16 roots remain in `0..64` and `0..16` with exact coordinates in $\{-2, 0, 2\}^8$, L1 norm 4, squared norm 8 (`states_and_coordinates_stay_within_exact_integer_bounds`).
  4. Non-commutative generators distinguish interior token swaps that `EndpointControlV1` cannot distinguish (`noncommutative_holonomy_detects_order_swaps_while_endpoint_control_is_blind`).
- **Pre-registered "helps order" bar (registered by the owner's standing-direction decision on `#2028`, 2026-10-23T02:36:36Z, comment `6356034555`; card `6337522273`; CHARTER §5.2):**
  - On the 14,287-row DEV panel (`P_succ` 3,847 copy-successor rows, plus the multi-occurrence order-sensitive slice $N \ge 200$ rows where the query token appears $\ge 2$ times in the prefix with distinct successors), across $\ge 2$ seeds:
    1. **Gain over matched-parameter control:** `H_pos(ordered)` beats `C_match(ordered)` on paired one-sided McNemar contrast by $b - c \ge F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ in both seeds, with non-inferior admission ($\Delta\text{admitted} \ge 0$).
    2. **Order-destruction drop:** Shuffling the prefix order on the trained `H_pos` model (`H_pos(ordered)` vs `C_shuf`) reduces top-1 hits by at least $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ in both seeds.
    3. **Fresh transfer:** Any DEV `KEEP` must replicate both contrasts on the sealed fresh V2 draw (`0b7e754c…`) before mechanism promotion.

## 4. Base artifact

- **Trunk / proxy:** Because the full Stage 3 trunk is not yet assembled (`m1-trunk-assembly-and-training`), this test runs on the active discrete M1 proxy trunk (`crates/uor-r4-discrete`, `ReadSpec::Relation { kept: 8, sweeps: 1 }`, A8 pinned codebook `6600bdb1…`), and is re-executed unchanged once `m1-trunk-assembly-and-training` lands.

## 5. Data split

- Inherits the pinned M1 splits from `m1-attention-a8-address-cells-learning-2026-10-19.md`; the full digests from the authoritative code pin (`crates/uor-r4-discrete/src/attention_learning.rs:1372-1384`) are:
  - TRAIN tokens `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (mask `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`; 7,512,797 tokens / 2,425,925 eligible targets);
  - DEV tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`; 42,856 payload tokens / 14,287 targets);
  - Sealed fresh V2 draw tokens / mask / seal `0b7e754c…` / `a1cda529…` / `df5d074a…` (full digests from the parent's registered record; the parent sheet is `TEST FITNESS: FIT`).
- **Errata (staging sheet):** `docs/harness/prep/m1-a8-registration-and-runbook-staging-2026-10-22.md:46` carries `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546` / `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327` for the same DEV split; those digests appear in no code or archived artifact and are wrong against the code pin above. That item (#2832) is `done`, so the one-line errata is a PM action, recorded on `#2028` (the item comment `6356047343`).
- **Order-sensitive slice (rule pinned; hash `PENDING — frozen subset`):** on the pinned DEV tokens, select every row whose query token appears at least twice in its causal prefix with pairwise-distinct successor tokens; require `N ≥ 200` such rows in the frozen subset. The selection is a deterministic function of the pinned DEV tokens and the model's tokenizer, so its row set and SHA-256 are fixed when the subset is frozen; the sheet's `SHA-256` slot stays `PENDING` until that frozen file exists.
- Training never touches DEV or fresh V2 rows (CHARTER §3.10, D22 §4).

## 6. Steps

1. Land `crates/uor-r4-discrete/src/position_mechanism.rs` with its 5 finite CPU controls (`cargo test -p uor-r4-discrete --lib`).
2. Obtain a non-author `TEST FITNESS: FIT` review on this sheet and the owner's answer to the §3 bar decision card on `#2028`.
3. Wire the 64-entry `PositionStepTable` kernel step into the discrete GPU read path (`discrete.cu`) behind `PositionMode` and qualify its PTX admission (`build.rs` `admit_ptx`).
4. After PM dispatch and Casey's typed D40 release, run `H_pos` and `C_match` across seeds 1 and 2 on GPU, plus `C_shuf` evaluation on the trained `H_pos` checkpoints.
5. Score the paired contrasts on DEV (`P_succ` and multi-occurrence order slice) and, if DEV passes, on the sealed fresh V2 draw.

## 7. Stop conditions

- Any PTX admission failure, checkpoint-0 reproduction mismatch, or out-of-range state is an `INSTRUMENT_UNREACHABLE` stop for diagnosis (D31), never a mechanism negative.
- Self-set step or wall-clock estimates are planning figures, not stop triggers; extend and renew rather than cancel a healthy run.
- Hard stops are the owner spend/pod caps ($\le 4$ pods, $\le \$8.00/\text{h}$ across all labs), the 30 GiB / 128 MiB disk floor, or a Q1 admission collapse gate failure.

## 8. Cost and minimum test power

- **Cost (planning estimate at $\$1.894/\text{h}$ on a single leased GPU via `scripts/pod/uor-pod`):**
  - `preparation`: $\$0.00$ (host metadata and finite CPU controls on laptop).
  - `run`: $\approx \$11.40$ per seed pair (`H_pos` + `C_match`, 16,404 batches per arm in parallel, $\approx 6\text{ h}$ wall time per seed pair; $\approx \$22.80$ for 2 seeds).
  - `controls`: included in the paired `C_match` arm above; `C_shuf` is a forward-only evaluation pass on the trained `H_pos` checkpoint ($\approx \$0.20$).
  - `evaluation`: $\approx \$0.40$ (DEV panel checkpoints plus sealed fresh V2 draw on `KEEP`).
  - `retries`: $\approx \$2.80$ (one Q1 diagnostic retry allowance within owner caps).
  - `storage`: $< 64\text{ MiB}$ per retained snapshot pair, respecting the 128 MiB / 30 GiB stop margin.
- **Minimum test power (D22 §3):**
  - Seeds: $2$ paired seeds (seeds 1 and 2).
  - Rows: $14{,}287$ DEV rows ($3{,}847$ `P_succ` rows; $\ge 200$ multi-occurrence order-sensitive rows) plus sealed fresh V2 transfer draw ($\ge 200$ rows).
  - Decisive run: YES ($\ge 2$ seeds and $\ge 200$ fresh rows, trained in from step 0 rather than bolted on).

## Verdict

### Why NOT_FIT (prerequisites in priority order)

1. **The owner bar is not registered.** The top Status line called comment `6337522273` "the owner's bar confirmation"; it is **antigravity's decision card** proposing `F_order`. No owner answer exists (the PM's routing note lists this card among those awaiting the owner), so no compute is cleared. Quote the owner's registered bar verbatim with an **ISO-UTC time**, or keep the card as `PENDING`.
2. **The DEV pin is ambiguous, so the inherited split is not actually pinned.** The sheet carries only prefixes (`0919e90f…`). The repo holds two conflicting full pairs:
   - the **authoritative code pin** in [`crates/uor-r4-discrete/src/attention_learning.rs`](../../../crates/uor-r4-discrete/src/attention_learning.rs) — tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (archive `a2-pilot` matches, and the expected `42,856` payload tokens / `14,287` targets match);
   - the staging sheet's `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546` / `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327`, which appears nowhere else.
   Pin the full code-pin pair in §5 and file an errata for the staging sheet, or show which file each pair belongs to.
3. **The new evaluation slice is unpinned.** The bar's "multi-occurrence order-sensitive slice (`N ≥ 200` rows)" has no deterministic definition (selection rule), row count or SHA-256. Pin the construction and the hash before a run; the inherited `14,287`-row DEV panel is pinned through the DEV tokens hash once finding 2 is settled.

Everything else checked and holds: all 8 blocks are present (`prep-check` `complete=yes`); `cargo test -p uor-r4-discrete --lib position_mechanism` = **5 passed**; the path has no float, no dense matmul and no squashing form; the step table exports as `16 × B` `u8` words; the A8 parent sheet is `TEST FITNESS: FIT`; the proxy trunk is named; seeds, rows, cost and stop rules are present.

Re-request FIT only when 1–3 are pinned in this sheet; this verdict clears no compute (D22).

### Post-registration update (deepseek-39, 2026-10-23)

The two owner/PM prerequisites from this review now stand as follows.

1. **Bar — REGISTERED.** The owner's standing-direction decision at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`, card `6337522273`) registers the §3 bar exactly as proposed: DEV 14,287 rows, ≥ 2 seeds, `H_pos(ordered)` vs `C_match(ordered)` McNemar `b − c ≥ max(24, ⌈2√(b+c)⌉)` in both seeds with `Δadmitted ≥ 0`, and the order-shuffle drop ≥ `F_I`. The sheet now quotes it with the ISO-UTC time.
2. **DEV pin — PINNED.** §5 now carries the full pair from the authoritative code pin (`0919e90f953e…`, `a838bb1a67e9…`, matching 42,856/14,287 and the archived `a2-pilot`). The staging-sheet pair is recorded as an **errata via the PM** (#2832 is `done`); it is not used here.
3. **Order-sensitive slice — RULE PINNED, HASH `PENDING`.** §5 states the deterministic selection rule and `N ≥ 200`; the frozen subset's SHA-256 stays `PENDING` because the frozen file is data a lab/PM must file.

`TEST FITNESS: NOT_FIT` still stands: the slice hash is not yet pinned and the FIT must come from a non-author lab (per the registration, Claude or Kimi — deepseek has authored text in this sheet). This update clears no compute.
### Recorded review

```text
TEST FITNESS: NOT_FIT - M1 #2029 PositionHolonomyV1
sheet: docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md
reviewer: deepseek (non-author; fresh-context FIT review for #2843)
note: Configuration not yet pinnable: (1) the sheet's 'owner bar confirmation' link is antigravity's decision card 6337522273, not an owner answer - the owner bar with ISO-UTC time is absent (PM routing lists the bar cards as waiting on the owner), so no compute is cleared; (2) the DEV pin is ambiguous - the sheet carries only prefixes, while the repo holds two conflicting full DEV pairs: the authoritative code pin in attention_learning.rs (tokens 0919e90f953e..., mask a838bb1a67e9..., matching expected 42,856/14,287 and archived a2-pilot) versus the staging sheet's 0919e90f813b.../a838bb1ae20c... which appears nowhere else; pin the full code-pin pair and file an errata for the staging sheet; (3) the new multi-occurrence order-sensitive slice (N>=200 rows) has no deterministic definition or SHA-256 and must be pinned. Everything else holds: 8/8 blocks, 5 finite CPU controls pass, no float/matmul/softmax on the path, D11 export as 16xB u8 words, parent A8 sheet is FIT-registered, proxy trunk named, seeds/rows/cost/stop rules present.
date: 2026-10-22
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
