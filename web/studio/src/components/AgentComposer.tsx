import { useEffect, useRef, useState } from 'preact/hooks';
import { readTextFiles } from '../adapters/composerInput.js';
import type { ComposerInputController } from '../adapters/composerInput.js';
import { GENERATION_FINISHED_EVENT } from '../adapters/generation.js';
import type { GenerationAdapter } from '../adapters/generation.js';
import type { Attachment, SessionApi, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import type { AgentClient } from '../agent/client.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import type { RunView, TaskView } from '../state/agent.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { AgentTranscript } from './AgentTranscript.js';
import { TaskDetail } from './TaskDetail.js';
import { attachmentSize, draftFromStoredAttachment, planAttachmentBatch } from './ComposerAttachments.js';
import type { AttachmentDraft } from './ComposerAttachments.js';
import { MAX_DOCUMENT_BYTES, MAX_TOTAL_ATTACHMENT_BYTES } from './ComposerAttachments.js';
import { LOCAL_TEMPLATES } from './ComposerMenu.js';
import { ComposerSurface } from './ComposerSurface.js';
import type { PermissionMode } from './PermissionPicker.js';
import { isActiveRun } from './SendModeControls.js';
import { Icon } from './ui/Icon.js';

interface AgentComposerProps {
    store: LifecycleStore;
    sessions: SessionApi;
    files: WorkspaceApi;
    ui: StudioUiApi;
    generation: GenerationAdapter;
    copy(text: string): void | Promise<void>;
    download(name: string, content: string): void;
    draft?: string;
    pasted?: string;
    onDraftChange?(value: string): void;
    onPastedChange?(value: string): void;
    agentClient?: AgentClient | null;
    agentTask?: TaskView | null;
    activeRun?: RunView | null;
    unresolvedRun?: boolean;
    runtimeTasksLoading?: boolean;
    runtimeTasksReady?: boolean;
    runtimeTaskCount?: number;
    showTaskDetail?: boolean;
    onTaskBack?(): void;
    onNewRun?(): void;
    onCreateTask?(): Promise<string | null>;
    onOpenSurface?(surface: 'changes' | 'terminal'): void;
}

let insertSequence = 0;
let imageSequence = 0;

function nextInsertName(): string {
    insertSequence += 1;
    return `agent-message-${Date.now()}-${insertSequence}.md`;
}

function revokePreview(draft: AttachmentDraft): void {
    if (draft.previewUrl && typeof URL.revokeObjectURL === 'function') {
        URL.revokeObjectURL(draft.previewUrl);
        draft.previewUrl = undefined;
    }
}

export function AgentComposer({ store, sessions, files, ui, generation, copy, download, draft: controlledDraft, pasted: controlledPasted, onDraftChange, onPastedChange, agentClient = null, agentTask = null, activeRun = null, unresolvedRun = false, runtimeTasksLoading = false, runtimeTasksReady = true, runtimeTaskCount, showTaskDetail = false, onTaskBack, onNewRun, onCreateTask, onOpenSurface }: AgentComposerProps) {
    const state = useLifecycle(store);
    const [pastedState, setPastedState] = useState('');
    const pasted = controlledPasted ?? pastedState;
    const setPasted = (value: string) => { setPastedState(value); onPastedChange?.(value); };
    const [visibleDraft, setVisibleDraft] = useState(controlledDraft ?? '');
    const [imageDrafts, setImageDrafts] = useState<AttachmentDraft[]>([]);
    const [runtimeError, setRuntimeError] = useState<string | null>(null);
    const [permissionMode, setPermissionMode] = useState<PermissionMode>('ask');
    const initialDraftRef = useRef(controlledDraft ?? '');
    const previousControlledDraft = useRef(controlledDraft);
    const ownControlledDraft = useRef<string | null>(null);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const controllerRef = useRef<ComposerInputController | null>(null);
    const imageDraftsRef = useRef<AttachmentDraft[]>([]);
    const mountedRef = useRef(true);
    const attachmentQueueRef = useRef<Promise<void>>(Promise.resolve());
    const attachmentEpochRef = useRef(0);
    const attachmentAbortsRef = useRef<Set<() => void>>(new Set());
    const attachmentNameVersionsRef = useRef<Map<string, number>>(new Map());
    const pendingAttachmentCountRef = useRef(0);

    function isAttachmentOperationCurrent(epoch: number): boolean {
        return mountedRef.current && epoch === attachmentEpochRef.current;
    }

    function invalidateAttachmentOperations(): void {
        attachmentEpochRef.current += 1;
        attachmentNameVersionsRef.current.clear();
        const aborts = Array.from(attachmentAbortsRef.current);
        attachmentAbortsRef.current.clear();
        for (const abort of aborts) abort();
    }

    function bumpAttachmentNameVersion(name: string): void {
        attachmentNameVersionsRef.current.set(name, (attachmentNameVersionsRef.current.get(name) ?? 0) + 1);
    }

    function attachmentRemovedMessage(name: string): string {
        return `${name} was removed before it could be attached.`;
    }

    function toastIfCurrent(epoch: number, text: string, tone = 'error'): void {
        if (isAttachmentOperationCurrent(epoch)) ui.toast(text, tone);
    }

    interface AttachmentOperationSignal {
        aborted: Promise<null>;
        isCurrent(): boolean;
        isNameCurrent(name: string): boolean;
    }

    function enqueueAttachmentOperation<T>(task: (epoch: number, signal: AttachmentOperationSignal) => Promise<T>, fallback: T): Promise<T> {
        const epoch = attachmentEpochRef.current;
        const nameVersions = new Map(attachmentNameVersionsRef.current);
        pendingAttachmentCountRef.current += 1;
        const run = attachmentQueueRef.current.then(async () => {
            if (!isAttachmentOperationCurrent(epoch)) return fallback;
            let abort: () => void = () => {};
            const aborted = new Promise<null>((resolve) => { abort = () => resolve(null); });
            attachmentAbortsRef.current.add(abort);
            const signal: AttachmentOperationSignal = {
                aborted,
                isCurrent: () => isAttachmentOperationCurrent(epoch),
                isNameCurrent: (name) => (attachmentNameVersionsRef.current.get(name) ?? 0) === (nameVersions.get(name) ?? 0),
            };
            try {
                return await task(epoch, signal);
            } catch (error) {
                if (isAttachmentOperationCurrent(epoch)) ui.toast(`Attachments could not be read: ${String(error)}`, 'error');
                return fallback;
            } finally {
                attachmentAbortsRef.current.delete(abort);
            }
        });
        const settle = () => { pendingAttachmentCountRef.current = Math.max(0, pendingAttachmentCountRef.current - 1); };
        run.then(settle, settle);
        attachmentQueueRef.current = run.then(() => undefined, () => undefined);
        return run;
    }

    async function awaitAttachmentRead<T>(read: Promise<T>, signal: AttachmentOperationSignal): Promise<T | null> {
        if (!signal.isCurrent()) return null;
        const result = await Promise.race([read, signal.aborted]);
        return result === null ? null : result;
    }

    function commitImageDrafts(next: AttachmentDraft[]): void {
        imageDraftsRef.current = next;
        setImageDrafts(next);
    }

    function notifyDraftChange(value: string) {
        ownControlledDraft.current = value;
        onDraftChange?.(value);
    }

    function setDraft(value: string) {
        if (controllerRef.current) controllerRef.current.reset(value);
        setVisibleDraft(value);
        notifyDraftChange(value);
    }

    useEffect(() => {
        const changed = () => refresh();
        document.addEventListener(GENERATION_FINISHED_EVENT, changed);
        document.addEventListener('uor-attachments-changed', changed);
        return () => {
            document.removeEventListener(GENERATION_FINISHED_EVENT, changed);
            document.removeEventListener('uor-attachments-changed', changed);
        };
    }, []);

    useEffect(() => () => {
        mountedRef.current = false;
        invalidateAttachmentOperations();
        for (const draft of imageDraftsRef.current) revokePreview(draft);
        imageDraftsRef.current = [];
    }, []);

    const selected = state.selectedSource;
    const activeGeneration = state.session.activeGeneration;
    const residentMatches = selected?.kind === 'artifact'
        && state.artifact.phase === 'ready'
        && state.residentArtifact?.sourceId === selected.id;
    const prediction = !activeGeneration && residentMatches
        && state.residentArtifact?.canGenerate === false
        && state.residentArtifact?.canPredict === true;
    const generating = state.session.phase === 'generating';
    const messages = sessions.getConversationMessages();
    const storedAttachments = ui.getAttachments();
    const attachments = [...storedAttachments.map((attachment) => draftFromStoredAttachment(attachment)), ...imageDrafts];
    const activeFile = files.activeFile();
    const unavailable = !activeGeneration && residentMatches
        && (state.residentArtifact?.engineSupported === false
            || (state.residentArtifact?.canGenerate === false && state.residentArtifact?.canPredict !== true));
    const predictionMode = activeGeneration ? activeGeneration.canStop === false : prediction && !unavailable;
    const selectionLabel = activeGeneration
        ? (predictionMode ? 'Local next-token prediction' : activeGeneration.source.kind === 'artifact' ? 'Local generation' : 'API chat')
        : !selected ? 'No source selected'
        : selected.kind === 'artifact' && !residentMatches ? 'Selected model; not ready'
        : selected.kind === 'api' && !store.canRun() ? 'Invalid API endpoint'
        : unavailable ? 'Generation unavailable'
        : predictionMode ? 'Local next-token prediction'
        : selected.kind === 'artifact' ? 'Local generation' : 'API chat';

    const activeTaskRun = isActiveRun(activeRun) ? activeRun : null;
    const taskCount = runtimeTaskCount ?? agentClient?.store.snapshot().tasks.length ?? 0;
    const runtimeOwned = agentClient !== null;
    const runtimeTaskPath = runtimeOwned && runtimeTasksReady && taskCount > 0;
    const legacyAllowed = agentClient === null;
    const agentMode = runtimeOwned;
    const legacyReady = store.canRun();
    const autoCreateTask = runtimeOwned && taskCount === 0 && onCreateTask !== undefined;
    const sendDisabled = runtimeOwned
        ? runtimeTasksLoading || !runtimeTasksReady || (!agentTask && !autoCreateTask) || Boolean(activeTaskRun) || unresolvedRun
        : runtimeTasksLoading || !legacyAllowed || !legacyReady;

    function failureMessage(prefix: string, failure: unknown): string {
        return `${prefix}: ${failure instanceof Error ? failure.message : String(failure)}`;
    }

    function send(prompt: string): boolean | Promise<boolean> {
        if (runtimeOwned && agentClient !== null) {
            if (!runtimeTasksReady) return false;
            if (unresolvedRun) {
                ui.toast('This task\'s current run is still loading. Wait for it to resume before starting another.', 'error');
                return false;
            }
            if (activeTaskRun) {
                ui.toast('This task already has an active run. Queue, steer or force the message instead.', 'error');
                return false;
            }
            const start = (taskId: string) => agentClient.startRun({ taskId, prompt }).then(() => {
                setRuntimeError(null);
                refresh();
                return true;
            }, (failure: unknown) => {
                const message = failureMessage('Run could not start', failure);
                setRuntimeError(message);
                ui.toast(message, 'error');
                return false;
            });
            if (!agentTask) {
                if (!autoCreateTask || onCreateTask === undefined) {
                    ui.toast('Select a task before sending.', 'error');
                    return false;
                }
                return onCreateTask().then((taskId) => {
                    if (taskId === null) return false;
                    return start(taskId);
                }, (failure: unknown) => {
                    const message = failureMessage('Task could not be created', failure);
                    setRuntimeError(message);
                    ui.toast(message, 'error');
                    return false;
                });
            }
            return start(agentTask.taskId);
        }
        if (runtimeTasksLoading || !legacyAllowed) return false;
        if (!generation.send(prompt)) return false;
        refresh();
        return true;
    }

    function queue(prompt: string): boolean | Promise<boolean> {
        if (!agentClient || !agentTask || !activeTaskRun) {
            ui.toast('Queue requires a selected task with an active run.', 'error');
            return false;
        }
        return agentClient.queueMessage({ taskId: agentTask.taskId, text: prompt }).then(() => {
            refresh();
            return true;
        }, (failure: unknown) => {
            ui.toast(failureMessage('Queue failed', failure), 'error');
            return false;
        });
    }

    function steer(prompt: string): boolean | Promise<boolean> {
        if (!agentClient || !agentTask || !activeTaskRun) {
            ui.toast('Steer requires a selected task with an active run.', 'error');
            return false;
        }
        return agentClient.steer({ taskId: agentTask.taskId, text: prompt }).then(() => {
            refresh();
            return true;
        }, (failure: unknown) => {
            ui.toast(failureMessage('Steer failed', failure), 'error');
            return false;
        });
    }

    function force(prompt: string): boolean | Promise<boolean> {
        if (!agentClient || !agentTask || !activeTaskRun) {
            ui.toast('Force send requires a selected task with an active run.', 'error');
            return false;
        }
        return agentClient.forceSend({ taskId: agentTask.taskId, text: prompt }).then(() => {
            refresh();
            return true;
        }, (failure: unknown) => {
            ui.toast(failureMessage('Force send failed', failure), 'error');
            return false;
        });
    }

    function stopTaskRun() {
        if (!agentClient || !agentTask || !activeTaskRun) {
            ui.toast('Stop run requires a selected task with an active run.', 'error');
            return;
        }
        void agentClient.stop({ taskId: agentTask.taskId, runId: activeTaskRun.runId }).then(() => {
            refresh();
        }, (failure: unknown) => {
            ui.toast(failureMessage('Stop run failed', failure), 'error');
        });
    }

    function budgetItems() {
        return [
            ...ui.getAttachments().map((attachment) => ({
                name: attachment.name,
                size: attachmentSize(attachment),
                kind: attachment.kind === 'image' ? 'image' as const : attachment.source === 'file' ? 'document' as const : 'text' as const,
            })),
            ...imageDraftsRef.current.map((attachment) => ({ name: attachment.name, size: attachment.size, kind: 'image' as const })),
        ];
    }

    function removeStoredOrImage(id: string) {
        const image = imageDraftsRef.current.find((attachment) => attachment.id === id);
        if (image) {
            bumpAttachmentNameVersion(image.name);
            revokePreview(image);
            commitImageDrafts(imageDraftsRef.current.filter((attachment) => attachment.id !== id));
            return;
        }
        if (id.startsWith('stored:')) {
            const name = id.slice('stored:'.length);
            bumpAttachmentNameVersion(name);
            ui.removeAttachment(name);
            refresh();
        }
    }

    function clearAllAttachments() {
        invalidateAttachmentOperations();
        for (const attachment of imageDraftsRef.current) revokePreview(attachment);
        commitImageDrafts([]);
        ui.clearAttachments();
        refresh();
    }

    function addBudgetedAttachment(entry: Attachment): Promise<boolean> {
        return enqueueAttachmentOperation(async (epoch, signal) => {
            const plan = planAttachmentBatch([{ name: entry.name, size: attachmentSize(entry), type: entry.kind === 'image' ? 'image/png' : 'text/plain' }], budgetItems());
            for (const error of plan.errors) toastIfCurrent(epoch, error, 'error');
            if (!plan.accepted.length || !signal.isCurrent()) return false;
            if (!signal.isNameCurrent(entry.name)) {
                toastIfCurrent(epoch, attachmentRemovedMessage(entry.name), 'error');
                return false;
            }
            const previous = imageDraftsRef.current.find((attachment) => attachment.name === entry.name);
            if (previous) {
                revokePreview(previous);
                commitImageDrafts(imageDraftsRef.current.filter((attachment) => attachment.id !== previous.id));
            }
            ui.addAttachment(entry);
            refresh();
            return true;
        }, false);
    }

    function handleAttachFiles(filesToAttach: File[]): Promise<void> {
        return enqueueAttachmentOperation<void>(async (epoch, signal) => {
            const plan = planAttachmentBatch(filesToAttach, budgetItems());
            for (const error of plan.errors) toastIfCurrent(epoch, error, 'error');
            if (!plan.accepted.length || !signal.isCurrent()) return;
            const planned = new Map<string, { file: File; kind: 'image' | 'document' }>();
            for (const entry of plan.accepted) planned.set(entry.file.name, entry);
            for (const name of Array.from(planned.keys())) {
                if (signal.isNameCurrent(name)) continue;
                planned.delete(name);
                toastIfCurrent(epoch, attachmentRemovedMessage(name), 'error');
            }
            if (!planned.size || !signal.isCurrent()) return;
            const images = new Map<string, File>();
            const documents = new Map<string, File>();
            for (const entry of planned.values()) {
                if (entry.kind === 'image') images.set(entry.file.name, entry.file);
                else documents.set(entry.file.name, entry.file);
            }
            const nextDrafts: AttachmentDraft[] = [];
            let imagesCommitted = false;
            try {
                if (images.size) {
                    for (const [name, file] of images) {
                        let bytes: Uint8Array;
                        try {
                            const data = await awaitAttachmentRead(file.arrayBuffer(), signal);
                            if (!data) return;
                            bytes = new Uint8Array(data);
                        } catch (error) {
                            toastIfCurrent(epoch, `${name} could not be read: ${String(error)}`, 'error');
                            continue;
                        }
                        if (!signal.isCurrent()) return;
                        if (!signal.isNameCurrent(name)) {
                            toastIfCurrent(epoch, attachmentRemovedMessage(name), 'error');
                            continue;
                        }
                        const previewUrl = typeof URL.createObjectURL === 'function' ? URL.createObjectURL(file) : undefined;
                        nextDrafts.push({ id: `image:${name}:${++imageSequence}`, name, kind: 'image', bytes, size: bytes.byteLength, mediaType: file.type || 'application/octet-stream', previewUrl, memoryOnly: true });
                    }
                    if (!signal.isCurrent()) return;
                    const committable = nextDrafts.filter((attachment) => signal.isNameCurrent(attachment.name));
                    for (const draft of nextDrafts) {
                        if (committable.includes(draft)) continue;
                        revokePreview(draft);
                        toastIfCurrent(epoch, attachmentRemovedMessage(draft.name), 'error');
                    }
                    if (committable.length) {
                        const names = new Set(committable.map((attachment) => attachment.name));
                        for (const previous of imageDraftsRef.current.filter((attachment) => names.has(attachment.name))) revokePreview(previous);
                        for (const name of names) if (ui.getAttachments().some((attachment) => attachment.name === name)) ui.removeAttachment(name);
                        commitImageDrafts([...imageDraftsRef.current.filter((attachment) => !names.has(attachment.name)), ...committable]);
                        imagesCommitted = true;
                    }
                }
                if (documents.size) {
                    const existing = ui.getAttachments().filter((attachment) => attachment.kind !== 'image');
                    let result: { attachments: Attachment[]; errors: string[] } | null = null;
                    try {
                        result = await awaitAttachmentRead(readTextFiles(Array.from(documents.values()), {
                            existingAttachments: existing,
                            maxFileBytes: MAX_DOCUMENT_BYTES,
                            maxTotalBytes: MAX_TOTAL_ATTACHMENT_BYTES,
                        }) as Promise<{ attachments: Attachment[]; errors: string[] }>, signal);
                    } catch (error) {
                        toastIfCurrent(epoch, `Attachments could not be read: ${String(error)}`, 'error');
                        return;
                    }
                    if (!result || !signal.isCurrent()) return;
                    for (const error of result.errors) toastIfCurrent(epoch, error, 'error');
                    const currentAttachments = result.attachments.filter((attachment) => signal.isNameCurrent(attachment.name));
                    for (const attachment of result.attachments) {
                        if (signal.isNameCurrent(attachment.name)) continue;
                        toastIfCurrent(epoch, attachmentRemovedMessage(attachment.name), 'error');
                    }
                    for (const attachment of currentAttachments) ui.addAttachment(attachment);
                    const storedNames = new Set(currentAttachments.map((attachment) => attachment.name));
                    const replacedImages = imageDraftsRef.current.filter((attachment) => storedNames.has(attachment.name));
                    for (const previous of replacedImages) revokePreview(previous);
                    if (replacedImages.length) commitImageDrafts(imageDraftsRef.current.filter((attachment) => !storedNames.has(attachment.name)));
                }
                if (signal.isCurrent()) refresh();
            } finally {
                if (!imagesCommitted) for (const draft of nextDrafts) revokePreview(draft);
            }
        }, undefined);
    }

    useEffect(() => {
        if (controlledDraft === undefined || controlledDraft === previousControlledDraft.current) return;
        previousControlledDraft.current = controlledDraft;
        if (ownControlledDraft.current === controlledDraft) {
            ownControlledDraft.current = null;
            return;
        }
        setVisibleDraft(controlledDraft);
        if (controllerRef.current) controllerRef.current.reset(controlledDraft);
    }, [controlledDraft]);

    const modelLabel = selected ? selected.id : 'Choose model';
    const storedImageCount = attachments.filter((attachment) => attachment.kind === 'image' && !attachment.memoryOnly).length;
    const memoryImageCount = attachments.filter((attachment) => attachment.kind === 'image' && attachment.memoryOnly).length;

    return (
        <section aria-label="Agent composer" class="agent-workspace">
            {showTaskDetail && agentTask && agentClient
                ? <TaskDetail task={agentTask} client={agentClient} onBack={onTaskBack ?? (() => {})} onNewRun={onNewRun} onOpenSurface={onOpenSurface ?? (() => {})} />
                : <>
                    <header class="agent-heading"><div><h1>Agent</h1><p>{selectionLabel}</p></div><span class="session-phase">Session: {state.session.phase}</span></header>
                    {state.session.error && <p role="alert" class="workbench-alert">{state.session.error}</p>}
                    {runtimeError !== null && <p role="alert" class="workbench-alert">{runtimeError}</p>}
                    {activeGeneration && <p class="generation-banner">Generating with {activeGeneration.source.id} ({activeGeneration.privacy})</p>}
                    <AgentTranscript messages={messages} canRun={store.canRun()} busy={generating} copy={copy} insert={(content) => files.insertText(content, nextInsertName())} download={(content) => download('studio-message.md', content)} retry={(content) => generation.send(content)} />
                </>}
            <details class="context-drawer" aria-label="Context attachments"><summary><span><Icon name="attach" />Context</span><span class="attachment-count">{attachments.length}</span></summary><div class="context-content">
                <div class="context-actions"><button type="button" disabled={!activeFile} onClick={() => { const file = files.activeFile(); if (file) void addBudgetedAttachment({ name: file.path, content: file.content, source: 'editor' }); }}>Attach current file</button><button type="button" disabled={!attachments.length} onClick={clearAllAttachments}>Clear attachments</button></div>
                <label>Pasted context<textarea aria-label="Pasted context" value={pasted} onInput={(event) => setPasted(event.currentTarget.value)} /></label>
                <button type="button" disabled={!pasted.trim()} onClick={() => { void addBudgetedAttachment({ name: 'pasted-context.txt', content: pasted, source: 'paste' }).then((accepted) => { if (accepted) setPasted(''); }); }}>Attach pasted text</button>
                {(storedImageCount > 0 || memoryImageCount > 0) && <p class="composer-attachment-note">{storedImageCount > 0 && 'Stored image context comes from the existing Studio store and is not guaranteed after reload. '}{memoryImageCount > 0 && 'New images stay in this browser tab and are not sent for visual understanding.'}</p>}
            </div></details>
            <details class="build-tasks" aria-label="Build task templates"><summary><span><Icon name="command" />Build tasks</span><span class="attachment-count">{LOCAL_TEMPLATES.length}</span></summary><div class="build-task-grid">{LOCAL_TEMPLATES.map(([kind, label]) => <button type="button" data-template={kind} onClick={() => { setDraft(ui.templatePrompt(kind)); }}>{label}</button>)}</div></details>
            <ComposerSurface
                value={visibleDraft}
                onChange={(value) => setVisibleDraft(value)}
                onDraftChange={(value) => notifyDraftChange(value)}
                attachments={attachments}
                onAttach={handleAttachFiles}
                onRemove={removeStoredOrImage}
                onSend={send}
                onQueue={agentMode ? queue : undefined}
                onSteer={agentMode ? steer : undefined}
                onForce={agentMode ? force : undefined}
                onStop={() => { generation.stop(); refresh(); }}
                onStopRun={stopTaskRun}
                showGenerationStop={!showTaskDetail}
                onActionError={(error) => ui.toast(failureMessage('Composer action failed', error), 'error')}
                activeRun={activeTaskRun}
                mode={permissionMode}
                onMode={setPermissionMode}
                model={modelLabel}
                onModel={() => ui.setView('model')}
                environment="Local browser"
                onEnvironment={() => {}}
                disabled={sendDisabled}
                runtimeAvailable={false}
                initialDraft={initialDraftRef.current}
                sendLabel={agentMode ? 'Send' : predictionMode ? 'Predict next tokens' : 'Send'}
                generating={generating}
                canStop={activeGeneration?.canStop !== false}
                editorFile={activeFile}
                artifacts={[]}
                templatePrompt={(kind) => ui.templatePrompt(kind)}
                onMentionAdd={(entry) => addBudgetedAttachment(entry)}
                onAttachmentBarrier={() => pendingAttachmentCountRef.current > 0 ? attachmentQueueRef.current : null}
                onController={(controller) => { controllerRef.current = controller; }}
            />
        </section>
    );
}
