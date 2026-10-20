# Card P7 — Per-mechanism falsification sweep (ablation-first)

Owner (human): Casey        Drafted by: Zed (agent)        Date: 2026-09-19
Signed:

## Hypothesis (one sentence, falsifiable)

Each mechanism currently contributing to the served score — VSA attention, engram,
hierarchical lattice (coarse and fine), JEPA state prediction, lane tables, S2 readout,
and the hand-coded induction term — changes held-out bits-per-byte by more than its
matched-control noise band when ablated in isolation; mechanisms that do not are inert.

## Why this and not something else

Links to `native-core-transition-plan.md` §2 and decision D1. The project has never
measured the per-mechanism contribution to held-out BPB. Because the artifact format
already gates optional mechanisms behind `flags` bits (`binary_model.rs:63-72`) and one
scalar (`vsa_scale_q15`, used at `binary_model.rs:1450`), the sweep requires **no
training** and can be run today. It is the cheapest work that can change the plan, and it
is a precondition for Stages 3–6.

It also resolves two specific open questions raised by source inspection:

- the VSA heads bind a **fixed random** token codebook (`vsa/codebook.rs`) that is
  disconnected from the learned 120-root assignment (`jepa_trainer.rs:1119`), so their
  expected contribution is ≈ 0;
- the reported BPB is computed by a *different* scorer (`evaluate_bpb_with_engram`,
  `jepa_trainer.rs:2216`) than the served output (`score_and_select_candidate`), so the
  headline number currently does not measure the product.

## Data

Read-only. The existing held-out split (first 64,000 tokens of
`tinystories_train.u16`, byte denominator from `tokenizer.token_byte_lengths()`), already
**opened** and therefore development evidence. No fresh draw; a fresh independent draw is
reserved for after design selection. Corpus: TinyStories-V2, tokenizer vocab 4096.

## Matched controls

Each ablation is a within-artifact disable, not a refit, and each is compared against:

1. `none` — the unmodified artifact (baseline);
2. a **magnitude-matched random replacement** for the ablated component where a refit is
   needed (P4 geometry controls: random phases, random token→root assignment);
3. the same-corpus Kneser-Ney 5-gram baseline already implemented in
   `train_native_prose.rs:48`.

Ablations: `vsa`, `engram`, `lattice`, `lattice_coarse`, `lattice_fine`, `jepa`, `lanes`,
`s2_readout`, `induction`.

## Primary metric + threshold

Held-out **bits-per-byte** under (a) the existing probability scorer and (b) an added
serving-scorer path, reported side by side. Threshold: a mechanism is **retained** only
if its ablation raises BPB by more than the bootstrap 95% CI of the `none` baseline;
otherwise it is **retired from the critical path**.

## Secondary metrics

Tokens/sec; µs/token; candidates scored per token; learned-store bytes read per token
(I1–I3); op census (I4); allocation count on the hot path.

## Budget

Engineering: ≤ 3 days. Machine: ≤ 2 h (evaluation only). Storage: ≤ 1 GB (derived
artifacts and receipts). Wall-clock cap: 3 h.

## Kill criterion (pre-registered)

If `vsa` and `jepa` ablations each move BPB by less than the noise band, both are retired,
Stage 3's codebook-consistency work is abandoned, and the plan proceeds with Stage 4
(nonlinearity) and Stage 5 (low-bit core) only. No redesign extends this card.

## Preservation

All 9,984 historical verification cases and the 7,680 retained traces must reproduce
bit-exactly under `none`. The existing artifact, its digest, and every prior result record
are preserved unchanged. Derived ablations are written to their own receipt directory and
never beneath a sealed root.

## Deliverables

- One row per ablation in `docs/integration/EVIDENCE.md` with the receipt path.
- `cards/P7-falsification-sweep-RESULT.md` with the BPB table, CIs and the
  serving-scorer vs probability-scorer comparison.
- An `E1.4` recall-attribution table: longest exact corpus match and ≥k-gram coverage of
  generated output.
- Replacement controlled tests for `test_head_1_induction_circuit` and
  `test_multi_head_roles_orthogonality`, plus the two provable checks
  (`hamming_equals_angle_table`, `byte_to_root_reachable`).
