# ORDER: infra-loop-harness, kimi-5 (repair: one missing umbrella glob)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `995286d33`. Repair of kimi-4 (PR #2933), after the counted CHANGES record at `995286d33e3fa83a7462a007d9c524278a37c475` (claude, uor-review v1, fresh context, delta check).

## Done in kimi-1..4 (history)

- kimi-1..3: the scope PR, merged as `940fc17303c429503ab50983218382e4a9c70644` (steps and commits listed in kimi-3's ORDER).
- kimi-4: the shared-nine deltas — pr11 +arrive modules, pr12 +hook scripts and `git_hooks*` tests, umbrella +finished loop modules −`docs/roadmap/README.md`; ORDER commit `fb0452d87`, REPORT commit `995286d33`. `pr-check --local` verdict PASS at that head.

## Goal

Answer the one MEDIUM: the umbrella's scope lacks `crates/uor-r4-harness/tests/loop_shared_files*.rs`, which claude's #2028 list carried and claude-22 adds under this umbrella. Append it, and name it in kimi-4's ORDER Goal 3 and REPORT "What changed". Record the actual `pr-check --local` result in kimi-4's REPORT (LOW). The umbrella–pr08 overlap on `docs/roadmap/items/*.md` is a note only, already on main.

## Cancel probe (run first)

Is the umbrella's scope still missing `crates/uor-r4-harness/tests/loop_shared_files*.rs`? Result at `995286d33`: yes. The repair stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md`
- `docs/loop/tasks/infra-loop-harness/kimi-4/ORDER.md` and `REPORT.md` — the named-in corrections.
- `docs/loop/tasks/infra-loop-harness/kimi-5/**`

## Done when

- The glob is in, the two kimi-4 records name it, `xtask roadmap check` and `render --check` pass, `loop pr-check --local` stays a full PASS, the kimi-5 REPORT is written, and a new review request naming the head is posted on #2028.

## Stop and ask if

- The shared list changes again mid-repair, or any file outside the list above needs to change.
