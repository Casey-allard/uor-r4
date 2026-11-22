// Usage: node web/studio/verify_public_m1.mjs <staged or live Studio URL>
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';
import { PUBLIC_M1_ID, validateM1CardPins, validateM1DownloadRequests } from '../../scripts/studio/public-m1.mjs';

const url = new URL(process.argv[2]);
const sha = '04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d';
const prompt = 'The capital of France is';
// IntegerDiscreteModel::evaluate_prefix on bound BPE IDs 413,2743,3695,372,3704,1651,435.
const expected = [223, 952, 2359, 2471, 23, 263, 964, 1520];
const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : { channel: 'chrome' });
try {
    const context = await browser.newContext();
    await context.addInitScript(() => {
        // Observe the ordinary page-to-worker call without replacing its payload or result.
        const post = Worker.prototype.postMessage;
        window.__m1LoadedEntries = [];
        document.addEventListener('uor-model-source-changed', ({ detail }) => {
            if (detail.kind === 'wasm' && detail.publishedEntry) window.__m1LoadedEntries.push(detail.publishedEntry);
        });
        const observations = [];
        window.__m1WorkerPredictions = observations;
        Worker.prototype.postMessage = function (message, ...args) {
            if (message?.action === 'predict_discrete') {
                const observation = { prefix: [...message.payload.tokenIds] };
                observations.push(observation);
                const onReply = event => {
                    const reply = event.data;
                    if (reply?.id !== message.id || reply.action !== 'discrete_prediction_result') return;
                    observation.reply = { sha256: reply.artifact_sha256, top8: reply.top8, token: reply.prediction, cells: reply.admitted_cells };
                    this.removeEventListener('message', onReply);
                };
                this.addEventListener('message', onReply);
            }
            return post.call(this, message, ...args);
        };
    });
    const page = await context.newPage();
    const external = [];
    const downloadsByRoot = new Map();
    const errors = [];
    const releaseBase = 'https://huggingface.co/caseyallard/uor-r4-m1-model/resolve/1fec0d4c42d4d5929a0cf08b1a0f17291d1dbc55/';
    const permittedRoots = [releaseBase + 't16-frozen-zero-seed1-trunk.d11', releaseBase + 'tokenizer-seed-1.json'];
    context.on('request', request => {
        const target = new URL(request.url());
        if (!/^https?:$/.test(target.protocol) || target.origin === url.origin) return;
        let root = request;
        while (root.redirectedFrom()) root = root.redirectedFrom();
        external.push({ method: request.method(), origin: target.origin, root: root.url(), request });
    });
    context.on('response', response => {
        if (response.status() !== 200) return;
        let root = response.request();
        while (root.redirectedFrom()) root = root.redirectedFrom();
        if (permittedRoots.includes(root.url())) downloadsByRoot.set(root.url(), response);
    });
    page.on('pageerror', error => errors.push(error.message));
    const catalogResponse = page.waitForResponse(new URL('models.json', url).href);
    await page.goto(url.href);
    await page.getByRole('button', { name: 'Models', exact: true }).click();
    const load = page.getByRole('button', { name: 'Load M1 next-token predictor (seed 1)', exact: true });
    assert.deepEqual(external, []);
    const response = await catalogResponse;
    assert.equal(response.status(), 200);
    const catalog = await response.json();
    const cards = catalog.models.filter(entry => entry.id === PUBLIC_M1_ID);
    assert.equal(cards.length, 1, 'expected precisely one M1 catalog entry');
    const [card] = cards;
    assert.equal(card.url, permittedRoots[0]);
    assert.equal(card.tokenizer_url, permittedRoots[1]);
    assert.equal(card.served_forward_audit_schema, 'uor-r4.m1-discrete-served-forward/1');
    const receiptUrl = new URL(card.audit_url, url);
    assert.equal(receiptUrl.origin, url.origin);
    const receipt = await context.request.get(receiptUrl.href, { maxRedirects: 0 });
    assert.equal(receipt.status(), 200);
    assert.equal(receipt.url(), receiptUrl.href);
    const receiptBytes = await receipt.body();
    const measured = JSON.parse(receiptBytes);
    const receiptSha = createHash('sha256').update(receiptBytes).digest('hex');
    assert.equal(receiptSha, 'ff370aefd6ba8062e2316c5478a8d428b8e220f8c5da2d47f7b0eeed24bfe479');
    assert.equal(card.served_forward_audit_sha256, receiptSha);
    validateM1CardPins(card, measured);
    await load.evaluate(button => button.addEventListener('click', event => {
        if (event.isTrusted) window.__m1LoadClickTime = performance.timeOrigin + event.timeStamp;
    }, { capture: true, once: true }));
    assert.equal(external.length, 0, 'unexpected external request before the model-load click');
    await load.click();
    const predict = page.getByRole('button', { name: 'Predict next tokens', exact: true });
    await predict.waitFor({ timeout: 180000 });
    // A Models-panel refresh can replace the boot catalog. Inspect the entry the loader
    // actually used for these bytes, including the receipt associated with that entry.
    const loadedEntries = await page.evaluate(() => window.__m1LoadedEntries);
    assert.equal(loadedEntries.length, 1, 'expected one completed published model load');
    const [loaded] = loadedEntries;
    assert.equal(loaded.id, 'published-' + PUBLIC_M1_ID);
    assert.equal(loaded.url, permittedRoots[0]);
    assert.equal(loaded.tokenizerUrl, permittedRoots[1]);
    assert.equal(loaded.auditSchema, measured.schema);
    assert.ok(loaded.auditUrl, 'missing receipt URL on the loaded M1 card');
    assert.equal(new URL(loaded.auditUrl, url).href, receiptUrl.href);
    assert.equal(loaded.auditSha256, receiptSha);
    validateM1CardPins({ sha256: loaded.sha256, tokenizer_sha256: loaded.tokenizerSha256, schema: loaded.schema, size_bytes: loaded.sizeBytes }, measured);
    const model = await page.locator('html').evaluate(node => ({ ...node.dataset }));
    assert.equal(model.modelSha256, sha);
    assert.equal(model.modelSize, '86536132');
    assert.equal(model.modelSchema, 'uor-r4.lut-discrete-trunk/1');
    assert.equal(model.modelShaVerified, 'true');
    assert.match(model.modelTokenizer, /stack-bound byte-bpe/);
    const clickedAt = await page.evaluate(() => window.__m1LoadClickTime);
    assert.ok(Number.isFinite(clickedAt), 'model-load click timestamp missing');
    const tokenizerResponse = downloadsByRoot.get(permittedRoots[1]);
    assert.ok(tokenizerResponse, 'missing browser tokenizer download');
    const tokenizerBytes = await tokenizerResponse.body();
    assert.equal(tokenizerBytes.length, measured.tokenizer.bytes);
    assert.equal(createHash('sha256').update(tokenizerBytes).digest('hex'), measured.tokenizer.sha256);
    const countBeforeOffline = external.length;
    await context.setOffline(true);
    await page.getByRole('textbox', { name: 'Prediction prefix', exact: true }).fill(prompt);
    await predict.click();
    const result = page.getByRole('region', { name: 'Next-token predictions' });
    await result.waitFor();
    assert.equal(await result.locator('p code').textContent(), JSON.stringify(prompt));
    const tokens = await result.getByRole('listitem').allTextContents();
    const calls = await page.evaluate(() => window.__m1WorkerPredictions);
    assert.equal(calls.length, 1);
    const workload = measured.browser_workload;
    assert.equal(prompt, workload.prompt);
    assert.deepEqual(calls[0].prefix, workload.prefix);
    assert.deepEqual(calls[0].reply, { sha256: sha, top8: workload.native_serving.top8, token: workload.native_serving.token, cells: workload.native_serving.admitted_cells });
    assert.deepEqual(tokens.map(text => Number(/^ID (\d+)/.exec(text)?.[1])), expected);
    assert.equal(await page.locator('html').getAttribute('data-predictor-prompt-length'), '7');
    const requests = external.map(({ request, ...entry }) => ({ ...entry, startTime: request.timing().startTime }));
    validateM1DownloadRequests(requests, permittedRoots, clickedAt, countBeforeOffline);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ prompt, worker_prefix: calls[0].prefix, artifact_sha256: sha, tokenizer_sha256: measured.tokenizer.sha256, tokenizer: model.modelTokenizer, tokens, external_before_load: 0, download_requests: requests, offline_prediction: true }));
} finally {
    await browser.close();
}
