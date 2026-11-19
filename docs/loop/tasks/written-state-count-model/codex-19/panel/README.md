# Requested-occurrence holdout

The panel has 64 supported prompts and six negative controls. Each of the two
fact families crosses the requested person, written fact order, presence of
“the,” choice of “about” versus “for,” and whether the explicit exclusion comes
before or after the request. Every cell occurs once per family. Both facts are
identical across a fact-order pair, including each person's connector and
object. Both people are named in the question on every supported row.

The grades record the full query region, the intended requested occurrence
and the other occurrence in separate half-open token spans. The
`distractor_first` flag is true if and only if the last of the two query names
is correct. A first-query-name rule succeeds on the 32 supported rows where
the flag is false. A last-query-name rule succeeds on the other 32. Each
constant fact position also succeeds on 32 rows. `census.jq` checks the
crossing, fact-order invariance and the original literal and EOS token
identities for both possible outcomes on all 64 supported rows. Each family
adds an absent person, two incompatible records presented as equally credible
and an unspecified person.

Both pinned tokenizers produce the same reader-facing raw text. The grade
for each seed binds each expected reply to its original token slice followed
by EOS. The D18 shape audit reports support in the same frozen 200-record
value TRAIN bank for all legal candidate shapes on all 64 supported rows on
both seeds. This static check measures shape support. Candidate nomination and
actual writing are checked during the device run.

The novelty census checks 46,019 historical strings including D18. All 70
prompts and the four factual names are new in that census. In frozen value
TRAIN, all four factual names and all four connector phrases are absent.
Three of four full object literals are absent from frozen value TRAIN, yielding
48 new-literal rows. The other 16 request the previously seen object label
“brushes.” The held prompt, named person and competing name for those rows
are still new. Keep this split when reporting exact reply rates.

## Reproduction

Build this standalone manifest with `cargo build --offline --locked`. Invoke
`requested-occurrence-panel SEED TOKENIZER NEW_ROOT` for each seed. The output
matches the D18 held schemas and adds occurrence-factor fields to the grade.
Pass only `held/public-text.json` to independent readers. Use the D18 reader
schema `{index,person,answer,status}`, and check the readings against both
tokenizers with `adjudicate.jq`. The D18 `grader-reading.jq` transforms a
verified reader into the observed-response grader input.

Project a verified reader to just `index` and `person`. Then run
`requested-occurrence-panel seal RAW PERSON_ONLY NAMES`. It resolves only the
question occurrence. Use D17 capture's tokenizer-only `prepare` command with
the pinned tokenizer, raw prompt file and name seal. Compare the reconstructed
blind file byte-for-byte to the generator output and check all resolved token
spans and prefix hashes against `held/resolved.json`. The runtime diagnostic
takes the `prepare` command's `resolved-requested.json` file. Use the D18
`shapes.jq` and `novelty.jq` procedures with the D17 base and supplemental
TRAIN, augmenting the archived historical input list with both D18 held public
texts and grades.
