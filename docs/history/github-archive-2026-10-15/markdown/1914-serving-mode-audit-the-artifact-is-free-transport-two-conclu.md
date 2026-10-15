# #1914 — Serving-mode audit: the artifact is free-transport; two conclusions withdrawn, headroom re-aimed (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T14:30:20Z
- **Updated:** 2026-10-09T13:22:11Z
- **Closed:** 2026-10-08T14:32:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1914
- **Merged:** 2026-10-08T14:32:09Z
- **Merge commit:** cc095f463a6cfc244891df213fef9c4c127dbf06
- **Base / Head:** main / UOR-Foundation:deepseek/serving-mode-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## How this arose

An independent review of the transport-snap question reported — **from reading the export and D10 format source rather than the numbers** — that the artifact behind the served result might not be snapped at all. The check is one command that had never been run.

## The check, and its result

Reading the header record out of each artifact (MAGIC 8 B, u64 LE header length, JSON header):

| artifact | served score | `transport_snap` in header |
|---|---:|---|
| round-16 `export-a1` (sha `2ac4c306…`) | **0.93277** | **ABSENT — free transport** |
| round-29/30 `export-f1` (500-step tune) | 0.95150 | `{"name":"icosian","roots":120}` |
| round-32/33 `export-f2` (250-step tune) | 0.95540 | `{"name":"icosian","roots":120}` |

The supporting code facts hold too: export records only the snap the model directory declares, and the D10 read path **refuses** any artifact whose header carries one — which is why `rounding-attribution` succeeded on the round-16 artifact at all, and why `d10` is present in the round-16 record but null in the fine-tuned ones.

## Correction 1 — the served gap has no snap term

The artifact scoring 0.93277 is served **free-transport**, and its own free float parent scores 0.87755. So the 0.05522 BPB gap is entirely within free transport:

```
weight rounding        0.0088 BPB   (round-20 attribution's 0.0173756 nats)
un-attributed integer  0.0464 BPB   (activations, norm, l2, tables)
```

The **0.03578 BPB "transport snap" term is a float counterfactual**, and the 0.01944 BPB "quantisation" figure was a **cross-mode difference**, not a quantisation measurement. `served_fidelity_decomposition_2026-10-08.txt` is marked superseded.

## Correction 2 — rounds 30 and 33 compared across serving modes

The baseline is free-served; both fine-tuned arms are snapped-served. Their apparent regression and the **"quantisation damage is a basin effect" conclusion are withdrawn**, pending a specified matched-mode control: export the same untuned weights *snapped* and score it. Near 0.93277 → basin effect real; near 0.95 → rounds 30/33 were measuring the serving mode throughout. `snapped_tune_sweep_2026-10-08.txt` is marked superseded in part.

## What survives

The **served result stands — 0.93277 vs 4.3885 served today (4.70×)**, reproduced to 2.1e-6. The data-vs-size finding stands (and was extended: 214 M at 1.0966 vs 7.16 M chat-only at 0.9607). The quantiser finding stands within its own mode (GPTQ > RTN by 0.00376).

## What it re-aims

**The largest term is now the un-attributed fixed-point serving arithmetic (~0.046 BPB)** — an order of magnitude more than weight rounding, and *not* the transport representation rounds 24–34 pursued. The recommended first measurement is `lut-evaluate … reference=true`, which reports float, artifact-in-float and integer NLL **on one identical position set** — the form every cross-mode number tonight lacked.

## Also landed: the data lever's route

`chat-v1-p2-train` (**1,388,814,239 tokens**) is **rebuildable** — tool on `main`, recipe in the runbook, independent arithmetic check agreeing to **0.05 %** — ranked against five alternatives, with the store confirmed corpus-free, every local chat store shown to be the same 82–83 M tokens, and no archive holding it. The review's own **strongest argument against buying it** (chat-v1 scales volume, not distribution) and the **cheaper dose-response test** that should precede the spend are both recorded, along with the single owner action that would unblock it.

## Process note, because it recurred

The snap term was built from a float counterfactual and treated as part of the served gap without checking which transport the artifact used. **The same pattern produced the round-29 float-gain error.** Both were caught by an independent reader checking the source — not by the numbers agreeing with themselves.

Evidence: `docs/evidence/serving_mode_audit_2026-10-08.txt`, `chat_corpus_route_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

