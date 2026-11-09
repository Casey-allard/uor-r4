# claude-339 early path: served artifacts and panel reports

[claude] Written by `early-commit-339.sh` as each checkpoint is exported. The owner asked for these while the
runs train (9 Nov 2026). Steps below 16,384 are trend points, not the registered read. The registered read of
step-016384 is `read-339.sh serve` after the seals, with its own pinned scorer.

The exports use the existing laptop snapshot path (`hamming-base-copy`, then `cpu-fbs-export`, then the pod's
`hamming_ask` spliced in). The `-C` artifact is `fbs-toks ARTIFACT --drop-hamming-carry --out NEW_FILE` at
`402b4091eb7f7e7ea174f96ed0bbd06cf58fa78b`. Scorer: `served-reply-panel --engine integer` at the same commit, on
`m2-fresh256` (256 rows, 128 pairs). "L-a" has no carry table, so it has no `-C` artifact.

| cell | step | artifact | asked first line | other first line | complete rows | complete-distinct pairs | EOS | BOS | cap | context | collapse | distinct replies | most common reply |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| L-a-s1 | step-016384 | carried | 2 | 7 | 0 | 0 | 6 | 0 | 250 | 0 | 4 | 38 | 172 |
| L-a-s2 | step-016384 | carried | 0 | 0 | 0 | 0 | 0 | 0 | 256 | 0 | 0 | 97 | 31 |
| L-aC-s1 | step-008192 | carried | 0 | 0 | 0 | 0 | 18 | 0 | 238 | 0 | 14 | 137 | 14 |
| L-aC-s1 | step-008192 | exact -C | 0 | 0 | 0 | 0 | 39 | 0 | 217 | 0 | 25 | 135 | 25 |
| L-aC-s1 | step-012288 | carried | 0 | 4 | 0 | 0 | 0 | 0 | 256 | 0 | 0 | 44 | 104 |
| L-aC-s1 | step-012288 | exact -C | 0 | 4 | 0 | 0 | 2 | 0 | 254 | 0 | 2 | 43 | 95 |
| L-aC-s1 | step-016384 | carried | 0 | 3 | 0 | 0 | 2 | 0 | 254 | 0 | 1 | 60 | 77 |
| L-aC-s1 | step-016384 | exact -C | 0 | 4 | 0 | 0 | 4 | 0 | 252 | 0 | 2 | 58 | 89 |
| L-aC-s2 | step-016384 | carried | 2 | 3 | 0 | 0 | 0 | 0 | 256 | 0 | 0 | 46 | 143 |
| L-aC-s2 | step-016384 | exact -C | 2 | 3 | 0 | 0 | 0 | 0 | 256 | 0 | 0 | 41 | 147 |

## Files and hashes

### L-a-s1 step-016384

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-a-s1/step-016384/` (fetch with `scripts/pod/uor-store get`).

```text
7b817369dbe0420a5dc409cdef84ce6598cafacae942b81931e2f7a88ebbed13 served.uorlut
7318f4a2867110b0db413e36e8f583103091d18552102fa5ecab4086a9a4d8c7 score.json
206737ebf33d145cf64af91f58b0d7a44e321504042f45ff5e240bb8c23bcc77 pod.snapshot.json
6b36db05497e6cfa3cc0987fcd94533f8f6d0d29c4596d4689d66f225c26c684 served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```

### L-a-s2 step-016384

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-a-s2/step-016384/` (fetch with `scripts/pod/uor-store get`).

```text
df3c8f60999fe1161d02f7931d1c12b4a0a236ff32c4f8cc187506640277feb1 served.uorlut
f2b5dcc065cd5ea91e382c5064bd2de1e9069f5158b1af5e8a8ca608654bd9b3 score.json
208299672e43354532c7e630386250dd2661e5b9679070b410fa04eaaf708a96 pod.snapshot.json
cd0b7958c1c12fc6e77b30ffb1f704603536c4307c8c3ee727e885d7b51f8035 served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```

### L-aC-s1 step-008192

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-aC-s1/step-008192/` (fetch with `scripts/pod/uor-store get`).

```text
d94669b12a0d5740fbe7b6a29692242653c4e8e1cf24e11dbaf57d6b3df7ec98 served.uorlut
b0d02801af4c25ab8b8b8c0fd5eb6254b93ee5e921e9d04275bdfb17acf693e2 served-minus-C.uorlut
5ddd94db16e7b0d265b086a9eef0a57b61098ad2d6a17f55b850d13c66587f47 score.json
db558bbcf8eb92d20a304c6f760564dbe53441b2be6df8f3fe7b31b5c66f6551 score-minus-C.json
d6b03418c1feba015ad90b8188e8e17a9a2fa3fdfc634fcebcb487d0a08c8458 pod.snapshot.json
01e532cdd9d025d71323e511a8650d89aa758ecc9caa4f8019f51d1253bba28b served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```

### L-aC-s1 step-012288

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-aC-s1/step-012288/` (fetch with `scripts/pod/uor-store get`).

```text
eae9838150790cc4cfe35b46dcab6d002ece1de3154cf4266abfcd83dc7ccb98 served.uorlut
d88a6f6eea24e42e6973bcce51e810115b8c8e0f7ae131115984eedb3818862c served-minus-C.uorlut
0f010a9dbf208386b9a1c3de785ffabdfad52e99618ce29540c2ff0904ae16ce score.json
1184810cd13377db25e461380fb27ddfe1e891ad496f4f22b4b1ccafc3088aae score-minus-C.json
dc48e39511dc937d1066fecbb2f843630fa9e88a0096fb54d7cb4f3631e71c3e pod.snapshot.json
8f65173c078c27681db31bb947eb4cf195923977ba96becf2af3f30b3e82399c served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```

### L-aC-s1 step-016384

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-aC-s1/step-016384/` (fetch with `scripts/pod/uor-store get`).

```text
ba221c27f846fc171856280135964e6a66a7a4ff5d7733b763570fa1d196b6e7 served.uorlut
6e85e4e201e09771b56aa54d0ed680120ed00fbb082caaf7a8459681296da5e6 served-minus-C.uorlut
f3a955ddbb1cf98e6d58a3987544e9fbf30f1d453151ed158053ac82a8748a1d score.json
24f0411424348440c60d60a35d48a0b39fbba8681829853395b44bb3347594de score-minus-C.json
17e908270e6e5d3b64e07975c312f5c306c7bd64a0b2dfbc8797b504ef080e57 pod.snapshot.json
e4ae9c820f54466ae5a81572a0841cb739c381328e8c203daa2783731a457f85 served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```

### L-aC-s2 step-016384

Store: `results/claude/one-model-end-to-end-language-carry-2026-11-09/early/L-aC-s2/step-016384/` (fetch with `scripts/pod/uor-store get`).

```text
2741d9383a80c300b90ca2d5cbc1e06a9b664d913971f84ddbc2916a1625a430 served.uorlut
391392f5c47af2cd6a1e091417dcc04c5340080474ebf6dad5d10ca2cfd277e5 served-minus-C.uorlut
33c1bcbc8d4585608f2e1a6a9518b14683cf58f42e501f3e21e97e875dc0b787 score.json
a7d8376bd0567e438788221c2719c0c8b5c797d72f9cfce5b393b9d0e9b7c380 score-minus-C.json
f0c11bf41d762e09694813d25722df10b09eb65c13fd16bdd6f6134db0be6f94 pod.snapshot.json
f090e8036060f5472635a7972d57f4ffd3e3a524118f305193cf70190fb5c4de served.snapshot.json
scorer served-reply-panel at main, b4aa8d85b72f17619234dee1a34b8949f4860bee53210c0e8823e24f63982b43
```
