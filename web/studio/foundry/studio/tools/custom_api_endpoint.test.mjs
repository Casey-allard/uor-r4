import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { createServer } from 'node:http';
import { test } from 'node:test';
import {
  validateCustomApiBaseUrl,
  resolveChatCompletionsUrl,
  validateCustomApiName,
  validateCustomApiModel,
  validateCustomApiKey,
  buildOpenAiMessages,
  createCustomApiClient,
} from '../shell/custom-api-client.mjs';
import { runUorR4AgentTurn } from '../shell/r4-agent-loop.mjs';

function makeChaChaCore() {
  return {
    seal(key, plaintext) {
      const iv = crypto.randomBytes(12);
      const cipher = crypto.createCipheriv('chacha20-poly1305', key, iv, { authTagLength: 16 });
      const ct = Buffer.concat([cipher.update(plaintext), cipher.final()]);
      const tag = cipher.getAuthTag();
      return new Uint8Array(Buffer.concat([iv, ct, tag]));
    },
    open(key, sealed) {
      const buf = Buffer.from(sealed);
      const iv = buf.subarray(0, 12);
      const tag = buf.subarray(buf.length - 16);
      const ct = buf.subarray(12, buf.length - 16);
      const decipher = crypto.createDecipheriv('chacha20-poly1305', key, iv, { authTagLength: 16 });
      decipher.setAuthTag(tag);
      return new Uint8Array(Buffer.concat([decipher.update(ct), decipher.final()]));
    },
  };
}

function startMockOpenAIServer() {
  const calls = [];
  let turnPhase = 0;

  const server = createServer(async (req, res) => {
    const chunks = [];
    for await (const c of req) chunks.push(c);
    const body = JSON.parse(Buffer.concat(chunks).toString('utf8') || '{}');
    calls.push({ url: req.url, auth: req.headers.authorization || '', body });

    if (req.url !== '/v1/chat/completions') {
      res.writeHead(404, { 'content-type': 'application/json' });
      res.end(JSON.stringify({ error: { message: 'Not found' } }));
      return;
    }
    if (req.headers.authorization === 'Bearer bad-secret') {
      res.writeHead(401, { 'content-type': 'application/json' });
      res.end(JSON.stringify({ error: { message: 'Invalid API key' } }));
      return;
    }

    const lastUser = [...(body.messages || [])].reverse().find((m) => m.role === 'user')?.content || '';
    if (!body.stream) {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(JSON.stringify({
        id: 'cmpl-test',
        model: body.model,
        choices: [{ message: { role: 'assistant', content: lastUser.includes('Reply with OK') ? 'OK' : `Non-stream reply: ${lastUser}` } }],
      }));
      return;
    }

    if (lastUser.includes('<tool_response>')) {
      turnPhase = 0;
      res.writeHead(200, { 'content-type': 'text/event-stream' });
      const parts = [
        { choices: [{ delta: { content: 'Listed root ' } }] },
        { choices: [{ delta: { content: 'files cleanly.' } }] },
      ];
      for (const p of parts) {
        res.write(`data: ${JSON.stringify(p)}\n\n`);
      }
      res.write('data: [DONE]\n\n');
      res.end();
      return;
    }

    res.writeHead(200, { 'content-type': 'text/event-stream' });
    if (lastUser.includes('TRIGGER_TOOL')) {
      turnPhase += 1;
      const toolChunk = '<tool_call>{"name":"workspace_list","arguments":{"dir":""}}</tool_call>\ntrailing noise';
      res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: toolChunk } }] })}\n\n`);
      res.write('data: [DONE]\n\n');
      res.end();
      return;
    }
    res.write(`data: ${JSON.stringify({ choices: [{ delta: { content: 'Streamed hello' } }] })}\n\n`);
    res.write('data: [DONE]\n\n');
    res.end();
  });

  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => {
      const addr = server.address();
      resolve({
        origin: `http://127.0.0.1:${addr.port}/v1`,
        calls,
        close: () => new Promise((r) => server.close(() => r())),
      });
    });
  });
}

test('custom-api validators normalize safe URLs and refuse unsafe inputs', () => {
  assert.equal(validateCustomApiBaseUrl('https://api.example.com/v1/'), 'https://api.example.com/v1');
  assert.equal(validateCustomApiBaseUrl('http://127.0.0.1:11434/v1/'), 'http://127.0.0.1:11434/v1');
  assert.equal(resolveChatCompletionsUrl('https://api.example.com/v1'), 'https://api.example.com/v1/chat/completions');
  assert.equal(resolveChatCompletionsUrl('https://api.example.com/v1/chat/completions'), 'https://api.example.com/v1/chat/completions');

  for (const badUrl of ['', 'ftp://host/v1', 'http://remote.example.com/v1', 'https://user:pass@api.example.com/v1', 'https://api.example.com/v1\n', '\thttps://api.example.com/v1']) {
    assert.throws(() => validateCustomApiBaseUrl(badUrl));
  }
  assert.throws(() => validateCustomApiName('   '));
  assert.throws(() => validateCustomApiName('\tBadName'));
  assert.throws(() => validateCustomApiModel('bad\x07model'));
  assert.throws(() => validateCustomApiKey('sk-\r\nbad'));
  assert.throws(() => validateCustomApiKey('sk-\x07bad'));
  assert.throws(() => validateCustomApiKey('sk-lead\t'));
  assert.equal(validateCustomApiName('  My Endpoint  '), 'My Endpoint');
  assert.equal(validateCustomApiModel('  qwen3:8b '), 'qwen3:8b');
});

test('custom-api client seals keys with ChaCha20-Poly1305 and never exposes plaintext secrets', async () => {
  const refs = new Map();
  const vaultKey = crypto.randomBytes(32);
  const client = createCustomApiClient({
    coreReady: Promise.resolve(makeChaChaCore()),
    getRef: async (k) => refs.get(k),
    putRef: async (k, v) => { refs.set(k, v); },
    deleteRef: async (k) => { refs.delete(k); },
    deriveVaultKey: async () => vaultKey,
  });

  const saved = await client.saveEndpoint({
    id: 'ep-local',
    name: 'Local Endpoint',
    baseUrl: 'https://api.example.com/v1',
    model: 'qwen3-custom',
    apiKey: 'sk-super-secret-987',
    contextWindow: 16384,
  });
  assert.equal(saved.endpoint.hasApiKey, true);
  assert.equal('apiKey' in saved.endpoint, false);
  assert.equal('sealedApiKey' in saved.endpoint, false);

  const rawStored = refs.get('vault:custom-api:endpoints');
  assert.ok(Array.isArray(rawStored) && rawStored.length === 1);
  assert.ok(rawStored[0].sealedApiKey instanceof Uint8Array);
  assert.equal(JSON.stringify(rawStored).includes('sk-super-secret-987'), false);

  const pubList = await client.listEndpoints();
  assert.equal(pubList.length, 1);
  assert.equal(pubList[0].hasApiKey, true);
  assert.equal('apiKey' in pubList[0], false);

  // Blank apiKey on edit preserves previously sealed secret.
  await client.saveEndpoint({
    id: 'ep-local',
    name: 'Renamed Endpoint',
    baseUrl: 'https://api.example.com/v1',
    model: 'qwen3-custom',
    apiKey: '',
  });
  const resolved = await client.resolveEndpoint('ep-local');
  assert.equal(resolved.name, 'Renamed Endpoint');
  assert.equal(resolved.apiKey, 'sk-super-secret-987');

  // Explicit clearApiKey removes the key.
  await client.saveEndpoint({
    id: 'ep-local',
    name: 'Renamed Endpoint',
    baseUrl: 'https://api.example.com/v1',
    model: 'qwen3-custom',
    clearApiKey: true,
  });
  const afterClear = await client.resolveEndpoint('ep-local');
  assert.equal(afterClear.hasApiKey, false);
  assert.equal(afterClear.apiKey, '');

  const delRes = await client.deleteEndpoint({ id: 'ep-local' });
  assert.equal(delRes.deleted, true);
  assert.deepEqual(delRes.endpoints, []);
  assert.equal(refs.has('vault:custom-api:endpoints'), false);
});

test('custom-api testEndpoint, streaming completion, stop-sequence cut, and Foundry agent tool loop work end-to-end', async () => {
  const srv = await startMockOpenAIServer();
  try {
    const refs = new Map();
    const vaultKey = crypto.randomBytes(32);
    const client = createCustomApiClient({
      coreReady: Promise.resolve(makeChaChaCore()),
      getRef: async (k) => refs.get(k),
      putRef: async (k, v) => { refs.set(k, v); },
      deleteRef: async (k) => { refs.delete(k); },
      deriveVaultKey: async () => vaultKey,
    });

    await client.saveEndpoint({
      id: 'ep-mock',
      name: 'Loopback Mock',
      baseUrl: srv.origin,
      model: 'mock-qwen3',
      apiKey: 'sk-mock-valid',
    });

    const okTest = await client.testEndpoint({ id: 'ep-mock' });
    assert.equal(okTest.ok, true);
    assert.equal(okTest.status, 200);
    assert.equal(okTest.replyPreview, 'OK');

    const badTest = await client.testEndpoint({ baseUrl: srv.origin, model: 'mock-qwen3', apiKey: 'bad-secret' });
    assert.equal(badTest.ok, false);
    assert.equal(badTest.status, 401);

    const endpoint = await client.resolveEndpoint('ep-mock');
    const seenTokens = [];
    const workspaceListCalls = [];
    const turn = await runUorR4AgentTurn({
      appId: 7,
      chatId: 99,
      modelId: endpoint.model,
      messages: [],
      userPrompt: 'TRIGGER_TOOL list root files',
      chatMode: 'build',
      maxNewTokens: 1024,
      generateFn: (opts) => client.generateChatCompletion({ endpoint, ...opts }),
      tools: {
        workspace_list: async (args) => {
          workspaceListCalls.push(args);
          return { ok: true, files: ['package.json', 'src/App.tsx'] };
        },
      },
      onText: (txt) => seenTokens.push(txt),
    });

    assert.equal(turn.weightsMatched, true);
    assert.equal(turn.toolExecutions, 1);
    assert.deepEqual(workspaceListCalls, [{ dir: '' }]);
    assert.ok(turn.fullText.includes('<tool_call>'));
    assert.ok(!turn.fullText.includes('trailing noise'), 'stop sequence </tool_call> must truncate trailing output');
    assert.ok(turn.fullText.includes('<tool_response>'));
    assert.ok(turn.fullText.includes('Listed root files cleanly.'));
    assert.ok(seenTokens.length >= 2);
    assert.equal(srv.calls.some((c) => c.auth === 'Bearer sk-mock-valid'), true);

    const builtin = buildOpenAiMessages({ systemPrompt: 'SYS', messages: [{ role: 'user', content: 'hi' }], prompt: 'hi' });
    assert.deepEqual(builtin, [
      { role: 'system', content: 'SYS' },
      { role: 'user', content: 'hi' },
    ]);
  } finally {
    await srv.close();
  }
});
