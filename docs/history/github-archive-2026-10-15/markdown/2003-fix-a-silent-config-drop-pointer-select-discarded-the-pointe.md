# #2003 — Fix a silent config drop: pointer_select= discarded the pointer_identity term (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T09:36:27Z
- **Updated:** 2026-10-09T13:20:40Z
- **Closed:** 2026-10-09T09:38:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2003
- **Merged:** 2026-10-09T09:38:06Z
- **Merge commit:** c18aeefd863cc498a7cd8a29af14f22a7da40668
- **Base / Head:** main / UOR-Foundation:deepseek/identity-select-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fix a silent config drop: pointer_select= discarded the pointer_identity term

FOUND BY MEASUREMENT on the pod, in the code merged as #2002. The documented way to switch the
token-identity term on for an existing checkpoint - init=<checkpoint> pointer_identity=W - works
ONLY if pointer_select= is not also given. With both flags the identity term is SILENTLY
DISCARDED: no error, no warning, and the training report records settings.pointer_identity="0.5"
while the saved config has no identity at all. Three control runs on the pod, reading the second
line of each log, show it directly:

  pointer_identity=0.5                      -> identity: Some(PointerIdentity { weight_bp: 5000 })
  pointer_identity=0.5 pointer_select=none  -> identity: None
  pointer_select=none pointer_identity=0.5  -> identity: None   (argument order is irrelevant)

ROOT CAUSE, exactly: in init_extended_config the identity branch builds
PointerConfig { identity, ..existing }, and the select branch then immediately rebuilds
PointerConfig { select, ..existing } FROM THE ORIGINAL SAVED HEAD, dropping the field the previous
branch had just set. The route branch three lines below already used the correct pattern, which is
why only select was affected.

THE FIX is one statement: build the select branch from the CURRENT pointer config
(config.pointer.map(|pointer| PointerConfig { select, ..pointer })) exactly as the route branch
does, with a comment saying why.

WHY THE TESTS MISSED IT: the crate's own test for this path
(a_saved_head_takes_an_identity_term_without_new_weights) never passes pointer_select, so it passes
on the buggy code - a coverage gap in a test that was written to cover exactly this feature, not a
wrong assertion. The fix is verified by the example's twelve CLI tests passing, and the pod-side
control runs that found the bug are the behavioural confirmation.

IMPACT IF UNNOTICED: every arm using the documented two-flag syntax would have been an ordinary
dot-pointer fine-tune reported as if the identity term were active - a silent negative that would
have been read as evidence against the token-identity change itself. That is the class of failure
this session has hit repeatedly, and it is why the arms now running were checked against their
saved configs before any scoring.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

