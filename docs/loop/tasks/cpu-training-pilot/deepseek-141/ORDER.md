# ORDER: cpu-training-pilot, deepseek-141 (content-serving scope)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Base: `70f45367d`.

## Goal

Authorize the integer-serving files and evidence paths required by the
already-ordered deepseek-140 content-readout serving unit. This is a
code-neutral item-scope prerequisite: another branch may not widen the item
scope in the same PR that consumes the widened paths.

## Cancel probe (run first)

At base `70f45367d`, the item scope excludes every `crates/uor-r4-integer/**`
path used by deepseek-140, `docs/integration/current-state.md`, and the
content-serving evidence tree. The consuming branch is based on the same item,
so the separate scope prerequisite is still required.

## Fixed construction

Add exactly the files consumed by deepseek-140:

- `crates/uor-r4-integer/Cargo.toml`;
- the integer README, `uor-chat`, OpenAI loopback, session codec and stack
  content/session/chat sources;
- the OpenAI-loopback and session-codec integration tests;
- `docs/integration/current-state.md`; and
- `docs/labs/cpu-training-pilot-content-serving-2026-11-04/**`.

No runtime, test, artifact, evidence, status, bar or hold changes.

## Files in scope

- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-141/**`

## Done when

1. The item scope contains every path used by deepseek-140 while retaining all
   existing scope entries.
2. Roadmap and documentation checks pass.
3. Exact-head review, protected merge and fresh-main verification complete.

## Stop and ask if

- Any runtime, test or evidence file would change.
- The required scope would authorize an unrelated integer subsystem.
