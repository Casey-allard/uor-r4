interface TaskSearchProps {
    value: string;
    onInput(value: string): void;
    resultCount: number;
    totalCount: number;
}

export function TaskSearch({ value, onInput, resultCount, totalCount }: TaskSearchProps) {
    const active = value.trim().length > 0;
    return <div class="task-search">
        <label class="task-search-label">
            <span>Search tasks</span>
            <input
                type="search"
                aria-label="Search tasks"
                placeholder="Title, project, repo or status"
                value={value}
                onInput={(event) => onInput(event.currentTarget.value)}
            />
        </label>
        {active && <p class="task-search-count" aria-live="polite">{resultCount} of {totalCount} {totalCount === 1 ? 'task' : 'tasks'}</p>}
    </div>;
}
