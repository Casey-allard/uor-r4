import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AgentClient, AgentClientError } from '../agent/client.js';
import type { StopReceipt } from '../agent/client.js';
import {
    createAgentStore,
    type AgentIncomingEvent,
    type AgentStore,
    type TaskView,
} from '../state/agent.js';
import { TaskDetail } from './TaskDetail.js';

const PROJECT_ID = 'prj_0000000000000001';
const TASK_ID = 'tsk_0000000000000001';
const RUN_ID = 'run_0000000000000001';
const MESSAGE_ID = 'msg_1f2e3d4c5b6a7988';
const DIGEST = `sha256:${'a'.repeat(64)}`;

function event(sequence: number, type: string, data: Record<string, unknown>, overrides: { runId?: string; generation?: number } = {}): AgentIncomingEvent {
    const runId = overrides.runId ?? RUN_ID;
    const generation = overrides.generation ?? 1;
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: `evt_${runId}_${generation}_${sequence.toString().padStart(12, '0')}`,
        project_id: PROJECT_ID,
        task_id: TASK_ID,
        run_id: runId,
        generation,
        sequence,
        recorded_at_ms: 1_794_283_200_000 + sequence,
        type,
        data,
    } as unknown as AgentIncomingEvent;
}

function queued(sequence = 0): AgentIncomingEvent {
    return event(sequence, 'run.queued', { queued_at_ms: 1_794_283_200_000 + sequence });
}

function started(sequence = 1): AgentIncomingEvent {
    return event(sequence, 'run.started', {
        started_at_ms: 1_794_283_200_001,
        model: { id: 'synthetic-replay', kind: 'synthetic-replay' },
        mode: 'agent',
        environment: { kind: 'browser-runtime' },
        retry_of_run_id: null,
    });
}

interface Fixture {
    store: AgentStore;
    task: TaskView;
    client: AgentClient;
}

function fixture(events: AgentIncomingEvent[] = [queued(), started()]): Fixture {
    const store = createAgentStore();
    for (const entry of events) {
        const result = store.apply(entry);
        expect(result.status, `${entry.type} at ${entry.sequence}: ${result.message ?? ''}`).toBe('applied');
    }
    const task = store.snapshot().tasks[0]!;
    const client = new AgentClient({ store });
    vi.spyOn(client, 'hostCapabilities').mockResolvedValue({
        tree: false,
        changes: false,
        readArtifact: false,
        terminal: false,
        git: false,
        mcp: false,
        approvals: false,
    });
    return { store, task, client };
}

function taskNow(fixtureValue: Fixture): TaskView {
    return fixtureValue.store.snapshot().tasks[0]!;
}

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

async function renderDetail(value: Fixture, props: { onBack?(): void; onOpenSurface?(surface: 'changes' | 'terminal'): void } = {}) {
    const onBack = props.onBack ?? vi.fn();
    const onOpenSurface = props.onOpenSurface ?? vi.fn();
    await act(async () => {
        render(<TaskDetail task={value.task} client={value.client} onBack={onBack} onOpenSurface={onOpenSurface} />, host);
        await Promise.resolve();
    });
    return { onBack, onOpenSurface };
}

function buttonNamed(name: string): HTMLButtonElement | null {
    return Array.from(host.querySelectorAll<HTMLButtonElement>('button')).find((node) => (node.textContent ?? '').trim() === name) ?? null;
}

const detailEvents = (): AgentIncomingEvent[] => [
    queued(0),
    started(1),
    event(2, 'assistant.delta', { message_id: MESSAGE_ID, text_delta: 'Hello ' }),
    event(3, 'assistant.delta', { message_id: MESSAGE_ID, text_delta: 'from replay' }),
    event(4, 'assistant.message', { message_id: MESSAGE_ID, text: 'Hello from replay' }),
    event(5, 'tool.requested', { tool_call_id: 'tool_1', tool: 'write_file', arguments_digest: DIGEST, arguments_preview: '{"path":"notes.md"}' }),
    event(6, 'tool.started', { tool_call_id: 'tool_1' }),
    event(7, 'tool.completed', { tool_call_id: 'tool_1', status: 'ok', output_digest: DIGEST, output_preview: 'wrote notes', truncated: false }),
    event(8, 'host.instruction', { text: '<script>hostile</script>', status: 'completed', tool_call_id: 'tool_hostile' }),
    event(9, 'usage.updated', { input_tokens: 12, output_tokens: 34, model_calls: 2 }),
    event(10, 'plan.updated', {
        plan_id: 'plan_1',
        title: 'Replay plan',
        steps: [
            { step_id: 'step_1', label: 'Read request', state: 'done' },
            { step_id: 'step_2', label: 'Write answer', state: 'active' },
        ],
    }),
    event(11, 'run.finished', { status: 'finished', finished_at_ms: 1_794_283_200_011, duration_ms: 11 }),
];

describe('TaskDetail', () => {
    it('collapses assistant deltas into one message', async () => {
        await renderDetail(fixture(detailEvents()));
        const messages = host.querySelectorAll('[data-timeline-kind="assistant"]');
        expect(messages).toHaveLength(1);
        expect(messages[0]!.querySelector('.message-content')?.textContent).toBe('Hello from replay');
        expect(host.textContent).not.toContain('Hello from replayHello');
    });

    it('pairs a tool request and completion into one card without a second completion row', async () => {
        await renderDetail(fixture(detailEvents()));
        const tools = host.querySelectorAll('[data-timeline-kind="tool"]');
        expect(tools).toHaveLength(1);
        const card = tools[0]!;
        expect(card.textContent).toContain('write_file');
        expect(card.textContent).toContain('{"path":"notes.md"}');
        expect(card.textContent).toContain('wrote notes');
        expect(card.getAttribute('data-tool-status')).toBe('completed');
        expect(host.querySelectorAll('[data-tool-call-id="tool_1"]')).toHaveLength(1);
    });

    it('preserves unknown events without rendering them as assistant text or tool completion', async () => {
        await renderDetail(fixture(detailEvents()));
        const unknown = host.querySelector('[data-timeline-kind="unknown"]')!;
        expect(unknown.textContent).toContain('host.instruction');
        expect(unknown.textContent).not.toContain('<script>');
        expect(host.textContent).not.toContain('hostile');
        expect(host.querySelector('[data-tool-call-id="tool_hostile"]')).toBeNull();
        expect(host.querySelectorAll('[data-timeline-kind="assistant"]')).toHaveLength(1);
        expect(host.querySelectorAll('[data-timeline-kind="tool"]')).toHaveLength(1);
    });

    it('shows error code and message without inventing retry for a finished run', async () => {
        const events = detailEvents();
        events.splice(9, 0, event(9, 'error', {
            code: 'malformed_event',
            message: 'The event stream contained an invalid event. The transcript was not changed.',
            retryable: false,
        }));
        const renumbered = events.map((entry, index) => index <= 9 ? entry : ({
            ...entry,
            sequence: entry.sequence + 1,
            recorded_at_ms: entry.recorded_at_ms + 1,
            event_id: `evt_${RUN_ID}_1_${(entry.sequence + 1).toString().padStart(12, '0')}`,
        } as AgentIncomingEvent));
        await renderDetail(fixture(renumbered));
        const row = host.querySelector('[data-task-error="run"]')!;
        expect(row.textContent).toContain('malformed_event');
        expect(row.textContent).toContain('The event stream contained an invalid event. The transcript was not changed.');
        expect(buttonNamed('Retry')).toBeNull();
    });

    it('shows usage only when a usage event was emitted', async () => {
        await renderDetail(fixture([queued(), started(), event(2, 'run.finished', { status: 'finished', finished_at_ms: 2, duration_ms: 2 })]));
        expect(host.querySelector('[data-timeline-kind="usage"]')).toBeNull();
        await renderDetail(fixture(detailEvents()));
        const usage = host.querySelector('[data-timeline-kind="usage"]')!;
        expect(usage.textContent).toContain('12 input tokens');
        expect(usage.textContent).toContain('34 output tokens');
        expect(usage.textContent).toContain('2 model calls');
    });

    it('renders a plan card only from an actual plan.updated event', async () => {
        await renderDetail(fixture([queued(), started(), event(2, 'run.finished', { status: 'finished', finished_at_ms: 2, duration_ms: 2 })]));
        expect(host.querySelector('[data-task-plan]')).toBeNull();
        await renderDetail(fixture(detailEvents()));
        const plan = host.querySelector('[data-task-plan]')!;
        expect(plan.textContent).toContain('Replay plan');
        expect(plan.textContent).toContain('Read request');
        expect(plan.textContent).toContain('Write answer');
        expect(plan.querySelector('[data-step-state="done"]')).not.toBeNull();
        expect(plan.querySelector('[data-step-state="active"]')).not.toBeNull();
    });

    it('shows status, prompt, project, repo, branch, environment and model in the header', async () => {
        await renderDetail(fixture(detailEvents()));
        const header = host.querySelector('[data-task-detail-header]')!;
        expect(header.textContent).toContain('Finished');
        expect(header.textContent).toContain('Title');
        expect(header.textContent).toContain('Starting prompt not recorded');
        expect(header.textContent).toContain(TASK_ID);
        expect(header.textContent).toContain(PROJECT_ID);
        expect(header.textContent).toContain('browser-runtime');
        expect(header.textContent).toContain('synthetic-replay');
        expect(host.querySelectorAll('h1')).toHaveLength(1);
    });

    it('labels replay output as synthetic', async () => {
        await renderDetail(fixture(detailEvents()));
        expect(host.textContent).toContain('Synthetic replay');
        expect(host.textContent).toContain('not an R4 model');
    });

    it('shows tool, scope, expiry and static host identity on an approval card and disables actions', async () => {
        const value = fixture([
            queued(0),
            started(1),
            event(2, 'tool.requested', { tool_call_id: 'tool_1', tool: 'write_file', arguments_digest: DIGEST, arguments_preview: '{"path":"notes.md"}' }),
            event(3, 'tool.approval_required', {
                approval_id: 'apr_1111111111111111',
                tool_call_id: 'tool_1',
                tool: 'write_file',
                scope: 'tool:write_file',
                request_digest: DIGEST,
                expires_at_ms: Date.UTC(2026, 10, 9, 12, 10, 0),
            }),
        ]);
        await renderDetail(value);
        const card = host.querySelector('[data-approval-id="apr_1111111111111111"]')!;
        expect(card.textContent).toContain('write_file');
        expect(card.textContent).toContain('tool:write_file');
        expect(card.textContent).toContain('2026-11-09T12:10:00.000Z');
        expect(card.textContent).toContain('Host: not connected');
        expect(card.textContent).toContain('This action needs host approval. Approve it in the local companion.');
        expect(buttonNamed('Approve')?.disabled).toBe(true);
        expect(buttonNamed('Deny')?.disabled).toBe(true);
        expect(host.textContent).toContain('Waiting for approval');
    });

    it('shows the bounded background hold as an optional service without a continuation control', async () => {
        const value = fixture([queued(0), started(1)]);
        const heldTask: TaskView = {
            ...value.task,
            state: 'waiting_for_background',
            hold: {
                reason: 'waiting_for_background',
                run_id: RUN_ID,
                tool_call_id: 'tool_1',
            },
        };
        await act(async () => {
            render(<TaskDetail task={heldTask} client={value.client} onBack={vi.fn()} onOpenSurface={vi.fn()} />, host);
            await Promise.resolve();
        });
        expect(host.querySelector('[data-static-background]')?.textContent).toBe(
            'Background and cross-device tasks require an optional service. Closing this tab stops local work.',
        );
        expect(host.textContent).not.toMatch(/continue in background|keep running after/i);
    });

    it('never uses a page boolean to enable approval authority', async () => {
        const value = fixture([
            queued(0),
            started(1),
            event(2, 'tool.approval_required', {
                approval_id: 'apr_1111111111111111',
                tool_call_id: 'tool_1',
                tool: 'write_file',
                scope: 'tool:write_file',
                request_digest: DIGEST,
                expires_at_ms: 1_794_283_800_000,
            }),
        ]);
        value.client.hostCapabilities = vi.fn().mockResolvedValue({
            tree: true, changes: true, readArtifact: true, terminal: true,
            git: true, mcp: true, approvals: false,
        });
        await renderDetail(value);
        expect(buttonNamed('Approve')?.disabled).toBe(true);
    });

    it('shows stop only for an active run and cancelling until the host acknowledgement', async () => {
        const running = fixture([queued(), started()]);
        let resolveStop!: (receipt: StopReceipt) => void;
        const stop = vi.spyOn(running.client, 'stop').mockReturnValue(new Promise((resolve) => { resolveStop = resolve; }));
        await renderDetail(running);
        const activeStop = buttonNamed('Stop');
        expect(activeStop).not.toBeNull();
        await act(async () => { activeStop!.click(); await Promise.resolve(); });
        expect(stop).toHaveBeenCalledWith({ taskId: TASK_ID, runId: RUN_ID });
        expect(buttonNamed('Cancelling…')?.disabled).toBe(true);
        expect(buttonNamed('Stop')).toBeNull();

        const cancelling = running.store.apply(event(2, 'run.status', { status: 'cancelling' }));
        expect(cancelling.status).toBe('applied');
        await renderDetail({ ...running, task: taskNow(running) });
        expect(buttonNamed('Cancelling…')?.disabled).toBe(true);

        await act(async () => { resolveStop({ taskId: TASK_ID, runId: RUN_ID, acknowledged: true }); await Promise.resolve(); });
        running.store.apply(event(3, 'run.cancelled', { safe_boundary: true, finished_at_ms: 1_794_283_200_003 }));
        await renderDetail({ ...running, task: taskNow(running) });
        expect(buttonNamed('Stop')).toBeNull();
        expect(buttonNamed('Cancelling…')).toBeNull();
        expect(buttonNamed('Pause')).toBeNull();
    });

    it('offers retry only for failed-retryable or cancelled runs', async () => {
        await renderDetail(fixture([
            queued(), started(),
            event(2, 'run.failed', { code: 'model_unavailable', message: 'retryable failure', retryable: true }),
        ]));
        expect(buttonNamed('Retry')).not.toBeNull();

        await renderDetail(fixture([
            queued(), started(),
            event(2, 'run.failed', { code: 'invalid_event', message: 'terminal failure', retryable: false }),
        ]));
        expect(buttonNamed('Retry')).toBeNull();

        await renderDetail(fixture([
            queued(), started(),
            event(2, 'run.cancelled', { safe_boundary: true, finished_at_ms: 2 }),
        ]));
        expect(buttonNamed('Retry')).not.toBeNull();

        await renderDetail(fixture([
            queued(), started(),
            event(2, 'run.finished', { status: 'finished', finished_at_ms: 2, duration_ms: 2 }),
        ]));
        expect(buttonNamed('Retry')).toBeNull();
    });

    it('calls the real retry path and reports a refusal without hiding it', async () => {
        const value = fixture([
            queued(), started(),
            event(2, 'run.failed', { code: 'model_unavailable', message: 'retryable failure', retryable: true }),
        ]);
        const retry = vi.spyOn(value.client, 'retry').mockRejectedValue(
            new AgentClientError('invalid_arguments', 'the original run prompt is unavailable'),
        );
        await renderDetail(value);
        await act(async () => { buttonNamed('Retry')!.click(); await Promise.resolve(); });
        expect(retry).toHaveBeenCalledWith({ taskId: TASK_ID, runId: RUN_ID });
        await vi.waitFor(() => {
            expect(host.querySelector('[data-task-error="action"]')?.textContent).toContain('invalid_arguments');
        });
        expect(host.querySelector('[data-task-error="action"]')?.textContent).toContain('the original run prompt is unavailable');
    });

    it('hides an uncompleted later tool when the run is terminal', async () => {
        await renderDetail(fixture([
            queued(), started(),
            event(2, 'tool.requested', { tool_call_id: 'tool_late', tool: 'write_file', arguments_digest: DIGEST, arguments_preview: '{}' }),
            event(3, 'run.cancelled', { safe_boundary: true, finished_at_ms: 3 }),
        ]));
        expect(host.querySelector('[data-tool-call-id="tool_late"]')).toBeNull();
        expect(host.querySelector('[data-timeline-kind="tool"]')).toBeNull();
    });

    it('shows a run failure with code, message and retry policy', async () => {
        await renderDetail(fixture([
            queued(), started(),
            event(2, 'run.failed', { code: 'model_unavailable', message: 'synthetic retryable failure', retryable: true }),
        ]));
        const row = host.querySelector('[data-task-error="run"]')!;
        expect(row.textContent).toContain('model_unavailable');
        expect(row.textContent).toContain('synthetic retryable failure');
        expect(buttonNamed('Retry')).not.toBeNull();
    });

    it('keeps message_not_delivered on the affected message instead of a run-level error', async () => {
        await renderDetail(fixture([
            queued(0),
            started(1),
            event(2, 'message.queued', { message_id: MESSAGE_ID, delivery: 'queue', boundary: 'next_turn', text_preview: 'queued follow-up' }),
            event(3, 'error', { code: 'message_not_delivered', message: `message ${MESSAGE_ID} was not delivered (cancelled)`, retryable: false }),
            event(4, 'run.cancelled', { safe_boundary: true, finished_at_ms: 4 }),
        ]));
        expect(host.querySelector('[data-task-error="run"]')).toBeNull();
        const delivery = host.querySelector('[data-timeline-kind="delivery"]')!;
        expect(delivery.textContent).toContain(MESSAGE_ID);
        expect(delivery.textContent).toContain('was not delivered (cancelled)');
        expect(delivery.querySelector('button')).toBeNull();
        expect(buttonNamed('Retry')).not.toBeNull();
    });

    it('renders no Restore Checkpoint control', async () => {
        await renderDetail(fixture(detailEvents()));
        const names = Array.from(host.querySelectorAll<HTMLElement>('button, a, summary, [role="tab"]'))
            .map((node) => (node.textContent ?? '').trim().toLowerCase());
        expect(names.some((name) => name.includes('restore checkpoint'))).toBe(false);
        expect(names.some((name) => name.includes('checkpoint'))).toBe(false);
    });

    it('renders no Pause, Delete, Logs tab or per-task cost controls', async () => {
        await renderDetail(fixture(detailEvents()));
        const names = Array.from(host.querySelectorAll<HTMLElement>('button, a, summary, [role="tab"]'))
            .map((node) => (node.textContent ?? '').trim().toLowerCase());
        for (const forbidden of ['pause', 'delete', 'delete task', 'logs', 'log', 'cost']) {
            expect(names.some((name) => name === forbidden), `unexpected ${forbidden} control`).toBe(false);
        }
        expect(host.textContent).not.toMatch(/\$\s?\d/);
    });

    it('wires back and work-surface callbacks', async () => {
        const value = fixture(detailEvents());
        const { onBack, onOpenSurface } = await renderDetail(value);
        await act(async () => { buttonNamed('Back to tasks')!.click(); });
        expect(onBack).toHaveBeenCalledOnce();
        await act(async () => { buttonNamed('Open changes')!.click(); });
        expect(onOpenSurface).toHaveBeenCalledWith('changes');
        await act(async () => { buttonNamed('Open terminal')!.click(); });
        expect(onOpenSurface).toHaveBeenCalledWith('terminal');
    });

    it('renders a persisted cancelling projection without an enabled stop', async () => {
        const value = fixture([
            queued(),
            started(),
            event(2, 'run.status', { status: 'cancelling' }),
        ]);
        const projected = value.task.runs[0]!;
        expect(projected.state).toBe('running');
        expect(projected.statusProjection).toBe('cancelling');
        await renderDetail(value);
        expect(host.querySelector('.run-status-label')?.textContent).toBe('Cancelling');
        const stop = host.querySelector<HTMLButtonElement>('.run-stop');
        expect(stop === null || stop.disabled).toBe(true);
        expect(host.querySelector('.run-progress')).toBeNull();
    });

    it('labels the task title honestly and does not invent a starting prompt', async () => {
        const value = fixture(detailEvents());
        value.store.upsertTask({
            taskId: TASK_ID,
            projectId: PROJECT_ID,
            title: 'Client resume fixture',
            createdAtMs: 1_794_283_200_000,
            latestRunId: RUN_ID,
        });
        await renderDetail({ ...value, task: taskNow(value) });
        const header = host.querySelector('[data-task-detail-header]')!;
        const labels = Array.from(header.querySelectorAll('dt')).map((node) => node.textContent);
        expect(labels).toContain('Title');
        expect(labels).not.toContain('Prompt');
        expect(labels).toContain('Starting prompt');
        expect(header.textContent).toContain('Client resume fixture');
        expect(header.textContent).toContain('Starting prompt not recorded');
        expect(header.textContent).not.toContain('resume fixture prompt');
    });

    it('tolerates an approval expiry outside the JavaScript Date range', async () => {
        const value = fixture([
            queued(0),
            started(1),
            event(2, 'tool.approval_required', {
                approval_id: 'apr_2222222222222222',
                tool_call_id: 'tool_1',
                tool: 'write_file',
                scope: 'tool:write_file',
                request_digest: DIGEST,
                expires_at_ms: Number.MAX_SAFE_INTEGER,
            }),
        ]);
        await renderDetail(value);
        const card = host.querySelector('[data-approval-id="apr_2222222222222222"]')!;
        expect(card).not.toBeNull();
        expect(card.textContent).toContain('Not recorded');
        expect(card.querySelector('time')).toBeNull();
        expect(host.textContent).not.toContain('RangeError');
    });

    it('keeps the approval heading directly under the task heading when no plan exists', async () => {
        const value = fixture([
            queued(0),
            started(1),
            event(2, 'tool.approval_required', {
                approval_id: 'apr_3333333333333333',
                tool_call_id: 'tool_1',
                tool: 'write_file',
                scope: 'tool:write_file',
                request_digest: DIGEST,
                expires_at_ms: 1_794_283_800_000,
            }),
        ]);
        await renderDetail(value);
        const levels = Array.from(host.querySelectorAll<HTMLElement>('h1, h2, h3, h4, h5, h6'))
            .map((node) => node.tagName);
        expect(levels[0]).toBe('H1');
        expect(levels[1]).toBe('H2');
        expect(levels).not.toContain('H3');
        expect(host.querySelector('[data-approval-id="apr_3333333333333333"] h2')?.textContent).toBe('Approval required');
    });

    it('moves focus to the detail heading when the task detail opens', async () => {
        await renderDetail(fixture(detailEvents()));
        const heading = host.querySelector<HTMLHeadingElement>('.task-detail-title')!;
        expect(heading.getAttribute('tabindex')).toBe('-1');
        expect(document.activeElement).toBe(heading);
    });

    it('does not treat a terminal run.status projection as an authoritative terminal', async () => {
        const value = fixture([
            queued(),
            started(),
            event(2, 'run.status', { status: 'finished' }),
        ]);
        expect(value.task.runs[0]?.state).toBe('running');
        expect(value.task.runs[0]?.statusProjection).toBe('finished');
        await renderDetail(value);
        expect(host.querySelector('.run-status-label')?.textContent).toBe('Running');
        expect(host.querySelector('.run-status')?.textContent).not.toContain('Finished');
        const stop = host.querySelector<HTMLButtonElement>('.run-stop');
        expect(stop).not.toBeNull();
        expect(stop!.disabled).toBe(false);
        expect(buttonNamed('Retry')).toBeNull();
    });
});

describe('TaskDetail restored and archived honesty', () => {
    it('does_not_show_the_background_service_copy_for_an_archived_hold', async () => {
        const value = fixture([queued(0), started(1)]);
        const archived: TaskView = {
            ...value.task,
            archived: true,
            state: 'archived',
            hold: {
                reason: 'waiting_for_background',
                run_id: RUN_ID,
                tool_call_id: 'tool_1',
            },
        };
        await act(async () => {
            render(<TaskDetail task={archived} client={value.client} onBack={vi.fn()} onOpenSurface={vi.fn()} />, host);
            await Promise.resolve();
        });
        expect(host.querySelector('[data-static-background]')).toBeNull();
    });

    it('disables_a_restored_approval_when_the_page_has_no_active_owner', async () => {
        const value = fixture([
            queued(0),
            started(1),
            event(2, 'tool.requested', {
                tool_call_id: 'tool_1',
                tool: 'write_file',
                arguments_digest: DIGEST,
                arguments_preview: '{"path":"notes.md"}',
            }),
            event(3, 'tool.approval_required', {
                approval_id: 'apr_1111111111111111',
                tool_call_id: 'tool_1',
                tool: 'write_file',
                scope: 'tool:write_file',
                request_digest: DIGEST,
                expires_at_ms: 1_900_000_000_000,
            }),
        ]);
        vi.spyOn(value.client, 'hostCapabilities').mockResolvedValue({
            tree: false,
            changes: false,
            readArtifact: false,
            terminal: false,
            git: false,
            mcp: false,
            approvals: true,
        });
        await renderDetail(value);
        expect(buttonNamed('Approve')?.disabled).toBe(true);
        expect(buttonNamed('Deny')?.disabled).toBe(true);
        expect(host.textContent).toContain('no active run owner');
        expect(host.textContent).toContain('Stop the restored run before starting a new run');
        expect(host.textContent).not.toContain('approval is not pending');
    });
});

describe('TaskDetail new run path', () => {
    it('does_not_offer_a_new_run_while_a_background_hold_exists', async () => {
        const finished = fixture(detailEvents());
        const held: TaskView = {
            ...finished.task,
            state: 'waiting_for_background',
            hold: {
                reason: 'waiting_for_background',
                run_id: RUN_ID,
                tool_call_id: 'tool_1',
            },
        };
        await act(async () => {
            render(
                <TaskDetail
                    task={held}
                    client={finished.client}
                    onBack={vi.fn()}
                    onOpenSurface={vi.fn()}
                    onNewRun={vi.fn()}
                />,
                host,
            );
            await Promise.resolve();
        });
        expect(buttonNamed('New run')).toBeNull();
    });

    it('offers_a_new_run_for_a_terminal_task_and_never_for_an_active_run', async () => {
        const finished = fixture(detailEvents());
        const onNewRun = vi.fn();
        await act(async () => {
            render(
                <TaskDetail
                    task={finished.task}
                    client={finished.client}
                    onBack={vi.fn()}
                    onOpenSurface={vi.fn()}
                    onNewRun={onNewRun}
                />,
                host,
            );
            await Promise.resolve();
        });
        const action = buttonNamed('New run');
        expect(action).not.toBeNull();
        expect(action!.disabled).toBe(false);
        act(() => { action!.click(); });
        expect(onNewRun).toHaveBeenCalledTimes(1);

        render(null, host);
        const running = fixture([queued(0), started(1)]);
        await act(async () => {
            render(
                <TaskDetail
                    task={running.task}
                    client={running.client}
                    onBack={vi.fn()}
                    onOpenSurface={vi.fn()}
                    onNewRun={onNewRun}
                />,
                host,
            );
            await Promise.resolve();
        });
        expect(buttonNamed('New run')).toBeNull();
    });
});
