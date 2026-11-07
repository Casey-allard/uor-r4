import { useEffect, useRef, useState } from 'preact/hooks';
import type { McpConnect, McpSession, McpTool } from '../adapters/mcpConnections.js';
import { connectRemote as defaultConnectRemote } from '../adapters/mcpConnections.js';
import type { StudioUiApi } from '../adapters/workflow.js';

interface McpConnectionsProps {
    ui: StudioUiApi;
    connectRemote?: McpConnect;
}

function errorText(error: unknown): string {
    return String(error instanceof Error ? error.message : error);
}

async function release(session: McpSession | null): Promise<string> {
    if (!session?.disconnect) return '';
    try {
        await session.disconnect();
        return '';
    } catch (error) {
        const message = errorText(error);
        return message === 'MCP session is already disconnected.' ? '' : message;
    }
}

function attachmentName(toolName: string): string {
    const safe = typeof toolName === 'string' ? toolName.replace(/[^a-zA-Z0-9_-]+/g, '-').replace(/^-+|-+$/g, '') : '';
    return `mcp-${safe || 'tool-result'}.json`;
}

function validTools(tools: McpTool[]): McpTool[] {
    return (Array.isArray(tools) ? tools : []).filter((tool): tool is McpTool => Boolean(
        tool
        && typeof tool.name === 'string'
        && tool.name.trim()
        && (tool.description === undefined || typeof tool.description === 'string')
    ));
}

export function McpConnections({ ui, connectRemote = defaultConnectRemote }: McpConnectionsProps) {
    const [url, setUrl] = useState('');
    const [credential, setCredential] = useState('');
    const [session, setSession] = useState<McpSession | null>(null);
    const sessionRef = useRef<McpSession | null>(null);
    sessionRef.current = session;
    const [tools, setTools] = useState<McpTool[]>([]);
    const [status, setStatus] = useState('Not connected. No request is made until Connect.');
    const [connecting, setConnecting] = useState(false);
    const connectingRef = useRef(false);
    const [calling, setCalling] = useState(false);
    const callingRef = useRef(false);
    const [args, setArgs] = useState('{}');
    const [resultText, setResultText] = useState('No tool result.');
    const [resultJson, setResultJson] = useState<string | null>(null);
    const [resultTool, setResultTool] = useState('');
    const [attachedName, setAttachedName] = useState('');
    const operation = useRef(0);

    useEffect(() => () => {
        operation.current += 1;
        setCredential('');
        void release(sessionRef.current);
    }, []);

    async function connect() {
        if (connectingRef.current || callingRef.current) return;
        connectingRef.current = true;
        setConnecting(true);
        const token = ++operation.current;
        const previous = sessionRef.current;
        let candidate: McpSession | null = null;
        setStatus('Connecting…');
        try {
            candidate = await connectRemote({ url: url.trim(), credential });
            if (token !== operation.current) {
                await release(candidate);
                return;
            }
            if (!candidate) {
                setStatus(`Connection approval was cancelled; nothing started.${previous ? ' Previous connection is still active.' : ''}`);
                return;
            }
            const listed = validTools(await candidate.listTools());
            if (token !== operation.current) {
                await release(candidate);
                return;
            }
            let cleanup = '';
            if (previous && previous !== candidate) cleanup = await release(previous);
            if (token !== operation.current) {
                await release(candidate);
                return;
            }
            if (cleanup) {
                const candidateCleanup = await release(candidate);
                setSession(previous);
                setStatus(`Previous session cleanup failed: ${cleanup}.${candidateCleanup ? ` Candidate cleanup failed: ${candidateCleanup}.` : ''} Replacement was not completed.`);
                return;
            }
            setSession(candidate);
            setTools(listed);
            setResultJson(null);
            setResultText('No tool result.');
            setAttachedName('');
            setCredential('');
            setStatus(`Connected · ${listed.length} tool${listed.length === 1 ? '' : 's'}${cleanup ? ` Previous session cleanup failed: ${cleanup}` : ''}`);
        } catch (error) {
            const cleanup = await release(candidate);
            if (token === operation.current) {
                setSession(previous);
                setStatus(`${errorText(error)}${cleanup ? ` Replacement cleanup failed: ${cleanup}` : ''}`);
                if (!previous) setTools([]);
            }
        } finally {
            if (token === operation.current) {
                connectingRef.current = false;
                setConnecting(false);
            }
        }
    }

    async function call(tool: McpTool) {
        if (callingRef.current || connectingRef.current || !sessionRef.current) return;
        let parsed: Record<string, unknown>;
        try {
            const value = JSON.parse(args || '{}');
            if (!value || Array.isArray(value) || typeof value !== 'object') throw new Error('Tool arguments must be a JSON object.');
            parsed = value as Record<string, unknown>;
        } catch (error) {
            setResultJson(null);
            setResultText(errorText(error));
            return;
        }
        callingRef.current = true;
        setCalling(true);
        const token = ++operation.current;
        setResultText(`Calling ${tool.name}…`);
        try {
            const called = await sessionRef.current.callTool(tool.name, parsed);
            if (token !== operation.current) return;
            const serialized = JSON.stringify(called, null, 2);
            setResultJson(serialized);
            setResultText(serialized);
            setResultTool(tool.name);
            setAttachedName('');
        } catch (error) {
            if (token !== operation.current) return;
            setResultJson(null);
            setResultText(errorText(error));
        } finally {
            if (token === operation.current) {
                callingRef.current = false;
                setCalling(false);
            }
        }
    }

    function attachResult() {
        if (!resultJson || attachedName) return;
        try {
            const name = attachmentName(resultTool);
            ui.addAttachment({ name, kind: 'text', content: resultJson, source: 'mcp' });
            const retained = ui.getAttachments().some((entry) =>
                entry.name === name
                && entry.kind === 'text'
                && entry.content === resultJson
                && entry.source === 'mcp'
            );
            if (!retained) {
                setAttachedName('');
                ui.toast('MCP result could not be attached: the attachment limit was reached.', 'error');
                return;
            }
            setAttachedName(name);
            ui.toast('MCP result attached.', 'success');
        } catch (error) {
            setAttachedName('');
            ui.toast(`MCP result could not be attached: ${errorText(error)}`, 'error');
        }
    }

    return (
        <section aria-label="MCP servers" className="mcp-connections">
            <h2>MCP servers</h2>
            <p>Remote Streamable HTTP. Credentials stay in memory and are cleared after connection and on unmount.</p>
            <label>MCP server URL<input id="mcpUrlInput" aria-label="MCP server URL" type="url" value={url} onInput={(event) => setUrl(event.currentTarget.value)} placeholder="http://127.0.0.1:9000/mcp" /></label>
            <label>MCP credential<input id="mcpCredentialInput" aria-label="MCP credential" type="password" value={credential} onInput={(event) => setCredential(event.currentTarget.value)} placeholder="Optional credential (memory only)" autocomplete="off" /></label>
            <button id="mcpConnectButton" type="button" disabled={connecting || calling || !url.trim()} onClick={() => void connect()}>{connecting ? 'Connecting…' : 'Connect'}</button>
            <p id="mcpConnectionStatus">{status}</p>
            <div id="mcpToolsList" className="mcp-tool-list">
                {tools.map((tool) => (
                    <button className="mcp-tool-button" type="button" disabled={calling || connecting} onClick={() => void call(tool)}>{`${tool.name} — ${tool.description || 'No description'}`}</button>
                ))}
            </div>
            <label>Tool arguments as JSON<textarea id="mcpToolArgs" aria-label="Tool arguments as JSON" rows={3} value={args} onInput={(event) => setArgs(event.currentTarget.value)} /></label>
            <button id="mcpAttachResult" type="button" disabled={!resultJson || Boolean(attachedName)} onClick={attachResult}>{attachedName ? 'Attached' : 'Attach result'}</button>
            <pre id="mcpResultOutput" className="mcp-result desktop-output" aria-label="MCP tool result">{resultText}</pre>
        </section>
    );
}
