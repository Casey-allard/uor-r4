# Stage M1 `ReadSpec::Relation` Source Slice (`crates/uor-r4-discrete`) — 2026-10-15

**Lab:** Antigravity (`antigravity/ag1`)
**Milestone:** M1 — Exact Addressed Memory ([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029))
**Pre-Registration:** [`docs/harness/prep/m1-finite-relational-head-2026-10-14.md`](../../harness/prep/m1-finite-relational-head-2026-10-14.md) (merged in [#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494), `TEST FITNESS: FIT` by Claude at `4f0181b2c` in [#2029 comment 6184281299](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6184281299))
**Mechanism Brief:** [`docs/mechanisms/finite-relational-head.md`](../../mechanisms/finite-relational-head.md) (§2–§4, §7.1, merged in [#2486](https://github.com/UOR-Foundation/uor-r4/pull/2486))
**Authority:** [D32](../../integration/DECISIONS.md#d32--discrete-geometric-substrate-ternaryhammingquaternion-computation-no-floating-point-in-model-computation-including-learning), [D34](../../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate)

## 1. Deliverable Summary

This slice implements the Stage M1 `ReadSpec::Relation { kept, sweeps }` bounded exact-relation reranker inside `crates/uor-r4-discrete` (`lib.rs`, `device.rs`, `discrete.cu`, `training.rs`, `main.rs`, and `relation_qualification.rs`) matching the merged D34 mechanism card and the FIT-reviewed Stage M1 pre-registration sheet (`m1-finite-relational-head-2026-10-14.md`):

1. **Explicit Artifact Identity (`crates/uor-r4-discrete/src/lib.rs`, `device.rs`, `training.rs`):**
   - Adds `pub const RELATION_EDGE_TYPES: u32 = 5` and `ReadSpec::Relation { kept: u32, sweeps: u32 }` (`{"kind":"relation","kept":K,"sweeps":S}`).
   - `ReadSpec::validate` enforces `kept in 1..=8` (`Invalid("relation kept count must be in 1..=8")`), `sweeps in 1..=5` (`Invalid("relation sweep count must be in 1..=5")`), and `Spec::validate` (`blocks in 2..=16`).
   - `ReadSpec::words(self) -> [u32; 3]` encodes `[0, 0, 0]` (`Route`), `[1, kept, 0]` (`Flock`), and `[2, kept, sweeps]` (`Relation`) into `spec_gpu` words `s[8..=10]`.
   - `ReadSpec::gate_count(self, blocks, stages)` allocates `blocks * stages` gate trits for `Route`/`Flock` and `blocks * stages + 5 * stages` gate trits for `Relation` (`20 + 25 = 45` trits at `C192` where `blocks = 4, stages = 5`; `4 + 10 = 14` trits at `context = 6` where `blocks = 2, stages = 2`).
   - `Device::parameter` and `main.rs` (`propose`) refuse `Parameter::Route(_)` on `ReadSpec::Relation` snapshots with `Invalid("route parameters are inactive in a relation model")`.
   - `PreparedDriver` (`training.rs`) maps all `blocks * stages + 5 * stages` read coordinates in `ReadSpec::Relation` to `CoordinateKind::Gate` (`Parameter::Gate(index)`).

2. **Integer-Only CUDA Relational Reranker (`crates/uor-r4-discrete/src/discrete.cu`):**
   - Defines `#define MODE_RELATION 2u`, `#define EDGE_TYPES 5u`, `#define MAX_SWEEPS 5u`, and `__device__ U gate_count(const U* s) { return s[1]*s[5] + (s[8]==MODE_RELATION ? EDGE_TYPES*s[5] : 0u); }`.
   - In `flock_vector`, tracks `first[MAX_NODES]` alongside `level[MAX_NODES]` and `last[MAX_NODES]` (`first[node] = atom` for leaves, `first[node] = first[c0]` for interior nodes).
   - When `s[8] == MODE_RELATION`:
     - Validates `sweeps in 1..=5` and validates all `5 * s[5]` relation gate trits (`gates[s[1]*s[5] + l*5 + e] in -1..=1`).
     - Computes exact anchor-orbit base match scores $m_j^{(0)} = 8 - d_{\text{L1}}(f_q, f_j) \in \{0, 4, 8\}$ on the $D_4$-embedded 16-root orbit.
     - Executes $S = \text{sweeps} \in \{1..5\}$ synchronous integer sweeps across the 5 retained-structure edge families (`0: ParentChild`, `1: Sibling`, `2: SequentialSuccessor`, `3: SameToken`, `4: SameFrame`, all enforcing `last < length` on both endpoints, with `3: SameToken` and `4: SameFrame` selecting only the most-recent prior matching node `largest j < i` at the same level via `esum = prev[j]` so total degree stays $\le 8$) using sign-conditioned integer addition/subtraction (`gt > 0 ? esum : -esum`, zero multiply or float instructions).
     - Ranks causally eligible level-$l$ nodes (`level[node] == l && last[node] + 1 < length`) by the 5-key total order `(rel[node] desc, distance asc, hamming asc, last[node] desc, node asc)`. When all relation trits are `0` ($r_i^{(S)} = 0$), this reduces bit-for-bit to `ReadSpec::Flock`.
     - Leaves the downstream `BFS[frame[node]*16 + fq]` transport, `snap_K` consensus, level-combine gate residual, `snap_terms` output root, and `writer` (`discrete.cu:229-253`) 100% untouched.
     - Appends per-node relation scores (`rows * blocks * nodes_count` words) after the `Flock` trace region when `trace != 0` in `MODE_RELATION`, keeping `Forward::flock_trace()` bit-for-bit identical between `Flock` and `Relation` while exposing `Forward::relation_scores()`.

3. **Stage M1 Qualification Suite (`crates/uor-r4-discrete/src/relation_qualification.rs`, `main.rs`):**
   - Registers the 8 control functions (`discrete-stack list-relation-controls` / `discrete-stack qualify-relation NEW_REPORT_DIR`, writing `relation-controls.json` with `applied_updates: 0` and zero `.apply` calls):
     - `relation_off_flock_parity` (`C1`)
     - `relation_copy_successor_fixture` (`C2`)
     - `relation_irrelevant_change_invariance` (`C3`)
     - `relation_conflicting_successors` (`C4`)
     - `relation_suffix_and_target_isolation` (`C5`)
     - `relation_endpoint_causal_boundary` (`C6`)
     - `relation_deterministic_tie_order` (`C7`)
     - `relation_bounded_sweeps_and_refusals` (`C8`, including `8a` multi-sweep `SequentialSuccessor`, `8b` all-`+1` `sweeps = 1..=5` worst-case score vectors, `8b2` 3-occurrence `SameToken` `e=3` and `SameFrame` `e=4` most-recent-prior pins on distinct token IDs across `sweeps = 1..=3`, and `8c` typed refusals)

## 2. Verification & Governance Status

- **Host metadata & ratchet verification (`0` GPU hours):**
  - `cargo test -p uor-r4-discrete --no-default-features` passes (`lib.rs`, `metadata.rs`, `corpus.rs`, `training.rs`, `controller.rs`, including `apply_calls_are_confined_to_bounded_fixture_and_admitted_transition`).
  - `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness check-formats`, `uor-harness shard-check`, and `python3 scripts/check_claim_wording.py` pass.
- **Execution boundary:** Source-only construction PR (`0` GPU hours, `0` `.apply` updates, `0` corpus rows). Both training holds remain `HELD`. Finite CUDA execution of `qualify`, `qualify-flock`, and `qualify-relation` on an admitted GPU pod awaits explicit owner release on `#2029`.
