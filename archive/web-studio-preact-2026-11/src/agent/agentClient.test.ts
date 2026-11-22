import { describe, expect, test } from 'vitest';

import {
    AgentClient,
    AgentClientError,
    type AgentIncomingEvent,
    type ChangesSnapshot,
    type ChangesSnapshotInput,
    type HostCapabilities,
    type ListTasksInput,
    type RunSummary,
    type TaskSummary,
} from './client.js';
import { FixtureTransport, createFixtureAgentClient } from './fixtureTransport.js';

const DUPLICATE_KEY_FRAME =
    '{"data":{"queued_at_ms":1,"queued_at_ms":2},"event_id":"evt_run_7f3a2c1b9d4e6a80_1_000000000000","generation":1,"project_id":"prj_4b1d9e0c7a2f6e51","recorded_at_ms":1,"run_id":"run_7f3a2c1b9d4e6a80","schema":"uor-r4.studio.agent-event/1","sequence":0,"task_id":"tsk_9a3c5e7b1d2f4a60","type":"run.queued"}';

const LOCAL_CHANGES_DIGEST = `sha256:${'a'.repeat(64)}`;
const HOST_CHANGES_DIGEST = `sha256:${'c'.repeat(64)}`;

function changesSnapshotEvent(
    task: TaskSummary,
    run: RunSummary,
    sequence: number,
    scope: 'last_turn' | 'branch',
    files: Array<Record<string, unknown>>,
): AgentIncomingEvent {
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: `evt_${run.runId}_1_${sequence.toString().padStart(12, '0')}`,
        project_id: task.projectId,
        task_id: task.taskId,
        run_id: run.runId,
        generation: 1,
        sequence,
        recorded_at_ms: 1_794_283_200_000 + sequence,
        type: 'changes.snapshot',
        data: {
            scope,
            files,
            head_digest: LOCAL_CHANGES_DIGEST,
            captured_at_ms: 1_794_283_200_000 + sequence,
        },
    } as unknown as AgentIncomingEvent;
}

function applyChanges(
    client: AgentClient,
    task: TaskSummary,
    run: RunSummary,
    sequence: number,
    scope: 'last_turn' | 'branch',
    files: Array<Record<string, unknown>>,
): void {
    const result = client.store.apply(
        changesSnapshotEvent(task, run, sequence, scope, files),
    );
    expect(result.status).toBe('applied');
}

class HostChangesTransport extends FixtureTransport {
    readonly hostChangesCalls: ChangesSnapshotInput[] = [];

    override async hostCapabilities(): Promise<HostCapabilities> {
        const capabilities = await super.hostCapabilities();
        return { ...capabilities, changes: true };
    }

    override async changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null> {
        this.hostChangesCalls.push(input);
        return {
            scope: input.scope,
            files: [{ path: 'host.ts' }],
            headDigest: HOST_CHANGES_DIGEST,
            capturedAtMs: 99,
        };
    }
}

class StaleMetadataTransport extends FixtureTransport {
    staleLatestRunId: string | null = null;

    override async listTasks(
        input: ListTasksInput = {},
    ): Promise<TaskSummary[]> {
        const tasks = await super.listTasks(input);
        return tasks.map((task) => ({
            ...task,
            latestRunId: this.staleLatestRunId,
            state: 'idle',
        }));
    }
}

describe('AgentClient', () => {
    test('unavailable_client_reports_runtime_unavailable', async () => {
        const client = new AgentClient();

        await expect(client.listTasks()).rejects.toMatchObject({
            name: 'AgentClientError',
            code: 'runtime_unavailable',
        });
        await expect(client.createTask({ title: 'demo' })).rejects.toMatchObject({
            code: 'runtime_unavailable',
        });
        await expect(client.hostCapabilities()).rejects.toMatchObject({
            code: 'runtime_unavailable',
        });
        expect(() => client.subscribe()).toThrowError(AgentClientError);
    });

    test('empty_task_list_is_normal_for_a_fixture_runtime', async () => {
        const client = new AgentClient({ transport: new FixtureTransport() });

        await expect(client.listTasks()).resolves.toEqual([]);
        await expect(client.hostCapabilities()).resolves.toEqual({
            tree: false,
            changes: false,
            readArtifact: false,
            terminal: false,
            git: false,
            mcp: false,
            approvals: false,
        });
    });

    test('resolve_approval_sends_only_id_and_decision', async () => {
        const transport = new FixtureTransport();
        const client = new AgentClient({ transport });
        const approvalId = 'apr_5c2e8a1f9b7d3e40';

        const receipt = await client.resolveApproval({
            approvalId,
            decision: 'approved',
            granted: true,
        } as unknown as { approvalId: string; decision: 'approved' });

        expect(receipt).toEqual({ approvalId, decision: 'approved' });
        const call = transport.calls
            .filter((entry) => entry.method === 'resolveApproval')
            .at(-1);
        expect(call).toBeDefined();
        expect(call?.payload).toEqual({ approvalId, decision: 'approved' });
        expect(Object.keys(call?.payload ?? {}).sort()).toEqual([
            'approvalId',
            'decision',
        ]);
        for (const forbidden of [
            'granted',
            'path',
            'command',
            'scope',
            'approvalKey',
            'mac',
        ]) {
            expect(call?.payload).not.toHaveProperty(forbidden);
        }
    });

    test('fixture_start_run_emits_run_queued_zero_and_run_started_one', async () => {
        const client = createFixtureAgentClient();
        const task = await client.createTask({ title: 'Fixture task' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'hello' });

        const view = client.store
            .snapshot()
            .runs.find((candidate) => candidate.runId === run.runId);
        expect(view).toBeDefined();
        expect(
            view?.events.slice(0, 2).map((event) => ({
                type: event.type,
                sequence: event.sequence,
            })),
        ).toEqual([
            { type: 'run.queued', sequence: 0 },
            { type: 'run.started', sequence: 1 },
        ]);
        expect(view?.state).toBe('running');
        expect(view?.lastDigest).toMatch(/^sha256:[0-9a-f]{64}$/);

        const startedEvent = view?.events[1];
        const startedData = startedEvent?.data as Record<string, unknown>;
        expect((startedData.model as Record<string, unknown>).kind).toBe('fixture');
        expect((startedData.environment as Record<string, unknown>).kind).toBe(
            'browser-fixture',
        );
        expect(startedData.retry_of_run_id).toBeNull();
    });

    test('local_changes_lookup_is_filtered_by_requested_scope', async () => {
        const client = createFixtureAgentClient();
        const task = await client.createTask({ title: 'Scopes' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'go' });
        applyChanges(client, task, run, 2, 'last_turn', [
            { path: 'local-last-turn.ts' },
        ]);
        applyChanges(client, task, run, 3, 'branch', [
            { path: 'local-branch.ts' },
        ]);

        const lastTurn = await client.changesSnapshot({
            taskId: task.taskId,
            scope: 'last_turn',
        });
        expect(lastTurn).toMatchObject({
            scope: 'last_turn',
            files: [{ path: 'local-last-turn.ts' }],
        });

        const branch = await client.changesSnapshot({
            taskId: task.taskId,
            scope: 'branch',
        }).then(
            () => null,
            (error: { code?: string }) => error.code ?? 'unknown',
        );
        expect(branch).toBe('host_unavailable');
    });

    test('scope_mismatch_does_not_leak_local_changes_or_mask_host_fallback', async () => {
        const lastTurnClient = createFixtureAgentClient();
        const lastTurnTask = await lastTurnClient.createTask({
            title: 'Last turn only',
        });
        const lastTurnRun = await lastTurnClient.startRun({
            taskId: lastTurnTask.taskId,
            prompt: 'go',
        });
        applyChanges(lastTurnClient, lastTurnTask, lastTurnRun, 2, 'last_turn', [
            { path: 'local-last-turn.ts' },
        ]);
        await expect(
            lastTurnClient.changesSnapshot({
                taskId: lastTurnTask.taskId,
                scope: 'branch',
            }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });

        const branchClient = createFixtureAgentClient();
        const branchTask = await branchClient.createTask({ title: 'Branch only' });
        const branchRun = await branchClient.startRun({
            taskId: branchTask.taskId,
            prompt: 'go',
        });
        applyChanges(branchClient, branchTask, branchRun, 2, 'branch', [
            { path: 'local-branch.ts' },
        ]);
        await expect(
            branchClient.changesSnapshot({
                taskId: branchTask.taskId,
                scope: 'last_turn',
            }),
        ).resolves.toBeNull();
        await expect(
            branchClient.changesSnapshot({
                taskId: branchTask.taskId,
                scope: 'branch',
            }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });

        const transport = new HostChangesTransport();
        const hostClient = new AgentClient({ transport });
        const hostTask = await hostClient.createTask({ title: 'Host fallback' });
        const hostRun = await hostClient.startRun({
            taskId: hostTask.taskId,
            prompt: 'go',
        });
        applyChanges(hostClient, hostTask, hostRun, 2, 'last_turn', [
            { path: 'local-last-turn.ts' },
        ]);
        applyChanges(hostClient, hostTask, hostRun, 3, 'branch', [
            { path: 'local-branch.ts' },
        ]);
        const hostBranch = await hostClient.changesSnapshot({
            taskId: hostTask.taskId,
            scope: 'branch',
        });
        expect(hostBranch).toMatchObject({
            scope: 'branch',
            files: [{ path: 'host.ts' }],
            headDigest: HOST_CHANGES_DIGEST,
        });
        expect(transport.hostChangesCalls).toHaveLength(1);

        const localLastTurn = await hostClient.changesSnapshot({
            taskId: hostTask.taskId,
            scope: 'last_turn',
        });
        expect(localLastTurn).toMatchObject({
            scope: 'last_turn',
            files: [{ path: 'local-last-turn.ts' }],
        });
        expect(transport.hostChangesCalls).toHaveLength(1);
    });

    test('transport_metadata_cannot_regress_event_derived_task_state', async () => {
        const transport = new StaleMetadataTransport();
        const client = new AgentClient({ transport });
        const task = await client.createTask({ title: 'Authoritative state' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'go' });

        const before = client.store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === task.taskId);
        expect(before?.state).toBe('running');
        expect(before?.latestRunId).toBe(run.runId);

        transport.staleLatestRunId = null;
        const nullListed = await client.listTasks();
        expect(nullListed[0]).toMatchObject({
            state: 'running',
            latestRunId: run.runId,
        });

        transport.staleLatestRunId = 'run_00000000000000aa';
        const olderListed = await client.listTasks();
        expect(olderListed[0]).toMatchObject({
            state: 'running',
            latestRunId: run.runId,
        });

        const after = client.store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === task.taskId);
        expect(after?.state).toBe('running');
        expect(after?.latestRunId).toBe(run.runId);
    });

    test('static_host_data_branches_fail_host_unavailable_and_empty_local_is_normal', async () => {
        const client = createFixtureAgentClient();
        const task = await client.createTask({ title: 'Static' });

        await expect(client.tree()).resolves.toEqual([]);
        await expect(client.tree({ source: 'host' })).rejects.toMatchObject({
            code: 'host_unavailable',
        });
        await expect(
            client.changesSnapshot({ taskId: task.taskId, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
        await expect(
            client.changesSnapshot({ taskId: task.taskId, scope: 'last_turn' }),
        ).resolves.toBeNull();
        await expect(
            client.readArtifact({ path: 'missing.txt' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
    });

    test('local_uploaded_files_are_served_before_host_branches', async () => {
        const client = createFixtureAgentClient();
        const digest = `sha256:${'a'.repeat(64)}`;
        client.registerLocalFile({
            path: 'src/a.ts',
            mediaType: 'text/plain',
            bytes: new TextEncoder().encode('hello'),
            digest,
        });

        await expect(client.tree()).resolves.toEqual([
            { path: 'src', kind: 'directory' },
            { path: 'src/a.ts', kind: 'file', sizeBytes: 5 },
        ]);

        const artifact = await client.readArtifact({ path: 'src/a.ts' });
        expect(artifact).toMatchObject({
            path: 'src/a.ts',
            mediaType: 'text/plain',
            sizeBytes: 5,
            digest,
        });
        expect(new TextDecoder().decode(artifact.bytes)).toBe('hello');
    });

    test('resume_task_uses_run_resumed_without_repeating_start_markers', async () => {
        const client = createFixtureAgentClient();
        const task = await client.createTask({ title: 'Resume' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'first' });
        const before = client.store
            .snapshot()
            .runs.find((candidate) => candidate.runId === run.runId);
        expect(before).toBeDefined();

        const snapshot = await client.resumeTask({
            taskId: task.taskId,
            runId: run.runId,
        });

        expect(snapshot.generation).toBe(before?.generation);
        expect(snapshot.lastSequence).toBe(before?.lastSequence);
        expect(snapshot.headDigest).toBe(before?.lastDigest);
        expect(snapshot.headDigest).toMatch(/^sha256:[0-9a-f]{64}$/);

        const after = client.store
            .snapshot()
            .runs.find((candidate) => candidate.runId === run.runId);
        expect(after?.generation).toBe((before?.generation ?? 0) + 1);
        expect(after?.lastSequence).toBe((before?.lastSequence ?? 0) + 1);
        expect(after?.state).toBe('running');
        expect(after?.events.filter((event) => event.type === 'run.queued')).toHaveLength(1);
        expect(after?.events.filter((event) => event.type === 'run.started')).toHaveLength(1);
        expect(after?.events.filter((event) => event.type === 'run.resumed')).toHaveLength(1);
    });

    test('stop_after_resume_uses_the_resumed_generation', async () => {
        const client = createFixtureAgentClient();
        const task = await client.createTask({ title: 'Resume stop' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'first' });
        await client.resumeTask({ taskId: task.taskId, runId: run.runId });

        const stopped = await client.stop({
            taskId: task.taskId,
            runId: run.runId,
        });
        expect(stopped.acknowledged).toBe(true);

        const view = client.store
            .snapshot()
            .runs.find((candidate) => candidate.runId === run.runId);
        expect(view?.generation).toBe(2);
        expect(view?.state).toBe('cancelled');
        expect(view?.needsResync).toBe(false);
    });

    test('fixture_message_and_stop_receipts_apply_events', async () => {
        const client = createFixtureAgentClient();
        const task = await client.createTask({ title: 'Flow' });
        const run = await client.startRun({ taskId: task.taskId, prompt: 'go' });

        const receipt = await client.queueMessage({
            taskId: task.taskId,
            text: 'next turn',
        });
        expect(receipt).toMatchObject({
            taskId: task.taskId,
            runId: run.runId,
            delivery: 'queue',
        });

        const stopped = await client.stop({
            taskId: task.taskId,
            runId: run.runId,
        });
        expect(stopped).toEqual({
            taskId: task.taskId,
            runId: run.runId,
            acknowledged: true,
        });

        const view = client.store
            .snapshot()
            .runs.find((candidate) => candidate.runId === run.runId);
        expect(view?.state).toBe('cancelled');
    });

    test('wire_ingestion_uses_the_strict_parser', async () => {
        const client = createFixtureAgentClient();

        await expect(client.ingestWireEvent(DUPLICATE_KEY_FRAME)).rejects.toMatchObject({
            name: 'AgentProtocolError',
            code: 'invalid_event',
        });
        expect(client.store.snapshot().runs).toHaveLength(0);
    });
});
