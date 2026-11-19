# Clause-relative role panel

Run `seal-clause-role-panel SEED ASSETS NEW_OUTPUT_DIRECTORY` with seed 1 or 2 and the corresponding frozen tokenizer and paired D11 files from D15. The executable checks both source hashes. Build from this directory with `cargo build --release --offline --locked` and a fresh or lab-owned target directory. Verify each fresh output with `shasum -a 256 -c seedN.sha256` from its directory, passing the full path to the checksum manifest.

## Prompts and discovery

Both seed packets use the same text. TRAIN and held have disjoint people, items and clause templates. The far and root-order families share “keep” and “drop” across splits. The four forms cross eight assignment arms, two name pairs for each of four support groups, two clause orders and two fact orders. Two support groups give both people an earlier assertion. The remaining two each give an earlier assertion to a different one of the two people. The absent fact is replaced by a neutral inventory sentence that supplies no person. Supporting facts place an unambiguous object after “owns”. The role labels are unspecified, requested and excluded, encoded as 0, 1 and 2. The held-only ambiguity control leaves a choice unresolved and supplies both person names in the same clause. Both receive unspecified, because the text selects neither. These 32 prompts form their own cohort.

For example, a held single-request polarity pair uses the same fact prefix “Facts. Emily owns brass lamps. William owns paper kites. Return only the requested inventory items.” followed by these two interchangeable continuations.

> choose to keep instead of drop Emily; choose to drop instead of keep William.
>
> choose to drop instead of keep Emily; choose to keep instead of drop William.

An ambiguous continuation with those facts is “copy exactly one item belonging to Emily or William; the instruction leaves the choice unresolved.”

The Rust author encodes the full assistant-prefix tape with the bound D11 tokenizer. It identifies the query by its last two full-stop tokens and the clauses by the single semicolon token between them. It uses native whole-word candidate discovery and accepts ASCII title-case spans with one leading space. It checks for an exact earlier statement-head token match to annotate source support. It sends every discovered mention to both clause-local projections, including mentions with no source and both co-mentioned people in the ambiguous cohort. The class head and clause strictness are diagnostic and punctuation-dependent. These constructed prompts restrict capitalized query words to people. The text grammar, names with a different capitalization convention, and independent recovery of arbitrary clause boundaries require separate work.

The projection uses `RolePrompt::feature` from `uor-r4-integer/src/ordered_role.rs`. Both variants have 32 signed integer coordinates and radius 12 on either side of the mention. The mention itself is excluded from both projections. Every coordinate reference and raw device-packet row uses the same discovered spans and fences. The program checks the discovered names and source support against panel truth only after generating both coordinate rows.

## Files per seed

| File | Contents |
|---|---|
| `tables.bin` | 4,096 packed-root/class pairs as little-endian `u32`, 16×8 signed orbit-root bytes and 64×8 signed axis-selector bytes, using D15's table layout |
| `{train,held}.candidates.txt` | One row per mention with mention start/end, clause start/end and full BOS-inclusive token IDs, separated by spaces. Offsets are zero-based and half-open. |
| `{train,held}.{identity,positional}.txt` | 32 signed integer coordinate words per mention, separated by single spaces |
| `{train,held}.slots.txt` | 24 packed-root words per mention, nearest to farthest left then nearest to farthest right, with −1 for a clipped slot |
| `{train,held}.rows.jsonl` | Mention metadata in coordinate-row order. `prompt_id`, `person`, `span`, `clause`, `clause_index`, `assertion_supported`, `assertion_span`, `family`, `form`, `variant`, `ambiguous`, `name_variant`, `clause_order`, `fact_order`, and four matched group IDs. `polarity_pair` identifies requested/excluded flips and is null for other arms. `name_variant_pair` associates a swap to a different name pair. `clause_order_pair` and `fact_order_pair` reverse the corresponding independently crossed order. |
| `{train,held}.prompts.jsonl` | One row per prompt with `prompt_id`, `text`, exact framed `prefix` and both `clauses` |
| `{train,held}.labels.txt` | Role truth in mention-row order. Keep held labels away from fit and device inference. |
| `{train,held}.writer-gold.jsonl` | Prompt ID and conditional `requested_person` / `answer_literal`. Both are null except where exactly one person is requested and has a known object. |
| `census.json` | Seed, input hashes, prompt/mention counts, per-family counts, role counts split by source support and maximum observed absolute coordinate |

Treat row metadata and prompt payloads as evaluator and audit inputs. The factorial IDs and metadata contain information about the prompt construction. The device receives the tables and candidate tapes or projected coordinates. TRAIN labels are consumed separately during fit.

## Finite inventory

Each seed has 1,024 distinct TRAIN prompts and 1,056 distinct held prompts, including 32 held cases with unresolved choices. Each role split has two discovered people per prompt. The eligible single-supported writer subset contains 384 held prompts.

On each seed and split, the root-order family has 64 person-level polarity comparisons. Every pair retains bit-identical Identity vectors and changes its Positional vector. The other three families change both vectors on all 64 comparisons per family. On held, all 1,056 person-level clause-order pairs and all 1,056 fact-order pairs are bit-identical in both variants, including the ambiguous control.

The contexts repeat a small set of geometric inputs. Each seed has 11 distinct TRAIN and 27 distinct held Identity vectors, plus 12 distinct TRAIN and 28 distinct held Positional vectors. Each variant has zero exact coordinate rows in common across TRAIN and held. Counts of unique raw prompts measure factorial coverage. Generalization beyond these phrasing families and eligibility rules remains open. The finite inventory compares coordinates without scoring or fitting.
