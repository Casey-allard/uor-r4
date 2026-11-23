#!/usr/bin/env node
// Automated end-to-end Chrome DevTools Protocol (CDP) verification for Foundry Studio +
// real UOR-R4 V13 Qwen3-0.6B Foundry WASM + 16-tool webharness + Home Chat mode +
// Stitch 2-of-3 XOR recovery + RV64IMAC MVM Micro-Linux WebAssembly terminal.
//
// Usage:
//   node web/studio/verify_foundry_studio.mjs [dist-dir]

import crypto from 'node:crypto';
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

async function clickCenter(cdp, sessionId, selector) {
  const rect = await evalAsync(
    cdp,
    sessionId,
    `
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return null;
      const r = el.getBoundingClientRect();
      return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
    `,
    10000,
    `locate ${selector}`,
  );
  assert(rect && rect.x >= 0 && rect.y >= 0, `Element not found or invisible: ${selector}`);
  await cdp.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: rect.x, y: rect.y, button: 'left', clickCount: 1 }, sessionId);
  await cdp.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: rect.x, y: rect.y, button: 'left', clickCount: 1 }, sessionId);
  await new Promise((r) => setTimeout(r, 200));
}

async function startMockGitHubServer() {
  const refs = new Map();
  const blobs = new Map();
  const trees = new Map();
  const commits = new Map();
  const repos = new Map();
  let seq = 1;
  let lastPatchForce = null;
  const sha = (s) => crypto.createHash('sha1').update(s).digest('hex');

  function seedRepo(owner, repo, files) {
    const tree = [];
    for (const [path, content] of Object.entries(files)) {
      const bSha = sha(`blob:${seq++}:${path}:${content}`);
      blobs.set(bSha, { sha: bSha, content: Buffer.from(content, 'utf8').toString('base64'), encoding: 'base64' });
      tree.push({ path, mode: '100644', type: 'blob', sha: bSha, size: Buffer.byteLength(content, 'utf8') });
    }
    const tSha = sha(`tree:${seq++}`);
    trees.set(tSha, tree);
    const cSha = sha(`commit:${seq++}`);
    commits.set(cSha, { sha: cSha, tree: { sha: tSha }, message: 'Initial commit', parents: [] });
    refs.set(`${owner}/${repo}:heads/main`, cSha);
    repos.set(`${owner}/${repo}`, {
      name: repo,
      full_name: `${owner}/${repo}`,
      private: false,
      default_branch: 'main',
      html_url: `https://github.com/${owner}/${repo}`,
      owner: { login: owner },
    });
  }

  seedRepo('octocat', 'hello-uor', {
    'README.md': '# Hello UOR\n',
    'src/main.ts': 'export const greeting = "hello v1";\n',
  });

  const server = createServer(async (req, res) => {
    res.setHeader('access-control-allow-origin', '*');
    res.setHeader('access-control-allow-methods', 'GET, POST, PATCH, OPTIONS');
    res.setHeader('access-control-allow-headers', 'Authorization, Content-Type, Accept, X-GitHub-Api-Version');
    if (req.method === 'OPTIONS') {
      res.writeHead(204);
      res.end();
      return;
    }
    const reqUrl = new URL(req.url, 'http://127.0.0.1');
    const chunks = [];
    for await (const c of req) chunks.push(c);
    const rawBody = Buffer.concat(chunks).toString('utf8');
    const body = rawBody ? JSON.parse(rawBody) : null;
    const send = (code, data) => {
      res.writeHead(code, { 'content-type': 'application/json' });
      res.end(JSON.stringify(data));
    };

    if (req.headers.authorization !== 'Bearer ghp_cdp_stage3_secret_token_777') {
      return send(401, { message: 'Bad credentials' });
    }
    if (req.method === 'GET' && reqUrl.pathname === '/user') {
      return send(200, { login: 'octocat', name: 'The Octocat', avatar_url: 'https://example.invalid/a.png' });
    }
    if (req.method === 'GET' && reqUrl.pathname === '/user/repos') {
      return send(200, Array.from(repos.values()));
    }
    const repoMatch = reqUrl.pathname.match(/^\/repos\/([^/]+)\/([^/]+)(.*)$/);
    if (!repoMatch) return send(404, { message: 'Not Found' });
    const [, owner, repo, tail] = repoMatch;
    const key = `${owner}/${repo}`;

    if (req.method === 'GET' && tail === '') {
      const r = repos.get(key);
      return r ? send(200, r) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'GET' && tail === '/branches') {
      return send(200, [{ name: 'main', commit: { sha: refs.get(`${key}:heads/main`) } }]);
    }
    if (req.method === 'GET' && tail === '/git/ref/heads/main') {
      const head = refs.get(`${key}:heads/main`);
      return head ? send(200, { ref: 'refs/heads/main', object: { sha: head, type: 'commit' } }) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'PATCH' && tail === '/git/refs/heads/main') {
      lastPatchForce = body?.force;
      if (body?.force !== false) return send(400, { message: 'CAS required (force:false)' });
      refs.set(`${key}:heads/main`, body.sha);
      return send(200, { ref: 'refs/heads/main', object: { sha: body.sha, type: 'commit' } });
    }
    let m;
    if (req.method === 'GET' && (m = tail.match(/^\/git\/commits\/([0-9a-f]+)$/))) {
      const c = commits.get(m[1]);
      return c ? send(200, c) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'GET' && (m = tail.match(/^\/git\/trees\/([0-9a-f]+)$/))) {
      const t = trees.get(m[1]);
      return t ? send(200, { sha: m[1], tree: t }) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'GET' && (m = tail.match(/^\/git\/blobs\/([0-9a-f]+)$/))) {
      const b = blobs.get(m[1]);
      return b ? send(200, b) : send(404, { message: 'Not Found' });
    }
    if (req.method === 'POST' && tail === '/git/blobs') {
      const bSha = sha(`blob:${seq++}:${body.content}`);
      blobs.set(bSha, { sha: bSha, content: body.content, encoding: body.encoding || 'base64' });
      return send(201, { sha: bSha });
    }
    if (req.method === 'POST' && tail === '/git/trees') {
      const baseEntries = new Map((trees.get(body.base_tree) || []).map((e) => [e.path, e]));
      for (const item of body.tree || []) {
        if (item.sha === null) baseEntries.delete(item.path);
        else baseEntries.set(item.path, { path: item.path, mode: '100644', type: 'blob', sha: item.sha });
      }
      const tSha = sha(`tree:${seq++}`);
      trees.set(tSha, Array.from(baseEntries.values()));
      return send(201, { sha: tSha });
    }
    if (req.method === 'POST' && tail === '/git/commits') {
      const cSha = sha(`commit:${seq++}:${body.message}`);
      commits.set(cSha, { sha: cSha, tree: { sha: body.tree }, message: body.message, parents: (body.parents || []).map((p) => ({ sha: p })) });
      return send(201, { sha: cSha });
    }
    return send(404, { message: `Unhandled ${req.method} ${reqUrl.pathname}` });
  });

  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const { port } = server.address();
  return {
    server,
    origin: `http://127.0.0.1:${port}`,
    getLastPatchForce: () => lastPatchForce,
    advanceRemote(owner, repo, branch, path, content) {
      const curHead = refs.get(`${owner}/${repo}:heads/${branch}`);
      const parentCommit = commits.get(curHead);
      const baseEntries = new Map((trees.get(parentCommit.tree.sha) || []).map((e) => [e.path, e]));
      const bSha = sha(`blob:${seq++}:${content}`);
      blobs.set(bSha, { sha: bSha, content: Buffer.from(content, 'utf8').toString('base64'), encoding: 'base64' });
      baseEntries.set(path, { path, mode: '100644', type: 'blob', sha: bSha });
      const tSha = sha(`tree:${seq++}`);
      trees.set(tSha, Array.from(baseEntries.values()));
      const cSha = sha(`commit:${seq++}:remote`);
      commits.set(cSha, { sha: cSha, tree: { sha: tSha }, message: 'Remote advance', parents: [{ sha: curHead }] });
      refs.set(`${owner}/${repo}:heads/${branch}`, cSha);
      return cSha;
    },
    deleteRemote(owner, repo, branch, path) {
      const curHead = refs.get(`${owner}/${repo}:heads/${branch}`);
      const parentCommit = commits.get(curHead);
      const baseEntries = new Map((trees.get(parentCommit.tree.sha) || []).map((e) => [e.path, e]));
      baseEntries.delete(path);
      const tSha = sha(`tree:${seq++}`);
      trees.set(tSha, Array.from(baseEntries.values()));
      const cSha = sha(`commit:${seq++}:delete:${path}`);
      commits.set(cSha, { sha: cSha, tree: { sha: tSha }, message: `Delete ${path}`, parents: [{ sha: curHead }] });
      refs.set(`${owner}/${repo}:heads/${branch}`, cSha);
      return cSha;
    },
  };
}

async function startMockOpenAIServer() {
  const calls = [];
  const server = createServer(async (req, res) => {
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Headers': 'content-type, authorization',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
    };
    if (req.method === 'OPTIONS') {
      res.writeHead(204, corsHeaders);
      res.end();
      return;
    }
    const chunks = [];
    for await (const c of req) chunks.push(c);
    const rawBody = Buffer.concat(chunks).toString('utf8');
    const body = rawBody ? JSON.parse(rawBody) : {};
    calls.push({ url: req.url, auth: req.headers.authorization || '', body });
    if (req.url !== '/v1/chat/completions') {
      res.writeHead(404, { ...corsHeaders, 'content-type': 'application/json' });
      res.end(JSON.stringify({ error: { message: 'Not found' } }));
      return;
    }
    if (req.headers.authorization !== 'Bearer sk-cdp-custom-secret-555') {
      res.writeHead(401, { ...corsHeaders, 'content-type': 'application/json' });
      res.end(JSON.stringify({ error: { message: 'Invalid Bearer token' } }));
      return;
    }
    const lastUser = [...(body.messages || [])].reverse().find((m) => m.role === 'user')?.content || '';
    if (!body.stream) {
      res.writeHead(200, { ...corsHeaders, 'content-type': 'application/json' });
      res.end(JSON.stringify({
        id: 'cmpl-cdp-test',
        model: body.model,
        choices: [{ message: { role: 'assistant', content: lastUser.includes('Reply with OK') ? 'OK' : `Reply: ${lastUser}` } }],
      }));
      return;
    }
    res.writeHead(200, { ...corsHeaders, 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
    if (lastUser.includes('<tool_response>')) {
      res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'Custom API Build App verified.' } }] })}\n\n`);
      res.write('data: [DONE]\n\n');
      res.end();
      return;
    }
    if (lastUser.includes('TRIGGER_CUSTOM_TOOL')) {
      res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: '<tool_call>{"name":"workspace_list","arguments":{"dir":""}}</tool_call>' } }] })}\n\n`);
      res.write('data: [DONE]\n\n');
      res.end();
      return;
    }
    res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'Custom API Home Chat verified.' } }] })}\n\n`);
    res.write('data: [DONE]\n\n');
    res.end();
  });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const { port } = server.address();
  return { server, origin: `http://127.0.0.1:${port}/v1`, calls };
}

async function verifyOriginPath(cdp, origin, basePath, packOrigin, mockGitHub, mockOpenAI) {
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

  // 1. Check catalog, scaffold & UI ModelPicker: built-in Qwen3-0.6B only; zero M1, zero weight upload, zero desktop window controls.
  const catalogCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const providers = await window.electron.ipcRenderer.invoke('get-language-model-providers');
      const models = await window.electron.ipcRenderer.invoke('get-language-models', { providerId: 'uor-r4' });
      const status = await window.electron.ipcRenderer.invoke('uor-r4:status');
      const settings = await window.electron.ipcRenderer.invoke('get-user-settings');
      const modelsJson = await (await fetch(new URL('models.json', location.href))).json();
      const scaffoldListResp = await fetch(new URL('scaffold/files.json', location.href));
      const viteEnvResp = await fetch(new URL('scaffold/src/vite-env.d.ts', location.href));
      const pickerBtn = document.querySelector('[data-testid="model-picker"]');
      const pickerText = pickerBtn?.textContent || '';
      const hasVoiceProBtn = Boolean(document.querySelector('[aria-label="Voice to text (Pro)"]'));
      const hasWinControls = ['Minimize', 'Maximize', 'Close'].some((label) =>
        Boolean(document.querySelector('[aria-label="' + label + '"]')),
      );
      const chatTabSelected = document.querySelector('[data-testid="home-mode-chat"]')?.getAttribute('aria-selected');
      const standards = await window.electron.ipcRenderer.invoke('standards:lookup', { query: 'softmax' });
      return {
        providerIds: providers.map((p) => p.id),
        uorModelNames: models.map((m) => m.apiName),
        localFileSource: modelsJson?.sources?.local_file,
        jsonIds: (modelsJson.models || []).map((m) => m.id),
        scaffoldFilesStatus: scaffoldListResp.status,
        viteEnvStatus: viteEnvResp.status,
        hasVoiceProBtn,
        hasWinControls,
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
  await clickCenter(cdp, sessionId, '[data-testid="model-picker"]');
  const pickerMenuCheck = await evalAsync(
    cdp,
    sessionId,
    `
      return {
        hasAddApiBtn: Boolean(document.querySelector('[data-testid="uor-r4-add-api-endpoint-button"]')),
        hasLocalUploadBtn: Boolean(document.querySelector('[data-testid="uor-r4-load-local-button"]')),
        hasLocalFileInput: Boolean(document.querySelector('[data-testid="uor-r4-local-file-input"], input[type="file"][accept*=".uorlut"]')),
        menuText: document.querySelector('[role="menu"]')?.textContent || '',
      };
    `,
    10000,
    `inspect open ModelPicker menu on ${basePath}`,
  );
  await cdp.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape' }, sessionId);
  await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape' }, sessionId);
  await new Promise((r) => setTimeout(r, 150));
  Object.assign(catalogCheck, pickerMenuCheck);
  assert(
    JSON.stringify(catalogCheck.providerIds) === JSON.stringify(['uor-r4']),
    `Expected sole default provider ['uor-r4'], got ${JSON.stringify(catalogCheck.providerIds)}`,
  );
  assert(
    JSON.stringify(catalogCheck.uorModelNames) === JSON.stringify(['uor-r4:qwen3-0.6b-foundry']),
    `Expected sole built-in model ['uor-r4:qwen3-0.6b-foundry'], got ${JSON.stringify(catalogCheck.uorModelNames)}`,
  );
  assert(
    JSON.stringify(catalogCheck.jsonIds) === JSON.stringify(['qwen3-0.6b-foundry']) && catalogCheck.localFileSource === false,
    `Unexpected models.json: ids=${JSON.stringify(catalogCheck.jsonIds)} local_file=${catalogCheck.localFileSource}`,
  );
  assert(
    catalogCheck.scaffoldFilesStatus === 200 && catalogCheck.viteEnvStatus === 200,
    `Expected HTTP 200 for scaffold/files.json & scaffold/src/vite-env.d.ts: ${JSON.stringify(catalogCheck)}`,
  );
  assert(
    catalogCheck.hasAddApiBtn && !catalogCheck.hasLocalUploadBtn && !catalogCheck.hasLocalFileInput,
    `Expected Custom API endpoint button and zero weight upload controls: ${JSON.stringify(catalogCheck)}`,
  );
  assert(
    !catalogCheck.hasVoiceProBtn && !catalogCheck.hasWinControls,
    `Desktop window controls or Pro voice button still rendered: ${JSON.stringify(catalogCheck)}`,
  );
  assert(
    !catalogCheck.pickerText.includes('synthetic-seed-5') &&
      !catalogCheck.pickerText.includes('Foundry reader') &&
      !catalogCheck.pickerText.includes('M1 '),
    `Removed/dummy label leaked in ModelPicker: ${catalogCheck.pickerText}`,
  );
  assert(catalogCheck.chatTabSelected === 'true', `Home Chat tab should default active: ${JSON.stringify(catalogCheck)}`);
  assert(catalogCheck.standardsCount > 0, `standards:lookup returned 0 matches for 'softmax'`);
  console.error(`[verify] catalog, scaffold & clean UI controls OK (${catalogCheck.uorModelNames.join(', ')})`);

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
      // Allow the V13 WASM worker to unwind the cancelled token chunk before starting the next turn.
      await new Promise((r) => setTimeout(r, 1500));
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

  // 5b. Verify create-app scaffold files (`src/vite-env.d.ts`), Zod-contracted internal controls, routes, and local preview mode.
  const controlsCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const appObj = await window.electron.ipcRenderer.invoke('get-app', ${builtFromHome.appId});
      const previewFolder = await window.electron.ipcRenderer.invoke('preview-app-folder-name', { name: 'My Renamed App', appId: ${builtFromHome.appId} });
      const commitObj = await window.electron.ipcRenderer.invoke('app:get-current-commit-hash', { appId: ${builtFromHome.appId} });
      const renamed = await window.electron.ipcRenderer.invoke('rename-app', {
        appId: ${builtFromHome.appId},
        appName: 'Build App Verified',
        appPath: 'build-app-verified',
      });
      const fav1 = await window.electron.ipcRenderer.invoke('add-to-favorite', { appId: ${builtFromHome.appId} });
      const col = await window.electron.ipcRenderer.invoke('appCollections:create', { name: 'Verified Apps', appIds: [${builtFromHome.appId}] });
      const colList = await window.electron.ipcRenderer.invoke('appCollections:list');
      await window.electron.ipcRenderer.invoke('set-app-env-vars', { appId: ${builtFromHome.appId}, envVars: [{ key: 'UOR_ENV', value: 'studio' }] });
      const envVars = await window.electron.ipcRenderer.invoke('get-app-env-vars', { appId: ${builtFromHome.appId} });
      await window.electron.ipcRenderer.invoke('update-app-commands', { appId: ${builtFromHome.appId}, installCommand: null, startCommand: null });
      const problems = await window.electron.ipcRenderer.invoke('check-problems', { appId: ${builtFromHome.appId} });
      const templates = await window.electron.ipcRenderer.invoke('get-templates');
      const mcpServers = await window.electron.ipcRenderer.invoke('mcp:list-servers');
      const imgSub = await window.electron.ipcRenderer.invoke('distributed-machine:subscribe', { protocolVersion: 1, machineId: 'image_generation', encodedKey: { scope: 'jobs' } });
      const imgDispatch = await window.electron.ipcRenderer.invoke('distributed-machine:dispatch', { protocolVersion: 1, machineId: 'image_generation', encodedKey: { scope: 'jobs' }, messageId: 'cdp-img-1', encodedEvent: { type: 'JOB_REQUESTED' } });
      const ghOpsSub = await window.electron.ipcRenderer.invoke('distributed-machine:subscribe', { protocolVersion: 1, machineId: 'github_ops', encodedKey: { appId: ${builtFromHome.appId} } });
      const ghOpsZeroReject = await window.electron.ipcRenderer.invoke('distributed-machine:dispatch', {
        protocolVersion: 1,
        machineId: 'github_ops',
        encodedKey: { appId: 0 },
        messageId: 'cdp-gh-zero',
        encodedEvent: { type: 'OP_REQUESTED', op: { type: 'pull' }, operationId: 'cdp-gh-zero' },
      });
      await window.electron.ipcRenderer.invoke('distributed-machine:dispatch', {
        protocolVersion: 1,
        machineId: 'app_run',
        encodedKey: { appId: ${builtFromHome.appId} },
        messageId: 'cdp-run-1',
        encodedEvent: { type: 'START', operationId: 'cdp-run-1', startedAt: Date.now(), expectedRevision: 0 },
      });
      let appRunState = null;
      for (let i = 0; i < 40; i++) {
        const sub = await window.electron.ipcRenderer.invoke('distributed-machine:subscribe', { protocolVersion: 1, machineId: 'app_run', encodedKey: { appId: ${builtFromHome.appId} } });
        appRunState = sub?.encodedState;
        if (appRunState?.phase === 'ready' || appRunState?.phase === 'errored') break;
        await new Promise((r) => setTimeout(r, 100));
      }
      const copied = await window.electron.ipcRenderer.invoke('copy-app', { appId: ${builtFromHome.appId}, newAppName: 'Temp Copied App', withHistory: false });
      const delCopied = await window.electron.ipcRenderer.invoke('delete-apps', { appIds: [copied.app.id] });
      await window.electron.ipcRenderer.invoke('appCollections:delete', col.id);

      const routeChecks = {};
      for (const h of [
        '#/app-details?appId=${builtFromHome.appId}',
        '#/settings',
        '#/plugins',
        '#/templates',
        '#/library',
        '#/chat?id=${builtFromHome.chatId}&appId=${builtFromHome.appId}',
      ]) {
        location.hash = h;
        await new Promise((r) => setTimeout(r, 500));
        const main = document.getElementById('layout-main-content-container');
        const txt = (main?.innerText || '').trim();
        const isAppDetails = h.startsWith('#/app-details');
        const isSettings = h === '#/settings';
        const hasError = /Something went wrong|Sorry, that shouldn't have happened|Error loading/i.test(txt);
        const appDetailsOk = !isAppDetails || Boolean(document.querySelector('[data-testid="app-details-page"]'));
        const settingsOk =
          !isSettings ||
          (txt.includes('Custom AI API') && (txt.includes('AI Model Endpoints') || txt.includes('AI model endpoints')));
        routeChecks[h] = { ok: txt.length > 0 && !hasError && appDetailsOk && settingsOk, preview: txt.slice(0, 120) };
      }
      const noSecurityTab = !document.querySelector('[data-testid="security-mode-button"]');
      const noTestsTab = !document.querySelector('[data-testid="tests-mode-button"]');
      return {
        hasViteEnv: (appObj.files || []).includes('src/vite-env.d.ts'),
        previewFolder,
        commitObj,
        renamed,
        fav1,
        colListLen: colList.length,
        envVars,
        problems,
        templatesCount: templates.length,
        mcpServersCount: mcpServers.length,
        imgJobs: imgSub?.encodedState?.jobs,
        imgDispatch,
        ghOpsType: ghOpsSub?.encodedState?.state?.type,
        ghOpsZeroReject,
        appRunMode: appRunState?.url?.mode || null,
        delCopied,
        routeChecks,
        noSecurityTab,
        noTestsTab,
      };
    `,
    45000,
    `internal controls & routes check on ${basePath}`,
  );
  assert(controlsCheck.hasViteEnv, `Created app missing src/vite-env.d.ts: ${JSON.stringify(controlsCheck)}`);
  assert(controlsCheck.previewFolder?.folderName === 'My Renamed App', `preview-app-folder-name mismatch: ${JSON.stringify(controlsCheck.previewFolder)}`);
  assert(typeof controlsCheck.commitObj?.commitHash === 'string', `app:get-current-commit-hash mismatch: ${JSON.stringify(controlsCheck.commitObj)}`);
  assert(controlsCheck.renamed?.name === 'Build App Verified' && controlsCheck.renamed?.path === 'build-app-verified', `rename-app mismatch: ${JSON.stringify(controlsCheck.renamed)}`);
  assert(controlsCheck.fav1?.isFavorite === true, `add-to-favorite failed: ${JSON.stringify(controlsCheck.fav1)}`);
  assert(controlsCheck.envVars?.[0]?.key === 'UOR_ENV', `env vars mismatch: ${JSON.stringify(controlsCheck.envVars)}`);
  assert(controlsCheck.problems?.outcome === 'passed', `check-problems failed: ${JSON.stringify(controlsCheck.problems)}`);
  assert(
    controlsCheck.templatesCount >= 1 &&
      Array.isArray(controlsCheck.imgJobs) &&
      controlsCheck.imgDispatch?.kind === 'ignored' &&
      Number.isSafeInteger(controlsCheck.imgDispatch?.transactionSequence) &&
      controlsCheck.ghOpsType === 'idle' &&
      controlsCheck.ghOpsZeroReject?.kind === 'rejected' &&
      controlsCheck.ghOpsZeroReject?.reason === 'invalid-key' &&
      controlsCheck.appRunMode === 'host',
    `controls check failed: ${JSON.stringify(controlsCheck)}`,
  );
  assert(controlsCheck.delCopied?.results?.[0]?.success === true, `copy/delete-apps failed: ${JSON.stringify(controlsCheck.delCopied)}`);
  assert(Object.values(controlsCheck.routeChecks).every((r) => r.ok), `A route failed or hit ErrorBoundary: ${JSON.stringify(controlsCheck.routeChecks)}`);
  assert(controlsCheck.noSecurityTab && controlsCheck.noTestsTab, `Unavailable security/tests tabs still visible`);
  console.error(`[verify] internal Studio controls & all routes OK: ${JSON.stringify(controlsCheck.routeChecks)}`);

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
      const debugTestIds = Array.from(document.querySelectorAll('[data-testid]')).map((el) => el.getAttribute('data-testid'));
      const debugMainText = (document.getElementById('layout-main-content-container')?.innerText || '').slice(0, 200);
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
        debugTestIds,
        debugMainText,
      };
    `,
    45000,
    `MVM & prism-write check on ${basePath}`,
  );
  assert(mvmCheck.unameStdout.includes('riscv64'), `MVM uname did not report riscv64: ${JSON.stringify(mvmCheck)}`);
  assert(mvmCheck.unameRetired > 0, `MVM instructions_retired must be > 0: ${mvmCheck.unameRetired}`);
  assert(mvmCheck.probeOut1.trim() === 'hello-cow', `MVM /tmp/probe.txt failed: ${JSON.stringify(mvmCheck)}`);
  assert(mvmCheck.readWritten === 'uor-v13-ok', `prism-write commit failed: ${JSON.stringify(mvmCheck)}`);
  assert(mvmCheck.hasTerminalTab, `Terminal tab [data-testid="terminal-mode-button"] missing (hash=${mvmCheck.hashAfterNav}, main=${JSON.stringify(mvmCheck.debugMainText)}, testids=${JSON.stringify(mvmCheck.debugTestIds)})`);

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
  await new Promise((r) => setTimeout(r, 1200));
  await waitForEval(cdp, sessionId, `Boolean(!window.__uorBeforeReload && window.electron && window.__uorR4 && (document.querySelector('#root')?.children.length || 0) > 0)`, 20000);
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
      location.hash = '#/app-details?appId=${mvmCheck.appId}';
      await new Promise((r) => setTimeout(r, 600));
      const appDetailsTextAfterReload = document.getElementById('layout-main-content-container')?.innerText || '';
      location.hash = ${JSON.stringify(mvmCheck.hashAfterNav)};
      await new Promise((r) => setTimeout(r, 400));
      return { ...res, written, appDetailsTextAfterReload, hashAfterReload: location.hash, rootMounted: (document.querySelector('#root')?.children.length || 0) > 0 };
    `,
    30000,
    `reloadCheck on ${basePath}`,
  );
  assert(reloadCheck.rootMounted, `Root UI failed to mount after reload on ${basePath}`);
  assert(
    reloadCheck.appDetailsTextAfterReload.includes('Build App Verified'),
    `Renamed app did not render on #/app-details after reload: ${reloadCheck.appDetailsTextAfterReload.slice(0, 200)}`,
  );
  assert(reloadCheck.stdout.trim() === 'hello-cow', `MVM CoW failed across reload: ${JSON.stringify(reloadCheck)}`);
  assert(reloadCheck.written === 'uor-v13-ok', `App file failed across reload: ${JSON.stringify(reloadCheck)}`);

  // 7. Check 6: Real Linux 6.18.7 VM (v86) + bidirectional host9p /workspace sync + runtime toggle.
  console.error(`[verify] booting Linux 6.18.7 v86 VM on ${basePath}...`);
  const linuxCheck = await evalAsync(
    cdp,
    sessionId,
    `
      await window.electron.ipcRenderer.invoke('terminal:set-runtime', { mode: 'linux', appId: ${mvmCheck.appId} });
      const rtLinux = await window.electron.ipcRenderer.invoke('terminal:get-runtime');
      const openLinux = await window.electron.ipcRenderer.invoke('terminal:open', { appId: ${mvmCheck.appId}, cols: 80, rows: 24 });
      const res = await window.__uorR4.linuxExec(
        'uname -sr && ls -la /workspace && mkdir -p /workspace/app2_only_dir && printf "nested" > /workspace/app2_only_dir/nested.txt && printf "from-linux-618" > /workspace/linux_note.txt && printf "temp-rm" > /workspace/linux_rm_probe.txt',
        ${mvmCheck.appId},
        { timeoutMs: 30000 }
      );
      const syncedFile = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: ${mvmCheck.appId},
        filePath: 'linux_note.txt',
      });
      const rmProbeSynced = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: ${mvmCheck.appId},
        filePath: 'linux_rm_probe.txt',
      });
      await window.__uorR4.linuxExec('rm -f /workspace/linux_rm_probe.txt', ${mvmCheck.appId}, { timeoutMs: 20000 });
      const rmProbeAfterDelete = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: ${mvmCheck.appId},
        filePath: 'linux_rm_probe.txt',
      }).catch(() => null);
      const lsAfterRm = await window.__uorR4.linuxExec('ls -1a /workspace', ${mvmCheck.appId}, { timeoutMs: 20000 });
      await window.electron.ipcRenderer.invoke('terminal:set-runtime', { mode: 'mvm', appId: ${mvmCheck.appId} });
      const rtMvm = await window.electron.ipcRenderer.invoke('terminal:get-runtime');
      const openMvm = await window.electron.ipcRenderer.invoke('terminal:open', { appId: ${mvmCheck.appId}, cols: 80, rows: 24 });
      await window.electron.ipcRenderer.invoke('terminal:set-runtime', { mode: 'linux', appId: ${mvmCheck.appId} });
      const crossAppSwitch = await window.__uorR4.linuxExec(
        'ls -1a /workspace',
        ${promoted.appId},
        { timeoutMs: 20000 }
      );
      const promotedHasApp2File = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: ${promoted.appId},
        filePath: 'linux_note.txt',
      }).catch(() => null);
      return { rtLinux, openLinux, res, syncedFile, rmProbeSynced, rmProbeAfterDelete, lsAfterRm, rtMvm, openMvm, crossAppSwitch, promotedHasApp2File };
    `,
    120000,
    `linuxVmCheck on ${basePath}`,
  );
  assert(linuxCheck.rtLinux.mode === 'linux', `Expected linux runtime mode: ${JSON.stringify(linuxCheck.rtLinux)}`);
  assert(linuxCheck.openLinux.sessionId.startsWith('linux-'), `Expected linux- sessionId: ${JSON.stringify(linuxCheck.openLinux)}`);
  assert(linuxCheck.res.exit_code === 0, `Linux exec failed: ${JSON.stringify(linuxCheck.res)}`);
  assert(linuxCheck.res.stdout.includes('Linux 6.18'), `Expected Linux 6.18.7 uname: ${JSON.stringify(linuxCheck.res)}`);
  assert(linuxCheck.res.stdout.includes('uor-probe.txt') || linuxCheck.res.stdout.includes('package.json') || linuxCheck.res.stdout.includes('src'), `Expected Studio files inside /workspace: ${linuxCheck.res.stdout}`);
  assert(linuxCheck.syncedFile === 'from-linux-618', `9p -> IndexedDB sync failed: ${JSON.stringify(linuxCheck)}`);
  assert(linuxCheck.rmProbeSynced === 'temp-rm', `Expected rm probe file to sync first: ${JSON.stringify(linuxCheck)}`);
  assert(linuxCheck.rmProbeAfterDelete == null, `In-VM rm did not delete file from IndexedDB: ${JSON.stringify(linuxCheck.rmProbeAfterDelete)}`);
  assert(!linuxCheck.lsAfterRm.stdout.includes('linux_rm_probe.txt'), `In-VM rm resurrected deleted file on re-sync: ${linuxCheck.lsAfterRm.stdout}`);
  assert(linuxCheck.rtMvm.mode === 'mvm' && linuxCheck.openMvm.sessionId.startsWith('mvm-'), `MVM runtime toggle failed`);
  assert(!linuxCheck.crossAppSwitch.stdout.includes('linux_note.txt') && !linuxCheck.crossAppSwitch.stdout.includes('app2_only_dir'), `Cross-app /workspace file or dir bleed detected: ${linuxCheck.crossAppSwitch.stdout}`);
  assert(linuxCheck.promotedHasApp2File == null, `Cross-app IndexedDB bleed detected into promoted app: ${linuxCheck.promotedHasApp2File}`);

  // 8. Check 7: Real browser-native GitHub flow (ChaCha20-Poly1305 vault, clone, diff, CAS push, conflict, pull).
  console.error(`[verify] verifying browser-native GitHub + ChaCha20-Poly1305 vault on ${basePath}...`);
  const ghStep1 = await evalAsync(
    cdp,
    sessionId,
    `
      localStorage.setItem('uor.github.apiBase', ${JSON.stringify(mockGitHub.origin)});
      const conn = await window.electron.ipcRenderer.invoke('github:connect-token', {
        token: 'ghp_cdp_stage3_secret_token_777',
        apiBase: ${JSON.stringify(mockGitHub.origin)},
      });
      const flowStates = await window.electron.ipcRenderer.invoke('connection-flow:get-states');
      const rawIdbDump = await new Promise((resolve, reject) => {
        const req = indexedDB.open('hologram-forge', 2);
        req.onsuccess = () => {
          const db = req.result;
          const tx = db.transaction('refs', 'readonly');
          const g = tx.objectStore('refs').get('vault:github:credentials');
          g.onsuccess = () => resolve(JSON.stringify(g.result || null));
          g.onerror = () => reject(g.error);
        };
        req.onerror = () => reject(req.error);
      });
      const localDump = JSON.stringify(localStorage);
      const cloned = await window.electron.ipcRenderer.invoke('github:clone-repo-from-url', {
        url: 'octocat/hello-uor',
      });
      if (cloned.error) throw new Error('clone failed: ' + cloned.error);
      // Verify cloning an unrelated app does NOT overwrite app #1's /workspace in Linux VM (activeAppId stays promotedAppId).
      const linuxStAfterClone = await window.__uorR4.linuxStatus();
      const clonedContent = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: cloned.app.id,
        filePath: 'src/main.ts',
      });
      await window.electron.ipcRenderer.invoke('edit-app-file', {
        appId: cloned.app.id,
        filePath: 'src/main.ts',
        content: 'export const greeting = "hello v2 from studio";\\n',
      });
      const diffRes = await window.electron.ipcRenderer.invoke('github:diff', { appId: cloned.app.id });
      const pushRes = await window.electron.ipcRenderer.invoke('github:push', {
        appId: cloned.app.id,
        message: 'CDP push from Studio',
      });
      return { conn, flowStates, rawIdbDump, localDump, clonedAppId: cloned.app.id, clonedContent, diffRes, pushRes, linuxStAfterClone };
    `,
    60000,
    `githubStep1 on ${basePath}`,
  );
  assert(ghStep1.linuxStAfterClone.mountedAppId === promoted.appId, `Clone hijacked Linux VM mount: ${JSON.stringify(ghStep1.linuxStAfterClone)}`);
  assert(ghStep1.conn.connected && ghStep1.conn.login === 'octocat', `GitHub connect failed: ${JSON.stringify(ghStep1.conn)}`);
  assert(
    ghStep1.flowStates?.github?.status === 'connected' &&
      ghStep1.flowStates?.github?.provider === 'github' &&
      ghStep1.flowStates?.github?.invocationRef?.kind === 'connection-flow' &&
      ghStep1.flowStates?.github?.invocationRef?.entityKey === 'github' &&
      Boolean(ghStep1.flowStates?.github?.invocationRef?.operationId),
    `Invalid connected connection-flow state: ${JSON.stringify(ghStep1.flowStates)}`,
  );
  assert(!ghStep1.rawIdbDump.includes('ghp_cdp_stage3_secret_token_777'), 'Plaintext PAT leaked into raw IndexedDB!');
  assert(!ghStep1.localDump.includes('ghp_cdp_stage3_secret_token_777'), 'Plaintext PAT leaked into localStorage!');
  assert(ghStep1.clonedContent === 'export const greeting = "hello v1";\n', `Clone content mismatch: ${ghStep1.clonedContent}`);
  assert(ghStep1.diffRes.modified.includes('src/main.ts'), `GitHub diff missed edit: ${JSON.stringify(ghStep1.diffRes)}`);
  assert(ghStep1.pushRes.ok && ghStep1.pushRes.filesPushed === 1, `GitHub push failed: ${JSON.stringify(ghStep1.pushRes)}`);
  assert(mockGitHub.getLastPatchForce() === false, `CAS push must set force: false, got ${mockGitHub.getLastPatchForce()}`);

  mockGitHub.advanceRemote('octocat', 'hello-uor', 'main', 'REMOTE.md', 'from-remote-head\n');
  const ghStep2 = await evalAsync(
    cdp,
    sessionId,
    `
      const stalePush = await window.electron.ipcRenderer.invoke('github:push', {
        appId: ${ghStep1.clonedAppId},
        message: 'Should refuse stale push',
      });
      const pullRes = await window.electron.ipcRenderer.invoke('github:pull', {
        appId: ${ghStep1.clonedAppId},
      });
      const remoteFile = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: ${ghStep1.clonedAppId},
        filePath: 'REMOTE.md',
      });
      return { stalePush, pullRes, remoteFile };
    `,
    60000,
    `githubStep2 on ${basePath}`,
  );
  assert(!ghStep2.stalePush.ok && ghStep2.stalePush.conflict, `Stale push did not conflict-refuse: ${JSON.stringify(ghStep2.stalePush)}`);
  assert(ghStep2.pullRes.ok && ghStep2.remoteFile === 'from-remote-head\n', `GitHub pull failed: ${JSON.stringify(ghStep2)}`);

  mockGitHub.deleteRemote('octocat', 'hello-uor', 'main', 'REMOTE.md');
  const ghStep3 = await evalAsync(
    cdp,
    sessionId,
    `
      const pullDelRes = await window.electron.ipcRenderer.invoke('github:pull', {
        appId: ${ghStep1.clonedAppId},
      });
      const deletedRemoteFile = await window.electron.ipcRenderer.invoke('read-app-file', {
        appId: ${ghStep1.clonedAppId},
        filePath: 'REMOTE.md',
      }).catch(() => null);
      await window.electron.ipcRenderer.invoke('github:disconnect');
      localStorage.removeItem('uor.github.apiBase');
      return { pullDelRes, deletedRemoteFile };
    `,
    60000,
    `githubStep3 on ${basePath}`,
  );
  assert(ghStep3.pullDelRes.ok && ghStep3.pullDelRes.deletedCount === 1, `GitHub pull deletion failed: ${JSON.stringify(ghStep3)}`);
  assert(ghStep3.deletedRemoteFile == null, `GitHub pull did not delete REMOTE.md: ${JSON.stringify(ghStep3)}`);

  // 9. Check 8: Stitch networking + afflom/me app.json & ?embed=me postMessage bridge.
  console.error(`[verify] verifying Stitch + afflom/me ?embed=me bridge on ${basePath}...`);
  const stitchCheck = await evalAsync(
    cdp,
    sessionId,
    `
      const appJsonResp = await fetch(new URL('app.json', location.href));
      const appJson = await appJsonResp.json();
      const stBefore = await window.__uorR4.stitchStatus();
      const stConn = await window.__uorR4.stitchConnect({ channelKey: 'uor-cdp-room' });
      const stDisc = await window.__uorR4.stitchDisconnect();

      // Test ?embed=me postMessage bridge in a same-origin hidden iframe.
      const iframe = document.createElement('iframe');
      iframe.style.cssText = 'position:fixed;left:-9999px;top:-9999px;width:320px;height:240px;';
      const bridgeResult = await new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error('?embed=me bridge timeout')), 20000);
        const onMsg = (ev) => {
          if (ev.origin !== location.origin || ev.data?.protocol !== 'uor-workspace-bridge/1') return;
          if (ev.data.event === 'ready') {
            iframe.contentWindow.postMessage({ protocol: 'uor-workspace-bridge/1', id: 'p1', action: 'ping' }, location.origin);
          } else if (ev.data.id === 'p1') {
            iframe.contentWindow.postMessage({ protocol: 'uor-workspace-bridge/1', id: 's1', action: 'get-status' }, location.origin);
          } else if (ev.data.id === 's1') {
            clearTimeout(timer);
            window.removeEventListener('message', onMsg);
            resolve({ ping: true, statusRes: ev.data });
          }
        };
        window.addEventListener('message', onMsg);
        const u = new URL(location.href);
        u.hash = '';
        u.search = '?embed=me';
        iframe.src = u.href;
        document.body.appendChild(iframe);
      });
      iframe.remove();
      return { appJsonOk: appJsonResp.ok, appJson, stBefore, stConn, stDisc, bridgeResult };
    `,
    60000,
    `stitchAndMeCheck on ${basePath}`,
  );
  assert(stitchCheck.appJsonOk && stitchCheck.appJson.embed?.protocol === 'uor-workspace-bridge/1', `Invalid app.json: ${JSON.stringify(stitchCheck.appJson)}`);
  assert(stitchCheck.stBefore.peerId?.startsWith('key-') && stitchCheck.stBefore.connected === false, `Stitch status unexpected: ${JSON.stringify(stitchCheck.stBefore)}`);
  assert(stitchCheck.stConn.connected === true && stitchCheck.stDisc.connected === false, `Stitch connect/disconnect failed`);
  assert(stitchCheck.bridgeResult.statusRes.ok && stitchCheck.bridgeResult.statusRes.result?.stitch?.peerId, `Bridge status failed: ${JSON.stringify(stitchCheck.bridgeResult)}`);

  // 10. Check 9: Custom AI API Endpoint UI configuration, ChaCha20-Poly1305 vault, Test Connection, Home/Studio tool loop, and fallback delete.
  console.error(`[verify] verifying Custom AI API Endpoint UI & R4 routed chat on ${basePath}...`);
  await evalAsync(
    cdp,
    sessionId,
    `
      location.hash = '#/settings';
    `,
    10000,
    `navigate to settings for Custom API UI`,
  );
  await waitForEval(cdp, sessionId, `Boolean(document.querySelector('[data-testid="custom-api-add-button"]'))`, 20000);
  await evalAsync(
    cdp,
    sessionId,
    `
      document.querySelector('[data-testid="custom-api-add-button"]').click();
      await new Promise((r) => setTimeout(r, 150));
      const setVal = (sel, val) => {
        const el = document.querySelector(sel);
        if (!el) throw new Error('Missing input: ' + sel);
        const desc = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
        desc.set.call(el, val);
        el.dispatchEvent(new Event('input', { bubbles: true }));
      };
      setVal('[data-testid="custom-api-name-input"]', 'Loopback Qwen');
      setVal('[data-testid="custom-api-model-input"]', 'custom-qwen3');
      setVal('[data-testid="custom-api-base-url-input"]', ${JSON.stringify(mockOpenAI.origin)});
      setVal('[data-testid="custom-api-key-input"]', 'sk-cdp-custom-secret-555');
      await new Promise((r) => setTimeout(r, 100));
      document.querySelector('[data-testid="custom-api-test-button"]').click();
    `,
    15000,
    `fill and test Custom AI API form`,
  );
  const customTestBanner = await waitForEval(
    cdp,
    sessionId,
    `document.querySelector('[data-testid="custom-api-test-result"]')?.textContent || null`,
    15000,
  );
  assert(customTestBanner.includes('Connected'), `Expected Connected test result, got: ${customTestBanner}`);

  const customChatCheck = await evalAsync(
    cdp,
    sessionId,
    `
      document.querySelector('[data-testid="custom-api-save-button"]').click();
      let settingsAfterSave = null;
      for (let i = 0; i < 60; i++) {
        const rows = document.querySelectorAll('[data-testid^="custom-api-row-"]');
        settingsAfterSave = await window.electron.ipcRenderer.invoke('get-user-settings');
        if (rows.length > 0 && settingsAfterSave?.selectedModel?.provider === 'custom-api') break;
        await new Promise((r) => setTimeout(r, 100));
      }
      const providers = await window.electron.ipcRenderer.invoke('get-language-model-providers');
      const endpoints = await window.electron.ipcRenderer.invoke('custom-api:list');
      location.hash = '#/';
      await new Promise((r) => setTimeout(r, 300));
      const pickerTextCustom = document.querySelector('[data-testid="model-picker"]')?.textContent || '';
      const rawIdbDump = await new Promise((resolve, reject) => {
        const req = indexedDB.open('hologram-forge', 2);
        req.onsuccess = () => {
          const db = req.result;
          const tx = db.transaction('refs', 'readonly');
          const g = tx.objectStore('refs').get('vault:custom-api:endpoints');
          g.onsuccess = () => resolve(JSON.stringify(g.result || null));
          g.onerror = () => reject(g.error);
        };
        req.onerror = () => reject(req.error);
      });
      const localDump = JSON.stringify(localStorage);

      const homeReply = await window.electron.ipcRenderer.invoke('home-chat:send', {
        prompt: 'Confirm custom endpoint connectivity',
        provider: 'custom-api',
        modelId: endpoints[0].id,
      });

      const appId = ${builtFromHome.appId};
      const chatId = ${builtFromHome.chatId};
      await window.electron.ipcRenderer.invoke('update-chat', {
        chatId,
        modelSelection: { provider: 'custom-api', name: endpoints[0].id, effortLevel: 'medium' },
      });
      await window.__uorR4.submitPrompt(chatId, appId, 'TRIGGER_CUSTOM_TOOL inspect workspace');
      let studioContent = '';
      for (let i = 0; i < 80; i++) {
        const sub = await window.electron.ipcRenderer.invoke('distributed-machine:subscribe', { protocolVersion: 1, machineId: 'chat_stream', encodedKey: { chatId } });
        const chat = await window.electron.ipcRenderer.invoke('get-chat', chatId);
        const last = (chat.messages || []).at(-1);
        if (sub?.encodedState?.phase === 'idle' && last?.role === 'assistant' && String(last.content || '').includes('Custom API Build App verified')) {
          studioContent = last.content;
          break;
        }
        await new Promise((r) => setTimeout(r, 150));
      }

      location.hash = '#/settings';
      await new Promise((r) => setTimeout(r, 300));
      document.querySelector('[data-testid^="custom-api-delete-"]')?.click();
      let settingsAfterDelete = null;
      let chatAfterDelete = null;
      for (let i = 0; i < 40; i++) {
        settingsAfterDelete = await window.electron.ipcRenderer.invoke('get-user-settings');
        chatAfterDelete = await window.electron.ipcRenderer.invoke('get-chat', chatId);
        if (settingsAfterDelete?.selectedModel?.provider === 'uor-r4' && chatAfterDelete?.modelSelection?.provider === 'uor-r4') break;
        await new Promise((r) => setTimeout(r, 100));
      }
      const countAfterDelete = await window.electron.ipcRenderer.invoke('chat:count-tokens', { chatId, input: 'check fallback tokens' });
      return {
        providerIds: providers.map((p) => p.id),
        endpointCount: endpoints.length,
        endpointPublic: endpoints[0],
        pickerTextCustom,
        settingsAfterSave: settingsAfterSave.selectedModel,
        rawIdbHasSecret: rawIdbDump.includes('sk-cdp-custom-secret-555'),
        localStorageHasSecret: localDump.includes('sk-cdp-custom-secret-555'),
        homeOutcome: homeReply.outcome,
        homeLast: homeReply.messages?.at(-1)?.content || '',
        studioContent,
        settingsAfterDelete: settingsAfterDelete.selectedModel,
        chatModelAfterDelete: chatAfterDelete?.modelSelection || null,
        countContextWindowAfterDelete: countAfterDelete?.contextWindow || 0,
      };
    `,
    60000,
    `Custom AI API Endpoint E2E on ${basePath}`,
  );
  assert(customChatCheck.providerIds.includes('custom-api'), `custom-api missing from providers: ${JSON.stringify(customChatCheck.providerIds)}`);
  assert(customChatCheck.pickerTextCustom.includes('Loopback Qwen'), `ModelPicker did not display Loopback Qwen: ${customChatCheck.pickerTextCustom}`);
  assert(!('apiKey' in (customChatCheck.endpointPublic || {})), `Public custom-api:list leaked apiKey`);
  assert(!customChatCheck.rawIdbHasSecret && !customChatCheck.localStorageHasSecret, `Plaintext Custom API key leaked in storage`);
  assert(customChatCheck.homeOutcome === 'completed' && customChatCheck.homeLast.includes('Custom API Home Chat verified'), `Custom API Home Chat failed: ${JSON.stringify(customChatCheck)}`);
  assert(customChatCheck.studioContent.includes('<tool_response>') && customChatCheck.studioContent.includes('Custom API Build App verified'), `Custom API Studio tool loop failed: ${JSON.stringify(customChatCheck)}`);
  assert(
    customChatCheck.settingsAfterDelete?.provider === 'uor-r4' &&
      customChatCheck.settingsAfterDelete?.name === 'uor-r4:qwen3-0.6b-foundry' &&
      customChatCheck.chatModelAfterDelete?.provider === 'uor-r4' &&
      customChatCheck.chatModelAfterDelete?.name === 'uor-r4:qwen3-0.6b-foundry' &&
      customChatCheck.countContextWindowAfterDelete > 0,
    `Delete fallback failed: ${JSON.stringify(customChatCheck)}`,
  );

  const forbiddenIpcErrors = [];
  for (const ev of cdp.events.slice(evStart)) {
    if (ev.method === 'Runtime.consoleAPICalled' || ev.method === 'Runtime.exceptionThrown') {
      const txt = JSON.stringify(ev.params || '');
      if (txt.includes('unsupportedChannel') || txt.includes('handlerFailed') || txt.includes('no browser machine')) {
        forbiddenIpcErrors.push(txt.slice(0, 300));
      }
    }
  }
  assert(forbiddenIpcErrors.length === 0, `Detected forbidden IPC errors during CDP walk on ${basePath}: ${JSON.stringify(forbiddenIpcErrors)}`);
  console.error(`[verify] Custom AI API Endpoint E2E & zero IPC error check OK`);

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
    linuxUname: linuxCheck.res.stdout.split('\n')[0],
    githubPushSha: ghStep1.pushRes.commitSha,
    stitchPeerId: stitchCheck.stBefore.peerId,
    embedMeBridgeOk: Boolean(stitchCheck.bridgeResult.statusRes.ok),
    customApiVerified: true,
  };
}

async function runIsolatedOriginPath(origin, basePath, packOrigin) {
  const profileDir = await mkdtemp(join(tmpdir(), 'uor-foundry-cdp-'));
  const mockGitHub = await startMockGitHubServer();
  const mockOpenAI = await startMockOpenAIServer();
  let chrome = null;
  let cdp = null;
  try {
    chrome = await launchChrome(profileDir);
    cdp = await cdpConnect(chrome.wsUrl);
    return await verifyOriginPath(cdp, origin, basePath, packOrigin, mockGitHub, mockOpenAI);
  } finally {
    try { cdp?.ws?.close(); } catch {}
    try { chrome?.child?.kill('SIGKILL'); } catch {}
    try { mockGitHub?.server?.close(); } catch {}
    try { mockOpenAI?.server?.close(); } catch {}
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
    const rootReport = await withTimeout(runIsolatedOriginPath(origin, '/', packSrv?.origin || null), 360000, 'origin /');
    const subpathReport = await withTimeout(runIsolatedOriginPath(origin, '/uor-r4/studio/', packSrv?.origin || null), 360000, 'origin /uor-r4/studio/');
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
