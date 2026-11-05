# CPU copy-key depth-age router

This record contains one completed registered run. The discrete depth-and-age
router was fitted on new `train,train` seed 2601 and tested on new `dev,dev`
seed 2602 under deepseek-151.

It was right on **913 / 1,655** copyable decision positions, eight below
longest-first at **921 / 1,655**, so the pre-registered three-control bar
failed. The router beat `k2>k1` by 913 to 912 and random depth by 913 to 876,
but all three comparisons were required. The decision is **REJECT** for this
router; longest-first remains the exact-copy candidate selected by deepseek-149.

Public M1 DEV reproduced the existing instrument exactly. Both new panels
exceeded 200 eligible positions, the random control changed 79 copyable
decisions, and the fresh-process repeat was byte-identical.

Published evidence is in [`evidence/`](evidence/). The raw sealed manifest,
preserved as `RAW-SHA256SUMS`, has digest
`8e9f04a86fbbe8c0846a6bd4f24f94d176eb43ade80744be4bd605ccd9894d27`.
Generated tokens and masks are omitted from Git; the committed panel manifests
preserve their hashes and generator seeds. The published subset verifies with
`PUBLISHED-SHA256SUMS`.
