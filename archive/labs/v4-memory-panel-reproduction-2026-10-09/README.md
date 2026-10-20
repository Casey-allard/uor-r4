# v4 memory panel: 31/40 reproduced from the artifact, the 9 misses are two mechanisms, no bounded lever — October 9

Lab: deepseek. Issue: [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). Scope: **offline
capability, not served.** **Result: the 31/40 baseline is reproduced exactly, and the piece is closed
NEGATIVE-ORIENTED — the nine failures are not dominated by one mechanism, and the only on-point lever on
this base is already a recorded negative.** No training run, no candidate, **zero pod spend**. Pod use was
not needed at any point: the panel, the grader and the artifacts were reached from the laptop.

Two identities in the round's brief were wrong and are corrected here; a third was ambiguous and is
pinned down.

## The question

M1 acceptance criterion 1 asks for `≥ 34/40 acceptable` on the frozen `chat-grade` v4 memory panel. The
recorded best is `31/40 at 214M` against `26/40 at 96M`. This round had to (1) read the frozen grader and
the sealed panel so the number means something, (2) reproduce 31/40, (3) classify the nine failures per
category with the mechanism named for each, and (4) say plainly whether one named mechanism dominates the
nine — a three-cell target against diffuse failures is not a single piece.

## The artifact, the panel and the rule (all identities verified)

**Panel `conversational-v4*`** (`data/panels/`), authored blind on 2026-10-05 as the Step 6c acceptance
panel, **64 rows, two categories only**: `multi_turn_memory` **40 rows** (`exact` check) and
`unknowable_or_impossible` **24 rows** (`abstain_exact`). All twenty files in `data/panels/` verify
against the frozen `MANIFEST.sha256` (`shasum -a 256 -c MANIFEST.sha256` → 20 × `OK`):

| file | sha256 |
|---|---|
| `conversational-v4.json` | `bea6743c4d6c7c97eead6232179b6d0620455ab26565bb813a863da41c9c5cd1` |
| `conversational-v4-a.json` (rows 1–32) | `f0029d57d8ab54e33709e933528e34a158b324c16cc53fb03ca75fbf47466ba0` |
| `conversational-v4-b.json` (rows 33–64) | `57cbe574d7c09a4e48101b036161a4a06966833a29e2f3ab954a03bc254b889d` |
| `conversational-v4-checks.tsv` | `b4238e112e5e66cfd773de75f71ae8eb39c9077c172fe8ff0f4ba8b33cc5793e` |
| `conversational-v4-swaps.tsv` | `465b5fd4f555facdddbd71ed96e03b958b8458d0aecb002468c293d69eeb381c` |

**The rule** (`crates/uor-r4-training/src/bin/chat-grade.rs`, `RowCheck::passes` at lines 924–938 with
`words`/`contains_phrase` at 614–627). For a `multi_turn_memory` row the reply **passes its check iff all
three hold**: it contains at least one of the row's expected spellings; it contains none of the row's
forbidden values (the distractor values stated in the conversation); and it contains no word of the
distractor's *key*. The row's tool-level `acceptable` is stricter still: the reply must additionally be
judged fluent and relevant by a local `qwen2.5:7b` Ollama grader, which the tool documents as unable to
tell a recalled fact from an invented one — the reason the deterministic check exists.

**The 214M artifact that actually holds 31/40** is **Step 8 arm Q**, not a Plan A base and not the raw
pretrain: pretrain sha256 `e448de86ae30ab0785e2c8400dcf32e29b97e8ed03830b14183371bb1664edcc`
(`bases/geo-214m-e448de86` in `caseyallard/uor-r4-store`, present in the HF cache and re-downloaded here
with the hash re-verified) **plus** the Step 7d fine-tune — `steps=2000 batch=16 lr=0.00015 warmup=100
protocol=2 context=384 policy=full_prefix pointer=32 data_seed=1 key_shift=false tf32=true`, no read
supervision. The fine-tuned model sha256 is
`9c0d901990f137728eac39c11ec3063fe90fe7ec93f3097316405328343d3c1b`, `pointer.dim=32`, 214,031,977
parameters; tokenizer sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`
(matches the recipe's recorded `inputs.tokenizer.sha256`). The argv is the **recorded** one from
`run-step8.sh` inside the arm's own archive (`results/claude/step8-capacity-20261007.tar` →
`step8-Q-7c682fdb.tar`), not a reconstruction; the same archive carries the sealed
`grade-v4-FT-Q-s1/report.json` and `replies-v4-FT-Q-s1/replies.json`.

Consequence: `bases/planA-214m-2b425ccd` and `models/chat-214m-planA-af-b8` (sha256 `11b84ba7…`) have
**no recorded v4 memory number at all**. There is no Plan A baseline to beat.

## The name of the number (this is a correction, and it changes the gap)

`31/40` is chat-grade's **`check_pass`** — the frozen deterministic check alone. It is **not** the tool's
`acceptable`. Both are in the sealed report:

| quantity (category `multi_turn_memory`, 40 rows) | sealed arm-Q report | this reproduction |
|---|---:|---:|
| `check_pass` (the frozen `exact` check) | **31** | **31** |
| `acceptable` (fluent ∧ relevant ∧ check) | **27** | **27** |
| `fluent` / `relevant` / `fluent_and_relevant` | 29 / 28 / 28 | 29 / 29 / 28 |
| `unknowable_or_impossible` `check_pass` (24 rows) | 10 | 10 |
| whole panel `acceptable` / `check_pass` (64 rows) | 31 / 41 | 31 / 41 |

The ladder column quoted in #2029 (`31/40` memory, `10/24` unknowable) is `check_pass` throughout, so
criterion 1's `≥ 34/40` is a **three-cell** gap read that way and a **seven-cell** gap read as the tool's
`acceptable`. Any candidate must report both, and the record must not let a reader infer which one is
meant. **Which one is primary is not a preference: `check_pass` is deterministic and `acceptable` is not.**
The re-grade below reproduces `check_pass` exactly and `acceptable` exactly, yet **4 of 64 rows carry a
different grader verdict at the same `qwen2.5:7b` digest** — so `check_pass` is the primary reading,
`acceptable` the secondary, and a candidate that moves `acceptable` without moving `check_pass` has to show
which rows the judge changed its mind on. The README's own cells mix the two metrics (`recall 31/40` is
`check_pass`; `open panel 36/232` is `acceptable`) — fixed there below.

## Reproduced (measured, laptop CPU, zero pod)

**(a) Scoring reproduction on the sealed replies.** `replies.json` sha256
`647215308b85f8969dc14e7e15654190e921487132f66c5d517899dc109352e4` — the same bytes the ladder used.

```text
chat-grade grade-replies out=/tmp/uor214/runs/grade-v4-repro1 \
  replies=.../replies-v4-FT-Q-s1/replies.json grader=qwen2.5:7b
```

Grader digest `845dbda0ea48ed749caafd9e6037047aa19acfcfd82e704d7ca97d631a0b697e` — **identical to the
digest recorded in the sealed report**, pulled from the same Ollama tag. Result: every headline number in
the table above, exact. 60 of 64 rows have field-identical grader verdicts; 4 rows differ only in
constant-control verdicts (and one relevance verdict) and **no headline number moves**. Wall 11 min 1 s.

**(b) Full local regeneration from the artifact.** An independent check implementation first reproduced
31/40 from the sealed bytes alone (`classify_v4_memory.py`, shipped beside this record). Then the model
was run end to end on CPU:

```text
chat-grade reply out=/tmp/uor214/runs/replies-v4-ftq-cpu \
  model=.../FT-Q-s1/model tokenizer=tokenizer.json \
  requests=data/panels/conversational-v4-a.json,data/panels/conversational-v4-b.json \
  max_new_tokens=64 device=cpu
```

`model_sha256`, `tokenizer_sha256`, `protocol` and `max_new_tokens` are identical to the sealed run;
only `device` differs (`cpu` vs `cuda`). Result: **31/40 `check_pass` with the identical nine failures**,
and **all 64 rows byte-identical to the sealed CUDA replies in both the reply strings and the generated
ids** (64/64 rows, all 150 reply calls). Greedy decoding on this artifact is device-independent between
the pod's GPU path and the laptop CPU path, so the panel number is reproducible from the artifact without
a pod at all. Wall 17 min 30 s (150 replies, 1,044.5 s reply time, 1.6 ids/s).

**(c) Where the outputs of this round are.** The sealed roots both runs produced are preserved as one
iCloud object, verified after storage:

| object | bytes | md5 | index row |
|---|---:|---|---|
| `icloud:UOR-R4/results/memory-v4/runs.tar` | 329,728 | `b7870d3f0c1c9d55c263c3f8bdfe13cc` | `results/memory-v4/runs.tar`, lab `memory-v4`, 2026-10-09T19:33:18Z |

It holds `runs/replies-v4-ftq-cpu/` (the CPU-generated `replies.json`, 64 rows, `device=cpu`,
`model_sha256 9c0d9019…`, with its `attempt.json`/`manifest.json`) and `runs/grade-v4-repro1/` (the
re-grade report — `per_category.multi_turn_memory` `check_pass` 31 / `acceptable` 27 — **including the four
rows whose grader verdicts differ from the pod's at the same digest**, with its own sealed root files).
The classifier runs of this round are the two invocations of the shipped `classify_v4_memory.py` documented
above, reproducible from that script plus either replies file. It is cited rather than committed because it
is run output, not source: the storage rule keeps bulk artifacts in iCloud with `main` holding the index
and the identities, and the sealed inputs it derives from are already addressable by sha256 above. It is
the evidence for the claim this record calls its strongest — **all 64 rows byte-identical to the sealed
CUDA replies in both reply strings and generated ids, so greedy decoding on this artifact is
device-independent and the panel reproduces with no pod.**

`~/.local/share/uor-r4/bin/cloud-store fetch runs <dest>` restores it (the object's name is `runs`; its
index row is `icloud:UOR-R4/index/runs.tsv`).

## The nine failures, per category, mechanism named (counts, not impressions)

Every one of the nine fails because **the expected value is not in the reply**. The frozen check's own
failure modes split them as follows (`classify_v4_memory.py`, identical on the sealed and the regenerated
replies):

| mechanism | count | cells |
|---|---:|---|
| **wrong value** — the reply names the row's forbidden distractor value in the asked slot | **3** | `mem-11`, `mem-20`, `mem-29` |
| **no value** — the reply names neither the expected value nor any forbidden value, nor a distractor key | **6** | `mem-23`, `mem-24`, `mem-28`, `mem-30`, `mem-35`, `mem-37` |
| form/prefix failure (value present but the reply carries scaffolding) | **0** | — |
| wrong-key binding (value present with a distractor key word) | **0** | — |
| abstention where an answer was required | **0** | — |

The three wrong-value replies, verbatim: `mem-11` → `' Odette is called turquoise.'` (expected
`pelican`; turquoise is the forbidden colour); `mem-20` → `" You're bringing a jigsaw."` (expected
`raincoat`); `mem-29` → `' Your apartment is on Juniper Street.'` (expected `47`; Juniper is the
forbidden street). In each the model selected the *other* stated value and bound it to the asked key.

The six no-value replies, verbatim: `mem-23` → `' Bodhi got a new toy.'` (echoes the setup sentence; the
value `xylophone` is in a later turn); `mem-24` → `' Under the leaves.'` (echoes the question's own
location phrase for `caterpillar`); `mem-28` → `' Nico is on weekends.'` (subject plus the question's time
phrase, for `archery`); `mem-30` → `' You told me the bake sale.'` (filler, for `cafeteria`);
`mem-35` → `' Your aunt is in winter.'` (takes the time from the *other* sentence, for `pumpkin`);
`mem-37` → `' Your brother is getting a kazoo.'` (right key, the name `oskar` dropped). Four of the six
are fluent non-answers assembled from the question or the setup; two pick up neighbouring content.

**The independent instrument agrees and splits them further.** The pre-registered binding probe (#1764)
was run on exactly this artifact at Step 11 and classified exactly these nine misses as **5 swap / 4
readout / 0 unreached** — "swaps are now the majority class at 214M", with capacity having halved the
readout misses (8 at 96M → 4 at 214M) and left swaps unchanged. That is consistent with the table above:
the 3 wrong-value cells plus 2 of the no-value cells are the 5 swaps whose read locked the distractor
value; the remaining 4 read the right value and never emitted it. **That split was not re-measured here**:
re-running the probe is a second full 214M generation on the laptop, which the owner's no-laptop-CPU-
fallback rule forbids, and the Step 11 result is already recorded per row on the tracker.

## Decision

**No single mechanism dominates, and no bounded intervention follows. Phase 2 is not opened.**

* By the pre-registered probe the nine split **5 / 4** across two mechanisms with different internal
  causes; by the frozen check they split **3 / 6**. Neither reading gives one named mechanism that
  accounts for the three cells needed.
* The majority class's only on-point lever has already been measured **on this exact base and this exact
  panel**: Step 8 arm P is the same pretrain and the same fine-tune plus
  `read_binding_supervision=0.1 read_binding_labels=… read_binding_source=dialogue-recall-v2
  read_binding_layer=14`, and it scored **29/40** against arm Q's 31/40 — a two-cell loss, rejected. The
  Step 11 record adds that swaps were unchanged by 7a/7c/7d/9 at 96M too, and that a re-test at 214M would
  need "a new causal handle". A config delta against the majority class is therefore not a new
  experiment; it is a repeat of a measured negative.
* The read-out line on this panel is already declared finished (run-length stop 65/84, token-identity stop
  72/84, span-open relaxation with an empty target population, span extraction negative, answer-span
  supervision inapplicable to the recorded corpus). A serving-time rule cannot reach a value that is
  never emitted, and it cannot distinguish the correct value when the read locked the distractor.

Frozen for the record: **success would have been ≥ 34/40 `check_pass` on this panel with the 31/40
baseline re-measured alongside the candidate, per-category reporting, and any D11/export impact measured.**
No candidate was trained, so none of that was spent. A negative is the result.

## D11 / export impact

Not assumed away, and not claimed as measured either. The artifact is a **float pointer model**
(`pointer.dim=32`). The training tool's own recorded scope for that head
(`crates/uor-r4-training/examples/geometric-stack.rs`, the `pointer` scope line) states: *"Neither a flock
nor a pointer selection, route or identity term has a D11 port; `export` writes a pointer that keeps every
source (both integer engines serve its mixture), and `qat=true` refuses a pointer."* So this model's
selection cannot be served as selected, and **no QAT/4-bit artifact can be built for it at all**, which is
the path criterion 2's `≤ 0.90 BPB` served target is measured on. Running `export` was not attempted here
(it would need a second local build of the large example binary); if the line is ever revived, exporting
this model and measuring what the served mixture actually scores is the first thing to do, and the
mechanism should be designed as a serving-contract instrument from the start rather than evaluated as
float recall and ported afterwards.

## Limitations

* **The v4 panel is a development panel from here.** Its own freeze statement says replies are not to be
  inspected row by row before the Step 6c acceptance run and that after that run v4 becomes a development
  panel. Criterion 3 requires exactly that inspection, and the ladder's Step 11 probe already inspected
  the misses. So this 31/40 and this failure classification are development evidence; a held-out
  acceptance number for criterion 1 would need a freshly frozen panel.
* One artifact, one fine-tune seed. Step 11 measured seed 2 of the same arm at **30/40**, so the arm's
  spread is 30–31, not a fixed 31.
* The raw pretrain `e448de86`, `bases/planA-214m-2b425ccd` and `models/chat-214m-planA-af-b8` were **not**
  scored on this panel; scoring any of them is a separate 214M generation. The README rows that attributed
  31/40 to the base checkpoint are corrected to say what is recorded rather than to state a second
  number that was never measured.
* The 5 / 4 probe split is quoted from the recorded Step 11 result, not re-measured here (see above). The
  3 / 6 check-level split is measured here on both the sealed and the regenerated replies.
* Grader verdicts are not perfectly stable: 4 of 64 rows differ between the pod run and the laptop re-grade
  with the same `qwen2.5:7b` digest. No headline number depends on those rows, which is the argument for
  reporting the deterministic `check_pass` first.

## Cost

**Zero pod spend. Nothing was leased.** Laptop only: 4 min 53 s release build
(`CARGO_TARGET_DIR=/tmp/cargo-v4`), 11 min 1 s Ollama re-grade, 17 min 30 s CPU regeneration of the 64
replies. Artifacts pulled to the laptop and used in place: the two 214M bases (856 MB each, hashes
re-verified), `data/tokenizer.json.tar`, and the 3.4 GB Step 8 iCloud archive (only arm Q's model and its
three sealed reports were extracted from it).

## Next

The v4 memory line is closed at **31/40 `check_pass` / 27/40 `acceptable`** with no bounded lever: the
failures split across two mechanisms and the majority class already carries a measured negative on this
base. Criterion 1's remaining work is therefore **not** a tuning exercise on this panel. What would change
the answer, in the order worth doing it: (1) if criterion 1 is to be pursued at all, score the base that
is actually on the serving path (`chat-214m-planA-af-b8`, or the 19.9M served stack) on a *freshly frozen*
memory panel and take that as the baseline, because v4 is now development evidence and Plan A has no
recorded memory number; (2) design the next memory mechanism as a serving-contract instrument from the
start — this artifact has no QAT port, so a float recall win on it cannot become a D11 result; (3) treat
the open reply panel (36/232 `acceptable` against a 116/232 target) as the hard half of criterion 1, since
the memory panel's three-cell gap is the small part of the acceptance bar. Any candidate must report
`check_pass` and `acceptable` separately, per category, with the failures named.
