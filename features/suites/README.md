# Suites

One Gherkin file per suite, one scenario per conformance ID (R3).

A scenario is tagged with its ID and its honesty level:

```gherkin
Feature: <suite name>

  <what the suite is about, in a sentence.>

  @CX-01 @build
  Scenario: <the statement, copied from model/ids.toml>
    Given <the fixture>
    When the suite exercises CX-01
    Then <the assertion, in the register's words>
```

The `suite` field of a row in `model/ids.toml` names the file its scenario
lives in, and `just bdd` fails if an ID has no scenario, a scenario has no ID,
or an ID has no test whose name ends in it lowercased with underscores.

Empty. The first capability this repository builds starts with a row in
`model/ids.toml`, then a scenario here, then a failing test --- in that order,
because the order is the discipline (`AGENTS.md`).
