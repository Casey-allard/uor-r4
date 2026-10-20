# #2115 — Plan: making a numeric value addressable — Options A and B costed, with the test and its falsifier (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T00:43:39Z
- **Updated:** 2026-10-10T03:23:24Z
- **Closed:** 2026-10-10T00:45:04Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2115
- **Merged:** 2026-10-10T00:45:04Z
- **Merge commit:** 8e9d5486c2f80f63f20afd5e0f346b97965182fb
- **Base / Head:** main / UOR-Foundation:numeric-addressability-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Planning only — $0, no pod, no training, no model run, CPU only

Record: [`docs/labs/numeric-addressability-plan-2026-10-09/README.md`](docs/labs/numeric-addressability-plan-2026-10-09/README.md). This document is the deliverable.

## Background: three readings were tested and two died

Preserved as structure, not compressed to a conclusion — the third is only worth acting on because the first two were killed.

| reading | status | what killed it |
|---|---|---|
| Token count explains the cluster | **REFUTED** | numbers tokenize in **2 tokens against a 3.74 mean** for words — length runs the wrong way |
| Minimal pairs / digit order | **REFUTED** | `mem-040`'s distractor `74` is a **transposition** of its expected `84` and it **passes**; and **0 of 12 numeric failures name the distractor** |
| **Value addressability** | **OPEN, replicated control** | `mem-037`/`mem-040` share key, kind and history, differ only in value, opposite outcomes; replicated within-key — `brother` 1/6 numeric, `grandpa` 2/3 word, `piano` 0/3, `treehouse` 0/2 |

## A correction to the framing, before costing anything

The earlier framing called `¹ ² ³` reclaimable digit slots. **They are byte tokens** — `0xB9`, `0xB2`, `0xB3` — and **all 256 byte tokens are present**, so removing them breaks byte-level coverage. **Option A cannot be funded by the superscripts.** It must be funded by **measured merge retirement** (3837 merges; retire the unused tail) or by **growing the vocabulary** (100 two-digit slots ≈ 115k parameters at this config, 0.4% of 29M).

## Option A — number-aware tokenizer

**Scope:** vocabulary and merges. **Sizing:** 100 slots for two-digit coverage, plus one/three/four-digit coverage **measured from the corpus, not assumed** — that measurement is the plan's first task.

**What it invalidates, exactly:** every artifact trained on `d36d3e87…` — `chat-29m-B-lr5e-4`, the whole `chat-100m-*` family, and the **comparability** (not the validity) of every sealed result citing it, including the reply panel's 43/232 and v5's 10 of 40. **It is a full retrain of every rung you want to compare, not a fine-tune** — the embedding changes shape. Purely additive to non-numeric encoding **only** if funded by appending tokens; funding by retiring merges **does** change non-numeric encoding, and that must be measured per corpus.

## Option B — explicit numeric slot in the memory path

**Scope:** how a numeric value is written and read — not the tokenizer, not the embedding. **Existing artifacts stay valid.** The decoder still emits digit tokens; the memory read returns the value as an ordered unit. **Partly testable without training:** whether the memory path carries a two-digit value as an ordered unit is a **CPU-only write/read probe**; whether a *model* uses the slot needs training, because the operators are learned.

## Recommendation and the smallest decisive version

**Build the untrained memory write/read probe first** — write `84`, read it back, report whether the run is `8`,`4`, whether order is preserved, and whether `74` is distinguishable from `84`. **CPU only, no GPU, no training.** It decides between A and B: if order is preserved, addressability is refuted and neither option is the lever.

If the probe shows the value is lost, take **Option B's smallest form**: one value type at the **existing 29M rung**, against the fresh sealed panel below. **Option A is held** until the probe reports.

## The test — a fresh sealed panel, never a v5 re-run

Must contain: numeric and word rows in **matched pairs sharing key, kind and history** (the `mem-037`/`mem-040` control built in by construction); value held fixed with the distractor varied and the reverse; **magnitude spread** one- to four-digit (the diagnosis could not test magnitude on an all-two-digit panel); **position spread** first/middle/last turn; **leading zeros and repeated digits**; both value types in one conversation; deterministic checks with per-row provenance and the v5 control set verified before sealing; and **the binding swaps actually embedded and asserted non-vacuous** — v5's control was vacuous for its whole life.

**Pre-declared target, frozen before the draw:** primary **≥ 34 of 40** numeric rows at `check_pass`; and the one that actually tests the mechanism — **≥ 30 of 40 numeric rows emit a digit run equal to the stored value**, because the failure is **emission, not selection**, and `check_pass` alone could rise if the model learns to guess. **Control, non-negotiable:** `binding_swap` 40 checked rows / 0 passes, `expected_value` 40 of 40, or the panel is void.

## Falsification, stated in advance

The fix is the wrong lever if: the untrained probe shows order preserved and `84` distinguishable from `74`; or `check_pass` rises while digit-run emission does not; or numeric **and** word rows improve together; or the numeric deficit fails to track magnitude or leading zeros on the fresh panel. Each is a result; none needs a v5 re-run.

## Cost, with the GPU/CPU split

**CPU, calibrated from measured runs tonight:** panel draw and seal **minutes**; `chat-grade check` **seconds**; 64-row generation at 29M **2m04s**; 64-row grading **9m46s**; the untrained probe CPU-only and small.

**GPU via `uor-pod` (caps ≤ 4 pods, ≤ $8/h):** the ladder's training throughput is **not recorded at a level this plan can cite**, so a short **timed calibration run is required first** and the projection is stated as a formula rather than a number — inventing it would be the error this line has been avoiding all night.

**The economic point:** Option B leaves every artifact valid and needs **one retrain at one rung**; Option A **invalidates the whole ladder** and needs each rung rebuilt before any comparison.

## What does not change

**Criterion 1 remains NOT MET on both halves; 43/232 unchanged. Nothing in this plan changes that, and no step of it would.** `check_claim_wording.py` passes; `scripts/__pycache__/` not committed; one live branch. Bundle on cloud-store: `numeric-addressability-plan-2026-10-09.tar`, 142,336 bytes, md5 `bfd775d0cb664469fdfeb8ca3797d125`.

**References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

