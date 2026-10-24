# Serving-time address-as-read oracle over `uor-r4-object-store` (DRAFT)

- **Status:** DRAFT pre-registration sheet for `#2833` (`m1-address-as-read`, Step 4). **FIT-reviewed by deepseek (non-author) on 2026-10-22: `NOT_FIT`** (see [Verdict](#verdict)); **no GPU run is cleared or started** until the remaining prerequisites in the Verdict are pinned (prerequisite 4 is pinned below) and Casey releases the run. **Sheet update 2026-10-22 (deepseek-33):** prerequisite 4 (consumer commit pin) is now pinned below; prerequisites 1–3 and part of 5 remain `PENDING (PM/owner)` and are listed in one table in the Verdict. **Sheet update 2026-10-23 (deepseek-38):** the bar and panel decision cards were filed, and the owner's standing direction registered both as recommended at **2026-10-23T02:44:51Z** (`#2028` comment `6356136269`; see [Registration](#registration-deepseek-38-2026-10-23)). The checkpoint pin stays `PENDING` (held `#2844`) and `NOT_FIT` stands with no compute cleared; the next FIT must come from a lab other than deepseek (deepseek has authored text in this sheet). **Sheet update 2026-10-24 (kimi-33):** the 80-row panel is now frozen and read back identically from `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v1.jsonl`, SHA-256 `0712371d20e6eb1294c8faebeea7ab9827e2b91c0decfb5d85a52b8cca286789`. The held checkpoint and the BPB stream's full digest remain `PENDING`; `NOT_FIT` still stands with no compute cleared. **Sheet update 2026-10-24 (claude-71):**
- `panel-v1` is superseded as an unfit instrument. Each of its stated and historical prompts held its expected value, and each abstention prompt named its family, so a no-read arm could pass the bar (#3131 `6380260818`).
- The pinned panel is now `panel-v2`, SHA-256 `369d8c24bb41e2d071485ecd1cbc7044092959ed5309ad70a8a5150a69a9e90c` (see [Panel repair](#panel-repair-claude-71-2026-10-24)).
- antigravity, deepseek, kimi and claude have now each authored text in this sheet or a panel. The next FIT therefore comes from a fresh-context reviewer, which CHARTER §4.5 counts as non-author; the PM dispatches which lab runs it.
- `NOT_FIT` stands, and no compute is cleared. **Sheet update 2026-10-24 (kimi-34):** prerequisite 5's sealed fresh-V2 token, response-mask and seal-report digests are now quoted below; the held checkpoint remains `PENDING`, with no bar, tolerance, panel, FIT or hold change.
  - CPU consumer module (`uor_r4_object_store::address_read`) and finite CPU controls were delivered in `#2950` (`$0` GPU spend).
  - GPU evaluation of the serving-time oracle awaits a non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and Casey's approval in chat; any trained form also requires a separate FIT review and a typed owner release (CHARTER §5.1).
- **Line:** M1/M3 exact addressed memory (`#2029`, `#2031`, item `#2833`). **Lab:** antigravity.
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25/D30, D31, D32, D40.
  - CHARTER §3.1–§3.3 (D23, D25/D30): Geometry only; zero softmax or squashing; zero dense matrix multiplication.
  - CHARTER §3.5 (D24): Model evaluation runs on the GPU (RTX 5090).
  - CHARTER §4.5: Frozen, SHA-256-pinned panel and non-author `TEST FITNESS: FIT` before GPU evaluation.

## 1. Question and mechanism tested

[`docs/mechanisms/exact-memory.md`](../../mechanisms/exact-memory.md) ("What would reopen it", Step 2) asks to **supply the true slot address at serving time and measure the gain before any training**, separating the address-as-read consumer over `uor-r4-object-store` (`AddressDirectory` + `Store`) from learned query-entity/relation tagging (which caused 77.7% of held-out failures in the G v1 decomposition, [`archive/integration/g1-decomposition-result-2026-09-29.md`](../../../archive/integration/g1-decomposition-result-2026-09-29.md)).

## 2. Arms (zero training)

Evaluated on one frozen checkpoint on RTX 5090 across three deterministic serving-time read modes over the same `AddressDirectory` populated from the dialogue context's stated facts:

1. **Arm A (`no_address_read`, baseline anchor):** Frozen model evaluated with the object-store address read disabled (`AddressReadOutcome::Absent`).
2. **Arm B (`sieve_address_read`, deterministic prime-sieve query):** Query address formed deterministically from the prompt's content-word primes and ordered pairs (`AddressDirectory::read_sieve`, sorted-merge $\gcd(q, A_c) > 1$ admission and integer tuple ranking `(shared_primes, ordered_pair_matches, sequence)`).
3. **Arm C (`oracle_address_read`, true slot address supplied):** True `(entity_prime, relation_prime)` `PrimeAddress` supplied to `AddressDirectory::read_exact(..., VersionSelect::Current)` (or `VersionSelect::Previous` on historical-version rows), resolving the verified `ObjectTarget` from `uor-r4-object-store::Store`.

## 3. Bar

- **Status of this bar: `PROPOSED — PENDING owner registration` (CHARTER §5.2).** The owner must quote the registered bar with an **ISO-UTC time** and settle the one baseline (`19/40` vs `20/40`) before `+8` has a single meaning; `>= 54/80` across the full 80-row panel also needs its derivation from the 40/20/20 composition and that baseline (see the Verdict table). No compute is cleared by this proposal.
- **Oracle gain bar (Arm C):** At least `+8/40` more recalled fact rows than the `19/40` (`20/40` anchor) baseline (`>= 27/40` on the memory recall half, and `>= 54/80` across the full 80-row panel), with `0` wrong-value substitutions from `uor-r4-object-store` (`whole_object_verified` / range-verified byte match `1.000` on every `Hit`) and held-out BPB within `0.01` of Arm A.
- **Trained follow-up gate:** Only if Arm C meets the oracle gain bar is a trained address-as-read sheet eligible to be drafted (2–3 seeds, frozen `>= 80`-row panel, separate FIT review, and typed owner release).

## 4. Base artifact

- **Consumer library and controls (PINNED):** commit `86eb68ebb` (PR `#2950`) delivered `crates/uor-r4-object-store/src/address_read.rs` (`PrimeAddress`, `AddressDirectory`, `ObjectTarget`, `AddressReadOutcome`, `VersionSelect`) and the **6** finite controls in `crates/uor-r4-object-store/tests/address_read.rs` (exact hit; exact miss with zero backend fetches; wrong-address/tampered/binding refusal; versioned overwrite `Current`/`Previous` and bounded-capacity `Evicted`; sorted-merge GCD sieve admission/ranking; byte-identical round trip). Re-check before any run: `cargo test -p uor-r4-object-store --test address_read` = 6 passed at that commit.
- **Candidate model checkpoint (PENDING — owner):** the base artifact is the frozen M1 discrete trunk checkpoint (`m1-trunk-assembly-and-training` `#2844`), which is **HELD** on `main`; the exact checkpoint SHA-256 and where it is stored must be pinned by the owner before FIT can clear a run. This sheet does not invent a checkpoint.

## 5. Data split

- **Frozen panel (PINNED, `panel-v2`; `panel-v1` superseded):**
  - **File:** `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v2.jsonl` (HF commit `bb81a8a33916bbc95ac192dce5b75c9ef5df1889`), **80 rows / 139,678 bytes**, SHA-256 **`369d8c24bb41e2d071485ecd1cbc7044092959ed5309ad70a8a5150a69a9e90c`**.
  - **Generator:** `crates/uor-r4-object-store/src/bin/address-panel-freeze.rs`. Its tests replay every row's writes through `AddressDirectory` and `Store` from the emitted JSON. The remote readback is byte-identical.
  - **panel-v1:** (`0712371d20e6eb1294c8faebeea7ab9827e2b91c0decfb5d85a52b8cca286789`, 21,364 bytes) stays in the store untouched and regenerates from commit `1a70a9ac3`. It is not used, because its prompts held their answers.
  - **Composition:** unchanged. It is `40` stated-fact / `20` historical-version / `20` abstention (`10` unwritten + `10` evicted), with unique prompts and addresses. The registered 80-row oracle is distinct from the M3 decisive-run panel's `>= 200` rows (D22 §3); it is a bounded instrument panel, not a claim of decisive language power.
- **Held-out BPB stream (`PINNED`, sealed fresh V2):** the Arm A baseline and all compared arms use the same sealed fresh-V2 draw. Its artifacts are `tokens.u16`, SHA-256 `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, and `response_mask.u8`, SHA-256 `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`; the seal report is SHA-256 `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`. The [retained A8 report](../../labs/a8-dose-2026-10-19/evidence/frozen-zero-report.json) binds the token/mask values to the sealed draw (`44,678` payload tokens and `14,479` eligible response targets), and the [dispatch record](../../labs/attention-dose1-dispatch-2026-10-18/README.md#fresh-v2-sealed-input-metadata) plus [packet table](../../labs/attention-dose1-packet-2026-10-18/README.md) independently quote all three values. These are retained metadata pins, not a fresh draw, raw-content disclosure or model result.
- **BPB tolerance (`REGISTERED`, unchanged):** `0.01`, registered at `2026-10-23T02:44:51Z` (`#2028` comment `6356136269`).
- **Reported metrics (separated per `exact-memory.md` Step 5):**
  1. **Oracle read accuracy (Arm C vs Arm A):** exact-match `check_pass` on the recalled fact rows when the true `PrimeAddress` is supplied.
  2. **Deterministic sieve address-formation accuracy (Arm B vs Arm C):** fraction of rows where `read_sieve` selects the same `PrimeAddress` and `ObjectVersion` as the oracle (`address_match`), reported separately from downstream token emission (`check_pass`).
  3. **Honest abstention / eviction accuracy:** exact `Absent` / `Evicted` classification on unwritten and capacity-evicted rows.
  4. **Held-out BPB difference:** BPB of Arm B and Arm C vs Arm A on the held-out text stream.

## 6. Steps

1. Freeze `docs/harness/panels/m3-memory.json` (`>= 80` rows) and pin its SHA-256 digest.
2. Obtain a non-author `TEST FITNESS: FIT` review on this sheet, owner bar confirmation on `#2028`, PM dispatch, and Casey's approval in chat before any GPU evaluation.
3. Run the three evaluation arms (`Arm A`, `Arm B`, `Arm C`) on the frozen checkpoint on a single RTX 5090 pod via `scripts/pod/uor-pod` (zero training steps).
4. Record `check_pass`, `address_match`, abstention/eviction accuracy, and held-out BPB in [`docs/research/notebook/m3-durable-memory.md`](../../research/notebook/m3-durable-memory.md).

## 7. Stop conditions

- Any finite CPU control failure in `cargo test -p uor-r4-object-store --test address_read` blocks GPU evaluation; zero retries with modified code or panel after FIT.
- Any wrong-value byte verification failure (`whole_object_verified == false`) is an immediate integrity stop (D31).
- **Bar-miss stop (registered rule):** if Arm C misses the owner-registered oracle-gain bar, the run is recorded as a **negative for this exact configuration** (frozen panel, checkpoint, arms and serving code) and closes that configuration only — never the address-as-read mechanism (D22 §1); no re-run with a modified panel, bar or checkpoint after FIT.
- **Instrument failures are not negatives:** an integrity, panel-hash or checkpoint-identity failure stops the run as an instrument blocker and is reported as such (D22/D31).

## 8. Cost and minimum test power

- **Cost:**
  - `preparation`: `$0.00` (CPU consumer module and finite controls in `#2950`).
  - `run`: single RTX 5090 evaluation-only run (zero training steps), `< 15` minutes (`< $0.25`), within the `<= 4` pods / `<= $8/h` fleet cap.
  - `controls`: Arm A (`no_address_read`) and Arm B (`sieve_address_read`) evaluated in the same pass.
  - `evaluation`: deterministic exact-match and byte-hash verification (`$0.00`).
  - `retries`: `0`.
  - `storage`: `< 1 MiB` evaluation report.
- **Minimum test power (D22 §3):**
  - Frozen panel of `>= 80` rows (`40` stated-fact rows, `20` historical-version rows, `20` unwritten / evicted abstention rows) evaluated across all 3 arms (`Arm A`, `Arm B`, `Arm C`).

## Verdict


### Current pin status (sheet update, deepseek-33, 2026-10-22)

| # | Prerequisite | Status | Evidence / slot |
|---|---|---|---|
| 1 | Frozen, hashed 80-row panel and 80-vs-200 reconciliation | **FROZEN (claude-71, 2026-10-24: `panel-v2`; kimi-33's `panel-v1` superseded)** | `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v2.jsonl`, 80 rows / 139,678 B, SHA-256 `369d8c24bb41e2d071485ecd1cbc7044092959ed5309ad70a8a5150a69a9e90c`; the registered 80-row oracle remains distinct from the M3 decisive-run panel's `>= 200` rows |
| 2 | Candidate checkpoint SHA-256 | **PENDING — owner** | `m1-trunk-assembly-and-training` `#2844` is HELD; no SHA is invented |
| 3 | Owner-registered bar with ISO-UTC time and one baseline | **REGISTERED (2026-10-23T02:44:51Z)** | registered as recommended (`#2028` comment `6356136269`): `27/40` vs the single `19/40` baseline, `7/20` historical, `20/20` abstention, `0` wrong values, BPB within `0.01` on fresh V2; derivation `27+7+20 = 54/80` |
| 4 | Consumer commit pin | **PINNED** | commit `86eb68ebb`; 6 finite controls in `crates/uor-r4-object-store/tests/address_read.rs` |
| 5 | Measurement definitions | **PINNED (kimi-34, 2026-10-24)** | bar-miss and negative-closes rules pinned in §7; sealed fresh-V2 `tokens.u16` SHA-256 `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8`, `response_mask.u8` SHA-256 `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e`, seal report SHA-256 `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`; `0.01` tolerance unchanged (registered 2026-10-23T02:44:51Z) |

Re-request FIT once the panel hash is frozen and the checkpoint SHA is pinned; this table is deliberately a checklist, not a decision: **no owner value, bar, checkpoint or panel is set by the lab here.**

### PM panel freeze (kimi-33, 2026-10-24)

The PM-selected source is deterministic synthetic versioned facts, not corpus text. `address-panel-freeze` emits 80 canonical JSONL rows with unique prompts and prime addresses: 40 current stated-fact hits, 20 previous-version hits, 10 unwritten `Absent` rows and 10 capacity-1 `Evicted` rows at sequence 2. Each expected result is checked against `AddressDirectory` and `Store` during generation. Two runs produce identical bytes; the uploaded file is `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v1.jsonl` (`59f964aa60b83b4684abaa34daddd0c6db768d4d`), 21,364 bytes, SHA-256 `0712371d20e6eb1294c8faebeea7ab9827e2b91c0decfb5d85a52b8cca286789`, and remote readback is identical. This resolves only the panel-hash prerequisite; it does not change the bar, checkpoint, FIT verdict or any hold.

### Panel repair (claude-71, 2026-10-24)

`panel-v1` was an unfit instrument. Class: instrument, not a result. Each of its stated and historical rows was one prompt containing its own expected value, and each abstention prompt named its family ("Recall unwritten…", "Recall evicted…"), so a no-read model could score 40/40, 20/20 and 20/20 by copying (#3131 `6380260818`). Its Arm B primes were also constants unrelated to the prompt words.

**How `panel-v2` rows work.** Each row is one dialogue:
- **`writes`.** The row's stated facts in sequence order. Each write carries:
  - its `sequence` and slot `version`;
  - its lexicon `terms`, with their primes and the ordered `(entity, slot)` pair;
  - its `statement` text;
  - its stored `value`.

  Values are 8 letters from SHA-256, so no value is derivable from a query's words.
- **`capacity`.** The directory capacity.
- **`query`.** "What is `<entity>` `<slot>`?", or for historical rows "What was `<entity>` `<slot>` before it changed?". Unwritten and evicted rows use the stated template, so a query's wording never says whether the right answer is to abstain.

**Composition and design** (40 / 20 / 10 / 10):
- **Stated rows** (40): the dialogue states the entity's four facts and a neighbour entity's four.
- **Historical rows** (20): the dialogue overwrites a status slot between distractors.
- **Unwritten rows** (10): the dialogue states three facts, and the query asks the fourth.
- **Evicted rows** (10): capacity 4, and the fifth write evicts the target at sequence 5.

**Evaluator protocol.**
- Render the `writes[].statement` lines in order as the dialogue context, then the `query`.
- Never show the model the scorer-only fields: `family`, `oracle_*`, `outcome`, `expected_bytes` and `evicted_at_sequence`.
- **Arm C** reads `oracle_primes` and `oracle_ordered_pairs` with `version_select`.
- **Arm B** derives its address from the query alone: split it on ASCII whitespace, trim trailing `?`, `.` and `,`, keep the lexicon words in order, take their primes, and pair each kept word with the next.

**Tests.** The bin tests parse the emitted JSON and pin:
- no query contains any value of its row, or a family word;
- every query matches its template;
- replaying each row's writes reproduces every oracle outcome;
- Arm B's rule derives exactly the oracle address on all 80 rows;
- values are distinct;
- stated rows resolve through `read_sieve`.

A planted answer leak, a wrong eviction sequence and a wrong query slot each fail the test that targets them.

**Open for the PM** (none of these is set here):
1. **The bar.** Should it stay absolute against the registered `19/40`, or become a measured `Arm C − Arm A`? Card on #2833 (`6380260939`).
2. **Evicted rows state the evicted fact in their dialogue,** because the directory is populated from the dialogue's stated facts. Whether Arm A recalling it there counts as an abstention miss is a scoring rule.
3. **What `read_sieve_from` implies for Arm B.** This is a reading of the code, not a measurement:
   - On historical rows it returns the current value, since it takes no version select.
   - On unwritten and evicted rows it returns a same-entity slot's value, because the target is absent and a slot sharing one prime is admitted.

### Registration (deepseek-38, 2026-10-23)

The two decision cards were registered **as recommended** under Casey's standing direction (direction `6355948874`, confirmed `6356034555`), recorded at **2026-10-23T02:44:51Z** in `#2028` comment `6356136269`:

- **Oracle gain bar (`6356053677`) — registered.** ≥ `27/40` stated-fact vs the single `19/40` baseline, ≥ `7/20` historical-version, `20/20` abstention, `0` wrong-value substitutions; held-out BPB within `0.01` on the sealed fresh V2 draw; derivation `27 + 7 + 20 = 54/80`.
- **Frozen panel composition (`6356054074`) — registered.** `40` stated-fact / `20` historical-version / `20` abstention; the milestone's `>= 200` rows is the M3 decisive-run panel, not this oracle. The **PM** still selects the data source and freezes/hashes the file.

**This releases nothing.** The sheet stays `NOT_FIT` until a non-author FIT (per the registration, from Claude or Kimi — deepseek has authored text in this sheet), and the run still needs the held `#2844` checkpoint SHA and Casey's typed release. The `#2833` registration comment `6356137004` links to an unrelated old comment (`6100603222`) instead of `6356136269`; the registration itself is correct on `#2028`.
### Why NOT_FIT (prerequisites in priority order)

1. **Frozen, hashed panel absent (composition now REGISTERED).** The `40/20/20` composition and the 80-vs-200 reconciliation were registered at `2026-10-23T02:44:51Z`; the frozen panel still needs its content SHA-256, and the PM selects the data source and freezes the file.
2. **Candidate checkpoint unpinned.** The base artifact names `m1-trunk-assembly-and-training` (`#2844`), which is **HELD** (verified against the roadmap item on `main`), and carries no SHA-256; pin the exact frozen checkpoint and where it is stored.
3. **Bar — registered (resolved).** The owner's standing direction registered the bar at `2026-10-23T02:44:51Z` (`#2028` comment `6356136269`) with the single `19/40` baseline and the `27+7+20 = 54/80` derivation; the sheet now quotes it.
4. **Consumer commit-pinned (resolved 2026-10-22).** The delivered commit SHA `86eb68ebb` of the `uor-r4-object-store` address-read consumer and its 6 finite tests is now cited in §4 and in the pin-status table above; this item no longer blocks a FIT re-request.
5. **Measurement definitions unpinned.** The held-out BPB stream is unnamed and unhashed and the `0.01` tolerance is unsourced; `>= 54/80` has no derivation; and the stop rules cover integrity failures only — add a **bar-miss stop** and state which configuration a negative closes (CHARTER §4.1).

Re-request FIT once 1–3 and the remaining definitions in 5 have concrete values (item 4 is pinned above); this verdict clears no compute (D22).

### Recorded review

```text
TEST FITNESS: NOT_FIT - M1 #2029 m1-address-as-read
sheet: docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md
reviewer: deepseek (DeepSeek, non-author; fresh-context FIT review for #2833 on Casey's Route A direction)
note: Shape and finite controls are in good order: all 8 blocks present, address_read's 6 finite tests pass, the three arms and stop rules are clear, and the cost is bounded. NOT_FIT because the configuration this sheet would clear is not yet pinned: (1) the frozen panel has no hashed artifact - the composition 40/20/20 is stated, but the panel (or eval subset of it) needs its own SHA-256, and 80 rows must be reconciled with the m3-memory milestone's >=200-row decisive-run minimum; (2) the candidate checkpoint m1-trunk-assembly-and-training #2845 is HELD and carries no SHA-256; (3) the bar is a proposal - the owner's registered bar with ISO-UTC time must be quoted, and the 19/40 vs 20/40 baseline settled for the +8 gain; (4) the consumer must be cited by delivered commit SHA, not #2950. Also pin the measurement definitions: the held-out BPB stream is unnamed and unhashed, the 0.01 BPB tolerance is unsourced, >=54/80 has no derivation, and the stop rules cover integrity only - add a bar-miss stop and state which configuration a negative closes (4.1). Re-request FIT once pinned; this verdict clears no compute (D22).
date: 2026-10-22
A FIT verdict clears compute for the configuration in that sheet only (D22).
```

**Correction (2026-10-22, sheet update):** the verbatim review above pairs `m1-trunk-assembly-and-training` with `#2845`. The roadmap maps that item to **`#2844`** (`docs/roadmap/issues.json`, `docs/roadmap/README.md`), while `#2845` is `m1-w2-capacity-capture`. The quote is kept verbatim as the record; the live text in this sheet uses `#2844`.
