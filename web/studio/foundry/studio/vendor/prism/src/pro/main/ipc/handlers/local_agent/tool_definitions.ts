// prism-prism replacement for the Functional Source License tool definitions. Not Prism's code.
// The renderer imports only the tool name type. The names are the PrismPM tool contract's; the
// model owns which tools exist and what needs approval, and this list must match `tools` there.
export const AGENT_TOOL_NAMES = ["read_file", "write_file", "edit_file", "list_files", "search_files", "snapshot", "rollback"] as const;
export type AgentToolName = (typeof AGENT_TOOL_NAMES)[number];
