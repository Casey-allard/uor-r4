# Attempt 3: completed candidate screen

The public-DEV instrument reproduced Claude-267 exactly: **8,243** copyable
positions, depth 1 **6,306 / 3,859**, depth 2 **4,383 / 3,506**, and
`k2>k1` **4,388**.

On the new `train,train` panel, `longest_4_to_1` was right on **815 / 1,508**
copyable positions against **807** for `k2>k1` and **790** for the random-depth
control. The paired comparison had 9 candidate-only against 1 control-only
decisions for `k2>k1`, and 32 against 7 for random depth.

On the new `dev,dev` panel, `longest_4_to_1` was right on **826 / 1,594**
copyable positions against **818** for `k2>k1` and **798** for random depth.
The paired comparison had 8 candidate-only against 0 control-only decisions for
`k2>k1`, and 33 against 5 for random depth.

Both panels exceeded 200 eligible positions and both random controls changed
copyable decisions. The fresh-process dev-panel repeat matched byte for byte.

Published output is in [`evidence-attempt-3/`](evidence-attempt-3/). The raw
manifest, preserved as `RAW-SHA256SUMS`, has digest
`bec19988e1b161973de1037f444ffc1516093f2bd33f54df4a35ee52a09c3a22`.
The published subset verifies with `PUBLISHED-SHA256SUMS`.
