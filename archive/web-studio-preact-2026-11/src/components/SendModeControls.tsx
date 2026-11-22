import type { RunSummary } from '../agent/client.js';

const TERMINAL_RUN_STATES: ReadonlySet<string> = new Set([
    'finished',
    'failed',
    'cancelled',
    'expired',
]);

export function isActiveRun(run: { state: string } | null | undefined): boolean {
    return run !== null && run !== undefined && !TERMINAL_RUN_STATES.has(run.state);
}

export interface SendModeControlsProps {
    activeRun: RunSummary | null;
    onQueue(): void;
    onSteer(): void;
    onForce(): void;
    onStop(): void;
}

function disabledHint(activeRun: RunSummary | null): string {
    return activeRun === null
        ? 'Select a task with an active run to use queue, steer, force send or stop run.'
        : 'This task run is not active.';
}

export function SendModeControls({ activeRun, onQueue, onSteer, onForce, onStop }: SendModeControlsProps) {
    const enabled = isActiveRun(activeRun);
    const hint = disabledHint(activeRun);
    const canStop = enabled && activeRun!.state !== 'cancelling';
    const stopTitle = !enabled
        ? hint
        : activeRun!.state === 'cancelling'
            ? 'This task run is already cancelling.'
            : 'Stop the active task run using its actual run id.';
    const title = (available: string) => enabled ? available : hint;
    return <div class="composer-run-modes" role="group" aria-label="Run message modes">
        <button type="button" disabled={!enabled} title={title('Queue this message for the task\'s next turn.')} onClick={onQueue}>Queue</button>
        <button type="button" disabled={!enabled} title={title('Steer the active task run at its next tool boundary.')} onClick={onSteer}>Steer</button>
        <button type="button" disabled={!enabled} title={title('Force this message into the active task run at its next tool boundary.')} onClick={onForce}>Force send</button>
        <button type="button" disabled={!canStop} title={stopTitle} onClick={onStop}>Stop run</button>
    </div>;
}
