export interface McpTool {
    name: string;
    description?: string;
}

export interface McpSession {
    listTools(): Promise<McpTool[]>;
    callTool(name: string, args: Record<string, unknown>): Promise<unknown>;
    disconnect?(): Promise<unknown>;
}

export type McpConnect = (config: { url: string; credential: string }) => Promise<McpSession | null>;

// @ts-expect-error The existing remote/native MCP session factory is plain JavaScript.
export { connectRemote } from '../../assets/js/mcp_client.js';
