#!/usr/bin/env node
// Automated end-to-end Chrome DevTools Protocol (CDP) verification for Foundry Studio +
// real UOR-R4 V13 Qwen3-0.6B Foundry WASM + 16-tool webharness + Home Chat mode +
// Stitch 2-of-3 XOR recovery + RV64IMAC MVM Micro-Linux WebAssembly terminal.
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
function resolveV13PackDir() {
  const candidates = [
    process.env.UOR_V13_PACK_DIR || '',
    resolve(repoRoot, 'local/antigravity/d90-foundry-lora/diag_v5/v13_pieces_foundry'),
    resolve(repoRoot, '../../local/antigravity/d90-foundry-lora/diag_v5/v13_pieces_foundry'),
  ].filter(Boolean);
  for (const dir of candidates) {
    if (existsSync(join(dir, 'manifest.stitch.json'))) return dir;
  }
  return null;
}
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

async function startStaticServer(rootDir, { subpathPrefix = null, cors = false } = {}) {
  const server = createServer((req, res) => {
    try {
      if (cors) {
        res.setHeader('access-control-allow-origin', '*');
        res.setHeader('access-control-allow-methods', 'GET, HEAD, OPTIONS');
        if (req.method === 'OPTIONS') {
          res.writeHead(204);
          res.end();
          return;
        }
      }
      const reqUrl = new URL(req.url, 'http://127.0.0.1');
      let reqPath = decodeURIComponent(reqUrl.pathname);
      if (subpathPrefix) {
        if (reqPath === subpathPrefix) {
          res.writeHead(301, { location: subpathPrefix + '/' });
          res.end();
          return;
        }
        if (reqPath.startsWith(subpathPrefix + '/')) {
          reqPath = '/' + reqPath.slice(subpathPrefix.length + 1);
        }
      }
      if (reqPath === '/') reqPath = '/index.html';
      const norm = normalize(reqPath).replace(/^\/+/, '');
      let full = resolve(rootDir, norm);
      if (!full.startsWith(rootDir + sep) && full !== rootDir) {
        res.writeHead(403);
        res.end('forbidden');
        return;
      }
      if (!existsSync(full) || statSync(full).isDirectory()) {
        if (!extname(norm) && existsSync(join(rootDir, 'index.html'))) {
          full = join(rootDir, 'index.html');
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
      }, 90000);
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

async function evalAsync(cdp, sessionId, fnBody, timeoutMs = 60000, label = 'eval') {
  const res = await withTimeout(
    cdp.send(
      'Runtime.evaluate',
      { expression: `(async () => { ${fnBody} })()`, awaitPromise: true, returnByValue: true },
      sessionId,
    ),
    timeoutMs,
    label,
  );
  if (res.exceptionDetails) {
    throw new Error(`${label} threw: ${JSON.stringify(res.exceptionDetails)}`);
  }
  return res.result?.value;
}

async function typeIntoHomeComposer(cdp, sessionId, text) {
  const focused = await evalAsync(
    cdp,
    sessionId,
    `
      const el = document.querySelector('[data-testid="home-chat-input-container"] [contenteditable="true"]')
        || document.querySelector('[contenteditable="true"][role="textbox"]');
      if (!el) return false;
      el.focus();
      const sel = window.getSelection();
      const range = document.createRange();
      range.selectNodeContents(el);
      sel.removeAllRanges();
      sel.addRange(range);
      return true;
    `,
    10000,
    'focus home composer',
  );
  assert(focused, 'Home composer contenteditable textbox not found');
  await cdp.send('Input.insertText', { text }, sessionId);
  await new Promise((r) => setTimeout(r, 200));
}

async function verifyOriginPath(cdp, origin, basePath, packOrigin) {
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

  const navUrl = packOrigin
    ? `${origin}${basePath}?v13Mirror=${encodeURIComponent(packOrigin + '/')}`
    : `${origin}${basePath}`;
  console.error(`[verify] navigating to ${navUrl}`);

  const handledChildSessions = new Set();
  const drainChildren = async () => {
    for (const ev of cdp.events) {
      if (ev.method === 'Target.attachedToTarget') {
        const cid = ev.params?.sessionId;
        if (cid && !handledChildSessions.has(cid)) {
          handledChildSessions.add(cid);
          void cdp.send('Network.enable', {}, cid).catch(() => {});
          if (ev.params?.waitingForDebugger) {
            void cdp.send('Runtime.runIfWaitingForDebugger', {}, cid).catch(() => {});
          }
        }
      }
    }
  };

  const evStart = cdp.events.length;
  await cdp.send('Page.navigate', { url: navUrl }, sessionId);

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

  const ready = await waitForEval(
    cdp,
    sessionId,
    `Boolean(window.electron && window.__host && window.__uorR4 && document.querySelector('#root')?.children.length > 0)`,
    25000,
  );
  assert(ready, 'Foundry UI did not mount inside #root');
  await new Promise((r) => setTimeout(r, 1500));
  await drainChildren();

  const externalReqs = [];
  const failedSameOrigin = [];
  const pageExceptions = [];
  for (const ev of cdp.events.slice(evStart)) {
    if (ev.method === 'Network.requestWillBeSent') {
      const reqUrl = ev.params?.request?.url || '';
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
  console.error(`[verify] boot clean on ${basePath}`);

  // 1. Check catalog & UI ModelPicker: ZERO dummy models (`synthetic-seed-5`, `foundry:reader`).
  const catalogCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const providers = await window.electron.ipcRenderer.invoke('get-language-model-providers');
      const models = await window.electron.ipcRenderer.invoke('get-language-models', { providerId: 'uor-r4' });
      const status = await window.electron.ipcRenderer.invoke('uor-r4:status');
      const settings = await window.electron.ipcRenderer.invoke('get-user-settings');
      const modelsJson = await (await fetch(new URL('models.json', location.href))).json();
      const pickerText = document.querySelector('[data-testid="model-picker"]')?.textContent || '';
      const chatTabSelected = document.querySelector('[data-testid="home-mode-chat"]')?.getAttribute('aria-selected');
      const standards = await window.electron.ipcRenderer.invoke('standards:lookup', { query: 'softmax' });
      return {
        providerIds: providers.map((p) => p.id),
        uorModelNames: models.map((m) => m.apiName),
        jsonIds: (modelsJson.models || []).map((m) => m.id),
        selectedModel: settings.selectedModel,
        catalogSchema: status.schema,
        pickerText,
        chatTabSelected,
        standardsCount: (standards.matches || []).length,
        standardsTopTitle: standards.matches?.[0]?.title || '',
      };
    `,
    25000,
    `catalog check on ${basePath}`,
  );
  assert(
    JSON.stringify(catalogCheck.providerIds) === JSON.stringify(['uor-r4']),
    `Expected sole provider ['uor-r4'], got ${JSON.stringify(catalogCheck.providerIds)}`,
  );
  assert(
    catalogCheck.uorModelNames.includes('uor-r4:qwen3-0.6b-foundry') &&
      catalogCheck.uorModelNames.includes('uor-r4:m1-discrete-trunk-seed-1') &&
      !catalogCheck.uorModelNames.some((n) => n.includes('synthetic-seed-5') || n.includes('foundry:reader')),
    `Unexpected model list: ${JSON.stringify(catalogCheck.uorModelNames)}`,
  );
  assert(
    !catalogCheck.pickerText.includes('synthetic-seed-5') && !catalogCheck.pickerText.includes('Foundry reader'),
    `Dummy label leaked in ModelPicker: ${catalogCheck.pickerText}`,
  );
  assert(catalogCheck.chatTabSelected === 'true', `Home Chat tab should default active: ${JSON.stringify(catalogCheck)}`);
  assert(catalogCheck.standardsCount > 0, `standards:lookup returned 0 matches for 'softmax'`);
  console.error(`[verify] catalog & standards OK (${catalogCheck.uorModelNames.join(', ')})`);

  // 2. Load Run 5 Qwen3-0.6B Foundry V13 WASM & verify Stitch 2-of-3 XOR recovery.
  const activeEvMark = cdp.events.length;
  const v13LoadCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const loaded = await window.electron.ipcRenderer.invoke('uor-r4:select-model', { modelId: 'uor-r4:qwen3-0.6b-foundry' });
      const stitch = await window.electron.ipcRenderer.invoke('uor-r4:test-stitch-recover', { groupIndex: 0, dropped: 2 });
      return { loaded, stitch };
    `,
    90000,
    `V13 WASM load + Stitch XOR recovery on ${basePath}`,
  );
  assert(
    v13LoadCheck.loaded?.capabilities?.engine?.includes('v13') &&
      v13LoadCheck.loaded?.capabilities?.sha256_verified === true &&
      v13LoadCheck.loaded?.sha256 === v13LoadCheck.loaded?.capabilities?.expected_sha256,
    `Unexpected V13 load result: ${JSON.stringify(v13LoadCheck.loaded)}`,
  );
  assert(
    v13LoadCheck.stitch?.ok === true &&
      typeof v13LoadCheck.stitch?.recoveredSha256 === 'string' &&
      v13LoadCheck.stitch.recoveredSha256 === v13LoadCheck.stitch.expectedSha256,
    `Stitch 2-of-3 XOR recovery failed: ${JSON.stringify(v13LoadCheck.stitch)}`,
  );
  console.error(`[verify] V13 WASM mounted & Stitch XOR group-0 recovered (${v13LoadCheck.stitch.recoveredSha256.slice(0, 12)})`);

  // Ensure network fetches during V13 load only targeted loopback origin / packOrigin (or pinned raw GitHub if no local mirror).
  const allowedOrigins = new Set([origin, ...(packOrigin ? [packOrigin] : ['https://raw.githubusercontent.com'])]);
  for (const ev of cdp.events.slice(activeEvMark)) {
    if (ev.method === 'Network.requestWillBeSent') {
      const reqUrl = ev.params?.request?.url || '';
      if (/^https?:\/\//i.test(reqUrl)) {
        const parsed = new URL(reqUrl);
        assert(
          allowedOrigins.has(parsed.origin),
          `Unexpected remote request during V13 load on ${basePath}: ${reqUrl}`,
        );
      }
    }
  }

  // 3. Exercise real Home Chat UI (`Chat` mode stays on `/` with zero app created).
  await typeIntoHomeComposer(cdp, sessionId, 'Explain the UOR-R4 Charter in one sentence.');
  const sentHome = await evalAsync(
    cdp,
    sessionId,
    `
      const btn = document.querySelector('[aria-label="Send message"]');
      if (!btn || btn.disabled) return false;
      btn.click();
      return true;
    `,
    10000,
    'click send in Home Chat mode',
  );
  assert(sentHome, 'Could not click Send in Home Chat mode');

  const homeChatResult = await waitForEval(
    cdp,
    sessionId,
    `(async () => {
      const thread = document.querySelector('[data-testid="home-chat-thread"]');
      const asst = thread?.querySelector('[data-role="assistant"]');
      const promoteBtn = document.querySelector('[data-testid="home-promote-to-studio"]');
      const stopBtn = document.querySelector('[data-testid="home-chat-cancel"]');
      const hist = await window.electron.ipcRenderer.invoke('home-chat:history');
      const msgs = hist.messages || [];
      const done =
        !stopBtn &&
        msgs.length >= 2 &&
        msgs[1].role === 'assistant' &&
        String(msgs[1].content || '').trim().length > 8 &&
        msgs[1].model === 'uor-r4:qwen3-0.6b-foundry';
      if (!done || !promoteBtn) return null;
      const apps = (await window.electron.ipcRenderer.invoke('list-apps')).apps || [];
      return {
        hash: location.hash,
        replyLen: msgs[1].content.length,
        replyPreview: msgs[1].content.slice(0, 120),
        appsCountBeforePromote: apps.length,
        asstText: (asst?.textContent || '').slice(0, 120),
      };
    })()`,
    90000,
  );
  assert(
    !homeChatResult.hash.startsWith('#/chat') && homeChatResult.appsCountBeforePromote === 0,
    `Home Chat navigated or created an app prematurely: ${JSON.stringify(homeChatResult)}`,
  );
  console.error(`[verify] Home Chat stayed on '/' and streamed ${homeChatResult.replyLen} chars: ${JSON.stringify(homeChatResult.replyPreview)}`);

  // 4. Click "Build this in Studio →" and confirm promotion creates an app & navigates into Studio chat.
  await evalAsync(
    cdp,
    sessionId,
    `document.querySelector('[data-testid="home-promote-to-studio"]')?.click();`,
    10000,
    'click Build this in Studio',
  );
  const promoted = await waitForEval(
    cdp,
    sessionId,
    `(async () => {
      const apps = (await window.electron.ipcRenderer.invoke('list-apps')).apps || [];
      if (apps.length < 1 || !location.hash.includes('chat')) return null;
      const appId = apps[apps.length - 1].id;
      const chats = await window.electron.ipcRenderer.invoke('get-chats', appId);
      const chatId = chats[0]?.id;
      if (!chatId) return null;
      return { hash: location.hash, appId, chatId, appCount: apps.length };
    })()`,
    25000,
  );
  console.error(`[verify] Promoted Home Chat -> Studio app #${promoted.appId} (${promoted.hash})`);
  await evalAsync(
    cdp,
    sessionId,
    `
      await window.electron.ipcRenderer.invoke('chat:cancel', ${promoted.chatId}).catch(() => {});
      for (let i = 0; i < 60; i++) {
        const sub = await window.electron.ipcRenderer.invoke('distributed-machine:subscribe', { protocolVersion: 1, machineId: 'chat_stream', encodedKey: { chatId: ${promoted.chatId} } });
        if (sub?.encodedState?.phase === 'idle' || sub?.encodedState?.phase === 'errored') break;
        await new Promise((r) => setTimeout(r, 200));
      }
    `,
    25000,
    'quiesce promoted chat stream',
  );

  // 5. Return to Home, switch to `Build App` mode, and verify `create-app` launches cleanly via composer UI.
  await evalAsync(
    cdp,
    sessionId,
    `
      location.hash = '#/';
    `,
    10000,
    'navigate back home for Build App test',
  );
  await waitForEval(cdp, sessionId, `Boolean(document.querySelector('[data-testid="home-mode-build"]'))`, 15000);
  await evalAsync(
    cdp,
    sessionId,
    `document.querySelector('[data-testid="home-mode-build"]')?.click();`,
    10000,
    'switch to Build App mode',
  );
  await typeIntoHomeComposer(cdp, sessionId, 'Call workspace_list for root.');
  await evalAsync(
    cdp,
    sessionId,
    `
      const btn = document.querySelector('[aria-label="Send message"]');
      if (!btn || btn.disabled) throw new Error('Build App send button disabled');
      btn.click();
    `,
    10000,
    'click send in Build App mode',
  );
  const builtFromHome = await waitForEval(
    cdp,
    sessionId,
    `(async () => {
      const apps = (await window.electron.ipcRenderer.invoke('list-apps')).apps || [];
      if (apps.length < 2 || !location.hash.includes('chat')) return null;
      const app = apps[apps.length - 1];
      const chats = await window.electron.ipcRenderer.invoke('get-chats', app.id);
      const chatId = chats[0]?.id;
      if (!chatId) return null;
      const sub = await window.electron.ipcRenderer.invoke('distributed-machine:subscribe', { protocolVersion: 1, machineId: 'chat_stream', encodedKey: { chatId } });
      if (sub?.encodedState?.phase !== 'idle') return null;
      const chat = await window.electron.ipcRenderer.invoke('get-chat', chatId);
      const asst = (chat.messages || []).find((m) => m.role === 'assistant');
      if (!asst || !asst.content || !asst.content.trim().length) return null;
      return {
        appId: app.id,
        chatId,
        hash: location.hash,
        model: asst.model,
        hasToolCall: asst.content.includes('<tool_call>'),
        hasToolResponse: asst.content.includes('<tool_response>'),
        replyPreview: asst.content.slice(0, 180),
      };
    })()`,
    120000,
  );
  assert(
    builtFromHome.model === 'uor-r4:qwen3-0.6b-foundry',
    `Build App chat used wrong model: ${JSON.stringify(builtFromHome)}`,
  );
  assert(
    builtFromHome.hasToolCall && builtFromHome.hasToolResponse,
    `Expected full <tool_call>/<tool_response> round-trip on Build App turn: ${JSON.stringify(builtFromHome)}`,
  );
  console.error(`[verify] Home Build App create-app + Run 5 generation succeeded: app=${builtFromHome.appId} toolCall=${builtFromHome.hasToolCall} toolResp=${builtFromHome.hasToolResponse} preview=${JSON.stringify(builtFromHome.replyPreview)}`);

  // 6. Test <prism-write> parsing + MVM RV64IMAC CoW persistence + reload survival.
  const mvmCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const appId = ${builtFromHome.appId};
      const chatId = ${builtFromHome.chatId};
      const open1 = await window.electron.ipcRenderer.invoke('terminal:open', { appId, cols: 80, rows: 24 });
      const unameRes = await window.electron.ipcRenderer.invoke('mvm:exec', { appId, command: 'uname -a' });
      const writeProbe = await window.electron.ipcRenderer.invoke('mvm:exec', { appId, command: 'echo hello-cow > /tmp/probe.txt' });
      const readProbe1 = await window.electron.ipcRenderer.invoke('mvm:exec', { appId, command: 'cat /tmp/probe.txt' });
      const mvmId = await window.electron.ipcRenderer.invoke('mvm:identity', { appId });

      // Apply a sample <prism-write> through the shared chat tag applicator and verify version commit.
      const tags = window.__chat.parsePrismTags('<prism-write path="src/uor-probe.txt" description="verify write">uor-v13-ok</prism-write>');
      const patched = await window.__chat.applyPrismTags(appId, tags);
      const readWritten = await window.electron.ipcRenderer.invoke('read-app-file', { appId, filePath: 'src/uor-probe.txt' });

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
        appId,
        chatId,
        openShell: open1.shell,
        unameStdout: unameRes.stdout,
        unameRetired: unameRes.instructions_retired,
        probeOut1: readProbe1.stdout,
        cowSha: writeProbe.cow_state_sha256,
        arch: mvmId.architecture,
        patchedChanged: patched.changed,
        readWritten,
        hasTerminalTab: Boolean(terminalBtn),
        xtermMounted,
        hashAfterNav: location.hash,
      };
    `,
    45000,
    `MVM & prism-write check on ${basePath}`,
  );
  assert(mvmCheck.unameStdout.includes('riscv64'), `MVM uname did not report riscv64: ${JSON.stringify(mvmCheck)}`);
  assert(mvmCheck.unameRetired > 0, `MVM instructions_retired must be > 0: ${mvmCheck.unameRetired}`);
  assert(mvmCheck.probeOut1.trim() === 'hello-cow', `MVM /tmp/probe.txt failed: ${JSON.stringify(mvmCheck)}`);
  assert(mvmCheck.readWritten === 'uor-v13-ok', `prism-write commit failed: ${JSON.stringify(mvmCheck)}`);
  assert(mvmCheck.hasTerminalTab, `Terminal tab [data-testid="terminal-mode-button"] missing`);

  // Reload to verify CoW + IndexedDB state persistence.
  console.error(`[verify] reloading ${basePath}${mvmCheck.hashAfterNav}`);
  await withTimeout(cdp.send('Runtime.evaluate', { expression: 'window.__uorBeforeReload = true;' }, sessionId), 10000, 'mark reload');
  const reloadEvStart = cdp.events.length;
  await withTimeout(cdp.send('Page.reload', { ignoreCache: false }, sessionId), 30000, 'Page.reload');
  const reloadDeadline = Date.now() + 45000;
  let reloadFired = false;
  while (Date.now() < reloadDeadline) {
    await drainChildren();
    if (cdp.events.slice(reloadEvStart).some((e) => e.method === 'Page.loadEventFired')) {
      reloadFired = true;
      break;
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  assert(
    reloadFired,
    `Page.reload loadEventFired timed out on ${basePath}; seen=[${cdp.events.slice(reloadEvStart).map((e) => e.method).slice(-12).join(',')}]`,
  );
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
  const reloadCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const res = await window.electron.ipcRenderer.invoke('mvm:exec', { appId: ${mvmCheck.appId}, command: 'cat /tmp/probe.txt' });
      const written = await window.electron.ipcRenderer.invoke('read-app-file', { appId: ${mvmCheck.appId}, filePath: 'src/uor-probe.txt' });
      return { ...res, written, hashAfterReload: location.hash, rootMounted: (document.querySelector('#root')?.children.length || 0) > 0 };
    `,
    30000,
    `reloadCheck on ${basePath}`,
  );
  assert(reloadCheck.rootMounted, `Root UI failed to mount after reload on ${basePath}`);
  assert(reloadCheck.stdout.trim() === 'hello-cow', `MVM CoW failed across reload: ${JSON.stringify(reloadCheck)}`);
  assert(reloadCheck.written === 'uor-v13-ok', `App file failed across reload: ${JSON.stringify(reloadCheck)}`);

  await cdp.send('Target.closeTarget', { targetId: target.targetId });
  return {
    basePath,
    providers: catalogCheck.providerIds,
    models: catalogCheck.uorModelNames,
    stitchRecoveredSha256: v13LoadCheck.stitch.recoveredSha256,
    homeChatReplyPreview: homeChatResult.replyPreview,
    promotedAppId: promoted.appId,
    builtFromHomeAppId: builtFromHome.appId,
    persistedAfterReload: true,
  };
}

async function runIsolatedOriginPath(origin, basePath, packOrigin) {
  const profileDir = await mkdtemp(join(tmpdir(), 'uor-foundry-cdp-'));
  let chrome = null;
  let cdp = null;
  try {
    chrome = await launchChrome(profileDir);
    cdp = await cdpConnect(chrome.wsUrl);
    return await verifyOriginPath(cdp, origin, basePath, packOrigin);
  } finally {
    try { cdp?.ws?.close(); } catch {}
    try { chrome?.child?.kill('SIGKILL'); } catch {}
    await rm(profileDir, { recursive: true, force: true }).catch(() => {});
  }
}

async function main() {
  const distDir = resolve(process.argv[2] || DEFAULT_DIST);
  assert(existsSync(join(distDir, 'index.html')), `dist directory not built: ${distDir}`);
  const packDir = resolveV13PackDir();

  const { server, origin } = await startStaticServer(distDir, { subpathPrefix: '/uor-r4/studio' });
  const packSrv = packDir ? await startStaticServer(packDir, { cors: true }) : null;
  try {
    const rootReport = await withTimeout(runIsolatedOriginPath(origin, '/', packSrv?.origin || null), 240000, 'origin /');
    const subpathReport = await withTimeout(runIsolatedOriginPath(origin, '/uor-r4/studio/', packSrv?.origin || null), 240000, 'origin /uor-r4/studio/');
    console.log(
      JSON.stringify({
        verdict: 'ALL FOUNDRY STUDIO CDP CHECKS PASSED',
        distDir,
        packMirrorUsed: Boolean(packSrv),
        root: rootReport,
        subpath: subpathReport,
      }),
    );
  } finally {
    try { server.close(); } catch {}
    try { packSrv?.server?.close(); } catch {}
  }
}

main().catch((err) => {
  console.error('verify_foundry_studio FAILED:', err?.stack || err);
  process.exit(1);
});
