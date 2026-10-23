#!/usr/bin/env node
// Runtime page-load trap for the Studio pages.
//
// Loads a page in headless Chrome through the DevTools protocol (no npm
// dependency: Node's global WebSocket + a built-in http server) and records
// every network request the page makes while loading. External requests are a
// failure for pages under the zero-external-load bar. A planted-external
// self-test proves the trap actually fires, so a clean run is evidence and not
// just an empty list.
//
// Usage:
//   node verify_studio_page_load.mjs                 # index.html must be clean; demo page reported
//   node verify_studio_page_load.mjs --strict-router # also require the router demo page clean
//
// Requires a local Google Chrome/Chromium; fails closed when it cannot launch.

import { createServer } from 'node:http';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { extname, join, normalize, resolve, sep } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = resolve(fileURLToPath(import.meta.url), '..');
const WORKTREE = resolve(__dirname, '..', '..');
const CHROME_CANDIDATES = [
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
];

const MIME = {
    '.html': 'text/html; charset=utf-8',
    '.js': 'text/javascript; charset=utf-8',
    '.mjs': 'text/javascript; charset=utf-8',
    '.css': 'text/css; charset=utf-8',
    '.json': 'application/json; charset=utf-8',
    '.wasm': 'application/wasm',
    '.lut': 'application/octet-stream',
    '.png': 'image/png',
    '.svg': 'image/svg+xml',
    '.map': 'application/json',
};

function assert(cond, msg) {
    if (!cond) throw new Error(msg);
}

function chromeBinary() {
    for (const c of CHROME_CANDIDATES) if (existsSync(c)) return c;
    throw new Error('no local Chrome/Chromium found for the runtime page-load trap');
}

async function startServer() {
    const webRoot = join(WORKTREE, 'web');
    const server = createServer(async (req, res) => {
        try {
            const url = new URL(req.url, 'http://127.0.0.1');
            if (url.pathname === '/__selftest_external.html') {
                // Planted channels: inline script fetch, a blob Web Worker fetch, and a
                // late fetch (2s). The trap must report all three, which proves that
                // worker-target traffic and deferred requests are captured.
                const planted = [
                    '<!doctype html><html><body>',
                    '<img src="https://example.invalid/planted.png" alt="">',
                    '<script>',
                    'fetch("https://inline.example.invalid/control").catch(() => {});',
                    'const wsrc = "self.onmessage=()=>{fetch(\'https://worker.example.invalid/w\').catch(()=>{})};";',
                    'const blob = new Blob([wsrc], { type: "text/javascript" });',
                    'const worker = new Worker(URL.createObjectURL(blob)); worker.postMessage(1);',
                    'setTimeout(() => fetch("https://late.example.invalid/x").catch(() => {}), 2000);',
                    '</script></body></html>',
                ].join('');
                res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
                res.end(planted);
                return;
            }
            const raw = normalize(decodeURIComponent(url.pathname));
            const rel = raw.startsWith(sep) ? raw.slice(1) : raw;
            const file = join(webRoot, rel);
            if (!file.startsWith(webRoot + sep) || !existsSync(file)) {
                res.writeHead(404, { 'content-type': 'text/plain' });
                res.end('not found');
                return;
            }
            const body = await readFile(file);
            res.writeHead(200, { 'content-type': MIME[extname(file)] || 'application/octet-stream' });
            res.end(body);
        } catch (error) {
            res.writeHead(500);
            res.end(String(error));
        }
    });
    await new Promise((resolveListen) => server.listen(0, '127.0.0.1', resolveListen));
    const port = server.address().port;
    return { server, origin: 'http://127.0.0.1:' + port };
}

async function launchChrome(profileDir) {
    const bin = chromeBinary();
    const child = spawn(bin, [
        '--headless=new',
        '--disable-gpu',
        '--no-first-run',
        '--no-default-browser-check',
        '--disable-background-networking',
        '--disable-component-update',
        '--disable-sync',
        '--disable-extensions',
        '--metrics-recording-only',
        '--mute-audio',
        '--remote-debugging-port=0',
        '--user-data-dir=' + profileDir,
        'about:blank',
    ], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stderr = '';
    const ready = new Promise((resolveReady, rejectReady) => {
        const timer = setTimeout(() => rejectReady(new Error('chrome did not expose a DevTools endpoint: ' + stderr.slice(-400))), 20000);
        child.stderr.on('data', (chunk) => {
            stderr += chunk.toString();
            const m = stderr.match(/DevTools listening on (ws:[^\s]+)/);
            if (m) {
                clearTimeout(timer);
                resolveReady(m[1]);
            }
        });
        child.on('exit', (code) => rejectReady(new Error('chrome exited early (code ' + code + '): ' + stderr.slice(-400))));
    });
    return { child, wsUrl: await ready };
}

async function cdpConnect(wsUrl) {
    const ws = new WebSocket(wsUrl);
    const pending = new Map();
    let nextId = 1;
    const events = [];
    ws.addEventListener('message', (event) => {
        const msg = JSON.parse(event.data);
        if (msg.id && pending.has(msg.id)) {
            const entry = pending.get(msg.id);
            pending.delete(msg.id);
            if (msg.error) entry.rejectSend(new Error('CDP ' + msg.error.code + ': ' + msg.error.message));
            else entry.resolveSend(msg.result);
        } else if (msg.method) {
            events.push(msg);
        }
    });
    await new Promise((resolveOpen, rejectOpen) => {
        ws.addEventListener('open', resolveOpen, { once: true });
        ws.addEventListener('error', () => rejectOpen(new Error('CDP websocket failed to open')), { once: true });
    });
    const send = (method, params, sessionId) => new Promise((resolveSend, rejectSend) => {
        const id = nextId++;
        pending.set(id, { resolveSend, rejectSend });
        ws.send(JSON.stringify({ id, method, params: params || {}, sessionId: sessionId }));
    });
    return { ws, send, events };
}

async function trapPageLoad(cdp, origin, path) {
    const target = await cdp.send('Target.createTarget', { url: 'about:blank' });
    const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    const sessionId = attached.sessionId;
    await cdp.send('Network.enable', {}, sessionId);
    await cdp.send('Page.enable', {}, sessionId);
    // Capture dedicated/service worker targets too: auto-attach to every child,
    // enable Network on its own session and resume it (waitForDebuggerOnStart).
    await cdp.send('Target.setAutoAttach', { autoAttach: true, waitForDebuggerOnStart: true, flatten: true }, sessionId);
    const handledSessions = new Set();
    const drainChildSessions = async () => {
        for (const ev of cdp.events) {
            if (ev.method !== 'Target.attachedToTarget') continue;
            const child = ev.params.sessionId;
            if (!child || handledSessions.has(child)) continue;
            handledSessions.add(child);
            try {
                await cdp.send('Network.enable', {}, child);
                await cdp.send('Runtime.runIfWaitingForDebugger', {}, child);
            } catch (_e) {
                // A target that vanished before enabling Network cannot make requests.
            }
        }
    };
    const before = cdp.events.length;
    await cdp.send('Page.navigate', { url: origin + path }, sessionId);
    const deadline = Date.now() + 25000;
    let loaded = false;
    while (Date.now() < deadline) {
        await drainChildSessions();
        if (cdp.events.slice(before).some((e) => e.method === 'Page.loadEventFired')) { loaded = true; break; }
        await new Promise((r) => setTimeout(r, 100));
    }
    // Settle window: late/deferred requests (e.g. a 2s timer) and worker-target
    // requests must land before the count is read.
    const settleDeadline = Date.now() + 5000;
    while (Date.now() < settleDeadline) {
        await drainChildSessions();
        await new Promise((r) => setTimeout(r, 100));
    }
    const requested = cdp.events.slice(before)
        .filter((e) => e.method === 'Network.requestWillBeSent')
        .map((e) => e.params.request.url);
    const external = requested.filter((u) => {
        if (!/^https?:/i.test(u)) return false;
        try { return new URL(u).origin !== origin; } catch (_e) { return true; }
    });
    await cdp.send('Target.closeTarget', { targetId: target.targetId });
    return { path: path, loaded: loaded, requested: requested.length, external: external };
}

async function main() {
    const strictRouter = process.argv.includes('--strict-router');
    const started = await startServer();
    const origin = started.origin;
    const profileDir = await mkdtemp(join(tmpdir(), 'uor-studio-page-trap-'));
    let chrome;
    let exitCode = 0;
    try {
        chrome = await launchChrome(profileDir);
        const cdp = await cdpConnect(chrome.wsUrl);
        const planted = await trapPageLoad(cdp, origin, '/__selftest_external.html');
        const plantedHits = ['inline.example.invalid', 'worker.example.invalid', 'late.example.invalid']
            .filter((domain) => planted.external.some((u) => u.indexOf(domain) >= 0));
        assert(plantedHits.length === 3, 'trap self-test failed: expected inline, worker and late planted requests, got ' + JSON.stringify({ detected: planted.external, hits: plantedHits }));
        const index = await trapPageLoad(cdp, origin, '/studio/index.html');
        assert(index.loaded, 'index.html never fired loadEventFired');
        assert(index.external.length === 0, 'index.html made external load requests: ' + JSON.stringify(index.external));
        const router = await trapPageLoad(cdp, origin, '/studio/geometric_prime_router_webapp.html');
        const report = {
            verdict: 'RUNTIME PAGE-LOAD TRAP PASSED',
            trapSelfTest: { plantedExternalDetected: true, channels: ['inline', 'blob-worker', 'late'] },
            pages: {
                'index.html': { loaded: index.loaded, requests: index.requested, external: index.external },
                'geometric_prime_router_webapp.html': { loaded: router.loaded, requests: router.requested, external: router.external },
            },
        };
        if (router.external.length > 0) {
            if (strictRouter) {
                report.verdict = 'RUNTIME PAGE-LOAD TRAP FAILED';
                report.failure = 'strict-router: the demo page made external load requests';
                exitCode = 1;
            } else {
                report.legacyResidual = { page: 'geometric_prime_router_webapp.html', external: router.external, note: 'known legacy page; outside the bar unless the PM rules otherwise' };
            }
        }
        console.log(JSON.stringify(report, null, 2));
        cdp.ws.close();
    } finally {
        if (chrome && chrome.child) chrome.child.kill('SIGTERM');
        started.server.close();
        await new Promise((r) => setTimeout(r, 500));
        await rm(profileDir, { recursive: true, force: true }).catch(() => {});
    }
    process.exit(exitCode);
}

main().catch((error) => {
    console.error('FAIL:', error.message);
    process.exit(1);
});
