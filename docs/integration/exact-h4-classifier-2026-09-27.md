# Exact integer classification of signed H4 roots

This source work prepares the classification step of native geometric attention.
It does not integrate a new reader or qualify a learned artifact. OpenCode owns
the finite signed-group reader and its language comparison. The fourth lab’s
active dialogue code-choice experiment and its fixed endpoints remain separate.
At source `43e543fd`, all three focused integer checks and release compilation
pass. The frozen library passes strict instruction inspection in 33 emitted
symbol ranges, including the classifier and both scalar helpers. The separate
training-crate donor-order check remains NOT_RUN; no reader integration follows.

## Chosen mathematical contract

`uor_r4_integer::h4_classifier::signed_h4_code_i32` accepts four signed i32
coordinates with a common positive scale. It selects the root with maximum
exact inner product in the 120-element historical H4 enumeration, retaining
antipodes. Ties between nonzero-query scores choose the lowest historical index.
The all-zero vector explicitly maps to identity index 1, matching the geometric
reader’s zero convention rather than ordinary all-tied argmax index 0.

This is an exact mathematical-root classifier on the declared quantized vector.
It does not promise identical decisions to normalized F32 vectors, rounded F32
root coordinates, or the unrelated three-coordinate S2/Q30 classifier. Those
numerical interfaces may disagree near boundaries. With a common Q8 input scale
and the current reader epsilon of 1e-6, every nonzero quantized lane lies above
the zero threshold. A different scale/epsilon requires an explicit contract.

The immutable `H4_ROOT_COEFFICIENTS` table stores 960 signed bytes. Its order and
coefficients follow `native_geometric::learner::prefix_artifact::historical_roots`,
introduced at `d15360527f7c69ac8b83eef0bbd5839b87c26f02`. A coordinate is
`(a + b*phi)/2`; the ignored factor one-half is shared by every candidate. The
integer crate gains no core dependency, floating constructor or heap allocation.
A separate offline training-crate test compares all coefficients against the
existing exact donor. Sorted closure IDs are a different ordering and cannot
be substituted without the explicit bijection.

## Exact family reduction

The roots partition into eight axis roots, sixteen signed half-roots and twelve
blocks of eight sign choices for the even permutations of
`(0, 1, phi, phi-1)/2`. Each nonzero magnitude is positive, so a family's best
sign matches its assigned query coordinate. A zero coordinate chooses the
negative sign, preserving the smallest historical ID within that family.

The classifier retains one winner from each family: the earliest axis with
maximum widened absolute coordinate, the best signed half-root, and one signed
root for each of the twelve golden permutations. A shared permutation constant
maps output axis to base coordinate. Candidate selection places each query sign
into its corresponding base sign bit, rather than mistakenly treating that
permutation as its inverse. The sign bits are `s1=4, s2=2, s3=1`.

These fourteen IDs occur in ascending historical-family blocks. Comparing only
their scores with strict-greater updates preserves the global lowest-ID tie.
The all-zero input still bypasses ordinary argmax and returns identity 1. The
existing score assembly and exact comparator are unchanged, as is the independent
offline oracle that scans all 120 roots.

This reduces root-score evaluations from **120 to 14** and exact score comparisons
from **119 to 13**, with three additional integer magnitude comparisons to choose
the axis. It uses no candidate approximation or learned selection and retains
the full signed-i32 domain by widening before taking absolute values. These are
structural operation counts, not a measured speed, energy or language result.

## Comparison and bounds

Each dot score is represented as `A + B*phi`, omitting the common half. The
small coefficients 0, ±1 and ±2 require only addition, subtraction and doubling.
For a score difference, let `P = 2*A + B`; twice that difference is
`P + B*sqrt(5)`. Zero and same-sign cases determine the sign immediately. When
signs oppose, comparing `P²` and `5*B²` determines which signed magnitude wins.
The existing checked shift/add product computes the squares; five times a square
is a two-bit shift plus the square. No irrational approximation is needed.

For input magnitude at most M, the donor coefficients give per-root bounds
`|A| <= 4M` and `|B| <= 2M`. Differences therefore have `|A| <= 8M`, `|B| <= 4M`
and conservatively `|P| <= 20M`. At M = 2^31, both required squared quantities
are below 2^71, inside u128. Inputs must be widened before negation or doubling,
including i32::MIN. The equality case of nonzero integer `P² = 5*B²` is
impossible; exact score ties are represented by both difference coefficients
being zero. Strict-greater selection preserves the earlier historical index.

The production source is intended to use only integer shifts, additions,
subtractions, comparisons and table reads. A source property does not establish
the emitted instructions of a particular optimized build. Later inspection must
cover the actual classifier and arithmetic callees, without treating an absent
inline symbol as inspected. No performance or energy measurement is supplied.

## Executed result and compiler correction

The first release artifact at `7e4129b0` passed the three arithmetic checks but
failed the existing strict instruction audit: LLVM emitted one `UMULH` for
checked `4*B²+B²`, plus nineteen `FMOV` transfers from vectorized golden sign
packing. That frozen artifact and failed result are retained.

At `43e543fd`, two private non-inlined scalar helpers preserve the same exact
algorithm while separating runtime sign scattering and checked addition. The
same three checks pass. The rebuilt library SHA256 is
`715bb4492a75c8a55ff63bde31553ff4d769d2503858f3d6726cf4d19c9d88c7`.
With Rust 1.97.1 / LLVM 22.1.6 on aarch64-apple-darwin, the unchanged strict
forbidden-instruction policy reports zero violations in 33 emitted ranges and
zero missing mandatory symbols. The classifier (651 instructions),
`golden_sign_bits` (67) and `checked_add_square_terms` (11) are actually present
and checked. Independent relocation inspection confirms that these two helpers
are the only numerical callees; shift/add squares, family selection and score
comparison occur inline. Other external calls are bounds-panic paths excluded
by the reviewed input/table/index bounds. Absent helpers are not independently
audited. This scoped H4 inspection is not whole-process certification, nor a
claim about future linked builds.

The corrected build/check supervisor took 81.655 seconds with sampled group
peak 353,845,248 bytes; its read-only audit took 1.499 seconds. Two earlier
correction attempts stopped on foreign Cargo, with no test failure: one before
launch and one after 22.519 seconds. Their elapsed time was deducted from the
same 180-second correction allowance, totaling 104.309 seconds across all three
attempts. The earlier source's complete build/check took 114.858 seconds and
failed opcode inspection took 1.976 seconds. Concurrent model/build elapsed,
preparation, reviews and delivery are charged once in the existing ledger.
Independent compiled-call inspection added 1.041 seconds of read-only work.
[Portable source, artifact, commands, check log and cost evidence](../evidence/exact-h4-classifier-2026-09-27.json)
preserve both outcomes. No core/training build or model execution occurred.

## Integration boundary and next evidence

Focused arithmetic checks cover zero, signed identities, ties, extreme i32
inputs and an independent native-i128 reference in offline tests. The separate
donor-order check prevents an internally consistent classifier from silently
using incompatible group IDs. These are mathematical/interface checks, not
language tests or a broader proof campaign.

If the learned reader earns continuation, integration must bind the actual Q/K
quantization, root order, inverse/product tables and legal learned score-table
compilation. It must then observe float-to-integer decision changes and actual
complete replies. This helper alone does not select MLP weight precision,
modify candidate admission, infer semantics from hash addresses, remove dense
vocabulary access, or establish geometric advantage. It also leaves the paired
H4/icosian state and its retained orientation untouched.

The [learning and exact-table reuse review](finite-h4-learning-and-native-reuse-2026-09-27.md)
separates estimator and capacity limitations from the numerical bridge and
identifies the existing exact group compiler to reuse. It selects no new fit.

Initial ownership is recorded in the [shared work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5857712244),
and the measured integer-only build exception in the
[scheduling amendment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5857979897).
References #973 and #820. The donor check and model-dependent integration retain
their separate scheduling. The fixed dialogue experiment and its recovery
status remain in [the owning issue](https://github.com/UOR-Foundation/uor-r4/issues/973).
