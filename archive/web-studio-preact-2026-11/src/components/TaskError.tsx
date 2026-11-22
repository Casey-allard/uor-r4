export type TaskErrorScope = 'run' | 'action';

export interface TaskErrorProps {
    code: string;
    message: string;
    scope?: TaskErrorScope;
}

export function TaskError({ code, message, scope = 'run' }: TaskErrorProps) {
    return <section class="task-error" data-task-error={scope} role="alert">
        <p class="task-error-code">{code}</p>
        <p class="task-error-message">{message}</p>
    </section>;
}
