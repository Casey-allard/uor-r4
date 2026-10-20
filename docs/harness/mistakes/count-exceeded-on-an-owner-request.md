---
id: count-exceeded-on-an-owner-request
title: A line past 3/3 carried on without a pivot because the delivery was "requested"
date: 2026-10-12
severity: warn
class: coordination
trigger: [4/3, 5/3, past three, owner request, no reset, explicit request, pivot card, count exceeded]
paths: [docs/labs/**, docs/integration/DECISIONS.md, .github/pull_request_template.md, docs/harness/milestones.json, crates/uor-r4-harness/src/*.rs]
milestones: [m2]
mechanism: protected-legal-construction
guard: prlint.pivot_at_three
evidence: [archive/labs/whole-path-implementation-2026-10-12/README.md, archive/labs/starling-pivot-2026-10-11/README.md, archive/labs/lab-pivot-2026-10-09/README.md]
status: active
---

## What happened

The bound whole-path delivery declared `count 4/3 -> 5/3 on this explicitly requested
source delivery, no reset` and stated, one paragraph later, that it is "not permission for
a third preparation loop or automatic compute". The starling pivot record declared
`count 3/3 (pivot)` and named the pivot card comment on the milestone. Both are honest
about being over the limit; neither pretends a moved headline reset it. The rule they
follow - only a moved headline resets the count, and renaming or re-scoping does not - is
why the declaration has to carry the exception out loud.

## Symptom

A line count reads 4/3 or 5/3, or a session declares "no reset" and continues on the same
line. The risk is not the arithmetic: it is that the exception becomes the norm and the
pivot never happens, which is exactly the 9 October outcome.

## Lesson

Passing 3/3 is allowed only with an owner request and an explicit statement that the count
is not being reset - and the body must still name what it is not permission for. A delivery
request is not a headline move; neither is a rename, a split or a re-scope.

## How to apply

- Declare the overrun in the Line, with the reason: `count 4/3 -> 5/3 on this explicitly
  requested source delivery, no reset`.
- Name the pivot card and its comment id when the line is at 3/3
  (`archive/labs/starling-pivot-2026-10-11/README.md` is the pattern).
- Say what the delivery is not permission for (another preparation loop, automatic
  compute, a mechanism closure).
- Putting the pivot in the body is what clears `prlint.pivot_at_three`; do not reword the
  line to hide the count instead.
