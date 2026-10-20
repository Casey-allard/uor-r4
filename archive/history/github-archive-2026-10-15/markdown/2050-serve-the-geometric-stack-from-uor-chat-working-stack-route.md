# #2050 — Serve the geometric stack from uor-chat: working --stack route and the measured bundle negative (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:59:15Z
- **Updated:** 2026-10-09T16:02:26Z
- **Closed:** 2026-10-09T16:00:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2050
- **Merged:** 2026-10-09T16:00:58Z
- **Merge commit:** a7d6d91a127135cdeabccd8345496094fd5fb227
- **Base / Head:** main / UOR-Foundation:deepseek/m4-stack-serve
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Serve the geometric stack from uor-chat: a working --stack route and the measured bundle negative (References #2032)

M4 asks for one served model through D11 integer serving and a CLI, and the stated gap was that uor-chat
cannot serve the stack. The gap is now closed by the route that exists, and the route that does not is
documented as a measured negative rather than quietly dropped.

THE NEGATIVE, PINNED AS A TEST: converting the exported stack into the integer crate's bundle is NOT a
repackaging job and is impossible without new training. ServingProfile::Retained requires vocabulary 4096,
width 128 or 256, read_width 64, context 2..=256 (config.rs:99), and Dialogue576 requires width 576
(config.rs:115-122); the exported dialogue stack is width 288, satisfying NEITHER. The two parameter
inventories share no name or shape - the stack has mlp/conv/decay/rotation/beta/offset maps with no
IntegerModel counterpart, and IntegerModel needs recurrent/update/copy/output.norm maps the stack lacks -
so a converter would have to invent weights, which is training by another name. Verified by construction:
a bundle directory that passes sealing, bundle.json schema/context/admission/hashes, tokenizer parse and
table manifest, but declares the stack's own shape, is refused by Bundle::load with 'joint config requires
vocabulary4096, width128/256, read_width64, context2..256'. That refusal is pinned in
crates/uor-r4-integer/tests/bundle_stack_boundary.rs so the boundary cannot be rediscovered by accident.

THE WORKING ROUTE: uor-chat --stack <ARTIFACT.lut> --tokenizer <TOKENIZER.json> [--protocol 1|2]
[--threads N] [--say TEXT] [--max-tokens N], via the D11 stack engine already in this crate. New
crates/uor-r4-integer/src/stack/chat.rs provides StackChat: literal-role protocol, greedy stack_argmax over
next_token_scores() (pointer mixtures included), stops on EOS / historical short cycle / token cap, exact
token history, an explicit Context error instead of silent truncation, Result with a focused enum, and no
unwrap/expect/panic on recoverable paths. The sealed --bundle path is unchanged. --temperature > 0 is
REFUSED because the D11 stack engine has no integer categorical sampler - greedy only, stated rather than
faked.

MEASURED: uor-chat --stack ... --say 'Hi there!' answers 'Hello! How can I help you today?' with ids
[1178,3,2577,469,340,436,351,1719,33,1], identical to the recorded D10 lut-chat transcript for that
request, and a replay over the whole recorded transcript reproduces 38 rows / 58 turns / 1505 reply ids
exactly with stop reasons and reconstructed histories equal to the recorded ones. Context matches 256 for
this artifact - reported, not assumed.

REGRESSIONS: none. The integer suite has six failing test_challenger_* tests, and a BASELINE RUN of clean
main with this change reverted fails the SAME six in the same order, so they are pre-existing and not
caused by this port. The crate builds; the port's own tests pass.

LIMITS, STATED: the float-vs-served and D11-vs-D10 fidelity runs (lut-evaluate, d11-evaluate) did not
complete - the driver process was found hung at 0.0% CPU with no child, so no fidelity figure is claimed
here and the transcript equality above is the evidence that exists. The pointer/gate-floor/identity-stop
line (best 72/84 exact recall of never-seen values) is NOT claimed as served: export refuses a pointer
carrying an identity term because there is no integer port.

References #2032


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

