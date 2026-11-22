#!/usr/bin/env node
// Automated end-to-end Chrome DevTools Protocol (CDP) verification for Foundry Studio +
// real UOR-R4 integer WASM serving + RV64IMAC MVM Micro-Linux WebAssembly terminal.
//
// Usage:
//   node web/studio/verify_foundry_studio.mjs [dist-dir]

import { createServer } from 'node:http';
import { existsSync, readFileSync, statSync } from 'node:fs';
import { mkdtemp, rm } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { tmpdir } from 'node:os';
import { extname, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = resolve(fileURLToPath(import.meta.url), '..');
const repoRoot = resolve(__dirname, '../..');
const DEFAULT_DIST = resolve(__dirname, 'dist');
const CHROME_CANDIDATES = [
  process.env.CHROME || '',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  '/Applications/Chromium.app/Contents/MacOS/Chromium',
  '/usr/bin/google-chrome',
  '/usr/bin/chromium',
  '/usr/bin/chromium-browser',
].filter(Boolean);

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.webmanifest': 'application/manifest+json; charset=utf-8',
  '.wasm': 'application/wasm',
  '.lut': 'application/octet-stream',
  '.holo': 'application/octet-stream',
  '.bin': 'application/octet-stream',
  '.gz': 'application/octet-stream',
  '.woff2': 'font/woff2',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.ico': 'image/x-icon',
  '.txt': 'text/plain; charset=utf-8',
  '.md': 'text/markdown; charset=utf-8',
  '.tex': 'text/plain; charset=utf-8',
};

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

function chromeBinary() {
  for (const candidate of CHROME_CANDIDATES) {
    if (existsSync(candidate)) return candidate;
  }
  throw new Error('No local Chrome/Chromium found (set CHROME env var)');
}

async function startServer(distDir) {
  const server = createServer((req, res) => {
    try {
      const reqUrl = new URL(req.url, 'http://127.0.0.1');
      let reqPath = decodeURIComponent(reqUrl.pathname);
      if (reqPath === '/uor-r4/studio') {
        res.writeHead(301, { location: '/uor-r4/studio/' });
        res.end();
        return;
      }
      if (reqPath.startsWith('/uor-r4/studio/')) {
        reqPath = '/' + reqPath.slice('/uor-r4/studio/'.length);
      }
      if (reqPath === '/') reqPath = '/index.html';
      const norm = normalize(reqPath).replace(/^\/+/, '');
      let full = resolve(distDir, norm);
      if (!full.startsWith(distDir + sep) && full !== distDir) {
        res.writeHead(403);
        res.end('forbidden');
        return;
      }
      if (!existsSync(full) || statSync(full).isDirectory()) {
        // SPA client route fallback inside Foundry Studio, matching 404.html / Pages behavior.
        if (!extname(norm) && existsSync(join(distDir, 'index.html'))) {
          full = join(distDir, 'index.html');
        } else {
          res.writeHead(404, { 'content-type': 'text/plain' });
          res.end('not found: ' + norm);
          return;
        }
      }
      const body = readFileSync(full);
      res.writeHead(200, {
        'content-type': MIME[extname(full).toLowerCase()] || 'application/octet-stream',
        'cache-control': 'no-store',
      });
      res.end(body);
    } catch (error) {
      res.writeHead(500);
      res.end(String(error));
    }
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  return { server, origin: `http://127.0.0.1:${server.address().port}` };
}

async function launchChrome(profileDir) {
  const bin = chromeBinary();
  const child = spawn(
    bin,
    [
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
      `--user-data-dir=${profileDir}`,
      'about:blank',
    ],
    { stdio: ['ignore', 'pipe', 'pipe'] },
  );
  let stderr = '';
  const wsUrl = await new Promise((resolveReady, rejectReady) => {
    const timer = setTimeout(
      () => rejectReady(new Error('Chrome did not expose DevTools endpoint: ' + stderr.slice(-400))),
      20000,
    );
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
      const match = stderr.match(/DevTools listening on (ws:[^\s]+)/);
      if (match) {
        clearTimeout(timer);
        resolveReady(match[1]);
      }
    });
    child.on('exit', (code) => {
      clearTimeout(timer);
      rejectReady(new Error(`Chrome exited early (${code}): ${stderr.slice(-400)}`));
    });
  });
  return { child, wsUrl };
}

async function cdpConnect(wsUrl) {
  const ws = new WebSocket(wsUrl);
  const pending = new Map();
  let nextId = 1;
  const events = [];
  const send = (method, params = {}, sessionId) =>
    new Promise((resolveSend, rejectSend) => {
      const id = nextId++;
      const timer = setTimeout(() => {
        if (pending.has(id)) {
          pending.delete(id);
          rejectSend(new Error(`CDP timeout calling ${method}`));
        }
      }, 50000);
      pending.set(id, {
        resolve: (res) => {
          clearTimeout(timer);
          resolveSend(res);
        },
        reject: (err) => {
          clearTimeout(timer);
          rejectSend(err);
        },
      });
      ws.send(JSON.stringify({ id, method, params, sessionId }));
    });
  ws.addEventListener('message', (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const entry = pending.get(msg.id);
      pending.delete(msg.id);
      if (msg.error) entry.reject(new Error(`CDP ${msg.error.code}: ${msg.error.message}`));
      else entry.resolve(msg.result);
    } else if (msg.method) {
      events.push(msg);
      if (msg.method === 'Target.attachedToTarget' && msg.params?.sessionId) {
        const cid = msg.params.sessionId;
        void send('Network.enable', {}, cid)
          .catch(() => {})
          .finally(() => {
            if (msg.params?.waitingForDebugger) {
              void send('Runtime.runIfWaitingForDebugger', {}, cid).catch(() => {});
            }
          });
      }
    }
  });
  await new Promise((res, rej) => {
    ws.addEventListener('open', res, { once: true });
    ws.addEventListener('error', () => rej(new Error('CDP WebSocket open error')), { once: true });
  });
  return { ws, send, events };
}

function withTimeout(promise, timeoutMs, label) {
  let timer = null;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error(`Timed out after ${timeoutMs}ms: ${label}`)), timeoutMs);
  });
  return Promise.race([promise, timeout]).finally(() => {
    if (timer) clearTimeout(timer);
  });
}

async function waitForEval(cdp, sessionId, fnExpr, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const res = await cdp.send(
      'Runtime.evaluate',
      { expression: fnExpr, awaitPromise: true, returnByValue: true },
      sessionId,
    );
    if (res.exceptionDetails) {
      throw new Error('Page evaluation exception: ' + JSON.stringify(res.exceptionDetails));
    }
    if (res.result?.value) return res.result.value;
    await new Promise((r) => setTimeout(r, 150));
  }
  throw new Error('Timed out waiting for condition: ' + fnExpr.slice(0, 120));
}

async function verifyOriginPath(cdp, origin, basePath) {
  const target = await cdp.send('Target.createTarget', { url: 'about:blank' });
  const attached = await cdp.send('Target.attachToTarget', { targetId: target.targetId, flatten: true });
  const sessionId = attached.sessionId;
  await cdp.send('Emulation.setDeviceMetricsOverride', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false }, sessionId);
  await cdp.send('Network.enable', {}, sessionId);
  await cdp.send('Page.enable', {}, sessionId);
  await cdp.send('Runtime.enable', {}, sessionId);
  await cdp.send('Log.enable', {}, sessionId);
  await cdp.send(
    'Target.setAutoAttach',
    { autoAttach: true, waitForDebuggerOnStart: false, flatten: true },
    sessionId,
  );
  console.error(`[verify] navigating to ${basePath}`);

  const handledChildSessions = new Set();
  const childSessionIds = new Set();
  const drainChildren = async () => {
    for (const ev of cdp.events) {
      if (ev.method === 'Target.attachedToTarget') {
        const cid = ev.params?.sessionId;
        if (cid && !handledChildSessions.has(cid)) {
          handledChildSessions.add(cid);
          childSessionIds.add(cid);
          void cdp.send('Network.enable', {}, cid).catch(() => {});
          if (ev.params?.waitingForDebugger) {
            void cdp.send('Runtime.runIfWaitingForDebugger', {}, cid).catch(() => {});
          }
        }
      }
    }
  };

  const evStart = cdp.events.length;
  await cdp.send('Page.navigate', { url: origin + basePath }, sessionId);

  const deadline = Date.now() + 25000;
  let loaded = false;
  while (Date.now() < deadline) {
    await drainChildren();
    if (cdp.events.slice(evStart).some((e) => e.method === 'Page.loadEventFired')) {
      loaded = true;
      break;
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  assert(loaded, `Page load did not fire for ${basePath}`);
  console.error(`[verify] loaded ${basePath}, waiting for UI mount`);

  const ready = await waitForEval(
    cdp,
    sessionId,
    `Boolean(window.electron && window.__host && window.__uorR4 && document.querySelector('#root')?.children.length > 0)`,
    20000,
  );
  assert(ready, 'Foundry UI did not mount inside #root');
  console.error(`[verify] UI mounted on ${basePath}, checking network/exceptions`);

  await new Promise((r) => setTimeout(r, 1500));
  await drainChildren();

  // Check zero external network requests and zero failed HTTP requests during boot.
  const externalReqs = [];
  const failedSameOrigin = [];
  const reqMap = new Map();
  const pageExceptions = [];
  for (const ev of cdp.events.slice(evStart)) {
    if (ev.method === 'Network.requestWillBeSent') {
      const reqUrl = ev.params?.request?.url || '';
      reqMap.set(ev.params.requestId, reqUrl);
      if (/^https?:\/\//i.test(reqUrl)) {
        const parsed = new URL(reqUrl);
        if (parsed.origin !== origin) {
          externalReqs.push(reqUrl);
        }
      }
    } else if (ev.method === 'Network.responseReceived') {
      const respUrl = ev.params?.response?.url || '';
      const status = ev.params?.response?.status || 0;
      if (/^https?:\/\//i.test(respUrl) && new URL(respUrl).origin === origin && status >= 400) {
        failedSameOrigin.push({ url: respUrl, status });
      }
    } else if (ev.method === 'Runtime.exceptionThrown') {
      pageExceptions.push(ev.params?.exceptionDetails?.text || 'exception');
    }
  }
  assert(externalReqs.length === 0, `Unexpected external boot requests on ${basePath}: ${JSON.stringify(externalReqs)}`);
  assert(failedSameOrigin.length === 0, `Failed HTTP responses on ${basePath}: ${JSON.stringify(failedSameOrigin)}`);
  assert(pageExceptions.length === 0, `Uncaught page exceptions on ${basePath}: ${JSON.stringify(pageExceptions)}`);
  console.error(`[verify] boot clean on ${basePath}, exercising UOR-R4 + MVM`);

  // Verify Foundry + UOR-R4 providers, models.json, core.wasm, and real uor_model_worker.js execution.
  const uorCheck = await withTimeout(
    cdp.send(
      'Runtime.evaluate',
      {
        expression: `(async () => {
        const providers = await window.electron.ipcRenderer.invoke('get-language-model-providers');
        const models = await window.electron.ipcRenderer.invoke('get-language-models', { providerId: 'uor-r4' });
        const status = await window.electron.ipcRenderer.invoke('uor-r4:status');
        const created = await window.electron.ipcRenderer.invoke('create-app', { name: 'verify-uor-mvm' });
        const appId = created.app.id;
        const chatId = created.chatId;

        // Exercise real UOR-R4 integer WASM generation through Foundry's canonical chat queue.
        const loaded = await window.electron.ipcRenderer.invoke('uor-r4:select-model', { modelId: 'uor-r4:synthetic-seed-5' });
        const submitRes = await window.__uorR4.submitPrompt(chatId, appId, 'hello foundry');

        // Poll chat until streaming finishes or timeout
        let chatState = null;
        for (let i = 0; i < 80; i++) {
          chatState = await window.electron.ipcRenderer.invoke('get-chat', chatId);
          const asst = (chatState.messages || []).find((m) => m.role === 'assistant');
          if (asst && asst.content && asst.content.trim().length > 0 && asst.model === 'uor-r4:synthetic-seed-5') break;
          await new Promise((r) => setTimeout(r, 150));
        }
        const finalAsst = (chatState?.messages || []).find((m) => m.role === 'assistant');

        // Exercise MVM WASM RV64IMAC terminal IPC + CoW persistence across simulated page reload.
        const open1 = await window.electron.ipcRenderer.invoke('terminal:open', { appId, cols: 80, rows: 24 });
        const unameRes = await window.electron.ipcRenderer.invoke('mvm:exec', { appId, command: 'uname -a' });
        const writeProbe = await window.electron.ipcRenderer.invoke('mvm:exec', { appId, command: 'echo hello-cow > /tmp/probe.txt' });
        const readProbe1 = await window.electron.ipcRenderer.invoke('mvm:exec', { appId, command: 'cat /tmp/probe.txt' });
        const mvmId = await window.electron.ipcRenderer.invoke('mvm:identity', { appId });

        // Navigate into chat workspace hash route and open Terminal tab in PreviewPanel
        location.hash = '#/chat?id=' + chatId + '&appId=' + appId;
        let terminalBtn = null;
        for (let i = 0; i < 40; i++) {
          terminalBtn = document.querySelector('[data-testid="terminal-mode-button"]');
          if (terminalBtn) break;
          await new Promise((r) => setTimeout(r, 150));
        }
        if (terminalBtn) {
          terminalBtn.click();
          await new Promise((r) => setTimeout(r, 600));
        }
        const xtermMounted = Boolean(document.querySelector('.xterm, [data-testid="terminal-panel"], canvas'));

        return {
          providerIds: providers.map((p) => p.id),
          uorModelNames: models.map((m) => m.apiName),
          catalogSchema: status.schema,
          loadedEngine: loaded.capabilities.engine,
          loadedSha: loaded.sha256,
          chatModel: finalAsst?.model || null,
          chatReplyLen: finalAsst?.content?.length || 0,
          chatReplyPreview: (finalAsst?.content || '').slice(0, 80),
          submitRes,
          openShell: open1.shell,
          unameStdout: unameRes.stdout,
          unameRetired: unameRes.instructions_retired,
          unameMachine: unameRes.compiled_elf_machine,
          probeOut1: readProbe1.stdout,
          cowSha: writeProbe.cow_state_sha256,
          arch: mvmId.architecture,
          hasTerminalTab: Boolean(terminalBtn),
          xtermMounted,
          appId,
          hashAfterNav: location.hash,
        };
      })()`,
        awaitPromise: true,
        returnByValue: true,
      },
      sessionId,
    ),
    45000,
    `uorCheck on ${basePath}`,
  );

  if (uorCheck.exceptionDetails) {
    throw new Error(`Verification script threw on ${basePath}: ${JSON.stringify(uorCheck.exceptionDetails)}`);
  }
  const v = uorCheck.result.value;
  console.error(`[verify] UOR-R4 + MVM exercised on ${basePath}: app=${v.appId} hash=${v.hashAfterNav} term=${v.hasTerminalTab} xterm=${v.xtermMounted}`);
  assert(v.providerIds.includes('uor-r4') && v.providerIds.includes('local'), `Missing providers: ${JSON.stringify(v)}`);
  assert(v.uorModelNames.includes('uor-r4:synthetic-seed-5'), `Missing synthetic-seed-5 model: ${JSON.stringify(v)}`);
  assert(v.loadedEngine === 'uor-r4-integer/serving-wasm', `Unexpected loaded engine: ${v.loadedEngine}`);
  assert(v.chatModel === 'uor-r4:synthetic-seed-5' && v.chatReplyLen > 0, `Real UOR-R4 chat stream failed: ${JSON.stringify(v)}`);
  assert(v.unameStdout.includes('riscv64'), `MVM uname did not report riscv64: ${JSON.stringify(v)}`);
  assert(v.unameRetired > 0, `MVM instructions_retired must be > 0: ${v.unameRetired}`);
  assert(v.probeOut1.trim() === 'hello-cow', `MVM /tmp/probe.txt failed: ${JSON.stringify(v)}`);
  assert(v.hasTerminalTab, `Terminal tab [data-testid="terminal-mode-button"] missing from Foundry UI`);

  // Reload the exact origin + hash route to verify both static-host reload and OPFS/IndexedDB CoW MVM persistence!
  console.error(`[verify] reloading ${basePath}${v.hashAfterNav || ''}`);
  await withTimeout(cdp.send('Runtime.evaluate', { expression: 'window.__uorBeforeReload = true;' }, sessionId), 10000, 'mark reload');
  const reloadEvStart = cdp.events.length;
  await withTimeout(cdp.send('Page.reload', { ignoreCache: true }, sessionId), 15000, 'Page.reload');
  const reloadDeadline = Date.now() + 20000;
  let reloadFired = false;
  while (Date.now() < reloadDeadline) {
    await drainChildren();
    if (cdp.events.slice(reloadEvStart).some((e) => e.method === 'Page.loadEventFired')) {
      reloadFired = true;
      break;
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  assert(reloadFired, `Page.reload loadEventFired timed out on ${basePath}`);
  console.error(`[verify] reload fired on ${basePath}, waiting for UI mount + MVM CoW`);
  await waitForEval(cdp, sessionId, `Boolean(!window.__uorBeforeReload && window.electron && window.__uorR4 && (document.querySelector('#root')?.children.length || 0) > 0)`, 20000);
  await new Promise((r) => setTimeout(r, 800));
  await drainChildren();
  const reloadExternalReqs = [];
  for (const ev of cdp.events.slice(reloadEvStart)) {
    if (ev.method === 'Network.requestWillBeSent') {
      const reqUrl = ev.params?.request?.url || '';
      if (/^https?:\/\//i.test(reqUrl) && new URL(reqUrl).origin !== origin) {
        reloadExternalReqs.push(reqUrl);
      }
    }
  }
  assert(reloadExternalReqs.length === 0, `Unexpected external requests on reload (${basePath}): ${JSON.stringify(reloadExternalReqs)}`);
  const reloadCheck = await withTimeout(
    cdp.send(
      'Runtime.evaluate',
      {
        expression: `(async () => {
        const res = await window.electron.ipcRenderer.invoke('mvm:exec', { appId: ${v.appId}, command: 'cat /tmp/probe.txt' });
        return { ...res, hashAfterReload: location.hash, rootMounted: (document.querySelector('#root')?.children.length || 0) > 0 };
      })()`,
        awaitPromise: true,
        returnByValue: true,
      },
      sessionId,
    ),
    30000,
    `reloadCheck on ${basePath}`,
  );
  if (reloadCheck.exceptionDetails) {
    throw new Error('Reload MVM CoW check failed: ' + JSON.stringify(reloadCheck.exceptionDetails));
  }
  const rVal = reloadCheck.result.value;
  console.error(`[verify] reload CoW check on ${basePath}: stdout=${JSON.stringify(rVal.stdout.trim())} hash=${rVal.hashAfterReload}`);
  assert(rVal.rootMounted, `Root UI failed to mount after hash route reload on ${basePath}: ${JSON.stringify(rVal)}`);
  assert(rVal.stdout.trim() === 'hello-cow', `MVM CoW snapshot did not persist across page reload: ${JSON.stringify(rVal)}`);

  await cdp.send('Target.closeTarget', { targetId: target.targetId });
  return { ...v, persistedAfterReload: rVal.stdout.trim() === 'hello-cow', basePath };
}

async function runIsolatedOriginPath(origin, basePath) {
  const profileDir = await mkdtemp(join(tmpdir(), 'uor-foundry-cdp-'));
  let chrome = null;
  let cdp = null;
  try {
    chrome = await launchChrome(profileDir);
    cdp = await cdpConnect(chrome.wsUrl);
    return await verifyOriginPath(cdp, origin, basePath);
  } finally {
    try { cdp?.ws?.close(); } catch {}
    try { chrome?.child?.kill('SIGKILL'); } catch {}
    await rm(profileDir, { recursive: true, force: true }).catch(() => {});
  }
}

async function main() {
  const distDir = resolve(process.argv[2] || DEFAULT_DIST);
  assert(existsSync(join(distDir, 'index.html')), `dist directory not built: ${distDir}`);

  const { server, origin } = await startServer(distDir);
  try {
    const rootReport = await withTimeout(runIsolatedOriginPath(origin, '/'), 60000, 'origin /');
    const subpathReport = await withTimeout(runIsolatedOriginPath(origin, '/uor-r4/studio/'), 60000, 'origin /uor-r4/studio/');
    console.log(
      JSON.stringify({
        verdict: 'ALL FOUNDRY STUDIO CDP CHECKS PASSED',
        distDir,
        root: rootReport,
        subpath: subpathReport,
      }),
    );
  } finally {
    try { server.close(); } catch {}
  }
}

main().catch((err) => {
  console.error('verify_foundry_studio FAILED:', err?.stack || err);
  process.exit(1);
});
