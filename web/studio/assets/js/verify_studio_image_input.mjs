#!/usr/bin/env node
// Real-browser image flow against a native uor-chat loopback with a synthetic stack.

import { createServer } from 'node:http';
import { execFileSync, spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { connect } from 'node:net';

const __dirname = dirname(fileURLToPath(import.meta.url));
const STUDIO_ROOT = resolve(__dirname, '..', '..');
const WORKTREE = resolve(STUDIO_ROOT, '..', '..');
const TOKEN = 'image-flow-token';
const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFElEQVR4nGP8z8Dwn4GBgYGJAQoAHxcCAk+Uzr4AAAAASUVORK5CYII=', 'base64');

function assert(value, message) {
    if (!value) throw new Error(message);
}

function tokenizerJson() {
    const table = Array(256).fill('\0');
    let extra = 0;
    for (let byte = 0; byte < 256; byte += 1) {
        const printable = (byte >= 0x21 && byte <= 0x7e) || (byte >= 0xa1 && byte <= 0xac) || (byte >= 0xae && byte <= 0xff);
        table[byte] = printable ? String.fromCharCode(byte) : String.fromCharCode(255 + (++extra));
    }
    let bytes = Array.from(Buffer.from('User: Assistant: System: Memory: b.\nHilmotwae'));
    bytes = [...new Set(bytes)].sort((a, b) => a - b);
    for (let byte = 0; byte <= 255 && bytes.length < 37; byte += 1) if (!bytes.includes(byte)) bytes.push(byte);
    bytes = bytes.sort((a, b) => a - b).slice(0, 37);
    const vocab = { '<|bos|>': 0, '<|eos|>': 1, '<|unk|>': 2 };
    bytes.forEach((byte, index) => { vocab[table[byte]] = index + 3; });
    return JSON.stringify({
        pre_tokenizer: { type: 'ByteLevel', add_prefix_space: false },
        added_tokens: ['<|bos|>', '<|eos|>', '<|unk|>'].map((content, id) => ({ id, content })),
        model: { type: 'BPE', vocab, merges: [] },
    });
}

async function freePort() {
    const server = createServer();
    await new Promise((resolveListen) => server.listen(0, '127.0.0.1', resolveListen));
    const port = server.address().port;
    await new Promise((resolveClose) => server.close(resolveClose));
    return port;
}

function waitPort(port, timeoutMs = 15000) {
    const deadline = Date.now() + timeoutMs;
    return new Promise((resolveReady, rejectReady) => {
        const tryConnect = () => {
            const socket = connect(port, '127.0.0.1');
            socket.once('connect', () => { socket.destroy(); resolveReady(); });
            socket.once('error', () => {
                socket.destroy();
                if (Date.now() >= deadline) rejectReady(new Error('native uor-chat did not listen on time'));
                else setTimeout(tryConnect, 100);
            });
        };
        tryConnect();
    });
}

async function nativeServer(root) {
    const targetDir = process.env.CARGO_TARGET_DIR || join(WORKTREE, 'target');
    execFileSync('cargo', [
        'build', '--locked', '-p', 'uor-r4-integer', '--features', 'serving-wasm',
        '--bin', 'mk-synthetic-stack', '--bin', 'uor-chat',
    ], { cwd: WORKTREE, env: { ...process.env, CARGO_TARGET_DIR: targetDir }, stdio: 'inherit' });
    const artifact = join(root, 'stack.lut');
    const tokenizer = join(root, 'tokenizer.json');
    execFileSync(join(targetDir, 'debug', 'mk-synthetic-stack'), ['5', artifact], { stdio: 'ignore' });
    await writeFile(tokenizer, tokenizerJson());
    const port = await freePort();
    const child = spawn(join(targetDir, 'debug', 'uor-chat'), [
        '--stack', artifact,
        '--tokenizer', tokenizer,
        '--serve', '127.0.0.1:' + port,
        '--studio-root', STUDIO_ROOT,
        '--token-stdin',
    ], { cwd: WORKTREE, stdio: ['pipe', 'pipe', 'pipe'] });
    let logs = '';
    child.stdout.on('data', (chunk) => { logs += chunk.toString(); });
    child.stderr.on('data', (chunk) => { logs += chunk.toString(); });
    child.stdin.end(TOKEN + '\n');
    await waitPort(port);
    return {
        origin: 'http://127.0.0.1:' + port,
        async close() {
            child.kill('SIGTERM');
            await new Promise((resolveExit) => {
                const timer = setTimeout(resolveExit, 2000);
                child.once('exit', () => { clearTimeout(timer); resolveExit(); });
            });
        },
        logs: () => logs,
    };
}

async function cdpConnect(wsUrl) {
    const ws = new WebSocket(wsUrl);
    const pending = new Map();
    let nextId = 1;
    ws.addEventListener('message', (event) => {
        const message = JSON.parse(event.data);
        if (!message.id || !pending.has(message.id)) return;
        const entry = pending.get(message.id);
        pending.delete(message.id);
        if (message.error) entry.reject(new Error('CDP ' + message.error.code + ': ' + message.error.message));
        else entry.resolve(message.result);
    });
    await new Promise((resolveOpen, rejectOpen) => {
        ws.addEventListener('open', resolveOpen, { once: true });
        ws.addEventListener('error', () => rejectOpen(new Error('CDP websocket failed')), { once: true });
    });
    return {
        ws,
        send(method, params = {}, sessionId) {
            return new Promise((resolveSend, rejectSend) => {
                const id = nextId++;
                pending.set(id, { resolve: resolveSend, reject: rejectSend });
                ws.send(JSON.stringify({ id, method, params, sessionId }));
            });
        },
    };
}

async function launchChrome(profileDir) {
    const candidates = ['/usr/bin/google-chrome', '/usr/bin/google-chrome-stable', '/usr/bin/chromium', '/usr/bin/chromium-browser', '/snap/bin/chromium'];
    const binary = candidates.find((candidate) => existsSync(candidate));
    assert(binary, 'no Chrome/Chromium found');
    const child = spawn(binary, [
        '--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
        '--disable-background-networking', '--disable-component-update', '--disable-sync',
        '--disable-extensions', '--metrics-recording-only', '--mute-audio',
        '--remote-debugging-port=0', '--user-data-dir=' + profileDir, 'about:blank',
    ], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stderr = '';
    const wsUrl = await new Promise((resolveReady, rejectReady) => {
        const timer = setTimeout(() => rejectReady(new Error('Chrome did not expose DevTools: ' + stderr.slice(-500))), 20000);
        child.stderr.on('data', (chunk) => {
            stderr += chunk.toString();
            const match = stderr.match(/DevTools listening on (ws:[^\s]+)/);
            if (match) { clearTimeout(timer); resolveReady(match[1]); }
        });
        child.once('exit', (code) => rejectReady(new Error('Chrome exited early (' + code + '): ' + stderr.slice(-500))));
    });
    return { child, wsUrl };
}

function delay(ms) {
    return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}

async function main() {
    const root = await mkdtemp(join(WORKTREE, '.uor-image-flow-'));
    const profileDir = join(root, 'chrome');
    let native;
    let chrome;
    try {
        native = await nativeServer(root);
        chrome = await launchChrome(profileDir);
        const cdp = await cdpConnect(chrome.wsUrl);
        const target = await cdp.send('Target.createTarget', { url: 'about:blank' });
        const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
        const session = attached.sessionId;
        await cdp.send('Page.enable', {}, session);
        await cdp.send('DOM.enable', {}, session);
        await cdp.send('Runtime.enable', {}, session);
        const evaluate = async (expression) => {
            const result = await cdp.send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true }, session);
            if (result.exceptionDetails) throw new Error(result.exceptionDetails.text || 'browser evaluation failed');
            return result.result.value;
        };
        const captureChatRequests = async () => evaluate(`(() => {
            if (window.__imageFlowFetchWrapped) return true;
            const originalFetch = window.fetch.bind(window);
            window.__imageFlowChatBodies = [];
            window.fetch = async function (resource, options) {
                const url = typeof resource === 'string' ? resource : resource && resource.url;
                if (String(url || '').includes('/v1/chat/completions') && options && typeof options.body === 'string') {
                    window.__imageFlowChatBodies.push(JSON.parse(options.body));
                }
                return originalFetch(resource, options);
            };
            window.__imageFlowFetchWrapped = true;
            return true;
        })()`);
        const waitFor = async (expression, timeout = 15000) => {
            const deadline = Date.now() + timeout;
            while (Date.now() < deadline) {
                if (await evaluate(expression)) return;
                await delay(100);
            }
            throw new Error('browser condition timed out: ' + expression);
        };
        await cdp.send('Page.navigate', { url: native.origin + '/index.html?max_new_tokens=1' }, session);
        await waitFor("document.readyState === 'complete'");
        await waitFor("document.querySelector('#chatSourceBadge') && document.querySelector('#chatSourceBadge').textContent.includes('Native uor-chat')", 25000);
        const pngHex = PNG.toString('hex');
        const attachImage = `(() => {
            const bytes = Uint8Array.from('${pngHex}'.match(/../g).map((x) => parseInt(x, 16)));
            const file = new File([bytes], 'red.png', { type: 'image/png' });
            const input = document.querySelector('#chatFileInput');
            Object.defineProperty(input, 'files', { value: [file], configurable: true });
            input.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
        })()`;
        await evaluate(attachImage);
        try {
            await waitFor("document.querySelectorAll('.attachment-chip-preview').length === 1");
        } catch (error) {
            const detail = await evaluate(`(async()=>{
                const m=await import('./assets/js/image_input.js');
                const f=document.querySelector('#chatFileInput').files[0];
                let read;
                try { read=await m.readImageFiles([f]); } catch (error) { read={thrown:String(error&&error.message||error)}; }
                return JSON.stringify({toasts:Array.from(document.querySelectorAll('.toast')).map(x=>x.textContent),files:document.querySelector('#chatFileInput').files.length,note:document.querySelector('#composerContextNote').textContent,read});
            })()`);
            throw new Error(error.message + ' ' + detail);
        }
        assert(await evaluate("document.querySelector('#composerContextNote').textContent.includes('R4 patch input - experimental; not visual understanding')"), 'experimental label missing');
        assert(await evaluate("!document.querySelector('.msg[data-role=assistant]')"), 'file selection sent without explicit Send');
        await captureChatRequests();
        await evaluate("document.querySelector('#btnSend').click(); true");
        await waitFor("document.querySelector('.msg[data-role=assistant] .msg-body') && document.querySelector('.msg[data-role=assistant] .msg-body').textContent.length > 0", 25000);
        assert(await evaluate(`(() => {
            const body = window.__imageFlowChatBodies[0];
            const content = body && body.messages && body.messages[body.messages.length - 1].content;
            return Array.isArray(content) && content.length === 1 && content[0].type === 'uor_r4_image';
        })()`), 'live image-only request contained a text part or an incorrect native image part');
        const metrics = await evaluate("document.querySelector('.msg[data-role=assistant] .msg-meta').textContent");
        assert(/\d+\/\d+\/\d+/.test(metrics) && !/usage unavailable/.test(metrics), 'exact prompt/completion/total usage missing: ' + metrics);
        assert(await evaluate("document.querySelectorAll('.msg[data-role=assistant]').length === 1"), 'unexpected extra generation');
        assert(await evaluate("sessionStorage.getItem('uor.studio.attachments.images.v1') === '1'"), 'image session marker missing');
        await cdp.send('Page.reload', {}, session);
        await waitFor("document.readyState === 'complete'");
        await waitFor("document.querySelector('#composerContextNote') && document.querySelector('#composerContextNote').textContent.includes('memory-only')");
        await waitFor("document.querySelector('#chatSourceBadge').textContent.includes('Native uor-chat')", 25000);
        assert(await evaluate("document.querySelectorAll('.attachment-chip-preview').length === 0"), 'image attachment restored from storage');
        await captureChatRequests();
        await evaluate(attachImage);
        await waitFor("document.querySelectorAll('.attachment-chip-preview').length === 1");
        await evaluate("document.querySelector('#btnSend').click(); true");
        await waitFor("document.querySelectorAll('.msg[data-role=assistant]').length === 2", 25000);
        const reloadedRequest = await evaluate(`(() => {
            const body = window.__imageFlowChatBodies[0];
            const messages = body && Array.isArray(body.messages) ? body.messages : [];
            const users = messages.filter((message) => message.role === 'user');
            const content = users.length ? users[users.length - 1].content : null;
            return JSON.stringify({
                valid: users.length === 2 && users[0].content === '[Image patch input]' &&
                    Array.isArray(content) && content.length === 1 && content[0].type === 'uor_r4_image',
                body,
            });
        })()`);
        const reloadedState = JSON.parse(reloadedRequest);
        assert(reloadedState.valid, 'reloaded history did not preserve the image marker or sent a non-image-only final turn: ' + reloadedRequest);
        await evaluate(attachImage);
        await waitFor("document.querySelectorAll('.attachment-chip-preview').length === 1");
        await evaluate("document.querySelector('.attachment-chip-remove').click(); true");
        await waitFor("document.querySelectorAll('.attachment-chip-preview').length === 0");
        console.log(JSON.stringify({ verdict: 'NATIVE IMAGE BROWSER FLOW PASSED', metrics, label: 'R4 patch input - experimental; not visual understanding', imageOnlySend: true, memoryOnlyReload: true, reloadedHistoryResend: true }, null, 2));
        await cdp.send('Target.closeTarget', { targetId: target.targetId }).catch(() => {});
        cdp.ws.close();
    } finally {
        if (chrome && chrome.child) {
            try { chrome.child.kill('SIGTERM'); } catch (_error) { /* wrapper may already be gone */ }
            await delay(500);
            try { chrome.child.kill('SIGKILL'); } catch (_error) { /* process already exited */ }
            try {
                for (const pid of execFileSync('pgrep', ['-f', profileDir], { encoding: 'utf8' }).trim().split(/\s+/).filter(Boolean)) {
                    try { process.kill(Number(pid), 'SIGTERM'); } catch (_error) { /* process already exited */ }
                }
            } catch (_error) { /* no process matched the private profile */ }
            await delay(2000);
        }
        if (native) await native.close();
        await rm(root, { recursive: true, force: true }).catch(() => {});
    }
}

main().then(() => process.exit(0)).catch((error) => {
    console.error('FAIL:', error.message);
    process.exit(1);
});
