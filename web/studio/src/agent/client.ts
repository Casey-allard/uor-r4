import type {
    AgentCursor,
    AgentEvent,
    RunState,
} from './generated/agentContract.js';
import {
    createAgentStore,
    parseAgentEventWire,
    type AgentAppendResult,
    type AgentChangeSnapshotView,
    type AgentIncomingEvent,
    type AgentStore,
    type AgentTaskRecord,
} from '../state/agent.js';
import {
    AgentClientError,
    isPlainObject,
    toClientError,
} from './storeTypes.js';
import type { AgentTaskStore, DiffCommentRecord } from './store.js';
import type { AddDiffCommentInput, AgentRuntime, RestoredApproval } from './runtime.js';
import { HostClient, sha256Bytes, type HostGrant, type HostToolResult } from './hostClient.js';
import {
    AgentTransportHostTransport,
    NullHostTransport,
} from './transports.js';

export { AgentClientError } from './storeTypes.js';
export type { AgentIncomingEvent } from '../state/agent.js';
export type { HostGrant } from './hostClient.js';

const DIGEST_PATTERN = /^sha256:[0-9a-f]{64}$/;
const ARTIFACT_ID_PATTERN = /^art_[0-9a-f]{64}$/;

export const MAX_HOST_TREE_ENTRIES = 512;
export const MAX_HOST_DIFF_FILES = 256;
export const MAX_HOST_ARTIFACT_RETURN_BYTES = 1024 * 1024;
export const MAX_HOST_ARTIFACT_BASE64_CHARS = 4 * Math.ceil(MAX_HOST_ARTIFACT_RETURN_BYTES / 3);

export interface ListTasksInput {
    includeArchived?: boolean;
    query?: string;
}

export interface TaskSummary {
    taskId: string;
    projectId: string;
    title: string;
    repo: string | null;
    branch: string | null;
    createdAtMs: number;
    archived: boolean;
    pinned: boolean;
    latestRunId: string | null;
    hold?: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
    unresolvedRunId?: string | null;
    latestRunLoaded?: boolean;
    state:
        | 'idle'
        | 'queued'
        | 'running'
        | 'waiting_for_approval'
        | 'waiting_for_background'
        | 'cancelling'
        | 'finished'
        | 'failed'
        | 'cancelled'
        | 'expired'
        | 'archived'
        | 'restoring';
}

export interface CreateTaskInput {
    projectId?: string;
    title: string;
    repo?: string;
    branch?: string;
}

export interface StartRunInput {
    taskId: string;
    prompt: string;
    mode?: 'plan' | 'agent';
}

export interface RunSummary {
    taskId: string;
    runId: string;
    generation: number;
    state: RunState;
    retryOfRunId: string | null;
}

export type MessageDelivery = 'queue' | 'steer' | 'force';

export interface QueueMessageInput {
    taskId: string;
    text: string;
}

export interface SteerMessageInput {
    taskId: string;
    text: string;
}

export interface ForceSendInput {
    taskId: string;
    text: string;
}

export interface MessageReceipt {
    taskId: string;
    runId: string;
    messageId: string;
    delivery: MessageDelivery;
}

export interface PinTaskInput {
    taskId: string;
    pinned: boolean;
}

export interface ArchiveTaskInput {
    taskId: string;
    archived: boolean;
}

export interface StopInput {
    taskId: string;
    runId: string;
}

export interface StopReceipt {
    taskId: string;
    runId: string;
    acknowledged: boolean;
}

export interface RetryRunInput {
    taskId: string;
    runId: string;
}

export interface ResolveApprovalInput {
    approvalId: string;
    decision: 'approved' | 'denied';
}

export interface ApprovalReceipt {
    approvalId: string;
    decision: 'approved' | 'denied';
}

export interface ResumeTaskInput {
    taskId: string;
    runId: string;
    cursor?: AgentCursor;
}

export interface TaskSnapshot {
    taskId: string;
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string;
    headDigest: string;
}

export interface ResumeTaskResult {
    snapshot: TaskSnapshot;
    event: AgentEvent;
}

export interface SubscribeInput {
    taskId?: string;
    runId?: string;
}

export type TreeSource = 'local' | 'host';

export interface TreeInput {
    source?: TreeSource;
    path?: string;
    grantId?: string;
    taskId?: string;
    runId?: string;
    generation?: number;
}

export interface TreeEntry {
    path: string;
    kind: 'file' | 'directory';
    sizeBytes?: number;
}

export interface TreePage {
    entries: TreeEntry[];
    /** The listing may be incomplete: reported by the host or inferred at the client cap. */
    truncated: boolean;
    /** The host explicitly reported truncation rather than the client inferring it. */
    truncatedReported: boolean;
    symlinksSkipped: number;
    readErrors: number;
}

export interface ChangesSnapshotInput {
    taskId: string;
    runId?: string;
    scope: 'last_turn' | 'branch';
    grantId?: string;
}

export interface ChangesSnapshot {
    scope: 'last_turn' | 'branch';
    files: Array<Record<string, unknown>>;
    headDigest: string;
    capturedAtMs: number;
    /** True when the host bounded the branch result; never a complete claim. */
    truncated?: boolean;
}

export interface ReadArtifactInput {
    taskId?: string;
    artifactId?: string;
    path?: string;
    start?: number;
    end?: number;
}

export interface ArtifactContent {
    /** Full host/browser artifact identity when the source reported it. */
    artifactId?: string;
    path: string;
    mediaType: string;
    /** Number of bytes returned, not necessarily the whole artifact size. */
    sizeBytes: number;
    /** Total artifact size when the source reported it. */
    artifactSizeBytes?: number;
    /** SHA-256 of the returned bytes. */
    digest: string;
    /** SHA-256 of the whole artifact when the source reported it. */
    wholeDigest?: string;
    range?: { start: number; end: number };
    bytes: Uint8Array;
}

export interface HostCapabilities {
    tree: boolean;
    changes: boolean;
    readArtifact: boolean;
    terminal: boolean;
    git: boolean;
    mcp: boolean;
    approvals: boolean;
}

export interface LocalAgentFile {
    path: string;
    mediaType: string;
    bytes: Uint8Array;
    digest: string;
    artifactId?: string;
}

export interface AgentTransport {
    listTasks(input: ListTasksInput): Promise<TaskSummary[]>;
    createTask(input: CreateTaskInput): Promise<TaskSummary>;
    startRun(input: StartRunInput): Promise<RunSummary>;
    queueMessage(input: QueueMessageInput): Promise<MessageReceipt>;
    steer(input: SteerMessageInput): Promise<MessageReceipt>;
    forceSend(input: ForceSendInput): Promise<MessageReceipt>;
    pinTask(input: PinTaskInput): Promise<TaskSummary>;
    archiveTask(input: ArchiveTaskInput): Promise<TaskSummary>;
    stop(input: StopInput): Promise<StopReceipt>;
    retry(input: RetryRunInput): Promise<RunSummary>;
    resolveApproval(input: ResolveApprovalInput): Promise<ApprovalReceipt>;
    resumeTask(input: ResumeTaskInput): Promise<ResumeTaskResult>;
    // digest is the canonical SHA-256 of the event. A transport that cannot
    // supply it must deliver canonical text through ingestWireEvent instead,
    // so the store keeps a verifiable digest chain.
    subscribe(
        listener: (event: AgentIncomingEvent, digest?: string) => void,
        input?: SubscribeInput,
    ): () => void;
    tree(input: TreeInput): Promise<TreeEntry[]>;
    changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null>;
    readArtifact(input: ReadArtifactInput): Promise<ArtifactContent>;
    hostCapabilities(): Promise<HostCapabilities>;
}

export interface AgentEventStream extends AsyncIterable<AgentIncomingEvent> {
    close(): void;
}

class BufferedAgentEventStream implements AgentEventStream {
    private readonly buffer: AgentIncomingEvent[] = [];
    private readonly waiters: Array<
        (result: IteratorResult<AgentIncomingEvent>) => void
    > = [];
    private closed = false;

    constructor(
        private readonly filter: SubscribeInput,
        private readonly onClose: () => void,
    ) {}

    push(event: AgentIncomingEvent): void {
        if (this.closed) return;
        if (
            this.filter.taskId !== undefined &&
            this.filter.taskId !== event.task_id
        ) {
            return;
        }
        if (
            this.filter.runId !== undefined &&
            this.filter.runId !== event.run_id
        ) {
            return;
        }
        const waiter = this.waiters.shift();
        if (waiter !== undefined) {
            waiter({ value: event, done: false });
            return;
        }
        this.buffer.push(event);
    }

    close(): void {
        if (this.closed) return;
        this.closed = true;
        for (const waiter of this.waiters.splice(0)) {
            waiter({ value: undefined as never, done: true });
        }
        this.onClose();
    }

    next(): Promise<IteratorResult<AgentIncomingEvent>> {
        const buffered = this.buffer.shift();
        if (buffered !== undefined) {
            return Promise.resolve({ value: buffered, done: false });
        }
        if (this.closed) {
            return Promise.resolve({ value: undefined as never, done: true });
        }
        return new Promise((resolve) => {
            this.waiters.push(resolve);
        });
    }

    return(): Promise<IteratorResult<AgentIncomingEvent>> {
        this.close();
        return Promise.resolve({ value: undefined as never, done: true });
    }

    [Symbol.asyncIterator](): AsyncIterator<AgentIncomingEvent> {
        return this;
    }
}

export interface AgentClientOptions {
    transport?: AgentTransport | null;
    runtime?: AgentRuntime | null;
    hostClient?: HostClient | null;
    store?: AgentStore;
    taskStore?: AgentTaskStore | null;
}

export class AgentClient {
    readonly store: AgentStore;

    private readonly transport: AgentTransport | null;
    private readonly taskStore: AgentTaskStore | null;
    private readonly runtime: AgentRuntime | null;
    private readonly hostClient: HostClient;
    private readonly explicitHostClient: boolean;
    private readonly ownsHostClient: boolean;
    private readonly streams = new Set<BufferedAgentEventStream>();
    private readonly localFiles = new Map<string, LocalAgentFile>();
    private readonly localArtifactIds = new Map<string, string>();
    private readonly unsubscribeTransport: (() => void) | null;
    private durableQueue: Promise<void> = Promise.resolve();
    private persistenceError: AgentClientError | null = null;

    constructor(options: AgentClientOptions = {}) {
        if (options.runtime != null && options.transport != null) {
            throw new AgentClientError(
                'invalid_arguments',
                'inject AgentRuntime or AgentTransport, not both send paths',
            );
        }
        if (
            options.runtime != null &&
            (options.store !== undefined || options.taskStore !== undefined)
        ) {
            throw new AgentClientError(
                'invalid_arguments',
                'an AgentRuntime owns its store and task adapter; do not inject a second authority',
            );
        }
        this.runtime = options.runtime ?? null;
        if (this.runtime !== null) {
            if (
                options.hostClient !== undefined &&
                options.hostClient !== this.runtime.hostClient
            ) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'the runtime owns its host client; do not inject a second host path',
                );
            }
            this.store = this.runtime.store;
            this.transport = this.runtime as unknown as AgentTransport;
            this.taskStore = null;
            this.explicitHostClient = false;
            this.ownsHostClient = false;
            this.hostClient =
                options.hostClient ??
                this.runtime.hostClient ??
                new HostClient({ transport: new NullHostTransport() });
            this.unsubscribeTransport = this.runtime.subscribe((event) => {
                // The runtime publishes only events it has already applied
                // through the shared AgentStore. Re-applying here returns
                // duplicate/ignored and would suppress every client stream.
                this.publish(event);
            });
            return;
        }
        this.transport = options.transport ?? null;
        this.store = options.store ?? createAgentStore();
        this.taskStore = options.taskStore ?? null;
        this.explicitHostClient =
            options.hostClient !== undefined && options.hostClient !== null;
        this.ownsHostClient = options.hostClient == null;
        this.hostClient =
            options.hostClient ??
            (this.transport === null
                ? new HostClient({ transport: new NullHostTransport() })
                : new HostClient({
                      transport: new AgentTransportHostTransport(
                          this.transport,
                      ),
                  }));
        this.unsubscribeTransport =
            this.transport === null
                ? null
                : this.transport.subscribe((event, digest) => {
                      if (this.taskStore === null) {
                          const result = this.store.apply(event, digest);
                          if (result.status === 'applied') {
                              this.publish(event);
                          }
                          return;
                      }
                      this.durableQueue = this.durableQueue.then(async () => {
                          try {
                              const result = await this.taskStore!.appendEvent({
                                  event,
                                  digest,
                                  writer_id: this.taskStore!.defaultWriterId,
                              });
                              if (result.status === 'applied') {
                                  this.persistenceError = null;
                                  this.publish(event);
                              } else if (result.status === 'rejected') {
                                  this.persistenceError = new AgentClientError(
                                      result.code ?? 'internal',
                                      result.message ?? 'durable append was rejected',
                                      true,
                                      result.storage,
                                  );
                              }
                          } catch (error) {
                              this.persistenceError = toClientError(error);
                          }
                      });
                  });
    }

    get lastPersistenceError(): AgentClientError | null {
        return this.persistenceError ?? this.runtime?.lastPersistenceError ?? null;
    }

    // Replays the durable log into the same AgentStore the UI reads. Callers
    // must await this before any persistence-backed operation.
    async open(): Promise<void> {
        if (this.runtime !== null) {
            try {
                await this.runtime.open();
            } catch (error) {
                throw toClientError(error);
            }
            return;
        }
        if (this.taskStore === null) return;
        try {
            await this.taskStore.open();
        } catch (error) {
            throw toClientError(error);
        }
    }

    async listTasks(input: ListTasksInput = {}): Promise<TaskSummary[]> {
        const durable = this.requireReady();
        const tasks = await this.requireTransport().listTasks(input);
        for (const task of tasks) {
            const record = this.persistedTaskRecord(task);
            if (durable !== null) await this.persistTask(record);
            else this.store.upsertTask(record);
        }
        const snapshot = this.store.snapshot();
        return tasks.map((task) => {
            const view = snapshot.tasks.find(
                (candidate) => candidate.taskId === task.taskId,
            );
            if (view === undefined) {
                return task;
            }
            const unresolved = view.unresolvedRunId;
            return {
                ...task,
                state: unresolved === null ? view.state : task.state,
                latestRunId: unresolved ?? view.latestRunId,
                unresolvedRunId: unresolved,
                latestRunLoaded:
                    view.latestRunId === null
                        ? true
                        : view.runs.some(
                              (run) => run.runId === view.latestRunId,
                          ),
                archived: view.archived,
                pinned: view.pinnedLocal,
            };
        });
    }

    async createTask(input: CreateTaskInput): Promise<TaskSummary> {
        const durable = this.requireReady();
        const task = await this.requireTransport().createTask(input);
        const record = this.persistedTaskRecord(task);
        if (durable !== null) await this.persistTask(record);
        else this.store.upsertTask(record);
        return task;
    }

    async startRun(input: StartRunInput): Promise<RunSummary> {
        this.requireReady();
        this.requireResolvedTask(input.taskId);
        return this.requireTransport().startRun(input);
    }

    async queueMessage(input: QueueMessageInput): Promise<MessageReceipt> {
        this.requireReady();
        this.requireResolvedTask(input.taskId);
        return this.requireTransport().queueMessage(input);
    }

    async steer(input: SteerMessageInput): Promise<MessageReceipt> {
        this.requireReady();
        this.requireResolvedTask(input.taskId);
        return this.requireTransport().steer(input);
    }

    async forceSend(input: ForceSendInput): Promise<MessageReceipt> {
        this.requireReady();
        this.requireResolvedTask(input.taskId);
        return this.requireTransport().forceSend(input);
    }

    async pinTask(input: PinTaskInput): Promise<TaskSummary> {
        const durable = this.requireReady();
        const task = await this.requireTransport().pinTask(input);
        const record = this.persistedTaskRecord(task);
        if (durable !== null) await this.persistTask(record);
        else this.store.upsertTask(record);
        return task;
    }

    async archiveTask(input: ArchiveTaskInput): Promise<TaskSummary> {
        const durable = this.requireReady();
        const task = await this.requireTransport().archiveTask(input);
        const record = this.persistedTaskRecord(task);
        if (durable !== null) await this.persistTask(record);
        else this.store.upsertTask(record);
        return task;
    }

    async stop(input: StopInput): Promise<StopReceipt> {
        this.requireReady();
        this.requireResolvedTask(input.taskId);
        return this.requireTransport().stop(input);
    }

    async retry(input: RetryRunInput): Promise<RunSummary> {
        this.requireReady();
        this.requireResolvedTask(input.taskId);
        return this.requireTransport().retry(input);
    }

    async resolveApproval(
        input: ResolveApprovalInput,
    ): Promise<ApprovalReceipt> {
        this.requireReady();
        if (input.decision !== 'approved' && input.decision !== 'denied') {
            throw new AgentClientError(
                'invalid_arguments',
                'approval decision must be approved | denied',
            );
        }
        // The page sends exactly these two fields. Granted/path/command/scope
        // or any page-held key never crosses this boundary.
        const payload: ResolveApprovalInput = {
            approvalId: input.approvalId,
            decision: input.decision,
        };
        if (this.runtime !== null) {
            return this.runtime.resolveApproval(payload);
        }
        if (this.transport !== null) {
            return this.transport.resolveApproval(payload);
        }
        if (this.explicitHostClient) {
            return this.hostClient.approve(payload);
        }
        return this.requireTransport().resolveApproval(payload);
    }

    async resumeTask(input: ResumeTaskInput): Promise<TaskSnapshot> {
        this.requireReady();
        if (this.runtime !== null) {
            const result = await this.runtime.resumeTask(input);
            return result.snapshot;
        }
        const transport = this.requireTransport();
        const result = await transport.resumeTask(input);
        const verification = this.store.verifySnapshot(result.snapshot);
        if (!verification.ok) {
            throw new AgentClientError(
                verification.code ?? 'snapshot_corrupt',
                verification.message ?? 'snapshot verification failed',
            );
        }
        const appended = await this.appendThroughStore(result.event, undefined, {
            snapshotAnchor: result.snapshot,
        });
        if (appended.status === 'rejected') {
            throw new AgentClientError(
                appended.code ?? 'internal',
                appended.message ?? 'resume event rejected',
                false,
                appended.storage ?? null,
            );
        }
        return result.snapshot;
    }

    subscribe(input: SubscribeInput = {}): AgentEventStream {
        this.requireReady();
        this.requireTransport();
        let stream: BufferedAgentEventStream;
        stream = new BufferedAgentEventStream(input, () => {
            this.streams.delete(stream);
        });
        this.streams.add(stream);
        return stream;
    }

    async ingestWireEvent(text: string): Promise<AgentAppendResult> {
        this.requireReady();
        if (this.runtime !== null) {
            return this.runtime.ingestWireEvent(text);
        }
        const event = parseAgentEventWire(text);
        return this.appendThroughStore(event);
    }

    registerLocalFile(file: LocalAgentFile): void {
        if (
            file.path.length === 0 ||
            file.path.startsWith('/') ||
            file.path.split('/').includes('..')
        ) {
            throw new AgentClientError(
                'invalid_arguments',
                'local file path must be a relative path without parent segments',
            );
        }
        if (!DIGEST_PATTERN.test(file.digest)) {
            throw new AgentClientError(
                'invalid_arguments',
                'local file digest must be sha256:<64 lowercase hex>',
            );
        }
        this.localFiles.set(file.path, {
            path: file.path,
            mediaType: file.mediaType,
            bytes: new Uint8Array(file.bytes),
            digest: file.digest,
            artifactId: file.artifactId,
        });
        if (file.artifactId !== undefined) {
            this.localArtifactIds.set(file.artifactId, file.path);
        }
    }

    async tree(input: TreeInput = {}): Promise<TreeEntry[]> {
        return (await this.treePage(input)).entries;
    }

    async treePage(input: TreeInput = {}): Promise<TreePage> {
        const source = input.source ?? 'local';
        if (source === 'local') {
            return {
                entries: this.localTree(input.path),
                truncated: false,
                truncatedReported: false,
                symlinksSkipped: 0,
                readErrors: 0,
            };
        }
        const result = await this.hostClient.execute({
            tool: 'file.tree',
            arguments: {
                ...(input.path === undefined ? {} : { path: input.path }),
                ...(input.grantId === undefined
                    ? {}
                    : { grant_id: input.grantId }),
            },
            ...(input.taskId === undefined ? {} : { taskId: input.taskId }),
            ...(input.runId === undefined ? {} : { runId: input.runId }),
            ...(input.generation === undefined
                ? {}
                : { generation: input.generation }),
        });
        return hostTreePage(result);
    }

    async changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null> {
        if (input.scope === 'last_turn') {
            const local: AgentChangeSnapshotView | null =
                this.store.latestChangesSnapshot(
                    input.taskId,
                    input.runId,
                    'last_turn',
                );
            if (local === null) return null;
            return {
                scope: 'last_turn',
                files: local.files,
                headDigest: local.headDigest,
                capturedAtMs: local.capturedAtMs,
            };
        }
        const result = await this.hostClient.execute({
            tool: 'git.changes',
            arguments: {
                scope: 'branch',
                ...(input.grantId === undefined
                    ? {}
                    : { grant_id: input.grantId }),
            },
            taskId: input.taskId,
            ...(input.runId === undefined ? {} : { runId: input.runId }),
        });
        return hostChangesSnapshot(result);
    }

    async addDiffComment(input: AddDiffCommentInput): Promise<DiffCommentRecord> {
        const taskStore = this.requireReady();
        if (this.runtime !== null) return this.runtime.addDiffComment(input);
        if (taskStore !== null) {
            const result = await taskStore.appendDiffComment({
                task_id: input.taskId,
                path: input.path,
                line: input.line,
                body: input.body,
                head_digest: input.headDigest,
                scope: input.scope,
                ...(input.commentId === undefined
                    ? {}
                    : { comment_id: input.commentId }),
                ...(input.createdAtMs === undefined
                    ? {}
                    : { created_at_ms: input.createdAtMs }),
            });
            return result.comment;
        }
        throw new AgentClientError(
            'runtime_unavailable',
            'durable local comments require the browser task store',
        );
    }

    async readDiffComments(taskId: string): Promise<DiffCommentRecord[]> {
        const taskStore = this.requireReady();
        if (this.runtime !== null) return this.runtime.readDiffComments(taskId);
        if (taskStore !== null) return taskStore.readDiffComments(taskId);
        throw new AgentClientError(
            'runtime_unavailable',
            'durable local comments require the browser task store',
        );
    }

    async readArtifact(input: ReadArtifactInput): Promise<ArtifactContent> {
        const range = artifactRange(input);
        const local = this.localArtifact(input);
        if (local !== null) {
            if (range === undefined) {
                return local;
            }
            if (range.end > local.bytes.byteLength) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'artifact range is outside the local artifact',
                );
            }
            if (range.end - range.start > MAX_HOST_ARTIFACT_RETURN_BYTES) {
                throw new AgentClientError(
                    'limit_exceeded',
                    'local artifact range exceeds the client return cap',
                );
            }
            const bytes = local.bytes.slice(range.start, range.end);
            return {
                ...(local.artifactId === undefined
                    ? {}
                    : { artifactId: local.artifactId }),
                path: local.path,
                mediaType: local.mediaType,
                sizeBytes: bytes.byteLength,
                artifactSizeBytes: local.bytes.byteLength,
                digest: await sha256Bytes(bytes),
                wholeDigest: local.digest,
                range,
                bytes,
            };
        }
        if (input.artifactId !== undefined) {
            try {
                if (this.runtime !== null) {
                    const content = await this.runtime.readArtifact(input);
                    if (
                        content.artifactId !== undefined &&
                        content.artifactId !== input.artifactId
                    ) {
                        throw new AgentClientError(
                            'invalid_event',
                            'browser artifact identity does not match the requested artifact',
                        );
                    }
                    return content.artifactId === undefined
                        ? { ...content, artifactId: input.artifactId }
                        : content;
                }
                if (this.taskStore !== null) {
                    const durable = await this.taskStore.readArtifact(
                        input.artifactId,
                        range,
                    );
                    if (durable !== null) {
                        return {
                            artifactId: durable.artifact_id,
                            path: durable.artifact_id,
                            mediaType: durable.media_type,
                            sizeBytes: durable.size_bytes,
                            artifactSizeBytes: durable.artifact_size_bytes,
                            digest: durable.digest,
                            wholeDigest: durable.whole_digest,
                            range: durable.range,
                            bytes: durable.bytes,
                        };
                    }
                }
            } catch (error) {
                const clientError = toClientError(error);
                if (
                    clientError.code !== 'host_unavailable' &&
                    clientError.code !== 'runtime_unavailable'
                ) {
                    throw clientError;
                }
            }
        }
        const result = await this.hostClient.execute({
            tool: 'artifact.read',
            arguments: {
                ...(input.path === undefined ? {} : { path: input.path }),
                ...(input.artifactId === undefined
                    ? {}
                    : { artifact_id: input.artifactId }),
                ...(range === undefined
                    ? {}
                    : { start: range.start, end: range.end }),
            },
            ...(input.taskId === undefined ? {} : { taskId: input.taskId }),
        });
        return artifactIdentity(
            await hostArtifactContent(result),
            input.artifactId,
        );
    }

    async hostCapabilities(): Promise<HostCapabilities> {
        if (
            this.runtime === null &&
            this.transport === null &&
            this.explicitHostClient
        ) {
            return this.hostClient.capabilities();
        }
        this.requireTransport();
        return this.hostClient.capabilities();
    }

    /** Host-approved folders through the existing host client. */
    async grants(): Promise<HostGrant[]> {
        return this.hostClient.grants();
    }

    /** Whether the host transport can supply approved-folder grants at all. */
    hostSupportsGrants(): boolean {
        return this.hostClient.supportsGrants();
    }

    /** Pending approvals replayed from the verified log, not a new authority. */
    async pendingApprovals(taskId?: string): Promise<RestoredApproval[]> {
        if (this.runtime === null) return [];
        return this.runtime.pendingApprovals(taskId);
    }

    /** True only while a live active control can resolve this approval. */
    approvalResolvable(approvalId: string): boolean {
        return this.runtime?.approvalResolvable(approvalId) ?? false;
    }

    async dispose(): Promise<void> {
        for (const stream of [...this.streams]) {
            stream.close();
        }
        this.unsubscribeTransport?.();
        if (this.runtime !== null) {
            try {
                await this.runtime.dispose();
            } catch (error) {
                this.persistenceError = toClientError(error);
            }
            return;
        }
        try {
            if (this.ownsHostClient) {
                this.hostClient.close();
            }
        } finally {
            this.taskStore?.close();
        }
    }

    private requireResolvedTask(taskId: string): void {
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === taskId,
        );
        if (task?.unresolvedRunId !== null && task?.unresolvedRunId !== undefined) {
            throw new AgentClientError(
                'out_of_order',
                'the task latest run is still loading; replays must complete before starting or steering',
            );
        }
    }

    private requireReady(): AgentTaskStore | null {
        if (this.runtime !== null) {
            if (!this.runtime.isOpen) {
                throw new AgentClientError(
                    'runtime_unavailable',
                    'browser runtime is not open; await AgentClient.open() first',
                );
            }
            return null;
        }
        if (this.taskStore === null) return null;
        if (!this.taskStore.isOpen) {
            throw new AgentClientError(
                'runtime_unavailable',
                'durable task store is not open; await AgentClient.open() first',
            );
        }
        return this.taskStore;
    }

    private async persistTask(record: AgentTaskRecord): Promise<void> {
        if (this.taskStore === null) return;
        try {
            await this.taskStore.writeTaskRecord({
                task_id: record.taskId,
                project_id: record.projectId,
                title: record.title,
                ...(record.repo === null || record.repo === undefined
                    ? {}
                    : { repo: record.repo }),
                ...(record.branch === null || record.branch === undefined
                    ? {}
                    : { branch: record.branch }),
                created_at_ms: record.createdAtMs,
                archived: record.archived ?? false,
                pinned_local: record.pinnedLocal ?? false,
                latest_run_id: record.latestRunId ?? null,
                ...(record.hold === undefined ? {} : { hold: record.hold }),
            });
        } catch (error) {
            throw toClientError(error);
        }
    }

    private async appendThroughStore(
        event: AgentIncomingEvent,
        digest?: string,
        options: { snapshotAnchor?: TaskSnapshot } = {},
    ): Promise<AgentAppendResult> {
        if (this.taskStore !== null) {
            const anchor = options.snapshotAnchor;
            const result = await this.taskStore.appendEvent({
                event,
                digest,
                writer_id: this.taskStore.defaultWriterId,
                ...(anchor === undefined
                    ? {}
                    : {
                          snapshot_anchor: {
                              task_id: anchor.taskId,
                              run_id: anchor.runId,
                              generation: anchor.generation,
                              last_sequence: anchor.lastSequence,
                              last_event_id: anchor.lastEventId,
                              head_digest: anchor.headDigest,
                          },
                      }),
            });
            if (result.status === 'applied') {
                this.persistenceError = null;
                this.publish(event);
            } else if (result.status === 'rejected') {
                this.persistenceError = new AgentClientError(
                    result.code ?? 'internal',
                    result.message ?? 'durable append was rejected',
                    true,
                    result.storage,
                );
            }
            return {
                status: result.status,
                code: result.code,
                message: result.message,
                requiresResync: result.status === 'rejected',
                eventId: result.event_id,
                digest: result.digest,
                storage: result.storage,
            };
        }
        const result = await this.store.appendEvent(event);
        if (result.status === 'applied') {
            this.persistenceError = null;
            this.publish(event);
        } else if (result.status === 'rejected') {
            this.persistenceError = new AgentClientError(
                result.code ?? 'internal',
                result.message ?? 'agent append was rejected',
            );
        }
        return result;
    }

    private requireTransport(): AgentTransport {
        if (this.transport === null) {
            throw new AgentClientError(
                'runtime_unavailable',
                'the browser task runtime is not initialized',
            );
        }
        return this.transport;
    }

    private publish(event: AgentIncomingEvent): void {
        for (const stream of this.streams) {
            stream.push(event);
        }
    }

    private persistedTaskRecord(task: TaskSummary): AgentTaskRecord {
        const record = this.toTaskRecord(task);
        const existing = this.store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === task.taskId);
        if (
            existing !== undefined &&
            existing.runs.length > 0 &&
            existing.latestRunId !== null
        ) {
            record.latestRunId = existing.latestRunId;
        }
        return record;
    }

    private toTaskRecord(task: TaskSummary): AgentTaskRecord {
        return {
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            repo: task.repo,
            branch: task.branch,
            createdAtMs: task.createdAtMs,
            archived: task.archived,
            pinnedLocal: task.pinned,
            latestRunId: task.latestRunId,
            ...(task.hold === undefined ? {} : { hold: task.hold }),
        };
    }

    private localTree(path: string | undefined): TreeEntry[] {
        const prefix = (path ?? '').replace(/\/+$/, '');
        const entries = new Map<string, TreeEntry>();
        for (const file of this.localFiles.values()) {
            if (
                prefix.length > 0 &&
                file.path !== prefix &&
                !file.path.startsWith(`${prefix}/`)
            ) {
                continue;
            }
            const segments = file.path.split('/').filter((segment) => segment.length > 0);
            let current = '';
            for (let index = 0; index < segments.length - 1; index += 1) {
                current =
                    current.length === 0
                        ? segments[index]!
                        : `${current}/${segments[index]}`;
                entries.set(current, { path: current, kind: 'directory' });
            }
            entries.set(file.path, {
                path: file.path,
                kind: 'file',
                sizeBytes: file.bytes.byteLength,
            });
        }
        return [...entries.values()].sort((left, right) =>
            left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
        );
    }

    private localArtifact(input: ReadArtifactInput): ArtifactContent | null {
        let path = input.path;
        if (path === undefined && input.artifactId !== undefined) {
            path = this.localArtifactIds.get(input.artifactId);
        }
        if (path === undefined) {
            if (input.path === undefined && input.artifactId === undefined) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'readArtifact requires path or artifactId',
                );
            }
            return null;
        }
        const file = this.localFiles.get(path);
        if (file === undefined) {
            return null;
        }
        return {
            ...(file.artifactId === undefined
                ? {}
                : { artifactId: file.artifactId }),
            path: file.path,
            mediaType: file.mediaType,
            sizeBytes: file.bytes.byteLength,
            artifactSizeBytes: file.bytes.byteLength,
            digest: file.digest,
            wholeDigest: file.digest,
            bytes: new Uint8Array(file.bytes),
        };
    }
}

function invalidHostData(message: string): never {
    throw new AgentClientError('invalid_event', message);
}

function artifactRange(
    input: ReadArtifactInput,
): { start: number; end: number } | undefined {
    if (input.start === undefined && input.end === undefined) return undefined;
    if (input.start === undefined || input.end === undefined) {
        throw new AgentClientError(
            'invalid_arguments',
            'artifact range requires both start and end',
        );
    }
    if (
        !Number.isSafeInteger(input.start) ||
        !Number.isSafeInteger(input.end) ||
        input.start < 0 ||
        input.start > input.end
    ) {
        throw new AgentClientError(
            'invalid_arguments',
            'artifact range is invalid',
        );
    }
    return { start: input.start, end: input.end };
}

function hostTreeEntries(values: unknown[]): TreeEntry[] {
    if (values.length > MAX_HOST_TREE_ENTRIES) {
        return invalidHostData('host file tree result exceeds the client cap');
    }
    return values.map((value) => {
        if (
            !isPlainObject(value) ||
            typeof value.path !== 'string' ||
            (value.kind !== 'file' && value.kind !== 'directory')
        ) {
            return invalidHostData('host file tree entry is malformed');
        }
        if (value.size_bytes === undefined) {
            return { path: value.path, kind: value.kind };
        }
        if (
            typeof value.size_bytes !== 'number' ||
            !Number.isSafeInteger(value.size_bytes) ||
            value.size_bytes < 0
        ) {
            return invalidHostData('host file tree entry size is malformed');
        }
        return {
            path: value.path,
            kind: value.kind,
            sizeBytes: value.size_bytes,
        };
    });
}

function hostTreePage(result: HostToolResult): TreePage {
    const output = result.output;
    if (!isPlainObject(output) || !Array.isArray(output.entries)) {
        return invalidHostData('host file tree result has no entries array');
    }
    const entries = hostTreeEntries(output.entries);
    const reported = hostTreeFlag(output.truncated);
    return {
        entries,
        truncated: reported ?? entries.length >= MAX_HOST_TREE_ENTRIES,
        truncatedReported: reported === true,
        symlinksSkipped: hostTreeCount(output.symlinks_skipped, 'symlinks_skipped'),
        readErrors: hostTreeCount(output.read_errors, 'read_errors'),
    };
}

function hostTreeFlag(value: unknown): boolean | undefined {
    if (value === undefined) return undefined;
    if (typeof value !== 'boolean') {
        return invalidHostData('host file tree truncated flag is malformed');
    }
    return value;
}

function hostTreeCount(value: unknown, field: string): number {
    if (value === undefined) return 0;
    if (
        typeof value !== 'number' ||
        !Number.isSafeInteger(value) ||
        value < 0
    ) {
        return invalidHostData(`host file tree ${field} count is malformed`);
    }
    return value;
}

function hostChangesSnapshot(
    result: HostToolResult,
): ChangesSnapshot | null {
    if (!isPlainObject(result.output) || !('snapshot' in result.output)) {
        return invalidHostData('host branch result has no snapshot');
    }
    const snapshot = result.output.snapshot;
    if (snapshot === null) return null;
    if (
        !isPlainObject(snapshot) ||
        snapshot.scope !== 'branch' ||
        !Array.isArray(snapshot.files) ||
        typeof snapshot.head_digest !== 'string' ||
        !DIGEST_PATTERN.test(snapshot.head_digest) ||
        typeof snapshot.captured_at_ms !== 'number' ||
        !Number.isSafeInteger(snapshot.captured_at_ms) ||
        snapshot.captured_at_ms < 0
    ) {
        return invalidHostData('host branch snapshot is malformed');
    }
    if (snapshot.files.length > MAX_HOST_DIFF_FILES) {
        return invalidHostData('host branch snapshot exceeds the client cap');
    }
    if (
        snapshot.truncated !== undefined &&
        typeof snapshot.truncated !== 'boolean'
    ) {
        return invalidHostData('host branch truncated flag is malformed');
    }
    const files = snapshot.files.map((value) => {
        if (
            !isPlainObject(value) ||
            typeof value.path !== 'string' ||
            value.path.length === 0
        ) {
            return invalidHostData('host branch file record is malformed');
        }
        return value;
    });
    return {
        scope: 'branch',
        files,
        headDigest: snapshot.head_digest,
        capturedAtMs: snapshot.captured_at_ms,
        ...(snapshot.truncated === true ? { truncated: true } : {}),
    };
}

async function hostArtifactContent(
    result: HostToolResult,
): Promise<ArtifactContent> {
    if (!isPlainObject(result.output)) {
        return invalidHostData('host artifact result has no output');
    }
    const output = result.output;
    for (const key of [
        'url',
        'artifact_url',
        'download_url',
        'link',
        'token',
        'expires_at',
    ]) {
        if (key in output) {
            return invalidHostData(
                'host artifact result must not carry a URL, token or expiring link',
            );
        }
    }
    let artifactId: string | undefined;
    if (output.artifact_id !== undefined) {
        if (
            typeof output.artifact_id !== 'string' ||
            !ARTIFACT_ID_PATTERN.test(output.artifact_id)
        ) {
            return invalidHostData('host artifact identity is malformed');
        }
        artifactId = output.artifact_id;
    }
    if (
        typeof output.path !== 'string' ||
        output.path.length === 0 ||
        typeof output.media_type !== 'string' ||
        output.media_type.length === 0 ||
        typeof output.digest !== 'string' ||
        !DIGEST_PATTERN.test(output.digest) ||
        !Number.isSafeInteger(output.size_bytes) ||
        (output.size_bytes as number) < 0 ||
        typeof output.bytes_base64 !== 'string'
    ) {
        return invalidHostData('host artifact result is malformed');
    }
    if ((output.size_bytes as number) > MAX_HOST_ARTIFACT_RETURN_BYTES) {
        return invalidHostData('host artifact size exceeds the client cap');
    }
    if (output.bytes_base64.length > MAX_HOST_ARTIFACT_BASE64_CHARS) {
        return invalidHostData('host artifact base64 exceeds the client cap');
    }
    let bytes: Uint8Array;
    try {
        const binary = globalThis.atob(output.bytes_base64);
        bytes = new Uint8Array(binary.length);
        for (let index = 0; index < binary.length; index += 1) {
            bytes[index] = binary.charCodeAt(index);
        }
    } catch {
        return invalidHostData('host artifact bytes are not base64');
    }
    if (bytes.byteLength !== output.size_bytes) {
        return invalidHostData('host artifact size does not match its bytes');
    }
    if ((await sha256Bytes(bytes)) !== output.digest) {
        return invalidHostData('host artifact digest does not match its bytes');
    }
    let rangeValue: { start: number; end: number } | undefined;
    let artifactSizeBytes: number | undefined;
    if (output.range !== undefined) {
        const range = output.range;
        if (
            !isPlainObject(range) ||
            typeof range.start !== 'number' ||
            !Number.isSafeInteger(range.start) ||
            range.start < 0 ||
            typeof range.end !== 'number' ||
            !Number.isSafeInteger(range.end) ||
            range.end < range.start ||
            range.end - range.start !== bytes.byteLength
        ) {
            return invalidHostData('host artifact range is malformed');
        }
        if (
            typeof output.artifact_size_bytes !== 'number' ||
            !Number.isSafeInteger(output.artifact_size_bytes) ||
            output.artifact_size_bytes < range.end
        ) {
            return invalidHostData('host artifact total size is malformed');
        }
        rangeValue = { start: range.start, end: range.end };
        artifactSizeBytes = output.artifact_size_bytes;
    } else if (output.artifact_size_bytes !== undefined) {
        if (
            typeof output.artifact_size_bytes !== 'number' ||
            !Number.isSafeInteger(output.artifact_size_bytes) ||
            output.artifact_size_bytes !== output.size_bytes
        ) {
            return invalidHostData('host whole artifact total size is malformed');
        }
        artifactSizeBytes = output.artifact_size_bytes;
    }
    let wholeDigest: string | undefined;
    if (output.whole_digest !== undefined) {
        if (
            typeof output.whole_digest !== 'string' ||
            !DIGEST_PATTERN.test(output.whole_digest)
        ) {
            return invalidHostData('host artifact whole digest is malformed');
        }
        wholeDigest = output.whole_digest;
    }
    if (
        rangeValue === undefined &&
        wholeDigest !== undefined &&
        wholeDigest !== output.digest
    ) {
        return invalidHostData(
            'host full artifact whole digest does not match its bytes',
        );
    }
    if (
        rangeValue !== undefined &&
        artifactSizeBytes !== undefined &&
        rangeValue.start === 0 &&
        rangeValue.end === artifactSizeBytes &&
        (wholeDigest === undefined || wholeDigest !== output.digest)
    ) {
        return invalidHostData(
            'host full artifact whole digest does not match its bytes',
        );
    }
    return {
        ...(artifactId === undefined ? {} : { artifactId }),
        path: output.path,
        mediaType: output.media_type,
        sizeBytes: bytes.byteLength,
        digest: output.digest,
        ...(artifactSizeBytes === undefined ? {} : { artifactSizeBytes }),
        ...(wholeDigest === undefined ? {} : { wholeDigest }),
        ...(rangeValue === undefined ? {} : { range: rangeValue }),
        bytes,
    };
}

function artifactIdentity(
    content: ArtifactContent,
    requested: string | undefined,
): ArtifactContent {
    if (requested === undefined || content.artifactId === undefined) {
        return content;
    }
    if (content.artifactId !== requested) {
        throw new AgentClientError(
            'invalid_event',
            'host artifact identity does not match the requested artifact',
        );
    }
    return content;
}
