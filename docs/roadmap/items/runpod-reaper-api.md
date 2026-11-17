---
id: runpod-reaper-api
title: Preserve RunPod pod cleanup across the API migration
milestone: infra
area: tooling
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
scope: [scripts/pod/uor-reaper.sh, scripts/pod/tests/uor-reaper*, docs/roadmap/items/runpod-reaper-api.md, docs/loop/tasks/runpod-reaper-api/**]
---

## Why it matters

Preserve provider-side cleanup when the discovery and termination API changes.

## What is missing

The reaper still uses RunPod GraphQL for discovery and termination. The prior REST call refused the pod credential with HTTP 403. A replacement needs equivalent authorization and a verified cleanup path. The retirement date in the original report is a contributor-reported planning date. [Source uor-reaper.sh](https://github.com/UOR-Foundation/uor-r4/blob/main/scripts/pod/uor-reaper.sh)

## Done when

A fixture or test account confirms discovery, authorization, and idempotent termination for the reaper. Missing permissions and unsuccessful cleanup remain visible in its exit status.

## How to start

Read the existing GraphQL calls and verify the replacement endpoint and its accepted credential type against the provider before proposing a migration.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/3957)

<details>
<summary>Original report by michaelmanly on 2026-11-13</summary>

## Affected code
[scripts/pod/uor-reaper.sh](https://github.com/UOR-Foundation/uor-r4/blob/main/scripts/pod/uor-reaper.sh)

`uor-reaper.sh` posts directly to `https://api.runpod.io/graphql` during GPU pod cleanup. RunPod plans to retire GraphQL in early 2027.

RunPod migration guide: https://docs.runpod.io/api-reference-v2/migrate-from-v1

## Suggested fix
Migrate the reaper's pod discovery/termination to REST v2, protect credentials, and verify orphaned pods are still cleaned up.

**Disclosure:** I'm building [AI Badgr](https://aibadgr.com/) for GPU execution across providers. Happy to help verify the fix or run a small cost-capped GPU workload through Badgr for free, if useful. The technical fix is independent of Badgr.

</details>
