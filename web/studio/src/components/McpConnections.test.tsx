import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Attachment } from '../adapters/workflow.js';
import { makeUi } from './studioTestFixtures.js';
import { McpConnections } from './McpConnections.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); window.localStorage.clear(); window.sessionStorage.clear(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

function ui() {
    const attachments: Attachment[] = [];
    const api = makeUi();
    api.getAttachments = vi.fn(() => attachments);
    api.addAttachment = vi.fn((entry: Attachment) => { attachments.push(entry); });
    return { api, attachments };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function input(label: string, value: string) {
    const field = host.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
    act(() => { field.value = value; field.dispatchEvent(new Event('input', { bubbles: true })); });
}

function args(value: string) {
    const field = host.querySelector<HTMLTextAreaElement>('textarea[aria-label="Tool arguments as JSON"]')!;
    act(() => { field.value = value; field.dispatchEvent(new Event('input', { bubbles: true })); });
}

describe('McpConnections', () => {
    it('connects, lists, calls, and attaches exact results with inert text and memory-only credentials', async () => {
        const result = { content: [{ type: 'text', text: '<img src=x onerror=window.__mcpMarkupExecuted=true>' }] };
        const session = {
            listTools: vi.fn(async () => [{ name: 'read_file', description: '<img src=x onerror=window.__mcpMarkupExecuted=true>' }]),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const connectRemote = vi.fn(async () => session);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        expect(connectRemote).not.toHaveBeenCalled();
        expect(session.listTools).not.toHaveBeenCalled();
        expect(button('Attach result').disabled).toBe(true);
        input('MCP server URL', 'http://127.0.0.1:9000/mcp');
        input('MCP credential', 'memory-only');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        expect(connectRemote).toHaveBeenCalledWith({ url: 'http://127.0.0.1:9000/mcp', credential: 'memory-only' });
        expect(session.listTools).toHaveBeenCalledOnce();
        expect((document.getElementById('mcpCredentialInput') as HTMLInputElement).value).toBe('');
        expect(document.querySelector('.mcp-tool-button img')).toBeNull();
        args('{"path":"notes.md","depth":2,"options":{"recursive":true,"labels":["alpha","beta"]}}');
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('<img'));
        expect(session.callTool).toHaveBeenCalledWith('read_file', { path: 'notes.md', depth: 2, options: { recursive: true, labels: ['alpha', 'beta'] } });
        expect(document.querySelector('#mcpResultOutput img')).toBeNull();
        expect((window as unknown as { __mcpMarkupExecuted?: boolean }).__mcpMarkupExecuted).toBeUndefined();
        await act(async () => { button('Attach result').click(); await Promise.resolve(); });
        expect(api.addAttachment).toHaveBeenCalledWith({ name: 'mcp-read_file.json', kind: 'text', content: JSON.stringify(result, null, 2), source: 'mcp' });
        expect(window.localStorage.getItem('uor.studio.mcp.credential')).toBeNull();
    });

    it('refuses invalid JSON and non-object arguments without calling or attaching', async () => {
        const session = { listTools: vi.fn(async () => [{ name: 'read_file', description: 'Read' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://127.0.0.1:9000/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        args('[]');
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        expect(document.getElementById('mcpResultOutput')?.textContent).toContain('Tool arguments must be a JSON object.');
        args('{bad');
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        expect(document.getElementById('mcpResultOutput')?.textContent).toContain('JSON');
        expect(session.callTool).not.toHaveBeenCalled();
        expect(api.addAttachment).not.toHaveBeenCalled();
        expect(button('Attach result').disabled).toBe(true);
    });

    it('disconnects the previous session after a successful replacement', async () => {
        const first = { listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const second = { listTools: vi.fn(async () => [{ name: 'second', description: 'Second' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => second);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        input('MCP server URL', 'http://second.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('second'));
        expect(first.disconnect).toHaveBeenCalledOnce();
    });

    it('preserves the working session and tools when a replacement fails', async () => {
        const first = { listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => { throw new Error('replacement failed'); });
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        input('MCP server URL', 'http://bad.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('replacement failed'));
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first');
        expect(first.disconnect).not.toHaveBeenCalled();
    });

    it('disconnects a stale candidate on unmount without listing or attaching it', async () => {
        let resolveCandidate!: () => void;
        const candidate = { listTools: vi.fn(async () => [{ name: 'late', description: 'Late' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn(() => new Promise<typeof candidate>((resolve) => { resolveCandidate = () => resolve(candidate); }));
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://late.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        act(() => { render(null, host); });
        await act(async () => { resolveCandidate(); await Promise.resolve(); });
        expect(candidate.disconnect).toHaveBeenCalledOnce();
        expect(candidate.listTools).not.toHaveBeenCalled();
        expect(api.addAttachment).not.toHaveBeenCalled();
    });

    it('reports connection cancellation without listing tools', async () => {
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => null)} />, host); });
        input('MCP server URL', 'http://cancel.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('cancelled'));
        expect(document.querySelector('.mcp-tool-button')).toBeNull();
        expect(button('Attach result').disabled).toBe(true);
    });

    it('disables Connect during a tool call and restores it afterward', async () => {
        let resolveCall!: () => void;
        const result = { content: [{ type: 'text', text: 'done' }] };
        const session = {
            listTools: vi.fn(async () => [{ name: 'slow_tool', description: 'Slow' }]),
            callTool: vi.fn(() => new Promise<typeof result>((resolve) => { resolveCall = () => resolve(result); })),
            disconnect: vi.fn(async () => undefined),
        };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://slow.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        expect(button('Connect').disabled).toBe(true);
        await act(async () => { resolveCall(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('done'));
        expect(button('Connect').disabled).toBe(false);
    });

    it('reports attachment refusal instead of claiming success', async () => {
        const result = { content: [{ type: 'text', text: 'x'.repeat(1_000_000) }] };
        const session = {
            listTools: vi.fn(async () => [{ name: 'large_tool', description: 'Large' }]),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const { api, attachments } = ui();
        attachments.push({ name: 'existing.txt', content: 'retained', kind: 'text', source: 'file', size: 3_500_000 });
        api.addAttachment = vi.fn();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://large.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('xxx'));
        await act(async () => { button('Attach result').click(); await Promise.resolve(); });
        expect(api.toast).toHaveBeenCalledWith(expect.stringContaining('could not be attached'), 'error');
        expect(button('Attach result').textContent).toBe('Attach result');
        expect(attachments.some((entry) => entry.name === 'mcp-large_tool.json')).toBe(false);
    });

    it('preserves the previous session when its cleanup fails and releases the candidate', async () => {
        const first = {
            listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]),
            callTool: vi.fn(),
            disconnect: vi.fn(async () => { throw new Error('cleanup failed'); }),
        };
        const second = { listTools: vi.fn(async () => [{ name: 'second', description: 'Second' }]), callTool: vi.fn(), disconnect: vi.fn(async () => undefined) };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => second);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        input('MCP server URL', 'http://second.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('cleanup failed'));
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first');
        expect(second.disconnect).toHaveBeenCalledOnce();
    });

    it('uses the actual remote factory through loopback HTTP JSON-RPC', async () => {
        const result = { content: [{ type: 'text', text: 'remote result' }] };
        const methods: string[] = [];
        const { createServer } = await vi.importActual<{
            createServer: (
                handler: (
                    req: AsyncIterable<{ toString(): string }>,
                    res: {
                        setHeader(name: string, value: string): void;
                        writeHead(status: number): void;
                        end(body?: string): void;
                    },
                ) => void) => {
                    listen(port: number, hostname: string): void;
                    once(event: 'listening', listener: () => void): void;
                    address(): { port: number } | string | null;
                    closeAllConnections(): void;
                    close(callback: () => void): void;
                };
        }>('node:http');
        const server = createServer((req, res) => {
            void (async () => {
                let raw = '';
                for await (const chunk of req) raw += chunk;
                const message = JSON.parse(raw || '{}') as { method: string };
                methods.push(message.method);
                res.setHeader('Access-Control-Allow-Origin', '*');
                res.setHeader('Content-Type', 'application/json');
                res.setHeader('Mcp-Session-Id', 'session-1');
                if (message.method === 'notifications/initialized') {
                    res.writeHead(202);
                    res.end();
                    return;
                }
                const payload = message.method === 'initialize' ? { result: {} }
                    : message.method === 'tools/list' ? { result: { tools: [{ name: 'remote_tool', description: 'Remote' }] } }
                    : message.method === 'tools/call' ? { result } : { error: { message: 'unexpected method' } };
                res.end(JSON.stringify(payload));
            })().catch((error) => {
                res.writeHead(500);
                res.end(String(error));
            });
        });
        server.listen(0, '127.0.0.1');
        await new Promise<void>((resolve) => server.once('listening', resolve));
        const address = server.address();
        if (!address || typeof address === 'string') throw new Error('Missing fixture address');
        const { api } = ui();
        try {
            act(() => { render(<McpConnections ui={api} />, host); });
            input('MCP server URL', `http://127.0.0.1:${address.port}/mcp`);
            await act(async () => { button('Connect').click(); await Promise.resolve(); });
            await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
            await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
            await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('remote result'));
            await act(async () => { button('Attach result').click(); await Promise.resolve(); });
            expect(methods).toEqual(['initialize', 'notifications/initialized', 'tools/list', 'tools/call']);
            expect(api.addAttachment).toHaveBeenCalledWith({ name: 'mcp-remote_tool.json', kind: 'text', content: JSON.stringify(result, null, 2), source: 'mcp' });
        } finally {
            server.closeAllConnections();
            await new Promise<void>((resolve) => server.close(() => resolve()));
        }
    });

    it('sanitizes hostile tool names before attaching', async () => {
        const result = { content: [{ type: 'text', text: 'safe' }] };
        const session = {
            listTools: vi.fn(async () => [{ name: '../../etc/passwd', description: 'Hostile' }]),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://names.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('safe'));
        await act(async () => { button('Attach result').click(); await Promise.resolve(); });
        expect(api.addAttachment).toHaveBeenCalledWith({ name: 'mcp-etc-passwd.json', kind: 'text', content: JSON.stringify(result, null, 2), source: 'mcp' });
    });

    it('does not expose credentials through storage, cookies, or console on unmount', () => {
        const consoleLog = vi.spyOn(console, 'log').mockImplementation(() => {});
        const consoleInfo = vi.spyOn(console, 'info').mockImplementation(() => {});
        const consoleWarn = vi.spyOn(console, 'warn').mockImplementation(() => {});
        const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn()} />, host); });
        input('MCP credential', 'secret-sentinel');
        act(() => { render(null, host); });
        expect(window.localStorage.getItem('uor.studio.mcp.credential')).toBeNull();
        expect(window.sessionStorage.getItem('uor.studio.mcp.credential')).toBeNull();
        expect(document.cookie).not.toContain('secret-sentinel');
        for (const spy of [consoleLog, consoleInfo, consoleWarn, consoleError]) {
            expect(spy.mock.calls.some((call) => call.some((value) => String(value).includes('secret-sentinel')))).toBe(false);
        }
    });

    it('treats a missing disconnect hook as a no-op during successful replacement', async () => {
        const first = { listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]), callTool: vi.fn() };
        const second = { listTools: vi.fn(async () => [{ name: 'second', description: 'Second' }]), callTool: vi.fn() };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => second);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first'));
        input('MCP server URL', 'http://second.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('second'));
        expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('Connected');
    });

    it('treats a missing disconnect hook as a no-op on unmount', async () => {
        const session = { listTools: vi.fn(async () => [{ name: 'browser_tool', description: 'Browser' }]), callTool: vi.fn() };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://browser.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        await act(async () => { render(null, host); await Promise.resolve(); });
        expect(document.querySelector('.mcp-connections')).toBeNull();
    });

    it('disconnects an active session on unmount', async () => {
        const session = {
            listTools: vi.fn(async () => [{ name: 'active_tool', description: 'Active' }]),
            callTool: vi.fn(),
            disconnect: vi.fn(async () => undefined),
        };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://active.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        await act(async () => { render(null, host); await Promise.resolve(); });
        expect(session.disconnect).toHaveBeenCalledOnce();
    });

    it('refuses Call while Connect is active without latching either busy state', async () => {
        const result = { content: [{ type: 'text', text: 'unexpected' }] };
        let resolveReplacement!: () => void;
        const first = {
            listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const second = {
            listTools: vi.fn(async () => [{ name: 'second', description: 'Second' }]),
            callTool: vi.fn(),
            disconnect: vi.fn(async () => undefined),
        };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(() => new Promise<typeof second>((resolve) => { resolveReplacement = () => resolve(second); }));
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first'));
        input('MCP server URL', 'http://second.invalid/mcp');
        await act(async () => {
            button('Connect').click();
            (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click();
            await Promise.resolve();
        });
        await act(async () => { resolveReplacement(); await Promise.resolve(); });
        expect(first.callTool).not.toHaveBeenCalled();
        await vi.waitFor(() => expect(document.getElementById('mcpConnectButton')?.hasAttribute('disabled')).toBe(false));
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('second');
    });

    it('surfaces candidate cleanup failure while preserving the previous session', async () => {
        const first = {
            listTools: vi.fn(async () => [{ name: 'first', description: 'First' }]),
            callTool: vi.fn(),
            disconnect: vi.fn(async () => { throw new Error('old-cleanup-failed'); }),
        };
        const second = {
            listTools: vi.fn(async () => [{ name: 'second', description: 'Second' }]),
            callTool: vi.fn(),
            disconnect: vi.fn(async () => { throw new Error('candidate-cleanup-failed'); }),
        };
        const connectRemote = vi.fn()
            .mockImplementationOnce(async () => first)
            .mockImplementationOnce(async () => second);
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={connectRemote} />, host); });
        input('MCP server URL', 'http://first.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first'));
        input('MCP server URL', 'http://second.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('old-cleanup-failed'));
        expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('candidate-cleanup-failed');
        expect(second.disconnect).toHaveBeenCalledOnce();
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('first');
    });

    it('filters null and non-string tool metadata before Call and Attach', async () => {
        const result = { content: [{ type: 'text', text: 'safe' }] };
        const session = {
            listTools: vi.fn(async () => [
                null,
                { name: 42, description: 'Non-string name' },
                { name: 'invalid_description', description: null },
                { name: 'safe_tool', description: 'Safe' },
            ] as unknown as Array<{ name: string; description?: string }>),
            callTool: vi.fn(async () => result),
            disconnect: vi.fn(async () => undefined),
        };
        const { api } = ui();
        act(() => { render(<McpConnections ui={api} connectRemote={vi.fn(async () => session)} />, host); });
        input('MCP server URL', 'http://metadata.invalid/mcp');
        await act(async () => { button('Connect').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('safe_tool');
        expect(document.querySelectorAll('.mcp-tool-button')).toHaveLength(1);
        await act(async () => { (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('safe'));
        expect(session.callTool).toHaveBeenCalledWith('safe_tool', {});
        await act(async () => { button('Attach result').click(); await Promise.resolve(); });
        expect(api.addAttachment).toHaveBeenCalledWith({ name: 'mcp-safe_tool.json', kind: 'text', content: JSON.stringify(result, null, 2), source: 'mcp' });
    });
});
