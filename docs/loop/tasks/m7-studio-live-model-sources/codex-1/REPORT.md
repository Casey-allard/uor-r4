# Public M1 seed-1 browser prediction

Lab: codex. References #3440 and the [ORDER](ORDER.md).

## What failed on production

On 17 November 2026, the production catalog SHA-256 was `fa33aa230b37b1d788983f9adb910f35d4562679db5351a5cf32e758d0c5ed79` and listed only the synthetic fixture. Chrome's URL loader downloaded the correct M1 artifact but omitted its tokenizer. The legacy predictor refused the text request. The public typed workbench's Send action created a completed synthetic task with no prediction transcript.

## What the staged client did

1. Downloaded the previously released M1 seed-1 model only after the browser operator chose **Models → Load M1 next-token predictor (seed 1)**. The public revision was `1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55`. The model was 86,536,132 bytes, SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`. Its tokenizer was 265,393 bytes, SHA-256 `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`. Both files were also downloaded anonymously and hashed independently of the browser. [Immutable release](https://huggingface.co/caseyallard/uor-r4-m1-model/tree/1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55).
2. Selected **Predict next tokens** after disabling Chrome's network. The prompt “The capital of France is” encoded to `[413,2743,3695,372,3704,1651,435]`. The visible ordered result was `[223,952,2359,2471,23,263,964,1520]`. The native Rust `IntegerDiscreteModel::evaluate_prefix` on the same public model and IDs returned the same ordering. The top candidate decoded to a space. Chrome observed zero off-origin requests before the load click. Only Hugging Face and its CDN received GETs during model loading.
3. The M1-specific publishing selector takes no operator-provided model path, URL, digest or schema. It checks pinned records for this exact release, including the seed-1 GPU score (14,287/14,287 rows agreed and B4 passed) and the separate seed-1 CPU/trunk parity record (616 rows, 19,712 query coordinates, zero top-eight disagreements). The generic FBS path still requires its served-forward receipt. The scope is a historical D11 next-token predictor. This unit measured no training, new panel, native-state milestone or chat capability.

## Changes after adversarial review

The initial selector emitted the correct model pins but lacked a discrete served-forward receipt. The original browser smoke accepted unrelated requests during the download. The parent approved the [bounded schema-specific release requirement and fixture scope](DECISION.md).

On an Apple M1 with macOS 27.2.1, the read-only `CpuDiscreteTrunkReader::decide` and `IntegerDiscreteModel::evaluate_prefix` each read the exact public bytes and seven IDs above. The [receipt](../../../../../scripts/studio/public-m1-served-forward.json) retains both raw decisions, including 32 signed Q12 coordinates, eight ordered cells, eight ranked token IDs and top-1. SHA-256 of the prefix followed by the native query, cells, candidates and top-1, each encoded as a 32-bit little-endian word, is `a009e0cbdc2a485583803ba9c9cc92e00810a2497c9413fc4a40057331e4cbbe`. The validator compares both raw records, checks that digest, matches the measured browser candidates, binds the historical panel’s manifest/token/mask digests, and pins all three historical evidence files. The served receipt SHA-256 is `ff370aefd6ba8062e2316c5478a8d428b8e220f8c5da2d47f7b0eeed24bfe479`. Mutations of either model identity, workload, output order, or both supposedly agreeing query words fail. The browser smoke checks each external method, HTTPS origin and original URL of every redirect chain, and downloads and verifies the same-origin receipt.

## Changes after the second adversarial review

The standalone predictor now has its own prefix field and results on the Models panel. Regression cases place a real resident-model selection in the task workspace and verify that both **New task** and **New run** send the prompt to the selected task. The publisher rejects duplicate final card IDs, including a fixed M1 selector mixed with an ordinary valid receipted entry in either order. Tokenizer verification hashes the response bytes before UTF-8 decoding. Prepending a BOM to pinned JSON now fails. The Chrome verifier measures the trusted load click against recorded request start times and requires zero earlier external requests.

## Changes after the third adversarial review

The release reserves the M1 catalog ID even when an ordinary trained manifest omits the M1 selector. The browser verifier reads the same-origin receipt using a direct HTTP 200 with redirects disabled. On the normal prediction form it observes the worker message and matching response, validates the actual seven input IDs against the receipt, and compares the returned artifact hash, top-1, ranked tokens and admitted cells to native serving. It also checks the downloaded tokenizer response’s raw byte count and digest. Chrome reports the loaded weights’ byte count and SHA-256, and the matching WASM worker response confirms the artifact SHA-256.

## Changes after the fourth adversarial review

The page previously recorded the resident hash as if the worker had reported it. The page now captures the artifact hash when it sends a prediction, checks the worker’s returned hash and the current resident model against that captured identity, and emits a correlated error before exposing any mismatched candidates. The page-hook tests cover a missing, malformed or different reply hash, a model change during prediction, and a successful retry. The live verifier reads the catalog response that Chrome actually received and compares the entry's model and tokenizer SHA-256, schema, and model byte count with the checksum-pinned receipt. Four locally served catalogs each omitted one field while retaining the identical model URLs and receipt. The verifier rejected each omission.

The top session approved the existing Python fixture's reserved and duplicate final-ID cases, including a mixed ordinary entry and M1 selector. It also approved the relevant catalog and page-hook regression cases and the request/response hash checks. The [decision review](DECISION.md) records the bounded source scope and the existing release gates. All changes were tested after merging `origin/main` at `86d39ed67`.

## Reproduction and handoff

The typed publisher builds the runtime catalog from the pinned selector by default. After review the staged catalog SHA-256 is `fae4cc14401055e511edfc76b64cf7f03622ff29e9acff6cc6613a872282c19f` and includes a same-origin served-forward receipt. The earlier publisher dry run staged 17 files. The revised candidate adds the receipt as its eighteenth file. The generated serving WASM SHA-256 was `a62f959dcbc6062b7bc147830749e1a1b05e350e9e6bdb4509e409d9949b91c3`. Its serving-call audit passed seven checks. The static page scan, Chrome load traps, synthetic worker, all 1,011 typed tests, TypeScript checks, 41 model-source tests and four catalog tests, 16 served-forward tests and the current harness native-space rule passed. The release-gate test fixture now supplies the runtime allowlist files its positive case needs.

```sh
scripts/studio/build-and-publish.sh --typed --target gh-pages-preview --stage-dir <empty-directory>
# Serve the staged directory with a local HTTP server, then run:
node web/studio/verify_public_m1.mjs http://127.0.0.1:<port>/
```

The parent session owns production publication from freshly merged main. Its separate live check uses `node web/studio/verify_public_m1.mjs https://uor-foundation.github.io/uor-r4/studio/` after Pages serves the candidate.

## Fifth adversarial review and candidate check

The boot catalog could pass even when a later fetch dropped the tokenizer digest from the card the operator selected. Ranked tokens could also appear beside an edited prefix. The loader now snapshots the current card, refuses an incomplete reserved M1 entry before downloading, and includes the selected entry in the successful load event. The browser verifier checks both the initial catalog and the selected entry against the release receipt. Each rendered result shows its submitted prefix, including after an edit during the worker request.

On macOS arm64 with Chrome 157.0.8103.79, a two-response catalog fixture served a complete boot card and omitted the tokenizer digest on the Models-panel refresh. The UI reported the missing release pins, made zero off-origin requests and offered no prediction. The complete staged candidate matched the seven prefix IDs and eight native candidates recorded above, with zero off-origin requests before loading and a successful offline prediction for the displayed prefix.

Host checks completed with Node 26.4.0:

- `node --test web/studio/assets/js/model_sources.test.mjs web/studio/assets/js/stage_dist_catalog.test.mjs` passed 46 tests, including the second-catalog regression.
- `npm --prefix web/studio test` passed 1,012 tests across 73 files, including prompt edits during and after a prediction. TypeScript and the typed production build passed.
- `scripts/stage-studio-site.sh --typed web/studio <empty-directory> '' 'Public model' --trained-manifest scripts/studio/public-m1.json` staged 18 allowlisted files and verified three catalog-bound artifacts. `node web/studio/verify_public_m1.mjs <local-stage-url>` passed the candidate smoke. Public deployment and its fresh readback belong to the separate release step.

## Sixth adversarial review and candidate check

The earlier typed source adapter could change its selected model before the page refused a load during prediction. The predictor then disappeared with the pending result. Separately, the verifier could count an off-origin request received while reading tokenizer bytes without checking its root, method or origin. The review observed the selection failure and identified the verifier gap by tracing the code.

The page now supplies its worker-busy reason to the typed adapter before any source change. The shared check covers bundled, published, file, URL and API selection. The active-provider edit and removal controls use it too. The verifier takes the offline request count separately, then checks the full request log and that count after reading the visible prediction. Regression cases exercise an unapproved late origin, an otherwise permitted request after the offline boundary and a request started before the click.

On macOS arm64 in Chrome 157.0.8103.79, the staged candidate fetched the pinned public release, encoded the seven recorded token IDs and reproduced all eight native candidates offline. All four off-origin requests were checked, with zero before the trusted click and zero during offline prediction. In a second browser session, after the public model loaded, the test held its outgoing prediction, clicked **Load bundled demo** and observed the refusal with M1 still selected. Releasing the request displayed the original prefix and the eight native IDs, and the predictor accepted another submission. An auxiliary request-interception attempt closed before the model was ready and supplied no result.

The full typed suite passed 1,014 tests in 73 files. The two Node suites passed 47 tests. TypeScript and the production build passed, and the candidate staging check verified three catalog-bound artifacts across 18 allowlisted files. The staged browser verification uses the reproduction command above. These checks cover the source candidate.
