# ORDER: m1-address-as-read, claude-71 (panel-v2 instrument repair)

Lab: claude. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `99cdddf60`. Casey, in Claude's chat at 2026-10-24T05:26Z: "Yes: claim and build address-panel-freeze v2 (panel-v2, queries free of answers, replayable writes, Arm B primes from query words) as a Claude unit on m1-address-as-read with ORDER, tests and cross-vendor review; leave the bar question to the PM."

## Goal

panel-v1 (`0712371d…`, #3131) is an unfit instrument (findings on #3131, 6380260818):
- every stated and historical prompt contains its expected value;
- the abstention prompts name their family;
- Arm B's primes are constants unrelated to the prompt words;
- the store state cannot be replayed from the file.

A no-read arm could therefore pass the registered bar. This unit replaces the generator's output with panel-v2. Class: instrument.

## Cancel probe (run first)

At the base, does the generator still interpolate the expected value into its prompts (`address-panel-freeze.rs:30–31`), and does no panel-v2 exist in the sheet or the store? If not, the task is void.

## Files in scope

- `crates/uor-r4-object-store/**`
- `docs/labs/m1-address-as-read/**`
- `docs/harness/prep/m1-address-as-read*`
- this folder

## Done when

1. **Generator.** `address-panel-freeze` emits panel-v2: 80 JSONL rows with the composition unchanged (40 stated, 20 historical, 10 unwritten, 10 evicted), byte-reproducible. Each row carries:
   - its dialogue `writes` (sequence, terms with their primes, statement, value);
   - its directory `capacity`;
   - a `query` and `query_terms`;
   - the oracle address;
   - the expected outcome.
2. **Tests (bin unit tests).** They parse the emitted JSON and pin:
   - no query contains any value of its row or a family word, and every non-historical query uses one template;
   - replaying each row's `writes` into a fresh `Store`/`AddressDirectory(capacity)` and reading the oracle address gives the recorded outcome, bytes and eviction sequence;
   - Arm B's stated rule (lexicon words of the query, in order) derives exactly the oracle address, from one panel-wide lexicon;
   - values are distinct and contain no lexicon word or digit.
3. **Freeze.**
   - `panel-v2.jsonl` is uploaded to `caseyallard/uor-r4-store/data/m1-address-as-read/` and read back identically.
   - The sheet and the lab README pin its SHA-256. panel-v1 stays untouched, marked superseded.
   - The sheet's FIT line names a lab that authored neither the sheet text nor a panel. *Amended before review:* no such lab exists, because antigravity drafted the sheet (`86eb68ebb`). The line instead names a fresh-context reviewer, which CHARTER §4.5 counts as non-author, with the PM choosing who runs it.
4. **Checks.** `cargo test -p uor-r4-object-store`, `loop report --check`, `loop pr-check --local` and `git diff --check` pass.

The bar, the 19/40 baseline, the checkpoint, the FIT verdict (`NOT_FIT`) and every hold are unchanged. The bar question is the PM's (#2833 6380260939).

## Stop and ask if

- the bar, the composition, the checkpoint or an external destination must change;
- a file outside scope must change.
