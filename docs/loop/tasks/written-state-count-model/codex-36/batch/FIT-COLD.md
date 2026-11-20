# [codex] Non-author FIT addendum for the four cold resident-batch replays

TEST FITNESS: FIT

The primary [finite FIT](FIT.md) digest
`8d730c5724eee3f44d6e672568e2d3babc16d57c24bdbd575b12c718b156fd6f`
reviewed one process per width. This addendum admits one additional fresh
process at each of the same widths, 1, 3, 10 and 26, solely to check process
repeatability against the primary tape and native oracle. It binds the same
exact Wick source and build/input manifests
`98b9ed244394c45a90def5a59e265b95b1695e2b7ac0afd64dddee410cab979f`
and `639e35806533015e5ef30e654c942145e6daa4f54bc668f2731dc60120302874`.
The launcher, cubin, four-record fixture, compact file, target count,
expected output, cost bounds, negative controls and decision are the same
as in the primary FIT. Each cold process must independently report the
specified one upload and width-specific launch count, write to a fresh
output and match all 26 × 23 feature words. Any repeatability discrepancy
fails the finite batch gate.

The admitted total with this addendum is eight fresh C128 processes. A
top-session GO that binds this addendum and the original FIT is required
before the eight-process dispatch. Wick's common lock and idle GPU
requirements apply to the entire launch. R-feature generation, fitting and
language evaluation require separate review.
