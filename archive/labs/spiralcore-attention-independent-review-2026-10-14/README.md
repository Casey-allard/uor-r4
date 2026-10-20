# Independent adversarial review: SpiralCore reconstruction-native attention

Claude lab (Claude Fable 5.1, model id `claude-fable-5-1`, max reasoning effort, fresh
authenticated Claude Desktop Code-tab session on the owner's account; no credit purchase,
provider substitution or access limit) · 14 October 2026 · owner request relayed on
[#2029 (comment 6174925713)](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6174925713).

**Reviewed object:** PR #2480 at merge `bbbbdf7fbb28c5c8f6f5cd83ab7aecf48e7d4889`
([record](../spiralcore-native-attention-review-2026-10-14/README.md), its eight specialist
reviews and three evidence directories), the linked [first assessment](../spiralcore-attention-assessment-2026-10-14/README.md)
(#2478, `67c57781e`), the six original attachments it identifies, and the local migration
decision brief. The attention note (A1) is treated as a proposal. The Codex source snapshot is
`88c413599`; `git diff --stat 88c413599..ae3aa4dc3 -- crates/uor-r4-discrete crates/uor-r4-core/src/spiralcore_operator.rs`
is empty, so every source line cited below is identical on current `origin/main` (`ae3aa4dc3`).

**What ran:** read-only source analysis plus the three retained byte/data-structure
reproductions on the original attachments ([evidence](evidence/README.md)). No tokenizer, model,
reference, attention simulation, CUDA, pod, rental, training, active-run change, model adoption or
hold release. This is not a TEST FITNESS review; it does not block the seed-1 completion executor
or its Q2 scoring.

## 0. Verdict

**On the migration question as posed: REVISE.** There is no migration to decide. The brief bundles
four separable items, and they get four different answers:

| Item | Verdict | One-line reason |
|---|---|---|
| Exact compressed-data reads (Showcase DAG, range reader) | **PROCEED TO PROTOTYPE** as a Rust content-addressed store for the D19 durable-memory/application line | Byte/data-structure evidence is sound and reproduced here; it is storage, not attention, and the discrete model path has no consumer for it today (§2) |
| Geometric transport (root-label packing, ≤16K Flock index) | **INSUFFICIENT EVIDENCE for any change** | The derivations are correct; at the registered shape they buy nothing measurable and the context is capped at 243 (§3) |
| Relational attention | **REVISE, then PROCEED TO PROTOTYPE** of a smaller candidate | The C6/G64 energy controller is not the minimal falsifier; a bounded exact-relation reranker with a source-derived capability target (copy-successor reads, 3,847 of 14,287 DEV targets) is (§4) |
| Shared writer | **No change; report separately in every arm** | Admission is the measured ceiling (2,201 of 14,287 targets at Q1) and none of the four proposals touches it (§5) |

**Parallel testing versus replacement: neither is the right frame.** The discrete path already
dispatches read laws by a kernel mode word (`discrete.cu:204-207`, `s[8]`), checkpoints and
drivers already carry the read law in their lineage (`lib.rs:108-114`, `training.rs:287-299`),
and rollback is "select `route`". A third `ReadSpec` variant is the parallel path. Replacement of
SpiralCore transport, the FBS binder or the writer is proposed by no evidence anywhere in the
reviewed material, and the brief does not actually propose it either. Most of the brief's Stage A
machinery (new checkpoint format, interchangeable read strategies, storage boundary before the
head) already exists or is not needed for the first decisive comparison.

**The sequencing correction that matters most (learning credit, §6):** under the registered
schedule `canonical_vector_then_read_v1`, read parameters are proposed only in the last
`blocks × stages = 20` of the 16,404 batches of a pass (`training.rs:224-241`). The running seed-1
completion will therefore produce the project's first evidence of whether *any* read parameter is
ever accepted by the proposal learner, at cursor 16,404 (about 06:30Z on 15 October per #2029).
A trained-in head comparison registered before that evidence exists, under that schedule, would
not test the head. The discriminator in §10 changes the schedule for both arms and waits for the
seed-1 Route rows.

## 1. Inputs, identities and reproductions

| Input | SHA-256 | Read |
|---|---|---|
| A1 attention note (PDF, 4 pages) | `78f709022c40731700676c920dd9a6532cf83237f2e00703d7b9ef4feb545a0d` | full text, PDFKit extraction |
| A2 Showcase ZIP (`core.js` `7ca3dcfb…`, `transport.js` `1d254a75…`) | `6399240e51ad5f5347390690ce45f460eda390bbe1d9c5e86dee0e8fd5c26754` | `core.js`, `transport.js`, `README.md` in full |
| A3 validation paper (PDF, 9 pages) | `ce1a88df14d6d2b14e57b3c96fadfb0f52cfcfe0816ee55b9fce618c0b057294` | full text, PDFKit extraction |
| A4 Lawful but Unselected v2 | `4c61fc789e9773b5fe5ae0fc6a96f985074bca55cb357a0e7b061785391d1d4c` | full |
| A5 SIOPL RFC draft v0.1 | `7a63158c32513de412f879217b84e8497ceb777f80f470a7ed362ead2837802e` | full |
| A6 saved HTML (1,026,033 B) | `770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8` | regions 780-852, 994-1000, 2918-2960, 3405-3434, 10610-10626, 10676-10725, 10855-10875 |
| Migration decision brief | `e10d702c1ce3288229208f6545040f62633e95fa5e538f23473ca6c12c141669` | full |
| Relay prompt / request | `4e3d88ee…` / `02d60956…` | full |

All six attachment hashes match the first assessment's manifest. The reproductions below reran
the retained adapters, unmodified, against the original attachments from a copy of the scripts in
an owned scratch directory (`local/repro/`, gitignored):

| Reproduction | Exit | Wall | Max RSS | Result vs retained |
|---|---:|---:|---:|---|
| `check-source-codecs.cjs` on A6 (address/radix primitives) | 0 | 0.61 s | 70,451,200 B | `results.json` byte-identical |
| `check-current-golden-codec.cjs` on A6 (current golden score/pack/FBS) | 0 | 0.09 s | 65,568,768 B | `golden-results.json` byte-identical |
| `make-fixtures.cjs` on A2 (original `fold`/`recordsFor`) | 0 | 0.11 s | 62,439,424 B | `construction.json` identical except `constructionNanoseconds` |
| `rustc --edition=2021 -O range_reader.rs` | 0 | 1.23 s | 144,621,568 B | builds |
| `range-reader` on the fixtures (813 ranges, 2,374 checks) | 0 | 0.15 s | 3,276,800 B | `results.txt` byte-identical |

Node `v26.4.0`, `rustc 1.97.1`. This is an independent-lab reproduction of Codex's byte evidence
on a different machine session; it is still local byte work, not a model or network measurement.
The adapters execute the supplied JavaScript codec functions in isolated `vm` contexts (two
scripts) or by requiring the pinned Showcase modules (fixture adapter); nothing was rewritten.

## 2. Claim 1: exact compressed-data reads

**What is proved by source.** The Showcase folds 32-byte atoms into an ordered ternary DAG with
BLAKE2b-96 node identities (`core.js:161-192`), rebuilds by walking frontier roots whose scales
are the base-3 digits of the atom count (`core.js:207-250`), refuses missing nodes, and checks the
whole-object SHA-256 at the end (`core.js:247`). `recordsFor` with `bulk` serializes the whole
reachable subtree once per unique node (`transport.js:67-74`). The frontier identities
`R(N)=s₃(N)`, `C(N)=(N−s₃(N))/2` are correct and were exercised to N=10⁶ in the first assessment.

**What is measured here.** Exact selective reconstruction over the real serialized records: a
23-byte range inside a 32,768-byte unique object touches 8 authenticated records (384 serialized
bytes of 70,972); a removed unrelated leaf leaves the range readable and blocks full
reconstruction; a 7,776-byte object of three repeated atoms occupies 8 records (380 bytes) and
still reconstructs 243 ordered leaf occurrences. I reproduced all of it.

**Corrections and limits that change the decision.**

1. *It is only compression when the receiver already holds ancestry.* Unique content expands:
   70,972 record bytes for 32,768 input bytes here, 284,427 for 131,072 in the first assessment
   (2.17× in both). The saving is the warm/edit case. That is the right statement of the benefit
   and the brief should not call the store "compression before tokenization" without that
   condition.
2. *The discrete model path has no transfer step to compress.* The corpus loader consumes a
   SHA-pinned pretokenized UORT file and a pinned mask and builds each target's window from the
   in-memory token vector (`corpus.rs:143-186`, `246-290`). There is no byte transport, no
   network, no per-query reconstruction anywhere in the registered path. A compressed object
   store therefore has exactly one consumer in this project: the durable conversation memory
   and retrieval line of D19, where exact quotation, versioned records and selective expansion
   are the actual requirements. That line is where this work belongs, as a Rust crate with no
   model dependency.
3. *Byte tree ≠ model tree.* Codex says this correctly; the operational consequence is that the
   only model-facing contract the store needs is occurrence→byte-span binding plus the pinned
   tokenizer identity. Parent hashes, chunk boundaries and dictionary state must not be visible
   to the read (causality, §7).
4. *Whole-object integrity for partial reads.* The Rust reader verifies visited nodes against the
   root; it cannot freshly verify the whole SHA-256. A production API must distinguish
   `RangeVerifiedAgainstRoot` from `WholeObjectDigestVerified`; the CS review already names this.

**Strongest implementable form.** A Rust `uor-r4-object-store` crate: `fold`, `rebuild`, `range`,
`missing`, binary record format as in the Showcase (1-byte scale, 12-byte id, 32/36-byte data),
ingestion that tokenizes once with the pinned tokenizer and records occurrence→span bindings,
typed `Missing`/`SchemaMismatch`/`IntegrityFailure` results, and the existing byte controls
(range parity, refusal, UTF-8 boundary, dedup-with-multiplicity) as its test suite. No GPU, no
FIT: it is data preparation under D24. **PROCEED TO PROTOTYPE on that line; do not count it as
attention evidence.**

## 3. Claim 2: geometric transport (packing and the ≤16K index)

**What is proved by source.** The compiled geometry is a 64-element signed-axis group
(`build.rs:179-205`, `spiralcore_operator.rs:398-416`); the anchor `[2,2,0,0,0,0,0,0]` has an orbit
of exactly 16 roots (`build.rs:216-223`); every persistent block is validated against those roots
(`discrete.cu:346-351`); Route output is an action on an admitted root, Flock output is a snap to
a root (`discrete.cu:71`, `196-198`). So a four-bit root label per block is an exact representation
of persistent state, and `64 × 16` action / `16 × 16` distance tables are exact. By Lagrange, a
group of order 64 has no element of order 3, so a faithful 120° phase cycle is not inside it.
All of this I agree with.

**The ≤16K Flock index: correct, and I checked the kernel.** The eligibility test is
`level[node]==l && last[node]+1<length` (`discrete.cu:168`); the insertion comparator is exactly
`(L1 asc, Hamming asc, last desc, node asc)` (`171-181`); frames and query frame are orbit roots
(`138`, `145`, `148`); payload enters only after selection (`186-189`). Within a level, `last` is
unique (leaves have distinct positions; same-level parents have disjoint spans), so the node-ID
key never actually decides. The per-frame top-K dominance argument holds.

**Corrections and limits that change the decision.**

1. *The index is per frame block, not per payload block.* `frame_block(b)` is `blocks−1` for
   `b=0` and `0` otherwise (`discrete.cu:107`), so at B4 payload blocks 1, 2 and 3 share one query
   frame and one candidate frame and compute the identical top-K three times. Codex's "fix the
   block" is right but overcounts: there are two selection problems per level, not four. The
   same fact removes two of the four selections per level today (a 2× reduction of selection
   work), without any index.
2. *No payoff at the registered shape.* At C192 the BinderPlan has 192/64/21/7/2 nodes at levels
   0..4 (286 total, 5 stages). `16K=128` at K8 beats the eligible scan only at level 0 (at most
   192 → 128 comparisons of 16-root tables), and the comparison is a root-table lookup either
   way. Context is capped at 243 (`metadata.rs:5`), so no near-term shape changes this.
3. *Packing is not a throughput lever here.* The V4096/B4 codebook is 524,288 B of i32
   coordinates (8,192 B packed); a 64-row C192 carried batch is 1,572,864 B (24,576 B packed).
   Both are a few megabytes at most, far below the L2 cache of the qualified RTX 5090. The measured D2 forward
   (37.8 ms Route, 40.6 ms Flock at batch 64) is dominated by the writer's vocabulary pass:
   `writer` computes `token_cell` for all 4,096 tokens (two `root_id` searches of up to 16 × 8
   compares each) per row and again inside each of the 8 admitted cells (`discrete.cu:231`,
   `241`): about 5 × 10⁶ compares per row on average, up to 9.4 × 10⁶, before `token_distance`.
   Packing changes none of that.
4. *Invalidation.* A VectorBlock proposal on the frame block of a token changes the frame class
   of every occurrence of that token and of every parent whose leftmost leaf is that token
   (`discrete.cu:148`), without any epoch increment (the overlay is `24-33`; the epoch moves only
   at apply, `335` and `344`). The index would be rebuilt on most
   proposals during training. Codex states this; I confirm it and note it removes the training
   case from the index's benefit entirely.
5. *The nibble-to-root byte code is a relabeling.* Encoding a byte as two of sixteen roots,
   transporting and inverting is exact and carries exactly the same eight bits. It is a correct
   statement about the geometry and no mechanism; it must not appear in any cost or capability
   claim.

**Verdict: INSUFFICIENT EVIDENCE for any change.** Keep both as documented exact options; the
condition to reopen them is a registered shape with context well above 243 or blocks well above
4, which does not exist and is not planned.

## 4. Claim 3: relational attention

### 4.1 What survives of the note after the two Codex reports

The first assessment's counterexamples to A1 (positive coupling need not lock at zero offset;
one negative edge prefers 180°, 120° triads need a frustrated triangle; PAS=1 for any identical
phases; orthogonal carriers do not make represented variables independent; kurtosis is
label-permutation invariant; the log frontier does not bound retained history; A1's own recipe
uses AdamW/Muon and cross-entropy, excluded by D23/D32) are correct and I do not re-argue them.
A1 is four pages and supplies no node domain, input map, readout or finite update law; its
numbered references [1]-[25] have no bibliography. The validation paper (A3) is about transport,
not attention: its CHORDLOCK/Voronoi section quantizes a continuous Clifford-torus trajectory to
120 icosians, which is a different object from token selection.

The surviving hypothesis, narrowed by Codex's ML cross-review, is the right one: **candidate-
candidate relations can change which immutable evidence references are selected while each
candidate's own query-distance keys stay fixed.** Everything below is about testing exactly
that.

### 4.2 Counterexample to the energy-controller framing

Both finite realizations in #2480 (G64 root-state energy; typed C6 phase controller) are
*settling* designs: sparse integer energy, strict-descent moves, a sweep cap, deterministic ties,
then a readout. The cap is required anyway (the termination bound only limits accepted moves,
not trials). But a sparse update rule run for at most S sweeps over a fixed bounded graph is a
deterministic S-sweep aggregate of its neighbourhood, and on the model tree the only graph in
play has at most 286 nodes and depth 5. So the controller, as specified, *is* a bounded sweep
reranker plus an energy that decides acceptance. The energy, the descent proof, the tie policy
for rejected trials and the cap-hit output are all machinery for a property (local
energy-optimality) that the readout never consumes. The minimal falsifier of the hypothesis is
the sweep aggregate alone. Adopting the controller first would make a negative result
uninterpretable: it could be the energy law, the schedule, or the hypothesis.

### 4.3 A source-derived capability gap the hypothesis can target

Neither registered read can perform a **copy-successor** (induction) read: "the query token
occurred earlier; carry the payload of the token that followed it."

- Route selects a path by comparing the transformed query frame against fixed port roots and
  carries the payload of the leaf it reaches (`discrete.cu:56-71`). Selection does not depend on
  which tokens the leaves hold (Codex states this too).
- Flock ranks each eligible node by *its own* frame root's distance to the query frame, then
  recency (`discrete.cu:167-182`), and carries *that node's own* payload (`186-189`). With the
  query token A at position t and an earlier A at p, the node at p ranks first in its frame
  class (a same-token node always has the query's frame root; collisions only add other tokens
  to that class) and Flock carries A's payload again. The payload at p+1 is ranked only by its own frame, which is unrelated to
  the match. Nothing in either read can express "rank p+1 by the frame at p".

This is precisely a relation that changes a candidate's rank while its own keys stay fixed, and
it is the classic relational primitive (previous-token match → successor) that contextual copy
behaviour needs. **It is also large on the registered panel:** on the pinned DEV corpus,
3,847 of the 14,287 eligible targets (26.9%) equal the token that followed the most recent
earlier occurrence of the query token inside the 192-token window; 5,182 (36.3%) equal the
successor of some earlier occurrence; 8,248 (57.7%) appear somewhere in the window
([data statistic](evidence/README.md), no model involved). The seed-1 model has 1,090 top-8
hits at Q1 and the count reference has 4,060 on the same panel. It is also exactly the owner's murmuration picture: a local rule between
neighbouring nodes, global behaviour from repeated local application. It does not need phases,
energies, 240 roots or a new input alphabet.

### 4.4 The strongest implementable interpretation (the design I recommend)

`ReadSpec::Relation { kept, sweeps }`, a third kernel mode beside Route and Flock, sharing Flock's
node construction (`discrete.cu:123-155`) and its payload transport, combine and snap.

- **Edges** are generated only from retained structure, with bounded degree and no scan beyond
  the node list: parent/child (plan words), siblings (same parent), sequential successor
  (`last[j]+1 == first[i]` at the same level; for leaves `atom+1`), same-token (equality of
  retained token IDs, `ids`), same-frame (equality of frame roots). All are compare/gather.
  Every edge endpoint must satisfy `last ≤ length−1`; the successor of a match may be the query
  position itself, never beyond it.
- **State** per node is a small integer score vector in a declared range, initialized from the
  node's own Flock keys; **one sweep** adds, for each edge type e at level l, a learned trit
  `g[e,l] ∈ {−1,0,+1}` times the neighbour's match score (its own frame distance to the query,
  tabled). `sweeps` is a declared constant (1 is enough for successor-of-match; depth 5 reaches
  the whole tree). Integer, bounded, deterministic, no energy.
- **Selection** keeps the top-K by the same comparator as Flock with the relation score as the
  first key and Flock's keys as the tie order; relation off (all trits 0) reproduces Flock
  exactly, which is the construction control that matters most (§9).
- **Learning slots** are the existing Gate parameter kind (`lib.rs:125-131`, kind 3, three
  ordinals, `discrete.cu:88-94`, `315-317`, `329-335`): no new proposal kind and no new
  counterfactual lane. Three existing checks must admit the new mode, and they are the whole
  kernel-side learner change: the Gate-only-in-Flock refusals (`discrete.cu:285`, `330`), the
  `blocks × stages` index bounds (`286`, `331`), and the two-arm `ReadSpec` match in
  `coordinate()` (`training.rs:236-239`). The new inventory is `edge_types × stages` trits
  (5 edge types × 5 stages = 25 at C192) plus Flock's existing `blocks × stages` combine gates.
- **Everything downstream is unchanged:** BFS payload transport, integer sum, `snap`, the
  writer, the objective, the epoch rule.
- **Cost at the registered shape:** per row and frame block, 286 nodes × ≤8 edges × sweeps
  table operations ≈ 10⁴ integer ops, against the writer's ≈ 5 × 10⁶ compares per row (about
  0.2%). Trace fields
  extend Flock's level trace with the relation score of each kept node.

This is a strict superset of Flock, so the comparison "Relation versus Flock" is one of trits;
the registered comparison is still against Route (§10).

**Verdict: REVISE the design to this, then PROCEED TO PROTOTYPE** as source-only work (kernel
mode, `init-read` CLI, validate status, trace, construction controls), with a prospective prep
sheet before any GPU qualification. The C6 controller and the instruction-native input alphabet
stay as later research, not as the first artifact.

## 5. Claim 4: the shared writer

Admission is the eight occupied product cells nearest to the read output's **endpoint blocks
only** (`address_distance` uses blocks 0 and B−1, `discrete.cu:208-210`, `235-239`); the target's
identity never enters admission (`233`, `240`); ranking inside admitted cells is full-vector L1
with token-ID ties (`241-245`). Two structural consequences for any head comparison:

1. Blocks 1 and 2 of the read output cannot affect admission at all; they affect only in-cell
   rank. A head that improves selection on those blocks can raise top-1 inside admitted cells
   and leave the admitted count unchanged.
2. The measured ceiling is admission: Q1 (seed 1, checkpoint 4,101) admitted 2,201 of 14,287
   DEV targets with 0 hits outside admitted rows; the pilot went 5,680 → 2,180 while top-8 rose
   173 → 1,169 ([Q1 post](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6174053268),
   [diagnosis](../a2-pilot-diagnosis-2026-10-14/README.md)). The count reference (top-8 4,060)
   is still above the model. None of the four proposals changes `kept_cells`, the cell law or
   the ranking law, and none should in the first comparison.

**Verdict: no change; admitted-target count, in-admitted-cell rank and loss components are
mandatory separate observables in every arm.** The brief already says this; the source makes it
exact.

## 6. Learning credit: the schedule fact

`PreparedDriver::coordinate` maps batch index to `VectorBlock` for the first
`vocabulary × blocks = 16,384` coordinates and to `Route` or `Gate` for the remaining
`blocks × stages = 20` (`training.rs:224-241`); one coordinate per batch, no wrap inside a pass
(`plan()`, `training.rs:287-292`, inventory at `177-185`; release record lines 140-142). The registered seed-1 completion
is exactly one pass (16,404 batches), so Route coordinates are attempted 20 times in total, at
batches 16,384-16,403, and the Q1 post confirms "Route not yet reached (starts at 16,384)".

Consequences:

- Whether the proposal learner ever accepts a read-parameter move is **currently unmeasured**;
  the first measurement arrives with seed-1's 16,404 checkpoint. A Relation head whose slots are
  Gate-kind trits would receive the same 20-ish proposals per pass under this schedule and would
  not be trained in. Codex says "an arm that never updates them is not a trained-in test"; the
  source shows the registered schedule is such an arm.
- The discriminator needs a declared interleaved schedule (for example every eighth batch takes
  the next read coordinate, wrapping) applied to **both** arms. That is a learner change with its
  own tiny construction control (`validate_cursor` arithmetic, `training.rs:244-259`) and it
  must be registered as a change of the learner, not smuggled in with the head.
- Sensitivity is already instrumented (`discrete_choose` reports `sensitive`,
  `discrete.cu:313-314`; Q1 reports attempted/any-candidate-differs/accepted per kind). Report
  the same three counters for each new parameter family.

## 7. Causality, identity, cache

- Flock's eligibility excludes the query and every node ending at or after it
  (`discrete.cu:168`); the BinderPlan is built for the full window while `length` is the valid
  prefix, so any new edge generator must filter both endpoints by `last ≤ length−1` in the
  kernel. The successor edge reaches at most the query position.
- Occurrence identity (`Identity { token, occurrence, record, version }`, `lib.rs:116-124`)
  is retained per window position; same-token edges compare retained IDs, never geometry, and
  do not change multiplicity. This is consistent with the owner's rule that identity stays
  separate from geometric address.
- Any store-side summary visible to the read must be built from the permitted prefix only; a
  Showcase-style parent hash spans children across the cut and must not be exposed. Codex's
  suffix/target isolation control is the right test and I add nothing to it.
- Cache keys during learning must include the virtual overlay, not only the epoch
  (`discrete.cu:24-33`); for the Relation head no cache is proposed, so this applies only to the
  deferred index.

## 8. Total cost at the registered shape

| Item | Model-path compute | Storage | Learner | Comment |
|---|---|---|---|---|
| Object store (D19 line) | none | new, outside the model path | none | CPU data preparation; warm-case benefit only |
| Root packing | none expected (cache-resident already) | −504 KiB codebook (516,096 B), −1.5 MiB per carried batch | none | deferred |
| ≤16K index | ≤ level-0 scan only; rebuilt on most proposals | +per-level lists | none | deferred |
| Relation read | ≈0.2% of the writer per row per frame block; trace +1 word per kept node | none | +25 trits (Gate kind) | §4.4 |
| Interleaved schedule | none | none | redistributes proposals; no new compute | required for any read-law test |
| Trained-in discriminator | two full passes (Route-interleaved, Relation-interleaved) ≈ 2 × 17.07 h on one RTX 5090 ≈ $30 at the recorded $0.892/h | checkpoints as today | — | within the ≤4-pod/≤$8/h caps; needs FIT and owner release |

Review cost: zero GPU, zero pod, zero paid resource; local byte reproductions and the DEV
statistic ≈ 2.4 s wall total.

## 9. Minimal reversible design

1. **M0, source only, now:** add `ReadSpec::Relation { kept, sweeps }` (`lib.rs`), kernel
   `relation_vector` sharing Flock's construction, `read_vector` dispatch for mode 2, `discrete_validate`
   status for the new mode/range, trace extension, `init-read` CLI acceptance, Gate-kind inventory
   `= blocks × stages + edge_types × stages` with the Gate mode checks and index bounds in
   `discrete_counterfactual`/`discrete_apply` (`discrete.cu:285-286`, `330-331`) and the
   `ReadSpec` match in `coordinate()` (`training.rs:236-239`) extended to the third variant.
   Older checkpoints load under their own `read_spec`;
   a Relation checkpoint fails lineage validation on a Route/Flock driver by the existing check
   (`training.rs:293-299`). Rollback is the registered `read_spec: route`; nothing destructive.
2. **M1, construction controls (prep sheet → FIT):** relation-off equals Flock on identical
   inputs and traces; copy-successor fixture (`x A B y z A` with distinct frame roots: Route and
   Flock cannot select B's occurrence except by frame coincidence; successor trit +1 selects it,
   verified by the kept-node trace); irrelevant-change control (change `y`,`z`); conflicting
   successors (`A B … A C … A`: most recent wins by the recency key); suffix/target isolation;
   `last ≤ length−1` boundary; deterministic ties; bounded work (`sweeps`); the existing 11 Route
   and 6 Flock base controls unchanged; `admit_ptx` with zero float instructions.
3. **M2, after FIT and the owner's release:** the discriminator in §10.
4. **Separately, any time, no FIT:** the Rust object store on the D19 line (§2).

Explicitly not in this design: tokenizer changes, instruction-native input alphabet, 240-root/15-orbit
carrier, C6 controller, root packing, the Flock index, a new checkpoint format, a second product
fork, any writer change.

## 10. A fair trained-in discriminator

- **Arms:** Route-interleaved and Relation-interleaved, same D5 initializer, data, masks,
  tokenizer, writer, batch rows, seed, panel and evaluator lineage; only `read_spec` differs.
  The existing canonical-schedule Route runs (seed 1 now, seed 2 when released) measure the
  schedule effect for free, so the decisive pair shares one learner.
- **Primary pre-registered measure:** top-8 and admitted-target count on the **copy-successor
  sub-panel** of DEV: rows whose target equals the token following the most recent earlier
  occurrence of the query token inside the window. This is a data statistic computable without
  the model: the standalone Rust tool in [evidence](evidence/README.md) counts **3,847 such
  rows** on the pinned DEV corpus, far above D22's 200-row floor. The bar is Relation's
  sub-panel top-8 above Route's by more than the seed-to-seed difference of the two canonical
  Route runs on the same sub-panel; no other tolerance.
- **Secondary:** full-panel top-1/top-8, admitted targets, token/address hinge components,
  attempted/sensitive/accepted per parameter family, fresh-transfer rows (D22), forward and
  proposal wall time (D2 method), decode tokens/s.
- **Attribution:** evaluate the trained Relation checkpoint once more with all relation trits
  forced to 0 at evaluation time (no retraining): the difference isolates selection interaction
  at the trained state.
- **Closure rule:** a Relation arm at or below Route on the sub-panel closes this configuration
  (one edge set, one sweep count, this shape), not the mechanism (D22).

## 11. Rollback and cutover criteria

- **Rollback:** `read_spec: route`, the registered checkpoints, no conversion, no data or
  tokenizer change. The active seed-1 run is untouched throughout.
- **Cutover (Relation becomes the registered arm):** M1 controls PASS at the exact head; the
  §10 primary bar met; full-panel top-1/top-8 and admitted targets not below Route beyond the
  measured seed noise; cost reported in forward ms, proposal s and decode tokens/s for the
  owner's quality/cost decision. No numeric cost tolerance is set here.

## 12. Corrections to #2480 and the brief, with evidence

| # | Statement under review | Correction | Evidence |
|---|---|---|---|
| 1 | "Fix the block" for the ≤16K index (README §5) | Index and selection are per frame block; blocks 1-3 share one selection at B4 | `discrete.cu:107`, `134-138` |
| 2 | Index/packing as migration candidates | No measurable benefit at C192/B4/V4096; context capped at 243 | `metadata.rs:5`; level counts 192/64/21/7/2; D2 timings |
| 3 | Relational head = C6 or G64 energy controller | A bounded sweep reranker is the minimal falsifier; the energy is unconsumed machinery | §4.2 |
| 4 | "Current evidence does not show Flock has failed this task" | True, and stronger: the copy-successor read is expressible by neither registered read, by source | `discrete.cu:56-71`, `167-189` |
| 5 | Brief Stage A: new checkpoint format must record the read strategy | Already true: `read_spec` is in `InitConfig` and driver lineage; checkpoints are refused across lineages | `lib.rs:108-114`, `training.rs:293-299` |
| 6 | Brief: "develop one bounded successor prototype alongside" | Correct in the narrow sense (a third `ReadSpec`), overbuilt in the broad sense (storage boundary, read-strategy versioning, two product forms) | §0, §9 |
| 7 | Trained-in comparison "with an actual learning schedule that trains the candidate" | The registered schedule gives read parameters 20 of 16,404 proposals; an interleaved schedule is a prerequisite for both arms | `training.rs:224-241`; release record 140-142; Q1 post |
| 8 | "Compression before tokenization" as a model-path saving | No transfer step exists in the model path; the consumer is D19 memory | `corpus.rs:143-186`, `246-290` |
| 9 | Showcase/A3 "compression" | Unique content expands 2.17× in records; benefit is warm/edit reuse | first assessment table; `construction.json` |
| 10 | Parent-frame citation line 144 (integrator) | 148, as the math cross-review already corrected | `discrete.cu:148` |
| 11 | Writer repair expectation (brief, correctly cautious) | Admission depends on endpoint blocks only; any head affects it only through blocks 0 and B−1 | `discrete.cu:208-210` |

Nothing above is a tolerance or a technicality; each row names the source line or receipt.

## 13. Proof, measurement, hypothesis

- **Proved (source/derivation):** orbit of 16; group of order 64 without order-3 elements;
  exact signed-permutation transport; per-frame top-K dominance for the present comparator;
  frame-block sharing across payload blocks; copy-successor inexpressibility in Route and
  Flock; schedule arithmetic; writer admission depends on endpoint blocks only.
- **Measured (reproduced here):** the three byte/data-structure suites, byte-identical to the
  retained outputs; timings in §1.
- **Measured (retained receipts, not rerun):** Q1 and pilot admission/rank/loss numbers; D2 API
  timings; first-assessment transfer tables.
- **Measured (data statistic, this review):** 3,847 / 5,182 / 8,248 copy-successor,
  any-successor and copy-ceiling rows of the 14,287 DEV targets (evidence README).
- **Hypotheses:** that relation-dependent selection improves grounded completion at this shape;
  that a trained Relation read converts a useful fraction of the 3,847 recoverable rows into
  admitted top-8 hits; that proposal learning can train any read parameter at all (first
  evidence due at seed-1 cursor 16,404).

## 14. What this review does not claim

No language result, no throughput result, no FIT, no adoption, no owner-hold change, no claim
about physical oscillators, audio channels or networking conformance beyond the reproduced
primitives. Codex's architectural statement ("coherent, neither established nor ruled out")
stands; this review changes the sequence, the size of the first artifact and the question it
must answer.

Line: owner-requested independent architecture review · count 0/3 (no model line advanced) ·
headline unchanged.
