#!/usr/bin/env node
import { execFileSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { cdpConnect, launchChrome, trapPageLoad } from './verify_studio_page_load.mjs';

const repo = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const evidenceBase = resolve(repo, '../..', 'local/kimi/studio-final-acceptance');
const hostBinary = resolve(repo, 'target/kimi-final-acceptance/debug/uor-desktop-client');
const distRoot = resolve(repo, 'web/studio/dist');
const git = (...args) => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const sourceIdentity = {
    head: git('rev-parse', 'HEAD'),
    tree: git('rev-parse', 'HEAD^{tree}'),
    status: git('status', '--porcelain').split('\n').filter(Boolean),
    hostBinary: { path: hostBinary, sha256: sha256(await readFile(hostBinary)) },
    dist: await (async () => {
        execFileSync('bash', ['scripts/studio/runtime-files.sh', '--verify', distRoot, '--typed'], { cwd: repo, stdio: 'pipe' });
        const files = {};
        const visit = async (directory) => {
            const entries = await readdir(directory, { withFileTypes: true });
            entries.sort((a, b) => a.name.localeCompare(b.name));
            for (const entry of entries) {
                const path = join(directory, entry.name);
                if (entry.isSymbolicLink()) throw new Error(`dist symlink is not evidence: ${path}`);
                if (entry.isDirectory()) await visit(path);
                else if (entry.isFile()) files[relative(distRoot, path).split(sep).join('/')] = sha256(await readFile(path));
            }
        };
        await visit(distRoot);
        return { root: distRoot, files, combinedSha256: sha256(JSON.stringify(files)) };
    })(),
};
const evidenceRoot = resolve(evidenceBase, sourceIdentity.head);
await mkdir(evidenceRoot, { recursive: true });
await writeFile(resolve(evidenceRoot, 'identity.json'), JSON.stringify({ capturedAt: new Date().toISOString(), ...sourceIdentity }, null, 2) + '\n');
const workspace = await mkdtemp(join(evidenceRoot, 'browser-workspace-'));
const profileDir = await mkdtemp(join(tmpdir(), 'uor-loopback-browser-'));
const origin = 'http://127.0.0.1:8765';
let host;
let chrome;
let cdp;

function runGit(...args) {
    return execFileSync('git', args, { cwd: workspace, encoding: 'utf8' });
}

async function evaluate(expression) {
    const { targetInfos } = await cdp.send('Target.getTargets');
    const target = targetInfos.find((entry) => entry.url.startsWith(origin) && entry.url.includes('typed=1'));
    if (!target) throw new Error('typed Studio target is unavailable: ' + JSON.stringify(targetInfos));
    const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    const result = await cdp.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true }, attached.sessionId);
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
}

async function openTypedPage() {
    const target = await cdp.send('Target.createTarget', { url: origin + '/?typed=1' });
    const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    await cdp.send('Page.enable', {}, attached.sessionId);
    for (let attempt = 0; attempt < 100; attempt += 1) {
        const result = await cdp.send('Runtime.evaluate', {
            expression: `document.getElementById('studio-typed-root')?.dataset.active === 'true'`,
            returnByValue: true,
        }, attached.sessionId);
        if (result.result.value) return;
        await new Promise((resolveWait) => setTimeout(resolveWait, 100));
    }
    throw new Error('retained typed page did not mount');
}

async function main() {
    await writeFile(join(workspace, 'note.txt'), 'Loopback browser acceptance: before.\n');
    runGit('init');
    runGit('add', '-N', 'note.txt');
    host = spawn(hostBinary, [
        '--serve-studio', '127.0.0.1:8765',
        '--studio-root', resolve(repo, 'web/studio/dist'),
        '--grant', workspace,
        '--enable-terminal',
        '--approve-command', workspace, 'pwd',
    ], { cwd: repo, stdio: ['ignore', 'pipe', 'pipe'] });
    let hostLog = '';
    host.stdout.on('data', (chunk) => { hostLog += chunk.toString(); });
    host.stderr.on('data', (chunk) => { hostLog += chunk.toString(); });
    const hostReady = Date.now() + 10000;
    while (!hostLog.includes('listening on') && Date.now() < hostReady) {
        if (host.exitCode !== null) throw new Error('loopback host exited: ' + hostLog);
        await new Promise((resolveWait) => setTimeout(resolveWait, 50));
    }
    if (!hostLog.includes('listening on')) throw new Error('loopback host did not become ready: ' + hostLog);

    chrome = await launchChrome(profileDir);
    cdp = await cdpConnect(chrome.wsUrl);
    const typed = await trapPageLoad(cdp, origin, '/?typed=1', { typed: true });
    if (!typed.loaded || typed.external.length || typed.rawTypeScript.length || typed.runtimeErrors.length) {
        throw new Error('loopback initial load violated runtime boundaries: ' + JSON.stringify(typed));
    }
    const checks = typed.typed;
    const flow = checks?.sessionFlow;
    const completeSessionFlow = flow?.created
        && flow.beforeSend?.idMatches && flow.beforeSend?.markerMatches
        && flow.answerReady && flow.firstTranscript?.includes('17+25')
        && flow.firstTranscript?.includes('assistant\n\n42\n')
        && flow.otherHasAnswer === false
        && flow.otherResume?.idMatches && flow.otherResume?.markerMatches
        && flow.restoredResume?.idMatches && flow.restoredResume?.markerMatches
        && flow.restoredAnswer && flow.generationComplete
        && flow.restoredMatchesOriginal && flow.restoredTranscript === flow.firstTranscript;
    if (!checks?.shell || checks.shellCount !== 1 || !checks.active || !checks.legacyHidden || !checks.artifactReady || !completeSessionFlow) {
        throw new Error('typed mount/session baseline failed: ' + JSON.stringify(checks));
    }

    await openTypedPage();
    const workflow = await evaluate(`(async()=>{
        const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
        const root = document.getElementById('studio-typed-root');
        const button = (label) => Array.from(root.querySelectorAll('button')).find((candidate) => candidate.textContent === label);
        const click = async (label) => {
            const target = button(label);
            if (!target) throw new Error('missing button: ' + label);
            target.click();
            await pause(120);
        };
        const waitFor = async (predicate) => {
            for (let attempt = 0; attempt < 50; attempt += 1) {
                const value = predicate();
                if (value) return value;
                await pause(100);
            }
            throw new Error('bounded UI wait failed');
        };
        await click('Workspace');
        await click('Open folder');
        await pause(500);
        const workspaceSummary = window.__studioFiles.treeSummary();
        const selected = window.UORDesktopActions ? await window.UORDesktopActions.chooseProjectFolder().catch((error) => ({ error: String(error) })) : { unavailable: true };
        const vendorResponse = await fetch('assets/js/uor_offline_vendor.js');
        const servedHtml = await (await fetch(location.href)).text();
        const diagnostics = {
            selected,
            tauri: typeof window.__TAURI__,
            invoke: typeof window.__TAURI__?.core?.invoke,
            desktopActions: typeof window.UORDesktopActions,
            readyState: document.readyState,
            vendorStatus: vendorResponse.status,
            vendorBytes: (await vendorResponse.text()).length,
            flagInHtml: servedHtml.includes('UOR_ENABLE_LOOPBACK_TAURI=true'),
            flagGlobal: window.UOR_ENABLE_LOOPBACK_TAURI,
            scripts: Array.from(document.scripts).map((script) => script.src),
        };
        if (!workspaceSummary.rootPath) throw new Error('workspace grant failed: ' + JSON.stringify({ diagnostics, workspaceSummary, text: root.textContent }));
        await click('New file');
        const editor = root.querySelector('textarea[aria-label="File contents"]');
        if (!editor) throw new Error('native editor is unavailable');
        editor.value = 'Loopback native edit/save acceptance.\\n';
        editor.dispatchEvent(new Event('input', { bubbles: true }));
        await click('Save');
        await waitFor(() => root.textContent.includes('File: saved'));
        await click('Agent');
        await click('Attach current file');
        const attached = window.__studioUi.getAttachments().map((entry) => ({ name: entry.name, content: entry.content }));
        if (!attached.some((entry) => entry.name === 'untitled.md' && entry.content === 'Loopback native edit/save acceptance.\\n')) {
            throw new Error('current-file attachment was not retained: ' + JSON.stringify(attached));
        }
        await click('Changes');
        await click('Refresh changes');
        await pause(500);
        const summary = document.getElementById('gitStatusSummary')?.textContent || '';
        const diff = document.getElementById('gitDiffOutput')?.textContent || '';
        if (summary.includes('timeout') || summary.includes('timed out')) throw new Error('Git inspection timed out: ' + summary);
        if (!summary.includes('2 changed files') || !diff.includes('note.txt')) {
            throw new Error('native Git inspection did not return the expected state: ' + JSON.stringify({ summary, diff }));
        }
        const wrongApproval = await window.__TAURI__.core.invoke('native_confirm_terminal_command', { cwd: ${JSON.stringify(workspace)}, command: 'ls' });
        if (wrongApproval.approved !== false) throw new Error('wrong command was approved');
        let selfEnableRefused = false;
        try { await window.__TAURI__.core.invoke('native_set_terminal_enabled', { enabled: true }); } catch (_error) { selfEnableRefused = true; }
        if (!selfEnableRefused) throw new Error('page RPC enabled terminal without host approval');
        await click('Terminal');
        await click('Enable terminal');
        await pause(100);
        const command = root.querySelector('input[aria-label="Terminal command"]');
        command.value = 'pwd';
        command.dispatchEvent(new Event('input', { bubbles: true }));
        await waitFor(() => button('Run command')?.disabled === false);
        await click('Run command');
        await waitFor(() => document.getElementById('terminalOutput')?.textContent.trim());
        const output = document.getElementById('terminalOutput')?.textContent || '';
        if (output.trim() !== ${JSON.stringify(workspace)}) {
            const diagnostic = {
                consent: document.getElementById('terminalConsentState')?.textContent,
                cwd: root.querySelector('input[aria-label="Working directory"]')?.value,
                command: command.value,
                runDisabled: button('Run command')?.disabled,
                output,
            };
            throw new Error('approved pwd output mismatch: ' + JSON.stringify(diagnostic));
        }
        await click('Run command');
        await pause(200);
        const second = document.getElementById('terminalOutput')?.textContent || '';
        if (!second.includes('cancelled')) throw new Error('one-shot approval was reused: ' + second);
        return { attached, summary, output, second, selfEnableRefused };
    })()`);

    const saved = await readFile(join(workspace, 'untitled.md'), 'utf8');
    if (saved !== 'Loopback native edit/save acceptance.\n') throw new Error('native save disk bytes differ: ' + JSON.stringify(saved));
    const initialSession = await evaluate(`(()=>({
        id: window.__studioUi.activeSessionId(),
        messages: window.__studioUi.getConversationMessages(),
    }))()`);
    if (!initialSession.id || !Array.isArray(initialSession.messages)) {
        throw new Error('initial session identity/transcript was not captured: ' + JSON.stringify(initialSession));
    }

    const reopened = await trapPageLoad(cdp, origin, '/?typed=1');
    if (!reopened.loaded || reopened.external.length || reopened.rawTypeScript.length || reopened.runtimeErrors.length) {
        throw new Error('reopened page violated runtime boundaries: ' + JSON.stringify(reopened));
    }
    await openTypedPage();
    const restored = await evaluate(`(()=>{
        const messages = window.__studioUi.getConversationMessages();
        const attachments = window.__studioUi.getAttachments();
        const active = window.__studioFiles.activeFile();
        return {
            exactTranscript: JSON.stringify(messages) === JSON.stringify(${JSON.stringify(initialSession.messages)}),
            sessionIdMatches: window.__studioUi.activeSessionId() === ${JSON.stringify(initialSession.id)},
            attachment: attachments.some((entry) => entry.name === 'untitled.md' && entry.content === 'Loopback native edit/save acceptance.\\n'),
            editor: active?.path === 'untitled.md' && active.content === 'Loopback native edit/save acceptance.\\n',
            rootPath: window.__studioFiles.treeSummary().rootPath,
            artifactText: document.getElementById('studio-typed-root').textContent.includes('Artifact: unselected') ? 'unselected' : 'selected-or-ready',
        };
    })()`);
    if (!restored.exactTranscript || !restored.sessionIdMatches || !restored.attachment || !restored.editor || restored.rootPath !== '') {
        throw new Error('reopened state or per-process grant state is wrong: ' + JSON.stringify(restored));
    }
    const reselectedRoot = await evaluate(`(async()=>{
        const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
        const root = document.getElementById('studio-typed-root');
        const workspace = Array.from(root.querySelectorAll('button')).find((button) => button.textContent === 'Workspace');
        workspace.click();
        await pause(100);
        const open = Array.from(root.querySelectorAll('button')).find((button) => button.textContent === 'Open folder');
        open.click();
        await pause(500);
        return window.__studioFiles.treeSummary().rootPath;
    })()`);
    if (reselectedRoot !== workspace) throw new Error('sole grant was not reselected after reopen: ' + reselectedRoot);
    restored.reselectedRoot = reselectedRoot;
    const { targetInfos } = await cdp.send('Target.getTargets');
    const target = targetInfos.find((entry) => entry.url.startsWith(origin) && entry.url.includes('typed=1'));
    const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    const screenshot = await cdp.send('Page.captureScreenshot', { format: 'png' }, attached.sessionId);
    await writeFile(join(evidenceRoot, 'loopback-acceptance.png'), Buffer.from(screenshot.data, 'base64'));
    const report = {
        verdict: 'LOOPBACK STUDIO ACCEPTANCE PASSED',
        completedAt: new Date().toISOString(),
        identity: {
            head: sourceIdentity.head,
            tree: sourceIdentity.tree,
            hostBinarySha256: sourceIdentity.hostBinary.sha256,
            distCombinedSha256: sourceIdentity.dist.combinedSha256,
            path: resolve(evidenceRoot, 'identity.json'),
        },
        workspace,
        initial: { requests: typed.requested, external: typed.external, rawTypeScript: typed.rawTypeScript, runtimeErrors: typed.runtimeErrors.length, sessionFlow: flow, sessionIdentity: initialSession },
        workflow,
        saved,
        reopened: { requests: reopened.requested, external: reopened.external, rawTypeScript: reopened.rawTypeScript, runtimeErrors: reopened.runtimeErrors.length, restored },
        screenshot: 'loopback-acceptance.png',
    };
    await writeFile(join(evidenceRoot, 'loopback-acceptance.json'), JSON.stringify(report, null, 2) + '\n');
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
