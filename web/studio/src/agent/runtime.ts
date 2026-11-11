import {
    AGENT_EVENT_SCHEMA,
    APPROVAL_TTL_MS,
    TERMINAL_AGENT_EVENT_TYPES,
    type AgentErrorCode,
    type AgentEvent,
    type RunState,
} from './generated/agentContract.js';
import { canonicalJson } from './generated/agentCanonical.js';
import {
    HostClient,
    canonicalToolArguments,
    sha256Canonical,
} from './hostClient.js';
import {
    type AgentEventStream,
    type ApprovalReceipt,
    type ArchiveTaskInput,
    type ArtifactContent,
    type ChangesSnapshot,
    type ChangesSnapshotInput,
    type CreateTaskInput,
    type ForceSendInput,
    type HostCapabilities,
    type ListTasksInput,
    type MessageDelivery,
    type MessageReceipt,
    type PinTaskInput,
    type QueueMessageInput,
    type ReadArtifactInput,
    type ResolveApprovalInput,
    type ResumeTaskInput,
    type ResumeTaskResult,
    type RetryRunInput,
    type RunSummary,
    type StartRunInput,
    type SteerMessageInput,
    type StopInput,
    type StopReceipt,
    type SubscribeInput,
    type TaskSnapshot,
    type TaskSummary,
    type TreeEntry,
    type TreeInput,
} from './client.js';
import type { AgentTaskStore, DiffCommentRecord } from './store.js';
import {
    AgentClientError,
    AgentStoreError,
    isAgentErrorCode,
    isPlainObject,
    LEASE_STALE_MS,
    toClientError,
} from './storeTypes.js';
import {
    createAgentStore,
    parseAgentEventWire,
    type AgentAppendResult,
    type AgentApplyResult,
    type AgentIncomingEvent,
    type AgentStore,
    type RunView,
    type TaskView,
} from '../state/agent.js';
import {
    ReplayModelPort,
    type ModelFinishReason,
    type ModelPort,
    type ModelRequest,
    type ModelStreamEvent,
} from './modelPort.js';
import { IngestOutcome, NdjsonIngest } from './ingest.js';

const PROJECT_ID_PATTERN = /^prj_[0-9a-f]{16}$/;
const RUN_ID_PATTERN = /^run_[0-9a-f]{16}$/;
const TERMINAL_RUN_STATES: ReadonlySet<RunState> = new Set([
    'finished',
    'failed',
    'cancelled',
    'expired',
]);
const TERMINAL_EVENT_TYPES: ReadonlySet<string> = new Set(
    TERMINAL_AGENT_EVENT_TYPES,
);
const DEFAULT_PROJECT_ID = 'prj_0000000000000001';
const DEFAULT_STOP_ACK_TIMEOUT_MS = 30_000;
const DEFAULT_BOUNDARY_WAIT_MS = 30_000;
const DEFAULT_MAX_QUEUED_MESSAGES = 100;
const MAX_PROMPT_HISTORY = 128;

export { LEASE_STALE_MS };

export function isLeaseStale(
    nowMs: number,
    heartbeatAtMs: number,
): boolean {
    return nowMs - heartbeatAtMs > LEASE_STALE_MS;
}

export interface RuntimeTimers {
    setTimeout(handler: () => void, timeoutMs: number): unknown;
    clearTimeout(handle: unknown): void;
    setInterval(handler: () => void, timeoutMs: number): unknown;
    clearInterval(handle: unknown): void;
}

export interface RuntimeIdFactory {
    taskId(): string;
    runId(): string;
    messageId(): string;
    approvalId(): string;
}

export interface RuntimeToolIntent {
    taskId: string;
    runId: string;
    generation: number;
    toolCallId: string;
    tool: string;
    arguments: Record<string, unknown>;
    approvalId?: string;
}

export interface RuntimeToolResult {
    status: 'ok' | 'error' | 'cancelled';
    output?: unknown;
    files?: Array<Record<string, unknown>>;
    head_digest?: string;
    code?: string;
    message?: string;
    truncated?: boolean;
}

export type RuntimeToolExecutor = (
    intent: RuntimeToolIntent,
) => Promise<RuntimeToolResult>;

export interface AgentRuntimeOptions {
    store?: AgentStore;
    taskStore?: AgentTaskStore | null;
    modelPort?: ModelPort;
    toolExecutor?: RuntimeToolExecutor;
    hostClient?: HostClient | null;
    now?: () => number;
    timers?: RuntimeTimers;
    ids?: RuntimeIdFactory;
    writerId?: string;
    stopAckTimeoutMs?: number;
    boundaryWaitMs?: number;
    approvalTtlMs?: number;
    leaseHeartbeatMs?: number;
    maxQueuedMessages?: number;
    // A borrowed task store stays open when the runtime is disposed; the
    // owner that constructed it remains responsible for closing it.
    ownsTaskStore?: boolean;
    requiresApproval?: (tool: string, args: Record<string, unknown>) => boolean;
    requestStopAck?: (input: {
        taskId: string;
        runId: string;
        toolCallId: string;
    }) => Promise<'ack' | 'refused'> | 'ack' | 'refused';
}

export interface ResolveBackgroundInput {
    taskId: string;
    runId: string;
    toolCallId: string;
}

export interface BackgroundResolutionReceipt {
    taskId: string;
    runId: string;
    toolCallId: string;
    cleared: boolean;
}

export interface HostApprovalResolution {
    approvalId: string;
    decision: 'approved' | 'denied';
    taskId: string;
    runId: string;
    generation: number;
    requestDigest: string;
}

export interface RestoredApproval {
    approvalId: string;
    taskId: string;
    runId: string;
    generation: number;
    toolCallId: string;
    tool: string;
    scope: string;
    requestDigest: string;
    expiresAtMs: number;
    expired: boolean;
}

export interface RuntimeEventIngest {
    push(bytes: Uint8Array): Promise<IngestOutcome[]>;
    eof(): Promise<IngestOutcome | null>;
    cursorExpired(): Promise<IngestOutcome>;
}

interface Deferred<T> {
    promise: Promise<T>;
    resolve(value: T): void;
}

function deferred<T>(): Deferred<T> {
    let resolve!: (value: T) => void;
    const promise = new Promise<T>((inner) => {
        resolve = inner;
    });
    return { promise, resolve };
}

interface PendingBoundaryMessage {
    messageId: string;
    text: string;
    delivery: 'steer' | 'force';
    timer: unknown;
}

interface QueuedTurn {
    messageId: string;
    text: string;
    delivery: MessageDelivery;
}

interface ApprovalRecord {
    approvalId: string;
    taskId: string;
    runId: string;
    generation: number;
    toolCallId: string;
    tool: string;
    scope: string;
    requestDigest: string;
    expiresAtMs: number;
    arguments: Record<string, unknown>;
    hostApproval: boolean;
}

interface InFlightTool {
    toolCallId: string;
    tool: string;
}

interface RunControl {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    mode: 'plan' | 'agent';
    prompt: string;
    startedAtMs: number;
    sequence: number;
    terminal: boolean;
    cancelRequested: boolean;
    inFlight: InFlightTool | null;
    pendingBoundary: PendingBoundaryMessage[];
    queued: QueuedTurn[];
    approvals: Map<string, ApprovalRecord>;
    pendingApproval: {
        record: ApprovalRecord;
        resolve: (decision: 'approved' | 'denied' | 'cancelled') => void;
    } | null;
    stopDeferred: Deferred<boolean> | null;
    stopTimer: unknown;
    modelIterator: AsyncIterator<ModelStreamEvent> | null;
    turnIndex: number;
    extraUserTexts: string[];
    forceTexts: string[];
    leaseHeld: boolean;
}

interface RuntimeListener {
    listener: (event: AgentIncomingEvent, digest?: string) => void;
    filter: SubscribeInput;
}

function isTerminalState(state: RunState): boolean {
    return TERMINAL_RUN_STATES.has(state);
}

function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

function errorCode(error: unknown): AgentErrorCode {
    if (error instanceof AgentClientError || error instanceof AgentStoreError) {
        return error.code;
    }
    return 'internal';
}

function randomHex(bytes: number): string {
    const values = new Uint8Array(bytes);
    const crypto = globalThis.crypto;
    if (crypto !== undefined && typeof crypto.getRandomValues === 'function') {
        crypto.getRandomValues(values);
    } else {
        for (let index = 0; index < values.length; index += 1) {
            values[index] = Math.floor(Math.random() * 256);
        }
    }
    let text = '';
    for (const value of values) text += value.toString(16).padStart(2, '0');
    return text;
}

function defaultIds(): RuntimeIdFactory {
    return {
        taskId: () => `tsk_${randomHex(8)}`,
        runId: () => `run_${randomHex(8)}`,
        messageId: () => `msg_${randomHex(8)}`,
        approvalId: () => `appr_${randomHex(8)}`,
    };
}

function defaultTimers(): RuntimeTimers {
    return {
        setTimeout: (handler, timeoutMs) =>
            globalThis.setTimeout(handler, timeoutMs),
        clearTimeout: (handle) => globalThis.clearTimeout(handle as never),
        setInterval: (handler, timeoutMs) =>
            globalThis.setInterval(handler, timeoutMs),
        clearInterval: (handle) => globalThis.clearInterval(handle as never),
    };
}

function boundedPreview(text: string): string {
    return text.length > 120 ? text.slice(0, 120) : text;
}

function cloneJson<T>(value: T): T {
    return JSON.parse(canonicalJson(value)) as T;
}

class RuntimeEventStream implements AgentEventStream {
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
        return new Promise((resolve) => this.waiters.push(resolve));
    }

    return(): Promise<IteratorResult<AgentIncomingEvent>> {
        this.close();
        return Promise.resolve({ value: undefined as never, done: true });
    }

    [Symbol.asyncIterator](): AsyncIterator<AgentIncomingEvent> {
        return this;
    }
}

export interface AddDiffCommentInput {
    taskId: string;
    path: string;
    line: number;
    body: string;
    headDigest: string;
    scope: 'last_turn' | 'branch';
    commentId?: string;
    createdAtMs?: number;
}

export class AgentRuntime {
    readonly store: AgentStore;

    private readonly taskStore: AgentTaskStore | null;
    private readonly modelPort: ModelPort;
    private readonly toolExecutor: RuntimeToolExecutor;
    private readonly hostClientInstance: HostClient | null;
    private readonly now: () => number;
    private readonly timers: RuntimeTimers;
    private readonly ids: RuntimeIdFactory;
    private readonly writerId: string;
    private readonly stopAckTimeoutMs: number;
    private readonly boundaryWaitMs: number;
    private readonly approvalTtlMs: number;
    private readonly leaseHeartbeatMs: number;
    private readonly maxQueuedMessages: number;
    private readonly ownsTaskStore: boolean;
    private readonly requiresApproval: (
        tool: string,
        args: Record<string, unknown>,
    ) => boolean;
    private readonly requestStopAck:
        | AgentRuntimeOptions['requestStopAck']
        | null;

    private readonly active = new Map<string, RunControl>();
    private readonly taskQueues = new Map<string, QueuedTurn[]>();
    private readonly promptByRun = new Map<string, string>();
    private readonly chains = new Map<string, Promise<void>>();
    private readonly modelTasks = new Set<Promise<void>>();
    private readonly listeners = new Set<RuntimeListener>();
    private readonly eventStreams = new Set<RuntimeEventStream>();
    private readonly heldLeases = new Set<string>();
    private restoredApprovals: RestoredApproval[] = [];
    private heartbeatHandle: unknown = null;
    private heartbeatTask: Promise<void> = Promise.resolve();
    private heartbeatInFlight = false;
    private opened = false;
    private disposed = false;
    private disposing = false;
    private persistenceError: AgentClientError | null = null;

    constructor(options: AgentRuntimeOptions = {}) {
        this.store = options.store ?? createAgentStore();
        this.taskStore = options.taskStore ?? null;
        if (
            this.taskStore !== null &&
            this.taskStore.store !== this.store
        ) {
            throw new AgentClientError(
                'invalid_arguments',
                'the task store must own the same AgentStore as the runtime',
            );
        }
        this.modelPort = options.modelPort ?? new ReplayModelPort();
        if (options.hostClient != null && options.toolExecutor != null) {
            throw new AgentClientError(
                'invalid_arguments',
                'inject a host client or a tool executor, not two host paths',
            );
        }
        this.hostClientInstance = options.hostClient ?? null;
        this.toolExecutor =
            options.toolExecutor ??
            (this.hostClientInstance !== null
                ? (intent) => this.executeHostTool(intent)
                : async () => {
                      throw new AgentClientError(
                          'host_unavailable',
                          'no host tool executor is connected',
                      );
                  });
        this.now = options.now ?? (() => Date.now());
        this.timers = options.timers ?? defaultTimers();
        this.ids = options.ids ?? defaultIds();
        this.writerId = options.writerId ?? `writer-${randomHex(8)}`;
        this.stopAckTimeoutMs =
            options.stopAckTimeoutMs ?? DEFAULT_STOP_ACK_TIMEOUT_MS;
        this.boundaryWaitMs =
            options.boundaryWaitMs ?? DEFAULT_BOUNDARY_WAIT_MS;
        this.approvalTtlMs = options.approvalTtlMs ?? APPROVAL_TTL_MS;
        this.leaseHeartbeatMs =
            options.leaseHeartbeatMs ??
            Math.max(1, Math.floor(LEASE_STALE_MS / 3));
        this.maxQueuedMessages =
            options.maxQueuedMessages ?? DEFAULT_MAX_QUEUED_MESSAGES;
        this.ownsTaskStore = options.ownsTaskStore ?? true;
        this.requiresApproval = options.requiresApproval ?? (() => false);
        this.requestStopAck = options.requestStopAck ?? null;
    }

    get isOpen(): boolean {
        return this.opened;
    }

    get hostClient(): HostClient | null {
        return this.hostClientInstance;
    }

    get lastPersistenceError(): AgentClientError | null {
        return this.persistenceError;
    }

    async open(): Promise<void> {
        if (this.opened) return;
        if (this.taskStore !== null) await this.taskStore.open();
        this.opened = true;
        this.reconstructApprovals();
        await this.reconcileTerminalHolds();
        if (this.taskStore !== null && this.heartbeatHandle === null) {
            this.heartbeatHandle = this.timers.setInterval(() => {
                this.tickHeartbeat();
            }, this.leaseHeartbeatMs);
        }
    }

    // A hold whose run is terminal cannot block new work forever. Reconcile it
    // through the same task-record path; a write failure propagates so the
    // caller sees an error instead of a silently stuck task.
    private async reconcileTerminalHolds(): Promise<void> {
        for (const task of this.store.snapshot().tasks) {
            if (task.hold === null) continue;
            const held = this.store.getRun(task.taskId, task.hold.run_id);
            if (held === null) {
                await this.setTaskHold(task.taskId, null);
                continue;
            }
            if (!isTerminalState(held.state)) continue;
            if (this.isDesignedBackgroundHold(held)) continue;
            await this.setTaskHold(task.taskId, null);
        }
    }

    // A stop_unacknowledged failure plus a hold is the designed background
    // wait; it must be resolved explicitly, not swept away as stale.
    private isDesignedBackgroundHold(run: RunView): boolean {
        return run.state === 'failed' && run.error?.code === 'stop_unacknowledged';
    }

    private recordPersistenceError(error: unknown): void {
        if (this.persistenceError !== null) return;
        this.persistenceError = new AgentClientError(
            errorCode(error),
            errorMessage(error),
            false,
            error instanceof AgentStoreError ? error.storage : null,
        );
    }

    async dispose(): Promise<void> {
        if (this.disposed || this.disposing) return;
        this.disposing = true;
        try {
            const record = (error: unknown): void =>
                this.recordPersistenceError(error);
            for (const control of [...this.active.values()]) {
                try {
                    await this.failAcceptedMessages(
                        control,
                        record,
                        'disposed',
                    );
                } catch (error) {
                    record(error);
                }
            }
            this.disposed = true;
            if (this.heartbeatHandle !== null) {
                try {
                    this.timers.clearInterval(this.heartbeatHandle);
                } catch (error) {
                    record(error);
                }
                this.heartbeatHandle = null;
            }
            const controls = [...this.active.values()];
            for (const control of controls) {
                control.terminal = true;
                control.cancelRequested = true;
                this.clearTimersSafely(control, record);
                control.pendingApproval?.resolve('cancelled');
                control.pendingApproval = null;
                control.stopDeferred?.resolve(false);
                control.stopDeferred = null;
            }
            // Controls are terminal; close the host before any chain/iterator
            // wait so a pending tool call can be rejected instead of holding
            // cleanup.
            let hostClosed = this.hostClientInstance === null;
            const closeHost = (): void => {
                if (hostClosed) return;
                try {
                    this.hostClientInstance?.close();
                    hostClosed = true;
                } catch (error) {
                    record(error);
                }
            };
            closeHost();
            try {
                try {
                    await this.heartbeatTask;
                } catch (error) {
                    record(error);
                }
                for (const control of controls) {
                    try {
                        await control.modelIterator?.return?.();
                    } catch (error) {
                        record(error);
                    }
                    this.active.delete(control.taskId);
                    try {
                        await this.releaseLease(control.taskId);
                    } catch (error) {
                        record(error);
                    }
                }
                this.taskQueues.clear();
                await Promise.allSettled([...this.chains.values()]);
            } finally {
                for (const stream of [...this.eventStreams]) {
                    try {
                        stream.close();
                    } catch (error) {
                        record(error);
                    }
                }
                this.eventStreams.clear();
                this.listeners.clear();
                this.restoredApprovals = [];
                closeHost();
                try {
                    if (this.ownsTaskStore) this.taskStore?.close();
                } catch (error) {
                    record(error);
                }
                this.opened = false;
            }
        } finally {
            this.disposing = false;
        }
    }

    async listTasks(input: ListTasksInput = {}): Promise<TaskSummary[]> {
        this.assertOpen();
        if (this.taskStore !== null) {
            const records = await this.taskStore.listTasks({
                ...(input.includeArchived === undefined
                    ? {}
                    : { includeArchived: input.includeArchived }),
                ...(input.query === undefined ? {} : { query: input.query }),
            });
            return records
                .map((record) => this.summaryFor(record.task_id))
                .filter((summary): summary is TaskSummary => summary !== null);
        }
        const query = input.query?.trim().toLowerCase() ?? '';
        return this.store
            .snapshot()
            .tasks.filter(
                (task) =>
                    (input.includeArchived === true || !task.archived) &&
                    (query === '' || task.title.toLowerCase().includes(query)),
            )
            .map((task) => this.summaryFor(task.taskId))
            .filter((summary): summary is TaskSummary => summary !== null);
    }

    async createTask(input: CreateTaskInput): Promise<TaskSummary> {
        this.assertOpen();
        const title = input.title.trim();
        if (title.length === 0) {
            throw new AgentClientError(
                'invalid_arguments',
                'task title must not be empty',
            );
        }
        const projectId = input.projectId ?? DEFAULT_PROJECT_ID;
        if (!PROJECT_ID_PATTERN.test(projectId)) {
            throw new AgentClientError(
                'invalid_arguments',
                'projectId must match prj_<16 lowercase hex>',
            );
        }
        const taskId = this.ids.taskId();
        if (this.taskStore !== null) {
            await this.taskStore.createTask({
                project_id: projectId,
                title,
                task_id: taskId,
                ...(input.repo === undefined ? {} : { repo: input.repo }),
                ...(input.branch === undefined ? {} : { branch: input.branch }),
                created_at_ms: this.now(),
            });
        } else {
            this.store.upsertTask({
                taskId,
                projectId,
                title,
                repo: input.repo ?? null,
                branch: input.branch ?? null,
                createdAtMs: this.now(),
                archived: false,
                pinnedLocal: false,
                latestRunId: null,
            });
        }
        const summary = this.summaryFor(taskId);
        if (summary === null) {
            throw new AgentClientError('internal', 'created task is missing');
        }
        return summary;
    }

    async startRun(input: StartRunInput): Promise<RunSummary> {
        this.assertOpen();
        return this.withTask(input.taskId, () =>
            this.startRunInternal(input, null),
        );
    }

    async queueMessage(input: QueueMessageInput): Promise<MessageReceipt> {
        this.assertOpen();
        this.requireText(input.text, 'message');
        const messageId = this.ids.messageId();
        return this.withTask(input.taskId, async () => {
            this.requireTask(input.taskId);
            const current = this.active.get(input.taskId);
            if (current !== undefined && !current.terminal) {
                if (current.cancelRequested) {
                    throw new AgentClientError(
                        'out_of_order',
                        'the run is cancelling; new messages are refused',
                    );
                }
                this.ensureQueueCapacity(current);
                await this.append(current, 'message.queued', {
                    message_id: messageId,
                    delivery: 'queue',
                    boundary: 'next_turn',
                    text_preview: boundedPreview(input.text),
                });
                this.queueFor(input.taskId).push({
                    messageId,
                    text: input.text,
                    delivery: 'queue',
                });
                return {
                    taskId: input.taskId,
                    runId: current.runId,
                    messageId,
                    delivery: 'queue' as const,
                };
            }
            const run = await this.startRunInternal(
                { taskId: input.taskId, prompt: input.text },
                null,
                {
                    messageId,
                    text: input.text,
                    delivery: 'queue',
                },
            );
            return {
                taskId: input.taskId,
                runId: run.runId,
                messageId,
                delivery: 'queue' as const,
            };
        });
    }

    async steer(input: SteerMessageInput): Promise<MessageReceipt> {
        return this.deliverBoundaryMessage(input, 'steer');
    }

    async forceSend(input: ForceSendInput): Promise<MessageReceipt> {
        return this.deliverBoundaryMessage(input, 'force');
    }

    async pinTask(input: PinTaskInput): Promise<TaskSummary> {
        this.assertOpen();
        if (this.taskStore !== null) {
            await this.taskStore.pinTask(input.taskId, input.pinned);
        } else {
            const task = this.requireTaskView(input.taskId);
            this.store.upsertTask({
                taskId: task.taskId,
                projectId: task.projectId,
                title: task.title,
                repo: task.repo,
                branch: task.branch,
                createdAtMs: task.createdAtMs,
                archived: task.archived,
                pinnedLocal: input.pinned,
                latestRunId: task.latestRunId,
            });
        }
        return this.requireSummary(input.taskId);
    }

    async archiveTask(input: ArchiveTaskInput): Promise<TaskSummary> {
        this.assertOpen();
        if (this.taskStore !== null) {
            await this.taskStore.archiveTask(input.taskId, input.archived);
        } else {
            const task = this.requireTaskView(input.taskId);
            this.store.upsertTask({
                taskId: task.taskId,
                projectId: task.projectId,
                title: task.title,
                repo: task.repo,
                branch: task.branch,
                createdAtMs: task.createdAtMs,
                archived: input.archived,
                pinnedLocal: task.pinnedLocal,
                latestRunId: task.latestRunId,
            });
        }
        return this.requireSummary(input.taskId);
    }

    async stop(input: StopInput): Promise<StopReceipt> {
        this.assertOpen();
        const init = await this.withTask(input.taskId, async () => {
            const current = this.active.get(input.taskId);
            if (current === undefined || current.terminal) {
                // A restored run has no live control. Settle it durably at the
                // safe boundary instead of pretending a worker was cancelled.
                await this.stopRestoredRun(input);
                return {
                    taskId: input.taskId,
                    runId: input.runId,
                    ack: Promise.resolve(true),
                };
            }
            if (current.runId !== input.runId) {
                throw new AgentClientError(
                    'out_of_order',
                    'stop does not match the active run',
                );
            }
            if (current.cancelRequested) {
                return {
                    taskId: input.taskId,
                    runId: input.runId,
                    ack:
                        current.stopDeferred?.promise ??
                        Promise.resolve(false),
                };
            }
            await this.append(current, 'run.status', {
                status: 'cancelling',
            });
            current.cancelRequested = true;
            current.pendingApproval?.resolve('cancelled');
            current.pendingApproval = null;

            if (current.inFlight === null) {
                const acknowledged = await this.finalizeCancelled(current);
                return {
                    taskId: input.taskId,
                    runId: input.runId,
                    ack: Promise.resolve(acknowledged),
                };
            }

            const stopDeferred = deferred<boolean>();
            current.stopDeferred = stopDeferred;
            current.stopTimer = this.timers.setTimeout(() => {
                void this.stopTimeout(current.taskId, current.runId).catch(
                    (error) => this.recoverStopFailure(
                        current.taskId,
                        current.runId,
                        error,
                    ),
                );
            }, this.stopAckTimeoutMs);
            if (this.requestStopAck !== null) {
                const toolCallId = current.inFlight.toolCallId;
                void Promise.resolve()
                    .then(() =>
                        this.requestStopAck!({
                            taskId: current.taskId,
                            runId: current.runId,
                            toolCallId,
                        }),
                    )
                    .then(
                        (outcome) => {
                            if (outcome === 'refused') {
                                void this.stopRefused(
                                    current.taskId,
                                    current.runId,
                                ).catch((error) =>
                                    this.recoverStopFailure(
                                        current.taskId,
                                        current.runId,
                                        error,
                                    ),
                                );
                            }
                        },
                        () => {
                            void this.stopRefused(
                                current.taskId,
                                current.runId,
                            ).catch((error) =>
                                this.recoverStopFailure(
                                    current.taskId,
                                    current.runId,
                                    error,
                                ),
                            );
                        },
                    );
            }
            return {
                taskId: input.taskId,
                runId: input.runId,
                ack: stopDeferred.promise,
            };
        });
        return {
            taskId: init.taskId,
            runId: init.runId,
            acknowledged: await init.ack,
        };
    }

    async retry(input: RetryRunInput): Promise<RunSummary> {
        this.assertOpen();
        return this.withTask(input.taskId, async () => {
            const view = this.store.getRun(input.taskId, input.runId);
            if (view === null) {
                throw new AgentClientError('run_not_found', 'run is missing');
            }
            const retryable =
                view.state === 'cancelled' ||
                (view.state === 'failed' &&
                    view.error?.retryable === true);
            if (!retryable) {
                throw new AgentClientError(
                    'run_not_retryable',
                    'only failed-retryable or cancelled runs are retryable',
                );
            }
            const task = this.requireTaskView(input.taskId);
            if (task.hold !== null) {
                await this.clearTerminalHold(task, 'retry');
            }
            const prompt = this.promptByRun.get(input.runId);
            if (prompt === undefined) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'the original run prompt is unavailable; durable prompt reconstruction is not implemented',
                );
            }
            return this.startRunInternal(
                { taskId: input.taskId, prompt },
                input.runId,
            );
        });
    }

    async resolveApproval(
        input: ResolveApprovalInput,
    ): Promise<ApprovalReceipt> {
        this.assertOpen();
        if (input.decision !== 'approved' && input.decision !== 'denied') {
            throw new AgentClientError(
                'invalid_arguments',
                'approval decision must be approved | denied',
            );
        }
        const initial = this.findApproval(input.approvalId);
        if (initial === null) {
            throw new AgentClientError(
                'approval_forbidden',
                'approval is not pending',
            );
        }
        if (initial.hostApproval) {
            const hostClient = this.hostClientInstance;
            if (hostClient === null) {
                throw new AgentClientError(
                    'host_unavailable',
                    'the host approval can no longer be resolved',
                );
            }
            const receipt = await hostClient.approve({
                approvalId: initial.approvalId,
                decision: input.decision,
            });
            if (
                receipt.approvalId !== initial.approvalId ||
                receipt.decision !== input.decision
            ) {
                throw new AgentClientError(
                    'approval_forbidden',
                    'the host approval receipt does not match the page decision',
                );
            }
        }
        await this.withTask(initial.taskId, async () => {
            const record = this.findApproval(input.approvalId);
            await this.resolveApprovalRecord(
                record,
                input.decision,
                record?.requestDigest ?? '',
                {
                    taskId: record?.taskId ?? '',
                    runId: record?.runId ?? '',
                    generation: record?.generation ?? 0,
                },
            );
        });
        return {
            approvalId: input.approvalId,
            decision: input.decision,
        };
    }

    async resolveHostApproval(
        input: HostApprovalResolution,
    ): Promise<ApprovalReceipt> {
        this.assertOpen();
        const initial = this.findApproval(input.approvalId);
        if (initial === null) {
            throw new AgentClientError(
                'approval_forbidden',
                'approval is not pending',
            );
        }
        await this.withTask(initial.taskId, async () => {
            const record = this.findApproval(input.approvalId);
            await this.resolveApprovalRecord(
                record,
                input.decision,
                input.requestDigest,
                {
                    taskId: input.taskId,
                    runId: input.runId,
                    generation: input.generation,
                },
            );
        });
        return {
            approvalId: input.approvalId,
            decision: input.decision,
        };
    }

    async resolveBackground(
        input: ResolveBackgroundInput,
    ): Promise<BackgroundResolutionReceipt> {
        this.assertOpen();
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === input.taskId,
        );
        if (task === undefined) {
            throw new AgentClientError('task_not_found', 'task is missing');
        }
        if (
            task.hold === null ||
            task.hold.run_id !== input.runId ||
            task.hold.tool_call_id !== input.toolCallId
        ) {
            throw new AgentClientError(
                'out_of_order',
                'background hold does not match the run and tool',
            );
        }
        await this.setTaskHold(input.taskId, null);
        return {
            taskId: input.taskId,
            runId: input.runId,
            toolCallId: input.toolCallId,
            cleared: true,
        };
    }

    async resumeTask(input: ResumeTaskInput): Promise<ResumeTaskResult> {
        this.assertOpen();
        return this.withTask(input.taskId, async () => {
            const current = this.active.get(input.taskId);
            if (current === undefined || current.runId !== input.runId) {
                throw new AgentClientError(
                    'runtime_unavailable',
                    'run is not active in this runtime',
                );
            }
            const cursor = this.store.getRunCursor(input.taskId, input.runId);
            if (
                cursor === null ||
                cursor.lastEventId === null ||
                cursor.lastDigest === null
            ) {
                throw new AgentClientError(
                    'snapshot_missing',
                    'run has no verified event chain',
                );
            }
            let snapshot: TaskSnapshot;
            if (this.taskStore !== null) {
                const record = await this.taskStore.writeSnapshot(
                    input.taskId,
                    input.runId,
                );
                snapshot = {
                    taskId: record.task_id,
                    runId: record.run_id,
                    generation: record.generation,
                    lastSequence: record.last_sequence,
                    lastEventId: record.last_event_id,
                    headDigest: record.head_digest,
                };
            } else {
                snapshot = {
                    taskId: input.taskId,
                    runId: input.runId,
                    generation: cursor.generation,
                    lastSequence: cursor.lastSequence,
                    lastEventId: cursor.lastEventId,
                    headDigest: cursor.lastDigest,
                };
                const verified = this.store.verifySnapshot(snapshot);
                if (!verified.ok) {
                    throw new AgentClientError(
                        verified.code ?? 'snapshot_corrupt',
                        verified.message ?? 'snapshot verification failed',
                    );
                }
            }
            const previousGeneration = current.generation;
            const event = await this.appendResume(
                current,
                previousGeneration,
                snapshot,
            );
            return { snapshot, event: event as AgentEvent };
        });
    }

    subscribe(
        listener: (event: AgentIncomingEvent, digest?: string) => void,
        input?: SubscribeInput,
    ): () => void;
    subscribe(input?: SubscribeInput): Promise<AgentEventStream>;
    subscribe(
        arg?: ((event: AgentIncomingEvent, digest?: string) => void) | SubscribeInput,
        input: SubscribeInput = {},
    ): (() => void) | Promise<AgentEventStream> {
        if (typeof arg === 'function') {
            const entry: RuntimeListener = { listener: arg, filter: input };
            this.listeners.add(entry);
            return () => {
                this.listeners.delete(entry);
            };
        }
        let stream: RuntimeEventStream;
        stream = new RuntimeEventStream(arg ?? {}, () => {
            this.eventStreams.delete(stream);
        });
        this.eventStreams.add(stream);
        return Promise.resolve(stream);
    }

    async ingestWireEvent(text: string): Promise<AgentAppendResult> {
        this.assertOpen();
        const event = parseAgentEventWire(text);
        return this.withTask(event.task_id, () => this.appendIncoming(event));
    }

    // One worker per run stream. A halted worker is not reset; after a gap or
    // cursor expiry the caller verifies a snapshot (resumeTask) and creates a
    // replacement worker for the new generation.
    createIngest(input: { taskId: string; runId: string }): RuntimeEventIngest {
        this.assertOpen();
        const worker = new NdjsonIngest({
            taskId: input.taskId,
            runId: input.runId,
            sink: (event) => this.appendIncomingBound(event, input),
        });
        return {
            push: (bytes) => worker.push(bytes),
            eof: () => worker.eof(),
            cursorExpired: async () => {
                const outcome = await worker.cursorExpired();
                try {
                    await this.resumeTask(input);
                } catch (error) {
                    return IngestOutcome.rejected(errorCode(error));
                }
                return outcome;
            },
        };
    }

    // Pending approvals replayed from the verified event log. The original
    // runtime is not alive after a reload, so these objects are exact
    // identity/digest/expiry records for display and refusal, not a second
    // resolution authority. An already-resolved approval is never listed.
    async pendingApprovals(taskId?: string): Promise<RestoredApproval[]> {
        this.assertOpen();
        return this.restoredApprovals
            .filter(
                (approval) =>
                    taskId === undefined || approval.taskId === taskId,
            )
            .map((approval) => ({ ...approval }));
    }

    // True only while this runtime holds the live control for the approval's
    // run and generation. A restored approval has no such owner.
    approvalResolvable(approvalId: string): boolean {
        const record = this.findApproval(approvalId);
        if (record === null) return false;
        const current = this.active.get(record.taskId);
        if (current === undefined || current.terminal) return false;
        if (
            current.runId !== record.runId ||
            current.generation !== record.generation
        ) {
            return false;
        }
        if (
            current.pendingApproval === null ||
            current.pendingApproval.record.approvalId !== approvalId
        ) {
            return false;
        }
        if (record.hostApproval && this.hostClientInstance === null) {
            return false;
        }
        return true;
    }

    private runtimeUnavailable(
        event: AgentIncomingEvent,
    ): AgentAppendResult | null {
        if (!this.disposed && this.opened) return null;
        return {
            status: 'rejected',
            code: 'runtime_unavailable',
            message: 'the browser runtime is disposed or not open',
            requiresResync: false,
            eventId: event.event_id,
            digest: null,
            storage: null,
        };
    }

    private async appendIncoming(
        event: AgentIncomingEvent,
    ): Promise<AgentAppendResult> {
        const unavailable = this.runtimeUnavailable(event);
        if (unavailable !== null) return unavailable;
        const control = this.active.get(event.task_id);
        if (
            control !== undefined &&
            control.runId === event.run_id &&
            TERMINAL_EVENT_TYPES.has(event.type)
        ) {
            return this.applyIngestedTerminal(control, event);
        }
        let result: AgentAppendResult;
        if (this.taskStore === null) {
            result = await this.store.appendEvent(event);
        } else {
            const stored = await this.taskStore.appendEvent({
                event,
                writer_id: this.writerId,
            });
            result = {
                status: stored.status,
                code: stored.code,
                message: stored.message,
                // The reducer's own needsResync decision distinguishes a gap
                // or duplicate (resyncable) from a terminal-run rejection
                // (not resyncable) and a cross-task event.
                requiresResync:
                    stored.status === 'rejected' &&
                    this.store.getRun(event.task_id, event.run_id)
                        ?.needsResync === true,
                eventId: stored.event_id,
                digest: stored.digest,
                storage: stored.storage,
            };
        }
        if (result.status === 'applied') {
            this.persistenceError = null;
            this.publish(event, result.digest ?? undefined);
            this.syncControlAfterIngest(event);
        } else if (result.status === 'rejected') {
            this.persistenceError = new AgentClientError(
                result.code ?? 'internal',
                result.message ?? 'durable append was rejected',
                true,
                result.storage ?? null,
            );
        }
        return result;
    }

    private async appendIncomingBound(
        event: AgentIncomingEvent,
        expected: { taskId: string; runId: string },
    ): Promise<AgentAppendResult> {
        const unavailable = this.runtimeUnavailable(event);
        if (unavailable !== null) return unavailable;
        if (event.task_id !== expected.taskId) {
            return {
                status: 'rejected',
                code: 'cross_task_event',
                message: 'event does not belong to the bound task',
                requiresResync: false,
                eventId: event.event_id,
                digest: null,
                storage: null,
            };
        }
        if (event.run_id !== expected.runId) {
            return {
                status: 'rejected',
                code: 'out_of_order',
                message: 'event does not belong to the bound run',
                requiresResync: false,
                eventId: event.event_id,
                digest: null,
                storage: null,
            };
        }
        return this.withTask(expected.taskId, () =>
            this.appendIncoming(event),
        );
    }

    // An already-applied non-terminal external event is authoritative for the
    // reducer cursor. Move this runtime's own append counter past it so the
    // next runtime-generated event does not collide with the ingested sequence.
    private syncControlAfterIngest(event: AgentIncomingEvent): void {
        const control = this.active.get(event.task_id);
        if (control === undefined || control.runId !== event.run_id) return;
        if (event.sequence >= control.sequence) {
            control.sequence = event.sequence + 1;
        }
        if (event.generation > control.generation) {
            control.generation = event.generation;
        }
    }

    // A terminal event from the host stream is a validated signal, not an
    // event appended before the page's pending records. Preflight it against
    // the authoritative reducer on a checkpoint, emit the normal §10.2
    // fallback/delivery-failure records before it, append the terminal data
    // last at the runtime's current sequence, then settle the control and
    // start the queued generation for run.finished.
    private preflightIngestedTerminal(
        event: AgentIncomingEvent,
    ): AgentApplyResult {
        const checkpoint = this.store.checkpoint();
        try {
            return this.store.apply(event);
        } catch (error) {
            return {
                status: 'rejected',
                code: 'internal',
                message: errorMessage(error),
                requiresResync: false,
            };
        } finally {
            this.store.restore(checkpoint);
        }
    }

    private async applyIngestedTerminal(
        control: RunControl,
        event: AgentIncomingEvent,
    ): Promise<AgentAppendResult> {
        const preflight = this.preflightIngestedTerminal(event);
        if (preflight.status === 'ignored') {
            return {
                status: 'ignored',
                code: null,
                message: 'duplicate terminal event ignored',
                requiresResync: false,
                eventId: event.event_id,
                digest: null,
                storage: null,
            };
        }
        if (preflight.status === 'rejected') {
            return {
                status: 'rejected',
                code: preflight.code ?? 'invalid_event',
                message: preflight.message ?? 'terminal event was rejected',
                requiresResync: preflight.requiresResync === true,
                eventId: event.event_id,
                digest: null,
                storage: null,
            };
        }
        if (control.terminal) {
            return {
                status: 'rejected',
                code: 'out_of_order',
                message: 'run is already terminal',
                requiresResync: false,
                eventId: event.event_id,
                digest: null,
                storage: null,
            };
        }
        const finished = event.type === 'run.finished';
        try {
            if (finished) {
                await this.flushFallback(control);
            } else {
                await this.failUndeliveredMessages(
                    control,
                    event.type === 'run.cancelled'
                        ? 'cancelled'
                        : event.type === 'run.expired'
                          ? 'expired'
                          : 'failed',
                );
            }
            const appended = await this.append(
                control,
                event.type,
                event.data as Record<string, unknown>,
            );
            await this.afterTerminal(control, {
                startQueued: finished,
                stopAcknowledged:
                    finished || event.type === 'run.cancelled',
            });
            return {
                status: 'applied',
                code: null,
                message: null,
                requiresResync: false,
                eventId: appended.event_id,
                digest: null,
                storage: null,
            };
        } catch (error) {
            this.persistenceError = new AgentClientError(
                errorCode(error),
                errorMessage(error),
                false,
                error instanceof AgentStoreError ? error.storage : null,
            );
            // The terminal could not be made durable after a preterminal
            // fallback was already committed. Settle the live control so no
            // stop receipt or queued turn waits forever, but append no
            // fabricated terminal event.
            try {
                await this.settleControlWithoutEvent(control, false);
            } catch (settleError) {
                this.recordPersistenceError(settleError);
            } finally {
                // Idempotent: a failed settle must still release any stop
                // receipt or queued turn that would otherwise wait forever.
                control.stopDeferred?.resolve(false);
                control.stopDeferred = null;
                this.taskQueues.delete(control.taskId);
            }
            return {
                status: 'rejected',
                code: errorCode(error),
                message: errorMessage(error),
                requiresResync: false,
                eventId: event.event_id,
                digest: null,
                storage: error instanceof AgentStoreError ? error.storage : null,
            };
        }
    }

    private reconstructApprovals(): void {
        const now = this.now();
        const restored: RestoredApproval[] = [];
        const seen = new Set<string>();
        for (const run of this.store.snapshot().runs) {
            if (run.needsResync || isTerminalState(run.state)) continue;
            for (const approval of run.approvals) {
                if (
                    approval.decision !== null ||
                    seen.has(approval.approvalId)
                ) {
                    continue;
                }
                seen.add(approval.approvalId);
                const required = run.events.find(
                    (event) =>
                        event.type === 'tool.approval_required' &&
                        event.data.approval_id === approval.approvalId,
                );
                restored.push({
                    approvalId: approval.approvalId,
                    taskId: run.taskId,
                    runId: run.runId,
                    generation: required?.generation ?? run.generation,
                    toolCallId: approval.toolCallId,
                    tool: approval.tool,
                    scope: approval.scope,
                    requestDigest: approval.requestDigest,
                    expiresAtMs: approval.expiresAtMs,
                    expired: now > approval.expiresAtMs,
                });
            }
        }
        this.restoredApprovals = restored;
    }

    async tree(_input: TreeInput): Promise<TreeEntry[]> {
        throw new AgentClientError(
            'host_unavailable',
            'the browser runtime has no host file tree',
        );
    }

    async changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null> {
        const local = this.store.latestChangesSnapshot(
            input.taskId,
            input.runId,
            input.scope,
        );
        if (local !== null) {
            return {
                scope: local.scope,
                files: local.files,
                headDigest: local.headDigest,
                capturedAtMs: local.capturedAtMs,
            };
        }
        if (input.scope === 'last_turn') return null;
        throw new AgentClientError(
            'host_unavailable',
            'the browser runtime has no branch diff',
        );
    }

    async addDiffComment(input: AddDiffCommentInput): Promise<DiffCommentRecord> {
        this.assertOpen();
        if (this.taskStore === null) {
            throw new AgentClientError(
                'runtime_unavailable',
                'durable local comments require the browser task store',
            );
        }
        return this.withTask(input.taskId, async () => {
            this.requireTaskView(input.taskId);
            const result = await this.taskStore!.appendDiffComment({
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
            const control = this.active.get(input.taskId);
            if (control !== undefined && control.runId === result.run_id) {
                control.sequence = result.sequence + 1;
            }
            return result.comment;
        });
    }

    async readDiffComments(taskId: string): Promise<DiffCommentRecord[]> {
        this.assertOpen();
        if (this.taskStore === null) {
            throw new AgentClientError(
                'runtime_unavailable',
                'durable local comments require the browser task store',
            );
        }
        return this.taskStore.readDiffComments(taskId);
    }

    async readArtifact(input: ReadArtifactInput): Promise<ArtifactContent> {
        if ((input.start === undefined) !== (input.end === undefined)) {
            throw new AgentClientError(
                'invalid_arguments',
                'artifact range requires both start and end',
            );
        }
        if (input.artifactId !== undefined && this.taskStore !== null) {
            try {
                const range =
                    input.start === undefined || input.end === undefined
                        ? undefined
                        : { start: input.start, end: input.end };
                const record = await this.taskStore.readArtifact(
                    input.artifactId,
                    range,
                );
                if (record !== null) {
                    return {
                        path: record.artifact_id,
                        mediaType: record.media_type,
                        sizeBytes: record.size_bytes,
                        artifactSizeBytes: record.artifact_size_bytes,
                        digest: record.digest,
                        wholeDigest: record.whole_digest,
                        range: record.range,
                        bytes: record.bytes,
                    };
                }
            } catch (error) {
                throw toClientError(error);
            }
        }
        throw new AgentClientError(
            'host_unavailable',
            'no durable browser artifact is available',
        );
    }

    async hostCapabilities(): Promise<HostCapabilities> {
        if (this.hostClientInstance !== null) {
            return this.hostClientInstance.capabilities();
        }
        return {
            tree: false,
            changes: false,
            readArtifact: false,
            terminal: false,
            git: false,
            mcp: false,
            approvals: false,
        };
    }

    private async startRunInternal(
        input: StartRunInput,
        retryOfRunId: string | null,
        queuedMessage?: QueuedTurn,
    ): Promise<RunSummary> {
        this.assertOpen();
        this.requireText(input.prompt, 'run prompt');
        const task = this.requireTask(input.taskId);
        if (this.active.has(input.taskId)) {
            throw new AgentClientError(
                'out_of_order',
                'task already has an active run',
            );
        }
        if (task.unresolvedRunId !== null) {
            throw new AgentClientError(
                'out_of_order',
                'the task latest run is still loading',
            );
        }
        if (task.hold !== null) {
            await this.clearTerminalHold(task, 'a new run');
        }
        const latestRun = task.latestRunId === null ? null : this.store.getRun(
            input.taskId,
            task.latestRunId,
        );
        if (latestRun !== null && !isTerminalState(latestRun.state)) {
            throw new AgentClientError(
                'out_of_order',
                'task latest run is still active',
            );
        }
        const generation =
            task.runs.reduce((max, run) => Math.max(max, run.generation), 0) +
            1;
        const runId = this.ids.runId();
        if (!RUN_ID_PATTERN.test(runId)) {
            throw new AgentClientError(
                'invalid_arguments',
                'run id factory produced an invalid run id',
            );
        }
        if (this.taskStore !== null) {
            try {
                const lease = await this.taskStore.acquireLease(
                    input.taskId,
                    this.writerId,
                    this.now(),
                );
                this.heldLeases.add(input.taskId);
                if (lease.writer_id !== this.writerId) {
                    throw new AgentClientError(
                        'stale_generation',
                        'task lease is held by another writer',
                    );
                }
            } catch (error) {
                if (error instanceof AgentClientError) throw error;
                throw new AgentClientError(
                    errorCode(error),
                    errorMessage(error),
                    false,
                    error instanceof AgentStoreError ? error.storage : null,
                );
            }
        }
        const control: RunControl = {
            taskId: input.taskId,
            projectId: task.projectId,
            runId,
            generation,
            mode: input.mode ?? 'agent',
            prompt: input.prompt,
            startedAtMs: this.now(),
            sequence: 0,
            terminal: false,
            cancelRequested: false,
            inFlight: null,
            pendingBoundary: [],
            queued: [],
            approvals: new Map(),
            pendingApproval: null,
            stopDeferred: null,
            stopTimer: null,
            modelIterator: null,
            turnIndex: 0,
            extraUserTexts: [],
            forceTexts: [],
            leaseHeld: this.taskStore !== null,
        };
        this.active.set(input.taskId, control);
        try {
            await this.append(control, 'run.queued', {
                queued_at_ms: this.now(),
            });
            await this.append(control, 'run.started', {
                started_at_ms: this.now(),
                model: {
                    id: this.modelPort.id,
                    kind: this.modelPort.kind,
                },
                mode: control.mode,
                environment: { kind: 'browser-runtime' },
                retry_of_run_id: retryOfRunId,
            });
            if (queuedMessage !== undefined) {
                await this.append(control, 'message.queued', {
                    message_id: queuedMessage.messageId,
                    delivery: queuedMessage.delivery,
                    boundary: 'next_turn',
                    text_preview: boundedPreview(queuedMessage.text),
                });
            }
        } catch (error) {
            control.terminal = true;
            this.active.delete(input.taskId);
            await this.releaseLease(input.taskId);
            throw error;
        }
        this.rememberPrompt(runId, input.prompt);
        const modelTask = this.runModel(control).finally(() => {
            this.modelTasks.delete(modelTask);
        });
        this.modelTasks.add(modelTask);
        return {
            taskId: input.taskId,
            runId,
            generation,
            state: 'running',
            retryOfRunId,
        };
    }

    private async deliverBoundaryMessage(
        input: { taskId: string; text: string },
        delivery: 'steer' | 'force',
    ): Promise<MessageReceipt> {
        this.assertOpen();
        this.requireText(input.text, 'message');
        const messageId = this.ids.messageId();
        return this.withTask(input.taskId, async () => {
            this.requireTask(input.taskId);
            const current = this.active.get(input.taskId);
            if (current === undefined || current.terminal) {
                throw new AgentClientError(
                    'out_of_order',
                    'steer and force require an active run',
                );
            }
            if (current.cancelRequested) {
                throw new AgentClientError(
                    'out_of_order',
                    'the run is cancelling; new messages are refused',
                );
            }
            this.ensureQueueCapacity(current);
            const pending: PendingBoundaryMessage = {
                messageId,
                text: input.text,
                delivery,
                timer: null,
            };
            pending.timer = this.timers.setTimeout(() => {
                void this.boundaryFallback(current.taskId, messageId).catch(
                    (error) =>
                        this.recoverPendingFailure(
                            current.taskId,
                            messageId,
                            error,
                        ),
                );
            }, this.boundaryWaitMs);
            current.pendingBoundary.push(pending);
            return {
                taskId: input.taskId,
                runId: current.runId,
                messageId,
                delivery,
            };
        });
    }

    private async boundaryFallback(
        taskId: string,
        messageId: string,
    ): Promise<void> {
        await this.withTask(taskId, async () => {
            const current = this.active.get(taskId);
            if (current === undefined || current.terminal) return;
            const index = current.pendingBoundary.findIndex(
                (pending) => pending.messageId === messageId,
            );
            if (index < 0) return;
            const [pending] = current.pendingBoundary.splice(index, 1);
            this.timers.clearTimeout(pending!.timer);
            if (current.cancelRequested) {
                await this.appendUndeliveredMessage(
                    current,
                    pending!.messageId,
                    'cancelled',
                );
                return;
            }
            await this.emitBoundaryFallback(current, pending!);
        });
    }

    private async emitBoundaryFallback(
        current: RunControl,
        pending: PendingBoundaryMessage,
    ): Promise<void> {
        await this.append(
            current,
            pending.delivery === 'steer' ? 'message.steered' : 'message.forced',
            {
                message_id: pending.messageId,
                boundary: 'next_turn',
            },
        );
        await this.append(current, 'message.queued', {
            message_id: pending.messageId,
            delivery: pending.delivery,
            boundary: 'next_turn',
            text_preview: boundedPreview(pending.text),
        });
        this.queueFor(current.taskId).push({
            messageId: pending.messageId,
            text: pending.text,
            delivery: pending.delivery,
        });
    }

    private async flushBoundary(current: RunControl): Promise<void> {
        const pending = current.pendingBoundary.splice(0);
        for (const message of pending) {
            this.timers.clearTimeout(message.timer);
            if (current.terminal) return;
            await this.append(
                current,
                message.delivery === 'steer'
                    ? 'message.steered'
                    : 'message.forced',
                {
                    message_id: message.messageId,
                    boundary: 'next_tool',
                },
            );
            if (message.delivery === 'force') {
                current.forceTexts.push(message.text);
            } else {
                current.extraUserTexts.push(message.text);
            }
        }
    }

    private async flushFallback(current: RunControl): Promise<void> {
        const pending = current.pendingBoundary.splice(0);
        for (const message of pending) {
            this.timers.clearTimeout(message.timer);
            await this.emitBoundaryFallback(current, message);
        }
    }

    private clearPendingBoundary(current: RunControl): void {
        for (const pending of current.pendingBoundary.splice(0)) {
            this.timers.clearTimeout(pending.timer);
        }
    }

    private async runModel(current: RunControl): Promise<void> {
        try {
            for (;;) {
                if (current.terminal || current.cancelRequested) return;
                const turnId = `turn_${current.turnIndex + 1}`;
                const index = current.turnIndex;
                await this.appendSerialized(current, 'turn.started', {
                    turn_id: turnId,
                    index,
                });
                const userTurn = [current.prompt, ...current.forceTexts].join(
                    '\n',
                );
                const steeredTurns = current.extraUserTexts.splice(0);
                const history = [
                    { role: 'user' as const, text: userTurn },
                    ...steeredTurns.map((text) => ({
                        role: 'user' as const,
                        text,
                    })),
                ];
                // A steer is the latest instruction for this turn; force text
                // is folded into the base user turn instead of becoming a
                // separate history message.
                const prompt =
                    steeredTurns.length === 0
                        ? userTurn
                        : steeredTurns[steeredTurns.length - 1]!;
                const request: ModelRequest = {
                    taskId: current.taskId,
                    runId: current.runId,
                    generation: current.generation,
                    mode: current.mode,
                    prompt,
                    history,
                };
                const iterator =
                    this.modelPort.open(request)[Symbol.asyncIterator]();
                current.modelIterator = iterator;
                let text = '';
                let messageId: string | null = null;
                let finishReason: ModelFinishReason | null = null;
                const toolCalls: Array<{
                    toolCallId: string;
                    tool: string;
                    arguments: unknown;
                }> = [];
                streamLoop: for (;;) {
                    const next = await iterator.next();
                    if (current.terminal) return;
                    if (next.done) break;
                    if (current.cancelRequested) {
                        await iterator.return?.();
                        break;
                    }
                    const event = next.value;
                    switch (event.type) {
                        case 'text-delta': {
                            if (event.text.length === 0) break;
                            messageId ??= this.ids.messageId();
                            text += event.text;
                            await this.appendSerialized(
                                current,
                                'assistant.delta',
                                {
                                    message_id: messageId,
                                    text_delta: event.text,
                                },
                            );
                            break;
                        }
                        case 'usage': {
                            if (
                                !Number.isSafeInteger(event.inputTokens) ||
                                event.inputTokens < 0 ||
                                !Number.isSafeInteger(event.outputTokens) ||
                                event.outputTokens < 0 ||
                                !Number.isSafeInteger(event.modelCalls) ||
                                event.modelCalls < 0
                            ) {
                                throw new AgentClientError(
                                    'model_protocol_error',
                                    'model usage is not non-negative safe integers',
                                );
                            }
                            await this.appendSerialized(
                                current,
                                'usage.updated',
                                {
                                    input_tokens: event.inputTokens,
                                    output_tokens: event.outputTokens,
                                    model_calls: event.modelCalls,
                                },
                            );
                            break;
                        }
                        case 'tool-call':
                            toolCalls.push({
                                toolCallId: event.toolCallId,
                                tool: event.tool,
                                arguments: event.arguments,
                            });
                            break streamLoop;
                        case 'finished':
                            finishReason = event.reason;
                            break;
                        case 'failed':
                            await this.withTask(current.taskId, () =>
                                this.finalizeFailed(current, {
                                    code: event.code,
                                    message: event.message,
                                    retryable: event.retryable,
                                }),
                            );
                            return;
                        default:
                            break;
                    }
                    if (finishReason !== null) break;
                }
                current.modelIterator = null;
                await iterator.return?.();
                if (current.terminal) return;
                if (current.cancelRequested) {
                    await this.withTask(current.taskId, () =>
                        this.finalizeCancelled(current),
                    );
                    return;
                }
                if (text.length > 0 && messageId !== null) {
                    await this.appendSerialized(current, 'assistant.message', {
                        message_id: messageId,
                        text,
                    });
                }
                const reason: 'stop' | 'length' | 'tool' | 'cancelled' | 'error' =
                    toolCalls.length > 0
                        ? 'tool'
                        : finishReason ?? 'stop';
                await this.appendSerialized(current, 'turn.finished', {
                    turn_id: turnId,
                    finish_reason: reason,
                });
                if (toolCalls.length > 0) {
                    for (const toolCall of toolCalls) {
                        if (current.terminal) return;
                        await this.handleToolCall(current, toolCall);
                        if (current.terminal) return;
                    }
                    current.turnIndex += 1;
                    continue;
                }
                if (finishReason === 'error') {
                    await this.withTask(current.taskId, () =>
                        this.finalizeFailed(current, {
                            code: 'model_protocol_error',
                            message: 'model reported an error finish',
                            retryable: true,
                        }),
                    );
                    return;
                }
                await this.withTask(current.taskId, () =>
                    this.finalizeFinished(current),
                );
                return;
            }
        } catch (error) {
            if (current.terminal) return;
            await this.withTask(current.taskId, () =>
                this.finalizeFailed(current, {
                    code: errorCode(error),
                    message: errorMessage(error),
                    retryable: false,
                }),
            ).catch(() => undefined);
        }
    }

    private async executeHostTool(
        intent: RuntimeToolIntent,
    ): Promise<RuntimeToolResult> {
        const hostClient = this.hostClientInstance;
        if (hostClient === null) {
            throw new AgentClientError(
                'host_unavailable',
                'the local host companion is not connected',
            );
        }
        const hostIntent = await hostClient.createIntent({
            taskId: intent.taskId,
            runId: intent.runId,
            generation: intent.generation,
            toolCallId: intent.toolCallId,
            tool: intent.tool,
            arguments: intent.arguments,
            ...(intent.approvalId === undefined
                ? {}
                : { approvalId: intent.approvalId }),
        });
        const result = await hostClient.execute(hostIntent);
        const output = result.output ?? {};
        let hostFiles: Array<Record<string, unknown>> | undefined;
        let hostHeadDigest: string | undefined;
        if (
            result.status === 'ok' &&
            isPlainObject(output) &&
            Array.isArray(output.files) &&
            typeof output.head_digest === 'string'
        ) {
            hostFiles = output.files as Array<Record<string, unknown>>;
            hostHeadDigest = output.head_digest;
        }
        return {
            status: result.status,
            output,
            ...(hostFiles === undefined || hostHeadDigest === undefined
                ? {}
                : { files: hostFiles, head_digest: hostHeadDigest }),
            ...(result.code === undefined ? {} : { code: result.code }),
            ...(result.message === undefined
                ? {}
                : { message: result.message }),
            ...(result.truncated === true ? { truncated: true } : {}),
        };
    }

    /**
     * Ask the host for its own approval identity without executing the tool.
     *
     * The host returns an explicit approval-required result and creates no
     * effect. A host without approval capability keeps the local approval flow.
     */
    private async preflightHostApproval(
        input: RuntimeToolIntent,
    ): Promise<{ approvalId: string; expiresAtMs: number } | null> {
        const hostClient = this.hostClientInstance;
        if (hostClient === null) return null;
        let capabilities: HostCapabilities;
        try {
            capabilities = await hostClient.capabilities();
        } catch (error) {
            const clientError = toClientError(error);
            if (clientError.code === 'host_unavailable') return null;
            throw clientError;
        }
        if (capabilities.approvals !== true) return null;
        const intent = await hostClient.createIntent({
            taskId: input.taskId,
            runId: input.runId,
            generation: input.generation,
            toolCallId: input.toolCallId,
            tool: input.tool,
            arguments: input.arguments,
        });
        const result = await hostClient.execute(intent);
        if (result.status === 'ok') {
            throw new AgentClientError(
                'approval_forbidden',
                'the host executed an intent that requires approval',
            );
        }
        const output = isPlainObject(result.output) ? result.output : {};
        const approvalId = output.approval_id;
        const expiresAtMs = output.expires_at_ms;
        if (
            result.code !== 'approval_forbidden' ||
            output.approval_required !== true ||
            typeof approvalId !== 'string' ||
            approvalId.length === 0 ||
            !Number.isSafeInteger(expiresAtMs) ||
            (expiresAtMs as number) <= this.now()
        ) {
            throw new AgentClientError(
                isAgentErrorCode(result.code)
                    ? result.code
                    : 'approval_forbidden',
                result.message ??
                    'the host did not issue an approval request',
            );
        }
        return { approvalId, expiresAtMs: expiresAtMs as number };
    }

    private async handleToolCall(
        current: RunControl,
        toolCall: {
            toolCallId: string;
            tool: string;
            arguments: unknown;
        },
    ): Promise<void> {
        if (current.terminal || current.cancelRequested) return;
        let args: Record<string, unknown>;
        try {
            args = canonicalToolArguments(toolCall.arguments);
        } catch (error) {
            await this.withTask(current.taskId, () =>
                this.finalizeFailed(current, {
                    code: 'invalid_arguments',
                    message: errorMessage(error),
                    retryable: false,
                }),
            );
            return;
        }

        const prepared = await this.withTask(current.taskId, async () => {
            if (current.terminal || current.cancelRequested) return null;
            const argumentsDigest = await sha256Canonical(args);
            const argumentsPreview = canonicalJson(args).slice(0, 512);
            await this.append(current, 'tool.requested', {
                tool_call_id: toolCall.toolCallId,
                tool: toolCall.tool,
                arguments_digest: argumentsDigest,
                arguments_preview: argumentsPreview,
            });
            let approvalId: string | undefined;
            let approvalPromise:
                | Promise<'approved' | 'denied' | 'cancelled'>
                | null = null;
            if (this.requiresApproval(toolCall.tool, args)) {
                const requestDigest = await sha256Canonical({
                    task_id: current.taskId,
                    run_id: current.runId,
                    generation: current.generation,
                    tool_call_id: toolCall.toolCallId,
                    tool: toolCall.tool,
                    arguments: args,
                });
                const hostApproval = await this.preflightHostApproval({
                    taskId: current.taskId,
                    runId: current.runId,
                    generation: current.generation,
                    toolCallId: toolCall.toolCallId,
                    tool: toolCall.tool,
                    arguments: args,
                });
                if (current.terminal || current.cancelRequested) return null;
                approvalId = hostApproval?.approvalId ?? this.ids.approvalId();
                const expiresAtMs =
                    hostApproval?.expiresAtMs ??
                    this.now() + this.approvalTtlMs;
                const record: ApprovalRecord = {
                    approvalId,
                    taskId: current.taskId,
                    runId: current.runId,
                    generation: current.generation,
                    toolCallId: toolCall.toolCallId,
                    tool: toolCall.tool,
                    scope: `tool:${toolCall.tool}`,
                    requestDigest,
                    expiresAtMs,
                    arguments: args,
                    hostApproval: hostApproval !== null,
                };
                current.approvals.set(approvalId, record);
                await this.append(current, 'tool.approval_required', {
                    approval_id: approvalId,
                    tool_call_id: toolCall.toolCallId,
                    tool: toolCall.tool,
                    scope: record.scope,
                    request_digest: requestDigest,
                    expires_at_ms: expiresAtMs,
                });
                approvalPromise = new Promise<
                    'approved' | 'denied' | 'cancelled'
                >((resolve) => {
                    current.pendingApproval = { record, resolve };
                });
            }
            return { approvalId, approvalPromise };
        });
        if (prepared === null) return;

        if (prepared.approvalPromise !== null) {
            const decision = await prepared.approvalPromise;
            if (current.terminal) return;
            if (decision !== 'approved') {
                await this.withTask(current.taskId, async () => {
                    if (current.terminal) return;
                    await this.append(current, 'tool.failed', {
                        tool_call_id: toolCall.toolCallId,
                        code:
                            decision === 'denied'
                                ? 'approval_denied'
                                : 'cancelled',
                        message:
                            decision === 'denied'
                                ? 'tool approval was denied'
                                : 'tool was cancelled',
                    });
                    if (current.cancelRequested) {
                        await this.finalizeCancelled(current);
                    }
                });
                return;
            }
        }

        const started = await this.withTask(current.taskId, async () => {
            if (current.terminal || current.cancelRequested) return false;
            await this.append(current, 'tool.started', {
                tool_call_id: toolCall.toolCallId,
            });
            current.inFlight = {
                toolCallId: toolCall.toolCallId,
                tool: toolCall.tool,
            };
            return true;
        });
        if (!started) return;

        let result: RuntimeToolResult;
        try {
            result = await this.toolExecutor({
                taskId: current.taskId,
                runId: current.runId,
                generation: current.generation,
                toolCallId: toolCall.toolCallId,
                tool: toolCall.tool,
                arguments: args,
                ...(prepared.approvalId === undefined
                    ? {}
                    : { approvalId: prepared.approvalId }),
            });
        } catch (error) {
            result = {
                status: 'error',
                output: {},
                code: errorCode(error),
                message: errorMessage(error),
            };
        }
        if (current.terminal) return;

        let output: unknown;
        let files: Array<Record<string, unknown>> | undefined;
        let headDigest: string | undefined;
        let invalidResult: string | null = null;
        try {
            if (
                result.status !== 'ok' &&
                result.status !== 'error' &&
                result.status !== 'cancelled'
            ) {
                throw new AgentClientError(
                    'invalid_arguments',
                    'tool result status is invalid',
                );
            }
            output = result.output ?? {};
            canonicalJson(output);
            if (result.files !== undefined || result.head_digest !== undefined) {
                if (
                    !Array.isArray(result.files) ||
                    typeof result.head_digest !== 'string' ||
                    result.head_digest.length === 0
                ) {
                    throw new AgentClientError(
                        'invalid_arguments',
                        'write result requires files and head_digest',
                    );
                }
                for (const file of result.files) {
                    if (!isPlainObject(file)) {
                        throw new AgentClientError(
                            'invalid_arguments',
                            'tool result file is not a canonical object',
                        );
                    }
                    canonicalJson(file);
                }
                files = cloneJson(result.files);
                headDigest = result.head_digest;
            }
        } catch (error) {
            invalidResult = errorMessage(error);
        }

        await this.withTask(current.taskId, async () => {
            if (current.terminal) return;
            if (invalidResult !== null) {
                current.inFlight = null;
                await this.append(current, 'tool.failed', {
                    tool_call_id: toolCall.toolCallId,
                    code: 'invalid_arguments',
                    message: invalidResult,
                });
                await this.finalizeFailed(current, {
                    code: 'invalid_arguments',
                    message: invalidResult,
                    retryable: false,
                });
                return;
            }
            const outputCanonical = canonicalJson(output);
            const outputDigest = await sha256Canonical(output);
            const outputPreview = outputCanonical.slice(0, 512);
            if (result.status === 'error') {
                await this.append(current, 'tool.failed', {
                    tool_call_id: toolCall.toolCallId,
                    code:
                        typeof result.code === 'string' && result.code.length > 0
                            ? result.code
                            : 'tool_error',
                    message:
                        typeof result.message === 'string'
                            ? result.message
                            : 'tool failed',
                });
            } else {
                await this.append(current, 'tool.completed', {
                    tool_call_id: toolCall.toolCallId,
                    status: result.status,
                    output_digest: outputDigest,
                    output_preview: outputPreview,
                    truncated: result.truncated === true,
                });
                if (
                    result.status === 'ok' &&
                    files !== undefined &&
                    headDigest !== undefined
                ) {
                    const capturedAtMs = this.now();
                    await this.append(current, 'changes.snapshot', {
                        scope: 'last_turn',
                        files,
                        head_digest: headDigest,
                        captured_at_ms: capturedAtMs,
                    });
                    if (this.taskStore !== null) {
                        await this.taskStore.recordChanges({
                            task_id: current.taskId,
                            run_id: current.runId,
                            head_digest: headDigest,
                            files,
                            captured_at_ms: capturedAtMs,
                            scope: 'last_turn',
                        });
                    }
                }
            }
            current.inFlight = null;
            if (current.cancelRequested) {
                await this.finalizeCancelled(current, toolCall.toolCallId);
                return;
            }
            await this.flushBoundary(current);
        });
    }

    // A restored run has no live RunControl, so the safe boundary is the
    // durable event chain itself: verify the current nonterminal run, take
    // the task lease, append cancelling then terminal cancelled through the
    // validated store sequence, and release the lease. No host worker is
    // claimed to have been cancelled.
    private async stopRestoredRun(input: StopInput): Promise<void> {
        const task = this.store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === input.taskId);
        if (task === undefined) {
            throw new AgentClientError('task_not_found', 'task is missing');
        }
        if (task.unresolvedRunId !== null) {
            throw new AgentClientError(
                'out_of_order',
                'the task latest run is still loading; wait for the replay before stopping it',
            );
        }
        if (task.latestRunId === null || task.latestRunId !== input.runId) {
            throw new AgentClientError(
                'out_of_order',
                'stop does not match the latest run',
            );
        }
        const run = task.runs.find(
            (candidate) => candidate.runId === input.runId,
        );
        if (run === undefined) {
            throw new AgentClientError('run_not_found', 'run is missing');
        }
        if (run.needsResync || isTerminalState(run.state)) {
            throw new AgentClientError(
                'out_of_order',
                'run is already terminal',
            );
        }
        if (this.taskStore === null) {
            throw new AgentClientError(
                'runtime_unavailable',
                'a restored run can only be stopped with the durable browser task store',
            );
        }
        try {
            const lease = await this.taskStore.acquireLease(
                task.taskId,
                this.writerId,
                this.now(),
            );
            this.heldLeases.add(task.taskId);
            if (lease.writer_id !== this.writerId) {
                throw new AgentClientError(
                    'stale_generation',
                    'task lease is held by another writer',
                );
            }
            const sequence = run.lastSequence + 1;
            await this.appendRestored(task, run, sequence, 'run.status', {
                status: 'cancelling',
            });
            await this.appendRestored(
                task,
                run,
                sequence + 1,
                'run.cancelled',
                {
                    safe_boundary: true,
                    finished_at_ms: this.now(),
                },
            );
            this.restoredApprovals = this.restoredApprovals.filter(
                (approval) => approval.runId !== run.runId,
            );
        } catch (error) {
            if (error instanceof AgentClientError) throw error;
            throw new AgentClientError(
                errorCode(error),
                errorMessage(error),
                false,
                error instanceof AgentStoreError ? error.storage : null,
            );
        } finally {
            await this.releaseLease(task.taskId);
        }
        if (task.hold !== null && task.hold.run_id === run.runId) {
            await this.setTaskHold(task.taskId, null);
        }
    }

    private async appendRestored(
        task: TaskView,
        run: RunView,
        sequence: number,
        type: string,
        data: Record<string, unknown>,
    ): Promise<void> {
        const event = {
            schema: AGENT_EVENT_SCHEMA,
            event_id: `evt_${run.runId}_${run.generation}_${sequence
                .toString()
                .padStart(12, '0')}`,
            project_id: task.projectId,
            task_id: task.taskId,
            run_id: run.runId,
            generation: run.generation,
            sequence,
            recorded_at_ms: this.now(),
            type,
            data,
        } as unknown as AgentIncomingEvent;
        const result = await this.taskStore!.appendEvent({
            event,
            writer_id: this.writerId,
            now_ms: this.now(),
        });
        if (result.status === 'rejected') {
            throw new AgentClientError(
                result.code ?? 'internal',
                result.message ?? 'durable restored stop was rejected',
                false,
                result.storage ?? null,
            );
        }
        if (result.status === 'applied') {
            this.publish(event, result.digest ?? undefined);
        }
    }

    private async stopTimeout(taskId: string, runId: string): Promise<void> {
        await this.withTask(taskId, async () => {
            const current = this.active.get(taskId);
            if (
                current === undefined ||
                current.runId !== runId ||
                current.terminal
            ) {
                return;
            }
            const toolCallId = current.inFlight?.toolCallId;
            if (toolCallId === undefined) {
                await this.finalizeCancelled(current);
                return;
            }
            current.stopTimer = null;
            await this.setTaskHold(taskId, {
                reason: 'waiting_for_background',
                run_id: runId,
                tool_call_id: toolCallId,
            });
            await this.finalizeFailed(current, {
                code: 'stop_unacknowledged',
                message: 'host did not acknowledge the stop request',
                retryable: false,
                stop_outcome: 'ack_timeout',
                in_flight_tool_id: toolCallId,
            });
        });
    }

    private async stopRefused(taskId: string, runId: string): Promise<void> {
        await this.withTask(taskId, async () => {
            const current = this.active.get(taskId);
            if (
                current === undefined ||
                current.runId !== runId ||
                current.terminal
            ) {
                return;
            }
            const toolCallId = current.inFlight?.toolCallId;
            if (toolCallId === undefined) {
                await this.finalizeCancelled(current);
                return;
            }
            if (current.stopTimer !== null) {
                this.timers.clearTimeout(current.stopTimer);
                current.stopTimer = null;
            }
            await this.finalizeFailed(current, {
                code: 'host_denied',
                message: 'host refused the stop request',
                retryable: false,
                stop_outcome: 'host_refused',
                in_flight_tool_id: toolCallId,
            });
            current.stopDeferred?.resolve(false);
            current.stopDeferred = null;
        });
    }

    private async finalizeFinished(current: RunControl): Promise<void> {
        if (current.terminal) return;
        await this.flushFallback(current);
        await this.append(current, 'run.finished', {
            status: 'finished',
            finished_at_ms: this.now(),
            duration_ms: Math.max(0, this.now() - current.startedAtMs),
        });
        await this.afterTerminal(current, {
            startQueued: true,
            stopAcknowledged: true,
        });
    }

    private async finalizeCancelled(
        current: RunControl,
        toolCallId?: string,
    ): Promise<boolean> {
        if (current.terminal) return true;
        try {
            await this.failUndeliveredMessages(current, 'cancelled');
            if (current.stopTimer !== null) {
                this.timers.clearTimeout(current.stopTimer);
                current.stopTimer = null;
            }
            await this.append(current, 'run.cancelled', {
                safe_boundary: true,
                finished_at_ms: this.now(),
                ...(toolCallId === undefined
                    ? {}
                    : { stopped_at_tool_id: toolCallId }),
            });
            await this.afterTerminal(current, {
                startQueued: false,
                stopAcknowledged: true,
            });
            return true;
        } catch (error) {
            this.recordPersistenceError(error);
            try {
                await this.settleControlWithoutEvent(current, false);
            } catch (settleError) {
                this.recordPersistenceError(settleError);
            } finally {
                // The failed append advanced the private sequence counter;
                // this control is discarded so no later append can reuse it.
                // A durable run.status=cancelling row is retained honestly:
                // no synthetic terminal is written, and a reload keeps the run
                // blocked until a verified recovery rather than claiming a
                // completion that never committed.
                current.stopDeferred?.resolve(false);
                current.stopDeferred = null;
                this.taskQueues.delete(current.taskId);
            }
            return false;
        }
    }

    private async finalizeFailed(
        current: RunControl,
        data: {
            code: string;
            message: string;
            retryable: boolean;
            stop_outcome?: 'ack_timeout' | 'host_refused';
            in_flight_tool_id?: string;
        },
    ): Promise<void> {
        if (current.terminal) return;
        await this.failUndeliveredMessages(current, 'failed');
        if (current.stopTimer !== null) {
            this.timers.clearTimeout(current.stopTimer);
            current.stopTimer = null;
        }
        await this.append(current, 'run.failed', data);
        await this.afterTerminal(current, {
            startQueued: false,
            stopAcknowledged: false,
        });
    }

    private async failUndeliveredMessages(
        current: RunControl,
        reason: string,
    ): Promise<void> {
        const pending = current.pendingBoundary.splice(0);
        for (const message of pending) {
            this.timers.clearTimeout(message.timer);
            await this.appendUndeliveredMessage(current, message.messageId, reason);
        }
        const queued = this.taskQueues.get(current.taskId) ?? [];
        this.taskQueues.delete(current.taskId);
        for (const message of queued) {
            await this.appendUndeliveredMessage(current, message.messageId, reason);
        }
    }

    private async appendUndeliveredMessage(
        current: RunControl,
        messageId: string,
        reason: string,
    ): Promise<void> {
        if (current.terminal) return;
        try {
            await this.append(current, 'error', {
                code: 'message_not_delivered',
                message: `message ${messageId} was not delivered (${reason})`,
                retryable: false,
            });
        } catch (error) {
            this.persistenceError = new AgentClientError(
                errorCode(error),
                errorMessage(error),
                false,
                error instanceof AgentStoreError ? error.storage : null,
            );
        }
    }

    private async recoverStopFailure(
        taskId: string,
        runId: string,
        error: unknown,
    ): Promise<void> {
        this.persistenceError = new AgentClientError(
            errorCode(error),
            errorMessage(error),
            false,
            error instanceof AgentStoreError ? error.storage : null,
        );
        try {
            await this.withTask(taskId, async () => {
                const current = this.active.get(taskId);
                if (
                    current === undefined ||
                    current.runId !== runId ||
                    current.terminal
                ) {
                    return;
                }
                await this.settleControlWithoutEvent(current, false);
            });
        } catch (recoveryError) {
            this.persistenceError = new AgentClientError(
                errorCode(recoveryError),
                errorMessage(recoveryError),
                false,
                recoveryError instanceof AgentStoreError
                    ? recoveryError.storage
                    : null,
            );
        }
    }

    private async recoverPendingFailure(
        taskId: string,
        messageId: string,
        error: unknown,
    ): Promise<void> {
        this.persistenceError = new AgentClientError(
            errorCode(error),
            errorMessage(error),
            false,
            error instanceof AgentStoreError ? error.storage : null,
        );
        try {
            await this.withTask(taskId, async () => {
                const current = this.active.get(taskId);
                if (current === undefined || current.terminal) return;
                const index = current.pendingBoundary.findIndex(
                    (pending) => pending.messageId === messageId,
                );
                if (index < 0) return;
                const [pending] = current.pendingBoundary.splice(index, 1);
                this.timers.clearTimeout(pending!.timer);
            });
        } catch {
            // The receipt for this message was already returned; the pending
            // delivery is removed and the persistence error is retained.
        }
    }

    private async settleControlWithoutEvent(
        current: RunControl,
        stopAcknowledged: boolean,
    ): Promise<void> {
        if (current.terminal) return;
        const record = (error: unknown): void =>
            this.recordPersistenceError(error);
        try {
            await this.failAcceptedMessages(current, record, 'cancelled');
        } catch (error) {
            record(error);
        }
        current.terminal = true;
        current.cancelRequested = true;
        try {
            this.clearTimersSafely(current, record);
            current.pendingApproval?.resolve('cancelled');
            current.pendingApproval = null;
            const iterator = current.modelIterator;
            current.modelIterator = null;
            try {
                await iterator?.return?.();
            } catch (error) {
                record(error);
            }
        } finally {
            if (this.active.get(current.taskId) === current) {
                this.active.delete(current.taskId);
            }
            current.stopDeferred?.resolve(stopAcknowledged);
            current.stopDeferred = null;
            this.taskQueues.delete(current.taskId);
            await this.releaseLease(current.taskId);
        }
    }

    private async afterTerminal(
        current: RunControl,
        options: { startQueued: boolean; stopAcknowledged: boolean },
    ): Promise<void> {
        current.terminal = true;
        current.cancelRequested = true;
        this.clearTimers(current);
        current.pendingApproval?.resolve('cancelled');
        current.pendingApproval = null;
        const iterator = current.modelIterator;
        current.modelIterator = null;
        try {
            await iterator?.return?.();
        } catch (error) {
            if (this.persistenceError === null) {
                this.persistenceError = new AgentClientError(
                    errorCode(error),
                    errorMessage(error),
                );
            }
        }
        if (this.active.get(current.taskId) === current) {
            this.active.delete(current.taskId);
        }
        current.stopDeferred?.resolve(options.stopAcknowledged);
        current.stopDeferred = null;
        await this.releaseLease(current.taskId);
        if (!options.startQueued) {
            this.taskQueues.delete(current.taskId);
            return;
        }
        const next = this.taskQueues.get(current.taskId)?.shift();
        if (next === undefined) {
            this.taskQueues.delete(current.taskId);
            return;
        }
        try {
            await this.startRunInternal(
                { taskId: current.taskId, prompt: next.text },
                null,
            );
        } catch (error) {
            this.persistenceError = new AgentClientError(
                errorCode(error),
                errorMessage(error),
            );
        }
    }

    private async failAcceptedMessages(
        current: RunControl,
        record: (error: unknown) => void,
        reason: 'disposed' | 'cancelled' = 'disposed',
    ): Promise<void> {
        if (current.terminal) return;
        const pending = current.pendingBoundary.splice(0);
        for (const message of pending) {
            try {
                this.timers.clearTimeout(message.timer);
            } catch (error) {
                record(error);
            }
            try {
                await this.appendUndeliveredMessage(
                    current,
                    message.messageId,
                    reason,
                );
            } catch (error) {
                record(error);
            }
        }
        const queued = this.taskQueues.get(current.taskId) ?? [];
        this.taskQueues.delete(current.taskId);
        for (const message of queued) {
            try {
                await this.appendUndeliveredMessage(
                    current,
                    message.messageId,
                    reason,
                );
            } catch (error) {
                record(error);
            }
        }
    }

    private clearTimersSafely(
        current: RunControl,
        record: (error: unknown) => void,
    ): void {
        if (current.stopTimer !== null) {
            try {
                this.timers.clearTimeout(current.stopTimer);
            } catch (error) {
                record(error);
            }
            current.stopTimer = null;
        }
        for (const pending of current.pendingBoundary.splice(0)) {
            try {
                this.timers.clearTimeout(pending.timer);
            } catch (error) {
                record(error);
            }
        }
    }

    private clearTimers(current: RunControl): void {
        if (current.stopTimer !== null) {
            this.timers.clearTimeout(current.stopTimer);
            current.stopTimer = null;
        }
        this.clearPendingBoundary(current);
    }

    private appendSerialized(
        current: RunControl,
        type: string,
        data: Record<string, unknown>,
    ): Promise<AgentIncomingEvent> {
        return this.withTask(current.taskId, () =>
            this.append(current, type, data),
        );
    }

    private async append(
        current: RunControl,
        type: string,
        data: Record<string, unknown>,
        options: { snapshotAnchor?: TaskSnapshot } = {},
    ): Promise<AgentIncomingEvent> {
        if (current.terminal) {
            throw new AgentClientError(
                'out_of_order',
                'no event may follow a terminal run event',
            );
        }
        if (this.disposed) {
            throw new AgentClientError(
                'runtime_unavailable',
                'the browser runtime is disposed',
            );
        }
        const sequence = current.sequence;
        const event = {
            schema: AGENT_EVENT_SCHEMA,
            event_id: `evt_${current.runId}_${current.generation}_${sequence
                .toString()
                .padStart(12, '0')}`,
            project_id: current.projectId,
            task_id: current.taskId,
            run_id: current.runId,
            generation: current.generation,
            sequence,
            recorded_at_ms: this.now(),
            type,
            data,
        } as unknown as AgentIncomingEvent;
        current.sequence += 1;
        const result =
            this.taskStore === null
                ? await this.store.appendEvent(event)
                : await this.taskStore.appendEvent({
                      event,
                      writer_id: this.writerId,
                      ...(options.snapshotAnchor === undefined
                          ? {}
                          : {
                                snapshot_anchor: {
                                    task_id: options.snapshotAnchor.taskId,
                                    run_id: options.snapshotAnchor.runId,
                                    generation:
                                        options.snapshotAnchor.generation,
                                    last_sequence:
                                        options.snapshotAnchor.lastSequence,
                                    last_event_id:
                                        options.snapshotAnchor.lastEventId,
                                    head_digest:
                                        options.snapshotAnchor.headDigest,
                                },
                            }),
                  });
        if (result.status === 'rejected') {
            this.persistenceError = new AgentClientError(
                result.code ?? 'internal',
                result.message ?? 'durable append was rejected',
                false,
                result.storage ?? null,
            );
            throw this.persistenceError;
        }
        this.persistenceError = null;
        if (result.status === 'applied') {
            this.publish(event, result.digest ?? undefined);
        }
        return event;
    }

    private async appendResume(
        current: RunControl,
        previousGeneration: number,
        snapshot: TaskSnapshot,
    ): Promise<AgentIncomingEvent> {
        const sequence = current.sequence;
        const event = {
            schema: AGENT_EVENT_SCHEMA,
            event_id: `evt_${current.runId}_${previousGeneration + 1}_${sequence
                .toString()
                .padStart(12, '0')}`,
            project_id: current.projectId,
            task_id: current.taskId,
            run_id: current.runId,
            generation: previousGeneration + 1,
            sequence,
            recorded_at_ms: this.now(),
            type: 'run.resumed',
            data: {
                resumed_at_ms: this.now(),
                previous_generation: previousGeneration,
                snapshot_digest: snapshot.headDigest,
            },
        } as unknown as AgentIncomingEvent;
        current.sequence += 1;
        const result =
            this.taskStore === null
                ? await this.store.appendEvent(event)
                : await this.taskStore.appendEvent({
                      event,
                      writer_id: this.writerId,
                      snapshot_anchor: {
                          task_id: snapshot.taskId,
                          run_id: snapshot.runId,
                          generation: snapshot.generation,
                          last_sequence: snapshot.lastSequence,
                          last_event_id: snapshot.lastEventId,
                          head_digest: snapshot.headDigest,
                      },
                  });
        if (result.status === 'rejected') {
            throw new AgentClientError(
                result.code ?? 'internal',
                result.message ?? 'resume append was rejected',
                true,
                result.storage ?? null,
            );
        }
        current.generation = previousGeneration + 1;
        if (result.status === 'applied') {
            this.publish(event, result.digest ?? undefined);
        }
        return event;
    }

    private async resolveApprovalRecord(
        record: ApprovalRecord | null,
        decision: 'approved' | 'denied',
        requestDigest: string,
        identity: { taskId: string; runId: string; generation: number },
    ): Promise<void> {
        if (record === null) {
            throw new AgentClientError(
                'approval_forbidden',
                'approval is not pending',
            );
        }
        const current = this.active.get(record.taskId);
        if (
            current === undefined ||
            current.terminal ||
            current.runId !== record.runId ||
            current.generation !== record.generation
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'approval does not belong to the active run',
            );
        }
        const pending = current.pendingApproval;
        if (
            pending === null ||
            pending.record.approvalId !== record.approvalId ||
            current.approvals.get(record.approvalId) !== record
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'approval is no longer pending',
            );
        }
        if (
            identity.taskId !== record.taskId ||
            identity.runId !== record.runId ||
            identity.generation !== record.generation
        ) {
            throw new AgentClientError(
                'approval_forbidden',
                'approval identity does not match task, run and generation',
            );
        }
        let expectedDigest: string;
        try {
            expectedDigest = await sha256Canonical({
                task_id: record.taskId,
                run_id: record.runId,
                generation: record.generation,
                tool_call_id: record.toolCallId,
                tool: record.tool,
                arguments: record.arguments,
            });
        } catch (error) {
            throw new AgentClientError(
                'approval_digest_mismatch',
                errorMessage(error),
            );
        }
        if (
            requestDigest !== expectedDigest ||
            record.requestDigest !== expectedDigest
        ) {
            throw new AgentClientError(
                'approval_digest_mismatch',
                'approval request digest does not match the tool request',
            );
        }
        if (this.now() > record.expiresAtMs) {
            current.pendingApproval = null;
            current.approvals.delete(record.approvalId);
            pending.resolve('cancelled');
            await this.append(current, 'tool.approval_resolved', {
                approval_id: record.approvalId,
                decision: 'expired',
                resolver: 'host',
            });
            await this.finalizeFailed(current, {
                code: 'approval_expired',
                message: 'approval has expired',
                retryable: false,
            });
            throw new AgentClientError(
                'approval_expired',
                'approval has expired',
            );
        }
        await this.append(current, 'tool.approval_resolved', {
            approval_id: record.approvalId,
            decision,
            resolver: 'host',
        });
        current.pendingApproval = null;
        current.approvals.delete(record.approvalId);
        pending.resolve(decision);
    }

    private findApproval(approvalId: string): ApprovalRecord | null {
        for (const current of this.active.values()) {
            const record = current.approvals.get(approvalId);
            if (record !== undefined) return record;
        }
        return null;
    }

    private async clearTerminalHold(
        task: AgentTaskView,
        action: string,
    ): Promise<void> {
        if (task.hold === null) return;
        const held = this.store.getRun(task.taskId, task.hold.run_id);
        if (held === null) {
            await this.setTaskHold(task.taskId, null);
            return;
        }
        if (!isTerminalState(held.state) || this.isDesignedBackgroundHold(held)) {
            throw new AgentClientError(
                'out_of_order',
                `background hold must be resolved before ${action}`,
            );
        }
        await this.setTaskHold(task.taskId, null);
    }

    private async setTaskHold(
        taskId: string,
        hold: {
            reason: 'waiting_for_background';
            run_id: string;
            tool_call_id: string;
        } | null,
    ): Promise<void> {
        if (this.taskStore !== null) {
            const records = await this.taskStore.listTasks({
                includeArchived: true,
            });
            const record = records.find(
                (candidate) => candidate.task_id === taskId,
            );
            if (record !== undefined) {
                await this.taskStore.writeTaskRecord({ ...record, hold });
            }
            this.upsertStoreHold(taskId, hold);
            return;
        }
        this.upsertStoreHold(taskId, hold);
    }

    private upsertStoreHold(
        taskId: string,
        hold: {
            reason: 'waiting_for_background';
            run_id: string;
            tool_call_id: string;
        } | null,
    ): void {
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === taskId,
        );
        if (task === undefined) return;
        this.store.upsertTask({
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            repo: task.repo,
            branch: task.branch,
            createdAtMs: task.createdAtMs,
            archived: task.archived,
            pinnedLocal: task.pinnedLocal,
            latestRunId: task.latestRunId,
            hold,
        });
    }

    private tickHeartbeat(): void {
        if (this.taskStore === null || this.heartbeatInFlight) return;
        this.heartbeatInFlight = true;
        this.heartbeatTask = this.heartbeat()
            .catch((error) => {
                this.persistenceError = new AgentClientError(
                    errorCode(error),
                    errorMessage(error),
                );
            })
            .finally(() => {
                this.heartbeatInFlight = false;
            });
    }

    private async heartbeat(): Promise<void> {
        if (this.taskStore === null) return;
        for (const taskId of [...this.heldLeases]) {
            try {
                await this.taskStore.heartbeatLease(
                    taskId,
                    this.writerId,
                    this.now(),
                );
            } catch (error) {
                this.persistenceError = new AgentClientError(
                    errorCode(error),
                    errorMessage(error),
                    false,
                    error instanceof AgentStoreError
                        ? error.storage
                        : null,
                );
            }
        }
    }

    private async releaseLease(taskId: string): Promise<void> {
        if (this.taskStore === null) return;
        if (!this.heldLeases.has(taskId)) return;
        this.heldLeases.delete(taskId);
        try {
            await this.taskStore.releaseLease(taskId, this.writerId);
        } catch (error) {
            this.persistenceError = new AgentClientError(
                errorCode(error),
                errorMessage(error),
                false,
                error instanceof AgentStoreError ? error.storage : null,
            );
        }
    }

    private withTask<T>(taskId: string, operation: () => Promise<T>): Promise<T> {
        const previous = this.chains.get(taskId) ?? Promise.resolve();
        const next = previous.then(operation, operation);
        this.chains.set(
            taskId,
            next.then(
                () => undefined,
                () => undefined,
            ),
        );
        return next;
    }

    private assertOpen(): void {
        if (this.disposing) {
            throw new AgentClientError(
                'runtime_unavailable',
                'the browser runtime is disposing',
            );
        }
        if (this.disposed) {
            throw new AgentClientError(
                'runtime_unavailable',
                'the browser runtime is disposed',
            );
        }
        if (this.taskStore !== null && !this.opened) {
            throw new AgentClientError(
                'runtime_unavailable',
                'durable task store is not open; await AgentRuntime.open() first',
            );
        }
    }

    private requireTask(taskId: string): AgentTaskView {
        const task = this.requireTaskView(taskId);
        if (task.archived) {
            throw new AgentClientError('task_archived', 'task is archived');
        }
        return task;
    }

    private requireTaskView(taskId: string): AgentTaskView {
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === taskId,
        );
        if (task === undefined) {
            throw new AgentClientError('task_not_found', 'task is missing');
        }
        return task;
    }

    private requireSummary(taskId: string): TaskSummary {
        const summary = this.summaryFor(taskId);
        if (summary === null) {
            throw new AgentClientError('task_not_found', 'task is missing');
        }
        return summary;
    }

    private summaryFor(taskId: string): TaskSummary | null {
        const task = this.store.snapshot().tasks.find(
            (candidate) => candidate.taskId === taskId,
        );
        if (task === undefined) return null;
        return {
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            repo: task.repo,
            branch: task.branch,
            createdAtMs: task.createdAtMs,
            archived: task.archived,
            pinned: task.pinnedLocal,
            latestRunId: task.latestRunId,
            hold: task.hold,
            unresolvedRunId: task.unresolvedRunId,
            latestRunLoaded:
                task.latestRunId === null ||
                task.runs.some((run) => run.runId === task.latestRunId),
            state: task.state,
        };
    }

    private rememberPrompt(runId: string, prompt: string): void {
        this.promptByRun.set(runId, prompt);
        while (this.promptByRun.size > MAX_PROMPT_HISTORY) {
            const oldest = this.promptByRun.keys().next().value;
            if (oldest === undefined || oldest === runId) break;
            this.promptByRun.delete(oldest);
        }
    }

    private requireText(text: string, label: string): void {
        if (typeof text !== 'string' || text.trim().length === 0) {
            throw new AgentClientError(
                'invalid_arguments',
                `${label} must not be empty`,
            );
        }
    }

    private ensureQueueCapacity(current: RunControl): void {
        const queued =
            (this.taskQueues.get(current.taskId)?.length ?? 0) +
            current.pendingBoundary.length;
        if (queued >= this.maxQueuedMessages) {
            throw new AgentClientError(
                'limit_exceeded',
                'the task message queue is full',
            );
        }
    }

    private queueFor(taskId: string): QueuedTurn[] {
        let queue = this.taskQueues.get(taskId);
        if (queue === undefined) {
            queue = [];
            this.taskQueues.set(taskId, queue);
        }
        return queue;
    }

    private publish(event: AgentIncomingEvent, digest?: string): void {
        for (const entry of this.listeners) {
            if (
                entry.filter.taskId !== undefined &&
                entry.filter.taskId !== event.task_id
            ) {
                continue;
            }
            if (
                entry.filter.runId !== undefined &&
                entry.filter.runId !== event.run_id
            ) {
                continue;
            }
            entry.listener(event, digest);
        }
        for (const stream of this.eventStreams) {
            stream.push(event);
        }
    }
}

type AgentTaskView = TaskView;
