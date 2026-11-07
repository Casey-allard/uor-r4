import { useEffect, useRef, useState } from 'preact/hooks';
import type { DesktopActions } from '../adapters/desktopActions.js';
import { desktopRepositoryPath } from '../adapters/desktopActions.js';
import type { WorkspaceApi } from '../adapters/workflow.js';

interface DesktopPanelProps {
    actions: DesktopActions;
    files: WorkspaceApi;
}

interface ChangesPanelProps extends DesktopPanelProps {
    copy(text: string): void | Promise<void>;
}

function useWorkspacePath(files: WorkspaceApi) {
    const [path, setPath] = useState(() => desktopRepositoryPath(files));
    const [revision, setRevision] = useState(0);
    useEffect(() => {
        const changed = (event: Event) => {
            const rootPath = String((event as CustomEvent<{ rootPath?: string }>).detail?.rootPath || '');
            setPath(rootPath);
            setRevision((value) => value + 1);
        };
        document.addEventListener('uor-files-changed', changed);
        return () => document.removeEventListener('uor-files-changed', changed);
    }, []);
    const updatePath = (value: string) => {
        setPath(value);
        setRevision((current) => current + 1);
    };
    return { path, revision, updatePath };
}

export function ChangesPanel({ actions, files, copy }: ChangesPanelProps) {
    const { path, revision, updatePath } = useWorkspacePath(files);
    const latestRevision = useRef(revision);
    latestRevision.current = revision;
    const operation = useRef(0);
    const [refreshing, setRefreshing] = useState(false);
    const [summary, setSummary] = useState('');
    const [diff, setDiff] = useState('');
    const [displayDiff, setDisplayDiff] = useState('');
    const diffRef = useRef('');

    useEffect(() => {
        operation.current += 1;
        setRefreshing(false);
        setSummary('');
        setDiff('');
        setDisplayDiff('');
        diffRef.current = '';
        return () => { operation.current += 1; };
    }, [revision]);

    async function refresh() {
        const repoPath = path.trim();
        if (!repoPath || !actions.available || refreshing) return;
        const id = ++operation.current;
        const startedRevision = latestRevision.current;
        setRefreshing(true);
        try {
            const [status, report] = await Promise.all([actions.gitStatus(repoPath), actions.gitDiff(repoPath)]);
            if (id !== operation.current || startedRevision !== latestRevision.current) return;
            const changedFiles = (status.modified_files?.length || 0) + (status.untracked_files?.length || 0);
            const limits = [
                status.truncated || report.truncated ? 'truncated' : '',
                status.timed_out || report.timed_out ? 'timed out' : '',
            ].filter(Boolean).join(' · ');
            setSummary(`${status.branch} · ${changedFiles} changed file${changedFiles === 1 ? '' : 's'}${limits ? ` · ${limits}` : ''}`);
            const rawDiff = String(report.diff || '');
            diffRef.current = rawDiff;
            setDiff(rawDiff);
            setDisplayDiff(rawDiff + (report.truncated ? '\n[diff truncated]' : '') + (report.timed_out ? '\n[diff timed out]' : ''));
        } catch (error) {
            if (id !== operation.current || startedRevision !== latestRevision.current) return;
            setSummary(`Git inspection failed: ${String(error instanceof Error ? error.message : error)}`);
            setDiff('');
            setDisplayDiff('No diff available.');
            diffRef.current = '';
        } finally {
            if (id === operation.current) setRefreshing(false);
        }
    }

    async function copyDiff() {
        if (!diffRef.current) return;
        try {
            await copy(diffRef.current);
        } catch (error) {
            setSummary(`Could not copy the diff: ${String(error instanceof Error ? error.message : error)}`);
        }
    }

    return (
        <section aria-label="Changes">
            <h1>Changes</h1>
            <p id="gitStatusSummary">{summary || (actions.available ? (path.trim() ? 'Refresh to inspect repository changes.' : 'Open a project folder in Files before inspecting changes.') : 'Native Git is unavailable in this browser build; use the desktop build.')}</p>
            <label>Repository path<input id="terminalCwd" aria-label="Repository path" value={path} onInput={(event) => updatePath(event.currentTarget.value)} /></label>
            <button id="btnRefreshChanges" type="button" disabled={!actions.available || refreshing || !path.trim()} onClick={() => void refresh()}>{refreshing ? 'Refreshing…' : 'Refresh changes'}</button>
            <button id="btnCopyDiff" type="button" disabled={!diff} onClick={() => void copyDiff()}>Copy diff</button>
            <pre id="gitDiffOutput" className="desktop-output" aria-label="Git diff">{displayDiff}</pre>
        </section>
    );
}

export function TerminalPanel({ actions, files }: DesktopPanelProps) {
    const { path, revision, updatePath } = useWorkspacePath(files);
    const latestRevision = useRef(revision);
    latestRevision.current = revision;
    const enableOperation = useRef(0);
    const runOperation = useRef(0);
    const [enabled, setEnabled] = useState(false);
    const [enabling, setEnabling] = useState(false);
    const [running, setRunning] = useState(false);
    const [command, setCommand] = useState('');
    const [consent, setConsent] = useState('');
    const [output, setOutput] = useState('');

    useEffect(() => {
        enableOperation.current += 1;
        runOperation.current += 1;
        setEnabling(false);
        setRunning(false);
        setOutput('');
        return () => {
            enableOperation.current += 1;
            runOperation.current += 1;
        };
    }, [revision]);

    async function enable() {
        if (!actions.available || enabling || running) return;
        const id = ++enableOperation.current;
        setEnabling(true);
        try {
            const result = await actions.enableTerminal();
            if (id !== enableOperation.current) return;
            const accepted = result?.enabled === true;
            setEnabled(accepted);
            setConsent(accepted
                ? 'Terminal is enabled for this app session. Every command still needs separate native approval.'
                : 'Terminal enablement was cancelled.');
        } catch (error) {
            if (id !== enableOperation.current) return;
            setEnabled(false);
            setConsent(`Could not enable terminal: ${String(error instanceof Error ? error.message : error)}`);
        } finally {
            if (id === enableOperation.current) setEnabling(false);
        }
    }

    async function run() {
        const cwd = path.trim();
        const allowedCommand = command.trim();
        if (!cwd || !allowedCommand || !enabled || !actions.available || running || enabling) return;
        const id = ++runOperation.current;
        const startedRevision = latestRevision.current;
        setRunning(true);
        setOutput('Waiting for native command approval…');
        try {
            const result = await actions.runTerminalCommand(cwd, allowedCommand);
            if (id !== runOperation.current || startedRevision !== latestRevision.current) return;
            setOutput(result === null ? 'Command approval was cancelled; nothing ran.' : String(result || '(no output)'));
        } catch (error) {
            if (id !== runOperation.current || startedRevision !== latestRevision.current) return;
            setOutput(`Command failed: ${String(error instanceof Error ? error.message : error)}`);
        } finally {
            if (id === runOperation.current) setRunning(false);
        }
    }

    const runDisabled = !actions.available || !enabled || enabling || running || !path.trim() || !command.trim();
    const emptyGuidance = !path.trim() ? 'Choose a project folder and enter an allowed command.' : !command.trim() ? 'Enter an allowed command.' : '';

    return (
        <section aria-label="Terminal">
            <h1>Terminal</h1>
            <p id="terminalConsentState">{consent || (actions.available ? 'Enable terminal for this app session. Every command still needs separate native approval.' : 'Guarded terminal actions are unavailable in this browser build.')}</p>
            <button id="btnTerminalEnable" type="button" disabled={!actions.available || enabling || running} onClick={() => void enable()}>{enabling ? 'Enabling…' : 'Enable terminal'}</button>
            <label>Working directory<input id="terminalCwd" aria-label="Working directory" value={path} onInput={(event) => updatePath(event.currentTarget.value)} /></label>
            <label>Command<input id="terminalCommand" aria-label="Terminal command" value={command} onInput={(event) => setCommand(event.currentTarget.value)} onKeyDown={(event) => {
                if (event.key === 'Enter' && !runDisabled) {
                    event.preventDefault();
                    void run();
                }
            }} /></label>
            <button id="btnRunTerminal" type="button" disabled={runDisabled} onClick={() => void run()}>{running ? 'Waiting for approval…' : 'Run command'}</button>
            <pre id="terminalOutput" className="desktop-output" aria-label="Terminal output">{output || emptyGuidance}</pre>
        </section>
    );
}
