# Clause polarity and source order panel

The panel has 96 supported prompts and six unsupported controls. Each of
three lexical families contains all 32 crossings of five binary factors:
query name order, clause polarity order, fact order, which person receives
each connector, and which person receives each object. In each family, both
people occur in each query position and each polarity. The gold person is
determined by the query name and polarity orders. The affirmative clause
requests the exact object wording and the other clause excludes one person.
The separator is a semicolon and the final terminator is a period in all
supported prompts. The inventory and collection blocks use two separately
authored query forms. The supplies transfer block uses the collection form
with new names, connectors and objects. The controls cover requests for
absent people, contradictory assertions of equal credibility, and requests
that omit which person the user means.

The grade fields `distractor_first`, `name_order_reversed` and `reversed`
encode exclusion-first, second canonical name first and reversed fact order,
respectively. `direction` is the exclusive OR of the first two flags. Each
constant query or fact position selects the correct person on 48 of 96
supported prompts. The supported grade binds the requested and excluded
names to disjoint half-open spans and the requested literal to its original
token slice through EOS. The generator also independently encodes both
possible replies to confirm that each literal and EOS are reachable from
the original writer tape.

The raw panel is identical across both pinned tokenizers, SHA-256
`52a2544fbff2ab62d4969091fb82f6bd576868abbf13f72f79ce0cfbbbf0691a`.
Under the D18 shape audit, all legal candidate shapes have support in the
frozen 200-record value TRAIN set on 80 of 96 rows per seed. The eight
`kept linen napkins` and eight `took brushes` rows have a shape without the
required support and retain statically copyable literals. The 32 supplies
transfer rows all have three legal starts on both facts and pass the shape
audit. The requested value begins at offset one on 16 and at offset three
on 16. Candidate distance admission and connector veto outcomes require
the separate runtime measurements.

The novelty census checked 46,420 unique JSON strings from the D20
historical inputs plus both D20 seed grades and raw text. No complete prompt
matches. Six factual names and six connectors have no exact field overlap
with the frozen TRAIN grades. Five of six complete objects, requested on
80 supported rows, have no exact overlap there.

## Reproduction

Build this standalone manifest offline with its checked lockfile. To
generate prompts for independent reading, run
`user-text-role-panel raw RAW`. The reader input contains only indices and
ordinary user text. Ask each reader for records with `index`, `person`,
`answer` and `status`. Use statuses `supported`, `unmentioned`, `conflict`
and `unspecified`.

Run `user-text-role-panel SEED TOKENIZER NEW_ROOT` with each pinned
tokenizer and the corresponding `STAGE/seed1` or `STAGE/seed2` root.
Reproduce each seed in a fresh root and compare all six files under
`held/`. Run `census.jq` with `jq -ne` and slurpfile variables `grade`
and `blind`. The D18 panel documents the shape audit and novelty
procedures. For the per-family shape counts, run `coverage-status.jq`
with `jq -ne` and slurpfile variables `candidates`, `grades` and `audit`.

After both readings, run `seal-panel.sh STAGE PANEL_BIN CAPTURE TOKENIZER1
TOKENIZER2 READER_A READER_B`. It requires exact agreement by both readers
and both seeds, then seals a projection containing only independently read
names. The capture binary is D17's standalone `uor-value-rank-capture`.
Its tokenizer-only `prepare` path independently reconstructs the ordinary
prefix and resolves the name seal to original token coordinates. The script
compares the reconstructed prefixes byte for byte and checks the ordinals,
requested spans and prefix hashes against each seed's generated tape.
The staged manifest covers the grades, reader receipts, independent
preparation outputs, and preflight evidence.
