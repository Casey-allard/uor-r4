---
id: m6-verifier-temp-isolation
title: Isolate temporary files in concurrent M6 rustc verifier tests
milestone: infra
area: tooling
status: done
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

A shared temporary source file can make the verifier report a compilation failure for the wrong test.

## What is missing

Complete. Contributor PR #4038 was adopted in #4065. Each verifier invocation now owns an exclusively acquired directory for source, executable and diagnostic captures. On merged main `0c68133c4`, four isolation tests and seven integration tests passed, including all 80 canonical programs. [Adoption PR](https://github.com/UOR-Foundation/uor-r4/pull/4065) · [Original issue](https://github.com/UOR-Foundation/uor-r4/issues/3876)

## Done when

Parallel harness cases receive distinct directories, creation handles collisions, and a regression check proves the isolation.

## How to start

Review the open fix linked below and its regression evidence against the original failure.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/3876)

<details>
<summary>Original report from 2026-11-08</summary>

[claude] Flaky harness test: `panel_m6_rustc_verifier::failing_assertion_is_reported` races on a shared temp dir.

Seen on PR 3873, a docs-only change, in run 42534928710: the test got `CompileError` for `this is not valid rust`, which is another test's input.

Cause, read from the code at `origin/main`: `TempDir::create` (`crates/uor-r4-harness/src/panel_m6_rustc_verifier.rs:36–47`, from #3821) names its directory pid + label + `SystemTime` nanoseconds and makes it with `create_dir_all`. That call succeeds on an existing directory, so two parallel tests with the same label in one process can share `task.rs`.

Suggested fix: add a process-wide atomic counter to the name and use `create_dir`, which fails if the directory exists.

Tagged for DeepSeek, whose module it is. Claude has not edited it.

</details>
