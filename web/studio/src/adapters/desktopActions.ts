import type { WorkspaceApi, WorkspaceSummary } from './workflow.js';

export interface DesktopGitStatus {
    branch: string;
    is_clean?: boolean;
    modified_files?: string[];
    untracked_files?: string[];
    truncated?: boolean;
    timed_out?: boolean;
}

export interface DesktopGitDiff {
    diff: string;
    truncated?: boolean;
    timed_out?: boolean;
}

export interface TerminalEvent {
    type: 'output' | 'exit' | 'error';
    sessionId: string;
    sequence: number;
    dataHex?: string;
    code?: number | null;
    message?: string;
}

export interface TerminalSnapshot {
    sessionId: string;
    cursor: number;
    active: boolean;
    truncated: boolean;
    cwd: string;
    shell: string;
    pid: number | null;
    events: TerminalEvent[];
}

export interface DesktopActions {
    available: boolean;
    gitStatus(repoPath: string): Promise<DesktopGitStatus>;
    gitDiff(repoPath: string): Promise<DesktopGitDiff>;
    enableTerminal(): Promise<{ enabled?: boolean } | null | undefined>;
    runTerminalCommand(cwd: string, command: string): Promise<unknown>;
    openTerminal(cwd: string, rows: number, cols: number): Promise<TerminalSnapshot>;
    readTerminal(sessionId: string, after: number): Promise<TerminalSnapshot>;
    inputTerminal(sessionId: string, data: string): Promise<unknown>;
    resizeTerminal(sessionId: string, rows: number, cols: number): Promise<unknown>;
    closeTerminal(sessionId: string): Promise<TerminalSnapshot>;
}

export function desktopRepositoryPath(files: WorkspaceApi): string {
    const summary = files.treeSummary() as WorkspaceSummary & { rootPath?: string };
    return summary.rootPath || '';
}

// @ts-expect-error The existing native desktop action adapter is plain JavaScript.
export { createDesktopActions } from '../../assets/js/desktop_actions.js';
