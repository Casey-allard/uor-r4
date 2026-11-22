import { describe, expect, test } from 'vitest';

import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
} from '../state/agent.js';
import { canonicalJson } from './generated/agentCanonical.js';
import { AGENT_EVENT_SCHEMA } from './generated/agentContract.js';
import {
    createRunQueuedEvent,
    createRunResumedEvent,
    createRunStartedEvent,
    fixtureEventId,
} from './fixtures.js';
import {
    IngestOutcome,
    NdjsonBuffer,
    NdjsonIngest,
    decodeEof,
    decodeFrame,
} from './ingest.js';

const PROJECT_ID = 'prj_0000000000000001';
const TASK_A = 'tsk_00000000000000a1';
const TASK_B = 'tsk_00000000000000b2';
const RUN_A = 'run_00000000000000a1';
const RUN_B = 'run_00000000000000b2';
const RUN_C = 'run_00000000000000c3';
const SNAPSHOT_DIGEST = `sha256:${'a'.repeat(64)}`;
const ARGUMENTS_DIGEST = `sha256:${'b'.repeat(64)}`;

function bytes(text: string): Uint8Array {
    return new TextEncoder().encode(text);
}

function text(buffer: Uint8Array): string {
    return new TextDecoder().decode(buffer);
}

function eventFor(input: {
    taskId: string;
    runId: string;
    generation?: number;
    sequence: number;
    type: string;
    data: Record<string, unknown>;
    recordedAtMs?: number;
}): AgentIncomingEvent {
    const generation = input.generation ?? 1;
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: fixtureEventId(input.runId, generation, input.sequence),
        project_id: PROJECT_ID,
        task_id: input.taskId,
        run_id: input.runId,
        generation,
        sequence: input.sequence,
        recorded_at_ms: input.recordedAtMs ?? 1_800_000_000_000 + input.sequence,
        type: input.type,
        data: input.data,
    } as unknown as AgentIncomingEvent;
}

function ndjson(event: AgentIncomingEvent): string {
    return `${canonicalJson(event)}\n`;
}

async function appendStored(
    store: AgentStore,
    event: AgentIncomingEvent,
): Promise<void> {
    const result = await store.appendEvent(event);
    expect(result.status, `${event.type}: ${result.message ?? ''}`).toBe(
        'applied',
    );
}

async function seedRun(
    store: AgentStore,
    taskId = TASK_A,
    runId = RUN_A,
): Promise<void> {
    await appendStored(
        store,
        createRunQueuedEvent({
            projectId: PROJECT_ID,
            taskId,
            runId,
            generation: 1,
            recordedAtMs: 1_800_000_000_000,
        }),
    );
    await appendStored(
        store,
        createRunStartedEvent({
            projectId: PROJECT_ID,
            taskId,
            runId,
            generation: 1,
            recordedAtMs: 1_800_000_000_001,
            startedAtMs: 1_800_000_000_001,
        }),
    );
}

function workerFor(
    store: AgentStore,
    taskId = TASK_A,
    runId = RUN_A,
): NdjsonIngest {
    return new NdjsonIngest({
        taskId,
        runId,
        sink: (event) => store.appendEvent(event),
    });
}

function runOf(store: AgentStore, taskId = TASK_A, runId = RUN_A) {
    const run = store.getRun(taskId, runId);
    if (run === null) throw new Error('seeded run is missing');
    return run;
}

function expectNoOutput(store: AgentStore): void {
    const run = runOf(store);
    expect(run.messages).toEqual([]);
    expect(run.toolCalls).toEqual([]);
    expect(run.state).not.toBe('finished');
    expect(run.events.map((event) => event.type)).not.toContain(
        'assistant.message',
    );
    expect(run.events.map((event) => event.type)).not.toContain(
        'tool.completed',
    );
    expect(run.events.map((event) => event.type)).not.toContain('run.finished');
}

describe('NDJSON frame decoding', () => {
    test('decode_frame_buffers_partial_frames_at_every_byte_boundary', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const event = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.delta',
            data: {
                message_id: 'msg_00000000000000a1',
                text_delta: 'héllo 🌍 — intact',
            },
        });
        const frame = canonicalJson(event);
        const encoded = bytes(`${frame}\n`);

        for (let split = 1; split < encoded.length; split += 1) {
            const first = decodeFrame(encoded.subarray(0, split));
            expect(first.frames, `split ${split} emitted a partial frame`).toEqual(
                [],
            );
            expect(first.error).toBeNull();
            const second = decodeFrame(
                encoded.subarray(split),
                first.buffer,
            );
            expect(second.frames, `split ${split} lost the frame`).toEqual([
                frame,
            ]);
            expect(second.buffer.length).toBe(0);
        }
    });

    test('decode_frame_returns_complete_frames_and_keeps_the_trailing_partial', () => {
        const first = '{"alpha":1}';
        const second = '{"beta":2}';
        const third = '{"gamma":3}';
        const decoded = decodeFrame(
            bytes(`${first}\n${second}\n${third.slice(0, 6)}`),
        );
        expect(decoded.frames).toEqual([first, second]);
        expect(text(decoded.buffer.toUint8Array())).toBe(third.slice(0, 6));
        expect(decoded.error).toBeNull();
    });

    test('decode_eof_reports_truncated_frame_and_never_emits_the_partial', () => {
        const partial = '{"schema":"uor-r4.studio.agent-event/1"';
        const outcome = decodeEof(bytes(partial));
        expect(outcome).not.toBeNull();
        expect(outcome!.kind).toBe('rejected');
        expect(outcome!.code()).toBe('truncated_frame');
        expect(decodeEof(new Uint8Array())).toBeNull();
    });

    test('decode_frame_preserves_the_complete_prefix_before_a_late_error', () => {
        const first = canonicalJson(
            eventFor({
                taskId: TASK_A,
                runId: RUN_A,
                sequence: 2,
                type: 'assistant.delta',
                data: {
                    message_id: 'msg_prefix',
                    text_delta: 'prefix survives',
                },
            }),
        );
        const invalidTail = new Uint8Array([
            ...bytes(`${first}\n`),
            0xff,
            0x0a,
        ]);
        const decoded = decodeFrame(invalidTail);
        expect(decoded.frames).toEqual([first]);
        expect(decoded.error).toBe('malformed_event');

        const oversizedTail = new Uint8Array([
            ...bytes(`${first}\n`),
            ...bytes('x'.repeat(1024 * 1024 + 1)),
            0x0a,
        ]);
        const limited = decodeFrame(oversizedTail);
        expect(limited.frames).toEqual([first]);
        expect(limited.error).toBe('limit_exceeded');
    });

    test('decode_frame_rejects_invalid_utf8_without_decoding_replacement_text', () => {
        const invalid = new Uint8Array([0x7b, 0xff, 0xfe, 0x0a]);
        const decoded = decodeFrame(invalid);
        expect(decoded.frames).toEqual([]);
        expect(decoded.error).toBe('malformed_event');
    });
});

describe('IngestOutcome', () => {
    test('exposes_the_four_variants_and_a_code_only_for_rejection', () => {
        expect(IngestOutcome.applied().kind).toBe('applied');
        expect(IngestOutcome.duplicate().kind).toBe('duplicate');
        expect(IngestOutcome.rejected('duplicate_event').kind).toBe('rejected');
        expect(IngestOutcome.unknown().kind).toBe('unknown');
        expect(IngestOutcome.applied().code()).toBeNull();
        expect(IngestOutcome.duplicate().code()).toBeNull();
        expect(IngestOutcome.unknown().code()).toBeNull();
        expect(IngestOutcome.rejected('out_of_order').code()).toBe(
            'out_of_order',
        );
        expect(
            IngestOutcome.rejected('out_of_order', true).requiresResync,
        ).toBe(true);
        expect(IngestOutcome.rejected('malformed_event').requiresResync).toBe(
            false,
        );
    });
});

describe('NdjsonIngest hostile frames', () => {
    test('rejects_malformed_frame_without_rendering_anything', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const outcomes = await worker.push(bytes('{"schema":oops}\n'));
        expect(outcomes.map((outcome) => outcome.code())).toEqual([
            'malformed_event',
        ]);
        expect(runOf(store).lastSequence).toBe(1);
        expectNoOutput(store);
    });

    test('rejects_malformed_utf8_bytes_before_json', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const outcomes = await worker.push(
            new Uint8Array([0x7b, 0xff, 0x0a]),
        );
        expect(outcomes.map((outcome) => outcome.code())).toEqual([
            'malformed_event',
        ]);
        expect(runOf(store).lastSequence).toBe(1);
        expectNoOutput(store);
    });

    test('rejects_truncated_frame_at_eof_without_applying_partial_data', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const partial = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.message',
            data: {
                message_id: 'msg_00000000000000a1',
                text: 'forged partial completion',
            },
        });
        const pushed = await worker.push(bytes(canonicalJson(partial)));
        expect(pushed).toEqual([]);
        const outcome = await worker.eof();
        expect(outcome?.code()).toBe('truncated_frame');
        expect(runOf(store).lastSequence).toBe(1);
        expectNoOutput(store);
    });

    test('rejects_sequence_gap_and_requires_snapshot_resync', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const delta = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.delta',
            data: {
                message_id: 'msg_00000000000000a1',
                text_delta: 'trusted delta',
            },
        });
        const applied = await worker.push(bytes(ndjson(delta)));
        expect(applied[0]?.kind).toBe('applied');

        const gap = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 4,
            type: 'assistant.delta',
            data: {
                message_id: 'msg_00000000000000a1',
                text_delta: 'gap text must not render',
            },
        });
        const rejected = await worker.push(bytes(ndjson(gap)));
        expect(rejected[0]?.code()).toBe('out_of_order');
        expect(rejected[0]?.requiresResync).toBe(true);
        expect(runOf(store).lastSequence).toBe(2);
        expect(runOf(store).needsResync).toBe(true);

        const late = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 3,
            type: 'assistant.delta',
            data: {
                message_id: 'msg_00000000000000a1',
                text_delta: 'later event must not apply',
            },
        });
        const halted = await worker.push(bytes(ndjson(late)));
        expect(halted[0]?.code()).toBe('out_of_order');
        expect(runOf(store).lastSequence).toBe(2);
        expect(runOf(store).messages.map((entry) => entry.text)).toEqual([
            'trusted delta',
        ]);
    });

    test('accepts_duplicate_same_digest_without_advancing_the_cursor', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const message = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.message',
            data: {
                message_id: 'msg_00000000000000a1',
                text: 'one honest message',
            },
        });
        const frame = bytes(ndjson(message));
        const first = await worker.push(frame);
        expect(first[0]?.kind).toBe('applied');
        const cursor = store.getRunCursor(TASK_A, RUN_A)!;
        const duplicate = await worker.push(frame);
        expect(duplicate[0]?.kind).toBe('duplicate');
        expect(duplicate[0]?.code()).toBeNull();
        expect(store.getRunCursor(TASK_A, RUN_A)).toEqual(cursor);
        expect(runOf(store).messages.map((entry) => entry.text)).toEqual([
            'one honest message',
        ]);
    });

    test('rejects_duplicate_different_digest_and_stops_the_stream', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const message = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.message',
            data: {
                message_id: 'msg_00000000000000a1',
                text: 'honest message',
            },
        });
        await worker.push(bytes(ndjson(message)));
        const cursor = store.getRunCursor(TASK_A, RUN_A)!;

        const forged = {
            ...message,
            data: {
                message_id: 'msg_00000000000000a1',
                text: 'forged replacement content',
            },
        } as AgentIncomingEvent;
        const rejected = await worker.push(bytes(ndjson(forged)));
        expect(rejected[0]?.code()).toBe('duplicate_event');
        expect(rejected[0]?.requiresResync).toBe(true);
        expect(store.getRunCursor(TASK_A, RUN_A)).toEqual(cursor);

        const next = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 3,
            type: 'assistant.message',
            data: {
                message_id: 'msg_00000000000000a2',
                text: 'stream was stopped',
            },
        });
        const halted = await worker.push(bytes(ndjson(next)));
        expect(halted[0]?.code()).toBe('duplicate_event');
        expect(store.getRunCursor(TASK_A, RUN_A)).toEqual(cursor);
        expect(runOf(store).messages.map((entry) => entry.text)).toEqual([
            'honest message',
        ]);
    });

    test('tolerates_unknown_event_without_rendering_assistant_or_tool_output', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const unknown = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.message.forged',
            data: {
                message_id: 'msg_forged',
                text: 'forged assistant text',
                tool_call_id: 'tool_forged',
                status: 'ok',
                output_digest: ARGUMENTS_DIGEST,
                output_preview: '{"forged":true}',
                artifact_id: 'art_forged',
                completion: true,
            },
        });
        const outcomes = await worker.push(bytes(ndjson(unknown)));
        expect(outcomes[0]?.kind).toBe('unknown');
        expect(outcomes[0]?.code()).toBeNull();
        const run = runOf(store);
        expect(run.unknownEvents).toHaveLength(1);
        expect(run.unknownEvents[0]).toMatchObject({
            type: 'assistant.message.forged',
            sequence: 2,
        });
        expect(run.messages).toEqual([]);
        expect(run.toolCalls).toEqual([]);
        expect(run.changesSnapshots).toEqual([]);
        expect(run.state).toBe('running');
        expect(run.events.map((event) => event.type)).not.toContain(
            'run.finished',
        );
    });

    test('tolerates_known_thinking_and_heartbeat_as_optional_non_text_events', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const thinking = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'thinking.delta',
            data: { text_delta: 'private reasoning' },
        });
        const heartbeat = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 3,
            type: 'heartbeat',
            data: { at_ms: 1_800_000_000_003 },
        });
        const outcomes = await worker.push(
            bytes(`${ndjson(thinking)}${ndjson(heartbeat)}`),
        );
        expect(outcomes.map((outcome) => outcome.kind)).toEqual([
            'applied',
            'applied',
        ]);
        const run = runOf(store);
        expect(run.unknownEvents).toEqual([]);
        expect(run.messages).toEqual([]);
        expect(run.lastSequence).toBe(3);
    });

    test('rejects_stale_generation_without_mutating_a_newer_run', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const cursor = store.getRunCursor(TASK_A, RUN_A)!;
        expect(
            store.verifySnapshot({
                taskId: TASK_A,
                runId: RUN_A,
                generation: cursor.generation,
                lastSequence: cursor.lastSequence,
                lastEventId: cursor.lastEventId!,
                headDigest: cursor.lastDigest!,
            }).ok,
        ).toBe(true);
        await appendStored(
            store,
            createRunResumedEvent({
                projectId: PROJECT_ID,
                taskId: TASK_A,
                runId: RUN_A,
                generation: 2,
                sequence: 2,
                recordedAtMs: 1_800_000_000_002,
                previousGeneration: 1,
                snapshotDigest: cursor.lastDigest!,
            }),
        );
        const worker = workerFor(store);
        const stale = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            generation: 1,
            sequence: 3,
            type: 'assistant.message',
            data: {
                message_id: 'msg_stale',
                text: 'stale generation output',
            },
        });
        const outcomes = await worker.push(bytes(ndjson(stale)));
        expect(outcomes[0]?.code()).toBe('stale_generation');
        expect(outcomes[0]?.requiresResync).toBe(true);
        const run = runOf(store);
        expect(run.generation).toBe(2);
        expect(run.lastSequence).toBe(2);
        expect(run.messages).toEqual([]);
    });

    test('rejects_cross_task_event_before_it_can_create_another_task', async () => {
        const store = createAgentStore();
        await seedRun(store);
        await seedRun(store, TASK_B, RUN_B);
        const worker = workerFor(store);
        const crossTask = eventFor({
            taskId: TASK_B,
            runId: RUN_B,
            sequence: 2,
            type: 'assistant.message',
            data: {
                message_id: 'msg_cross',
                text: 'cross task output',
            },
        });
        const outcomes = await worker.push(bytes(ndjson(crossTask)));
        expect(outcomes[0]?.code()).toBe('cross_task_event');
        expect(runOf(store, TASK_A, RUN_A).messages).toEqual([]);
        expect(runOf(store, TASK_B, RUN_B).messages).toEqual([]);
        expect(runOf(store, TASK_B, RUN_B).lastSequence).toBe(1);
    });

    test('rejects_cross_run_queued_event_without_creating_a_second_run', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const foreignRun = eventFor({
            taskId: TASK_A,
            runId: RUN_C,
            sequence: 0,
            type: 'run.queued',
            data: { queued_at_ms: 1_800_000_000_010 },
        });
        const outcomes = await worker.push(bytes(ndjson(foreignRun)));
        expect(outcomes[0]?.code()).toBe('out_of_order');
        const task = store
            .snapshot()
            .tasks.find((candidate) => candidate.taskId === TASK_A)!;
        expect(task.runs.map((run) => run.runId)).toEqual([RUN_A]);
    });

    test('hostile_input_for_one_task_never_mutates_another_live_task', async () => {
        const store = createAgentStore();
        await seedRun(store);
        await seedRun(store, TASK_B, RUN_B);
        const workerA = workerFor(store);
        const workerB = workerFor(store, TASK_B, RUN_B);
        const cursorB = store.getRunCursor(TASK_B, RUN_B)!;

        const malformed = await workerA.push(bytes('{not-json}\n'));
        expect(malformed[0]?.code()).toBe('malformed_event');
        const unknown = eventFor({
            taskId: TASK_B,
            runId: RUN_B,
            sequence: 2,
            type: 'tool.completed.forged',
            data: {
                tool_call_id: 'tool_forged',
                output_digest: ARGUMENTS_DIGEST,
                output_preview: '{"forged":true}',
            },
        });
        const crossWorker = workerFor(store);
        const cross = await crossWorker.push(bytes(ndjson(unknown)));
        expect(cross[0]?.code()).toBe('cross_task_event');

        expect(store.getRunCursor(TASK_B, RUN_B)).toEqual(cursorB);
        expect(runOf(store, TASK_B, RUN_B).messages).toEqual([]);
        expect(runOf(store, TASK_B, RUN_B).toolCalls).toEqual([]);

        const honestB = eventFor({
            taskId: TASK_B,
            runId: RUN_B,
            sequence: 2,
            type: 'assistant.message',
            data: {
                message_id: 'msg_b',
                text: 'task B remains independent',
            },
        });
        const applied = await workerB.push(bytes(ndjson(honestB)));
        expect(applied[0]?.kind).toBe('applied');
        expect(runOf(store, TASK_A, RUN_A).messages).toEqual([]);
    });
});

describe('NdjsonIngest fix round 1 buffering and halt bounds', () => {
    test('large_frame_fed_one_byte_at_a_time_keeps_byte_copies_linear', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const message = eventFor({
            taskId: TASK_A,
            runId: RUN_A,
            sequence: 2,
            type: 'assistant.message',
            data: {
                message_id: 'msg_large',
                text: 'x'.repeat(64 * 1024),
            },
        });
        const encoded = bytes(ndjson(message));

        for (let index = 0; index < encoded.length; index += 1) {
            await worker.push(encoded.subarray(index, index + 1));
        }
        expect(runOf(store).messages.map((entry) => entry.text)).toEqual([
            'x'.repeat(64 * 1024),
        ]);
        expect(
            worker.copyBytes,
            `copied ${worker.copyBytes} bytes for a ${encoded.length}-byte frame`,
        ).toBeLessThan(encoded.length * 8);
    });

    test('compact_copies_are_counted_in_the_structural_bound', () => {
        const buffer = new NdjsonBuffer();
        buffer.append(new TextEncoder().encode('abcdef'));
        const before = buffer.copyBytes;
        buffer.compact(2);
        expect(buffer.length).toBe(4);
        expect(buffer.copyBytes - before).toBe(4);
    });

    test('one_mib_buffer_appends_one_byte_at_a_time_stay_amortized', () => {
        const buffer = new NdjsonBuffer();
        const total = 1024 * 1024;
        const byte = new Uint8Array([0x61]);
        for (let index = 0; index < total; index += 1) {
            buffer.append(byte);
        }
        expect(buffer.length).toBe(total);
        expect(buffer.capacity).toBeLessThanOrEqual(total * 2);
        expect(
            buffer.copyBytes,
            `copied ${buffer.copyBytes} bytes for ${total} one-byte appends`,
        ).toBeLessThan(total * 4);
    });

    test('newline_flood_is_rejected_before_a_frame_per_newline_is_materialized', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const outcomes = await worker.push(bytes('\n'.repeat(1_000_000)));
        expect(outcomes).toHaveLength(1);
        expect(outcomes[0]?.code()).toBe('limit_exceeded');
        expect(runOf(store).lastSequence).toBe(1);
        expect(worker.bufferedBytes).toHaveLength(0);
    });

    test('oversized_partial_is_rejected_without_waiting_for_eof', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        const outcomes = await worker.push(
            bytes('x'.repeat(1024 * 1024 + 1)),
        );
        expect(outcomes).toHaveLength(1);
        expect(outcomes[0]?.code()).toBe('limit_exceeded');
        expect(worker.bufferedLength).toBe(0);
        expect(runOf(store).lastSequence).toBe(1);
    });

    test('more_than_64_complete_frames_in_one_push_apply_none', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        let payload = '';
        for (let sequence = 2; sequence < 67; sequence += 1) {
            payload += ndjson(
                eventFor({
                    taskId: TASK_A,
                    runId: RUN_A,
                    sequence,
                    type: 'assistant.delta',
                    data: {
                        message_id: 'msg_flood',
                        text_delta: `frame ${sequence}`,
                    },
                }),
            );
        }
        const outcomes = await worker.push(bytes(payload));
        expect(outcomes).toHaveLength(1);
        expect(outcomes[0]?.code()).toBe('limit_exceeded');
        expect(runOf(store).lastSequence).toBe(1);
        expect(runOf(store).messages).toEqual([]);
    });

    test('stops_after_the_first_halt_without_decoding_later_frames', async () => {
        const store = createAgentStore();
        await seedRun(store);
        const worker = workerFor(store);
        let payload = '{"schema":oops}\n';
        for (let sequence = 2; sequence < 12; sequence += 1) {
            payload += ndjson(
                eventFor({
                    taskId: TASK_A,
                    runId: RUN_A,
                    sequence,
                    type: 'assistant.delta',
                    data: {
                        message_id: 'msg_after_halt',
                        text_delta: 'must not decode',
                    },
                }),
            );
        }
        const outcomes = await worker.push(bytes(payload));
        expect(outcomes).toHaveLength(1);
        expect(outcomes[0]?.code()).toBe('malformed_event');
        expect(runOf(store).lastSequence).toBe(1);
        expect(runOf(store).messages).toEqual([]);
    });

    test('cross_task_and_terminal_out_of_order_do_not_claim_resync', async () => {
        expect(IngestOutcome.rejected('cross_task_event').requiresResync).toBe(
            false,
        );

        const store = createAgentStore();
        await seedRun(store);
        await appendStored(
            store,
            eventFor({
                taskId: TASK_A,
                runId: RUN_A,
                sequence: 2,
                type: 'run.finished',
                data: {
                    status: 'finished',
                    finished_at_ms: 1_800_000_000_002,
                    duration_ms: 2,
                },
            }),
        );
        const worker = workerFor(store);
        const afterTerminal = await worker.push(
            bytes(
                ndjson(
                    eventFor({
                        taskId: TASK_A,
                        runId: RUN_A,
                        sequence: 3,
                        type: 'assistant.message',
                        data: {
                            message_id: 'msg_after_terminal',
                            text: 'no append after terminal',
                        },
                    }),
                ),
            ),
        );
        expect(afterTerminal[0]?.code()).toBe('out_of_order');
        expect(afterTerminal[0]?.requiresResync).toBe(false);
    });
});
