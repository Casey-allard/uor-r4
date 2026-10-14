# Discrete geometric learning through exact SpiralCore/FBS moves

**Current D1/D2 status:** the native Rust corpus/schedule/checkpoint driver and
integer GPU evaluator are implemented at `e7d926fbe38537f4faa90b817705c61a780778d6`
([PR #2400](https://github.com/UOR-Foundation/uor-r4/pull/2400)). The A1 release build
passed (Cargo16.06s, controller17s), with executable SHA256
`399ab42713589439dd963ec50515133e723abda79cd4b1d90330d9710d80c117`;
PTX/geometry identities are unchanged by the A1 correction. The **twenty-five CUDA
construction controls PASS** at this exact source/binary: eleven Route in5.989098677s,
six flock in2.833462330s and eight driver in2.948602761s. The controller started at
`2026-10-14T00:42:39Z`; there were no failed, skipped or NOT_RUN controls. Exactly
eleven isolated authored fixture apply calls produced eight updates, with zero
corpus updates. D2 operation-cost measurement **PASS**, completed at
`2026-10-14T00:43:57Z` in66.633432500s:30/30 timed API calls,12/12 proposal
samples, applied updates0. [Raw measurement](../labs/discrete-driver-cost-2026-10-13/receipts/measurement.json).

At the authored V4096/B4/context192/64-row shape, synchronized native API medians
(three timed repetitions after separate warm-up) are:

| Operation | Route | Active Flock |
| --- | ---: | ---: |
| Plain forward | 37.837521ms;1691 target rows/s | 40.576018ms;1577 target rows/s |
| VectorBlock proposal | 3.903280515s | 4.206217974s |
| Read-parameter proposal | Route3.959623663s | Gate4.180529788s |

These are host wall/API durations with CUDA synchronization, including the actual
operation's allocations and completed-result readbacks. Target rows/s is
teacher-forced fixed-batch forward output rate, not autoregressive generation or
training throughput. The153 active candidates and baseline replay are virtual;
no production-shape proposal was applied. Serial per-candidate counterfactual
row/writer replay is the observed cost bottleneck, not a negative about the
geometric law. For16,384 vector coordinates plus20 read coordinates, the unchanged
64-row fixed-batch projection is64,030.540431020s Route (about17.786h) or
68,998.285881776s Flock (about19.166h). Those are **UNVERIFIED estimates**, not
measured sweeps, corpus passes or a training dose; recarry, corpus-order variation,
checkpoint/evaluation cadence and total provider cost require separate accounting.
The full evidence archive passed its iCloud round-trip; owned pod vast-55833746 was deleted at00:48:44Z. Lifecycle2,544s at$0.908/h projects to$0.642, including setup/review/preservation; no active pod remains.

The fresh prospective FIT for this run came from the independent non-author Codex
reviewer at [PR #2400 comment6158277830](https://github.com/UOR-Foundation/uor-r4/pull/2400#issuecomment-6158277830),
mirrored on [M1 comment6158278165](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6158278165),
under the [separate finite sheet](../harness/prep/m1-discrete-driver-finite-2026-10-13.md).
Claude's original conditional review is preserved; its follow-up was unavailable
(the desktop inbox did not wake and CLI OAuth was expired). The new FIT is not
attributed to Claude. The earlier seventeen-control result below retains its
prior source scope and Claude FIT attribution.

**Prior integration result, 13 October 2026 (source76198c3b5):** optional `ReadSpec::Flock { kept }` and ternary `Parameter::Gate` proposals are integrated with the same integer codebook and geometric writer under snapshot schema2. At exact source `76198c3b5321cc42ced8a0b4a3728aebfd89c2a4`, eleven Route regressions and six new flock controls passed **17/17**, exit0, from23:14:21Z to23:14:36Z after [prospective Claude FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6157157748). Applied updates0; training and M2 remain **HELD**, and M1's reply-quality target remains **SUSPENDED** under R14. Delivered by [PR #2395](https://github.com/UOR-Foundation/uor-r4/pull/2395), merged at `69206d854e4920dc21765345618db8cbfe78ebb1`. [Integration and finite qualification record](../labs/discrete-flock-writer-integration-2026-10-13/README.md).

The following #2379 status and finite result retain their prior **Route-only** scope; they do not qualify the new flock read.

**Status, 13 October 2026:** source `a2a5b89cdddd1b32bf32f0e3c18d6b21e994e523` passes all eleven admitted CUDA construction controls after [Claude FIT](https://github.com/UOR-Foundation/uor-r4/pull/2379#issuecomment-6156328158) and [PM acceptance](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6156348413). Attempt3 finished at22:24:58Z, exit0; source/executable and exact bars are unchanged. Three host metadata tests pass. Attempts1/2 remain **OUT_OF_REQUIRED_REVIEW_ORDER** observations. Delivery: [PR #2379](https://github.com/UOR-Foundation/uor-r4/pull/2379). All training holds remain **HELD**, `apply` has never executed, and M1's quality target remains **SUSPENDED** pending a discriminating reply grader under R14. [Canonical source/qualification record](../labs/discrete-geometric-learning-2026-10-13/README.md).

**Idea:** learn integer carriers, categorical geometric frames and optional ternary read gates directly. For a selected geometric parameter, replay the bounded context-to-writer path under identity and each of15 actual bivector moves, over baseline-fixed eligible rows. Choose minimum integer cost, identity first on ties, then lowest resulting port identity, then declared enumeration index. A flock gate instead compares its current trit and the two siblings in `[0,-1,+1]` order with the current value skipped; ties retain the earliest ordinal. The source commit rule permits only an improving move; eight disposable fixture updates now qualify its finite boundary, while corpus commits remain held. This replaces a floating-point optimizer/surrogate proposal with a finite exact geometric neighborhood; it does not establish that this neighborhood is sufficient for language learning.

**Implemented roles:** the existing exact 64-state signed-axis group supplies action/composition/inverse tables and actual BFS transport; its sixteen-root anchor orbit supplies 8D integer vector blocks. Ordered ternary FBS children/frontier retain the supplied context. In `ReadSpec::Route`, **selection is query-addressed:** the transformed query is compared with fixed branch/frontier roots, not leaf contents. Coordinate L1 orders route ports, coordinate Hamming breaks displacement ties, and stable port identity resolves remaining route ties. Leaf contents determine the carried payload. The query's own leaf can be selected; it is causal because that leaf belongs to the supplied prefix, not the next-token target. A shared CUDA codebook is used natively at token input and output, including codebook counterfactuals at all repeated input and target/candidate occurrences. Exact token identity resolves output ties; no hash-bit semantic metric is adopted.

In optional `ReadSpec::Flock { kept }`, the integrated [Claude design](../labs/discrete-murmuration-read-2026-10-13/README.md) selects up to1–8 earlier atoms or eligible same-level FBS binders by their own frame contents: frame L1, coordinate Hamming, recency, then stable node identity. Every binder descendant must precede the last valid query. A payload block uses another block of the same vector as its frame (`f(0)=blocks-1`, otherwise `f(b)=0`); actual BFS actions between source/query frames carry the payload rather than mapping every payload to the query. Binder summaries transport children into the first child's frame. An exact integer snap chooses an orbit root from the sum using the actual kept count. Ternary gates add, ignore or subtract each present level consensus, followed by an exact residual snap. All-zero gates return the query vector. This construction retains the same carrier and writer; gate monotonicity under L1 remains open, and neither collision freedom nor universal sensitivity is claimed.

The writer derives every token's product cell from its **current first/last vector root indices**, ranks admitted identities and separately scores omitted targets. Cells change with real or virtual codebook moves; token numbers do not define geometric admission. Bounded full-vocabulary scans preserve all identities in crowded cells and skip empty cells. The integer numerator is `8*token_hinge + width*address_hinge` over `8*width`, with one best token/address rival and margins `width`/`8`. An absent comparison contributes zero and an explicit zero rival count. Learning fixes eligibility from the baseline, skips baseline-ineligible rows, refuses wholly ineligible batches and makes rival-removing moves noncompetitive. A learned zero cannot be inferred from a vacuous comparison. This is a new objective, not a renamed old multi-rival or normalized-exponential law.

**Learning scope:** `Parameter::Route` changes one categorical frame in Route mode; `Parameter::VectorBlock` changes one codebook block by signed-axis action in either mode. `Device::propose` replays the same selected read/writer cost for every move on GPU, changes repeated input/candidate/target reads consistently (including flock frames and binder summaries), and keeps state unchanged. The exact 640→544 virtual improvement witness passed in admitted attempt3; it is a qualified finite proposal witness, not a committed learning or language result. In Flock mode, `Parameter::Gate` kind3 has three active trit candidates; its sixteen physical loss slots retain thirteen MAX sentinels. The exact gate-cost witness `[448,256,64]` selects ordinal2 without mutation. Route proposals in Flock mode and Gate proposals in Route mode refuse. The source commit path binds model identity/epoch and refuses stale/foreign handles. The owner hold prevents corpus updates; isolated authored fixture apply qualification is separately scoped, and no training result exists.

**Shared carrier/read contract:** exact token, occurrence, record and version remain separate from geometry and model epoch. `Device::carry` supplies compiler/store callers with vectors from the same table. `carry_windows` accepts1–64 prefixes with explicit lengths1..context and per-row ordered FBS plans. The query is each row's last valid token; structural padding has no selectable atom. Logical identities exclude padding, and full-length calls share the same plan ABI. The caller owns document/BOS segmentation and next-token target alignment. Outputs expose rival counts and admitted cells alongside vectors/IDs. Live-model and snapshot geometry/source/PTX/seed bindings reject foreign/stale handles. Vector collisions can remain; emission need not invert every identity.

Snapshot schema2 explicitly binds `ReadSpec` and the full `[blocks,stages]` gate inventory; missing fields/schema1 refuse rather than silently migrate. These join the live-model and snapshot geometry/source/PTX/seed bindings. Optional `forward_traced` and `Proposal::candidate_losses` expose completed GPU construction observations without host scoring.

**Driver implementation:** SHA-bound UORT tokens, binary response masks and explicit
tokenizer/BOS identity determine ascending target positions. Every eligible target
gets its own causal prefix, reset at its document BOS, with tails included exactly
once and rows/context bounded64/243. Seed controls initialization; corpus order and
the canonical vector-then-active-Route/Gate coordinate cycle are common across seeds,
with one coordinate per batch. Checkpoints bind configuration, source, data, masks,
tokenizer, roles, cursor and model storage. Evaluation uses CUDA top8/margin/eligibility
and TRAIN-masked count reference, with DEV oracle diagnostic only; zero-rival zero
loss is not eligible learning. Reports distinguish evaluation lineage and actual
snapshot identity. The complete transition loop accepts an opaque owner-admission
capability; default requests return `TrainingHeld`. The
[A2 admission controller](../labs/discrete-admission-2026-10-14/README.md) adds an
explicitly released seed1/1640-batch diagnostic pilot and trained-checkpoint
evaluation, gated by pinned source/artifact/data, Claude review and Casey D4
release records. Its changed receipt ABI passed all25 finite CUDA controls
at `35a48f586`, with11 fixture apply calls/8 disposable updates and0 corpus
updates;18 host metadata tests cover admission/persistence. Corpus controller
execution remains NOT_RUN; source delivery lifts no hold. `prepare-plan` is metadata only;
`init-corpus`/`eval-corpus` are CUDA entry points, with the latter still limited to
lineage-bound epoch0 checkpoints. The new `eval-pilot` retains the pilot release
and registered recovery-checkpoint provenance.

**What is not implemented or established:** full 240-root E8 coverage, the complete marked external FBS object, a migrated old floating-point trunk, a corpus learning result or measured D2 cost, a trained intent compiler, full M2 or D11 qualification, useful chat, language advantage or energy savings. Static geometric construction is build-time metadata; model initialization/reads/scoring/proposals execute only through the integer CUDA module, with no CPU model/reference fallback. [Prior floating-point evidence remains preserved](https://github.com/UOR-Foundation/uor-r4/pull/2370).

**Finite test result:** after required Claude FIT, admitted attempt3 passed the same eleven exact controls, exit0. They check literal writer costs, earlier-child transport, row isolation, collisions/omitted targets, zero-eligibility semantics, artifact/live identity, bounds, virtual proposals, unequal prefix lengths, target-independent admission and geometry-preserving token relabelling. First attempt c967 preserved one PASS, one CUDA launch-resource hard stop and nine NOT_RUN; the host-only launch repair rebuilt in0.73s and produced eleven observed PASS at a2. The law/kernel/controller/bars are unchanged. The earlier observations stay outside required review order. No `apply` belongs to that slice. The launch defect and review-order error are blockers, not mechanism negatives; counts/loss numerators alone are not language results.

A later learned comparison needs the owner hold release and Kimi's source-bound successor sheet: genuine trained-in discrete moves, fixed data/tokenizer/panels, paired seeds or sufficient fresh rows, a mechanism target and fresh transfer. No specific campaign is approved here. A negative would close only its declared configuration, not E8, FBS, quaternions or discrete geometry as mechanisms.

**Integrated finite result:** source `76198c3b5321cc42ced8a0b4a3728aebfd89c2a4` passed the eleven Route regressions plus `flock_identity_read`, `flock_gate_counterfactual`, `flock_transport_witnesses`, `flock_payload_witness`, `flock_selection_and_eligibility` and `flock_refusals_and_noop`, after the [flock sheet's prospective FIT](../harness/prep/m4-discrete-flock-construction-2026-10-13.md). These exercise the same-path optional trace, literal frame/payload transport, nonvacuity, ranking/recency, prefix/binder exclusion, independently rerun gate costs and no virtual mutation. C3 covers literal snap witnesses only; C9 monotonicity is diagnostic/open; C6 covers valid prefixes, not arbitrary masked children. Wrong-mode host refusals passed; shadowed GPU status9 was not exercised. All seventeen controls are finite construction evidence, not seventeen training runs. Applied updates0, no corpus run, no throughput or D11 result. The flock-specific source is `crates/uor-r4-discrete/src/flock_qualification.rs`; `src/lib.rs` and `src/main.rs` expose the read-mode/artifact and CLI bindings.

**Source:** `crates/uor-r4-discrete/{build.rs,src/metadata.rs,src/discrete.cu,src/device.rs,src/qualification.rs}`. [PM ownership](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154849555), [initial law](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6154897103), [Claude review](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155013108) and [PM consolidated corrections](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155105001) separate Codex's library/driver/writer, Claude's read and DeepSeek's geometry/binder review. The [required review-order correction](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6155397253) preserves both earlier attempts. Claude FIT is now accepted; DeepSeek's confirmation is non-blocking under [PM6156348413](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6156348413). [Finite sheet, FIT accepted/eleven admitted controls PASS](../harness/prep/m1-discrete-spiralcore-fbs-2026-10-13.md).
