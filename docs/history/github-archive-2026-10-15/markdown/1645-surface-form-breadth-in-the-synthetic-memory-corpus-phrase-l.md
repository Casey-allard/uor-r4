# #1645 — Surface-form breadth in the synthetic memory corpus (phrase lock-in instrument)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T04:58:06Z
- **Updated:** 2026-10-09T13:17:11Z
- **Closed:** 2026-10-03T12:32:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1645
- **Merged:** 2026-10-03T12:32:52Z
- **Merge commit:** a621514c21b429f36f0115611deeee8c18178b68
- **Base / Head:** main / UOR-Foundation:codex/copy-fidelity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Measurement complete; reviewable. Refs #1512, #820.

## Question

Binding was measured to generalise from 10 to 76 trained relations but to break
on a one-letter change of a relation phrase and on a question about a fact just
stated. Hypothesis: binding is keyed to the trained *surface phrase*, so training
many surface forms per relation should generalise to unseen *forms* the way 76
relations generalised to unseen relations.

## Instrument

`crates/uor-r4-training/src/bin/synthetic-memory-corpus.rs`, extended on top of
#1641's 76-relation table:

* `forms=N` — surface forms per relation. Index 0 is the canonical phrase,
  1..7 are paraphrase question and statement templates over a hand-authored
  per-relation topic, and the reply keeps the single canonical answer form so
  the output shape is held fixed between arms.
* Indices 8..12 are **held out**: `forms` is clamped to 1..=8, so the draw index
  never reaches the tail (`forms=13` is refused).
* `panel_out=DIR` writes the held-out-form panel: 464 rows, 74 relations x
  (`seen`, `seen-comp`, `unseen-state`, `unseen-ask`, `unseen`, `unseen-comp`)
  plus 20 `perturb-ask` rows. The competing rows reuse their twin's exact turns,
  so competition is the only difference.
* Startup validation of the hand-authored slot/value tables.

Scripts: `copyfid-run.sh` (chain), `copyfid-score.py` (strict/lenient, Wilson
CIs, per-condition exact McNemar, distractor-copy and degeneracy rates),
`copyfid-analyse.py` (per-held-out-template breakdown), `copyfid-split-panel.py`
(`lut-chat` refuses a panel above 128 requests; the panel is run as four chunks
and scored as a union), `copyfid-post-train.sh`.

**Byte identity of the fixed-form path:** `forms=1 rows=30000 seed=17` gives
`tokens.u16` `9bae7202ca4a9455...` and `response_mask.u8` `ed15cb21a948c7e3...`,
the hashes of `wide-memory-corpus2` — the store the existing wide-relation arm
trained on, independently reproduced by the pre-change wide generator built in
another lane.

## Arms (matched: init, steps 2500, batch 16, lr 2e-4, warmup, min_lr, clip,
weight decay, data/dev seeds, `full_prefix`, 2 threads, ~2% synthetic share)

| | F0 | F1 |
|---|---|---|
| forms per relation | 1 | 8 |
| synthetic store | `wide-memory-corpus2` 30,000 rows / 1,717,822 tokens | `copyfid-f1-corpus` 25,991 rows / 1,723,669 tokens |
| arm | existing `wide-bind-1` (settings identical) | new `copyfid-f1-arm` |

Control: the shared init (`chat-dose-continue-1`), which never trained on the
wide relations.

## Results (strict = value as a standalone phrase; n = 74 per cell)

| condition | F0 | F1 | dose control | F0 vs F1 McNemar p |
|---|---|---|---|---|
| seen | 74/74 = 100% | 73/74 = 98.6% | 22/74 = 29.7% | 1 |
| seen-comp | 72/74 = 97.3% | 71/74 = 95.9% | 18/74 = 24.3% | 1 |
| unseen-state | 28/74 = 37.8% | 70/74 = 94.6% | 15/74 = 20.3% | **3.1e-11** |
| unseen-ask | 43/74 = 58.1% | 47/74 = 63.5% | 18/74 = 24.3% | 0.54 |
| unseen (both) | 35/74 = 47.3% | 44/74 = 59.5% | 14/74 = 18.9% | 0.078 |
| unseen-comp | 34/74 = 45.9% | 39/74 = 52.7% | 10/74 = 13.5% | 0.44 |
| perturb-ask (n=20) | 19/20 = 95% | 18/20 = 90% | 11/20 = 55% | 1 |

Held-out *relations* (the three standard panels): F0 31/120 = 25.8%, F1
22/120 = 18.3% (pooled exact McNemar p = 0.16).

## Verdict

* Surface-form breadth **does** generalise, decisively, on the statement axis:
  37.8% -> 94.6%.
* It does **not** fix the question form: 58.1% -> 63.5% (not resolved), even
  though F1 trained on seven paraphrase question forms per relation. The
  residual binding failure is request-form-specific.
* The lock is a **frame** lock, not a string lock: one-word substitutions of the
  trained question cost ~nothing.
* Form breadth is **not free** at a fixed token budget: per-form repetition falls
  from ~395 to ~43 and unseen-relation transfer does not improve.

## Limits, stated

Single seed per arm; rows cluster by relation so the Wilson intervals are
optimistic for relation-level effects (the 3.1e-11 result is far outside that,
the 0.54/0.078 ones are not). F1's statement variety is the same core frame
("my {t} is {v}") under different prefixes, so its gain is prefix
generalisation, not paraphrase-invariance in general. ~40 relations whose
canonical sentence carries a verb are restated at topic level rather than
strictly paraphrased. The `seen` cell cannot separate in-context reading from
weight-level phrase -> relation association. Greedy decoding only. The 14/120
figure previously quoted for the same arm and panels is not reproduced under
strict (31/120), lenient (32/120), <=12-word (29/120), <=8-word (22/120) or
exact-reply (0/120) criteria; the difference is flagged, not reconciled.

Full report: `docs/integration/surface-form-breadth-2026-10-03.md`.

## Relation to other open work

* #1641 supplies the 76-relation table and `drops`/`files` metadata (taken
  verbatim as the base; that branch is not on `main` yet).
* #1642 edits the **same file** with a `distance` feature, so the two conflict
  textually; nothing else from it is imported.
* #1640 is the `main` revision this branch is based on.

## Merge with #1642 (distance curriculum) — both invariants verified

Rebased onto `49829d05` (#1642). The two knob families now live in one
`document()`: one form index is drawn per selection site in the order the narrow
generator used, so `forms=1` adds `below(1)` draws that leave the rng stream
untouched, and the filler turns sit between the statement pair and the question
exactly as #1642 wrote them. Five conflict blocks were resolved by combining
both sides; no side was dropped.

| args (`rows=64 seed=17`) | tokens.u16 | response_mask.u8 | against main |
|---|---|---|---|
| `distance=0 forms=1` | `380ebdaf690283cd…` | `c53704a4026838ca…` | **byte-identical** to main `distance=0` |
| `distance=3 forms=1` | `4722a1f03f0caf38…` | `bd1deb4530c8853d…` | **byte-identical** to main `distance=3` |
| `curriculum=1 forms=1` | `38e9e39c0f15a534…` | `32ca23a45da7def1…` | **byte-identical** to main `curriculum=1` |
| `distance=0 forms=8` | `8132103eaa929433…` | `d966c51d54d9d59f…` | paraphrases drawn, no held-out form |
| `curriculum=1 forms=8` | `de7c8ecd0e63883b…` | `3d40d4a5e699ed95…` | fillers and paraphrases together |

**Published reference store:** `rows=30000 seed=17 distance=0 forms=1` gives
tokens `9bae7202ca4a94552b8894739e7f001fb8967d28da79ed04e99c3bc524c703fd` and
mask `ed15cb21a948c7e308609ada9f352b3d7cf079609818ee9b80274174d5f2fa41`,
unchanged.

Content check (decoded from the stores, not inferred from the options): at
`distance=3` three filler turns sit between the fact and the question; at
`forms=8` the fact is stated as "By the way, my market is St George." and asked
as "Who's my teacher?"; at `forms=1` both are canonical. `panel_out=` still
writes 464 rows over seven conditions.

Checks run: `cargo fmt --check` clean, the bin builds, and the seven focused
tests pass (four from #1642 — fillers assert no relation value, filler
determinism, distance 0 adds no turns, distance refusals — plus three for the
forms invariants: canonical-only at `forms=1`, paraphrases but never the
held-out tail at `forms=8`, panel completeness and self-consistency).

Known cosmetic issue, deliberately not changed by this merge: `{q}` renders
capitalised inside a sentence ("Can you tell me What my museum is?"). Fixing it
would change the `copyfid-f1-corpus` store the F1 arm and panel were measured
with, so it is left for a follow-up that retrains.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

