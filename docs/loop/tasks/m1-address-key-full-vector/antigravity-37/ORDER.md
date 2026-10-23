# ORDER: m1-address-key-full-vector, antigravity-37

Lab: antigravity. Item: [m1-address-key-full-vector](../../../../roadmap/items/m1-address-key-full-vector.md). Base: `fb6156b46`.

## Goal

The item's done-when reads:

> Stage 1 bar: "enough distinct vectors that the number of positions still winnable clears the run bar of 1,941, and a starting reach above the address-cell model's 1,351 of 3,847", measured before any training. The audit also lists within-cell tie mass. The bar is not G-Pre of 4,894 at checkpoint 0, which every K16 arm already met.

## Cancel probe (run first)

Command:

```sh
~/.cargo/bin/cargo test -p uor-r4-discrete --lib address_cells::tests::full_vector_address_metric_expands_levels_and_lifts_fixture_admission
```

Result on base `fb6156b46` before adding the full-vector address metric and `AddressCellsMode::FullVectorV1`: `running 0 tests` (the host full-vector address metric, `FullVectorV1` placement mode, and finite CPU controls do not exist yet). With a deliberate break that drops the interior-block distance term in `full_vector_cell_l1` (reverting to the 5-value outer-block `cell_l1`), `full_vector_address_metric_expands_levels_and_lifts_fixture_admission` fails (`FAILED`, `distinct_levels == 5`).

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/mechanisms/discrete-learning.md`
- `docs/harness/prep/m1-address-key-full-vector*`
- `docs/labs/m1-address-key-full-vector/**`

## Done when

- `crates/uor-r4-discrete/src/address_cells.rs` implements the integer-only, softmax-free, matmul-free full-vector / interior-block address metric (`full_vector_cell_l1`, `cell_envelope_l1`, and `AddressCellsMode::FullVectorV1`) and finite CPU controls verifying `distinct_vectors == V`, full DEV alias ceiling (`dev_alias_ceiling_rows == dev_rows`), zero within-cell alias tie mass, expanded distance levels (`> 5`), reduced $K$-boundary tie bucket size, and higher fixture admission than the 5-value outer-block `cell_l1` metric.
- `docs/labs/m1-address-key-full-vector/README.md` records the design note and finite CPU control results, and the design note is posted on `#2834`.
- `docs/harness/prep/m1-address-key-full-vector-DRAFT-2026-10-23.md` stages the one-page pre-registration sheet (`TEST FITNESS: PENDING`, all 8 required `prep.sheet_shape` blocks) for the zero-update GPU pre-flight against the Stage 1 bar.

## Stop and ask if

- Any file outside the `ORDER.md` scope must change.
- Any GPU execution, CUDA kernel replacement on a pod, or bar/spend registration is needed (awaits non-author `TEST FITNESS: FIT` and owner release on `#2834`).
