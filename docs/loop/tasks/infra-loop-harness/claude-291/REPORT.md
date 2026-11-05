# REPORT: infra-loop-harness, claude-291 (register real-text-corpus)

Lab: claude. Order: [ORDER.md](ORDER.md). PR #3731. Plan step: the decided order of 4 Nov (#2028 6634218223), the real-text corpus before P5 on real text. References #2904.

**Speed.** No speed change: this unit registers an item, and nothing that trains or serves changes.

## What did not work, first

- **The render and the item's issue could not be made.** `roadmap check` fails on `origin/main` (`3f08ccfd1`) with 2 problems, both in another lab's item: `infra-desktop-client-tauri.md` has status `in_progress`, which is not one of ready, held, decision and done, and `issues.json` maps `infra-desktop-client-tauri` to no item. `render` and `sync` run the same check, so neither runs.
  - **What is owed:** `docs/roadmap/README.md`, `issues.json` and the item's issue (`roadmap sync --apply`), as soon as main's check is green. Kimi was asked on 5 Nov (#2028 6653582249), and has not answered yet.
  - **Why the item can still be claimed:** in the loop harness, only `loopclose.rs` reads `issues.json`, and a missing id there means no `References` line (`loopclose.rs:446-457`). xtask's `roadmap` and `roadmap_sync` read it too, and those are the commands that cannot run. Kimi registered `m7-lexlean-studio-tools.md` the same way in #3730.
- **The draft item had two faults that the checker would have refused, or that would have misled the first unit:**
  - `area: data`, which is not one of the seven areas `xtask/src/roadmap.rs` allows. It is now `training`;
  - it pointed the first unit at `knowledge_corpus::PanelExclusion` as code to compile back in. That module imports `EXCLUSION_NGRAM` from `milestone_world_v2_probe`, which is compiled out too (`lib.rs:171-172`). The item now carries the rule into the new `real_text.rs`.
- **My first item described only half of the exclusion rule,** found by the fresh fact-check. It named the 8-word windows and left out the rule for short panel turns: a turn of 4 to 7 words is matched as a whole word sequence, and a shorter one is counted as uncovered (`knowledge_corpus.rs:4-18`, `:34`). A unit porting only the windows would have left every short panel prompt unguarded, while done-when 3 still passed on the token census. The item now names both parts, and done-when 1's tests pin them.
- **Nothing was built or measured.** The corpus, its panel and its census are the item's two units.

## What changed

- **New: `docs/roadmap/items/real-text-corpus.md`,** `status: ready`, `needs_gpu: false`, `milestone: research`. It asks for:
  - the builders, in a new `real_text.rs` and one bin, `prepare-real-text`:
    - OASST2's trees to English conversations, with deleted, unreviewed and synthetic messages dropped;
    - Wikipedia's parquet to BOS-opened documents;
    - the split by tree id and article id;
    - the tokenizer pin, refused on a wrong SHA;
    - the exclusion rule against panel user turns, both its parts;
  - the corpus and a held-out panel, built on the laptop, with manifests that record source, revision, licence, attribution, the counts, the drops and bytes per token;
  - the census against DEV, Fresh V2, P-dev, p-long and p-long4k;
  - storage in the private store, read back, with no corpus committed;
  - a notebook row and a REPORT.
- **Its scope** is the training crate's `Cargo.toml` and `lib.rs`, the new module, bin and tests, the prep sheet, the item, its task folder and the `data-tokenizer` notebook.

## Measurements

None. This unit makes no run.

## Checks run at the head

- **`roadmap check` on the branch:** exit 1 with exactly main's 2 problems, and none in `real-text-corpus.md` (local `roadmap-check-1.log`).
- **Negative gate:** with the item's status set to `planted-fault`, the same check reports 3 problems, the third being `real-text-corpus.md: status planted-fault is not one of ready, held, decision, done`. Restored; the file's SHA-256 matched its value before the plant (`1f4534cd…`).
- **Cancel probe:** no item names OASST, Wikipedia, real-text or human-written; the only open PR was this one; every scope path exists on `origin/main` except the files the units create.
- **The fresh fact-check** of the item and the ORDER against `origin/main`, given only the spec and the diff: APPROVE WITH CHANGES, three minors and two nits, all applied:
  - the exclusion rule's second part (above);
  - §3.5 does not say data preparation is allowed on the CPU. It says research compute is GPU-first for training, qualification and the model and parity executions. The rule now says why building a corpus falls outside it;
  - `open_to: lab`, so outputs go to `results/<lab>/` and `local/<lab>/`, not claude's folders;
  - xtask reads `issues.json` too; `loopclose.rs`'s lookup is `:446-457`;
  - Wikipedia gets only the census's n-gram counts, and OASST2's reply rule now covers unranked replies and ties.

  It verified every other line number, path, count and front-matter value it checked.

## Skills used

None applied.

## Still open

1. **The render, `issues.json` and the item's issue,** once `roadmap check` is green on main.
2. **The corpus units:** claim `real-text-corpus`. Unit 1 builds the builders with tests. Unit 2 builds the corpus, the panel and the census, and stores them.
3. **Then P5 on real text,** through its sheet unit's decision card.

## Plain summary

The next step in the plan is a training set of real human writing: volunteer chat conversations (OASST2) and Wikipedia articles, with no machine-written text. This unit writes that work down as a tracked roadmap item, checked line by line against the code, so the next unit can claim it and build it. One part of the usual paperwork, the rendered roadmap page and a GitHub issue for the item, could not be made today. Another lab's item has a status the checker refuses, so the page will be rendered once that is fixed.
