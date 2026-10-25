# REPORT: infra-loop-harness, claude-88

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #3179 (the new item's issue stays open; the sheet and the run are later units).

## What did not work, first

- **The sheet cannot ride in this PR.** `loop pr-check`'s `order` rule needs an ORDER's item to exist at the merge base (`loopprcheck.rs`, the `item {id} does not exist at the merge base` check). So the sheet comes as a second unit, with its ORDER under `docs/loop/tasks/m1-w2-code-utilization/`, after this merges. This follows the item-PR pattern of kimi-27 (#3072) and deepseek-44 (#3040).
- **The sync ran before the merge.** `roadmap sync --apply` ran from this branch, so issue #3179 exists before its item file reaches `main`. Its plan was the three changes the ORDER allows:
  - create #3179;
  - refresh #2845's body, which went `done` in #3177;
  - add the child to the #2029 checklist.

  If this PR were abandoned, #3179 would be an orphan to close by hand.
- **The first item text had five faults; the fresh review at `276bd12b4` found them, and the re-review at `47dfdd572` confirmed the fixes.**
  - It credited CHARTER §3.1–§3.3 with an exact-integer rule for information measures. Those sections have no such rule, and the W2 sheet lists "code–target mutual information" as a diagnostic. The rule is now labelled as this item's own choice. The reason given is CHARTER §3.7 and the banned-presence gate, and the measure is kept in exact forms.
  - It sent admission losses to #2834 without a source. The link now quotes #2834's own "which cells open" text, and Sources lists that file.
  - It spoke of the not-yet-written sheet in the present tense ("registers"). It now says "will register".
  - The scope lacked `Cargo.toml` and `tests/w2_*.rs`, though W2's bins are declared in `Cargo.toml`. It has both now.
  - The source range for the W2 sheet stopped before row 5. It now runs to line 145.

## What changed

Size: 1 new item file (72 lines), plus 1 line in `issues.json` and 7 changed lines in the rendered page, plus this task folder. No code.

- `docs/roadmap/items/m1-w2-code-utilization.md` (new).
  - `status: ready`, `needs_gpu: false`, `done_class: host`, `depends_on: [m1-w2-capacity-capture]`.
  - **Scope:** the W2 host modules and bins (`crates/uor-r4-discrete/src/w2_*.rs`, `src/bin/w2-*.rs`, `src/lib.rs`, `Cargo.toml`, `tests/w2_*.rs`), its prep sheet, its lab folder and its task folder. This is narrower than the `crates/uor-r4-discrete/**` that the other M1 items hold, so a claim on it does not block them.
  - `notebook: docs/research/notebook/m1-discrete-attention.md`.
  - A `Cancel if` section, which names the stored capture's pins.
- `docs/roadmap/issues.json`: `"m1-w2-code-utilization": 3179`.
- `docs/roadmap/README.md`: re-rendered (shared file).

## Measurements

None. This is a docs unit.

## Checks run at the head

Recorded in the PR's review record.

## Skills used

- holding-the-goal: the M1 bar (beat 6,300) is why the item exists.
- working-discipline: arrive by checking; the repository wins.

## Still open

- The pre-registration sheet, with its non-author FIT, is unit claude-89 under `docs/loop/tasks/m1-w2-code-utilization/`.
- The diagnostic run itself is a later unit that `loop next` offers once the sheet is on `main`.

## Plain summary

W2 found that the model's 16-bit read code holds more next-word information than the trained model uses, but less than a plain previous-word table. Its sheet says the next step is to measure how well the code is used before changing how the model learns. This PR adds that measurement as a roadmap item (#3179) so the loop can hand it to a lab. It changes no code and runs nothing.
