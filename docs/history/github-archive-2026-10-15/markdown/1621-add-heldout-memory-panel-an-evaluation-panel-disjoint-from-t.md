# #1621 — Add heldout-memory-panel: an evaluation panel disjoint from training

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T14:39:32Z
- **Updated:** 2026-10-09T13:17:39Z
- **Closed:** 2026-10-02T14:42:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1621
- **Merged:** 2026-10-02T14:42:00Z
- **Merge commit:** 0aa4a50c92d9f7f4b5f83b472d0f2f4d998ffc74
- **Base / Head:** main / UOR-Foundation:codex/mix-chat-corpus-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

An evaluation panel whose **relation ids, value vocabularies, question forms and statement forms are all disjoint** from the synthetic memory generator's training tables.

**Why it was needed, and what it caught.** The synthetic emitter fit was being measured on panels that changed the *test wording* but **reused the training relations and values**. That produced a number that could not be interpreted: **held-out wording 2/10 → 6/10**, reported by this lane as a gain. The generator emits ten relations and nineteen answer templates, and the fitted model scores 10/10 on panels drawn from them — so a panel sharing the template set cannot distinguish a learned **binding** from a memorised **surface**.

This panel shares nothing with training. Ten new relations — `hometown, allergy, degree, flatmate, commute_time, holiday, bank, shoe_size, vet, landline` — with new value vocabularies, asked in sentences the generator never emits, with statement forms it never emits. The fact and its competing statement are both stated, in the generator's turn shape, so it measures exactly the competing-turn binding the fit targeted, against surfaces the model has never seen.

Output: `requests.json` (a panel for `geometric-stack lut-chat requests=`), `expected.json` (id → the value a correct reply must contain), and a `README.md`, so scoring needs no hand transcription and the evaluation can be re-run against any future model.

## Exact-head evidence

**Reviewed head:** `bd3560d963a688d41724d7f2bb02f5d4d2c81739` (one commit on `origin/main` `5311e663`).

- **Compile:** `cargo check --features cpu-accelerate -p uor-r4-training --bin heldout-memory-panel` → Finished. `cargo fmt` applied.
- **Generated and run**, 40 rows, against the sealed parent and the fitted model from the same dev store, greedy, 32 tokens:

| model | disjoint held-out (40 rows) |
|---|---|
| parent (pre-fit) | **0/40** |
| fitted | **1/40** |

- **The panel does what it was built to do — it separates the two readings.** On disjoint relations the fitted model writes the right *frame* with an unrelated value: *"Your vet is Dr Bhatt."* where the panel wants `Okafor`; *"Your extension is six three zero."* where it wants `four two seven`. So it learned sentence shapes for statement-like contexts and **did not** learn to retrieve a stated value for an unseen relation. The result is a **scoped negative** on the emitter line, reported on #1512 and #820.

## Scope

An **instrument**, not a result — the finding lives on #1512/#820 and the artifact in the cloud store. It validates the *evaluation*, and it does not qualify the model: ten relations, 40 rows, one seed, greedy, 32 tokens, and the panel is synthetic so it measures in-context binding only, saying nothing about general language or memory capability.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

