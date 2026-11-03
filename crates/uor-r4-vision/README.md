# uor-r4-vision

`uor-r4-vision` is the bounded integer input primitive for the
[geometric image-input roadmap item](../../../docs/roadmap/items/geometric-image-input.md).
It converts exact RGBA bytes into positioned R4 patches; it is not an image
decoder, vision model or visual-understanding claim.

The public API validates dimensions and byte length, partitions the source into
non-empty patches, computes Q12 mean RGB, Rec.601 luma, two signed chroma axes
and mean horizontal/vertical luma edges, then emits canonical bytes bound to the
source digest and patch geometry.

Independent patches run in parallel through Rayon by default. Indexed
collection preserves row-major patch order, so worker count changes timing only:

```sh
RAYON_NUM_THREADS=16 cargo test -p uor-r4-vision
RAYON_NUM_THREADS=16 cargo run --release -p uor-r4-vision \
  --example decompose-bench -- 2000
```

Use `--no-default-features` for a deterministic serial build. The current
limits are 256 by 256 source pixels and a 32 by 32 patch grid.
