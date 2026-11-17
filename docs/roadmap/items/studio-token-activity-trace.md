---
id: studio-token-activity-trace
title: Expose a verified Studio token timeline and optional 3D activity view
milestone: M7
area: tooling
status: ready
open_to: lab
needs_gpu: false
depends_on: []
scope: [web/studio/assets/js/uor_model_worker.js, web/studio/assets/js/studio_ui.js, web/studio/verify_studio_worker.mjs, web/studio/src/adapters/generation*, web/studio/src/components/TaskTimeline*, web/studio/src/components/TimelineEvent.tsx, web/studio/src/components/TokenActivity*, web/studio/src/studio.css, web/studio/tests/e2e/cursor-tasks.spec.ts, docs/roadmap/items/studio-token-activity-trace.md, docs/loop/tasks/studio-token-activity-trace/**]
---

## Why it matters

A reader should be able to inspect what the running model actually emitted.

## What is missing

The task timeline reports browser workspace events. This item needs the tokens that the model worker emitted, their source and available margins or memory fields. A provider can expose only the fields it actually returns. [Issue 3572](https://github.com/UOR-Foundation/uor-r4/issues/3572) · [PR 3915](https://github.com/UOR-Foundation/uor-r4/pull/3915)

## Done when

Show events emitted by the worker with real token metadata in an accessible timeline, default to metadata-only export, and supply reduced-motion and 2D alternatives to the optional activity view. Preserve the original limits on a future detailed engine trace.

## How to start

Inventory worker events and fields that are available at the supported worker boundary before adding a view.

## Rules that apply

The project's evidence and contribution rules apply. [Charter](../../../CHARTER.md) · [Contributor guide](../../../CONTRIBUTING.md)

## Sources

[Original issue and discussion](https://github.com/UOR-Foundation/uor-r4/issues/3572)

<details>
<summary>Original report from 2026-11-02</summary>

Lab: kimi. Owner-requested follow-up, separate from #3500 and #3569.

Build an auditable token timeline from real worker events, then an optional self-hosted 3D UOR activity visualization. Use actual token IDs/text, margins, generation/source identity, artifact SHA-256, and memory metadata. Do not simulate model thoughts, UOR activity for external APIs, or unavailable connections.

Initial scope: real existing worker events, a native accessible timeline, explicit metadata-only export defaults, and a reduced-motion/2D fallback. Three.js may be pinned and self-hosted for the optional activity view; it is not an app framework.

Richer actual address paths require a separate read-only engine-trace decision with bounded allocation and measured overhead. No engine change, production publish, OpenTelemetry content-hash execution IDs, or model-capability claim is part of this initial item. Execution IDs must be random; artifact hashes remain metadata. References #3569 without closing it.

</details>
