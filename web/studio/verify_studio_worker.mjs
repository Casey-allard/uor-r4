#!/usr/bin/env node
// Automated verification of the Studio Web Worker (`uor_model_worker.js`) and
// `web/studio/index.html` against the `crates/uor-r4-integer` (`serving-wasm`)
// browser engine and the reference `docs/wasm-serving/verify.mjs` oracle.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { buildArtifact } from './tools/make_tokenizer.mjs';
import { validateTokenizerArtifact, tokenizerLabel, encodeBytesWithArtifact } from './assets/js/uor_tokenizer_artifact.js';

const __dirname = dirname(fileURLToPath(import.meta.url));

const wasmPath = process.env.UOR_WASM_PATH
    || resolve(__dirname, 'pkg/uor_r4_integer_bg.wasm');
const stackPath = process.env.UOR_STACK_PATH
    || resolve(__dirname, 'pkg/synthetic-stack.lut');

const wasmBytes = readFileSync(wasmPath);
const stackBytes = readFileSync(stackPath);

// The stack-bound tokenizer artifact is generated from the shipped stack at build
// time (web/studio/tools/make_tokenizer.mjs). Write it next to the bundle so the
// worker loads it exactly as it would in the browser; assert the fallback helpers
// too, so the unbound path stays covered without a second worker process.
const tokenizerArtifact = buildArtifact(stackBytes);
assert(validateTokenizerArtifact(tokenizerArtifact, tokenizerArtifact.stack_sha256, tokenizerArtifact.vocab_size), 'generated tokenizer artifact must validate against the stack');
assert(tokenizerLabel(tokenizerArtifact, tokenizerArtifact.vocab_size).includes('stand-in') && tokenizerLabel(tokenizerArtifact, tokenizerArtifact.vocab_size).includes('stack sha256'), 'generated tokenizer artifact must keep the stand-in label and name the stack digest');
assert(tokenizerLabel(null, 40).includes('unbound'), 'missing artifact must fall back to the explicit unbound label');
assert(JSON.stringify(encodeBytesWithArtifact(new Uint8Array([0, 1, 41]), 40, tokenizerArtifact)) === JSON.stringify([0, 1, 1]), 'stack-bound byte table encodes 41 mod 40 as 1');
assert(JSON.stringify(encodeBytesWithArtifact(new Uint8Array([0, 1, 41]), 40, null)) === JSON.stringify([0, 1, 1]), 'unbound fallback encodes identically for the byte-mod stand-in');
mkdirSync(resolve(__dirname, 'pkg'), { recursive: true });
writeFileSync(resolve(__dirname, 'pkg/tokenizer.json'), JSON.stringify(tokenizerArtifact, null, 2) + '\n');

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

// 1. Intercept all network/beacon/event-stream APIs to enforce zero remote model calls.
const remoteRequests = [];
function recordForbiddenCall(apiName, target) {
    const entry = `${apiName}:${String(target)}`;
    remoteRequests.push(entry);
    throw new Error(`Forbidden ${apiName} in Studio worker: ${String(target)}`);
}

globalThis.fetch = async (url) => recordForbiddenCall('fetch', url);
globalThis.XMLHttpRequest = class {
    open(_method, url) {
        recordForbiddenCall('XMLHttpRequest', url);
    }
};
globalThis.WebSocket = class {
    constructor(url) {
        recordForbiddenCall('WebSocket', url);
    }
};
globalThis.EventSource = class {
    constructor(url) {
        recordForbiddenCall('EventSource', url);
    }
};
globalThis.importScripts = (...urls) => recordForbiddenCall('importScripts', urls.join(','));
globalThis.Worker = class {
    constructor(url) {
        recordForbiddenCall('Worker', url);
    }
};
globalThis.SharedWorker = class {
    constructor(url) {
        recordForbiddenCall('SharedWorker', url);
    }
};

const trappedNavigator = {
    sendBeacon(url, _data) {
        recordForbiddenCall('navigator.sendBeacon', url);
    },
    storage: {
        async estimate() {
            return { usage: 0, quota: 67108864 };
        },
    },
};
Object.defineProperty(globalThis, 'navigator', {
    value: trappedNavigator,
    configurable: true,
    writable: false,
});

// Provide preloaded WASM bytes for Node execution without network fetch.
globalThis.__UOR_WASM_BYTES__ = wasmBytes;

// Minimal Web Worker `self` harness for Node ESM execution, carrying all network traps.
const emittedMessages = [];
let messageWaiters = [];

globalThis.self = {
    fetch: globalThis.fetch,
    XMLHttpRequest: globalThis.XMLHttpRequest,
    WebSocket: globalThis.WebSocket,
    EventSource: globalThis.EventSource,
    importScripts: globalThis.importScripts,
    Worker: globalThis.Worker,
    SharedWorker: globalThis.SharedWorker,
    navigator: trappedNavigator,
    postMessage(msg) {
        emittedMessages.push(msg);
        const remaining = [];
        for (const w of messageWaiters) {
            if (w.predicate(msg)) {
                w.resolve(msg);
            } else {
                remaining.push(w);
            }
        }
        messageWaiters = remaining;
    },
    onmessage: null,
};

// Self-test that every network trap (`fetch`, `XMLHttpRequest`, `WebSocket`,
// `EventSource`, `navigator.sendBeacon`, `importScripts`) fires and throws.
const trapProbes = [
    () => globalThis.fetch('https://example.invalid/probe'),
    () => new globalThis.XMLHttpRequest().open('POST', 'https://example.invalid/xhr'),
    () => new globalThis.WebSocket('wss://example.invalid/ws'),
    () => new globalThis.EventSource('https://example.invalid/sse'),
    () => globalThis.navigator.sendBeacon('https://example.invalid/beacon', 'payload'),
    () => globalThis.self.importScripts('https://example.invalid/script.js'),
];
for (const probe of trapProbes) {
    let caught = false;
    try {
        await probe();
    } catch (_err) {
        caught = true;
    }
    assert(caught, 'network trap probe failed to throw');
}
assert(remoteRequests.length === 6, `expected 6 trap self-test hits, got ${remoteRequests.length}`);
remoteRequests.length = 0;

function waitForMessage(predicate, timeoutMs = 5000) {
    for (const msg of emittedMessages) {
        if (predicate(msg)) {
            const idx = emittedMessages.indexOf(msg);
            if (idx !== -1) emittedMessages.splice(idx, 1);
            return Promise.resolve(msg);
        }
    }
    return new Promise((resolvePromise, rejectPromise) => {
        const timer = setTimeout(() => {
            rejectPromise(new Error(`Timed out waiting for worker message; seen: ${JSON.stringify(emittedMessages)}`));
        }, timeoutMs);
        messageWaiters.push({
            predicate,
            resolve: (msg) => {
                clearTimeout(timer);
                resolvePromise(msg);
            },
        });
    });
}

async function sendToWorker(data) {
    if (typeof globalThis.self.onmessage !== 'function') {
        throw new Error('uor_model_worker.js did not attach self.onmessage');
    }
    await globalThis.self.onmessage({ data });
}

// Import the Studio worker module and trigger `prewarm`.
await import('./assets/js/uor_model_worker.js');
await sendToWorker({ action: 'prewarm' });

const runtimeMsg = await waitForMessage((m) => m.action === 'runtime_loaded' || m.action === 'operation_error');
assert(runtimeMsg.action === 'runtime_loaded', `prewarm failed: ${JSON.stringify(runtimeMsg)}`);
assert(runtimeMsg.engine === 'uor-r4-integer/serving-wasm', `unexpected engine tag: ${runtimeMsg.engine}`);

// 2. Load the synthetic stack via `load_model` -> `loadStack`.
await sendToWorker({
    action: 'load_model',
    payload: {
        bytes: stackBytes.buffer.slice(stackBytes.byteOffset, stackBytes.byteOffset + stackBytes.byteLength),
        fileName: 'synthetic-stack.lut',
    },
});

const loadedMsg = await waitForMessage((m) => m.action === 'model_loaded' || m.action === 'operation_error');
assert(loadedMsg.action === 'model_loaded', `load_model failed: ${JSON.stringify(loadedMsg)}`);
assert(loadedMsg.capabilities.schema === 'uor-r4.lut-stack/1', `schema mismatch: ${loadedMsg.capabilities.schema}`);
assert(loadedMsg.capabilities.synthetic === true, 'expected synthetic === true for mk-synthetic-stack artifact');
assert(loadedMsg.capabilities.status_label === 'synthetic stand-in', `expected "synthetic stand-in", got "${loadedMsg.capabilities.status_label}"`);
assert(loadedMsg.capabilities.tokenizer_label.includes('stand-in') && loadedMsg.capabilities.tokenizer_label.includes('stack sha256'), `expected a stand-in label naming the stack digest, got: ${loadedMsg.capabilities.tokenizer_label}`);
assert(loadedMsg.capabilities.weights_per_token === 15968, `expected weights_per_token=15968, got ${loadedMsg.capabilities.weights_per_token}`);
assert(loadedMsg.capabilities.shape.vocab === 40 && loadedMsg.capabilities.shape.width === 32, 'shape mismatch');

// 3. Run 32-token generation from PROMPT = [0] and compare with docs/wasm-serving/verify.mjs.
const PROMPT = [0];
const EXPECTED_32 = [
    38, 9, 33, 8, 25, 38, 30, 25, 17, 30, 30, 30, 17, 30, 30, 30,
    30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
];

await sendToWorker({
    action: 'generate',
    id: 'gen_full_32',
    payload: {
        conversationId: 'conv_verify_full',
        tokenIds: PROMPT,
        options: { max_new_tokens: 32 },
    },
});

const genFull = await waitForMessage((m) => (m.action === 'generate_complete' || m.action === 'generate_error') && m.id === 'gen_full_32');
assert(genFull.action === 'generate_complete', `generate failed: ${JSON.stringify(genFull)}`);
assert(JSON.stringify(genFull.tokenIds) === JSON.stringify(EXPECTED_32), `32-token output mismatch: got ${JSON.stringify(genFull.tokenIds)}, expected ${JSON.stringify(EXPECTED_32)}`);
assert(genFull.tokenCount === 32, `expected tokenCount=32, got ${genFull.tokenCount}`);
assert(genFull.allocDelta === 0, `expected zero steady-state allocations during step loop, got ${genFull.allocDelta}`);

await sendToWorker({
    action: 'export_session',
    payload: { conversationId: 'conv_verify_full' },
});
const exportedFull32 = await waitForMessage((m) => m.action === 'session_exported' && m.conversationId === 'conv_verify_full');
assert(exportedFull32.bytes && exportedFull32.bytes.byteLength === 9782, `expected 9782 checkpoint bytes after 32 steps, got ${exportedFull32.bytes?.byteLength}`);

// 4. Mid-stream saveSession / restoreSession round-trip (16 + 16 tokens).
await sendToWorker({
    action: 'generate',
    id: 'gen_first_16',
    payload: {
        conversationId: 'conv_split_a',
        tokenIds: PROMPT,
        options: { max_new_tokens: 16 },
    },
});
const genFirst16 = await waitForMessage((m) => m.action === 'generate_complete' && m.id === 'gen_first_16');
assert(genFirst16.tokenIds.length === 16, 'first half length != 16');

await sendToWorker({
    action: 'export_session',
    payload: { conversationId: 'conv_split_a' },
});
const exportedMsg = await waitForMessage((m) => m.action === 'session_exported' && m.conversationId === 'conv_split_a');
assert(exportedMsg.bytes && exportedMsg.bytes.byteLength === 5366, `expected 5366 checkpoint bytes after 16 steps, got ${exportedMsg.bytes?.byteLength}`);

await sendToWorker({
    action: 'import_session',
    payload: {
        conversationId: 'conv_split_b',
        bytes: exportedMsg.bytes,
        lastToken: exportedMsg.lastToken,
    },
});
await waitForMessage((m) => m.action === 'session_imported' && m.conversationId === 'conv_split_b');

await sendToWorker({
    action: 'generate',
    id: 'gen_second_16',
    payload: {
        conversationId: 'conv_split_b',
        tokenIds: [genFirst16.tokenIds[15]],
        options: { max_new_tokens: 16 },
    },
});
const genSecond16 = await waitForMessage((m) => m.action === 'generate_complete' && m.id === 'gen_second_16');
const combined32 = [...genFirst16.tokenIds, ...genSecond16.tokenIds];
assert(JSON.stringify(combined32) === JSON.stringify(EXPECTED_32), `save/restore continuation mismatch: ${JSON.stringify(combined32)}`);

// 5. Verify mid-generation close_session / reset_session race guard:
// Start a 16-token generation without awaiting completion; when step 8 yields,
// dispatch close_session and reset_session and verify both are rejected while
// the active generation finishes all 16 tokens with uncorrupted state.
const genRacePromise = sendToWorker({
    action: 'generate',
    id: 'gen_race_16',
    payload: {
        conversationId: 'conv_race',
        tokenIds: PROMPT,
        options: { max_new_tokens: 16 },
    },
});

await waitForMessage((m) => m.action === 'stream_token' && m.id === 'gen_race_16' && m.tokenCount === 8);

await sendToWorker({
    action: 'close_session',
    id: 'close_during_gen',
    payload: { conversationId: 'conv_race' },
});
const closeRaceErr = await waitForMessage((m) => m.action === 'operation_error' && m.id === 'close_during_gen');
assert(
    closeRaceErr.error.includes('Stop generation before closing or resetting a session'),
    `expected activeJob guard on close_session, got ${JSON.stringify(closeRaceErr)}`,
);

await sendToWorker({
    action: 'reset_session',
    id: 'reset_during_gen',
    payload: { conversationId: 'conv_race' },
});
const resetRaceErr = await waitForMessage((m) => m.action === 'operation_error' && m.id === 'reset_during_gen');
assert(
    resetRaceErr.error.includes('Stop generation before closing or resetting a session'),
    `expected activeJob guard on reset_session, got ${JSON.stringify(resetRaceErr)}`,
);

await genRacePromise;
const genRaceDone = await waitForMessage((m) => m.action === 'generate_complete' && m.id === 'gen_race_16');
assert(
    JSON.stringify(genRaceDone.tokenIds) === JSON.stringify(EXPECTED_32.slice(0, 16)),
    `mid-generation close/reset corrupted stream: ${JSON.stringify(genRaceDone.tokenIds)}`,
);

// 6. Verify static source invariants on uor_model_worker.js and index.html.
const workerSrc = readFileSync(resolve(__dirname, 'assets/js/uor_model_worker.js'), 'utf8')
    .replace(/\/\/.*$/gm, '');
for (const forbidden of ['fetch(', 'XMLHttpRequest', 'WebSocket', 'EventSource', 'sendBeacon', 'importScripts', 'http://', 'https://']) {
    assert(!workerSrc.includes(forbidden), `uor_model_worker.js contains forbidden network token: ${forbidden}`);
}

const indexHtml = readFileSync(resolve(__dirname, 'index.html'), 'utf8');
assert(indexHtml.includes('id="d36EngineStatusBanner"'), 'missing d36EngineStatusBanner in index.html');
assert(indexHtml.includes('0/8') && indexHtml.includes('P_succ = 35.1%'), 'd36EngineStatusBanner must cite 0/8 milestones and A8 preflight P_succ = 35.1%');
assert(!indexHtml.includes('35.1%</code> tok/s') && !indexHtml.includes('35.1% tok/s'), 'unsourced "35.1% tok/s" must not appear in index.html');
assert(!indexHtml.includes('en.wikipedia.org/w/api.php'), 'live Wikipedia search fetch must be removed from index.html');
assert(!indexHtml.includes('api.duckduckgo.com'), 'live DuckDuckGo search fetch must be removed from index.html');
assert(remoteRequests.length === 0, `expected 0 remote requests, got ${JSON.stringify(remoteRequests)}`);

console.log(JSON.stringify({
    verdict: 'ALL STUDIO WORKER CHECKS PASSED',
    engine: runtimeMsg.engine,
    statusLabel: loadedMsg.capabilities.status_label,
    tokenizerLabel: loadedMsg.capabilities.tokenizer_label,
    weightsPerToken: loadedMsg.capabilities.weights_per_token,
    tokens32: genFull.tokenIds,
    checkpointBytes32: exportedFull32.bytes.byteLength,
    checkpointBytes16: exportedMsg.bytes.byteLength,
    saveRestoreBitIdentical: true,
    midGenCloseResetGuarded: true,
    trapSelfTestHits: 6,
    remoteRequests: remoteRequests.length,
}));
