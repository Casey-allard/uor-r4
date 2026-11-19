# [codex] D20 supported span and first-token results

Both seed artifacts agree on every supported row. Each coordinate is a zero-based, half-open span on the original framed prompt. The ordinary arms agree on their selected span and emitted tape in every row. Supplying a span puts the correct query occurrence into the original value selector. The last two columns give the emitted first token ID for wrong replies or exact completion including EOS.

| Row | Person | Connector and object | Ordinary span | Supplied span | Original gold start | Expected first ID | Ordinary outcome | Supplied outcome |
|---:|---|---|---|---|---:|---:|---|---|
| 0 | Adelaide | found red brass keys | [53,57) | [39,44) | 10 | 677 | token 0 = 1155 | token 0 = 1155 |
| 1 | Adelaide | found red brass keys | [53,57) | [39,44) | 21 | 677 | token 0 = 1155 | token 0 = 1155 |
| 2 | Bertram | might take silk scarves | [52,57) | [39,43) | 22 | 262 | token 0 = 1155 | token 0 = 1155 |
| 3 | Bertram | might take silk scarves | [52,57) | [39,43) | 10 | 262 | token 0 = 1155 | token 0 = 1155 |
| 4 | Adelaide | might take red brass keys | [53,57) | [39,44) | 11 | 677 | token 0 = 1155 | token 0 = 1155 |
| 5 | Adelaide | might take red brass keys | [53,57) | [39,44) | 21 | 677 | token 0 = 1155 | token 0 = 1155 |
| 6 | Bertram | found silk scarves | [52,57) | [39,43) | 22 | 262 | token 0 = 1155 | token 0 = 1155 |
| 7 | Bertram | found silk scarves | [52,57) | [39,43) | 9 | 262 | token 0 = 1155 | token 0 = 1155 |
| 8 | Adelaide | found silk scarves | [53,57) | [39,44) | 10 | 262 | token 0 = 1155 | token 0 = 1155 |
| 9 | Adelaide | found silk scarves | [53,57) | [39,44) | 22 | 262 | token 0 = 1155 | token 0 = 1155 |
| 10 | Bertram | might take red brass keys | [52,57) | [39,43) | 21 | 677 | token 0 = 1155 | token 0 = 1155 |
| 11 | Bertram | might take red brass keys | [52,57) | [39,43) | 10 | 677 | token 0 = 1155 | token 0 = 1155 |
| 12 | Adelaide | might take silk scarves | [53,57) | [39,44) | 11 | 262 | token 0 = 1155 | token 0 = 1155 |
| 13 | Adelaide | might take silk scarves | [53,57) | [39,44) | 22 | 262 | token 0 = 1155 | token 0 = 1155 |
| 14 | Bertram | found red brass keys | [52,57) | [39,43) | 21 | 677 | token 0 = 1155 | token 0 = 1155 |
| 15 | Bertram | found red brass keys | [52,57) | [39,43) | 9 | 677 | token 0 = 1155 | token 0 = 1155 |
| 19 | Winifred | could still find brushes | [57,61) | [45,49) | 11 | 564 | token 0 = 563 | token 0 = 1155 |
| 20 | Winifred | could still find brushes | [57,61) | [45,49) | 21 | 564 | token 0 = 563 | token 0 = 1155 |
| 21 | Caspian | took small glass jars | [57,61) | [45,49) | 20 | 563 | token 0 = 1155 | exact through EOS |
| 22 | Caspian | took small glass jars | [57,61) | [45,49) | 9 | 563 | token 0 = 1155 | exact through EOS |
| 23 | Winifred | took brushes | [57,61) | [45,49) | 9 | 564 | token 0 = 1155 | token 0 = 1155 |
| 24 | Winifred | took brushes | [57,61) | [45,49) | 21 | 564 | token 0 = 1155 | token 0 = 1155 |
| 25 | Caspian | could still find small glass jars | [57,61) | [45,49) | 20 | 563 | token 0 = 1155 | token 0 = 1155 |
| 26 | Caspian | could still find small glass jars | [57,61) | [45,49) | 11 | 563 | token 0 = 1155 | token 0 = 1155 |
| 27 | Winifred | could still find small glass jars | [57,61) | [45,49) | 11 | 563 | token 0 = 1155 | token 0 = 1155 |
| 28 | Winifred | could still find small glass jars | [57,61) | [45,49) | 20 | 563 | token 0 = 1155 | token 0 = 1155 |
| 29 | Caspian | took brushes | [57,61) | [45,49) | 21 | 564 | token 0 = 1155 | token 0 = 1155 |
| 30 | Caspian | took brushes | [57,61) | [45,49) | 9 | 564 | token 0 = 1155 | token 0 = 1155 |
| 31 | Winifred | took small glass jars | [57,61) | [45,49) | 9 | 563 | token 0 = 1155 | exact through EOS |
| 32 | Winifred | took small glass jars | [57,61) | [45,49) | 20 | 563 | token 0 = 1155 | exact through EOS |
| 33 | Caspian | could still find brushes | [57,61) | [45,49) | 21 | 564 | token 0 = 563 | token 0 = 1155 |
| 34 | Caspian | could still find brushes | [57,61) | [45,49) | 11 | 564 | token 0 = 563 | token 0 = 1155 |
