# Release pipeline (#741)

Adopted by maintainer decision on 2026-08-19 (recorded on #741/#655):
the "package + build release pipeline now" option — both frontends, a
GitHub Release as the distribution artifact, an explicit verified fetch
command, and a versioning convention that binds each tag to its exact
code and bundle identity. **This pipeline publishes nothing by itself**:
every piece below is inert until the maintainer pushes a `v*` tag, and
even then the release starts as a draft that only the maintainer
publishes.

## Versioning convention

A release tag `vX.Y` binds, in one auditable place:

- **code identity** — the tag's commit SHA (in the release notes and on
  the tag itself);
- **contract identity** — the inference-contract version from
  `docs/inference_contract.md` (in the release notes);
- **bundle identity** — the blake3 component digests (graph, signature
  artifact, tokenizer, score/compile reports) declared by the attached
  `release-bundle.json`, the #655-D sidecar manifest produced by
  `r4 package-release-bundle`.

A tag with no bundle assets attached is a code-only release; the
verified fetch command simply reports the missing asset.

## What the tag triggers (`.github/workflows/release.yml`)

Pushing `vX.Y` creates a **draft** release and attaches both frontends:

- `r4-vX.Y-x86_64-unknown-linux-gnu.tar.gz` — native CLI, linux;
- `r4-vX.Y-aarch64-apple-darwin.tar.gz` — native CLI, macOS arm64;
- `r4-wasm-vX.Y.tar.gz` — the wasm frontend (`wasm-pack build`, the
  same build the Pages deploy runs);
- a `.sha256` beside each.

CI cannot compile the model (hours of teacher-bound compute against a
pinned snapshot), so the model bundle is attached by the maintainer:

## Maintainer steps to cut a release

1. Pick the bundle (current candidate per the decision record:
   the `smollm2-360m-broad` compile) and package its sidecar:

   ```sh
   r4 package-release-bundle \
     --compiled .uor-models/compiled/<name> \
     --model-id r4 --capability instruction-chat \
     --source .uor-models/sources/<source> \
     --tokenizer-family hf-byte-bpe --tokenizer-version 1
   ```

2. Archive exactly the attested component files (the packaged layout —
   nothing else; `r4 install-release` refuses archives carrying any
   unattested entry):

   ```sh
   cd .uor-models/compiled/<name>
   tar -czf /tmp/release-bundle.tar.gz \
     graph/score.r4g1 tless_artifacts.bin tokenizer.bin \
     graph/score_report.json graph-cover/cover_report.json
   ```

3. Tag and push: `git tag vX.Y && git push origin vX.Y` — the workflow
   drafts the release and attaches the frontend builds.

4. Attach the bundle and publish:

   ```sh
   gh release upload vX.Y /tmp/release-bundle.tar.gz \
     .uor-models/compiled/<name>/release-bundle.json
   gh release edit vX.Y --draft=false
   ```

## The explicit verified fetch

```sh
r4 install-release --tag vX.Y            # from UOR-Foundation/uor-r4
r4 install-release --tag vX.Y --repo owner/name --name custom-install
```

`install-release` (`src/release_install.rs`) downloads the two bundle
assets, hard-verifies **every** declared component digest against the
extracted bytes, refuses archives containing anything unattested (or
any symlink), never overwrites an existing install, and only then moves
the bundle into `.uor-models/compiled/<name>` with its sidecar beside
it (so serving-time advisory verification, #655-C1c, sees the same
manifest). A failure at any step installs nothing. The fetch is always
explicit: no serving path, first request, or setup step triggers it —
exactly #655's "first request must not silently download" scope line.

## What this pipeline deliberately does not do

- It does not flip any serving default or canonical model identity —
  that is #655-F, separately gated on its own preconditions and a fresh
  maintainer sign-off.
- It does not auto-publish: drafts require the maintainer.
- It does not compile models in CI.
