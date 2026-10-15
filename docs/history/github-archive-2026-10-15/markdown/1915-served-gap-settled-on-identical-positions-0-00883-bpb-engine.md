# #1915 — Served gap settled on identical positions: 0.00883 BPB, engine exact (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:39:55Z
- **Updated:** 2026-10-09T13:22:07Z
- **Closed:** 2026-10-08T14:41:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1915
- **Merged:** 2026-10-08T14:41:40Z
- **Merge commit:** a1882e62ddf6b90d5e338e7597e3cf0d24df9a16
- **Base / Head:** main / UOR-Foundation:deepseek/served-gap-settled
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Why this measurement was needed

Round 36 established the served artifact is **free-transport** and that the 0.05522 BPB gap had been assembled from numbers taken on **different position sets** — a 512-window float evaluation, a 64-window integer evaluation, and a `snap-evaluate` counterfactual in between. An independent review's inference that ~0.046 BPB was un-attributed integer arithmetic rested explicitly on that cross-set arithmetic and asked for this instead.

## The measurement — one identical position set, 24,576 targets

`lut-evaluate artifact=… model=… valid=… windows=64 reference=true`, chat-v0-p2/heldout (sha `e5f400b0…`):

| | nll | bits/byte |
|---|---:|---:|
| **float** — the model's own f32 kernels | 1.8171591 | 0.92394 |
| **reference** — its dequantized matrices, grid-code scalars and integer biases, f32 | 1.8345347 | 0.93277 |
| **integer** — the multiplier-free served engine | 1.8345360 | 0.93277 |

```
quantisation   float → reference      0.0174 nats    0.00883 BPB
engine         reference → integer    0.000001 nats  0.000001 BPB
total          float → integer        0.0174 nats    0.00884 BPB
```

## What it settles

1. **The integer engine is exact** — it reproduces its own dequantized reference to **one micro-nat per token**. There is no serving-arithmetic loss to find: not in activation fixed-point, not in the tables, norm, or l2 distance. The ~0.046 BPB hypothesis is **refuted**, and it was an artefact of comparing different position sets.
2. **The whole served cost is weight quantisation, 0.00883 BPB** — matching the round-20 rounding attribution (0.0173756 nats) exactly, which independently confirms both tools measure the same thing.
3. **The served artifact delivers 99.0 % of its own float model.** The earlier "0.05522 BPB served gap" was a **protocol artefact**, not a serving deficit. Its published decomposition into snap and quantisation terms is superseded; the real one is one line.

## The consequence

**There is no large serving headroom left.** Rounds 24–34 worked on the transport snap (0.03578 BPB credited — now known to be a float counterfactual the artifact never pays) and on quantiser arms (GPTQ vs RTN, 0.00376 BPB). Both were real quantities; neither decided the served number.

**What decides it is the model's own quality** — which is exactly where round 12's measurement (in-domain data **0.1592** vs size **0.0837** BPB) and round 35's 10× extension (214 M at 1.0966 losing to 7.16 M chat-only at 0.9607) already pointed.

Scope: one model, one artifact, one corpus, 64 windows, single deterministic run — **exactly matched on positions**, which is what makes this decisive where the earlier cross-set numbers were not. Evidence: `docs/evidence/served_gap_settled_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

