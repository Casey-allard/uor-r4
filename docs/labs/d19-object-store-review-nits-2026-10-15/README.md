# D19 review follow-up and seed-2 guard handoff — 15 October 2026

Addresses PM [review6183140856](https://github.com/UOR-Foundation/uor-r4/pull/2495#issuecomment-6183140856)
on approved source `c8441c0c6`. This follow-up is stacked on #2495 while that
protected PR waits for CI; it does not change its reviewed source.

- Current-state title uses the neighboring `YYYY-MM-DD — … (lab, line)` style.
- The implementation claim pins its approved delivery head `c8441c0c6`, replacing
  the original pre-development base. This is an exact source pin, not a claim of
  a merge that has not happened. After protected merge, verify on fresh main and
  release the claim during owned cleanup; do not invent a future merge SHA.
- `Error::Io` now carries typed `std::io::ErrorKind` and optional raw OS code,
  enabling callers to classify it without parsing a string. The enum still
  supports Clone/Eq; Display exposes these structured fields.
- Ingest and quotation intentionally use different policy ceilings: ingestion
  bounds a stored object's bytes (`max_object_bytes`), while an occurrence read
  bounds one returned response (`max_returned_bytes`) on the receiving store.
  Both preflight decoded-token size. A large valid stored token may therefore
  be refused by a tighter reader. API docs and a control make that distinction
  explicit; encoding/decoding and byte exactness are unchanged. The actual API
  name is `Bindings::read_occurrence` (the review's `read_object_occurrence`).

Focused validation: 22 object-store controls pass (20 existing + typed I/O mapping
and independent ingestion/quotation ceilings); fmt and Clippy are checked at the
posted source head. No model test, CUDA, pod or training is run. Existing reader
seam remains provisional until DeepSeek returns; the typed error change maps
through the adapter's existing display conversion without a trait shape change.

The separate [seed-2 guard appendix](../seed2-guard-integration-2026-10-15/README.md)
is a docs-only source trace for Claude's future sheet. It enumerates the exact25
finite controls, emitted-PTX admission, checkpoint0 receipt paths and failure
semantics. It explicitly records that current `completion` is not an
authenticate-and-stop command and high-u marks instrument_invalid without an
automatic stop. No build, FIT or personal release is implied.

Line: D19 infrastructure / evaluation integrity; retained M2 headline437/512,
count0/3 unchanged; seed2/M2 HELD and released seed1 untouched. Next: protected
review, parent-first delivery and verified cleanup; Claude prepares the re-issue
only after16,404; guard execution support activates only on Casey's GO.
