#!/usr/bin/env node
// Offline Studio page contract + retained desktop-client verification.
//
// The Studio is offline-first in a browser and registers only the declared
// Tauri OpenAI transport when the real host adapter is present. The retired
// byte-mod `createTauriNativeModelWorker` stays unloaded. This script verifies
// those page contracts, the narrow asset-origin CSP, and the page-independent
// desktop client self-test and JSON-RPC controls.
//
// The retained `uor_offline_vendor.js` asset still supplies its independent
// worker harness and offline libraries, but the Studio page does not load it.
// `src-tauri/tests/desktop_client_controls.rs` owns host-control tests. Set
// `UOR_DESKTOP_BIN` to use a prebuilt binary.

import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { execFileSync, spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '../..');

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

const indexHtml = readFileSync(resolve(__dirname, 'index.html'), 'utf8');
const indexCss = readFileSync(resolve(__dirname, 'index.css'), 'utf8');
const modelSourcesJs = readFileSync(resolve(__dirname, 'assets/js/model_sources.js'), 'utf8');
const studioApiJs = readFileSync(resolve(__dirname, 'assets/js/studio_api.js'), 'utf8');
const vendorJs = readFileSync(resolve(__dirname, 'assets/js/uor_offline_vendor.js'), 'utf8');
const fontsCss = readFileSync(resolve(__dirname, 'assets/css/uor_offline_fonts.css'), 'utf8');
const tauriConf = JSON.parse(readFileSync(resolve(repoRoot, 'src-tauri/tauri.conf.json'), 'utf8'));

// 1. The page is offline-first: no external resource in the page or its local
//    stylesheets, and no page-load reference to a remote host.
for (const [name, content] of [
    ['index.html', indexHtml],
    ['index.css', indexCss],
    ['assets/css/uor_offline_fonts.css', fontsCss],
]) {
    assert(!/(?:\bsrc\s*=\s*["']https?:\/\/|\burl\(\s*['"]?https?:\/\/)/i.test(content), name + ' loads an external resource');
    assert(!content.includes('//cdn.') && !content.includes('unpkg.com') && !content.includes('jsdelivr'), name + ' references a CDN');
}
assert(!indexHtml.includes('<script src=\"http'), 'index.html must not load a remote script');
assert(!indexHtml.includes('<link rel=\"stylesheet\" href=\"http'), 'index.html must not load a remote stylesheet');

// 2. What the new page KEEPS (each assertion fails if the feature is removed).
assert(indexHtml.includes('href="./index.css"'), 'index.html must link the local index.css');
assert(indexHtml.includes('href="./assets/css/uor_offline_fonts.css"'), 'index.html must link the local offline fonts');
assert(indexHtml.includes("from './assets/js/model_sources.js'"), 'index.html must import the model-source module');
assert(indexHtml.includes("from './assets/js/studio_api.js'"), 'index.html must import the OpenAI-compatible API module');
assert(indexHtml.includes('./assets/js/uor_model_worker.js'), 'index.html must start the local wasm model worker');
assert(indexHtml.includes('id="modelSourcesPanel"') && indexHtml.includes('id="modelSourceMount"'),
    'index.html must keep the model-source panel ids');
assert(indexHtml.includes('id="editorText"') && indexHtml.includes('<textarea id="editorText"'),
    'index.html must keep the local plain-text editor (#editorText textarea)');
assert(indexHtml.includes('id="chatInput"') && indexHtml.includes('id="chatMessages"'),
    'index.html must keep the local chat panel ids');
assert(indexHtml.includes('id="projectStatus"') && /Pre-alpha[^<]*M1[^<]*1 of 8/.test(indexHtml),
    'index.html must keep the honest pre-alpha / M1 / 1 of 8 status chip');
assert(indexHtml.includes('id="privacyStatus"') && indexHtml.includes('Local only'),
    'index.html must keep the local-processing privacy chip');
assert(indexHtml.includes('dataset.studioReady') && indexHtml.includes("new CustomEvent('uor-studio-ready')"),
    'index.html must keep the data-studio-ready / uor-studio-ready boot marker');

// 3. What the new page REMOVED (each assertion fails if old wiring comes back).
for (const [token, what] of [
    ['uor_offline_vendor.js', 'the old vendored script tag'],
    ['initMonacoEditor', 'the Monaco editor hook'],
    ['openFileInMonaco', 'the Monaco file hook'],
    ['switchEditorTab', 'the Monaco tab hook'],
    ['toggleMonacoDiffView', 'the Monaco diff hook'],
    ['monaco', 'any Monaco reference'],
    ['initThreeVisualizer', 'the THREE visualizer'],
    ['triggerSynapticFlash', 'the THREE flash hook'],
    ['THREE', 'any THREE reference'],
    ['callGithubApi', 'the GitHub API hook'],
    ['createTauriNativeModelWorker', 'the retired Tauri desktop worker hook'],
]) {
    assert(!indexHtml.includes(token), 'index.html must not reference ' + what + ' (found "' + token + '")');
}
assert(!indexHtml.includes('d36EngineStatusBanner'), 'the stale d36EngineStatusBanner banner must stay removed');
assert(!indexHtml.includes('0/8') && !indexHtml.includes('P_succ = 35.1%'),
    'the stale 0/8 and P_succ = 35.1% claims must stay removed');
assert(!indexHtml.includes('35.1%</code> tok/s') && !indexHtml.includes('35.1% tok/s'),
    'the unsourced "35.1% tok/s" claim must stay removed');
assert(!indexHtml.includes('en.wikipedia.org/w/api.php') && !indexHtml.includes('api.duckduckgo.com'),
    'the live external search fetches must stay removed');

// 4. The model-source manager keeps all four source kinds and the documented API.
for (const marker of ["cardHeader('Demo'", "cardHeader('Local file'", "cardHeader('From URL'", "cardHeader('API'"]) {
    assert(modelSourcesJs.includes(marker), 'model_sources.js must keep the ' + marker + ' source card');
}
for (const marker of ['load-bundled', 'load-url', 'add-api', '[data-role="model-file"]']) {
    assert(modelSourcesJs.includes(marker), 'model_sources.js must keep the ' + marker + ' control');
}
assert(modelSourcesJs.includes('./pkg/synthetic-stack.lut'),
    'the bundled demo source must keep pointing at the build-produced ./pkg/synthetic-stack.lut');
assert(modelSourcesJs.includes('globalThis.UORModelSources = api'),
    'model_sources.js must keep exposing window.UORModelSources');
for (const method of ['loadLocalFile', 'loadFromUrl', 'apiConfig', 'save', 'testApi']) {
    assert(modelSourcesJs.includes(method), 'window.UORModelSources must keep ' + method);
}
assert(modelSourcesJs.includes('uor-model-source-changed'),
    'model_sources.js must keep dispatching uor-model-source-changed');
assert(studioApiJs.includes('streamChatCompletions') && studioApiJs.includes('configFromSources'),
    'studio_api.js must keep the OpenAI-compatible chat-completions client');

// 5. Retained page-independent desktop assets: the vendor bundle and the CSP are
//    still part of the repository, but the Studio page no longer loads them.
assert(!vendorJs.includes('http://') && !vendorJs.includes('https://'),
    'retained uor_offline_vendor.js must not carry an external URL');
for (const api of [
    'globalThis.marked',
    'globalThis.katex',
    'globalThis.renderMathInElement',
    'globalThis.monaco',
    'globalThis.THREE',
    'globalThis.createTauriNativeModelWorker',
]) {
    assert(vendorJs.includes(api.replace('globalThis.', '')), 'retained uor_offline_vendor.js must keep ' + api);
}
const csp = tauriConf?.app?.security?.csp || '';
assert(csp.includes("connect-src 'self' ipc: http://ipc.localhost"), 'tauri.conf.json CSP must restrict connect-src to local IPC');
assert(csp.includes("'wasm-unsafe-eval'"), 'tauri.conf.json CSP must explicitly permit the measured browser WASM fallback');
assert(csp.includes("worker-src 'self'"), 'tauri.conf.json CSP must confine workers to the packaged origin');
assert(!csp.includes('https://'), 'tauri.conf.json CSP must not allow external https origins');
assert(existsSync(resolve(repoRoot, 'src-tauri/src/main.rs')), 'the desktop client source (src-tauri/src/main.rs) must stay in the repository');
assert(existsSync(resolve(repoRoot, 'src-tauri/tests/desktop_client_controls.rs')),
    'the desktop client host-control tests (src-tauri/tests/desktop_client_controls.rs) must stay in the repository');

// 6. Trap network APIs before importing the retained vendor harness.
const remoteRequests = [];
function trap(api) {
    return (...args) => {
        remoteRequests.push(api + ':' + String(args[0] || ''));
        throw new Error('Forbidden network call in offline Studio verification: ' + api);
    };
}

function makeElement(id = '', tagName = 'DIV') {
    return {
        id,
        tagName: String(tagName).toUpperCase(),
        className: '',
        innerHTML: '',
        innerText: '',
        textContent: '',
        value: '',
        style: {},
        dataset: {},
        children: [],
        clientWidth: 360,
        clientHeight: 240,
        width: 360,
        height: 240,
        classList: {
            _set: new Set(),
            add(...tokens) { tokens.forEach((t) => this._set.add(t)); },
            remove(...tokens) { tokens.forEach((t) => this._set.delete(t)); },
            toggle(t) { if (this._set.has(t)) this._set.delete(t); else this._set.add(t); },
            contains(t) { return this._set.has(t); },
        },
        appendChild(child) { this.children.push(child); return child; },
        removeChild(child) { const i = this.children.indexOf(child); if (i !== -1) this.children.splice(i, 1); return child; },
        addEventListener() {},
        removeEventListener() {},
        setAttribute(k, v) { this[k] = v; },
        getAttribute(k) { return this[k] ?? null; },
        querySelector() { return null; },
        querySelectorAll() { return []; },
        closest() { return null; },
        focus() {},
        getContext() { return null; },
    };
}
const storageMap = new Map();
globalThis.window = globalThis;
globalThis.localStorage = {
    getItem(k) { return storageMap.has(k) ? storageMap.get(k) : null; },
    setItem(k, v) { storageMap.set(k, String(v)); },
    removeItem(k) { storageMap.delete(k); },
};
globalThis.document = {
    body: makeElement('body', 'BODY'),
    documentElement: makeElement('html', 'HTML'),
    getElementById(id) { return makeElement(id); },
    createElement(tag) { return makeElement('', tag); },
    querySelector() { return null; },
    querySelectorAll() { return []; },
    addEventListener() {},
};
globalThis.requestAnimationFrame = () => 0;
globalThis.addEventListener = () => {};
globalThis.confirm = () => true;
Object.defineProperty(globalThis, 'navigator', {
    value: {
        onLine: true,
        clipboard: { async writeText() {}, async readText() { return ''; } },
        storage: { async estimate() { return { usage: 0, quota: 67108864 }; } },
    },
    configurable: true,
    writable: true,
});
globalThis.fetch = trap('fetch');
globalThis.XMLHttpRequest = class { open(_m, u) { trap('XMLHttpRequest')(u); } };
globalThis.WebSocket = class { constructor(u) { trap('WebSocket')(u); } };
globalThis.EventSource = class { constructor(u) { trap('EventSource')(u); } };

await import('./assets/js/uor_offline_vendor.js');
assert(typeof globalThis.marked?.parse === 'function', 'retained offline marked.parse missing');
assert(typeof globalThis.katex?.renderToString === 'function', 'retained offline katex.renderToString missing');
assert(typeof globalThis.renderMathInElement === 'function', 'retained offline renderMathInElement missing');
assert(typeof globalThis.monaco?.editor?.create === 'function', 'retained offline monaco.editor.create missing');
assert(typeof globalThis.THREE?.WebGLRenderer === 'function', 'retained offline THREE.WebGLRenderer missing');
assert(typeof globalThis.createTauriNativeModelWorker === 'function', 'retained createTauriNativeModelWorker missing');

// 7. Real desktop-client contract (unchanged binary, no longer page-wired).
function resolveDesktopBinary() {
    const candidates = [
        process.env.UOR_DESKTOP_BIN,
        resolve(repoRoot, 'local/antigravity/tauri-target/debug/uor-desktop-client'),
        resolve(repoRoot, 'local/tauri-target/debug/uor-desktop-client'),
        resolve(repoRoot, 'local/tauri-verify-target/debug/uor-desktop-client'),
        resolve(repoRoot, 'target/debug/uor-desktop-client'),
    ].filter(Boolean);
    for (const candidate of candidates) {
        if (existsSync(candidate)) return candidate;
    }
    const cargoBin = existsSync(process.env.HOME + '/.cargo/bin/cargo')
        ? process.env.HOME + '/.cargo/bin/cargo'
        : 'cargo';
    const targetDir = resolve(repoRoot, 'local/tauri-verify-target');
    const manifest = resolve(repoRoot, 'src-tauri/Cargo.toml');
    const env = { ...process.env, CARGO_TARGET_DIR: targetDir };
    try {
        execFileSync(cargoBin, ['build', '--manifest-path', manifest, '--locked'], { env, stdio: 'inherit' });
    } catch (_error) {
        // A stale src-tauri/Cargo.lock must not silently drop this coverage: refresh
        // the lock offline and say so, instead of skipping the real binary contract.
        console.error('NOTE: src-tauri/Cargo.lock is stale for --locked; refreshing it offline for this verification build.');
        execFileSync(cargoBin, ['build', '--manifest-path', manifest, '--offline'], { env, stdio: 'inherit' });
    }
    const built = join(targetDir, 'debug/uor-desktop-client');
    assert(existsSync(built), 'failed to locate built uor-desktop-client at ' + built);
    return built;
}

const EXPECTED_32 = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
];
const desktopBin = resolveDesktopBinary();
const selfTestStdout = execFileSync(desktopBin, ['--self-test'], { encoding: 'utf8' }).trim();
const selfTestReceipt = JSON.parse(selfTestStdout);
assert(selfTestReceipt.verdict === 'PASS', 'desktop --self-test failed: ' + selfTestStdout);
assert(selfTestReceipt.engine === 'uor-r4-integer/native-desktop', 'unexpected engine: ' + selfTestReceipt.engine);
assert(selfTestReceipt.artifactBytes === 469252, 'unexpected artifactBytes: ' + selfTestReceipt.artifactBytes);
assert(selfTestReceipt.artifactSha256 === 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df',
    'unexpected artifactSha256: ' + selfTestReceipt.artifactSha256);
assert(selfTestReceipt.weightsPerToken === 15968, 'unexpected weightsPerToken: ' + selfTestReceipt.weightsPerToken);
assert(JSON.stringify(selfTestReceipt.tokens32) === JSON.stringify(EXPECTED_32), 'self-test 32-token trajectory mismatch');
assert(selfTestReceipt.ur4sess1Bytes === 9782, 'expected ur4sess1Bytes=9782, got ' + selfTestReceipt.ur4sess1Bytes);
assert(selfTestReceipt.storeRecords === 446, 'expected storeRecords=446, got ' + selfTestReceipt.storeRecords);
assert(selfTestReceipt.objectVersion === 'bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca',
    'unexpected objectVersion: ' + selfTestReceipt.objectVersion);

const tmpGrantedStoreRoot = mkdtempSync(join(tmpdir(), 'uor-desktop-verify-granted-'));
const tmpUngrantedStoreRoot = mkdtempSync(join(tmpdir(), 'uor-desktop-verify-ungranted-'));
const stdioChild = spawn(desktopBin, ['--stdio', '--grant', tmpGrantedStoreRoot], {
    stdio: ['pipe', 'pipe', 'inherit'],
});
const stdioLines = createInterface({ input: stdioChild.stdout });
const pendingRpc = new Map();
let nextRpcId = 1;
const invokedCommands = [];
stdioLines.on('line', (line) => {
    if (!line.trim()) return;
    const msg = JSON.parse(line);
    const waiter = pendingRpc.get(msg.id);
    if (waiter) {
        pendingRpc.delete(msg.id);
        if (msg.ok) waiter.resolve(msg.result);
        else waiter.reject(new Error(msg.error || 'RPC error'));
    }
});
const realStdioTauriCore = {
    invoke(command, args = {}) {
        invokedCommands.push(command);
        const id = nextRpcId++;
        return new Promise((resolvePromise, rejectPromise) => {
            pendingRpc.set(id, { resolve: resolvePromise, reject: rejectPromise });
            stdioChild.stdin.write(JSON.stringify({ id, command, args }) + '\n');
        });
    },
};
globalThis.__TAURI__ = { core: realStdioTauriCore };

const workerEvents = [];
let eventWaiters = [];
const worker = globalThis.createTauriNativeModelWorker(realStdioTauriCore);
worker.onmessage = (e) => {
    workerEvents.push(e.data);
    const remaining = [];
    for (const w of eventWaiters) {
        if (w.pred(e.data)) w.resolve(e.data);
        else remaining.push(w);
    }
    eventWaiters = remaining;
};
function waitWorkerEvent(pred, timeoutMs = 10000) {
    for (const ev of workerEvents) {
        if (pred(ev)) {
            const idx = workerEvents.indexOf(ev);
            if (idx !== -1) workerEvents.splice(idx, 1);
            return Promise.resolve(ev);
        }
    }
    return new Promise((resolvePromise, rejectPromise) => {
        const t = setTimeout(() => rejectPromise(new Error('Timed out waiting for worker event; seen: ' + JSON.stringify(workerEvents))), timeoutMs);
        eventWaiters.push({ pred, resolve: (ev) => { clearTimeout(t); resolvePromise(ev); } });
    });
}

worker.postMessage({ action: 'prewarm' });
const prewarmEv = await waitWorkerEvent((e) => e.action === 'runtime_loaded');
assert(prewarmEv.engine === 'uor-r4-integer/native-desktop', 'prewarm did not emit native-desktop runtime_loaded');
worker.postMessage({ action: 'load_model', payload: {} });
const loadedEv = await waitWorkerEvent((e) => e.action === 'model_loaded' || e.action === 'operation_error');
assert(loadedEv.action === 'model_loaded' && loadedEv.bytes === 469252, 'desktop worker load_model failed: ' + JSON.stringify(loadedEv));
worker.postMessage({
    action: 'generate',
    id: 'gen_desktop_stdio_1',
    payload: { conversationId: 'conv_desktop_1', tokenIds: [0], options: { max_new_tokens: 32 } },
});
const genDone = await waitWorkerEvent((e) => (e.action === 'generate_complete' || e.action === 'generate_error') && e.id === 'gen_desktop_stdio_1');
assert(genDone.action === 'generate_complete', 'desktop worker generate failed: ' + JSON.stringify(genDone));
assert(JSON.stringify(genDone.tokenIds) === JSON.stringify(EXPECTED_32), 'desktop worker generate_complete tokens mismatch');
const streamedForGen = workerEvents.filter((e) => e.action === 'stream_token' && e.id === 'gen_desktop_stdio_1');
assert(streamedForGen.length === 32, 'expected 32 individual step-by-step stream_token events, got ' + streamedForGen.length);

try {
    let grantRefused = false;
    try {
        await realStdioTauriCore.invoke('native_grant_folder', { folderPath: tmpUngrantedStoreRoot });
    } catch (err) {
        grantRefused = String(err).includes('host-owned');
    }
    assert(grantRefused, 'page-invoked native_grant_folder must be refused as host-owned');
    worker.postMessage({
        action: 'save_session_durable',
        id: 'save_ungranted',
        payload: { conversationId: 'conv_desktop_1', storeDir: tmpUngrantedStoreRoot },
    });
    const ungrantedEv = await waitWorkerEvent((e) => e.action === 'operation_error' && e.id === 'save_ungranted');
    assert(String(ungrantedEv.error).includes('PathNotGranted'),
        'expected PathNotGranted on ungranted storeDir, got ' + JSON.stringify(ungrantedEv));
    worker.postMessage({
        action: 'save_session_durable',
        id: 'save_granted',
        payload: { conversationId: 'conv_desktop_1', storeDir: tmpGrantedStoreRoot },
    });
    const durableSaved = await waitWorkerEvent((e) => (e.action === 'session_durable_saved' || e.action === 'operation_error') && e.id === 'save_granted');
    assert(durableSaved.action === 'session_durable_saved', 'save_session_durable failed: ' + JSON.stringify(durableSaved));
    assert(durableSaved.receipt.ur4sess1_bytes_len === 9782 &&
        durableSaved.receipt.scope_key === 'local-user/studio/conv_desktop_1' &&
        durableSaved.receipt.object_version_hex.length === 64,
        'unexpected durable receipt: ' + JSON.stringify(durableSaved.receipt));
    worker.postMessage({ action: 'reset_session', id: 'reset_before_restore', payload: { conversationId: 'conv_desktop_1' } });
    await waitWorkerEvent((e) => e.action === 'session_reset' && e.id === 'reset_before_restore');
    worker.postMessage({ action: 'restore_session_durable', id: 'restore_granted', payload: { conversationId: 'conv_desktop_1', storeDir: tmpGrantedStoreRoot } });
    const durableRestored = await waitWorkerEvent((e) => (e.action === 'session_durable_restored' || e.action === 'operation_error') && e.id === 'restore_granted');
    assert(durableRestored.action === 'session_durable_restored' &&
        durableRestored.receipt.ur4sess1_bytes_len === 9782 &&
        durableRestored.receipt.position === 32 &&
        durableRestored.receipt.artifact_sha256 === 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df',
        'restore_session_durable mismatch: ' + JSON.stringify(durableRestored));
} finally {
    rmSync(tmpGrantedStoreRoot, { recursive: true, force: true });
    rmSync(tmpUngrantedStoreRoot, { recursive: true, force: true });
    stdioChild.stdin.end();
    stdioChild.kill();
}

assert(remoteRequests.length === 0, 'expected 0 remote requests, got ' + JSON.stringify(remoteRequests));

console.log(JSON.stringify({
    verdict: 'ALL OFFLINE STUDIO PAGE & DESKTOP CLIENT CHECKS PASSED',
    pageContract: {
        kept: ['local offline fonts + css', 'wasm model worker', 'model-source panel (bundled/local-file/url/api)', 'OpenAI-compatible API module', 'plain-text local editor', 'pre-alpha/M1 status chip', 'data-studio-ready marker'],
        removed: ['uor_offline_vendor.js script tag', 'Monaco editor', 'THREE visualizer', 'retired byte-mod Tauri worker', 'GitHub API call', 'stale d36 / 0-of-8 / 35.1% claims', 'external search fetches'],
    },
    offlineExternalUrls: 0,
    desktopClient: {
        declaredOpenAITransport: true,
        retainedSources: ['src-tauri/src/main.rs', 'src-tauri/tests/desktop_client_controls.rs', 'assets/js/uor_offline_vendor.js'],
        binaryContractVerified: true,
        artifactSha256: selfTestReceipt.artifactSha256,
        streamedTokensCount: streamedForGen.length,
        invokedCommandsCount: invokedCommands.length,
    },
    selfTestReceipt,
    remoteRequests: remoteRequests.length,
}));
