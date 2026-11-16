# ORDER: one-model-end-to-end, claude-408 (the Pick read: an exact XOR + popcount hard read in the discrete model, the same in CUDA, the CPU oracle and serving)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Base: claude-407's branch `claude/row-cap-407` (PR #4044, queued). It moves the same driver-source pin, so this branch starts from that head and is merged up to `main` once #4044 merges.

## Why this unit

Casey, 16 Nov 2026, in claude's chat, after the gap analysis: "Yes, that base". The discrete model's integer state is the one D85 base (PLAN.md, owner directions, 16 Nov). The first gap in that order is the state-keyed hard read by XOR + popcount, as one operator shared by CUDA training, the CPU oracle and serving.

Today the discrete model reads by Route, Flock or Relation (`discrete.cu:353`). None of them copies an earlier match: Flock keeps K candidates by frame ranking and snaps their sum. The float line's evidence says the hard copy is the step that works: copying only the best match recalled the asked fact 96.5–98.4% of the time, against 33–37% for the averaged read (claude-388), and its integer training found the pick 98–99.6% of the time held out (claude-395).

This unit is a finite kernel and oracle unit: forward and counterfactual scoring on fixtures and one real batch, no training run. Under §4.5 it needs no sheet.

## The exact identity it rests on

The 16 anchor-orbit roots are all `2·(±e_i ± e_j)` (`ROOTS` in the generated `geometry.rs`). Every coordinate is −2, 0 or +2. Code each coordinate in two bits (positive, negative), so a block is one 16-bit word. For two roots `a` and `b`, `L1(a − b) = 2 · popcount(code(a) XOR code(b))`. The read's distance is therefore exactly the model's own L1 root distance, computed by XOR and popcount, with no approximation. Blocks stay in the orbit: table blocks are roots (`d11_export` checks it), and the position actions are signed permutations that map the orbit to itself (`position_mechanism.rs:305` refuses any that leaves it).

## Native-space items (D85)

1. **State schema.** One new read mode in the existing snapshot: `ReadSpec::Pick`, spec word `s[8] = 3`. It reuses the existing gate trits (`s[1]` per stage-0 level, Gate kind) and `s[14]` position transport. No new learned array.
2. **Shared operators.** One definition of the Pick read, written three times behind one contract: `discrete.cu` (CUDA forward and counterfactual), the CPU oracle in `uor-r4-cpu-train` (`discrete_route.rs`), and the serving reader `uor-r4-integer` `discrete_d11.rs`. Parity tests hold them word for word.
3. **Replay oracle.** The CPU oracle's forward and loss words. The CUDA and CPU words must agree on fixtures and on one real 64-row student batch (claude-404's stream), with one planted fault caught.
4. **Forward, gradient, update.** Forward and Gate-kind counterfactual scoring. The existing choose/apply path is not changed; its Gate proposals reach the Pick gates as they reach Flock's.
5. **Serialization.** The D11 artifact carries the read mode. `discrete_d11` loads a Pick artifact and serves the same picks and tokens as the CUDA forward.

## Design: the Pick read for one row

Let `t = length − 1` be the query position, `tok(p)` the token at position `p`, and `B = s[1]` blocks.

- **Codes.** `code(p, b)` is the 16-bit code of block `b` of `tok(p)` after the position transport `s[14]` from `p` to `t` (the same `position_action` Flock uses). `s[14] = 3` (identity) and `s[14] = 1` (causal prefix holonomy) are admitted for Pick; 0 and 2 are refused.
- **Key.** The pair ending at `p`: `d(p) = Σ_b popcount(code(p−1, b) XOR code(t−1, b)) + popcount(code(p, b) XOR code(t, b))`, for `1 ≤ p ≤ t − 1`. The pair is the first conjunction key, as the binding finding asks (two tokens, not one).
- **Pick.** The least `d(p)`; a tie goes to the latest `p`. With `t < 2` there is no eligible `p`, and the read returns the query blocks unchanged.
- **Payload.** The blocks of `tok(p + 1)`, the successor of the pick, transported to `t`.
- **Output.** Per block, `u = query_b + gate_b · payload_b` with `gate_b ∈ {−1, 0, +1}`, then `snap_n(u)` with `n = 1 + |gate_b|`, as Flock snaps its residual. Gate 0 leaves the block as the query.
- **Trace** (forward only, same path): per row, the pick `p` (or ABSENT), `d(p)`, the count of positions tied at `d(p)`, and per block the gate and the output root.

The Pick read is integer only: gather, XOR, popcount, add, compare and the existing action tables. No float, no squashing, no product of learned values.

## Tests

1. **Identity test.** For all 16 × 16 root pairs, `L1 = 2 · popcount(XOR)` of their codes, in the CPU oracle and on CUDA.
2. **Planted copy test** (CPU oracle and CUDA, fixtures). Sequences built so that the query pair occurs exactly once earlier with a unique pair code, and the target is that occurrence's successor. With gates +1, the pick is the planted position on every row, and the output's writer pick is the successor token on every row where its blocks differ from the query's. With every gate 0 the output equals the read-off forward word for word.
3. **Parity.** CUDA against the CPU oracle, word for word, on the fixtures and on one 64-row student batch at the full model size (claude-404's harness), forward words and Gate-kind loss words; one planted fault (a shifted pick) is caught.
4. **Serving parity.** A Pick snapshot exported through `d11_export` and loaded by `discrete_d11` gives the same picks and the same greedy tokens as the CUDA forward on the fixtures.
5. **Refusals.** Pick with `s[14] ∈ {0, 2}`, a Route proposal in Pick mode, and an out-of-orbit block are refused, each by a test.

## Files in scope

- `crates/uor-r4-discrete/src/discrete.cu`: `MODE_PICK`, `pick_vector`, the dispatch in `read_vector`;
- `crates/uor-r4-discrete/src/lib.rs`: `ReadSpec::Pick` and its validation and serde;
- `crates/uor-r4-discrete/src/device.rs`: the spec words for Pick;
- `crates/uor-r4-discrete/src/d11_export.rs`: the read mode in the artifact;
- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs` and one new test file `crates/uor-r4-discrete/src/pick_read.rs`;
- `crates/uor-r4-cpu-train/src/discrete_route.rs`: the oracle's Pick forward and Gate scoring;
- `crates/uor-r4-integer/src/discrete_d11.rs`: `DiscreteReadMode::Pick` in the serving reader;
- `crates/uor-r4-discrete/src/driver_source_pin.rs`, the `prime_zeta` copies, and any other pin the full suite names;
- `docs/loop/tasks/one-model-end-to-end/claude-408/**`;
- one row in `docs/research/notebook/m1-discrete-attention.md`.

## Stop list

- No training run, no pod; CUDA tests on wick's 4090 only.
- Route, Flock and Relation are byte-identical in behaviour; their tests are unchanged.
- No margin threshold, no learned key gates and no conjunction table: they are later units (the conjunction memory is the next gap in the order).
- The corpus trainer and the 1B run wait (PLAN.md, 16 Nov).

## Done when

1. The full `uor-r4-discrete`, `uor-r4-cpu-train` and `uor-r4-integer` test suites pass on the laptop, and the CUDA suite builds.
2. Tests 1–5 pass, with the CUDA ones run on wick's 4090.
3. The REPORT gives the planted copy result, the parity counts and the serving parity, with what they do not show (no learning claim).
4. `loop pr-check --local` passes.
