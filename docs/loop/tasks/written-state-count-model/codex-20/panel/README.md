# Crossed role and written-value panel

The panel contains 32 supported prompts and six unsupported controls. Each
of two lexical families crosses requested person, assignment of the two
objects, assignment of the two connectors and fact order. Each family uses its
own wording with the requested person named before the explicit exclusion.
Within each family each connector and object combination occurs four times
as the supporting assertion. Both first and second fact support 16 rows.
The same connector and object strings from D19 are retained, and every full
prompt is fresh. Wording and lexical family are fixed together.

The 38 raw prompts are identical across both pinned tokenizers. The raw
SHA-256 is `d9b76a360be2560f88a3f41216a4c8b202b51ebd6e73bdb1705e8750b5a4fb85`.
The grade binds each supported answer to a half-open source interval in the
original protocol-2 prefix and the exact source token slice through EOS.
The generator separately encodes every fact's literal as an ordinary
assistant reply and compares it with that fact's source slice plus EOS. Both
supported facts on every row have legal starts with distinct possible
completions. The retained negative controls request an absent person,
present two mutually exclusive accounts, or omit the requested person.

## Preflight

The D18 anonymous shape audit against the frozen base and supplemental FIT
records returns 28 of 32 covered supported rows on both tokenizers. The four
remaining rows have the shorter inventory connector `took` and `brushes`
as the target fact. Their offset-one candidate of shape
`[1,3,[1,1,2,2]]` lacks the required frozen shape support. Both original
literals on those rows still have legal writer starts. Keep all rows and
treat geometric nomination and actual device writing as separate checks.

The D18 novelty procedure scanned 46,289 historical JSON strings, including
D19. It found no exact prompt matches. The four fact names and four
connector phrases have no overlap with the frozen 200-record value FIT
grade sets. Three literals have no exact field overlap there, accounting for
24 supported rows. The other eight request `brushes`. This census concerns
the frozen value FIT grades and the pinned historical input list.

## Reproduction and reader seal

Build this standalone manifest offline with the checked lockfile. Invoke
`user-text-role-panel SEED TOKENIZER NEW_ROOT` on each seed's pinned
tokenizer. Run the census with `jq -ne --slurpfile grade held/grades.json
--slurpfile blind held/blind.json -f census.jq`. The D18 panel documents
the shape and novelty programs. Add both D19 raw text and grade files to
its historical inputs.

Give independent readers only `held/public-text.json`. Require records
with `index`, `person`, `answer` and `status`. Use `adjudicate.jq`
with slurpfile variables `raw`, `reader_a`, `reader_b`, `seed1` and
`seed2` (the last two are the grade files). Check `.all_agree` before
sealing. Project either verified reader to `map({index,person})`; pass that
result to `user-text-role-panel seal RAW PERSON_ONLY HELD_NAMES`. Use
D18's `grader-reading.jq` on the verified reader for
`reader-grader.json`.

Use D17 capture's tokenizer-only `prepare` command with the pinned
tokenizer, raw prompt file and held-name seal, as described in D19.
Compare its blind tape with `held/blind.json` byte for byte. Compare
its requested spans and prefix hashes with both seed grades. The staged
panel contains `seed{1,2}/held`, `seed{1,2}/prepared`,
`held-names.json`, `reader-grader.json`, the reader/adjudication receipts
and `INPUTS.sha256`. The runner independently repeats tokenizer preparation
before scoring.
