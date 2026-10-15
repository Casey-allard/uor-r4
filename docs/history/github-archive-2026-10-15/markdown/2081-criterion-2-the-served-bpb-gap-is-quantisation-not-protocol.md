# #2081 — Criterion 2: the served-BPB gap is quantisation, not protocol (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T20:08:17Z
- **Updated:** 2026-10-09T20:20:29Z
- **Closed:** 2026-10-09T20:13:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2081
- **Merged:** 2026-10-09T20:13:34Z
- **Merge commit:** accc4a495d764fe441a3289f6affe2812d2ebcfe
- **Base / Head:** main / UOR-Foundation:bpb-split
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**CPU-only integer/LUT evaluation of existing artifacts. No training, no pod, no local float generation. Zero pod spend.**

Closes the question the criterion-2 arithmetic left open on [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029): is the ~0.024 BPB residual to the 0.90 target protocol, or float model quality?

## Finding

The recorded **float 0.877 / served 0.933 / quantisation 0.00883** position is a **cross-position-set comparison**. The 0.0552 BPB "served gap" is 0.0088 of quantisation measured on identical positions plus 0.0464 of pure position-set difference; the engine contributes ~1e-6 BPB, not the 0.0464 the [serving-mode audit](docs/evidence/serving_mode_audit_2026-10-08.txt) inferred from a cross-set subtraction (already refuted by [served_gap_settled](docs/evidence/served_gap_settled_2026-10-08.txt); this round reproduces that independently on another artifact at two window sizes and both protocols).

Three-way attribution on **identical positions**, artifact `sha256 10d8d60b…` (`transport_snap=None`), checkpoint `sha256 4ef5f38f…`, stream `e5f400b0…` (6,194,589 tokens, 2.837427 bytes/token, the sealed-league basis):

| windows / targets | float | reference | integer | quantisation | engine |
|---|---|---|---|---|---|
| 64 / 24,576 | 1.796884 | 1.800973 | 1.800973 | **0.004089 BPB** | −0.00000077 BPB |
| 512 / 196,608 | 1.767493 | 1.771344 | 1.771342 | **0.003841 BPB** | −0.0000043 nats |

`d11-evaluate`: `d11 = d10 = 3.5420708435`, `d11_minus_d10_nll 0.0`, `top1_agreement 1.0`, `max_abs_logit_difference 0`. The integer column is byte-reproducible across runs, with and without the float model loaded.

- **(a) quantisation — measured**, small and stable: 0.00409 ± 0.00141 BPB (64 w), 0.00384 ± 0.00049 BPB (512 w).
- **(b) protocol/engine — measured, and not in the serving path**: ≤ 1e-5 BPB everywhere. What is large is the *measurement's* position-set sensitivity — per-window float spread 0.50 BPB (64 w) / 0.44 BPB (512 w), and the two window sets on one model differ by 0.029 BPB. The quantisation term is ~50× less sensitive.
- **(c) float model quality — measured by elimination**: on the record's own matched table the float side is 0.92394 against a 0.90 target, so export work cannot close the gap alone.

**Decision:** criterion 2 is not met and cannot be met by export/quantiser work; the serving path is within 0.4 % of its own float model. The lever is the float model. Second decision: criterion 2 as written does not pin its protocol, and on this stream the same model moves further between window sets than the whole gap to target — a future criterion-2 claim should name the window count and the byte basis.

## Limitation, stated plainly

The artifact that produced the recorded 0.877/0.933 pair (`batch 32`, `lr 4e-4`, `chat-v0-p2` train only) is **not in the cloud-store index** and is not on the laptop (searched the full 466-entry index, the local checkouts and the filesystem). The measured artifact is the same-architecture term-weight 19.9 M family at a much worse absolute level (1.77–1.80 BPB), so this is a protocol/structure measurement; the record's own 0.00883 remains the comparable quantisation cell for its artifact.

## Delivery

- `docs/labs/chat-served-gap-attribution-2026-10-09/README.md` (plus `make_lens.py`, the byte-basis derivation)
- `docs/evidence/chat_served_gap_attribution_2026-10-09.txt`
- `docs/integration/current-state.md` newest-first entry ending in **Next:**
- STATUS/ROADMAP/#2028 unchanged — no served model, BPB headline or milestone moved.
- `python3 scripts/check_claim_wording.py` passes.

Cost: zero pod spend; ~60 min laptop, release build 4 m 31 s.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

