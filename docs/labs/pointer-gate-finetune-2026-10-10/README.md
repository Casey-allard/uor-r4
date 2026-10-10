# The pointer-gate fine-tune moves v5 memory 10/40 → 21/40 and costs the open reply panel 43/232 → 28/232 (matched control 22/232) — the gain and the damage are both the training data, not `gate_supervised_loss`

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek. Session `deepseek/pointer-ft`, opened
2026-10-09 ET, run 2026-10-10 03:40–04:03 UTC. **One pod, 2 × RTX 5090, 22 minutes, ≈ $0.87 of the
$3.60 ceiling the pre-registration declared**; corpus generation, mixing, training and the three
development curves are inside that lease, and every acceptance reading ran on the owner's laptop CPU.

> **One line.** D21 told this lab to train the pointer fix. The run was executed exactly as
> pre-registered. The frozen memory panel moved **+11 rows on the pre-declared arm** (bar met, p = 0.013);
> the open reply panel fell **43/232 → 28/232**, **missing the guard by one cell**; and the **matched
> control is at least as good on the memory half and no better on the reply half**: so the result is a
> **trade-off carried by the training data**, `gate_supervised_loss` is **not** the cause, and the artifact
> is **not kept as the base**.

## The pre-registration, and what was actually run

[Pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) before any pod existed:
bar, base checkpoint, stream, arms, readings and cost. Nothing below departs from it.

| | |
|---|---|
| base | `chat-29m-B-lr5e-4`, `model.safetensors` sha256 **`d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068`** — the artifact that produced 43/232 and 10/40 |
| lineage | the base's **own** run `report.json` as `init=` provenance: tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, `merges: null`. Fetched from `icloud:UOR-R4/results/deepseek/ladder.tar` (3,306,919,424 B, md5 `ed9456d63472b7a424fe6ca62d36baa8`, **md5 verified on restore**); the same archive's model copy was re-hashed to `d8a3c971…`, and its `report.json` records the base's own fine-tune settings (`lr = 0.0003`, 4,000 steps, `pointer=32`, `policy=full_prefix`, `data_seed=1`) |
| the one change | `dialogue-train … pointer_gate_supervision=W` (`StackModel::gate_supervised_loss`): on each scored target whose id an input position `0..=t` holds, `W·(BCE(g_t,1) − log p_copy(target))`; on a scored target no source holds, `W·BCE(g_t,0)` |
| stream | the base's own fine-tune store **plus** a generated recall corpus, concatenated by `mix-chat-corpus` |
| arms | **S = 0.1 (primary, pre-declared)**, **C = 0.0 (matched control)**, **H = 0.5 (dose only)** — identical in every other field |
| training | `steps=2000 batch=16 lr=2e-4 warmup=100 eval_every=250 checkpoint_every=500 data_seed=20261009 policy=full_prefix context=384 device=cuda qat=false`; **104–105 s per arm** |
| development instrument | the recall generator's own `dev/` split, whose curve carries `dev_pointer_nll`, `dev_pointer_hit_rate`, `dev_pointer_reachable_rate`, `dev_response_nll` |
| pod | `wua3r5aex4mmwo`, `--ref 5b241d89c4fd632b8839c436f42fd154b178f718` (the commit the pre-registration was written against), bootstrap parity PASS (37 passed / 0 failed), lease released and the pod deleted at 04:03 UTC |

### The stream, with its identities

1. **The base's own fine-tune store.** `ladder/corpora/ft-mixed-mworld-chatv0` in the same archive:
   `tokens.u16` sha256 **`8794ad2710aa9e2a34772235d2c966d03bd672bf7d3f97385365e77b1a20c3b8`**,
   `response_mask.u8` sha256 **`953f1224576629cb8a52ac8f19bc4f6c65039f27c6b4bef6a99d4239c52e61b1`** — **both equal to the
   `inputs.train` of the base's own `report.json`**, which is why this store and not a look-alike was
   used: the run's "before" is the store that produced the base. 82,879,894 tokens, **147,104 response
   episodes**, mixed from M-world parar-p2-train ×3 + chat-v0-p2.
   (*A near-miss worth recording:* `data/ft-mixed-c.tar` on the canonical volume is a **different** mix of
   the same family — 84,142,760 tokens, sha256 `7282b7a6…` — and is **not** the store the base's report
   records. It was discarded before any training.)
2. **The generated recall corpus.** `dialogue-recall-corpus generate generator=v2 seed=1 dialogues=124000
   dev_dialogues=300 dev_seed=1000003 protocol=2 context=384 samples=200 train_on=answers
   binding_labels=1 source_commit=5b241d89c…`: **26,212,659 tokens** (`tokens.u16` sha256
   `dd61295eebd308e6d0c7dc33a666854a1849324416afcf6b4cef07c1621ad277`, `response_mask.u8`
   `9f833b9b51456ce0399504ae3ee01ffe3b02ce8a0ead3c822227d70df7d7b328`), **180,200 trained answer replies**
   across twelve answer families — `binding` 39,010, **`abstain` 24,428**, `order` 16,653, `attribute`
   15,276, `rule` 14,675, `count` 14,320, `self_fact` 14,194, `implicit_update` 13,229, `update` 9,452,
   `cross_relation` 6,973, `reverse` 6,882, `assistant_stated` 5,108 — and a 300-dialogue dev split (427
   replies); generated in 47 s. Its own `leak.json` is the mechanical
   disjointness assertion — `panels=` was **`data/panels` plus a directory holding the three
   open-reply-panel request files** (`everyday-32.json` sha256 `945c0c97…`, `heldout-200-a.json`
   `019cc6f6…`, `heldout-200-b.json` `5434cfd9…`) — so **both frozen panels** were checked — **5,678 panel
   strings, 3,861 panel 6-grams, 0 matching examples in train or dev, `pass: true`**.
3. **The mixture.** `mix-chat-corpus inputs=<1>,<2> labels=base-mixed,recall` →
   **109,092,553 tokens** (`tokens.u16` sha256 `f6ae278dfed8b6e5d88f1b8b4c19e22359c1027b2cf053580c9d2fc47c763994`, mask
   `cc574a5506f4d3f182b2f61fb2f3e6f6fd8706c33455c51dd15194fec0ed6591`), **327,304 response episodes, recall
   share 55.06 %** — inside the pre-registered
   40–60 % band. The sampler is response-uniform, so the mixture ratio is a *response* ratio, not a token
   ratio; the base's own span is 44.94 % and is present at every step, which is the whole point of
   `mix-chat-corpus`'s documented lesson (fitting the added rows alone destroys the model).

## The arms

**Development instrument** (the recall generator's own dev split, 32 source-stratified responses):

| arm | `pointer_gate_supervision` | `dev_pointer_hit_rate` step 0 → 2000 | `dev_pointer_nll` | `dev_pointer_mean_gate` | `dev_response_nll` | `train_seconds` |
|---|---:|---:|---:|---:|---:|---:|
| S — primary | **0.1** | 0.142 → **0.761** | **0.0931** | 0.721 | 0.2294 | 104.8 |
| C — matched control | 0.0 (off) | 0.142 → **0.377** | — (off) | 0.0823 | 0.2221 | 105.0 |
| H — high dose | 0.5 | 0.142 → **0.755** | 0.1069 | 0.775 | 0.2547 | 104.4 |

**The supervision does exactly what it was designed to do on this stream**: it doubles the pointer's hit
rate over the matched control (0.377 → 0.761) and drives `p_copy(target)` up (pointer NLL 0.093). The
control's own 0.142 → 0.377 is what two thousand updates on recall-shaped dialogues buy without any
supervision.

**Frozen v5 acceptance panel** (`data/panels/conversational-v5.json`, cap 64, protocol 2, the frozen
`conversational-v5-checks.tsv`, deterministic `check_pass` primary — this is the pre-registered bar):

| | memory `check_pass` | memory `acceptable` | unknowable `check_pass` | unknowable `acceptable` | panel `check_pass` | panel `acceptable` | derangement `check_pass` |
|---|---:|---:|---:|---:|---:|---:|---:|
| base (re-run on this instrument) | **10/40** | 4 | 1/24 | 0 | 11/64 | 4 | 0 |
| **S — primary (W=0.1)** | **21/40** | 12 | 16/24 | 7 | 37/64 | 19 | 0 |
| C — matched control (W=0) | **22/40** | 11 | 19/24 | 9 | 41/64 | 20 | 0 |
| H — high dose (W=0.5) | **19/40** | 11 | 16/24 | 7 | 35/64 | 18 | 0 |

Paired exact McNemar over the 40 memory rows: base → S **p = 0.0127** (base-only 3, S-only 14);
base → C **p = 0.0018** (1, 13); base → H **p = 0.0225** (2, 11); **S vs C p = 1.00** (4, 5);
S vs H p = 0.73; C vs H p = 0.51.

**The base reproduces the sealed declared reading exactly** — memory `check_pass` 10/40, unknowable 1/24,
`acceptable` 4 and 0, panel 11/64 — on a build of current `main`, so the comparison is same-panel,
same-checks, same-cap.

**The open reply panel is the guard.** The pre-registered metric is the sealed report's `acceptable`
(fluent ∧ relevant; this panel carries no frozen row checks), so the like-for-like field is the report's
`fluent_and_relevant`. Both readings are given, because passing `checks=` to `grade-replies` makes
`acceptable` *stricter* (fluent ∧ relevant ∧ the row's frozen check on the 88 checked rows) and is
therefore not the number the pre-registration's 43/232 baseline was measured on.

| | `fluent_and_relevant` — the pre-registered metric | `acceptable` with the 88 row checks | fluent | relevant | derangement `fluent_and_relevant` |
|---|---:|---:|---:|---:|---:|
| base, **sealed** (`chat-29m-B-lr5e-4`, 2026-10-03) | **43/232** | — | 70 | 53 | 4 |
| base, **re-graded on this build** from the sealed replies | **43/232** | 27/232 | 70 | 53 | 4 |
| **S — primary (W=0.1)** | **28/232** | 16/232 | 55 | 37 | 11 |
| **C — matched control (W=0)** | **22/232** | 12/232 | 36 | 31 | 3 |
| H — high dose (W=0.5) | not run — the pre-registration bounds this reading to S and C | | | | |

**The base reproduces the sealed 43/232 exactly, per category** (`follow_up` 1, `heldout_first_turn` 34,
`simple_instruction` 1, `simple_question` 1, `smalltalk` 6 = 43), on a build of current `main`, from the
same sealed replies — so the guard comparison is instrument-for-instrument.

Per category, `fluent_and_relevant` for base → S → C: `follow_up` 1 → 2 → 1; **`heldout_first_turn`
34 → 21 → 17**; `simple_instruction` 1 → 0 → 1; `simple_question` 1 → 0 → 0; `smalltalk` 6 → 5 → 3.
Paired exact McNemar on the 232 rows: base vs S base-only 27 / S-only 12, **p = 0.0237**; base vs C
base-only 26 / C-only 5, **p = 0.0002**; S vs C 17 / 11, p = 0.34. Both arms still discriminate their own
derangement control (base p ≈ 0, S p = 0.0046, C p ≈ 0), but **S's derangement rises from 4 to 11**: more
of its replies are generic enough to be judged fluent-and-relevant against the *next* row's conversation.

**What the damaged replies look like** (row inspection, `heldout_first_turn`): the primary arm's failures
are dominated by the recall corpus's own habits leaking into open-ended requests — `heldout-000`
"*I'm not sure. You never told me about a group of people dealing with the aftermath of a flood…*" and
`heldout-005` "*1. Are you looking for a new chapter or a story that is engaging for you? 2. Are you
looking for a new personal character…*" are the abstention and clarify families answering a request that
does not need them, and `answer:abstain` is **24,428 of the 180,200 recall replies (13.6 %)**. The control
arm's failures are more often degenerate repetition (`heldout-000` "*A group of people dealing with the
aftermath of a flood, dealing with the aftermath of a flood.*"). Both point at the same lever: the
mixture's composition, not the supervision.

## Decision

**Primary bar MET, guard MISSED BY ONE CELL, artifact NOT KEPT, and the pre-registered attribution REFUTED.**

- The pre-registered primary bar (v5 memory `check_pass` **≥ 14/40** on arm S) is **MET**: 21/40.
- The pre-registered guard (open reply panel `acceptable` **≥ 29/232**, baseline 43/232) is **MISSED BY
  ONE CELL**: **28/232** on the primary arm and **22/232** on the matched control, on the like-for-like
  metric. The paired test says the drop is real rather than judge noise (43 → 28, base-only 27 / S-only
  12, **p = 0.0237**; the control's 43 → 22 is p = 0.0002), and the panel's own ±14-cell heuristic puts
  the primary's drop one cell outside its resolution. The pre-registration's rule was "*KEEP iff primary
  and guard hold*", so this is **not KEEP** — but it is a one-cell miss on the pre-registered metric, and
  both numbers are reported so the width of the miss is visible.
- The pre-registered attribution is **REFUTED**: the matched control is **higher on the memory panel**
  (22/40), higher on the whole v5 panel (41/64 against 37/64) and higher on the unknowable rows (19/24
  against 16/24); S and C are statistically indistinguishable on the memory rows (**p = 1.00**), and on the
  reply panel the control is *lower* (22 against 28, p = 0.34, not significant), so the supervision does
  not explain the damage either. The dose arm lands below both on memory. **The movement and the damage
  are the recall-dialogue mixture, not `gate_supervised_loss`.**
- The supervision's own effect is **not** dismissed: on the development stream it is large and in the
  intended direction at the same data, steps and seed. It is simply **not measurable on either acceptance
  panel**, which is the reading that decides.
- **Criterion 1 remains NOT MET on both halves** and is not claimed: memory 21/40 against ≥ 34/40, reply
  28/232 against ≥ 116/232. The base artifact's own 43/232 and 10/40 are unchanged and it stays the base.

**What the result says to do next, in order.** (1) The lever is the mixture, so the next M1 piece is a
**mixture-dose experiment**: the same run at a lower recall share with the reply panel as the pre-declared
guard — a run this result *designed*. (2) `pointer_gate_supervision` is dropped from the recipe: it costs a
hyper-parameter for nothing measurable on either panel. (3) The read/emit split that started this line is
unchanged by this run — see the instrument section.

## The read-level instrument, re-run beside the panel

The `pcopy-mass-read` example was re-run on all four artifacts with its three enforced checks (trace gate,
attention-argmax gate, two-paths-agree gate); on the base **all three reproduce the published record**:
13 rows checked, **0 trace mismatches**, cross-path delta **2.329e-06** — the same value the record
published — **0 of 13** numeric rows put the value above the frame, and **21 of 27** word rows do.

| | numeric rows, value > frame | mean max value share (numeric) | mean max frame share (numeric) | word rows, value > frame |
|---|---:|---:|---:|---:|
| base | 0/13 | 0.2361 | 0.7430 | 21/27 |
| S — primary | 1/13 | **0.5571** | **0.9153** | 23/27 |
| C — matched control | 2/13 | 0.5015 | 0.9041 | 24/27 |
| H — high dose | 1/13 | 0.5929 | 0.9799 | 24/27 |

**So the fine-tune roughly doubles the copy channel's value share on the panel's own numeric rows
(0.236 → 0.50–0.56) and the frame's share rises with it (0.74 → 0.90–0.92): the pointer did not stop
attending the sentence frame, and the panel gain is not explained by the read.** That is consistent with
the control tie and with the reply-panel damage: what changed is the answer behaviour the mixture teaches,
not the attention target the diagnosis had indicted.

## Limitations

- **One seed per arm.** The arms' spread is unmeasured; the ±1-row differences among arms are inside what
  one seed can produce, which is why they are reported as ties.
- **40 rows and 232 rows are small.** One v5 row is 2.5 points; the McNemar tests are on 4–17 discordant
  pairs; the reply panel carries the judge's measured instability.
- **The reply-panel baseline is re-graded, not assumed**: the sealed replies score 43/232 on this build
  with the same per-category split, which is what makes the ±1-cell guard call meaningful; the ±14-cell
  heuristic is the panel's published noise floor, and the paired McNemar above is the sharper test.
- **The trade-off is dose-dependent and only one dose was run.** 55.06 % recall episodes is one point;
  nothing here says a lower share cannot buy the memory rows for less reply damage.
- **v5 was never trained on** — the corpus's own leak check against `data/panels` *and* the three
  open-reply-panel request files passes — but the recall generator's *templates* resemble the panel's
  question shapes by construction. That is the intended transfer, and it is also why the **unknowable-row**
  movement must be read as behaviour transfer rather than as new memory.
- **The dev instrument is not an acceptance panel.** It says what the supervision did to the pointer, not
  what the panels will say; this run is the evidence for that distinction.
- Training length, evaluation length, memory access and vector width are separate parameters; this run
  exercises training length and evaluation length only, and the artifact still carries no memory operator
  (`memory_read_diagnostic` returns `None` by construction), so nothing here is evidence about addressed
  memory.

## Cost

| | |
|---|---|
| pod | 2 × RTX 5090, leased 03:40:49Z → 04:03Z, **≈ 22 min ≈ $0.87** against the pre-registered $3.60 ceiling; lease released, pod deleted, nothing left leased |
| GPU work | 3 arms × 105 s = 5.3 min of training; corpus generation 47 s; mixing 2 s |
| bootstrap | 271 s (shared binary cache hit for the commit; CUDA parity 37 passed / 0 failed) |
| laptop CPU | `chat-grade` build 5 m 19 s; `pcopy-mass-read` build 2 m 03 s; v5 replies 4 × ~90 s and v5 grading 4 × ~10 min; reply-panel replies 2 × ~10–25 min and reply-panel grading 3 × ~35 min; the deterministic row check reimplemented independently in ~1 s, and it returns the recorded 10/40 for the base, which is how it was validated before use |
| external | none; no new spending class, no owner decision required |

## Evidence

- Arms, curves and settings: `runs/arm-{primary,control,high}/report.json` (pod volume
  `/workspace/uor-r4/deepseek/pointer-ft-20261009/`, pulled to the laptop with sha256 verified for every
  `model.safetensors`).
- Acceptance reports: `score/v5g-{base,primary,control,high}/report.json`,
  `score/rpg-{primary,control,base}/report.json`, `score/pcopy-{base,primary,control,high}.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/pointer-gate-finetune-2026-10-10.tar`, **25,377,280 bytes, md5 `cb7da3a3286db49b0e48b7222970604a`** (object `pointer-gate-finetune-2026-10-10`,
  uploaded and MD5-verified by `cloud-store put`, whose index entry records the same checksum). It holds
  the three arm reports and curves, every acceptance report and reply file, the four `pcopy-mass-read`
  outputs, the row-by-row v5 verdict table, the corpus `generator.json`/`leak.json` and manifests, and
  the pod-side run scripts and scoring logs.
- Pre-registration and claim: [#2029 comment 6093396145](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6093396145);
  status cards [#2029 comment 6093373990](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6093373990)
  and [#2029 comment 6093638213](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6093638213).
