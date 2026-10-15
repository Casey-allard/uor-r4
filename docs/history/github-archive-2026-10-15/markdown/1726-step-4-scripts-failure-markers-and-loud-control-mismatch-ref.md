# #1726 — Step 4 scripts: failure markers and loud control mismatch (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T06:58:32Z
- **Updated:** 2026-10-09T13:08:47Z
- **Closed:** 2026-10-05T07:13:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1726
- **Merged:** 2026-10-05T07:13:46Z
- **Merge commit:** 81cb72978915cdcd2c92b5ddedfe311c25f97adf
- **Base / Head:** main / UOR-Foundation:codex/step4-script-markers
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. This follows up the review nits on #1723 (https://github.com/UOR-Foundation/uor-r4/pull/1723#issuecomment-5989523511), items 1, 3 and 4. It is stacked on `codex/step4-key-casing` because #1723 is not merged yet. Retarget the base to `main` once #1723 lands.

## What changed

**`scripts/pod/step4-corpus.sh`**
- **Failure marker.** Under `set -Eeuo pipefail`, an ERR trap writes `$OUT_ROOT/STEP4_FAILED` with the status, line, command and UTC time. It prints `STEP4 CORPUS FAILED …` and exits with the failing status. Success writes `$OUT_ROOT/STEP4_DONE` and prints `STEP4 CORPUS DONE`, so a run ends with exactly one marker for a completion watcher.
- **Exclusive root.** A root that already holds either marker is refused with exit 2, so pick a new `OUT_ROOT` for a retry. `OUT_ROOT` is now created before the build, so a build failure also leaves a marker.
- **Loud control mismatch.** If the rebuilt v2 control DIFFERS from Step 1's `mw-hi`, the script writes `V2_CONTROL_DIFFERS` and fails with exit 3. Setting `ALLOW_V2_DIFFER=1` records the marker and continues. The A/B uses the rebuilt `hi-v2`, so it stays valid, but Step 1 comparisons are not. The header also notes that Step 1's binary was at `dec3c096` and the pinned digest was taken at `e4922ec5`.

**`crates/uor-r4-training/examples/m-world.rs`**
- `fit_route` (used by `evaluate_v2 recall=route`) now takes the evaluated `Variant` and draws `MWorld2::new(..).with_variant(variant)`. Its route record gains `world` and `world_digest`, so the route's provenance matches the report's world.
- `world=v2` is unchanged, because `Variant::V2` is the default and `with_variant` only sets the field.

Not addressed here: item 2, the v2/v2c pairing and decision rule, belongs on the work card before compute. Items 5 and 6 are outside these files.

## Tests run (local, at fb6c449c)
- `cargo check -j 2 --offline -p uor-r4-training --example m-world`: clean. The only warnings are pre-existing ones in uor-r4-core.
- `rustfmt --check` on `m-world.rs`: OK.
- `cargo test -p uor-r4-training --example m-world`: 20 passed.
- `cargo test -p uor-r4-training --lib -- milestone_world_v2::tests::the_v2_seed_9101 milestone_world_v2::tests::variants`: 2 passed.
- `bash -n` on the script: OK.
- **Stub harness for the script.** Fake cargo, m-world and mix binaries were used. Results:

  | Case | Exit | Markers |
  |---|---|---|
  | Control matches | 0 | `STEP4_DONE` |
  | Rerun on the same root | 2 | refused |
  | Control differs | 3 | `STEP4_FAILED` + `V2_CONTROL_DIFFERS`; stops before `hi-v2` |
  | Differs with `ALLOW_V2_DIFFER=1` | 0 | `STEP4_DONE` + `V2_CONTROL_DIFFERS` |
  | Missing tokenizer | 1 | `STEP4_FAILED`, line 72 |
  | Build fails | 1 | `STEP4_FAILED` |
  | Mix fails | 7 | `STEP4_FAILED` with that command |

- **Not run:** an actual `evaluate_v2 recall=route` (it needs a model), and an actual pod corpus build.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5989809212 — 2026-10-05T07:10:52Z — Casey-allard

Adversarial review at fb6c449cec4d8e5b7c0fbd6d3345b00593ee49e3:

**Verdict: APPROVE.** This closes review items 1, 3 and 4 from the #1723 review (https://github.com/UOR-Foundation/uor-r4/pull/1723#issuecomment-5989523511). Nothing blocks the merge. Item 2 (freezing the v2/v2c decision rule) still has to go on the work card before compute, as the PR says.

**What I ran.** I ran these at the exact head, in a `git archive` copy in `$TMPDIR` with `CARGO_TARGET_DIR=~/.cache/uor-claude-target -j 2 --offline`. I touched the sources first because of the known mtime fingerprint issue.
- **`cargo test -p uor-r4-training --example m-world`:** 20 passed.
- **`cargo test -p uor-r4-training --lib -- milestone_world_v2::tests::the_v2_seed_9101 milestone_world_v2::tests::variants`:** 2 passed, including `the_v2_seed_9101_draw_is_unchanged`.
- **`rustfmt --check` on `m-world.rs`:** OK.
- **`bash -n scripts/pod/step4-corpus.sh`:** OK.
- **My own stub harness** (fake cargo, m-world and mix):
  - The control matches: exit 0, `STEP4_DONE` only.
  - The control differs: exit 3, with `STEP4_FAILED` (`status=3 line=110`) and `V2_CONTROL_DIFFERS`.

**Item 1 (route provenance): closed.** `fit_route` has one caller, `evaluate_v2`. It now passes `v2_variant(args)?`, the same variant that `evaluate_v2` uses for its own world (line 1815) and its report (lines 1950-1951). The route draws use `.with_variant(variant)`. The record carries `world` and `world_digest` through the same `world_name`/`digest_for` helpers as every other report. Since `with_variant` only sets the field, world=v2 is unchanged.

**Item 3 (loud DIFFER): closed.** A DIFFER result writes `V2_CONTROL_DIFFERS` and calls `fail 3`. `ALLOW_V2_DIFFER=1` records the marker and continues. The comparison is inside an `if cmp -s` condition, so it cannot set off the ERR trap by itself. The header notes both `dec3c096` and `e4922ec5`.

**Item 4 (failure marker): closed** for every failure that goes through errexit:
- The trap is set after `OUT_ROOT` exists, so a missing tokenizer or a failed build also leaves `STEP4_FAILED`.
- `fail` clears the trap before it writes, so it cannot recurse.
- Success clears the trap before it writes `STEP4_DONE`.

**Nits (non-blocking)**
1. **Signals leave no marker.** SIGTERM, SIGHUP and SIGINT (for example a pod stop or an ssh drop) leave neither marker. In my stub run, a SIGTERM exited with 143 and no marker. So the header's claim that a run leaves "exactly one of the two markers" holds only for normal exits. Either add `trap 'fail 143 $LINENO signal' TERM HUP INT` or scope the sentence, and keep checking PID liveness in the watcher either way. A killed root has no marker, so the exclusivity check would also accept it again. That is acceptable only because the m-world and mix steps claim their own outputs.
2. **A failure inside a command substitution.** Under bash 5 with `set -E`, an ERR trap is inherited into `$(...)`. If `git rev-parse HEAD` failed (for example, `REPO` is not a checkout), `fail` would run in the subshell and write `STEP4_FAILED`, and then the parent would continue and could also write `STEP4_DONE`. I could not reproduce this locally, because macOS bash 3.2 does not fire the trap there. It needs an unusual setup and is low-risk. `head=$(git rev-parse HEAD); echo "head $head"` would close it.
3. **The route path is not run end to end.** No `recall=route` evaluation has run on v2c, because it needs a model. The change is type-checked, and it matches the pattern used by the other variant callers.


### comment 5989836580 — 2026-10-05T07:12:54Z — Casey-allard

Rebased onto main after #1723's squash merge: the script commit was cherry-picked alone, giving head 4c7d5c845695ca0ce3705711915c8ee839ed243b with 2 files (step4-corpus.sh and m-world.rs). Its adversarial review approved the identical change. At this head: `bash -n` passes, m-world.rs passes rustfmt, and `cargo check --example m-world` passes. Queuing.


---
_Comment/review ids in this file: 5989809212, 5989836580_

