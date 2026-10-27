# REPORT: infra-loop-harness, antigravity-114

References #2904, #2858, #2859, #2862, #2868, #2869.

## What did not work

- Leaving `m2-train-grounded-replies` in `docs/roadmap/items/m3-qualify-memory.md` (`#2858`) `depends_on:` blocked `m3-qualify-memory` (`#2858`) and all 6 downstream `M4`–`M8` milestone items (`m4-export-uorlut01-cli` `#2859`, `m5-laptop-full-path-measurement` `#2862`, `m6-qualify-reasoning-coding` `#2868`, `m7-parity-library-cli-browser` `#2869`, `m8-release-packaging` `#2871`, and `m8-independent-verification` `#2870`) in `uor-harness loop next` behind `m2-donor-corpus-generation` (`#2848`, `status: held` under D41's `CHARTER.md` §3.10 / §5.1 owner-dispatch gate), even though `docs/integration/project-track.md:116` (*"On the selected M1/M2 artifact, `m3-memory` passes..."*) and `docs/roadmap/items/m3-qualify-memory.md:19,36` (*"waits on an M1 or M2 model"*, *"Name the selected artifact by hash (blocked until M1 or M2 yields one)"*) require the selected M1 **or** M2 artifact and M1 (`#2029` / `m1-close-saved-model` `#2837`) is now `status: done` with the selected two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `04652a3d…` and `1b49e8bd…`) integer exports.
- Leaving `m3-qualify-memory.md` (`#2858`), `m4-export-uorlut01-cli.md` (`#2859`), `m5-laptop-full-path-measurement.md` (`#2862`), `m6-qualify-reasoning-coding.md` (`#2868`), and `m7-parity-library-cli-browser.md` (`#2869`) without their `notebook:` and `docs/harness/prep/<id>*` / `docs/loop/tasks/<id>/**` (and `crates/uor-r4-discrete/**` for GPU `DeviceScorer` evaluation in M4/M6) scope entries would prevent executing sessions from writing pre-registration sheets, notebook rows, or GPU evaluators inside item scope.

## What changed

- **Size:** `9` files changed (`0` code lines, `0` test lines, `175` docs lines, `1` generated line in `docs/roadmap/README.md`), `1.0x` of the small docs-only decision-card estimate (`~175` lines).
- **Files modified:**
  1. [`docs/loop/tasks/infra-loop-harness/antigravity-114/ORDER.md`](ORDER.md) — Scoped order and cancel probe.
  2. [`docs/loop/tasks/infra-loop-harness/antigravity-114/DECISION.md`](DECISION.md) — `CHARTER.md` §5.1 (`D68`) decision card and whole-project context.
  3. [`docs/roadmap/items/m3-qualify-memory.md`](../../../../roadmap/items/m3-qualify-memory.md) — Updated `depends_on:` from `[m2-train-grounded-replies, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]` to `[m1-close-saved-model, object-store-consumer-m3, m3-memory-turn-discriminator-gate, m3-policy-precedence-and-sieve-tests]`, added `notebook: docs/research/notebook/m3-durable-memory.md`, and updated `scope:` to `[crates/uor-r4-integer/**, crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/harness/prep/m3-qualify-memory*, docs/labs/m3-qualify-memory/**, docs/loop/tasks/m3-qualify-memory/**, docs/research/notebook/m3-durable-memory.md]`.
  4. [`docs/roadmap/items/m4-export-uorlut01-cli.md`](../../../../roadmap/items/m4-export-uorlut01-cli.md) — Added `crates/uor-r4-discrete/**`, `docs/harness/prep/m4-export-uorlut01-cli*`, `docs/loop/tasks/m4-export-uorlut01-cli/**`, and `docs/research/notebook/m4-serving.md` to `scope:`.
  5. [`docs/roadmap/items/m5-laptop-full-path-measurement.md`](../../../../roadmap/items/m5-laptop-full-path-measurement.md) — Added `notebook: docs/research/notebook/m4-serving.md` and `docs/harness/prep/m5-laptop-full-path-measurement*`, `docs/loop/tasks/m5-laptop-full-path-measurement/**`, `docs/research/notebook/m4-serving.md` to `scope:`.
  6. [`docs/roadmap/items/m6-qualify-reasoning-coding.md`](../../../../roadmap/items/m6-qualify-reasoning-coding.md) — Added `notebook: docs/research/notebook/m6-reasoning-coding.md` and `crates/uor-r4-integer/**`, `crates/uor-r4-discrete/**`, `docs/loop/tasks/m6-qualify-reasoning-coding/**`, `docs/research/notebook/m6-reasoning-coding.md` to `scope:`.
  7. [`docs/roadmap/items/m7-parity-library-cli-browser.md`](../../../../roadmap/items/m7-parity-library-cli-browser.md) — Added `notebook: docs/research/notebook/m4-serving.md` and `docs/harness/prep/m7-parity-library-cli-browser*`, `docs/loop/tasks/m7-parity-library-cli-browser/**`, `docs/research/notebook/m4-serving.md` to `scope:`.
  8. [`docs/roadmap/README.md`](../../../roadmap/README.md) — Re-rendered via `cargo run -p xtask -- roadmap render`.
  9. [`docs/loop/tasks/infra-loop-harness/antigravity-114/REPORT.md`](REPORT.md) — This report.

## Measurements

- **Host-only roadmap validation (`$0` GPU spend, `0` tokens/s model decode run in this governance unit):**
  - Command: `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target/antigravity-m3-m4-unblock ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check`
  - Conditions: macOS host, clean worktree at `antigravity/m3-m4-dependency-unblock`.
  - Result: `roadmap check: 95 items OK`.

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target/antigravity-m3-m4-unblock ~/.cargo/bin/cargo run --offline -p xtask -- roadmap render` -> wrote `docs/roadmap/README.md` (`95 items`)
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/target/antigravity-m3-m4-unblock ~/.cargo/bin/cargo run --offline -p xtask -- roadmap check` -> `roadmap check: 95 items OK`
- `uor-harness loop gates --full` -> `11 / 11` GREEN
- `uor-harness loop report --check --order docs/loop/tasks/infra-loop-harness/antigravity-114/ORDER.md` -> `report check: OK`
- `uor-harness loop pr-check --local` -> `PASS`

## Skills used

- `scope-guard`: Kept changes strictly inside `docs/roadmap/items/*.md`, `docs/roadmap/README.md`, and `docs/loop/tasks/infra-loop-harness/antigravity-114/`.
- `solution-integrity`: Verified whole-project milestone ordering (`PLAN.md` M1 -> M3 -> M4 -> M5..M8 and `docs/integration/project-track.md:116`) before updating `depends_on:` and `scope:`.

## Still open

- `infra-loop-harness` (`#2904`) remains `status: ready` as the shared loop umbrella item (`References #2904`).
- `m3-qualify-memory` (`#2858`) becomes immediately claimable in `uor-harness loop next` once `object-store-consumer-m3` (`#2873`, PR `#3323`) merges, followed by `m4-export-uorlut01-cli` (`#2859`) and `M5`–`M8`.

## Plain summary

- **`UORLUT01` (Lookup-Table Serving Artifact):** The project's whole-number model file format that runs using only integer addition, bit shifts, and table lookups (`D11`).
- **`D68` (Decision Review Rule in `CHARTER.md` §5.1):** The project rule allowing a lab to approve a scope, status, or dependency fix after a fresh-context review of the whole project, logging the decision for the owner (`Casey`) to reverse if needed.
- **`D41` (One-Time Offline Donor Exception in `CHARTER.md` §3.10):** A rule requiring the project owner's explicit release at dispatch before running an external donor model (`m2-donor-corpus-generation`, `#2848`).
- **What this unit did:** Milestone 1 (`M1`, `#2029`) is already complete and produced two selected whole-number model files (`UORLUT01` seeds 1 and 2). Milestone 3 (`m3-qualify-memory`, `#2858`) is defined in the project plan as running on the selected Milestone 1 or Milestone 2 model, and Milestone 4 through Milestone 8 depend on Milestone 3. However, `m3-qualify-memory.md` listed `m2-train-grounded-replies` (which is waiting on the owner-dispatched `D41` donor step) in its `depends_on:` header instead of `m1-close-saved-model` and `object-store-consumer-m3`. This unit updates `m3-qualify-memory.md` to depend on `m1-close-saved-model` and `object-store-consumer-m3` and completes the `notebook:` and `scope:` declarations for `M3`–`M7`, unblocking 7 milestone items across `M3`–`M8`.
