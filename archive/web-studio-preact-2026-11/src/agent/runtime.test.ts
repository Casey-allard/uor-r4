import { describe, expect, test } from 'vitest';

import { AgentClient, AgentClientError } from './client.js';
import { canonicalJson } from './generated/agentCanonical.js';
import type { AgentTaskStore } from './store.js';
import { AgentStoreError } from './storeTypes.js';
import type { ChangeInput, ChangeRecord, TaskRecord } from './storeTypes.js';
import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
    type RunView,
    type TaskView,
} from '../state/agent.js';
import {
    AGENT_EVENT_SCHEMA,
    type AgentErrorCode,
    type AgentEvent,
    type RunState,
} from './generated/agentContract.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from './hostClient.js';
import { StaticHostTransport } from './transports.js';
import { ReplayModelPort } from './modelPort.js';
import type { ModelPort, ModelRequest, ModelStreamEvent } from './modelPort.js';
import {
    AgentRuntime,
    isLeaseStale,
    type AgentRuntimeOptions,
    type ResolveBackgroundInput,
    type RuntimeToolExecutor,
    type RuntimeToolResult,
} from './runtime.js';

const PROJECT_ID = 'prj_0000000000000001';
const LEASE_HEARTBEAT_MS = 1_000;

interface TimerEntry {
    due: number;
    every: number | null;
    handler: () => void;
}

// Deterministic injected clock/timers. No test sleeps or wall-clock waits.
class TestClock {
    nowMs = 1_800_000_000_000;
    private nextHandle = 1;
    private readonly entries = new Map<number, TimerEntry>();

    now = (): number => this.nowMs;

    setTimeout = (handler: () => void, timeoutMs: number): unknown => {
        const handle = this.nextHandle++;
        this.entries.set(handle, {
            due: this.nowMs + Math.max(0, timeoutMs),
            every: null,
            handler,
        });
        return handle;
    };

    clearTimeout = (handle: unknown): void => {
        if (typeof handle === 'number') this.entries.delete(handle);
    };

    setInterval = (handler: () => void, timeoutMs: number): unknown => {
        const handle = this.nextHandle++;
        const every = Math.max(1, timeoutMs);
        this.entries.set(handle, {
            due: this.nowMs + every,
            every,
            handler,
        });
        return handle;
    };

    clearInterval = (handle: unknown): void => {
        if (typeof handle === 'number') this.entries.delete(handle);
    };

    timeoutCount(): number {
        return [...this.entries.values()].filter((entry) => entry.every === null)
            .length;
    }

    intervalCount(): number {
        return [...this.entries.values()].filter((entry) => entry.every !== null)
            .length;
    }

    advance(ms: number): void {
        const target = this.nowMs + ms;
        for (;;) {
            let selected: [number, TimerEntry] | null = null;
            for (const [handle, entry] of this.entries) {
                if (entry.due > target) continue;
                if (selected === null || entry.due < selected[1].due) {
                    selected = [handle, entry];
                }
            }
            if (selected === null) break;
            const [handle, entry] = selected;
            this.nowMs = entry.due;
            if (entry.every === null) this.entries.delete(handle);
            else entry.due += entry.every;
            entry.handler();
        }
        this.nowMs = target;
    }
}

class DeferredModelStream implements AsyncIterable<ModelStreamEvent> {
    private readonly queue: ModelStreamEvent[] = [];
    private readonly waiters: Array<{
        resolve: (result: IteratorResult<ModelStreamEvent>) => void;
        reject: (error: unknown) => void;
    }> = [];
    private closed = false;
    private failure: unknown = null;
    rejectReturn = false;

    push(event: ModelStreamEvent): void {
        if (this.closed) return;
        const waiter = this.waiters.shift();
        if (waiter !== undefined) {
            waiter.resolve({ value: event, done: false });
            return;
        }
        this.queue.push(event);
    }

    end(): void {
        if (this.closed) return;
        this.closed = true;
        for (const waiter of this.waiters.splice(0)) {
            waiter.resolve({ value: undefined as never, done: true });
        }
    }

    fail(error: unknown): void {
        if (this.closed) return;
        this.closed = true;
        this.failure = error;
        for (const waiter of this.waiters.splice(0)) {
            waiter.reject(error);
        }
    }

    next(): Promise<IteratorResult<ModelStreamEvent>> {
        if (this.failure !== null) return Promise.reject(this.failure);
        const event = this.queue.shift();
        if (event !== undefined) {
            return Promise.resolve({ value: event, done: false });
        }
        if (this.closed) {
            return Promise.resolve({ value: undefined as never, done: true });
        }
        return new Promise((resolve, reject) => {
            this.waiters.push({ resolve, reject });
        });
    }

    return(): Promise<IteratorResult<ModelStreamEvent>> {
        this.end();
        if (this.rejectReturn) {
            return Promise.reject(new Error('injected iterator return failure'));
        }
        return Promise.resolve({ value: undefined as never, done: true });
    }

    [Symbol.asyncIterator](): AsyncIterator<ModelStreamEvent> {
        return this;
    }
}

// Labelled synthetic model port. It only exposes deterministic scripted
// streams; it is not an R4 model or a task brain.
class ControlledModel implements ModelPort {
    readonly id = 'controlled-replay';
    readonly kind = 'synthetic-replay';
    readonly synthetic = true;
    readonly requests: ModelRequest[] = [];
    readonly streams: DeferredModelStream[] = [];

    open(request: ModelRequest): AsyncIterable<ModelStreamEvent> {
        this.requests.push(request);
        const stream = new DeferredModelStream();
        this.streams.push(stream);
        return stream;
    }
}

class FakeTaskStore {
    isOpen = false;
    closed = false;
    readonly writerId = 'writer-runtime-test';
    readonly events: AgentIncomingEvent[] = [];
    readonly changes: ChangeRecord[] = [];
    readonly acquires: Array<{ taskId: string; writerId: string; now: number }> = [];
    readonly heartbeats: Array<{ taskId: string; writerId: string; now: number }> = [];
    readonly releases: Array<{ taskId: string; writerId: string }> = [];
    failNextAppend = false;
    failNextList = false;
    failNextRelease = false;
    failNextTaskWrite = false;
    heartbeatGate: Promise<void> | null = null;
    private readonly records = new Map<string, TaskRecord>();
    private readonly leases = new Map<string, { task_id: string; writer_id: string; started_at_ms: number; heartbeat_at_ms: number }>();

    constructor(
        private readonly store: AgentStore,
        private readonly clock: TestClock,
    ) {}

    get defaultWriterId(): string {
        return this.writerId;
    }

    async open(): Promise<{ tasks: TaskRecord[]; runs: unknown[]; incomplete: unknown[] }> {
        this.isOpen = true;
        return { tasks: [...this.records.values()], runs: [], incomplete: [] };
    }

    close(): void {
        this.isOpen = false;
        this.closed = true;
    }

    async createTask(input: {
        project_id?: string;
        title: string;
        repo?: string | null;
        branch?: string | null;
        task_id?: string;
        created_at_ms?: number;
    }): Promise<TaskRecord> {
        const record: TaskRecord = {
            task_id: input.task_id ?? 'tsk_0000000000000001',
            project_id: input.project_id ?? PROJECT_ID,
            title: input.title,
            ...(input.repo == null ? {} : { repo: input.repo }),
            ...(input.branch == null ? {} : { branch: input.branch }),
            created_at_ms: input.created_at_ms ?? this.clock.now(),
            archived: false,
            pinned_local: false,
            latest_run_id: null,
        };
        this.records.set(record.task_id, record);
        this.store.upsertTask({
            taskId: record.task_id,
            projectId: record.project_id,
            title: record.title,
            repo: record.repo ?? null,
            branch: record.branch ?? null,
            createdAtMs: record.created_at_ms,
            archived: record.archived,
            pinnedLocal: record.pinned_local,
            latestRunId: null,
        });
        return record;
    }

    async writeTaskRecord(record: TaskRecord): Promise<TaskRecord> {
        if (this.failNextTaskWrite) {
            this.failNextTaskWrite = false;
            throw new AgentStoreError(
                'limit_exceeded',
                'injected task write failure',
                'indexeddb',
            );
        }
        const existing = this.records.get(record.task_id);
        const normalized: TaskRecord = {
            ...record,
            ...(record.hold === null
                ? {}
                : record.hold === undefined
                  ? existing?.hold === undefined
                      ? {}
                      : { hold: existing.hold }
                  : { hold: record.hold }),
        };
        if (record.hold === null) delete (normalized as { hold?: unknown }).hold;
        this.records.set(record.task_id, normalized);
        this.store.upsertTask({
            taskId: normalized.task_id,
            projectId: normalized.project_id,
            title: normalized.title,
            repo: normalized.repo ?? null,
            branch: normalized.branch ?? null,
            createdAtMs: normalized.created_at_ms,
            archived: normalized.archived,
            pinnedLocal: normalized.pinned_local,
            latestRunId: normalized.latest_run_id ?? null,
            ...(normalized.hold === undefined ? {} : { hold: normalized.hold }),
        });
        return normalized;
    }

    async pinTask(taskId: string, pinned: boolean): Promise<TaskRecord> {
        const record = this.records.get(taskId);
        if (record === undefined) throw new AgentStoreError('task_not_found', 'missing');
        record.pinned_local = pinned;
        await this.writeTaskRecord(record);
        return record;
    }

    async archiveTask(taskId: string, archived: boolean): Promise<TaskRecord> {
        const record = this.records.get(taskId);
        if (record === undefined) throw new AgentStoreError('task_not_found', 'missing');
        record.archived = archived;
        await this.writeTaskRecord(record);
        return record;
    }

    async listTasks(options: { includeArchived?: boolean; query?: string } = {}): Promise<TaskRecord[]> {
        if (this.failNextList) {
            this.failNextList = false;
            throw new AgentStoreError(
                'limit_exceeded',
                'injected task list failure',
                'indexeddb',
            );
        }
        const query = options.query?.trim().toLowerCase() ?? '';
        return [...this.records.values()]
            .filter((record) => options.includeArchived === true || !record.archived)
            .filter((record) => query === '' || record.title.toLowerCase().includes(query))
            .map((record) => ({ ...record }));
    }

    async appendEvent(input: {
        event: AgentIncomingEvent;
        digest?: string;
        writer_id?: string;
        now_ms?: number;
        snapshot?: boolean;
        snapshot_anchor?: unknown;
    }): Promise<{
        status: 'applied' | 'ignored' | 'rejected';
        code: AgentErrorCode | null;
        message: string | null;
        event_id: string;
        digest: string | null;
        storage: 'indexeddb' | null;
    }> {
        if (this.failNextAppend) {
            this.failNextAppend = false;
            return {
                status: 'rejected',
                code: 'limit_exceeded',
                message: 'injected durable append failure',
                event_id: input.event.event_id,
                digest: null,
                storage: 'indexeddb',
            };
        }
        this.events.push(input.event);
        const result = await this.store.appendEvent(input.event);
        return {
            status: result.status,
            code: result.status === 'rejected' ? result.code : null,
            message: result.status === 'rejected' ? result.message : null,
            event_id: result.eventId,
            digest: result.digest,
            storage: null,
        };
    }

    async recordChanges(input: ChangeInput): Promise<ChangeRecord> {
        const snapshot = this.store.latestChangesSnapshot(
            input.task_id,
            input.run_id,
            'last_turn',
        );
        if (
            snapshot === null ||
            snapshot.headDigest !== input.head_digest ||
            JSON.stringify(snapshot.files) !== JSON.stringify(input.files)
        ) {
            throw new AgentStoreError(
                'invalid_arguments',
                'change record is not bound to a changes.snapshot event',
            );
        }
        const record: ChangeRecord = {
            task_id: input.task_id,
            run_id: input.run_id,
            scope: 'last_turn',
            files: input.files,
            head_digest: input.head_digest,
            captured_at_ms: input.captured_at_ms,
        };
        this.changes.push(record);
        return record;
    }

    async acquireLease(
        taskId: string,
        writerId: string,
        nowMs = this.clock.now(),
    ): Promise<{ task_id: string; writer_id: string; started_at_ms: number; heartbeat_at_ms: number }> {
        this.acquires.push({ taskId, writerId, now: nowMs });
        const existing = this.leases.get(taskId);
        if (
            existing !== undefined &&
            existing.writer_id !== writerId &&
            nowMs - existing.heartbeat_at_ms <= 30_000
        ) {
            throw new AgentStoreError(
                'stale_generation',
                'task lease is held by another active writer',
            );
        }
        const record = {
            task_id: taskId,
            writer_id: writerId,
            started_at_ms:
                existing !== undefined && existing.writer_id === writerId
                    ? existing.started_at_ms
                    : nowMs,
            heartbeat_at_ms: nowMs,
        };
        this.leases.set(taskId, record);
        return record;
    }

    async heartbeatLease(
        taskId: string,
        writerId: string,
        nowMs = this.clock.now(),
    ): Promise<{ task_id: string; writer_id: string; started_at_ms: number; heartbeat_at_ms: number }> {
        this.heartbeats.push({ taskId, writerId, now: nowMs });
        if (this.heartbeatGate !== null) await this.heartbeatGate;
        const existing = this.leases.get(taskId);
        if (existing === undefined || existing.writer_id !== writerId) {
            throw new AgentStoreError('stale_generation', 'lease is missing');
        }
        existing.heartbeat_at_ms = nowMs;
        return { ...existing };
    }

    async releaseLease(
        taskId: string,
        writerId: string,
    ): Promise<{ released: boolean }> {
        this.releases.push({ taskId, writerId });
        if (this.failNextRelease) {
            this.failNextRelease = false;
            throw new AgentStoreError(
                'limit_exceeded',
                'injected release failure',
                'indexeddb',
            );
        }
        const existing = this.leases.get(taskId);
        if (existing === undefined) return { released: false };
        if (existing.writer_id !== writerId) {
            throw new AgentStoreError('stale_generation', 'lease is held by another writer');
        }
        this.leases.delete(taskId);
        return { released: true };
    }

    async readLease(taskId: string): Promise<unknown> {
        return this.leases.get(taskId) ?? null;
    }

    leaseFor(taskId: string): unknown {
        return this.leases.get(taskId) ?? null;
    }
}

function deterministicIds(): NonNullable<AgentRuntimeOptions['ids']> {
    let counter = 0;
    const next = (): number => {
        counter += 1;
        return counter;
    };
    return {
        taskId: () => `tsk_${next().toString(16).padStart(16, '0')}`,
        runId: () => `run_${next().toString(16).padStart(16, '0')}`,
        messageId: () => `msg_${next().toString(16).padStart(16, '0')}`,
        approvalId: () => `appr_${next().toString(16).padStart(16, '0')}`,
    };
}

function makeRuntime(
    overrides: Partial<AgentRuntimeOptions> = {},
    executor?: RuntimeToolExecutor,
): {
    runtime: AgentRuntime;
    store: AgentStore;
    taskStore: FakeTaskStore;
    clock: TestClock;
    model: ControlledModel;
} {
    const store = overrides.store ?? createAgentStore();
    const clock = new TestClock();
    const taskStore = new FakeTaskStore(store, clock);
    const model = (overrides.modelPort as ControlledModel | undefined) ?? new ControlledModel();
    const runtime = new AgentRuntime({
        store,
        taskStore: taskStore as unknown as AgentTaskStore,
        modelPort: model,
        ...(overrides.hostClient != null && executor === undefined
            ? {}
            : {
                  toolExecutor:
                      executor ??
                      (async () => ({
                          status: 'ok',
                          output: {},
                      }) as RuntimeToolResult),
              }),
        now: clock.now,
        timers: clock,
        writerId: 'writer-runtime-test',
        ids: deterministicIds(),
        leaseHeartbeatMs: LEASE_HEARTBEAT_MS,
        stopAckTimeoutMs: 30_000,
        boundaryWaitMs: 30_000,
        approvalTtlMs: 600_000,
        ...overrides,
    });
    return { runtime, store, taskStore, clock, model };
}

function yieldTask(): Promise<void> {
    return new Promise<void>((resolve) => {
        const immediate = (
            globalThis as {
                setImmediate?: (callback: () => void) => unknown;
            }
        ).setImmediate;
        if (immediate !== undefined) {
            immediate(resolve);
            return;
        }
        setTimeout(resolve, 0);
    });
}

async function flush(rounds = 100): Promise<void> {
    for (let index = 0; index < rounds; index += 1) {
        await Promise.resolve();
        await yieldTask();
    }
}

async function until(
    predicate: () => boolean,
    label: string,
    rounds = 2_000,
): Promise<void> {
    for (let index = 0; index < rounds; index += 1) {
        if (predicate()) return;
        await flush(1);
    }
    throw new Error(`timed out waiting for ${label}`);
}

async function nextEvent(
    iterator: AsyncIterator<AgentIncomingEvent>,
): Promise<IteratorResult<AgentIncomingEvent> | null> {
    return Promise.race([
        iterator.next(),
        flush(30).then(() => null),
    ]);
}

async function startOne(
    runtime: AgentRuntime,
    model: ControlledModel,
    prompt = 'first prompt',
): Promise<{
    task: TaskView;
    run: Awaited<ReturnType<AgentRuntime['startRun']>>;
    stream: DeferredModelStream;
}> {
    const summary = await runtime.createTask({ title: 'runtime task' });
    const run = await runtime.startRun({ taskId: summary.taskId, prompt });
    await until(() => model.streams.length >= 1, 'first model stream');
    const task = runtime.store.snapshot().tasks.find(
        (candidate) => candidate.taskId === summary.taskId,
    )!;
    return {
        task,
        run,
        stream: model.streams[model.streams.length - 1]!,
    };
}

function runView(store: AgentStore, taskId: string, runId: string): RunView {
    const run = store.getRun(taskId, runId);
    if (run === null) throw new Error('run is missing');
    return run;
}

function eventTypes(run: RunView): string[] {
    return run.events.map((event) => event.type);
}

function eventIndex(run: RunView, type: string): number {
    return run.events.findIndex((event) => event.type === type);
}

function eventData(run: RunView, sequence: number): Record<string, unknown> {
    const event = run.events.find((candidate) => candidate.sequence === sequence);
    if (event === undefined) throw new Error(`event at sequence ${sequence} is missing`);
    return event.data;
}

function terminalEvent(run: RunView): AgentEvent {
    const event = run.events[run.events.length - 1];
    if (event === undefined) throw new Error('run has no events');
    return event as unknown as AgentEvent;
}

function taskState(store: AgentStore, taskId: string): RunState | TaskView['state'] {
    return store.snapshot().tasks.find((task) => task.taskId === taskId)!.state;
}

describe('AgentRuntime run sequencing', () => {
    test('run_queued_is_sequence_zero', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(runtime, model);
            const view = runView(store, task.taskId, run.runId);
            expect(view.events[0]).toMatchObject({
                type: 'run.queued',
                generation: 1,
                sequence: 0,
            });
            expect(view.events[0]!.eventId).toBe(
                `evt_${run.runId}_1_${'0'.padStart(12, '0')}`,
            );
            expect(view.state).toBe('running');
            expect(view.lastSequence).toBeGreaterThanOrEqual(1);
            model.streams[0]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished run',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('run_started_is_sequence_one_with_retry_of_run_id_null_or_string', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            const view = runView(store, task.taskId, run.runId);
            expect(view.events[1]).toMatchObject({
                type: 'run.started',
                generation: 1,
                sequence: 1,
            });
            expect(eventData(view, 1).retry_of_run_id).toBeNull();
            expect(view.retryOfRunId).toBeNull();

            stream.push({
                type: 'failed',
                code: 'model_unavailable',
                message: 'synthetic failure',
                retryable: true,
            });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'failed',
                'failed run',
            );
            const retried = await runtime.retry({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(() => model.streams.length >= 2, 'retry stream');
            const retryView = runView(store, task.taskId, retried.runId);
            expect(retryView.generation).toBe(2);
            expect(eventData(retryView, 1).retry_of_run_id).toBe(run.runId);
            expect(retryView.retryOfRunId).toBe(run.runId);
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, retried.runId).state === 'finished',
                'retry finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('queue_message_is_delivered_at_next_turn', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model, 'first');
            const receipt = await runtime.queueMessage({
                taskId: task.taskId,
                text: 'queued second',
            });
            expect(receipt).toMatchObject({
                taskId: task.taskId,
                runId: run.runId,
                delivery: 'queue',
            });
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).toContain('message.queued');
            expect(
                eventData(view, view.lastSequence).delivery,
            ).toBe('queue');

            stream.end();
            await until(() => model.requests.length >= 2, 'second run request');
            expect(model.requests[1]!.prompt).toContain('queued second');
            const nextRunId = store.snapshot().tasks.find(
                (candidate) => candidate.taskId === task.taskId,
            )!.latestRunId!;
            expect(nextRunId).not.toBe(run.runId);
            const nextView = runView(store, task.taskId, nextRunId);
            expect(nextView.generation).toBe(2);
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, nextRunId).state === 'finished',
                'second run finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('queue_is_fifo_and_bounded', async () => {
        const { runtime, store, model } = makeRuntime({ maxQueuedMessages: 2 });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model, 'first');
            await runtime.queueMessage({ taskId: task.taskId, text: 'second' });
            await runtime.queueMessage({ taskId: task.taskId, text: 'third' });
            await expect(
                runtime.queueMessage({ taskId: task.taskId, text: 'overflow' }),
            ).rejects.toMatchObject({ code: 'limit_exceeded' });
            expect(
                eventTypes(runView(store, task.taskId, run.runId)).filter(
                    (type) => type === 'message.queued',
                ),
            ).toHaveLength(2);

            stream.end();
            await until(() => model.requests.length >= 2, 'second queued run');
            expect(model.requests[1]!.prompt).toBe('second');
            model.streams[1]!.end();
            await until(() => model.requests.length >= 3, 'third queued run');
            expect(model.requests[2]!.prompt).toBe('third');
            model.streams[2]!.end();
            await until(
                () => model.requests.length === 3 &&
                    store.snapshot().runs.every((candidate) => candidate.state === 'finished'),
                'all queued runs finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('start_run_returns_after_two_durable_events_before_model_progress', async () => {
        const { runtime, store, taskStore, model } = makeRuntime();
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'durable start' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'first prompt',
            });
            expect(model.streams).toHaveLength(0);
            expect(taskStore.events.slice(0, 2).map((event) => event.type)).toEqual([
                'run.queued',
                'run.started',
            ]);
            const view = runView(store, task.taskId, run.runId);
            expect(view.lastSequence).toBeGreaterThanOrEqual(1);
            expect(view.events[0]!.type).toBe('run.queued');
            expect(view.events[1]!.type).toBe('run.started');
            await until(() => model.streams.length >= 1, 'model progress');
            model.streams[0]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished durable start',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('steer_is_delivered_at_next_tool_boundary', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_boundary',
                tool: 'read_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => releaseTool !== null,
                'tool execution',
            );
            const steer = await runtime.steer({
                taskId: task.taskId,
                text: 'steer at boundary',
            });
            expect(steer.delivery).toBe('steer');
            let view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).not.toContain('message.steered');

            releaseTool!({ status: 'ok', output: { text: 'read' } });
            await until(
                () => eventTypes(runView(store, task.taskId, run.runId)).includes('message.steered'),
                'steered event',
            );
            view = runView(store, task.taskId, run.runId);
            const steeredIndex = eventIndex(view, 'message.steered');
            const completedIndex = eventIndex(view, 'tool.completed');
            expect(completedIndex).toBeGreaterThan(-1);
            expect(steeredIndex).toBeGreaterThan(completedIndex);
            expect(view.events[steeredIndex]!.data).toMatchObject({
                message_id: steer.messageId,
                boundary: 'next_tool',
            });

            await until(() => model.requests.length >= 2, 'next model turn');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after steer',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('steer_falls_back_to_queue_when_run_finishes', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            const steer = await runtime.steer({
                taskId: task.taskId,
                text: 'late steer',
            });
            stream.end();
            await until(() => model.requests.length >= 2, 'fallback run');
            const view = runView(store, task.taskId, run.runId);
            const steered = eventIndex(view, 'message.steered');
            const queued = eventIndex(view, 'message.queued');
            const finished = eventIndex(view, 'run.finished');
            expect(steered).toBeGreaterThan(-1);
            expect(queued).toBeGreaterThan(steered);
            expect(finished).toBeGreaterThan(queued);
            expect(view.events[steered]!.data).toMatchObject({
                message_id: steer.messageId,
                boundary: 'next_turn',
            });
            expect(view.events[queued]!.data).toMatchObject({
                message_id: steer.messageId,
                delivery: 'steer',
                boundary: 'next_turn',
            });
            expect(model.requests[1]!.prompt).toContain('late steer');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'old run finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('force_send_falls_back_to_queue_when_run_finishes', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            const force = await runtime.forceSend({
                taskId: task.taskId,
                text: 'late force',
            });
            stream.end();
            await until(() => model.requests.length >= 2, 'force fallback run');
            const view = runView(store, task.taskId, run.runId);
            const forced = eventIndex(view, 'message.forced');
            const queued = eventIndex(view, 'message.queued');
            const finished = eventIndex(view, 'run.finished');
            expect(forced).toBeGreaterThan(-1);
            expect(queued).toBeGreaterThan(forced);
            expect(finished).toBeGreaterThan(queued);
            expect(view.events[forced]!.data).toMatchObject({
                message_id: force.messageId,
                boundary: 'next_turn',
            });
            expect(view.events[queued]!.data).toMatchObject({
                message_id: force.messageId,
                delivery: 'force',
                boundary: 'next_turn',
            });
            expect(model.requests[1]!.prompt).toContain('late force');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'old run finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('fallback_queue_event_precedes_run_finished', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            await runtime.steer({ taskId: task.taskId, text: 'ordering steer' });
            await runtime.forceSend({ taskId: task.taskId, text: 'ordering force' });
            stream.end();
            await until(() => model.requests.length >= 2, 'next run');
            const view = runView(store, task.taskId, run.runId);
            const types = eventTypes(view);
            const finished = types.lastIndexOf('run.finished');
            expect(finished).toBe(types.length - 1);
            for (const type of [
                'message.steered',
                'message.forced',
                'message.queued',
            ]) {
                const index = types.indexOf(type);
                expect(index).toBeGreaterThan(-1);
                expect(index).toBeLessThan(finished);
            }
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'old run finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('fallback_starts_new_run_with_new_generation', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            await runtime.steer({ taskId: task.taskId, text: 'new generation' });
            stream.end();
            await until(() => model.requests.length >= 2, 'fallback new run');
            const taskView = store.snapshot().tasks.find(
                (candidate) => candidate.taskId === task.taskId,
            )!;
            const nextRunId = taskView.latestRunId!;
            expect(nextRunId).not.toBe(run.runId);
            const next = runView(store, task.taskId, nextRunId);
            expect(next.generation).toBe(2);
            expect(next.events[0]!.type).toBe('run.queued');
            expect(next.events[0]!.sequence).toBe(0);
            expect(next.events[1]!.type).toBe('run.started');
            expect(next.events[1]!.sequence).toBe(1);
            expect(eventData(next, 1).retry_of_run_id).toBeNull();
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, nextRunId).state === 'finished',
                'new run finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('steer_boundary_wait_is_bounded_and_falls_back_to_next_turn', async () => {
        const { runtime, store, model, clock } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(runtime, model);
            await runtime.steer({ taskId: task.taskId, text: 'bounded steer' });
            clock.advance(29_999);
            await flush();
            expect(eventTypes(runView(store, task.taskId, run.runId))).not.toContain(
                'message.steered',
            );
            clock.advance(1);
            await flush(200);
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).toContain('message.steered');
            expect(eventTypes(view)).toContain('message.queued');
            expect(eventTypes(view)).not.toContain('run.finished');
            model.streams[0]!.end();
            await until(() => model.requests.length >= 2, 'bounded fallback run');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'old run finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });
});

describe('AgentRuntime tool results, stop and host resolution', () => {
    test('write_tool_completion_records_changes_snapshot', async () => {
        const files = [{ path: 'src/a.ts', change: 'modified' }];
        const headDigest = `sha256:${'a'.repeat(64)}`;
        const executor: RuntimeToolExecutor = async () => ({
            status: 'ok',
            output: { written: 1 },
            files,
            head_digest: headDigest,
        });
        const { runtime, store, taskStore, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_write',
                tool: 'write_file',
                arguments: { path: 'src/a.ts' },
            });
            await until(
                () => eventTypes(runView(store, task.taskId, run.runId)).includes('changes.snapshot'),
                'changes snapshot',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).toContain('tool.completed');
            expect(view.changesSnapshots).toHaveLength(1);
            expect(view.changesSnapshots[0]).toMatchObject({
                scope: 'last_turn',
                files,
                headDigest,
            });
            expect(taskStore.changes).toHaveLength(1);
            expect(taskStore.changes[0]).toMatchObject({
                task_id: task.taskId,
                run_id: run.runId,
                scope: 'last_turn',
                files,
                head_digest: headDigest,
            });
            await until(() => model.requests.length >= 2, 'post-write turn');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after write',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_prevents_later_tool_dispatch', async () => {
        const dispatched: string[] = [];
        const executor: RuntimeToolExecutor = async (intent) => {
            dispatched.push(intent.tool);
            return { status: 'ok', output: {} };
        };
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            const stop = await runtime.stop({ taskId: task.taskId, runId: run.runId });
            expect(stop.acknowledged).toBe(true);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_after_stop',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await flush(300);
            expect(dispatched).toEqual([]);
            const view = runView(store, task.taskId, run.runId);
            expect(view.state).toBe('cancelled');
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(eventTypes(view)).toContain('run.cancelled');
            expect(view.events[view.events.length - 1]!.type).toBe('run.cancelled');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_without_in_flight_tool_does_not_timeout', async () => {
        const { runtime, store, model, clock } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(runtime, model);
            const stop = await runtime.stop({ taskId: task.taskId, runId: run.runId });
            expect(stop.acknowledged).toBe(true);
            expect(clock.timeoutCount()).toBe(0);
            const view = runView(store, task.taskId, run.runId);
            expect(view.state).toBe('cancelled');
            expect(view.events[view.events.length - 1]!.data).toMatchObject({
                safe_boundary: true,
            });
            expect(taskState(store, task.taskId)).toBe('cancelled');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_ack_produces_tool_completion_then_cancelled', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_ack',
                tool: 'read_file',
                arguments: { path: 'a.ts' },
            });
            await until(() => releaseTool !== null, 'in-flight tool');
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            expect(runView(store, task.taskId, run.runId).state).toBe('running');
            releaseTool!({ status: 'ok', output: { text: 'done' } });
            const receipt = await stopPromise;
            expect(receipt.acknowledged).toBe(true);
            const view = runView(store, task.taskId, run.runId);
            const completed = eventIndex(view, 'tool.completed');
            const cancelled = eventIndex(view, 'run.cancelled');
            expect(completed).toBeGreaterThan(-1);
            expect(cancelled).toBeGreaterThan(completed);
            expect(view.events[cancelled]!.data).toMatchObject({
                safe_boundary: true,
            });
            expect(view.state).toBe('cancelled');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('terminal_events_are_authoritative_over_run_status', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_status',
                tool: 'read_file',
                arguments: {},
            });
            await until(() => releaseTool !== null, 'in-flight tool');
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(view.state).toBe('running');
            expect(view.statusProjection).toBe('cancelling');
            expect(view.state).not.toBe('cancelled');
            releaseTool!({ status: 'ok', output: {} });
            await stopPromise;
            expect(runView(store, task.taskId, run.runId).state).toBe('cancelled');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_ack_timeout_marks_waiting_for_background', async () => {
        const executor: RuntimeToolExecutor = () => new Promise<RuntimeToolResult>(() => {});
        const { runtime, store, taskStore, model, clock } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_timeout',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => eventTypes(runView(store, task.taskId, run.runId)).includes('tool.started'),
                'tool started',
            );
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            clock.advance(29_999);
            await flush();
            expect(runView(store, task.taskId, run.runId).state).toBe('running');
            clock.advance(1);
            const receipt = await stopPromise;
            expect(receipt.acknowledged).toBe(false);
            const view = runView(store, task.taskId, run.runId);
            expect(view.state).toBe('failed');
            const failed = view.events[view.events.length - 1]!;
            expect(failed.type).toBe('run.failed');
            expect(failed.data).toMatchObject({
                code: 'stop_unacknowledged',
                stop_outcome: 'ack_timeout',
                in_flight_tool_id: 'tool_timeout',
                retryable: false,
            });
            const durable = await taskStore.listTasks({ includeArchived: true });
            expect(durable[0]!.hold).toEqual({
                reason: 'waiting_for_background',
                run_id: run.runId,
                tool_call_id: 'tool_timeout',
            });
            expect(taskState(store, task.taskId)).toBe('waiting_for_background');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('host_resolution_clears_background_hold', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, taskStore, model, clock } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_resolution',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(() => releaseTool !== null, 'in-flight tool');
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            clock.advance(30_000);
            await stopPromise;
            const before = runView(store, task.taskId, run.runId);
            const countBefore = before.events.length;

            releaseTool!({ status: 'ok', output: { late: true } });
            await flush(300);
            expect(runView(store, task.taskId, run.runId).events).toHaveLength(
                countBefore,
            );

            const input: ResolveBackgroundInput = {
                taskId: task.taskId,
                runId: run.runId,
                toolCallId: 'tool_resolution',
            };
            const receipt = await runtime.resolveBackground(input);
            expect(receipt.cleared).toBe(true);
            const durable = await taskStore.listTasks({ includeArchived: true });
            expect(durable[0]!.hold).toBeUndefined();
            expect(taskState(store, task.taskId)).toBe('failed');
            const after = runView(store, task.taskId, run.runId);
            expect(after.events).toHaveLength(countBefore);
            expect(eventTypes(after)).not.toContain('tool.completed');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_host_refusal_marks_idle', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, taskStore, model } = makeRuntime(
            {
                requestStopAck: async () => 'refused' as const,
            },
            executor,
        );
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_refused',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(() => releaseTool !== null, 'in-flight tool');
            const receipt = await runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            expect(receipt.acknowledged).toBe(false);
            const view = runView(store, task.taskId, run.runId);
            expect(view.state).toBe('failed');
            const failed = view.events[view.events.length - 1]!;
            expect(failed.data).toMatchObject({
                code: 'host_denied',
                stop_outcome: 'host_refused',
                in_flight_tool_id: 'tool_refused',
                retryable: false,
            });
            expect(taskState(store, task.taskId)).toBe('failed');
            const durable = await taskStore.listTasks({ includeArchived: true });
            expect(durable[0]!.hold).toBeUndefined();
            releaseTool!({ status: 'ok', output: { late: true } });
            await flush(300);
            expect(eventTypes(runView(store, task.taskId, run.runId))).not.toContain(
                'tool.completed',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('canonical_tool_arguments_reject_floats_negative_integers_and_non_ascii_keys', async () => {
        const cases: Array<Record<string, unknown>> = [
            { ratio: 1.5 },
            { count: -1 },
            { 'non-ascii': 1, 'é': 2 },
        ];
        for (const argumentsValue of cases) {
            const dispatched: string[] = [];
            const executor: RuntimeToolExecutor = async (intent) => {
                dispatched.push(intent.tool);
                return { status: 'ok', output: {} };
            };
            const { runtime, store, model } = makeRuntime({}, executor);
            await runtime.open();
            try {
                const { task, run, stream } = await startOne(runtime, model);
                stream.push({
                    type: 'tool-call',
                    toolCallId: 'tool_bad_args',
                    tool: 'write_file',
                    arguments: argumentsValue,
                });
                await until(
                    () => runView(store, task.taskId, run.runId).state === 'failed',
                    'invalid arguments failure',
                );
                expect(dispatched).toEqual([]);
                const view = runView(store, task.taskId, run.runId);
                expect(eventTypes(view)).not.toContain('tool.started');
                expect(view.events[view.events.length - 1]!.data).toMatchObject({
                    code: 'invalid_arguments',
                    retryable: false,
                });
            } finally {
                model.streams.forEach((stream) => stream.end());
                await runtime.dispose();
            }
        }
    });

    test('canonical_tool_results_are_validated_before_recording_changes', async () => {
        const executor: RuntimeToolExecutor = async () => ({
            status: 'ok',
            output: { count: -1 },
            files: [{ path: 'a.ts' }],
            head_digest: `sha256:${'b'.repeat(64)}`,
        });
        const { runtime, store, taskStore, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_bad_result',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'failed',
                'invalid result failure',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).toContain('tool.failed');
            expect(eventTypes(view)).not.toContain('changes.snapshot');
            expect(view.changesSnapshots).toHaveLength(0);
            expect(taskStore.changes).toHaveLength(0);
            expect(view.error?.code).toBe('invalid_arguments');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });
});

describe('AgentRuntime approvals', () => {
    test('approval_resolution_requires_matching_digest', async () => {
        const dispatched: string[] = [];
        const executor: RuntimeToolExecutor = async (intent) => {
            dispatched.push(intent.tool);
            return { status: 'ok', output: { written: true } };
        };
        const { runtime, store, model } = makeRuntime(
            {
                requiresApproval: (tool) => tool === 'write_file',
            },
            executor,
        );
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_approval',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => runView(store, task.taskId, run.runId).approvals.length === 1,
                'approval required',
            );
            const approval = runView(store, task.taskId, run.runId).approvals[0]!;
            expect(runtime.approvalResolvable(approval.approvalId)).toBe(true);
            await expect(
                runtime.resolveHostApproval({
                    approvalId: approval.approvalId,
                    decision: 'approved',
                    taskId: task.taskId,
                    runId: run.runId,
                    generation: run.generation,
                    requestDigest: `sha256:${'0'.repeat(64)}`,
                }),
            ).rejects.toMatchObject({ code: 'approval_digest_mismatch' });
            expect(dispatched).toEqual([]);

            const receipt = await runtime.resolveApproval({
                approvalId: approval.approvalId,
                decision: 'approved',
            });
            expect(receipt).toEqual({
                approvalId: approval.approvalId,
                decision: 'approved',
            });
            expect(runtime.approvalResolvable(approval.approvalId)).toBe(false);
            await until(() => dispatched.length === 1, 'approved dispatch');
            await until(() => model.requests.length >= 2, 'post-approval turn');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after approval',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('approval_identity_must_match_task_run_and_generation', async () => {
        const executor: RuntimeToolExecutor = async () => ({
            status: 'ok',
            output: {},
        });
        const { runtime, store, model } = makeRuntime(
            { requiresApproval: () => true },
            executor,
        );
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_identity',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => runView(store, task.taskId, run.runId).approvals.length === 1,
                'approval required',
            );
            const approval = runView(store, task.taskId, run.runId).approvals[0]!;
            await expect(
                runtime.resolveHostApproval({
                    approvalId: approval.approvalId,
                    decision: 'approved',
                    taskId: task.taskId,
                    runId: run.runId,
                    generation: run.generation + 1,
                    requestDigest: approval.requestDigest,
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });
            await expect(
                runtime.resolveHostApproval({
                    approvalId: approval.approvalId,
                    decision: 'approved',
                    taskId: 'tsk_1111111111111111',
                    runId: run.runId,
                    generation: run.generation,
                    requestDigest: approval.requestDigest,
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });
            await runtime.resolveApproval({
                approvalId: approval.approvalId,
                decision: 'denied',
            });
            await until(
                () => eventTypes(runView(store, task.taskId, run.runId)).includes('tool.failed'),
                'denied tool failure',
            );
            expect(eventTypes(runView(store, task.taskId, run.runId))).not.toContain(
                'tool.started',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('approval_expiry_refuses_late_resolution', async () => {
        const dispatched: string[] = [];
        const executor: RuntimeToolExecutor = async (intent) => {
            dispatched.push(intent.tool);
            return { status: 'ok', output: {} };
        };
        const { runtime, store, model, clock } = makeRuntime(
            {
                requiresApproval: (tool) => tool === 'write_file',
            },
            executor,
        );
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_expired',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => runView(store, task.taskId, run.runId).approvals.length === 1,
                'approval required',
            );
            const approval = runView(store, task.taskId, run.runId).approvals[0]!;
            clock.advance(600_001);
            await expect(
                runtime.resolveApproval({
                    approvalId: approval.approvalId,
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'approval_expired' });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'failed',
                'expired approval failure',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(dispatched).toEqual([]);
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(view.error?.code).toBe('approval_expired');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });
});

describe('AgentRuntime retry and leases', () => {
    test('retry_only_accepts_failed_retryable_or_cancelled', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished run',
            );
            await expect(
                runtime.retry({ taskId: task.taskId, runId: run.runId }),
            ).rejects.toMatchObject({ code: 'run_not_retryable' });

            const stopped = await startOne(runtime, model, 'second prompt');
            await runtime.stop({
                taskId: stopped.task.taskId,
                runId: stopped.run.runId,
            });
            const retried = await runtime.retry({
                taskId: stopped.task.taskId,
                runId: stopped.run.runId,
            });
            expect(retried.generation).toBe(stopped.run.generation + 1);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('retry_of_run_id_is_a_string_for_retry_and_never_for_fresh_start', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            expect(eventData(runView(store, task.taskId, run.runId), 1).retry_of_run_id).toBeNull();
            stream.push({
                type: 'failed',
                code: 'model_unavailable',
                message: 'synthetic failure',
                retryable: false,
            });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'failed',
                'nonretryable failure',
            );
            await expect(
                runtime.retry({ taskId: task.taskId, runId: run.runId }),
            ).rejects.toMatchObject({ code: 'run_not_retryable' });
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('runtime_acquires_heartbeats_and_releases_a_unique_writer_lease', async () => {
        const { runtime, taskStore, clock, model } = makeRuntime();
        await runtime.open();
        try {
            expect(clock.intervalCount()).toBe(1);
            const { task, run, stream } = await startOne(runtime, model);
            expect(taskStore.acquires).toHaveLength(1);
            expect(taskStore.acquires[0]).toMatchObject({
                taskId: task.taskId,
                writerId: 'writer-runtime-test',
            });
            const startedAt = clock.nowMs;
            clock.advance(LEASE_HEARTBEAT_MS);
            await flush();
            expect(taskStore.heartbeats.length).toBeGreaterThanOrEqual(1);
            const lease = taskStore.leaseFor(task.taskId) as {
                started_at_ms: number;
                heartbeat_at_ms: number;
            };
            expect(lease.started_at_ms).toBe(startedAt);
            expect(lease.heartbeat_at_ms).toBe(startedAt + LEASE_HEARTBEAT_MS);

            stream.end();
            await until(
                () => taskStore.releases.length === 1,
                'lease release on terminal',
            );
            expect(taskStore.releases[0]).toMatchObject({
                taskId: task.taskId,
                writerId: 'writer-runtime-test',
            });
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('lease_liveness_is_strictly_greater_than_thirty_seconds', () => {
        expect(isLeaseStale(1_000_000, 1_000_000 - 30_000)).toBe(false);
        expect(isLeaseStale(1_000_001, 1_000_001 - 30_001)).toBe(true);
    });

    test('dispose_settles_timers_and_queued_work_exactly_once', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, taskStore, model, clock } = makeRuntime({}, executor);
        await runtime.open();
        const { task, run, stream } = await startOne(runtime, model);
        stream.push({
            type: 'tool-call',
            toolCallId: 'tool_dispose',
            tool: 'read_file',
            arguments: {},
        });
        await until(() => releaseTool !== null, 'in-flight tool');
        void runtime.stop({ taskId: task.taskId, runId: run.runId });
        await until(
            () =>
                runView(store, task.taskId, run.runId).statusProjection ===
                'cancelling',
            'cancelling projection',
        );
        await runtime.dispose();
        await runtime.dispose();
        expect(clock.timeoutCount()).toBe(0);
        expect(clock.intervalCount()).toBe(0);
        expect(taskStore.releases).toHaveLength(1);
        clock.advance(30_001);
        await flush(300);
        expect(runView(store, task.taskId, run.runId).state).toBe('running');
        releaseTool!({ status: 'ok', output: {} });
        await flush(300);
        expect(eventTypes(runView(store, task.taskId, run.runId))).not.toContain(
            'tool.completed',
        );
    });
});

describe('AgentRuntime fix round 1', () => {
    test('agent_client_subscribes_to_real_runtime_events_exactly_once', async () => {
        const { runtime, store, model } = makeRuntime();
        const client = new AgentClient({ runtime });
        await client.open();
        const stream = client.subscribe();
        try {
            const task = await client.createTask({ title: 'client runtime' });
            const run = await client.startRun({
                taskId: task.taskId,
                prompt: 'client prompt',
            });
            const iterator = stream[Symbol.asyncIterator]();
            const first = await nextEvent(iterator);
            const second = await nextEvent(iterator);
            expect(first?.value).toMatchObject({ type: 'run.queued' });
            expect(second?.value).toMatchObject({ type: 'run.started' });

            await until(
                () =>
                    store.getRunCursor(task.taskId, run.runId)!.lastSequence >= 2,
                'turn started before ingest',
            );
            const cursor = store.getRunCursor(task.taskId, run.runId);
            expect(cursor).not.toBeNull();
            const event = {
                schema: AGENT_EVENT_SCHEMA,
                event_id: `evt_${run.runId}_${cursor!.generation}_${(
                    cursor!.lastSequence + 1
                )
                    .toString()
                    .padStart(12, '0')}`,
                project_id: store.snapshot().tasks.find(
                    (candidate) => candidate.taskId === task.taskId,
                )!.projectId,
                task_id: task.taskId,
                run_id: run.runId,
                generation: cursor!.generation,
                sequence: cursor!.lastSequence + 1,
                recorded_at_ms: cursor!.lastSequence + 2,
                type: 'assistant.delta',
                data: {
                    message_id: 'msg_ingested',
                    text_delta: 'ingested',
                },
            } as unknown as AgentIncomingEvent;
            const turnStarted = await nextEvent(iterator);
            expect(turnStarted?.value).toMatchObject({ type: 'turn.started' });
            const appended = await client.ingestWireEvent(canonicalJson(event));
            expect(appended.status).toBe('applied');
            const third = await nextEvent(iterator);
            expect(third?.value).toMatchObject({
                type: 'assistant.delta',
                data: { message_id: 'msg_ingested', text_delta: 'ingested' },
            });
            model.streams.forEach((stream) => stream.end());
        } finally {
            model.streams.forEach((stream) => stream.end());
            await client.dispose();
        }
    });

    test('approval_double_resolution_is_rejected_before_a_second_decision_event', async () => {
        let dispatched = 0;
        const executor: RuntimeToolExecutor = async () => {
            dispatched += 1;
            return { status: 'ok', output: {} };
        };
        const { runtime, store, model } = makeRuntime(
            { requiresApproval: () => true },
            executor,
        );
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_double',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(
                () => runView(store, task.taskId, run.runId).approvals.length === 1,
                'approval required',
            );
            const approval = runView(store, task.taskId, run.runId).approvals[0]!;
            const results = await Promise.allSettled([
                runtime.resolveApproval({
                    approvalId: approval.approvalId,
                    decision: 'approved',
                }),
                runtime.resolveApproval({
                    approvalId: approval.approvalId,
                    decision: 'denied',
                }),
            ]);
            expect(
                results.filter((result) => result.status === 'fulfilled'),
            ).toHaveLength(1);
            expect(
                results.filter((result) => result.status === 'rejected'),
            ).toHaveLength(1);
            const view = runView(store, task.taskId, run.runId);
            expect(
                view.events.filter(
                    (event) => event.type === 'tool.approval_resolved',
                ),
            ).toHaveLength(1);
            expect(dispatched).toBeLessThanOrEqual(1);
            await until(() => model.requests.length >= 2, 'post-approval turn');
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after double resolution',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_timeout_write_failure_settles_receipt_without_terminal_fabrication', async () => {
        const executor: RuntimeToolExecutor = () => new Promise<RuntimeToolResult>(() => {});
        const { runtime, store, taskStore, model, clock } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_timeout_fault',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(() => eventTypes(runView(store, task.taskId, run.runId)).includes('tool.started'), 'tool started');
            taskStore.failNextList = true;
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            clock.advance(30_000);
            const receipt = await Promise.race([
                stopPromise,
                flush(300).then(() => null),
            ]);
            expect(receipt).not.toBeNull();
            expect(receipt!.acknowledged).toBe(false);
            expect(runtime.lastPersistenceError?.code).toBe('limit_exceeded');
            expect(clock.timeoutCount()).toBe(0);
            expect(taskStore.releases).toHaveLength(1);
            const view = runView(store, task.taskId, run.runId);
            expect(view.state).toBe('running');
            expect(eventTypes(view)).not.toContain('run.failed');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_refusal_write_failure_settles_receipt_without_unhandled_rejection', async () => {
        const executor: RuntimeToolExecutor = () => new Promise<RuntimeToolResult>(() => {});
        let releaseRefusal: (() => void) | null = null;
        const refusalGate = new Promise<void>((resolve) => {
            releaseRefusal = resolve;
        });
        const { runtime, store, taskStore, model } = makeRuntime(
            {
                requestStopAck: async () => {
                    await refusalGate;
                    return 'refused' as const;
                },
            },
            executor,
        );
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_refusal_fault',
                tool: 'write_file',
                arguments: { path: 'a.ts' },
            });
            await until(() => eventTypes(runView(store, task.taskId, run.runId)).includes('tool.started'), 'tool started');
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            taskStore.failNextAppend = true;
            releaseRefusal!();
            const receipt = await Promise.race([
                stopPromise,
                flush(300).then(() => null),
            ]);
            expect(receipt).not.toBeNull();
            expect(receipt!.acknowledged).toBe(false);
            expect(runtime.lastPersistenceError?.code).toBe('limit_exceeded');
            expect(taskStore.releases).toHaveLength(1);
            expect(runView(store, task.taskId, run.runId).state).toBe('running');
            expect(eventTypes(runView(store, task.taskId, run.runId))).not.toContain('run.failed');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('no_inflight_cancelled_append_failure_settles_false_stop_and_releases_lease', async () => {
        const { runtime, store, taskStore, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            const originalAppend = taskStore.appendEvent.bind(taskStore);
            let failCancelled = true;
            taskStore.appendEvent = async (input) => {
                if (failCancelled && input.event.type === 'run.cancelled') {
                    failCancelled = false;
                    return {
                        status: 'rejected' as const,
                        code: 'limit_exceeded' as const,
                        message: 'injected no-in-flight cancellation failure',
                        event_id: input.event.event_id,
                        digest: null,
                        storage: 'indexeddb' as const,
                    };
                }
                return originalAppend(input);
            };
            const before = runView(store, task.taskId, run.runId);
            const receipt = await runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            expect(receipt.acknowledged).toBe(false);
            expect(runtime.lastPersistenceError?.code).toBe('limit_exceeded');
            expect(taskStore.releases).toHaveLength(1);
            expect(taskStore.leaseFor(task.taskId)).toBeNull();
            expect(
                (runtime as unknown as { active: Map<string, unknown> }).active
                    .has(task.taskId),
            ).toBe(false);
            expect(
                (runtime as unknown as { taskQueues: Map<string, unknown> })
                    .taskQueues.has(task.taskId),
            ).toBe(false);
            const after = runView(store, task.taskId, run.runId);
            expect(after.lastSequence).toBe(before.lastSequence + 1);
            expect(eventTypes(after).at(-1)).toBe('run.status');
            expect(eventTypes(after)).not.toContain('run.cancelled');
            stream.push({ type: 'text-delta', text: 'advanced sequence must not append' });
            await flush(100);
            expect(runView(store, task.taskId, run.runId).lastSequence).toBe(
                after.lastSequence,
            );
            await expect(
                runtime.stop({ taskId: task.taskId, runId: run.runId }),
            ).resolves.toMatchObject({ acknowledged: true });
            const settled = runView(store, task.taskId, run.runId);
            expect(settled.state).toBe('cancelled');
            expect(eventTypes(settled).at(-1)).toBe('run.cancelled');
        } finally {
            model.streams.forEach((item) => item.end());
            await runtime.dispose();
        }
    });

    test('boundary_fallback_write_failure_settles_pending_and_timer', async () => {
        const { runtime, store, taskStore, model, clock } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(runtime, model);
            await runtime.steer({ taskId: task.taskId, text: 'fallback fault' });
            taskStore.failNextAppend = true;
            clock.advance(30_000);
            await flush(300);
            expect(runtime.lastPersistenceError?.code).toBe('limit_exceeded');
            expect(clock.timeoutCount()).toBe(0);
            expect(eventTypes(runView(store, task.taskId, run.runId))).not.toContain(
                'message.steered',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('stop_settles_when_model_iterator_return_rejects', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.rejectReturn = true;
            const receipt = await runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            expect(receipt.acknowledged).toBe(true);
            expect(runView(store, task.taskId, run.runId).state).toBe('cancelled');
            expect(runtime.lastPersistenceError?.code).toBe('internal');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('dispose_settles_when_model_iterator_return_rejects', async () => {
        const { runtime, model } = makeRuntime();
        await runtime.open();
        const { stream } = await startOne(runtime, model);
        stream.rejectReturn = true;
        await expect(runtime.dispose()).resolves.toBeUndefined();
    });

    test('retry_without_original_prompt_is_refused_honestly', async () => {
        const { runtime, store, taskStore, model, clock } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'failed',
                code: 'model_unavailable',
                message: 'synthetic failure',
                retryable: true,
            });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'failed',
                'failed run',
            );
            const modelB = new ControlledModel();
            const runtimeB = new AgentRuntime({
                store,
                taskStore: taskStore as unknown as AgentTaskStore,
                modelPort: modelB,
                toolExecutor: async () => ({ status: 'ok', output: {} }),
                now: clock.now,
                timers: clock,
                writerId: 'writer-runtime-b',
                ids: deterministicIds(),
                leaseHeartbeatMs: LEASE_HEARTBEAT_MS,
            });
            await runtimeB.open();
            try {
                await expect(
                    runtimeB.retry({ taskId: task.taskId, runId: run.runId }),
                ).rejects.toMatchObject({ code: 'invalid_arguments' });
            } finally {
                modelB.streams.forEach((stream) => stream.end());
                await runtimeB.dispose();
            }
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('force_appends_to_the_most_recent_user_turn_at_a_tool_boundary', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model, 'base prompt');
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_force',
                tool: 'read_file',
                arguments: {},
            });
            await until(() => releaseTool !== null, 'tool execution');
            await runtime.forceSend({ taskId: task.taskId, text: 'force text' });
            releaseTool!({ status: 'ok', output: {} });
            await until(() => model.requests.length >= 2, 'post-force model turn');
            expect(model.requests[1]!.prompt).toBe('base prompt\nforce text');
            expect(model.requests[1]!.history).toHaveLength(1);
            expect(model.requests[1]!.history[0]).toMatchObject({
                role: 'user',
                text: 'base prompt\nforce text',
            });
            model.streams[1]!.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after force',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('cancelling_rejects_new_messages_and_fails_accepted_pending_messages', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_pending',
                tool: 'read_file',
                arguments: {},
            });
            await until(() => releaseTool !== null, 'tool execution');
            const queued = await runtime.queueMessage({
                taskId: task.taskId,
                text: 'queued before stop',
            });
            const steered = await runtime.steer({
                taskId: task.taskId,
                text: 'steered before stop',
            });
            const stopPromise = runtime.stop({
                taskId: task.taskId,
                runId: run.runId,
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).statusProjection ===
                    'cancelling',
                'cancelling projection',
            );
            await expect(
                runtime.queueMessage({ taskId: task.taskId, text: 'late queue' }),
            ).rejects.toMatchObject({ code: 'out_of_order' });
            await expect(
                runtime.steer({ taskId: task.taskId, text: 'late steer' }),
            ).rejects.toMatchObject({ code: 'out_of_order' });
            await expect(
                runtime.forceSend({ taskId: task.taskId, text: 'late force' }),
            ).rejects.toMatchObject({ code: 'out_of_order' });
            releaseTool!({ status: 'ok', output: {} });
            const receipt = await stopPromise;
            expect(receipt.acknowledged).toBe(true);
            const view = runView(store, task.taskId, run.runId);
            const errors = view.events.filter((event) => event.type === 'error');
            expect(
                errors.some((event) => String(event.data.message).includes(queued.messageId)),
            ).toBe(true);
            expect(
                errors.some((event) => String(event.data.message).includes(steered.messageId)),
            ).toBe(true);
            expect(view.state).toBe('cancelled');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('constructor_rejects_a_task_store_with_a_different_agent_store', () => {
        const { taskStore, clock } = makeRuntime();
        const otherStore = createAgentStore();
        expect(
            () =>
                new AgentRuntime({
                    store: otherStore,
                    taskStore: taskStore as unknown as AgentTaskStore,
                    modelPort: new ControlledModel(),
                    now: clock.now,
                    timers: clock,
                    ids: deterministicIds(),
                }),
        ).toThrowError(/invalid_arguments/);
    });

    test('heartbeat_keeps_one_request_in_flight_without_chain_growth', async () => {
        const { runtime, taskStore, clock, model } = makeRuntime();
        await runtime.open();
        try {
            const { stream } = await startOne(runtime, model);
            let releaseHeartbeat: (() => void) | null = null;
            taskStore.heartbeatGate = new Promise<void>((resolve) => {
                releaseHeartbeat = resolve;
            });
            clock.advance(LEASE_HEARTBEAT_MS);
            await flush();
            expect(taskStore.heartbeats).toHaveLength(1);
            clock.advance(LEASE_HEARTBEAT_MS);
            await flush();
            expect(taskStore.heartbeats).toHaveLength(1);
            releaseHeartbeat!();
            await flush(100);
            expect(taskStore.heartbeats).toHaveLength(1);
            stream.end();
            await until(
                () => taskStore.releases.length === 1,
                'lease release',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('resume_appends_exact_g_plus_1_n_plus_1_previous_generation_and_digest', async () => {
        const { runtime, store, model } = makeRuntime({
            taskStore: null as unknown as AgentTaskStore,
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            const cursor = store.getRunCursor(task.taskId, run.runId)!;
            const resumed = await runtime.resumeTask({
                taskId: task.taskId,
                runId: run.runId,
            });
            expect(resumed.snapshot).toMatchObject({
                taskId: task.taskId,
                runId: run.runId,
                generation: cursor.generation,
                lastSequence: cursor.lastSequence,
                lastEventId: cursor.lastEventId,
                headDigest: cursor.lastDigest,
            });
            expect(resumed.event.generation).toBe(cursor.generation + 1);
            expect(resumed.event.sequence).toBe(cursor.lastSequence + 1);
            expect(resumed.event.data).toMatchObject({
                previous_generation: cursor.generation,
                snapshot_digest: cursor.lastDigest,
            });
            stream.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after resume',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });
});

describe('ReplayModelPort deterministic scripts', () => {
    test('replay_script_yields_tool_usage_finished_and_failed_events', async () => {
        const port = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tool_script',
                        tool: 'write_file',
                        arguments: { path: 'a.ts' },
                    },
                    {
                        type: 'usage',
                        inputTokens: 2,
                        outputTokens: 3,
                        modelCalls: 1,
                    },
                    { type: 'finished', reason: 'tool' },
                ],
                [
                    { type: 'text-delta', text: 'scripted text' },
                    { type: 'failed', code: 'model_unavailable', message: 'fail', retryable: true },
                ],
            ],
        });
        const request: ModelRequest = {
            taskId: 'tsk_0000000000000001',
            runId: 'run_0000000000000001',
            generation: 1,
            mode: 'agent',
            prompt: 'script',
            history: [{ role: 'user', text: 'script' }],
        };
        const first: ModelStreamEvent[] = [];
        for await (const event of port.open(request)) first.push(event);
        const second: ModelStreamEvent[] = [];
        for await (const event of port.open(request)) second.push(event);
        expect(first.map((event) => event.type)).toEqual([
            'tool-call',
            'usage',
            'finished',
        ]);
        expect(second.map((event) => event.type)).toEqual([
            'text-delta',
            'failed',
        ]);
    });

    test('replay_port_drives_approval_write_changes_and_finish_through_runtime', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const taskStore = new FakeTaskStore(store, clock);
        const files = [{ path: 'src/scripted.ts', change: 'added' }];
        const headDigest = `sha256:${'c'.repeat(64)}`;
        const runtime = new AgentRuntime({
            store,
            taskStore: taskStore as unknown as AgentTaskStore,
            modelPort: new ReplayModelPort({
                script: [
                    [
                        {
                            type: 'tool-call',
                            toolCallId: 'tool_scripted_write',
                            tool: 'write_file',
                            arguments: { path: 'src/scripted.ts' },
                        },
                        {
                            type: 'usage',
                            inputTokens: 4,
                            outputTokens: 2,
                            modelCalls: 1,
                        },
                        { type: 'finished', reason: 'tool' },
                    ],
                    [
                        { type: 'text-delta', text: 'scripted completion' },
                        {
                            type: 'usage',
                            inputTokens: 4,
                            outputTokens: 2,
                            modelCalls: 1,
                        },
                        { type: 'finished', reason: 'stop' },
                    ],
                ],
            }),
            toolExecutor: async () => ({
                status: 'ok',
                output: { written: 1 },
                files,
                head_digest: headDigest,
            }),
            requiresApproval: () => true,
            now: clock.now,
            timers: clock,
            writerId: 'writer-replay-port',
            ids: deterministicIds(),
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'replay script' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'scripted prompt',
            });
            await until(
                () => runView(store, task.taskId, run.runId).approvals.length === 1,
                'scripted approval',
            );
            const approval = runView(store, task.taskId, run.runId).approvals[0]!;
            await runtime.resolveApproval({
                approvalId: approval.approvalId,
                decision: 'approved',
            });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'scripted finish',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(view.error).toBeNull();
            expect(view.changesSnapshots[0]).toMatchObject({
                scope: 'last_turn',
                files,
                headDigest,
            });
            expect(taskStore.changes[0]).toMatchObject({
                files,
                head_digest: headDigest,
            });
            expect(eventTypes(view)).toContain('tool.completed');
            expect(eventTypes(view)).toContain('assistant.message');
        } finally {
            await runtime.dispose();
        }
    });

    test('replay_port_drives_a_retryable_failure_through_runtime', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const taskStore = new FakeTaskStore(store, clock);
        const runtime = new AgentRuntime({
            store,
            taskStore: taskStore as unknown as AgentTaskStore,
            modelPort: new ReplayModelPort({
                script: [
                    [
                        {
                            type: 'failed',
                            code: 'model_unavailable',
                            message: 'synthetic replay failure',
                            retryable: true,
                        },
                    ],
                ],
            }),
            now: clock.now,
            timers: clock,
            writerId: 'writer-replay-failure',
            ids: deterministicIds(),
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'replay failure' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'fail',
            });
            await until(
                () => runView(store, task.taskId, run.runId).state === 'failed',
                'replay failure',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(view.error).toEqual({
                code: 'model_unavailable',
                message: 'synthetic replay failure',
                retryable: true,
            });
            expect(eventTypes(view)).toContain('run.failed');
        } finally {
            await runtime.dispose();
        }
    });
});

const INGEST_TASK_A = 'tsk_00000000000000a1';
const INGEST_TASK_B = 'tsk_00000000000000b2';
const INGEST_RUN_A = 'run_00000000000000a1';
const INGEST_RUN_B = 'run_00000000000000b2';
const INGEST_DIGEST = `sha256:${'c'.repeat(64)}`;

function runtimeEvent(input: {
    taskId: string;
    runId: string;
    sequence: number;
    type: string;
    data: Record<string, unknown>;
    generation?: number;
}): AgentIncomingEvent {
    const generation = input.generation ?? 1;
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: `evt_${input.runId}_${generation}_${input.sequence
            .toString()
            .padStart(12, '0')}`,
        project_id: PROJECT_ID,
        task_id: input.taskId,
        run_id: input.runId,
        generation,
        sequence: input.sequence,
        recorded_at_ms: 1_800_000_000_000 + input.sequence,
        type: input.type,
        data: input.data,
    } as unknown as AgentIncomingEvent;
}

function frame(event: AgentIncomingEvent): Uint8Array {
    return new TextEncoder().encode(`${canonicalJson(event)}\n`);
}

function streamForTask(
    model: ControlledModel,
    taskId: string,
): DeferredModelStream {
    let index = -1;
    for (let cursor = 0; cursor < model.requests.length; cursor += 1) {
        if (model.requests[cursor]!.taskId === taskId) index = cursor;
    }
    if (index < 0) throw new Error(`model request for ${taskId} is missing`);
    return model.streams[index]!;
}

describe('AgentRuntime Task 13 two live tasks and hostile ingestion', () => {
    test('two_live_tasks_keep_independent_cursors', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const taskA = await runtime.createTask({ title: 'live A' });
            const taskB = await runtime.createTask({ title: 'live B' });
            const runA = await runtime.startRun({
                taskId: taskA.taskId,
                prompt: 'live A prompt',
            });
            const runB = await runtime.startRun({
                taskId: taskB.taskId,
                prompt: 'live B prompt',
            });
            await until(
                () =>
                    model.requests.some(
                        (request) => request.taskId === taskA.taskId,
                    ) &&
                    model.requests.some(
                        (request) => request.taskId === taskB.taskId,
                    ),
                'two live model turns',
            );

            const cursorB = store.getRunCursor(taskB.taskId, runB.runId)!;
            streamForTask(model, taskA.taskId).push({
                type: 'text-delta',
                text: 'alpha only',
            });
            await until(
                () =>
                    runView(store, taskA.taskId, runA.runId).messages.length ===
                    1,
                'task A delta',
            );
            expect(store.getRunCursor(taskB.taskId, runB.runId)).toEqual(
                cursorB,
            );
            expect(
                runView(store, taskB.taskId, runB.runId).messages,
            ).toEqual([]);

            const streamA = streamForTask(model, taskA.taskId);
            streamA.push({ type: 'finished', reason: 'stop' });
            streamA.end();
            await until(
                () =>
                    runView(store, taskA.taskId, runA.runId).state ===
                    'finished',
                'task A finished',
            );
            expect(
                runView(store, taskB.taskId, runB.runId).state,
            ).toBe('running');
            expect(store.getRunCursor(taskB.taskId, runB.runId)).toEqual(
                cursorB,
            );

            const streamB = streamForTask(model, taskB.taskId);
            streamB.push({ type: 'text-delta', text: 'beta only' });
            streamB.push({ type: 'finished', reason: 'stop' });
            streamB.end();
            await until(
                () =>
                    runView(store, taskB.taskId, runB.runId).state ===
                    'finished',
                'task B finished',
            );
            expect(
                runView(store, taskA.taskId, runA.runId).messages.map(
                    (message) => message.text,
                ),
            ).toEqual(['alpha only']);
            expect(
                runView(store, taskB.taskId, runB.runId).messages.map(
                    (message) => message.text,
                ),
            ).toEqual(['beta only']);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('two_live_tasks_do_not_share_artifacts_or_approvals', async () => {
        const executor: RuntimeToolExecutor = async (intent) => ({
            status: 'ok',
            output: { path: `artifacts/${intent.taskId}.txt` },
            files: [
                {
                    path: `artifacts/${intent.taskId}.txt`,
                    change: 'added',
                },
            ],
            head_digest: `sha256:${intent.taskId.endsWith('1') ? '1'.repeat(64) : '2'.repeat(64)}`,
        });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const taskA = await runtime.createTask({ title: 'artifact A' });
            const taskB = await runtime.createTask({ title: 'artifact B' });
            const runA = await runtime.startRun({
                taskId: taskA.taskId,
                prompt: 'artifact A prompt',
            });
            const runB = await runtime.startRun({
                taskId: taskB.taskId,
                prompt: 'artifact B prompt',
            });
            await until(
                () =>
                    model.requests.some(
                        (request) => request.taskId === taskA.taskId,
                    ) &&
                    model.requests.some(
                        (request) => request.taskId === taskB.taskId,
                    ),
                'two live model turns',
            );
            streamForTask(model, taskA.taskId).push({
                type: 'tool-call',
                toolCallId: 'tool_artifact_a',
                tool: 'write_file',
                arguments: {},
            });
            streamForTask(model, taskB.taskId).push({
                type: 'tool-call',
                toolCallId: 'tool_artifact_b',
                tool: 'write_file',
                arguments: {},
            });
            await until(
                () =>
                    model.requests.filter(
                        (request) => request.taskId === taskA.taskId,
                    ).length >= 2 &&
                    model.requests.filter(
                        (request) => request.taskId === taskB.taskId,
                    ).length >= 2,
                'post-tool turns',
            );

            const changesA = await runtime.changesSnapshot({
                taskId: taskA.taskId,
                runId: runA.runId,
                scope: 'last_turn',
            });
            const changesB = await runtime.changesSnapshot({
                taskId: taskB.taskId,
                runId: runB.runId,
                scope: 'last_turn',
            });
            expect(changesA?.files).toEqual([
                { path: `artifacts/${taskA.taskId}.txt`, change: 'added' },
            ]);
            expect(changesB?.files).toEqual([
                { path: `artifacts/${taskB.taskId}.txt`, change: 'added' },
            ]);
            expect(changesA?.headDigest).not.toBe(changesB?.headDigest);
            expect(
                store.latestChangesSnapshot(taskA.taskId, runA.runId)?.files,
            ).toEqual([
                { path: `artifacts/${taskA.taskId}.txt`, change: 'added' },
            ]);

            const streamA = streamForTask(model, taskA.taskId);
            streamA.push({ type: 'finished', reason: 'stop' });
            streamA.end();
            const streamB = streamForTask(model, taskB.taskId);
            streamB.push({ type: 'finished', reason: 'stop' });
            streamB.end();
            await until(
                () =>
                    runView(store, taskA.taskId, runA.runId).state ===
                        'finished' &&
                    runView(store, taskB.taskId, runB.runId).state ===
                        'finished',
                'both artifact tasks finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('two_live_tasks_keep_independent_tool_approvals', async () => {
        const executor: RuntimeToolExecutor = async () => ({
            status: 'ok',
            output: {},
        });
        const { runtime, store, model } = makeRuntime(
            { requiresApproval: (tool) => tool === 'write_file' },
            executor,
        );
        await runtime.open();
        try {
            const taskA = await runtime.createTask({ title: 'approval A' });
            const taskB = await runtime.createTask({ title: 'approval B' });
            const runA = await runtime.startRun({
                taskId: taskA.taskId,
                prompt: 'approval A prompt',
            });
            const runB = await runtime.startRun({
                taskId: taskB.taskId,
                prompt: 'approval B prompt',
            });
            await until(
                () =>
                    model.requests.some(
                        (request) => request.taskId === taskA.taskId,
                    ) &&
                    model.requests.some(
                        (request) => request.taskId === taskB.taskId,
                    ),
                'two approval model turns',
            );
            streamForTask(model, taskA.taskId).push({
                type: 'tool-call',
                toolCallId: 'tool_approval_a',
                tool: 'write_file',
                arguments: { path: 'a.txt' },
            });
            streamForTask(model, taskB.taskId).push({
                type: 'tool-call',
                toolCallId: 'tool_approval_b',
                tool: 'write_file',
                arguments: { path: 'b.txt' },
            });
            await until(
                () =>
                    runView(store, taskA.taskId, runA.runId).approvals
                        .length === 1 &&
                    runView(store, taskB.taskId, runB.runId).approvals
                        .length === 1,
                'both tool approvals',
            );
            const approvalA = runView(store, taskA.taskId, runA.runId)
                .approvals[0]!;
            const approvalB = runView(store, taskB.taskId, runB.runId)
                .approvals[0]!;
            expect(approvalA.approvalId).not.toBe(approvalB.approvalId);
            expect(approvalA.requestDigest).not.toBe(approvalB.requestDigest);

            await runtime.resolveApproval({
                approvalId: approvalA.approvalId,
                decision: 'approved',
            });
            await until(
                () =>
                    model.requests.filter(
                        (request) => request.taskId === taskA.taskId,
                    ).length >= 2,
                'task A post-approval turn',
            );
            expect(
                runView(store, taskB.taskId, runB.runId).approvals[0]
                    ?.decision,
            ).toBeNull();
            expect(
                runView(store, taskB.taskId, runB.runId).toolCalls[0]?.status,
            ).toBe('requested');

            await runtime.resolveApproval({
                approvalId: approvalB.approvalId,
                decision: 'approved',
            });
            await until(
                () =>
                    model.requests.filter(
                        (request) => request.taskId === taskB.taskId,
                    ).length >= 2,
                'task B post-approval turn',
            );
            const streamA = streamForTask(model, taskA.taskId);
            streamA.push({ type: 'finished', reason: 'stop' });
            streamA.end();
            const streamB = streamForTask(model, taskB.taskId);
            streamB.push({ type: 'finished', reason: 'stop' });
            streamB.end();
            await until(
                () =>
                    runView(store, taskA.taskId, runA.runId).state ===
                        'finished' &&
                    runView(store, taskB.taskId, runB.runId).state ===
                        'finished',
                'both approval runs finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('one_task_hostile_and_terminal_input_cannot_mutate_another_live_task', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const taskA = await runtime.createTask({ title: 'hostile A' });
            const taskB = await runtime.createTask({ title: 'hostile B' });
            const runA = await runtime.startRun({
                taskId: taskA.taskId,
                prompt: 'hostile A prompt',
            });
            const runB = await runtime.startRun({
                taskId: taskB.taskId,
                prompt: 'hostile B prompt',
            });
            await until(
                () =>
                    model.requests.some(
                        (request) => request.taskId === taskA.taskId,
                    ) &&
                    model.requests.some(
                        (request) => request.taskId === taskB.taskId,
                    ),
                'two live model turns',
            );
            const workerA = runtime.createIngest({
                taskId: taskA.taskId,
                runId: runA.runId,
            });
            const workerB = runtime.createIngest({
                taskId: taskB.taskId,
                runId: runB.runId,
            });
            const cursorB = store.getRunCursor(taskB.taskId, runB.runId)!;
            const nextSequence =
                runView(store, taskA.taskId, runA.runId).lastSequence + 1;

            const unknown = await workerA.push(
                frame(
                    runtimeEvent({
                        taskId: taskA.taskId,
                        runId: runA.runId,
                        sequence: nextSequence,
                        type: 'assistant.message.forged',
                        data: {
                            message_id: 'msg_forged',
                            text: 'forged assistant output',
                            tool_call_id: 'tool_forged',
                            status: 'ok',
                        },
                    }),
                ),
            );
            expect(unknown[0]?.kind).toBe('unknown');
            const cross = await workerA.push(
                frame(
                    runtimeEvent({
                        taskId: taskB.taskId,
                        runId: runB.runId,
                        sequence: 2,
                        type: 'assistant.message',
                        data: {
                            message_id: 'msg_cross',
                            text: 'cross task output',
                        },
                    }),
                ),
            );
            expect(cross[0]?.code()).toBe('cross_task_event');
            const workerA2 = runtime.createIngest({
                taskId: taskA.taskId,
                runId: runA.runId,
            });
            const malformed = await workerA2.push(
                new TextEncoder().encode('{hostile}\n'),
            );
            expect(malformed[0]?.code()).toBe('malformed_event');

            expect(store.getRunCursor(taskB.taskId, runB.runId)).toEqual(
                cursorB,
            );
            expect(
                runView(store, taskB.taskId, runB.runId).messages,
            ).toEqual([]);
            expect(runView(store, taskA.taskId, runA.runId).messages).toEqual(
                [],
            );
            expect(runView(store, taskA.taskId, runA.runId).state).toBe(
                'running',
            );

            const streamA = streamForTask(model, taskA.taskId);
            streamA.push({ type: 'finished', reason: 'stop' });
            streamA.end();
            await until(
                () =>
                    runView(store, taskA.taskId, runA.runId).state ===
                    'finished',
                'hostile task A finished',
            );
            expect(
                runView(store, taskB.taskId, runB.runId).state,
            ).toBe('running');
            expect(store.getRunCursor(taskB.taskId, runB.runId)).toEqual(
                cursorB,
            );

            const honestB = await workerB.push(
                frame(
                    runtimeEvent({
                        taskId: taskB.taskId,
                        runId: runB.runId,
                        sequence:
                            runView(store, taskB.taskId, runB.runId)
                                .lastSequence + 1,
                        type: 'assistant.message',
                        data: {
                            message_id: 'msg_b',
                            text: 'task B still writes honestly',
                        },
                    }),
                ),
            );
            expect(honestB[0]?.kind).toBe('applied');
            expect(runView(store, taskA.taskId, runA.runId).messages).toEqual(
                [],
            );
            const streamB = streamForTask(model, taskB.taskId);
            streamB.push({ type: 'finished', reason: 'stop' });
            streamB.end();
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('runtime_ingest_cursor_expiry_resyncs_without_fabricated_completion', async () => {
        const { runtime, store, model } = makeRuntime({
            taskStore: null as unknown as AgentTaskStore,
        });
        await runtime.open();
        try {
            const { task, run } = await startOne(runtime, model);
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const outcome = await worker.cursorExpired();
            expect(outcome.code()).toBe('cursor_expired');
            const view = runView(store, task.taskId, run.runId);
            expect(view.generation).toBe(2);
            expect(view.events.at(-1)?.type).toBe('run.resumed');
            expect(view.events.map((event) => event.type)).not.toContain(
                'run.finished',
            );
            expect(view.events.map((event) => event.type)).not.toContain(
                'assistant.message',
            );
            expect(view.messages).toEqual([]);

            const fresh = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const stale = await fresh.push(
                frame(
                    runtimeEvent({
                        taskId: task.taskId,
                        runId: run.runId,
                        generation: 1,
                        sequence: 3,
                        type: 'assistant.message',
                        data: {
                            message_id: 'msg_stale',
                            text: 'stale output',
                        },
                    }),
                ),
            );
            expect(stale[0]?.code()).toBe('stale_generation');
            expect(
                runView(store, task.taskId, run.runId).messages,
            ).toEqual([]);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('runtime_ingest_preserves_unknown_events_without_output', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(runtime, model);
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const unknown = await worker.push(
                frame(
                    runtimeEvent({
                        taskId: task.taskId,
                        runId: run.runId,
                        sequence:
                            runView(store, task.taskId, run.runId)
                                .lastSequence + 1,
                        type: 'assistant.attachment.forged',
                        data: {
                            message_id: 'msg_forged',
                            text: 'forged output',
                            output_digest: INGEST_DIGEST,
                        },
                    }),
                ),
            );
            expect(unknown[0]?.kind).toBe('unknown');
            const view = runView(store, task.taskId, run.runId);
            expect(view.unknownEvents).toHaveLength(1);
            expect(view.messages).toEqual([]);
            expect(view.toolCalls).toEqual([]);
            model.streams.forEach((stream) => stream.end());
        } finally {
            await runtime.dispose();
        }
    });

    test('dispose_respects_a_borrowed_task_store_owner', async () => {
        const { runtime, taskStore } = makeRuntime({ ownsTaskStore: false });
        await runtime.open();
        expect(taskStore.isOpen).toBe(true);
        await runtime.dispose();
        expect(taskStore.isOpen).toBe(true);
        expect(taskStore.closed).toBe(false);
    });
});

describe('AgentRuntime Task 13 replay corrections', () => {
    test('replay_model_one_turn_tool_script_terminates_instead_of_repeating', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const port = new ReplayModelPort({
            script: [
                [
                    {
                        type: 'tool-call',
                        toolCallId: 'tool_single_turn',
                        tool: 'read_file',
                        arguments: {},
                    },
                ],
            ],
        });
        let executions = 0;
        const runtime = new AgentRuntime({
            store,
            taskStore: null as unknown as AgentTaskStore,
            modelPort: port,
            toolExecutor: async () => {
                executions += 1;
                return { status: 'ok', output: { executions } };
            },
            now: clock.now,
            timers: clock,
            writerId: 'writer-single-turn',
            ids: deterministicIds(),
        });
        await runtime.open();
        try {
            const task = await runtime.createTask({ title: 'one turn' });
            const run = await runtime.startRun({
                taskId: task.taskId,
                prompt: 'one turn prompt',
            });
            await until(
                () =>
                    runView(store, task.taskId, run.runId).state ===
                    'finished',
                'one-turn script termination',
                120,
            );
            const view = runView(store, task.taskId, run.runId);
            expect(executions).toBe(1);
            expect(
                view.events.filter((event) => event.type === 'tool.requested'),
            ).toHaveLength(1);
            expect(view.events.at(-1)?.type).toBe('run.finished');
        } finally {
            await runtime.dispose();
        }
    });

    test('steer_prompt_uses_latest_history_text_while_force_merges_the_base_turn', async () => {
        let releaseTool: ((result: RuntimeToolResult) => void) | null = null;
        const executor: RuntimeToolExecutor = () =>
            new Promise<RuntimeToolResult>((resolve) => {
                releaseTool = resolve;
            });
        const { runtime, store, model } = makeRuntime({}, executor);
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(
                runtime,
                model,
                'base prompt',
            );
            stream.push({
                type: 'tool-call',
                toolCallId: 'tool_steer_force',
                tool: 'read_file',
                arguments: {},
            });
            await until(() => releaseTool !== null, 'tool execution');
            await runtime.steer({
                taskId: task.taskId,
                text: 'steer text',
            });
            await runtime.forceSend({
                taskId: task.taskId,
                text: 'force text',
            });
            releaseTool!({ status: 'ok', output: {} });
            await until(() => model.requests.length >= 2, 'steered turn');
            const request = model.requests[1]!;
            expect(request.prompt).toBe('steer text');
            expect(request.history).toEqual([
                { role: 'user', text: 'base prompt\nforce text' },
                { role: 'user', text: 'steer text' },
            ]);
            const streamNext = model.streams[1]!;
            streamNext.push({ type: 'finished', reason: 'stop' });
            streamNext.end();
            await until(
                () => runView(store, task.taskId, run.runId).state === 'finished',
                'finished after steer and force',
            );
        } finally {
            model.streams.forEach((item) => item.end());
            await runtime.dispose();
        }
    });

    test('pending_approval_is_reconstructed_from_verified_events_after_reopen', async () => {
        const store = createAgentStore();
        await seedRunInStore(store);
        const expiresAtMs = 1_900_000_000_000;
        await appendToStore(
            store,
            runtimeEvent({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
                sequence: 2,
                type: 'tool.requested',
                data: {
                    tool_call_id: 'tool_restored',
                    tool: 'write_file',
                    arguments_digest: INGEST_DIGEST,
                    arguments_preview: '{"path":"restored.txt"}',
                },
            }),
        );
        await appendToStore(
            store,
            runtimeEvent({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
                sequence: 3,
                type: 'tool.approval_required',
                data: {
                    approval_id: 'appr_restored',
                    tool_call_id: 'tool_restored',
                    tool: 'write_file',
                    scope: 'tool:write_file',
                    request_digest: INGEST_DIGEST,
                    expires_at_ms: expiresAtMs,
                },
            }),
        );
        const clock = new TestClock();
        const runtime = new AgentRuntime({
            store,
            taskStore: null as unknown as AgentTaskStore,
            now: clock.now,
            timers: clock,
            ids: deterministicIds(),
        });
        await runtime.open();
        try {
            const restored = await runtime.pendingApprovals(INGEST_TASK_A);
            expect(restored).toEqual([
                {
                    approvalId: 'appr_restored',
                    taskId: INGEST_TASK_A,
                    runId: INGEST_RUN_A,
                    generation: 1,
                    toolCallId: 'tool_restored',
                    tool: 'write_file',
                    scope: 'tool:write_file',
                    requestDigest: INGEST_DIGEST,
                    expiresAtMs,
                    expired: false,
                },
            ]);
            await expect(
                runtime.resolveApproval({
                    approvalId: 'appr_restored',
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });
            const view = runView(store, INGEST_TASK_A, INGEST_RUN_A);
            expect(view.approvals[0]?.decision).toBeNull();
            expect(
                view.events.filter(
                    (event) => event.type === 'tool.approval_resolved',
                ),
            ).toHaveLength(0);
        } finally {
            await runtime.dispose();
        }
    });
});

async function seedRunInStore(store: AgentStore): Promise<void> {
    await appendToStore(
        store,
        runtimeEvent({
            taskId: INGEST_TASK_A,
            runId: INGEST_RUN_A,
            sequence: 0,
            type: 'run.queued',
            data: { queued_at_ms: 1_800_000_000_000 },
        }),
    );
    await appendToStore(
        store,
        runtimeEvent({
            taskId: INGEST_TASK_A,
            runId: INGEST_RUN_A,
            sequence: 1,
            type: 'run.started',
            data: {
                started_at_ms: 1_800_000_000_001,
                model: { id: 'replay', kind: 'fixture' },
                mode: 'agent',
                environment: { kind: 'browser-fixture' },
                retry_of_run_id: null,
            },
        }),
    );
}

async function appendToStore(
    store: AgentStore,
    event: AgentIncomingEvent,
): Promise<void> {
    const result = await store.appendEvent(event);
    expect(result.status, `${event.type}: ${result.message ?? ''}`).toBe(
        'applied',
    );
}

describe('ReplayModelPort per-run script positions', () => {
    test('script_position_is_independent_per_live_run_and_exhausts', async () => {
        const firstTurn: ModelStreamEvent[] = [
            { type: 'text-delta', text: 'first scripted turn' },
            { type: 'finished', reason: 'stop' },
        ];
        const secondTurn: ModelStreamEvent[] = [
            { type: 'text-delta', text: 'second scripted turn' },
            { type: 'finished', reason: 'stop' },
        ];
        const port = new ReplayModelPort({
            script: [firstTurn, secondTurn],
        });
        const request = (runId: string): ModelRequest => ({
            taskId: INGEST_TASK_A,
            runId,
            generation: 1,
            mode: 'agent',
            prompt: 'script',
            history: [{ role: 'user', text: 'script' }],
        });
        const collect = async (runId: string): Promise<ModelStreamEvent[]> => {
            const events: ModelStreamEvent[] = [];
            for await (const event of port.open(request(runId))) {
                events.push(event);
            }
            return events;
        };

        expect(await collect(INGEST_RUN_A)).toEqual(firstTurn);
        expect(await collect(INGEST_RUN_B)).toEqual(firstTurn);
        expect(await collect(INGEST_RUN_A)).toEqual(secondTurn);
        expect(await collect(INGEST_RUN_A)).toEqual([]);
        expect(await collect(INGEST_RUN_B)).toEqual(secondTurn);
    });
});

describe('AgentRuntime Task 13 fix round 1 lifecycle and resync', () => {
    function nextFrame(
        taskId: string,
        runId: string,
        generation: number,
        sequence: number,
        type: string,
        data: Record<string, unknown>,
    ): Uint8Array {
        return frame(
            runtimeEvent({
                taskId,
                runId,
                generation,
                sequence,
                type,
                data,
            }),
        );
    }

    test('ingest_worker_rejects_after_dispose_with_an_owned_store', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        const { task, run } = await startOne(runtime, model);
        const cursor = store.getRunCursor(task.taskId, run.runId)!;
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId: run.runId,
        });
        await runtime.dispose();
        const outcomes = await worker.push(
            nextFrame(
                task.taskId,
                run.runId,
                cursor.generation,
                cursor.lastSequence + 1,
                'assistant.message',
                { message_id: 'msg_after_dispose', text: 'must not apply' },
            ),
        );
        expect(outcomes[0]?.code()).toBe('runtime_unavailable');
        expect(store.getRunCursor(task.taskId, run.runId)).toEqual(cursor);
        model.streams.forEach((stream) => stream.end());
    });

    test('ingest_worker_rejects_after_dispose_with_a_borrowed_store', async () => {
        const { runtime, store, taskStore, model } = makeRuntime({
            ownsTaskStore: false,
        });
        await runtime.open();
        const { task, run } = await startOne(runtime, model);
        const cursor = store.getRunCursor(task.taskId, run.runId)!;
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId: run.runId,
        });
        await runtime.dispose();
        expect(taskStore.isOpen).toBe(true);
        const outcomes = await worker.push(
            nextFrame(
                task.taskId,
                run.runId,
                cursor.generation,
                cursor.lastSequence + 1,
                'assistant.message',
                { message_id: 'msg_after_dispose', text: 'must not apply' },
            ),
        );
        expect(outcomes[0]?.code()).toBe('runtime_unavailable');
        expect(store.getRunCursor(task.taskId, run.runId)).toEqual(cursor);
        taskStore.close();
        model.streams.forEach((stream) => stream.end());
    });

    test('ingest_worker_rejects_after_dispose_with_a_null_store', async () => {
        const { runtime, store, model } = makeRuntime({
            taskStore: null as unknown as AgentTaskStore,
        });
        await runtime.open();
        const { task, run } = await startOne(runtime, model);
        const cursor = store.getRunCursor(task.taskId, run.runId)!;
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId: run.runId,
        });
        await runtime.dispose();
        const outcomes = await worker.push(
            nextFrame(
                task.taskId,
                run.runId,
                cursor.generation,
                cursor.lastSequence + 1,
                'assistant.message',
                { message_id: 'msg_after_dispose', text: 'must not apply' },
            ),
        );
        expect(outcomes[0]?.code()).toBe('runtime_unavailable');
        expect(store.getRunCursor(task.taskId, run.runId)).toEqual(cursor);
        model.streams.forEach((stream) => stream.end());
    });

    test('ingested_terminal_event_settles_control_without_a_later_append', async () => {
        const { runtime, store, taskStore, model } = makeRuntime();
        await runtime.open();
        const { task, run, stream } = await startOne(runtime, model);
        const cursor = store.getRunCursor(task.taskId, run.runId)!;
        const worker = runtime.createIngest({
            taskId: task.taskId,
            runId: run.runId,
        });
        const outcomes = await worker.push(
            nextFrame(
                task.taskId,
                run.runId,
                cursor.generation,
                cursor.lastSequence + 1,
                'run.finished',
                {
                    status: 'finished',
                    finished_at_ms: 1_800_000_000_010,
                    duration_ms: 10,
                },
            ),
        );
        expect(outcomes[0]?.kind).toBe('applied');
        const settled = runView(store, task.taskId, run.runId);
        expect(settled.state).toBe('finished');
        const eventCount = settled.events.length;
        await until(
            () => taskStore.releases.length === 1,
            'terminal ingest lease release',
        );

        stream.push({ type: 'text-delta', text: 'late generated output' });
        stream.push({
            type: 'tool-call',
            toolCallId: 'tool_after_terminal',
            tool: 'read_file',
            arguments: {},
        });
        stream.end();
        await flush(30);
        const after = runView(store, task.taskId, run.runId);
        expect(after.events).toHaveLength(eventCount);
        expect(after.messages).toEqual([]);
        expect(eventTypes(after)).not.toContain('tool.requested');

        const next = await runtime.startRun({
            taskId: task.taskId,
            prompt: 'next after terminal ingest',
        });
        expect(next.generation).toBe(cursor.generation + 1);
        model.streams.forEach((item) => item.end());
    });

    test('replacement_worker_after_gap_and_verified_resume_applies_next_frame', async () => {
        const { runtime, store, model } = makeRuntime({
            taskStore: null as unknown as AgentTaskStore,
        });
        await runtime.open();
        const { task, run } = await startOne(runtime, model);
        const cursor = store.getRunCursor(task.taskId, run.runId)!;
        const oldWorker = runtime.createIngest({
            taskId: task.taskId,
            runId: run.runId,
        });
        const gap = await oldWorker.push(
            nextFrame(
                task.taskId,
                run.runId,
                cursor.generation,
                cursor.lastSequence + 2,
                'assistant.delta',
                { message_id: 'msg_gap', text_delta: 'gap output' },
            ),
        );
        expect(gap[0]?.code()).toBe('out_of_order');
        expect(gap[0]?.requiresResync).toBe(true);
        const halted = await oldWorker.push(
            nextFrame(
                task.taskId,
                run.runId,
                cursor.generation,
                cursor.lastSequence + 1,
                'assistant.delta',
                { message_id: 'msg_gap', text_delta: 'old worker output' },
            ),
        );
        expect(halted[0]?.code()).toBe('out_of_order');

        await runtime.resumeTask({ taskId: task.taskId, runId: run.runId });
        const resumed = store.getRunCursor(task.taskId, run.runId)!;
        expect(resumed.generation).toBe(cursor.generation + 1);
        const replacement = runtime.createIngest({
            taskId: task.taskId,
            runId: run.runId,
        });
        const applied = await replacement.push(
            nextFrame(
                task.taskId,
                run.runId,
                resumed.generation,
                resumed.lastSequence + 1,
                'assistant.message',
                {
                    message_id: 'msg_after_resume',
                    text: 'after verified resume',
                },
            ),
        );
        expect(applied[0]?.kind).toBe('applied');
        expect(
            runView(store, task.taskId, run.runId).messages.map(
                (message) => message.text,
            ),
        ).toEqual(['after verified resume']);
        model.streams.forEach((item) => item.end());
    });

    test('two_live_tasks_queue_and_steer_with_differing_generations', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const taskA = await runtime.createTask({ title: 'queue A' });
            const taskB = await runtime.createTask({ title: 'steer B' });
            const runA = await runtime.startRun({
                taskId: taskA.taskId,
                prompt: 'A first generation',
            });
            const runB = await runtime.startRun({
                taskId: taskB.taskId,
                prompt: 'B first generation',
            });
            await until(
                () =>
                    model.requests.some(
                        (request) => request.taskId === taskA.taskId,
                    ) &&
                    model.requests.some(
                        (request) => request.taskId === taskB.taskId,
                    ),
                'two generation-one turns',
            );
            const queued = await runtime.queueMessage({
                taskId: taskA.taskId,
                text: 'A queued second generation',
            });
            const steered = await runtime.steer({
                taskId: taskB.taskId,
                text: 'B steered same generation',
            });
            expect(eventTypes(runView(store, taskA.taskId, runA.runId))).toContain(
                'message.queued',
            );
            expect(
                eventTypes(runView(store, taskB.taskId, runB.runId)),
            ).not.toContain('message.queued');
            expect(
                eventTypes(runView(store, taskB.taskId, runB.runId)),
            ).not.toContain('message.steered');

            const streamA = streamForTask(model, taskA.taskId);
            streamA.push({ type: 'finished', reason: 'stop' });
            streamA.end();
            await until(
                () =>
                    model.requests.filter(
                        (request) => request.taskId === taskA.taskId,
                    ).length >= 2,
                'A queued second generation',
            );
            const taskViewA = store
                .snapshot()
                .tasks.find((candidate) => candidate.taskId === taskA.taskId)!;
            expect(taskViewA.latestRunId).not.toBe(runA.runId);
            const runA2 = runView(
                store,
                taskA.taskId,
                taskViewA.latestRunId!,
            );
            expect(runA2.generation).toBe(2);
            expect(runA2.events[0]?.type).toBe('run.queued');
            expect(runA2.events[1]?.type).toBe('run.started');
            expect(
                model.requests.filter(
                    (request) => request.taskId === taskA.taskId,
                )[1]!.prompt,
            ).toBe('A queued second generation');
            expect(
                eventTypes(runView(store, taskB.taskId, runB.runId)),
            ).not.toContain('message.queued');
            expect(runView(store, taskB.taskId, runB.runId).generation).toBe(1);

            const streamB = streamForTask(model, taskB.taskId);
            streamB.push({
                type: 'tool-call',
                toolCallId: 'tool_steer_generation',
                tool: 'read_file',
                arguments: {},
            });
            await until(
                () =>
                    model.requests.filter(
                        (request) => request.taskId === taskB.taskId,
                    ).length >= 2,
                'B steered same generation',
            );
            const requestB = model.requests.filter(
                (request) => request.taskId === taskB.taskId,
            )[1]!;
            expect(requestB.prompt).toBe('B steered same generation');
            expect(requestB.generation).toBe(1);
            expect(
                eventTypes(runView(store, taskB.taskId, runB.runId)),
            ).toContain('message.steered');
            expect(runView(store, taskA.taskId, runA2.runId).generation).toBe(2);
            expect(
                runView(store, taskA.taskId, runA2.runId).events.map(
                    (event) => event.data.message_id,
                ),
            ).not.toContain(steered.messageId);
            expect(
                runView(store, taskB.taskId, runB.runId).events.map(
                    (event) => event.data.message_id,
                ),
            ).not.toContain(queued.messageId);

            const streamA2 = streamForTask(model, taskA.taskId);
            streamA2.push({ type: 'finished', reason: 'stop' });
            streamA2.end();
            const streamB2 = streamForTask(model, taskB.taskId);
            streamB2.push({ type: 'finished', reason: 'stop' });
            streamB2.end();
            await until(
                () =>
                    runView(store, taskA.taskId, runA2.runId).state ===
                        'finished' &&
                    runView(store, taskB.taskId, runB.runId).state ===
                        'finished',
                'queue and steer runs finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });
});

describe('ReplayModelPort script position bound', () => {
    test('per_run_script_positions_are_bounded_without_cross_consuming', async () => {
        const turn: ModelStreamEvent[] = [
            { type: 'text-delta', text: 'bounded script turn' },
            { type: 'finished', reason: 'stop' },
        ];
        const port = new ReplayModelPort({ script: [turn] });
        const request = (runId: string): ModelRequest => ({
            taskId: INGEST_TASK_A,
            runId,
            generation: 1,
            mode: 'agent',
            prompt: 'bounded',
            history: [{ role: 'user', text: 'bounded' }],
        });
        const collect = async (runId: string): Promise<ModelStreamEvent[]> => {
            const events: ModelStreamEvent[] = [];
            for await (const event of port.open(request(runId))) {
                events.push(event);
            }
            return events;
        };
        for (let index = 0; index < 400; index += 1) {
            await collect(`run_${index.toString(16).padStart(16, '0')}`);
        }
        const bounded = port as unknown as { scriptPositionCount: number };
        expect(bounded.scriptPositionCount).toBeLessThanOrEqual(256);
        expect(
            await collect(`run_${'f'.repeat(16)}`),
        ).toEqual(turn);
    });
});

describe('AgentRuntime Task 13 fix round 2 ingested terminal delivery', () => {
    function terminalFrame(
        taskId: string,
        runId: string,
        generation: number,
        sequence: number,
        type: string,
        data: Record<string, unknown>,
    ): Uint8Array {
        return frame(
            runtimeEvent({
                taskId,
                runId,
                generation,
                sequence,
                type,
                data,
            }),
        );
    }

    function messageIndex(
        view: RunView,
        type: string,
        messageId: string,
        after = -1,
    ): number {
        return view.events.findIndex(
            (event, index) =>
                index > after &&
                event.type === type &&
                event.data.message_id === messageId,
        );
    }

    test('ingested_finished_flushes_pending_steer_and_force_before_terminal_and_starts_queued_run', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(
                runtime,
                model,
                'terminal delivery base',
            );
            const queued = await runtime.queueMessage({
                taskId: task.taskId,
                text: 'queued first generation two',
            });
            const steered = await runtime.steer({
                taskId: task.taskId,
                text: 'late steer',
            });
            const forced = await runtime.forceSend({
                taskId: task.taskId,
                text: 'late force',
            });
            const cursor = store.getRunCursor(task.taskId, run.runId)!;
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const outcomes = await worker.push(
                terminalFrame(
                    task.taskId,
                    run.runId,
                    cursor.generation,
                    cursor.lastSequence + 1,
                    'run.finished',
                    {
                        status: 'finished',
                        finished_at_ms: 1_800_000_000_020,
                        duration_ms: 20,
                    },
                ),
            );
            expect(outcomes[0]?.kind).toBe('applied');

            const view = runView(store, task.taskId, run.runId);
            const types = eventTypes(view);
            const finishedIndex = types.lastIndexOf('run.finished');
            expect(types.at(-1)).toBe('run.finished');
            const steeredIndex = messageIndex(
                view,
                'message.steered',
                steered.messageId,
            );
            const forcedIndex = messageIndex(
                view,
                'message.forced',
                forced.messageId,
            );
            const steeredQueued = messageIndex(
                view,
                'message.queued',
                steered.messageId,
                steeredIndex,
            );
            const forcedQueued = messageIndex(
                view,
                'message.queued',
                forced.messageId,
                forcedIndex,
            );
            for (const index of [
                steeredIndex,
                forcedIndex,
                steeredQueued,
                forcedQueued,
            ]) {
                expect(index).toBeGreaterThan(-1);
                expect(index).toBeLessThan(finishedIndex);
            }
            expect(steeredQueued).toBeGreaterThan(steeredIndex);
            expect(forcedQueued).toBeGreaterThan(forcedIndex);
            expect(
                view.events.filter((event) => event.type === 'error'),
            ).toHaveLength(0);

            const taskView = store
                .snapshot()
                .tasks.find((candidate) => candidate.taskId === task.taskId)!;
            expect(taskView.latestRunId).not.toBe(run.runId);
            const nextRun = runView(
                store,
                task.taskId,
                taskView.latestRunId!,
            );
            expect(nextRun.generation).toBe(2);
            expect(nextRun.events[0]?.type).toBe('run.queued');
            expect(nextRun.events[1]?.type).toBe('run.started');
            await until(
                () =>
                    model.requests.filter(
                        (request) => request.taskId === task.taskId,
                    ).length >= 2,
                'queued generation-two run',
            );
            expect(
                model.requests.filter(
                    (request) => request.taskId === task.taskId,
                ).at(-1)!.prompt,
            ).toBe('queued first generation two');
            expect(queued.messageId).toBeTruthy();

            const nextStream = streamForTask(model, task.taskId);
            nextStream.push({ type: 'finished', reason: 'stop' });
            nextStream.end();
            await until(
                () =>
                    runView(store, task.taskId, nextRun.runId).state ===
                    'finished',
                'queued generation-two finished',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('ingested_failed_fails_pending_and_queued_before_terminal_without_new_run', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(
                runtime,
                model,
                'failed terminal base',
            );
            await runtime.queueMessage({
                taskId: task.taskId,
                text: 'queued before failure',
            });
            await runtime.steer({
                taskId: task.taskId,
                text: 'pending steer before failure',
            });
            await runtime.forceSend({
                taskId: task.taskId,
                text: 'pending force before failure',
            });
            const cursor = store.getRunCursor(task.taskId, run.runId)!;
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const outcomes = await worker.push(
                terminalFrame(
                    task.taskId,
                    run.runId,
                    cursor.generation,
                    cursor.lastSequence + 1,
                    'run.failed',
                    {
                        code: 'model_unavailable',
                        message: 'host terminal failure',
                        retryable: false,
                    },
                ),
            );
            expect(outcomes[0]?.kind).toBe('applied');

            const view = runView(store, task.taskId, run.runId);
            const types = eventTypes(view);
            const terminalIndex = types.lastIndexOf('run.failed');
            expect(types.at(-1)).toBe('run.failed');
            const errors = view.events
                .map((event, index) => ({ event, index }))
                .filter((entry) => entry.event.type === 'error');
            expect(errors).toHaveLength(3);
            for (const entry of errors) {
                expect(entry.index).toBeLessThan(terminalIndex);
                expect(entry.event.data.code).toBe('message_not_delivered');
            }
            expect(view.error?.code).toBe('model_unavailable');
            const taskView = store
                .snapshot()
                .tasks.find((candidate) => candidate.taskId === task.taskId)!;
            expect(taskView.latestRunId).toBe(run.runId);
            expect(taskView.state).toBe('failed');
            expect(types).not.toContain('message.steered');
            expect(types).not.toContain('message.forced');
            model.streams.forEach((stream) => stream.end());
        } finally {
            await runtime.dispose();
        }
    });

    test('ingested_cancelled_fails_pending_before_terminal_without_new_run', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run } = await startOne(
                runtime,
                model,
                'cancelled terminal base',
            );
            await runtime.queueMessage({
                taskId: task.taskId,
                text: 'queued before cancellation',
            });
            await runtime.steer({
                taskId: task.taskId,
                text: 'pending steer before cancellation',
            });
            const cursor = store.getRunCursor(task.taskId, run.runId)!;
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const outcomes = await worker.push(
                terminalFrame(
                    task.taskId,
                    run.runId,
                    cursor.generation,
                    cursor.lastSequence + 1,
                    'run.cancelled',
                    {
                        safe_boundary: true,
                        finished_at_ms: 1_800_000_000_030,
                    },
                ),
            );
            expect(outcomes[0]?.kind).toBe('applied');
            const view = runView(store, task.taskId, run.runId);
            const types = eventTypes(view);
            const terminalIndex = types.lastIndexOf('run.cancelled');
            expect(types.at(-1)).toBe('run.cancelled');
            const errors = view.events
                .map((event, index) => ({ event, index }))
                .filter((entry) => entry.event.type === 'error');
            expect(errors).toHaveLength(2);
            for (const entry of errors) {
                expect(entry.index).toBeLessThan(terminalIndex);
                expect(entry.event.data.code).toBe('message_not_delivered');
            }
            const taskView = store
                .snapshot()
                .tasks.find((candidate) => candidate.taskId === task.taskId)!;
            expect(taskView.latestRunId).toBe(run.runId);
            expect(taskView.state).toBe('cancelled');
            model.streams.forEach((stream) => stream.end());
        } finally {
            await runtime.dispose();
        }
    });

    test('invalid_ingested_terminal_emits_no_fallback_and_keeps_pending_messages', async () => {
        const { runtime, store, model } = makeRuntime();
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(
                runtime,
                model,
                'invalid terminal base',
            );
            await runtime.queueMessage({
                taskId: task.taskId,
                text: 'queued survives invalid terminal',
            });
            const steered = await runtime.steer({
                taskId: task.taskId,
                text: 'steer survives invalid terminal',
            });
            const forced = await runtime.forceSend({
                taskId: task.taskId,
                text: 'force survives invalid terminal',
            });
            const cursor = store.getRunCursor(task.taskId, run.runId)!;
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const outcomes = await worker.push(
                terminalFrame(
                    task.taskId,
                    run.runId,
                    cursor.generation,
                    cursor.lastSequence + 1,
                    'run.finished',
                    {
                        status: 'failed',
                        finished_at_ms: 1_800_000_000_040,
                        duration_ms: 1,
                    },
                ),
            );
            expect(outcomes[0]?.code()).toBe('invalid_event');
            const untouched = runView(store, task.taskId, run.runId);
            expect(store.getRunCursor(task.taskId, run.runId)).toEqual(cursor);
            expect(eventTypes(untouched)).not.toContain('message.steered');
            expect(eventTypes(untouched)).not.toContain('message.forced');
            expect(
                untouched.events.filter((event) => event.type === 'error'),
            ).toHaveLength(0);

            stream.push({ type: 'finished', reason: 'stop' });
            stream.end();
            await until(
                () =>
                    messageIndex(
                        runView(store, task.taskId, run.runId),
                        'message.steered',
                        steered.messageId,
                    ) >= 0 &&
                    messageIndex(
                        runView(store, task.taskId, run.runId),
                        'message.forced',
                        forced.messageId,
                    ) >= 0,
                'pending messages survive invalid terminal',
            );
            await until(
                () => store.getTaskLatestRunId(task.taskId) !== run.runId,
                'queued run after invalid terminal',
            );
            const taskView = store
                .snapshot()
                .tasks.find((candidate) => candidate.taskId === task.taskId)!;
            expect(taskView.latestRunId).not.toBe(run.runId);
            const nextStream = streamForTask(model, task.taskId);
            nextStream.push({ type: 'finished', reason: 'stop' });
            nextStream.end();
        } finally {
            model.streams.forEach((item) => item.end());
            await runtime.dispose();
        }
    });

    test('cursor_expiry_requires_a_replacement_worker_that_applies_the_next_frame', async () => {
        const { runtime, store, model } = makeRuntime({
            taskStore: null as unknown as AgentTaskStore,
        });
        await runtime.open();
        try {
            const { task, run } = await startOne(
                runtime,
                model,
                'cursor expiry replacement',
            );
            const worker = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const expired = await worker.cursorExpired();
            expect(expired.code()).toBe('cursor_expired');
            const resumed = store.getRunCursor(task.taskId, run.runId)!;
            expect(resumed.generation).toBe(2);
            const replacement = runtime.createIngest({
                taskId: task.taskId,
                runId: run.runId,
            });
            const applied = await replacement.push(
                terminalFrame(
                    task.taskId,
                    run.runId,
                    resumed.generation,
                    resumed.lastSequence + 1,
                    'assistant.message',
                    {
                        message_id: 'msg_after_cursor_expiry',
                        text: 'after cursor expiry resync',
                    },
                ),
            );
            expect(applied[0]?.kind).toBe('applied');
            expect(
                runView(store, task.taskId, run.runId).messages.map(
                    (message) => message.text,
                ),
            ).toEqual(['after cursor expiry resync']);
            model.streams.forEach((stream) => stream.end());
        } finally {
            await runtime.dispose();
        }
    });
});

class RecordingHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly supportsStream = false;
    readonly intents: HostToolIntent[] = [];
    readonly approvals: Array<{
        approvalId: string;
        decision: 'approved' | 'denied';
    }> = [];
    readonly hostApprovalId = 'apr_5c2e8a1f9b7d3e40';
    readonly hostExpiresAtMs = 1_800_000_120_000;
    approvalsCapability = true;
    approveFailure: AgentClientError | null = null;
    preflightExecutes = false;
    executionFailure: { code: AgentErrorCode; message: string } | null = null;

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'none',
            tree: false,
            changes: false,
            readArtifact: false,
            terminal: false,
            git: false,
            mcp: false,
            approvals: this.approvalsCapability,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostToolResult> {
        this.intents.push(intent);
        if (intent.approval_id === undefined) {
            if (this.preflightExecutes) {
                return await hostResult(intent, 'ok', { written: true });
            }
            return await hostResult(
                intent,
                'error',
                {
                    approval_required: true,
                    approval_id: this.hostApprovalId,
                    expires_at_ms: this.hostExpiresAtMs,
                    approved: false,
                },
                'approval_forbidden',
                'host approval is required; approve it in the local companion',
            );
        }
        if (this.executionFailure !== null) {
            return await hostResult(
                intent,
                'error',
                {},
                this.executionFailure.code,
                this.executionFailure.message,
                `hmac-sha256:${'a'.repeat(64)}`,
            );
        }
        return await hostResult(
            intent,
            'ok',
            { written: true },
            undefined,
            undefined,
            `hmac-sha256:${'a'.repeat(64)}`,
        );
    }

    async approve(input: {
        approvalId: string;
        decision: 'approved' | 'denied';
    }): Promise<{ approvalId: string; decision: 'approved' | 'denied' }> {
        this.approvals.push(input);
        if (this.approveFailure !== null) throw this.approveFailure;
        return { approvalId: input.approvalId, decision: input.decision };
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }
}

async function hostResult(
    intent: HostToolIntent,
    status: 'ok' | 'error' | 'cancelled',
    output: Record<string, unknown>,
    code?: string,
    message?: string,
    receipt?: string,
): Promise<HostToolResult> {
    const canonical = canonicalJson(output);
    return {
        schema: HOST_RESULT_SCHEMA,
        intent_id: intent.intent_id,
        task_id: intent.task_id,
        run_id: intent.run_id,
        generation: intent.generation,
        tool_call_id: intent.tool_call_id,
        request_scope: intent.request_scope ?? 'task',
        status,
        output,
        output_digest: await sha256Canonical(output),
        output_preview: canonical.slice(0, 512),
        ...(code === undefined ? {} : { code }),
        ...(message === undefined ? {} : { message }),
        ...(receipt === undefined ? {} : { approval_receipt: receipt }),
        occurred_at_ms: 1_800_000_000_000,
    };
}

function pushHostWrite(stream: {
    push(event: {
        type: 'tool-call';
        toolCallId: string;
        tool: string;
        arguments: Record<string, unknown>;
    }): void;
}): void {
    stream.push({
        type: 'tool-call',
        toolCallId: 'tool_host_write',
        tool: 'file.write',
        arguments: {
            grant_id: 'gr_9a3c5e7b1d2f4a60',
            path: 'note.txt',
            content: 'one',
        },
    });
}

describe('AgentRuntime host approval handshake', () => {
    test('host_approval_preflight_adopts_host_issued_identity', async () => {
        const transport = new RecordingHostTransport();
        const hostClient = new HostClient({ transport });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).approvals.length === 1,
                'host approval required',
            );
            const view = runView(store, task.taskId, run.runId);
            const approval = view.approvals[0]!;
            expect(approval.approvalId).toBe(transport.hostApprovalId);
            expect(approval.expiresAtMs).toBe(transport.hostExpiresAtMs);
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(
                eventTypes(view).filter(
                    (type) => type === 'tool.approval_required',
                ),
            ).toHaveLength(1);

            expect(transport.intents).toHaveLength(1);
            const preflight = transport.intents[0]!;
            expect(preflight.approval_id).toBeUndefined();
            expect(preflight.task_id).toBe(task.taskId);
            expect(preflight.run_id).toBe(run.runId);
            expect(preflight.generation).toBe(run.generation);
            expect(preflight.tool_call_id).toBe('tool_host_write');
            expect(preflight.tool).toBe('file.write');

            const approvalEvent = view.events.find(
                (event) => event.type === 'tool.approval_required',
            )!;
            expect(approvalEvent.data).toMatchObject({
                approval_id: transport.hostApprovalId,
                expires_at_ms: transport.hostExpiresAtMs,
                tool_call_id: 'tool_host_write',
                tool: 'file.write',
            });
            expect(
                transport.intents.some(
                    (intent) => intent.approval_id !== undefined,
                ),
            ).toBe(false);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('host_approval_resolution_uses_exact_payload_and_executes_once', async () => {
        const transport = new RecordingHostTransport();
        const hostClient = new HostClient({ transport });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).approvals.length === 1,
                'host approval required',
            );
            const approval =
                runView(store, task.taskId, run.runId).approvals[0]!;
            const receipt = await runtime.resolveApproval({
                approvalId: approval.approvalId,
                decision: 'approved',
            });
            expect(receipt).toEqual({
                approvalId: transport.hostApprovalId,
                decision: 'approved',
            });
            expect(transport.approvals).toEqual([
                {
                    approvalId: transport.hostApprovalId,
                    decision: 'approved',
                },
            ]);

            await until(
                () => transport.intents.length === 2,
                'one host execution',
            );
            const execution = transport.intents[1]!;
            expect(execution.approval_id).toBe(transport.hostApprovalId);
            expect(execution.task_id).toBe(task.taskId);
            expect(execution.run_id).toBe(run.runId);
            expect(execution.generation).toBe(run.generation);
            expect(execution.tool_call_id).toBe('tool_host_write');

            await until(
                () =>
                    eventTypes(runView(store, task.taskId, run.runId)).includes(
                        'tool.completed',
                    ),
                'host tool completion',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(view.approvals).toHaveLength(1);
            expect(view.approvals[0]?.decision).toBe('approved');
            expect(
                eventTypes(view).filter((type) => type === 'tool.started'),
            ).toHaveLength(1);
            expect(
                eventTypes(view).filter((type) => type === 'tool.completed'),
            ).toHaveLength(1);
            await flush(20);
            expect(transport.intents).toHaveLength(2);
            await expect(
                runtime.resolveApproval({
                    approvalId: transport.hostApprovalId,
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });
            expect(transport.approvals).toHaveLength(1);
            expect(transport.intents).toHaveLength(2);

            model.streams[1]!.end();
            await until(
                () =>
                    runView(store, task.taskId, run.runId).state === 'finished',
                'finished after host approval',
            );
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('host_approval_resolution_failure_does_not_execute', async () => {
        const transport = new RecordingHostTransport();
        transport.approveFailure = new AgentClientError(
            'approval_forbidden',
            'host refused the page decision',
        );
        const hostClient = new HostClient({ transport });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).approvals.length === 1,
                'host approval required',
            );
            await expect(
                runtime.resolveApproval({
                    approvalId: transport.hostApprovalId,
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });
            await flush(20);
            const view = runView(store, task.taskId, run.runId);
            expect(view.approvals).toHaveLength(1);
            expect(transport.intents).toHaveLength(1);
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(eventTypes(view)).not.toContain('tool.approval_resolved');
            expect(eventTypes(view)).not.toContain('tool.completed');
            expect(
                transport.intents.some(
                    (intent) => intent.approval_id !== undefined,
                ),
            ).toBe(false);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('host_approval_denied_resolves_without_execution', async () => {
        const transport = new RecordingHostTransport();
        const hostClient = new HostClient({ transport });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).approvals.length === 1,
                'host approval required',
            );
            await runtime.resolveApproval({
                approvalId: transport.hostApprovalId,
                decision: 'denied',
            });
            expect(transport.approvals).toEqual([
                {
                    approvalId: transport.hostApprovalId,
                    decision: 'denied',
                },
            ]);
            await until(
                () =>
                    eventTypes(runView(store, task.taskId, run.runId)).includes(
                        'tool.failed',
                    ),
                'denied host tool',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(
                view.events.find((event) => event.type === 'tool.failed')!
                    .data,
            ).toMatchObject({ code: 'approval_denied' });
            expect(transport.intents).toHaveLength(1);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('host_approval_cross_task_refusal_is_explicit_and_not_completed', async () => {
        const transport = new RecordingHostTransport();
        transport.executionFailure = {
            code: 'cross_task_event',
            message: 'the approval belongs to a different task',
        };
        const hostClient = new HostClient({ transport });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).approvals.length === 1,
                'host approval required',
            );
            await runtime.resolveApproval({
                approvalId: transport.hostApprovalId,
                decision: 'approved',
            });
            await until(
                () =>
                    eventTypes(runView(store, task.taskId, run.runId)).includes(
                        'tool.failed',
                    ),
                'cross-task host refusal',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).not.toContain('tool.completed');
            expect(
                view.events.find((event) => event.type === 'tool.failed')!
                    .data,
            ).toMatchObject({ code: 'cross_task_event' });
            expect(transport.intents).toHaveLength(2);
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('host_preflight_that_executes_is_refused_without_second_dispatch', async () => {
        const transport = new RecordingHostTransport();
        transport.preflightExecutes = true;
        const hostClient = new HostClient({ transport });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).state === 'failed',
                'preflight execution refusal',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(transport.intents).toHaveLength(1);
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(view.error?.code).toBe('approval_forbidden');
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });

    test('static_host_without_approval_capability_keeps_local_approval_flow', async () => {
        const hostClient = new HostClient({
            transport: new StaticHostTransport(),
        });
        const { runtime, store, model } = makeRuntime({
            hostClient,
            requiresApproval: (tool) => tool === 'file.write',
        });
        await runtime.open();
        try {
            const { task, run, stream } = await startOne(runtime, model);
            pushHostWrite(stream);
            await until(
                () =>
                    runView(store, task.taskId, run.runId).approvals.length === 1,
                'local approval required',
            );
            const approval =
                runView(store, task.taskId, run.runId).approvals[0]!;
            expect(approval.approvalId).toMatch(/^appr_/);
            expect(approval.approvalId).not.toBe('apr_5c2e8a1f9b7d3e40');
            await runtime.resolveApproval({
                approvalId: approval.approvalId,
                decision: 'denied',
            });
            await until(
                () =>
                    eventTypes(runView(store, task.taskId, run.runId)).includes(
                        'tool.failed',
                    ),
                'local denied tool',
            );
            const view = runView(store, task.taskId, run.runId);
            expect(eventTypes(view)).not.toContain('tool.started');
            expect(
                view.events.find((event) => event.type === 'tool.failed')!
                    .data,
            ).toMatchObject({ code: 'approval_denied' });
        } finally {
            model.streams.forEach((stream) => stream.end());
            await runtime.dispose();
        }
    });
});

describe('AgentRuntime restored and disposing reconciliation', () => {
    function restoredRuntime(
        store: AgentStore,
        clock: TestClock,
        taskStore: FakeTaskStore | null,
    ): AgentRuntime {
        return new AgentRuntime({
            store,
            taskStore: taskStore as unknown as AgentTaskStore,
            now: clock.now,
            timers: clock,
            ids: deterministicIds(),
            writerId: 'writer-runtime-test',
            leaseHeartbeatMs: LEASE_HEARTBEAT_MS,
        });
    }

    test('restored_stop_appends_safe_boundary_and_allows_a_new_run', async () => {
        const store = createAgentStore();
        await seedRunInStore(store);
        await appendToStore(
            store,
            runtimeEvent({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
                sequence: 2,
                type: 'assistant.message',
                data: {
                    message_id: 'msg_restored_stop',
                    text: 'restored output',
                },
            }),
        );
        const clock = new TestClock();
        const taskStore = new FakeTaskStore(store, clock);
        const runtime = restoredRuntime(store, clock, taskStore);
        await runtime.open();
        try {
            await expect(
                runtime.stop({
                    taskId: INGEST_TASK_A,
                    runId: INGEST_RUN_B,
                }),
            ).rejects.toMatchObject({ code: 'out_of_order' });
            await expect(
                runtime.stop({
                    taskId: INGEST_TASK_A,
                    runId: 'run_1111111111111111',
                }),
            ).rejects.toMatchObject({ code: 'out_of_order' });

            const receipt = await runtime.stop({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
            });
            expect(receipt).toEqual({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
                acknowledged: true,
            });
            const view = runView(store, INGEST_TASK_A, INGEST_RUN_A);
            expect(view.state).toBe('cancelled');
            expect(view.events.map((event) => event.type).slice(-2)).toEqual([
                'run.status',
                'run.cancelled',
            ]);
            expect(view.events.at(-2)?.data.status).toBe('cancelling');
            expect(view.events.at(-1)?.data.safe_boundary).toBe(true);
            expect(
                view.events.filter((event) => event.type === 'run.cancelled'),
            ).toHaveLength(1);
            expect(taskStore.acquires.map((entry) => entry.taskId)).toContain(
                INGEST_TASK_A,
            );
            expect(taskStore.releases.map((entry) => entry.taskId)).toContain(
                INGEST_TASK_A,
            );

            await expect(
                runtime.stop({
                    taskId: INGEST_TASK_A,
                    runId: INGEST_RUN_A,
                }),
            ).rejects.toMatchObject({ code: 'out_of_order' });

            const next = await runtime.startRun({
                taskId: INGEST_TASK_A,
                prompt: 'after a restored stop',
            });
            expect(next.runId).not.toBe(INGEST_RUN_A);
            await until(
                () =>
                    runView(store, INGEST_TASK_A, next.runId).state ===
                    'finished',
                'new run after restored stop',
            );
        } finally {
            await runtime.dispose();
        }
    });

    test('restored_stop_refuses_a_missing_durable_store', async () => {
        const store = createAgentStore();
        await seedRunInStore(store);
        const clock = new TestClock();
        const runtime = new AgentRuntime({
            store,
            taskStore: null,
            now: clock.now,
            timers: clock,
            ids: deterministicIds(),
        });
        await runtime.open();
        try {
            await expect(
                runtime.stop({
                    taskId: INGEST_TASK_A,
                    runId: INGEST_RUN_A,
                }),
            ).rejects.toMatchObject({ code: 'runtime_unavailable' });
            expect(runView(store, INGEST_TASK_A, INGEST_RUN_A).state).toBe(
                'running',
            );
        } finally {
            await runtime.dispose();
        }
    });

    test('restored_approval_is_not_resolvable_without_an_active_owner', async () => {
        const store = createAgentStore();
        await seedRunInStore(store);
        await appendToStore(
            store,
            runtimeEvent({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
                sequence: 2,
                type: 'tool.requested',
                data: {
                    tool_call_id: 'tool_restored_owner',
                    tool: 'write_file',
                    arguments_digest: INGEST_DIGEST,
                    arguments_preview: '{"path":"restored.txt"}',
                },
            }),
        );
        await appendToStore(
            store,
            runtimeEvent({
                taskId: INGEST_TASK_A,
                runId: INGEST_RUN_A,
                sequence: 3,
                type: 'tool.approval_required',
                data: {
                    approval_id: 'appr_restored_owner',
                    tool_call_id: 'tool_restored_owner',
                    tool: 'write_file',
                    scope: 'tool:write_file',
                    request_digest: INGEST_DIGEST,
                    expires_at_ms: 1_900_000_000_000,
                },
            }),
        );
        const clock = new TestClock();
        const runtime = restoredRuntime(store, clock, null);
        await runtime.open();
        try {
            expect(runtime.approvalResolvable('appr_restored_owner')).toBe(
                false,
            );
            expect(
                await runtime.pendingApprovals(INGEST_TASK_A),
            ).toHaveLength(1);
        } finally {
            await runtime.dispose();
        }
    });

    test('dispose_blocks_new_work_while_disposing', async () => {
        const { runtime, model } = makeRuntime();
        await runtime.open();
        const { stream } = await startOne(runtime, model);
        const disposing = runtime.dispose();
        await expect(
            runtime.createTask({ title: 'during dispose' }),
        ).rejects.toMatchObject({ code: 'runtime_unavailable' });
        stream.end();
        await disposing;
    });

    test('read_diff_comments_reports_unavailable_without_a_task_store', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const runtime = new AgentRuntime({
            store,
            taskStore: null,
            now: clock.now,
            timers: clock,
            ids: deterministicIds(),
        });
        await runtime.open();
        try {
            await expect(
                runtime.readDiffComments(INGEST_TASK_A),
            ).rejects.toMatchObject({ code: 'runtime_unavailable' });
        } finally {
            await runtime.dispose();
        }
        const client = new AgentClient({ store });
        await expect(
            client.readDiffComments(INGEST_TASK_A),
        ).rejects.toMatchObject({ code: 'runtime_unavailable' });
    });
});

describe('AgentRuntime terminal background holds', () => {
    async function seedHold(
        store: AgentStore,
        taskStore: FakeTaskStore,
        terminal: boolean,
    ): Promise<{ taskId: string; runId: string }> {
        await seedRunInStore(store);
        if (terminal) {
            await appendToStore(
                store,
                runtimeEvent({
                    taskId: INGEST_TASK_A,
                    runId: INGEST_RUN_A,
                    sequence: 2,
                    type: 'run.finished',
                    data: {
                        status: 'finished',
                        finished_at_ms: 1_800_000_000_002,
                        duration_ms: 2,
                    },
                }),
            );
        }
        const task = store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === INGEST_TASK_A)!;
        await taskStore.writeTaskRecord({
            task_id: task.taskId,
            project_id: task.projectId,
            title: task.title,
            created_at_ms: task.createdAtMs,
            archived: false,
            pinned_local: false,
            latest_run_id: task.latestRunId,
            hold: {
                reason: 'waiting_for_background',
                run_id: INGEST_RUN_A,
                tool_call_id: 'tool_terminal_hold',
            },
        });
        return { taskId: INGEST_TASK_A, runId: INGEST_RUN_A };
    }

    function holdFor(store: AgentStore): unknown {
        return store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === INGEST_TASK_A)?.hold;
    }

    function runtimeWith(
        store: AgentStore,
        clock: TestClock,
        taskStore: FakeTaskStore,
    ): AgentRuntime {
        return new AgentRuntime({
            store,
            taskStore: taskStore as unknown as AgentTaskStore,
            now: clock.now,
            timers: clock,
            ids: deterministicIds(),
            writerId: 'writer-runtime-test',
            leaseHeartbeatMs: LEASE_HEARTBEAT_MS,
        });
    }

    test('open_clears_a_hold_whose_run_is_terminal', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const taskStore = new FakeTaskStore(store, clock);
        await seedHold(store, taskStore, true);
        expect(holdFor(store)).not.toBeNull();
        const runtime = runtimeWith(store, clock, taskStore);
        await runtime.open();
        try {
            expect(holdFor(store)).toBeNull();
        } finally {
            await runtime.dispose();
        }
    });

    test('a_failed_open_hold_clear_is_not_swallowed_and_a_later_start_clears_it', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const taskStore = new FakeTaskStore(store, clock);
        await seedHold(store, taskStore, true);
        taskStore.failNextTaskWrite = true;
        const runtime = runtimeWith(store, clock, taskStore);
        await expect(runtime.open()).rejects.toMatchObject({
            code: 'limit_exceeded',
        });
        expect(holdFor(store)).not.toBeNull();
        const next = await runtime.startRun({
            taskId: INGEST_TASK_A,
            prompt: 'after a failed open hold clear',
        });
        await until(
            () => runView(store, INGEST_TASK_A, next.runId).state === 'finished',
            'new run after failed open hold clear',
        );
        expect(holdFor(store)).toBeNull();
        await runtime.dispose();
    });

    test('a_failed_hold_clear_after_a_restored_stop_does_not_block_a_later_start', async () => {
        const store = createAgentStore();
        const clock = new TestClock();
        const taskStore = new FakeTaskStore(store, clock);
        await seedHold(store, taskStore, false);
        const runtime = runtimeWith(store, clock, taskStore);
        await runtime.open();
        try {
            taskStore.failNextTaskWrite = true;
            await expect(
                runtime.stop({
                    taskId: INGEST_TASK_A,
                    runId: INGEST_RUN_A,
                }),
            ).rejects.toMatchObject({ code: 'limit_exceeded' });
            expect(holdFor(store)).not.toBeNull();
            expect(runView(store, INGEST_TASK_A, INGEST_RUN_A).state).toBe(
                'cancelled',
            );

            const next = await runtime.startRun({
                taskId: INGEST_TASK_A,
                prompt: 'after a failed restored hold clear',
            });
            await until(
                () =>
                    runView(store, INGEST_TASK_A, next.runId).state ===
                    'finished',
                'new run after failed restored hold clear',
            );
            expect(holdFor(store)).toBeNull();
        } finally {
            await runtime.dispose();
        }
    });
});
