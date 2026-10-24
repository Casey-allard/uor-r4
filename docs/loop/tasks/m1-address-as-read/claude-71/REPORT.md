# REPORT: m1-address-as-read, claude-71 (panel-v2 instrument repair)

Lab: claude. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). The item status is unchanged. Nothing is released and no compute is cleared.

## What did not work, first

1. **panel-v1 was an unfit instrument, and it merged before review finished.** #3131 merged 53 s after its one cross-vendor APPROVE, as the rules allow.
   - Each stated and historical prompt held its expected value: `address-panel-freeze.rs:30–31` and `:42–44` at `3454b69ed`.
   - Each abstention prompt named its family: `:56` and `:58`.
   - The values (`entity-00-fact-0-v3`) could be rebuilt from the prompt's own words.

   So a no-read arm could reach 40/40, 20/20 and 20/20, and the registered absolute bar (≥ 27/40 against a constant 19/40) would record a false PASS. Class: instrument. Nothing ran on it.
2. **The first build of v2 failed every test.** I had read `AddressDirectory::write`'s return value as the directory sequence; it is the slot version. Each write now records both, and the replay checks the version.
3. **The ORDER's FIT line could not be met as written.** No lab is left that authored neither the sheet nor a panel: antigravity drafted the sheet (`86eb68ebb`), deepseek and kimi updated it, and kimi and claude each built a panel. The amended line names a fresh-context reviewer instead, which CHARTER §4.5 counts as non-author.
4. **Three Arm B properties follow from the design and are not fixed here.** They come from reading `read_sieve_from`, not from a measurement.
   - On historical rows the sieve returns the current value, since it takes no version select.
   - On unwritten and evicted rows it returns a same-entity slot's value.
   - On evicted rows the evicted fact is still in the dialogue.

   These are scoring questions for the PM.

## Cancel probe

At `99cdddf60` the generator still interpolated `{value}` into its prompts (`:30–31`), and no panel-v2 existed in the sheet or the store. The task stands.

## What changed

| File | Change |
|---|---|
| `crates/uor-r4-object-store/src/bin/address-panel-freeze.rs` | Emits panel-v2. Each row is a dialogue: `writes` (sequence, version, terms with primes, address, statement, value) under a `capacity`, then a `query` that names only an entity and a slot. Values are SHA-256 letters. Every row is replayed through `Store`/`AddressDirectory` before it is emitted. Six tests (below). |
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | panel-v2 pinned and panel-v1 superseded, in the status line, the panel pin and the prerequisite table. A new "Panel repair" section gives the evaluator protocol, the Arm B word rule and the points open for the PM. "overwrite recall rows" is corrected to "stated-fact rows". The bar, the baseline, the checkpoint and `NOT_FIT` are unchanged. |
| `docs/labs/m1-address-as-read/README.md` | A panel-repair record. |
| this folder | ORDER (committed first; done-when 3 amended before review) and this REPORT |

## Measurements

- **Panel.** 80 rows (40 stated, 20 historical, 10 unwritten, 10 evicted), 139,678 bytes, SHA-256 `369d8c24bb41e2d071485ecd1cbc7044092959ed5309ad70a8a5150a69a9e90c`.
  - Uploaded to `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v2.jsonl` in HF commit `bb81a8a33916bbc95ac192dce5b75c9ef5df1889`.
  - The readback at that revision is byte-identical (`cmp`).
  - `panel-v1.jsonl` in the same revision still hashes to `0712371d…`.
- **Generator tests**, 6/6, all on the JSON as emitted:
  - **Composition and reproduction:** two runs are byte-identical, and the 40/20/10/10 split, the ids and the dialogue+query pairs are unique.
  - **No answer or family word in a query:** no query holds any value of its row or a family word, and each matches its template.
  - **Replay:** every row's writes, replayed, reproduce its oracle outcome, bytes and eviction sequence.
  - **Arm B:** the query-word rule derives the oracle address on all 80 rows, from one panel-wide one-to-one lexicon, and each statement's words give its write address.
  - **Values:** all distinct, 8 lowercase letters, with no lexicon word in any of them.
  - **Sieve:** stated rows resolve through `read_sieve` to their expected value.
- **Planted faults.** Each was planted after the commit, the bin tests were run, and the file was restored from the commit.
  - A dialogue value appended to the query fails "no answer or family word".
  - `evicted_at_sequence + 1` fails "replay".
  - The query naming the next slot fails "Arm B", "sieve" and the template check.
- **Suite.** `cargo test -q --locked -p uor-r4-object-store` gives 82 passed, 0 failed, 1 ignored. Before this unit it gave 78 passed, of which 2 were v1's generator tests.

## Checks run at the head

`cargo test -p uor-r4-object-store`, `loop report --check`, `loop pr-check --local`, `git diff --check`.

## Skills used

- **`solution-integrity`:** every v1 failure has a test that fails when it is planted. The lexicon is frozen into the rows, not left implicit.
- **`scope-guard`:** only the item's scoped paths and this folder.

## Still open

- **The PM's card on #2833 (`6380260939`):** should the bar stay absolute against the registered 19/40, or become a measured `Arm C − Arm A`?
- **The PM's scoring calls on the Arm B and evicted-row points above.**
- **Before any run:** the held `#2844` checkpoint SHA, the BPB stream's digest, a fresh-context FIT, the PM's dispatch, and Casey's approval.

## Plain summary

The test panel for the memory-read experiment gave away its answers, so a model with no memory could have passed. The new panel separates what is said from what is asked. It keeps every answer out of the questions, and it records enough state for anyone to replay each row exactly. Tests fail if a leak comes back.
