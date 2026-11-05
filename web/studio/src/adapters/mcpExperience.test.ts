import { afterEach, describe, expect, it, vi } from 'vitest';
// @ts-expect-error The plain-JavaScript MCP controller has no declaration file.
import { mountMcpConnections } from '../../assets/js/mcp_client.js';

afterEach(() => {
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __mcpMarkupExecuted?: boolean }).__mcpMarkupExecuted;
});

describe('MCP Connections', () => {
    it('connects and calls only on explicit actions, rendering all tool text as inert text', async () => {
        document.body.innerHTML = '<div id="modelSourceMount"></div>';
        const connections: Array<{ url: string; credential: string }> = [];
        const session = {
            listTools: vi.fn(async () => [{ name: 'read_file', description: '<img src=x onerror=window.__mcpMarkupExecuted=true>' }]),
            callTool: vi.fn(async () => ({ content: [{ type: 'text', text: '<img src=x onerror=window.__mcpMarkupExecuted=true>' }] })),
        };
        mountMcpConnections({
            connectRemote: async (config: { url: string; credential: string }) => { connections.push(config); return session; },
        });
        expect(connections).toHaveLength(0);
        expect(session.listTools).not.toHaveBeenCalled();
        expect((document.getElementById('mcpStdioButton') as HTMLButtonElement).disabled).toBe(true);
        expect(document.getElementById('mcpStdioButton')?.textContent).toContain('browser build');

        (document.getElementById('mcpUrlInput') as HTMLInputElement).value = 'http://127.0.0.1:9000/mcp';
        (document.getElementById('mcpCredentialInput') as HTMLInputElement).value = 'memory-only';
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        expect(connections).toEqual([{ url: 'http://127.0.0.1:9000/mcp', credential: 'memory-only' }]);
        expect(session.listTools).toHaveBeenCalledOnce();
        expect(document.querySelector('.mcp-tool-button')?.textContent).toContain('<img');
        expect(document.querySelector('.mcp-tool-button img')).toBeNull();

        (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('<img'));
        expect(session.callTool).toHaveBeenCalledWith('read_file', {});
        expect(document.querySelector('#mcpResultOutput img')).toBeNull();
        expect((window as unknown as { __mcpMarkupExecuted?: boolean }).__mcpMarkupExecuted).toBeUndefined();
        expect(window.localStorage.getItem('uor.studio.mcp.credential')).toBeNull();

        document.body.innerHTML = '<div id="modelSourceMount"></div>';
        mountMcpConnections({ connectRemote: vi.fn() });
        expect((document.getElementById('mcpCredentialInput') as HTMLInputElement).value).toBe('');
        expect(document.querySelector('.mcp-tool-button')).toBeNull();
    });

    it('does not treat the loopback core-only shim as full native MCP availability', () => {
        document.body.innerHTML = '<div id="mcpMount"></div>';
        (window as unknown as { __TAURI__?: unknown }).__TAURI__ = { core: { invoke: vi.fn() } };
        mountMcpConnections({ connectRemote: vi.fn() });
        expect((document.getElementById('mcpStdioButton') as HTMLButtonElement).disabled).toBe(true);
        delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
    });

    it('keeps MCP mounted when the model-source container rerenders', () => {
        document.body.innerHTML = '<div id="modelSourceMount"></div><div id="mcpMount"></div>';
        mountMcpConnections({ connectRemote: vi.fn() });
        expect(document.getElementById('mcpConnections')).not.toBeNull();
        document.getElementById('modelSourceMount')!.innerHTML = '';
        expect(document.getElementById('mcpConnections')).not.toBeNull();
    });
});
