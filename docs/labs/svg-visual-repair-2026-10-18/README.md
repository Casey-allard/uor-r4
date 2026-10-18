# Native SVG composition repair — 18 October 2026

Owner-requested documentation repair after #2643, based on main
`59fb17c3` (including #2661 and #2662). **Decision: visual repair, pending
independent visual review; no capability or scientific status change.**
The PR remains a draft and must not merge before that review. The owner's
standing authorization covers subsequent protected delivery without another
confirmation. No model, training, workflow, permission or scientific gate changes.
Main `7937ce10` was merged forward, preserving #2659's source/status updates,
#2662's timing waiver and #2663's peer claim removals. The authored diff against
that main remains limited to figures, their generators, documentation and the
own coordination receipt. Its SVG identities match the initial visual review set.

## Original references and visual diagnosis

| Reference | Assets and generating source | What is retained |
|---|---|---|
| [6e857870, #2057](https://github.com/UOR-Foundation/uor-r4/commit/6e857870) | First `docs/figures/architecture.svg`, 1200×900 | Landscape lanes, shared geometry strip, fine outlines and status key |
| [11d61955, #2059](https://github.com/UOR-Foundation/uor-r4/commit/11d619557d44c623cf0a2a2cbbcb320be6f569cc) | `docs/figures/geometry/generate.py`: quaternion, S³ transport, Hopf, 600-cell, golden integers, zeta phases, prime addressing | Computed mathematical geometry, 1000×700 family, light panels, small annotations |
| [65f2eb04, #2090](https://github.com/UOR-Foundation/uor-r4/commit/65f2eb0468b50004dc0482c12e5671d5f6869e0a) | Connection map and its generator; exact icosian, signed-binding and Hamming figures | Outline schema and semantic edge colours/styles |
| [3778e1d6, #2298](https://github.com/UOR-Foundation/uor-r4/commit/3778e1d6d5f17611ab5a13929c13a4ffdf912ce9) | `xtask/src/figures.rs`, generated architecture and status figures | Rust generation, restrained typography, status legend; the architecture is unchanged at pre-#2643 `43bca5e1` |

Those commits explicitly credit Claude Opus 5.5. References provide visual
grammar, not permission to restore their historical scientific/runtime claims.

Inspection of actual Chromium pixels confirms the main regression in
[`9fd1b8ba`](https://github.com/UOR-Foundation/uor-r4/commit/9fd1b8baec6a7586a2595cd4be94f6df77fd1c66):
the architecture changed from 1200×900 lanes into a 600×1320 vertical text card.
At equal width its title scales 2× and its body about 3×; its height scales
2.93×. The real README capped that narrow source at 600px, leaving a large
desktop gutter. Empty overflow reports did not capture either composition flaw.
The mathematical drawings survived; replacing them would discard useful,
correct computed geometry. Zeta labels did have inherited collisions, including
γ₃'s “7”/“2” reading as “72”.

## Repair and representative previews

The new [architecture](../../figures/architecture.svg) is a 1200×660 landscape
schema: the shared table supplies both input encoding and the output writer,
ordered FBS plans feed a geometric read, and the learning lane compares exact
virtual moves. Held M2 research and further integration/serving obligations have
separate boundaries without speculative integration arrows. A generated
[380×810 narrow view](../../figures/architecture-narrow.svg) preserves the same
flow at readable mobile label sizes. The README, geometry guide and source map
use `<picture>` to choose it, with a native-width landscape fallback.

The mathematical figures keep their original 1000×700 canvases and all
non-text primitives. Quaternion notation now names the unit axis `n`, separately
from the quaternion components `(w,x,y,z)`. The 600-cell caption describes the
projection; historical snapping and its distinction from M1's 16-root carrier
are explained outside the image. The historical geometry inventory qualifies
its reduced-metric E8 construction. Zeta changes affect labels only, including
the shared “2,5” annotation for visually coincident γ₉ positions.

| Figure | Before | Repaired |
|---|---|---|
| Architecture, actual README, desktop | [1280px](previews/architecture-before-1280.png) | [1280px](previews/architecture-after-1280.png) |
| Architecture, actual README, narrow | [390px](previews/architecture-before-390.png) | [390px](previews/architecture-after-390.png) |
| Zeta phases, actual geometry page | [1280px](previews/zeta-phases-before-1280.png) | [1280px](previews/zeta-phases-after-1280.png) |
| Quaternion, actual geometry page | [1280px](previews/quaternion-rotation-before-1280.png) | [1280px](previews/quaternion-rotation-after-1280.png), [390px](previews/quaternion-rotation-after-390.png) |
| 600-cell, actual geometry page | [1280px](previews/600-cell-icosians-before-1280.png) | [1280px](previews/600-cell-icosians-after-1280.png), [390px](previews/600-cell-icosians-after-390.png) |
| Connections, preserved corrected map | [1280px](previews/connections-before-1280.png) | [1280px](previews/connections-after-1280.png), [390px](previews/connections-after-390.png) |

[Original Claude architecture rendered at desktop width](previews/architecture-reference-1280.png)
is a historical reference, not a proposed restoration of obsolete runtime labels.
All preview PNGs are QA captures; the documentation embeds native SVGs.

## Accuracy and execution boundary

- [`discrete.cu`](../../../crates/uor-r4-discrete/src/discrete.cu): `discrete_encode`
  performs the lookup; `route_vector` transports one carrier; `flock_vector`
  selects, equally sums transported roots and snaps; Relation adds finite
  reranking. `writer` admits product-address cells and compares integer
  distances with deterministic ties. No probability normalization is introduced.
- [`build.rs`](../../../crates/uor-r4-discrete/build.rs): 8D blocks, 16 roots,
  64 signed actions are different quantities. These are not full E8 coverage.
- [`prime_zeta.rs`](../../../crates/uor-r4-discrete/src/prime_zeta.rs): four
  complex channels per block become two R⁴ lanes, then Q30 storage and carrier
  alignment, subject to allocation controls. No C⁸-per-R⁸-block claim is restored.
- [E1/E2 checks](../../figures/geometry/verify_connections.py) keep standard-coordinate
  signed E8 actions separate from reduced-metric icosian roots. The connection
  figure retains two distinct E8 nodes and states the missing intertwiner.
- [Quaternion and Hopf prose](../../geometry.md) keeps R³ conjugation separate
  from R⁴ left action, preserves the additive-recurrence norm caveat, and names
  the float-backed offline Hopf/fibre helpers. Q30 storage is not evidence of
  integer computation. Historical softmax diagrams remain explicitly historical.

## Validation and limits

The [renderer](render.mjs) parses the actual README and geometry Markdown using
GFM, serves their real relative SVGs, and captures Chromium at 1280px and 390px.
Its CSS approximates GitHub's 1012px article with 32px desktop padding (946px
content), and 16px narrow padding. All 28 before/after captures were visually
inspected, including the original 600-cell linework, corrected zeta labels,
distinct E8 boxes, Hopf fibres, desktop spacing and narrow architecture. The
browser selects `architecture-narrow.svg` at 390px. Screenshots and bounds
checks supplement, rather than replace, that inspection.

Detailed mathematical labels are too small to read at 390px without zoom.
Their diagrams preserve the original research composition, link to native
vectors for magnification, and have descriptive alternatives plus adjacent
equations/prose. The compact architecture has a separate narrow composition;
there is no forced global font enlargement. The layout is GitHub-like, not an
authenticated screenshot of GitHub's production renderer; web font fallback
can vary slightly by operating system.

Executed checks and exact SVG identities are in [checks/](checks/): Rust 1.97.1
build and formatting; 8 current Rust figures; 15 mathematical figures unchanged
on regeneration; 29/29 finite/sampled geometry checks; the wording gate and all
17 prior rejected-claim regressions; 337 local links/anchors with no failures;
and 23 native accessible SVGs with no raster/foreign content or clipped text.
All six inspected mathematical figures have identical non-text primitive
hashes before/after, and the zeta annotation overlap list is empty. The harness
check reports zero errors and inherited/sandbox warnings; its GitHub CLI
registry cross-check is unavailable here, not counted as passed. No model
computation is run. The harness live
tier is redirected to `local/harness-live` in this isolated sandbox; it is not
a claim of presence on the owner's machine. No external labs were contacted.

Reproduce with Rust 1.97.1 and local development installs of `marked` and
`playwright-core` (or point `MARKED_MODULE` / `PLAYWRIGHT_MODULE` at an existing
installation):

```sh
cargo run -q --locked -p xtask -- figures
python3 docs/figures/geometry/generate.py
python3 docs/figures/geometry/verify_connections.py
cargo run -q --locked -p uor-r4-claim-wording -- .
node docs/labs/svg-visual-repair-2026-10-18/links.mjs
node docs/labs/svg-visual-repair-2026-10-18/render.mjs
```

**Next:** independent review of the exact PR head and these image pixels;
hold merge until that review, then use the authorized protected delivery path.
