import test from 'node:test';
import assert from 'node:assert/strict';

function artifactBytes() {
    const header = new TextEncoder().encode(JSON.stringify({ schema: 'uor-r4.lut-stack/1', shape: { vocab: 17, context: 64 } }));
    const bytes = new Uint8Array(16 + header.byteLength);
    bytes.set(new TextEncoder().encode('UORLUT01'), 0);
    new DataView(bytes.buffer).setBigUint64(8, BigInt(header.byteLength), true);
    bytes.set(header, 16);
    return bytes;
}

function response(bytes, total = bytes.byteLength) {
    const headers = total === null ? {} : { 'content-length': String(total) };
    return new Response(bytes, { headers });
}

function deferred() {
    let resolve;
    let reject;
    const promise = new Promise((resolvePromise, rejectPromise) => {
        resolve = resolvePromise;
        reject = rejectPromise;
    });
    return { promise, resolve, reject };
}

let moduleCounter = 0;
async function fresh(fetchImpl, seed, storageMode = 'working') {
    const listeners = new Map();
    const events = [];
    const values = new Map();
    if (seed) values.set('uor.studio.modelSources.v1', JSON.stringify(seed));
    const storage = {
        fail: false,
        getItem(key) { return values.has(key) ? values.get(key) : null; },
        setItem(key, value) { if (this.fail) throw new Error('storage full'); values.set(key, value); },
        removeItem(key) { values.delete(key); },
    };
    if (storageMode === 'missing') {
        delete globalThis.localStorage;
    } else if (storageMode === 'throwing') {
        Object.defineProperty(globalThis, 'localStorage', { configurable: true, get() { throw new Error('storage denied'); } });
    } else {
        Object.defineProperty(globalThis, 'localStorage', { configurable: true, writable: true, value: storage });
    }
    globalThis.document = {
        baseURI: 'https://example.test/',
        documentElement: { dataset: {} },
        addEventListener(type, listener) {
            if (!listeners.has(type)) listeners.set(type, []);
            listeners.get(type).push(listener);
        },
        dispatchEvent(event) { events.push(event); return true; },
    };
    globalThis.fetch = fetchImpl;
    globalThis.CustomEvent = class {
        constructor(type, init = {}) { this.type = type; this.detail = init.detail; }
    };
    moduleCounter += 1;
    const module = await import(`./model_sources.js?test=${moduleCounter}`);
    const fire = (type, event) => (listeners.get(type) || []).map((listener) => listener(event));
    return { api: module.ModelSources, events, storage, fire, LOAD_PROGRESS_EVENT: module.LOAD_PROGRESS_EVENT };
}

test('loadFromUrl emits real byte progress and one finalization', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes));
    const result = await api.loadFromUrl('https://example.com/model.lut', { operationId: 1, sourceId: 'url' });
    assert.equal(result.stale, undefined);
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.ok(progress.some((event) => event.detail.phase === 'downloading' && event.detail.loaded === bytes.byteLength && event.detail.total === bytes.byteLength));
    assert.equal(progress.at(-1).detail.phase, 'finalizing');
    assert.ok(events.some((event) => event.type === 'uor-model-source-changed'));
});

test('unknown totals never acquire a percentage', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes, null));
    await api.loadFromUrl('https://example.com/model.lut', { operationId: 1, sourceId: 'url' });
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.ok(progress.length > 0);
    for (const event of progress) {
        assert.equal(event.detail.total, null);
        assert.equal('percent' in event.detail, false);
    }
});

test('a stale completion cannot select or emit a source change', async () => {
    const bytes = artifactBytes();
    const first = deferred();
    const second = deferred();
    const responses = [first.promise, second.promise];
    const { api, events } = await fresh(async () => responses.shift());
    const firstLoad = api.loadFromUrl('https://example.com/a.lut', { operationId: 1, sourceId: 'a' });
    const secondLoad = api.loadFromUrl('https://example.com/b.lut', { operationId: 2, sourceId: 'b' });
    second.resolve(response(bytes));
    const secondResult = await secondLoad;
    first.resolve(response(bytes));
    const firstResult = await firstLoad;
    assert.equal(secondResult.stale, undefined);
    assert.equal(firstResult.stale, true);
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed').length, 1);
});

test('an explicit API selection supersedes an in-flight metadata load', async () => {
    const bytes = artifactBytes();
    const gate = deferred();
    const { api, events } = await fresh(() => gate.promise);
    const active = api.list().api[0];
    const load = api.loadFromUrl('https://example.com/a.lut', { operationId: 1, sourceId: 'url' });
    api.setActive({ kind: 'api', id: active.id });
    gate.resolve(response(bytes));
    const result = await load;
    assert.equal(result.stale, true);
    assert.deepEqual(api.active(), { kind: 'api', id: active.id });
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed' && event.detail.kind === 'wasm').length, 0);
});

test('legacy calls without metadata retain their behavior', async () => {
    const bytes = artifactBytes();
    const { api, events } = await fresh(async () => response(bytes));
    const result = await api.loadFromUrl('https://example.com/model.lut');
    assert.equal(result.stale, undefined);
    assert.equal(api.active().kind, 'wasm');
    assert.equal(api.active().id, 'url');
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed').length, 1);
});

test('published loads forward operation metadata', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes));
    const entry = { id: 'published-a', label: 'Published A', url: 'https://example.com/published.lut' };
    const result = await api.loadPublished(entry, { operationId: 4, sourceId: 'published-a' });
    assert.equal(result.stale, undefined);
    assert.ok(events.some((event) => event.type === LOAD_PROGRESS_EVENT && event.detail.operationId === 4));
});

test('local files finalize without fabricated byte stages', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => { throw new Error('network must not run'); });
    const file = new File([bytes], 'local.lut');
    const result = await api.loadLocalFile(file, { operationId: 2, sourceId: 'local-file' });
    assert.equal(result.stale, undefined);
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.equal(progress.length, 1);
    assert.equal(progress[0].detail.phase, 'finalizing');
});

test('bundled loads accept operation metadata and finalize', async () => {
    const bytes = artifactBytes();
    const { api, events, LOAD_PROGRESS_EVENT } = await fresh(async () => response(bytes));
    const result = await api.loadBundled({ operationId: 3, sourceId: 'bundled-demo' });
    assert.equal(result.stale, undefined);
    const progress = events.filter((event) => event.type === LOAD_PROGRESS_EVENT);
    assert.equal(progress.at(-1).detail.phase, 'finalizing');
    assert.equal(progress.at(-1).detail.operationId, 3);
});

test('model catalog loading emits the actual catalog change', async () => {
    const { api, events } = await fresh(async () => new Response(JSON.stringify({ schema: 'uor-r4.studio-models/1', models: [{ id: 'm1', label: 'M1', url: 'https://example.com/m1.lut' }] }), { status: 200, headers: { 'content-type': 'application/json' } }));
    await api.loadModelCatalog();
    assert.ok(events.some((event) => event.type === 'uor-model-catalog-changed'));
    assert.equal(api.catalog().models[0].id, 'published-m1');
    assert.equal(api.catalog().models[0].label, 'M1');
});

test('non-HTTP API sources cannot be selected or persisted', async () => {
    const invalid = { id: 'bad', label: 'Bad', preset: 'custom', baseUrl: 'ftp://example.com/v1', model: 'model', apiKey: '', createdAt: 1 };
    const { api } = await fresh(async () => { throw new Error('network must not run'); }, { wasm: {}, api: [invalid], active: { kind: 'wasm', id: 'bundled-demo' } });
    assert.throws(() => api.updateApi(invalid.id, { baseUrl: 'ftp://example.com/v1' }), /Only HTTP\(S\) API endpoints are supported/);
    assert.throws(() => api.setActive({ kind: 'api', id: invalid.id }), /Only HTTP\(S\) API endpoints are supported/);
    assert.deepEqual(api.active(), { kind: 'wasm', id: 'bundled-demo' });
});

test('a persisted active non-HTTP API source falls back to the local selection', async () => {
    const invalid = { id: 'bad', label: 'Bad', preset: 'custom', baseUrl: 'ftp://example.com/v1', model: 'model', apiKey: '', createdAt: 1 };
    const { api } = await fresh(async () => { throw new Error('network must not run'); }, { wasm: {}, api: [invalid], active: { kind: 'api', id: 'bad' } });
    assert.deepEqual(api.active(), { kind: 'wasm', id: 'bundled-demo' });
    assert.equal(api.apiConfig(), null);
});

test('API provider writes roll back when browser storage fails', async () => {
    const { api, storage, events } = await fresh(async () => { throw new Error('network must not run'); });
    const existing = api.list().api[0];
    storage.fail = true;
    assert.throws(() => api.addApi({ id: 'api-new', label: 'New', baseUrl: 'https://new.example.com/v1', model: 'model' }), /Could not persist model sources/);
    assert.equal(api.list().api.some((item) => item.id === 'api-new'), false);
    assert.throws(() => api.updateApi(existing.id, { baseUrl: 'https://changed.example.com/v1' }), /Could not persist model sources/);
    assert.equal(api.list().api.find((item) => item.id === existing.id).baseUrl, existing.baseUrl);
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed').length, 0);
    storage.fail = false;
    api.setActive({ kind: 'api', id: existing.id });
    events.length = 0;
    storage.fail = true;
    assert.throws(() => api.setActive({ kind: 'wasm', id: 'bundled-demo' }), /Could not persist model sources/);
    assert.deepEqual(api.active(), { kind: 'api', id: existing.id });
    assert.throws(() => api.removeApi(existing.id), /Could not persist model sources/);
    assert.equal(api.list().api.some((item) => item.id === existing.id), true);
    assert.deepEqual(api.active(), { kind: 'api', id: existing.id });
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed').length, 0);
});

test('the synthetic default is stable and a catalog primary flag does not auto-select', async () => {
    const { api } = await fresh(async () => new Response(JSON.stringify({
        schema: 'uor-r4.studio-models/1',
        models: [{ id: 'lexlean-r4', label: 'External research model', primary: true, url: 'https://example.com/lexlean.lut' }],
    }), { status: 200, headers: { 'content-type': 'application/json' } }));
    assert.deepEqual(api.active(), { kind: 'wasm', id: 'bundled-demo' });
    await api.loadModelCatalog();
    assert.equal(api.catalog().models[0].id, 'published-lexlean-r4');
    assert.deepEqual(api.active(), { kind: 'wasm', id: 'bundled-demo' });
});

test('missing or throwing storage keeps the synthetic default and permits bundled loading', async () => {
    for (const storageMode of ['missing', 'throwing']) {
        const bytes = artifactBytes();
        const { api } = await fresh(async () => response(bytes), null, storageMode);
        assert.deepEqual(api.active(), { kind: 'wasm', id: 'bundled-demo' }, storageMode);
        const result = await api.loadBundled({ operationId: 1, sourceId: 'bundled-demo' });
        assert.equal(result.stale, undefined, storageMode);
        assert.deepEqual(api.active(), { kind: 'wasm', id: 'bundled-demo' }, storageMode);
    }
});

async function pageAnonymousFunction(marker, context) {
    const { readFileSync } = await import('node:fs');
    const { default: vm } = await import('node:vm');
    const html = readFileSync(new URL('../../index.html', import.meta.url), 'utf8');
    const markerIndex = html.indexOf(marker);
    if (markerIndex === -1) throw new Error(`missing anonymous page function: ${marker}`);
    const start = html.indexOf('function', markerIndex);
    const open = html.indexOf('{', start);
    let depth = 0;
    for (let index = open; index < html.length; index += 1) {
        if (html[index] === '{') depth += 1;
        if (html[index] === '}') {
            depth -= 1;
            if (depth === 0) return vm.runInNewContext(`(${html.slice(start, index + 1)})`, context);
        }
    }
    throw new Error(`unterminated anonymous page function: ${marker}`);
}

async function applyPageOperationGuard(api, toast = () => {}, app = { generating: false }) {
    const { readFileSync } = await import('node:fs');
    const { default: vm } = await import('node:vm');
    const html = readFileSync(new URL('../../index.html', import.meta.url), 'utf8');
    const start = html.indexOf('function guardModelSourceOperations(');
    if (start === -1) return false;
    const open = html.indexOf('{', start);
    let depth = 0;
    for (let index = open; index < html.length; index += 1) {
        if (html[index] === '{') depth += 1;
        if (html[index] === '}') {
            depth -= 1;
            if (depth === 0) {
                const source = html.slice(start, index + 1);
                vm.runInNewContext(`${source};guardModelSourceOperations`, { toast, app })(api, globalThis.document);
                return true;
            }
        }
    }
    throw new Error('unterminated page source-operation guard');
}

async function legacyBundledClick(loadBundled) {
    const { readFileSync } = await import('node:fs');
    const { default: vm } = await import('node:vm');
    const path = process.env.UOR_LEGACY_ONCLICK_SOURCE || new URL('./model_sources.js', import.meta.url);
    const html = readFileSync(path, 'utf8');
    const start = html.indexOf('async function onClick(');
    const open = html.indexOf('{', start);
    let depth = 0;
    for (let index = open; index < html.length; index += 1) {
        if (html[index] === '{') depth += 1;
        if (html[index] === '}') {
            depth -= 1;
            if (depth === 0) {
                const source = html.slice(start, index + 1);
                const messages = [];
                const context = {
                    MODEL_SCOPE_SELECTOR: '[data-scope="models"]',
                    mounted: {},
                    latestOperationId: 0,
                    loadBundled,
                    setMessage(tone, text) { messages.push({ tone, text }); },
                };
                const onClick = vm.runInNewContext(`${source};onClick`, context);
                const target = { dataset: { action: 'load-bundled' }, closest() { return target; } };
                return { context, messages, run: () => onClick({ target }) };
            }
        }
    }
    throw new Error('unterminated legacy click handler');
}

test('legacy bundled clicks surface current rejection and suppress stale rejection', async () => {
    const current = await legacyBundledClick(async () => {
        current.context.latestOperationId += 1;
        throw new Error('current source failed');
    });
    await current.run();
    assert.deepEqual(current.messages, [{ tone: 'error', text: 'current source failed' }]);

    let rejectLoad;
    const stale = await legacyBundledClick(() => {
        stale.context.latestOperationId += 1;
        return new Promise((_resolve, reject) => { rejectLoad = reject; });
    });
    const running = stale.run();
    stale.context.latestOperationId += 1;
    rejectLoad(new Error('stale source failed'));
    await running;
    assert.deepEqual(stale.messages, []);
});

async function applyModelSourceChangeGuard(app) {
    const { readFileSync } = await import('node:fs');
    const { default: vm } = await import('node:vm');
    const html = readFileSync(new URL('../../index.html', import.meta.url), 'utf8');
    const start = html.indexOf('function onModelSourceChanged(');
    const open = html.indexOf('{', start);
    let depth = 0;
    for (let index = open; index < html.length; index += 1) {
        if (html[index] === '{') depth += 1;
        if (html[index] === '}') {
            depth -= 1;
            if (depth === 0) {
                const source = html.slice(start, index + 1);
                const cleared = [];
                vm.runInNewContext(`${source};onModelSourceChanged`, {
                    app,
                    abortApiForLocalSource() {},
                    clearWorkerModel() { cleared.push(true); app.caps = null; app.readyLoadId = null; },
                    setModelStatus() {},
                    updatePrivacyStatus() {},
                    updateComposerState() {},
                    setView() {},
                    toast() {},
                })({ detail: { kind: 'wasm', bytes: new Uint8Array([1]) } });
                return cleared;
            }
        }
    }
    throw new Error('unterminated model-source change handler');
}

test('a deferred source commit during generation preserves healthy capabilities', async () => {
    const app = { generating: true, caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7, worker: null };
    const cleared = await applyModelSourceChangeGuard(app);
    assert.deepEqual(cleared, []);
    assert.deepEqual(app.caps, { schema: 'healthy' });
    assert.equal(app.readyLoadId, 7);
});

async function pageFunction(marker, name, context, file = new URL('../../index.html', import.meta.url)) {
    const { readFileSync } = await import('node:fs');
    const { default: vm } = await import('node:vm');
    const html = readFileSync(file, 'utf8');
    const start = html.indexOf(marker);
    if (start === -1) throw new Error(`missing page function ${name}`);
    const open = html.indexOf('{', start);
    let depth = 0;
    for (let index = open; index < html.length; index += 1) {
        if (html[index] === '{') depth += 1;
        if (html[index] === '}') {
            depth -= 1;
            if (depth === 0) {
                return vm.runInNewContext(`${html.slice(start, index + 1)};${name}`, context);
            }
        }
    }
    throw new Error(`unterminated page function ${name}`);
}

async function pageFunctionIfPresent(marker, name, context) {
    const { readFileSync } = await import('node:fs');
    const html = readFileSync(new URL('../../index.html', import.meta.url), 'utf8');
    if (!html.includes(marker)) return null;
    return pageFunction(marker, name, context);
}

async function installPageSourceHandlers(api, app, harness, notices = []) {
    const cleared = [];
    let aborts = 0;
    const toast = (text, tone) => notices.push({ text, tone });
    const captureHealthyModelState = await pageFunctionIfPresent(
        'function captureHealthyModelState(',
        'captureHealthyModelState',
        { app, document: globalThis.document },
    ) || (() => null);
    const abortForLocalSource = await pageFunction('function abortApiForLocalSource(', 'abortApiForLocalSource', { app });
    const handler = await pageFunction('function onModelSourceChanged(', 'onModelSourceChanged', {
        app,
        ModelSources: api,
        toast,
        captureHealthyModelState,
        window: { performance: { now: () => 0 } },
        performance: { now: () => 0 },
        abortApiForLocalSource() { aborts += 1; abortForLocalSource(); },
        clearWorkerModel() {
            cleared.push(true);
            app.caps = null;
            app.modelMeta = null;
            app.tokenizerArtifact = null;
            app.readyLoadId = null;
            app.loadMs = null;
            ['modelSchema', 'modelSha256', 'modelShaVerified', 'modelSize', 'modelTokenizer', 'modelSource', 'modelSourceLabel', 'modelDiscrete', 'modelLoadMs'].forEach((name) => {
                globalThis.document.documentElement.dataset[name] = '';
            });
        },
        setModelStatus(text, tone, detail) { (app.modelStatusLog || (app.modelStatusLog = [])).push({ text, tone, detail }); },
        updatePrivacyStatus() {},
        updateComposerState() {},
        setView() {},
        renderMessages() {},
        isNativeLoopback() { return false; },
    });
    const guard = await pageFunction('function guardModelSourceOperations(', 'guardModelSourceOperations', { toast, app, onModelSourceChanged: handler });
    guard(api, globalThis.document);
    globalThis.document.addEventListener('uor-model-source-changed', handler);
    const dispatch = globalThis.document.dispatchEvent;
    globalThis.document.dispatchEvent = function (event) {
        dispatch.call(this, event);
        harness.fire(event.type, event);
        return true;
    };
    return { cleared, toast, handle: handler, get aborts() { return aborts; } };
}

async function installWorkerMessageHandler(app, notices = []) {
    const setModelStatus = (text, tone, detail) => { (app.modelStatusLog || (app.modelStatusLog = [])).push({ text, tone, detail }); };
    const restoreHealthyModelState = await pageFunctionIfPresent(
        'function restoreHealthyModelState(',
        'restoreHealthyModelState',
        { app, document: globalThis.document, updateCapabilitiesFooter() {}, setModelStatus, setEngineStatus() {}, describeCapabilities() { return ''; } },
    ) || (() => false);
    const clearWorkerModel = () => {
        app.caps = null;
        app.modelMeta = null;
        app.tokenizerArtifact = null;
        app.readyLoadId = null;
        app.loadMs = null;
        ['modelSchema', 'modelSha256', 'modelShaVerified', 'modelSize', 'modelTokenizer', 'modelSource', 'modelSourceLabel', 'modelDiscrete', 'modelLoadMs'].forEach((name) => {
            globalThis.document.documentElement.dataset[name] = '';
        });
    };
    return pageFunction('function onWorkerMessage(', 'onWorkerMessage', {
        app,
        document: globalThis.document,
        toast(text, tone) { notices.push({ text, tone }); },
        updateComposerState() {},
        restoreHealthyModelState,
        clearWorkerModel,
        setEngineStatus() {},
        describeCapabilities() { return ''; },
        setModelStatus,
        isDiscreteCapabilities() { return false; },
        setPredictorMode() {},
        fillPredictorSha() {},
        fillPredictorModelInfo() {},
        updateCapabilitiesFooter() {},
        renderMessages() {},
        restoreActiveConversation() {},
        window: { performance: { now: () => 0 } },
        performance: { now: () => 0 },
    });
}

test('a load started before generation cannot replace the healthy model when it commits during generation', async () => {
    const bytes = artifactBytes();
    const gate = deferred();
    const harness = await fresh(() => gate.promise);
    const app = { generating: false, mode: 'wasm', caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7, workerState: 'ready' };
    const page = await installPageSourceHandlers(harness.api, app, harness);
    const load = harness.api.loadFromUrl('https://example.com/late.lut', { operationId: 1, sourceId: 'url' });
    app.generating = true;
    gate.resolve(response(bytes));
    const result = await load;
    assert.equal(result.stale, undefined);
    assert.deepEqual(page.cleared, []);
    assert.deepEqual(app.caps, { schema: 'healthy' });
    assert.equal(app.readyLoadId, 7);
    assert.equal(app.currentLoadId, 7);
    assert.equal(harness.events.filter((event) => event.type === 'uor-model-source-changed' && event.detail.kind === 'wasm').length, 1);
});

test('API use during generation is blocked before the active source changes', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const notices = [];
    const app = { generating: true, mode: 'wasm', caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7, apiConfig: null };
    await installPageSourceHandlers(harness.api, app, harness, notices);
    const before = harness.api.active();
    const card = { dataset: { apiId: 'api-1' } };
    const target = {
        closest(selector) {
            if (selector === '[data-action="api-use"]') return target;
            if (selector === '[data-api-id]') return card;
            return null;
        },
    };
    let prevented = false;
    let stopped = false;
    harness.fire('click', {
        target,
        preventDefault() { prevented = true; },
        stopImmediatePropagation() { stopped = true; },
    });
    assert.equal(prevented, true);
    assert.equal(stopped, true);
    assert.deepEqual(harness.api.active(), before);
    assert.deepEqual(app.caps, { schema: 'healthy' });
    assert.equal(app.readyLoadId, 7);
    assert.deepEqual(notices, [{ text: 'Stop the current reply before changing models.', tone: 'warn' }]);

    app.generating = false;
    let preventedAfter = false;
    harness.fire('click', {
        target,
        preventDefault() { preventedAfter = true; },
        stopImmediatePropagation() {},
    });
    assert.equal(preventedAfter, false);
    assert.equal(notices.length, 1);
    assert.deepEqual(harness.api.active(), before);
});

test('API selection supersedes an in-flight source rejection', async () => {
    const gate = deferred();
    const harness = await fresh(() => gate.promise);
    const notices = [];
    const app = { generating: false, mode: 'wasm', caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7 };
    await installPageSourceHandlers(harness.api, app, harness, notices);
    const load = harness.api.loadFromUrl('https://example.com/a.lut');
    harness.api.setActive({ kind: 'api', id: harness.api.list().api[0].id });
    gate.reject(new Error('late source failed'));
    await assert.rejects(load, (error) => {
        assert.equal(error.uorStale, true);
        return /late source failed/.test(error.message);
    });
    assert.deepEqual(notices.filter((notice) => notice.tone === 'error'), []);
    assert.equal(harness.api.active().kind, 'api');
});

test('a stale load-url rejection is not shown as the current source error', async () => {
    const staleError = new Error('late source failed');
    staleError.uorStale = true;
    const currentError = new Error('current source failed');
    for (const [error, expected] of [[staleError, []], [currentError, [{ tone: 'error', text: 'current source failed' }]]]) {
        const messages = [];
        const context = {
            MODEL_SCOPE_SELECTOR: '[data-scope="models"]',
            mounted: {},
            latestOperationId: 1,
            document: { querySelector() { return { value: 'https://example.com/a.lut' }; } },
            api: { loadFromUrl() { return Promise.reject(error); } },
            setMessage(tone, text) { messages.push({ tone, text }); },
        };
        const onClick = await pageFunction('async function onClick(', 'onClick', context, new URL('./model_sources.js', import.meta.url));
        const target = {
            dataset: { action: 'load-url' },
            closest(selector) { return selector === '[data-action]' ? target : {}; },
        };
        await onClick({ target });
        assert.deepEqual(messages, expected);
    }
});

test('programmatic API selection during generation is applied after the reply finishes', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const notices = [];
    const app = { generating: true, mode: 'wasm', caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7, apiConfig: null, worker: null };
    const page = await installPageSourceHandlers(harness.api, app, harness, notices);
    const entry = harness.api.list().api[0];
    harness.api.setActive({ kind: 'api', id: entry.id });
    assert.equal(app.mode, 'wasm');
    assert.deepEqual(app.caps, { schema: 'healthy' });
    assert.equal(app.readyLoadId, 7);
    assert.deepEqual(page.cleared, []);
    app.generating = false;
    harness.fire('uor-generation-finished', { type: 'uor-generation-finished', detail: { outcome: 'complete' } });
    assert.equal(app.mode, 'api');
    assert.equal(app.apiConfig.id, entry.id);
    assert.deepEqual(page.cleared, [true]);
    assert.equal(harness.api.active().kind, 'api');
});

test('a source load started before generation is applied after the reply finishes', async () => {
    const bytes = artifactBytes();
    const gate = deferred();
    const harness = await fresh(() => gate.promise);
    const app = { generating: false, mode: 'wasm', caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7, worker: null, workerState: 'ready' };
    const page = await installPageSourceHandlers(harness.api, app, harness);
    const load = harness.api.loadFromUrl('https://example.com/late.lut', { operationId: 1, sourceId: 'url' });
    app.generating = true;
    gate.resolve(response(bytes));
    await load;
    assert.deepEqual(page.cleared, []);
    assert.equal(app.currentLoadId, 7);
    assert.equal(app.mode, 'wasm');
    const posts = [];
    app.worker = { postMessage(message) { posts.push(message); } };
    app.generating = false;
    harness.fire('uor-generation-finished', { type: 'uor-generation-finished', detail: { outcome: 'complete' } });
    assert.equal(app.currentLoadId, 8);
    assert.equal(posts.length, 1);
    assert.equal(posts[0].action, 'load_model');
    assert.equal(app.workerState, 'loading');
    assert.deepEqual(page.cleared, [true]);
});

test('the latest deferred source selection wins after the reply finishes', async () => {
    const bytes = artifactBytes();
    const gate = deferred();
    const harness = await fresh(() => gate.promise);
    const app = { generating: false, mode: 'wasm', caps: { schema: 'healthy' }, readyLoadId: 7, currentLoadId: 7, worker: null, workerState: 'ready' };
    const page = await installPageSourceHandlers(harness.api, app, harness);
    const load = harness.api.loadFromUrl('https://example.com/late.lut', { operationId: 1, sourceId: 'url' });
    app.generating = true;
    gate.resolve(response(bytes));
    await load;
    const entry = harness.api.list().api[0];
    harness.api.setActive({ kind: 'api', id: entry.id });
    const posts = [];
    app.worker = { postMessage(message) { posts.push(message); } };
    app.generating = false;
    harness.fire('uor-generation-finished', { type: 'uor-generation-finished', detail: { outcome: 'complete' } });
    assert.equal(app.mode, 'api');
    assert.equal(app.apiConfig.id, entry.id);
    assert.equal(posts.length, 0);
    assert.equal(app.currentLoadId, 8);
    assert.deepEqual(page.cleared, [true]);
});

test('removing the active API during generation fails closed', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const notices = [];
    const app = { generating: false, mode: 'wasm', caps: null, readyLoadId: 9, currentLoadId: 9, apiConfig: null, worker: null, workerState: 'ready' };
    const page = await installPageSourceHandlers(harness.api, app, harness, notices);
    const entry = harness.api.list().api[0];
    harness.api.setActive({ kind: 'api', id: entry.id });
    assert.equal(app.mode, 'api');
    app.generating = true;
    app.activeGenerationKind = 'api';
    app.abortController = new AbortController();
    harness.api.removeApi(entry.id);
    assert.equal(app.abortController.signal.aborted, true);
    assert.equal(page.aborts, 1);
    assert.equal(app.mode, 'wasm');
    assert.equal(app.apiConfig, null);
    assert.equal(app.caps, null);
    assert.equal(harness.api.active().kind, 'wasm');
});

test('API removal during generation is blocked before the endpoint is deleted', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const notices = [];
    const app = { generating: true, mode: 'api', apiConfig: { id: 'api-1' }, caps: null, readyLoadId: 9, currentLoadId: 9 };
    await installPageSourceHandlers(harness.api, app, harness, notices);
    const before = harness.api.list();
    const card = { dataset: { apiId: 'api-1' } };
    const target = {
        closest(selector) {
            if (selector === '[data-action="api-remove"]') return target;
            if (selector === '[data-api-id]') return card;
            return null;
        },
    };
    let prevented = false;
    let stopped = false;
    harness.fire('click', {
        target,
        preventDefault() { prevented = true; },
        stopImmediatePropagation() { stopped = true; },
    });
    assert.equal(prevented, true);
    assert.equal(stopped, true);
    assert.deepEqual(notices, [{ text: 'Stop the current reply before changing models.', tone: 'warn' }]);
    assert.deepEqual(harness.api.list(), before);

    app.generating = false;
    let preventedAfter = false;
    harness.fire('click', {
        target,
        preventDefault() { preventedAfter = true; },
        stopImmediatePropagation() {},
    });
    assert.equal(preventedAfter, false);
    assert.equal(notices.length, 1);
    assert.deepEqual(harness.api.list(), before);
});

test('local-file change errors suppress stale rejections and surface current ones', async () => {
    const staleError = new Error('late local source failed');
    staleError.uorStale = true;
    const currentError = new Error('current local source failed');
    for (const [error, expected] of [[staleError, []], [currentError, [{ tone: 'error', text: 'current local source failed' }]]]) {
        const messages = [];
        const context = {
            MODEL_SCOPE_SELECTOR: '[data-scope="models"]',
            api: { loadLocalFile() { return Promise.reject(error); } },
            setMessage(tone, text) { messages.push({ tone, text }); },
        };
        const onChange = await pageFunction('async function onChange(', 'onChange', context, new URL('./model_sources.js', import.meta.url));
        const input = { files: [{}] };
        const target = {
            closest(selector) {
                if (selector === '[data-scope="models"]') return {};
                if (selector === '[data-role="model-file"]') return input;
                return null;
            },
        };
        await onChange({ target });
        assert.deepEqual(messages, expected);
    }
});

test('page source loads preserve start-time order when legacy calls omit metadata', async () => {
    const bytes = artifactBytes();
    const first = deferred();
    const second = deferred();
    const responses = [first.promise, second.promise];
    const { api, events } = await fresh(async () => responses.shift());
    await applyPageOperationGuard(api);
    const firstLoad = api.loadFromUrl('https://example.com/a.lut');
    const secondLoad = api.loadFromUrl('https://example.com/b.lut');
    second.resolve(response(bytes));
    const secondResult = await secondLoad;
    first.resolve(response(bytes));
    const firstResult = await firstLoad;
    assert.equal(secondResult.stale, undefined);
    assert.equal(firstResult.stale, true);
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed' && event.detail.kind === 'wasm').length, 1);
});

test('an API selection supersedes a page source load without operation metadata', async () => {
    const bytes = artifactBytes();
    const gate = deferred();
    const { api, events } = await fresh(() => gate.promise);
    await applyPageOperationGuard(api);
    const active = api.list().api[0];
    const load = api.loadFromUrl('https://example.com/a.lut');
    api.setActive({ kind: 'api', id: active.id });
    gate.resolve(response(bytes));
    const result = await load;
    assert.equal(result.stale, true);
    assert.deepEqual(api.active(), { kind: 'api', id: active.id });
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed' && event.detail.kind === 'wasm').length, 0);
});

test('a panel bundled load cannot replace a newer URL source', async () => {
    const bytes = artifactBytes();
    const bundled = deferred();
    const url = deferred();
    const responses = [bundled.promise, url.promise];
    const { api, events, fire } = await fresh(async () => responses.shift());
    const directBundled = api.loadBundled;
    await applyPageOperationGuard(api);
    const clicks = fire('click', {
        target: { closest() { return {}; } },
        preventDefault() {},
        stopImmediatePropagation() {},
    });
    const bundledLoad = clicks.length ? clicks[0] : directBundled();
    const urlLoad = api.loadFromUrl('https://example.com/newer.lut');
    url.resolve(response(bytes));
    const urlResult = await urlLoad;
    bundled.resolve(response(bytes));
    const bundledResult = await bundledLoad;
    assert.equal(urlResult.stale, undefined);
    assert.equal(bundledResult.stale, true);
    assert.equal(api.active().id, 'url');
    assert.equal(events.filter((event) => event.type === 'uor-model-source-changed' && event.detail.kind === 'wasm').length, 1);
});

test('a rejected panel bundled load toasts without an unhandled rejection', async () => {
    const { api, fire } = await fresh(async () => { throw new Error('bundled artifact missing'); });
    const notices = [];
    await applyPageOperationGuard(api, (text, tone) => notices.push({ text, tone }));
    const unhandled = [];
    const onUnhandled = (reason) => unhandled.push(reason);
    process.on('unhandledRejection', onUnhandled);
    try {
        const clicks = fire('click', {
            target: { closest() { return {}; } },
            preventDefault() {},
            stopImmediatePropagation() {},
        });
        assert.equal(clicks.length, 1);
        await new Promise((resolve) => setTimeout(resolve, 0));
        assert.deepEqual(unhandled, []);
        assert.deepEqual(notices, [{ text: 'bundled artifact missing', tone: 'error' }]);
    } finally {
        process.off('unhandledRejection', onUnhandled);
    }
});

test('boot and quick demo failures each toast once through the real page wrapper', async () => {
    for (const kind of ['boot', 'quick']) {
        const { api } = await fresh(async () => { throw new Error('bundled artifact missing'); });
        const notices = [];
        const toast = (text, tone) => notices.push({ text, tone });
        await applyPageOperationGuard(api, toast);
        if (kind === 'boot') {
            const onError = await pageAnonymousFunction(
                "ModelSources.loadBundled({ operationId: 1, sourceId: 'bundled-demo' }).catch(",
                { toast },
            );
            await api.loadBundled({ operationId: 1, sourceId: 'bundled-demo' }).catch(onError);
        } else {
            const quick = await pageAnonymousFunction("quickButton('Try offline demo',", {
                window: { UORModelSources: api },
                ModelSources: api,
                toast,
            });
            quick();
            await new Promise((resolve) => setTimeout(resolve, 0));
        }
        assert.equal(notices.length, 1, `${kind} produced duplicate error toasts: ${JSON.stringify(notices)}`);
        assert.equal(notices[0].tone, 'error');
        assert.ok(notices[0].text.includes('bundled artifact missing'));
    }
});

test('source changes are blocked during local generation without discarding the active source', async () => {
    const bytes = artifactBytes();
    const { api } = await fresh(async () => response(bytes));
    const before = api.active();
    const notices = [];
    await applyPageOperationGuard(api, (text, tone) => notices.push({ text, tone }), { generating: true });
    const result = await api.loadBundled();
    assert.equal(result.blocked, true);
    assert.deepEqual(api.active(), before);
    assert.deepEqual(notices, [{ text: 'Stop the current reply before changing models.', tone: 'warn' }]);
});

test('a stale bundled rejection after a newer source does not toast', async () => {
    const bytes = artifactBytes();
    const bundled = deferred();
    const url = deferred();
    const responses = [bundled.promise, url.promise];
    const { api, fire } = await fresh(async () => responses.shift());
    const notices = [];
    await applyPageOperationGuard(api, (text, tone) => notices.push({ text, tone }));
    const clicks = fire('click', {
        target: { closest() { return {}; } },
        preventDefault() {},
        stopImmediatePropagation() {},
    });
    const bundledLoad = clicks[0];
    const urlLoad = api.loadFromUrl('https://example.com/newer.lut');
    url.resolve(response(bytes));
    await urlLoad;
    bundled.reject(new Error('older source failed'));
    await assert.rejects(bundledLoad, /older source failed/);
    assert.deepEqual(notices, []);
});

test('a stop-generation load rejection restores the healthy ready model and ignores the stale load', async () => {
    const app = { currentLoadId: 8, readyLoadId: 7, caps: { schema: 'healthy' }, workerState: 'loading', generating: true, activeGenerationKind: 'wasm' };
    const notices = [];
    const onWorkerMessage = await installWorkerMessageHandler(app, notices);
    onWorkerMessage({ data: { action: 'operation_error', operation: 'load_model', id: 8, error: 'Stop generation before loading a stack artifact.' } });
    assert.equal(app.currentLoadId, 7);
    assert.equal(app.readyLoadId, 7);
    assert.equal(app.workerState, 'ready');
    assert.deepEqual(notices, [{ text: 'Stop generation before loading a stack artifact.', tone: 'warn' }]);
    const ready = !(app.workerState !== 'ready' || (app.currentLoadId !== undefined && app.readyLoadId !== app.currentLoadId));
    assert.equal(ready, true);
    onWorkerMessage({ data: { action: 'model_loaded', id: 8, capabilities: { file_name: 'stale.lut', artifact_sha256: '0'.repeat(64) } } });
    assert.deepEqual(app.caps, { schema: 'healthy' });
    assert.equal(app.readyLoadId, 7);
});

function healthyModelFixture() {
    const caps = { schema: 'uor-r4.lut-stack/1', file_name: 'healthy.lut', artifact_sha256: 'a'.repeat(64), sha256_verified: true, artifact_bytes: 1024, tokenizer_label: 'healthy-tokenizer', source: 'file', source_label: 'healthy', synthetic: true, can_generate: true, engine_supported: true, status_label: 'synthetic stand-in', engine: 'serving-wasm' };
    const meta = { kind: 'local-file', label: 'healthy', fileName: 'healthy.lut' };
    const tokenizer = { mode: 'byte-bpe', vocab: 17 };
    const dataset = globalThis.document.documentElement.dataset;
    Object.keys(dataset).forEach((key) => { delete dataset[key]; });
    Object.assign(dataset, {
        modelSchema: caps.schema,
        modelSha256: caps.artifact_sha256,
        modelShaVerified: 'true',
        modelSize: '1024',
        modelTokenizer: caps.tokenizer_label,
        modelSource: 'file',
        modelSourceLabel: 'healthy',
        modelDiscrete: 'false',
        modelLoadMs: '12',
    });
    const app = {
        generating: false,
        mode: 'wasm',
        caps,
        modelMeta: meta,
        tokenizerArtifact: tokenizer,
        loadMs: 12,
        readyLoadId: 6,
        currentLoadId: 6,
        requestedFileName: 'healthy.lut',
        expectedSha256: 'a'.repeat(64),
        workerState: 'ready',
        worker: null,
        apiConfig: null,
        predictionRequests: {},
        healthyModelSnapshot: null,
        modelStatusLog: [],
    };
    return { app, caps, meta, tokenizer, dataset };
}

test('a stop-generation load rejection restores the state captured before the source change', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const { app, caps, meta, tokenizer, dataset } = healthyModelFixture();
    const notices = [];
    const page = await installPageSourceHandlers(harness.api, app, harness, notices);
    page.handle({ detail: { kind: 'wasm', bytes: new Uint8Array([1]), fileName: 'new.lut', meta: { kind: 'local-file', label: 'new' }, expectedSha256: '', tokenizerArtifact: null } });
    assert.equal(app.caps, null);
    assert.ok(app.healthyModelSnapshot, 'the healthy state must be captured before the source change clears it');
    assert.deepEqual(app.healthyModelSnapshot.caps, caps);
    const newLoadId = app.currentLoadId;
    const onWorkerMessage = await installWorkerMessageHandler(app, notices);
    onWorkerMessage({ data: { action: 'operation_error', operation: 'load_model', id: newLoadId, error: 'Stop generation before loading a stack artifact.' } });
    assert.deepEqual(app.caps, caps);
    assert.deepEqual(app.modelMeta, meta);
    assert.deepEqual(app.tokenizerArtifact, tokenizer);
    assert.equal(app.loadMs, 12);
    assert.equal(app.currentLoadId, 6);
    assert.equal(app.readyLoadId, 6);
    assert.equal(app.workerState, 'ready');
    assert.equal(app.healthyModelSnapshot, null);
    assert.deepEqual(dataset, {
        modelSchema: caps.schema,
        modelSha256: caps.artifact_sha256,
        modelShaVerified: 'true',
        modelSize: '1024',
        modelTokenizer: caps.tokenizer_label,
        modelSource: 'file',
        modelSourceLabel: 'healthy',
        modelDiscrete: 'false',
        modelLoadMs: '12',
    });
    const ready = !(app.workerState !== 'ready' || (app.currentLoadId !== undefined && app.readyLoadId !== app.currentLoadId));
    assert.equal(ready, true);
    onWorkerMessage({ data: { action: 'model_loaded', id: newLoadId, capabilities: { ...caps, file_name: 'new.lut', artifact_sha256: 'b'.repeat(64) } } });
    assert.deepEqual(app.caps, caps);
    assert.equal(app.readyLoadId, 6);
});

test('a different load error after a source change does not restore the old model', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const { app } = healthyModelFixture();
    const notices = [];
    const page = await installPageSourceHandlers(harness.api, app, harness, notices);
    page.handle({ detail: { kind: 'wasm', bytes: new Uint8Array([1]), fileName: 'new.lut', meta: { kind: 'local-file', label: 'new' }, expectedSha256: '', tokenizerArtifact: null } });
    const newLoadId = app.currentLoadId;
    const onWorkerMessage = await installWorkerMessageHandler(app, notices);
    onWorkerMessage({ data: { action: 'operation_error', operation: 'load_model', id: newLoadId, error: 'The artifact header is invalid.' } });
    assert.equal(app.caps, null);
    assert.equal(app.modelMeta, null);
    assert.equal(app.healthyModelSnapshot, null);
    assert.equal(app.workerState, 'error');
    assert.equal(app.currentLoadId, newLoadId);
});

test('a successful new load clears the captured state instead of restoring it', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const { app, dataset } = healthyModelFixture();
    const notices = [];
    const page = await installPageSourceHandlers(harness.api, app, harness, notices);
    page.handle({ detail: { kind: 'wasm', bytes: new Uint8Array([1]), fileName: 'new.lut', meta: { kind: 'local-file', label: 'new' }, expectedSha256: '', tokenizerArtifact: null } });
    const newLoadId = app.currentLoadId;
    const onWorkerMessage = await installWorkerMessageHandler(app, notices);
    const newCaps = { schema: 'uor-r4.lut-stack/2', file_name: 'new.lut', artifact_sha256: 'b'.repeat(64), sha256_verified: false, artifact_bytes: 2048, tokenizer_label: 'new-tokenizer', source: 'url', source_label: 'new', synthetic: false };
    onWorkerMessage({ data: { action: 'model_loaded', id: newLoadId, capabilities: newCaps } });
    assert.deepEqual(app.caps, newCaps);
    assert.equal(app.readyLoadId, newLoadId);
    assert.equal(app.workerState, 'ready');
    assert.equal(app.healthyModelSnapshot, null);
    assert.equal(dataset.modelSchema, newCaps.schema);
    assert.equal(dataset.modelSource, 'url');
});

test('a second overlapping source change keeps the first healthy snapshot', async () => {
    const harness = await fresh(async () => { throw new Error('network must not run'); });
    const { app, caps } = healthyModelFixture();
    const notices = [];
    const page = await installPageSourceHandlers(harness.api, app, harness, notices);
    page.handle({ detail: { kind: 'wasm', bytes: new Uint8Array([1]), fileName: 'first.lut', meta: { kind: 'local-file', label: 'first' }, expectedSha256: '', tokenizerArtifact: null } });
    const firstSnapshot = app.healthyModelSnapshot;
    assert.ok(firstSnapshot, 'the first load must capture the healthy snapshot');
    page.handle({ detail: { kind: 'wasm', bytes: new Uint8Array([2]), fileName: 'second.lut', meta: { kind: 'local-file', label: 'second' }, expectedSha256: '', tokenizerArtifact: null } });
    assert.equal(app.healthyModelSnapshot, firstSnapshot, 'the second overlapping load must retain the first snapshot');
    const onWorkerMessage = await installWorkerMessageHandler(app, notices);
    onWorkerMessage({ data: { action: 'operation_error', operation: 'load_model', id: app.currentLoadId, error: 'Stop generation before loading a stack artifact.' } });
    assert.deepEqual(app.caps, caps);
    assert.equal(app.currentLoadId, 6);
    assert.equal(app.readyLoadId, 6);
});

test('restoring the healthy model updates the visible status label and readiness', async () => {
    const { app } = healthyModelFixture();
    const capture = await pageFunction('function captureHealthyModelState(', 'captureHealthyModelState', { app, document: globalThis.document });
    app.healthyModelSnapshot = capture();
    assert.ok(app.healthyModelSnapshot, 'the healthy state must be captured for the restore');
    app.caps = null;
    app.currentLoadId = 7;
    const notices = [];
    const onWorkerMessage = await installWorkerMessageHandler(app, notices);
    onWorkerMessage({ data: { action: 'operation_error', operation: 'load_model', id: 7, error: 'Stop generation before loading a stack artifact.' } });
    assert.equal(app.currentLoadId, 6);
    assert.deepEqual(app.modelStatusLog.at(-1), { text: 'synthetic stand-in · uor-r4.lut-stack/1', tone: 'warn', detail: '' });
    const chatReadiness = await pageFunction('function chatReadiness(', 'chatReadiness', {
        app,
        normalizeBaseUrl() { return 'http://local'; },
        isDiscreteCapabilities() { return false; },
    });
    assert.equal(chatReadiness(), true);
});
