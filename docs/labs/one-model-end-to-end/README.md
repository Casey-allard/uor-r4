# one-model-end-to-end: the census, the held-out-phrasing panel and the overturn check (claude-202, claude-204, claude-205; 29 Oct 2026)

## The capacity sweep's served evidence (claude-259, 2 Nov 2026)

[`capacity-2026-11-02/`](capacity-2026-11-02/README.md) holds what the laptop served from the capacity sweep's six exports: the conversations, the transcripts, the decode timings, the parity summaries and a trace of the copy read. The [REPORT](../../loop/tasks/one-model-end-to-end/claude-259/REPORT.md) reads them.

## The CPU evidence lane (deepseek-1, 2 Nov 2026)

[`instrument/`](instrument) also builds a CPU-only `m2-evidence` command. It evaluates a discrete integer artifact on a canonical UORT panel and separates next-token accuracy into prompt-overlap, first-8 key, reply-overlap and opening/continuation strata. It reads an optional TRAIN-format split as the reference index. It also recomputes the writer independently from the serving Q12 query. It does not require a GPU and does not edit the serving engine.

```sh
cd docs/labs/one-model-end-to-end/instrument
cargo run --release --bin m2-evidence -- \
  artifact=MODEL.d11 \
  tokenizer=TOKENIZER.json \
  panel-tokens=PANEL/tokens.u16 panel-mask=PANEL/response_mask.u8 panel-manifest=PANEL/manifest.json \
  train-tokens=REFERENCE/tokens.u16 train-mask=REFERENCE/response_mask.u8 \
  train-manifest=REFERENCE/manifest.json train-role=generated_control \
  max-targets=1024 out=NEW_REPORT.json
```

The report binds artifact and corpus manifests, tokenizer and content hashes; counts admission, top-1 and top-8; reports exact and value-masked prompt/reply overlap, reply 4-/8-gram novelty and a new-template stratum at K = 1,024; and marks missing controls or data as unavailable. It distinguishes `canonical_train` from `generated_control`; the role is required when a reference index is supplied. The current 19-table M1 smoke has exact writer parity from the serving Q12 query. Full two-sided CPU-trunk parity remains unavailable because the CPU research path does not implement the 15 additional M1 trunk tables. The first public artifact/panel smoke run is in the [deepseek-1 REPORT](../../loop/tasks/one-model-end-to-end/deepseek-1/REPORT.md).

## The overturn check (claude-205)

**The question.** Does the trunk that learned to lean on the turn memory read worse than one that never had it, on the rows the memory names none of? The rows are DEV's 3,406 unnamed later rows and 32 unkeyed openings, 3,438 pooled. It is done-when 2, under [its sheet](../../harness/prep/one-model-end-to-end-overturn-2026-10-29.md) (FIT, PRE-RUN PASS).

**How it ran.**
- **On the GPU:** `train-in-space --score-rows CHECKPOINT --out DIR` (`in_space_long::score_rows`) scored claude-194's step-16,384 (r) and (r0) at both seeds. It wrote every DEV and Fresh V2 row's nearest and writer pick, with the memory on and taken out, and 64 greedy samples.
- **On the laptop:** `train-in-space --overturn 1 --scored ARM-sSEED=DIR ...` (`in_space_long::overturn`) graded those files into [overturn-2026-10-29.json](overturn-2026-10-29.json).
- **The outputs** are in the HF store at `results/claude/one-model-end-to-end-2026-10-29/overturn-b6f7bdc9/`.

| Seed | (r) | (r0) | b | c | d | Floor `⌈2√(b+c)⌉` | Band | Cell |
|---|---|---|---|---|---|---|---|---|
| 1 | 2,078 | 1,530 | 999 | 451 | +548 | 77 | 915 | inside |
| 2 | 1,929 | 2,296 | 273 | 640 | −367 | 61 | 915 | inside |

**Verdict: mixed.**
- **What the seeds say.** They disagree in sign, each past its own paired floor (77 and 61). The two-seed spread, 915, is larger than both floors, so it is the band, and both seeds fall inside it.
- **What it changes** (the registered outcome). The training run carries (r) and a memory-dropout arm as equal candidates. The REPORT also recommends at least three seeds; that is advice, not part of the rule.
- **The details** (free generation, the writer's readout, instrument checks) are in the [REPORT](../../loop/tasks/one-model-end-to-end/claude-205/REPORT.md).

## The held-out-phrasing panel (claude-204)

**How the panels were made.** The writer is [`instrument/`](instrument) (`heldout-panel`), a standalone tool calling M-world v2's live library, `MWorld2::conversation_in` at the default mix, in dialogue protocol 2, with no probe screen. It produced three panels of 400 conversations each. They are in the HF store at `inputs/claude/heldout-panels-2026-10-29/` with `SHA256SUMS`, and counted by `train-in-space --census 1 --panel NAME=DIR` into [census-panels-2026-10-29.json](census-panels-2026-10-29.json).

| Panel | Cell, seed | Tokens / targets / prompts | `tokens.u16` sha256 |
|---|---|---|---|
| P-dev | `dev,dev`, 2041 | 51,117 / 16,417 / 1,443 | `790b7e12…` |
| P-ret | `train,dev`, 2042 | 49,399 / 16,266 / 1,422 | `b4d3bf42…` |
| P-train (control) | `train,train`, 2043 | 48,983 / 16,222 / 1,425 | `7546c6d7…` |

| Overlap with TRAIN | DEV | **P-train** (control) | P-ret | **P-dev** |
|---|---|---|---|---|
| Prompts equal to a TRAIN prompt | 0.608 | 0.608 | 0.541 | **0.000** |
| Masked prompt templates, K = 256 / 1,024 | 0.644 / 0.642 | 0.644 / 0.638 | 0.604 / 0.585 | **0.049 / 0.000** |
| Prompt 8-grams / 13-grams | 0.521 / 0.282 | 0.527 / 0.287 | 0.478 / 0.244 | 0.159 / 0.006 |
| Replies equal to a TRAIN reply | 0.750 | 0.726 | 0.634 | 0.640 |
| Masked reply templates, K = 256 / 1,024 | 0.781 / 0.766 | 0.768 / 0.757 | 0.731 / 0.710 | 0.726 / 0.705 |
| Reply 8-grams / 13-grams | 0.605 / 0.367 | 0.626 / 0.470 | 0.558 / 0.388 | 0.569 / 0.393 |
| Openings keyed by TRAIN's first-8 table | 1,194 / 1,226 | 1,365 / 1,425 | 1,332 / 1,422 | 245 / 1,443 |
| **New templates** (K = 1,024 prompt share ≤ 0.5) | no | no ✓ | no | **yes** |

**What it shows.**
- **The control passes, and DEV looks like the generator's train cell.** P-train, drawn as TRAIN was, matches DEV within about 0.01 on every prompt measure. That fits DEV's manifest, which says `split: train`.
- **P-dev holds new templates.** None of its prompts, and no masked template at K = 1,024, occurs in TRAIN. 13-gram overlap is 0.006.
- **Its replies still overlap TRAIN's** (0.640 verbatim), as claude-203 predicted: M-world v2 splits user-turn phrasings only, and reply templates are shared. **So P-dev tests reading a new prompt, not writing a new reply.**
- **The two named risks are small.**
  - Near-duplicate paraphrases: P-dev's prompt 8-gram share is 0.159 and its 13-gram share 0.006. The 8-gram share includes generic phrases and the role markers.
  - chat-v0 coincidences: 0 exact prompts.
- **245 of P-dev's openings are keyed** by TRAIN's first-8 table. A turn key's first 8 ids include the "User:" marker, so common short openings collide. Those rows are the panel's keyed stratum, and the other 1,198 its unkeyed stratum.
- **The panel writer's first draft failed the control.** Under protocol 1, P-train's exact-prompt share was 0. TRAIN is in protocol 2 (the role marker's space merged into the first word), and the panels above are all protocol 2.
  - Each document passes `corpus_v2`'s own checks: the spaced meter matches the protocol-2 length, and every document fits the 256-token context.
  - The panels written with those checks are byte-identical to the ones in the store.
- **The missing probe screen can only lower P-train's and P-ret's shares, never P-dev's.** TRAIN redrew conversations that share an 8-gram with the sealed probe. The unscreened panels may hold some, and TRAIN never saw those. P-dev's 0.000 cannot go lower, and P-train still matched DEV.

# The data census (claude-202, 29 Oct 2026)

Step 1 of [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md) (#3407), from the [assessment](../../loop/tasks/infra-loop-harness/claude-201/ASSESSMENT.md).
- **How it ran:** a count with no model and no GPU. `train-in-space --data-root DIR --fresh-v2 DIR --census 1` (`crates/uor-r4-float-train/src/in_space_long.rs`, `census`) took 4.2 s and 1.1 GB on the M1.
- **What it read:** TRAIN (`mw-balp/train`) and DEV (`ft/dev`), from the HF store's `w2-inputs/2026-10-25/driver-data/`, under their sheet pins; and the sealed Fresh V2 panel, from its preservation archive under its pins (`0b7e754c…`, `a1cda529…`), counted only.
- **What it did not open:** `m2-fresh256` (M2's sealed acceptance panel) and the K16 fresh slice.
- **Output:** [census-2026-10-29.json](census-2026-10-29.json). It holds counts and DEV corpus positions only, never token text.

## The instrument check (M1): every pinned count, exactly

| Count | Pinned (`float-training.md`, rows 14–15) | Census |
|---|---|---|
| DEV openings | 1,226 | 1,226 |
| Openings whose first-8 key TRAIN holds | 1,194 | 1,194 |
| Later rows the turn-8 + position + reply-last-1 table names at share 0.5 | 9,655 | 9,655 |
| … named right | 8,822 | 8,822 |
| Reply-only control: most later-row hits among no-turn keys with precision ≥ 0.83 | 8,089 | 8,089 (reply-last-4, share 0.5, 8,089 / 8,982 = 0.900) |

## The sizes (M0)

| Split | Bytes | Tokens | Eligible targets | Documents | Prompts | Replies |
|---|---|---|---|---|---|---|
| TRAIN | 15,025,658 | **7,512,797** | 2,425,925 | 60,000 | 212,968 | 212,968 |
| DEV | 85,776 | 42,856 | 14,287 | 400 | 1,226 | 1,226 |
| Fresh V2 | 89,420 | 44,678 | 14,479 | 400 | 1,279 | 1,279 |

TRAIN's file is its 64-byte header plus 2 × 7,512,797 bytes. This is the sha pinned at `hamilton_sheet.rs:42–43`.

`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:62` gives 26,214,400 tokens and 10,975,281 eligible targets for the same sha. The census finds 7,512,797 tokens and 2,425,925 targets. The origin of the sheet's figures is not established: they match neither this file's header nor its payload. (26,214,400 is 25 × 2²⁰.) That sheet's own counts are a question for its unit, not for this one.

## How much of each panel TRAIN already holds (M3–M6)

| | DEV | Fresh V2 |
|---|---|---|
| Replies equal to a TRAIN reply | **0.750** | **0.777** |
| Prompts equal to a TRAIN prompt | 0.608 | 0.639 |
| Reply 8-grams / 13-grams in TRAIN replies | 0.605 / 0.367 | 0.655 / 0.460 |
| Prompt 8-grams / 13-grams in TRAIN prompts | 0.521 / 0.282 | 0.524 / 0.291 |
| Value-masked prompt templates in TRAIN, K = 256 / 1,024 | 0.644 / **0.642** | 0.668 / **0.665** |
| Value-masked reply templates in TRAIN, K = 256 / 1,024 | 0.781 / 0.766 | 0.810 / 0.798 |
| Openings whose first-8 key TRAIN holds | 1,194 / 1,226 | 1,234 / 1,279 |
| **New templates** (K = 1,024 prompt share ≤ 0.5, fixed in the ORDER) | **no** | **no** |

**Step 2's decision rows (M2):** DEV's 3,406 later rows the reply table does not name at share 0.5, and its 32 unkeyed openings. That is 3,438 positions, pooled, listed in the JSON. Each is the corpus position of the row's *target* token, and all are distinct.

**How the shares count:**
- Exact shares count runs: replies, or prompts.
- n-gram shares count every window of n tokens inside a run, repeats included. A run shorter than n has no window. For example, only 2,277 reply 13-gram windows exist in DEV; the JSON gives each count.

## What it means

- **Three quarters of the *replies* in both panels occur verbatim in TRAIN** (0.750 and 0.777). Fresh V2's "0 exact full TRAIN/DEV document overlap" holds for whole documents, not for turns.
  - This counts replies, not scored tokens. Long replies are less often verbatim: reply 13-gram shares are 0.367 and 0.460. So a share weighted by tokens would be lower.
- **Neither panel qualifies as a held-out-template panel by the ORDER's rule.** That verdict does not depend on how rough the value mask is. The mask is a fixed function of each run, so a run that matches exactly also matches after masking, and the masked share can never fall below the exact share. The exact prompt shares alone, 0.608 and 0.639, already exceed the 0.5 bar.
- **About a third of each panel's prompts have masked forms TRAIN does not hold** (0.358 for DEV, 0.335 for Fresh V2). They are rows a reading of the prompt could still be tested on. The item's next unit builds a panel with held-out templates, with its own census, before the training run's protocol is written.
- **The panels can still test new values inside known templates.** The step-2 check runs on DEV's 3,438 unnamed rows as planned.
