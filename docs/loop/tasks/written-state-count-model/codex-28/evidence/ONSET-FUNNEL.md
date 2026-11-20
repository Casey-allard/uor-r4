# D28 matched gold onset and rival veto

The independent reducer validated all six grades against the 35 sealed prompts. Each arm contained 206 row/lane records and an accounted visit ledger. The supported denominator is 33 including the duplicate-agreement control. The address analysis below uses the 32 matched positives, one supplied original assertion per prompt, crossed by object inside eight lexical families. The first original gold token was 1–3 pieces after the head on 16 short rows and 4–8 pieces after the head on 16 long rows. The visit ledgers, input packets and expected device traces from grader replay were byte-identical to the corresponding six staged device inputs and host traces.

## Gold candidate progression

Each row is out of 16. Eligible includes gold candidates actually emitted by the selector trace. Supported requires both exact shape/signature classes and a finite radius. Within radius requires scorer admission at or below that radius. Positive requires a strictly positive normalized signed margin. A sole unseen-rival veto means the gold was the uniquely highest resolved candidate and another candidate triggered the rule that withholds the assertion on any unseen candidate. An unseen rival has no resolved margin; its hypothetical margin is unmeasured.

| Bank and enumeration | Connector | Eligible | Supported | Within radius | Positive | Sole unseen-rival veto | Unseen veto and resolved rival matches or exceeds gold | Gold selected |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| Frozen legacy | Short | 16 | 5 | 4 | 1 | 0 | 0 | 1 |
| Frozen legacy | Long | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Frozen whole | Short | 16 | 5 | 4 | 1 | 0 | 0 | 1 |
| Frozen whole | Long | 16 | 15 | 12 | 4 | 2 | 1 | 1 |
| Fitted legacy | Short | 16 | 4 | 4 | 4 | 4 | 0 | 0 |
| Fitted legacy | Long | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Fitted whole | Short | 16 | 4 | 4 | 4 | 4 | 0 | 0 |
| Fitted whole | Long | 16 | 7 | 6 | 6 | 6 | 0 | 0 |
| Augmented legacy | Short | 16 | 11 | 6 | 3 | 0 | 0 | 3 |
| Augmented legacy | Long | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Augmented whole | Short | 16 | 11 | 6 | 3 | 2 | 0 | 1 |
| Augmented whole | Long | 16 | 15 | 12 | 4 | 2 | 1 | 1 |

The long frozen and augmented gold failures before a positive margin divide into one missing both classes, three outside positive radius and eight nonpositive. The long fitted gold failures divide into eight missing positive support, one missing negative support and one outside positive radius. All short fitted positive golds encountered a rival missing positive support. Across the six long fitted rows with a sole unseen-rival veto, seven rival visits lacked positive support and eleven were outside positive radius. The two corresponding long frozen rows had two rival visits missing both classes and one missing the negative class. The two long augmented rows had two rival visits missing both classes. Each whole frozen and augmented arm also vetoed row 28211, where another resolved candidate already matched or exceeded the gold margin.

## Paired ordinary writer outcomes

| Arm | Exact short / 16 | Exact long / 16 | Exact all supported / 33 | Wrong original source / 33 | Unsupported source release / 2 |
|---|---:|---:|---:|---:|---:|
| Frozen legacy | 1 | 0 | 1 | 4 | 0 |
| Frozen whole | 1 | 1 | 2 | 4 | 0 |
| Fitted legacy | 0 | 0 | 0 | 0 | 0 |
| Fitted whole | 0 | 0 | 0 | 0 | 0 |
| Augmented legacy | 3 | 0 | 3 | 4 | 0 |
| Augmented whole | 1 | 1 | 2 | 4 | 0 |

Frozen whole gained one exact long row and introduced a wrong original source on another long row. The augmented legacy arm gained short rows 28208–28209 with no added wrong source relative to frozen legacy. Whole enumeration on that augmented bank gained the same one exact long row, lost the two newfound exact short rows to unseen rivals, and introduced the same long wrong source. Supplying the requested person and then supplying the assertions changed no exact counts or paired exact outcomes. The only missing ordinary query endpoint was the absent-person control, row 28282, in all six arms. Forced original gold completed all 33 supported tapes and the opposed control missed all 33 in every arm. Disabled copying reproduced none of the 33 gold tapes. The agreement row failed normal release and the two unsupported controls released no source.

- **28203, Beatrice and “grass mats.”** Gold source 25 sits seven actual connector pieces beyond the factual head. Frozen whole and augmented whole both completed exact IDs `[1241,1472,85,1]`. Legacy fallback first emitted 1155 instead of 1241 at offset zero.
- **28206, Roman and “brown envelopes.”** With fitted whole, the gold onset at 22 had distances 40 and 96, radius 76 and positive margin 56/4. Source 19 lacked positive support and sources 20, 21 and 23 were outside their positive radii. The assertion withheld the source. With frozen whole and augmented whole, source 23 won and emitted `[3464,529,382,1]` instead of `[1730,3464,529,382,1]`. The first mismatch at offset zero drops “brown.”
- **28208 and 28209, Phoebe.** Old enumeration with augmented evidence released both “round dishes” and “soup bowls” exactly. Extended enumeration admitted an additional unseen rival missing both classes on each row and withheld both otherwise uniquely positive golds.

The verified report SHA-256 is `be8c19bf9b99fe4f1ee56ff00aa8dc1ed39ca56fd9ff7fbf321b9158492713f4`. Replay the tables with the [paired reducer](../analysis/README.md) and [gold funnel](../analysis/gold-funnel.jq). The rows inside a family are paired perturbations. One successful long family does not establish transfer across all eight families.
