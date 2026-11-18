# [codex] Paired entity span and connector fixtures

These are input checks before fitting. The native Rust tokenizer generated the data, and the Studio ByteLevel BPE independently re-encoded all 3,456 new user prefixes, expected ID tapes and complete user/assistant documents. Both released seed tokenizers independently produced the same eight files byte-for-byte. The tokenizer-only census and Wick checksums were recorded before scoring. This procedure does not load a model.

## Matched design

Each split has 1,728 prompts. Each of the nine requested-length by distractor-length cells has 192 prompts. All 576 triplets per split preserve every prefix ID outside the requested connector: deleting that span, whose length ranges over 1, 2 and 3, yields three identical residual tapes. In each of 864 role-reversal pairs, the written facts stay identical, the requested values trade places with the distractors, and all other factors stay fixed. Even when the two facts have different connector lengths the pair is graded on both questions. Both value positions and the absolute query and fact entity starts and ends are stored separately in the grade file.

| Factor | TRAIN | Held-out |
|---|---|---|
| Single-token names | Tim / Sue | Joe / Leo |
| Split names | Nala / Navi (three tokens each) | Rana / Rani (three tokens each) |
| Values | gray / blue or small cup / small hat | yellow / pink or round stone / round coin |
| Shared connector set, length 1 / 2 / 3 | has / has kept / has just kept | same strings |
| Split-specific connector set, length 1 / 2 / 3 | found / once found / once again found | took / chose / also chose |
| Question form | requested name early after “For”, or late after “for” | early after “About”, or late between “that” and “has” |
| Exclusion | absent, “Ignore Y.” before or after question | absent, “Set Y aside.” before or after question |
| Trailer | empty or “Give only the requested item.” | empty or “Write just the name of the item here.” |

Lengths count tokenizer IDs. **chose** occupies two IDs inside one word. The common connector set makes a short trained connector an exact prefix of both longer trained connectors. Each held-out name and each entire held-out value phrase is unseen in the training fixture. On the held-out split the names Rana and Rani share their first two IDs (1049, 304); the terminal ID is 67 for Rana and 75 for Rani. The independent lexical check includes six unique connector token sequences per split, with three in the intersection; 864 held-out prompts use shared connector surfaces and 864 use split-specific surfaces.

For each name family, both assignments of values, both orders of facts, two question positions, three exclusion conditions, two tails, both requested identities and the two written connectors' lengths were crossed. Value assignment determines shared vs split-specific connector vocabulary; the XOR of family and fact order determines the single-token vs shared-prefix value set. These couplings and the exact words used to vary connector length limit how one can interpret a vocabulary contrast. The study tests transfer to the chosen values and connectors; the number of tokens alone does not determine item difficulty.

## Interfaces and preservation

Absolute entity starts and ends are **inclusive**; the value spans retain #4087's **half-open** convention. TRAIN has exactly prefix, query_start, query_end, fact_start, fact_end and source. The blind arrays contain only prefix. The returned choices join by hashed token IDs. The new test data are held-blind.json and held-grade.json, the historical 288 are prior-blind.json and prior-grade.json, and the original sixteen are original-blind.json and original-grade.json. The full prompt tapes, IDs, roles, pairs, two value ranges and expected answers in both historical grade arrays match #4087. Their blind inputs are byte-identical to #4087. All 3,760 prompt tapes across train, held, prior and original are distinct.

The Rust generator validated the unique requested and distractor values, all name mention counts, excluded mention order and full dialogue boundaries during construction. The independent Studio check verified the role pairs, triplet invariant, 9-cell census, strict blindness, exact token targets, inclusive name occurrences and joins to both historical fixtures. Connector transfer describes the finite strings here; geometric closeness of value roots remains a separate descriptive measurement.

## Reproduction

Use the two #4087 release tokenizers separately and an unused output directory each time. PREVIOUS contains the original and held blind/grade JSON from the #4087 bundle. The two original tokenizer SHA-256 values are a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377 and f804ecbee34e3c4e0001a737d003e3d620ba2c40722d416e7f3dd64843e2b30e. The generator SHA-256 at the pre-fit freeze is 5ee5f84c607ff8d256b1a08a7ce6e160c5b41e01b12f7c6c38e106caf2907c76.

```sh
~/.cargo/bin/cargo run --locked -p uor-r4-tokenizer --example span-connector-panel -- "$TOKENIZER" "$PREVIOUS" "$NEW_OUT"
```

[Input digests](PANEL-SHA256SUMS) · [Census, name IDs, connector IDs and exact ranges](PANEL-CENSUS.json)
