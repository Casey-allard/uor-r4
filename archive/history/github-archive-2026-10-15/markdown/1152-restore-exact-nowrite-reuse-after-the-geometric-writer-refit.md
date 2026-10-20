# #1152 — Restore exact NoWrite reuse after the geometric writer refit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T18:02:06Z
- **Updated:** 2026-10-09T13:17:59Z
- **Closed:** 2026-09-06T18:25:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1152
- **Merged:** 2026-09-06T18:25:35Z
- **Merge commit:** 8d530260a13701062cee30d2ffeaf9f420e6efb0
- **Base / Head:** main / UOR-Foundation:codex/writer-nowrite-reuse
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The corrected relation writer uses a new cue dictionary, so its inherited NoWrite signatures never hit. Repeated long-context text consequently triggers 226,101,330 writer row comparisons despite unchanged correct writes. This change compiles a writer-scoped exact cache and reduces that count to 539,448 on the same 28 prompts, preserving generated text, token IDs, state and copy/entry traces.

The cache binds the complete unchanged writer artifact. Compilation uses its actual cue namespace and individually certifies every NoWrite signature. Replacement writers retain up to 256 entries; this artifact has 140 observed negatives plus one certified alternate phase of an exactly periodic construction window. The short construction text lacks that phase of the longer repetition. The periodic proposal rule adds no observed frequency or serving heuristic; an exact guard and unchanged-writer fallback remain mandatory. Legacy cache lineage and the 64-entry legacy bound are preserved.

Artifact `2600b95b` leaves all learned writer/reader/router/tokenizer parameters fixed. It adds 40,995 serialized bytes and no session state. Exact skips are 21,799/21,907; writer feature queries fall from 17,392,410 to 41,496. The larger cache increases admission metadata reads, retained in the complete work report. This is a measured exact-reuse repair, not new language capability or evidence of general geometric semantic advantage.

Validation: 48/48 dependent answers and writes; 62/62 earlier answers; 24/24 transfer; 28/28 exposed-name answers/writes; 28/28 long-context answers/writes; 5/5 persistent turns with restore, forged-commit rejection and isolation. Four byte-identical generated Rust functions recompile and pass twelve semantic assertions. Eight focused relation/cache tests, formatting, architecture policy and claim wording pass. The allocation census is zero on the 140-entry predecessor with identical serving code and bound; a dedicated final-artifact allocation rerun and final CLI invocation are explicitly NOT_RUN. Final behavior runs through the actual shared Rust generation/session methods. All evaluation sets are already exposed preservation checks.

The record preserves both partial repairs, including the disproved cutoff diagnosis, and all resource receipts. This cycle uses 65.590 seconds of model work; cumulative use is 1796.744/1800 seconds, leaving 3.256. Monitored engineering is 1117.187/1200 seconds. A necessary 64 MiB storage increment uses standing owner authorization; no material is deleted and no paid external compute is used.

Updates current state, README, research, roadmap and the prior writer record. Next is learned admission of existing typed operators before execution, then committed derived-value use; full #1139/#1140 qualification remains open. See `docs/native_geometric_writer_admission_1139.md` and its evidence JSON for exact scope, commands and identities.

Refs #1139, #1140. Follows merged #1151. Protected CI names must be interpreted by their actual steps; compatibility acknowledgements do not certify audit/fuzz/WASM/Gate C.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

