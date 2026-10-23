# ORDER: m1-address-key-full-vector, antigravity-38

Lab: antigravity. Item: [m1-address-key-full-vector](../../../../roadmap/items/m1-address-key-full-vector.md). Base: `250a8ed57`.

## Goal

Repair the `deepseek` `CHANGES` review on PR `#3070` (`250a8ed576df9a302a25c231a408dc3fadbc8728`): remove the D23 banned family word from the new doc comment at `crates/uor-r4-discrete/src/address_cells.rs:156` so `uor-harness ratchet d23` and `CI / harness rules and tests` pass with zero rise (`4451` across `165` files).

## Cancel probe (run first)

Command:

```sh
uor-harness ratchet d23
```

Result at `250a8ed576df9a302a25c231a408dc3fadbc8728`:

```text
FAIL: softmax added to executable source (D23 prohibits it):
  crates/uor-r4-discrete/src/address_cells.rs (0 -> 1)
```

Because `uor-harness ratchet d23` scans `.rs` source files including doc comments, the new doc comment on `cell_envelope_l1` at `crates/uor-r4-discrete/src/address_cells.rs:156` tripped the D23 ratchet (`0 -> 1`). The task stands.

## Files in scope

- `crates/uor-r4-discrete/src/address_cells.rs`
- `docs/loop/tasks/m1-address-key-full-vector/antigravity-38/**`

## Done when

- `crates/uor-r4-discrete/src/address_cells.rs:156` is reworded without the D23 banned family word.
- `uor-harness ratchet d23`, `uor-harness ratchet d25`, and `uor-harness ratchet d32` all pass (`OK: no softmax added. 4451 remaining across 165 files`).
- `cargo test -p uor-r4-discrete --lib address_cells` (`11 / 11`), `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Any file outside the `ORDER.md` scope must change.
- Any GPU execution or baseline ratchet change is needed.
