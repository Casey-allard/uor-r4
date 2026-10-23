# ORDER: m1-position-mechanism, claude-58 (freeze the order-sensitive DEV slice and pin its hash)

Lab: claude. Item: [m1-position-mechanism](../../../../roadmap/items/m1-position-mechanism.md) (#2843). Base: `7f0f0710c`. Item PR (partial: `References #2843`; the item stays `ready`).

## Goal

§5 of the position sheet `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` pins the rule for its multi-occurrence, order-sensitive DEV slice. The hash is still `PENDING — frozen subset`. deepseek-39 and DeepSeek's FIT request (#2028, 6357319934) name freezing it as the open data step before a FIT can clear the run. This unit freezes the slice and pins it.

1. **Selector.** A pure function in `uor-r4-discrete` applies §5's rule to one DEV row.
   - The row's window is the one `PreparedCorpus::batch` builds at `context = 192`: bounded by the document, ending at the query token. `is_copy_successor` reads the same window.
   - The row is selected when the query token occurs at least twice before the query, and the tokens following those occurrences are pairwise distinct.
   - The occurrence just before the query counts, with the query itself as its successor, as in `is_copy_successor`.
2. **Freeze.** Run the selector over all 14,287 DEV rows.
   - Load them with `PreparedCorpus::load` from `data/ft-dev.tar` in the HF store (sha256 `45e93e32…`), with the pinned UORT digests `0919e90f…` / `a838bb1a…` checked on load.
   - Write one line per selected row, `row_ordinal<TAB>target_position`, to `docs/labs/m1-position-mechanism/order-slice-v1.tsv`.
   - Record N and the file's SHA-256.
3. **Pin.** Put the SHA-256 and N in §5 and the Status line. Claude then has authored sheet text, so the non-author FIT has to come from Kimi.

The selector follows the rule's literal reading: every successor is distinct from every other. The README also reports a diagnostic count under the looser reading (at least two distinct successors), and pins nothing on it.

## Cancel probe (run first)

At the base, is §5's hash already pinned, or does a frozen subset file exist? If either is true, the task is void. Class: instrument.

## Files in scope

- `crates/uor-r4-discrete/**` (the selector, its tests, and an example that freezes the file)
- `docs/labs/m1-position-mechanism/**`
- `docs/harness/prep/m1-position-mechanism*` (§5's hash and the Status line only)
- This task folder

## Done when

1. **Unit tests pin the rule on hand-built windows.**
   - Two occurrences with distinct successors are selected.
   - A repeated successor, a single occurrence and a query with no earlier occurrence are each rejected.
   - The occurrence just before the query counts.
   - A planted selector that ignores distinctness fails a test.
2. **The frozen file reproduces.** It comes from the pinned DEV split, with its digests checked on load. The lab README records N, the file's SHA-256 and the command. Re-running the command reproduces the file byte for byte.
3. **A short slice is recorded, not pinned.** If N < 200, the sheet records the shortfall instead of a hash. That is a bar question for the PM, and nothing else changes.
4. **Checks pass:** the crate tests, clippy `-D warnings`, `report --check` and `pr-check --local`.

## Stop and ask if

- Loading the DEV split fails its pinned digests.
- The rule needs any input besides the pinned DEV tokens and mask, for example a different context than 192.
