---
name: native-space-cycle
description: Harness pointer to the native UOR-R4 train/serve state and operator parity protocol.
version: 1
harness: 1
kind: research
applies-to: [.agents/skills/native-space-cycle/SKILL.md, CHARTER.md, AGENTS.md]
when: [model training, optimizer, carry state, memory state, serving artifact, train-serve parity]
requires: [uor-research-architect]
outputs: [a native-state implementation card or a bit-identical transition result]
forbidden: [treating this pointer as the protocol]
milestones: [m1, m2, m3, m4]
---

<!-- native-space-protocol: .agents/skills/native-space-cycle/SKILL.md -->

## When to use

Use before any unit that trains, updates, exports or serves model state.

## Do this

1. Read the authoritative protocol at
   `.agents/skills/native-space-cycle/SKILL.md`.
2. Read `CHARTER.md` §3.7, `D85`, and `docs/harness/native-space.json`.
3. Run `uor-harness check --only harness.native_space_contract`.
4. Keep the exact canonical marker from `native-space.json`; do not paraphrase it.

## Output

Either a native-state implementation card naming every carried field and shared operator, or
a bit-identical train/serve transition result. The pointer never substitutes for the source
protocol.
