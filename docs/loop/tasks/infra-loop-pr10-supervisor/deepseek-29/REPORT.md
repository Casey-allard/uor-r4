# REPORT: infra-loop-pr10-supervisor, deepseek-29 (repair: whole-argument model pin and the note lows)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Setup row: 10. Base: `5864dc83b`. Item PR repair; the item stays `ready`.

## What did not work, first

- **The model pin was a substring match.** `--model deepseek` passed against `deepseek-flash`, `--model prompt` passed against the `{prompt}` placeholder, and only the empty string was refused. It now matches whole arguments only: the exact argument, `--model=<model>`, `-m=<model>`, or the value after `--model`/`-m`; the negative cases Claude named are tests.
- **The CLI target choice had no test.** The resolution is extracted as `note_issue` (#<n> target → n, else #2028) and tested; the `notice` closure calls it. The failed-post-to-stderr path remains covered only by reading the code, which is the LOW Claude noted.
- **Two notes in one second overwrote each other.** The writer now finds the first free `note-<time>[-n].md` name; a second note in the same second keeps its own file (asserted).
- **The fake-`gh` test leaked its temp root and never read the kept copy.** It now asserts the kept note content and removes the root; a second post proves distinct files.
- **Unchanged and deliberate:** the disputed stop-card clause is still on the #2028 decision card; the item is not flipped. No GPU, no owner path, no spend.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `crates/uor-r4-harness/src/loopdrive.rs` | code | +~40/−5 | whole-argument `model_in_launch`, `note_issue`, collision-free note paths, CLI target via `note_issue` |
| `crates/uor-r4-harness/tests/loop_drive.rs` | test | +~55/−8 | prefix/placeholder/`=`/bare-value pin cases; `note_issue` cases; kept-copy content, temp-root cleanup and two-files-in-one-second assertions |
| `docs/loop/tasks/infra-loop-pr10-supervisor/deepseek-29/{ORDER,REPORT}.md` | docs | small | this order and report, plus receipts |

**Size:** about 100 changed lines (ORDER estimate about 90, 1.1).

## Measurements

- **`cargo test -p uor-r4-harness --test loop_drive`:** 22 passed (was 21; the model test grew, `note_issue` and the duplicate-note check are new).
- **Mutation checks:** removing the whole-argument match or changing the kept name makes the new assertions fail; the executed gate set is in the receipts.
- **Clippy `-D warnings` for the harness:** run before the review request.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check` and `cargo clippy -p uor-r4-harness --all-targets -- -D warnings`: invoked below.
- `cargo test -p uor-r4-harness --test loop_drive`: 22 passed.
- `UOR_LAB=deepseek uor-harness loop gates --full`: nine GREEN; `--break all`: nine CAUGHT.
- `uor-harness loop report --check` and `loop pr-check --local`: PASS at the receipt head.

## Skills used

- **solution-integrity**: the pin is proved by negative cases Claude named, not by the happy path; the docs keep the honest limit (the pin source is still the launcher until the owner registry stores model ids).
- **scope-guard**: only the supervisor source and its tests changed in this repair.

## Still open

- **The stop-card clause** (supervisor stops only vs every self-exit) remains on the #2028 decision card; once decided, a small follow-up adds it and flips the item.
- **The model pin source** stays the launch command until the owner registry carries model ids.

## Plain summary

- **The model pin now means what it says.** Only a whole-argument match counts (exact, `--model=x`, `-m=x`, or the value after the flag); the prefix and `{prompt}` tricks Claude demonstrated are negative tests now.
- **The note lows are folded in:** target choice is a tested helper, note files cannot collide within a second, and the fake-`gh` test proves the kept copy and cleans up.
- **The item stays `ready`**; the remaining clause is a decision card, not a status flip.