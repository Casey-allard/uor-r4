# [codex] Trace the frozen ordinary requested role

## Sealed D25

The first lost stage is the query fence. The role state detects a candidate word whose class is 1 and takes the last recognized D11 delimiter before that word. The delimiter is token ID 16, immediately before `If the phrase cannot be determined…`, in all eleven held rows. All nine annotated occurrences appear intact in native candidate enumeration and fail `outside_query`. None of the trailing candidates has an exact earlier assertion head. The other two rows are unsupported and have no annotated gold occurrence. Requested best and runner-up both equal `i32::MIN` in all eleven sealed client records.

| Row | Annotated query | Token span | Decoded byte range | Query fence token | Supplied-query matches / complete | Values disagree | Supplied source |
|---|---|---|---|---:|---:|---|---:|
| 201 | Asha | [78,81) | [296,301) | 104 | 1/1 | no | 56 |
| 202 | Asha | [76,79) | [282,287) | 102 | 2/1 | no | — |
| 203 | Ravi | [86,89) | [273,278) | 116 | 2/2 | yes | — |
| 204 | Ravi | [87,90) | [277,282) | 117 | 2/2 | yes | — |
| 205 | Lina | [72,74) | [269,274) | 97 | 2/1 | no | — |
| 206 | Lina | [83,85) | [288,293) | 108 | 2/1 | no | — |
| 207 | Omar | [73,76) | [256,261) | 99 | 2/2 | yes | — |
| 208 | Omar | [82,85) | [268,273) | 108 | 2/1 | no | — |
| 209 | Asha | [69,72) | [227,232) | 95 | 2/1 | no | — |
| 210 | — | — | — | 61 | — | — | — |
| 211 | — | — | — | 99 | — | — | — |

Bytes are zero-based on the complete decoded protocol-2 tape, including the leading blank of each name. The nine names all consult role-cell keys `[24576,75913,142267,210019,262702,336969]`. All six keys miss the actual frozen bank, giving scores [0,0]. Keeping just the first question re-admits each of the nine at its original token coordinates. Each wins alone at contrast 0 against the uninitialized runner-up. That intervention copies an unsupported assertion on 211. It does not justify a fence repair by itself. The ordinary requested rank accepts a unique greatest signed contrast and has no minimum magnitude. The excluded rank uses minimum evidence 12 and also requires a unique lead.

All eleven unchanged native ordinary traces agree with the sealed ordinary clients on prompt IDs, complete role and binder output, generated IDs and every copy-cursor step. Their first wrong generated ID is 1155 at offset 0 on the eight directional prompts. The supplied query alone chooses the wrong onset 56 on 201 and abstains on the other seven. Completing and agreeing across all matching assertions is required for a final copy. The sealed D25 isolated-assertion source control gives the original source reader 4/8 exact, the fitted arms 0/8, and the forced correct source 8/8 with the same writer. The D25 device receipts cover source positions and geometric read parity. They do not measure a repaired query fence.

## Disjoint supported controls

Rows 501–504 admit both Rowan and Isabel with exact earlier heads at [10,13) and [19,23). The frozen role cells give zero contrasts for both names under each order and polarity, leaving each pair tied. The dual request in 505 has eligible contrast 0 for Rowan and 12 for Isabel. It releases only Isabel and cannot express both endpoints. The same person in 506–508 is selected at contrast 0. Isolated and duplicate-identical assertions release a value. Duplicate conflicting assertions each yield a completed value, disagree, and withhold.

| Row | Ordered candidate name and requested/excluded scores | Ordinary result |
|---|---|---|
| 501 | Rowan −14/−14, Isabel −6/−6 | nil, best/runner 0/0 |
| 502 | Isabel −12/−12, Rowan −18/−18 | nil, best/runner 0/0 |
| 503 | Rowan −12/−12, Isabel −18/−18 | nil, best/runner 0/0 |
| 504 | Isabel −14/−14, Rowan −6/−6 | nil, best/runner 0/0 |
| 505 | Rowan −14/−14, Isabel −8/−20 | Isabel only, source 26 |
| 506 | Rowan −14/−14 | one complete, source 14 |
| 507 | Rowan −14/−14 | both complete and equal, source 14 |
| 508 | Rowan −14/−14 | both complete and unequal, nil source |

## Reproduce

Build the standalone Cargo manifest with `cargo build --locked`. Invoke `d26-ordinary-role-trace PARENT TOKENIZER GRADE ORDINARY CLIENTS_OR_DASH` with the pinned D25 parent, tokenizer, grade and ordinary JSON. For the disjoint controls, use `PARENT TOKENIZER controls controls.tsv -`. Optional `--last-supported` enables the separate registered text-only diagnostic and omits the first-question-only oracle. This binary is a read-only client replay, with no changes to model or production source. It prints original IDs, candidate geometry and cell counts, byte offsets, assertion and writer traces, and the first mismatching output token.

The D25 local role trace is `~/uor-r4/local/codex/pre-gate-source-26/role/d25.json`. Sealed inputs are under `~/uor-r4/local/codex/contextual-source-25`. See [D25 REPORT](../../codex-25/REPORT.md) for writer and device receipts and [last supported clause](LAST-SUPPORTED.md) for the new diagnostic.

The frozen fallback comparison and its controls are reported in [D26 results](RESULT.md).
