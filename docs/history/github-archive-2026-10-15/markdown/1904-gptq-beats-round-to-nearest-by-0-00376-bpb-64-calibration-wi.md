# #1904 — GPTQ beats round-to-nearest by 0.00376 BPB; 64 calibration windows is an interior optimum (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T13:00:03Z
- **Updated:** 2026-10-09T13:21:48Z
- **Closed:** 2026-10-08T13:01:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1904
- **Merged:** 2026-10-08T13:01:42Z
- **Merge commit:** 0da19684e1ff13ab78c477900852d6683ef8a31e
- **Base / Head:** main / UOR-Foundation:deepseek/quantiser-verdict
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The controlled comparison

Same 19.9 M chat stack, same held-out stream (`e5f400b0…`), same integer evaluation (`d11-evaluate`, 24,576 targets, engine top-1 agreement **1.0 in every arm**). Only the export's quantisation differs:

| arm | export record | d11 nll | bits/byte |
|---|---|---:|---:|
| round-to-nearest (no calibration) | `round_to_nearest` | 1.8419342517773867 | 0.93653 |
| **GPTQ, 64 windows, damp 0.01** | `gptq` | **1.8345359993679817** | **0.93277** |
| GPTQ, 256 windows, damp 0.001 | `gptq` | 1.836327732755273 | 0.93368 |

**GPTQ wins by 0.00376 BPB.**

## What it settles

1. **The export path's default is the right one.** GPTQ beats round-to-nearest, so the calibration-bearing export every artifact tonight used is buying real fidelity — and round 21's "try GPTQ" proposal was chasing something already in use.
2. **The calibration setting is an interior optimum, not a floor.** Both directions are worse: 4× the windows with 10× finer damping loses 0.00091 BPB, and removing calibration loses 0.00376. The default sits *between* two measured-worse options — a stronger statement than "the default is fine".
3. The quantisation term of the served gap is **0.01944 BPB at its measured optimum**, and round 20 shows 97 % of it is the weight matrices.

## The served gap, final form for this model

```
fused float reference      0.87755
transport snap             +0.03578   (64.8 %)
quantisation (GPTQ)        +0.01944   (35.2 %, 97 % of it weight matrices)
served                      0.93277
```

**The remaining recoverable term is the transport snap** — a representation question, not an export setting — and the quantisation term now sits at the optimum of both quantisers this repository has.

Scope: one model, one corpus, one stream, one export per arm, deterministic integer evaluation; arms differ only in quantisation method and calibration parameters. Evidence: `docs/evidence/quantiser_arm_verdict_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

