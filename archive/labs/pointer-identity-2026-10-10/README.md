# Token-identity pointer: the term reaches the failure mode (wrong-value 17 → 12) and still leaves memory at 19/40 — the line is archived under the 3/3 pivot

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek, session `deepseek/ptr-identity`, **cycle 4** of
the standing goal adopted from `docs/labs/session-goal.md` @ `8421e605f…`, and **the decisive run named by the
3/3 pivot card**. Run 2026-10-10 13:09–13:21 UTC. **One pod, 2 × RTX 5090, ~12 minutes, ≈ $0.55**;
acceptance scoring on the owner's laptop CPU.

> **One line.** The pivot named the one mechanism this line had never trained — a **token-identity pointer**,
> which scores a source by the multiset overlap of the six tokens before it and the six before the query, so
> "copy the same token" no longer has to be learned by a 32-wide projection of hidden states. It is real and
> it is aimed correctly: the **wrong-value failures fall 17 → 16 → 12 of 40** across `pointer_identity=0.25`
> and `1.0`, the best binding result this line has produced. And the **bar is still not met**: v5 memory
> `check_pass` is **19/40** on both arms against the ≥ 21/40 the pivot required, so **the line is archived**,
> as the pivot card declared it would be, and M1's next piece is the addressed-memory operator the panel has
> never had.

## The pre-registration, and what was actually run

[Pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097793586)
before any compute: bar, base artifact, arms, split, stop rule and cost, with the archive rule carried from
the [pivot card](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097789124).

| | |
|---|---|
| base artifact | `chat-29m-B-lr5e-4` `d8a3c971…`, its own run `report.json` as `init=` provenance |
| the one change | **`pointer_identity=WEIGHT`** — the multiset overlap of the six tokens before a source with the six before the query, mixed into the pointer's dot score; **no parameters**, refused only with `pointer_score=lorentz` or a route, neither of which applies |
| mixture | cycle 1's `mix-10` store, **unchanged** (`tokens.u16` `888241261c378138…`), so the identity term is the only difference from the anchor |
| arms | **I25 = 0.25** (recorded as `identity.weight_bp: 2500`) and **I100 = 1.0** (`10000`); the matched anchor is cycle 1's **D10** (`pointer_identity=none`, same mixture, seed and steps) |
| steps / stop rule | `steps=2000 batch=16 lr=2e-4 warmup=100 data_seed=20261009 policy=full_prefix context=384 device=cuda`, `eval_every=250 checkpoint_every=500`, `max_seconds=3600`; each arm ≈ 105 s |
| pod | `gvos14qt7mbwrp`, `--ref 020a13fe7c…`, CUDA parity PASS; lease released and pod deleted at 13:22 UTC |
| the term is confirmed active | both arms' saved configs record it — `{"dim": 32, "identity": {"weight_bp": 2500}, "init_seed": 1}` and `{"…": 10000}` — rather than being assumed from the flag |

## Results

### The mechanism the pivot named, measured by its own failure mode
| arm | `pointer_identity` | v5 memory `check_pass` | failures: **wrong-value** / distractor-key / no-other | `dev_response_nll` | dev pointer hit rate |
|---|---:|---:|---|---:|---:|
| D10 (anchor, `none`) | — | **20/40** | **17** / 0 / 3 | 0.3773 | 0.377 |
| **I25** | **0.25** | **19/40** | **16** / 0 / 5 | 0.3671 | 0.287 |
| **I100** | **1.0** | **19/40** | **12** / 1 / 8 | 0.4666 | **0.003** |

**The identity term does what the pivot said it would do to the failure mode and still does not move the
panel.** Wrong-value rows fall from 17 to 16 and 12 — the best binding reading on this panel — but the total
stays at 19/40, because the failures redistribute into the "no value / other" bucket (3 → 5 → 8). At
`pointer_identity=1.0` the pure identity score also collapses the pointer's dev hit rate to **0.003**, i.e.
a source whose six-token context matches the query's is usually *not* the source that holds the answer — a
direct measurement of why the term alone cannot carry the pointer.

### Frozen v5 acceptance panel — official `chat-grade grade-replies`
**I25** (`pointer_identity=0.25`): memory `check_pass` **19/40**, unknowable **0/24**, panel 19/64,
`acceptable` 10/64, derangement **0**. **I100** (`pointer_identity=1.0`): memory **19/40**, unknowable
**5/24**, panel 24/64, `acceptable` 10/64, derangement 1. Both arms therefore land one row under the
anchor's 20/40 and two under the bar.

### Open reply panel (the guard), the primary arm — pending, declared
The pre-registration named the reply panel for the primary arm and said anything the shared judge does not
reach is reported **pending, never guessed**. I25's 232 replies are generating as this record is written; the
number will be posted on #2029 and carried into the next cycle's bundle. It cannot change the archive verdict,
which the memory half decides (19/40 against a 21/40 bar).

## Decision

**ARCHIVE — the bar is not met, so this line closes exactly as the pivot card declared, and M1's next piece is
the addressed-memory operator.**

- The pre-registered bar (**memory ≥ 21/40 and reply ≥ 29/232 at the same time**) is not met: **both arms are
  officially graded at 19/40** (I25 and I100). The secondary mechanism threshold (**wrong-value < 12 of 40**)
  is also missed, by one row on I100 and by four on I25 — so the term counts as an improvement in the failure
  mode and not as a solution.
- **The line is archived under D21 §1.** Four cycles, four merged result PRs, one measured lever and one bar
  never met:

  | cycle | PR | lever | v5 memory | reply `f_and_r` |
  |---|---|---|---:|---:|
  | 1 | [#2151](https://github.com/UOR-Foundation/uor-r4/pull/2151) | recall mixture, dose 0/10/25/55 % | 5 → **20** → 20 → 22 of 40 | 36 → **37** → 28 → 22 of 232 |
  | 2 | [#2155](https://github.com/UOR-Foundation/uor-r4/pull/2155) | read-binding supervision | 19 / 17 of 40 | 39 of 232 |
  | 3 | [#2158](https://github.com/UOR-Foundation/uor-r4/pull/2158) | binding-dense mixture (`generator=v1`) | 17 / 17 of 40 | pending |
  | 4 | this record | token-identity pointer | 19 / 19 of 40 | pending |

  No source is deactivated by the archive — every arm was trained with options the trainer already carries, and
  the trainer, base checkpoint and panels are untouched — so the archive is the record plus the INDEX row, and
  the line's artefacts remain reproducible from the commands in the four records.
- **What M1 does next, from the pivot card:** an **addressed-memory operator in the dialogue stack**.
  `dialogue-train` cannot build one today — `DialogueSettings` carries no memory fields while the LM trainer
  exposes `memory_layers=`/`memory_sub_keys=`/`memory_top_k=`/`memory_score=`/`memory_codebook=` — so every
  artifact behind the numbers above answers rows named `multi_turn_memory` with a **copy pointer and no memory
  reader**. The first step of that piece is source work, not data.
- **Criterion 1 remains NOT MET on both halves** and is not claimed: memory 19/40 against ≥ 34/40, reply
  pending against ≥ 116/232. The base artifact's 10/40 and 43/232 are unchanged, and the base
  artifact remains the base.

## Limitations

- **One seed per arm** on a 40-row memory panel: 19 vs 20 is one row, and the wrong-value counts (17 → 16 → 12)
  are the row-paired signal, which moved in the intended direction.
- **`pointer_identity=1.0` is not a usable pointer** (dev hit rate 0.003); it is reported as the dose–response
  endpoint, not as a candidate.
- **The reply-panel guard covers the primary arm only** and follows the shared-judge ordering declared in the
  pre-registration; anything unreached is named pending rather than guessed.
- **The archive is a research-line closure, not a source disposition**: nothing is removed, deactivated or
  deprecated in the trainer, and the four records' commands remain runnable.
- Nothing here is evidence about addressed memory; that is precisely what the next piece builds.

## Cost

| | |
|---|---|
| pod | `gvos14qt7mbwrp`, 2 × RTX 5090, 13:09Z → 13:22Z (~13 min incl. bootstrap, ≈ $0.50); lease released, pod deleted |
| GPU work | two arms × ~105 s = 3.5 min (I25 and I100 ran concurrently on the two GPUs) |
| laptop CPU | v5 replies 2 × ~2 min; v5 grading 2 × ~12 min; failure-cause analysis seconds; reply-panel replies + grading for I25 on the shared judge |
| external | none; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports, configs and curves: pod volume `/workspace/uor-r4/deepseek/pointer-ft-20261009/runs5/ident-{I25,I100}/`,
  pulled to the laptop and bundled.
- Acceptance reports: `score/v5g-{I25,I100}/report.json`, `score/v5-{I25,I100}/replies.json`, reply panel
  `score/rpg-I25/report.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/pointer-identity-2026-10-10.tar`, **756,736 bytes,
  md5 `f366ee2d4099b05ac7d3b1728f933e82`** (object `pointer-identity-2026-10-10`, uploaded and MD5-verified
  by `cloud-store put`): both arm reports and configs (which record `identity.weight_bp` 2500 and 10000),
  both official v5 acceptance reports and reply files, the primary arm's reply file, the mixture manifest,
  the failure-cause script and the scoring logs.
- Pre-registration and pivot: [#2029 comment 6097793586](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097793586)
  (pre-registration), [6097789124](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097789124)
  (the 3/3 pivot card this run answers).
- The closed line's four records: [mixture-dose-2026-10-10](../mixture-dose-2026-10-10/README.md),
  [read-binding-2026-10-10](../read-binding-2026-10-10/README.md),
  [binding-dense-2026-10-10](../binding-dense-2026-10-10/README.md), and this one.
