# D27 assertion retrieval and writer replay

The fixed BPE orbit cue failed every lexical transfer row. Exact and geometric
retrieval both returned an empty set on all 32 transfer prompts, with or
without supplied person and query-relation spans. Each family had zero
correct two-way assertion flips across its eight matched pairs.

Twenty-four transfer rows were ineligible for comparison because the
supporting assertion cue and question cue had different BPE lengths. On the
eight equal-length gathers/collects rows, the cue had neither an exact hit
nor an admitted geometric distance. This accounts for the primary transfer
negative at the assertion stage, before release or writing.

| Assertion / query cue | Descriptive match | BPE pieces | Correct held head sets, geometry |
| --- | --- | ---: | ---: |
| colors / paints | Weaker | 2 / 1 | 0/8 |
| mends / repairs | Stronger | 2 / 3 | 0/8 |
| gathers / collects | Stronger | 2 / 2 | 0/8 |
| indexes / catalogs | Weaker | 4 / 3 | 0/8 |

The GPU selector replay verified all 276 records (92 prompts × 3 input lanes)
in each of the primary and cold runs. The result SHA-256 was
`d73912c8f121b0e3b68bfe3198f0382671c53006edaf98a53d6e1f723c994312`
for both. The ordered and recurrence parents staged byte-identical selector
inputs and Rust oracle records. Device verification covers the cue selector
on those staged spans. The deployed client captures and writer replay
measure the remaining gates.

Ordinary requested-person spans matched on 92/92 prompts. The exact and
geometric selectors each recovered 32/32 overlap head sets, including every
bare and instructed variant, and 28/28 control sets. In each of the two
overlap families, both arms flipped to the correct original assertion on
both sides of all eight pairs. The assertions sharing a compatible or
conflicting cue both reached the next gate. Each selector result was the
same across ordinary text, supplied person and supplied person plus
query-relation span.

## Ordinary client and frozen reader

In the primary ordered client, filtering changed original assertion
selection without improving any reply. The ordered unfiltered and exact
arms completed zero value reads across 92 prompts. Row 2703 illustrates
the failed gate. The ordinary exact arm located Owen's collecting assertion
and admitted one supporting fact. The value reader completed zero values
and released no source. The reply was a generic acknowledgement.

The secondary recurrence client released some exact values once the
selected assertion heads reached the same agreement gate. Its ordinary
exact arm produced 12/86 exact tapes through EOS versus 0/86 unfiltered.
The gain was 10 overlap replies and two controls. Transfer remained 0/32.
The exact arm copied an incorrect original address on ten supported rows,
including suffix-only `boxes`, `swings`, and `lamps`. Unfiltered recurrence
copied two incorrect supported addresses. This exposes residual value
selection errors after assertion routing.

Unfiltered recurrence emitted unsupported source copies on rows 2725, 2762,
2799, and 2836. The first two replies were ` boxes` and the last two were
` porch swings`. Exact filtering prevented all four writes.

The ordered writer grade found a strict head-set gain from 4/92 unfiltered
to 60/92 with either selector. The ordered oracle supplied all 92 correct
assertion sets and still released zero correct values. The disabled and
all ordinary and supplied ordered arms were also 0/86 on full tapes. Both
forced-address controls separated cleanly. Forcing the gold address copied
80/80 exact values through EOS, while forcing an opposing original address
matched 0/80 gold tapes. For row 2703 the ordinary exact and assertion-oracle
paths both first differed at offset zero, emitting ID `1155` in place of
`581`.

| Frozen bank and input lane | Head sets / 92 | Correct value starts / 80 | Exact tapes / 86 | EOS / 92 | Unsupported source writes / 12 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ordered, ordinary unfiltered | 4 | 0 | 0 | 87 | 0 |
| Ordered, ordinary exact or geometry | 60 | 0 | 0 | 87 | 0 |
| Ordered, supplied person + query cue, exact or geometry | 60 | 0 | 0 | 87 | 0 |
| Ordered, assertion oracle | 92 | 0 | 0 | 87 | 0 |
| Recurrence, ordinary unfiltered | 4 | 0 | 0 | 87 | 4 |
| Recurrence, ordinary exact or geometry | 60 | 12 | 12 | 89 | 0 |
| Recurrence, supplied person + query cue, exact or geometry | 60 | 12 | 12 | 89 | 0 |
| Recurrence, assertion oracle | 92 | 22 | 22 | 89 | 0 |

Supplying the person alone also left the scores unchanged for each matching
ordinary arm. Both recurrence selector arms gave five exact overlap replies
on bare prompts and five on instructed prompts. Both misleading-value
successes came from the same underlying fact pair, once with each tail.
No matched pair in either family had both recurrence replies exact. Supplying
the true assertion set added ten recurrence exact transfer replies, five per
tail. The recurrence oracle still missed 58 of 80 supported value starts.
The recurrence forced controls reproduced the same 80 gold and zero opposing
exact tapes. On recurrence row 2711 the selected suffix started at original
address 38 and emitted `2527` instead of `581` at offset zero.

All 12 unsupported controls reached EOS per ordinary arm in both carriers.
The exact and geometry arms emitted no unsupported object. Their fallback
texts were generic acknowledgements or self-description. The six
unsupported prompts asking for literal refusal produced zero exact refusals
in every ordinary, supplied, oracle and disabled arm. The six bare
unsupported prompts were graded on source withholding, with no literal
target tape.

The primary ordinary reply bar failed. The exact arm gained correct original
heads and passed both two-way overlap family checks, but the primary ordered
bank had no reply gain. Recurrence provides a bounded downstream
improvement on this frame, with residual wrong addresses and no complete
two-way reply pairs. The geometry arm had the same decisions as the exact
arm and missed the transfer bar.

The ordinary records came from fresh protocol-2 text requests to the deployed
integer client. The grade checked original prompt IDs, full emitted IDs,
every copy cursor record, the paired decision, terminal stop and artifact
pins against the replay. `writer/summary.jq` groups the checked tapes by
stratum, instruction, lane, arm, matched pair and transfer cue. All four
independent held cells share one question frame. Their eight pairs each
measure within-family perturbations.
