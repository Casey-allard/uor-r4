import { AgentClient, AgentClientError, type TaskSummary } from './client.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Bytes,
    sha256Canonical,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from './hostClient.js';
import { FixtureTransport } from './fixtureTransport.js';
import { AgentTaskStore } from './store.js';
import { AgentRuntime } from './runtime.js';
import {
    type ModelPort,
    type ModelRequest,
    type ModelStreamEvent,
} from './modelPort.js';
import { AgentStoreError } from './storeTypes.js';
import { AGENT_EVENT_SCHEMA } from './generated/agentContract.js';
import { canonicalJson } from './generated/agentCanonical.js';
import {
    createRunQueuedEvent,
    createRunStartedEvent,
    fixtureEventId,
} from './fixtures.js';
import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
    type RunView,
    type TaskView,
} from '../state/agent.js';

const TERMINAL_STATES = new Set(['finished', 'failed', 'cancelled', 'expired']);

function base64Bytes(bytes: Uint8Array): string {
    let binary = '';
    for (const byte of bytes) binary += String.fromCharCode(byte);
    return btoa(binary);
}

class RecordingArtifactHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'none',
            tree: false,
            changes: false,
            readArtifact: true,
            terminal: false,
            git: false,
            mcp: false,
            approvals: false,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        this.intents.push(intent);
        const bytes = new TextEncoder().encode('host artifact');
        const digest = await sha256Bytes(bytes);
        const output = {
            path: 'host.txt',
            media_type: 'text/plain',
            size_bytes: bytes.byteLength,
            artifact_size_bytes: bytes.byteLength,
            digest,
            whole_digest: digest,
            range: { start: 0, end: bytes.byteLength },
            bytes_base64: base64Bytes(bytes),
        };
        return {
            schema: HOST_RESULT_SCHEMA,
            intent_id: intent.intent_id,
            task_id: intent.task_id,
            run_id: intent.run_id,
            generation: intent.generation,
            tool_call_id: intent.tool_call_id,
            request_scope: intent.request_scope ?? 'task',
            status: 'ok',
            output,
            output_digest: await sha256Canonical(output),
            output_preview: '',
            occurred_at_ms: 1_800_000_000_000,
        };
    }

    subscribe() {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }) {
        return { approvalId: input.approvalId, decision: input.decision };
    }
}

interface StoreFaults {
    abort_next_append: boolean;
    abort_next_task_write: boolean;
    fail_import_publish: boolean;
    fail_import_cleanup: boolean;
}

interface FixtureSummary {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    eventIds: string[];
    digests: string[];
}

interface CursorView {
    taskId: string;
    runId: string;
    generation: number;
    sequence: number;
    eventId: string;
    digest: string;
    runState: string;
    statusProjection: string | null;
    taskState: string;
    messageCount: number;
    lastMessageText: string | null;
}

interface ActiveRunView {
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string | null;
    state: string;
    statusProjection: string | null;
}

interface RestoreView {
    phase: 'idle' | 'loading' | 'ready' | 'incomplete' | 'error';
    sendEnabled: boolean;
    canStartRun: boolean;
    error: { code: string; message: string } | null;
    incomplete: Array<{
        taskId: string | null;
        runId: string | null;
        code: string;
        message: string;
    }>;
    activeRun: ActiveRunView | null;
}

interface TaskViewSummary {
    taskId: string;
    title: string;
    pinned: boolean;
    archived: boolean;
    state: string;
    latestRunId: string | null;
    unresolvedRunId: string | null;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
    runs: Array<{ runId: string; lastSequence: number; state: string }>;
}

interface Outcome {
    ok: boolean;
    code?: string;
    message?: string;
    status?: string;
    storage?: string | null;
    record?: unknown;
    records?: unknown;
}

interface LiveTaskSeed {
    taskId: string;
    runId: string;
    generation: number;
    title: string;
    prompt: string;
}

interface LiveRunSnapshot {
    taskId: string;
    title: string;
    taskState: string;
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null;
    latestRunId: string | null;
    runId: string | null;
    retryOfRunId: string | null;
    generation: number | null;
    lastSequence: number | null;
    lastEventId: string | null;
    lastDigest: string | null;
    runState: string | null;
    messages: string[];
    changes: Array<{
        files: Array<Record<string, unknown>>;
        headDigest: string;
    }>;
    approvals: Array<{
        approvalId: string;
        toolCallId: string;
        requestDigest: string;
        expiresAtMs: number;
        decision: string | null;
    }>;
    unknownEvents: number;
}

interface ReloadTaskSeed {
    taskId: string;
    runId: string;
    generation: number;
    title: string;
    message: string;
    files: Array<Record<string, unknown>>;
    headDigest: string;
    snapshot: {
        generation: number;
        lastSequence: number;
        lastEventId: string;
        headDigest: string;
    };
}

export interface StudioAgentOwnerForTest {
    client: AgentClient;
    store: AgentStore;
    taskStore: AgentTaskStore | null;
    runtime: AgentRuntime | null;
    modelPort?: ModelPort;
}

export interface StudioTestApi {
    reset(): Promise<void>;
    clientForTest(): AgentClient | null;
    artifactFallbackProbeForTest(artifactId: string): Promise<Outcome>;
    open(): Promise<RestoreView>;
    reopen(): Promise<RestoreView>;
    memoryCursor(): CursorView | null;
    beginOpen(): Promise<void>;
    whenOpen(): Promise<RestoreView>;
    restoreState(): RestoreView;
    createTaskAndEvents(options?: {
        mode?: 'message' | 'cancelling' | 'finished';
    }): Promise<FixtureSummary>;
    createSecondRun(): Promise<FixtureSummary>;
    createClientRun(): Promise<FixtureSummary>;
    createQueuedOnlyTask(): Promise<FixtureSummary>;
    targetFixture(taskId: string, runId: string, generation?: number): void;
    listUnloadedActiveTask(mode: 'running' | 'cancelling'): Promise<Outcome>;
    listActiveSummary(
        taskId: string,
        latestRunId: string | null,
        mode: 'running' | 'cancelling',
    ): Promise<Outcome>;
    appendQueuedSafe(taskId: string, runId: string): Promise<Outcome>;
    createTaskAt(createdAtMs: number): Promise<Outcome>;
    resumeViaClient(): Promise<Outcome>;
    recordChangesWithFiles(files: Array<Record<string, unknown>>): Promise<Outcome>;
    seedSmallEvents(total: number): Promise<{
        events: number;
        sequence: number;
        durationMs: number;
        heapUsed: number | null;
    }>;
    scaleMetrics(): {
        events: number;
        sequence: number;
        durationMs: number;
        heapUsed: number | null;
    } | null;
    cursor(): CursorView | null;
    taskView(): TaskViewSummary | null;
    writeSnapshot(taskId?: string, runId?: string): Promise<Outcome>;
    readSnapshot(): Promise<Outcome>;
    appendChanges(): Promise<Outcome>;
    finishFixtureForTest(): Promise<Outcome>;
    appendResume(): Promise<Outcome>;
    recordChanges(headDigest?: string): Promise<Outcome>;
    readChanges(): Promise<Outcome>;
    pin(pinned: boolean): Promise<Outcome>;
    archive(archived: boolean): Promise<Outcome>;
    seedBackgroundHoldForTest(): Promise<Outcome>;
    startRunOnHeldTaskForTest(taskId: string): Promise<Outcome>;
    resolveBackgroundHoldForTest(
        taskId: string,
        runId: string,
        toolCallId: string,
    ): Promise<Outcome>;
    startTwoLiveTasksForTest(): Promise<Outcome>;
    queueTaskMessageForTest(taskId: string, text: string): Promise<Outcome>;
    liveTextForTest(runId: string, text: string): Promise<Outcome>;
    failLiveRunForTest(runId: string): Promise<Outcome>;
    liveToolForTest(runId: string, path: string): Promise<Outcome>;
    finishLiveRunForTest(runId: string): Promise<Outcome>;
    resyncLiveRunForTest(runId: string): Promise<Outcome>;
    ingestHostileForTest(
        runId: string,
        mode:
            | 'malformed'
            | 'partial'
            | 'utf8_split'
            | 'unknown'
            | 'gap'
            | 'cross_task'
            | 'duplicate'
            | 'forged_duplicate',
    ): Promise<Outcome>;
    eventsForTest(taskId: string, runId: string): Outcome;
    ingestTerminalForTest(
        runId: string,
        type: 'run.finished' | 'run.failed' | 'run.cancelled' | 'run.expired',
    ): Promise<Outcome>;
    ingestThinkingHeartbeatForTest(runId: string): Promise<Outcome>;
    lateToolForTest(runId: string, path: string): Promise<Outcome>;
    ownerIdentityForTest(): {
        clientAdopted: boolean;
        runtimeAdopted: boolean;
    };
    seedTwoFinishedTasksForTest(): Promise<Outcome>;
    taskSnapshotForTest(taskId: string): Outcome;
    verifyRunForTest(taskId: string, runId: string): Promise<Outcome>;
    seedPendingApprovalForTest(): Promise<Outcome>;
    pendingApprovalsForTest(taskId: string): Promise<Outcome>;
    resolveRestoredApprovalForTest(approvalId: string): Promise<Outcome>;
    seedFailedRunForRetryProbe(): Promise<Outcome>;
    retryRestoredRunForTest(taskId: string, runId: string): Promise<Outcome>;
    clearHoldNullForTest(taskId: string): Promise<Outcome>;
    appendCollidingRunQueuedForTest(
        taskId: string,
        writerId: string,
    ): Promise<Outcome>;
    acquireLease(writerId: string, nowMs?: number): Promise<Outcome>;
    heartbeatLease(writerId: string, nowMs?: number): Promise<Outcome>;
    releaseLease(writerId: string): Promise<Outcome>;
    lease(): Promise<Outcome>;
    armAbortNextAppend(): void;
    armAbortNextTaskWrite(): void;
    armFailImportPublish(): void;
    armFailImportCleanup(): void;
    disposeDuringImport(): void;
    beginHeldQueuedAppend(taskId: string, runId: string): Promise<void>;
    awaitAppendHeld(): Promise<void>;
    releaseHeldAppend(): void;
    finishHeldAppend(): Promise<Outcome>;
    beginHeldTaskWrite(taskId: string): Promise<void>;
    awaitTaskWriteHeld(): Promise<void>;
    releaseTaskWrite(): void;
    finishHeldTaskWrite(): Promise<Outcome>;
    beginImport(text: string, options?: { holdPublish?: boolean }): void;
    awaitImportPublishGate(): Promise<void>;
    releaseImportPublishGate(): void;
    whenImport(): Promise<Outcome>;
    advanceImportClock(ms: number): Promise<Outcome>;
    freezeImportClock(ms: number): Promise<Outcome>;
    appendAssistantSafe(
        text: string,
        options?: { writerId?: string; nowMs?: number },
    ): Promise<Outcome>;
    appendAssistant(
        text: string,
        options?: { writerId?: string; nowMs?: number },
    ): Promise<Outcome>;
    exportNdjson(): Promise<string>;
    importNdjson(text: string): Promise<Outcome>;
}

interface TestState {
    client: AgentClient | null;
    taskStore: AgentTaskStore | null;
    store: AgentStore | null;
    runtime: AgentRuntime | null;
    adoptedClient: AgentClient | null;
    adoptedRuntime: AgentRuntime | null;
    liveModel: GatedTestModelPort | null;
    liveRuns: Map<string, { taskId: string; runId: string }>;
    nowMs: number;
    phase: RestoreView['phase'];
    error: { code: string; message: string } | null;
    incomplete: RestoreView['incomplete'];
    fixture: FixtureSummary | null;
    openPromise: Promise<void> | null;
    openGate: (() => void) | null;
    faults: StoreFaults;
    scale: {
        events: number;
        sequence: number;
        durationMs: number;
        heapUsed: number | null;
    } | null;
    importPromise: Promise<Outcome> | null;
    importGateRelease: (() => void) | null;
    appendGateRelease: (() => void) | null;
    appendGateReached: Promise<void> | null;
    heldAppend: Promise<Outcome> | null;
    taskWriteGateRelease: (() => void) | null;
    taskWriteGateReached: Promise<void> | null;
    heldTaskWrite: Promise<Outcome> | null;
    importPublishGateRelease: (() => void) | null;
    importPublishGateReached: Promise<void> | null;
    timers: TestTimer[];
    nextTimerHandle: number;
}

interface TestTimer {
    handle: number;
    periodMs: number;
    nextAtMs: number;
    callback: () => void | Promise<void>;
}

let state: TestState = freshState();

function freshState(): TestState {
    return {
        client: null,
        taskStore: null,
        store: null,
        runtime: null,
        adoptedClient: null,
        adoptedRuntime: null,
        liveModel: null,
        liveRuns: new Map(),
        nowMs: 1_794_283_200_000,
        phase: 'idle',
        error: null,
        incomplete: [],
        fixture: null,
        openPromise: null,
        openGate: null,
        faults: {
            abort_next_append: false,
            abort_next_task_write: false,
            fail_import_publish: false,
            fail_import_cleanup: false,
        },
        scale: null,
        importPromise: null,
        importGateRelease: null,
        appendGateRelease: null,
        appendGateReached: null,
        heldAppend: null,
        taskWriteGateRelease: null,
        taskWriteGateReached: null,
        heldTaskWrite: null,
        importPublishGateRelease: null,
        importPublishGateReached: null,
        timers: [],
        nextTimerHandle: 1,
    };
}

function randomHex(bytes: number): string {
    const values = new Uint8Array(bytes);
    crypto.getRandomValues(values);
    return [...values]
        .map((value) => value.toString(16).padStart(2, '0'))
        .join('');
}

function testSetInterval(
    handler: () => void | Promise<void>,
    timeout: number,
): number {
    const periodMs = Math.max(1, timeout);
    const handle = state.nextTimerHandle;
    state.nextTimerHandle += 1;
    state.timers.push({
        handle,
        periodMs,
        nextAtMs: state.nowMs + periodMs,
        callback: handler,
    });
    return handle;
}

function testClearInterval(handle: unknown): void {
    const index = state.timers.findIndex((timer) => timer.handle === handle);
    if (index >= 0) state.timers.splice(index, 1);
}

class SummaryTransport extends FixtureTransport {
    constructor(private readonly taskSummary: TaskSummary) {
        super();
    }

    override async listTasks(): Promise<TaskSummary[]> {
        return [this.taskSummary];
    }
}

class DeferredTestModelStream implements AsyncIterable<ModelStreamEvent> {
    private readonly queue: ModelStreamEvent[] = [];
    private readonly waiters: Array<
        (result: IteratorResult<ModelStreamEvent>) => void
    > = [];
    private closed = false;

    push(event: ModelStreamEvent): void {
        if (this.closed) return;
        const waiter = this.waiters.shift();
        if (waiter !== undefined) {
            waiter({ value: event, done: false });
            return;
        }
        this.queue.push(event);
    }

    end(): void {
        if (this.closed) return;
        this.closed = true;
        for (const waiter of this.waiters.splice(0)) {
            waiter({ value: undefined as never, done: true });
        }
    }

    next(): Promise<IteratorResult<ModelStreamEvent>> {
        const queued = this.queue.shift();
        if (queued !== undefined) {
            return Promise.resolve({ value: queued, done: false });
        }
        if (this.closed) {
            return Promise.resolve({ value: undefined as never, done: true });
        }
        return new Promise((resolve) => this.waiters.push(resolve));
    }

    return(): Promise<IteratorResult<ModelStreamEvent>> {
        this.end();
        return Promise.resolve({ value: undefined as never, done: true });
    }

    [Symbol.asyncIterator](): AsyncIterator<ModelStreamEvent> {
        return this;
    }
}

// Test-only synthetic port. It exposes one live stream per model turn so a
// gated probe can drive two independent runs from the same runtime.
export class GatedTestModelPort implements ModelPort {
    readonly id = 'test-gated';
    readonly kind = 'synthetic-test';
    readonly synthetic = true;
    readonly requests: ModelRequest[] = [];
    private readonly streamsByRun = new Map<string, DeferredTestModelStream[]>();

    open(request: ModelRequest): AsyncIterable<ModelStreamEvent> {
        this.requests.push(request);
        const stream = new DeferredTestModelStream();
        const streams = this.streamsByRun.get(request.runId) ?? [];
        streams.push(stream);
        this.streamsByRun.set(request.runId, streams);
        return stream;
    }

    latest(runId: string): DeferredTestModelStream | null {
        return this.streamsByRun.get(runId)?.at(-1) ?? null;
    }
}

export function createGatedTestModelPort(): GatedTestModelPort {
    return new GatedTestModelPort();
}

export function studioTestNow(): number {
    return state.nowMs;
}

export function createGatedTestToolExecutor() {
    return testToolExecutor();
}

function testToolExecutor() {
    return async (intent: {
        taskId: string;
        arguments: Record<string, unknown>;
    }) => {
        const path =
            typeof intent.arguments.path === 'string'
                ? intent.arguments.path
                : `artifacts/${intent.taskId}.txt`;
        return {
            status: 'ok' as const,
            output: { path },
            files: [{ path, change: 'added' as const }],
            head_digest: digestFor(intent.taskId),
        };
    };
}

function createTestClient(store: AgentStore, taskStore: AgentTaskStore): void {
    const liveModel = new GatedTestModelPort();
    const runtime = new AgentRuntime({
        store,
        taskStore,
        modelPort: liveModel,
        ownsTaskStore: false,
        toolExecutor: testToolExecutor(),
        now: () => state.nowMs,
        timers: {
            setInterval: testSetInterval,
            clearInterval: testClearInterval,
            setTimeout: (handler, timeout) =>
                globalThis.setTimeout(handler, timeout),
            clearTimeout: (handle) =>
                globalThis.clearTimeout(handle as never),
        },
        writerId: `runtime-probe-${randomHex(8)}`,
    });
    state.store = store;
    state.taskStore = taskStore;
    state.runtime = runtime;
    state.adoptedClient = null;
    state.adoptedRuntime = null;
    state.liveModel = liveModel;
    state.client = new AgentClient({ runtime });
}

function ensureClient(): AgentClient {
    if (state.client !== null) return state.client;
    const store = createAgentStore();
    const taskStore = new AgentTaskStore({
        store,
        now: () => state.nowMs,
        faults: state.faults,
        timers: {
            setInterval: testSetInterval,
            clearInterval: testClearInterval,
        },
    });
    createTestClient(store, taskStore);
    return state.client!;
}

function requireClient(): AgentClient {
    if (state.client === null) throw new Error('studio test client is not initialized');
    return state.client;
}

function requireTaskStore(): AgentTaskStore {
    if (state.taskStore === null) throw new Error('studio test store is not initialized');
    return state.taskStore;
}

function currentTask(): TaskView | null {
    const client = state.client;
    if (client === null) return null;
    const snapshot = client.store.snapshot();
    return (
        snapshot.selectedTask ??
        snapshot.tasks.find((task) => task.taskId === state.fixture?.taskId) ??
        snapshot.tasks[0] ??
        null
    );
}

function currentRun(task: TaskView | null): RunView | null {
    if (task === null) return null;
    if (task.latestRunId !== null) {
        return task.runs.find((run) => run.runId === task.latestRunId) ?? null;
    }
    return task.runs.length === 0 ? null : task.runs[task.runs.length - 1]!;
}

function currentFixture(): FixtureSummary {
    if (state.fixture !== null) return state.fixture;
    const task = currentTask();
    const run = currentRun(task);
    if (task === null || run === null) {
        throw new Error('no task/run fixture is present');
    }
    const fixture: FixtureSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: run.runId,
        generation: run.generation,
        eventIds: run.events.map((event) => event.eventId),
        digests: run.lastDigest === null ? [] : [run.lastDigest],
    };
    state.fixture = fixture;
    return fixture;
}

function errorCode(error: unknown): string {
    if (error instanceof AgentStoreError || error instanceof AgentClientError) {
        return error.code;
    }
    return 'internal';
}

function errorMessage(error: unknown): string {
    if (error instanceof AgentStoreError) {
        const prefix = `${error.code}: `;
        return error.message.startsWith(prefix)
            ? error.message.slice(prefix.length)
            : error.message;
    }
    return error instanceof Error ? error.message : String(error);
}

function failure(error: unknown): Outcome {
    return {
        ok: false,
        code: errorCode(error),
        message: errorMessage(error),
        ...(error instanceof AgentStoreError && error.storage !== null
            ? { storage: error.storage }
            : {}),
    };
}

function appendOutcome(result: {
    status: string;
    code: string | null;
    message: string | null;
    storage: string | null;
}): Outcome {
    return {
        ok: result.status !== 'rejected',
        status: result.status,
        ...(result.code === null ? {} : { code: result.code }),
        ...(result.message === null ? {} : { message: result.message }),
        storage: result.storage,
    };
}

function restoreState(): RestoreView {
    const task = currentTask();
    const run = currentRun(task);
    const activeRun =
        run !== null && !TERMINAL_STATES.has(run.state)
            ? {
                  runId: run.runId,
                  generation: run.generation,
                  lastSequence: run.lastSequence,
                  lastEventId: run.lastEventId,
                  state: run.state,
                  statusProjection: run.statusProjection,
              }
            : null;
    const incompleteForTask =
        task !== null && state.incomplete.some((entry) => entry.taskId === task.taskId);
    const latestLoaded =
        task !== null &&
        task.latestRunId !== null &&
        run !== null &&
        run.runId === task.latestRunId;
    const unresolvedRun =
        task !== null &&
        (task.unresolvedRunId !== null ||
            (task.latestRunId !== null && !latestLoaded));
    const canStartRun =
        state.phase === 'ready' &&
        task?.state !== 'waiting_for_background' &&
        !incompleteForTask &&
        activeRun === null &&
        !unresolvedRun;
    return {
        phase: state.phase,
        sendEnabled: canStartRun,
        canStartRun,
        error: state.error,
        incomplete: state.incomplete,
        activeRun,
    };
}

async function performOpen(): Promise<void> {
    const client = ensureClient();
    state.phase = 'loading';
    state.error = null;
    state.incomplete = [];
    try {
        if (state.runtime !== null) await state.runtime.open();
        const report = await requireTaskStore().open();
        state.incomplete = report.incomplete.map((entry) => ({
            taskId: entry.task_id,
            runId: entry.run_id,
            code: entry.code,
            message: entry.message,
        }));
        state.phase = state.incomplete.length > 0 ? 'incomplete' : 'ready';
    } catch (error) {
        state.phase = 'error';
        state.error = { code: errorCode(error), message: errorMessage(error) };
    }
    void client;
}

async function open(): Promise<RestoreView> {
    if (
        (state.phase === 'ready' || state.phase === 'incomplete') &&
        state.taskStore?.isOpen === true
    ) {
        return restoreState();
    }
    if (state.openGate !== null) {
        const release = state.openGate;
        state.openGate = null;
        release();
    }
    ensureClient();
    await performOpen();
    return restoreState();
}

function eventFor(
    fixture: FixtureSummary,
    sequence: number,
    generation: number,
    type: string,
    data: Record<string, unknown>,
): AgentIncomingEvent {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: fixtureEventId(fixture.runId, generation, sequence),
        project_id: fixture.projectId,
        task_id: fixture.taskId,
        run_id: fixture.runId,
        generation,
        sequence,
        recorded_at_ms: state.nowMs + sequence,
        type,
        data,
    } as unknown as AgentIncomingEvent;
}

async function appendFixtureEvent(
    fixture: FixtureSummary,
    type: string,
    data: Record<string, unknown>,
    options: { writerId?: string; nowMs?: number } = {},
): Promise<Outcome & { digest?: string | null }> {
    const task = requireClient().store.snapshot().tasks.find(
        (candidate) => candidate.taskId === fixture.taskId,
    );
    const run = task?.runs.find((candidate) => candidate.runId === fixture.runId) ?? null;
    if (run === null) return { ok: false, code: 'run_not_found', message: 'run is missing' };
    const event = eventFor(
        fixture,
        run.lastSequence + 1,
        run.generation,
        type,
        data,
    );
    const result = await requireTaskStore().appendEvent({
        event,
        writer_id: options.writerId,
        now_ms: options.nowMs,
    });
    if (result.status === 'applied') {
        fixture.eventIds.push(event.event_id);
        if (result.digest !== null) fixture.digests.push(result.digest);
    }
    return appendOutcome(result);
}

async function createTaskAndEvents(options: {
    mode?: 'message' | 'cancelling' | 'finished' | 'false-finished' | 'approval' | 'failed' | 'detail';
    title?: string;
    text?: string;
    expiresAtMs?: number;
} = {}): Promise<FixtureSummary> {
    if (state.phase !== 'ready') {
        const opened = await open();
        if (opened.phase !== 'ready') {
            throw new Error(`studio test store is ${opened.phase}: ${opened.error?.message ?? ''}`);
        }
    }
    const taskStore = requireTaskStore();
    const task = await taskStore.createTask({
        title: options.title ?? 'Fixture task',
    });
    const runId = `run_${randomHex(8)}`;
    const fixture: FixtureSummary = {
        taskId: task.task_id,
        projectId: task.project_id,
        runId,
        generation: 1,
        eventIds: [],
        digests: [],
    };
    const queued = createRunQueuedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: 1,
        recordedAtMs: state.nowMs,
        queuedAtMs: state.nowMs,
    });
    const queuedResult = await taskStore.appendEvent({ event: queued });
    if (queuedResult.status !== 'applied') {
        throw new Error(`queued fixture was rejected: ${queuedResult.message ?? ''}`);
    }
    fixture.eventIds.push(queued.event_id);
    fixture.digests.push(queuedResult.digest!);

    const started = createRunStartedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: 1,
        recordedAtMs: state.nowMs + 1,
        startedAtMs: state.nowMs + 1,
        mode: 'agent',
        modelId: 'replay',
        modelKind: 'fixture',
        environmentKind: 'browser-fixture',
    });
    const startedResult = await taskStore.appendEvent({ event: started });
    if (startedResult.status !== 'applied') {
        throw new Error(`started fixture was rejected: ${startedResult.message ?? ''}`);
    }
    fixture.eventIds.push(started.event_id);
    fixture.digests.push(startedResult.digest!);

    const mode = options.mode ?? 'message';
    if (mode === 'message') {
        const message = eventFor(fixture, 2, 1, 'assistant.message', {
            message_id: `msg_${randomHex(8)}`,
            text: options.text ?? 'replayed fixture message',
        });
        const messageResult = await taskStore.appendEvent({ event: message });
        if (messageResult.status !== 'applied') {
            throw new Error(`message fixture was rejected: ${messageResult.message ?? ''}`);
        }
        fixture.eventIds.push(message.event_id);
        fixture.digests.push(messageResult.digest!);
    } else if (mode === 'cancelling') {
        const status = eventFor(fixture, 2, 1, 'run.status', {
            status: 'cancelling',
        });
        const statusResult = await taskStore.appendEvent({ event: status });
        if (statusResult.status !== 'applied') {
            throw new Error(`cancelling fixture was rejected: ${statusResult.message ?? ''}`);
        }
        fixture.eventIds.push(status.event_id);
        fixture.digests.push(statusResult.digest!);
    } else if (mode === 'approval') {
        const toolCallId = 'tool_reload_approval';
        const approvalId = 'appr_reload_approval';
        const requestDigest = digestFor('reload-approval');
        const requested = await appendFixtureEvent(fixture, 'tool.requested', {
            tool_call_id: toolCallId,
            tool: 'write_file',
            arguments_digest: requestDigest,
            arguments_preview: '{"path":"approval.txt"}',
        });
        if (requested.ok !== true) {
            throw new Error(`approval request fixture failed: ${requested.code ?? ''}`);
        }
        const required = await appendFixtureEvent(
            fixture,
            'tool.approval_required',
            {
                approval_id: approvalId,
                tool_call_id: toolCallId,
                tool: 'write_file',
                scope: 'tool:write_file',
                request_digest: requestDigest,
                expires_at_ms: options.expiresAtMs ?? state.nowMs + 600_000,
            },
        );
        if (required.ok !== true) {
            throw new Error(`approval fixture failed: ${required.code ?? ''}`);
        }
    } else if (mode === 'failed') {
        const failed = await appendFixtureEvent(fixture, 'run.failed', {
            code: 'model_unavailable',
            message: 'synthetic retryable failure',
            retryable: true,
        });
        if (failed.ok !== true) {
            throw new Error(`failed fixture failed: ${failed.code ?? ''}`);
        }
    } else if (mode === 'false-finished') {
        const status = eventFor(fixture, 2, 1, 'run.status', {
            status: 'finished',
        });
        const statusResult = await taskStore.appendEvent({ event: status });
        if (statusResult.status !== 'applied') {
            throw new Error(`false-finished fixture was rejected: ${statusResult.message ?? ''}`);
        }
        fixture.eventIds.push(status.event_id);
        fixture.digests.push(statusResult.digest!);
    } else if (mode === 'detail') {
        const messageId = `msg_${randomHex(8)}`;
        const requestDigest = digestFor('detail-tool');
        const steps: Array<[string, Record<string, unknown>]> = [
            ['assistant.delta', { message_id: messageId, text_delta: 'Hello ' }],
            ['assistant.delta', { message_id: messageId, text_delta: 'from replay' }],
            ['assistant.message', { message_id: messageId, text: 'Hello from replay' }],
            ['tool.requested', {
                tool_call_id: 'tool_detail_1',
                tool: 'write_file',
                arguments_digest: requestDigest,
                arguments_preview: '{"path":"notes.md"}',
            }],
            ['tool.started', { tool_call_id: 'tool_detail_1' }],
            ['tool.completed', {
                tool_call_id: 'tool_detail_1',
                status: 'ok',
                output_digest: requestDigest,
                output_preview: 'wrote notes',
                truncated: false,
            }],
            ['host.instruction', {
                text: '<script>hostile</script>',
                status: 'completed',
                tool_call_id: 'tool_hostile',
            }],
            ['error', {
                code: 'malformed_event',
                message: 'The event stream contained an invalid event. The transcript was not changed.',
                retryable: false,
            }],
            ['usage.updated', { input_tokens: 12, output_tokens: 34, model_calls: 2 }],
            ['plan.updated', {
                plan_id: 'plan_detail_1',
                title: 'Replay plan',
                steps: [
                    { step_id: 'step_read', label: 'Read request', state: 'done' },
                    { step_id: 'step_write', label: 'Write answer', state: 'active' },
                ],
            }],
        ];
        for (const [type, data] of steps) {
            const appended = await appendFixtureEvent(fixture, type, data);
            if (appended.ok !== true) {
                throw new Error(`detail fixture ${type} failed: ${appended.code ?? ''}`);
            }
        }
        const finished = await appendFixtureEvent(fixture, 'run.finished', {
            status: 'finished',
            finished_at_ms: state.nowMs + fixture.eventIds.length + 1,
            duration_ms: fixture.eventIds.length + 1,
        });
        if (finished.ok !== true) {
            throw new Error(`detail finish failed: ${finished.code ?? ''}`);
        }
    } else {
        const finished = eventFor(fixture, 2, 1, 'run.finished', {
            status: 'finished',
            finished_at_ms: state.nowMs + 2,
            duration_ms: 2,
        });
        const finishedResult = await taskStore.appendEvent({ event: finished });
        if (finishedResult.status !== 'applied') {
            throw new Error(`finished fixture was rejected: ${finishedResult.message ?? ''}`);
        }
        fixture.eventIds.push(finished.event_id);
        fixture.digests.push(finishedResult.digest!);
    }

    state.fixture = fixture;
    requireClient().store.selectTask(fixture.taskId);
    return fixture;
}

async function createSecondRun(): Promise<FixtureSummary> {
    const task = currentTask();
    const first = currentRun(task);
    if (task === null || first === null) {
        throw new Error('a first run is required before the second run');
    }
    if (!TERMINAL_STATES.has(first.state)) {
        throw new Error('the first run must be terminal before a new run');
    }
    const runId = `run_${randomHex(8)}`;
    const fixture: FixtureSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId,
        generation: first.generation + 1,
        eventIds: [],
        digests: [],
    };
    const queued = createRunQueuedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: fixture.generation,
        recordedAtMs: state.nowMs + 10,
        queuedAtMs: state.nowMs + 10,
    });
    const queuedResult = await requireTaskStore().appendEvent({ event: queued });
    if (queuedResult.status !== 'applied') {
        throw new Error(`second queued fixture was rejected: ${queuedResult.message ?? ''}`);
    }
    fixture.eventIds.push(queued.event_id);
    fixture.digests.push(queuedResult.digest!);
    const started = createRunStartedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: fixture.generation,
        recordedAtMs: state.nowMs + 11,
        startedAtMs: state.nowMs + 11,
        mode: 'agent',
        modelId: 'replay',
        modelKind: 'fixture',
        environmentKind: 'browser-fixture',
    });
    const startedResult = await requireTaskStore().appendEvent({ event: started });
    if (startedResult.status !== 'applied') {
        throw new Error(`second started fixture was rejected: ${startedResult.message ?? ''}`);
    }
    fixture.eventIds.push(started.event_id);
    fixture.digests.push(startedResult.digest!);
    state.fixture = fixture;
    requireClient().store.selectTask(fixture.taskId);
    return fixture;
}

async function ensureReady(): Promise<void> {
    if (state.phase === 'ready') return;
    const opened = await open();
    if (opened.phase !== 'ready') {
        throw new Error(`studio test store is ${opened.phase}: ${opened.error?.message ?? ''}`);
    }
}

async function createQueuedOnlyTask(): Promise<FixtureSummary> {
    await ensureReady();
    const taskStore = requireTaskStore();
    const task = await taskStore.createTask({ title: 'Queued only task' });
    const runId = `run_${randomHex(8)}`;
    const fixture: FixtureSummary = {
        taskId: task.task_id,
        projectId: task.project_id,
        runId,
        generation: 1,
        eventIds: [],
        digests: [],
    };
    const queued = createRunQueuedEvent({
        projectId: fixture.projectId,
        taskId: fixture.taskId,
        runId,
        generation: 1,
        recordedAtMs: state.nowMs,
        queuedAtMs: state.nowMs,
    });
    const result = await taskStore.appendEvent({ event: queued });
    if (result.status !== 'applied') {
        throw new Error(`queued-only fixture was rejected: ${result.message ?? ''}`);
    }
    fixture.eventIds.push(queued.event_id);
    fixture.digests.push(result.digest!);
    state.fixture = fixture;
    requireClient().store.selectTask(fixture.taskId);
    return fixture;
}

async function listUnloadedActiveTask(
    mode: 'running' | 'cancelling',
): Promise<Outcome> {
    await ensureReady();
    const taskId = `tsk_${randomHex(8)}`;
    const runId = `run_${randomHex(8)}`;
    const summary: TaskSummary = {
        taskId,
        projectId: 'prj_0000000000000001',
        title: `Unloaded ${mode} task`,
        repo: null,
        branch: null,
        createdAtMs: state.nowMs,
        archived: false,
        pinned: false,
        latestRunId: runId,
        state: mode,
    };
    const client = new AgentClient({
        transport: new SummaryTransport(summary),
        store: requireClient().store,
        taskStore: requireTaskStore(),
    });
    await client.listTasks({ includeArchived: true });
    requireClient().store.selectTask(taskId);
    state.fixture = {
        taskId,
        projectId: summary.projectId,
        runId,
        generation: 1,
        eventIds: [],
        digests: [],
    };
    return { ok: true, record: summary };
}

async function listActiveSummary(
    taskId: string,
    latestRunId: string | null,
    mode: 'running' | 'cancelling',
): Promise<Outcome> {
    await ensureReady();
    const task = requireClient()
        .store.snapshot()
        .tasks.find((candidate) => candidate.taskId === taskId);
    if (task === undefined) {
        return { ok: false, code: 'task_not_found', message: 'task is not loaded' };
    }
    const summary: TaskSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        title: task.title,
        repo: task.repo,
        branch: task.branch,
        createdAtMs: task.createdAtMs,
        archived: task.archived,
        pinned: task.pinnedLocal,
        latestRunId,
        unresolvedRunId: null,
        state: mode,
    };
    const client = new AgentClient({
        transport: new SummaryTransport(summary),
        store: requireClient().store,
        taskStore: requireTaskStore(),
    });
    try {
        await client.listTasks({ includeArchived: true });
        return { ok: true, record: summary };
    } catch (error) {
        return failure(error);
    }
}

async function appendQueuedSafe(taskId: string, runId: string): Promise<Outcome> {
    try {
        const fixture: FixtureSummary = {
            taskId,
            projectId: 'prj_0000000000000001',
            runId,
            generation: 1,
            eventIds: [],
            digests: [],
        };
        const event = createRunQueuedEvent({
            projectId: fixture.projectId,
            taskId,
            runId,
            generation: 1,
            recordedAtMs: state.nowMs,
            queuedAtMs: state.nowMs,
        });
        const result = await requireTaskStore().appendEvent({ event });
        return appendOutcome(result);
    } catch (error) {
        return failure(error);
    }
}

async function appendAssistantSafe(
    text: string,
    options: { writerId?: string; nowMs?: number } = {},
): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const task = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === fixture.taskId);
        const run =
            task?.runs.find((candidate) => candidate.runId === fixture.runId) ??
            null;
        const event = eventFor(
            fixture,
            run === null ? 3 : run.lastSequence + 1,
            run?.generation ?? fixture.generation,
            'assistant.message',
            { message_id: `msg_${randomHex(8)}`, text },
        );
        const result = await requireTaskStore().appendEvent({
            event,
            writer_id: options.writerId,
            now_ms: options.nowMs,
        });
        if (result.status === 'applied') {
            fixture.eventIds.push(event.event_id);
            if (result.digest !== null) fixture.digests.push(result.digest);
        }
        return appendOutcome(result);
    } catch (error) {
        return failure(error);
    }
}

async function createClientRun(
    options: { title?: string; prompt?: string } = {},
): Promise<FixtureSummary> {
    await ensureReady();
    const client = requireClient();
    const task = await client.createTask({ title: options.title ?? 'Client resume fixture' });
    const run = await client.startRun({
        taskId: task.taskId,
        prompt: options.prompt ?? 'resume fixture',
    });
    const deadline = Date.now() + 10_000;
    let cursor = client.store.getRunCursor(task.taskId, run.runId);
    while (
        (cursor === null || cursor.lastSequence < 2) &&
        Date.now() < deadline
    ) {
        await new Promise((resolve) => setTimeout(resolve, 10));
        cursor = client.store.getRunCursor(task.taskId, run.runId);
    }
    if (cursor === null || cursor.lastSequence < 2 || cursor.lastDigest === null) {
        throw new Error('client fixture run did not reach turn.started');
    }
    state.liveRuns.set(run.runId, {
        taskId: task.taskId,
        runId: run.runId,
    });
    const fixture: FixtureSummary = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: run.runId,
        generation: cursor.generation,
        eventIds: [
            fixtureEventId(run.runId, cursor.generation, 0),
            fixtureEventId(run.runId, cursor.generation, 1),
        ],
        digests: [cursor.lastDigest],
    };
    state.fixture = fixture;
    client.store.selectTask(task.taskId);
    return fixture;
}

async function createTaskAt(createdAtMs: number): Promise<Outcome> {
    try {
        await ensureReady();
        const record = await requireTaskStore().createTask({
            title: 'timestamp task',
            created_at_ms: createdAtMs,
        });
        return { ok: true, record };
    } catch (error) {
        return failure(error);
    }
}

async function resumeViaClient(): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const snapshot = await requireClient().resumeTask({
            taskId: fixture.taskId,
            runId: fixture.runId,
        });
        return { ok: true, record: snapshot };
    } catch (error) {
        return failure(error);
    }
}

async function recordChangesWithFiles(
    files: Array<Record<string, unknown>>,
): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const view = requireTaskStore().store.getRun(
            fixture.taskId,
            fixture.runId,
        );
        const latest = view?.changesSnapshots[view.changesSnapshots.length - 1];
        if (latest === undefined) {
            return {
                ok: false,
                code: 'invalid_arguments',
                message: 'no changes.snapshot is present',
            };
        }
        const record = await requireTaskStore().recordChanges({
            task_id: fixture.taskId,
            run_id: fixture.runId,
            head_digest: latest.headDigest,
            files,
            captured_at_ms: latest.capturedAtMs,
        });
        return { ok: true, record };
    } catch (error) {
        return failure(error);
    }
}

async function seedSmallEvents(total: number): Promise<{
    events: number;
    sequence: number;
    durationMs: number;
    heapUsed: number | null;
}> {
    await ensureReady();
    if (state.fixture === null) await createTaskAndEvents();
    if (!Number.isSafeInteger(total) || total < 3) {
        throw new Error('scale target must be at least 3 events');
    }
    const fixture = currentFixture();
    const taskStore = requireTaskStore();
    const client = requireClient();
    const startedAt = performance.now();
    let cursor = client.store.getRunCursor(fixture.taskId, fixture.runId);
    if (cursor === null) throw new Error('scale run is missing');
    while (cursor.lastSequence < total - 1) {
        const sequence = cursor.lastSequence + 1;
        if (sequence >= 10_000) {
            throw new Error('scale target exceeds max_events_per_run');
        }
        const event = eventFor(
            fixture,
            sequence,
            cursor.generation,
            'assistant.message',
            {
                message_id: `msg_${sequence.toString().padStart(8, '0')}`,
                text: 'x',
            },
        );
        const result = await taskStore.appendEvent({ event });
        if (result.status !== 'applied') {
            throw new Error(
                `scale append ${sequence} was rejected: ${result.code ?? ''} ${result.message ?? ''}`,
            );
        }
        fixture.eventIds.push(event.event_id);
        if (result.digest !== null) fixture.digests.push(result.digest);
        const next = client.store.getRunCursor(fixture.taskId, fixture.runId);
        if (next === null || next.lastSequence !== sequence) {
            throw new Error('scale cursor did not advance');
        }
        cursor = next;
    }
    const durationMs = performance.now() - startedAt;
    const heapUsed =
        (performance as unknown as { memory?: { usedJSHeapSize: number } })
            .memory?.usedJSHeapSize ?? null;
    const result = {
        events: cursor.lastSequence + 1,
        sequence: cursor.lastSequence,
        durationMs,
        heapUsed,
    };
    state.scale = result;
    return result;
}

function memoryCursorView(): CursorView | null {
    const task = currentTask();
    const run = currentRun(task);
    if (task === null || run === null || run.lastEventId === null || run.lastDigest === null) {
        return null;
    }
    return {
        taskId: task.taskId,
        runId: run.runId,
        generation: run.generation,
        sequence: run.lastSequence,
        eventId: run.lastEventId,
        digest: run.lastDigest,
        runState: run.state,
        statusProjection: run.statusProjection,
        taskState: task.state,
        messageCount: run.messages.length,
        lastMessageText: run.messages.length === 0
            ? null
            : run.messages[run.messages.length - 1]!.text,
    };
}

function cursorView(): CursorView | null {
    if (state.phase === 'loading' || state.phase === 'error' || state.phase === 'idle') {
        return null;
    }
    return memoryCursorView();
}

const HOLD_TOOL_ID = 'tool_hold_probe';

function digestFor(text: string): string {
    let hex = '';
    for (let index = 0; index < 64; index += 1) {
        const code = text.charCodeAt(index % Math.max(1, text.length));
        hex += ((code + index) % 16).toString(16);
    }
    return `sha256:${hex}`;
}

async function runtimeForTest(): Promise<AgentRuntime> {
    ensureClient();
    if (state.runtime === null) {
        throw new Error('the shared studio test runtime is not initialized');
    }
    await state.runtime.open();
    return state.runtime;
}

async function disposeRuntimeForTest(): Promise<void> {
    const runtime = state.runtime;
    state.runtime = null;
    state.liveModel = null;
    state.liveRuns.clear();
    if (runtime !== null) await runtime.dispose();
}

function requireLiveModel(): GatedTestModelPort {
    if (state.liveModel === null) {
        throw new Error('the gated live model is not initialized');
    }
    return state.liveModel;
}

function liveTaskSnapshot(taskId: string): LiveRunSnapshot {
    const store = requireClient().store;
    const task = store
        .snapshot()
        .tasks.find((candidate) => candidate.taskId === taskId);
    if (task === undefined) {
        throw new Error(`task ${taskId} is missing`);
    }
    const run =
        task.latestRunId === null
            ? null
            : store.getRun(task.taskId, task.latestRunId);
    return {
        taskId: task.taskId,
        title: task.title,
        taskState: task.state,
        hold: task.hold,
        latestRunId: task.latestRunId,
        runId: run?.runId ?? null,
        retryOfRunId: run?.retryOfRunId ?? null,
        generation: run?.generation ?? null,
        lastSequence: run?.lastSequence ?? null,
        lastEventId: run?.lastEventId ?? null,
        lastDigest: run?.lastDigest ?? null,
        runState: run?.state ?? null,
        messages: run?.messages.map((message) => message.text) ?? [],
        changes:
            run?.changesSnapshots.map((snapshot) => ({
                files: snapshot.files,
                headDigest: snapshot.headDigest,
            })) ?? [],
        approvals:
            run?.approvals.map((approval) => ({
                approvalId: approval.approvalId,
                toolCallId: approval.toolCallId,
                requestDigest: approval.requestDigest,
                expiresAtMs: approval.expiresAtMs,
                decision: approval.decision,
            })) ?? [],
        unknownEvents: run?.unknownEvents.length ?? 0,
    };
}

async function seedBackgroundHoldForTest(): Promise<Outcome> {
    try {
        const fixture = await createTaskAndEvents({ mode: 'cancelling' });
        const failed = await appendFixtureEvent(fixture, 'run.failed', {
            code: 'stop_unacknowledged',
            message: 'probe stop was not acknowledged',
            retryable: false,
            stop_outcome: 'ack_timeout',
            in_flight_tool_id: HOLD_TOOL_ID,
        });
        if (failed.ok !== true) return failed;
        const records = await requireTaskStore().listTasks({
            includeArchived: true,
        });
        const record = records.find(
            (candidate) => candidate.task_id === fixture.taskId,
        );
        if (record === undefined) {
            throw new Error('hold probe task record is missing');
        }
        await requireTaskStore().writeTaskRecord({
            ...record,
            hold: {
                reason: 'waiting_for_background',
                run_id: fixture.runId,
                tool_call_id: HOLD_TOOL_ID,
            },
        });
        return {
            ok: true,
            record: {
                taskId: fixture.taskId,
                runId: fixture.runId,
                toolCallId: HOLD_TOOL_ID,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function startRunOnHeldTaskForTest(taskId: string): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const run = await runtime.startRun({
            taskId,
            prompt: 'must not start on a hold',
        });
        return { ok: true, record: run };
    } catch (error) {
        return failure(error);
    }
}

async function resolveBackgroundHoldForTest(
    taskId: string,
    runId: string,
    toolCallId: string,
): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const receipt = await runtime.resolveBackground({
            taskId,
            runId,
            toolCallId,
        });
        return { ok: true, record: receipt };
    } catch (error) {
        return failure(error);
    }
}

function waitFor(
    predicate: () => boolean,
    label: string,
    attempts = 400,
): Promise<void> {
    return new Promise((resolve, reject) => {
        let remaining = attempts;
        const tick = (): void => {
            if (predicate()) {
                resolve();
                return;
            }
            remaining -= 1;
            if (remaining <= 0) {
                reject(new Error(`timed out waiting for ${label}`));
                return;
            }
            globalThis.setTimeout(tick, 5);
        };
        tick();
    });
}

function liveRunFor(runId: string): {
    taskId: string;
    runId: string;
    run: RunView;
} {
    const entry = state.liveRuns.get(runId);
    if (entry === undefined) throw new Error(`live run ${runId} is unknown`);
    const run = requireClient().store.getRun(entry.taskId, runId);
    if (run === null) throw new Error(`live run ${runId} is missing`);
    return { ...entry, run };
}

function encodeFrame(event: Record<string, unknown>): Uint8Array {
    return new TextEncoder().encode(`${canonicalJson(event)}\n`);
}

function syntheticEvent(input: {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    sequence: number;
    type: string;
    data: Record<string, unknown>;
    eventId?: string;
}): Record<string, unknown> {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id:
            input.eventId ??
            `evt_${input.runId}_${input.generation}_${input.sequence
                .toString()
                .padStart(12, '0')}`,
        project_id: input.projectId,
        task_id: input.taskId,
        run_id: input.runId,
        generation: input.generation,
        sequence: input.sequence,
        recorded_at_ms: state.nowMs + input.sequence,
        type: input.type,
        data: input.data,
    };
}

function lastAppliedEventFrame(runId: string, forged: boolean): Uint8Array {
    const entry = liveRunFor(runId);
    const task = requireClient()
        .store.snapshot()
        .tasks.find((candidate) => candidate.taskId === entry.taskId);
    if (task === undefined) throw new Error('live task is missing');
    const event = entry.run.events.at(-1);
    if (event === undefined) throw new Error('live run has no applied event');
    const data = forged
        ? { ...event.data, text: 'forged duplicate content' }
        : event.data;
    return encodeFrame(
        syntheticEvent({
            taskId: task.taskId,
            projectId: task.projectId,
            runId: entry.runId,
            generation: event.generation,
            sequence: event.sequence,
            type: event.type,
            data,
            eventId: event.eventId,
        }),
    );
}

async function startTwoLiveTasksForTest(): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const model = requireLiveModel();
        const specs = [
            { title: 'Live task A', prompt: 'live task A prompt' },
            { title: 'Live task B', prompt: 'live task B prompt' },
        ];
        const seeds: LiveTaskSeed[] = [];
        for (const spec of specs) {
            const task = await runtime.createTask({ title: spec.title });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: spec.prompt,
            });
            state.liveRuns.set(run.runId, {
                taskId: task.taskId,
                runId: run.runId,
            });
            seeds.push({
                taskId: task.taskId,
                runId: run.runId,
                generation: run.generation,
                title: spec.title,
                prompt: spec.prompt,
            });
        }
        await waitFor(
            () => seeds.every((seed) => model.latest(seed.runId) !== null),
            'two live model streams',
        );
        return { ok: true, record: seeds };
    } catch (error) {
        return failure(error);
    }
}

async function liveTextForTest(runId: string, text: string): Promise<Outcome> {
    try {
        const stream = requireLiveModel().latest(runId);
        if (stream === null) {
            throw new Error(`live model stream for ${runId} is missing`);
        }
        stream.push({ type: 'text-delta', text });
        const entry = liveRunFor(runId);
        await waitFor(
            () => liveTaskSnapshot(entry.taskId).messages.includes(text),
            'live text delivery',
        );
        return { ok: true, record: liveTaskSnapshot(entry.taskId) };
    } catch (error) {
        return failure(error);
    }
}

async function liveToolForTest(runId: string, path: string): Promise<Outcome> {
    try {
        const model = requireLiveModel();
        const before = model.latest(runId);
        if (before === null) {
            throw new Error(`live model stream for ${runId} is missing`);
        }
        before.push({
            type: 'tool-call',
            toolCallId: `tool_${randomHex(4)}`,
            tool: 'write_file',
            arguments: { path },
        });
        const entry = liveRunFor(runId);
        await waitFor(
            () =>
                liveTaskSnapshot(entry.taskId).changes.some((change) =>
                    change.files.some((file) => file.path === path),
                ),
            'live tool artifact',
        );
        await waitFor(
            () => model.latest(runId) !== before,
            'post-tool model turn',
        );
        return { ok: true, record: liveTaskSnapshot(entry.taskId) };
    } catch (error) {
        return failure(error);
    }
}

async function finishLiveRunForTest(runId: string): Promise<Outcome> {
    try {
        const stream = requireLiveModel().latest(runId);
        if (stream === null) {
            throw new Error(`live model stream for ${runId} is missing`);
        }
        stream.push({ type: 'finished', reason: 'stop' });
        stream.end();
        const entry = liveRunFor(runId);
        await waitFor(
            () => {
                const snapshot = liveTaskSnapshot(entry.taskId);
                return (
                    snapshot.runState === 'finished' ||
                    snapshot.runState === 'failed'
                );
            },
            'live run terminal',
        );
        return { ok: true, record: liveTaskSnapshot(entry.taskId) };
    } catch (error) {
        return failure(error);
    }
}

async function resyncLiveRunForTest(runId: string): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const entry = liveRunFor(runId);
        const result = await runtime.resumeTask({
            taskId: entry.taskId,
            runId,
        });
        return {
            ok: true,
            record: {
                snapshot: result.snapshot,
                generation: result.event.generation,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

function eventsForTest(taskId: string, runId: string): Outcome {
    try {
        const run = requireClient().store.getRun(taskId, runId);
        if (run === null) throw new Error('run is missing');
        return {
            ok: true,
            record: {
                state: run.state,
                generation: run.generation,
                lastSequence: run.lastSequence,
                lastEventId: run.lastEventId,
                lastDigest: run.lastDigest,
                events: run.events.map((event) => ({
                    sequence: event.sequence,
                    type: event.type,
                    eventId: event.eventId,
                })),
            },
        };
    } catch (error) {
        return failure(error);
    }
}

function terminalDataForTest(
    type: 'run.finished' | 'run.failed' | 'run.cancelled' | 'run.expired',
): Record<string, unknown> {
    const now = state.nowMs;
    if (type === 'run.finished') {
        return {
            status: 'finished',
            finished_at_ms: now,
            duration_ms: 10,
        };
    }
    if (type === 'run.failed') {
        return {
            code: 'host_failed',
            message: 'host terminal failure',
            retryable: false,
        };
    }
    if (type === 'run.cancelled') {
        return { safe_boundary: true, finished_at_ms: now };
    }
    return { expired_at_ms: now, reason: 'host terminal expiry' };
}

async function ingestTerminalForTest(
    runId: string,
    type: 'run.finished' | 'run.failed' | 'run.cancelled' | 'run.expired',
): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const entry = liveRunFor(runId);
        const task = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === entry.taskId);
        if (task === undefined) throw new Error('live task is missing');
        const run = requireClient().store.getRun(entry.taskId, runId);
        if (run === null) throw new Error('live run is missing');
        const frame = syntheticEvent({
            taskId: task.taskId,
            projectId: task.projectId,
            runId,
            generation: run.generation,
            sequence: run.lastSequence + 1,
            type,
            data: terminalDataForTest(type),
        });
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId,
        });
        const outcomes = await worker.push(encodeFrame(frame));
        const eof = await worker.eof();
        if (eof !== null) outcomes.push(eof);
        await waitFor(
            () => {
                const view = liveTaskSnapshot(entry.taskId);
                return view.runState === 'finished'
                    || view.runState === 'failed'
                    || view.runState === 'cancelled'
                    || view.runState === 'expired';
            },
            'terminal ingestion',
        );
        return {
            ok: true,
            record: {
                type,
                eventId: frame.event_id,
                outcomes: outcomes.map((outcome) => ({
                    kind: outcome.kind,
                    code: outcome.code(),
                    requiresResync: outcome.requiresResync,
                })),
                snapshot: liveTaskSnapshot(entry.taskId),
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function ingestThinkingHeartbeatForTest(runId: string): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const entry = liveRunFor(runId);
        const task = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === entry.taskId);
        if (task === undefined) throw new Error('live task is missing');
        const run = requireClient().store.getRun(entry.taskId, runId);
        if (run === null) throw new Error('live run is missing');
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId,
        });
        const outcomes = [];
        outcomes.push(
            ...(await worker.push(
                encodeFrame(
                    syntheticEvent({
                        taskId: task.taskId,
                        projectId: task.projectId,
                        runId,
                        generation: run.generation,
                        sequence: run.lastSequence + 1,
                        type: 'thinking.delta',
                        data: { text_delta: 'tolerated thought' },
                    }),
                ),
            )),
        );
        outcomes.push(
            ...(await worker.push(
                encodeFrame(
                    syntheticEvent({
                        taskId: task.taskId,
                        projectId: task.projectId,
                        runId,
                        generation: run.generation,
                        sequence: run.lastSequence + 2,
                        type: 'heartbeat',
                        data: { nonce: 'tolerated-heartbeat' },
                    }),
                ),
            )),
        );
        return {
            ok: true,
            record: {
                outcomes: outcomes.map((outcome) => ({
                    kind: outcome.kind,
                    code: outcome.code(),
                })),
                snapshot: liveTaskSnapshot(entry.taskId),
            },
        };
    } catch (error) {
        return failure(error);
    }
}

function ownerIdentityForTest(): {
    clientAdopted: boolean;
    runtimeAdopted: boolean;
} {
    return {
        clientAdopted:
            state.adoptedClient !== null && state.client === state.adoptedClient,
        runtimeAdopted:
            state.adoptedRuntime !== null &&
            state.runtime === state.adoptedRuntime,
    };
}

async function lateToolForTest(runId: string, path: string): Promise<Outcome> {
    try {
        const model = requireLiveModel();
        const entry = liveRunFor(runId);
        const stream = model.latest(runId);
        if (stream === null) {
            throw new Error(`live model stream for ${runId} is missing`);
        }
        const before = liveTaskSnapshot(entry.taskId);
        stream.push({
            type: 'tool-call',
            toolCallId: `tool_late_${randomHex(4)}`,
            tool: 'write_file',
            arguments: { path },
        });
        let stableSamples = 0;
        let previousSequence = before.lastSequence;
        for (let attempt = 0; attempt < 40; attempt += 1) {
            await new Promise<void>((resolve) =>
                globalThis.setTimeout(resolve, 25),
            );
            const current = liveTaskSnapshot(entry.taskId);
            if (current.lastSequence === previousSequence) {
                stableSamples += 1;
            } else {
                stableSamples = 0;
                previousSequence = current.lastSequence;
            }
            if (stableSamples >= 3) break;
        }
        return {
            ok: true,
            record: {
                before,
                after: liveTaskSnapshot(entry.taskId),
                quiescent: stableSamples >= 3,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function ingestHostileForTest(
    runId: string,
    mode:
        | 'malformed'
        | 'partial'
        | 'utf8_split'
        | 'unknown'
        | 'gap'
        | 'cross_task'
        | 'duplicate'
        | 'forged_duplicate',
): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const entry = liveRunFor(runId);
        const task = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === entry.taskId);
        if (task === undefined) throw new Error('live task is missing');
        const nextSequence = entry.run.lastSequence + 1;
        const unknownFrame = syntheticEvent({
            taskId: task.taskId,
            projectId: task.projectId,
            runId,
            generation: entry.run.generation,
            sequence: nextSequence,
            type: 'assistant.message.forged',
            data: {
                message_id: 'msg_forged',
                text: 'forged hostile output',
                tool_call_id: 'tool_forged',
                status: 'ok',
                output_digest: digestFor('forged-output'),
                completion: true,
            },
        });
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId,
        });
        const outcomes = [];
        if (mode === 'malformed') {
            outcomes.push(
                ...(await worker.push(
                    new TextEncoder().encode('{"schema":oops}\n'),
                )),
            );
        } else if (mode === 'partial') {
            const encoded = encodeFrame(unknownFrame);
            outcomes.push(
                ...(await worker.push(encoded.subarray(0, encoded.length - 1))),
            );
        } else if (mode === 'utf8_split') {
            const event = syntheticEvent({
                taskId: task.taskId,
                projectId: task.projectId,
                runId,
                generation: entry.run.generation,
                sequence: nextSequence,
                type: 'thinking.delta',
                data: { text_delta: 'héllo 🌍 — split' },
            });
            const encoded = encodeFrame(event);
            for (const byte of encoded) {
                outcomes.push(...(await worker.push(new Uint8Array([byte]))));
            }
        } else if (mode === 'unknown') {
            outcomes.push(...(await worker.push(encodeFrame(unknownFrame))));
        } else if (mode === 'gap') {
            const gap = syntheticEvent({
                taskId: task.taskId,
                projectId: task.projectId,
                runId,
                generation: entry.run.generation,
                sequence: nextSequence + 1,
                type: 'assistant.delta',
                data: {
                    message_id: 'msg_gap',
                    text_delta: 'gap output',
                },
            });
            outcomes.push(...(await worker.push(encodeFrame(gap))));
        } else if (mode === 'cross_task') {
            const cross = syntheticEvent({
                taskId: 'tsk_ffffffffffffffff',
                projectId: task.projectId,
                runId,
                generation: entry.run.generation,
                sequence: nextSequence,
                type: 'assistant.message',
                data: {
                    message_id: 'msg_cross',
                    text: 'cross task output',
                },
            });
            outcomes.push(...(await worker.push(encodeFrame(cross))));
        } else {
            outcomes.push(
                ...(await worker.push(
                    lastAppliedEventFrame(runId, mode === 'forged_duplicate'),
                )),
            );
        }
        const eof = await worker.eof();
        if (eof !== null) outcomes.push(eof);
        return {
            ok: true,
            record: {
                mode,
                outcomes: outcomes.map((outcome) => ({
                    kind: outcome.kind,
                    code: outcome.code(),
                    requiresResync: outcome.requiresResync,
                })),
                snapshot: liveTaskSnapshot(entry.taskId),
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function seedTwoFinishedTasksForTest(): Promise<Outcome> {
    try {
        const specs = [
            {
                title: 'Reload task A',
                text: 'alpha reload message',
                path: 'artifacts/reload-a.txt',
                headDigest: digestFor('reload-a'),
            },
            {
                title: 'Reload task B',
                text: 'beta reload message',
                path: 'artifacts/reload-b.txt',
                headDigest: digestFor('reload-b'),
            },
        ];
        const seeds: ReloadTaskSeed[] = [];
        for (const spec of specs) {
            const fixture = await createTaskAndEvents({
                mode: 'message',
                title: spec.title,
                text: spec.text,
            });
            const changes = await appendFixtureEvent(
                fixture,
                'changes.snapshot',
                {
                    scope: 'last_turn',
                    files: [{ path: spec.path, change: 'added' }],
                    head_digest: spec.headDigest,
                    captured_at_ms: state.nowMs + 50,
                },
            );
            if (changes.ok !== true) {
                throw new Error(`change fixture failed: ${changes.code ?? ''}`);
            }
            const finished = await appendFixtureEvent(
                fixture,
                'run.finished',
                {
                    status: 'finished',
                    finished_at_ms: state.nowMs + 60,
                    duration_ms: 60,
                },
            );
            if (finished.ok !== true) {
                throw new Error(`finish fixture failed: ${finished.code ?? ''}`);
            }
            const snapshot = await requireTaskStore().writeSnapshot(
                fixture.taskId,
                fixture.runId,
            );
            seeds.push({
                taskId: fixture.taskId,
                runId: fixture.runId,
                generation: fixture.generation,
                title: spec.title,
                message: spec.text,
                files: [{ path: spec.path, change: 'added' }],
                headDigest: spec.headDigest,
                snapshot: {
                    generation: snapshot.generation,
                    lastSequence: snapshot.last_sequence,
                    lastEventId: snapshot.last_event_id,
                    headDigest: snapshot.head_digest,
                },
            });
        }
        return { ok: true, record: seeds };
    } catch (error) {
        return failure(error);
    }
}

function taskSnapshotForTest(taskId: string): Outcome {
    try {
        return { ok: true, record: liveTaskSnapshot(taskId) };
    } catch (error) {
        return failure(error);
    }
}

async function verifyRunForTest(
    taskId: string,
    runId: string,
): Promise<Outcome> {
    try {
        const read = await requireTaskStore().readRun(taskId, runId);
        const snapshot = await requireTaskStore().readSnapshot(taskId, runId);
        const cursor = requireClient().store.getRunCursor(taskId, runId);
        return {
            ok: read.ok && snapshot.ok,
            record: {
                runOk: read.ok,
                runCode: read.ok ? null : read.code,
                eventCount: read.ok ? read.events.length : 0,
                snapshotOk: snapshot.ok,
                snapshotCode: snapshot.ok ? null : snapshot.code,
                snapshot: snapshot.ok
                    ? {
                          generation: snapshot.record.generation,
                          lastSequence: snapshot.record.last_sequence,
                          lastEventId: snapshot.record.last_event_id,
                          headDigest: snapshot.record.head_digest,
                      }
                    : null,
                cursor,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function seedPendingApprovalForTest(): Promise<Outcome> {
    try {
        const fixture = await createTaskAndEvents({
            mode: 'approval',
            title: 'Approval reload task',
        });
        return { ok: true, record: fixture };
    } catch (error) {
        return failure(error);
    }
}

async function pendingApprovalsForTest(taskId: string): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const pending = await runtime.pendingApprovals(taskId);
        return {
            ok: true,
            record: {
                pending,
                resolvable: pending.map((approval) =>
                    runtime.approvalResolvable(approval.approvalId),
                ),
                view: liveTaskSnapshot(taskId).approvals,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

async function resolveRestoredApprovalForTest(
    approvalId: string,
): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        await runtime.resolveApproval({ approvalId, decision: 'approved' });
        return { ok: true };
    } catch (error) {
        return failure(error);
    }
}

async function retryRestoredRunForTest(
    taskId: string,
    runId: string,
): Promise<Outcome> {
    try {
        const runtime = await runtimeForTest();
        const run = await runtime.retry({ taskId, runId });
        return { ok: true, record: run };
    } catch (error) {
        return failure(error);
    }
}

async function seedFailedRunForRetryProbe(): Promise<Outcome> {
    try {
        const fixture = await createTaskAndEvents({
            mode: 'failed',
            title: 'Retry honesty task',
        });
        return { ok: true, record: fixture };
    } catch (error) {
        return failure(error);
    }
}

async function appendCollidingRunQueuedForTest(
    taskId: string,
    writerId: string,
): Promise<Outcome> {
    try {
        const task = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === taskId);
        if (task === undefined) throw new Error(`task ${taskId} is missing`);
        const generation =
            task.runs.reduce((max, run) => Math.max(max, run.generation), 0) + 1;
        const runId = `run_${randomHex(8)}`;
        const event = {
            schema: AGENT_EVENT_SCHEMA,
            event_id: `evt_${runId}_${generation}_000000000000`,
            project_id: task.projectId,
            task_id: taskId,
            run_id: runId,
            generation,
            sequence: 0,
            recorded_at_ms: state.nowMs,
            type: 'run.queued',
            data: { queued_at_ms: state.nowMs },
        } as unknown as AgentIncomingEvent;
        const result = await requireTaskStore().appendEvent({
            event,
            writer_id: writerId,
            now_ms: state.nowMs,
        });
        if (result.status === 'applied') {
            return { ok: true, record: result };
        }
        return {
            ok: false,
            code: result.code ?? 'internal',
            message: result.message ?? 'colliding run was rejected',
        };
    } catch (error) {
        return failure(error);
    }
}

async function clearHoldNullForTest(taskId: string): Promise<Outcome> {
    try {
        const store = requireTaskStore();
        const records = await store.listTasks({ includeArchived: true });
        const record = records.find(
            (candidate) => candidate.task_id === taskId,
        );
        if (record === undefined) {
            throw new Error(`task ${taskId} is missing`);
        }
        const written = await store.writeTaskRecord({ ...record, hold: null });
        const memory = requireClient()
            .store.snapshot()
            .tasks.find((candidate) => candidate.taskId === taskId);
        const durable = (
            await store.listTasks({ includeArchived: true })
        ).find((candidate) => candidate.task_id === taskId);
        return {
            ok: true,
            record: {
                memoryHold: memory?.hold ?? null,
                durableHold: durable?.hold ?? null,
                writtenHold: written.hold ?? null,
            },
        };
    } catch (error) {
        return failure(error);
    }
}

function taskView(): TaskViewSummary | null {
    const task = currentTask();
    if (task === null) return null;
    return {
        taskId: task.taskId,
        title: task.title,
        pinned: task.pinnedLocal,
        archived: task.archived,
        state: task.state,
        latestRunId: task.latestRunId,
        unresolvedRunId: task.unresolvedRunId,
        hold: task.hold,
        runs: task.runs.map((run) => ({
            runId: run.runId,
            lastSequence: run.lastSequence,
            state: run.state,
        })),
    };
}

async function updateFlag(kind: 'pin' | 'archive', value: boolean): Promise<Outcome> {
    try {
        const fixture = currentFixture();
        const record =
            kind === 'pin'
                ? await requireTaskStore().pinTask(fixture.taskId, value)
                : await requireTaskStore().archiveTask(fixture.taskId, value);
        return { ok: true, record };
    } catch (error) {
        return failure(error);
    }
}

function importFixtureFromStore(): void {
    const task = currentTask();
    const run = currentRun(task);
    if (task === null || run === null) {
        state.fixture = null;
        return;
    }
    state.fixture = {
        taskId: task.taskId,
        projectId: task.projectId,
        runId: run.runId,
        generation: run.generation,
        eventIds: run.events.map((event) => event.eventId),
        digests: [],
    };
}

async function queueTaskMessageForTest(
    taskId: string,
    text: string,
): Promise<Outcome> {
    try {
        const client = requireClient();
        await client.open();
        const receipt = await client.queueMessage({ taskId, text });
        return { ok: true, record: receipt };
    } catch (error) {
        return failure(error);
    }
}

async function failLiveRunForTest(runId: string): Promise<Outcome> {
    try {
        const stream = requireLiveModel().latest(runId);
        if (stream === null) {
            throw new Error(`live model stream for ${runId} is missing`);
        }
        stream.push({
            type: 'failed',
            code: 'model_unavailable',
            message: 'synthetic retryable failure',
            retryable: true,
        });
        stream.end();
        const entry = liveRunFor(runId);
        await waitFor(
            () => liveTaskSnapshot(entry.taskId).runState === 'failed',
            'live run failure',
        );
        return { ok: true, record: liveTaskSnapshot(entry.taskId) };
    } catch (error) {
        return failure(error);
    }
}

function adoptStudioAgentOwner(owner: StudioAgentOwnerForTest): void {
    state.client = owner.client;
    state.store = owner.store;
    state.taskStore = owner.taskStore;
    state.runtime = owner.runtime;
    state.adoptedClient = owner.client;
    state.adoptedRuntime = owner.runtime;
    state.liveModel =
        owner.modelPort instanceof GatedTestModelPort ? owner.modelPort : null;
}

export function installStudioTestApi(owner?: StudioAgentOwnerForTest): void {
    if (window.__studioTest !== undefined) return;
    if (owner !== undefined) adoptStudioAgentOwner(owner);
    window.__studioTest = {
        clientForTest: () => state.client,
        async artifactFallbackProbeForTest(artifactId: string): Promise<Outcome> {
            try {
                await ensureReady();
                const store = state.store;
                const taskStore = state.taskStore;
                if (store === null || taskStore === null) {
                    throw new Error('studio test store is not initialized');
                }
                const transport = new RecordingArtifactHostTransport();
                const hostClient = new HostClient({ transport });
                const runtime = new AgentRuntime({
                    store,
                    taskStore,
                    hostClient,
                    ownsTaskStore: false,
                    now: () => state.nowMs,
                    timers: {
                        setInterval: testSetInterval,
                        clearInterval: testClearInterval,
                        setTimeout: (handler, timeout) =>
                            globalThis.setTimeout(handler, timeout),
                        clearTimeout: (handle) =>
                            globalThis.clearTimeout(handle as never),
                    },
                    writerId: `artifact-probe-${randomHex(8)}`,
                });
                const client = new AgentClient({ runtime });
                try {
                    const content = await client.readArtifact({ artifactId });
                    return {
                        ok: true,
                        record: {
                            intents: transport.intents.map((intent) => ({
                                tool: intent.tool,
                                requestScope:
                                    intent.request_scope ?? 'task',
                                artifactId:
                                    typeof intent.arguments.artifact_id ===
                                    'string'
                                        ? intent.arguments.artifact_id
                                        : null,
                            })),
                            content: {
                                bytes: [...content.bytes],
                                digest: content.digest,
                                sizeBytes: content.sizeBytes,
                                path: content.path,
                                mediaType: content.mediaType,
                            },
                        },
                    };
                } finally {
                    await client.dispose();
                }
            } catch (error) {
                return failure(error);
            }
        },
        async reset() {
            await disposeRuntimeForTest();
            await state.client?.dispose();
            if (state.taskStore !== null) {
                await state.taskStore.deleteDatabase();
            } else {
                const temporary = new AgentTaskStore({
                    store: createAgentStore(),
                    now: () => Date.now(),
                });
                await temporary.deleteDatabase();
            }
            state = freshState();
        },
        open,
        // API-only control: the mounted StudioApp keeps the client it was
        // given. This rebuilds the test owner and must not be read as a UI
        // hot-swap claim.
        async reopen() {
            const store = state.client?.store ?? state.store ?? createAgentStore();
            await disposeRuntimeForTest();
            state.taskStore?.close();
            const taskStore = new AgentTaskStore({
                store,
                now: () => state.nowMs,
                faults: state.faults,
                timers: {
                    setInterval: testSetInterval,
                    clearInterval: testClearInterval,
                },
            });
            createTestClient(store, taskStore);
            state.phase = 'idle';
            return open();
        },
        memoryCursor: memoryCursorView,
        async beginOpen() {
            ensureClient();
            state.phase = 'loading';
            state.error = null;
            state.incomplete = [];
            const gate = new Promise<void>((resolve) => {
                state.openGate = resolve;
            });
            state.openPromise = (async () => {
                await gate;
                await performOpen();
                state.openPromise = null;
            })();
        },
        async whenOpen() {
            if (state.openGate !== null) {
                const release = state.openGate;
                state.openGate = null;
                release();
            }
            if (state.openPromise !== null) {
                await state.openPromise;
            }
            return restoreState();
        },
        restoreState,
        createTaskAndEvents,
        createSecondRun,
        createClientRun,
        createQueuedOnlyTask,
        targetFixture(taskId: string, runId: string, generation = 1) {
            state.fixture = {
                taskId,
                projectId: 'prj_0000000000000001',
                runId,
                generation,
                eventIds: [],
                digests: [],
            };
            requireClient().store.selectTask(taskId);
        },
        listUnloadedActiveTask,
        listActiveSummary,
        appendQueuedSafe,
        createTaskAt,
        resumeViaClient,
        recordChangesWithFiles,
        seedSmallEvents,
        scaleMetrics() {
            return state.scale;
        },
        cursor: cursorView,
        taskView,
        async writeSnapshot(taskId?: string, runId?: string) {
            try {
                const fixture = currentFixture();
                const record = await requireTaskStore().writeSnapshot(
                    taskId ?? fixture.taskId,
                    runId ?? fixture.runId,
                );
                return { ok: true, record };
            } catch (error) {
                return failure(error);
            }
        },
        async readSnapshot() {
            try {
                const fixture = currentFixture();
                const result = await requireTaskStore().readSnapshot(
                    fixture.taskId,
                    fixture.runId,
                );
                return result.ok
                    ? { ok: true, record: result.record }
                    : { ok: false, code: result.code, message: result.message };
            } catch (error) {
                return failure(error);
            }
        },
        async appendChanges() {
            const fixture = currentFixture();
            return appendFixtureEvent(fixture, 'changes.snapshot', {
                scope: 'last_turn',
                files: [{ path: 'src/a.ts', status: 'modified' }],
                head_digest: 'turn:3:fixture',
                captured_at_ms: state.nowMs + 3,
            });
        },
        async finishFixtureForTest() {
            try {
                const fixture = currentFixture();
                const task = requireClient().store.snapshot().tasks.find(
                    (candidate) => candidate.taskId === fixture.taskId,
                );
                const run = task?.runs.find(
                    (candidate) => candidate.runId === fixture.runId,
                ) ?? null;
                if (run === null) {
                    return { ok: false, code: 'run_not_found', message: 'fixture run is missing' };
                }
                if (TERMINAL_STATES.has(run.state)) {
                    return { ok: true, record: { already: true } };
                }
                return appendFixtureEvent(fixture, 'run.finished', {
                    status: 'finished',
                    finished_at_ms: state.nowMs + run.lastSequence + 1,
                    duration_ms: run.lastSequence + 1,
                });
            } catch (error) {
                return failure(error);
            }
        },
        async appendResume() {
            const fixture = currentFixture();
            const run = currentRun(currentTask());
            if (run === null || run.lastDigest === null) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'no verified run cursor is available to resume',
                };
            }
            const event = eventFor(
                fixture,
                run.lastSequence + 1,
                run.generation + 1,
                'run.resumed',
                {
                    resumed_at_ms: state.nowMs + run.lastSequence + 1,
                    previous_generation: run.generation,
                    snapshot_digest: run.lastDigest,
                },
            );
            const verification = requireClient().store.verifySnapshot({
                taskId: fixture.taskId,
                runId: fixture.runId,
                generation: run.generation,
                lastSequence: run.lastSequence,
                lastEventId: run.lastEventId!,
                headDigest: run.lastDigest,
            });
            if (!verification.ok) {
                return {
                    ok: false,
                    code: verification.code ?? 'snapshot_corrupt',
                    message: verification.message ?? 'snapshot verification failed',
                };
            }
            const result = await requireTaskStore().appendEvent({
                event,
                snapshot_anchor: {
                    task_id: fixture.taskId,
                    run_id: fixture.runId,
                    generation: run.generation,
                    last_sequence: run.lastSequence,
                    last_event_id: run.lastEventId!,
                    head_digest: run.lastDigest,
                },
            });
            if (result.status === 'applied') {
                fixture.eventIds.push(event.event_id);
                if (result.digest !== null) fixture.digests.push(result.digest);
            }
            return appendOutcome(result);
        },
        async recordChanges(headDigest?: string) {
            try {
                const fixture = currentFixture();
                const view = requireTaskStore().store.getRun(
                    fixture.taskId,
                    fixture.runId,
                );
                const latest = view?.changesSnapshots[view.changesSnapshots.length - 1];
                const head = headDigest ?? latest?.headDigest;
                if (head === undefined) {
                    return {
                        ok: false,
                        code: 'invalid_arguments',
                        message: 'no changes.snapshot is present',
                    };
                }
                const record = await requireTaskStore().recordChanges({
                    task_id: fixture.taskId,
                    run_id: fixture.runId,
                    head_digest: head,
                    files: latest?.files ?? [],
                    captured_at_ms: latest?.capturedAtMs ?? state.nowMs,
                });
                return { ok: true, record };
            } catch (error) {
                return failure(error);
            }
        },
        async readChanges() {
            try {
                const fixture = currentFixture();
                const records = await requireTaskStore().readChanges(
                    fixture.taskId,
                    fixture.runId,
                );
                return { ok: true, record: { records } };
            } catch (error) {
                return failure(error);
            }
        },
        pin: (pinned: boolean) => updateFlag('pin', pinned),
        archive: (archived: boolean) => updateFlag('archive', archived),
        seedBackgroundHoldForTest,
        startRunOnHeldTaskForTest,
        resolveBackgroundHoldForTest,
        startTwoLiveTasksForTest,
        queueTaskMessageForTest,
        liveTextForTest,
        failLiveRunForTest,
        liveToolForTest,
        finishLiveRunForTest,
        resyncLiveRunForTest,
        ingestHostileForTest,
        eventsForTest,
        ingestTerminalForTest,
        ingestThinkingHeartbeatForTest,
        lateToolForTest,
        ownerIdentityForTest,
        seedTwoFinishedTasksForTest,
        taskSnapshotForTest,
        verifyRunForTest,
        seedPendingApprovalForTest,
        pendingApprovalsForTest,
        resolveRestoredApprovalForTest,
        seedFailedRunForRetryProbe,
        retryRestoredRunForTest,
        clearHoldNullForTest,
        appendCollidingRunQueuedForTest,
        async acquireLease(writerId: string, nowMs?: number) {
            try {
                const fixture = currentFixture();
                return {
                    ok: true,
                    record: await requireTaskStore().acquireLease(
                        fixture.taskId,
                        writerId,
                        nowMs,
                    ),
                };
            } catch (error) {
                return failure(error);
            }
        },
        async heartbeatLease(writerId: string, nowMs?: number) {
            try {
                const fixture = currentFixture();
                return {
                    ok: true,
                    record: await requireTaskStore().heartbeatLease(
                        fixture.taskId,
                        writerId,
                        nowMs,
                    ),
                };
            } catch (error) {
                return failure(error);
            }
        },
        async releaseLease(writerId: string) {
            try {
                const fixture = currentFixture();
                const result = await requireTaskStore().releaseLease(
                    fixture.taskId,
                    writerId,
                );
                return { ok: true, record: result };
            } catch (error) {
                return failure(error);
            }
        },
        async lease() {
            try {
                const fixture = currentFixture();
                return {
                    ok: true,
                    record: await requireTaskStore().readLease(fixture.taskId),
                };
            } catch (error) {
                return failure(error);
            }
        },
        armAbortNextAppend() {
            state.faults.abort_next_append = true;
        },
        armAbortNextTaskWrite() {
            state.faults.abort_next_task_write = true;
        },
        armFailImportPublish() {
            state.faults.fail_import_publish = true;
        },
        armFailImportCleanup() {
            state.faults.fail_import_cleanup = true;
        },
        disposeDuringImport() {
            state.client?.dispose();
        },
        async beginHeldQueuedAppend(taskId: string, runId: string) {
            await ensureReady();
            const gate = new Promise<void>((resolve) => {
                state.appendGateRelease = resolve;
            });
            state.appendGateReached = requireTaskStore().setAppendGate(gate);
            const fixture: FixtureSummary = {
                taskId,
                projectId: 'prj_0000000000000001',
                runId,
                generation: 1,
                eventIds: [],
                digests: [],
            };
            state.fixture = fixture;
            const queued = createRunQueuedEvent({
                projectId: fixture.projectId,
                taskId,
                runId,
                generation: 1,
                recordedAtMs: state.nowMs,
                queuedAtMs: state.nowMs,
            });
            state.heldAppend = (async () => {
                try {
                    const result = await requireTaskStore().appendEvent({
                        event: queued,
                    });
                    if (result.status === 'applied') {
                        fixture.eventIds.push(queued.event_id);
                        if (result.digest !== null) fixture.digests.push(result.digest);
                    }
                    return appendOutcome(result);
                } catch (error) {
                    return failure(error);
                }
            })();
        },
        async awaitAppendHeld() {
            if (state.appendGateReached !== null) {
                await state.appendGateReached;
            }
        },
        releaseHeldAppend() {
            const release = state.appendGateRelease;
            state.appendGateRelease = null;
            release?.();
        },
        async finishHeldAppend() {
            if (state.heldAppend === null) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'no held append is running',
                };
            }
            const result = await state.heldAppend;
            state.heldAppend = null;
            state.appendGateReached = null;
            return result;
        },
        async beginHeldTaskWrite(taskId: string) {
            await ensureReady();
            const gate = new Promise<void>((resolve) => {
                state.taskWriteGateRelease = resolve;
            });
            state.taskWriteGateReached = requireTaskStore().setTaskWriteGate(gate);
            const record = {
                task_id: taskId,
                project_id: 'prj_0000000000000001',
                title: 'Held task write',
                created_at_ms: state.nowMs,
                archived: false,
                pinned_local: false,
                latest_run_id: null,
            };
            state.heldTaskWrite = (async () => {
                try {
                    const written = await requireTaskStore().writeTaskRecord(record);
                    return { ok: true, record: written };
                } catch (error) {
                    return failure(error);
                }
            })();
        },
        async awaitTaskWriteHeld() {
            if (state.taskWriteGateReached !== null) {
                await state.taskWriteGateReached;
            }
        },
        releaseTaskWrite() {
            const release = state.taskWriteGateRelease;
            state.taskWriteGateRelease = null;
            release?.();
        },
        async finishHeldTaskWrite() {
            if (state.heldTaskWrite === null) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'no held task write is running',
                };
            }
            const result = await state.heldTaskWrite;
            state.heldTaskWrite = null;
            state.taskWriteGateReached = null;
            return result;
        },
        beginImport(text: string, options: { holdPublish?: boolean } = {}) {
            state.phase = 'loading';
            state.error = null;
            const gate = new Promise<void>((resolve) => {
                state.importGateRelease = resolve;
            });
            requireTaskStore().setImportGate(gate);
            if (options.holdPublish === true) {
                const publishGate = new Promise<void>((resolve) => {
                    state.importPublishGateRelease = resolve;
                });
                state.importPublishGateReached =
                    requireTaskStore().setImportPublishGate(publishGate);
            } else {
                state.importPublishGateRelease = null;
                state.importPublishGateReached = null;
            }
            state.importPromise = (async () => {
                try {
                    const summary = await requireTaskStore().importNdjson(text);
                    importFixtureFromStore();
                    state.phase = 'ready';
                    state.error = null;
                    return { ok: true, record: summary };
                } catch (error) {
                    state.phase = 'error';
                    state.error = { code: errorCode(error), message: errorMessage(error) };
                    return failure(error);
                }
            })();
        },
        async awaitImportPublishGate() {
            if (state.importPublishGateReached !== null) {
                await state.importPublishGateReached;
            }
        },
        releaseImportPublishGate() {
            const release = state.importPublishGateRelease;
            state.importPublishGateRelease = null;
            release?.();
        },
        async whenImport() {
            if (state.importGateRelease !== null) {
                const release = state.importGateRelease;
                state.importGateRelease = null;
                release();
            }
            if (state.importPromise === null) {
                return { ok: false, code: 'invalid_arguments', message: 'no import is running' };
            }
            const result = await state.importPromise;
            state.importPromise = null;
            return result;
        },
        async advanceImportClock(ms: number) {
            if (!Number.isFinite(ms) || ms < 0) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'ms must be a non-negative number',
                };
            }
            state.nowMs += ms;
            const due = state.timers.filter((timer) => timer.nextAtMs <= state.nowMs);
            for (const timer of due) {
                if (!state.timers.includes(timer)) continue;
                timer.nextAtMs += timer.periodMs;
                try {
                    await timer.callback();
                } catch (error) {
                    return failure(error);
                }
            }
            return {
                ok: true,
                record: { now_ms: state.nowMs, timers: state.timers.length },
            };
        },
        async freezeImportClock(ms: number) {
            if (!Number.isFinite(ms) || ms < 0) {
                return {
                    ok: false,
                    code: 'invalid_arguments',
                    message: 'ms must be a non-negative number',
                };
            }
            state.nowMs += ms;
            return {
                ok: true,
                record: { now_ms: state.nowMs, timers: state.timers.length },
            };
        },
        appendAssistantSafe,
        async appendAssistant(
            text: string,
            options: { writerId?: string; nowMs?: number } = {},
        ) {
            const fixture = currentFixture();
            return appendFixtureEvent(
                fixture,
                'assistant.message',
                {
                    message_id: `msg_${randomHex(8)}`,
                    text,
                },
                options,
            );
        },
        async exportNdjson() {
            return requireTaskStore().exportNdjson();
        },
        async importNdjson(text: string) {
            try {
                const summary = await requireTaskStore().importNdjson(text);
                importFixtureFromStore();
                return { ok: true, record: summary };
            } catch (error) {
                return failure(error);
            }
        },
    };
}

declare global {
    interface Window {
        __studioTest?: StudioTestApi;
    }
}
