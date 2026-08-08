default:
    @just --list

# Run the Cucumber/Gherkin behavior suite.
bdd:
    cargo test --test bdd

# --- #273 template adoption, Phase 1: the register conformance gate ---
# R1/R2/R3 only. The R4/R5 deferral/limits audits and R6 cargo-deny are Phase 2:
# R4 needs the dormant-module carve-out wired into `xtask audit-deferral`
# (the four modules are registered `open` in model/ledger.toml), and R6 needs
# uor-r4's git sources allowlisted in deny.toml.

# R1: model/*.toml is the single source; CONFORMANCE.md is generated from it.
check-model:
    cargo run -q -p xtask -- check-model

# Regenerate CONFORMANCE.md from the register after changing model/*.toml.
model-write:
    cargo run -q -p xtask -- check-model --write

# R2/R3: the honesty meta-gate and the ID-tagged BDD conformance suite.
register-bdd:
    cargo test -q -p repo-conformance

# The register acceptance gate --- the Phase-1 slice of the template's `just vv`.
vv-register: check-model register-bdd
    @echo "vv-register: the register conformance gate passed"
