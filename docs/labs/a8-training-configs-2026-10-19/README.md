# A8 address-cells learning sheet: FIT, registered in source, configs staged (2026-10-19)

- **Line:** M1 learned attention (#2029). **Lab:** claude. **Count:** registration only. It changes no headline number and runs no compute.
- **Sheet:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md), **TEST FITNESS: FIT**, SHA-256 `3e622bebd7c468c6cd3d15a38486768c7902807998f71cae039a4341ab55da2b`.
  - Its parent is the PrimeZeta injective sheet (FIT #2712). Exactly one variable changes: the pinned table, now `address_structured_v1` (file `6600bdb1…`, table `4122cbab…`).
  - The sheet exists because the address-cells pre-flight (#2751) promoted A8 under its step 4: B3_init(P_succ) 1,351 / 3,847 and net G-Cap 1,351.
- **Review:** a fresh-context adversarial reviewer subagent read draft `6b4e9762` against `origin/main` and returned **FIT-WITH-FIXES**. All five findings are applied:
  1. **The exact sheet mapping (the material fix).** Before this change, every pinned table bound the PrimeZeta sheet. S8, or any other injective table, would have run under PrimeZeta's D22 scope. Unregistered pinned tables are now refused.
  2. §3 is inherited with A8's pins, and negative (b) is rebuilt from A8.
  3. 133 is labelled as the full-panel upper bound, and the FrozenZero P_succ control term is recorded at the pre-run gate.
  4. The reproduction gate applies to both arms, with each value's row basis named.
  5. The float-free claim cites its grep.

## Source (`crates/uor-r4-discrete/src/attention_learning.rs`)

`registered_sheet` now returns `Result` and maps exactly:

| Codebook initialization | Registered sheet |
|---|---|
| `SeededDeviceV1` | the grandparent `m1-learned-attention-link-2026-10-17.md` (unchanged) |
| `PinnedTableV1`, file `83c1380a…` (PrimeZeta) | `m1-attention-injective-init-DRAFT-2026-10-18.md` (unchanged) |
| `PinnedTableV1`, file `6600bdb1…` (A8) | **this sheet**, `A8_SHEET_SHA256` |
| any other `PinnedTableV1` | **refused**: "attention binds a sheet only for a registered pinned table" |

Its two callers, `validate_plan_header` and `validate_plan`, propagate the error.

### Tests

- `registered_sheet_constants_match_committed_sheets` now also pins this sheet's SHA and title. It checks that the title does not start with the parent's prefix.
- `pinned_plan_binds_injective_sheet_only`: its fixture changes from a placeholder SHA to the PrimeZeta file SHA, as the stricter rule requires. It also refuses the A8 sheet.
- New `a8_plan_binds_a8_sheet_only`: an A8 plan binds its sheet and refuses the PrimeZeta and seeded sheets.
- New `unregistered_pinned_table_binds_no_sheet`: an S8-pinned plan is refused under either pinned sheet.
- New `registered_table_file_shas_match_committed_artifacts`: both table-file constants are pinned to committed artifacts — PrimeZeta to the runbook table file and both runbook configs, A8 to both staged A8 configs (code-review finding 1).

### Gates (local, at this PR's head)

- `cargo test -p uor-r4-discrete --lib attention_learning`: 34 / 34 pass.
- The full `uor-r4-discrete` lib suite: 142 pass, 1 ignored.
- **Negative gate:** restoring the old fallback (any pinned table binds PrimeZeta) makes `unregistered_pinned_table_binds_no_sheet` fail at `attention_learning.rs:1662`. Restored, the suite returns to 33 / 33.
- `cargo fmt --check` is clean.
- Clippy reports 0 lints in `attention_learning.rs`. Pre-existing lints elsewhere (`uor-r4-source-interface`, `completion_baseline.rs`, `controller.rs`) are outside this diff.
- A CUDA-feature typecheck (`--features cuda --all-targets`) compiles. It used a synthetic nvcc stub, so it is a host typecheck only, not a GPU result.
- The banned-token scan, and `uor-harness ratchet d23` / `d25`: OK.

A fresh-context code review of the Rust diff returned PASS-WITH-FIXES: finding 1 (pin the PrimeZeta file-SHA constant to a committed artifact) is applied by the new test, and finding 2 (A8 plans keep the `injective-<12-hex>-` scope wording, distinguished by table prefix) is kept as a deliberate choice.

This changes `driver_source_sha256`. The release packet binds the new value through `execution.driver_source_sha256`, and nothing in code pinned the old one.

## Configs (`evidence/`)

| File | SHA-256 | Arm |
|---|---|---|
| `a8-relation-seed1.json` | `469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e` | Relation `learn_v1`; byte-identical to the pre-flight's `a8.json` |
| `a8-frozen-zero-seed1.json` | `cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479` | FrozenZero `frozen_zero_v1` |

- Each config differs from the parent runbook config (`docs/labs/injective-attention-runbook-2026-10-19/evidence/injective-{relation,frozen-zero}-seed1.json`) only in `codebook_initialization`.
- The two arms differ only in `relation_learning_policy`.

## Next

PM dispatch with an exact costed plan, then Casey's typed D40 release. At the new head, the run first passes:
- the inherited pre-flight battery;
- the §3 gates on both arms, with the unpinned G-Init comparison and negative (b) rebuilt from A8;
- the reproduction gate (1,351 / 4,244 / 133).

Training stays HELD until then.
