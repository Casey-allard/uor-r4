# REPORT: infra-loop-pr08-migration, kimi-9

Lab: kimi. Order: [ORDER.md](ORDER.md). References #2904. Closes #2913.

## What did not work, first

- **I clobbered two existing files on the first pass (repaired before push).** My `milestones.json` edit re-serialized the whole file (indent 2 → 1, 707 churn lines), and my `pm-rules.md` write replaced an existing owner-ratified 393-line file (2026-10-13) I had never read — the probe's `ls` output listed it and I misread the listing. Both restored byte-identical and redone surgically (the M1 headline edit is now one block; pm-rules.md is untouched, and the "duplicates removed" step is the pointer rows elsewhere). Fixed in the repair commit `4c7970ea7`.
- **The first NOW.md move wrote the pointer into the archive copy.** The first `git mv` failed (an earlier same-name retirement holds `archive/integration/NOW.md`), and my stub write landed before the re-move, so the dated archive copy briefly held the stub text. Caught by the byte-identity check against the base; repaired (the dated copy now holds the exact pre-move content, verified by diff).
- **The lesson keying is blocked by a rule conflict I did not expect.** The renderer reads keys as `id: L-NNN` (looprender.rs `lesson_id`), but the mistakes registry's kebab-case rule (mistakes.rs:148-150, `^[a-z0-9][a-z0-9-]{2,63}$`) refuses uppercase ids, and its id==stem rule blocks any other arrangement. The content is staged and verified in the worktree (L-001 created from the loop page's committed record; L-002..L-024 renamed in date order), but commits only after the scope widening carded on #2028 (add `crates/uor-r4-harness/src/mistakes.rs` to this item's scope) lands and the validator accepts `^L-\d{3}$` equal to the stem. Until then `uor-harness check`'s `mistakes.registry_valid` errors on the keyed files — documented, not hidden.
- **One transient gate red.** `loop-drive` failed once (exit 101) inside the gate runner's scratch worktree; it passes 18/18 on a direct run and on every re-run. Recorded as a flake to watch, not chased.
- **The `loop-migrate` gate's first break was REFUSED** ("red without 'broken link'"): the planted fault disabled the link check, and my test's exit-code assert fired before the message assert, so the declared failure text never printed. The test now asserts the message first; the fault is CAUGHT with "broken link".

## What changed

- **Retired the hand-written notes (§4.7):** `STATUS.md` → `archive/STATUS.md` and `docs/integration/NOW.md` → `archive/integration/NOW-2026-10-22.md` (dated suffix; an earlier retirement holds the plain name), both verified byte-identical against the base, one-line pointer stubs left at the old paths, two `status-snapshot` entries appended to `archive/catalog.jsonl`, and `archive/CATALOG.md` regenerated.
- **PLAN.md (root, PM-written):** the goal, the `## Premises` table (Premise/Probe/Status headers matching the deepseek-25 reader, two premises closed by today's proofs/ruled items, five open with probes and cancel lines), the `## Owner directions` table (Direction/Status/Date/Note headers matching the deepseek-27 reader; three done directions recorded verbatim with times; pending owner *answers* are noted as never entering this table — they are the cards on #2028), the backwards M1–M8 route, the approved sequences (the W-sequence, the K16 follow-ups, the build plan), and the not-doing list.
- **AGENTS.md (owner path, approval recorded above):** the `loop:lessons-top-ten` markers with the rendered block between them, the `NOW.md` table row re-pointed to PLAN.md, pm-rules.md and LESSONS.md rows added, the workspace row dropped, the two "How to work" opener bullets merged — 76 lines, under the 80-line gate.
- **`docs/harness/milestones.json` reconciled to CHARTER §4.8:** M1's headline now reads the closing bar (D11-export integer top-1 beats the unigram 1,226 and T2 6,300 tables of 14,287 on DEV, two seeds, tokens/s reported), with the R14-blocked reply bar noted as blocked-not-closing and the D39 interim admission headline noted as interim. Surgical edit, original formatting preserved.
- **`docs/start/FILEMAP.md`:** one row per tracked directory two levels deep (~115 rows).
- **`docs/harness/claims.json` dead-entry cleanup:** all 67 committed entries named branches the merge queue had already deleted from GitHub; the file now reads `{}` and the live tier carries today's active claims. The registry-gap code question (pr-check's allowlist) stays carded for the loop architect.
- **The verifier:** `uor-harness loop migrate --check` (new `loopmigrate.rs`, registered in `lib.rs`/`loopgates.rs`/`doccheck.rs`) checks every migration invariant — retired-note pointers + byte-identity via git history, AGENTS markers, L-NNN keys, DECISIONS columns, FILEMAP coverage, read-path budgets, and link resolution over the migrated views — with six integration tests (`tests/loop_migrate.rs`) and a `loop-migrate` gate whose planted fault (the link check disabled) goes red with "broken link".
- DECISIONS.md verified already carrying the two columns ("Alternatives turned down", "Cost to undo") — no edit needed.

## Measurements

- `cargo test -p uor-r4-harness --test loop_migrate`: 6 passed, 0 failed (complete fixture passes; each broken invariant fails named).
- `loop gates --full`: 10/10 GREEN at the head (including the new `loop-migrate` gate, 15.3s); `loop gates --break loop-migrate`: CAUGHT, red with "broken link".
- `uor-harness loop render --check` at the head: STATE/HANDOFF/LESSONS/AGENTS-block byte-match a fresh render.
- The retired-note byte-identity: `diff` against `fb6df367b` shows both archive copies identical to the pre-move content.
- Read path at the head: AGENTS.md 76 lines; STATE.md ~4.9 KB, HANDOFF.md ~18 KB (under the 24,576 B each); four-file read path ~53 KB (under 76,800 B).

## Checks run at the head

- `uor-harness loop gates --quick` and `--full` — GREEN (10/10).
- `uor-harness loop gates --break loop-migrate` — CAUGHT (red with the declared text).
- `uor-harness loop report --check`, `uor-harness loop pr-check --local` — recorded at push.
- `cargo test -p uor-r4-harness` — full suite result recorded at push.
- `cargo run -q --locked -p xtask -- roadmap check` / `render` — recorded at push.
- `cargo run -q --locked -p xtask -- links --baseline … --renames-from …` — the forward references to PLAN.md/STATE.md/HANDOFF.md resolve post-migration; result recorded at push.
- `uor-harness check` — `mistakes.registry_valid` errors on the keyed lesson files until the carded parser extension lands (expected; documented above).

## Skills used

- `scope-guard`, `solution-integrity`, `working-discipline` (the loop's private skills; no kit text in this repo).

## Still open

- The mistakes-parser scope widening (carded on #2028, first taker) and then the `L-NNN` validator acceptance in this PR, the staged lesson renames commit, and the top-ten block's first populated render.
- PLAN.md's premises are a PM first draft; probes and cancel lines sharpen as the runs they name register their sheets.
- The owner-directions table lists only answered directions; the seven pending bar cards stay on #2028 until the owner answers.

## Plain summary

The loop's shared notes used to be hand-edited pages that drifted. This PR retires STATUS.md and NOW.md into the archive byte-for-byte (with pointer signs left at the old addresses), writes PLAN.md as the PM's one plan page with the goal, premises, owner directions, the backwards route and the not-doing list, commits the first machine-rendered STATE/HANDOFF/LESSONS views (stamped so a hand edit fails a re-render check), teaches AGENTS.md the lessons top-ten block within its 80-line budget, reconciles the M1 headline with the charter's own closing bar, keys every paid-for lesson permanently (pending one parser line), maps every directory in FILEMAP, clears the dead branch-claim registry, and adds `loop migrate --check` so all of it is provable by anyone, with a planted broken link turning its gate red.
