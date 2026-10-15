# #511 — feat(#273): adopt the template register machinery and bring uor-r4's BDD under it

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-08T21:52:23Z
- **Updated:** 2026-10-09T13:08:51Z
- **Closed:** 2026-08-08T22:12:00Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/511
- **Merged:** 2026-08-08T22:12:00Z
- **Merge commit:** b47ffc2e4544c9c7672e370478ac208fcca72b32
- **Base / Head:** main / UOR-Foundation:feat/273-register-adoption
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Rebases uor-r4 onto Alex's `UOR-Foundation/template` register machinery (Casey's Phase-0 sign-off) and — the actual point — brings **uor-r4's real BDD suites under it**. The register describes uor-r4's actual capabilities; it does not stand up an empty register beside them.

### The machinery (from the template, self-contained)
- **`repo-model`** — parses `model/*.toml`, the single source of every registered claim (R1); generates `CONFORMANCE.md`.
- **`repo-conformance`** — the honesty meta-gate + BDD runner (R2/R3).
- **`xtask`** — `check-model` and the audit gates.
- `deny.toml` (R6 config) staged; the crates are self-contained (no workspace inheritance, root manifest untouched) and added to `members` only (not `default-members`), so default builds are unchanged.

### The migration (the substance)
uor-r4's **29 feature suites / 101 scenarios** are now registered: one conformance ID per feature-capability (**RF-01…RF-29**, level `build`), every scenario tagged `@RF-NN @build`, a register row per ID, and an explicit **marker test per ID** (`repo-conformance/tests/registered.rs`) that binds each ID to its real, well-formed scenarios and fails if the suite regresses.

### The R4 carve-out (Phase-0 decision)
The four dormant modules (`cd_space`, `lie_jordan`, `bott_fock`, `quantum_cover`) are registered as **`open` ledger claims** with activation gates — registered, not deleted, which is how R4 "nothing deferred" is honored without dropping measured-negative-but-kept capabilities.

### CI + Justfile
An **advisory** `register conformance (R1/R2/R3)` job runs `check-model` + the meta-gate (not in the required set — cannot block the queue; it is green). `Justfile` gains `check-model` / `register-bdd` / `vv-register`.

## Verified green

| gate | result |
|---|---|
| `check-model` (R1) | 29 ids, CONFORMANCE.md equals the model |
| meta-gate CM-01/02/03 (R2/R3) | 4 passed |
| marker tests (RF-01…29) | 29 passed |
| **existing cucumber `cargo test --test bdd`** | **101/101 scenarios — unbroken by the @RF tags** |
| fmt / clippy (`-D warnings`, `--all-features`) / claim-wording gate | green |

## Not in scope — follow-up #510

The R4 `audit-deferral` (needs the carve-out taught to it), R5 `audit-limits` (65k LOC, incremental), and R6 `cargo deny` (allowlist uor-r4's git sources) gates. When green they promote the register job to required. The register itself is already the single source; #510 finishes the gate coverage on top of it.

Closes #273.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

