import { useEffect, useRef, useState } from 'preact/hooks';
import type { DesktopActions } from '../adapters/desktopActions.js';
import { desktopRepositoryPath } from '../adapters/desktopActions.js';
import type { WorkspaceApi } from '../adapters/workflow.js';
import { AgentClientError, type AgentClient, type ChangesSnapshot, type HostGrant } from '../agent/client.js';
import type { DiffCommentRecord } from '../agent/store.js';
import { DiffReview, createUploadedSnapshot, snapshotFileLines, type DiffComment, type DiffCommentDraft, type DiffSource, type DiffScope } from './DiffReview.js';
import { isActiveRun } from './SendModeControls.js';
import { Icon } from './ui/Icon.js';

interface DesktopPanelProps { actions: DesktopActions; files: WorkspaceApi; idPrefix?: string }
interface ChangesPanelProps extends DesktopPanelProps {
    copy(text: string): void | Promise<void>;
    agentClient?: AgentClient | null;
    taskId?: string | null;
}

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

function commentFromRecord(record: DiffCommentRecord): DiffComment {
    return {
        commentId: record.comment_id,
        taskId: record.task_id,
        scope: record.scope,
        path: record.path,
        line: record.line,
        body: record.body,
        headDigest: record.head_digest,
        createdAtMs: record.created_at_ms,
    };
}

export function ChangesPanel({ actions, files, copy, idPrefix = '', agentClient = null, taskId = null }: ChangesPanelProps) {
    const { path, revision, updatePath } = useWorkspacePath(files);
    const latestRevision = useRef(revision); latestRevision.current = revision;
    const operation = useRef(0); const [refreshing, setRefreshing] = useState(false); const [summary, setSummary] = useState(''); const [diff, setDiff] = useState(''); const [displayDiff, setDisplayDiff] = useState(''); const diffRef = useRef('');
    const [scope, setScope] = useState<DiffScope>(taskId === null ? 'branch' : 'last_turn');
    const [uploaded, setUploaded] = useState<ChangesSnapshot | null>(null);
    const [snapshot, setSnapshot] = useState<ChangesSnapshot | null>(null);
    const [source, setSource] = useState<DiffSource>(null);
    const [reviewLoading, setReviewLoading] = useState(false);
    const [reviewError, setReviewError] = useState<string | null>(null);
    const [comments, setComments] = useState<DiffComment[]>([]);
    const [commentError, setCommentError] = useState<string | null>(null);
    const [commentRevision, setCommentRevision] = useState(0);
    const [grants, setGrants] = useState<HostGrant[]>([]);
    const [grantId, setGrantId] = useState('');
    const [grantsLoaded, setGrantsLoaded] = useState(agentClient === null);
    const [grantError, setGrantError] = useState<string | null>(null);
    const grantCapability = agentClient !== null
        && typeof agentClient.hostSupportsGrants === 'function'
        && agentClient.hostSupportsGrants();
    const [loadedKey, setLoadedKey] = useState<string | null>(null);
    const [commentsKey, setCommentsKey] = useState<string | null>(null);
    const reviewOperation = useRef(0);
    const identityKey = `${taskId ?? ''}:${scope}`;
    const reviewReady = loadedKey === identityKey;
    const commentsReady = commentsKey === taskId;
    const identityRef = useRef<{
        key: string;
        taskId: string | null;
        scope: DiffScope;
        loadedKey: string | null;
        snapshot: ChangesSnapshot | null;
    }>({ key: identityKey, taskId, scope, loadedKey: null, snapshot: null });
    identityRef.current = {
        key: identityKey,
        taskId,
        scope,
        loadedKey: reviewReady ? identityKey : null,
        snapshot: reviewReady ? snapshot : null,
    };
    const visibleComments = reviewReady && commentsReady ? comments : [];
    const id = (name: string) => `${idPrefix}${name}`;
    useEffect(() => { operation.current += 1; setRefreshing(false); setSummary(''); setDiff(''); setDisplayDiff(''); diffRef.current = ''; return () => { operation.current += 1; }; }, [revision]);

    useEffect(() => {
        let active = true;
        if (!grantCapability) {
            setGrants([]);
            setGrantId('');
            setGrantsLoaded(agentClient === null);
            setGrantError(null);
            return () => { active = false; };
        }
        setGrantsLoaded(false);
        void agentClient.grants().then(
            (available) => {
                if (!active) return;
                setGrants(available);
                setGrantId(available.length === 1 ? available[0]!.grantId : '');
                setGrantsLoaded(true);
                setGrantError(null);
            },
            (error: unknown) => {
                if (!active) return;
                setGrants([]);
                setGrantId('');
                setGrantsLoaded(true);
                setGrantError(`Approved folders could not be loaded: ${error instanceof Error ? error.message : String(error)}`);
            },
        );
        return () => { active = false; };
    }, [agentClient]);

    useEffect(() => {
        const request = ++reviewOperation.current;
        const currentKey = `${taskId ?? ''}:${scope}`;
        setSnapshot(null);
        setSource(null);
        setReviewError(null);
        setLoadedKey(null);
        if (scope === 'branch' && uploaded !== null) {
            setSnapshot(uploaded);
            setSource('upload');
            setReviewLoading(false);
            setLoadedKey(currentKey);
            return () => { reviewOperation.current += 1; };
        }
        if (agentClient === null || taskId === null) {
            setReviewLoading(false);
            return () => { reviewOperation.current += 1; };
        }
        const requiresGrant = scope === 'branch' && grantCapability;
        if (requiresGrant && (!grantsLoaded || grantError !== null || grants.length === 0 || grantId === '')) {
            if (!grantsLoaded) {
                setReviewLoading(true);
            } else {
                setReviewLoading(false);
                setSnapshot(null);
                setSource(null);
                setLoadedKey(null);
                setReviewError(
                    grantError ??
                        (grants.length === 0
                            ? 'Branch changes require an approved local folder grant. The local companion reports no approved folders.'
                            : null),
                );
            }
            return () => { reviewOperation.current += 1; };
        }
        setReviewLoading(true);
        void (async () => {
            try {
                const result = await agentClient.changesSnapshot({
                    taskId,
                    scope,
                    ...(requiresGrant ? { grantId } : {}),
                });
                if (request !== reviewOperation.current) return;
                setSnapshot(result);
                setSource(scope === 'last_turn' ? 'browser' : 'host');
                setLoadedKey(currentKey);
            } catch (error) {
                if (request !== reviewOperation.current) return;
                setSnapshot(null);
                setSource(null);
                setReviewError(
                    error instanceof AgentClientError && error.code === 'host_unavailable'
                        ? 'Branch changes require the local companion. Upload a diff to review one locally.'
                        : `Changes could not be loaded: ${error instanceof Error ? error.message : String(error)}`,
                );
            } finally {
                if (request === reviewOperation.current) setReviewLoading(false);
            }
        })();
        return () => { reviewOperation.current += 1; };
    }, [agentClient, taskId, scope, uploaded, grantsLoaded, grants, grantId, grantError, grantCapability]);

    useEffect(() => {
        let active = true;
        if (agentClient === null || taskId === null) {
            setComments([]);
            setCommentsKey(null);
            setCommentError(null);
            return () => { active = false; };
        }
        void agentClient.readDiffComments(taskId).then(
            (records) => {
                if (!active) return;
                setComments(records.map(commentFromRecord));
                setCommentsKey(taskId);
                setCommentError(null);
            },
            (error: unknown) => {
                if (!active) return;
                setComments([]);
                setCommentsKey(taskId);
                setCommentError(`Local comments could not be loaded: ${error instanceof Error ? error.message : String(error)}`);
            },
        );
        return () => { active = false; };
    }, [agentClient, taskId, commentRevision]);

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
    async function upload(input: HTMLInputElement) {
        const selected = input.files === null ? [] : Array.from(input.files);
        input.value = '';
        if (selected.length === 0) return;
        setLoadedKey(null);
        setSnapshot(null);
        setSource(null);
        setReviewError(null);
        setReviewLoading(true);
        try {
            const next = await createUploadedSnapshot(selected);
            setUploaded(next);
            setScope('branch');
        } catch (error) {
            setReviewLoading(false);
            setReviewError(`The uploaded file could not be read: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    function changeScope(next: DiffScope): void {
        if (next === scope) return;
        setLoadedKey(null);
        setSnapshot(null);
        setSource(null);
        setReviewError(null);
        setReviewLoading(true);
        setScope(next);
    }
    async function addComment(draft: DiffCommentDraft): Promise<void> {
        const identity = identityRef.current;
        if (agentClient === null || identity.taskId === null) throw new Error('Select a browser task before adding a local comment.');
        const current = identity.snapshot;
        if (identity.loadedKey !== identity.key || current === null) {
            throw new Error('The change source is still loading; wait for the new snapshot before commenting.');
        }
        const file = current.files.find((candidate) => candidate.path === draft.path);
        if (file === undefined) throw new Error('This file is not in the loaded snapshot.');
        const lines = snapshotFileLines(file);
        if (draft.line > 0 && (lines === null || draft.line > lines.length)) {
            throw new Error(`Line ${draft.line} is not present in this snapshot.`);
        }
        const requestKey = identity.key;
        const record = await agentClient.addDiffComment({
            taskId: identity.taskId,
            scope: identity.scope,
            path: draft.path,
            line: draft.line,
            body: draft.body,
            headDigest: current.headDigest,
        });
        if (identityRef.current.key !== requestKey) return;
        const saved = commentFromRecord(record);
        setComments((current) => [
            ...current.filter((item) => !(item.scope === saved.scope && item.path === saved.path && item.line === saved.line && item.headDigest === saved.headDigest)),
            saved,
        ]);
        setCommentsKey(saved.taskId);
        setCommentError(null);
        setCommentRevision((value) => value + 1);
    }

    const selectedTask = agentClient?.store?.snapshot().tasks.find((task) => task.taskId === taskId) ?? null;
    const selectedRun = selectedTask?.latestRunId === null || selectedTask?.latestRunId === undefined
        ? null
        : selectedTask.runs.find((run) => run.runId === selectedTask.latestRunId) ?? null;
    const runActive = isActiveRun(selectedRun);

    return <section aria-label="Changes" class="desktop-panel changes-panel"><header class="panel-heading"><div><h1>Changes</h1><p>Review the working tree before choosing what to ship.</p></div><span class="panel-icon"><Icon name="changes" /></span></header>
        <section class="changes-review" aria-label="Task changes review">
            <div class="diff-upload">
                <label for={id('diffUpload')}>Upload a diff or text file to review locally</label>
                <input id={id('diffUpload')} type="file" multiple accept=".diff,.patch,.txt,.md,text/plain" onChange={(event) => { void upload(event.currentTarget); }} />
                {uploaded !== null && <button id={id('btnClearUpload')} type="button" onClick={() => { setLoadedKey(null); setSnapshot(null); setSource(null); setReviewLoading(true); setUploaded(null); }}>Remove uploaded diff</button>}
            </div>
            {scope === 'branch' && uploaded === null && grants.length > 1 && <div class="diff-grant">
                <label for={id('diffGrant')}>Approved folder</label>
                <select
                    id={id('diffGrant')}
                    aria-label="Approved folder"
                    value={grantId}
                    onChange={(event) => { setGrantId(event.currentTarget.value); setLoadedKey(null); }}
                >
                    <option value="">Select an approved folder</option>
                    {grants.map((grant) => <option key={grant.grantId} value={grant.grantId}>{grant.path}</option>)}
                </select>
                {grantId === '' && <p class="diff-grant-prompt">Choose an approved folder before loading branch changes.</p>}
            </div>}
            {scope === 'branch' && uploaded === null && grants.length === 1 && <p class="diff-grant-single">Approved folder: {grants[0]!.path}</p>}
            {commentError !== null && <p class="panel-summary" role="alert">{commentError}</p>}
            <DiffReview
                taskId={taskId}
                scope={scope}
                onScope={changeScope}
                comments={visibleComments}
                onComment={addComment}
                snapshot={reviewReady ? snapshot : null}
                source={reviewReady ? source : null}
                loading={reviewLoading || !reviewReady}
                error={reviewError}
                commentDisabledReason={
                    agentClient === null || taskId === null
                        ? 'Select a browser task to add a local comment.'
                        : !reviewReady || reviewLoading
                          ? 'Loading this change source; wait for the new snapshot before commenting.'
                          : runActive
                            ? 'Local comments can be added after the current run finishes; this run is still active.'
                            : null
                }
                idPrefix={id('diffReview-')}
            />
        </section>
        <p id={id('gitStatusSummary')} class="panel-summary">{summary || (actions.available ? (path.trim() ? 'Refresh to inspect repository changes.' : 'Open a project folder in Files before inspecting changes.') : 'Git requires the local companion. Uploaded files can still be reviewed locally.')}</p>
        <label>Repository path<input id={id('terminalCwd')} aria-label="Repository path" value={path} onInput={(event) => updatePath(event.currentTarget.value)} /></label>
        <div class="panel-actions"><button id={id('btnRefreshChanges')} type="button" disabled={!actions.available || refreshing || !path.trim()} onClick={() => void refresh()}>{refreshing ? 'Refreshing…' : 'Refresh changes'}</button><button id={id('btnCopyDiff')} type="button" disabled={!diff} onClick={() => void copyDiff()}>Copy diff</button></div>
        <pre id={id('gitDiffOutput')} class="desktop-output" aria-label="Git diff">{displayDiff}</pre></section>;
}

export { TerminalPanel } from './TerminalSession.js';
