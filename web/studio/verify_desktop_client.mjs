#!/usr/bin/env node
// Automated offline & real-IPC verification for the native desktop Studio
// (`src-tauri/` + `web/studio/index.html` + `web/studio/assets/js/uor_offline_vendor.js`).
//
// Exercises:
// 1. Static offline invariants (`0` external http(s) URLs across Studio HTML/JS/CSS and CSP).
// 2. Real `uor-desktop-client --self-test` and real `uor-desktop-client --stdio` child-process
//    JSON-RPC bridge (never skipped, zero mock `invoke` returning canned tokens).
// 3. Step-by-step token streaming (`32` individual `stream_token` events matching `EXPECTED_32`),
//    granted-folder durable store confinement (`PathNotGranted` on ungranted `storeDir`), and
//    durable `uor-r4-object-store` close/reopen session continuation.
// 4. Actual `web/studio/index.html` script functions (`initMonacoEditor`, `openFileInMonaco`,
//    `switchEditorTab`, `toggleMonacoDiffView`, `renderRichMarkdown`, `initThreeVisualizer`,
//    `triggerSynapticFlash`, `callGithubApi`) evaluated against `uor_offline_vendor.js`.

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

// 1. Verify offline static invariants on web/studio/index.html and vendor assets.
const indexHtml = readFileSync(resolve(__dirname, 'index.html'), 'utf8');
const vendorJs = readFileSync(resolve(__dirname, 'assets/js/uor_offline_vendor.js'), 'utf8');
const fontsCss = readFileSync(resolve(__dirname, 'assets/css/uor_offline_fonts.css'), 'utf8');
const tauriConf = JSON.parse(readFileSync(resolve(repoRoot, 'src-tauri/tauri.conf.json'), 'utf8'));

for (const [name, content] of [
    ['index.html', indexHtml],
    ['uor_offline_vendor.js', vendorJs],
    ['uor_offline_fonts.css', fontsCss],
]) {
    assert(!content.includes('http://') && !content.includes('https://'), `${name} contains external http(s) URL`);
}

assert(indexHtml.includes('./assets/css/uor_offline_fonts.css'), 'index.html must link local uor_offline_fonts.css');
assert(indexHtml.includes('./assets/js/uor_offline_vendor.js'), 'index.html must load local uor_offline_vendor.js');
assert(indexHtml.includes('createTauriNativeModelWorker'), 'index.html must wire createTauriNativeModelWorker in desktop mode');
assert(indexHtml.includes("invoke('native_list_granted_folders'"), 'index.html must query host-granted folders before git/file access');
assert(!indexHtml.includes("invoke('native_grant_folder'"), 'index.html must not self-grant folders from page script');
assert(indexHtml.includes("invoke('native_list_directory'"), 'index.html must list workspace files via native_list_directory');
assert(
    indexHtml.includes('userConfirmed: true') && indexHtml.includes('confirmedCommand: cmd'),
    'index.html must pass userConfirmed and confirmedCommand to native_run_terminal_command',
);
assert(
    indexHtml.includes("action: 'save_session_durable'"),
    'index.html must persist conversations via save_session_durable in desktop mode',
);

const csp = tauriConf?.app?.security?.csp || '';
assert(csp.includes("connect-src 'self' ipc: http://ipc.localhost"), 'tauri.conf.json CSP must restrict connect-src to local IPC');
assert(!csp.includes('https://'), 'tauri.conf.json CSP must not allow external https origins');

// 2. Resolve or build the real `uor-desktop-client` binary (never skip).
function resolveDesktopBinary() {
    const candidates = [
        process.env.UOR_DESKTOP_BIN,
        resolve(repoRoot, 'local/antigravity/tauri-target/debug/uor-desktop-client'),
        resolve(repoRoot, 'local/tauri-target/debug/uor-desktop-client'),
        resolve(repoRoot, 'target/debug/uor-desktop-client'),
    ].filter(Boolean);

    for (const candidate of candidates) {
        if (existsSync(candidate)) return candidate;
    }

    const cargoBin = existsSync(`${process.env.HOME}/.cargo/bin/cargo`)
        ? `${process.env.HOME}/.cargo/bin/cargo`
        : 'cargo';
    const targetDir = resolve(repoRoot, 'local/tauri-verify-target');
    execFileSync(
        cargoBin,
        ['build', '--manifest-path', resolve(repoRoot, 'src-tauri/Cargo.toml'), '--locked'],
        {
            env: { ...process.env, CARGO_TARGET_DIR: targetDir },
            stdio: 'inherit',
        },
    );
    const built = join(targetDir, 'debug/uor-desktop-client');
    assert(existsSync(built), `failed to locate built uor-desktop-client at ${built}`);
    return built;
}

const desktopBin = resolveDesktopBinary();

// 3. Trap all network APIs and set up a lightweight DOM harness to evaluate both
// `uor_offline_vendor.js` and the `<script type="module">` body from `web/studio/index.html`.
const remoteRequests = [];
function trap(api) {
    return (...args) => {
        remoteRequests.push(`${api}:${String(args[0] || '')}`);
        throw new Error(`Forbidden network call in offline desktop Studio: ${api}`);
    };
}

const domElements = new Map();
function makeElement(id = '', tagName = 'DIV') {
    const el = {
        id,
        tagName: tagName.toUpperCase(),
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
        appendChild(child) {
            this.children.push(child);
            return child;
        },
        removeChild(child) {
            const idx = this.children.indexOf(child);
            if (idx !== -1) this.children.splice(idx, 1);
            return child;
        },
        addEventListener(_ev, _fn) {},
        removeEventListener(_ev, _fn) {},
        setAttribute(k, v) { this[k] = v; },
        getAttribute(k) { return this[k] ?? null; },
        querySelector() { return null; },
        querySelectorAll() { return []; },
        closest() { return null; },
        focus() {},
        getContext(kind) {
            if (kind === '2d') {
                return {
                    fillStyle: '',
                    strokeStyle: '',
                    lineWidth: 1,
                    shadowColor: '',
                    shadowBlur: 0,
                    clearRect() {},
                    fillRect() {},
                    beginPath() {},
                    moveTo() {},
                    lineTo() {},
                    stroke() {},
                    createRadialGradient() {
                        return { addColorStop() {} };
                    },
                };
            }
            return null;
        },
    };
    return el;
}

const domContentLoadedHandlers = [];
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
    getElementById(id) {
        if (!domElements.has(id)) {
            domElements.set(id, makeElement(id));
        }
        return domElements.get(id);
    },
    createElement(tag) {
        return makeElement('', tag);
    },
    querySelector() {
        return null;
    },
    querySelectorAll() {
        return [];
    },
    addEventListener(type, fn) {
        if (type === 'DOMContentLoaded' && typeof fn === 'function') {
            domContentLoadedHandlers.push(fn);
        }
    },
};
globalThis.requestAnimationFrame = () => 0;
globalThis.addEventListener = (type, fn) => {
    if (type === 'DOMContentLoaded' && typeof fn === 'function') {
        domContentLoadedHandlers.push(fn);
    }
};
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

assert(typeof globalThis.marked?.parse === 'function', 'offline marked.parse missing');
assert(typeof globalThis.katex?.renderToString === 'function', 'offline katex.renderToString missing');
assert(typeof globalThis.renderMathInElement === 'function', 'offline renderMathInElement missing');
assert(typeof globalThis.monaco?.editor?.create === 'function', 'offline monaco.editor.create missing');
assert(typeof globalThis.monaco?.editor?.setModelLanguage === 'function', 'offline monaco.editor.setModelLanguage missing');
assert(typeof globalThis.monaco?.editor?.createDiffEditor === 'function', 'offline monaco.editor.createDiffEditor missing');
assert(typeof globalThis.THREE?.WebGLRenderer === 'function', 'offline THREE.WebGLRenderer missing');
assert(typeof globalThis.require === 'function' && typeof globalThis.require.config === 'function', 'offline AMD require shim missing');
assert(typeof globalThis.createTauriNativeModelWorker === 'function', 'createTauriNativeModelWorker missing');

// 4. Execute `uor-desktop-client --self-test` and spawn a live `uor-desktop-client --stdio` child process.
const EXPECTED_32 = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
];

const selfTestStdout = execFileSync(desktopBin, ['--self-test'], { encoding: 'utf8' }).trim();
const selfTestReceipt = JSON.parse(selfTestStdout);
assert(selfTestReceipt.verdict === 'PASS', `desktop --self-test failed: ${selfTestStdout}`);
assert(selfTestReceipt.engine === 'uor-r4-integer/native-desktop', `unexpected engine: ${selfTestReceipt.engine}`);
assert(selfTestReceipt.artifactBytes === 469252, `unexpected artifactBytes: ${selfTestReceipt.artifactBytes}`);
assert(
    selfTestReceipt.artifactSha256 === 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df',
    `unexpected artifactSha256: ${selfTestReceipt.artifactSha256}`,
);
assert(selfTestReceipt.weightsPerToken === 15968, `unexpected weightsPerToken: ${selfTestReceipt.weightsPerToken}`);
assert(JSON.stringify(selfTestReceipt.tokens32) === JSON.stringify(EXPECTED_32), 'self-test 32-token trajectory mismatch');
assert(selfTestReceipt.ur4sess1Bytes === 9782, `expected ur4sess1Bytes=9782, got ${selfTestReceipt.ur4sess1Bytes}`);
assert(selfTestReceipt.storeRecords === 446, `expected storeRecords=446, got ${selfTestReceipt.storeRecords}`);
assert(
    selfTestReceipt.objectVersion === 'bc98ac3d0a8f363ea6e7bcb10dea9acf59bd1a43859d9e6dd4417f0e1d5364ca',
    `unexpected objectVersion: ${selfTestReceipt.objectVersion}`,
);

// Spawn real `uor-desktop-client --stdio --grant <tmpGrantedStoreRoot>` child process and wire `window.__TAURI__.core.invoke`.
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
        if (msg.ok) {
            waiter.resolve(msg.result);
        } else {
            waiter.reject(new Error(msg.error || 'RPC error'));
        }
    }
});

const realStdioTauriCore = {
    invoke(command, args = {}) {
        invokedCommands.push(command);
        const id = nextRpcId++;
        return new Promise((resolvePromise, rejectPromise) => {
            pendingRpc.set(id, { resolve: resolvePromise, reject: rejectPromise });
            stdioChild.stdin.write(`${JSON.stringify({ id, command, args })}\n`);
        });
    },
};

globalThis.__TAURI__ = { core: realStdioTauriCore };

// 5. Evaluate the actual `<script type="module">` from `web/studio/index.html` and exercise
// editor, diff, markdown, math, 3D visualizer, and GitHub API gating.
const scriptMatch = indexHtml.match(/<script type="module">([\s\S]*?)<\/script>/);
assert(scriptMatch && scriptMatch[1], 'failed to locate <script type="module"> in web/studio/index.html');
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const moduleBody = scriptMatch[1].replace(/\bimport\.meta\.url\b/g, JSON.stringify(import.meta.url));
const runIndexScript = new AsyncFunction(moduleBody);
await runIndexScript();
for (const handler of domContentLoadedHandlers) {
    handler();
}
await new Promise((r) => setTimeout(r, 25));

assert(typeof globalThis.switchEditorTab === 'function', 'index.html did not register window.switchEditorTab');
assert(typeof globalThis.openFileInMonaco === 'function', 'index.html did not register window.openFileInMonaco');
assert(typeof globalThis.toggleMonacoDiffView === 'function', 'index.html did not register window.toggleMonacoDiffView');
assert(typeof globalThis.renderRichMarkdown === 'function', 'index.html did not register window.renderRichMarkdown');
assert(typeof globalThis.initThreeVisualizer === 'function', 'index.html did not register window.initThreeVisualizer');
assert(typeof globalThis.callGithubApi === 'function', 'index.html did not register window.callGithubApi');

// Exercise Monaco editor & setModelLanguage via index.html's openFileInMonaco and switchEditorTab:
globalThis.openFileInMonaco('src/main.rs', 'fn main() {\n    println!("old line");\n}\n', 'rs');
globalThis.switchEditorTab('src/main.rs');
assert(
    globalThis.monacoEditorInstance.getModel().getLanguageId() === 'rust',
    `expected model language 'rust' after switchEditorTab, got ${globalThis.monacoEditorInstance.getModel().getLanguageId()}`,
);

// Exercise Monaco diff view via index.html's toggleMonacoDiffView:
globalThis.remoteOriginalFiles['src/main.rs'] = 'fn main() {\n    println!("old line");\n}\n';
globalThis.monacoEditorInstance.setValue('fn main() {\n    println!("new line");\n}\n');
globalThis.toggleMonacoDiffView();
const diffChanges = globalThis.monacoDiffInstance.getLineChanges();
assert(
    diffChanges.some((c) => c.type === 'del' && c.text.includes('old line')) &&
        diffChanges.some((c) => c.type === 'ins' && c.text.includes('new line')),
    `expected real line-by-line diff with del and ins entries, got ${JSON.stringify(diffChanges)}`,
);
const diffHtml = globalThis.document.getElementById('monacoDiffRoot').innerHTML;
assert(diffHtml.includes('uor-diff-del') && diffHtml.includes('uor-diff-ins'), 'diff container missing del/ins markup');

// Exercise Markdown + KaTeX rendering via index.html's renderRichMarkdown:
const mdTarget = globalThis.document.createElement('div');
globalThis.renderRichMarkdown(
    mdTarget,
    '# Heading 1\n\n- bullet alpha\n- bullet beta\n\n1. step one\n2. step two\n\n> quoted line\n\n| ColA | ColB |\n|---|---|\n| v1 | v2 |\n\nVisit [Spec](docs/README.md) with *italics* and **bold** and $\\frac{\\alpha}{\\beta} \\le x^2_i$.',
);
const richHtml = mdTarget.innerHTML;
assert(richHtml.includes('<ul><li>bullet alpha</li><li>bullet beta</li></ul>'), 'marked failed to render unordered list');
assert(richHtml.includes('<ol><li>step one</li><li>step two</li></ol>'), 'marked failed to render ordered list');
assert(richHtml.includes('<blockquote>quoted line</blockquote>'), 'marked failed to render blockquote');
assert(richHtml.includes('<table><thead><tr><th>ColA</th><th>ColB</th></tr></thead>'), 'marked failed to render pipe table');
assert(richHtml.includes('<a href="docs/README.md"'), 'marked failed to render link');
assert(richHtml.includes('<em>italics</em>') && richHtml.includes('<strong>bold</strong>'), 'marked failed to render italics/bold');
assert(richHtml.includes('katex-frac') && richHtml.includes('α') && richHtml.includes('β'), 'katex failed to format fraction and Greek letters');

// Exercise 3D visualizer via index.html's initThreeVisualizer & triggerSynapticFlash:
globalThis.initThreeVisualizer();
globalThis.triggerSynapticFlash();
const threeRenderer = globalThis.getThreeRenderer();
assert(threeRenderer && threeRenderer.renderCount >= 1, 'THREE.WebGLRenderer did not execute render(scene, camera)');

// Exercise callGithubApi gating in desktop mode vs browser mode:
let desktopGithubBlocked = false;
try {
    await globalThis.callGithubApi('/user');
} catch (err) {
    desktopGithubBlocked = String(err).includes('disabled in desktop/offline Studio');
}
assert(desktopGithubBlocked, 'callGithubApi must be blocked when window.__TAURI__ is active');

// 6. Drive `createTauriNativeModelWorker` against the live `uor-desktop-client --stdio` process.
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
        const t = setTimeout(
            () => rejectPromise(new Error(`Timed out waiting for worker event; seen: ${JSON.stringify(workerEvents)}`)),
            timeoutMs,
        );
        eventWaiters.push({
            pred,
            resolve: (ev) => {
                clearTimeout(t);
                resolvePromise(ev);
            },
        });
    });
}

worker.postMessage({ action: 'prewarm' });
const prewarmEv = await waitWorkerEvent((e) => e.action === 'runtime_loaded');
assert(prewarmEv.engine === 'uor-r4-integer/native-desktop', 'prewarm did not emit native-desktop runtime_loaded');

worker.postMessage({ action: 'load_model', payload: {} });
const loadedEv = await waitWorkerEvent((e) => e.action === 'model_loaded' || e.action === 'operation_error');
assert(loadedEv.action === 'model_loaded' && loadedEv.bytes === 469252, `load_model failed: ${JSON.stringify(loadedEv)}`);

worker.postMessage({
    action: 'generate',
    id: 'gen_desktop_stdio_1',
    payload: { conversationId: 'conv_desktop_1', tokenIds: [0], options: { max_new_tokens: 32 } },
});
const genDone = await waitWorkerEvent(
    (e) => (e.action === 'generate_complete' || e.action === 'generate_error') && e.id === 'gen_desktop_stdio_1',
);
assert(genDone.action === 'generate_complete', `generate failed: ${JSON.stringify(genDone)}`);
assert(JSON.stringify(genDone.tokenIds) === JSON.stringify(EXPECTED_32), 'desktop worker generate_complete tokens mismatch');

const streamedForGen = workerEvents.filter((e) => e.action === 'stream_token' && e.id === 'gen_desktop_stdio_1');
assert(streamedForGen.length === 32, `expected 32 individual step-by-step stream_token events, got ${streamedForGen.length}`);

// Verify that page-invoked `native_grant_folder` is refused over RPC (folder grants are host-owned)
// and that saving a durable session to an ungranted storeDir is refused with PathNotGranted:
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
    assert(
        String(ungrantedEv.error).includes('PathNotGranted'),
        `expected PathNotGranted on ungranted storeDir, got ${JSON.stringify(ungrantedEv)}`,
    );

    // Verify durable save + restore to the host-granted `--grant` folder over real stdio IPC:
    worker.postMessage({
        action: 'save_session_durable',
        id: 'save_granted',
        payload: { conversationId: 'conv_desktop_1', storeDir: tmpGrantedStoreRoot },
    });
    const durableSaved = await waitWorkerEvent(
        (e) => (e.action === 'session_durable_saved' || e.action === 'operation_error') && e.id === 'save_granted',
    );
    assert(durableSaved.action === 'session_durable_saved', `save_session_durable failed: ${JSON.stringify(durableSaved)}`);
    assert(
        durableSaved.receipt.ur4sess1_bytes_len === 9782 &&
            durableSaved.receipt.scope_key === 'local-user/studio/conv_desktop_1' &&
            durableSaved.receipt.object_version_hex.length === 64,
        `unexpected durable receipt: ${JSON.stringify(durableSaved.receipt)}`,
    );

    worker.postMessage({
        action: 'reset_session',
        id: 'reset_before_restore',
        payload: { conversationId: 'conv_desktop_1' },
    });
    await waitWorkerEvent((e) => e.action === 'session_reset' && e.id === 'reset_before_restore');

    worker.postMessage({
        action: 'restore_session_durable',
        id: 'restore_granted',
        payload: { conversationId: 'conv_desktop_1', storeDir: tmpGrantedStoreRoot },
    });
    const durableRestored = await waitWorkerEvent(
        (e) => (e.action === 'session_durable_restored' || e.action === 'operation_error') && e.id === 'restore_granted',
    );
    assert(
        durableRestored.action === 'session_durable_restored' &&
            durableRestored.receipt.ur4sess1_bytes_len === 9782 &&
            durableRestored.receipt.position === 32 &&
            durableRestored.receipt.artifact_sha256 === 'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df',
        `restore_session_durable mismatch: ${JSON.stringify(durableRestored)}`,
    );
} finally {
    rmSync(tmpGrantedStoreRoot, { recursive: true, force: true });
    rmSync(tmpUngrantedStoreRoot, { recursive: true, force: true });
    stdioChild.stdin.end();
    stdioChild.kill();
}

assert(remoteRequests.length === 0, `expected 0 remote requests, got ${JSON.stringify(remoteRequests)}`);

console.log(JSON.stringify({
    verdict: 'ALL DESKTOP CLIENT & OFFLINE STUDIO CHECKS PASSED',
    offlineExternalUrls: 0,
    streamedTokensCount: streamedForGen.length,
    diffLineChangesCount: diffChanges.length,
    threeRenderCount: threeRenderer.renderCount,
    invokedCommandsCount: invokedCommands.length,
    selfTestReceipt,
    remoteRequests: remoteRequests.length,
}));
