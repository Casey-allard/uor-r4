import { describe, expect, test } from 'vitest';

import { AgentClient } from './client.js';
import {
    AGENT_EVENT_SCHEMA,
    type AgentEvent,
} from './generated/agentContract.js';
import type { AgentTaskStore } from './store.js';
import { AgentStoreError } from './storeTypes.js';
import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
    type RunView,
} from '../state/agent.js';
import {
    GatedTestModelPort,
    createGatedTestToolExecutor,
} from './testApi.js';
import type { IngestOutcome } from './ingest.js';
import {
    AgentRuntime,
    type AgentRuntimeOptions,
    type RuntimeTimers,
    type RuntimeToolExecutor,
} from './runtime.js';
import { studioTestApiEnabled } from '../bootstrap.js';

const PROJECT_ID = 'prj_0000000000000001';

interface Harness {
    store: AgentStore;
    model: GatedTestModelPort;
    runtime: AgentRuntime;
    client: AgentClient;
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

function makeHarness(
    options: {
        taskStore?: AgentTaskStore | null;
        timers?: RuntimeTimers;
        toolExecutor?: RuntimeToolExecutor;
        requiresApproval?: AgentRuntimeOptions['requiresApproval'];
        approvalTtlMs?: number;
    } = {},
): Harness {
    const store = createAgentStore();
    const model = new GatedTestModelPort();
    const runtime = new AgentRuntime({
        store,
        taskStore: options.taskStore ?? null,
        modelPort: model,
        toolExecutor: options.toolExecutor ?? createGatedTestToolExecutor(),
        now: () => 1_794_283_200_000,
        ids: deterministicIds(),
        writerId: 'writer-acceptance-test',
        approvalTtlMs: options.approvalTtlMs ?? 600_000,
        stopAckTimeoutMs: 30_000,
        boundaryWaitMs: 30_000,
        ...(options.timers === undefined ? {} : { timers: options.timers }),
        ...(options.requiresApproval === undefined
            ? {}
            : { requiresApproval: options.requiresApproval }),
    });
    const client = new AgentClient({ runtime });
    return { store, model, runtime, client };
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

async function until(
    predicate: () => boolean,
    label: string,
    rounds = 4_000,
): Promise<void> {
    for (let index = 0; index < rounds; index += 1) {
        if (predicate()) return;
        await Promise.resolve();
        await yieldTask();
    }
    throw new Error(`timed out waiting for ${label}`);
}

async function settle(rounds = 50): Promise<void> {
    for (let index = 0; index < rounds; index += 1) {
        await Promise.resolve();
        await yieldTask();
    }
}

function taskFor(store: AgentStore, taskId: string) {
    const task = store.snapshot().tasks.find(
        (candidate) => candidate.taskId === taskId,
    );
    if (task === undefined) throw new Error(`task ${taskId} is missing`);
    return task;
}

function runView(store: AgentStore, taskId: string, runId: string): RunView {
    const run = store.getRun(taskId, runId);
    if (run === null) throw new Error(`run ${runId} is missing`);
    return run;
}

function eventTypes(run: RunView): string[] {
    return run.events.map((event) => event.type);
}

function eventFrame(event: AgentIncomingEvent): Uint8Array {
    return new TextEncoder().encode(`${JSON.stringify(event)}\n`);
}

function outcomeView(outcome: IngestOutcome): {
    kind: string;
    code: string | null;
    requiresResync: boolean;
} {
    return {
        kind: outcome.kind,
        code: outcome.code(),
        requiresResync: outcome.requiresResync,
    };
}

function incomingEvent(
    taskId: string,
    projectId: string,
    runId: string,
    generation: number,
    sequence: number,
    type: string,
    data: Record<string, unknown>,
): AgentIncomingEvent {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: `evt_${runId}_${generation}_${String(sequence).padStart(12, '0')}`,
        project_id: projectId,
        task_id: taskId,
        run_id: runId,
        generation,
        sequence,
        recorded_at_ms: 1_794_283_200_000 + sequence,
        type,
        data,
    } as unknown as AgentIncomingEvent;
}

async function startTwo(
    harness: Harness,
): Promise<{
    a: { taskId: string; runId: string; projectId: string };
    b: { taskId: string; runId: string; projectId: string };
}> {
    const taskA = await harness.client.createTask({ title: 'Acceptance task A' });
    const taskB = await harness.client.createTask({ title: 'Acceptance task B' });
    const runA = await harness.client.startRun({
        taskId: taskA.taskId,
        prompt: 'acceptance prompt A',
    });
    const runB = await harness.client.startRun({
        taskId: taskB.taskId,
        prompt: 'acceptance prompt B',
    });
    await until(
        () =>
            harness.model.latest(runA.runId) !== null &&
            harness.model.latest(runB.runId) !== null,
        'two live model streams',
    );
    return {
        a: { taskId: taskA.taskId, runId: runA.runId, projectId: taskA.projectId },
        b: { taskId: taskB.taskId, runId: runB.runId, projectId: taskB.projectId },
    };
}

describe('Task 15 browser runtime acceptance', () => {
    test('two_live_tasks_run_concurrently', async () => {
        const harness = makeHarness({
            requiresApproval: (tool, args) =>
                tool === 'write_file' &&
                typeof args.path === 'string' &&
                args.path.startsWith('approvals/'),
        });
        await harness.client.open();
        try {
            const { a, b } = await startTwo(harness);

            const runA = runView(harness.store, a.taskId, a.runId);
            const runB = runView(harness.store, b.taskId, b.runId);
            expect(runA.events[0]).toMatchObject({
                type: 'run.queued',
                sequence: 0,
            });
            expect(runA.events[1]).toMatchObject({
                type: 'run.started',
                sequence: 1,
            });
            expect(runB.events[0]).toMatchObject({
                type: 'run.queued',
                sequence: 0,
            });
            expect(runB.events[1]).toMatchObject({
                type: 'run.started',
                sequence: 1,
            });
            expect(runA.runId).not.toBe(runB.runId);
            expect(runA.events[0]!.eventId).not.toBe(runB.events[0]!.eventId);

            harness.model.latest(a.runId)!.push({
                type: 'text-delta',
                text: 'alpha only',
            });
            await until(
                () =>
                    taskFor(harness.store, a.taskId).runs
                        .find((run) => run.runId === a.runId)!
                        .messages.some((message) => message.text === 'alpha only'),
                'task A text',
            );
            const bBefore = runView(harness.store, b.taskId, b.runId);
            expect(bBefore.messages).toEqual([]);
            expect(bBefore.lastSequence).toBe(runB.lastSequence);

            harness.model.latest(b.runId)!.push({
                type: 'text-delta',
                text: 'beta only',
            });
            await until(
                () =>
                    taskFor(harness.store, b.taskId).runs
                        .find((run) => run.runId === b.runId)!
                        .messages.some((message) => message.text === 'beta only'),
                'task B text',
            );
            expect(
                runView(harness.store, a.taskId, a.runId).messages.map(
                    (message) => message.text,
                ),
            ).toEqual(['alpha only']);
            expect(
                runView(harness.store, b.taskId, b.runId).messages.map(
                    (message) => message.text,
                ),
            ).toEqual(['beta only']);

            const turnA = harness.model.latest(a.runId);
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_accept_a',
                tool: 'write_file',
                arguments: { path: 'artifacts/accept-a.txt' },
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).changesSnapshots
                        .length === 1,
                'task A artifact',
            );
            const aChanges = runView(
                harness.store,
                a.taskId,
                a.runId,
            ).changesSnapshots;
            const bChanges = runView(
                harness.store,
                b.taskId,
                b.runId,
            ).changesSnapshots;
            expect(aChanges).toHaveLength(1);
            expect(aChanges[0]!.files).toEqual([
                { path: 'artifacts/accept-a.txt', change: 'added' },
            ]);
            expect(bChanges).toEqual([]);
            expect(aChanges[0]!.headDigest).not.toBe('');
            expect(
                runView(harness.store, a.taskId, a.runId).lastDigest,
            ).not.toBe(runView(harness.store, b.taskId, b.runId).lastDigest);
            await until(
                () => harness.model.latest(a.runId) !== turnA,
                'post-artifact model turn',
            );

            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_approval_a',
                tool: 'write_file',
                arguments: { path: 'approvals/a.txt' },
            });
            harness.model.latest(b.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_approval_b',
                tool: 'write_file',
                arguments: { path: 'approvals/b.txt' },
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).approvals.length ===
                        1 &&
                    runView(harness.store, b.taskId, b.runId).approvals.length ===
                        1,
                'two independent approvals',
            );
            const approvalA = runView(
                harness.store,
                a.taskId,
                a.runId,
            ).approvals[0]!;
            const approvalB = runView(
                harness.store,
                b.taskId,
                b.runId,
            ).approvals[0]!;
            expect(approvalA.approvalId).not.toBe(approvalB.approvalId);
            expect(approvalA.toolCallId).toBe('tool_approval_a');
            expect(approvalB.toolCallId).toBe('tool_approval_b');
            expect(approvalA.decision).toBeNull();
            expect(approvalB.decision).toBeNull();

            await harness.client.resolveApproval({
                approvalId: approvalA.approvalId,
                decision: 'approved',
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).approvals[0]!
                        .decision === 'approved' &&
                    eventTypes(
                        runView(harness.store, a.taskId, a.runId),
                    ).filter((type) => type === 'tool.started').length === 2,
                'approval A dispatch',
            );
            const afterApprovalA = runView(harness.store, a.taskId, a.runId);
            const afterApprovalB = runView(harness.store, b.taskId, b.runId);
            expect(
                eventTypes(afterApprovalA).filter(
                    (type) => type === 'tool.started',
                ),
            ).toHaveLength(2);
            expect(afterApprovalA.approvals[0]!.decision).toBe('approved');
            expect(afterApprovalB.approvals[0]!.decision).toBeNull();
            expect(
                eventTypes(afterApprovalB).filter(
                    (type) => type === 'tool.started',
                ),
            ).toHaveLength(0);

            await harness.client.resolveApproval({
                approvalId: approvalB.approvalId,
                decision: 'approved',
            });
            await until(
                () =>
                    runView(harness.store, b.taskId, b.runId).approvals[0]!
                        .decision === 'approved' &&
                    eventTypes(
                        runView(harness.store, b.taskId, b.runId),
                    ).filter((type) => type === 'tool.started').length === 1,
                'approval B dispatch',
            );

            await harness.client.stop({ taskId: a.taskId, runId: a.runId });
            await harness.client.stop({ taskId: b.taskId, runId: b.runId });
            expect(runView(harness.store, a.taskId, a.runId).state).toBe(
                'cancelled',
            );
            expect(runView(harness.store, b.taskId, b.runId).state).toBe(
                'cancelled',
            );
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('stop_at_safe_boundary_blocks_later_tools', async () => {
        const toolGateControl: { release: (() => void) | null } = {
            release: null,
        };
        const toolGate = new Promise<void>((resolve) => {
            toolGateControl.release = resolve;
        });
        let toolCalls = 0;
        const harness = makeHarness({
            toolExecutor: async (intent) => {
                toolCalls += 1;
                await toolGate;
                return {
                    status: 'ok' as const,
                    output: { path: intent.arguments.path },
                    files: [
                        {
                            path: String(intent.arguments.path),
                            change: 'added' as const,
                        },
                    ],
                    head_digest: `sha256:${'a'.repeat(64)}`,
                };
            },
        });
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_stop_boundary',
                tool: 'write_file',
                arguments: { path: 'artifacts/stop-boundary.txt' },
            });
            await until(
                () =>
                    eventTypes(runView(harness.store, a.taskId, a.runId)).includes(
                        'tool.started',
                    ),
                'in-flight tool',
            );

            const stop = harness.client.stop({
                taskId: a.taskId,
                runId: a.runId,
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).events.some(
                        (event) =>
                            event.type === 'run.status' &&
                            event.data.status === 'cancelling',
                    ),
                'cancelling status',
            );
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_after_stop',
                tool: 'write_file',
                arguments: { path: 'artifacts/after-stop.txt' },
            });
            toolGateControl.release!();
            const receipt = await stop;
            expect(receipt.acknowledged).toBe(true);

            await settle();
            const view = runView(harness.store, a.taskId, a.runId);
            expect(view.state).toBe('cancelled');
            expect(eventTypes(view).at(-1)).toBe('run.cancelled');
            expect(
                eventTypes(view).filter((type) => type === 'tool.started'),
            ).toHaveLength(1);
            expect(toolCalls).toBe(1);
            expect(
                view.events.some(
                    (event) =>
                        event.type === 'tool.requested' &&
                        event.data.tool_call_id === 'tool_after_stop',
                ),
            ).toBe(false);
            expect(view.messages).toEqual([]);
            expect(view.changesSnapshots).toHaveLength(1);
            expect(view.changesSnapshots[0]!.files).toEqual([
                { path: 'artifacts/stop-boundary.txt', change: 'added' },
            ]);
        } finally {
            toolGateControl.release?.();
            await harness.runtime.dispose();
        }
    });

    test('forged_approval_is_refused', async () => {
        const harness = makeHarness({
            requiresApproval: (tool) => tool === 'write_file',
            toolExecutor: async (intent) => ({
                status: 'ok' as const,
                output: { path: intent.arguments.path },
                files: [
                    {
                        path: String(intent.arguments.path),
                        change: 'added' as const,
                    },
                ],
                head_digest: `sha256:${'b'.repeat(64)}`,
            }),
        });
        await harness.client.open();
        try {
            const { a, b } = await startTwo(harness);
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_forged_approval',
                tool: 'write_file',
                arguments: { path: 'artifacts/forged.txt' },
            });
            await until(
                () =>
                    eventTypes(runView(harness.store, a.taskId, a.runId)).includes(
                        'tool.approval_required',
                    ),
                'approval request',
            );
            const run = runView(harness.store, a.taskId, a.runId);
            const required = run.events.find(
                (event) => event.type === 'tool.approval_required',
            )!;
            const approvalId = String(required.data.approval_id);

            await expect(
                harness.client.resolveApproval({
                    approvalId: 'appr_forged_not_pending',
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });

            await expect(
                harness.runtime.resolveHostApproval({
                    approvalId,
                    decision: 'approved',
                    taskId: b.taskId,
                    runId: b.runId,
                    generation: 1,
                    requestDigest: String(required.data.request_digest),
                }),
            ).rejects.toMatchObject({ code: 'approval_forbidden' });
            const refused = runView(harness.store, a.taskId, a.runId);
            expect(
                eventTypes(refused).filter((type) => type === 'tool.started'),
            ).toHaveLength(0);
            expect(
                eventTypes(refused).filter((type) => type === 'tool.completed'),
            ).toHaveLength(0);
            expect(refused.changesSnapshots).toEqual([]);
            expect(refused.approvals[0]?.decision).toBeNull();

            await harness.client.resolveApproval({
                approvalId,
                decision: 'approved',
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).changesSnapshots
                        .length === 1,
                'real approval dispatch',
            );
            const afterReal = runView(harness.store, a.taskId, a.runId);
            expect(
                eventTypes(afterReal).filter((type) => type === 'tool.started'),
            ).toHaveLength(1);
            expect(afterReal.changesSnapshots).toHaveLength(1);
            expect(afterReal.changesSnapshots[0]!.files).toEqual([
                { path: 'artifacts/forged.txt', change: 'added' },
            ]);
            await harness.client.stop({ taskId: a.taskId, runId: a.runId });
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('host_channel_approval_resolution_is_display_only_and_never_dispatches', async () => {
        const harness = makeHarness({
            requiresApproval: (tool) => tool === 'write_file',
            toolExecutor: async (intent) => ({
                status: 'ok' as const,
                output: { path: intent.arguments.path },
                files: [
                    {
                        path: String(intent.arguments.path),
                        change: 'added' as const,
                    },
                ],
                head_digest: `sha256:${'f'.repeat(64)}`,
            }),
        });
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_host_resolution',
                tool: 'write_file',
                arguments: { path: 'artifacts/host-resolution.txt' },
            });
            await until(
                () =>
                    eventTypes(runView(harness.store, a.taskId, a.runId)).includes(
                        'tool.approval_required',
                    ),
                'approval request',
            );
            const run = runView(harness.store, a.taskId, a.runId);
            const required = run.events.find(
                (event) => event.type === 'tool.approval_required',
            )!;
            const approvalId = String(required.data.approval_id);
            const worker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const resolved = await worker.push(
                eventFrame(
                    incomingEvent(
                        a.taskId,
                        a.projectId,
                        a.runId,
                        run.generation,
                        run.lastSequence + 1,
                        'tool.approval_resolved',
                        {
                            approval_id: approvalId,
                            decision: 'approved',
                            resolver: 'host',
                        },
                    ),
                ),
            );
            expect(resolved[0]?.kind).toBe('applied');
            await settle();
            const afterHost = runView(harness.store, a.taskId, a.runId);
            // A host-channel resolution is trusted journal/display state. It
            // cannot dispatch: the runtime's in-memory pending approval is the
            // only dispatch gate, and only a trusted resolution settles it.
            expect(afterHost.approvals[0]?.decision).toBe('approved');
            expect(
                eventTypes(afterHost).filter((type) => type === 'tool.started'),
            ).toHaveLength(0);
            expect(
                eventTypes(afterHost).filter(
                    (type) => type === 'tool.completed',
                ),
            ).toHaveLength(0);
            expect(afterHost.changesSnapshots).toEqual([]);

            await harness.client.resolveApproval({
                approvalId,
                decision: 'approved',
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).changesSnapshots
                        .length === 1,
                'trusted host resolution dispatch',
            );
            const afterTrusted = runView(harness.store, a.taskId, a.runId);
            expect(
                eventTypes(afterTrusted).filter(
                    (type) => type === 'tool.started',
                ),
            ).toHaveLength(1);
            expect(afterTrusted.changesSnapshots).toHaveLength(1);
            await harness.client.stop({ taskId: a.taskId, runId: a.runId });
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('malformed_partial_duplicate_out_of_order_never_become_output', async () => {
        const harness = makeHarness();
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            const run = runView(harness.store, a.taskId, a.runId);
            const malformedWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const malformed = await malformedWorker.push(
                new TextEncoder().encode('{"schema":oops}\n'),
            );
            const malformedEof = await malformedWorker.eof();
            if (malformedEof !== null) malformed.push(malformedEof);
            expect(outcomeView(malformed.at(-1)!)).toMatchObject({
                kind: 'rejected',
                code: 'malformed_event',
            });

            const partialWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const partialEvent = incomingEvent(
                a.taskId,
                a.projectId,
                a.runId,
                run.generation,
                run.lastSequence + 1,
                'assistant.message',
                { message_id: 'msg_partial', text: 'partial hostile' },
            );
            const partialFrame = eventFrame(partialEvent);
            const partial = await partialWorker.push(
                partialFrame.subarray(0, partialFrame.length - 1),
            );
            const partialEof = await partialWorker.eof();
            if (partialEof !== null) partial.push(partialEof);
            expect(outcomeView(partial.at(-1)!)).toMatchObject({
                kind: 'rejected',
                code: 'truncated_frame',
            });

            const duplicateEvent = incomingEvent(
                a.taskId,
                a.projectId,
                a.runId,
                run.generation,
                run.lastSequence + 1,
                'assistant.delta',
                { message_id: 'msg_dup', text_delta: 'duplicate hostile' },
            );
            const duplicateWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            expect(
                outcomeView(
                    (await duplicateWorker.push(eventFrame(duplicateEvent)))[0]!,
                ),
            ).toMatchObject({ kind: 'applied' });
            const duplicateAgain = await duplicateWorker.push(
                eventFrame(duplicateEvent),
            );
            expect(outcomeView(duplicateAgain[0]!)).toMatchObject({ kind: 'duplicate' });

            const gapWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const gapEvent = incomingEvent(
                a.taskId,
                a.projectId,
                a.runId,
                runView(harness.store, a.taskId, a.runId).generation,
                runView(harness.store, a.taskId, a.runId).lastSequence + 5,
                'assistant.message',
                { message_id: 'msg_gap', text: 'gap hostile' },
            );
            const gap = await gapWorker.push(eventFrame(gapEvent));
            expect(outcomeView(gap[0]!)).toMatchObject({
                kind: 'rejected',
                code: 'out_of_order',
                requiresResync: true,
            });

            const view = runView(harness.store, a.taskId, a.runId);
            expect(view.messages.map((message) => message.text)).toEqual([
                'duplicate hostile',
            ]);
            expect(
                eventTypes(view).filter((type) => type === 'tool.completed'),
            ).toHaveLength(0);
            expect(eventTypes(view)).not.toContain('run.finished');
            expect(eventTypes(view)).not.toContain('run.failed');
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('unknown_thinking_and_heartbeat_tolerated', async () => {
        const harness = makeHarness();
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            const run = runView(harness.store, a.taskId, a.runId);
            const worker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const thinking = await worker.push(
                eventFrame(
                    incomingEvent(
                        a.taskId,
                        a.projectId,
                        a.runId,
                        run.generation,
                        run.lastSequence + 1,
                        'thinking.delta',
                        { text_delta: 'tolerated private thought' },
                    ),
                ),
            );
            expect(outcomeView(thinking[0]!)).toMatchObject({ kind: 'applied' });
            const heartbeat = await worker.push(
                eventFrame(
                    incomingEvent(
                        a.taskId,
                        a.projectId,
                        a.runId,
                        run.generation,
                        run.lastSequence + 2,
                        'heartbeat',
                        { nonce: 'heartbeat-tolerated' },
                    ),
                ),
            );
            expect(outcomeView(heartbeat[0]!)).toMatchObject({ kind: 'applied' });
            const unknown = await worker.push(
                eventFrame(
                    incomingEvent(
                        a.taskId,
                        a.projectId,
                        a.runId,
                        run.generation,
                        run.lastSequence + 3,
                        'host.thinking.future',
                        { text_delta: 'must never render' },
                    ),
                ),
            );
            expect(outcomeView(unknown[0]!)).toMatchObject({ kind: 'unknown' });
            await settle();
            const view = runView(harness.store, a.taskId, a.runId);
            expect(view.messages).toEqual([]);
            expect(view.unknownEvents).toHaveLength(1);
            expect(eventTypes(view)).not.toContain('assistant.message');
            expect(eventTypes(view)).not.toContain('assistant.delta');
        } finally {
            await harness.runtime.dispose();
        }
    });
});

describe('Task 15 approval authority', () => {
    test('expired_approval_never_dispatches_a_write', async () => {
        const harness = makeHarness({
            approvalTtlMs: -1,
            requiresApproval: (tool) => tool === 'write_file',
            toolExecutor: async (intent) => ({
                status: 'ok' as const,
                output: { path: intent.arguments.path },
                files: [
                    {
                        path: String(intent.arguments.path),
                        change: 'added' as const,
                    },
                ],
                head_digest: `sha256:${'e'.repeat(64)}`,
            }),
        });
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_expired_approval',
                tool: 'write_file',
                arguments: { path: 'artifacts/expired.txt' },
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).approvals.length ===
                    1,
                'expired approval request',
            );
            const approval = runView(
                harness.store,
                a.taskId,
                a.runId,
            ).approvals[0]!;
            await expect(
                harness.client.resolveApproval({
                    approvalId: approval.approvalId,
                    decision: 'approved',
                }),
            ).rejects.toMatchObject({ code: 'approval_expired' });
            await settle();
            const view = runView(harness.store, a.taskId, a.runId);
            expect(view.approvals[0]?.decision).toBe('expired');
            expect(
                eventTypes(view).filter((type) => type === 'tool.started'),
            ).toHaveLength(0);
            expect(
                eventTypes(view).filter((type) => type === 'tool.completed'),
            ).toHaveLength(0);
            expect(view.changesSnapshots).toEqual([]);
            expect(view.state).toBe('failed');
            expect(view.error?.code).toBe('approval_expired');
        } finally {
            await harness.runtime.dispose();
        }
    });
});

describe('Task 15 stop failure and dispose receipt settlement', () => {
    test('rejected_no_inflight_cancelled_append_settles_false_stop_exactly_once', async () => {
        const harness = makeHarness();
        const originalAppend = harness.store.appendEvent.bind(harness.store);
        let failCancelled = true;
        harness.store.appendEvent = async (event) => {
            if (failCancelled && event.type === 'run.cancelled') {
                failCancelled = false;
                throw new AgentStoreError(
                    'limit_exceeded',
                    'injected no-in-flight cancellation failure',
                    'indexeddb',
                );
            }
            return originalAppend(event);
        };
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            const before = runView(harness.store, a.taskId, a.runId);
            const receipt = await harness.client.stop({
                taskId: a.taskId,
                runId: a.runId,
            });
            expect(receipt.acknowledged).toBe(false);

            const after = runView(harness.store, a.taskId, a.runId);
            expect(after.lastSequence).toBe(before.lastSequence + 1);
            expect(after.events.at(-1)?.type).toBe('run.status');
            expect(after.events.at(-1)?.data.status).toBe('cancelling');
            expect(eventTypes(after)).not.toContain('run.cancelled');
            expect(eventTypes(after)).not.toContain('run.finished');
            expect(eventTypes(after)).not.toContain('run.failed');
            const state = harness.runtime as unknown as {
                active: Map<string, unknown>;
                taskQueues: Map<string, unknown>;
                persistenceError: AgentStoreError | null;
            };
            expect(state.active.has(a.taskId)).toBe(false);
            expect(state.taskQueues.has(a.taskId)).toBe(false);
            expect(state.persistenceError?.code).toBe('limit_exceeded');
            expect(state.persistenceError?.storage).toBe('indexeddb');

            await expect(
                harness.client.stop({ taskId: a.taskId, runId: a.runId }),
            ).rejects.toMatchObject({ code: 'runtime_unavailable' });

            harness.model.latest(a.runId)?.push({
                type: 'text-delta',
                text: 'must never reuse the advanced sequence',
            });
            await settle();
            expect(
                runView(harness.store, a.taskId, a.runId).lastSequence,
            ).toBe(after.lastSequence);
            expect(
                runView(harness.store, a.taskId, a.runId).messages,
            ).toEqual([]);
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('dispose_fails_accepted_pending_and_queued_messages_before_cleanup', async () => {
        let timerThrows = 1;
        const timers: RuntimeTimers = {
            setTimeout: (handler, timeoutMs) =>
                globalThis.setTimeout(handler, timeoutMs),
            clearTimeout: (handle) => {
                if (timerThrows > 0) {
                    timerThrows -= 1;
                    throw new AgentStoreError(
                        'limit_exceeded',
                        'injected dispose timer clear failure',
                        'indexeddb',
                    );
                }
                globalThis.clearTimeout(handle as never);
            },
            setInterval: (handler, timeoutMs) =>
                globalThis.setInterval(handler, timeoutMs),
            clearInterval: (handle) => globalThis.clearInterval(handle as never),
        };
        const harness = makeHarness({ timers });
        await harness.client.open();
        const { a } = await startTwo(harness);
        const queued = await harness.client.queueMessage({
            taskId: a.taskId,
            text: 'queued message accepted before dispose',
        });
        const steered = await harness.client.steer({
            taskId: a.taskId,
            text: 'pending steer accepted before dispose',
        });
        const stream = await harness.runtime.subscribe({});
        (stream as unknown as { close: () => void }).close = () => {
            throw new AgentStoreError(
                'limit_exceeded',
                'injected dispose stream close failure',
                'indexeddb',
            );
        };
        await expect(harness.runtime.dispose()).resolves.toBeUndefined();

        const view = runView(harness.store, a.taskId, a.runId);
        const errors = view.events.filter(
            (event) =>
                event.type === 'error' &&
                event.data.code === 'message_not_delivered',
        );
        expect(errors).toHaveLength(2);
        const messages = errors.map((event) => String(event.data.message));
        for (const messageId of [queued.messageId, steered.messageId]) {
            expect(
                messages.filter((message) => message.includes(messageId)),
            ).toHaveLength(1);
        }
        expect(
            messages.filter((message) =>
                message.includes('queued message accepted before dispose'),
            ),
        ).toHaveLength(0);
        expect(eventTypes(view)).not.toContain('run.cancelled');
        expect(eventTypes(view)).not.toContain('run.finished');
        expect(eventTypes(view)).not.toContain('run.failed');
        expect(view.messages).toEqual([]);
        expect(view.changesSnapshots).toEqual([]);
        const state = harness.runtime as unknown as {
            active: Map<string, unknown>;
            listeners: Set<unknown>;
            eventStreams: Set<unknown>;
            taskQueues: Map<string, unknown>;
            opened: boolean;
        };
        expect(state.active.size).toBe(0);
        expect(state.listeners.size).toBe(0);
        expect(state.eventStreams.size).toBe(0);
        expect(state.taskQueues.size).toBe(0);
        expect(state.opened).toBe(false);
    });
});

describe('Task 15 assigned lifecycle regressions', () => {
    test('no_late_tool_event_after_dispose', async () => {
        const toolGateControl: { release: (() => void) | null } = {
            release: null,
        };
        const toolGate = new Promise<void>((resolve) => {
            toolGateControl.release = resolve;
        });
        const harness = makeHarness({
            toolExecutor: async (intent) => {
                await toolGate;
                return {
                    status: 'ok' as const,
                    output: { path: intent.arguments.path },
                    files: [
                        {
                            path: String(intent.arguments.path),
                            change: 'added' as const,
                        },
                    ],
                    head_digest: `sha256:${'c'.repeat(64)}`,
                };
            },
        });
        await harness.client.open();
        const { a } = await startTwo(harness);
        harness.model.latest(a.runId)!.push({
            type: 'tool-call',
            toolCallId: 'tool_dispose_late',
            tool: 'write_file',
            arguments: { path: 'artifacts/late.txt' },
        });
        await until(
            () =>
                eventTypes(runView(harness.store, a.taskId, a.runId)).includes(
                    'tool.started',
                ),
            'tool started before dispose',
        );
        await harness.runtime.dispose();
        toolGateControl.release!();
        await settle();
        const view = runView(harness.store, a.taskId, a.runId);
        expect(eventTypes(view)).not.toContain('tool.completed');
        expect(eventTypes(view)).not.toContain('tool.failed');
        expect(view.changesSnapshots).toEqual([]);
        expect(view.state).toBe('running');
    });

    test('dispose_survives_throwing_timer_clear_and_resets_listeners', async () => {
        let clearTimeoutThrows = 0;
        const timers: RuntimeTimers = {
            setTimeout: (handler, timeoutMs) =>
                globalThis.setTimeout(handler, timeoutMs),
            clearTimeout: (handle) => {
                if (clearTimeoutThrows > 0) {
                    clearTimeoutThrows -= 1;
                    throw new AgentStoreError(
                        'limit_exceeded',
                        'injected timer clear failure',
                        'indexeddb',
                    );
                }
                globalThis.clearTimeout(handle as never);
            },
            setInterval: (handler, timeoutMs) =>
                globalThis.setInterval(handler, timeoutMs),
            clearInterval: (handle) => globalThis.clearInterval(handle as never),
        };
        const harness = makeHarness({
            timers,
            toolExecutor: async () => {
                await new Promise<void>(() => {
                    // Never settles: the stop timer is what dispose must clear.
                });
                return { status: 'ok' as const, output: {} };
            },
        });
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_timer_fault',
                tool: 'write_file',
                arguments: { path: 'artifacts/timer.txt' },
            });
            await until(
                () =>
                    eventTypes(runView(harness.store, a.taskId, a.runId)).includes(
                        'tool.started',
                    ),
                'tool started for timer fault',
            );
            const stop = harness.client.stop({
                taskId: a.taskId,
                runId: a.runId,
            });
            await settle(10);
            clearTimeoutThrows = 1;
            await expect(harness.runtime.dispose()).resolves.toBeUndefined();
            await expect(stop).resolves.toMatchObject({ acknowledged: false });
            const state = harness.runtime as unknown as {
                active: Map<string, unknown>;
                listeners: Set<unknown>;
                eventStreams: Set<unknown>;
                taskQueues: Map<string, unknown>;
                opened: boolean;
            };
            expect(state.active.size).toBe(0);
            expect(state.listeners.size).toBe(0);
            expect(state.eventStreams.size).toBe(0);
            expect(state.taskQueues.size).toBe(0);
            expect(state.opened).toBe(false);
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('dispose_continues_when_event_stream_close_throws', async () => {
        const harness = makeHarness();
        await harness.client.open();
        try {
            const listener = () => undefined;
            const unsubscribe = harness.runtime.subscribe(listener);
            const stream = await harness.runtime.subscribe({});
            const throwingClose = stream as unknown as { close: () => void };
            throwingClose.close = () => {
                throw new AgentStoreError(
                    'limit_exceeded',
                    'injected stream close failure',
                );
            };
            await expect(harness.runtime.dispose()).resolves.toBeUndefined();
            const state = harness.runtime as unknown as {
                listeners: Set<unknown>;
                eventStreams: Set<unknown>;
                opened: boolean;
            };
            expect(state.listeners.size).toBe(0);
            expect(state.eventStreams.size).toBe(0);
            expect(state.opened).toBe(false);
            unsubscribe();
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('runtime_dispose_retries_a_failed_early_host_close', async () => {
        let closes = 0;
        const fakeHost = {
            close: () => {
                closes += 1;
                if (closes === 1) {
                    throw new AgentStoreError(
                        'host_unavailable',
                        'injected early close failure',
                    );
                }
            },
            capabilities: async () => ({
                tree: false,
                changes: false,
                readArtifact: false,
                terminal: false,
                git: false,
                mcp: false,
                approvals: false,
            }),
        };
        const store = createAgentStore();
        const runtime = new AgentRuntime({
            store,
            taskStore: null,
            modelPort: new GatedTestModelPort(),
            hostClient: fakeHost as never,
            now: () => 1_794_283_200_000,
            writerId: 'writer-close-fault',
        });
        await runtime.open();
        await expect(runtime.dispose()).resolves.toBeUndefined();
        expect(closes).toBe(2);
        const state = runtime as unknown as { opened: boolean; disposed: boolean };
        expect(state.opened).toBe(false);
        expect(state.disposed).toBe(true);
    });

    test('run_expired_fails_pending_messages_before_terminal_and_starts_no_new_run', async () => {
        const harness = makeHarness();
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            await harness.client.queueMessage({
                taskId: a.taskId,
                text: 'queued before expiry',
            });
            const run = runView(harness.store, a.taskId, a.runId);
            const worker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const outcomes = await worker.push(
                eventFrame(
                    incomingEvent(
                        a.taskId,
                        a.projectId,
                        a.runId,
                        run.generation,
                        run.lastSequence + 1,
                        'run.expired',
                        {
                            expired_at_ms: 1_794_283_260_000,
                            reason: 'host lease expired',
                        },
                    ),
                ),
            );
            expect(outcomeView(outcomes[0]!)).toMatchObject({ kind: 'applied' });
            await settle();
            const view = runView(harness.store, a.taskId, a.runId);
            const types = eventTypes(view);
            const terminalIndex = types.lastIndexOf('run.expired');
            expect(types.at(-1)).toBe('run.expired');
            expect(view.state).toBe('expired');
            const errors = view.events
                .map((event, index) => ({ event, index }))
                .filter((entry) => entry.event.type === 'error');
            expect(errors).toHaveLength(1);
            expect(errors[0]!.index).toBeLessThan(terminalIndex);
            expect(errors[0]!.event.data.code).toBe('message_not_delivered');
            expect(taskFor(harness.store, a.taskId).latestRunId).toBe(a.runId);
            expect(eventTypes(view)).not.toContain('run.finished');
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('old_worker_push_after_cursor_expiry_resume_is_rejected', async () => {
        const harness = makeHarness();
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            const originalGeneration = runView(
                harness.store,
                a.taskId,
                a.runId,
            ).generation;
            const worker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const expired = await worker.cursorExpired();
            expect(expired.code()).toBe('cursor_expired');
            const resumed = harness.store.getRunCursor(a.taskId, a.runId)!;
            expect(resumed.generation).toBe(2);
            const oldRun = runView(harness.store, a.taskId, a.runId);
            const oldFrame = eventFrame(
                incomingEvent(
                    a.taskId,
                    a.projectId,
                    a.runId,
                    originalGeneration,
                    resumed.lastSequence + 1,
                    'assistant.message',
                    {
                        message_id: 'msg_old_worker',
                        text: 'old worker hostile output',
                    },
                ),
            );
            const lateOldWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const oldPush = await lateOldWorker.push(oldFrame);
            expect(outcomeView(oldPush[0]!)).toMatchObject({
                kind: 'rejected',
                code: 'stale_generation',
                requiresResync: true,
            });
            const view = runView(harness.store, a.taskId, a.runId);
            expect(view.messages).toEqual([]);
            expect(eventTypes(view)).not.toContain('assistant.message');
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('ingested_terminal_dedup_and_different_generation_do_not_fabricate_completion', async () => {
        const harness = makeHarness();
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            const run = runView(harness.store, a.taskId, a.runId);
            const terminal = incomingEvent(
                a.taskId,
                a.projectId,
                a.runId,
                run.generation,
                run.lastSequence + 1,
                'run.finished',
                {
                    status: 'finished',
                    finished_at_ms: 1_794_283_240_000,
                    duration_ms: 40,
                },
            );
            const firstWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const first = await firstWorker.push(eventFrame(terminal));
            expect(outcomeView(first[0]!)).toMatchObject({ kind: 'applied' });
            await settle();
            const finished = runView(harness.store, a.taskId, a.runId);
            expect(finished.state).toBe('finished');
            expect(eventTypes(finished).at(-1)).toBe('run.finished');
            const appendedTerminal = finished.events.at(-1)!;
            expect(appendedTerminal.eventId).toBe(
                terminal.event_id,
            );
            expect(
                eventTypes(finished).filter((type) => type === 'run.finished'),
            ).toHaveLength(1);

            const duplicateWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const duplicate = await duplicateWorker.push(eventFrame(terminal));
            expect(outcomeView(duplicate[0]!)).toMatchObject({
                kind: 'rejected',
                code: 'duplicate_event',
            });
            const exactWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const exactDuplicate = await exactWorker.push(
                eventFrame({
                    schema: AGENT_EVENT_SCHEMA,
                    event_id: appendedTerminal.eventId,
                    project_id: a.projectId,
                    task_id: a.taskId,
                    run_id: a.runId,
                    generation: appendedTerminal.generation,
                    sequence: appendedTerminal.sequence,
                    recorded_at_ms: appendedTerminal.recordedAtMs,
                    type: appendedTerminal.type,
                    data: appendedTerminal.data,
                } as unknown as AgentIncomingEvent),
            );
            expect(outcomeView(exactDuplicate[0]!)).toMatchObject({
                kind: 'duplicate',
            });
            expect(
                eventTypes(runView(harness.store, a.taskId, a.runId)).filter(
                    (type) => type === 'run.finished',
                ),
            ).toHaveLength(1);

            const secondRun = await harness.client.startRun({
                taskId: a.taskId,
                prompt: 'generation two after terminal identity',
            });
            expect(secondRun.generation).toBe(2);
            await until(
                () => harness.model.latest(secondRun.runId) !== null,
                'generation two stream',
            );
            const secondContainer = runView(
                harness.store,
                a.taskId,
                secondRun.runId,
            );
            const staleTerminal = incomingEvent(
                a.taskId,
                a.projectId,
                secondRun.runId,
                run.generation,
                secondContainer.lastSequence + 1,
                'run.finished',
                {
                    status: 'finished',
                    finished_at_ms: 1_794_283_250_000,
                    duration_ms: 10,
                },
            );
            const staleWorker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: secondRun.runId,
            });
            const stale = await staleWorker.push(eventFrame(staleTerminal));
            expect(outcomeView(stale[0]!)).toMatchObject({
                kind: 'rejected',
                code: 'stale_generation',
                requiresResync: true,
            });
            expect(
                runView(harness.store, a.taskId, secondRun.runId).state,
            ).toBe('running');
            expect(
                eventTypes(runView(harness.store, a.taskId, secondRun.runId)),
            ).not.toContain('run.finished');
        } finally {
            await harness.runtime.dispose();
        }
    });

    test('durable_terminal_failure_after_preterminal_fallback_settles_stop', async () => {
        const toolGateControl: { release: (() => void) | null } = {
            release: null,
        };
        const toolGate = new Promise<void>((resolve) => {
            toolGateControl.release = resolve;
        });
        let failTerminal = false;
        let failSettleClear = false;
        const timers: RuntimeTimers = {
            setTimeout: (handler, timeoutMs) =>
                globalThis.setTimeout(handler, timeoutMs),
            clearTimeout: (handle) => {
                if (failSettleClear) {
                    failSettleClear = false;
                    throw new AgentStoreError(
                        'limit_exceeded',
                        'injected settle timer failure',
                        'indexeddb',
                    );
                }
                globalThis.clearTimeout(handle as never);
            },
            setInterval: (handler, timeoutMs) =>
                globalThis.setInterval(handler, timeoutMs),
            clearInterval: (handle) => globalThis.clearInterval(handle as never),
        };
        const harness = makeHarness({
            timers,
            toolExecutor: async (intent) => {
                await toolGate;
                return {
                    status: 'ok' as const,
                    output: { path: intent.arguments.path },
                    files: [
                        {
                            path: String(intent.arguments.path),
                            change: 'added' as const,
                        },
                    ],
                    head_digest: `sha256:${'d'.repeat(64)}`,
                };
            },
        });
        const originalAppend = harness.store.appendEvent.bind(harness.store);
        harness.store.appendEvent = async (event) => {
            if (failTerminal && event.type === 'run.failed') {
                failTerminal = false;
                throw new AgentStoreError(
                    'limit_exceeded',
                    'injected durable terminal failure',
                    'indexeddb',
                );
            }
            return originalAppend(event);
        };
        await harness.client.open();
        try {
            const { a } = await startTwo(harness);
            const queued = await harness.client.queueMessage({
                taskId: a.taskId,
                text: 'must settle despite failed terminal',
            });
            harness.model.latest(a.runId)!.push({
                type: 'tool-call',
                toolCallId: 'tool_terminal_failure',
                tool: 'write_file',
                arguments: { path: 'artifacts/terminal-failure.txt' },
            });
            await until(
                () =>
                    eventTypes(runView(harness.store, a.taskId, a.runId)).includes(
                        'tool.started',
                    ),
                'in-flight tool before terminal failure',
            );
            const stop = harness.client.stop({
                taskId: a.taskId,
                runId: a.runId,
            });
            await until(
                () =>
                    runView(harness.store, a.taskId, a.runId).events.some(
                        (event) =>
                            event.type === 'run.status' &&
                            event.data.status === 'cancelling',
                    ),
                'cancelling before terminal failure',
            );
            failTerminal = true;
            failSettleClear = true;
            const run = runView(harness.store, a.taskId, a.runId);
            const worker = harness.runtime.createIngest({
                taskId: a.taskId,
                runId: a.runId,
            });
            const outcomes = await worker.push(
                eventFrame(
                    incomingEvent(
                        a.taskId,
                        a.projectId,
                        a.runId,
                        run.generation,
                        run.lastSequence + 1,
                        'run.failed',
                        {
                            code: 'host_failed',
                            message: 'terminal append will fail durably',
                            retryable: false,
                        },
                    ),
                ),
            );
            expect(outcomeView(outcomes[0]!)).toMatchObject({
                kind: 'rejected',
                code: 'limit_exceeded',
            });
            toolGateControl.release!();
            const settled = await Promise.race([
                stop.then(() => 'settled' as const),
                new Promise<'timeout'>((resolve) =>
                    globalThis.setTimeout(() => resolve('timeout'), 200),
                ),
            ]);
            expect(settled).toBe('settled');
            await expect(stop).resolves.toMatchObject({ acknowledged: false });
            const state = harness.runtime as unknown as {
                active: Map<string, unknown>;
                taskQueues: Map<string, unknown>;
            };
            expect(state.active.has(a.taskId)).toBe(false);
            expect(state.taskQueues.has(a.taskId)).toBe(false);
            expect(harness.runtime.lastPersistenceError?.code).toBe(
                'limit_exceeded',
            );
            expect(harness.runtime.lastPersistenceError?.message).toContain(
                'injected durable terminal failure',
            );
            expect(harness.runtime.lastPersistenceError?.message).not.toContain(
                'injected settle timer failure',
            );

            const view = runView(harness.store, a.taskId, a.runId);
            const types = eventTypes(view);
            const errors = view.events
                .map((event, index) => ({ event, index }))
                .filter(
                    (entry) =>
                        entry.event.type === 'error' &&
                        entry.event.data.code === 'message_not_delivered',
                );
            expect(errors).toHaveLength(1);
            expect(String(errors[0]!.event.data.message)).toContain(
                queued.messageId,
            );
            const statusIndex = types.findIndex(
                (type) => type === 'run.status',
            );
            expect(statusIndex).toBeGreaterThan(-1);
            expect(errors[0]!.index).toBeGreaterThan(statusIndex);
            expect(types.at(-1)).toBe('error');
            expect(types).not.toContain('run.failed');
            expect(types).not.toContain('run.cancelled');
            expect(
                types.filter((type) => type === 'tool.completed'),
            ).toHaveLength(0);
            expect(view.messages).toEqual([]);
            expect(view.changesSnapshots).toEqual([]);
        } finally {
            toolGateControl.release?.();
            await harness.runtime.dispose();
        }
    });
});

describe('Task 15 test API gates', () => {
    test('both_gates_required_for_test_api', () => {
        expect(studioTestApiEnabled('1', '?studioTest=1')).toBe(true);
        expect(studioTestApiEnabled('0', '?studioTest=1')).toBe(false);
        expect(studioTestApiEnabled('1', '?typed=1')).toBe(false);
        expect(studioTestApiEnabled(undefined, '?studioTest=1')).toBe(false);
    });
});
