import type { AgentApprovalView } from '../state/agent.js';
import { ApprovalCard, type ApprovalHost } from './ApprovalCard.js';
import { TimelineEvent, type TimelineEntry } from './TimelineEvent.js';

export type { TimelineEntry } from './TimelineEvent.js';

export interface TimelineApproval {
    approval: AgentApprovalView;
    hostIdentity: ApprovalHost | null;
    resolvable?: boolean;
}

export interface TaskTimelineProps {
    events: TimelineEntry[];
    approvals: TimelineApproval[];
    onResolve(approvalId: string, decision: 'approved' | 'denied'): void;
}

export function TaskTimeline({ events, approvals, onResolve }: TaskTimelineProps) {
    const ordered = [...events].sort((left, right) => left.sequence - right.sequence);
    const approvalsByTool = new Map<string, TimelineApproval[]>();
    for (const entry of approvals) {
        const list = approvalsByTool.get(entry.approval.toolCallId) ?? [];
        list.push(entry);
        approvalsByTool.set(entry.approval.toolCallId, list);
    }
    const renderedTools = new Set(ordered.filter((entry) => entry.kind === 'tool').map((entry) => entry.toolCall.toolCallId));
    const unmatched = approvals.filter((entry) => !renderedTools.has(entry.approval.toolCallId));
    if (ordered.length === 0 && approvals.length === 0) {
        return <div class="task-timeline" data-task-timeline="empty"><p class="task-timeline-empty">No runtime events for this task yet.</p></div>;
    }
    return <div class="task-timeline">
        {ordered.map((entry) => entry.kind === 'tool'
            ? <TimelineEvent key={entry.id} event={entry}>
                {(approvalsByTool.get(entry.toolCall.toolCallId) ?? []).map((approval) => <ApprovalCard key={approval.approval.approvalId} approval={approval.approval} hostIdentity={approval.hostIdentity} resolvable={approval.resolvable} onResolve={onResolve} />)}
            </TimelineEvent>
            : <TimelineEvent key={entry.id} event={entry} />)}
        {unmatched.map((entry) => <ApprovalCard key={entry.approval.approvalId} approval={entry.approval} hostIdentity={entry.hostIdentity} resolvable={entry.resolvable} onResolve={onResolve} />)}
    </div>;
}
