# #1871 — Repair main's example compile: invalid pub(super) in geometric-native-compiler (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T02:48:42Z
- **Updated:** 2026-10-09T13:21:17Z
- **Closed:** 2026-10-08T02:50:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1871
- **Merged:** 2026-10-08T02:50:18Z
- **Merge commit:** 692a259c74c61bea52132cf7cf1b215a05579f2c
- **Base / Head:** main / UOR-Foundation:deepseek/native-compiler-visibility
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The break

`main` does not compile all targets. `cargo check -p uor-r4-training --all-targets` fails:

```
error[E0433]: too many leading `super` keywords
    --> crates/uor-r4-training/examples/geometric-native-compiler.rs:1086:5
     |
1086 | pub(super) struct ReferenceRule {
```

`ReferenceRule` and `new_with_templates` carry `pub(super)` at the crate root of an example, where `super` does not resolve. Introduced by #1739 and invisible since: the required CI job publishes status names and runs no compile, so nothing in the merge path exercises example targets.

## The repair

Both items are used only inside this file (`ReferenceRule::new_with_templates` at lines ~944/946 and `ReferenceRule::new` at ~1352), so plain `pub` is the minimal correct visibility. Two lines changed, nothing else.

## Verification at the exact head

- `cargo check -p uor-r4-training --all-targets` — **Finished clean**, where it previously failed on this target.
- No other file differs from `main`.

## Scope

Not my file and not my break. Reporting it and repairing it at its own scope because a red `main` blocks every lab's delivery, and the owner's direction of 7 October is that completed work must reach a verified `main`. If the native-compiler owner prefers a different visibility, the change is two tokens.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

