# #1599 — Learn latent geometric context with separate native address readout

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T03:31:37Z
- **Updated:** 2026-10-09T13:17:20Z
- **Closed:** 2026-10-02T05:54:16Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1599
- **Merged:** 2026-10-02T05:54:16Z
- **Merge commit:** db075163f812dbb0db6cee5e6bedbd65dbe3b0a1
- **Base / Head:** main / UOR-Foundation:codex/geometric-context-native-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The current reader gets contextual addresses from a floating recurrent trunk. This adds a learned latent signed-H4 state producer with a separate root/radius/presence observation, compiling state transitions and readouts into bounded Q24 integer tables. Emitted addresses no longer substitute for hidden state, and absence does not erase it.

Both fixed 640-update seeds completed on the existing authored development panels. Seed 1 baseline/native scores: 99/100 original and 109/110 stress (each of128). Seed 2: 128/128 original and 128/127 stress. Hard-reference and independently reloaded native execution have zero state/action/root/category/code/prediction disagreements throughout. Retain the stress regression and distinguish donor approximation from compiler fidelity. The second seed native and reset both total127 but miss different rows.

Validation: twelve focused Rust checks and actual example build passed at executed source01545afa; both model runs completed, reports sealed and Rust-verified. Independent source review approved. Receipt and current state record executable/data/artifact identities. Claim wording check passed for the result documentation. Queue acknowledgements are not executed tests.

This changes context production only: donor float values/NoRead/downstream remain unfinished. No general language, geometric advantage, complete integer serving or energy qualification. Next work is the native weighted reader and geometric value representation/producer. References #1512 and #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

