import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import { connectRemote, createMcpClient } from './mcp_client.js';

const PROTOCOL = '2025-03-26';

async function fixture(options = {}) {
    const requests = [];
    const server = http.createServer(async (req, res) => {
        const origin = req.headers.origin || 'http://127.0.0.1:8765';
        res.setHeader('Access-Control-Allow-Origin', origin);
        res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Accept, Authorization, Mcp-Session-Id, MCP-Protocol-Version');
        res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
        if (req.method === 'OPTIONS') { res.writeHead(204); res.end(); return; }
        const chunks = [];
        for await (const chunk of req) chunks.push(chunk);
        const raw = Buffer.concat(chunks).toString('utf8');
        const message = raw ? JSON.parse(raw) : null;
        requests.push({ method: req.method, headers: req.headers, message });
        if (options.delay) await new Promise((resolve) => setTimeout(resolve, options.delay));
        if (options.auth !== false && req.headers.authorization !== 'Bearer secret') {
            res.writeHead(401, { 'content-type': 'application/json' });
            res.end(JSON.stringify({ error: { message: 'bad credential' } }));
            return;
        }
        if (message?.method === 'notifications/initialized') { res.writeHead(202); res.end(); return; }
        const reply = (result, status = 200) => {
            const payload = { jsonrpc: '2.0', id: message.id, result };
            res.setHeader('Mcp-Session-Id', 'session-test');
            if (options.sse) {
                res.writeHead(status, { 'content-type': 'text/event-stream' });
                res.end(`data: ${JSON.stringify(payload)}\r\n\r\n`);
            } else {
                res.writeHead(status, { 'content-type': 'application/json' });
                res.end(JSON.stringify(payload));
            }
        };
        if (message?.method === 'initialize') {
            reply({ protocolVersion: PROTOCOL, capabilities: { tools: {} }, serverInfo: { name: 'fixture', version: '1' } });
        } else if (message?.method === 'tools/list') {
            if (options.large) {
                res.writeHead(200, { 'content-type': 'application/json' });
                res.end(JSON.stringify({ jsonrpc: '2.0', id: message.id, result: { tools: [{ name: 'x'.repeat(4096) }] } }));
            } else {
                reply({ tools: [{ name: 'read_file', description: '<img src=x onerror=alert(1)>', inputSchema: { type: 'object' } }], nextCursor: options.paginate ? 'next' : undefined });
            }
        } else if (message?.method === 'tools/call') {
            reply({ content: [{ type: 'text', text: '<img src=x onerror=alert(1)>' }], isError: false });
        } else {
            res.writeHead(404); res.end();
        }
    });
    server.listen(0, '127.0.0.1');
    await new Promise((resolve) => server.once('listening', resolve));
    return {
        url: `http://127.0.0.1:${server.address().port}/mcp`,
        requests,
        close: () => { server.closeAllConnections(); return new Promise((resolve) => server.close(resolve)); },
    };
}

function config(server, extra = {}) {
    return { url: server.url, credential: 'secret', ...extra };
}

test('JSON MCP performs initialize/initialized, session headers, list and call without persisting credentials', async () => {
    const server = await fixture();
    const writes = [];
    globalThis.localStorage = { setItem: (...args) => writes.push(args) };
    const session = await connectRemote(config(server));
    const tools = await session.listTools();
    const result = await session.callTool('read_file', { path: 'a.rs' });
    assert.equal(tools[0].name, 'read_file');
    assert.match(result.content[0].text, /<img/);
    assert.deepEqual(server.requests.map((request) => request.message?.method), ['initialize', 'notifications/initialized', 'tools/list', 'tools/call']);
    assert.equal(server.requests[0].headers['mcp-protocol-version'], PROTOCOL);
    assert.equal(server.requests[2].headers['mcp-session-id'], 'session-test');
    assert.equal(server.requests[3].headers['mcp-session-id'], 'session-test');
    assert.deepEqual(writes, []);
    assert.rejects(createMcpClient().listTools(), /not connected/);
    const preflight = await fetch(server.url, { method: 'OPTIONS', headers: { Origin: 'http://127.0.0.1:8765' } });
    assert.equal(preflight.status, 204);
    assert.equal(preflight.headers.get('access-control-allow-origin'), 'http://127.0.0.1:8765');
    await server.close();
});

test('SSE MCP responses normalize to the same tools and result shapes', async () => {
    const server = await fixture({ sse: true });
    const session = await connectRemote(config(server));
    assert.equal((await session.listTools()).length, 1);
    assert.match((await session.callTool('read_file', {})).content[0].text, /<img/);
    await server.close();
});

test('authentication, timeout and response-size failures are explicit', async () => {
    const unauthorized = await fixture();
    await assert.rejects(connectRemote(config(unauthorized, { credential: 'wrong' })), /401|authentication|bad credential/i);
    await unauthorized.close();
    const slow = await fixture({ delay: 80 });
    await assert.rejects(connectRemote(config(slow, { timeoutMs: 15 })), /timed out/i);
    await slow.close();
    const large = await fixture({ large: true });
    const session = await connectRemote(config(large, { maxResponseBytes: 1024 }));
    await assert.rejects(session.listTools(), /exceeds|size/i);
    await large.close();
});

test('pagination stops at its declared cap instead of following forever', async () => {
    const server = await fixture({ paginate: true });
    const session = await connectRemote(config(server, { maxPages: 2 }));
    await assert.rejects(session.listTools(), /pagination cap/i);
    assert.equal(server.requests.filter((request) => request.message?.method === 'tools/list').length, 2);
    await server.close();
});

test('declared Tauri MCP transport handles remote calls without direct browser fetch', async () => {
    const calls = [];
    globalThis.__TAURI__ = { core: { Channel: class {}, invoke: async (command, args) => {
        calls.push({ command, args });
        if (command === 'native_mcp_connect_remote') return { sessionId: 'native-1' };
        if (command === 'native_mcp_list_tools') return [{ name: 'native_tool', description: 'host' }];
        if (command === 'native_mcp_call_tool') return { content: [{ type: 'text', text: 'host result' }] };
        if (command === 'native_mcp_disconnect') return { disconnected: true };
        throw new Error('unexpected ' + command);
    } } };
    const session = await connectRemote({ url: 'https://example.invalid/mcp', credential: 'memory-only', fetchImpl: async () => { throw new Error('direct fetch must not run'); } });
    assert.deepEqual(await session.listTools(), [{ name: 'native_tool', description: 'host' }]);
    assert.equal((await session.callTool('native_tool', {})).content[0].text, 'host result');
    await session.disconnect();
    assert.deepEqual(calls.map((call) => call.command), ['native_mcp_connect_remote', 'native_mcp_list_tools', 'native_mcp_call_tool', 'native_mcp_disconnect']);
    assert.equal(calls[0].args.credential, 'memory-only');
    assert.equal('credential' in calls[1].args, false);
    delete globalThis.__TAURI__;
});

test('cancelling native remote approval returns null without creating a session', async () => {
    globalThis.__TAURI__ = { core: { Channel: class {}, invoke: async () => null } };
    assert.equal(await connectRemote({ url: 'https://example.invalid/mcp', credential: 'secret' }), null);
    delete globalThis.__TAURI__;
});

test('HTTPS Pages get native-setup guidance for insecure loopback before any request', async () => {
    let calls = 0;
    const fetchImpl = async () => { calls += 1; throw new TypeError('Failed to fetch'); };
    await assert.rejects(connectRemote({
        url: 'http://127.0.0.1:8000/mcp',
        credential: 'secret',
        fetchImpl,
        location: new URL('https://uor-foundation.github.io/uor-r4/studio/'),
    }), /HTTPS Pages cannot call insecure HTTP loopback.*native-runner\.sh|HTTPS Pages cannot call insecure HTTP loopback.*runner-served local Studio/);
    assert.equal(calls, 0);
});


test('non-ASCII tool arguments are limited by UTF-8 bytes before any call request', async () => {
    const server = await fixture();
    const session = await connectRemote(config(server));
    await assert.rejects(session.callTool('read_file', { text: '€'.repeat(22000) }), /64 KiB/);
    assert.equal(server.requests.filter((request) => request.message?.method === 'tools/call').length, 0);
    await server.close();
});
