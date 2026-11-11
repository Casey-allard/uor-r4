import { describe, expect, test } from 'vitest';

import {
    AgentClient,
    type ListTasksInput,
    type TaskSummary,
} from './client.js';
import type { AgentRuntime } from './runtime.js';
import { FixtureTransport } from './fixtureTransport.js';
import { createRunQueuedEvent } from './fixtures.js';
import { canonicalJson, digestEvent } from './generated/agentCanonical.js';
import type { AgentEvent } from './generated/agentContract.js';
import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
} from '../state/agent.js';
import type { AgentTaskStore } from './store.js';
import type {
    AppendEventStoreInput,
    AppendStoreResult,
    TaskRecord,
} from './storeTypes.js';
import { AgentStoreError } from './storeTypes.js';

class MemoryTaskStore {
    isOpen = false;
    closed = false;
    readonly writer = 'writer-client-test';
    readonly appended: AppendEventStoreInput[] = [];
    readonly tasks: TaskRecord[] = [];
    failWrites = false;
    rejectNextAppend = false;

    constructor(private readonly target: AgentStore) {}

    get defaultWriterId(): string {
        return this.writer;
    }

    async open(): Promise<{ tasks: TaskRecord[]; runs: unknown[]; incomplete: unknown[] }> {
        this.isOpen = true;
        return { tasks: [], runs: [], incomplete: [] };
    }

    async appendEvent(input: AppendEventStoreInput): Promise<AppendStoreResult> {
        this.appended.push(input);
        if (this.rejectNextAppend) {
            this.rejectNextAppend = false;
            return {
                status: 'rejected',
                code: 'limit_exceeded',
                message: 'injected task-store write failure',
                event_id: input.event.event_id,
                digest: null,
                storage: 'indexeddb',
            };
        }
        const digest = await digestEvent(input.event as AgentEvent);
        const applied = this.target.apply(input.event, digest);
        return {
            status: applied.status === 'ignored' ? 'ignored' : applied.status,
            code: applied.code,
            message: applied.message,
            event_id: input.event.event_id,
            digest,
            storage: null,
        };
    }

    async writeTaskRecord(record: TaskRecord): Promise<TaskRecord> {
        if (this.failWrites) {
            throw new AgentStoreError(
                'limit_exceeded',
                'injected task write failure',
                'indexeddb',
            );
        }
        const normalized = { ...record };
        if (normalized.hold === null) delete normalized.hold;
        this.tasks.push(normalized);
        this.target.upsertTask({
            taskId: record.task_id,
            projectId: record.project_id,
            title: record.title,
            repo: record.repo ?? null,
            branch: record.branch ?? null,
            createdAtMs: record.created_at_ms,
            archived: record.archived,
            pinnedLocal: record.pinned_local,
            latestRunId: record.latest_run_id ?? null,
        });
        return record;
    }

    close(): void {
        this.closed = true;
        this.isOpen = false;
    }
}

function taskStoreFor(store: AgentStore): {
    stub: MemoryTaskStore;
    durable: AgentTaskStore;
} {
    const stub = new MemoryTaskStore(store);
    return { stub, durable: stub as unknown as AgentTaskStore };
}

class EmittingTransport extends FixtureTransport {
    emitForTest(event: AgentIncomingEvent, digest: string): void {
        const listeners = (
            this as unknown as {
                listeners: Set<{
                    listener: (event: AgentIncomingEvent, digest?: string) => void;
                }>;
            }
        ).listeners;
        for (const entry of listeners) entry.listener(event, digest);
    }
}

class HoldTransport extends FixtureTransport {
    hold: {
        reason: 'waiting_for_background';
        run_id: string;
        tool_call_id: string;
    } | null = {
        reason: 'waiting_for_background',
        run_id: 'run_7f3a2c1b9d4e6a80',
        tool_call_id: 'tool_fixture',
    };

    override async listTasks(input: ListTasksInput = {}): Promise<TaskSummary[]> {
        const tasks = await super.listTasks(input);
        return tasks.map(
            (task) =>
                ({
                    ...task,
                    hold: this.hold,
                }) as unknown as TaskSummary,
        );
    }
}

class StubRuntime {
    readonly store = createAgentStore();
    isOpen = false;
    opened = 0;
    disposed = 0;
    readonly startRunInputs: Array<{ taskId: string; prompt: string }> = [];

    async open(): Promise<void> {
        this.isOpen = true;
        this.opened += 1;
    }

    async startRun(input: {
        taskId: string;
        prompt: string;
    }): Promise<{
        taskId: string;
        runId: string;
        generation: number;
        state: 'running';
        retryOfRunId: null;
    }> {
        this.startRunInputs.push(input);
        return {
            taskId: input.taskId,
            runId: 'run_0000000000000001',
            generation: 1,
            state: 'running',
            retryOfRunId: null,
        };
    }

    subscribe(): () => void {
        return () => {};
    }

    async dispose(): Promise<void> {
        await Promise.resolve();
        this.disposed += 1;
    }
}

describe('AgentClient durable task-store wiring', () => {
    test('requires open before persistence-backed operations', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });

        await expect(client.listTasks()).rejects.toMatchObject({
            code: 'runtime_unavailable',
        });
        await client.open();
        await expect(client.listTasks()).resolves.toEqual([]);
    });

    test('persists task summaries through the durable adapter', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();

        const task = await client.createTask({ title: 'durable task' });
        expect(stub.tasks).toContainEqual(
            expect.objectContaining({
                task_id: task.taskId,
                title: 'durable task',
                latest_run_id: null,
            }),
        );
        expect(client.store.snapshot().tasks).toHaveLength(1);
    });

    test('ingest uses the durable append path and preserves writer identity', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();

        const event = createRunQueuedEvent({
            projectId: 'prj_4b1d9e0c7a2f6e51',
            taskId: 'tsk_9a3c5e7b1d2f4a60',
            runId: 'run_7f3a2c1b9d4e6a80',
            generation: 1,
            recordedAtMs: 1_794_283_200_000,
        });
        const result = await client.ingestWireEvent(canonicalJson(event));
        expect(result.status).toBe('applied');
        expect(stub.appended).toHaveLength(1);
        expect(stub.appended[0]!.writer_id).toBe(stub.writer);
        expect(client.store.snapshot().runs).toHaveLength(1);
    });

    test('transport events are appended durably before publishing', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const task = await client.createTask({ title: 'stream task' });
        const stream = client.subscribe();
        try {
            await client.startRun({ taskId: task.taskId, prompt: 'hello' });
            const iterator = stream[Symbol.asyncIterator]();
            const first = await iterator.next();
            const second = await iterator.next();
            expect(first.done).toBe(false);
            expect(second.done).toBe(false);
            expect(first.value?.type).toBe('run.queued');
            expect(second.value?.type).toBe('run.started');
            expect(stub.appended.map((entry) => entry.event.type)).toEqual([
                'run.queued',
                'run.started',
            ]);
            for (const entry of stub.appended) {
                expect(entry.writer_id).toBe(stub.writer);
            }
        } finally {
            stream.close();
            client.dispose();
        }
        expect(stub.closed).toBe(true);
    });

    test('failed task metadata writes leave no non-durable state', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();

        stub.failWrites = true;
        await expect(client.createTask({ title: 'failed create' })).rejects.toMatchObject({
            code: 'limit_exceeded',
            storage: 'indexeddb',
        });
        expect(client.store.snapshot().tasks).toHaveLength(0);

        stub.failWrites = false;
        const task = await client.createTask({ title: 'healthy create' });
        expect(client.store.snapshot().tasks).toHaveLength(1);

        stub.failWrites = true;
        await expect(
            client.pinTask({ taskId: task.taskId, pinned: true }),
        ).rejects.toMatchObject({ code: 'limit_exceeded' });
        expect(
            client.store.snapshot().tasks.find((entry) => entry.taskId === task.taskId)
                ?.pinnedLocal,
        ).toBe(false);

        stub.failWrites = false;
        await client.pinTask({ taskId: task.taskId, pinned: true });
        expect(
            client.store.snapshot().tasks.find((entry) => entry.taskId === task.taskId)
                ?.pinnedLocal,
        ).toBe(true);

        stub.failWrites = true;
        await expect(
            client.archiveTask({ taskId: task.taskId, archived: true }),
        ).rejects.toMatchObject({ code: 'limit_exceeded' });
        expect(
            client.store.snapshot().tasks.find((entry) => entry.taskId === task.taskId)
                ?.archived,
        ).toBe(false);

        stub.failWrites = false;
        await client.archiveTask({ taskId: task.taskId, archived: true });
        expect(
            client.store.snapshot().tasks.find((entry) => entry.taskId === task.taskId)
                ?.archived,
        ).toBe(true);
    });

    test('listTasks does not expose a summary whose durable write failed', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const extra = await transport.createTask({ title: 'undurable summary' });

        stub.failWrites = true;
        await expect(client.listTasks()).rejects.toMatchObject({
            code: 'limit_exceeded',
            storage: 'indexeddb',
        });
        expect(
            client.store.snapshot().tasks.find((task) => task.taskId === extra.taskId),
        ).toBeUndefined();
    });

    test('subscription rejection is surfaced and a healthy retry publishes', async () => {
        const transport = new EmittingTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const stream = client.subscribe();
        try {
            const event = createRunQueuedEvent({
                projectId: 'prj_4b1d9e0c7a2f6e51',
                taskId: 'tsk_9a3c5e7b1d2f4a60',
                runId: 'run_7f3a2c1b9d4e6a80',
                generation: 1,
                recordedAtMs: 1_794_283_200_000,
            });
            const digest = await digestEvent(event as AgentEvent);
            stub.rejectNextAppend = true;
            transport.emitForTest(event, digest);
            await new Promise((resolve) => setTimeout(resolve, 0));
            expect(client.lastPersistenceError).toMatchObject({
                code: 'limit_exceeded',
                storage: 'indexeddb',
            });
            expect(client.store.snapshot().runs).toHaveLength(0);

            transport.emitForTest(event, digest);
            await new Promise((resolve) => setTimeout(resolve, 0));
            const iterator = stream[Symbol.asyncIterator]();
            const next = await iterator.next();
            expect(next.done).toBe(false);
            expect(next.value?.type).toBe('run.queued');
            expect(client.store.snapshot().runs).toHaveLength(1);
            expect(client.lastPersistenceError).toBeNull();
        } finally {
            stream.close();
            client.dispose();
        }
    });

    test('retry is refused for a task with an unloaded latest run', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const task = await client.createTask({ title: 'retry unresolved' });
        client.store.upsertTask({
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            createdAtMs: task.createdAtMs,
            latestRunId: 'run_1111111111111111',
        });
        const before = transport.calls.filter((call) => call.method === 'retry').length;
        await expect(
            client.retry({ taskId: task.taskId, runId: 'run_1111111111111111' }),
        ).rejects.toMatchObject({ code: 'out_of_order' });
        expect(transport.calls.filter((call) => call.method === 'retry')).toHaveLength(before);
    });

    test('direct resume rejection preserves storage', async () => {
        const transport = new FixtureTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const task = await client.createTask({ title: 'resume storage' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'initial' });
        for (let attempt = 0; attempt < 20; attempt += 1) {
            const cursor = client.store.getRunCursor(task.taskId, run.runId);
            if (cursor !== null && cursor.lastSequence >= 1) break;
            await new Promise((resolve) => setTimeout(resolve, 0));
        }
        stub.rejectNextAppend = true;
        await expect(
            client.resumeTask({ taskId: task.taskId, runId: run.runId }),
        ).rejects.toMatchObject({
            code: 'limit_exceeded',
            storage: 'indexeddb',
        });
    });

    test('optional hold metadata survives a durable summary write', async () => {
        const transport = new HoldTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const task = await client.createTask({ title: 'held task' });
        await client.listTasks();
        const stored = stub.tasks
            .filter((record) => record.task_id === task.taskId)
            .at(-1);
        expect(stored?.hold).toEqual(transport.hold);

        transport.hold = null;
        await client.listTasks();
        const cleared = stub.tasks
            .filter((record) => record.task_id === task.taskId)
            .at(-1);
        expect(cleared?.hold).toBeUndefined();
    });

    test('rejects ambiguous runtime and transport injection', () => {
        const runtime = new StubRuntime();
        expect(
            () =>
                new AgentClient({
                    runtime: runtime as unknown as AgentRuntime,
                    transport: new FixtureTransport(),
                }),
        ).toThrowError(/invalid_arguments/);
        expect(
            () =>
                new AgentClient({
                    runtime: runtime as unknown as AgentRuntime,
                    store: createAgentStore(),
                }),
        ).toThrowError(/invalid_arguments/);
    });

    test('delegates open, start, and dispose to the injected runtime', async () => {
        const runtime = new StubRuntime();
        const client = new AgentClient({
            runtime: runtime as unknown as AgentRuntime,
        });
        expect(client.store).toBe(runtime.store);
        await client.open();
        expect(runtime.opened).toBe(1);
        const summary = await client.startRun({
            taskId: 'tsk_0000000000000001',
            prompt: 'runtime delegated',
        });
        expect(runtime.startRunInputs).toEqual([
            { taskId: 'tsk_0000000000000001', prompt: 'runtime delegated' },
        ]);
        expect(summary.runId).toBe('run_0000000000000001');
        const disposal = client.dispose();
        expect(disposal).toBeInstanceOf(Promise);
        await disposal;
        expect(runtime.disposed).toBe(1);
    });

    test('direct ingest rejection preserves storage and a healthy event clears the error', async () => {
        const transport = new EmittingTransport();
        const store = createAgentStore();
        const { stub, durable } = taskStoreFor(store);
        const client = new AgentClient({ transport, store, taskStore: durable });
        await client.open();
        const event = createRunQueuedEvent({
            projectId: 'prj_4b1d9e0c7a2f6e51',
            taskId: 'tsk_9a3c5e7b1d2f4a60',
            runId: 'run_7f3a2c1b9d4e6a80',
            generation: 1,
            recordedAtMs: 1_794_283_200_000,
        });
        const wire = canonicalJson(event);

        stub.rejectNextAppend = true;
        const rejected = await client.ingestWireEvent(wire);
        expect(rejected.status).toBe('rejected');
        expect(rejected.storage).toBe('indexeddb');
        expect(client.lastPersistenceError).toMatchObject({
            code: 'limit_exceeded',
            storage: 'indexeddb',
        });

        const applied = await client.ingestWireEvent(wire);
        expect(applied.status).toBe('applied');
        expect(client.lastPersistenceError).toBeNull();
    });
});
