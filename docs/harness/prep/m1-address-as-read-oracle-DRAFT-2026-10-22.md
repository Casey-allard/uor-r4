# Serving-time address-as-read oracle over `uor-r4-object-store` (DRAFT)

- **Status:** DRAFT pre-registration sheet for `#2833` (`m1-address-as-read`, Step 4). **Not yet FIT-reviewed**, and **no GPU run is started**.
  - CPU consumer module (`uor_r4_object_store::address_read`) and finite CPU controls are delivered in this PR (`$0` GPU spend).
  - GPU evaluation of the serving-time oracle awaits a non-author `TEST FITNESS: FIT` review (CHARTER §4.5) and Casey's approval in chat; any trained form also requires a separate FIT review and a typed owner release (CHARTER §5.1).
- **Line:** M1/M3 exact addressed memory (`#2029`, `#2031`, item `#2833`). **Lab:** antigravity.
- **Rules:**
  - CHARTER §3.1–§3.3 (D23, D25/D30): Geometry only; zero softmax or squashing; zero dense matrix multiplication.
  - CHARTER §3.5 (D24): Model evaluation runs on the GPU (RTX 5090).
  - CHARTER §4.5: Frozen, SHA-256-pinned panel and non-author `TEST FITNESS: FIT` before GPU evaluation.

## 1. Question and mechanism tested

`docs/mechanisms/exact-memory.md` ("What would reopen it", Step 2) asks to **supply the true slot address at serving time and measure the gain before any training**, separating the address-as-read consumer over `uor-r4-object-store` (`AddressDirectory` + `Store`) from learned query-entity/relation tagging (which caused 77.7% of held-out failures in the G v1 decomposition, `archive/integration/g1-decomposition-result-2026-09-29.md`).

## 2. Arms (zero training)

Evaluated on one frozen checkpoint on RTX 5090 across three deterministic serving-time read modes over the same `AddressDirectory` populated from the dialogue context's stated facts:

1. **Arm A (`no_address_read`, baseline anchor):** Frozen model evaluated with the object-store address read disabled (`AddressReadOutcome::Absent`).
2. **Arm B (`sieve_address_read`, deterministic prime-sieve query):** Query address formed deterministically from the prompt's content-word primes and ordered pairs (`AddressDirectory::read_sieve`, sorted-merge $\gcd(q, A_c) > 1$ admission and integer tuple ranking `(shared_primes, ordered_pair_matches, sequence)`).
3. **Arm C (`oracle_address_read`, true slot address supplied):** True `(entity_prime, relation_prime)` `PrimeAddress` supplied to `AddressDirectory::read_exact(..., VersionSelect::Current)` (or `VersionSelect::Previous` on historical-version rows), resolving the verified `ObjectTarget` from `uor-r4-object-store::Store`.

## 3. Frozen panel and metrics

- **Frozen panel:** At least `80` rows (`40` stated-fact / overwrite recall rows, `20` previous-version / reassertion rows, and `20` unwritten / capacity-evicted abstention rows), SHA-256-pinned in the FIT-locked sheet before any GPU run, plus held-out text stream for bits-per-byte (BPB) non-regression.
- **Reported metrics (separated per `exact-memory.md` Step 5):**
  1. **Oracle read accuracy (Arm C vs Arm A):** exact-match `check_pass` on the recalled fact rows when the true `PrimeAddress` is supplied.
  2. **Deterministic sieve address-formation accuracy (Arm B vs Arm C):** fraction of rows where `read_sieve` selects the same `PrimeAddress` and `ObjectVersion` as the oracle (`address_match`), reported separately from downstream token emission (`check_pass`).
  3. **Honest abstention / eviction accuracy:** exact `Absent` / `Evicted` classification on unwritten and capacity-evicted rows.
  4. **Held-out BPB difference:** BPB of Arm B and Arm C vs Arm A on the held-out text stream.

## 4. Pre-registered bar (to lock at FIT)

- **Oracle gain bar (Arm C):** At least `+8/40` more recalled fact rows than the `19/40` (`20/40` anchor) baseline (`>= 27/40` on the memory recall half, and `>= 54/80` across the full 80-row panel), with `0` wrong-value substitutions from `uor-r4-object-store` (`whole_object_verified` / range-verified byte match `1.000` on every `Hit`) and held-out BPB within `0.01` of Arm A.
- **Trained follow-up gate:** Only if Arm C meets the oracle gain bar is a trained address-as-read sheet eligible to be drafted (2–3 seeds, frozen `>= 80`-row panel, separate FIT review, and typed owner release).

## 5. Stopping criteria and cost

- **Stopping criteria:** Any finite CPU control failure in `cargo test -p uor-r4-object-store --test address_read` blocks GPU evaluation; zero retries with modified code or panel after FIT.
- **Estimated GPU cost:** Single RTX 5090 evaluation-only run (zero training steps), `< 15` minutes (`< $0.25`), within the `<= 4` pods / `<= $8/h` fleet cap.

TEST FITNESS: PENDING
