# #741 — release: what shipping a product looks like (CLI vs WASM frontend, bundle distribution) — deferred

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-16T20:23:53Z
- **Updated:** 2026-08-19T01:58:21Z
- **Closed:** 2026-08-19T01:58:21Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/741
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Casey (2026-08-16): we are not close to wanting a shipping/distribution
pipeline yet. That work is real and will happen, but only after the
transformerless engines are actually verified to produce coherent, correct
text generation — right now that verification work (see #653, prioritized
ahead of this) is the real blocker, not packaging.

This issue exists so the *question* of what shipping a product looks like
doesn't get lost or reinvented ad hoc later — it stays open, unworked,
until the team is ready to pick it up.

# Problem

#655's own decomposition named a "release-bundle manifest + shared startup
loader" (C1) and "produce + package the pinned serving bundle... shipped
with the release" (D), but an audit before starting D found this repo has
*no* release/packaging infrastructure at all: no GitHub Releases workflow,
no Dockerfile, no git-lfs, no versioning/tagging convention. Distributing a
compiled model bundle (tens to low-hundreds of MB, depending on which
pinned model size ships) is a real product/infra decision with trade-offs
(repo size if git-committed; a release workflow and hosting if shipped as a
GitHub Release asset; a build-time network fetch, which cuts against this
project's stated local-only/no-network runtime contract, if fetched from an
external URL at install time) — not something to default into silently.

# Open questions to resolve when this is picked up

- **What ships, exactly?** The CLI (`r4` binary + `ask`/`chat`), the HTTP
  server (`r4 serve` + the OpenAI-compatible API), a WASM frontend, or some
  combination? Casey (2026-08-16): "we will need to revisit what a release
  looks like... if it will come with the cli or wasm frontend or both or
  what" — explicitly undecided, to be revisited after the model-quality
  work lands.
- **How does a compiled model bundle reach a fresh install?** Options
  surfaced during #655-D's orientation: git-committed (repo-size/LFS
  concern), a GitHub Release binary asset (needs a release workflow +
  versioning convention that doesn't exist yet), or a build/install-time
  fetch from a pinned external host (adds a network dependency, in tension
  with `docs/transformerless/LOCAL_ONLY.md`'s local-only contract).
- **Which pinned model size(s) ship by default**, if any? (The three
  currently-pinned SmolLM2 sizes referenced throughout `src/server.rs`
  suggest 135M is the natural "fits in a release" candidate, but this
  hasn't been decided as a product choice.)
- **Versioning convention** for the CLI/server/WASM build itself, separate
  from any model-bundle versioning (`ReleaseBundleManifest`'s `abi`/
  `contract_*` fields already version the wire format; nothing yet
  versions "the product" a user installs).
- Relationship to #655's still-open C1 (shared startup loader) and D
  (bundle packaging) — this issue is the product-level "what and how do we
  ship" question that those engineering slices depend on having an answer
  to before they can be finished, not a replacement for either.

# Non-goals (for now)

- Do not start any packaging/release-workflow implementation from this
  issue alone — it is a scoping/tracking placeholder, explicitly deferred
  until the team is ready, per Casey's direction.
- Does not block or gate #653 or the model-quality/coherent-generation
  work, which is intentionally being prioritized first.

# Status

Open, intentionally unworked. Revisit after #653 (and #655's remaining
engineering slices, C1/E/F) establish a verified-working transformerless
model.


## Comments (7)

### comment 5316909437 — 2026-08-17T14:00:07Z — Casey-allard

Re-reviewed this issue in full, including the explicit direction in it: "we are not close to wanting a shipping/distribution pipeline yet," and the non-goal "Do not start any packaging/release-workflow implementation from this issue alone — it is a scoping/tracking placeholder, explicitly deferred until the team is ready."

That direction is unambiguous and still the right call — implementing a packaging/distribution decision here would override an explicit maintainer instruction recorded in the issue itself, not resolve an open question I have standing to decide. Confirming the deferral holds rather than silently skipping this issue, so it's clear it was checked, not missed.

The issue's own bar for revisiting is "#653 (and #655's remaining engineering slices, C1/E/F) establish a verified-working transformerless model." This session landed real progress on both #653 (phase 1 vendor-in already in, phase 2 integration matrix just landed in #765) and the model-coherence track (#745/#755/#758/#759/#762), but neither reaches "verified-working transformerless model" — #762's own measurement still shows a degenerate/word-salad bundle at baseline, sampling only fixes exact-repeat collapse, not output quality. So the bar this issue set for picking itself back up isn't met yet. Leaving open, unworked, exactly as scoped.


### comment 5329925429 — 2026-08-18T14:52:26Z — Casey-allard

## Baseline audit (2026-08-18) — status check

Still relevant, still maintainer-deferred; nothing here proposes starting it. Two updates from today's full-repo audit at `aea30bae` (`docs/project_baseline_audit_2026_08_18.md`, pending docs-only reconciliation diff):

1. **The packaging half this issue would build on is verified in place**: D1 `package_release_bundle` + D2 `r4 package-release-bundle` + D3 golden round-trip, and C1c's sidecar verifier is live on the server's bundle-resolution path, surfacing as `verified` on `/v1/models` + `/uor/v1/status` (C1d) — advisory-only by design, never consulted by a load/serve decision. The distribution question this issue owns (CLI vs WASM frontend, how bytes reach a fresh install) remains untouched, as intended.
2. **The quality gate this issue was deferred behind now has a concrete decider**: #783 (recompile the canonical bundles with the #755 fix + fresh #758 quality read). Separately, one small defect touches any future distribution story: `uor-r4-cli`'s 1.7B menu option pins `HF_REV="main"`, which the downloader refuses by design — item 1 of #790.


### comment 5335803275 — 2026-08-19T00:06:19Z — Casey-allard

## Decision brief (prep only — nothing here starts work; the 2026-08-16 quality-first framing stands)

Purpose: when the quality gate clears, the shipping decision should be a ten-minute read, not an archaeology dig. Facts current at `main` 2026-08-19.

### What exists now that didn't when this was filed

- **Packaging + verification are done** (#655-D0–D3, C1c/C1d): `r4 package-release-bundle --compiled <dir> --capability … --model-id r4` emits the `release-bundle.json` sidecar; the loader verifies declared digests against the bundle on every load and surfaces `verified` on `/v1/models` + `/uor/v1/status`. The packaging default model-id is already `r4` (F-ready).
- **Serving is release-shaped**: one store root honoring `UOR_MODEL_STORE` (#790-6), real default-model discovery (#790-3), fail-safe `Production` profile with honest discovery and typed declines (#655-E2 + #789), atomic state writes (#790-8).
- **The WASM static path became real** (#790-5): the dashboard now installs `./graph/score.r4g1` + `./tokenizer.bin` served next to `index.html` — a static host (GitHub Pages included) can serve the real engine with zero install.
- **Still missing** (unchanged since filing): any release/tagging/CI-publish infrastructure; C1e (`ModelStore`'s pre-R4G1 gate — reconciliation in progress under #655); the quality gate itself (see the F-p2 canary packet on #655, 2026-08-19: sampling fixes validity 0/15→15/15 but distinctness is substrate-bound at 5/15).

### Sizes (the pinned `smollm2-360m-broad` artifact, today's best broad-text bundle)

Serving-minimal (R4G1 path): `graph/score.r4g1` 29 MB + `tokenizer.bin` 0.5 MB + `tless_artifacts.bin` 1.4 MB + sidecar ≈ **31 MB**. With the plain-path store (`tless_store.bin` 29 MB) ≈ 60 MB. Full compiled dir incl. corpus ≈ 126 MB. All comfortably GitHub-Release-asset-sized; none comfortable to git-commit.

### The four decisions this issue ultimately needs

1. **Frontend of record** — (a) CLI (`r4` binary + local serve), (b) WASM static site (zero-install, Pages-hostable, now genuinely wired), or (c) both from one release. (c) costs little extra since the same bundle feeds both; the real question is which one README leads with.
2. **Bundle distribution channel** — recommended shape on the evidence: **GitHub Release asset** — tag → CI builds binaries → `package-release-bundle` → attach `bundle.tar.zst` + sidecar; install is an *explicit* verified fetch (a `r4 setup --release <tag>`-style step checking the sidecar κs), satisfying F's "first request must not silently download, contact a provider, or compile." Alternatives with known costs: git-LFS (quota + clone weight for every dev), container image (adds a runtime this CPU-first project doesn't need), source-compile-only (hours-class teacher run on first use — violates F's own text).
3. **Versioning/tagging convention** (none exists today) — proposal: `vX.Y` tags whose release notes pin (code SHA, bundle κs from the sidecar, `INFERENCE_OPERATION_CONTRACT` version) so a release is a verifiable (code, model, contract) triple; the sidecar manifest already carries the κs, so this is convention + CI, not new machinery.
4. **Which bundle ships** — `smollm2-360m-broad` is the pinned broad-text baseline and the natural candidate; the quality gate (not packaging) decides when it's good enough, per this issue's own framing.

### Effort once decided

CI release workflow + explicit-install UX + docs ≈ 2–4 days; nothing blocking discovered. The long pole remains what it was on 2026-08-16: generation quality — now precisely characterized on #655 (decode-policy default = maintainer decision, one-sided evidence; distinctness = #784-class substrate work or an S1-positive route-attention).


### comment 5335968482 — 2026-08-19T00:27:43Z — Casey-allard

**Maintainer decisions adopted (2026-08-19)** — from the brief above, all four: **(1)** both frontends ship from one release (CLI leads, WASM static site as the zero-install demo); **(2)** distribution = **GitHub Release asset** (bundle + sidecar manifest) with an **explicit verified fetch** (`r4 setup --release <tag>`-style, sidecar-κ checked; first request never silently downloads or compiles, per #655-F's own text); **(3)** versioning = `vX.Y` tags binding (code SHA, bundle κs, contract version); **(4)** candidate bundle = `smollm2-360m-broad`. **Sequencing**: the pipeline gets built and tested NOW but publishes nothing until a tag is cut — the quality gate (#655) still controls the actual release, so the 2026-08-16 quality-first framing stands. Implementation starting in the established cadence.

### comment 5336335234 — 2026-08-19T01:22:30Z — Casey-allard

**Pipeline built** (PR #815, auto-merge armed) — the adopted decision implemented end to end, publishing nothing until a tag is cut:

- **Tag-triggered workflow** (`.github/workflows/release.yml`): `v*` push → DRAFT release binding tag → commit SHA + inference-contract version in the notes; native `r4` CLI built + attached for linux x86_64 and macOS arm64; the wasm frontend (same `wasm-pack` build the Pages deploy runs); `.sha256` beside every asset. The model bundle is attached by the maintainer from a locally packaged compile (CI cannot compile the model — hours of teacher-bound compute against a pinned snapshot).
- **Explicit verified fetch**: `r4 install-release --tag vX.Y [--repo owner/name] [--name install-name]` — fetches `release-bundle.json` + `release-bundle.tar.gz`, hard-verifies **every** declared component digest against the extracted bytes, refuses any unattested archive entry or symlink, never overwrites an existing install, and installs atomically with the sidecar beside the bundle (so #655-C1c serving-time verification sees the same manifest). Failure at any step installs nothing. Tests cover the happy path, tampered bytes, smuggled files, overwrite refusal, and pre-fetch shape validation — with a fixture fetcher, so the network never runs in CI.
- **`docs/RELEASE_PIPELINE.md`**: the vX.Y convention (code SHA + bundle κs + contract version in one auditable place), the exact maintainer steps to cut a release (bundle candidate per this decision record: the `smollm2-360m-broad` compile — pending its S1/quality story), and the fetch contract.

**What this closes**: #655 precondition 1's mechanism — a fresh clone reaching a real bundle is now one explicit verified command away *once a release exists*. What remains for #741 itself is the first actual tag cut, which is yours: pick the bundle, run the packaging steps in `docs/RELEASE_PIPELINE.md`, push the tag, attach, publish. The #655-F identity flip stays separately gated on its own preconditions + fresh sign-off, untouched by this pipeline.


### comment 5336515298 — 2026-08-19T01:48:33Z — Casey-allard

**Maintainer decision (Casey, 2026-08-19 ~01:50Z): cut and publish v0.1 now.** Executing per `docs/RELEASE_PIPELINE.md`, with two companion decisions recorded on #655 (implement #811 now; F flip authorized, sequenced after this release + #811).

Plan of record for this cut:
- Bundle: `smollm2-360m-broad` (the standing candidate from the adoption decision), packaged as `model_id: r4`, capability `instruction-chat`, adapter resolved from the pinned source snapshot.
- Tag `v0.1` on current main (contains the decode-default flip #814 — the release binaries serve 15/15-valid completions by default; canary evidence on #655, issuecomment-5336289791).
- Release notes disclose honestly: validity 15/15 under the default decode; distinctness limitation open as #784 (5/15 distinct on the declared prompt set); ask-path abstention gap being closed as #811; S1 real-arm verdict to follow on #605/#804 (does not change these bundle bytes).
- Publish (not draft — the verified fetch can only serve from a published release), then an end-to-end `r4 install-release --tag v0.1` verification run before closing this issue.


### comment 5336577298 — 2026-08-19T01:58:20Z — Casey-allard

**Closing: v0.1 is published and the whole pipeline is proven end to end.**

https://github.com/UOR-Foundation/uor-r4/releases/tag/v0.1

- **Cut executed per the maintainer decision** (issuecomment-5336515298): bundle = the `smollm2-360m-broad` compile, packaged as `model_id: r4` / `instruction-chat` with the adapter resolved from the pinned source snapshot; tag `v0.1` on `d1db980c` (contains the sampled-decode default, #814).
- **Workflow ran green on the first tag**: draft release created with identity bindings (code SHA + contract 1.0.0), native CLI built + attached for linux x86_64 and macOS arm64, wasm frontend attached, `.sha256` beside every asset. Bundle archive (16 MB) + `release-bundle.json` attached; notes disclose the quality status honestly (15/15 valid under the default decode; #784 distinctness limitation; #811 in flight; S1 verdict pending and independent of these bytes).
- **E2E verified fetch proven against the published release**: `r4 install-release --tag v0.1 --name r4-release-verify` → downloaded both assets, verified every component digest (graph `d82d9d9a…`, signature artifact `0b85c43d…`, tokenizer `70af0cb0…`, both reports), installed atomically with the sidecar beside the bundle — and the installed bundle **served a completion** via the default decode. Verification install cleaned up afterward.

That is the full answer to this issue's question — what shipping looks like: **both frontends built per tag, the model bundle as an attested GitHub Release asset, one explicit hard-verified install command, and a version convention binding code + bundle + contract.** #655 precondition 1 is now real: a fresh clone is one command from a verified bundle.

Follow-ups live elsewhere: quality/distinctness #784, ask-path abstention #811, S1 evidence #605/#804, and the #655-F identity flip (authorized, sequenced after #811 per issuecomment-5336515423).


---
_Comment/review ids in this file: 5316909437, 5329925429, 5335803275, 5335968482, 5336335234, 5336515298, 5336577298_

