# REPORT: end-to-end-integration, codex-15

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Every k-means site ADR-0007 names is uncompiled.** `addressing_arms` and `joint_admission` are
  `#[cfg(any())]` in `crates/uor-r4-training/src/lib.rs` (lines 26-27 and 126-127). The
  transformerless compiler is in `transformerless/mod.rs`, which the lib target does not mount: the
  target is `src/native_support.rs`, whose `transformerless` is `native_mod.rs` (tokenizers only,
  and its header calls `compiler.rs` "inactive evidence"). The graph compiler's `induction.rs` has
  the same shape: `src/native_support.rs` and "No Cargo feature re-enables the historical surface."
  So "attach e-means at the four sites" cannot be done without reviving a parked line.
- **The only live codebook is not a k-means fit.** A8 address cells (`uor-r4-discrete`, ACTIVE)
  place tokens by a greedy, quota-constrained weighted-L1 rule. e-means on the same task is
  **2.165x worse** on the fixture's weighted cell-L1 (31,780 vs 14,676). The arm is opt-in and is
  **not promoted**; the default placement is unchanged.
- **D32 refused the first, float fitter.** The discrete crate is integer-only; the harness's
  `banned-presence` ratchet counted 66 float sites in the new module and failed the PR. The fitter
  is now Q16 fixed-point: counts and sums at `1 << 16`, a 33-entry Q16 cosine table with linear
  interpolation, and `i128` squared distances. No float token remains outside a comment, including
  in the tests.
- **The first EMA update skipped untouched clusters**, so their counts never decayed and the
  dead-cluster revival named in the module docs was inert (found by the exact-head review). Every
  cluster now updates — an untouched one decays — and revival scores every point once and gives the
  farthest distinct points to the dead clusters, so it is one pass plus a sort rather than one scan
  per dead cluster.
- **The fitter is parity at this scale, not a win.** 4 integer e-means epochs vs integer Lloyd 16
  passes: total squared error 6,533,628,331 vs 6,533,624,246 (1.000x); vs Lloyd at 4 passes:
  6,533,687,065 (0.999x). The paper's large-scale advantage is not reproduced at `n = 2,048`,
  `k = 16`, `dim = 8`, which is the ADR's own caveat about adopting its scaling laws.
- **A pre-existing library test fails on clean `origin/main` too**:
  `tests::apply_calls_are_confined_to_bounded_fixture_and_admitted_transition`
  (`crates/uor-r4-discrete/src/lib.rs:426` (425 on `main`)). It is not introduced by this unit.
- No model or quality claim; no GPU, no float training, no serving change.

## What changed

- `crates/uor-r4-discrete/src/e_means.rs` (new, live through `lib.rs`): the integer fitter (Q16
  counts/sums and cosine, `i128` distances), its k-means++ initialization, `squared_error`, and two
  integer-only tests (the parity gate and a bad-shape refusal).
- `crates/uor-r4-discrete/src/address_cells.rs`: `AddressCellsMode::EMeansV1 { seed }`, its label,
  the `e_means_cells` placement, and a determinism/validity/measurement test.
- `crates/uor-r4-discrete/src/bin/address-cells-codebook.rs`: `e-means` and `e-means:SEED`.
- `docs/reference/adr/0007-stage-interfaces-for-end-to-end-integration.md`: Amendment 2 records the
  uncompiled sites and the negative measurement; it changes no rule.

## Measurements

- Fitter, synthetic 16 clusters / 2,048 points / 8 dims: e-means 4 epochs **1.000x** Lloyd 16 and
  **0.999x** Lloyd 4 (total squared error as above); byte-identical centroids across two runs.
- A8 fixture (512 tokens, 4 blocks, quota 2): greedy **14,676**, e-means **31,780** weighted
  cell-L1 (**2.165x**); the e-means table is injective (512 distinct) and occupies all 256 cells at
  the exact quota; two runs are identical.
- `cargo test -p uor-r4-discrete --lib`: **359 passed, 1 failed (pre-existing), 1 ignored**.

## Checks run at the head

- `cargo test --locked -p uor-r4-discrete --lib e_means` — 4 passed (including a direct revival
  test with a crafted dead count).
- `cargo test --locked -p uor-r4-discrete --lib` — 359 passed, the pre-existing failure, 1 ignored.
- `cargo check --locked -p uor-r4-discrete --bin address-cells-codebook` — clean.
- `cargo clippy --locked --no-deps -p uor-r4-discrete --lib --bin address-cells-codebook` — no
  warning in the changed lines.
- `rustfmt --check` on the three changed files — clean.
- `uor-harness loop pr-check --local` — D32 clean after the integer rewrite.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at
  the head.

## Skills used

- `decide-first`: the arm's gate (parity at 4 epochs, deterministic table) was declared before the
  measurement; the placement loss is reported rather than tuned away.
- `solution-integrity`: the default path is untouched and the new arm is opt-in; no existing
  assertion is weakened.
- `scope-guard`: one crate, one fitter, one mode, one ADR amendment; no serving or artifact change.

## Still open

- I3 needs a genuine live k-means target, e.g. a fitted read-key codebook in the integer path, or a
  revived site with its reopen condition met. The greedy A8 placement is not that target.
- The A8 arm's promotion would need a frozen panel and its control; none is claimed here.
- D86 step 6 still gates any serving use of a fitted codebook.

## Plain summary

The e-means fitter now exists as live code and passes its parity gate, but the only live codebook
places tokens by a greedy quota rule, where e-means is 2.17x worse. The arm is opt-in and unpromoted,
the default is unchanged, and ADR-0007 now records that its four named fit sites are uncompiled.
