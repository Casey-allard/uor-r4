# #983 — A1Q-L2: construction-transferred candidate-conditioned geometric attention

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-28T03:46:21Z
- **Updated:** 2026-08-28T07:25:45Z
- **Closed:** 2026-08-28T07:25:45Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/983
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Objective and authority

Qualify exactly one independently frozen, construction-transferred, candidate-conditioned local geometric mechanism before #953 is touched again. This issue is the active child of #820 between closed #969/#970 and parked #953.

The bounded claim under test is: past/current exact ordered H4 state, evaluated independently for each already-admitted candidate, transfers a construction-derived `SELECT`/`REJECT` relation to six held-out natural choices under equal support and work.

This issue does **not** run or modify the known #953 fixture, decode loop, or `run/runs` population. It does not implement #973 higher-scope attention, #954 correctness, #955 reasoning, chat, optimization, formal closure, product QA, or release QA.

## Evidence-bearing predecessors and native order

Direct evidence predecessors:

- #969: retain its exact observed-only prefix products, natural schema-2 admission, equal-work table shape, exact payload inversion, and causal append discipline. Its positive smoke was identity-derived and did not establish lexical semantics.
- #970: retain the ordered H4 state but reject its paired-H4 heatmap readout, which transferred on 0/6 validation cases.

Additional retained evidence:

- #967 established exact associative noncommutative ordered state but not a selecting metric.
- #953 established admission and decode plumbing, then failed its known placement population at 0/2 real versus 2/2 placement-permuted. That population is now an untouched later integration regression.
- #486 requires compiler/query operands to inhabit one exact frame; exact self-match is only a tautological sanity check.

Native target order:

```text
closed #967 / #970 / #969
             ↓
         this A1Q-L2
             ↓
       open parked #953
             ↓
          open #973
             ↓
          #954 → #955 → #962 → #963 → #964 → #965
```

Closed #974 remains dormant design provenance.

## Frozen mechanism: `ConstructionCausalReturnV1`

This section is frozen before branch creation, implementation, or any geometric census.

### Exact algebra and orientation

- Lexical leaf `L(x)` is the existing S0 canonical address-to-H4 assignment.
- The exact 120-root H4 table uses row-major `left * right`, quaternion basis `(1,i,j,k)`, right-handed orientation, and the bound repository root/multiplication-table kappas.
- Prefixes fold left to right:

```text
P_0 = identity
P_i = P_{i-1} * L(x_i),  1 <= i <= t
```

- For every naturally admitted candidate `c` and every earlier prefix `0 <= i < t`:

```text
S_i(H)   = P_i^-1 * P_t
R_i(H,c) = ((S_i * L(c)) * S_i^-1) * L(c)^-1
```

- `i=t` is excluded because it yields a candidate-independent trivial identity.
- Construction history ends immediately before the observed candidate. Later routes in the construction sentence are inaccessible. Validation takes only observed history plus candidates from the existing schema-2 admission path. No API accepts a validation label or actual next route.

### Exact two-level representation

Maximum observed prefix: 8 lexical routes. Maximum retained construction prototypes: exactly 2 per candidate. Every representation has 8 typed slots; unused slots are occupancy-false identity no-ops and may never alias an occupied identity result.

For each populated slot retain exact `S_i` and `R_i` H4 table witnesses, signed H4 coordinates, the existing nine-class round-S3 shell, and past-only age. Exact relation-element equality is the 120-way orientation-sensitive class; the shell is retained explicitly as its declared nine-way coarse observation.

For every populated `i`, define the exact class event:

```text
C_i = (exact R_i signed-H4 coordinate,
       round-S3 angular shell,
       observed_lease_age = t-i,
       total multiplicity of exact R_i in R_0..R_{t-1})
```

`observed_lease_age` is past-only, with `1` the most recent earlier prefix and maximum `8`; it is never literal `next_return_gap`. Exact coordinate equality is authoritative and is never interpreted as a scalar, distance, or semantic order.

`R_min` is the one predeclared representative chosen over all populated `C_i` by: angular shell ascending, multiplicity descending, then observed lease age ascending. Age is unique at the final tie, so exact coordinates are not numerically ordered. This reuses #969's least-shell/lease convention while making the supplied candidate commutator and observed return multiplicity operative.

`R_full` is the complete occupancy-tagged ordered word `C_0..C_7`; every populated entry retains exact class data plus exact `(P_i,S_i,R_i)` witnesses, and every padding entry is typed separately.

Class serialization order is exact signed-H4 coordinate, shell order `Coincident < 36 < 60 < 72 < Orthogonal < 108 < 120 < 144 < Antipodal`, increasing observed lease age, increasing multiplicity, then occupancy. Equality uses the full tuple. Serialization order is distinct from the frozen `R_min` comparison and is not candidate ranking, a scalar score, or semantic distance.

### Construction-only promotion and selector

Each construction transition evaluates both naturally admitted candidates. The observed next candidate gives `SELECT`; the other admitted candidate gives `REJECT`.

- Use `R_min` when every construction row with that exact class has the same `SELECT`/`REJECT` label.
- Only an impure construction `R_min` class promotes to its exact `R_full` subclasses.
- A rich class is usable only if it is construction-covered and pure.
- Unseen, impure, multiply mapped, or malformed classes abstain.
- A validation decision selects only when exactly one admitted candidate resolves `SELECT` and the other resolves `REJECT`.
- Any other combination is a tie-to-abstention. Canonical address, support counts, payload bytes, validation labels, and control outcomes never break a tie.

### Frame and artifact contract

Compiler and query must bind the same canonical codec/vocabulary, schema-2 manifest, route-address mapping, H4 root table, multiplication table, policy, prefix/prototype ceilings, class order, occupancy rules, construction partition, and control definitions. Any mismatch returns:

```text
UNAVAILABLE_FRAME_MISMATCH
```

Versioned identities:

```text
uor-r4.construction-causal-return-fixture/1
uor-r4.construction-causal-return-codec/1
uor-r4.construction-causal-return-artifact/1
uor-r4.construction-causal-return-policy/1
uor-r4.construction-causal-return-validation-input/1
uor-r4.construction-causal-return-raw-census/1
uor-r4.construction-causal-return-construction-label-join/1
uor-r4.construction-causal-return-validation-label-join/1
uor-r4.construction-causal-return-outcome/1
```

Canonical records use the repository canonical JSON serializer and `blake3:<lowercase hex>` identity. Before geometry runs, post the fixture/partition, codec/vocabulary, construction artifact, mechanism policy, and label-free validation-input identities. Post the raw-census identity before loading the sealed validation-label join.

## Independently frozen population

Exact text is lowercase UTF-8, one ASCII space between lexical units, and no punctuation. Each construction candidate occurs immediately before the final completion word. The candidate's predecessor history is the prefix only.

Exactly three families, exactly two construction transitions per candidate, and exactly one matched validation pair per family are permitted.

### Family A — `is / are`, anchor `noon`

Construction:

```text
pilot near painters at station before noon is ready
baker beside sailors by harbor before noon is calm
artists near mechanic at station before noon are ready
nurses beside driver by harbor before noon are calm
```

Sealed validation pair:

```text
A-singular: curator beside dancers by gallery at noon -> is
A-plural:   dancers beside curator by gallery at noon -> are
```

Shared trailing four units: `by gallery at noon`.

### Family B — `has / have`, anchor `already`

Construction:

```text
captain near sailors at harbor today already has departed
doctor beside nurses at clinic today already has arrived
workers near manager at factory today already have gathered
pilots beside mechanic at hangar today already have departed
```

Sealed validation pair:

```text
B-singular: editor beside authors quietly at office already -> has
B-plural:   authors beside editor quietly at office already -> have
```

Shared trailing four units: `quietly at office already`.

### Family C — `was / were`, anchor `yesterday`

Construction:

```text
teacher near students at school early yesterday was absent
driver beside workers at depot early yesterday was late
students near teacher at school early yesterday were absent
workers beside driver at depot early yesterday were late
```

Sealed validation pair:

```text
C-singular: architect beside builders quietly at site yesterday -> was
C-plural:   builders beside architect quietly at site yesterday -> were
```

Shared trailing four units: `quietly at site yesterday`.

For every pair, prompt length, token multiset, trailing-four suffix, candidate union, support ceiling, and work are identical. The controlling validation lexemes (`curator/dancers`, `editor/authors`, `architect/builders`) are absent from their construction family. No validation history or four-unit suffix is a construction predecessor. No `run` or `runs` surface, #953 history, #953 route witness, or #970 population is present.

The existing schema-2 admission path must return exactly:

```text
noon      -> {is, are}
already   -> {has, have}
yesterday -> {was, were}
```

with no truncation or candidate injection. Otherwise the population is unavailable.

## Frozen controls and equal-work contract

Every arm first uses the same natural support, then performs 8 typed prefix slots, 2 candidate evaluations, and 2 prototype/class slots per candidate with the same table-operation shape per decision. Across six decisions this is 42 schema-2 row reads, 96 prefix/relation slots, and 24 performed prototype/class-slot reads, plus identical candidate-entry accounting. The encoder declares 12 payload-inversion slots so a permitted post-Gate-0 selector would preserve shape, but Gate 0 performs zero payload inversions; payload work remains `NOT_RUN` unless the hard gate passes. Occupancy-aware no-ops preserve shape.

Negative controls:

1. `state_disabled`: all history operands are typed inactive identities.
2. `last_only`: only `i=t-1` is operative; earlier slots perform typed no-ops.
3. `order_shuffled_history`: reverse the observed history before the same fold.
4. `causal_return_lease_disabled`: retain the candidate and table work but replace `S_i` conjugation and lease tag with typed neutral identities.
5. `construction_content_current_pairing_shuffle`: canonical cyclic rotation of construction predecessor histories against their observed-next labels within each family.
6. `candidate_prototype_placement_permutation`: canonical swap of the two candidates' complete prototype/class slots within each family.
7. `prime_placement_permutation`: canonical one-step rotation of the population's address-to-H4 leaf placements, applied coherently to construction and query while route/payload identity stays fixed.
8. `exact_recall_only`: usable only on complete raw predecessor-history equality.
9. `content_swap`: swap the two controller lexical routes in each held-out pair while retaining row identity and its sealed label.
10. `construction_key_shuffle`: canonical cyclic rotation of compiled class keys while construction labels remain fixed.
11. `incoherent_candidate_relabeling`: swap candidate labels without coherently moving prototypes, support, payloads, or expected choices.

Positive metamorphic controls:

- `complete_preselector_qualification_record_alpha_equivariance`: apply one coherent bijection to candidate address, support, prototypes/classes, construction labels, and the separately sealed validation-label commitment; the complete pre-selector qualification record must reproduce equivariantly, and applying the involution twice must reproduce the original bytes. Native codec/placement rebuild and payload/CID association are outside Gate 0 and remain `NOT_RUN`.
- full-history and incremental prefix computation must reproduce exactly.
- two complete deterministic builds and outcome replays must be byte-identical.

Report a separate non-geometric `count_only_last_anchor` comparator from construction counts. It may diagnose ordinary signal only. It cannot admit, rank, break ties, populate geometric classes, or be called attention.

## Gate 0 — label-free capacity and transfer preflight

The mechanism and raw census may load construction labels but may not load sealed validation labels. The raw census must report:

- compiler/query same-frame reproduction;
- exact natural support and candidate-union equality;
- complete `R_min` and `R_full` inventories;
- construction purity, promotion, and coverage;
- construction-to-validation class coverage without expected outcomes;
- operative raw-history, suffix, ordered-route-witness, and representation overlap;
- populated/padded alias count;
- exact real/control work counts;
- source/provider/teacher/future-route input counts, all zero.

Freeze and post the raw-census identity. Only then attach construction labels and the separately sealed validation-label join to calculate strict ceilings.

Hard-stop before selector execution unless all are true:

- every construction class used for selection is pure;
- all six validation decisions have both candidate actions covered;
- real strict ceiling is 6/6;
- no operative raw prototype recall exists;
- populated and padded states never alias;
- real ceiling is strictly greater than every causal derangement control;
- all arms preserve identical support and declared work;
- source, hosted-provider, teacher, future-route, and validation-label inputs are all zero during the raw census.

Failure terminals:

```text
UNAVAILABLE_FRAME_MISMATCH
UNAVAILABLE_NO_OPERATIVE_ANTI_RECALL
UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER
REDESIGN_CANDIDATE_CONDITIONED_CAUSAL_RETURN_REPRESENTATION
```

Any failure records the evidence and stops this issue before selector execution. The fixture and representation do not change, and no second mechanism runs.

## One permitted selector run

Only after Gate 0 passes may the frozen `SELECT`/`REJECT` selector run once.

Success requires 6/6 held-out decisions; no real-arm tie or abstention; every causal negative control strictly below real; coherent relabeling equivariance; exact incremental reproduction; byte-identical rebuild/outcome replay; exact payload inversion; source/provider absence; and no candidate injection or semantic tiebreak.

A positive terminal is only:

```text
PROCEED_TO_I1_WITH_CONSTRUCTION_TRANSFERRED_LOCAL_GEOMETRIC_ATTENTION
```

It does not establish broad lexical semantics, grammar, decoded generation, correctness, knowledge, reasoning, paragraph/conversation/global attention, performance, chat quality, production readiness, or release readiness.

## Run contract and outcome actions

```text
metric to move:       strict construction-transferred held-out choices, current 0/6 qualified
reachability ceiling: 6 decisions x exactly 2 naturally admitted candidates; maximum 6/6
instrument:           label-free Gate 0 census; it must authorize a 6/6 strict real ceiling and beat every derangement
exit rule:            selector success only at 6/6 with all controls/replay/equality conditions
if positive:          protected close; later session applies this algorithm unchanged to quarantined #953
if negative:          protected evidence with References; keep this issue open and #953 blocked; maintainer chooses semantic-placement or table-native-value direction
cost estimate:        bounded local compile/census plus at most one six-decision selector/replay; no corpus/model/teacher or broad QA run
```

If ordered causal-return classes transfer but real placement loses to derangements, the next maintainer decision is a fresh-population corpus-induced semantic-placement test. If the count-only comparator is positive while geometry does not transfer, retain geometry as addressing/transport and evaluate the table-native predictor as the semantic value engine. Do not add H4/Hopf/Poincare/harmonic/SpiralCore plumbing in this issue.

## Observed outcome — 2026-08-28

The label-free raw census formed 21 `R_min` and 24 `R_full` classes from 24 construction rows. Every usable class was pure, but the real arm structurally covered 0/6 held-out decisions. After the separately sealed validation-label join was attached, the offline no-class-splitting lookup also reached 0/6 with six abstentions. All eleven negative controls and the count-only comparator were 0/6.

- raw census: `blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017`
- sealed validation-label join: `blake3:5aa4b1c5880f660bc463a0650ebf21603d63170252b56e03e9a93e0a07e28c76`
- sealed outcome: `blake3:58fba09dba1b9245cb62a73bf8e3ac153242dc0730e3df7586446aa2820d4587`
- terminal: `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`
- deployed selector: `NOT_RUN_DEPLOYED_SELECTOR_ABSENT_OFFLINE_STRICT_CEILING_LOOKUP_ONLY`
- payload inversion: `NOT_RUN_SELECTOR_ONLY_NO_GATE0_ARTIFACT_INVERSION`
- #953 decoded generation: `NOT_RUN`

The offline lookup measures the frozen construction map's selector capacity; it is not a deployed attention selector. This negative is scoped to the frozen representation, placement, construction population, and validation envelope. It does not rule out H4 state, paired-H4/E8 structure, exact trigonometric channels, ordered prime/semiprime channels, corpus-induced placement, or a different candidate-relative mechanism. The next maintainer decision remains corpus-induced semantic placement versus a table-native semantic value engine; no second representation runs in this issue.

## Verification and delivery

Activated checks only:

- exact Gate 0 census;
- selector witness only if Gate 0 passes;
- deterministic rebuild/replay;
- focused #969 regression;
- focused quarantine checks proving #953 was not run or modified;
- focused tests and clippy for changed packages;
- formatting, diff whitespace, and claim-wording checks;
- targeted WASM only if the changed core API reaches the exported WASM boundary.

Workspace, BDD, kappa reproduction, teacher/model, corpus-scale, audit, fuzz, conformance, product, performance, formal, and release suites remain `NOT_RUN` unless a changed path makes one decision-bearing.

Positive protected delivery uses `Closes #<this issue>`. Negative/unavailable/partial delivery uses `References #<this issue>` and leaves the issue open. Stop after this one issue outcome; do not begin #953.


## Final pre-implementation relation-class audit

A read-only enumeration of the bound 120-root table was completed before branch creation or implementation. Its nine group-conjugacy classes coincide exactly with the nine existing real-coordinate angular shells, with sizes `1,20,12,12,30,12,12,20,1`. A conjugacy-class key would therefore duplicate the shell and erase orientation-sensitive information. The frozen representation consequently uses exact `R_i` element equality by canonical signed H4 coordinate, while retaining the shell only as the explicit coarse observation. No outcome or validation geometry informed this decision. The fixed `R_min` comparison is the minimum over all populated earlier-prefix events by shell, return multiplicity, and past lease age; this completes the pre-implementation freeze.

## Comments (7)

### comment 5448325027 — 2026-08-28T04:15:03Z — Casey-allard

_edited 2026-08-28T04:48:40Z_

## Pre-geometry immutable checkpoint

This checkpoint was reproduced after the population, codec, construction-only transition set, sealed validation-label commitment, policy, budgets, controls, and stop rules were frozen, and before any ConstructionCausalReturnV1 relation geometry was computed.

Content-addressed identities:

- human-readable fixture and partition: blake3:c3f917a91b036555ff276bfdfb6ab198732999c9f082c1058813d14ffd243d69
- codec: blake3:b1e8baf2ad3e6b9eb58f8d8c06809e76f37bdf91bfcce462a4e868e9952d654a
- vocabulary: blake3:bd8c3c458f0c92fbbf9003b722417335678a1adfdfd7a7c0c68580dc129ffb75
- codec/vocabulary record: blake3:0c45cd205ab0b14197ae178a668dbbbc893fc5c4e7beae97aeaf8a0be4b8be61
- construction route artifact: blake3:45c2475983fb047a9cedda6ef28edee832326cf730faeb5b590b7afc305505db
- existing schema-2 attention manifest: blake3:3e47dc0475c8f9da017ec1df456485d0c4957fe2203022dce7dcb537576b659a
- construction artifact record: blake3:c014cc92eef32fdf8804adf6efba90c26c991f1d8ebed57700ffeaa2e0b1f7d0
- ConstructionCausalReturnV1 policy: blake3:0ab5118269a6aacbb4293ad876edcc82bf8f4ecca8b2121409b2bd8e0ff887c0
- label-free validation input: blake3:18b8568ea11255ca601a7453ab28bbb9d4bfb26c479e37c814ff08643e0561b4
- sealed validation-label join: blake3:8e5fbeb4d78ba763bc4fdd3ca1522587b417f4d52e87ac9d4b8f58dfe6889f3b
- exact 12-transition route-address partition: blake3:e9eaaa021752335149cd6cbd5fd17faac9e31f61db6741532292e6c6d1ef58ba
- compiler/query frame: blake3:fe0fa1d5f56e97ba6508bf290c644b059caf4d23a96296e078b78dc522767c1a
- H4 root table: blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76
- H4 multiplication table: blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759

Population/boundary witness:

- exactly 3 candidate-pair families, 12 construction transitions, and 6 sealed matched validation decisions;
- 23 rebuild witnesses total: 12 transition-bearing construction sentences, 10 singleton validation-address registration witnesses, and 1 singleton global registration witness;
- only the 12 declared construction sentences carry transitions or may become prototypes; all 11 registration witnesses are zero-transition;
- every validation prompt reaches exactly its two declared natural candidates through the unchanged schema-2 admission path;
- matched partners have identical candidate union, rows read, available/examined/admitted entries, prefix width, and declared support work;
- the fixture binds the sealed expected continuations by commitment, but the mechanism and label-free support/raw interfaces have no validation-label field and do not load that record;
- the only public frame constructor requires the actual codec, route artifact, schema-2 attention artifact, exact H4 tables, and compiled construction partition to reproduce this same frame;
- actual future routes, teacher/provider/source data, and issue #953 are absent;
- the #983 focused freeze/support test is 2/2; no causal-return relation census or selector has run.

Any subsequent identity drift is a frame mismatch, not permission to re-freeze after observing validation geometry.

### comment 5448859603 — 2026-08-28T05:34:51Z — Casey-allard

## Corrective pre-geometry checkpoint — opaque validation keys and exact Gate 0 work shape

This supersedes only the four identities below from the earlier pre-geometry checkpoint. Before any `ConstructionCausalReturnV1` relation geometry or raw census ran, the six label-free validation decision IDs were changed from family/ordinal-bearing names to fixed opaque `V-<hex>` keys. The separately sealed label join uses those same opaque keys. The first identity reproduction failed against the old pins as intended; the four changed identities were reviewed and pinned, and the exact pre-geometry reproduction then passed.

Changed identities:

- human-readable fixture and partition: `blake3:289cab4b5e22d45a61324137f2cc229c473570e6b9d3358dec16652dbfc84f83`
- construction artifact record: `blake3:c4bca03f4c06d2ce58ed7167842b72491f4c9275c25fa9599f7bf30f54b1a670`
- label-free validation input: `blake3:17246f23c14a81ea83d388e5592af634027d07625ad7bdb1fdf743c1b562712a`
- sealed validation-label join: `blake3:5aa4b1c5880f660bc463a0650ebf21603d63170252b56e03e9a93e0a07e28c76`

All other previously posted identities reproduced unchanged, including codec, vocabulary, codec/vocabulary record, construction route artifact, schema-2 attention manifest, policy, construction partition, compiler/query frame, H4 root table, and H4 multiplication table.

The equal-work contract is also now explicit before the raw run:

- every arm performs two probes through the same unified typed `BTreeMap` class-slot shape, bound as `uor-r4.gate0-two-unified-typed-btree-class-slot-probes/1`;
- support, raw frame provenance, work, and class-probe equality are derived from serialized per-arm records and are hard-gated;
- Gate 0 declares 12 payload-inversion slots but performs zero payload inversions because no selector is authorized yet;
- native payload/CID association and exact inversion remain `NOT_EXERCISED` / `NOT_RUN` and cannot be claimed unless Gate 0 authorizes the one permitted selector run;
- the positive coherent alpha control covers the complete pre-selector qualification artifact: partition, support, raw candidate representations and work, construction actions, validation queries, registered address-to-surface commitments, compiled class map, and structural coverage. It makes no synthetic payload/CID claim.

No #953 fixture, generation, validation label, provider, teacher, source tensor, actual future route, selector, or causal-return raw geometry has run at this checkpoint.


### comment 5448953563 — 2026-08-28T05:49:05Z — Casey-allard

## Frozen raw selection-blind Gate 0 census — labels still sealed

The unchanged `ConstructionCausalReturnV1` raw census completed after one audit-only canonical-order repair to the coherent candidate-relabeling witness. The failed attempt produced no census identity and never loaded validation labels. The successful run used the frozen population, representation, policy, frame, and controls.

Raw census identity:

```text
blake3:2d2b1164fe6967260205fcf7f017f60cb6d11f108007640aedae9a7ba788aa2d
```

Selection-blind result:

- sealed validation-label loads observed: `0`;
- real construction classes used for selection are pure: `true`;
- real construction-to-validation structural coverage: `0/6` decisions;
- promoted construction rows: `4/24`;
- operative raw-prototype recall: `false`;
- populated/padded aliases: `0`;
- all 11 required negative controls: `0/6` structural coverage;
- all arms complete, natural support-equal, declared-work-equal, raw-frame exact, and equal under the same two-probe unified typed class-map operation shape;
- incremental/full-history reproduction and two complete deterministic raw builds are exact inside the frozen record;
- source, provider, teacher, actual-future-route, validation-label, and #953 inputs: `0`;
- raw canonical JSON size: `4,472,618` bytes.

The full record contains the exact `R_min` and `R_full` construction inventories, per-candidate structural coverage, support rows, work ledgers, anti-recall witnesses, class-map kappas, and real/control reports.

Because real structural transfer is `0/6`, Gate 0 cannot authorize a selector regardless of the later label join. The sealed join will now be attached exactly once only to freeze the strict ceilings and terminal. No selector or #953 generation will run.


### comment 5449058908 — 2026-08-28T06:04:45Z — Casey-allard

## Superseding raw Gate 0 identity — audit-only alpha-order correction, labels still sealed

This supersedes only the raw-census identity in the preceding checkpoint. The coherent full-artifact relabeling audit had renamed and canonically re-sorted rows, but then compared them positionally against the pre-renaming order. That made a bijective candidate-name swap appear non-equivariant solely because sort order changed.

The audit now constructs the fully relabeled expected rows, sorts both sides canonically, and compares them exactly. It does not change the frozen fixture, partition, codec, vocabulary, construction artifact, geometry, class representation, placement, support, work budget, labels, or selector boundary.

The corrected selection-blind builder failed closed on the stale pin before the sealed validation-label join was loaded and exposed this replacement raw identity:

    blake3:0a894f16af65d5001c451744d56119d176e522a32c510b594e52cf148176ea45

Label-load count at that point remained 0. The measured geometric result is otherwise unchanged: real construction-to-validation structural coverage 0/6, all eleven negative controls 0/6, pure usable construction classes, promoted rows 4/24, operative raw-prototype recall false, populated/padded aliases 0, and equal support/work/typed map-operation shape.

The earlier raw ID is retained as historical audit evidence but is not the binding #983 census. This corrected identity is now frozen before the sealed outcome is attached.

### comment 5449278989 — 2026-08-28T06:35:08Z — Casey-allard

## Binding raw selection-blind Gate 0 census — final audit form, labels sealed

This is the binding #983 raw checkpoint. It supersedes the two earlier raw IDs, which remain historical evidence of the canonical-order audit repairs. The frozen mechanism, population, geometry, support, and controls never changed.

    raw census: blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017
    construction label join: blake3:f6608fb43212910731dc602d776ac7e955d41cbf12f8db62e46b926214c52720
    real class map: blake3:01fb0224d4b30707c50bdd3848d811309f2f907bb0c1acca6650fe3d54debe12

Label-load count remained 0 throughout this exact raw-only run.

Binding selection-blind evidence:

- complete construction inventory: 21 exact R_min classes and 24 exact R_full classes;
- 2 impure R_min classes promote to pure R_full subclasses, covering 4/24 construction rows;
- every usable construction selection class is pure;
- real construction-to-validation structural coverage: 0/6 decisions;
- all eleven required negative controls: 0/6 structural coverage;
- anti-recall: 0 raw histories, 0 trailing-four suffixes, 0 ordered-route witnesses, and 0 complete operative candidate representations overlap between 12 construction and 6 validation histories;
- populated/padded aliases: 0;
- exact validation work: 42 natural schema-2 rows, 96 relation slots, and 24/24 declared/performed unified typed class-map reads;
- 12 payload-inversion slots are declared, but 0 are performed because the deployed selector is not authorized or compiled;
- complete pre-selector qualification-record alpha-renaming, full/incremental reproduction, and two complete deterministic raw builds: exact;
- source/provider/teacher/actual-future/validation-label inputs: 0;
- canonical raw JSON: 4,472,678 bytes.

The class inventory is emitted in full by the exact raw test and content-addresses every R_min record and every R_full word. The deployed selector remains NOT_COMPILED / NOT_RUN; the only lookups exercised here are the offline label-free structural-coverage evaluator.

Because the real structural transfer is 0/6, the hard gate cannot pass. The sealed validation-label join may now be attached once to freeze strict ceilings, replay, and the required UNAVAILABLE terminal. It cannot authorize selector execution, payload inversion, a second representation, or any #953 work.

### comment 5449490232 — 2026-08-28T07:02:31Z — Casey-allard

## Sealed Gate 0 outcome - binding negative

The raw census was reproduced before attaching the separately sealed validation-label join.

- commit: `cff0690b`
- PR: #984 (`References #983`)
- raw census: `blake3:5e970efe79c13d38e02eab6ff60642d3d449ce9dc571af6425b16d0d94858017`
- sealed label join: `blake3:5aa4b1c5880f660bc463a0650ebf21603d63170252b56e03e9a93e0a07e28c76`
- sealed outcome: `blake3:58fba09dba1b9245cb62a73bf8e3ac153242dc0730e3df7586446aa2820d4587`
- construction map: 21 `R_min`, 24 `R_full`, 2 promoted minimum classes, 4/24 promoted rows
- usable construction classes pure: yes
- real structural coverage: 0/6
- real sealed strict ceiling: 0/6, six abstentions
- all eleven negative controls: 0/6
- count-only comparator: 0/6
- raw-history / suffix-4 / ordered-route / complete-representation overlap: 0 / 0 / 0 / 0
- populated/padding aliases: 0
- work per arm: 42 support rows, 96 relation slots, 24 declared and performed class reads, 12 declared and 0 performed payload inversions
- complete pre-selector alpha-equivariance, incremental reproduction, independent rebuild, and outcome replay: exact
- source/provider/teacher/future/validation-label inputs during raw census: 0
- terminal: `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`
- deployed selector: `NOT_RUN_DEPLOYED_SELECTOR_ABSENT_OFFLINE_STRICT_CEILING_LOOKUP_ONLY`
- payload inversion: `NOT_RUN_SELECTOR_ONLY_NO_GATE0_ARTIFACT_INVERSION`
- #953 decoded generation/replay: `NOT_RUN`

The offline strict-ceiling lookup measures the frozen construction map's capacity; it is not a deployed selector. The full compact 21-class/24-rich-class inventory and all bound identities are reviewable in the [immutable evidence record](https://github.com/UOR-Foundation/uor-r4/blob/cff0690b/docs/construction_causal_return_attention_983.md).

Focused verification passed: both ignored binding runs, four unignored #983 tests, the #969 decoded intervention regression, core all-target check, #983-target Clippy after isolating enumerated unrelated baseline lints, formatting, diff/claim wording, #953 changed-path/import quarantine, and WASM compatibility. Dependency-inclusive all-target Clippy remains non-clean on pre-existing warnings in unchanged files. Broad workspace/BDD/teacher/kappa/corpus/audit/fuzz/conformance/product/performance/formal/release suites remain `NOT_RUN`.

Decision: record the negative through protected delivery, leave #983 open as the evidence and next-representation home, keep #953 parked/unassigned/untouched/blocked, and do not run a second representation in this issue. The next maintainer choice is a fresh-population corpus-induced semantic-placement experiment versus retaining exact geometry as addressing/transport around a table-native semantic value engine.

### comment 5449685947 — 2026-08-28T07:25:44Z — Casey-allard

## Successor handoff and closure

The bounded #983 contract is complete at `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`; its 0/6 raw structural coverage, 0/6 sealed strict ceiling, frozen identities, controls, and `NOT_RUN` selector/generation boundaries remain immutable negative evidence.

The maintainer decision requested after that terminal is now explicit: fresh native child #986, `A1Q-L3: qualify corpus-induced signed transport attention`, owns exactly one new contract. It first tests construction-only corpus-induced semantic placement on a CID-disjoint induction/calibration/test population, then asks whether candidate-relative signed zero-sum Cl(0,6)/SpiralCore transport adds causal value beyond a matched table-native semantic-value comparator.

Native dependencies were updated before this closure: #986 is blocked by #983 and now blocks parked #953; #983 no longer directly blocks #953. #953 remains unassigned and untouched, with its fixture, selector, generation, and replay `NOT_RUN` under this handoff.

This closure does not reinterpret #983 or authorize another representation under its failed contract. It records that the next representation decision has moved to a new issue with a fresh population and frozen decision branches.



---
_Comment/review ids in this file: 5448325027, 5448859603, 5448953563, 5449058908, 5449278989, 5449490232, 5449685947_

