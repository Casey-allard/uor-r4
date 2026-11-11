import type { RunState } from '../agent/generated/agentContract.js';

const TERMINAL_STATES: ReadonlySet<string> = new Set(['finished', 'failed', 'cancelled', 'expired']);

const STATE_LABELS: Record<string, string> = {
    idle: 'Idle',
    queued: 'Queued',
    running: 'Running',
    waiting_for_approval: 'Waiting for approval',
    waiting_for_background: 'Waiting for background',
    cancelling: 'Cancelling',
    finished: 'Finished',
    failed: 'Failed',
    cancelled: 'Cancelled',
    expired: 'Expired',
    archived: 'Archived',
    restoring: 'Restoring',
};

export function isTerminalRunState(state: RunState | string | null | undefined): boolean {
    return state !== null && state !== undefined && TERMINAL_STATES.has(state);
}

export function isActiveRunState(state: RunState | string | null | undefined): boolean {
    return state !== null && state !== undefined && !TERMINAL_STATES.has(state);
}

export function runStateLabel(state: RunState | string): string {
    return STATE_LABELS[state] ?? state;
}

export interface RunStatusProps {
    state: RunState | string;
    detail?: string | null;
    stopping?: boolean;
    stopDisabled?: boolean;
    onStop?(): void;
}

// One polite status line for the run. Assistant deltas never get their own
// live region; this line and the transcript's aria-busy state carry progress.
export function RunStatus({ state, detail = null, stopping = false, stopDisabled = false, onStop }: RunStatusProps) {
    const active = isActiveRunState(state);
    const cancelling = stopping || state === 'cancelling';
    const label = runStateLabel(state);
    return <section class="run-status" data-run-state={state} role="status" aria-live="polite" aria-atomic="true">
        <div class="run-status-line">
            <span class="run-status-label">{label}</span>
            {detail !== null && <span class="run-status-detail">{detail}</span>}
            {active && state !== 'cancelling' && <progress class="run-progress" aria-label="Run in progress" />}
        </div>
        {onStop && active && <button type="button" class="run-stop" disabled={cancelling || stopDisabled} onClick={onStop}>{cancelling ? 'Cancelling…' : 'Stop'}</button>}
    </section>;
}
