default:
    @just --list

# Launch the bounded native geometric router dashboard. This does not download
# weights, compile a corpus, score a graph, or start a qualification run.
demo:
    cargo run --bin r4 -- demo

# Build and serve the browser-only geometric router demo. The dashboard uses
# the local WASM router when no native `/api/sysinfo` endpoint is present.
wasm-dashboard:
    wasm-pack build --target web
    python3 -m http.server 8000

# Run the Cucumber/Gherkin behavior suite.
bdd:
    cargo test --test bdd

# --- #273 template adoption: the register conformance gate ---
# R1/R2/R3 landed in Phase 1. #510 wires R4 (deferral), R5 (audit-limits) and
# R6 (cargo-deny): R4's `audit-deferral` honours the carve-out --- a marker is
# a violation only when its line does not cite an `open` claim in
# model/ledger.toml --- and R6's deny.toml allow-lists uor-r4's pinned git
# sources. R5's incremental migration is COMPLETE (graph-cli hit 0 in #587):
# every shipped crate's `Result` error surface now names only the four
# sanctioned types (see `xtask/src/audit.rs`), and CI's `register-conformance`
# job (.github/workflows/ci.yml) has enforced it since #588. This recipe was
# left listing only R1/R2/R3/R4/R6 after that landed, which meant `just
# vv-register` could pass locally on an R5 violation that CI would then catch
# --- silently, since nothing here said R5 was even expected to run. Brought
# back in sync with CI: R5 joins below.

# R1: model/*.toml is the single source; CONFORMANCE.md is generated from it.
check-model:
    cargo run -q -p xtask -- check-model

# Regenerate CONFORMANCE.md from the register after changing model/*.toml.
model-write:
    cargo run -q -p xtask -- check-model --write

# R2/R3: the honesty meta-gate and the ID-tagged BDD conformance suite.
register-bdd:
    cargo test -q -p repo-conformance

# R4: nothing is deferred and unregistered. A marker (TODO, unimplemented!, ...)
# fails unless its line cites an `open` ledger claim (the dormant carve-out).
audit-deferral:
    cargo run -q -p xtask -- audit-deferral

# R5: every shipped crate's Result names only a sanctioned error type
# (NotAProduct / ObservedBound / KappaError / SourceUnavailable) --- no
# arbitrary limitation. Migration complete since #587/#588.
audit-limits:
    cargo run -q -p xtask -- audit-limits

# R6: supply-chain hygiene over the real graph --- advisories, licences, bans,
# and sources (the pinned git deps are allow-listed in deny.toml).
deny:
    cargo deny check

# The register acceptance gate --- the wired slice of the template's `just vv`
# (R1/R2/R3 + R4/R5/R6), matching CI's `register-conformance` job exactly.
vv-register: check-model register-bdd audit-deferral audit-limits deny
    @echo "vv-register: the register conformance gate passed (R1/R2/R3/R4/R5/R6)"
