#!/usr/bin/env node
// Three-host M7 bit-identity parity driver (Browser WASM + Studio Web Worker leg)
// for `m7-parity-library-cli-browser` (Issue #2867, Milestone #2035).
//
// Exercises:
// 1. Direct `wasm-pack` WebAssembly module (`uor_r4_integer_bg.wasm` + `uor_r4_integer.js`)
//    across all 8 prompts of `m7-parity-prompts-v1` (`48` uninterrupted greedy tokens +
//    `32 + 16` fresh-instance save/restore continuation).
// 2. Studio Web Worker (`web/studio/assets/js/uor_model_worker.js`) with all 6 network APIs
//    trapped (`remoteRequests === 0`) across the same 8 prompts (`48` uninterrupted greedy
//    tokens + `32 + 16` `export_session`/`import_session` continuation).
// 3. Emits a deterministic JSON report (`schema: "uor-r4.m7-browser-parity/1"`) with token
//    trajectories and SHA-256 digests of every `UR4SESS1` session snapshot for comparison
//    against the Rust library (`IntegerStackModel` / `IntegerStackSession`) and `uor-chat` CLI.

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '../..');
const pkgDir = resolve(__dirname, 'pkg');

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

function sha256Hex(bytes) {
    return createHash('sha256').update(bytes).digest('hex');
}

function ensureWasmAndSyntheticStack() {
    const wasmFile = process.env.UOR_WASM_PATH || resolve(pkgDir, 'uor_r4_integer_bg.wasm');
    const jsFile = resolve(dirname(wasmFile), 'uor_r4_integer.js');
    const stackFile = process.env.UOR_STACK_PATH || resolve(pkgDir, 'synthetic-stack.lut');

    const cargoBin = existsSync(`${process.env.HOME}/.cargo/bin/cargo`)
        ? `${process.env.HOME}/.cargo/bin/cargo`
        : 'cargo';
    const wasmPackBin = existsSync(`${process.env.HOME}/.cargo/bin/wasm-pack`)
        ? `${process.env.HOME}/.cargo/bin/wasm-pack`
        : 'wasm-pack';
    const targetDir = process.env.CARGO_TARGET_DIR || resolve(repoRoot, 'local/antigravity/target');

    if (!existsSync(wasmFile) || !existsSync(jsFile)) {
        mkdirSync(dirname(wasmFile), { recursive: true });
        execFileSync(
            wasmPackBin,
            [
                'build',
                resolve(repoRoot, 'crates/uor-r4-integer'),
                '--target',
                'web',
                '--no-opt',
                '--out-dir',
                dirname(wasmFile),
                '--',
                '--features',
                'serving-wasm',
            ],
            {
                env: {
                    ...process.env,
                    CARGO_TARGET_DIR: targetDir,
                    RUSTFLAGS: '-C target-feature=-simd128',
                },
                stdio: 'inherit',
            },
        );
    }

    if (!existsSync(stackFile)) {
        mkdirSync(dirname(stackFile), { recursive: true });
        execFileSync(
            cargoBin,
            [
                'run',
                '-p',
                'uor-r4-integer',
                '--features',
                'serving-wasm',
                '--bin',
                'mk-synthetic-stack',
                '--',
                '5',
                stackFile,
            ],
            {
                env: {
                    ...process.env,
                    CARGO_TARGET_DIR: targetDir,
                },
                stdio: 'inherit',
            },
        );
    }

    return { wasmFile, stackFile };
}

export const M7_PARITY_PROMPTS = [
    { id: 'P0_bos', promptIds: [0] },
    { id: 'P1_quad', promptIds: [0, 7, 19, 31] },
    { id: 'P2_boundary', promptIds: [0, 1, 38, 39] },
    { id: 'P3_fibonacci', promptIds: [0, 1, 2, 3, 5, 8, 13, 21] },
    { id: 'P4_primes', promptIds: [0, 2, 3, 5, 7, 11, 13, 17] },
    { id: 'P5_alternating', promptIds: [0, 39, 1, 38, 2, 37, 3, 36] },
    { id: 'P6_repeated', promptIds: [0, 17, 17, 17, 25, 25] },
    { id: 'P7_octave', promptIds: [0, 4, 8, 12, 16, 20, 24, 28] },
];

const { wasmFile, stackFile } = ensureWasmAndSyntheticStack();
const wasmBytes = readFileSync(wasmFile);
const stackBytes = readFileSync(stackFile);
const wasmSha256 = sha256Hex(wasmBytes);
const artifactSha256 = sha256Hex(stackBytes);

// 1. Trap all network APIs to enforce zero remote calls during WASM and Worker execution.
const remoteRequests = [];
function recordForbiddenCall(apiName, target) {
    const entry = `${apiName}:${String(target)}`;
    remoteRequests.push(entry);
    throw new Error(`Forbidden ${apiName} in M7 browser parity harness: ${String(target)}`);
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

globalThis.__UOR_WASM_BYTES__ = wasmBytes;

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
            rejectPromise(
                new Error(`Timed out waiting for worker message; seen: ${JSON.stringify(emittedMessages)}`),
            );
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

// 2. Initialize the Studio Web Worker (`uor_model_worker.js`) and load the stack artifact.
// Note: `uor_model_worker.js` loads slot handle 0 inside the shared wasm-bindgen module.
await import('./assets/js/uor_model_worker.js');
await sendToWorker({ action: 'prewarm' });
const runtimeMsg = await waitForMessage((m) => m.action === 'runtime_loaded' || m.action === 'operation_error');
assert(runtimeMsg.action === 'runtime_loaded', `prewarm failed: ${JSON.stringify(runtimeMsg)}`);

await sendToWorker({
    action: 'load_model',
    payload: {
        bytes: stackBytes.buffer.slice(stackBytes.byteOffset, stackBytes.byteOffset + stackBytes.byteLength),
        fileName: 'synthetic-stack.lut',
    },
});
const loadedMsg = await waitForMessage((m) => m.action === 'model_loaded' || m.action === 'operation_error');
assert(loadedMsg.action === 'model_loaded', `load_model failed: ${JSON.stringify(loadedMsg)}`);

// 3. Import the raw `wasm-pack` module (`./pkg/uor_r4_integer.js`) and allocate two independent
// stack handles (`rawHandleA`, `rawHandleB`) for direct WASM + fresh-handle restore checks.
const wasmMod = await import('./pkg/uor_r4_integer.js');
const rawHandleA = wasmMod.loadStack(new Uint8Array(stackBytes));
const rawHandleB = wasmMod.loadStack(new Uint8Array(stackBytes));
const blankStateA = new Uint8Array(wasmMod.saveSession(rawHandleA));
const blankStateB = new Uint8Array(wasmMod.saveSession(rawHandleB));
assert(
    sha256Hex(blankStateA) === sha256Hex(blankStateB),
    'initial blank UR4SESS1 snapshot differs between rawHandleA and rawHandleB',
);
const weightsPerToken = Number(wasmMod.weightsPerToken(rawHandleA));

function runDirectWasmSequence(handle, initialStateBytes, promptIds, maxNewTokens) {
    wasmMod.restoreSession(handle, initialStateBytes);
    let tok = promptIds[0] >>> 0;
    for (let i = 1; i < promptIds.length; i++) {
        wasmMod.step(handle, tok);
        tok = promptIds[i] >>> 0;
    }
    const generatedIds = [];
    for (let stepIdx = 0; stepIdx < maxNewTokens; stepIdx++) {
        const out = wasmMod.step(handle, tok);
        tok = out[0] >>> 0;
        generatedIds.push(tok);
    }
    const saved = new Uint8Array(wasmMod.saveSession(handle));
    return {
        generatedIds,
        lastToken: tok,
        sessionBytesLen: saved.byteLength,
        sessionSha256: sha256Hex(saved),
        sessionBytes: saved,
    };
}

const rows = [];
for (const entry of M7_PARITY_PROMPTS) {
    const { id: promptId, promptIds } = entry;

    // 3a. Direct WASM uninterrupted 48-token run on rawHandleA.
    const wasmFull48 = runDirectWasmSequence(rawHandleA, blankStateA, promptIds, 48);

    // 3b. Direct WASM split 32 + 16 run: save on rawHandleA after 32 generated tokens,
    // restore into fresh handle rawHandleB, and continue for 16 tokens.
    const wasmFirst32 = runDirectWasmSequence(rawHandleA, blankStateA, promptIds, 32);
    const wasmCont16 = runDirectWasmSequence(
        rawHandleB,
        wasmFirst32.sessionBytes,
        [wasmFirst32.lastToken],
        16,
    );
    const wasmCombined48 = [...wasmFirst32.generatedIds, ...wasmCont16.generatedIds];
    assert(
        JSON.stringify(wasmCombined48) === JSON.stringify(wasmFull48.generatedIds),
        `Direct WASM 32+16 token mismatch on ${promptId}`,
    );
    assert(
        wasmCont16.sessionSha256 === wasmFull48.sessionSha256,
        `Direct WASM 32+16 UR4SESS1 snapshot SHA-256 mismatch on ${promptId}`,
    );

    // 4a. Studio Web Worker uninterrupted 48-token run.
    const convFull = `conv_full_${promptId}`;
    await sendToWorker({
        action: 'generate',
        id: `gen_full_${promptId}`,
        payload: {
            conversationId: convFull,
            tokenIds: promptIds,
            options: { max_new_tokens: 48 },
        },
    });
    const workerFullGen = await waitForMessage(
        (m) => (m.action === 'generate_complete' || m.action === 'generate_error') && m.id === `gen_full_${promptId}`,
    );
    assert(
        workerFullGen.action === 'generate_complete',
        `Worker 48-token generate failed on ${promptId}: ${JSON.stringify(workerFullGen)}`,
    );
    assert(workerFullGen.allocDelta === 0, `Worker allocDelta != 0 on ${promptId}: ${workerFullGen.allocDelta}`);
    await sendToWorker({
        action: 'export_session',
        payload: { conversationId: convFull },
    });
    const workerFullExport = await waitForMessage(
        (m) => m.action === 'session_exported' && m.conversationId === convFull,
    );
    const workerFull48Sha256 = sha256Hex(workerFullExport.bytes);

    // 4b. Studio Web Worker split 32 + 16 save/restore continuation across conversations.
    const convSplitA = `conv_split_a_${promptId}`;
    const convSplitB = `conv_split_b_${promptId}`;
    await sendToWorker({
        action: 'generate',
        id: `gen_32_${promptId}`,
        payload: {
            conversationId: convSplitA,
            tokenIds: promptIds,
            options: { max_new_tokens: 32 },
        },
    });
    const workerGen32 = await waitForMessage(
        (m) => (m.action === 'generate_complete' || m.action === 'generate_error') && m.id === `gen_32_${promptId}`,
    );
    assert(
        workerGen32.action === 'generate_complete',
        `Worker 32-token generate failed on ${promptId}: ${JSON.stringify(workerGen32)}`,
    );
    await sendToWorker({
        action: 'export_session',
        payload: { conversationId: convSplitA },
    });
    const workerExport32 = await waitForMessage(
        (m) => m.action === 'session_exported' && m.conversationId === convSplitA,
    );
    const worker32Sha256 = sha256Hex(workerExport32.bytes);

    await sendToWorker({
        action: 'import_session',
        payload: {
            conversationId: convSplitB,
            bytes: workerExport32.bytes,
            lastToken: workerExport32.lastToken,
        },
    });
    await waitForMessage((m) => m.action === 'session_imported' && m.conversationId === convSplitB);

    await sendToWorker({
        action: 'generate',
        id: `gen_16_${promptId}`,
        payload: {
            conversationId: convSplitB,
            tokenIds: [workerGen32.tokenIds[31]],
            options: { max_new_tokens: 16 },
        },
    });
    const workerGen16 = await waitForMessage(
        (m) => (m.action === 'generate_complete' || m.action === 'generate_error') && m.id === `gen_16_${promptId}`,
    );
    assert(
        workerGen16.action === 'generate_complete',
        `Worker 16-token continuation failed on ${promptId}: ${JSON.stringify(workerGen16)}`,
    );
    await sendToWorker({
        action: 'export_session',
        payload: { conversationId: convSplitB },
    });
    const workerExportRestored48 = await waitForMessage(
        (m) => m.action === 'session_exported' && m.conversationId === convSplitB,
    );
    const workerRestored48Sha256 = sha256Hex(workerExportRestored48.bytes);

    // Clean up worker sessions so we never hit MAX_SESSIONS (32).
    for (const cId of [convFull, convSplitA, convSplitB]) {
        await sendToWorker({ action: 'close_session', payload: { conversationId: cId } });
        await waitForMessage((m) => m.action === 'session_reset' && m.conversationId === cId);
    }

    const workerCombined48 = [...workerGen32.tokenIds, ...workerGen16.tokenIds];
    assert(
        JSON.stringify(workerFullGen.tokenIds) === JSON.stringify(wasmFull48.generatedIds),
        `Worker vs Direct WASM 48-token mismatch on ${promptId}`,
    );
    assert(
        JSON.stringify(workerCombined48) === JSON.stringify(wasmFull48.generatedIds),
        `Worker 32+16 continuation token mismatch on ${promptId}`,
    );
    assert(
        worker32Sha256 === wasmFirst32.sessionSha256,
        `Worker vs Direct WASM 32-step UR4SESS1 SHA-256 mismatch on ${promptId}`,
    );
    assert(
        workerFull48Sha256 === wasmFull48.sessionSha256 &&
            workerRestored48Sha256 === wasmFull48.sessionSha256,
        `Worker vs Direct WASM 48-step UR4SESS1 SHA-256 mismatch on ${promptId}`,
    );

    rows.push({
        id: promptId,
        prompt_ids: promptIds,
        generated_48: wasmFull48.generatedIds,
        first_32: wasmFirst32.generatedIds,
        cont_16: wasmCont16.generatedIds,
        session_32_bytes_len: wasmFirst32.sessionBytesLen,
        session_32_sha256: wasmFirst32.sessionSha256,
        session_48_bytes_len: wasmFull48.sessionBytesLen,
        session_48_sha256: wasmFull48.sessionSha256,
        session_restored_48_sha256: wasmCont16.sessionSha256,
        worker_matches_wasm: true,
    });
}

assert(remoteRequests.length === 0, `expected zero remote requests, got ${JSON.stringify(remoteRequests)}`);

const report = {
    schema: 'uor-r4.m7-browser-parity/1',
    panel_id: 'm7-parity-prompts-v1',
    wasm_sha256: wasmSha256,
    wasm_bytes_len: wasmBytes.byteLength,
    artifact_sha256: artifactSha256,
    artifact_bytes_len: stackBytes.byteLength,
    weights_per_token: weightsPerToken,
    remote_requests: remoteRequests.length,
    rows,
};

console.log(JSON.stringify(report));
