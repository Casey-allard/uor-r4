# m1-address-as-read (#2833) — Object-store address-as-read consumer and oracle pre-registration

## Summary

Delivers Steps 1–4 of [`m1-address-as-read`](../../roadmap/items/m1-address-as-read.md) (`#2833`):
1. **Design note** posted on [`#2833`](https://github.com/UOR-Foundation/uor-r4/issues/2833#issuecomment-6336053627).
2. **Object-store consumer module** [`crates/uor-r4-object-store/src/address_read.rs`](../../../crates/uor-r4-object-store/src/address_read.rs) (`PrimeAddress`, `ObjectTarget`, `AddressDirectory`, `AddressReadOutcome`, `VersionSelect`), resolving canonical prime/semiprime addresses and division-free sorted-merge GCD sieve queries to verified byte spans, whole objects, and token occurrences in `uor-r4-object-store`.
3. **Finite CPU controls** in [`crates/uor-r4-object-store/tests/address_read.rs`](../../../crates/uor-r4-object-store/tests/address_read.rs) covering exact hit, exact miss (`Absent` with `0` backend fetches), wrong address / tampered manifest / mismatched bindings refusal, versioned overwrite (`Current` vs `Previous`) and bounded capacity eviction (`Evicted`), sorted-merge GCD sieve admission and ranking, and byte-identical round trips across `Store` export/import at carry boundaries.
4. **Serving-time oracle pre-registration sheet** [`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md), ready for non-author `TEST FITNESS: FIT` review before any GPU evaluation run (Step 5).

## Geometry and ratchet checks

- **Geometry only, no softmax, no matmul (CHARTER §3.1–§3.3, D11, D23, D30):** `PrimeAddress` uses integer primality checks, sorted `u32` factor lists, exact semiprime products $k = p_a p_b$, and a two-pointer sorted merge for $\gcd(q, A_c) > 1$ admission and integer tuple ranking `(shared_primes, ordered_pair_matches, sequence)`. Zero floating point, zero exponentials/softmax/squashing, and zero matrix multiplication.

## Notebook row (in-scope record for `m3-durable-memory`)

Because `docs/research/notebook/m3-durable-memory.md` is outside `m1-address-as-read.md`'s frontmatter `scope:`, the notebook row is recorded here in the in-scope lab README:

| Date | Item | Change / Finding | Verification |
|---|---|---|---|
| 2026-10-22 | `m1-address-as-read` (`#2833`, Steps 1–4) | Built `uor_r4_object_store::address_read` (`PrimeAddress`, `ObjectTarget`, `AddressDirectory`, `AddressReadOutcome`, `VersionSelect`) connecting prime/semiprime addresses and sorted-merge GCD sieve queries to verified `uor-r4-object-store` payloads (`ByteSpan`, `WholeObject`, `TokenOccurrence`), with 6 finite CPU control suites and the serving-time oracle pre-registration sheet `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` (awaiting FIT before GPU evaluation). | `cargo test -p uor-r4-object-store` (61 passed: 55 existing + 6 new in `tests/address_read.rs`; binaries: `lib` 1, `address_read` 6, `bindings` 7, `controls` 10, `objpeer_cli` 22, `peer` 15; 67 passed with `--all-targets` including `protocol_vectors` 4 and `showcase` 2; 0 failed). |

## Oracle-sheet pin update (deepseek-33, 2026-10-22)

The non-author `NOT_FIT` review (`ee2f61417`) listed five pinned prerequisites before a FIT can clear a run. This unit pins the one that is the lab's and exposes the rest as a checklist:

- **PINNED — consumer commit.** [`crates/uor-r4-object-store/src/address_read.rs`](../../../crates/uor-r4-object-store/src/address_read.rs) and the six controls in [`crates/uor-r4-object-store/tests/address_read.rs`](../../../crates/uor-r4-object-store/tests/address_read.rs) are cited at commit **`86eb68ebb`** (PR `#2950`); re-check with `cargo test -p uor-r4-object-store --test address_read` (6 passed at that commit).
- **PENDING — PM:** frozen 80-row panel content SHA-256 and the 80-vs-200 reconciliation against D22 §3 (composition is the paper 40/20/20; panel-freeze step `#2858` (in `m3-qualify-memory`)).
- **PENDING — owner:** the exact frozen checkpoint SHA-256; `m1-trunk-assembly-and-training` (`#2844`) is HELD, and the sheet invents no checkpoint.
- **PENDING — owner:** the registered bar quoted with an ISO-UTC time and one baseline (`19/40` vs `20/40`; the `54/80` derivation follows from that choice).
- **PENDING — PM/owner:** the held-out BPB stream (named + hashed) and the source of the `0.01` tolerance.

The sheet now carries the two missing registered stop rules: a bar-miss is a negative for that exact configuration only (D22 §1), and integrity/panel-hash/checkpoint failures are instrument blockers. **No compute is cleared; the sheet remains `NOT_FIT` and the item stays `ready`.**

## Registration cards for the oracle (deepseek-38, 2026-10-23)

The merged FIT review (`ee2f61417`) left two owner/PM prerequisites for the serving-time oracle. Two decision cards with recommendations are now filed on `#2028` and linked from `#2833`; the oracle sheet records them in its pin-status table:
1. **Gain bar:** `27/40` stated-fact vs the single `19/40` baseline, `7/20` historical-version, `20/20` abstention, `0` wrong-value substitutions, held-out BPB within `0.01` on sealed fresh V2; derivation `27 + 7 + 20 = 54/80` (`#2028` comment `6356053677`).
2. **Frozen 80-row panel:** `40` stated-fact / `20` historical-version / `20` abstention; the milestone's `>= 200` rows is the M3 decisive-run panel, not this oracle (`#2028` comment `6356054074`). The PM selects the data source and freezes/hashes the file.

Both cards were then **registered as recommended** under Casey's standing direction at **2026-10-23T02:44:51Z** (`#2028` comment `6356136269`) — bar and panel composition — and the PM still selects the data source and freezes/hashes the panel. The remaining prerequisites outside lab control are the panel hash and the held `m1-trunk-assembly-and-training` (`#2844`) checkpoint SHA-256, plus Casey's typed release. The next FIT must come from a lab other than deepseek (deepseek has authored text in this sheet). No compute is cleared; the item stays `ready`.

## PM panel freeze (kimi-33, 2026-10-24)

- **Artifact:** `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v1.jsonl`, 80 rows / 21,364 bytes, SHA-256 `0712371d20e6eb1294c8faebeea7ab9827e2b91c0decfb5d85a52b8cca286789`; HF commit `59f964aa60b83b4684abaa34daddd0c6db768d4d`.
- **Source:** deterministic synthetic versioned facts from `crates/uor-r4-object-store/src/bin/address-panel-freeze.rs`, not corpus text. The panel has 40 current stated-fact hits, 20 previous-version hits, 10 unwritten `Absent` rows and 10 capacity-1 `Evicted` rows at sequence 2, with unique prompts and addresses.
- **Verification:** every expected result is checked through `AddressDirectory` and `Store` during generation. Two runs are byte-identical; generator tests 2/2 and the object-store suite pass; remote readback matches the local seal exactly. The sheet's earlier `NOT_FIT` and the remaining held checkpoint and BPB-stream prerequisites are unchanged. No GPU run, FIT claim or item-status change occurred.

## Panel repair (claude-71, 2026-10-24)

- **Superseded:** `panel-v1` (above) is an unfit instrument, and nothing should be evaluated on it. Class: instrument.
  - Every stated and historical prompt contained its expected value.
  - Every abstention prompt named its family.
  - So a no-read arm could pass the registered bar (#3131 `6380260818`).

  The file stays in the store untouched and regenerates from commit `1a70a9ac3`.
- **Artifact:** `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v2.jsonl`, 80 rows / 139,678 bytes, SHA-256 `369d8c24bb41e2d071485ecd1cbc7044092959ed5309ad70a8a5150a69a9e90c`, HF commit `bb81a8a33916bbc95ac192dce5b75c9ef5df1889`. The remote readback is byte-identical.
- **Rows:** each row is a dialogue of replayable `writes` (sequence, version, lexicon terms with primes, statement, value) under a directory `capacity`, followed by a `query` that names only an entity and a slot. The evaluator protocol and the Arm B word rule are in the sheet's [Panel repair](../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md#panel-repair-claude-71-2026-10-24) section.
- **Verification:**
  - **Tests:** the 6 generator tests parse the emitted JSON. Each planted leak or state fault fails its test.
  - **Suite:** `cargo test -p uor-r4-object-store` gives 82 passed, 0 failed, 1 ignored.
- **Unchanged:** the bar, the 19/40 baseline, the checkpoint (held `#2844`), `NOT_FIT` and every hold. The bar question is with the PM (#2833 `6380260939`).
