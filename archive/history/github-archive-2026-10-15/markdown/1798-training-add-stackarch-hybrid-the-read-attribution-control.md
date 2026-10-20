# #1798 — training: add StackArch::Hybrid, the read-attribution control

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T14:23:59Z
- **Updated:** 2026-10-09T13:20:33Z
- **Closed:** 2026-10-06T14:44:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1798
- **Merged:** 2026-10-06T14:44:02Z
- **Merge commit:** 4cee4a4b802f21daee28509633761418bc756b2e
- **Base / Head:** main / UOR-Foundation:deepseek/hybrid-attrib
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Adds `StackArch::Hybrid`: the same `pattern=` as the geometric stack, but every **`a` read layer runs ordinary causal attention** instead of the fused geometric read. The `r` recurrence layers are unchanged.

**One enum variant, five match arms, three parser lines** — at exactly the three sites `cargo check` reports as non-exhaustive. No behaviour change for `Geometric` or `Transformer`.

## Why

The D20 §2 verdict (#820) could only be stated as *geometric stack versus ordinary attention*: `rrarra` bundles four recurrence layers that read no context, so nothing attributed the effect to the **geometric read**. The named missing component was *"an arm with `rrarra`'s pattern but ordinary attention reads — the only experiment that attributes the effect; it does not exist."*

## Result — three seeds, all sealed, lr=3e-4, `pattern=rrarra`, 1800 steps

| configuration | in-class per seed | mean | variance |
|---|---|---|---|
| attention reads only (C6 matched) | 0.253 / 0.251 / 0.256 | 0.2534 | 7.5e-6 |
| **recurrence + ATTENTION reads (this PR)** | **0.0107 / 0.7207 / 0.9829** | **0.5714** | **0.2530** |
| recurrence + GEOMETRIC reads | 0.9980 / 1.0000 / 0.9995 | 0.9992 | **1.03e-6** |

Held-out: hybrid **0.0000 / 0.5940 / 0.9590**; geometric 0.9951 / 0.9990 / 0.9990; control ~0.0007. All runs `stopped_early_at_max_seconds FALSE` with 460,800 supervised queries.

**Both components contribute, and the geometric read's distinctive contribution is reliability:**

1. **The recurrence layers add capability** — hybrid mean **0.5714** vs the attention-only control's **0.2534**.
2. **Attention reads can nearly reach the ceiling** — hybrid max **0.9829** vs the geometric arm's **0.9992** — when a seed cooperates.
3. **The geometric read eliminates the seed lottery** — at lr=3e-4 the geometric arm's variance is **1.03e-6** while the hybrid's is **0.2530**, spanning **0.0107 to 0.9829**. The hybrid is bimodal at the very rate at which the geometric arm is not.

Not "ordinary attention cannot solve this task" — rather, **with attention reads whether a seed solves is a lottery; with the geometric read at the corrected rate it is not.**

## Scope and honesty

Two earlier claims in #820 were retracted after seeding out: *"the attribution is definitive"* and *"the recurrence layers carry nothing."* Both came from **n=1** (hybrid seed 1, which happened to sit at chance). This PR's body supersedes them.

## Instrument note

`probe_steps>none` is rejected for the hybrid — `Invalid("read binding needs a declared geometric read layer and head")` — which is **correct**: the binding requires a geometric read, and a hybrid has none by construction. Its condition-3 evidence is the arch-symmetric `context_reachability` counter, the same position the ordinary-attention control occupies.

## Checks

- `cargo check -p uor-r4-training --offline` — clean
- `cargo check -p uor-r4-training --example mqar-bench --offline` — clean
- three seeds × 1800 steps sealed at lr=3e-4

Additive; nothing retired or relabelled; D20 §3 not invoked. Roots: `~/uor-r4-local/mqar-bench/hybrid-attrib-20261006-1010/hybrid-s{1,2,3}`.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

