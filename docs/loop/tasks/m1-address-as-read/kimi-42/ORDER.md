# ORDER: m1-address-as-read, kimi-42

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `911998594`.

## Goal

Implement the approved object-store oracle instrument: versioned 40-current/20-previous/20-abstention manifest, pure exact/sieve controls, and a five-arm GPU run through the existing D11 scorer. Require exact true-address hard emission, baseline parity and planted-wrong controls, then report true/sieve/no-memory contrasts and resource cost. After the measured result and D68 review, this unit **Closes #2833 at the exact-read/oracle scope only** and removes the stale M2-trunk dependency. No hidden-fact or learned-memory claim follows.

The item's done-when remains:

> A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## Cancel probe (run first)

kimi-40 repeated Stage 0 with no new information. kimi-41's D68-approved design names the unused dependency: no existing five-arm composition joins `AddressDirectory` exact/sieve reads to the Stage 0 GPU scorer. Source inspection confirms `read_exact_from` and `read_sieve_from` are delivered with finite controls, while no `address-oracle` binary or `uor-r4.address-oracle-panel/1` manifest exists. The next step is new composition, not a repeated measurement. `classification: instrument`. **Not cancelled.**

## Files in scope

- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/labs/m1-address-as-read/**`
- `docs/harness/prep/m1-address-as-read*`
- `docs/loop/tasks/m1-address-as-read/kimi-42/**`
- `docs/research/notebook/m3-durable-memory.md`
- `docs/roadmap/items/m1-address-as-read.md`
- `docs/roadmap/README.md` (rendered only)
- `PLAN.md`

No integer crate, D11 artifact or default-serving change. Any dependency change needs a declared decision record before it is made.

## Done when

- Failing-first pure controls cover canonical/exact/eviction semantics, invalid primes/pairs/token bindings, missing previous versions, bounded limits, deterministic manifest generation and no unauthorized fallback.
- The versioned manifest supports 40 current, 20 previous and 20 abstention rows, a deterministic sieve lexicon, present/value-different random addresses and cross-row/TRAIN/DEV leak checks.
- A pre-registered GPU run uses no-memory, no-address, sieve, true-address and random-address arms, preserves 14,287-row parity and byte-identical repeats, and yields exactly 40/40 current, 20/20 previous, 20/20 typed abstention and zero wrong substitutions.
- Receipts separate `sieve_address_match` from `sieve_value_check_pass`, exclude previous rows from true-vs-sieve, report paired contrasts, baseline/assisted throughput and store/access costs, and include a ceiling outcome.
- FIT and D67 reviews precede dispatch; full gates, REPORT, local policy and fresh exact-head review pass. Preserve negative results and do not freeze a hidden-fact panel in this unit.

## Stop and decide if

- Any model evaluation would run on the host, including fallback; use only the existing GPU scorer.
- Hard store emission would be called learned model capability, or a no-gain ceiling would be called an instrument failure.
- A second store, unbounded scan, dense matmul, softmax/squashing or unversioned artifact change appears necessary.
- Any wrong substitution, baseline drift, missing control, leak or nondeterminism occurs: classify as instrument failure and do not train or enlarge the panel.
- The work would require code outside this ORDER's scope; stop and re-scope rather than overlapping another lab.
