import { describe, expect, test } from 'vitest';

import { MAX_EVENT_BYTES } from '../agent/generated/agentContract.js';
import { canonicalJson } from '../agent/generated/agentCanonical.js';
import {
    AgentProtocolError,
    createAgentStore,
    parseAgentEventWire,
    type AgentIncomingEvent,
    type RunView,
} from './agent.js';

const PROJECT_ID = 'prj_4b1d9e0c7a2f6e51';
const TASK_ID = 'tsk_9a3c5e7b1d2f4a60';
const RUN_ID = 'run_7f3a2c1b9d4e6a80';
const MESSAGE_ID = 'msg_1f2e3d4c5b6a7988';

interface EventOverrides {
    projectId?: string;
    taskId?: string;
    runId?: string;
    generation?: number;
    eventId?: string;
    recordedAtMs?: number;
}

function eventIdFor(runId: string, generation: number, sequence: number): string {
    return `evt_${runId}_${generation}_${sequence.toString().padStart(12, '0')}`;
}

function makeEvent(
    sequence: number,
    type: string,
    data: Record<string, unknown>,
    overrides: EventOverrides = {},
): AgentIncomingEvent {
    const generation = overrides.generation ?? 1;
    const runId = overrides.runId ?? RUN_ID;
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: overrides.eventId ?? eventIdFor(runId, generation, sequence),
        project_id: overrides.projectId ?? PROJECT_ID,
        task_id: overrides.taskId ?? TASK_ID,
        run_id: runId,
        generation,
        sequence,
        recorded_at_ms: overrides.recordedAtMs ?? 1_794_283_200_000 + sequence,
        type,
        data,
    } as unknown as AgentIncomingEvent;
}

function queued(sequence = 0, overrides: EventOverrides = {}): AgentIncomingEvent {
    return makeEvent(
        sequence,
        'run.queued',
        { queued_at_ms: 1_794_283_200_000 + sequence },
        overrides,
    );
}

function startedData(): Record<string, unknown> {
    return {
        started_at_ms: 1_794_283_200_001,
        model: { id: 'replay', kind: 'fixture' },
        mode: 'agent',
        environment: { kind: 'browser-fixture' },
        retry_of_run_id: null,
    };
}

function started(sequence = 1, overrides: EventOverrides = {}): AgentIncomingEvent {
    return makeEvent(sequence, 'run.started', startedData(), overrides);
}

function resumed(
    sequence: number,
    previousGeneration: number,
    snapshotDigest: string,
): AgentIncomingEvent {
    return makeEvent(
        sequence,
        'run.resumed',
        {
            resumed_at_ms: 1_794_283_200_000 + sequence,
            previous_generation: previousGeneration,
            snapshot_digest: snapshotDigest,
        },
        { generation: previousGeneration + 1 },
    );
}

function verificationFor(run: RunView): {
    taskId: string;
    runId: string;
    generation: number;
    lastSequence: number;
    lastEventId: string;
    headDigest: string;
} {
    if (run.lastEventId === null || run.lastDigest === null) {
        throw new Error('test run has no event digest chain');
    }
    return {
        taskId: run.taskId,
        runId: run.runId,
        generation: run.generation,
        lastSequence: run.lastSequence,
        lastEventId: run.lastEventId,
        headDigest: run.lastDigest,
    };
}

function wireCodeOf(text: string): string {
    try {
        parseAgentEventWire(text);
    } catch (error) {
        expect(error).toBeInstanceOf(AgentProtocolError);
        return (error as AgentProtocolError).code;
    }
    throw new Error('expected the wire parser to reject the frame');
}

describe('AgentStore reducer', () => {
    test('run_queued_is_sequence_zero', () => {
        const store = createAgentStore();
        expect(store.apply(queued(0))).toMatchObject({
            status: 'applied',
            code: null,
            requiresResync: false,
        });

        const snapshot = store.snapshot();
        expect(snapshot.runs).toHaveLength(1);
        expect(snapshot.runs[0].runId).toBe(RUN_ID);
        expect(snapshot.runs[0].generation).toBe(1);
        expect(snapshot.runs[0].state).toBe('queued');
        expect(snapshot.runs[0].lastSequence).toBe(0);
        expect(snapshot.runs[0].lastEventId).toBe(eventIdFor(RUN_ID, 1, 0));
        expect(snapshot.tasks).toHaveLength(1);
        expect(snapshot.tasks[0].projectId).toBe(PROJECT_ID);
        expect(snapshot.tasks[0].state).toBe('queued');

        const late = createAgentStore().apply(queued(1));
        expect(late.status).toBe('rejected');
        expect(late.code).toBe('out_of_order');
        expect(late.requiresResync).toBe(true);
    });

    test('rejected_first_run_does_not_create_task_state', () => {
        const store = createAgentStore();
        const result = store.apply(queued(0, { generation: 2 }));

        expect(result.status).toBe('rejected');
        expect(result.code).toBe('out_of_order');
        expect(store.snapshot().tasks).toHaveLength(0);
        expect(store.snapshot().runs).toHaveLength(0);
    });

    test('terminal_events_are_authoritative_over_run_status', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        store.apply(makeEvent(2, 'run.status', { status: 'finished' }));

        expect(store.snapshot().runs[0].state).not.toBe('finished');
        expect(store.snapshot().runs[0].state).toBe('running');
        expect(store.snapshot().runs[0].statusProjection).toBe('finished');
        expect(store.snapshot().tasks[0].state).toBe('running');

        store.apply(
            makeEvent(3, 'run.finished', {
                status: 'finished',
                finished_at_ms: 2,
                duration_ms: 1,
            }),
        );

        expect(store.snapshot().runs[0].state).toBe('finished');
        expect(store.snapshot().runs[0].endedAtMs).toBe(2);
        expect(store.snapshot().tasks[0].state).toBe('finished');
    });

    test('waiting_for_background_hold_survives_the_terminal_run_and_clears', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        store.apply(
            makeEvent(2, 'run.failed', {
                code: 'stop_unacknowledged',
                message: 'acknowledgement timed out',
                retryable: false,
                in_flight_tool_id: 'tool_hold',
                stop_outcome: 'ack_timeout',
            }),
        );
        expect(store.snapshot().tasks[0]!.state).toBe('failed');

        const baseRecord = {
            taskId: TASK_ID,
            projectId: PROJECT_ID,
            title: 'Held task',
            createdAtMs: 1,
            archived: false,
            pinnedLocal: false,
            latestRunId: RUN_ID,
        };
        store.upsertTask({
            ...baseRecord,
            hold: {
                reason: 'waiting_for_background',
                run_id: RUN_ID,
                tool_call_id: 'tool_hold',
            },
        });
        const held = store.snapshot().tasks[0]!;
        expect(held.state).toBe('waiting_for_background');
        expect(held.hold).toEqual({
            reason: 'waiting_for_background',
            run_id: RUN_ID,
            tool_call_id: 'tool_hold',
        });
        expect(store.snapshot().runs[0]!.state).toBe('failed');

        store.upsertTask({ ...baseRecord, hold: null });
        expect(store.snapshot().tasks[0]!.state).toBe('failed');
        expect(store.snapshot().tasks[0]!.hold).toBeNull();
    });

    test('hold_for_a_previous_run_does_not_override_the_latest_run', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        store.apply(
            makeEvent(2, 'run.finished', {
                status: 'finished',
                finished_at_ms: 2,
                duration_ms: 1,
            }),
        );
        store.upsertTask({
            taskId: TASK_ID,
            projectId: PROJECT_ID,
            title: 'Old hold',
            createdAtMs: 1,
            archived: false,
            pinnedLocal: false,
            latestRunId: RUN_ID,
            hold: {
                reason: 'waiting_for_background',
                run_id: 'run_1111111111111111',
                tool_call_id: 'tool_old',
            },
        });
        expect(store.snapshot().tasks[0]!.state).toBe('finished');
    });

    test('reducer_ignores_duplicate_event', async () => {
        const store = createAgentStore();
        store.apply(queued(0));
        const duplicate = store.apply(queued(0));

        expect(duplicate.status).toBe('ignored');
        expect(duplicate.code).toBeNull();
        expect(duplicate.requiresResync).toBe(false);
        expect(store.snapshot().runs[0].events).toHaveLength(1);
        expect(store.snapshot().runs[0].lastSequence).toBe(0);

        const appendStore = createAgentStore();
        const first = await appendStore.appendEvent(queued(0));
        const second = await appendStore.appendEvent(queued(0));

        expect(first.status).toBe('applied');
        expect(second.status).toBe('ignored');
        expect(second.digest).toBe(first.digest);
        expect(appendStore.snapshot().runs[0].events).toHaveLength(1);
    });

    test('reducer_flags_gap_and_requires_resync', () => {
        const store = createAgentStore();
        store.apply(queued(0));

        const gap = store.apply(started(2));
        expect(gap.status).toBe('rejected');
        expect(gap.code).toBe('out_of_order');
        expect(gap.requiresResync).toBe(true);
        expect(store.snapshot().runs[0].needsResync).toBe(true);
        expect(store.snapshot().runs[0].lastSequence).toBe(0);

        const late = store.apply(started(1));
        expect(late.status).toBe('rejected');
        expect(late.requiresResync).toBe(true);
        expect(store.snapshot().runs[0].events).toHaveLength(1);
    });

    test('reducer_does_not_render_unknown_event_as_message', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        store.apply(
            makeEvent(2, 'assistant.message', {
                message_id: MESSAGE_ID,
                text: 'known assistant output',
            }),
        );

        const opaqueData = {
            opaque_payload: { count: 1 },
            text: 'MUST NOT RENDER',
        };
        const unknown = store.apply(makeEvent(3, 'future.event', opaqueData));

        expect(unknown.status).toBe('applied');
        const run = store.snapshot().runs[0];
        expect(run.state).toBe('running');
        expect(run.messages).toHaveLength(1);
        expect(run.messages[0].text).toBe('known assistant output');
        expect(run.toolCalls).toHaveLength(0);
        expect(run.unknownEvents).toHaveLength(1);
        expect(run.unknownEvents[0].type).toBe('future.event');
        expect(run.unknownEvents[0].data).toEqual(opaqueData);
        expect(run.events.map((event) => event.type)).toContain('future.event');
        expect(store.snapshot().tasks[0].state).toBe('running');
    });

    test('changes_snapshot_missing_files_is_atomic_rejection', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        const digest = `sha256:${'c'.repeat(64)}`;
        const malformed = makeEvent(2, 'changes.snapshot', {
            scope: 'last_turn',
            head_digest: digest,
            captured_at_ms: 2,
        });

        const rejected = store.apply(malformed);
        expect(rejected.status).toBe('rejected');
        expect(rejected.code).toBe('invalid_event');
        expect(store.apply(malformed).status).toBe('rejected');

        let run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(1);
        expect(run.events).toHaveLength(2);
        expect(run.changesSnapshots).toHaveLength(0);
        expect(run.state).toBe('running');

        const valid = store.apply(
            makeEvent(2, 'changes.snapshot', {
                scope: 'last_turn',
                files: [{ path: 'src/a.ts' }],
                head_digest: digest,
                captured_at_ms: 2,
            }),
        );
        expect(valid.status).toBe('applied');

        run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(2);
        expect(run.changesSnapshots).toHaveLength(1);
    });

    test('changes_snapshot_accepts_non_sha_non_empty_head_digest_and_rejects_empty_atomically', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        const gitCommit = '0123456789abcdef0123456789abcdef01234567';

        const accepted = store.apply(
            makeEvent(2, 'changes.snapshot', {
                scope: 'last_turn',
                files: [{ path: 'src/a.ts' }],
                head_digest: gitCommit,
                captured_at_ms: 2,
            }),
        );
        expect(accepted.status).toBe('applied');

        let run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(2);
        expect(run.changesSnapshots[0]?.headDigest).toBe(gitCommit);
        expect(run.needsResync).toBe(false);

        const following = store.apply(
            makeEvent(3, 'assistant.message', {
                message_id: 'msg_1f2e3d4c5b6a7988',
                text: 'after changes',
            }),
        );
        expect(following.status).toBe('applied');

        run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(3);
        expect(run.messages).toHaveLength(1);
        expect(run.needsResync).toBe(false);

        const empty = store.apply(
            makeEvent(4, 'changes.snapshot', {
                scope: 'last_turn',
                files: [],
                head_digest: '',
                captured_at_ms: 4,
            }),
        );
        expect(empty.status).toBe('rejected');
        expect(empty.code).toBe('invalid_event');

        run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(3);
        expect(run.events).toHaveLength(4);
        expect(run.changesSnapshots).toHaveLength(1);
        expect(run.needsResync).toBe(false);

        const afterRejection = store.apply(
            makeEvent(4, 'assistant.message', {
                message_id: 'msg_2a3b4c5d6e7f8091',
                text: 'after rejection',
            }),
        );
        expect(afterRejection.status).toBe('applied');

        run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(4);
        expect(run.messages).toHaveLength(2);
        expect(run.messages[1]?.text).toBe('after rejection');
    });

    test('required_tool_payloads_are_rejected_atomically', () => {
        const store = createAgentStore();
        store.apply(queued(0));
        store.apply(started(1));
        const digest = `sha256:${'d'.repeat(64)}`;
        const malformed = [
            makeEvent(2, 'tool.requested', {
                tool_call_id: 'tc_02',
                tool: 'file.read',
                arguments_preview: '{}',
            }),
            makeEvent(2, 'tool.started', {}),
            makeEvent(2, 'tool.completed', {
                tool_call_id: 'tc_02',
                status: 'ok',
                output_preview: 'ok',
                truncated: false,
            }),
            makeEvent(2, 'tool.failed', {
                tool_call_id: 'tc_02',
                code: 'io',
            }),
            makeEvent(2, 'tool.approval_required', {
                approval_id: 'apr_5c2e8a1f9b7d3e40',
                tool_call_id: 'tc_02',
                tool: 'git.commit',
                scope: 'write',
                request_digest: digest,
            }),
            makeEvent(2, 'error', {
                code: 'model_protocol_error',
                message: 'bad',
            }),
        ];

        for (const event of malformed) {
            const result = store.apply(event);
            expect(result.status).toBe('rejected');
            expect(result.code).toBe('invalid_event');
        }

        const run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(1);
        expect(run.events).toHaveLength(2);
        expect(run.toolCalls).toHaveLength(0);
        expect(run.approvals).toHaveLength(0);
        expect(run.error).toBeNull();
        expect(run.state).toBe('running');

        const valid = store.apply(
            makeEvent(2, 'tool.requested', {
                tool_call_id: 'tc_02',
                tool: 'file.read',
                arguments_digest: digest,
                arguments_preview: '{}',
            }),
        );
        expect(valid.status).toBe('applied');
        expect(store.snapshot().runs[0].lastSequence).toBe(2);
        expect(store.snapshot().runs[0].toolCalls).toHaveLength(1);
    });

    test('reducer_marks_stale_generation', async () => {
        const store = createAgentStore();
        await store.appendEvent(queued(0));
        await store.appendEvent(started(1));
        const beforeResume = store.snapshot().runs[0];
        expect(store.verifySnapshot(verificationFor(beforeResume)).ok).toBe(true);
        await store.appendEvent(
            resumed(2, beforeResume.generation, beforeResume.lastDigest ?? ''),
        );

        const stale = store.apply(
            makeEvent(3, 'assistant.message', {
                message_id: MESSAGE_ID,
                text: 'old generation',
            }, { generation: 1 }),
        );

        expect(stale.status).toBe('rejected');
        expect(stale.code).toBe('stale_generation');
        expect(stale.requiresResync).toBe(true);
        const run = store.snapshot().runs[0];
        expect(run.generation).toBe(2);
        expect(run.lastSequence).toBe(2);
        expect(run.needsResync).toBe(true);
        expect(run.messages).toHaveLength(0);
    });

    test('reducer_rejects_cross_task_event', () => {
        const store = createAgentStore();
        store.apply(queued(0));

        const foreignTask = store.apply(
            started(1, { taskId: 'tsk_00000000000000ff' }),
        );
        expect(foreignTask.status).toBe('rejected');
        expect(foreignTask.code).toBe('cross_task_event');
        expect(foreignTask.requiresResync).toBe(false);

        const foreignProject = store.apply(
            started(1, { projectId: 'prj_00000000000000ff' }),
        );
        expect(foreignProject.status).toBe('rejected');
        expect(foreignProject.code).toBe('cross_task_event');
        expect(foreignProject.requiresResync).toBe(false);

        const snapshot = store.snapshot();
        expect(snapshot.tasks).toHaveLength(1);
        expect(snapshot.runs).toHaveLength(1);
        expect(snapshot.runs[0].lastSequence).toBe(0);
    });

    test('duplicate_event_id_with_different_digest_is_a_hard_refusal', async () => {
        const store = createAgentStore();
        const first = await store.appendEvent(queued(0));

        const conflicting = await store.appendEvent(
            makeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_999 }),
        );

        expect(conflicting.status).toBe('rejected');
        expect(conflicting.code).toBe('duplicate_event');
        expect(conflicting.requiresResync).toBe(true);
        expect(conflicting.digest).not.toBe(first.digest);

        const run = store.snapshot().runs[0];
        expect(run.lastSequence).toBe(0);
        expect(run.events).toHaveLength(1);
        expect(run.needsResync).toBe(true);

        const forgedPosition = store.apply(
            queued(0, { eventId: 'evt_run_7f3a2c1b9d4e6a80_1_000000000001' }),
        );
        expect(forgedPosition.status).toBe('rejected');
        expect(forgedPosition.code).toBe('invalid_event');
    });

    test('raw_duplicate_keys_and_lexical_numbers_are_rejected_by_the_wire_parser', () => {
        const canonical = canonicalJson(queued(0));
        expect(parseAgentEventWire(canonical).event_id).toBe(
            eventIdFor(RUN_ID, 1, 0),
        );

        const duplicateKeys = canonical.replace(
            '{"queued_at_ms":1794283200000}',
            '{"queued_at_ms":1794283200000,"queued_at_ms":1794283200001}',
        );
        expect(duplicateKeys).not.toBe(canonical);
        expect(wireCodeOf(duplicateKeys)).toBe('invalid_event');
        expect(canonicalJson(JSON.parse(duplicateKeys))).toContain(
            '"queued_at_ms":1794283200001',
        );

        const lexicalExponent = canonical.replace(
            '"queued_at_ms":1794283200000',
            '"queued_at_ms":1e3',
        );
        expect(wireCodeOf(lexicalExponent)).toBe('invalid_event');
        expect(canonicalJson(JSON.parse(lexicalExponent))).toContain(
            '"queued_at_ms":1000',
        );

        const unsafeInteger = canonical.replace(
            '"queued_at_ms":1794283200000',
            '"queued_at_ms":9007199254740992',
        );
        expect(wireCodeOf(unsafeInteger)).toBe('invalid_event');
    });

    test('runtime_validation_enforces_envelope_and_payload_constraints', () => {
        const store = createAgentStore();

        const badIdentity = store.apply(
            queued(0, { eventId: 'evt_run_7f3a2c1b9d4e6a80_1_000000000001' }),
        );
        expect(badIdentity.code).toBe('invalid_event');

        const badSchema = store.apply({
            ...queued(0),
            schema: 'uor-r4.studio.agent-event/2',
        } as unknown as AgentIncomingEvent);
        expect(badSchema.code).toBe('invalid_event');

        const missingRetry = store.apply(
            makeEvent(1, 'run.started', {
                ...startedData(),
                retry_of_run_id: undefined,
            }),
        );
        expect(missingRetry.code).toBe('invalid_event');

        const numericRetry = store.apply(
            makeEvent(1, 'run.started', {
                ...startedData(),
                retry_of_run_id: 7,
            }),
        );
        expect(numericRetry.code).toBe('invalid_event');

        const badResumeDigest = store.apply(
            makeEvent(
                2,
                'run.resumed',
                {
                    resumed_at_ms: 2,
                    previous_generation: 1,
                    snapshot_digest: 'sha256:xyz',
                },
                { generation: 2 },
            ),
        );
        expect(badResumeDigest.code).toBe('invalid_event');

        const unsafeValue = store.apply(
            makeEvent(0, 'run.queued', { queued_at_ms: 9_007_199_254_740_992 }),
        );
        expect(unsafeValue.code).toBe('invalid_event');

        const oversized = store.apply({
            ...queued(0),
            padding: 'x'.repeat(MAX_EVENT_BYTES),
        } as unknown as AgentIncomingEvent);
        expect(oversized.code).toBe('limit_exceeded');
    });

    test('run_resumed_advances_generation_and_sequence_only_after_verified_snapshot', async () => {
        const store = createAgentStore();
        await store.appendEvent(queued(0));
        await store.appendEvent(started(1));
        await store.appendEvent(
            makeEvent(2, 'assistant.message', {
                message_id: MESSAGE_ID,
                text: 'hello',
            }),
        );

        const before = store.snapshot().runs[0];
        expect(before.messages).toHaveLength(1);
        expect(before.lastDigest).toMatch(/^sha256:[0-9a-f]{64}$/);
        const verification = verificationFor(before);
        const resumeEvent = resumed(3, before.generation, verification.headDigest);

        const withoutSnapshot = await store.appendEvent(resumeEvent);
        expect(withoutSnapshot.status).toBe('rejected');
        expect(withoutSnapshot.code).toBe('snapshot_missing');
        expect(withoutSnapshot.requiresResync).toBe(true);
        expect(store.snapshot().runs[0].needsResync).toBe(true);

        expect(store.verifySnapshot(verification)).toMatchObject({
            ok: true,
            code: null,
        });
        expect(store.snapshot().runs[0].needsResync).toBe(false);

        const freshJump = await store.appendEvent(
            makeEvent(3, 'run.started', startedData(), { generation: 2 }),
        );
        expect(freshJump.status).toBe('rejected');
        expect(freshJump.code).toBe('out_of_order');
        expect(freshJump.requiresResync).toBe(true);

        expect(store.verifySnapshot(verification).ok).toBe(true);
        const applied = await store.appendEvent(resumeEvent);
        expect(applied.status).toBe('applied');

        const after = store.snapshot().runs[0];
        expect(after.generation).toBe(2);
        expect(after.lastSequence).toBe(3);
        expect(after.lastEventId).toBe(eventIdFor(RUN_ID, 2, 3));
        expect(after.state).toBe('running');
        expect(after.messages).toHaveLength(1);
        expect(after.messages[0].text).toBe('hello');
        expect(after.events.filter((event) => event.type === 'run.queued')).toHaveLength(1);
        expect(after.events.filter((event) => event.type === 'run.started')).toHaveLength(1);
        expect(after.events.filter((event) => event.type === 'run.resumed')).toHaveLength(1);
        expect(after.generationTransitions).toEqual([
            {
                from: 1,
                to: 2,
                atSequence: 3,
                previousGeneration: 1,
                snapshotDigest: verification.headDigest,
            },
        ]);

        const repeated = await store.appendEvent(
            resumed(4, after.generation, verification.headDigest),
        );
        expect(repeated.status).toBe('rejected');
        expect(repeated.code).toBe('snapshot_missing');
    });

    test('run_resumed_rejects_wrong_previous_generation_and_snapshot_digest', async () => {
        const store = createAgentStore();
        await store.appendEvent(queued(0));
        await store.appendEvent(started(1));
        const before = store.snapshot().runs[0];
        const verification = verificationFor(before);

        expect(store.verifySnapshot(verification).ok).toBe(true);
        const wrongGeneration = await store.appendEvent(
            makeEvent(
                2,
                'run.resumed',
                {
                    resumed_at_ms: 1_794_283_200_002,
                    previous_generation: before.generation + 1,
                    snapshot_digest: verification.headDigest,
                },
                { generation: before.generation + 1 },
            ),
        );
        expect(wrongGeneration.status).toBe('rejected');
        expect(wrongGeneration.code).toBe('stale_generation');
        expect(wrongGeneration.requiresResync).toBe(true);

        expect(store.verifySnapshot(verification).ok).toBe(true);
        const wrongDigest = await store.appendEvent(
            resumed(2, before.generation, `sha256:${'b'.repeat(64)}`),
        );
        expect(wrongDigest.status).toBe('rejected');
        expect(wrongDigest.code).toBe('snapshot_corrupt');
        expect(wrongDigest.requiresResync).toBe(true);

        expect(store.snapshot().runs[0].generation).toBe(1);
        expect(store.snapshot().runs[0].lastSequence).toBe(1);
    });

    test('verify_snapshot_rejects_a_tampered_chain', async () => {
        const store = createAgentStore();
        await store.appendEvent(queued(0));
        await store.appendEvent(started(1));
        const run = store.snapshot().runs[0];
        const verification = verificationFor(run);

        const tampered = store.verifySnapshot({
            ...verification,
            lastSequence: verification.lastSequence + 1,
        });
        expect(tampered.ok).toBe(false);
        expect(tampered.code).toBe('snapshot_corrupt');
        expect(store.snapshot().runs[0].needsResync).toBe(true);
    });

    test('a null-latest task record cannot clear an unresolved latest run', () => {
        const store = createAgentStore();
        const record = {
            taskId: TASK_ID,
            projectId: PROJECT_ID,
            title: 'Unloaded task',
            createdAtMs: 1,
            archived: false,
            pinnedLocal: false,
        };
        store.upsertTask({ ...record, latestRunId: RUN_ID });
        expect(store.snapshot().tasks[0]!.unresolvedRunId).toBe(RUN_ID);

        store.upsertTask({ ...record, latestRunId: null });
        const view = store.snapshot().tasks[0]!;
        expect(view.latestRunId).toBe(RUN_ID);
        expect(view.unresolvedRunId).toBe(RUN_ID);
        expect(view.state).toBe('restoring');
    });
});

describe('AgentStore scoped mutation journal', () => {
    const digestFor = (sequence: number): string =>
        `sha256:${sequence.toString(16).padStart(64, '0')}`;

    function applyEvents(store: ReturnType<typeof createAgentStore>, events: AgentIncomingEvent[]) {
        for (const event of events) {
            const result = store.apply(event, digestFor(event.sequence));
            expect(result.status, `${event.type} baseline`).toBe('applied');
        }
    }

    function baseStore(): ReturnType<typeof createAgentStore> {
        const store = createAgentStore();
        applyEvents(store, [queued(0), started(1)]);
        return store;
    }

    function expectRollback(
        store: ReturnType<typeof createAgentStore>,
        event: AgentIncomingEvent,
    ): void {
        const before = JSON.parse(JSON.stringify(store.snapshot()));
        const journaled = store.applyJournaled(event, digestFor(event.sequence));
        expect(journaled.result.status, `${event.type} apply`).toBe('applied');
        journaled.rollback();
        expect(JSON.parse(JSON.stringify(store.snapshot())), `${event.type} rollback`).toEqual(before);
    }

    const toolRequested = (sequence: number): AgentIncomingEvent =>
        makeEvent(sequence, 'tool.requested', {
            tool_call_id: 'tool_1',
            tool: 'read',
            arguments_digest: `sha256:${'a'.repeat(64)}`,
            arguments_preview: '{}',
        });
    const toolStarted = (sequence: number): AgentIncomingEvent =>
        makeEvent(sequence, 'tool.started', { tool_call_id: 'tool_1' });
    const approvalRequired = (sequence: number): AgentIncomingEvent =>
        makeEvent(sequence, 'tool.approval_required', {
            approval_id: 'apr_1',
            tool_call_id: 'tool_1',
            tool: 'write',
            scope: 'file:src/a.ts',
            request_digest: `sha256:${'b'.repeat(64)}`,
            expires_at_ms: 1_794_283_300_000,
        });
    const approvalResolved = (sequence: number): AgentIncomingEvent =>
        makeEvent(sequence, 'tool.approval_resolved', {
            approval_id: 'apr_1',
            decision: 'approved',
            resolver: 'host',
        });

    test('rollback restores each event family exactly', () => {
        const cases: Array<{
            name: string;
            setup?: AgentIncomingEvent[];
            event: AgentIncomingEvent;
        }> = [
            { name: 'run.status', event: makeEvent(2, 'run.status', { status: 'cancelling' }) },
            { name: 'assistant.message', event: makeEvent(2, 'assistant.message', { message_id: 'msg_1', text: 'hello' }) },
            { name: 'assistant.delta', event: makeEvent(2, 'assistant.delta', { message_id: 'msg_1', text_delta: 'hi' }) },
            { name: 'tool.requested', event: toolRequested(2) },
            { name: 'tool.started', setup: [toolRequested(2)], event: toolStarted(3) },
            {
                name: 'tool.completed',
                setup: [toolRequested(2), toolStarted(3)],
                event: makeEvent(4, 'tool.completed', {
                    tool_call_id: 'tool_1',
                    status: 'ok',
                    output_digest: `sha256:${'c'.repeat(64)}`,
                    output_preview: 'ok',
                    truncated: false,
                }),
            },
            {
                name: 'tool.failed',
                setup: [toolRequested(2), toolStarted(3)],
                event: makeEvent(4, 'tool.failed', { tool_call_id: 'tool_1', code: 'denied', message: 'no' }),
            },
            { name: 'approval_required', setup: [toolRequested(2)], event: approvalRequired(3) },
            {
                name: 'approval_resolved',
                setup: [toolRequested(2), approvalRequired(3)],
                event: approvalResolved(4),
            },
            {
                name: 'changes.snapshot',
                event: makeEvent(2, 'changes.snapshot', {
                    scope: 'last_turn',
                    files: [{ path: 'src/a.ts' }],
                    head_digest: 'turn:1',
                    captured_at_ms: 1_794_283_200_002,
                }),
            },
            {
                name: 'existing assistant.message update',
                setup: [makeEvent(2, 'assistant.message', { message_id: 'msg_1', text: 'first' })],
                event: makeEvent(3, 'assistant.message', { message_id: 'msg_1', text: 'second' }),
            },
            {
                name: 'error',
                event: makeEvent(2, 'error', {
                    code: 'fixture',
                    message: 'fixture error',
                    retryable: false,
                }),
            },
            {
                name: 'run.finished',
                event: makeEvent(2, 'run.finished', {
                    status: 'finished',
                    finished_at_ms: 1_794_283_200_002,
                    duration_ms: 1,
                }),
            },
            {
                name: 'run.failed',
                event: makeEvent(2, 'run.failed', {
                    code: 'fixture',
                    message: 'failed',
                    retryable: true,
                }),
            },
            {
                name: 'run.cancelled',
                event: makeEvent(2, 'run.cancelled', {
                    safe_boundary: true,
                    finished_at_ms: 1_794_283_200_002,
                }),
            },
            {
                name: 'run.expired',
                event: makeEvent(2, 'run.expired', {
                    expired_at_ms: 1_794_283_200_002,
                    reason: 'retention',
                }),
            },
        ];
        for (const entry of cases) {
            const store = baseStore();
            if (entry.setup !== undefined) applyEvents(store, entry.setup);
            expectRollback(store, entry.event);
        }
    });

    test('rollback removes a new run and restores task metadata', () => {
        const store = baseStore();
        const before = JSON.parse(JSON.stringify(store.snapshot()));
        const next = makeEvent(0, 'run.queued', { queued_at_ms: 1_794_283_200_010 }, {
            generation: 2,
            runId: 'run_1111111111111111',
        });
        const journaled = store.applyJournaled(next, digestFor(0));
        expect(journaled.result.status).toBe('applied');
        journaled.rollback();
        expect(store.snapshot().runs).toHaveLength(1);
        expect(store.snapshot().tasks[0]?.latestRunId).toBe(RUN_ID);
        expect(JSON.parse(JSON.stringify(store.snapshot()))).toEqual(before);
    });

    test('rollback removes a brand-new task and run', () => {
        const store = createAgentStore();
        const journaled = store.applyJournaled(queued(0), digestFor(0));
        expect(journaled.result.status).toBe('applied');
        expect(store.snapshot().tasks).toHaveLength(1);
        journaled.rollback();
        expect(store.snapshot().tasks).toHaveLength(0);
        expect(store.snapshot().runs).toHaveLength(0);
    });

    test('rollback restores the verified snapshot needed by run.resumed', () => {
        const store = baseStore();
        const run = store.snapshot().runs[0];
        const verification = verificationFor(run);
        expect(store.verifySnapshot(verification).ok).toBe(true);
        const before = JSON.parse(JSON.stringify(store.snapshot()));
        const resumeEvent = resumed(2, run.generation, verification.headDigest);
        const journaled = store.applyJournaled(resumeEvent, digestFor(2));
        expect(journaled.result.status).toBe('applied');
        journaled.rollback();
        expect(JSON.parse(JSON.stringify(store.snapshot()))).toEqual(before);
        expect(store.apply(resumeEvent, digestFor(2)).status).toBe('applied');
        expect(store.snapshot().runs[0]?.generation).toBe(2);
    });
});

describe('AgentStore delivery-error projection', () => {
    function applyDeliveryEvents(store: ReturnType<typeof createAgentStore>, events: AgentIncomingEvent[]) {
        for (const event of events) {
            const result = store.apply(event);
            expect(result.status, `${event.type} at ${event.sequence}`).toBe('applied');
        }
    }

    test('message_not_delivered_is_scoped_to_the_message_and_cleared_from_run_error', () => {
        const store = createAgentStore();
        applyDeliveryEvents(store, [
            queued(0),
            started(1),
            makeEvent(2, 'message.queued', {
                message_id: MESSAGE_ID,
                delivery: 'queue',
                boundary: 'next_turn',
                text_preview: 'queued follow-up',
            }),
            makeEvent(3, 'error', {
                code: 'message_not_delivered',
                message: `message ${MESSAGE_ID} was not delivered (cancelled)`,
                retryable: false,
            }),
            makeEvent(4, 'run.cancelled', {
                safe_boundary: true,
                finished_at_ms: 1_794_283_200_004,
            }),
        ]);

        const run = store.snapshot().runs[0]!;
        expect(run.state).toBe('cancelled');
        expect(run.error, 'a delivery error is not a run-level failure').toBeNull();
        expect(run.deliveryErrors).toEqual([
            {
                code: 'message_not_delivered',
                messageId: MESSAGE_ID,
                message: `message ${MESSAGE_ID} was not delivered (cancelled)`,
            },
        ]);
    });

    test('message_not_delivered_does_not_survive_a_successful_terminal_event', () => {
        const store = createAgentStore();
        applyDeliveryEvents(store, [
            queued(0),
            started(1),
            makeEvent(2, 'error', {
                code: 'message_not_delivered',
                message: `message ${MESSAGE_ID} was not delivered (finished)`,
                retryable: false,
            }),
            makeEvent(3, 'run.finished', {
                status: 'finished',
                finished_at_ms: 1_794_283_200_003,
                duration_ms: 3,
            }),
        ]);

        const run = store.snapshot().runs[0]!;
        expect(run.state).toBe('finished');
        expect(run.error).toBeNull();
        expect(run.deliveryErrors).toHaveLength(1);
        expect(run.deliveryErrors[0]?.messageId).toBe(MESSAGE_ID);
    });

    test('genuine_error_and_run_failure_remain_run_level', () => {
        const errorStore = createAgentStore();
        applyDeliveryEvents(errorStore, [
            queued(0),
            started(1),
            makeEvent(2, 'error', {
                code: 'model_protocol_error',
                message: 'bad model frame',
                retryable: true,
            }),
        ]);
        const transient = errorStore.snapshot().runs[0]!;
        expect(transient.error).toEqual({
            code: 'model_protocol_error',
            message: 'bad model frame',
            retryable: true,
        });
        expect(transient.deliveryErrors).toEqual([]);

        const failedStore = createAgentStore();
        applyDeliveryEvents(failedStore, [
            queued(0),
            started(1),
            makeEvent(2, 'run.failed', {
                code: 'model_unavailable',
                message: 'synthetic retryable failure',
                retryable: true,
            }),
        ]);
        const failed = failedStore.snapshot().runs[0]!;
        expect(failed.error).toEqual({
            code: 'model_unavailable',
            message: 'synthetic retryable failure',
            retryable: true,
        });
        expect(failed.deliveryErrors).toEqual([]);
    });

    test('journal rollback restores the previous delivery-error projection', () => {
        const store = createAgentStore();
        applyDeliveryEvents(store, [
            queued(0),
            started(1),
            makeEvent(2, 'error', {
                code: 'message_not_delivered',
                message: `message ${MESSAGE_ID} was not delivered (cancelled)`,
                retryable: false,
            }),
        ]);
        const before = JSON.parse(JSON.stringify(store.snapshot().runs[0]));
        const journaled = store.applyJournaled(
            makeEvent(3, 'run.cancelled', {
                safe_boundary: true,
                finished_at_ms: 1_794_283_200_003,
            }),
        );
        expect(journaled.result.status).toBe('applied');
        journaled.rollback();
        expect(JSON.parse(JSON.stringify(store.snapshot().runs[0]))).toEqual(before);
    });
});

describe('AgentStore task selection', () => {
    test('clear_selection_removes_only_the_selected_task_view', () => {
        const store = createAgentStore();
        for (const entry of [queued(0), started(1)]) {
            expect(store.apply(entry).status).toBe('applied');
        }
        expect(store.selectTask(TASK_ID)?.taskId).toBe(TASK_ID);
        expect(store.snapshot().selectedTaskId).toBe(TASK_ID);
        expect(store.snapshot().selectedTask?.taskId).toBe(TASK_ID);

        store.clearSelection();
        expect(store.snapshot().selectedTaskId).toBeNull();
        expect(store.snapshot().selectedTask).toBeNull();
        expect(store.snapshot().tasks.map((task) => task.taskId)).toEqual([TASK_ID]);
    });
});

describe('run.status projection authority', () => {
    test('a terminal projection cannot make a running run terminal', () => {
        const store = createAgentStore();
        for (const entry of [
            queued(0),
            started(1),
            makeEvent(2, 'run.status', { status: 'finished' }),
        ]) {
            expect(store.apply(entry).status).toBe('applied');
        }
        const run = store.snapshot().runs[0]!;
        expect(run.state).toBe('running');
        expect(run.statusProjection).toBe('finished');
        expect(store.snapshot().tasks[0]?.state).toBe('running');
    });

    test('only the non-terminal cancelling projection changes the task display', () => {
        const store = createAgentStore();
        for (const entry of [
            queued(0),
            started(1),
            makeEvent(2, 'run.status', { status: 'cancelling' }),
        ]) {
            expect(store.apply(entry).status).toBe('applied');
        }
        const run = store.snapshot().runs[0]!;
        expect(run.state).toBe('running');
        expect(run.statusProjection).toBe('cancelling');
        expect(store.snapshot().tasks[0]?.state).toBe('cancelling');
    });
});
