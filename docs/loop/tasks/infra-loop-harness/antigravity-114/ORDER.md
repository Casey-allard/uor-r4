# ORDER: infra-loop-harness, antigravity-114

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Setup row: 2. Base: `8f8f06dd5`.

## Goal

Execute a `CHARTER.md` §5.1 (`D68`) decision review to align the dependency and scope declarations of [`m3-qualify-memory`](../../../../roadmap/items/m3-qualify-memory.md) (`#2858`), [`m4-export-uorlut01-cli`](../../../../roadmap/items/m4-export-uorlut01-cli.md) (`#2859`), [`m5-laptop-full-path-measurement`](../../../../roadmap/items/m5-laptop-full-path-measurement.md) (`#2862`), [`m6-qualify-reasoning-coding`](../../../../roadmap/items/m6-qualify-reasoning-coding.md) (`#2868`), and [`m7-parity-library-cli-browser`](../../../../roadmap/items/m7-parity-library-cli-browser.md) (`#2869`) with the project track (`docs/integration/project-track.md:116`) now that M1 (`#2029` / `m1-close-saved-model` `#2837`) is `status: done`:

1. **`docs/roadmap/items/m3-qualify-memory.md` (`#2858`, M3):** Update `depends_on:` from `[m2-train-grounded-replies, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]` to `[m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]` (matching `project-track.md:116` and `m3-qualify-memory.md:19,36`: *"On the selected M1/M2 artifact"*, *"waits on an M1 or M2 model"*), add `notebook: docs/research/notebook/m3-durable-memory.md`, and update `scope:` to `[crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/harness/prep/m3-qualify-memory*, docs/labs/m3-qualify-memory/**, docs/loop/tasks/m3-qualify-memory/**, docs/research/notebook/m3-durable-memory.md]` (removing `crates/uor-r4-training/**`, which `m3-qualify-memory.md:19` explicitly excludes from the integer-path verdict). This unblocks `m3-qualify-memory` (`#2858`) and 6 downstream `M4`–`M8` items (`7` total milestone items) that were falsely blocked on D41's owner-dispatched `m2-donor-corpus-generation` (`#2848`).
2. **`docs/roadmap/items/m4-export-uorlut01-cli.md` (`#2859`, M4), `m5-laptop-full-path-measurement.md` (`#2862`, M5), `m6-qualify-reasoning-coding.md` (`#2868`, M6), and `m7-parity-library-cli-browser.md` (`#2869`, M7):** Add missing `notebook:`, `docs/harness/prep/<id>*`, `docs/loop/tasks/<id>/**`, and `crates/uor-r4-discrete/**` (for GPU `DeviceScorer` evaluation in M4/M6) entries to `scope:`, and re-render `docs/roadmap/README.md`.

## Cancel probe (run first)

Ran `uor-harness loop next` at `8f8f06dd5` and inspected `docs/roadmap/items/{m3-qualify-memory,m4-export-uorlut01-cli,m5-laptop-full-path-measurement,m6-qualify-reasoning-coding,m7-parity-library-cli-browser}.md`: `m3-qualify-memory` (`#2858`) and all 6 downstream `M4`–`M8` items (`m4-export-uorlut01-cli`, `m5-laptop-full-path-measurement`, `m6-qualify-reasoning-coding`, `m7-parity-library-cli-browser`, `m8-release-packaging`, `m8-independent-verification`) are blocked in `loop next` solely because `m3-qualify-memory.md` lists `m2-train-grounded-replies` (which depends on `m2-donor-corpus-generation`, `status: held` under D41 §3.10) instead of `m1-close-saved-model` and `object-store-consumer-m3`. Classification: `instrument`. Proceed.

## Files in scope

- `docs/roadmap/items/*.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/**`

## Done when

1. `docs/loop/tasks/infra-loop-harness/antigravity-114/DECISION.md` is written and passes a fresh-context D68 decision review (`CHARTER.md` §5.1), logged with `[antigravity] owner log:` on `#2858`, `#2859`, and `#2904`.
2. `docs/roadmap/items/{m3-qualify-memory,m4-export-uorlut01-cli,m5-laptop-full-path-measurement,m6-qualify-reasoning-coding,m7-parity-library-cli-browser}.md` and `docs/roadmap/README.md` are updated and `cargo run -p xtask -- roadmap check` and `uor-harness loop gates --full` pass (`11 / 11` GREEN).

## Stop and ask if

- Any edit touches an owner-only path (`docs/harness/owner-paths.json`) or changes `CHARTER.md` §3 or GPU caps.
