# Suites

One Gherkin file per suite, one scenario per conformance ID (R3).

A scenario is tagged with its ID and its honesty level:

```gherkin
Feature: <suite name>

  <what the suite is about, in a sentence.>

  @RF-01 @build
  Scenario: <the statement, from model/ids.toml>
    Given <the fixture>
    When the suite exercises RF-01
    Then <the assertion, in the register's words>
```

Each row in `model/ids.toml` names the suite file its scenario lives in, its
honesty level (`build`), and — since #830 — its execution `scope`, its serving
`reachability`, and an `evidence` pointer. `CONFORMANCE.md` is generated from the
register (never edited by hand); `just bdd` / `cargo test -p repo-conformance`
fail if an ID has no scenario, a scenario has no ID, a scenario's level disagrees
with the register, or an ID has no marker test whose name ends in it lowercased
with underscores.

`build` is *harness-built* (structural) status: the suite is constructed and
validated against its oracle. It is a separate axis from an empirical verdict
(`PASS` / `FAIL` / `UNAVAILABLE`); a fixture-gated suite that runs with its pinned
fixtures absent is `UNAVAILABLE`, never a `PASS` (see RF-29 and
`crates/repo-model/src/empirical.rs`).

The register currently carries the 29 `RF-*` feature-capability suites migrated
under #273. A new capability still starts with a row in `model/ids.toml`, then a
scenario here, then a failing marker test — in that order, because the order is
the discipline (`AGENTS.md`).
