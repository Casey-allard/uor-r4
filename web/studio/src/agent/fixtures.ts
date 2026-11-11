import {
    AGENT_EVENT_SCHEMA,
    type AgentEvent,
} from './generated/agentContract.js';

export type RunQueuedEvent = Extract<AgentEvent, { type: 'run.queued' }>;
export type RunStartedEvent = Extract<AgentEvent, { type: 'run.started' }>;
export type RunResumedEvent = Extract<AgentEvent, { type: 'run.resumed' }>;

export interface FixtureEventIdentity {
    projectId: string;
    taskId: string;
    runId: string;
    generation: number;
    recordedAtMs: number;
}

export function fixtureEventId(
    runId: string,
    generation: number,
    sequence: number,
): string {
    return `evt_${runId}_${generation}_${sequence
        .toString()
        .padStart(12, '0')}`;
}

export function createRunQueuedEvent(
    input: FixtureEventIdentity & { queuedAtMs?: number },
): RunQueuedEvent {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: fixtureEventId(input.runId, input.generation, 0),
        project_id: input.projectId,
        task_id: input.taskId,
        run_id: input.runId,
        generation: input.generation,
        sequence: 0,
        recorded_at_ms: input.recordedAtMs,
        type: 'run.queued',
        data: {
            queued_at_ms: input.queuedAtMs ?? input.recordedAtMs,
        },
    };
}

export function createRunStartedEvent(
    input: FixtureEventIdentity & {
        startedAtMs?: number;
        mode?: 'plan' | 'agent';
        retryOfRunId?: string | null;
        modelId?: string;
        modelKind?: string;
        environmentKind?: string;
    },
): RunStartedEvent {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: fixtureEventId(input.runId, input.generation, 1),
        project_id: input.projectId,
        task_id: input.taskId,
        run_id: input.runId,
        generation: input.generation,
        sequence: 1,
        recorded_at_ms: input.recordedAtMs,
        type: 'run.started',
        data: {
            started_at_ms: input.startedAtMs ?? input.recordedAtMs,
            model: {
                id: input.modelId ?? 'replay',
                kind: input.modelKind ?? 'fixture',
            },
            mode: input.mode ?? 'agent',
            environment: {
                kind: input.environmentKind ?? 'browser-fixture',
            },
            retry_of_run_id: input.retryOfRunId ?? null,
        },
    };
}

export function createRunResumedEvent(
    input: FixtureEventIdentity & {
        sequence: number;
        previousGeneration: number;
        snapshotDigest: string;
        resumedAtMs?: number;
    },
): RunResumedEvent {
    return {
        schema: AGENT_EVENT_SCHEMA,
        event_id: fixtureEventId(input.runId, input.generation, input.sequence),
        project_id: input.projectId,
        task_id: input.taskId,
        run_id: input.runId,
        generation: input.generation,
        sequence: input.sequence,
        recorded_at_ms: input.recordedAtMs,
        type: 'run.resumed',
        data: {
            resumed_at_ms: input.resumedAtMs ?? input.recordedAtMs,
            previous_generation: input.previousGeneration,
            snapshot_digest: input.snapshotDigest,
        },
    };
}
