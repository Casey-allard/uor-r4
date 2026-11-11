import type { ComponentChildren } from 'preact';
import type { TaskSummary } from '../agent/client.js';
import { TaskGroup } from './TaskGroup.js';
import { TaskSearch } from './TaskSearch.js';

export const LOCAL_SESSIONS_LABEL_ID = 'task-rail-local-sessions';

export interface TaskRailProps {
    tasks: TaskSummary[];
    query: string;
    onQuery(value: string): void;
    onSelect(taskId: string): void;
    onPin(taskId: string, pinned: boolean): void;
    onArchive(taskId: string, archived: boolean): void;
    selectedTaskId: string | null;
    onCreate?(): void;
    createPending?: boolean;
    nowMs?: number;
    runtimeUnavailable?: boolean;
    loading?: boolean;
    error?: string | null;
    showEmpty?: boolean;
    legacySessions?: ComponentChildren;
    legacySessionsEmpty?: boolean;
    collapseLocalSessions?: boolean;
}

export interface TaskBuckets {
    today: TaskSummary[];
    lastSevenDays: TaskSummary[];
    older: TaskSummary[];
    archived: TaskSummary[];
}

function startOfLocalDay(value: number): number {
    const date = new Date(value);
    date.setHours(0, 0, 0, 0);
    return date.getTime();
}

function startOfLocalDayDaysAgo(value: number, days: number): number {
    const date = new Date(value);
    date.setHours(0, 0, 0, 0);
    date.setDate(date.getDate() - days);
    return date.getTime();
}

function searchText(task: TaskSummary): string {
    return [task.title, task.projectId, task.repo ?? '', task.branch ?? '', task.state].join('\n').toLowerCase();
}

export function filterTasks(tasks: TaskSummary[], query: string): TaskSummary[] {
    const needle = query.trim().toLowerCase();
    if (needle.length === 0) return tasks;
    return tasks.filter((task) => searchText(task).includes(needle));
}

function compareTasks(left: TaskSummary, right: TaskSummary): number {
    if (left.pinned !== right.pinned) return left.pinned ? -1 : 1;
    if (left.createdAtMs !== right.createdAtMs) return right.createdAtMs - left.createdAtMs;
    return left.title.localeCompare(right.title);
}

// Today is the current local calendar day. Last 7 days is the seven full
// local calendar days before today; Older is anything earlier. Day boundaries
// use local midnight, so DST changes do not stretch a bucket by an hour.
export function groupTasks(tasks: TaskSummary[], nowMs: number): TaskBuckets {
    const todayStart = startOfLocalDay(nowMs);
    const sevenDaysStart = startOfLocalDayDaysAgo(nowMs, 7);
    const buckets: TaskBuckets = { today: [], lastSevenDays: [], older: [], archived: [] };
    for (const task of [...tasks].sort(compareTasks)) {
        if (task.archived || task.state === 'archived') buckets.archived.push(task);
        else if (task.createdAtMs >= todayStart) buckets.today.push(task);
        else if (task.createdAtMs >= sevenDaysStart) buckets.lastSevenDays.push(task);
        else buckets.older.push(task);
    }
    return buckets;
}

export function TaskRail({
    tasks,
    query,
    onQuery,
    onSelect,
    onPin,
    onArchive,
    selectedTaskId,
    onCreate,
    createPending = false,
    nowMs = Date.now(),
    runtimeUnavailable = false,
    loading = false,
    error = null,
    showEmpty = true,
    legacySessions,
    legacySessionsEmpty = false,
    collapseLocalSessions = false,
}: TaskRailProps) {
    const displayTasks = tasks.map((task) =>
        task.unresolvedRunId ||
        (task.latestRunId !== null && task.latestRunLoaded === false)
            ? { ...task, state: 'restoring' as TaskSummary['state'] }
            : task,
    );
    const visible = filterTasks(displayTasks, query);
    const buckets = groupTasks(visible, nowMs);
    const activeCount = buckets.today.length + buckets.lastSevenDays.length + buckets.older.length;
    const total = displayTasks.length;
    const empty = total === 0;
    const noMatches = !empty && visible.length === 0;

    return <nav aria-label="Tasks and local sessions" class="task-rail">
        <header class="task-rail-header">
            <h2 class="sidebar-label">Tasks</h2>
            {onCreate !== undefined && !runtimeUnavailable && error === null && <div class="task-create">
                <button type="button" class="task-new" data-action="new-task" disabled={createPending} onClick={onCreate}>{createPending ? 'Creating…' : 'New task'}</button>
                <p class="task-create-hint">Runs use a deterministic synthetic replay fixture, not an R4 model.</p>
            </div>}
            <p class="task-pin-scope">Pins stay on this device.</p>
        </header>
        <TaskSearch value={query} onInput={onQuery} resultCount={visible.length} totalCount={total} />
        {loading && <p class="task-loading" role="status">Loading tasks…</p>}
        {runtimeUnavailable && <p class="task-runtime-error" role="alert">The task runtime is not initialized. Reload the page.</p>}
        {error !== null && <p class="task-load-error" role="alert">{error}</p>}
        {empty && showEmpty && !loading && !runtimeUnavailable && error === null && <p class="task-empty">No tasks yet. Tasks appear here when this project records one.</p>}
        {noMatches && <p class="task-empty" role="status">No tasks match “{query.trim()}”.</p>}
        {!empty && activeCount > 0 && <div class="task-groups">
            {buckets.today.length > 0 && <TaskGroup kind="today" label="Today" tasks={buckets.today} selectedTaskId={selectedTaskId} onSelect={onSelect} onPin={onPin} onArchive={onArchive} />}
            {buckets.lastSevenDays.length > 0 && <TaskGroup kind="last-7-days" label="Last 7 days" tasks={buckets.lastSevenDays} selectedTaskId={selectedTaskId} onSelect={onSelect} onPin={onPin} onArchive={onArchive} />}
            {buckets.older.length > 0 && <TaskGroup kind="older" label="Older" tasks={buckets.older} selectedTaskId={selectedTaskId} onSelect={onSelect} onPin={onPin} onArchive={onArchive} collapsed />}
        </div>}
        {buckets.archived.length > 0 && <TaskGroup kind="archived" label="Archived" tasks={buckets.archived} selectedTaskId={selectedTaskId} onSelect={onSelect} onPin={onPin} onArchive={onArchive} collapsed />}
        {legacySessions != null && (collapseLocalSessions
            ? <details class="task-group task-legacy-sessions" data-task-group="local-sessions">
                <summary><h3 id={LOCAL_SESSIONS_LABEL_ID}>Local sessions</h3></summary>
                {legacySessionsEmpty && <p class="task-empty">No sessions yet</p>}
                {legacySessions}
            </details>
            : <div class="task-group task-legacy-sessions" data-task-group="local-sessions">
                <header class="task-group-header">
                    <h3 id={LOCAL_SESSIONS_LABEL_ID}>Local sessions</h3>
                </header>
                {legacySessionsEmpty && <p class="task-empty">No sessions yet</p>}
                {legacySessions}
            </div>)}
    </nav>;
}
