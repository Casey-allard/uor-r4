const PROTOCOL_VERSION = '2025-03-26';
const DEFAULT_MAX_RESPONSE_BYTES = 1024 * 1024;
const DEFAULT_TIMEOUT_MS = 10000;
const DEFAULT_MAX_PAGES = 4;
const MAX_TOOLS = 256;
const LOOPBACK = new Set(['127.0.0.1', 'localhost', '[::1]']);

function mcpError(message) {
    return new Error(message);
}

function insecureLoopbackMessage() {
    return 'HTTPS Pages cannot call insecure HTTP loopback. Open the runner-served local Studio at http://127.0.0.1:8000/ instead; CORS alone does not make this safe.';
}

async function readBounded(response, maxBytes) {
    if (!response.body) return '';
    const reader = response.body.getReader();
    const chunks = [];
    let total = 0;
    while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        total += value.byteLength;
        if (total > maxBytes) {
            await reader.cancel();
            throw mcpError(`MCP response exceeds the ${maxBytes}-byte size cap.`);
        }
        chunks.push(value);
    }
    const merged = new Uint8Array(total);
    let offset = 0;
    for (const chunk of chunks) { merged.set(chunk, offset); offset += chunk.byteLength; }
    return new TextDecoder().decode(merged);
}

function parsePayload(text, contentType) {
    if (!text.trim()) return null;
    if (String(contentType).includes('text/event-stream')) {
        let latest = null;
        for (const block of text.split(/\r?\n\r?\n/)) {
            const data = block.split(/\r?\n/).filter((line) => line.startsWith('data:')).map((line) => line.slice(5).trim()).join('\n');
            if (!data || data === '[DONE]') continue;
            latest = JSON.parse(data);
        }
        return latest;
    }
    if (!String(contentType).includes('application/json') && !String(contentType).includes('+json')) {
        throw mcpError('MCP server returned an unsupported content type.');
    }
    return JSON.parse(text);
}

export function createMcpClient(options = {}) {
    const fetchImpl = options.fetchImpl || globalThis.fetch;
    const location = options.location || globalThis.location;
    const maxResponseBytes = Number(options.maxResponseBytes) > 0 ? Number(options.maxResponseBytes) : DEFAULT_MAX_RESPONSE_BYTES;
    const timeoutMs = Number(options.timeoutMs) > 0 ? Number(options.timeoutMs) : DEFAULT_TIMEOUT_MS;
    const maxPages = Number(options.maxPages) > 0 ? Number(options.maxPages) : DEFAULT_MAX_PAGES;
    let url = null;
    let credential = '';
    let sessionId = '';
    let connected = false;
    let serial = 0;

    function guardTransport(parsed) {
        if (location?.protocol === 'https:' && parsed.protocol === 'http:' && LOOPBACK.has(parsed.hostname)) {
            throw mcpError(insecureLoopbackMessage());
        }
    }

    async function request(method, params, notification = false) {
        if (typeof fetchImpl !== 'function') throw mcpError('MCP transport is unavailable.');
        guardTransport(url);
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), timeoutMs);
        const headers = {
            'Content-Type': 'application/json',
            Accept: 'application/json, text/event-stream',
            'MCP-Protocol-Version': PROTOCOL_VERSION,
        };
        if (credential) headers.Authorization = 'Bearer ' + credential;
        if (sessionId) headers['Mcp-Session-Id'] = sessionId;
        const message = { jsonrpc: '2.0', method, params };
        if (!notification) message.id = ++serial;
        try {
            const response = await fetchImpl(url.toString(), {
                method: 'POST',
                headers,
                body: JSON.stringify(message),
                signal: controller.signal,
                cache: 'no-store',
                credentials: 'omit',
            });
            const text = await readBounded(response, maxResponseBytes);
            if (!response.ok) {
                const detail = text.slice(0, 400);
                if (response.status === 401 || response.status === 403) throw mcpError(`MCP authentication failed with HTTP ${response.status}: ${detail}`);
                throw mcpError(`MCP request failed with HTTP ${response.status}: ${detail}`);
            }
            if (notification && !text.trim()) return null;
            const payload = parsePayload(text, response.headers.get('content-type') || '');
            if (payload?.error) throw mcpError('MCP server error: ' + String(payload.error.message || payload.error));
            return { payload: payload?.result ?? payload, headers: response.headers };
        } catch (error) {
            if (error?.name === 'AbortError') throw mcpError('MCP request timed out.');
            if (error instanceof TypeError) throw mcpError('MCP endpoint is unreachable or blocked by CORS. Check the server URL, CORS policy, and authentication: ' + error.message);
            throw error;
        } finally {
            clearTimeout(timer);
        }
    }

    return {
        async connectRemote(config = {}) {
            const parsed = new URL(String(config.url || ''));
            if (!['http:', 'https:'].includes(parsed.protocol)) throw mcpError('MCP requires an HTTP or HTTPS URL.');
            if (parsed.username || parsed.password) throw mcpError('Put MCP credentials in the memory-only credential field, not the URL.');
            guardTransport(parsed);
            url = parsed;
            credential = String(config.credential || '');
            sessionId = '';
            connected = false;
            serial = 0;
            const initialized = await request('initialize', {
                protocolVersion: PROTOCOL_VERSION,
                capabilities: {},
                clientInfo: { name: 'uor-studio', version: '0.1.0' },
            });
            if (!initialized.payload || typeof initialized.payload !== 'object') throw mcpError('MCP initialize returned no result.');
            sessionId = initialized.headers.get('mcp-session-id') || '';
            await request('notifications/initialized', {}, true);
            connected = true;
            return this;
        },
        async listTools() {
            if (!connected) throw mcpError('MCP is not connected.');
            const tools = [];
            let cursor;
            for (let page = 1; page <= maxPages; page += 1) {
                const response = await request('tools/list', cursor ? { cursor } : {});
                const result = response.payload || {};
                if (Array.isArray(result.tools)) tools.push(...result.tools);
                if (tools.length > MAX_TOOLS) throw mcpError(`MCP tool list exceeds the ${MAX_TOOLS}-tool cap.`);
                if (!result.nextCursor) return tools;
                cursor = String(result.nextCursor);
                if (page === maxPages) throw mcpError(`MCP pagination cap reached (${maxPages} pages).`);
            }
            return tools;
        },
        async callTool(name, args = {}) {
            if (!connected) throw mcpError('MCP is not connected.');
            const serialized = JSON.stringify(args ?? {});
            if (new TextEncoder().encode(serialized).length > 64 * 1024) throw mcpError('MCP tool arguments exceed 64 KiB.');
            const response = await request('tools/call', { name: String(name), arguments: args ?? {} });
            return response.payload;
        },
        async connectStdio() {
            throw mcpError('MCP stdio is unavailable in the browser build; use the desktop app.');
        },
    };
}

function declaredNativeInvoke() {
    const invoke = globalThis.__TAURI__?.core?.invoke;
    return typeof invoke === 'function' && typeof globalThis.__TAURI__?.core?.Channel === 'function'
        ? invoke.bind(globalThis.__TAURI__.core)
        : null;
}

function nativeSession(sessionId, invoke) {
    return {
        listTools: () => invoke('native_mcp_list_tools', { sessionId }),
        callTool: (name, args = {}) => invoke('native_mcp_call_tool', { sessionId, name, args }),
        disconnect: () => invoke('native_mcp_disconnect', { sessionId }),
    };
}

export async function connectNativeStdio(config = {}) {
    const invoke = declaredNativeInvoke();
    if (!invoke) throw mcpError('MCP stdio is unavailable in the browser build; use the desktop app.');
    const selected = await invoke('native_mcp_connect_stdio', {
        executable: String(config.executable || ''),
        args: Array.isArray(config.args) ? config.args.map(String) : [],
        cwd: String(config.cwd || ''),
    });
    if (!selected) return null;
    return nativeSession(selected.sessionId, invoke);
}

export async function connectRemote(config = {}) {
    const invoke = declaredNativeInvoke();
    if (invoke) {
        const selected = await invoke('native_mcp_connect_remote', {
            url: String(config.url || ''),
            credential: String(config.credential || ''),
        });
        if (!selected) return null;
        return nativeSession(selected.sessionId, invoke);
    }
    const client = createMcpClient(config);
    return client.connectRemote(config);
}

function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
}

export function mountMcpConnections({ root = globalThis.document, connectRemote: connect = connectRemote } = {}) {
    if (!root) return;
    root.getElementById('mcpConnections')?.remove();
    const mount = root.getElementById('mcpMount') || root.getElementById('modelSourceMount');
    if (!mount) return;
    let session = null;
    const section = element('section', 'source-group mcp-connections');
    section.id = 'mcpConnections';
    section.appendChild(element('h3', '', 'MCP servers'));
    section.appendChild(element('p', 'hint', 'Remote Streamable HTTP. Credentials stay in memory and disappear on reload.'));
    const url = element('input', 'input');
    url.id = 'mcpUrlInput';
    url.type = 'url';
    url.placeholder = 'http://127.0.0.1:9000/mcp';
    url.setAttribute('aria-label', 'MCP server URL');
    const credential = element('input', 'input');
    credential.id = 'mcpCredentialInput';
    credential.type = 'password';
    credential.placeholder = 'Optional credential (memory only)';
    credential.autocomplete = 'off';
    credential.setAttribute('aria-label', 'MCP credential');
    const connectButton = element('button', 'btn btn-secondary btn-small', 'Connect');
    connectButton.id = 'mcpConnectButton';
    connectButton.type = 'button';
    const status = element('p', 'hint', 'Not connected. No request is made until Connect.');
    status.id = 'mcpConnectionStatus';
    const tools = element('div', 'mcp-tool-list');
    tools.id = 'mcpToolsList';
    const args = element('textarea', 'input mcp-tool-args');
    args.id = 'mcpToolArgs';
    args.value = '{}';
    args.rows = 3;
    args.setAttribute('aria-label', 'Tool arguments as JSON');
    const result = element('pre', 'mcp-result', 'No tool result.');
    result.id = 'mcpResultOutput';
    const native = declaredNativeInvoke();
    const stdio = element('button', 'btn btn-ghost btn-small', native ? 'Connect stdio with native approval' : 'Stdio unavailable in browser build');
    stdio.id = 'mcpStdioButton';
    stdio.type = 'button';
    stdio.disabled = !native;
    section.append(url, credential, connectButton, status, tools, args, result);
    let executable;
    let stdioArgs;
    let cwd;
    if (native) {
        executable = element('input', 'input');
        executable.id = 'mcpStdioExecutable';
        executable.placeholder = '/absolute/path/to/mcp-server';
        executable.setAttribute('aria-label', 'MCP stdio executable');
        stdioArgs = element('input', 'input');
        stdioArgs.id = 'mcpStdioArgs';
        stdioArgs.placeholder = 'Optional JSON argument array, for example ["--safe"]';
        stdioArgs.setAttribute('aria-label', 'MCP stdio arguments');
        cwd = element('input', 'input');
        cwd.id = 'mcpStdioCwd';
        cwd.placeholder = '/granted/project';
        cwd.setAttribute('aria-label', 'MCP stdio working directory');
        section.append(executable, stdioArgs, cwd);
    }
    section.appendChild(stdio);
    mount.appendChild(section);

    function renderTools(listed) {
        tools.textContent = '';
        for (const tool of listed) {
            const button = element('button', 'btn btn-secondary btn-small mcp-tool-button', `${tool.name} — ${tool.description || 'No description'}`);
            button.type = 'button';
            button.addEventListener('click', async () => {
                try {
                    const parsed = JSON.parse(args.value || '{}');
                    if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') throw mcpError('Tool arguments must be a JSON object.');
                    result.textContent = 'Calling ' + tool.name + '…';
                    const called = await session.callTool(tool.name, parsed);
                    result.textContent = JSON.stringify(called, null, 2);
                } catch (error) {
                    result.textContent = String(error?.message || error);
                }
            });
            tools.appendChild(button);
        }
    }

    let connectionToken = 0;
    let connecting = false;

    function setConnecting(value) {
        connecting = value;
        connectButton.disabled = value;
        stdio.disabled = value || !native;
    }

    async function release(candidate) {
        if (!candidate?.disconnect) return '';
        try {
            await candidate.disconnect();
            return '';
        } catch (error) {
            const message = String(error?.message || error);
            return message === 'MCP session is already disconnected.' ? '' : message;
        }
    }

    async function connectWith(action) {
        if (connecting) return;
        setConnecting(true);
        const token = ++connectionToken;
        const previous = session;
        status.textContent = 'Connecting…';
        let candidate = null;
        try {
            candidate = await action();
            if (token !== connectionToken) { await release(candidate); return; }
            if (!candidate) {
                status.textContent = 'Connection approval was cancelled; nothing started.' + (previous ? ' Previous connection is still active.' : '');
                return;
            }
            const listed = await candidate.listTools();
            if (token !== connectionToken) { await release(candidate); return; }
            let previousCleanup = '';
            if (previous && previous !== candidate) previousCleanup = await release(previous);
            if (token !== connectionToken) { await release(candidate); return; }
            session = candidate;
            status.textContent = `Connected · ${listed.length} tool${listed.length === 1 ? '' : 's'}` + (previousCleanup ? ' Previous session cleanup failed: ' + previousCleanup : '');
            renderTools(listed);
        } catch (error) {
            const cleanup = await release(candidate);
            if (token === connectionToken) {
                session = previous;
                status.textContent = String(error?.message || error) + (cleanup ? ' Replacement cleanup failed: ' + cleanup : '');
                if (!previous) tools.textContent = '';
            }
        } finally {
            if (token === connectionToken) setConnecting(false);
        }
    }

    connectButton.addEventListener('click', () => connectWith(() => connect({ url: url.value.trim(), credential: credential.value })));
    if (native) {
        stdio.addEventListener('click', () => connectWith(() => {
            const parsed = stdioArgs.value.trim() ? JSON.parse(stdioArgs.value) : [];
            if (!Array.isArray(parsed)) throw mcpError('MCP stdio arguments must be a JSON array.');
            return connectNativeStdio({ executable: executable.value.trim(), args: parsed, cwd: cwd.value.trim() });
        }));
    }
    globalThis.addEventListener?.('pagehide', () => {
        connectionToken += 1;
        if (session?.disconnect) session.disconnect().catch(() => {});
    });
    return section;
}

function bootMcp() {
    if (new URLSearchParams(globalThis.location?.search || '').get('typed') === '1') return;
    if (globalThis.document?.getElementById('modelSourceMount')) mountMcpConnections();
}

if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', bootMcp, { once: true });
    else bootMcp();
}
