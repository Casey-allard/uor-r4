# `m1-address-key-full-vector` — Full-Vector / Interior-Block Address Metric and Finite CPU Controls

- **Roadmap item:** [`docs/roadmap/items/m1-address-key-full-vector.md`](../../roadmap/items/m1-address-key-full-vector.md) (`#2834`, Milestone `#2029`)
- **Pre-registration sheet:** [`docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md`](../../harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md) (`DRAFT`, `TEST FITNESS: PENDING`)
- **Host implementation & finite CPU controls:** [`crates/uor-r4-discrete/src/address_cells.rs`](../../../crates/uor-r4-discrete/src/address_cells.rs) and [`crates/uor-r4-discrete/src/bin/address-cells-codebook.rs`](../../../crates/uor-r4-discrete/src/bin/address-cells-codebook.rs) (`11 / 11` unit tests pass)

---

## 1. Geometric Diagnosis of the 5-Value Outer-Block Address Key

1. **Exact shell decomposition of the 2-block outer metric (`cell_outer_l1` / `discrete.cu:274–276`):**
   - In [`crates/uor-r4-discrete/build.rs`](../../../crates/uor-r4-discrete/build.rs) (`lines 232–241`), the 16 orbit roots `geometry::ROOTS[0..16]` are the SpiralCore signed-axis orbit of `[2, 2, 0, 0, 0, 0, 0, 0]`.
   - For every root $r \in \text{ROOTS}[0..16]$, the pairwise $L_1$ distance `root_l1(r, r')` across the 16 orbit roots takes only 3 values $\{0, 4, 8\}$ with exact multiplicity **$(1, 2, 13)$** ($1$ root at distance $0$, $2$ roots at distance $4$, and $13$ roots at distance $8$).
   - Consequently, when `address_distance(q, g, s, cell)` in [`crates/uor-r4-discrete/src/discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu) (`lines 274–276`) sums `root_cost` over only `block 0` ($\lfloor c/16 \rfloor$) and `block B-1` ($c \bmod 16$), the 256 writer cells partition into **5 exact distance shells** for every snapped query vector $q$:
     - $d_{\text{outer}} = 0$: $1 \times 1 = \mathbf{1}$ cell (cumulative: $1$)
     - $d_{\text{outer}} = 4$: $1 \times 2 + 2 \times 1 = \mathbf{4}$ cells (cumulative: $5$)
     - $d_{\text{outer}} = 8$: $1 \times 13 + 2 \times 2 + 13 \times 1 = \mathbf{30}$ cells (cumulative: $35$)
     - $d_{\text{outer}} = 12$: $2 \times 13 + 13 \times 2 = \mathbf{52}$ cells (cumulative: $87$)
     - $d_{\text{outer}} = 16$: $13 \times 13 = \mathbf{169}$ cells (cumulative: $256$)
   - Because $1 + 4 = 5 < 8$, **both the $K = 8$ admission boundary (slots $6..8$) and the $K = 16$ admission boundary (slots $6..16$) lie inside the 30-cell $d_{\text{outer}} = 8$ tie shell**. Under `cell_precedes` (`discrete.cu:301–303`), those 30 tied cells are ordered purely by raw cell index `c = first_root * 16 + last_root`.

2. **Why `StructuredV1` (`address_cells.rs`) collapses interior cell-envelope geometry:**
   - In `AddressCellsMode::StructuredV1`, outer blocks `(0, B-1)` are placed by greedy predecessor pull (`quota = V / 256`), while interior blocks `1..B-1` are set to `ROOTS[k]` for within-cell rank $k \in 0..\text{quota}$.
   - Because every cell $c \in 0..255$ receives the identical set of diagonal interior root pairs $\{(k, k) : k \in 0..\text{quota}\}$, the minimum interior $L_1$ distance $\min_{t \in c} \sum_{b=1}^{B-2} \|q_b - \text{table}[t, b]\|_1$ is constant across all 256 cells, leaving even a full-vector cell-envelope minimum at **5 distinct levels** on `StructuredV1`.

---

## 2. Full-Vector / Interior-Block Placement and Metric (`No Softmax`, `No Matmul`, `Integer Only`)

### 2.1 `AddressCellsMode::FullVectorV1` (and `FullVectorShuffledV1 { seed }`)
- Keeps the exact outer-cell greedy placement (`cell_of[t]`, `quota = V / 256`, `occupied_cells = 256`) of `StructuredV1` so `placed.cells` is unchanged.
- For $B \ge 4$, places each token's interior root pair `ic = (r_1, r_2) \in 0..256` (`block 1 = ROOTS[ic / 16]`, `block 2 = ROOTS[ic % 16]`) in the same descending target-frequency pass:
  - Enforces **within-cell uniqueness** (`!used_interior[cell][ic]`, guaranteeing `distinct_vectors == V` and zero within-cell alias tie mass) and **global interior quota balance** (`interior_load[ic] < quota` when available, preventing tree-root collapse onto a single interior pair).
  - Minimizes predecessor-weighted interior $L_1$ distance $\sum_{(s, n)} n \cdot \text{cell\_l1}(\text{interior\_of}[s], \text{ic})$ over already-placed predecessors $s$, with tie-break and zero-mass fallback to the deterministic Latin-square bijection `latin_interior_anchor(cell) = ((r0 ^ r3) * 16) + (r0 ^ ((3 * r3 + 5) & 15))`.

### 2.2 Full-Vector Cell Address Metrics (`cell_envelope_l1` and `full_vector_cell_l1`)
Both metrics use only integer subtraction, absolute value, bit-shift, addition, and minimum across the $B$ blocks (`8B` integers) of query $q$ and `table` (zero float, zero softmax/squashing, zero matmul):
1. **Unweighted cell-envelope full-vector distance (`cell_envelope_l1`):**
   $$d_{\text{env}}(q, c) = \min_{t \,:\, \text{cell}(t) = c} \sum_{b=0}^{B-1} \|q_b - \text{table}[t, b]\|_1$$
2. **Two-scale full-vector cell distance (`full_vector_cell_l1`):**
   $$d_{\text{full}}(q, c) = \bigl(d_{\text{env}}(q, c) \ll 8\bigr) + \sum_{t \,:\, \text{cell}(t) = c} \sum_{b=0}^{B-1} \|q_b - \text{table}[t, b]\|_1$$
   Because every snapped root $L_1$ distance is a multiple of $4$ ($4 \times 256 = 1,024 > 16 \times 32 = 512$), $d_{\text{env}}(q, c)$ is strictly lexicographically primary while the within-cell full-vector distance sum breaks ties geometrically across all $B$ blocks (invariant to token-ID ordering inside the cell) before `cell_precedes` falls back to `cell < other_cell`.

---

## 3. Finite CPU Controls on `address_cells.rs` (`11 / 11` Passed)

Verified via `~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells`:

| Control / Metric on Fixed Chain Fixture (`V = 512, B = 4, C = 8, K = 8`) | 5-Value Outer Key (`cell_outer_l1` on `StructuredV1`) | Unweighted Envelope (`cell_envelope_l1` on `FullVectorV1`) | Two-Scale Full-Vector (`full_vector_cell_l1` on `FullVectorV1`) |
|---|---:|---:|---:|
| `distinct_vectors` (`coverage_census::census`) | `512 / 512` | `512 / 512` | `512 / 512` |
| `occupied_cells` (`quota = 2` per cell) | `256 / 256` | `256 / 256` | `256 / 256` |
| `dev_alias_ceiling_rows` (`G-Cap` verdict) | `512 / 512` (`ABOVE_BAR`) | `512 / 512` (`ABOVE_BAR`) | `512 / 512` (`ABOVE_BAR`) |
| `within_cell_tie_mass` (`tokens_in_alias_groups` / non-rep rows) | `0 / 0` | `0 / 0` | `0 / 0` |
| Distinct cell distance levels on query `q0` | `5` (`{0,4,8,12,16}`) | `7` | `15` |
| $K = 8$ boundary tie-bucket size on query `q0` | `30` cells (`d = 8` shell) | `12` cells | `6` cells (`5x` reduction) |
| Predecessor-query (`q = table[t/2]`) target cell admitted at $K = 8$ (`511` edges) | `186 / 511` (`36.4%`) | `349 / 511` (`68.3%`) | `446 / 511` (`87.3%`) |

- Note on D24 / CHARTER §3.5: All numbers in the table above are finite synthetic-fixture CPU controls in `address_cells::tests`. Real corpus/model admission is never estimated on the host and will be measured exclusively on the GPU under [`docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md`](../../harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md) after non-author `TEST FITNESS: FIT` and owner/PM release.

---

## GPU pre-flight result (claude-110, 25 Oct 2026): PASS

- **Sheet:** [`m1-address-key-full-vector-2026-10-25.md`](../../harness/prep/m1-address-key-full-vector-2026-10-25.md), which supersedes the draft above. TEST FITNESS: FIT, and D67 pre-run review PASS (#2834). D67 replaced the owner/PM release.
- **Run:** one RTX 5090 (`vast-57698566`), source `77123dd995be44d5b0a5f482c8b82e270dd9aa42`, job [`address-key-job.sh`](address-key-job.sh), configs in [`configs/`](configs/).
- **Outputs:** HF store `results/claude/address-key-2026-10-25/run/` (30 files, SHA256SUMS verified). Two earlier runs stopped as instrument faults and are kept as `fault1/` and `fault2/`. The grade is [`grade.json`](grade.json), written by `address-key-grade`.

| Arm | P_succ admitted (of 3,847) | Full-DEV admitted (of 14,287) | Top-1 | Top-8 |
|---|---:|---:|---:|---:|
| A8-Outer (outer key) | 1,351 | 4,244 | 133 | 513 |
| FV8-Env (cell envelope) | 1,980 | 5,709 | 118 | 1,126 |
| FV8-TwoScale | 1,802 | 5,666 | 118 | 1,179 |
| FVS8-Env (shuffled control) | 645 | 1,998 | 34 | 856 |
| FVS8-TwoScale (shuffled control) | 663 | 2,028 | 34 | 865 |
| FV16-TwoScale (bracket) | — | 7,207 | 118 | 1,185 |

Paired on P_succ, the rule is that b − c must exceed max(24, 2√(b+c)):

| Comparison | b / c |
|---|---:|
| FV8-Env against A8 | 730 / 101 |
| FV8-Env against FVS8-Env | 1,726 / 391 |
| FV8-TwoScale against A8 | 562 / 111 |
| FV8-TwoScale against FVS8-TwoScale | 1,555 / 416 |

- **Instrument gates:** G-Q passed. A8-Outer reproduced 4,244 / 1,351 / 133 exactly. Both tables show 4,096 distinct vectors, 256 occupied cells, zero tie mass, and an alias ceiling of 14,287.
- **What it means:** a full-vector metric on a predecessor-pulled full-vector placement admits the target's cell for 1,980 of 3,847 P_succ rows at checkpoint 0, against A8's 1,351. The shuffled arms keep the metric and change only the placement, and they admit about half of A8. So the predecessor-pulled placement matters under a full-vector metric, beyond packing balance. No arm reads the full-vector table with the outer key, so the metric's own share of the gain over A8 is not separated here.
- **What it does not establish:** top-1 falls at checkpoint 0, from 133 to 118. Training and serving under a full-vector metric are not registered: update capabilities and the D11 exporter refuse it.
