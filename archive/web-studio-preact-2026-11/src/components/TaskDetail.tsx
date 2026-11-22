import { useEffect, useMemo, useRef, useState } from 'preact/hooks';
import type { AgentClient } from '../agent/client.js';
import { displayRunState, type TaskView, type RunView, type AgentToolCallView } from '../state/agent.js';
import type { ApprovalHost } from './ApprovalCard.js';
import { RunStatus, isActiveRunState, isTerminalRunState, runStateLabel } from './RunStatus.js';
import { TaskError } from './TaskError.js';
import { TaskTimeline, type TimelineApproval, type TimelineEntry } from './TaskTimeline.js';

export type TaskSurface = 'changes' | 'terminal';

export interface TaskDetailProps {
    task: TaskView;
    client: AgentClient;
    onBack(): void;
    onNewRun?(): void;
    onOpenSurface(surface: TaskSurface): void;
}

function latestRun(task: TaskView): RunView | null {
    if (task.latestRunId === null) return task.runs.length === 0 ? null : task.runs[task.runs.length - 1]!;
    return task.runs.find((run) => run.runId === task.latestRunId) ?? task.runs[task.runs.length - 1] ?? null;
}

function errorView(failure: unknown): { code: string; message: string } {
    const code = failure !== null && typeof failure === 'object' && 'code' in failure && typeof (failure as { code?: unknown }).code === 'string'
        ? (failure as { code: string }).code
        : 'internal';
    return { code, message: failure instanceof Error ? failure.message : String(failure) };
}

function firstSequences(run: RunView): { byMessage: Map<string, number>; byTool: Map<string, number> } {
    const byMessage = new Map<string, number>();
    const byTool = new Map<string, number>();
    for (const event of run.events) {
        const data = event.data;
        if ((event.type === 'assistant.delta' || event.type === 'assistant.message') && typeof data.message_id === 'string' && !byMessage.has(data.message_id)) {
            byMessage.set(data.message_id, event.sequence);
        }
        if (event.type.startsWith('tool.') && typeof data.tool_call_id === 'string' && !byTool.has(data.tool_call_id)) {
            byTool.set(data.tool_call_id, event.sequence);
        }
    }
    return { byMessage, byTool };
}

function terminalSequence(run: RunView): number | null {
    let sequence: number | null = null;
    for (const event of run.events) {
        if (event.type === 'run.finished' || event.type === 'run.failed' || event.type === 'run.cancelled' || event.type === 'run.expired') {
            sequence = event.sequence;
        }
    }
    return sequence;
}

function visibleTool(toolCall: AgentToolCallView, sequence: number | undefined, terminal: number | null): boolean {
    if (terminal === null) return true;
    if (toolCall.status === 'requested' || toolCall.status === 'started') return false;
    return sequence === undefined || sequence <= terminal;
}

function modelLabel(run: RunView | null): string {
    const started = run?.events.find((event) => event.type === 'run.started');
    const model = started?.data.model;
    if (model !== null && typeof model === 'object' && 'id' in model && 'kind' in model) {
        return `${String((model as { id: unknown }).id)} (${String((model as { kind: unknown }).kind)})`;
    }
    return 'Not recorded yet';
}

function environmentLabel(run: RunView | null): string {
    const started = run?.events.find((event) => event.type === 'run.started');
    const environment = started?.data.environment;
    if (environment !== null && typeof environment === 'object' && 'kind' in environment) {
        return String((environment as { kind: unknown }).kind);
    }
    return 'Not recorded yet';
}

function isSynthetic(run: RunView | null): boolean {
    const started = run?.events.find((event) => event.type === 'run.started');
    const model = started?.data.model;
    if (model === null || typeof model !== 'object' || !('kind' in model)) return false;
    const kind = String((model as { kind: unknown }).kind).toLowerCase();
    return kind.includes('synthetic') || kind.includes('replay') || kind.includes('fixture');
}

function usageEntry(run: RunView): TimelineEntry | null {
    let inputTokens = 0;
    let outputTokens = 0;
    let modelCalls = 0;
    let sequence: number | null = null;
    for (const event of run.events) {
        if (event.type !== 'usage.updated') continue;
        const data = event.data;
        if (typeof data.input_tokens !== 'number' || typeof data.output_tokens !== 'number' || typeof data.model_calls !== 'number') continue;
        inputTokens += data.input_tokens;
        outputTokens += data.output_tokens;
        modelCalls += data.model_calls;
        sequence = event.sequence;
    }
    if (sequence === null) return null;
    return { kind: 'usage', id: `usage_${sequence}`, sequence, inputTokens, outputTokens, modelCalls };
}

function deliverySequence(run: RunView, message: string, used: Set<number>): number {
    for (const event of run.events) {
        if (event.type === 'error' && event.data.code === 'message_not_delivered' && event.data.message === message && !used.has(event.sequence)) return event.sequence;
    }
    return terminalSequence(run) ?? 0;
}

function timelineEntries(run: RunView): TimelineEntry[] {
    const sequences = firstSequences(run);
    const terminal = terminalSequence(run);
    const terminalState = isTerminalRunState(run.state);
    const entries: TimelineEntry[] = [];
    const messageIds = new Set(run.messages.map((message) => message.messageId));
    for (const message of run.messages) {
        entries.push({
            kind: 'assistant',
            id: `assistant:${message.messageId}`,
            sequence: sequences.byMessage.get(message.messageId) ?? message.sequence,
            text: message.text,
            streaming: isActiveRunState(run.state) && !message.finalized,
            deliveryError: run.deliveryErrors.find((error) => error.messageId === message.messageId) ?? null,
        });
    }
    for (const toolCall of run.toolCalls) {
        const sequence = sequences.byTool.get(toolCall.toolCallId);
        if (!visibleTool(toolCall, sequence, terminalState ? terminal : null)) continue;
        entries.push({ kind: 'tool', id: `tool:${toolCall.toolCallId}`, sequence: sequence ?? 0, toolCall });
    }
    for (const unknown of run.unknownEvents) {
        if (terminalState && terminal !== null && unknown.sequence > terminal) continue;
        entries.push({ kind: 'unknown', id: `unknown:${unknown.eventId}`, sequence: unknown.sequence, type: unknown.type });
    }
    const usage = usageEntry(run);
    if (usage !== null) entries.push(usage);
    const usedSequences = new Set<number>();
    for (let index = 0; index < run.deliveryErrors.length; index += 1) {
        const error = run.deliveryErrors[index]!;
        let sequence = deliverySequence(run, error.message, usedSequences);
        if (usedSequences.has(sequence)) sequence = terminal ?? sequence;
        usedSequences.add(sequence);
        if (error.messageId !== null && messageIds.has(error.messageId)) continue;
        entries.push({ kind: 'delivery', id: `delivery:${index}:${error.messageId ?? 'unknown'}`, sequence, messageId: error.messageId, message: error.message });
    }
    return entries.sort((left, right) => left.sequence - right.sequence);
}

function approvalEntries(run: RunView, hostIdentity: ApprovalHost | null, client: AgentClient): TimelineApproval[] {
    const terminal = isTerminalRunState(run.state);
    return run.approvals
        .filter((approval) => !terminal || approval.decision !== null)
        .map((approval) => ({
            approval,
            hostIdentity,
            resolvable: client.approvalResolvable(approval.approvalId),
        }));
}

function planData(run: RunView | null): { title: string; steps: Array<{ step_id: string; label: string; state: string }> } | null {
    if (run === null) return null;
    const planEvent = [...run.events].reverse().find((event) => event.type === 'plan.updated');
    if (planEvent === undefined) return null;
    const data = planEvent.data;
    if (typeof data.title !== 'string' || !Array.isArray(data.steps)) return null;
    const steps = data.steps.filter((step): step is { step_id: string; label: string; state: string } =>
        step !== null && typeof step === 'object'
        && typeof (step as { step_id?: unknown }).step_id === 'string'
        && typeof (step as { label?: unknown }).label === 'string'
        && typeof (step as { state?: unknown }).state === 'string');
    return { title: data.title, steps };
}

export function TaskDetail({ task, client, onBack, onNewRun, onOpenSurface }: TaskDetailProps) {
    const run = latestRun(task);
    const headingRef = useRef<HTMLHeadingElement>(null);
    const [hostIdentity, setHostIdentity] = useState<ApprovalHost | null>(null);
    const [stopPending, setStopPending] = useState(false);
    const [retrying, setRetrying] = useState(false);
    const [actionError, setActionError] = useState<{ code: string; message: string } | null>(null);

    useEffect(() => {
        let active = true;
        void client.hostCapabilities().then((capabilities) => {
            if (active) setHostIdentity(capabilities.approvals ? { id: 'Local companion', canApprove: true } : null);
        }, () => {
            if (active) setHostIdentity(null);
        });
        return () => { active = false; };
    }, [client]);

    useEffect(() => {
        headingRef.current?.focus();
    }, [task.taskId]);

    useEffect(() => {
        if (run === null || isTerminalRunState(run.state)) setStopPending(false);
    }, [run?.state]);

    const entries = useMemo(() => run === null ? [] : timelineEntries(run), [run]);
    const approvals = useMemo(() => run === null ? [] : approvalEntries(run, hostIdentity, client), [run, hostIdentity, client]);
    const plan = useMemo(() => planData(run), [run]);
    const retryable = run !== null && (run.state === 'cancelled' || (run.state === 'failed' && run.error?.retryable === true));
    // Terminal events are authoritative; only a non-terminal cancelling
    // projection changes the displayed state. The rail uses the same helper.
    const status = run === null ? task.state : displayRunState(run);
    const active = run !== null && isActiveRunState(status);
    const busy = active && (status === 'running' || status === 'queued');
    const statusDetail = run === null
        ? null
        : status === 'waiting_for_approval'
            ? `for ${run.approvals.find((approval) => approval.decision === null)?.tool ?? 'host approval'}`
            : status === 'waiting_for_background'
                ? 'for background work'
                : status === 'cancelling'
                    ? 'waiting for host acknowledgement'
                    : status === 'queued' ? 'waiting to start' : null;

    const stop = () => {
        if (run === null || stopPending) return;
        setStopPending(true);
        setActionError(null);
        void client.stop({ taskId: task.taskId, runId: run.runId }).then((receipt) => {
            if (!receipt.acknowledged) {
                setStopPending(false);
                setActionError({ code: 'stop_unacknowledged', message: 'The host did not acknowledge the stop request.' });
            }
        }, (failure: unknown) => {
            setStopPending(false);
            setActionError(errorView(failure));
        });
    };

    const retry = () => {
        if (run === null || retrying) return;
        setRetrying(true);
        setActionError(null);
        void client.retry({ taskId: task.taskId, runId: run.runId }).catch((failure: unknown) => {
            setActionError(errorView(failure));
        }).finally(() => setRetrying(false));
    };

    const resolve = (approvalId: string, decision: 'approved' | 'denied') => {
        setActionError(null);
        void client.resolveApproval({ approvalId, decision }).catch((failure: unknown) => setActionError(errorView(failure)));
    };

    return <section class="task-detail" aria-label="Task detail">
        <header class="task-detail-header" data-task-detail-header>
            <button type="button" class="task-detail-back" onClick={onBack}>Back to tasks</button>
            <p class="task-detail-status">Status: {runStateLabel(status)}</p>
            <h1 ref={headingRef} tabIndex={-1} class="task-detail-title">{task.title}</h1>
            <dl class="task-detail-facts">
                <div><dt>Title</dt><dd>{task.title}</dd></div>
                <div><dt>Starting prompt</dt><dd>Starting prompt not recorded</dd></div>
                <div><dt>Project</dt><dd>{task.projectId}</dd></div>
                <div><dt>Repo</dt><dd>{task.repo ?? 'Not set'}</dd></div>
                <div><dt>Branch</dt><dd>{task.branch ?? 'Not set'}</dd></div>
                <div><dt>Environment</dt><dd>{environmentLabel(run)}</dd></div>
                <div><dt>Model</dt><dd>{modelLabel(run)}</dd></div>
            </dl>
            <div class="task-detail-actions">
                <button type="button" onClick={() => onOpenSurface('changes')}>Open changes</button>
                <button type="button" onClick={() => onOpenSurface('terminal')}>Open terminal</button>
                {run !== null && isTerminalRunState(run.state) && !task.archived && task.hold === null && onNewRun !== undefined && <button type="button" class="task-new-run" onClick={onNewRun}>New run</button>}
            </div>
            {isSynthetic(run) && <p class="task-synthetic">Synthetic replay: deterministic fixture output, not an R4 model or a task brain.</p>}
        </header>
        <RunStatus state={status} detail={statusDetail} stopping={stopPending} stopDisabled={retrying} onStop={active ? stop : undefined} />
        {task.state === 'waiting_for_background' && task.hold !== null && <p class="task-service-state" data-static-background>Background and cross-device tasks require an optional service. Closing this tab stops local work.</p>}
        {plan !== null && <section class="task-plan" data-task-plan aria-label="Plan">
            <h2>{plan.title}</h2>
            <ol class="task-plan-steps">
                {plan.steps.map((step) => <li key={step.step_id} data-step-state={step.state} data-step-id={step.step_id}>{step.label}<span class="task-plan-state">{step.state}</span></li>)}
            </ol>
        </section>}
        {run?.error != null && <TaskError code={run.error.code} message={run.error.message} />}
        {retryable && <div class="task-run-actions"><button type="button" class="retry-run" disabled={retrying} onClick={retry}>Retry</button></div>}
        {actionError !== null && <TaskError scope="action" code={actionError.code} message={actionError.message} />}
        <div class="agent-transcript task-detail-transcript" aria-label="Task transcript" aria-live="polite" aria-busy={busy ? 'true' : 'false'}>
            <TaskTimeline events={entries} approvals={approvals} onResolve={resolve} />
        </div>
    </section>;
}
