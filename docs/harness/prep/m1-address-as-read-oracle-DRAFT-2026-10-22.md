# Serving-time address-as-read oracle over `uor-r4-object-store` (DRAFT)

- **Status:** DRAFT pre-registration sheet for `#2833` (`m1-address-as-read`, Step 4). **FIT-reviewed by deepseek (non-author) on 2026-10-22: `NOT_FIT`** (see [Verdict](#verdict)); **no GPU run is cleared or started** until the four pinned prerequisites in the Verdict are met and Casey releases the run.
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

- **Oracle gain bar (Arm C):** At least `+8/40` more recalled fact rows than the `19/40` (`20/40` anchor) baseline (`>= 27/40` on the memory recall half, and `>= 54/80` across the full 80-row panel), with `0` wrong-value substitutions from `uor-r4-object-store` (`whole_object_verified` / range-verified byte match `1.000` on every `Hit`) and held-out BPB within `0.01` of Arm A.
- **Trained follow-up gate:** Only if Arm C meets the oracle gain bar is a trained address-as-read sheet eligible to be drafted (2–3 seeds, frozen `>= 80`-row panel, separate FIT review, and typed owner release).

## 4. Base artifact

- **Consumer library and controls:** `crates/uor-r4-object-store/src/address_read.rs` (`PrimeAddress`, `AddressDirectory`, `ObjectTarget`, `AddressReadOutcome`) and `crates/uor-r4-object-store/tests/address_read.rs` (landed in `#2950`).
- **Candidate model checkpoint:** Frozen M1 discrete trunk checkpoint (`m1-trunk-assembly-and-training` `#2845`), SHA-256-pinned before evaluation.

## 5. Data split

- **Frozen panel:** At least `80` rows (`40` stated-fact / overwrite recall rows, `20` previous-version / reassertion rows, and `20` unwritten / capacity-evicted abstention rows) in `docs/harness/panels/m3-memory.json` (once frozen by `m3-memory-panel-freeze` `#2853`), SHA-256-pinned in the FIT-locked sheet before any GPU run, plus held-out text stream for bits-per-byte (BPB) non-regression.
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

## 8. Cost and minimum test power

- **Cost:**
  - `preparation`: `$0.00` (CPU consumer module and finite controls in `#2950`).
  - `run`: single RTX 5090 evaluation-only run (zero training steps), `< 15` minutes (`< $0.25`), within the `<= 4` pods / `<= $8/h` fleet cap.
  - `controls`: Arm A (`no_address_read`) and Arm B (`sieve_address_read`) evaluated in the same pass.
  - `evaluation`: deterministic exact-match and byte-hash verification (`$0.00`).
  - `retries`: `0`.
  - `storage`: `< 1 MiB` evaluation report.
- **Minimum test power (D22 §3):**
  - Frozen panel of `>= 80` rows (`40` stated-fact / overwrite recall rows, `20` historical-version rows, `20` unwritten / evicted abstention rows) evaluated across all 3 arms (`Arm A`, `Arm B`, `Arm C`).

## Verdict

### Why NOT_FIT (prerequisites in priority order)

1. **Frozen, hashed panel absent.** The 40/20/20 composition is stated, and matches the paper design, but the frozen panel (or its evaluation subset) has no content SHA-256, and the m3-memory milestone entry says `>= 200` rows for a decisive run; state the hash and reconcile 80 vs 200 against D22 §3.
2. **Candidate checkpoint unpinned.** The base artifact names `m1-trunk-assembly-and-training` (`#2845`), which is **HELD** (verified against the roadmap item on `main`), and carries no SHA-256; pin the exact frozen checkpoint and where it is stored.
3. **Bar is still a proposal.** Quote the owner's registered bar with an ISO-UTC time (CHARTER §5.2), and settle the `19/40` vs `20/40` baseline wording so the `+8` gain has one baseline.
4. **Consumer not commit-pinned.** "Landed in `#2950`" should be the delivered commit SHA of the `uor-r4-object-store` address-read consumer and its 6 finite tests.
5. **Measurement definitions unpinned.** The held-out BPB stream is unnamed and unhashed and the `0.01` tolerance is unsourced; `>= 54/80` has no derivation; and the stop rules cover integrity failures only — add a **bar-miss stop** and state which configuration a negative closes (CHARTER §4.1).

Re-request FIT once 1–5 are pinned; this verdict clears no compute (D22).

### Recorded review

```text
TEST FITNESS: NOT_FIT - M1 #2029 m1-address-as-read
sheet: docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md
reviewer: deepseek (DeepSeek, non-author; fresh-context FIT review for #2833 on Casey's Route A direction)
note: Shape and finite controls are in good order: all 8 blocks present, address_read's 6 finite tests pass, the three arms and stop rules are clear, and the cost is bounded. NOT_FIT because the configuration this sheet would clear is not yet pinned: (1) the frozen panel has no hashed artifact - the composition 40/20/20 is stated, but the panel (or eval subset of it) needs its own SHA-256, and 80 rows must be reconciled with the m3-memory milestone's >=200-row decisive-run minimum; (2) the candidate checkpoint m1-trunk-assembly-and-training #2845 is HELD and carries no SHA-256; (3) the bar is a proposal - the owner's registered bar with ISO-UTC time must be quoted, and the 19/40 vs 20/40 baseline settled for the +8 gain; (4) the consumer must be cited by delivered commit SHA, not #2950. Also pin the measurement definitions: the held-out BPB stream is unnamed and unhashed, the 0.01 BPB tolerance is unsourced, >=54/80 has no derivation, and the stop rules cover integrity only - add a bar-miss stop and state which configuration a negative closes (4.1). Re-request FIT once pinned; this verdict clears no compute (D22).
date: 2026-10-22
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
