# ConstructionCausalReturnV1 construction-transfer gate — issue #983

## Status and decision

- **Issue:** [#983](https://github.com/UOR-Foundation/uor-r4/issues/983)
- **Date:** 2026-08-28
- **Terminal bounded result:** `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`
- **Raw census identity:**
  `blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017`
- **Sealed outcome identity:**
  `blake3:58fba09dba1b9245cb62a73bf8e3ac153242dc0730e3df7586446aa2820d4587`

The label-free construction map was internally pure, but it structurally
covered **0/6** validation decisions. Attaching the separately sealed labels
after the raw census produced a strict construction-transfer ceiling of
**0/6**, with six abstentions. The predeclared hard gate therefore failed
before any deployed selector or payload inversion was run. #953 generation was
not run.

This is a bounded negative for this frozen candidate-conditioned causal-return
representation, placement, construction population, and validation envelope.
It does not rule out H4 state, paired-H4/E8 structure, exact trigonometric
channels, ordered prime or semiprime channels, alternative corpus-induced
placement, or a different candidate-relative attention mechanism.

## Frozen mechanism

For an observed route history `x_1 ... x_t` and naturally admitted candidate
`c`, the frozen exact H4 construction is:

```text
P_0    = identity
P_i    = P_(i-1) * L(x_i)
S_i    = P_i^-1 * P_t
R_i(c) = ((S_i * L(c)) * S_i^-1) * L(c)^-1
```

The current prefix `i=t` is excluded. Each candidate has eight fixed slots.
Unused slots contain exact identity `P/S/R` witnesses with `occupied=false`;
they are typed padding and cannot alias an occupied identity event.

Each occupied class event retains:

```text
(exact signed H4 relation coordinate,
 angular shell,
 observed lease age,
 multiplicity,
 occupancy)
```

`R_full` is the complete ordered eight-slot word. `R_min` is the least occupied
event under the frozen comparison order:

```text
angular shell ascending,
multiplicity descending,
observed lease age ascending.
```

The canonical class-field serialization order remains coordinate, shell,
lease age, multiplicity, occupancy. A pure `R_min` resolves directly. Only an
impure `R_min` may promote to its covered `R_full` classes, and those rich
classes must be pure. The prospective selector rule was frozen as “select iff
exactly one admitted candidate is `SELECT` and the other is `REJECT`,” but no
deployed selector type is present in this Gate 0 harness.

The frozen product orientation is row-major `left * right`, quaternion basis
`(1,i,j,k)`, right-handed. Candidate admission remains the existing natural
schema-2 support path; the mechanism does not manufacture candidates.

## Frozen populations and identity boundary

The construction partition contains 12 distinct observed transitions over six
candidate identities. Each transition contributes the observed candidate and
its matched reject candidate, giving 24 construction observation rows. Each
candidate has two observed prototypes and two matched reject rows.

Validation contains six independent decisions and 12 candidate rows. The
public validation IDs are opaque:

```text
V-3f2c9a71  V-8d04e6b5  V-51ab7c90
V-c7e2384d  V-046fd1a3  V-b9a572ce
```

The raw census completed and was pinned before the validation-label join was
loaded. Raw reports contain no expected candidate, target-label role, or
future continuation field. The sealed label join was attached only for the
post-raw strict-ceiling lookup.

## Bound identities

| Object | Frozen identity |
|---|---|
| Fixture partition | `blake3:289cab4b5e22d45a61324137f2cc229c473570e6b9d3358dec16652dbfc84f83` |
| Lexical codec | `blake3:b1e8baf2ad3e6b9eb58f8d8c06809e76f37bdf91bfcce462a4e868e9952d654a` |
| Vocabulary | `blake3:bd8c3c458f0c92fbbf9003b722417335678a1adfdfd7a7c0c68580dc129ffb75` |
| Codec/vocabulary record | `blake3:0c45cd205ab0b14197ae178a668dbbbc893fc5c4e7beae97aeaf8a0be4b8be61` |
| Rebuilt route artifact | `blake3:45c2475983fb047a9cedda6ef28edee832326cf730faeb5b590b7afc305505db` |
| Embedded attention manifest | `blake3:3e47dc0475c8f9da017ec1df456485d0c4957fe2203022dce7dcb537576b659a` |
| Construction artifact record | `blake3:c4bca03f4c06d2ce58ed7167842b72491f4c9275c25fa9599f7bf30f54b1a670` |
| Mechanism policy | `blake3:0ab5118269a6aacbb4293ad876edcc82bf8f4ecca8b2121409b2bd8e0ff887c0` |
| Label-free validation input | `blake3:17246f23c14a81ea83d388e5592af634027d07625ad7bdb1fdf743c1b562712a` |
| Construction partition | `blake3:e9eaaa021752335149cd6cbd5fd17faac9e31f61db6741532292e6c6d1ef58ba` |
| Compiler/query frame | `blake3:fe0fa1d5f56e97ba6508bf290c644b059caf4d23a96296e078b78dc522767c1a` |
| H4 root table | `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` |
| H4 multiplication table | `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759` |
| Raw Gate 0 census | `blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017` |
| Sealed validation-label join | `blake3:5aa4b1c5880f660bc463a0650ebf21603d63170252b56e03e9a93e0a07e28c76` |
| Sealed Gate 0 outcome | `blake3:58fba09dba1b9245cb62a73bf8e3ac153242dc0730e3df7586446aa2820d4587` |

The corresponding typed record identities are
`uor-r4.construction-causal-return-fixture/1`,
`uor-r4.construction-causal-return-codec/1`,
`uor-r4.construction-causal-return-artifact/1`,
`uor-r4.construction-causal-return-class-map/1`,
`uor-r4.construction-causal-return-policy/1`,
`uor-r4.construction-causal-return-validation-input/1`,
`uor-r4.construction-causal-return-raw-census/1`,
`uor-r4.construction-causal-return-construction-label-join/1`,
`uor-r4.construction-causal-return-validation-label-join/1`, and
`uor-r4.construction-causal-return-outcome/1`.

## Label-free raw census

| Measure | Real arm result |
|---|---:|
| Construction observation rows | 24 |
| `R_min` classes | 21 |
| `R_full` classes | 24 |
| Promoted minimum classes | 2 / 21 |
| Rows in promoted classes | 4 / 24 |
| Usable direct or promoted rich classes pure | yes |
| Structurally covered validation decisions | 0 / 6 |
| Populated/padding aliases | 0 |

The complete ordered 21-class inventory is serialized inside the canonical raw
record rather than duplicated as lossy prose. It includes every typed
`R_min`, every covered `R_full`, construction/select/reject multiplicities,
direct actions, promotion flags, and rich-class purity. The lookup maps
themselves are not JSON object keys because their exact typed H4 tuples are not
valid JSON keys; the canonical ordered inventory is the serialization source
and is bound by the raw-census identity above.

The following compact, lossless index makes all 21 ordered minimum classes and
their 24 associated rich classes reviewable. `m` is the `R_min` record kappa,
`c` is the complete minimum-class record kappa, `n/s/r` are construction/select/
reject row counts, `a` is the direct action (`P` means promotion), and `f` lists
the covered `R_full` kappas in canonical order:

```text
00 m=9f3a736fa5f6702cd70c99d4a6192b9ab1ba03f112478f75183a7cb34f30f794 c=5dcba7574ad0c2cccb573d62cf737d44f2d2ea9380e8a5701229a289a80e243d n/s/r=1/0/1 a=REJECT f=b72fd36252c1e60bec7b230854461af107adb62f46d6360a7a3c22254d7d4df8
01 m=9773f204ce588062c61ef7b15a8ed3574f64b0bddc0bf67c42bef2a4ff883fff c=9d92aa101b1b6f72395b33377a8f3feb512b995dedf561d3d18616a44418caac n/s/r=2/1/1 a=P f=33f9c6538ce6ed93c6f995c4aea6ef09b3f2d1804b4b7c916f91d693e8748a7d,e42ec06f1dff86ed04132e1166a594d68da512530f0afd6e154af743822ac1d9
02 m=817318a09044f814fafa41001f5df92a77a87d446e9d032dc677e735c804fd57 c=56b6c6d483b8c8849dcebcb10a67554c8f5877e5edf5a40a8bec109a91e24cee n/s/r=2/1/1 a=P f=3fb80eeeaff5c64135f11465574368a439944c3481dd56a95f50166f9ba99eec,59e47a8cd9c833bd8015e5d9cf3e08b44846e30bb409868e606a6e008fb84420
03 m=efdf22428d10db02b46a77c7276cc3a20ec6c1177451696dc42977b1e83e6176 c=193828a813e7259c367104bb47b939e9b650016aabdcc569a23834c7395d448e n/s/r=1/0/1 a=REJECT f=82b2bace5e9401d3a460134c343c47f213d4d9ca3e7679c4fd0d16ebb4863a21
04 m=8376bc5569e8f9120e6ffc3f336b1bf129c39186dccfaf3c740a0429c2091ab4 c=eb037dd10242c1027d9e5ffa7c8cb496ec3e42c19c6988fbf758bb688a4879a3 n/s/r=1/1/0 a=SELECT f=515b9aeb0da473d672f39d1bb512aa6cf6cf7e90303ec02fe600e108a5dc626a
05 m=31e7ae408c6ab1cdc2099dd4a9a1ae326344305016c50c380ed14d36aea82813 c=3bdc844e224350d35dfc6acc9b91a2e65af28ac627170631d2bef44a4d80fd87 n/s/r=1/1/0 a=SELECT f=9d70cad18b62b6a01c4b50d84c3a033f844dada6ae953bdae1bc294b7721a8bf
06 m=150812e0a27f807cd336f9421d7919da22cb4487605726820e3161880e7fe249 c=3b019a31b96e97fb203ae8ba87dc2d812c8cd2bb863f7a3b50e8c467a92525eb n/s/r=1/1/0 a=SELECT f=6133725b6c8ba9f0f3c07cd7aa882f1d7014d506a1a4e727d9f58817ab68315d
07 m=0ec4b64a3eb00ad9269085d00706c92713e3a0cf4441ddeac675239effd7a80d c=b9e7ae2bcb759d9876976393c3a900c32f5d6ecf2442b74d1663f0ba186a7d4f n/s/r=1/1/0 a=SELECT f=6cfea888a6272b4363ec4f1c7d877d4104b9e7a31ae5be3a2b16af1399009e53
08 m=58397177973bfc9c1215940e42cdbab47cf7f7efb6c230f679e63810c54243b8 c=d67a391c3e5b5b065ea0356dce1a9553c40ffb057a2d9587e6e5efb1c445ea69 n/s/r=1/1/0 a=SELECT f=bf0229e9c9bbe18d5d47bc27f4a8fb5fe7e3becdbc05637d2cc52c2e90e45d10
09 m=56c04bed54ece293c8b9a295503312e3ce4a2ef1f7ef0a6030717e6fa03a95ab c=bffab32c6126fdcaff0d21d76bec1fd6b1f3695dc50e165611ec06e35fa4a83f n/s/r=1/0/1 a=REJECT f=30cd6406fd700760f805b9c1c0859af6e86da8cb9d4e2e4bea9192349efa81a3
10 m=78eee11ef3d1439ce622425e875f71d207a5ac1649a51eb5b01b771cd13b9953 c=19700bf850d90717eae2c73557ef855d571d06029654ca26899f6b063976e0a0 n/s/r=1/1/0 a=SELECT f=374ab58b6187744f2e5a6cb5451a3807510c959081cd6ebea0a21d008d6d47ea
11 m=c03369a4bd08749f289eb201af77e495299ff5ec6cbbbd392ea6ad080c3bf3e2 c=8f31026b57085ef6a66d251561043fc19ad865eee6d7bac3f93944fa5e682685 n/s/r=1/0/1 a=REJECT f=054d2be254695779564f2f4d42d2f4774ab75824813a9028434ecc5b08ced4f8
12 m=4f1fb2386f4461a586ee5e1c557d9c7d4ccc09c2945803ff9ddac720c61050c5 c=7aba5598b655dc6bec1b41909aca2dbe1aa1003751f0e3ed243ffba4bfd11673 n/s/r=1/0/1 a=REJECT f=8bb6e9f2b6fc7f9920dd21a3031b93b4f02ee9858b0eb137f772a5a2af515902
13 m=24dc71b99e15e5cd110d458852fdcfb5579db9f3075c310190458d9c4f0ae864 c=9921f4878817e13d3ff1595f60c58013f5cd5234c15ad5c0a2d26e83fd1bd6a9 n/s/r=2/2/0 a=SELECT f=989c5a8cea631669196144647433c8e55325ef57c414ba89b46f0a9b4d81d48e,b561877b541191aa901e4c2d9e5abe831b7112a0517134af5377091d846daad3
14 m=db40cdd8d8bebd66fa0c1fdfa8a48b6e727e44445f3ba1b43f4f2ff57b22c19c c=f3bd7fce09b0f1b5303dc98a5560ec59dd64133da3ccc3f8b64e88e058aa1e2f n/s/r=1/1/0 a=SELECT f=721725754f4c4203e355fc22f515a3d67851e00c5f4c81f204e09640abaf6df3
15 m=220c090f2b80f737e691b1277c5b36fc0a6732797e546363ac1f4b272fe07a70 c=46ed524692d6450647efea178a90c310cf33cfa71f761ffce61364494280fdb2 n/s/r=1/0/1 a=REJECT f=628d8745cd38af1027c3b79b76b93c3366f11e3d4cab6a2254b4d8efdbe05374
16 m=b4b52538fe283b4bf91186ecb1d2ad1e02f01be6d5e8b2d5879efc4a877fe8c0 c=d675e91f2b141a41a43fb4b75e1c4dfcf52d3b243b6f93e2745cfa190ecd612e n/s/r=1/0/1 a=REJECT f=5877cf235b124da5ef03ea9ede478148813fb3496ae705052903ed454c640826
17 m=14bcf318e5c989e81d6b1676208a57fa0118081dace2de18c8b78bdabecb619e c=ebf98d508cfa59324225520c6fed86293a6f66071acf040c5836017ffbf586a3 n/s/r=1/0/1 a=REJECT f=bb015a076004b5a8a5ecc536e5d50e84780ccebe489e7ffa9ba9e784ac45907b
18 m=9ac423d2055ec656e3c9f7fd2a7b7bcfc95e61fc833acc8743960f7f01f3ce66 c=7653a6e79c95bcba5bda6bfcbfd15ecfcffc23932cc21c6ca02d5e700d0ed0ea n/s/r=1/0/1 a=REJECT f=ec77d27d110ce5e87b293e8cbbdc3bad28be7fc4e2d6fff39ba3930915bbcb57
19 m=76bd0d302974a509f5031e62b2dc0a6b0bebf573f10dbc513d06f93a9d278128 c=690c0ea2d9cc1c0d828272e68c419b344b537785e23e1eb71ed139366539ebc5 n/s/r=1/1/0 a=SELECT f=136db4ec002a95e14065e127a0e408c985f82726f23eb05e229216b8cc4c5019
20 m=3607477e75c822145e830594f8ee9d55786cf78415d19cd34413f13af651ca9d c=5beb8f659489f9d0ebf56839c78def9a6a22353fd6e4dc7f98811d82001f685b n/s/r=1/0/1 a=REJECT f=6f134f1a07a6af03384f7344abd403e07575ff05fc07a9cbc74c8f8169746e2e
```

The compact index is reproduced directly by the ignored raw-census test. Its
aggregate map kappa is
`blake3:01fb0224d4b30707c50bdd3848d811309f2f907bb0c1acca6650fe3d54debe12`.

### Anti-recall boundary

Construction and validation were disjoint on every operative comparison:

| Comparison | Overlaps |
|---|---:|
| Exact raw history | 0 |
| Exact trailing-four suffix | 0 |
| Exact ordered-route witness | 0 |
| Exact complete candidate representation | 0 |

Accordingly, operative raw prototype recall was false. These checks exclude
the listed exact-recall paths on this fixture; they do not establish a general
non-memorization guarantee outside the frozen envelope.

### Equal-work ledger

Every real/control validation arm used the same complete support and declared
lookup shape:

| Work item | Per six-decision arm |
|---|---:|
| Support rows read | 42 |
| Candidate relation slots | 96 |
| Declared class reads | 24 |
| Performed class reads | 24 |
| Declared payload-inversion slots | 12 |
| Performed payload inversions | 0 |

The two class reads per candidate are actual probes against one typed
`BTreeMap` slot domain: minimum/exact plus rich/typed-no-op. Equal declared
work here means equal serialized support and class-read accounting, not a
claim of identical wall-clock or instruction-level cost.

Source, hosted-provider, teacher, future-route, and validation-label inputs
were all zero during the raw census. The raw record's #953-load field is a
diagnostic assertion, not an access-log proof; the separate changed-path,
import, and surface audit below establishes the delivered quarantine boundary.

## Negative controls

All eleven frozen negative controls had label-free structural coverage **0/6**
and sealed strict ceiling **0/6**. Each sealed control produced six
abstentions and zero tie-or-multiply selections.

| Control | Structural coverage | Strict ceiling |
|---|---:|---:|
| `state_disabled` | 0 / 6 | 0 / 6 |
| `last_only` | 0 / 6 | 0 / 6 |
| `order_shuffled_history` | 0 / 6 | 0 / 6 |
| `causal_return_lease_disabled` | 0 / 6 | 0 / 6 |
| `construction_content_current_pairing_shuffle` | 0 / 6 | 0 / 6 |
| `candidate_prototype_placement_permutation` | 0 / 6 | 0 / 6 |
| `prime_placement_permutation` | 0 / 6 | 0 / 6 |
| `exact_recall_only` | 0 / 6 | 0 / 6 |
| `content_swap` | 0 / 6 | 0 / 6 |
| `construction_key_shuffle` | 0 / 6 | 0 / 6 |
| `incoherent_candidate_relabeling` | 0 / 6 | 0 / 6 |

The real arm therefore was not strictly above any control; all were at the
same zero-transfer boundary.

## Positive metamorphic controls

The negative decision is not caused by an unstable build or a broken
serialization correspondence:

- Complete pre-selector qualification-record candidate relabeling preserved
  that record under one bijection and returned to the original bytes after the
  involution was applied twice. The internal frozen field retains the legacy
  identifier `coherent_full_artifact_candidate_relabeling`; it does not claim a
  native codec/placement rebuild or payload/CID association, which were not
  exercised.
- Full-history incremental prefix construction reproduced the full-build
  support and frozen representations exactly.
- Two independent complete builds produced the same frozen bytes and raw
  census record.
- The sealed label-attached evaluation was replayed and produced byte-identical
  outcome bytes.

These are representation and reproducibility controls. They do not convert the
zero construction-transfer result into attention evidence.

## Sealed strict-ceiling outcome

The sealed label join was loaded only after the raw census identity was
reproduced. The offline lookup then evaluated the no-class-splitting ceiling of
the already-frozen construction map:

| Arm/comparator | Hits | Abstentions | Interpretation |
|---|---:|---:|---|
| Real causal-return class map | 0 / 6 | 6 | no construction-transfer decision |
| Each of 11 negative controls | 0 / 6 | 6 | no construction-transfer decision |
| Count-only last-anchor comparator | 0 / 6 | not a deployed selector | comparison only |

The hard gate required all six decisions to be structurally covered, a real
strict ceiling of 6/6, clean anti-recall, zero padding aliases, real-arm
superiority to every negative control, equal support/work, zero forbidden
inputs, and exact positive metamorphic controls. Purity, anti-recall, work,
input, and metamorphic checks held; construction transfer did not. The binding
terminal is therefore:

```text
UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER
```

### Execution-status boundary

| Operation | Status |
|---|---|
| Label-free structural class lookup | `RUN_OFFLINE_GATE0` |
| Sealed no-class-splitting strict-ceiling lookup | `RUN_OFFLINE_GATE0` |
| Deployed selector | `NOT_RUN_DEPLOYED_SELECTOR_ABSENT_OFFLINE_STRICT_CEILING_LOOKUP_ONLY` |
| Candidate payload inversion | `NOT_RUN_SELECTOR_ONLY_NO_GATE0_ARTIFACT_INVERSION` |
| #953 decoded generation | `NOT_RUN` |

The offline lookups report whether the frozen construction classes could
support a selector. They are not themselves deployed candidate selection,
decoding, or generation.

## Verification ledger

| Check | Status |
|---|---|
| Pre-geometry fixture/codec/artifact/policy/input identity freeze | `PASS` |
| Frozen compiler/query frame and exact H4-table identity freeze | `PASS` |
| Label-free raw Gate 0 census and raw κ reproduction | `PASS` |
| Sealed post-raw outcome and outcome κ reproduction | `PASS` |
| Deterministic complete-build and outcome replay | `PASS` |
| Focused #969 regression | `PASS` |
| Focused changed-path/import/surface proof that #953 remained quarantined | `PASS` |
| Four unignored #983 tests; focused core all-target check | `PASS` |
| #983-target Clippy with unrelated baseline lints isolated | `PASS` |
| Formatting, diff whitespace, and claim wording | `PASS` |
| WASM compatibility check for the new public core surface | `PASS_WITH_PRE_EXISTING_WARNINGS` |
| Dependency-inclusive all-target Clippy with `-D warnings` | `NOT_CLEAN_PRE_EXISTING_UNRELATED_WARNINGS` |
| Workspace/BDD/teacher/κ/corpus/audit/fuzz/product/release suites | `DORMANT_SCOPE_NOT_RUN` |

The #953 quarantine audit found no changed #953 implementation, test, fixture,
or record path and no import/reference from the #983 implementation or harness.
The dependency-inclusive Clippy command reached pre-existing lints in
`uor-r4-graph-runtime`, `uor-r4-model-source`, and unchanged `uor-r4-core`
modules; no unrelated code was changed to make this issue green. The targeted
#983 Clippy pass remained strict after allowing only those enumerated baseline
lint classes. `DORMANT_SCOPE_NOT_RUN` records an intentionally dormant check
and is not evidence about that suite.

## Claim boundary and next decision

`ConstructionCausalReturnV1` retained exact ordered causal-return structure and
produced pure construction classes, but the fixed representation did not
transfer a selectable positive/negative pair to any held-out decision. No
threshold, scalar scorer, or selector should be searched on this zero-transfer
representation.

This result establishes no learned semantics, lexical understanding, deployed
attention, inference, coherent generation, knowledge, correctness, reasoning,
performance advantage, chat quality, or release readiness. #983 remains the
home of the negative evidence and the next representation decision; #953
remains blocked and unexercised by this gate.

## Successor handoff (2026-08-28)

The final sentence above records the ownership state at the time this
append-only evidence closed. Live programme sequencing now preserves #983 as
completed bounded negative evidence and assigns the next representation
decision to fresh, unassigned #986. #986 tests corpus-induced semantic value, a
static self-plus-six harmonic link-state field, and candidate-relative signed
transport on a new CID-disjoint population; it is the native blocker of parked,
untouched #953. No #983 class, population, label, or `NOT_RUN` operation is
reinterpreted by that handoff.
