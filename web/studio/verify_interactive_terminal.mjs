#!/usr/bin/env node
import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cdpConnect, launchChrome, trapPageLoad } from './verify_studio_page_load.mjs';

const repo = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const hostBinary = resolve(repo, 'target/cowork-tests/debug/uor-desktop-client');
const dist = resolve(repo, 'web/studio/dist');
const evidence = resolve(repo, 'local/kimi/harness-parity');
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const git = (...args) => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
const head = git('rev-parse', 'HEAD');
const tree = git('rev-parse', 'HEAD^{tree}');
const status = git('status', '--porcelain').split('\n').filter(Boolean);
const outputDir = resolve(evidence, head);
await mkdir(outputDir, { recursive: true });
const workspace = await mkdtemp(join(outputDir, 'pty-workspace-'));
const profileDir = await mkdtemp(join(tmpdir(), 'uor-interactive-terminal-'));
const origin = 'http://127.0.0.1:8766';
let host;
let chrome;
let cdp;
let pageSessionId;

async function evaluate(expression) {
    const { targetInfos } = await cdp.send('Target.getTargets');
    const target = targetInfos.find((entry) => entry.url.startsWith(origin) && entry.url.includes('typed=1'));
    if (!target) throw new Error('typed Studio target is unavailable');
    const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    pageSessionId = attached.sessionId;
    const result = await cdp.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }, attached.sessionId);
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
}

async function waitFor(predicate, label) {
    for (let attempt = 0; attempt < 600; attempt += 1) {
        if (await predicate()) return;
        await new Promise((resolveWait) => setTimeout(resolveWait, 50));
    }
    throw new Error(`bounded wait failed: ${label}`);
}

async function click(label) {
    await evaluate(`(()=>{
        const button = Array.from(document.querySelectorAll('button')).find((entry) => entry.textContent === ${JSON.stringify(label)});
        if (!button) throw new Error('missing button: ' + ${JSON.stringify(label)});
        button.click();
    })()`);
}

async function type(command) {
    await evaluate(`document.querySelector('.xterm-helper-textarea').focus()`);
    for (const key of command) {
        await cdp.send('Input.dispatchKeyEvent', { type: 'keyDown', key, text: key }, pageSessionId);
        await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key }, pageSessionId);
    }
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 }, pageSessionId);
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, nativeVirtualKeyCode: 13 }, pageSessionId);
}

async function output(sessionId) {
    return evaluate(`(async()=>{
        const snapshot = await window.__TAURI__.core.invoke('native_read_terminal', { sessionId: ${JSON.stringify(sessionId)}, after: 0 });
        return {
            active: snapshot.active,
            cursor: snapshot.cursor,
            text: snapshot.events.filter((event) => event.type === 'output').map((event) => {
                const bytes = new Uint8Array(event.dataHex.match(/.{2}/g).map((pair) => Number.parseInt(pair, 16)));
                return new TextDecoder().decode(bytes);
            }).join(''),
        };
    })()`);
}

async function expectOutput(sessionId, expected, label = expected) {
    try {
        await waitFor(async () => (await output(sessionId)).text.includes(expected), label);
    } catch (error) {
        console.error('PTY output state:', JSON.stringify(await output(sessionId)));
        console.error('PTY interface state:', await evaluate(`({
            status: document.getElementById('terminalConsentState')?.textContent,
            focused: document.activeElement?.className,
            hasFocus: document.hasFocus(),
            calls: window.__ptyCalls,
            text: document.querySelector('.xterm-rows')?.textContent,
        })`));
        throw error;
    }
}

async function main() {
    host = spawn(hostBinary, [
        '--serve-studio', '127.0.0.1:8766',
        '--studio-root', dist,
        '--grant', workspace,
        '--enable-terminal',
        '--approve-shell', workspace, '/bin/zsh',
        '--terminal-home', workspace,
    ], { cwd: repo, stdio: ['ignore', 'pipe', 'pipe'] });
    let hostLog = '';
    host.stdout.on('data', (chunk) => { hostLog += chunk.toString(); });
    host.stderr.on('data', (chunk) => { hostLog += chunk.toString(); });
    await waitFor(() => {
        if (host.exitCode !== null) throw new Error(`loopback host exited: ${hostLog}`);
        return hostLog.includes('listening on');
    }, 'loopback host readiness');
    chrome = await launchChrome(profileDir);
    cdp = await cdpConnect(chrome.wsUrl);
    const loaded = await trapPageLoad(cdp, origin, '/?typed=1', { typed: true, keepOpen: true });
    if (!loaded.loaded || loaded.external.length || loaded.rawTypeScript.length || loaded.runtimeErrors.length) throw new Error(JSON.stringify(loaded));
    await click('Workspace');
    await click('Open folder');
    await waitFor(() => evaluate(`window.__studioFiles.treeSummary().rootPath === ${JSON.stringify(workspace)}`), 'granted workspace loading');
    await click('Terminal');
    await waitFor(() => evaluate(`document.getElementById('terminalCwd').value === ${JSON.stringify(workspace)}`), 'terminal workspace path');
    await click('Open terminal');
    try {
        await waitFor(() => evaluate(`Boolean(document.querySelector('.xterm-helper-textarea') && document.getElementById('terminalOutput').dataset.sessionId)`), 'interactive xterm mount');
    } catch (error) {
        console.error('PTY mount state:', await evaluate(`({
            sessionId: document.getElementById('terminalOutput')?.dataset.sessionId,
            status: document.getElementById('terminalConsentState')?.textContent,
            terminalPresent: Boolean(document.querySelector('.xterm')),
            textareas: Array.from(document.querySelectorAll('textarea')).map((node) => node.className),
            native: Boolean(window.__TAURI__?.core?.invoke),
        })`));
        throw error;
    }
    const sessionId = await evaluate(`document.getElementById('terminalOutput').dataset.sessionId`);
    await evaluate(`(()=>{
        const core = window.__TAURI__.core;
        const invoke = core.invoke.bind(core);
        window.__ptyCalls = [];
        core.invoke = async (name, args) => {
            if (name !== 'native_read_terminal') window.__ptyCalls.push({ name, args });
            try { return await invoke(name, args); }
            catch (error) { window.__ptyCalls.push({ name, error: String(error) }); throw error; }
        };
    })()`);
    await type("printf 'READY\\n'");
    await expectOutput(sessionId, 'READY', 'first PTY output');
    await type('read answer');
    await type('fixture-answer');
    await type("printf 'ANSWER:%s\\n' \"$answer\"");
    await expectOutput(sessionId, 'ANSWER:fixture-answer', 'read reply through xterm input');
    await type('pwd');
    await expectOutput(sessionId, workspace, 'PTY working directory');
    await cdp.send('Emulation.setDeviceMetricsOverride', { width: 1100, height: 700, deviceScaleFactor: 1, mobile: false }, pageSessionId);
    await type('stty size');
    await waitFor(async () => /\d+ \d+/.test((await output(sessionId)).text), 'PTY resize output');
    const size = (await output(sessionId)).text.match(/\d+ \d+/)?.[0] || '';
    if (!/^\d+ \d+$/.test(size)) throw new Error(`missing terminal size: ${size}`);
    await type('exit');
    await waitFor(async () => !(await output(sessionId)).active, 'shell exit');
    await click('Close shell');
    const rejected = await evaluate(`(async()=>{
        try { await window.__TAURI__.core.invoke('native_terminal_input', { sessionId: ${JSON.stringify(sessionId)}, data: 'exit\\n' }); return false; }
        catch (_) { return true; }
    })()`);
    if (!rejected) throw new Error('closed PTY accepted more input');
    const { targetInfos } = await cdp.send('Target.getTargets');
    const target = targetInfos.find((entry) => entry.url.startsWith(origin) && entry.url.includes('typed=1'));
    const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    const screenshot = await cdp.send('Page.captureScreenshot', { format: 'png' }, attached.sessionId);
    await writeFile(join(outputDir, 'interactive-terminal.png'), Buffer.from(screenshot.data, 'base64'));
    const files = execFileSync('bash', ['scripts/studio/runtime-files.sh', '--list', '--typed', dist], { cwd: repo, encoding: 'utf8' }).trim().split('\n').filter(Boolean).sort();
    const distFiles = {};
    for (const file of files) distFiles[file] = sha256(await readFile(join(dist, file)));
    const report = {
        verdict: 'INTERACTIVE PTY ACCEPTANCE PASSED',
        identity: { head, tree, status, hostBinarySha256: sha256(await readFile(hostBinary)), distCombinedSha256: sha256(JSON.stringify(distFiles)) },
        workspace,
        sessionId,
        shell: '/bin/zsh',
        commands: ["printf 'READY\\n'", 'read answer', 'fixture-answer', "printf 'ANSWER:%s\\n' \"$answer\"", 'pwd', 'stty size', 'exit'],
        size,
        closedInputRejected: rejected,
        initial: { requests: loaded.requested, external: loaded.external, rawTypeScript: loaded.rawTypeScript, runtimeErrors: loaded.runtimeErrors.length },
        screenshot: 'interactive-terminal.png',
    };
    await writeFile(join(outputDir, 'interactive-terminal.json'), JSON.stringify(report, null, 2) + '\n');
    console.log(JSON.stringify(report, null, 2));
}

try {
    await main();
} finally {
    if (cdp?.ws) cdp.ws.close();
    if (chrome?.child) chrome.child.kill('SIGTERM');
    if (host && host.exitCode === null) host.kill('SIGTERM');
    await new Promise((resolveWait) => setTimeout(resolveWait, 300));
    await rm(profileDir, { recursive: true, force: true }).catch(() => {});
}
