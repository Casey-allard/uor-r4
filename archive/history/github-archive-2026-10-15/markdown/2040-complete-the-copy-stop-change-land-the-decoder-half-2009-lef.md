# #2040 — Complete the copy-stop change: land the decoder half #2009 left behind (fixes main) (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:54:03Z
- **Updated:** 2026-10-09T15:00:12Z
- **Closed:** 2026-10-09T14:55:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2040
- **Merged:** 2026-10-09T14:55:53Z
- **Merge commit:** dddc719b52a0e07e6239ba5f772950e43a678993
- **Base / Head:** main / UOR-Foundation:deepseek/copy-stop-completion
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Complete the copy-stop change: land the decoder half that #2009 left behind

MAIN WAS BROKEN AND THIS FIXES IT. Commit 9eb767a58 (#2009) merged only half of the copy-stop
change: geometric_stack.rs gained CopyStop, CopyStopRun, PointerSelection and three tests that
import crate::stack_dialogue::greedy_reply_with_copy_stop, but stack_dialogue.rs had ZERO
occurrences of copy_stop and no such function existed anywhere in the tree. As a result
cargo test -p uor-r4-training --lib did not compile on main, and the probe that produced the
sealed records could not be rebuilt from main either.

CAUSE, RECORDED HONESTLY: when landing #2009 I staged named paths only - geometric_stack.rs,
examples/geometric-stack.rs and the evidence doc - and deliberately excluded five other modified
files because I judged them to be another lab's work in the shared worktree. They were in fact MY
own change's call sites. The caution was right in principle and wrong in this instance, and the cost
was a main that does not build its tests.

THE FIX lands those five files: stack_dialogue.rs plus the four call sites (stack_memory_replies.rs,
stack_grounded_session.rs, s2-behavior-codec.rs, s2-behavior-distortion.rs), restored verbatim from
the preserved pod copy (stack_dialogue.rs md5 8d3cd621604ab5e71a92a330e736a3b6).

VERIFIED LOCALLY, which also keeps this CPU work off the paid pod: the crate builds
(cargo build -p uor-r4-training --lib), the three copy-stop tests pass
(the_copy_stop_rule_fires_on_a_copied_contiguous_span,
copy_stop_is_off_by_default_and_refuses_a_non_span,
the_default_decoder_is_unchanged_by_the_copy_stop), and the whole lib test target compiles.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

