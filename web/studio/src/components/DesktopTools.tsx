import { useEffect, useRef, useState } from 'preact/hooks';
import type { DesktopActions } from '../adapters/desktopActions.js';
import { desktopRepositoryPath } from '../adapters/desktopActions.js';
import type { WorkspaceApi } from '../adapters/workflow.js';
import { Icon } from './ui/Icon.js';

interface DesktopPanelProps { actions: DesktopActions; files: WorkspaceApi; idPrefix?: string }
interface ChangesPanelProps extends DesktopPanelProps { copy(text: string): void | Promise<void> }

export function useWorkspacePath(files: WorkspaceApi) {
    const [path, setPath] = useState(() => desktopRepositoryPath(files));
    const [revision, setRevision] = useState(0);
    useEffect(() => {
        const changed = (event: Event) => { const rootPath = String((event as CustomEvent<{ rootPath?: string }>).detail?.rootPath || ''); setPath(rootPath); setRevision((value) => value + 1); };
        document.addEventListener('uor-files-changed', changed);
        return () => document.removeEventListener('uor-files-changed', changed);
    }, []);
    return { path, revision, updatePath: (value: string) => { setPath(value); setRevision((current) => current + 1); } };
}

export function ChangesPanel({ actions, files, copy, idPrefix = '' }: ChangesPanelProps) {
    const { path, revision, updatePath } = useWorkspacePath(files);
    const latestRevision = useRef(revision); latestRevision.current = revision;
    const operation = useRef(0); const [refreshing, setRefreshing] = useState(false); const [summary, setSummary] = useState(''); const [diff, setDiff] = useState(''); const [displayDiff, setDisplayDiff] = useState(''); const diffRef = useRef('');
    const id = (name: string) => `${idPrefix}${name}`;
    useEffect(() => { operation.current += 1; setRefreshing(false); setSummary(''); setDiff(''); setDisplayDiff(''); diffRef.current = ''; return () => { operation.current += 1; }; }, [revision]);
    async function refresh() {
        const repoPath = path.trim(); if (!repoPath || !actions.available || refreshing) return;
        const request = ++operation.current; const startedRevision = latestRevision.current; setRefreshing(true);
        try {
            const [status, report] = await Promise.all([actions.gitStatus(repoPath), actions.gitDiff(repoPath)]);
            if (request !== operation.current || startedRevision !== latestRevision.current) return;
            const changedFiles = (status.modified_files?.length || 0) + (status.untracked_files?.length || 0);
            const limits = [status.truncated || report.truncated ? 'truncated' : '', status.timed_out || report.timed_out ? 'timed out' : ''].filter(Boolean).join(' · ');
            setSummary(`${status.branch} · ${changedFiles} changed file${changedFiles === 1 ? '' : 's'}${limits ? ` · ${limits}` : ''}`);
            const rawDiff = String(report.diff || ''); diffRef.current = rawDiff; setDiff(rawDiff); setDisplayDiff(rawDiff + (report.truncated ? '\n[diff truncated]' : '') + (report.timed_out ? '\n[diff timed out]' : ''));
        } catch (error) {
            if (request !== operation.current || startedRevision !== latestRevision.current) return;
            setSummary(`Git inspection failed: ${String(error instanceof Error ? error.message : error)}`); setDiff(''); setDisplayDiff('No diff available.'); diffRef.current = '';
        } finally { if (request === operation.current) setRefreshing(false); }
    }
    async function copyDiff() { if (!diffRef.current) return; try { await copy(diffRef.current); } catch (error) { setSummary(`Could not copy the diff: ${String(error instanceof Error ? error.message : error)}`); } }
    return <section aria-label="Changes" class="desktop-panel changes-panel"><header class="panel-heading"><div><h1>Changes</h1><p>Review the working tree before choosing what to ship.</p></div><span class="panel-icon"><Icon name="changes" /></span></header>
        <p id={id('gitStatusSummary')} class="panel-summary">{summary || (actions.available ? (path.trim() ? 'Refresh to inspect repository changes.' : 'Open a project folder in Files before inspecting changes.') : 'Native Git is unavailable in this browser build; use the desktop build.')}</p>
        <label>Repository path<input id={id('terminalCwd')} aria-label="Repository path" value={path} onInput={(event) => updatePath(event.currentTarget.value)} /></label>
        <div class="panel-actions"><button id={id('btnRefreshChanges')} type="button" disabled={!actions.available || refreshing || !path.trim()} onClick={() => void refresh()}>{refreshing ? 'Refreshing…' : 'Refresh changes'}</button><button id={id('btnCopyDiff')} type="button" disabled={!diff} onClick={() => void copyDiff()}>Copy diff</button></div>
        <pre id={id('gitDiffOutput')} class="desktop-output" aria-label="Git diff">{displayDiff}</pre></section>;
}

export { TerminalPanel } from './TerminalSession.js';
