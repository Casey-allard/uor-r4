# Lawful Integer Pointer and Forced-Copy Span Oracle (`r-lawful-pointer-forced-copy-oracle`)

- **Roadmap item:** [`docs/roadmap/items/r-lawful-pointer-forced-copy-oracle.md`](../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md) (Issue `#2880`)
- **Mechanism:** [`docs/mechanisms/pointer-key.md`](../../mechanisms/pointer-key.md) (`pointer-key`, status: `PARKED`)
- **Implementation:** [`crates/uor-r4-training/src/lawful_pointer.rs`](../../../crates/uor-r4-training/src/lawful_pointer.rs)
- **Pre-registration sheet:** [`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`](../../harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING`)

---

## 1. Design Note: Geometric Construction of the Lawful Pointer

### 1.1 Why the historical pointer was retired

Every historical pointer in `crates/uor-r4-training/src/geometric_stack.rs` blended a dense vocabulary distribution with a copy distribution via two normalized-exponential operations and a dense inner-product score:

$$p(v) = (1 - g)\,\mathrm{softmax}(z)_v + g \sum_{j:\,x_j = v} a_j$$

That formulation violates three standing rules:
1. **No normalized exponential or squashing (CHARTER §3.2, D23):** Both the vocabulary mixture and the source attention $a_j$ used normalized exponentials.
2. **No dense matrix multiplication or inner-product contraction (CHARTER §3.3, D25, D30):** The pointer query/key score sat inside a dense floating-point stack.
3. **Integer-only serving (CHARTER §3.4, D11):** `stack_export` refused any pointer carrying an identity or prime-route term.

### 1.2 Lawful integer pointer construction (`crates/uor-r4-training/src/lawful_pointer.rs`)

The lawful pointer replaces both normalized-exponential terms and all dot-product/matmul operations with exact integer number-theoretic admission and integer rank/margin selection:

1. **Exact slot-address and prime-atom admission (`RouteAdmissionMode`):**
   - **Slot-address semiprime (`RouteAdmissionMode::SlotAddressSemiprime`, [`SlotAddressKey`](../../../crates/uor-r4-training/src/lawful_pointer.rs)):**
     Given distinct registered primes $p_{\text{entity}}, p_{\text{attribute}} \in \mathbb{P}$ (verified by deterministic integer primality `is_prime_u64`), the slot key is the square-free semiprime:
     $$K_{\text{slot}} = p_{\text{entity}} \cdot p_{\text{attribute}} \in \mathbb{Z}_{>1}$$
     By unique factorization in $\mathbb{Z}$, $K_{\text{slot}}$ uniquely identifies the unordered slot pair $\{p_{\text{entity}}, p_{\text{attribute}}\}$. A candidate earlier value span $c = ([s, e), \pi_c, b_{\text{rank}})$ with clause atom product $\pi_c \in \mathbb{Z}_{>0}$ is admitted iff:
     $$K_{\text{slot}} \mid \pi_c \iff \gcd(K_{\text{slot}}, \pi_c) = K_{\text{slot}}$$
   - **Shared prime-atom divisibility (`RouteAdmissionMode::SharedAtomGcd`):**
     Admits a candidate span $c$ iff $\gcd(q, \pi_c) > 1$. Instead of floating-point $\ln\gcd(q, \pi_c)$, rarity is weighted by the exact Q8 integer bit-rank:
     $$w_{\gcd}(d) = \begin{cases} 0 & \text{if } d \le 1 \\ \lfloor \log_2 d \rfloor \cdot 256 & \text{if } d > 1 \end{cases}$$
     computed via `127 - d.leading_zeros()` (`ilog2_gcd_weight_q8`), so rarer (larger) prime factors strictly dominate smaller prime factors in pure integer arithmetic.
   - **Ordered $n$-let equality (`RouteAdmissionMode::OrderedNlet { width }`):**
     Admits a candidate span $c$ iff the `width` tokens immediately preceding $s$ match the `width` tokens preceding the query under [`CanonicalFoldTable`](../../../crates/uor-r4-training/src/lawful_pointer.rs) (one table lookup per token folding ASCII case and at most one leading space `b' '` to the least token ID in the equivalence class). The fold table affects **only** match admission and never alters the emitted token ID $x_{s+\delta}$.

2. **Hard integer margin selection (`LawfulPointerHead::select_step`):**
   - Let $M_{\text{base}}[v] \in \mathbb{Z}$ be the base integer margin table over the vocabulary $v \in \{0, \dots, V-1\}$, and let
     $$v_{\text{base}} = \arg\max_{v \in \{0, \dots, V-1\}} M_{\text{base}}[v]$$
     with ties broken deterministically by lowest token ID.
   - Among admitted source candidates $c = ([s, e), \pi_c, b_{\text{rank}})$, the winning span $c^\star = ([s^\star, e^\star), \pi_{c^\star}, b_{\text{rank}})$ maximizes the integer route score
     $$S_{\text{route}}(c) = w_{\gcd}(\gcd(q, \pi_c)) + b_{\text{rank}}(c)$$
     with ties broken deterministically by latest span start $s$.
   - At span offset $\delta \in [0, e^\star - s^\star)$, the candidate copied token is $v_{\text{copy}} = x_{s^\star + \delta}$ with total integer copy margin
     $$M_{\text{copy}} = S_{\text{route}}(c^\star) + m_{\text{gate}}$$
     where $m_{\text{gate}} \in \mathbb{Z}$ is the head's integer gate margin (`effective_gate_margin`) and $\tau_{\text{gate}} \in \mathbb{Z}$ is the gate threshold margin.
   - Under all three oracle conditions (`N`, `O1`, `O2`), the head emits $v_{\text{copy}}$ iff $m_{\text{gate}} \ge \tau_{\text{gate}}$ and $M_{\text{copy}} > M_{\text{base}}[v_{\text{base}}]$; otherwise it emits $v_{\text{base}}$ bit-for-bit.

---

## 2. Settling the Source Disagreement on Stored Outputs vs Interventional Oracle

[`docs/roadmap/items/r-lawful-pointer-forced-copy-oracle.md`](../../roadmap/items/r-lawful-pointer-forced-copy-oracle.md) notes a disagreement between two sources:
- [`docs/mechanisms/pointer-key.md`](../../mechanisms/pointer-key.md) (`What would reopen it`, item 1) says: *"using a softmax-free, matmul-free pointer or stored outputs only."*
- [`docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md`](../repo-renormalization-2026-10-19/audit/synthesis.md) (line 54) states that the forced-copy oracle is **interventional**, so stored outputs cannot answer it.

**Resolution:** This design follows the audit synthesis. The forced-copy oracle (`N` / `O1` / `O2`) is an interventional test on the live copy selection step:
- Condition `N` (`CopySpanOracle::Off`) runs the unforced model and verifies bit-for-bit reproduction of its recorded memory score.
- Condition `O1` (`CopySpanOracle::O1SpanLearnedGate { start, end }`) overrides source admission at serving time with the true value span `[start, end)` located in the row's history (assigned its canonical `oracle_route_score`) while keeping the model's `learned_gate_margin` and the exact same `effective_gate_margin >= gate_threshold_margin && total_copy_margin > base_top_margin` comparison as `N`. When `O1` is supplied the same span that `N` already admitted, `O1` and `N` make the bit-for-bit identical decision on every row, so `O1 - N` isolates **source addressing alone**.
- Condition `O2` (`CopySpanOracle::O2SpanForcedGate { start, end, forced_gate_margin }`) overrides both source admission to `[start, end)` and replaces `learned_gate_margin` with `forced_gate_margin` (above threshold and base margin). This isolates **gating and margin threshold** from **multi-token value span emission**.

Static stored outputs from the retired 2026-10-10 dense float runs record only condition `N` (`23/40`) and contain no counterfactual `O1` or `O2` decode traces, while re-executing the retired dense float checkpoint is barred by CHARTER §3.2–§3.4 (D23, D30). Therefore, the oracle requires the lawful integer pointer with `CopySpanOracle`.

### Does the pointer need `SlotAddressKey` (`entity_prime * attribute_prime`) or only span copying?

- **For the forced-copy oracle (`O1` and `O2`):** `CopySpanOracle` supplies the true window span `[start, end)` located by `locate_last_value_span` (the last occurrence of the expected value across token boundaries, with ASCII alphanumeric boundary checks so `"4"` never matches inside `"47"` or `"14"`), so `O1` and `O2` test whether a perfect multi-token span address improves memory answers under learned gating (`O1`) and forced gating (`O2`).
- **For unforced lawful pointer operation (`N` and follow-on training):** `SlotAddressKey` ($p_{\text{entity}} \cdot p_{\text{attribute}}$) is required so that admission keys on the exact slot identity rather than the 6-token context frame (`pointer_identity`, hit rate `0.003`) or a single shared atom (`prime-ranked:1`, whose reachability ceiling was `0.6677`). Both `SlotAddressKey` and `CopySpanOracle` are implemented and verified in [`crates/uor-r4-training/src/lawful_pointer.rs`](../../../crates/uor-r4-training/src/lawful_pointer.rs).

---

## 3. Sheet Type Selection: D32 Discrete Integer Path

We select a **D32 discrete integer sheet** ([`docs/harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md`](../../harness/prep/r-lawful-pointer-forced-copy-oracle-DRAFT-2026-10-23.md)) rather than a D37 offline float sheet (`float-training-module`):
1. Every operation in `LawfulPointerHead` (`gcd_u128`, `ilog2_gcd_weight_q8`, `SlotAddressKey::divides`, `CanonicalFoldTable::fold`, and integer margin comparison) is already exact integer arithmetic (`u32`, `u64`, `u128`, `i64`) and directly D11-servable with zero export gap on the pointer route.
2. Choosing D32 avoids introducing a separately scanned offline float module (`float-training-module`) for a mechanism whose geometry (prime divisibility, canonical token-id fold table, and rank/margin selection) is natively discrete.

---

## 4. Finite Host Controls (`crates/uor-r4-training/src/lawful_pointer.rs`)

All nine finite host unit tests in `crates/uor-r4-training` pass on CPU (`cargo test -p uor-r4-training --lib lawful_pointer`):

| Test name | What it verifies | Status |
|---|---|---|
| `slot_address_semiprime_admits_exact_entity_attribute_pair_only` | `SlotAddressKey::from_entity_attribute_primes(5, 11)` forms semiprime `55`, rejects composite (`4`) or duplicate (`5, 5`) atoms, admits Ana's locker span `[3, 5)` (`" 4"`, `"7"`) over Bob's locker span `[9, 11)` (`5 * 11 = 55` vs `7 * 11 = 77`) even when Bob's candidate has a larger `learned_rank_bonus`, and emits both tokens of the two-token value span across `step_offset = 0, 1`. | `PASS` |
| `oracle_off_reproduces_base_scorer_bit_for_bit_when_gate_closed_or_unadmitted` | With `CopySpanOracle::Off`, verifies **both** the gate-closed case (`learned_gate_margin = 40 < 100` on an admitted candidate) **and** the unadmitted-candidate case (`learned_gate_margin = 500 >= 100` on a candidate whose slot product `55` is not divisible by query `77`), returning the base scorer's exact argmax token (`7`) and margin (`105`) bit-for-bit. | `PASS` |
| `base_margin_comparison_blocks_copy_when_base_outscores_copy_in_both_n_and_o1` | Pins `total_copy_margin > base_top_margin`: when `learned_gate_margin = 120 >= 100` and `route_score = 1354` (`total_copy_margin = 1474`), a `base_top_margin` of `1500` blocks the copy (`copied == false`, `emitted_token == 7`) in **both** `N` and `O1` (`o1_blocked == n_blocked`), whereas lowering `base_top_margin` to `1400` opens the copy in both `N` and `O1`. | `PASS` |
| `shared_atom_gcd_ranks_rarer_primes_higher_and_breaks_ties_by_latest_span_start` | Verifies `RouteAdmissionMode::SharedAtomGcd` ranks a candidate sharing rarer prime `13` (`route_score = 768`) above one sharing prime `3` (`route_score = 256`), and breaks equal-score ties in favor of the latest span start (`(9, 11)` over `(3, 5)`). | `PASS` |
| `copy_span_oracle_separates_o1_learned_gate_from_o2_forced_gate_and_restores_on_clear` | On fixed weights where `learned_gate_margin = 25 < 100`, `O1SpanLearnedGate` selects the supplied span `[9, 11)` (`candidate_copy_token = Some(10)`, `route_score = Some(1536)`) but does not emit it (`copied = false`, `emitted_token = 7`), whereas `O2SpanForcedGate` (`forced_gate_margin = 250 >= 100`) emits both tokens `10` (`" 9"`) and `11` (`"2"`); `clear_copy_span_oracle` restores `CopySpanOracle::Off` bit-for-bit. | `PASS` |
| `copy_span_oracle_refuses_empty_reversed_or_out_of_window_spans` | `set_copy_span_oracle` returns `Err(TrainingError::Invalid)` on empty (`4..4`), reversed (`5..3`), and out-of-window (`2..9` on window length `8`) spans without silent fallback. | `PASS` |
| `locate_last_value_span_finds_last_occurrence_across_token_boundaries` | `locate_last_value_span` locates the last occurrence `[15, 17)` of `"47"` across two tokens (`" 4"` and `"7"`), locates `"92"` at `[9, 11)`, and returns `None` for unstated values (`"38"`). | `PASS` |
| `locate_last_value_span_enforces_alphanumeric_boundaries` | Searching for single-digit `"4"` or `"7"` inside `"Ana locker is 47."` returns `None` (no substring match inside `"47"`), and when standalone `"4"` is followed by `"47"`, searching for `"4"` skips `"47"` and returns the standalone `"4"` span `[3, 4)`. | `PASS` |
| `canonical_fold_table_matches_case_and_leading_space_without_altering_emitted_token` | `CanonicalFoldTable` folds `"Ana"` (`1`) and `" ana"` (`2`) to `1`, and `"locker"` (`4`) and `" locker"` (`5`) to `4`, admitting an `OrderedNlet { width: 2 }` span while emitting the raw window token `10` untouched. | `PASS` |
