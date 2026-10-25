# ORDER: d11-exporter-float-trunk, antigravity-75

Lab: antigravity. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md) (#2820). Base: `9e7f7a0fdd40b03dc5aa63d0588c605a025d246d`.

## Goal

The item's done-when reads:

> A float-trunk snapshot with recurrence and the Hamilton gate exports to D11 with the audits green and the export gap measured inside the band. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

Build the D11 integer exporter (`export_float_trunk_uorlut01`) and reader/evaluator support for a D37 offline float trunk (`RecGates::Geometric` dyadic recurrence with quaternion transport and `Gate::Hamilton` with 120-element icosian snap) in `crates/uor-r4-integer/src/stack/`, verify finite CPU controls across all three read geometries (`dot`, `lorentz`, `l2`) and the native zero-multiply audit (`scripts/audit_zero_matmul_serving.py --stack`), record the result in `docs/mechanisms/lut-serving.md`, and complete the non-author `TEST FITNESS: FIT` and D68 decision reviews for the Stage 4 float-trunk export-gap evaluation (`status: held` until a Stage 3 float trunk is trained).

## Cancel probe (run first)

1. Verify `uor-harness loop claim d11-exporter-float-trunk --lab-topic d11-exporter-float` refusal is solely the `infra-loop-harness` (`PR #3169`, `codex-1`) umbrella overlap on `docs/mechanisms/lut-serving.md`, and verify via `gh pr view 3169 --json files,isDraft,updatedAt` that PR #3169 (`codex-1`, idle draft) modifies only `PLAN.md`, `docs/integration/DECISIONS.md`, and `docs/loop/tasks/infra-loop-harness/codex-1/**` (`0` overlapping files). Claimed under the `run-antigravity.txt` Part 3 Step 2 umbrella-lock exception (`instrument`).
2. Verify all three dependencies (`float-training-module` #2823, `m1-d11-exporter-discrete` #2838, `owner-decision-float-path-scope` #2875) are `status: done` on `origin/main` (`9e7f7a0fdd40b03dc5aa63d0588c605a025d246d`), while no trained Stage 3 float-trunk checkpoint exists yet (`#2890`, `#2824`, `#2891` wait on `#2886`). Classify the absence of a trained Stage 3 float trunk for the GPU DEV export-gap run as `instrument` (upstream Stage 3 training dependency, not a mechanism negative).

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/mechanisms/lut-serving.md`

## Done when

- A design note and one-page export-gap pre-registration are posted on Issue #2820 covering: (a) snapshot format handed to the exporter, (b) dyadic recurrence (`RATE_TABLE` / `KEEP_TABLE` at `i = min(floor(16 * clamp(g + d, 0, 1)), 15)`) with quaternion transport as shifts, adds, and table lookups, (c) 120-element icosian snap (`stack_snap_select` / `stack_snap_rotation`) for `Gate::Hamilton`, (d) absence of softmax, squashing, and dense GEMM, and (e) finite CPU control specification and DEV two-seed export-gap band (`398` top-1, `748` top-8, `930` admission);
- `crates/uor-r4-integer/src/stack/` exports and evaluates a geometric float-trunk snapshot (`RecGates::Geometric` + `Gate::Hamilton` with icosian snap and flock rank read) in `UORLUT01` (`uor-r4.lut-stack/3`), with finite CPU controls comparing the float forward and exported integer forward across every input of a small vocabulary (`cargo test -p uor-r4-integer --features serving-wasm --lib --test memory_shape --no-fail-fast`) and `python3 scripts/audit_zero_matmul_serving.py <rlib> --stack` passing with zero `mul`/`div`/`float` instructions;
- Fresh-context non-author `TEST FITNESS: FIT` and D68 decision reviews pass and are logged on #2820 (`[antigravity] owner log:`), `docs/mechanisms/lut-serving.md` is updated, `docs/roadmap/items/d11-exporter-float-trunk.md` is set to `status: held` pending Stage 3 float-trunk training, and `uor-harness loop pr-check --local` and `uor-harness loop gates --full` pass (`References #2820`).

## Stop and ask if

- Any edit to `crates/uor-r4-training/src/stack_export.rs` or any file outside the declared scope is needed;
- Any check in `uor-harness loop gates --full` or `scripts/audit_zero_matmul_serving.py --stack` fails.
