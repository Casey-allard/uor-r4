import { AGENT_EVENT_SCHEMA, type AgentEvent } from './generated/agentContract.js';
import { digestEvent } from './generated/agentCanonical.js';
import {
    createRunQueuedEvent,
    createRunResumedEvent,
    createRunStartedEvent,
    fixtureEventId,
} from './fixtures.js';
import {
    AgentClient,
    AgentClientError,
    type AgentIncomingEvent,
    type AgentTransport,
    type ApprovalReceipt,
    type ArchiveTaskInput,
    type ArtifactContent,
    type ChangesSnapshot,
    type ChangesSnapshotInput,
    type CreateTaskInput,
    type ForceSendInput,
    type HostCapabilities,
    type ListTasksInput,
    type MessageReceipt,
    type PinTaskInput,
    type QueueMessageInput,
    type ReadArtifactInput,
    type ResolveApprovalInput,
    type ResumeTaskInput,
    type ResumeTaskResult,
    type RetryRunInput,
    type RunSummary,
    type StartRunInput,
    type SteerMessageInput,
    type StopInput,
    type StopReceipt,
    type SubscribeInput,
    type TaskSummary,
    type TreeEntry,
    type TreeInput,
} from './client.js';
import type { AgentStore } from '../state/agent.js';

const PROJECT_PATTERN = /^prj_[0-9a-f]{16}$/;

export interface FixtureTransportOptions {
    nowMs?: number;
    projectId?: string;
}

interface FixtureTask {
    taskId: string;
    projectId: string;
    title: string;
    repo: string | null;
    branch: string | null;
    createdAtMs: number;
    archived: boolean;
    pinned: boolean;
    latestRunId: string | null;
    generation: number;
}

interface FixtureRun {
    taskId: string;
    projectId: string;
    runId: string;
    generation: number;
    state:
        | 'queued'
        | 'running'
        | 'waiting_for_approval'
        | 'waiting_for_background'
        | 'cancelling'
        | 'finished'
        | 'failed'
        | 'cancelled'
        | 'expired';
    retryOfRunId: string | null;
    lastSequence: number;
    lastEventId: string | null;
    lastDigest: string | null;
}

interface FixtureListener {
    listener: (event: AgentIncomingEvent, digest?: string) => void;
    filter: SubscribeInput;
}

interface FixtureCall {
    method: string;
    payload: unknown;
}

const TERMINAL_STATES = new Set([
    'finished',
    'failed',
    'cancelled',
    'expired',
]);

function runKey(taskId: string, runId: string): string {
    return `${taskId}\u0000${runId}`;
}

// Labelled synthetic transport. Its task/run bookkeeping exists only to
// produce deterministic fixture events and receipts; AgentStore remains the
// sole authoritative task/run/event reducer. This is not a host, model or R4
// adapter.
export class FixtureTransport implements AgentTransport {
    readonly kind = 'fixture' as const;
    readonly calls: FixtureCall[] = [];

    private readonly tasks = new Map<string, FixtureTask>();
    private readonly runs = new Map<string, FixtureRun>();
    private readonly listeners = new Set<FixtureListener>();
    private readonly projectId: string;
    private clock: number;
    private counter = 0;

    constructor(options: FixtureTransportOptions = {}) {
        this.projectId = options.projectId ?? 'prj_0000000000000001';
        this.clock = options.nowMs ?? 1_794_283_200_000;
    }

    async listTasks(input: ListTasksInput = {}): Promise<TaskSummary[]> {
        this.record('listTasks', input);
        const query = input.query?.toLowerCase() ?? '';
        return [...this.tasks.values()]
            .filter((task) => input.includeArchived === true || !task.archived)
            .filter((task) => query === '' || task.title.toLowerCase().includes(query))
            .map((task) => this.summary(task));
    }

    async createTask(input: CreateTaskInput): Promise<TaskSummary> {
        this.record('createTask', input);
        const title = input.title.trim();
        if (title.length === 0) {
            throw new AgentClientError(
                'invalid_arguments',
                'task title must not be empty',
            );
        }
        const projectId = input.projectId ?? this.projectId;
        if (!PROJECT_PATTERN.test(projectId)) {
            throw new AgentClientError(
                'invalid_arguments',
                'projectId must match prj_<16 lowercase hex>',
            );
        }
        const task: FixtureTask = {
            taskId: this.nextId('tsk_'),
            projectId,
            title,
            repo: input.repo ?? null,
            branch: input.branch ?? null,
            createdAtMs: this.tick(),
            archived: false,
            pinned: false,
            latestRunId: null,
            generation: 0,
        };
        this.tasks.set(task.taskId, task);
        return this.summary(task);
    }

    async startRun(input: StartRunInput): Promise<RunSummary> {
        this.record('startRun', input);
        const task = this.requireTask(input.taskId);
        if (task.archived) {
            throw new AgentClientError('task_archived', 'task is archived');
        }
        if (input.prompt.trim().length === 0) {
            throw new AgentClientError(
                'invalid_arguments',
                'run prompt must not be empty',
            );
        }
        if (this.activeRun(task.taskId) !== null) {
            throw new AgentClientError(
                'out_of_order',
                'task already has an active run',
            );
        }
        const run = await this.createRun(task, null, input.mode ?? 'agent');
        return this.runSummary(run);
    }

    async queueMessage(input: QueueMessageInput): Promise<MessageReceipt> {
        this.record('queueMessage', input);
        const run = this.requireActiveRun(input.taskId);
        const messageId = this.nextId('msg_');
        const event = this.buildEvent(run, 'message.queued', {
            message_id: messageId,
            delivery: 'queue',
            boundary: 'next_turn',
            text_preview: input.text.slice(0, 120),
        });
        await this.track(run, event);
        return {
            taskId: input.taskId,
            runId: run.runId,
            messageId,
            delivery: 'queue',
        };
    }

    async steer(input: SteerMessageInput): Promise<MessageReceipt> {
        this.record('steer', input);
        const run = this.requireActiveRun(input.taskId);
        const messageId = this.nextId('msg_');
        const event = this.buildEvent(run, 'message.steered', {
            message_id: messageId,
            boundary: 'next_tool',
        });
        await this.track(run, event);
        return {
            taskId: input.taskId,
            runId: run.runId,
            messageId,
            delivery: 'steer',
        };
    }

    async forceSend(input: ForceSendInput): Promise<MessageReceipt> {
        this.record('forceSend', input);
        const run = this.requireActiveRun(input.taskId);
        const messageId = this.nextId('msg_');
        const event = this.buildEvent(run, 'message.forced', {
            message_id: messageId,
            boundary: 'next_tool',
        });
        await this.track(run, event);
        return {
            taskId: input.taskId,
            runId: run.runId,
            messageId,
            delivery: 'force',
        };
    }

    async pinTask(input: PinTaskInput): Promise<TaskSummary> {
        this.record('pinTask', input);
        const task = this.requireTask(input.taskId);
        task.pinned = input.pinned;
        return this.summary(task);
    }

    async archiveTask(input: ArchiveTaskInput): Promise<TaskSummary> {
        this.record('archiveTask', input);
        const task = this.requireTask(input.taskId);
        task.archived = input.archived;
        return this.summary(task);
    }

    async stop(input: StopInput): Promise<StopReceipt> {
        this.record('stop', input);
        const run = this.requireRun(input.taskId, input.runId);
        if (TERMINAL_STATES.has(run.state)) {
            throw new AgentClientError(
                'out_of_order',
                'run is already terminal',
            );
        }
        run.state = 'cancelling';
        const status = this.buildEvent(run, 'run.status', {
            status: 'cancelling',
        });
        await this.track(run, status);

        run.state = 'cancelled';
        const cancelled = this.buildEvent(run, 'run.cancelled', {
            safe_boundary: true,
            finished_at_ms: this.tick(),
        });
        await this.track(run, cancelled);
        return {
            taskId: input.taskId,
            runId: input.runId,
            acknowledged: true,
        };
    }

    async retry(input: RetryRunInput): Promise<RunSummary> {
        this.record('retry', input);
        const task = this.requireTask(input.taskId);
        const previous = this.requireRun(input.taskId, input.runId);
        if (previous.state !== 'failed' && previous.state !== 'cancelled') {
            throw new AgentClientError(
                'run_not_retryable',
                'only failed or cancelled runs are retryable',
            );
        }
        if (task.archived) {
            throw new AgentClientError('task_archived', 'task is archived');
        }
        const run = await this.createRun(task, previous.runId, 'agent');
        return this.runSummary(run);
    }

    async resolveApproval(
        input: ResolveApprovalInput,
    ): Promise<ApprovalReceipt> {
        this.record('resolveApproval', input);
        if (input.decision !== 'approved' && input.decision !== 'denied') {
            throw new AgentClientError(
                'invalid_arguments',
                'approval decision must be approved | denied',
            );
        }
        return {
            approvalId: input.approvalId,
            decision: input.decision,
        };
    }

    async resumeTask(input: ResumeTaskInput): Promise<ResumeTaskResult> {
        this.record('resumeTask', input);
        const task = this.requireTask(input.taskId);
        const run = this.requireRun(input.taskId, input.runId);
        if (TERMINAL_STATES.has(run.state)) {
            throw new AgentClientError(
                'out_of_order',
                'a terminal run is not resumable',
            );
        }
        if (run.lastDigest === null || run.lastEventId === null) {
            throw new AgentClientError(
                'snapshot_missing',
                'fixture run has no verified event digest chain',
            );
        }
        const snapshot = {
            taskId: task.taskId,
            runId: run.runId,
            generation: run.generation,
            lastSequence: run.lastSequence,
            lastEventId: run.lastEventId,
            headDigest: run.lastDigest,
        };
        const recordedAtMs = this.tick();
        const event = createRunResumedEvent({
            projectId: task.projectId,
            taskId: task.taskId,
            runId: run.runId,
            generation: run.generation + 1,
            recordedAtMs,
            sequence: run.lastSequence + 1,
            previousGeneration: run.generation,
            snapshotDigest: run.lastDigest,
            resumedAtMs: recordedAtMs,
        });
        // This event is not emitted here: the browser client verifies the
        // returned snapshot before appending it to the store. The fixture
        // advances to the returned event so a later stop or retry stays in
        // the same generation/sequence the client will hold.
        const digest = await digestEvent(event);
        task.generation = Math.max(task.generation, event.generation);
        run.generation = event.generation;
        run.lastSequence = event.sequence;
        run.lastEventId = event.event_id;
        run.lastDigest = digest;
        return { snapshot, event };
    }

    subscribe(
        listener: (event: AgentIncomingEvent, digest?: string) => void,
        input: SubscribeInput = {},
    ): () => void {
        this.record('subscribe', input);
        const entry: FixtureListener = { listener, filter: input };
        this.listeners.add(entry);
        return () => {
            this.listeners.delete(entry);
        };
    }

    async tree(input: TreeInput): Promise<TreeEntry[]> {
        this.record('tree', input);
        throw new AgentClientError(
            'host_unavailable',
            'the fixture transport has no host file tree',
        );
    }

    async changesSnapshot(
        input: ChangesSnapshotInput,
    ): Promise<ChangesSnapshot | null> {
        this.record('changesSnapshot', input);
        throw new AgentClientError(
            'host_unavailable',
            'the fixture transport has no host changes API',
        );
    }

    async readArtifact(input: ReadArtifactInput): Promise<ArtifactContent> {
        this.record('readArtifact', input);
        throw new AgentClientError(
            'host_unavailable',
            'the fixture transport has no host artifact reader',
        );
    }

    async hostCapabilities(): Promise<HostCapabilities> {
        this.record('hostCapabilities', null);
        return {
            tree: false,
            changes: false,
            readArtifact: false,
            terminal: false,
            git: false,
            mcp: false,
            approvals: false,
        };
    }

    private async createRun(
        task: FixtureTask,
        retryOfRunId: string | null,
        mode: 'plan' | 'agent',
    ): Promise<FixtureRun> {
        const generation = task.generation + 1;
        task.generation = generation;
        const run: FixtureRun = {
            taskId: task.taskId,
            projectId: task.projectId,
            runId: this.nextId('run_'),
            generation,
            state: 'queued',
            retryOfRunId,
            lastSequence: -1,
            lastEventId: null,
            lastDigest: null,
        };
        this.runs.set(runKey(run.taskId, run.runId), run);
        task.latestRunId = run.runId;

        const queuedAtMs = this.tick();
        const queued = createRunQueuedEvent({
            projectId: task.projectId,
            taskId: task.taskId,
            runId: run.runId,
            generation,
            recordedAtMs: queuedAtMs,
            queuedAtMs,
        });
        await this.track(run, queued);
        run.state = 'queued';

        const startedAtMs = this.tick();
        const started = createRunStartedEvent({
            projectId: task.projectId,
            taskId: task.taskId,
            runId: run.runId,
            generation,
            recordedAtMs: startedAtMs,
            startedAtMs,
            mode,
            retryOfRunId,
        });
        await this.track(run, started);
        run.state = 'running';
        return run;
    }

    private async track(
        run: FixtureRun,
        event: AgentIncomingEvent,
    ): Promise<void> {
        const digest = await this.emit(event);
        run.lastSequence = event.sequence;
        run.lastEventId = event.event_id;
        run.lastDigest = digest;
    }

    private async emit(event: AgentIncomingEvent): Promise<string> {
        const digest = await digestEvent(event as AgentEvent);
        for (const entry of [...this.listeners]) {
            if (
                entry.filter.taskId !== undefined &&
                entry.filter.taskId !== event.task_id
            ) {
                continue;
            }
            if (
                entry.filter.runId !== undefined &&
                entry.filter.runId !== event.run_id
            ) {
                continue;
            }
            entry.listener(event, digest);
        }
        return digest;
    }

    private buildEvent(
        run: FixtureRun,
        type: string,
        data: Record<string, unknown>,
    ): AgentIncomingEvent {
        const sequence = run.lastSequence + 1;
        return {
            schema: AGENT_EVENT_SCHEMA,
            event_id: fixtureEventId(run.runId, run.generation, sequence),
            project_id: run.projectId,
            task_id: run.taskId,
            run_id: run.runId,
            generation: run.generation,
            sequence,
            recorded_at_ms: this.tick(),
            type,
            data,
        } as unknown as AgentIncomingEvent;
    }

    private requireTask(taskId: string): FixtureTask {
        const task = this.tasks.get(taskId);
        if (task === undefined) {
            throw new AgentClientError('task_not_found', 'task is not in the fixture');
        }
        return task;
    }

    private requireRun(taskId: string, runId: string): FixtureRun {
        const run = this.runs.get(runKey(taskId, runId));
        if (run === undefined) {
            throw new AgentClientError('run_not_found', 'run is not in the fixture');
        }
        return run;
    }

    private activeRun(taskId: string): FixtureRun | null {
        let active: FixtureRun | null = null;
        for (const run of this.runs.values()) {
            if (run.taskId === taskId && !TERMINAL_STATES.has(run.state)) {
                active = run;
            }
        }
        return active;
    }

    private requireActiveRun(taskId: string): FixtureRun {
        const run = this.activeRun(taskId);
        if (run === null) {
            throw new AgentClientError(
                'run_not_found',
                'task has no active run',
            );
        }
        return run;
    }

    private runSummary(run: FixtureRun): RunSummary {
        return {
            taskId: run.taskId,
            runId: run.runId,
            generation: run.generation,
            state: run.state,
            retryOfRunId: run.retryOfRunId,
        };
    }

    private summary(task: FixtureTask): TaskSummary {
        const latestRun =
            task.latestRunId === null
                ? null
                : this.runs.get(runKey(task.taskId, task.latestRunId)) ?? null;
        return {
            taskId: task.taskId,
            projectId: task.projectId,
            title: task.title,
            repo: task.repo,
            branch: task.branch,
            createdAtMs: task.createdAtMs,
            archived: task.archived,
            pinned: task.pinned,
            latestRunId: task.latestRunId,
            state: this.taskState(task, latestRun),
        };
    }

    private taskState(
        task: FixtureTask,
        run: FixtureRun | null,
    ): TaskSummary['state'] {
        if (task.archived) {
            return 'archived';
        }
        if (run === null) {
            return 'idle';
        }
        switch (run.state) {
            case 'queued':
                return 'queued';
            case 'running':
                return 'running';
            case 'waiting_for_approval':
                return 'waiting_for_approval';
            case 'waiting_for_background':
                return 'waiting_for_background';
            case 'cancelling':
                return 'cancelling';
            default:
                return 'idle';
        }
    }

    private nextId(prefix: 'tsk_' | 'run_' | 'msg_'): string {
        this.counter += 1;
        return `${prefix}${this.counter.toString(16).padStart(16, '0')}`;
    }

    private tick(): number {
        const value = this.clock;
        this.clock += 1;
        return value;
    }

    private record(method: string, payload: unknown): void {
        this.calls.push({ method, payload });
    }
}

export interface FixtureAgentClientOptions extends FixtureTransportOptions {
    store?: AgentStore;
}

export function createFixtureAgentClient(
    options: FixtureAgentClientOptions = {},
): AgentClient {
    const transport = new FixtureTransport(options);
    return new AgentClient({ transport, store: options.store });
}
