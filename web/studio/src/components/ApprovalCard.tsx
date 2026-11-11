import type { AgentApprovalView } from '../state/agent.js';

export interface ApprovalHost {
    id: string;
    canApprove: boolean;
}

export interface ApprovalCardProps {
    approval: AgentApprovalView;
    hostIdentity: ApprovalHost | null;
    resolvable?: boolean;
    onResolve(approvalId: string, decision: 'approved' | 'denied'): void;
}

function expiryISO(expiresAtMs: number): string | null {
    if (!Number.isFinite(expiresAtMs)) return null;
    const date = new Date(expiresAtMs);
    return Number.isNaN(date.getTime()) ? null : date.toISOString();
}

function decisionLabel(decision: AgentApprovalView['decision']): string | null {
    if (decision === 'approved') return 'Approved by the host.';
    if (decision === 'denied') return 'Denied by the host.';
    if (decision === 'expired') return 'This approval expired before the host answered.';
    return null;
}

// The page is never the approval authority. The only evidence that can enable
// a decision is a host capability handshake carried by hostIdentity; without
// it the controls stay disabled and the exact companion guidance is shown.
export function ApprovalCard({ approval, hostIdentity, resolvable = true, onResolve }: ApprovalCardProps) {
    const resolved = decisionLabel(approval.decision);
    const hostCanApprove = hostIdentity?.canApprove === true;
    const canResolve = resolved === null && hostCanApprove && resolvable;
    const ownerless = resolved === null && hostCanApprove && !resolvable;
    const expiry = expiryISO(approval.expiresAtMs);
    return <article class="approval-card" data-approval-id={approval.approvalId} aria-label="Approval required">
        <header class="approval-heading"><h2>Approval required</h2><span class="approval-state">{resolved === null ? 'Pending host decision' : resolved}</span></header>
        <dl class="approval-facts">
            <div><dt>Tool</dt><dd>{approval.tool}</dd></div>
            <div><dt>Scope</dt><dd>{approval.scope}</dd></div>
            <div><dt>Expiry</dt><dd>{expiry === null ? <span data-approval-expiry="unrecorded">Not recorded</span> : <time dateTime={expiry}>{expiry}</time>}</dd></div>
        </dl>
        <p class="approval-host">Host: {hostIdentity === null ? 'not connected' : hostIdentity.id}</p>
        {!canResolve && resolved === null && <p class="approval-guidance">{ownerless ? 'This approval has no active run owner in this page. Stop the restored run before starting a new run.' : 'This action needs host approval. Approve it in the local companion.'}</p>}
        {resolved === null && <div class="approval-actions">
            <button type="button" disabled={!canResolve} onClick={() => onResolve(approval.approvalId, 'approved')}>Approve</button>
            <button type="button" disabled={!canResolve} onClick={() => onResolve(approval.approvalId, 'denied')}>Deny</button>
        </div>}
    </article>;
}
