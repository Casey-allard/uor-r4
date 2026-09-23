# The ordinary-prose deficit was the training curriculum, not the objective — September 23, 2026

This record executes the milestone that the [audit of September 23](ordinary-lexical-audit-2026-09-23.md)
recorded as its one evidence-supported next step: *"train the retained `TlModel` on the per-position,
full-context objective with the larger target batch while keeping the grounded Copy/Stop supervision,
confirm on open development that it retains the ~1.17-bit prose recovery and still passes the
authored/held-out panels, and only then attempt the remaining gap to the retained count reference."*

It is executed on `codex/kimi-ready-20260923` at protected main `79b5d50c`, in the isolated Kimi
worktree `/Users/casey.allard/uor-r4-kimi`. Evidence is
[`docs/evidence/ordinary-lexical-formulation-2026-09-23.json`](../evidence/ordinary-lexical-formulation-2026-09-23.json).
Sealed roots: `olx-form-1..5`, `olx-condition-1/2` (all under
`/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/`). The delivered artifact root
`ordinary-lexical-3`, the retained local-channel root `olx-channel-1` and the audit roots
`olx-audit-1..5` are preserved and unmodified.

**The model source `learner/transferable_lexical.rs` is unchanged** (`sha256 03f83eb8…b250`, the
principal's reviewed hash). Only the runner `crates/uor-r4-core/src/bin/ordinary-lexical.rs` gained
declared options: `--objective window|position`, `--curriculum delivered|interleaved`, `--prose-slots`,
`--ground-slots`, `--ground-weight`, and a new read-only scoring mode `--condition` (score one sealed
artifact under both prose conditionings) with `--artifact`.

## 1. The extended runner reproduces the delivered artifact bit for bit

`olx-form-1` runs the **default** path — no new option set — on the delivered configuration (2000 steps,
batch 4, seed 13, lr 0.02, the delivered three-phase curriculum). Its artifact hashes to
`fe3e8a638cd50a19741065cb00ac787b63bb59f8ad5776fd8bfcb162c37a912a` and its development score is
`7.372198274238398`, **identical to the delivered artifact and receipt**. Every arm below therefore
runs through one unchanged learner and one unchanged default path, and all prose comparisons are on
the same 5,376 held-out development targets.

`olx-condition-1` reloads the delivered artifact in a fresh process and reproduces the stored
window-conditioned score exactly (`7.372198274238398`), which is the same guard through an independent
code path.

The guard run's step probe is itself informative. The delivered schedule reaches its best 8-window dev
probe **during the prose warm-up** (7.1080 at step 800), never materially below it afterwards; grounded
supervision is saturated by about step 1300; the grounded-only final phase (steps 1700–2000 at 0.6×
learning rate) ends at 7.3672 instead of restoring prose. The earlier audit could not make this
comparison because no intermediate checkpoint was persisted; the probe trajectory is the substitute,
and it is a *probe*, not the sealed full-development score.

## 2. Result: five matched arms

All arms: same pinned corpus and fit/tune/development split, same tokenizer, same learner, same seed
13, lr 0.02, 2000 steps. `served` is the window conditioning (frozen prefix advanced with `OBSERVE`,
later positions supervised as `GENERATE`) — **the recurrence the served multi-token path actually
executes**. `position` is the per-position all-`OBSERVE` conditioning, a diagnostic.

| root | objective | grounded draws / step (weight) | served bits/target | position bits/target | preflight A | held-out | preflight B | training panel |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `olx-form-1` (guard = delivered) | window | delivered 3-phase schedule | **7.3722** | 7.2573 | 32/32 | 3/3 | 2/2 | 36/36 |
| `olx-form-2` | window | 0 (none) | **6.6841** | 6.7296 | 0/32 | 0/3 | 0/2 | 0/36 |
| `olx-form-3` | window | 2 (4.0) | **6.9686** | 7.0187 | 31/32 | 1/3 | 2/2 | 35/36 |
| `olx-form-4` | position | 24 (4.0) | **7.0327** | 6.3387 | 32/32 | 3/3 | 2/2 | 36/36 |
| `olx-form-5` | position | 0 (none) | **6.6903** | 6.2039 | 0/32 | 0/3 | 0/2 | 0/36 |
| `olx-channel-1` (retained, prose-only) | position | 0 (none) | 6.6903 | 6.2039 | not applicable (prose-only) | | | |

References on the same target population: fit-only unigram **9.0827**, tuned interpolated two-token
count **5.1217**, donor E prior (exposed) **6.8701**.

Paired document-cluster bootstraps on the served conditioning (the runner's `paired_interval`
algorithm, 2,000 draws; each of the 24 development documents carries exactly 224 targets):

| difference (served conditioning) | bits/target | interval |
| --- | ---: | --- |
| `olx-form-1` − delivered | **+0.0000** | [+0.0000, +0.0000] |
| `olx-form-2` − delivered | **−0.6881** | [−0.7686, −0.6091] |
| `olx-form-3` − delivered | **−0.4036** | [−0.4668, −0.3412] |
| `olx-form-4` − delivered | **−0.3395** | [−0.4388, −0.2447] |
| `olx-form-3` − `olx-form-2` | **+0.2845** | [+0.2354, +0.3326] |
| `olx-form-4` − `olx-form-2` | **+0.3486** | [+0.2835, +0.4223] |
| `olx-form-4` − `olx-form-3` | **+0.0641** | [−0.0142, +0.1490] |

## 3. The prose deficit was the curriculum

`olx-form-2` changes **only** the declared schedule of the delivered objective: the grounded rehearsal
mix and the grounded-only final phase are removed, the learning rate stays at its base value, and prose
is drawn at the full declared batch for all 2,000 steps. Nothing about the objective, the conditioning,
the architecture or the corpus changes. Served prose moves **7.3722 → 6.6841**, a **0.688-bit
improvement [0.769, 0.609]** that crosses the donor E prior (6.8701) for the first time.

That single change accounts for the entire per-position diagnostic's served-conditioning recovery
(6.6903, §4). **The recorded hypothesis — that the shared learner needed a different prose objective —
is not supported.** The delivered deficit was how the prose gradient budget was spent, not which
conditioning event produced it. The remove-grounded arm is a prose diagnostic, not a successor: it
retains no grounded behaviour at all (0/32, 0/3, 0/2).

`olx-form-3` and `olx-form-4` both reintroduce grounded Copy/Stop supervision under the repaired
schedule and both keep a real prose gain while retaining the grounded responsibility; §5 gives the
price.

## 4. Correction: the retained audit's "~1.17 bits" is conditioning-inflated

The audit compared the delivered artifact's window-conditioned 7.3722 with the per-position
diagnostic's own position-conditioned 6.2039. Those are **different scoring events on the same
targets**, not two measurements of one quantity. Scoring both artifacts under both conditionings
(`olx-condition-1`, `olx-condition-2`) gives:

| artifact (training) | served window conditioning | position conditioning | position − window |
| --- | ---: | ---: | ---: |
| delivered (`window` objective) | 7.372198 | 7.257288 | −0.115 [−0.134, −0.097] |
| `olx-channel-1` `recurrence_only` (`position` objective) | 6.690311 | 6.203876 | −0.486 [−0.547, −0.421] |

Under the **served** conditioning, the position-trained prose-only artifact recovers **0.682 bits
[0.581, 0.790]** against the delivered artifact — not 1.17. The remaining ~0.49 bits is the
conditioning change, which serving does not enjoy: the per-position conditioning is only the served
recurrence for the *first* generated token, after which serving advances the state with `GENERATE`
events the per-position objective never trains. This is a scope correction to the retained
[audit](ordinary-lexical-audit-2026-09-23.md) and its public restatements; it does not change the
audit's dataset, artifacts or sealed numbers.

## 5. The price of retaining grounded Copy/Stop supervision

`olx-form-3` is `olx-form-2` plus two grounded examples per step at the declared weight 4.0 — a
single-option change. It costs **0.2845 bits [0.2354, 0.3326]** of served prose and buys back 31/32
authored temporal regimes, 1/3 held-out compositions, 2/2 post-copy preflight B, 35/36 authored
training cases and 4/4 class cases. It is one case short on three of the four grounded panels, so it
nearly — but does not fully — retain the grounded responsibility.

`olx-form-4` (the recorded milestone) trains the **per-position** objective with 24 grounded draws per
step at weight 4.0 and 224 prose positions per step. It fully retains the grounded responsibility —
**32/32, 3/3, 2/2, 36/36 training, 4/4 class, separate-process restore `REPLAY_OK 36 cases`** — and still
improves served prose by **0.3395 bits [0.2447, 0.4388]**. It is the milestone's positive result: the
shared grounded learner can carry a per-position full-context prose objective without giving up exact
copying.

But `olx-form-4` − `olx-form-3` is **+0.0641 [−0.0142, +0.1490]** — indistinguishable from zero. The
per-position objective is **not** what produced its grounded retention or its prose gain; the repaired
schedule and the grounded share did. And `olx-form-4`'s own conditioning gap is the largest measured
(6.3387 position versus 7.0327 served, −0.694), i.e. the artifact is much better under a conditioning
that serving cannot use beyond the first generated token. On the metric that matches serving, the
serving-conditioned `olx-form-3` is at least as good. **The per-position objective is retired as a
necessary training change** for this milestone; it remains a legitimate diagnostic conditioning.

## 6. The served-conditioned objective is a wash, and the retained arm reproduces exactly

`olx-form-5` runs the per-position objective with **no** grounded supervision under the repaired
schedule. It reproduces the retained `olx-channel-1` `recurrence_only` arm **exactly**, through a
different runner path and a fresh process: served **6.6903113844186946**, position
**6.203876300792837**, position-minus-window **[−0.486435, −0.546994, −0.420634]** — every digit equal
to the sealed prior receipt. That is an independent re-derivation of the audit-era per-position
diagnostic and the check on §4's correction.

Against `olx-form-2` (the window objective, prose-only, same repaired schedule, same seed, same 2,000
steps) at **6.6841**, the two objectives are separated by **+0.0062 [−0.0883, +0.1012]** — a clean
matched pair whose interval straddles zero by a wide margin. **The objective is a wash for the served
metric; the schedule was the lever.** This is why the per-position objective is retired as a necessary
training change rather than promoted: it buys nothing that the serving-consistent objective does not
already have, and it leaves the model better-optimized under a conditioning that serving can use only
for the first generated token.

## 6a. Independent verification of the milestone artifact

`olx-form-4-audit` deserialises the milestone artifact in a fresh process and replays it: authored
training **36/36**, held-out **3/3**, class **4/4**, preflight A **32/32**, preflight B actual **2/2**
with the identity-erased arms aliasing and the source-disabled arm losing the exact copy, all matching
the stored panels; the five mixed-session phases match the stored run. The crossed `Copy`-feedback
intervention still classifies as fingerprint-driven on this artifact. So the milestone's grounded
retention holds on the **serialised** artifact, not only on the in-memory instance. The twelve retained
`transferable_lexical` unit tests, `cargo fmt --check` and an offline release build pass; the model
source is byte-identical to the principal-reviewed hash.

A window-objective successor that fully retains the grounded panels is the obvious next arm: increase
the grounded presence under the served conditioning until 32/32, 3/3 and 4/4 are recovered, then spend
the remaining prose budget. `olx-form-3` shows that two grounded draws at weight 4.0 are 31/32 and 1/3;
the gap is small and bounded.

## 7. What still fails

- **The tuned two-token count reference still wins.** The best arm (`olx-form-2`, 6.6841) trails the
  fit/tune-separated interpolated `(prev, cur)` count model at **5.1217** by **1.56 bits/target**.
  Every grounded arm trails it by 1.85–1.91. A 64-coordinate ternary recurrence over a 4,096-token
  vocabulary still uses local two-token information less effectively than a lookup table. **This is the
  binding local-competence fact, and it is a local-recency/optimization question, not a long-range
  transport question.**
- **Greedy generation remains degenerate.** All arms produce a locally plausible prefix and then
  collapse into repetition; the count reference degrades similarly on the same prompts. Sampling is
  NOT_RUN.
- **Energy remains UNAVAILABLE.** Served cost is unchanged in kind: 466,711-byte artifact, no multiplier
  and no floating-point arithmetic in the declared kernel, 52,681–147,456 nonzero weight reads per
  served step depending on the arm.

## Limits and claim boundaries

Measured: the default path's artifact-level identity to the delivered baseline; the prose deficit's
attribution to the declared schedule on this corpus, split and seed; the grounded-supervision price;
the conditioning correction; the per-position objective's neutrality for the served metric.
Hypothesis: that a larger grounded presence under the served conditioning recovers the panels without
returning to the delivered prose level.
Not claimed: general prose, chat, reasoning, geometric advantage, any promotion of the model, or
energy. One pinned local corpus, one split, one seed per arm, one schedule family; the grounded world
is authored and tiny, so per-arm panel retention is a fixture result. A failed arm does not demote the
grounded or per-position mechanism families; each negative holds at its exact tested configuration.

## Resources

Five fits plus two read-only scoring runs: `olx-form-1` 500 s, `olx-form-2` 696 s, `olx-form-3` 697 s,
`olx-form-4` 1,285 s, `olx-form-5` recorded in the evidence file, `olx-condition-1/2` 27 s and 37 s.
One process at a time, two threads. New retained evidence is well under the declared 128 MiB; physical
free space stayed above the 24 GiB working reserve plus the 128 MiB stop margin. Complete projection,
the prospective ledger extension and the final charge are in the
[resource ledger](resource-ledger-2026-09-19.md) and
[`docs/evidence/lexical-formulation-projection-2026-09-23.json`](../evidence/lexical-formulation-projection-2026-09-23.json).
No paid or external compute; no unique artifact or preserved root was modified.

## One evidence-supported next step

Keep the **served window conditioning** and raise the grounded presence under it until the authored
panels are fully retained, then recover prose exposure — the configuration that `olx-form-3` was one
case short of. Pre-declare 32/32 preflight A, 3/3 held-out and 4/4 class as the acceptance, and measure
the served prose against `olx-form-3`'s 6.9686 and `olx-form-2`'s 6.6841 as the two ends of the
trade-off. Then attack the remaining **1.56-bit** deficit to the tuned two-token count reference, which
is a local-recency problem and the strongest simple witness that the shared state is not yet using the
information available at the decision point. Signed H4/shared relative transport stays conditional on a
witnessed order/role/distant-interference failure against an information- and compute-matched ordinary
control; Hopf fiber, S7/E8 and harmonics remain conditional tools.
