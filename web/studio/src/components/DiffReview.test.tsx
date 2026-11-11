import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

import { AgentClient, type AgentIncomingEvent, type ChangesSnapshot, type HostCapabilities } from '../agent/client.js';
import { FixtureTransport } from '../agent/fixtureTransport.js';
import {
    HOST_RESULT_SCHEMA,
    HostClient,
    sha256Canonical,
    type HostInvokeResponse,
    type HostSubscription,
    type HostToolIntent,
    type HostToolResult,
    type HostTransport,
    type HostTransportCapabilities,
} from '../agent/hostClient.js';
import { AgentTaskStore, type DiffCommentRecord } from '../agent/store.js';
import { createDesktopActions } from '../adapters/desktopActions.js';
import { ChangesPanel } from './DesktopTools.js';
import { makeFiles } from './studioTestFixtures.js';
import { DiffReview, createUploadedSnapshot, MAX_UPLOAD_FILE_BYTES, type DiffComment } from './DiffReview.js';

const ALL_FALSE: HostCapabilities = {
    tree: false,
    changes: false,
    readArtifact: false,
    terminal: false,
    git: false,
    mcp: false,
    approvals: false,
};

let host: HTMLDivElement;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.restoreAllMocks();
});

class RecordingFixtureTransport extends FixtureTransport {
    readonly branchCalls: Array<{ taskId: string; scope: string }> = [];

    override async changesSnapshot(input: {
        taskId: string;
        scope: 'last_turn' | 'branch';
    }): Promise<ChangesSnapshot | null> {
        this.branchCalls.push({ taskId: input.taskId, scope: input.scope });
        return {
            scope: input.scope,
            files: [{ path: 'fixture-branch.ts', status: 'modified' }],
            headDigest: `sha256:${'d'.repeat(64)}`,
            capturedAtMs: 7,
        };
    }

    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...(await super.hostCapabilities()), changes: true };
    }
}

class TruncatingFixtureTransport extends FixtureTransport {
    override async hostCapabilities(): Promise<HostCapabilities> {
        return { ...(await super.hostCapabilities()), changes: true };
    }

    override async changesSnapshot(input: {
        taskId: string;
        scope: 'last_turn' | 'branch';
    }): Promise<ChangesSnapshot | null> {
        return {
            scope: 'branch',
            files: [{ path: 'bounded.ts', status: 'modified' }],
            headDigest: `sha256:${'f'.repeat(64)}`,
            capturedAtMs: 11,
            truncated: true,
        };
    }
}

class FakeHostTransport implements HostTransport {
    readonly kind = 'bridge' as const;
    readonly intents: HostToolIntent[] = [];

    async capabilities(): Promise<HostTransportCapabilities> {
        return {
            kind: 'bridge',
            hostAvailable: true,
            streaming: 'none',
            ...ALL_FALSE,
            changes: true,
        };
    }

    async invoke(intent: HostToolIntent): Promise<HostInvokeResponse> {
        this.intents.push(intent);
        const output = {
            snapshot: {
                scope: 'branch',
                files: [
                    { path: 'src/host-branch.ts', status: 'modified' },
                    { path: 'src/new.ts', status: 'added' },
                ],
                head_digest: `sha256:${'e'.repeat(64)}`,
                head_commit: '1'.repeat(40),
                base_ref: 'main',
                captured_at_ms: 1_800_000_000_000,
                truncated: true,
            },
        };
        const result: HostToolResult = {
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
            output_preview: 'branch',
            occurred_at_ms: 1_800_000_000_000,
        };
        return result;
    }

    subscribe(): HostSubscription {
        return { close: () => undefined, cursor: () => null };
    }

    async approve(input: { approvalId: string; decision: 'approved' | 'denied' }) {
        return input;
    }
}

function changesSnapshotEvent(
    taskId: string,
    projectId: string,
    runId: string,
    generation: number,
    sequence: number,
    scope: 'last_turn' | 'branch',
    files: Array<Record<string, unknown>>,
    headDigest: string,
): AgentIncomingEvent {
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: `evt_${runId}_${generation}_${sequence.toString().padStart(12, '0')}`,
        project_id: projectId,
        task_id: taskId,
        run_id: runId,
        generation,
        sequence,
        recorded_at_ms: 1_794_283_200_000 + sequence,
        type: 'changes.snapshot',
        data: {
            scope,
            files,
            head_digest: headDigest,
            captured_at_ms: 1_794_283_200_000 + sequence,
        },
    } as unknown as AgentIncomingEvent;
}

async function seedLastTurn(
    client: AgentClient,
    files: Array<Record<string, unknown>> = [{ path: 'src/a.ts', change: 'modified' }],
    headDigest = `sha256:${'a'.repeat(64)}`,
): Promise<{ taskId: string; runId: string; files: Array<Record<string, unknown>>; headDigest: string }> {
    const task = await client.createTask({ title: 'Review fixture' });
    const run = await client.startRun({ taskId: task.taskId, prompt: 'review' });
    const cursor = client.store.getRunCursor(task.taskId, run.runId);
    if (cursor === null) throw new Error('run cursor is missing');
    const applied = client.store.apply(
        changesSnapshotEvent(
            task.taskId,
            task.projectId,
            run.runId,
            cursor.generation,
            cursor.lastSequence + 1,
            'last_turn',
            files,
            headDigest,
        ),
    );
    if (applied.status !== 'applied') {
        throw new Error(`changes.snapshot was rejected: ${applied.message ?? ''}`);
    }
    return { taskId: task.taskId, runId: run.runId, files, headDigest };
}

function applyLocalBranch(
    client: AgentClient,
    seeded: { taskId: string; runId: string },
    files: Array<Record<string, unknown>> = [{ path: 'local-branch.ts', status: 'modified' }],
): void {
    const task = client.store.snapshot().tasks.find((candidate) => candidate.taskId === seeded.taskId);
    if (task === undefined) throw new Error('task is missing');
    const run = client.store.getRunCursor(seeded.taskId, seeded.runId);
    if (run === null) throw new Error('run cursor is missing');
    const applied = client.store.apply(
        changesSnapshotEvent(
            seeded.taskId,
            task.projectId,
            seeded.runId,
            run.generation,
            run.lastSequence + 1,
            'branch',
            files,
            `sha256:${'b'.repeat(64)}`,
        ),
    );
    if (applied.status !== 'applied') {
        throw new Error(`local branch changes.snapshot was rejected: ${applied.message ?? ''}`);
    }
}

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
    let resolve!: (value: T) => void;
    const promise = new Promise<T>((settle) => { resolve = settle; });
    return { promise, resolve };
}

function snapshot(
    overrides: Partial<ChangesSnapshot> = {},
): ChangesSnapshot {
    return {
        scope: 'last_turn',
        files: [{ path: 'src/a.ts', change: 'modified', content: 'line one\nline two\nline three' }],
        headDigest: `sha256:${'a'.repeat(64)}`,
        capturedAtMs: 1_794_283_200_000,
        ...overrides,
    };
}

function comment(overrides: Partial<DiffComment> = {}): DiffComment {
    return {
        commentId: 'cmt_1',
        taskId: 'tsk_9a3c5e7b1d2f4a60',
        scope: 'last_turn',
        path: 'src/a.ts',
        line: 2,
        body: 'check this line',
        headDigest: `sha256:${'a'.repeat(64)}`,
        createdAtMs: 1_794_283_200_001,
        ...overrides,
    };
}

function renderReview(
    overrides: Partial<Parameters<typeof DiffReview>[0]> = {},
): void {
    act(() => {
        render(
            <DiffReview
                taskId="tsk_9a3c5e7b1d2f4a60"
                scope="last_turn"
                onScope={vi.fn()}
                comments={[]}
                onComment={vi.fn()}
                snapshot={snapshot()}
                source="browser"
                {...overrides}
            />,
            host,
        );
    });
}

describe('DiffReview change sources', () => {
    test('changes_snapshot_last_turn_scope', async () => {
        const transport = new RecordingFixtureTransport();
        const client = new AgentClient({ transport });
        const seeded = await seedLastTurn(client);
        applyLocalBranch(client, seeded);

        const result = await client.changesSnapshot({
            taskId: seeded.taskId,
            scope: 'last_turn',
        });

        expect(result).toMatchObject({
            scope: 'last_turn',
            files: seeded.files,
            headDigest: seeded.headDigest,
        });
        const branch = await client.changesSnapshot({
            taskId: seeded.taskId,
            scope: 'branch',
        });
        expect(branch).toMatchObject({
            scope: 'branch',
            files: [{ path: 'fixture-branch.ts', status: 'modified' }],
            headDigest: `sha256:${'d'.repeat(64)}`,
        });
        expect(transport.branchCalls).toHaveLength(1);
    });

    test('local_branch_snapshot_never_becomes_companion_data', async () => {
        const client = new AgentClient({ transport: new FixtureTransport() });
        const seeded = await seedLastTurn(client);
        applyLocalBranch(client, seeded);

        await expect(
            client.changesSnapshot({ taskId: seeded.taskId, scope: 'branch' }),
        ).rejects.toMatchObject({ code: 'host_unavailable' });
        await expect(
            client.changesSnapshot({ taskId: seeded.taskId, scope: 'last_turn' }),
        ).resolves.toMatchObject({
            scope: 'last_turn',
            files: seeded.files,
        });
    });

    test('changes_snapshot_branch_scope', async () => {
        const transport = new RecordingFixtureTransport();
        const fakeHost = new FakeHostTransport();
        const client = new AgentClient({
            transport,
            hostClient: new HostClient({ transport: fakeHost }),
        });
        const task = await client.createTask({ title: 'Branch review' });
        await client.startRun({ taskId: task.taskId, prompt: 'branch' });

        const result = await client.changesSnapshot({
            taskId: task.taskId,
            scope: 'branch',
        });

        expect(result).toMatchObject({
            scope: 'branch',
            files: [
                { path: 'src/host-branch.ts', status: 'modified' },
                { path: 'src/new.ts', status: 'added' },
            ],
            headDigest: `sha256:${'e'.repeat(64)}`,
            truncated: true,
        });
        expect(fakeHost.intents.map((intent) => intent.tool)).toEqual(['git.changes']);
        expect(transport.branchCalls).toEqual([]);
    });

    test('durable_diff_comments_are_keyed_by_task_path_line_and_head', () => {
        const prototype = AgentTaskStore.prototype as unknown as Record<string, unknown>;
        expect(typeof prototype.appendDiffComment).toBe('function');
        expect(typeof prototype.readDiffComments).toBe('function');
    });

    test('bridge_transport_preserves_branch_truncation', async () => {
        const client = new AgentClient({ transport: new TruncatingFixtureTransport() });
        const task = await client.createTask({ title: 'Bounded branch' });
        await client.startRun({ taskId: task.taskId, prompt: 'bounded' });

        const result = await client.changesSnapshot({
            taskId: task.taskId,
            scope: 'branch',
        });

        expect(result).toMatchObject({
            scope: 'branch',
            truncated: true,
            files: [{ path: 'bounded.ts', status: 'modified' }],
        });
    });
});

describe('DiffReview static browser review', () => {
    test('static_browser_reviews_uploaded_file_locally', async () => {
        const file = new File(
            ['export function add(a, b) {\n    return a + b;\n}\n'],
            'src/math.ts',
            { type: 'text/plain' },
        );
        const uploaded = await createUploadedSnapshot([file]);

        expect(uploaded.scope).toBe('branch');
        expect(uploaded.truncated).toBe(false);
        expect(uploaded.files).toHaveLength(1);
        expect(uploaded.files[0]).toMatchObject({
            path: 'src/math.ts',
            status: 'modified',
        });
        expect(uploaded.headDigest).toMatch(/^sha256:[0-9a-f]{64}$/);

        const onComment = vi.fn();
        renderReview({
            scope: 'branch',
            snapshot: uploaded,
            source: 'upload',
            onComment,
        });

        expect(host.textContent).toContain('src/math.ts');
        expect(host.textContent).toContain('export function add(a, b)');
        expect(host.textContent).toContain('Uploaded file');
        expect(host.textContent).toContain('stored in this browser');
        expect(host.querySelector('[data-diff-file="src/math.ts"]')).not.toBeNull();
        expect(host.querySelector('[data-diff-line="2"]')?.textContent).toContain('return a + b;');
    });

    test('uploaded_reading_is_byte_bounded_and_multibyte_safe', async () => {
        const prefix = 'a'.repeat(MAX_UPLOAD_FILE_BYTES - 2);
        const file = new File([`${prefix}😀tail`], 'emoji.txt', { type: 'text/plain' });
        expect(file.size).toBeGreaterThan(MAX_UPLOAD_FILE_BYTES);

        const uploaded = await createUploadedSnapshot([file]);
        const content = uploaded.files[0]?.content;
        expect(typeof content).toBe('string');
        expect(uploaded.truncated).toBe(true);
        expect((content as string).includes('\uFFFD')).toBe(false);
        expect(new TextEncoder().encode(content as string).byteLength).toBeLessThanOrEqual(MAX_UPLOAD_FILE_BYTES);
        expect((content as string).endsWith('a')).toBe(true);
        expect((content as string).endsWith('tail')).toBe(false);
    });

    test('branch_truncation_is_surfaced_accessible', () => {
        renderReview({
            scope: 'branch',
            source: 'host',
            snapshot: snapshot({
                scope: 'branch',
                files: [{ path: 'src/host.ts', status: 'modified' }],
                truncated: true,
            }),
        });

        expect(host.textContent).toContain('truncated');
        expect(host.textContent).toContain('not the complete');
        expect(host.querySelector('[data-diff-truncated]')).not.toBeNull();
    });
});

describe('DiffReview local comments', () => {
    test('stale_comment_line_is_marked', () => {
        renderReview({
            comments: [
                comment(),
                comment({
                    commentId: 'cmt_old_head',
                    body: 'old head note',
                    headDigest: `sha256:${'b'.repeat(64)}`,
                }),
                comment({
                    commentId: 'cmt_missing_file',
                    body: 'missing file note',
                    path: 'src/gone.ts',
                    line: 1,
                }),
                comment({
                    commentId: 'cmt_missing_line',
                    body: 'missing line note',
                    line: 99,
                }),
            ],
        });

        const current = host.querySelector('[data-diff-current-comments]');
        const stale = host.querySelector('[data-diff-stale-comments]');
        expect(current?.textContent).toContain('check this line');
        expect(current?.textContent).not.toContain('old head note');
        expect(stale?.textContent).toContain('old head note');
        expect(stale?.textContent).toContain('missing file note');
        expect(stale?.textContent).toContain('missing line note');
        expect(stale?.textContent).toContain('head changed');
        expect(stale?.textContent).toContain('no longer in this snapshot');
        expect(stale?.textContent).toContain('Line 99 is not present');
        expect(host.querySelectorAll('[data-diff-stale="true"]')).toHaveLength(3);
    });

    test('comment_form_adds_a_line_bound_local_comment', async () => {
        const onComment = vi.fn();
        renderReview({ onComment });

        const body = host.querySelector<HTMLTextAreaElement>('textarea')!;
        const line = host.querySelector<HTMLInputElement>('input[type="number"]')!;
        act(() => {
            line.value = '3';
            line.dispatchEvent(new Event('input', { bubbles: true }));
            body.value = 'please rename this';
            body.dispatchEvent(new Event('input', { bubbles: true }));
        });
        await act(async () => {
            host.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
            await Promise.resolve();
        });

        expect(onComment).toHaveBeenCalledWith({
            path: 'src/a.ts',
            line: 3,
            body: 'please rename this',
        });
    });

    test('comment_form_rejects_a_line_that_is_not_in_the_snapshot', async () => {
        const onComment = vi.fn();
        renderReview({ onComment });

        const body = host.querySelector<HTMLTextAreaElement>('textarea')!;
        const line = host.querySelector<HTMLInputElement>('input[type="number"]')!;
        act(() => {
            line.value = '99';
            line.dispatchEvent(new Event('input', { bubbles: true }));
            body.value = 'ghost line';
            body.dispatchEvent(new Event('input', { bubbles: true }));
        });
        await act(async () => {
            host.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
            await Promise.resolve();
        });

        expect(onComment).not.toHaveBeenCalled();
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Line 99 is not present');
    });

    test('scope_and_task_transition_cannot_write_with_old_snapshot', async () => {
        const taskA = 'tsk_9a3c5e7b1d2f4a60';
        const taskB = 'tsk_1b2c3d4e5f607182';
        const snapshotA = snapshot();
        const branch = deferred<ChangesSnapshot | null>();
        const taskBSnapshot = deferred<ChangesSnapshot | null>();
        const taskBComments = deferred<DiffCommentRecord[]>();
        const recordA: DiffCommentRecord = {
            task_id: taskA,
            comment_id: 'cmt_saved',
            path: 'src/a.ts',
            line: 2,
            body: 'check this line',
            head_digest: snapshotA.headDigest,
            scope: 'last_turn',
            created_at_ms: 1,
        };
        const changesSnapshot = vi.fn((input: { taskId: string; scope: string }) => {
            if (input.taskId === taskB) return taskBSnapshot.promise;
            if (input.scope === 'branch') return branch.promise;
            return Promise.resolve(snapshotA);
        });
        const readDiffComments = vi.fn((taskId: string) =>
            taskId === taskA ? Promise.resolve([recordA]) : taskBComments.promise,
        );
        const addDiffComment = vi.fn(async (input: {
            taskId: string; scope: string; path: string; line: number; body: string; headDigest: string;
        }): Promise<DiffCommentRecord> => ({
            task_id: input.taskId,
            comment_id: 'cmt_new',
            path: input.path,
            line: input.line,
            body: input.body,
            head_digest: input.headDigest,
            scope: input.scope as 'last_turn' | 'branch',
            created_at_ms: 2,
        }));
        const fake = { changesSnapshot, readDiffComments, addDiffComment } as unknown as AgentClient;
        const actions = createDesktopActions({ invoke: vi.fn(async () => { throw new Error('no host'); }) });
        const panel = (taskId: string) => <ChangesPanel actions={actions} files={makeFiles()} copy={vi.fn()} agentClient={fake} taskId={taskId} />;

        act(() => { render(panel(taskA), host); });
        await vi.waitFor(() => expect(host.querySelector('textarea')).not.toBeNull());
        const body = host.querySelector<HTMLTextAreaElement>('textarea')!;
        act(() => {
            body.value = 'old scope note';
            body.dispatchEvent(new Event('input', { bubbles: true }));
        });
        const oldForm = host.querySelector('form')!;

        const branchButton = Array.from(host.querySelectorAll('button'))
            .find((button) => button.textContent === 'Branch')!;
        act(() => { branchButton.click(); });
        expect(host.querySelector('form')).toBeNull();
        act(() => { oldForm.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        await Promise.resolve();
        expect(addDiffComment).not.toHaveBeenCalled();

        await act(async () => {
            branch.resolve(snapshot({
                scope: 'branch',
                files: [{ path: 'src/branch.ts', status: 'modified' }],
            }));
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.querySelector('textarea')).not.toBeNull());
        const branchForm = host.querySelector('form')!;

        act(() => { render(panel(taskB), host); });
        expect(host.querySelector('form')).toBeNull();
        expect(host.textContent).not.toContain('check this line');
        act(() => { branchForm.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        await Promise.resolve();
        expect(addDiffComment).not.toHaveBeenCalled();

        await act(async () => {
            taskBSnapshot.resolve(snapshot({ headDigest: `sha256:${'b'.repeat(64)}` }));
            taskBComments.resolve([]);
            await Promise.resolve();
        });
        await vi.waitFor(() => expect(host.querySelector('textarea')).not.toBeNull());

        const bodyB = host.querySelector<HTMLTextAreaElement>('textarea')!;
        act(() => {
            bodyB.value = 'new task note';
            bodyB.dispatchEvent(new Event('input', { bubbles: true }));
        });
        await act(async () => {
            host.querySelector<HTMLButtonElement>('button[type="submit"]')!.click();
            await Promise.resolve();
        });
        expect(addDiffComment).toHaveBeenCalledWith(expect.objectContaining({
            taskId: taskB,
            scope: 'branch',
            headDigest: `sha256:${'b'.repeat(64)}`,
            body: 'new task note',
        }));
    });
});

describe('DiffReview control surface', () => {
    test('no_merge_control_without_service_identity', () => {
        renderReview();

        expect(host.querySelector('[data-diff-review]')).not.toBeNull();
        expect(host.textContent).toContain('Local review only');
        expect(host.textContent).not.toMatch(/\bmerge\b/i);
        expect(Array.from(host.querySelectorAll('button')).map((button) => button.textContent).join(' ')).not.toMatch(/\bmerge\b/i);
    });

    test('no_checkpoint_restore_control_is_rendered', () => {
        renderReview();

        expect(host.querySelector('[data-diff-review]')).not.toBeNull();
        expect(host.textContent).toContain('Local review only');
        expect(host.textContent).not.toMatch(/restore checkpoint/i);
        expect(host.textContent).not.toMatch(/\bcheckpoint\b/i);
        expect(Array.from(host.querySelectorAll('button')).map((button) => button.textContent).join(' ')).not.toMatch(/restore|checkpoint/i);
    });
});

describe('ChangesPanel host grant selection', () => {
    const TASK = 'tsk_9a3c5e7b1d2f4a60';
    const GRANT_A = 'gr_9a3c5e7b1d2f4a60';
    const GRANT_B = 'gr_0b1d2f4a6c8e9a30';

    function grantClient(
        grants: Array<{ grantId: string; path: string }>,
        calls: Array<{ taskId: string; scope: string; grantId?: string }>,
    ): AgentClient {
        return {
            hostSupportsGrants: () => true,
            grants: vi.fn(async () => grants),
            changesSnapshot: vi.fn(async (input: { taskId: string; scope: 'last_turn' | 'branch'; grantId?: string }) => {
                calls.push(input);
                return snapshot({
                    scope: input.scope,
                    files: [{ path: 'src/branch.ts', status: 'modified', content: 'branch line' }],
                    headDigest: `sha256:${'d'.repeat(64)}`,
                });
            }),
            readDiffComments: vi.fn(async () => []),
            addDiffComment: vi.fn(),
            store: {
                snapshot: () => ({ tasks: [], runs: [], selectedTaskId: null, selectedTask: null }),
            },
        } as unknown as AgentClient;
    }

    async function renderAndChooseBranch(client: AgentClient): Promise<void> {
        const actions = createDesktopActions({ invoke: vi.fn(async () => { throw new Error('no host'); }) });
        act(() => {
            render(<ChangesPanel actions={actions} files={makeFiles()} copy={vi.fn()} agentClient={client} taskId={TASK} />, host);
        });
        await vi.waitFor(() => expect(host.querySelectorAll('button').length).toBeGreaterThan(0));
        const branch = Array.from(host.querySelectorAll('button')).find((button) => button.textContent === 'Branch')!;
        act(() => { branch.click(); });
    }

    test('single_grant_is_auto_selected_and_passed_to_branch_changes', async () => {
        const calls: Array<{ taskId: string; scope: string; grantId?: string }> = [];
        await renderAndChooseBranch(grantClient([{ grantId: GRANT_A, path: '/granted/project' }], calls));
        await vi.waitFor(() => expect(calls.some((call) => call.scope === 'branch')).toBe(true));
        expect(calls.find((call) => call.scope === 'branch')).toMatchObject({
            taskId: TASK,
            grantId: GRANT_A,
        });
        expect(host.textContent).toContain('/granted/project');
        expect(host.querySelector('select[aria-label="Approved folder"]')).toBeNull();
    });

    test('multiple_grants_require_a_labelled_selection_before_branch_changes', async () => {
        const calls: Array<{ taskId: string; scope: string; grantId?: string }> = [];
        await renderAndChooseBranch(grantClient([
            { grantId: GRANT_A, path: '/granted/project' },
            { grantId: GRANT_B, path: '/other/project' },
        ], calls));
        await vi.waitFor(() =>
            expect(host.querySelector('select[aria-label="Approved folder"]')).not.toBeNull(),
        );
        const select = host.querySelector<HTMLSelectElement>('select[aria-label="Approved folder"]')!;
        expect(calls.some((call) => call.scope === 'branch')).toBe(false);
        expect(select.options).toHaveLength(3);
        expect(select.value).toBe('');
        act(() => {
            select.value = GRANT_B;
            select.dispatchEvent(new Event('change', { bubbles: true }));
        });
        await vi.waitFor(() => expect(calls.some((call) => call.scope === 'branch')).toBe(true));
        expect(calls.find((call) => call.scope === 'branch')).toMatchObject({
            taskId: TASK,
            grantId: GRANT_B,
        });
    });

    test('no_grants_reports_unavailable_instead_of_calling_without_a_grant', async () => {
        const calls: Array<{ taskId: string; scope: string; grantId?: string }> = [];
        await renderAndChooseBranch(grantClient([], calls));
        await vi.waitFor(() => expect(host.textContent).toContain('reports no approved folders'));
        expect(calls.some((call) => call.scope === 'branch')).toBe(false);
    });

    test('active_run_disables_the_comment_form_with_an_honest_reason', async () => {
        const client = {
            changesSnapshot: vi.fn(async () => snapshot()),
            readDiffComments: vi.fn(async () => []),
            addDiffComment: vi.fn(),
            store: {
                snapshot: () => ({
                    tasks: [{
                        taskId: TASK,
                        latestRunId: 'run_0000000000000001',
                        runs: [{ runId: 'run_0000000000000001', state: 'running' }],
                    }],
                    runs: [],
                    selectedTaskId: TASK,
                    selectedTask: null,
                }),
            },
        } as unknown as AgentClient;
        const actions = createDesktopActions({ invoke: vi.fn(async () => { throw new Error('no host'); }) });
        act(() => {
            render(<ChangesPanel actions={actions} files={makeFiles()} copy={vi.fn()} agentClient={client} taskId={TASK} />, host);
        });
        await vi.waitFor(() => expect(host.textContent).toContain('this run is still active'));
        expect(host.querySelector('form')).toBeNull();
    });
});
