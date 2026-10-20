# #796 — fix(#790): CLI batch — pin the 1.7B revision, loud unknown subcommands, real default-model discovery (items 1-3)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T18:26:06Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-18T18:49:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/796
- **Merged:** 2026-08-18T18:49:58Z
- **Merge commit:** ef35625771847b8d661705bdb078fa9abc1de747
- **Base / Head:** main / UOR-Foundation:issue-790-cli-batch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #790 (items 1-3; the batch the issue's next-action names — pure CLI/orchestrator, no serving semantics; items 4-8 stay individual).

- **Item 1**: `uor-r4-cli` menu 3 and the in-client 1.7B compile trigger pinned to `31b70e2e869a7173562077fd711b654946d38674` (resolved from HuggingFaceTB/SmolLM2-1.7B-Instruct main, 2026-08-18) — `HF_REV=main` was refused by the 40-hex rule by design, breaking Stage 1 on fresh machines.
- **Item 2**: unknown `transformerless` subcommands now exit nonzero naming the subcommand (`help`/no-args keeps the successful usage print); the banner no longer advertises `compare`/`compare-report`, which this dispatcher never handled. Verified live: `r4 transformerless compare` exit 1 (was 0), `help` exit 0. Falsifier tests included.
- **Item 3**: bare `r4 ask` default-model discovery skips tokenizer-only descriptors (architecture+weight_format required), prefers a locally compiled bundle, and orders (compiled, mtime, name) deterministically. Was: picks mtime-newest json → t5-base-tokenizer → hard error (audit-verified). Now: bare `r4 ask` serves smollm2-135m-instruct on this machine, exit 0. Falsifier test seeds a newest tokenizer-only descriptor and asserts it cannot win.

One commit per item per the batch-flow convention. Gates: fmt, clippy -D warnings (touched crates), wasm-router lib 241 green, graph-cli lib 128 green, xtask/register gates green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

