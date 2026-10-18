# PROJECT_MAP ops row and heading spacing — 2026-10-18

Closes the open docs task #2679 (front-door audit 2.6, part of #2028).

- Adds an `[ops/](../ops/)` row to "Where work and evidence are stored", naming the RAPL
  energy-measurement protocol and synthetic-stack receipt (provisional, not full-path M5 energy), the D5-b
  address-assignment memo (formulation only; no solver), the Braket Bell-state toy receipt (unverified)
  and the measurement report.
- Fixes `## Causal continuation interfaces, September 24 UTC` and
  `## September 23 principal continuation components`.
- Leaves every other section, claim and link unchanged.

Checks: `cargo run -q --locked -p uor-r4-claim-wording -- .` PASS; `git diff --check`; only intended
docs/record/current-state files changed. No code, model or compute; $0.