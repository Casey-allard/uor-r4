# Per-prompt movement under requested-entity ranking

Offsets are zero-based, half-open intervals in the original protocol-2 token tape. “Exact” means every expected ID plus EOS was emitted. Rows with the same facts vary only the requested person, fact order, wording or exclusion placement. The six questions with no unique supported answer are graded separately.

| Row | Requested person | Baseline selected span | Treatment selected span | First error, baseline → treatment | Expected literal |
|---:|---|---|---|---|---|
| 0 | Elspeth | “Elspeth” [37,41) | “Elspeth” [37,41) | 0 → 0 | red brass keys |
| 1 | Elspeth | “Elspeth” [44,48) | “Elspeth” [44,48) | 0 → 0 | red brass keys |
| 2 | Elspeth | “Leander” [15,18) | “Leander” [44,47) | 0 → 0 | red brass keys |
| 3 | Elspeth | “Leander” [15,18) | “Elspeth” [44,48) | 0 → 0 | red brass keys |
| 4 | Elspeth | “Elspeth” [38,42) | “Elspeth” [38,42) | 0 → 0 | red brass keys |
| 5 | Elspeth | “Elspeth” [45,49) | “Elspeth” [45,49) | 0 → 0 | red brass keys |
| 6 | Elspeth | “Leander” [15,18) | “Leander” [45,48) | 0 → 0 | red brass keys |
| 7 | Elspeth | “Leander” [15,18) | “Elspeth” [45,49) | 0 → 0 | red brass keys |
| 8 | Leander | “Leander” [37,40) | “Leander” [37,40) | 0 → 0 | silk scarves |
| 9 | Leander | “Leander” [45,48) | “Leander” [45,48) | 0 → 0 | silk scarves |
| 10 | Leander | “Leander” [15,18) | “Elspeth” [43,47) | 0 → 0 | silk scarves |
| 11 | Leander | “Leander” [15,18) | “Leander” [45,48) | 0 → 0 | silk scarves |
| 12 | Leander | “Leander” [38,41) | “Leander” [38,41) | 0 → 0 | silk scarves |
| 13 | Leander | “Leander” [46,49) | “Leander” [46,49) | 0 → 0 | silk scarves |
| 14 | Leander | “Leander” [15,18) | “Elspeth” [44,48) | 0 → 0 | silk scarves |
| 15 | Leander | “Leander” [15,18) | “Leander” [46,49) | 0 → 0 | silk scarves |
| 16 | Elspeth | “Elspeth” [37,41) | “Elspeth” [37,41) | 0 → 0 | red brass keys |
| 17 | Elspeth | “Elspeth” [44,48) | “Elspeth” [44,48) | 0 → 0 | red brass keys |
| 18 | Elspeth | abstained | “Leander” [44,47) | 0 → 0 | red brass keys |
| 19 | Elspeth | “Ignoring” [25,30) | “Elspeth” [44,48) | 0 → 0 | red brass keys |
| 20 | Elspeth | “Elspeth” [38,42) | “Elspeth” [38,42) | 0 → 0 | red brass keys |
| 21 | Elspeth | “Elspeth” [45,49) | “Elspeth” [45,49) | 0 → 0 | red brass keys |
| 22 | Elspeth | abstained | “Leander” [45,48) | 0 → 0 | red brass keys |
| 23 | Elspeth | “Ignoring” [25,30) | “Elspeth” [45,49) | 0 → 0 | red brass keys |
| 24 | Leander | “Leander” [37,40) | “Leander” [37,40) | 0 → 0 | silk scarves |
| 25 | Leander | “Leander” [45,48) | “Leander” [45,48) | 0 → 0 | silk scarves |
| 26 | Leander | abstained | “Elspeth” [43,47) | 0 → 0 | silk scarves |
| 27 | Leander | “Ignoring” [25,30) | “Leander” [45,48) | 0 → 0 | silk scarves |
| 28 | Leander | “Leander” [38,41) | “Leander” [38,41) | 0 → 0 | silk scarves |
| 29 | Leander | “Leander” [46,49) | “Leander” [46,49) | 0 → 0 | silk scarves |
| 30 | Leander | abstained | “Elspeth” [44,48) | 0 → 0 | silk scarves |
| 31 | Leander | “Ignoring” [25,30) | “Leander” [46,49) | 0 → 0 | silk scarves |
| 35 | Maren | “Maren” [35,38) | “Maren” [35,38) | 0 → 0 | brushes |
| 36 | Maren | “Maren” [42,45) | “Maren” [42,45) | 0 → 0 | brushes |
| 37 | Maren | “took” [17,18) | “Ivor” [41,44) | 0 → 0 | brushes |
| 38 | Maren | “took” [17,18) | “Maren” [42,45) | 0 → 0 | brushes |
| 39 | Maren | “Maren” [36,39) | “Maren” [36,39) | 0 → 0 | brushes |
| 40 | Maren | “Maren” [43,46) | “Maren” [43,46) | 0 → 0 | brushes |
| 41 | Maren | “took” [17,18) | “Ivor” [42,45) | 0 → 0 | brushes |
| 42 | Maren | “took” [17,18) | “Maren” [43,46) | 0 → 0 | brushes |
| 43 | Ivor | “Ivor” [35,38) | “Ivor” [35,38) | exact → exact | small glass jars |
| 44 | Ivor | “Ivor” [42,45) | “Ivor” [42,45) | exact → exact | small glass jars |
| 45 | Ivor | “took” [17,18) | “Maren” [41,44) | 0 → 0 | small glass jars |
| 46 | Ivor | “took” [17,18) | “Ivor” [42,45) | 0 → exact | small glass jars |
| 47 | Ivor | “Ivor” [36,39) | “Ivor” [36,39) | exact → exact | small glass jars |
| 48 | Ivor | “Ivor” [43,46) | “Ivor” [43,46) | exact → exact | small glass jars |
| 49 | Ivor | “took” [17,18) | “Maren” [42,45) | 0 → 0 | small glass jars |
| 50 | Ivor | “took” [17,18) | “Ivor” [43,46) | 0 → exact | small glass jars |
| 51 | Maren | “Maren” [35,38) | “Maren” [35,38) | 0 → 0 | brushes |
| 52 | Maren | “Maren” [42,45) | “Maren” [42,45) | 0 → 0 | brushes |
| 53 | Maren | “took” [7,8) | “Ivor” [41,44) | 0 → 0 | brushes |
| 54 | Maren | “took” [7,8) | “Maren” [42,45) | 0 → 0 | brushes |
| 55 | Maren | “Maren” [36,39) | “Maren” [36,39) | 0 → 0 | brushes |
| 56 | Maren | “Maren” [43,46) | “Maren” [43,46) | 0 → 0 | brushes |
| 57 | Maren | “took” [7,8) | “Ivor” [42,45) | 0 → 0 | brushes |
| 58 | Maren | “took” [7,8) | “Maren” [43,46) | 0 → 0 | brushes |
| 59 | Ivor | “Ivor” [35,38) | “Ivor” [35,38) | exact → exact | small glass jars |
| 60 | Ivor | “Ivor” [42,45) | “Ivor” [42,45) | exact → exact | small glass jars |
| 61 | Ivor | “took” [7,8) | “Maren” [41,44) | 0 → 0 | small glass jars |
| 62 | Ivor | “took” [7,8) | “Ivor” [42,45) | 0 → exact | small glass jars |
| 63 | Ivor | “Ivor” [36,39) | “Ivor” [36,39) | exact → exact | small glass jars |
| 64 | Ivor | “Ivor” [43,46) | “Ivor” [43,46) | exact → exact | small glass jars |
| 65 | Ivor | “took” [7,8) | “Maren” [42,45) | 0 → 0 | small glass jars |
| 66 | Ivor | “took” [7,8) | “Ivor” [43,46) | 0 → exact | small glass jars |
