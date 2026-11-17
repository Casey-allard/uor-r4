# ORDER for past-window scorer input refusals

Lab: codex. Item: [integer-long-context](../../../../roadmap/items/integer-long-context.md). References #3714. Source: [PR #4039](https://github.com/UOR-Foundation/uor-r4/pull/4039) by @Rajaykumar12 at `ad1035e499ad53ea6c32a82ccc1af7327253ed54`. Adoption base: `9465b92d6e11ca0ee913ebe34aaaddff0cdf97f5`.

## Cancel probe

The issue and PR were open and the source head matched the reviewed Rust diff. The changes apply to the adoption base. Stop for a moved contributor head, unrelated changes, or overlap that would overwrite work on main.

## Scope

Refuse a missing deciding objective, band/loss length mismatch, mixed panel carry contexts and zero-width document windows. The model context used for carry must equal the context recorded for each panel even when the checkpoint window is larger. Only the past-window scorer and its document window helpers, their host tests and this task folder.

## Done when

Run the host refusal, default-window and carry-context controls. Pin W=4096 and C=2048. Item 5 of the issue stays open. Preserve the contributor commit, record outcomes and unresolved checks, then use the protected PR review and merge queue. Stop before any training, panel evaluation or change to a research bar.
