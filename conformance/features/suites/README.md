# UOR-R4 Geometric Language Model — historical conformance suites

**Dormant historical conformance inventory.** These suites preserve the
earlier R4G1 programme’s contracts and evidence. They are not an automatic
development gate for the active geometric-intelligence research programme.
Use focused checks required by the changed behavior under the
[canonical plan](../../../docs/integration/project-track.md) and current agent
policy; broader conformance and release checks apply when their boundaries are
affected. PR/merge-queue statuses are compatibility acknowledgements, not suite
execution. `CONFORMANCE.md` remains generated history and is not edited by
hand.

The [project map](../../../docs/PROJECT_MAP.md),
[current-state](../../../docs/integration/current-state.md) and
[model direction](../../../archive/integration/model-direction-2026-09.md) distinguish
accepted native bounded attention/inference from these historical contracts.
A conformance marker does not establish general prose completion or reasoning.

One Gherkin file per suite, one scenario per conformance ID (R3).

A scenario is tagged with its ID and its honesty level:

```gherkin
Feature: <suite name>

  <what the suite is about, in a sentence.>

  @RF-01 @build
  Scenario: <the statement, from conformance/ids.toml>
    Given <the fixture>
    When the suite exercises RF-01
    Then <the assertion, in the register's words>
```

Each row in `conformance/ids.toml` names the suite file its scenario lives in, its
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
under #273. Within this historical register, new conformance entries start with a row in
`conformance/ids.toml`, then a scenario and marker test. That register procedure does
not impose a new conformance campaign on every native model change.
