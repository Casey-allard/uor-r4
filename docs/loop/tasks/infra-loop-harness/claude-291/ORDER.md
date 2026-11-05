# ORDER: infra-loop-harness, claude-291

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). Base: `3f08ccfd1`.

Setup row: 2 (item-file maintenance: a new gap item).

## Goal

The item's done-when reads:

> Every PR in the build plan on [the loop page](../../loop/README.md#build-plan) is merged, each with its planted fault turning its gate red. The four labs (Kimi, Claude, DeepSeek, Antigravity) are onboarded, each with a committed onboarding receipt. Each lab has completed one loop task end to end: claim, probe, work, gates, report, review by another vendor, queue, and verify on fresh `main`.

This unit registers one roadmap item, `real-text-corpus`, so that `loop claim` can take its units. It is the umbrella's item-registration work, as in claude-280.

- **Claim check, (a) the step:** the decided order (#2028 6634218223) puts a human-written real-text corpus, 2 CPU units, before P5 on real text. P4 is done (claude-287, claude-288), and P6 reached its 2-unit bound with claude-290 (#2028 6660311849), so the corpus is next.
- **Claim check, (b) the model:** this unit changes nothing that trains or serves. It unblocks the corpus units, whose corpus and held-out panel P5 trains and scores on, so that P5's scale slope is read on real text rather than on the templated M-world.

## Cancel probe (run first)

- **Does an item already cover this?** Search `docs/roadmap/items/` on `origin/main` for OASST, Wikipedia, real-text and human-written.
- **Does an open order hold any file in the new item's scope?** Check every open PR's file list.
- **Are the scope's paths real?** Check each named file with `git cat-file -e origin/main:<path>`; a path may name a file a later unit creates.
- **Classed:** instrument. Nothing cancels the unit unless an item already covers it.

## Files in scope

- `docs/roadmap/items/real-text-corpus.md`
- `docs/loop/tasks/infra-loop-harness/claude-291/**`

`docs/roadmap/issues.json` and `docs/roadmap/README.md` are left out: the render cannot run (below).

## Done when

1. **The item** is on the branch with the six required sections, `status: ready`, `needs_gpu: false`, and a `scope:` naming only the files its units edit.
2. **`roadmap check` on the branch reports only main's two existing problems**, both in another lab's item, and none naming `real-text-corpus`.
   - **Negative gate:** with the item's status set to an invalid value, the same check names `real-text-corpus`, so the check does read the new file. Restored afterwards.
3. **A fresh exact-head review passes.** It checks every line number, path and count the item cites against `origin/main`, and its scope against the work its done-when needs.
4. **`loop report --check` and `loop pr-check --local`** pass.

**Owed, not in this unit:** the render (`README.md`, `issues.json`) and the item's issue (`roadmap sync --apply`), once `roadmap check` is green on main. The item can be claimed without them: in the loop harness, only `loopclose.rs` reads `issues.json` (xtask's `roadmap` and `roadmap_sync` read it too), and a missing entry there means no `References` line (`loopclose.rs:446-457`). Kimi registered `m7-lexlean-studio-tools.md` the same way in #3730.

## Stop and decide (never wait) if

- the review finds a cited line, path or count wrong. Then fix the item before the merge;
- another open order holds a file in the scope. Then narrow the scope, or order the units;
- `roadmap check` names `real-text-corpus` for any reason other than the planted fault. Then fix the item.

## Cancel probe result (5 Nov, about 14:30Z)

- **No item covers it.** On `origin/main` (`3f08ccfd1`), no file in `docs/roadmap/items/` names OASST, Wikipedia, real-text or human-written.
- **No open order holds a file in the scope.** The only open PR is this unit's own draft, #3731.
- **The scope's paths are real.** Each named file exists on `origin/main`, except `real_text.rs`, `bin/prepare-real-text.rs`, `tests/real_text*.rs`, the prep sheet and the task folder, which the item's units create.
- **Why the render cannot run:** `roadmap render --check` on `origin/main` fails with 2 problems: `infra-desktop-client-tauri.md` has status `in_progress`, outside ready, held, decision and done, and `issues.json` maps `infra-desktop-client-tauri` to no item. The file is Kimi's (#3675, 4 Nov); my note to Kimi is #2028 6653582249.
- **Found while checking the draft:** `knowledge_corpus.rs` (where `PanelExclusion` lives) imports `milestone_world_v2_probe::EXCLUSION_NGRAM`, and that module is compiled out too (`lib.rs:171-172`). So the item carries the exclusion rule into the new `real_text.rs`, rather than turning two historical modules back on.
- **Classed:** instrument. Nothing cancels the unit.
