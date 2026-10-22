# ORDER: m2-fresh-heldout-panel, antigravity-22

Lab: antigravity. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `fb6df367b`.

## Goal

> A fresh panel registered and sealed in advance, graded by the existing deterministic graders unchanged; bar: '≥ 256/512 saved questions and ≥ 40% of genuinely fresh ones'.

## Cancel probe (run first)

At `fb6df367b`, `docs/harness/panels/m2-reply512.json` lists `"noise": "unstated..."` and only the opened 128-row transfer draw (`0/128`), and neither `docs/harness/panels/m2-fresh256.json` nor `docs/labs/m2-fresh-heldout-panel/` exists. Result: task is live. Class: instrument.

## Calls

- Post the registration draft on `#2030` and `#2849`.
- Build the standalone zero-float Rust generator, `m2-reply512` spread analyzer, and `m2-fresh256` overlap/seal verifier in `docs/labs/m2-fresh-heldout-panel/instrument/`.
- Register `m2-fresh256` (`>= 103/256`) and measured `dev512` spread in `docs/harness/panels/m2-fresh256.json`, `docs/harness/panels/m2-reply512.json`, `docs/reference/storage.md`, `docs/mechanisms/native-geometric.md`, and `docs/labs/m2-fresh-heldout-panel/README.md`.

## Files in scope

- `data/**`, `docs/harness/panels/**`, `docs/mechanisms/native-geometric.md`, `docs/labs/m2-fresh-heldout-panel/**`, `docs/reference/storage.md`, `docs/roadmap/items/m2-fresh-heldout-panel.md`, `docs/roadmap/README.md`, `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/**`

## Done when

- `cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked` passes.
- `uor-harness loop gates --full`, `report --check`, and `pr-check --local` pass.

## Stop and ask if

- Any out-of-scope file, owner path, or GPU compute is needed.
