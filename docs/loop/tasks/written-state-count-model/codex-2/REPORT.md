# REPORT: written-state-count-model, codex-2

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

The docs and full loop CI jobs at a38aa28d6 stopped at locked dependency resolution. The root Cargo.lock lacked the role diagnostic’s direct serde dependency despite both standalone research manifests resolving. The root graph also reaches the scorer as a dependency.

## What changed

Cargo added one dependency edge to the existing scorer package in the root lock. The fix introduces no new package or version. The experiment and source remain as reviewed in the [combined research report](../codex-1/REPORT.md).

## Measurements

On Wick, both released seeds produced token 1155 as the first token on 16/16 role-swap prompts. The exact source intervention repaired zero openings. Direct pre-writer identity repaired 16/16 per seed. One pair of identical token vectors then collapsed kite into ak. [First-token evidence and per-table attribution](../codex-1/ROLE-RESULT.md)

On 61 odd DEV-tune records the 131,072-byte two-frame count model used 4.606925 bits per byte and the one-token control used 3.844426. The diagnostic falls short of the registered minimum of 200 records. [Count reports and replay](../codex-1/COUNT-RESULT.md)

## Checks run at the head

Cargo generated the single lockfile line with offline workspace resolution. The root command `cargo metadata --locked --offline --format-version 1` then passed. The original [role](../codex-1/ROLE-RESULT.md) and [count](../codex-1/COUNT-RESULT.md) receipts include the independent Wick controls.

## Skills used

The native-space-cycle and UOR research architect procedures used in the first order apply to its measurements.

## Still open

The next research question is whether learned source selection can control an identity-preserving copy action and a span cursor. The full written-state P1–P5 gates and M2 remain open.

## Plain summary

Two teams located the first incorrect token and compared a geometric count model with alternatives of the same size. This repair lets the repository’s normal locked build resolve the diagnostic tool.
