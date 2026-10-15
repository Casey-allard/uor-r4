# #2066 — Fix main: the geometric-stack example did not compile after #2062 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T18:03:59Z
- **Updated:** 2026-10-09T18:07:13Z
- **Closed:** 2026-10-09T18:05:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2066
- **Merged:** 2026-10-09T18:05:22Z
- **Merge commit:** fa786e4ef2e3beecf6e0dec925df649d5796d186
- **Base / Head:** main / UOR-Foundation:deepseek/fix-main-example
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fix main: the geometric-stack example did not compile after #2062

MAIN WAS BROKEN AND THIS REPAIRS IT. The span-extraction merge (#2062) added
Reply::span_extract but left the Reply initializer in
crates/uor-r4-training/examples/geometric-stack.rs (line 5403) without the new field:

  error[E0063]: missing field `span_extract` in initializer of `Reply`
    --> crates/uor-r4-training/examples/geometric-stack.rs:5403:12

so `cargo check -p uor-r4-training --example geometric-stack` failed on main. The fix is the
missing field, set to None, which is the documented default - the span-extraction rule is off
unless explicitly enabled, so no behaviour changes for any existing invocation.

CAUSE, RECORDED BECAUSE IT IS MINE: this is the second time today that a delivery of mine
landed a change and left a caller unupdated - the first was #2009, which merged the copy-stop
types without their decoder half and left main's lib tests not compiling, fixed in #2040. In
both cases the change itself was correct and its call sites were not carried with it.

FOUND BY the answer-span teammate during its pre-registration recon, which is exactly why that
gate exists: it hit the error on a pristine main before spending any pod time, reported it
rather than silently working around it, and fixed it on its own path so as not to block.

VERIFIED: `cargo check -p uor-r4-training --example geometric-stack` now finishes
(dev profile, 5 pre-existing warnings, no errors).

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

