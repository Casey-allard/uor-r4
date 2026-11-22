export interface OperationGate {
    run<T>(operation: string, action: Promise<T>, onSuccess: (operation: string) => void, onError: (error: unknown, operation: string) => void): Promise<T | undefined>;
}

export function createOperationGate(): OperationGate {
    const versions = new Map<string, number>();
    return {
        async run(operation, action, onSuccess, onError) {
            const version = (versions.get(operation) || 0) + 1;
            versions.set(operation, version);
            try {
                const result = await action;
                if (versions.get(operation) === version) onSuccess(operation);
                return result;
            } catch (error) {
                if (versions.get(operation) === version) onError(error, operation);
                return undefined;
            }
        },
    };
}
