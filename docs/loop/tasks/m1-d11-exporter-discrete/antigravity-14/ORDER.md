# ORDER: m1-d11-exporter-discrete, antigravity-14

Lab: antigravity. Item: [m1-d11-exporter-discrete](../../../../roadmap/items/m1-d11-exporter-discrete.md). Base: `55311a46b`.

## Goal

The item's done-when reads:

> Exporter on main; a discrete snapshot round-trips through `crates/uor-r4-integer` with matching integer outputs on finite tests; the serving audits stay green (0 float, 0 mul/div). The audits that exist today are 44 native symbol ranges and 7 of 7 WebAssembly serving functions.

## Cancel probe (run first)

Cancel probe: not applicable (host-only export and integer serving container task; no pod or background job is launched).

## Files in scope

- `crates/uor-r4-discrete/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-lut/**`
- `docs/mechanisms/lut-serving.md`
- `docs/model/serving-and-export.md`
- `docs/roadmap/items/m1-d11-exporter-discrete.md`
- `docs/loop/tasks/m1-d11-exporter-discrete/antigravity-14/**`

## Done when

1. A design note is posted on `#2838` mapping each discrete `Snapshot` field ([`crates/uor-r4-discrete/src/snapshot.rs`](../../../../../crates/uor-r4-discrete/src/snapshot.rs)) to its `UORLUT01` header and table sections under schema `uor-r4.lut-discrete/1`, explaining why the discrete read/writer path requires no exponent/softmax table and no dense matmul, naming the crates touched, and listing the finite round-trip and negative controls.
2. [`crates/uor-r4-lut/src/format.rs`](../../../../../crates/uor-r4-lut/src/format.rs) defines `DISCRETE_SCHEMA = "uor-r4.lut-discrete/1"` and the discrete shape/header descriptor and validates discrete `UORLUT01` containers.
3. [`crates/uor-r4-discrete`](../../../../../crates/uor-r4-discrete/src/lib.rs) exports a validated discrete `Snapshot` to a `UORLUT01` (`uor-r4.lut-discrete/1`) integer container and decodes it back losslessly, with negative controls rejecting malformed or non-discrete snapshots and mutated sections.
4. [`crates/uor-r4-integer`](../../../../../crates/uor-r4-integer/src/native_geometry.rs) loads the exported `UORLUT01` (`uor-r4.lut-discrete/1`) artifact and evaluates finite prefixes in integer-only arithmetic (`Route`, `Flock`, and `Relation` read modes, cell admission, and token ranking), matching the discrete reference outputs bit-for-bit on finite tests and rejecting corrupted containers.
5. Both serving instruction audits (`scripts/audit_zero_matmul_serving.py --stack` over all 44 native symbol ranges and `scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap` over 7/7 WebAssembly serving functions) pass with `0 float`, `0 mul`, and `0 div`.
6. [`docs/mechanisms/lut-serving.md`](../../../../mechanisms/lut-serving.md), [`docs/model/serving-and-export.md`](../../../../model/serving-and-export.md), and [`docs/roadmap/items/m1-d11-exporter-discrete.md`](../../../../roadmap/items/m1-d11-exporter-discrete.md) are updated and `uor-harness loop gates --full` and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the declared scope in [`docs/roadmap/items/m1-d11-exporter-discrete.md`](../../../../roadmap/items/m1-d11-exporter-discrete.md) would need to change.
- Any GPU run, bar change, or charter exception were required.
