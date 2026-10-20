# #1890 — The D11 serving engine is bit-exact, so the barrier is which model is served (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T09:44:01Z
- **Updated:** 2026-10-09T13:19:47Z
- **Closed:** 2026-10-08T09:45:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1890
- **Merged:** 2026-10-08T09:45:39Z
- **Merge commit:** 193085f7ec5785dc3d2e47e7fb0021d61d045b57
- **Base / Head:** main / UOR-Foundation:deepseek/d11-fidelity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Found on disk, never reported

`~/uor-r4-local/d11-l2/` holds a complete **D11 conversion and evaluation of a geometric stack** (source `geo-20m`, pattern `rrarrarr`, width 512, heads 8, ctx 384, read l2), produced 2026-10-03, with `bits_per_byte: null` in every record.

**3,072 targets on `tinystories-v2-valid`:**

| field | value |
|---|---|
| d10 nll | **1.4595371721399573** |
| d11 nll | **1.4595371721399573** |
| `d11_minus_d10_nll` | **0.0** |
| `top1_agreement` | **1.0** |
| `max_abs_logit_difference` | **0** |
| `positions_with_a_difference` | **0** |
| protocol | evenly spaced windows, fresh session per window, position by position, logits compared as integers |

**The multiplier-free D11 engine reproduces the reference engine exactly** — identical nll to sixteen digits, every top-1 identical, not one logit difference anywhere. There is no discretisation loss in the serving kernel to recover.

(A second conversion in the same directory, GPTQ, records `top1_agreement` **0.9379** over 24,576 targets — so the *quantisation choice* matters even though the engine is faithful.)

## What this settles

Rounds 3–4 decomposed a 3.27 BPB gap into 2.57 BPB of domain mismatch and 0.58 BPB of in-domain export loss. **Neither is the integer engine — that engine is exact.** Both are about *which model is exported*:

| | artifact | size | properties | chat BPB |
|---|---|---:|---|---:|
| served today | `native_geometric_prose_model.rgm` | 2.7 MB | 120-root features, TinyStories | 4.3885 |
| trained, idle | `ladder/runs/geo-8m-l2` | 28.6 MB | width 288, l2 read | **1.1199** |
| converted + proved | geo-20m D11 export | — | engine parity exact | — |

The project has, in three places on one laptop: **a model that scores 1.1199 bits/byte on real chat**, an **exporter proven to produce bit-identical multiplier-free artifacts**, and a **proven engine**. It has never joined the three on the chat distribution.

## The run that joins them (work card, next round)

- **Deliverable**: a served multiplier-free artifact for the chat distribution, evaluated with `d11-evaluate` on chat-v0-p2/heldout, reported in bits per byte.
- **Recipe**: train a stack with `qat=true` (docs are explicit that a model without a served representation exports only as that), `geometric-stack export`, then `geometric-stack d11-evaluate artifact=ROOT/model.lut valid=VALID.u16` — the exact path the 20M precedent above used.
- **Cost**: measured 0.09–0.10 s/step/GPU; a 300M-token chat-only 20M run ≈ **1.3 GPU-hours ≈ $1.30**, inside the owner's pod caps, no new approval class.
- **Expected**: within a few hundredths of the float run — order **1.1–1.2 BPB** against the 4.3885 served today.
- **Kill**: if `d11` differs from its own float reference beyond what the GPTQ row implies, or the run lands above 2.0 BPB, then chat is not reachable by this stack at this size and the next spend belongs to data and capacity, not serving.
- **Precedent**: the lab recorded a D11 integer export at 0.793384 BPB against 0.789108 for its QAT float parent (**0.005 BPB cost**).

Scope: everything read from artifacts on disk; nothing retrained. The 1.4595 nll is the geo-20m conversion on TinyStories and is cited for engine fidelity, not quality. Evidence: `docs/evidence/d11_serving_fidelity_2026-10-08.txt`.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

