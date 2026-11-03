# REPORT: geometric-image-input, deepseek-3

Lab: deepseek. Item:
[geometric-image-input](../../../../roadmap/items/geometric-image-input.md)
(#3619). Order: [ORDER.md](ORDER.md). Base:
`55432f5f88e6c8abd349df88d4944c19bb0c7d23`.

## What did not work, first

- The first one-pixel PNG fixture was not decodable by Chrome. The browser test
  now uses a generated two-pixel RGBA PNG and fails on decode instead of
  treating an invalid fixture as an application fault.
- Direct CDP file assignment did not fire the page controller in the installed
  browser. The real-browser test now constructs a real browser `File`, assigns
  the picker's files and dispatches the same change event a picker emits.
- Making the whole composer send path asynchronous broke the existing typed
  generation adapter. Image preparation is now a deferred content promise that
  only API streaming resolves; ordinary text and local sends remain synchronous.
- The first exact-head adversarial review found that a raster filename could
  bypass an explicit non-raster MIME type. The first type check accepted all
  `image/*` MIME values and let the filename extension override a conflicting
  MIME value. Type detection now fails closed through an explicit raster MIME
  allowlist, with extension fallback only when MIME is absent.
- The same review found that an image-only turn persisted empty content and the
  next API request after reload included an invalid blank user message. History
  reconstruction now substitutes the bounded marker `[Image patch input]` for
  a historical empty user turn. The first repair incorrectly applied that
  marker to the current turn too, causing a text part in the live image-only
  request. A second exact-head review caught it. The final rule leaves the
  final turn empty and replaces only earlier empty user turns.
- The first Studio test run after merging current `main` exited 1 on an
  unhandled teardown exception from the unchanged `Inspector.tsx` after all 266
  tests had passed. An isolated immediate rerun exited 0. The transient output
  is retained in the evidence rather than reported as a clean first run.
- Strict Clippy over `uor-r4-integer` still exits 101 on 51 pre-existing
  diagnostics. None names a changed file.

## What changed

Added `image_input.js`: raster type checks, 8 MiB per-image and 16 MiB total
bounds, exact-dimensional Canvas decode/downsample to at most 256 by 256, exact
RGBA extraction and a bounded preview. Studio attachments now distinguish text
from images, show image thumbnails/name/dimensions/size/removal, and retain only
text in localStorage. Session metadata asks for reattachment after reload; no
image bytes are restored.

Native Send resolves each image only after the explicit action, calls the
authenticated decomposition route, and attaches a canonical `uor_r4_image`.
Custom providers receive the bounded preview as ordinary `image_url`. Image-only
Send is supported for API providers, while local WASM explains that it ignores
image context. The required label
`R4 patch input - experimental; not visual understanding` appears in the
composer and context inspector.

The server now renders exact signed Q12 R4 bytes as compact base64 patch data
instead of verbose decimal lines. One 1 by 1 patch fits the 64-position
development context, while four images overflow it and return 413.

## Measurements

- 17 focused Node image/API/composer tests passed.
- 266 Studio tests passed across 31 files.
- Native loopback: 14 passed. Integer library: 451 passed, 1 ignored.
- Vision crate: 9 tests passed in the default parallel build.
- Real Chrome 157 against native `uor-chat`: exact metrics were
  **34 prompt / 6 completion / 40 total tokens**, 470.3 tok/s, TTFT 10.953 ms,
  12.758 ms elapsed, stop. Selecting the image produced no generation. The
  label matched exactly, reload retained no image bytes and showed the
  reattachment notice, reattachment restored the preview, and removal cleared
  it.
- After the review repair, the real-browser flow performed an image-only Send
  with no text part, reloaded, reattached the removed image and sent again.
  Both generations completed. The retained final reading was
  **31 prompt / 6 completion / 37 total tokens**, 534.9 tok/s, TTFT 9.472 ms
  and 11.217 ms elapsed. The verifier inspected the actual request bodies: the
  live final turn contained exactly one `uor_r4_image` part and no text part,
  while the reloaded request contained the historical
  `[Image patch input]` marker followed by another image-only final turn.
- The review repair was reverified with 14 focused Node tests, all 266 Studio
  tests and the static/load/worker/browser checks. Raw output is in
  [review-repair.txt](evidence/review-repair.txt).
- Static page checks, zero-external runtime trap and WASM worker checks passed.
  `npm run typecheck` and `npm run build` passed.

## Checks run at the head

- The evidence files under [evidence/](evidence/) retain the Node, Studio,
  native, WASM, page and browser outputs plus their SHA-256 manifest.
- `rustfmt --check` on changed Rust files, `node --check` on changed JavaScript,
  and `git diff --check`: passed.
- Strict Clippy on `uor-r4-vision`: passed. Integer Clippy has the 51
  pre-existing diagnostics reproduced on base and no changed-file diagnostic.
- `uor-harness loop gates --full`: all 11 gates passed on the first attempt.

## Skills used

No available Codex skill applied. L-019 was considered; this unit validates and
prepares browser input and adds no CPU model fallback.

## Still open

Visual understanding, semantic image answers, useful multimodal conversation,
M4 and M7 remain open. The image route makes R4 data available as experimental
context; it does not train the model to interpret that context.

## Plain summary

The Studio can now paste, drop or select an image, show it, remove it, and send
its deterministic R4 decomposition through the native local chat path only on
explicit Send. The browser keeps image bytes in memory, custom providers receive
ordinary image URLs, and the complete native/WASM/browser path is verified.
Conflicting MIME/extension inputs fail closed, and an image-only conversation
can be reloaded, reattached and continued without sending an invalid blank
historical turn.
