# Discrete flock and shared vector writer integration

Date: 2026-10-13. Owning milestone M4 #2032, shared M1 learning interface #2029.
Status: **17/17 finite CUDA construction controls PASS**, at exact source
`76198c3b5321cc42ced8a0b4a3728aebfd89c2a4`, after prospective Claude FIT.
Training and M2 remain **HELD**; applied updates0. Delivery: [PR #2395](https://github.com/UOR-Foundation/uor-r4/pull/2395), not yet verified merged.

Claude's corrected design was approved at `7e3d1995fa5f8775182c17d65c51e517bcbc80a6`
and merged as #2378 at `5e85f696e3d23f9a6028bcd6ecf411a28111a373`, byte-identical.
This integration uses the shared integer codebook, SpiralCore frame/payload transport,
ordered FBS binder plans and geometric writer introduced by #2379. It does not inherit
#2379's numerical qualification for its new read or gate paths.

## Source ownership and interface

Claude owns `src/discrete.cu` and `src/flock_qualification.rs`; Codex owns the Rust
host, CLI, artifact and build bindings. All files are in `crates/uor-r4-discrete`.
The source is developed in separately claimed worktrees; the owner checkout is read-only.

The read configuration is artifact-bound: `ReadSpec::Route` or `Flock { kept }`,
with flock K in 1..8. Existing `Device::new` selects Route. `new_with_read` selects
explicitly. Snapshot schema2 includes the read configuration and the full i32 gate
inventory `[blocks, stages]`. Missing fields or schema1 do not silently migrate.
Mode and K occupy device spec words8/9; gates follow routes in the six affected
kernel interfaces. The existing seven PTX entry names remain the current contract.

Gate kind3 uses exactly three active candidates: ordinal0 is the current trit;
ordinals1/2 are the other values in `[0,-1,+1]` order, skipping current. The physical
loss buffer and launch retain sixteen slots/lanes, with unused thirteen set to MAX.
The chooser examines only three active gate candidates. Route/VectorBlock retain
sixteen active candidates. There is no host scoring, transport, model fallback or
floating-point model arithmetic. Route parameters in Flock mode and Gate parameters
in Route mode refuse. The CLI still exposes no apply command; training admission
continues to return TrainingHeld.

`Proposal::candidate_losses()` is readback of completed CUDA observations for
qualification. It does not compute candidates or select a winner on the host.

## Verification and limits

Host metadata-only library tests: 4/4 PASS. Scoped Rust formatting and diff checks
PASS. The new test exposed Serde accepting extra fields on an internally tagged
unit Route variant; strict deserialization through an empty-struct wire variant
now rejects them. This is metadata validation, not model execution.

[Claude's prospective FIT6157157748](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6157157748)
was published at `2026-10-13T23:11:42Z`, before numerical execution. It reviewed the
integrated tree matching exact executable source
`76198c3b5321cc42ced8a0b4a3728aebfd89c2a4`, the six flock controls and eleven
unchanged Route controls, the host/kernel ABI, and the literal witnesses on the
[finite sheet](../../harness/prep/m4-discrete-flock-construction-2026-10-13.md).

The once-only admitted controls ran from `23:14:21Z` to `23:14:36Z` on 13 October,
**17/17 PASS, exit0, applied updates0**: eleven Route regression controls plus six
new flock controls (`flock_identity_read`, `flock_gate_counterfactual`,
`flock_transport_witnesses`, `flock_payload_witness`,
`flock_selection_and_eligibility`, `flock_refusals_and_noop`). Gate candidate costs
`[448,256,64]` with thirteen MAX slots, ordinal2 strict improvement, independent
changed-gate snapshot forwards, zero virtual mutation, signed payload transport,
rank/recency discrimination and prefix/binder exclusion pass at this exact scope.
Typed wrong-mode host refusals pass; GPU status9 is shadowed and was not exercised.
**C3 covers literal snap witnesses only; C9 monotonicity remains diagnostic/open;
C6 covers valid prefixes, not arbitrary masked children.**

Setup/build attempt1 failed with archive EOF because extraction started before
upload completion. It is retained as a transfer/setup failure; no model ran.
Fresh source-complete extraction after checksum verification produced build2
**PASS in35.29s**, with the exact seven-entry AOT PTX admission. Environment:
Ubuntu24.04, Rust1.97.1, nvcc12.8.93, `compute_120`, RTX5090 driver595.84.
Executable SHA256:
`826bdf1a9ec87d03249c369d0e571d1cecd13c4c9ad49e7726f020f8c98e8d78`;
PTX SHA256:
`726937ebb907c3c01fe7a6b967a885f3fbe9327aaef6c8a9242007afb7c97f88`;
generated geometry SHA256:
`3c7a88bfc82e0fc2f4031ec35cfd69516596dce385b792767675ea937763cc50`.
Preserved before pod deletion at
`icloud:UOR-R4/results/codex/flock-qualified-20261013-2315.tar`, 83,313,664bytes,
verified MD5 round-trip `681a266b77d7bf7bb0ba86fb3c2be060`. Its nested restoration
bundle SHA256 is `df946496e077db984a397da7ea70c9c326f0d557f386726aedc0df03c90a2edd`,
equal on pod and laptop; it contains source archive, binary, PTX, geometry, both
setup/build records, controller/job logs and both numerical reports. The raw
[Route report](receipts/route-controls.json), [flock report](receipts/flock-controls.json)
and [controller log](receipts/controller.log) are also committed here.
Owned pod `vast-55826965` was released and deleted, with zero running pods verified.
The provider ledger records902seconds at$0.842/h, about$0.211 quoted compute;
including the preceding #2379 qualification the cumulative quoted GPU cost is
about$1.078. These are rate/time estimates, not invoices or all-in labor costs.

No applied update, corpus learning, learned-language result, throughput or full E8
coverage follows. The geometric atlas remains the sixteen-root anchor orbit, not
all240 E8 roots. The fifteen-second control interval is controller/control elapsed,
not training throughput or total preparation cost. Both owner holds and R14's
suspended reply-quality target remain unchanged.

## D2 static cost review supporting Kimi's pending learning sheet

This is a source-loop bound for the pre-flock Route kernel, SHA256
`9d3368a4c526686c53826cfa0d6dc0e2b164bc2cff384b3f8ea520af488e503c`.
It is not measured GPU instruction count, time, throughput or a spend projection.
Let V=vocabulary, B=blocks, C=context, m=batch rows, K=writer kept cells,
S=1+floor(log3 C), O=occupied cells and k=min(K,O). Flock neighbour K is a separate
quantity and must not be substituted for writer K.

For a fully eligible sixteen-move proposal, current source evaluates sixteen
baselines plus sixteen candidate paths:32m read/writer calls. A single baseline
and fifteen nonidentity candidates would need only16m distinct evaluations.
This redundancy is an implementation cost to account for, not a geometric negative.

Each writer performs at most:

| Work | Source-loop ceiling |
|---|---:|
| token-cell lookups | (k+1)V+1 |
| root-ID coordinate checks within those lookups | 256((k+1)V+1) |
| address-distance calls | kO-k(k-1)/2+1 |
| full token-distance calls | V+1 |
| token-distance coordinate comparisons | 8B(V+1) |

The writer scans identities for occupied cells and again for each kept cell,
recomputing first/last root IDs. Source-loop ceilings ignore early exits. Full
Route/VectorBlock sweep inventory is B(S+V) proposals on the declared fixed batch.
Initial carry touches8BmC physical elements; accepted updates invalidate carried
epochs, so the eventual driver must include recarry work in its dose accounting.

Illustrative dimensions ONLY (not a registered run):V4096,C192,m64,writer K8.
S=5; B3 yields12303 proposals and25196544 read/writer calls per full sweep;
B4 yields16404 proposals and33595392 read/writer calls. One proposal has a ceiling
of19327877120 root-ID coordinate checks. These bounds cannot be converted to dollars
or tokens/s without an admitted instrument and measured execution.

At qualified source `76198c3b5`, Gate3 has **three active candidate lanes**, each
replaying a baseline and its candidate; lanes3..15 retain MAX and return before
row evaluation. A fully eligible gate proposal therefore has at most **6m writer
calls**, now an actual source-loop bound rather than a conditional design estimate.
VectorBlock retains32m calls. Route parameters are inactive in Flock mode; a full
vector/gate sweep has `VB + BS` proposals and at most `m(32VB + 6BS)` writer calls
on the fixed batch, before early exits. Flock neighbour/binder/snap read work is
additional; it is not priced by the Route descent bound above. These are static
counts, not measured sweep throughput or dollars. The eventual learning instrument
must pin source/shape/dose and charge recarry, evaluation, setup and preservation.
Training remains held regardless of this finite qualification result.

## Learning-sheet review

Kimi's #2391 successor sheet is PENDING, not a training release. Its current arm
explicitly excludes flock/Gate. Codex's review is
[#2391 comment6156858784](https://github.com/UOR-Foundation/uor-r4/pull/2391#issuecomment-6156858784).
D1 requires actual seeded initialization, corpus/mask/BOS segmentation, exact scored
coverage, GPU count reference and top8 evaluation, parameter schedule and qualified
update boundaries. D2 requires a concrete cost model; static bounds above do not
supply measured throughput. Pin full representation dimensions and evaluator
eligibility before assigning a model negative. Missing rivals can produce zero
hinge by law; those rows cannot count as successful learning. The owner training
release, fresh-transfer identity and R14 suspended reply-quality headline remain
separate constraints.
