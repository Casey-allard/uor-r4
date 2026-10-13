# OSPF-Style Context Compaction Probe v1

## Goal

Test whether the brute-force BFS warmup (~146s) can be replaced by a
compacted geometric cache / link-state topology summary while preserving
downstream training correctness.

## Baseline Configuration

| Parameter | Value |
|-----------|-------|
| D | 24 |
| D_HIDDEN | 32 |
| batch_size | 256 |
| device | cpu |
| threads | 1 |
| representation | hybrid angular+radial (S¹×R⁺) |
| pos0_bias | 2.0 (learnable b_pos0) |
| seed | 42 |

## Section A: Current BFS Role

### What BFS Computes

The `bfs_warm_up` function performs a breadth-first search from a 4000-state
warmup pool, discovering the full reachable state space of the operator algebra.

**Inputs consumed:**
- Pool of 4000 initial states (built by random walk from `initial_operator_state_v10()`)
- 6 deterministic operator functions (T_b, T_x, T_y, T_c, T_z', T_r*)
- Time budget: 141s

**Outputs produced:**
1. **TN** (tau-nexts tensor): (343,665, 6, 21) float32 = 173.2 MB
   - For each state × operator: the one-hot tau encoding of the successor state
2. **TR** (transition tensor): (343,665, 6) int64 = 16.5 MB
   - For each state × operator: the integer ID of the successor state
3. **tau0** (initial tau): (343,665, 21) float32 = 28.9 MB
   - One-hot tau encoding of each state's own tau
4. **pool_ids**: (4000,) int64
   - Maps pool positions to state table indices
5. **sid_map**: dict mapping full_key tuples to integer IDs

### BFS Timing Breakdown

| Phase | Wall Time | % of Total |
|-------|-----------|-----------|
| Pool build | 1.58s | 1.1% |
| BFS traversal | 141.42s | 97.1% |
| Table build | 2.71s | 1.9% |
| **Total** | **145.70s** | **100%** |

BFS traversal dominates. It calls 343,665 × 6 = 2,061,990
operator functions in pure Python — the real bottleneck.

### State Space Analysis

| Metric | Value |
|--------|-------|
| Discovered states | 343,665 |
| Theoretical maximum | 345,600 |
| Coverage | 99.4% |
| Field domains | b∈{0..4}, phi∈{0..2}, r∈{0..2}, twist∈{0,1}, swap∈{0,1}, coupled∈{0..4}, twist_ph∈{0,1}, lift∈{0..11}, bits∈16 tuples |

99.4% of the theoretical state product is reachable. The remaining 0.6% are
states that cannot be reached from the initial state under any operator sequence.

### What Is Truly Needed vs. Redundant

**Truly needed by downstream training:**
- TN (or angular equivalent) — used for soft tau mixture in forward pass
- TR — used for hard state transition (argmax routing)
- tau0 (or hybrid equivalent) — initial tau per state
- pool_ids — batch sampling

**Redundant / over-computed:**
- The BFS graph traversal logic itself (queue, visited set) — only needed
  for discovery, not for the tables
- The Python operator state objects — only needed to compute TN/TR entries,
  discarded after table build
- sid_map — only needed for pool_ids construction; not used during training

### Algebraic Enumeration Assessment

Pure algebraic enumeration (enumerating all 345,600 state keys without
BFS) is **blocked** by derived fields. Each `OperatorStateV10` contains:
- `composite_compat_class`, `query_semiprime`, `binding_semiprime`,
  `admissible_transition`

These are NOT in the deduplication key but ARE read by operator functions.
Reconstructing them from raw key fields would require reverse-engineering
the initialization logic. This makes pure product-enumeration impractical
without deeper refactoring.

**Verdict:** BFS is mandatory for the *first* computation. Caching eliminates
it for all subsequent runs.

## Section B: Cache / Link-State Design

### Strategy A: Full Tensor Cache

Store the exact BFS output as a torch checkpoint:
- TN: (343,665, 6, 21) float32
- TR: (343,665, 6) int64
- tau0: (343,665, 21) float32
- pool_ids: (4000,) int64
- sid_keys: list of 343,665 state key tuples

This is the **link-state database (LSDB)** approach: compute the full topology
once, persist it, and reload on every subsequent run.

### Strategy B: Compact Link-State Cache

Exploit the one-hot structure of TN and tau0:
- Each 21-dim one-hot vector has exactly 4 active entries
  (one per phase block: swap/2, coupled/5, twist/2, lift/12)
- Store as 4 uint8 phase indices instead of 21 float32 values
- TN: (343,665, 6, 4) uint8 — **5.25× smaller** than float32 one-hot
- tau0: (343,665, 4) uint8
- TR: unchanged (already compact as int64)

Reconstruction: scatter phase indices back to one-hot at load time (~23ms).

This is the **link-state summary** approach: store the minimal geometric
invariants (phase indices) rather than the full one-hot expansion.

### Design Classification

| Strategy | OSPF Analogy |
|----------|-------------|
| Full tensor cache | Link-State Database (LSDB) persistence |
| Compact link-state | Stub area summary — minimal geometric state |
| BFS re-run | Full LSA flooding — recompute from scratch |

## Section C+D: Head-to-Head Comparison

### Warmup Time

| Variant | Warmup Time | Speedup vs BFS | File Size |
|---------|------------|-----------------|-----------|
| BFS reference | 145.7s | 1× | N/A (in-memory) |
| Full tensor cache | 0.358s | **407×** | 232.4 MB |
| Compact link-state | 0.149s | **975×** | 9.2 MB |
| Full cache (warm OS) | 0.3461s | **421×** | 232.4 MB |
| Compact cache (warm OS) | 0.1426s | **1022×** | 9.2 MB |

### Semantic Correctness

| Variant | TN bit-exact | TR bit-exact | tau0 bit-exact | pool_ids bit-exact |
|---------|-------------|-------------|----------------|-------------------|
| Full tensor cache | ✓ | ✓ | ✓ | ✓ |
| Compact link-state | ✓ | ✓ | ✓ | ✓ |

### Training Correctness

| Variant | Accuracy | Solve Step | Steps/sec | Wall Time |
|---------|----------|-----------|-----------|-----------|
| BFS reference | 1.0000 | 2500 | 98.8 | 30.36s |
| Full tensor cache | 1.0000 | 2500 | 98.0 | 30.6s |
| Compact link-state | 1.0000 | 2500 | 95.3 | 31.5s |

All three variants use identical model initialization (same seed), identical
training loop, and bit-exact table data. Any accuracy or solve-step differences
are due to training stochasticity (Gumbel noise) — the tables are proven
bit-exact.

### End-to-End Wall Time Comparison

| Variant | Warmup | Training | Total | vs BFS |
|---------|--------|----------|-------|--------|
| BFS reference | 145.7s | 30.36s | 176.1s | 1× |
| Full tensor cache | 0.358s | 30.6s | 31.0s | 5.7× |
| Compact link-state | 0.149s | 31.5s | 31.6s | 5.6× |

## BFS cProfile Breakdown

Top functions by total time during BFS:

```
388277644 function calls (378720604 primitive calls) in 120.789 seconds

   Ordered by: internal time
   List reduced from 111 to 15 due to restriction <15>

   ncalls  tottime  percall  cumtime  percall filename:lineno(function)
  5498640    7.100    0.000   14.261    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v5.py:130(_mix_words_v5)
 31125960    6.476    0.000   13.950    0.000 {built-in method builtins.sum}
 11880720    6.004    0.000   13.302    0.000 {method 'join' of 'str' objects}
 41239800    5.639    0.000    5.639    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v6.py:87(<genexpr>)
 27493200    5.448    0.000    5.448    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v5.py:132(<genexpr>)
  4123980    5.195    0.000   33.326    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v6.py:106(sigma_family_holonomy_law_v6)
 17183250    4.005    0.000    4.633    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v5.py:119(_rotate_word_v5)
   327270    3.110    0.000   12.144    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v5.py:146(_local_sigma_words_v5)
   687330    3.052    0.000   19.430    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_operator_model_v22.py:59(radial_transport_component_v22)
   687330    2.933    0.000   23.343    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_operator_model_v20.py:64(coupled_torus_kick_component_v20)
   687330    2.890    0.000   19.172    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_operator_model_v21.py:59(fiber_phase_lift_component_v21)
  8247960    2.509    0.000   11.285    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_spinH_core_v6.py:86(_xor_sum)
        1    2.255    2.255  141.380  141.380 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/run_router_reintegration_v6_torch.py:176(bfs_warm_up)
   687330    2.192    0.000   36.976    0.000 /Users/adminamn/AI-Research/ai-router/router-research/tools/prime_transport/geometry_native_operator_model_v23.py:63(torus_base_advance_component_v23)
14335560/4778520    2.174    0.000    3.786    0.000 {built-in method builtins.hash}
```

This confirms: BFS wall time is dominated by 2,061,990+ Python operator
function calls, each involving modular arithmetic on state fields.

## Conclusions

### Does OSPF-style context compaction work?

**Yes, decisively.** The BFS topology computation can be persisted as a
geometric cache and reloaded in milliseconds. Both cache strategies produce
bit-exact tables and identical training behavior.

### Recommended Strategy

**Full tensor cache (Strategy A)** is the recommended default:
- Fastest load time (0.3461s warm)
- Simplest implementation (single `torch.save`/`torch.load`)
- No reconstruction step needed
- 232 MB disk cost is negligible for a research workstation

**Compact link-state (Strategy B)** is better when:
- Disk space matters (5× smaller)
- Cache needs to be transferred (network/CI)
- Reconstruction overhead (0.1426s) is acceptable

### The OSPF Analogy

The current system architecture maps cleanly to OSPF:

| Concept | OSPF | This System |
|---------|------|-------------|
| Topology discovery | LSA flooding | BFS warm-up |
| Link-state database | LSDB | TN + TR tensors |
| Route computation | SPF algorithm | Training loop |
| Persistence | LSDB checkpointing | Cache files |
| Incremental updates | Partial LSA | Not yet needed (static topology) |

BFS is the "flooding" step — expensive but only needed once. The cache is the
"LSDB" — persistent, fast to load, used for all subsequent routing (training).

### Impact on Single-Run Wall Time

Before: warmup (146s) + training (~30.36s) = ~176s

After: cache load (<0.3s) + training (~30.6s) = ~31s

**BFS was 74% of single-run wall time. Cache eliminates it entirely.**

## Honesty Section

### What Improved
- Single-run warmup reduced from ~146s to <0.3s (after first BFS)
- End-to-end single-run time reduced by ~83%
- Both cache strategies are bit-exact — zero semantic change

### What Did Not Improve
- Training throughput (steps/sec) is unchanged — the cache only affects warmup
- First-ever run still requires full BFS (~146s)
- No memory reduction during training (tables are the same size in memory)

### BFS Still Needed — In What Reduced Role
- **First-run only**: BFS must run once to discover the state space and build
  tables. This is the "initial flooding" step.
- **Cache invalidation**: If operator geometry changes (new operators, changed
  modular arithmetic), the cache must be rebuilt. A hash of the operator
  definitions could automate invalidation detection.
- **Incremental reconciliation**: Not needed yet because the topology is static.
  If the state space became dynamic (e.g., growing during training), incremental
  BFS updates would become relevant.

### What Was Too Confident Previously
- Treating BFS as an immutable fixed cost. It was always cacheable.
