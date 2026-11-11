import type { TaskSummary } from '../agent/client.js';

export type TaskGroupKind = 'today' | 'last-7-days' | 'older' | 'archived';

interface TaskGroupProps {
    kind: TaskGroupKind;
    label: string;
    tasks: TaskSummary[];
    selectedTaskId: string | null;
    onSelect(taskId: string): void;
    onPin(taskId: string, pinned: boolean): void;
    onArchive(taskId: string, archived: boolean): void;
    collapsed?: boolean;
}

function taskCountLabel(count: number): string {
    return `${count} ${count === 1 ? 'task' : 'tasks'}`;
}

function stateLabel(state: TaskSummary['state']): string {
    switch (state) {
        case 'waiting_for_approval': return 'Waiting for approval';
        case 'waiting_for_background': return 'Waiting for background';
        case 'archived': return 'Archived';
        case 'idle': return 'Idle';
        default: return state.charAt(0).toUpperCase() + state.slice(1);
    }
}

function projectLabel(task: TaskSummary): string {
    return task.repo ?? task.projectId;
}

interface TaskRowProps {
    task: TaskSummary;
    selected: boolean;
    archived: boolean;
    onSelect(taskId: string): void;
    onPin(taskId: string, pinned: boolean): void;
    onArchive(taskId: string, archived: boolean): void;
}

function TaskRow({ task, selected, archived, onSelect, onPin, onArchive }: TaskRowProps) {
    return <li data-task-id={task.taskId} aria-current={selected ? 'true' : undefined} class="task-row">
        <button type="button" class="task-select" aria-label={`Open ${task.title}`} onClick={() => onSelect(task.taskId)}>
            <strong>{task.title}</strong>
        </button>
        <div class="task-meta">
            <span class="task-project" title={task.projectId}>{projectLabel(task)}</span>
            <span class="task-state">{stateLabel(task.state)}</span>
        </div>
        <div class="task-actions">
            {!archived && <button
                type="button"
                class="task-pin"
                aria-pressed={task.pinned}
                aria-label={`${task.pinned ? 'Unpin' : 'Pin'} ${task.title} on this device`}
                onClick={() => onPin(task.taskId, !task.pinned)}
            >{task.pinned ? 'This device' : 'Pin'}</button>}
            <button
                type="button"
                class="task-archive"
                aria-label={`${archived ? 'Restore' : 'Archive'} ${task.title}`}
                onClick={() => onArchive(task.taskId, !archived)}
            >{archived ? 'Restore' : 'Archive'}</button>
        </div>
    </li>;
}

function TaskRows({ tasks, selectedTaskId, archived, onSelect, onPin, onArchive }: {
    tasks: TaskSummary[];
    selectedTaskId: string | null;
    archived: boolean;
    onSelect(taskId: string): void;
    onPin(taskId: string, pinned: boolean): void;
    onArchive(taskId: string, archived: boolean): void;
}) {
    return <ul class="task-list">
        {tasks.map((task) => <TaskRow
            key={task.taskId}
            task={task}
            selected={task.taskId === selectedTaskId}
            archived={archived}
            onSelect={onSelect}
            onPin={onPin}
            onArchive={onArchive}
        />)}
    </ul>;
}

export function TaskGroup({ kind, label, tasks, selectedTaskId, onSelect, onPin, onArchive, collapsed = false }: TaskGroupProps) {
    const archived = kind === 'archived';
    if (collapsed) {
        return <details class="task-group" data-task-group={kind}>
            <summary>
                <span data-group-label role="heading" aria-level={3}>{label}</span>{' '}
                <span class="task-group-count">{taskCountLabel(tasks.length)}</span>
            </summary>
            <TaskRows tasks={tasks} selectedTaskId={selectedTaskId} archived={archived} onSelect={onSelect} onPin={onPin} onArchive={onArchive} />
        </details>;
    }
    return <section class="task-group" data-task-group={kind} aria-labelledby={`task-group-${kind}`}>
        <header class="task-group-header">
            <h3 id={`task-group-${kind}`} data-group-label>{label}</h3>
            <span class="task-group-count">{taskCountLabel(tasks.length)}</span>
        </header>
        <TaskRows tasks={tasks} selectedTaskId={selectedTaskId} archived={archived} onSelect={onSelect} onPin={onPin} onArchive={onArchive} />
    </section>;
}
