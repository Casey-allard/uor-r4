import { LOAD_PROGRESS_EVENT } from '../../assets/js/model_sources.js';
import type { LoadOperation, LoadResult, ModelSourcesApi } from '../../assets/js/model_sources.js';
import { classifyEndpoint, LifecycleStore } from '../state/lifecycle.js';

export { LOAD_PROGRESS_EVENT };

export interface WorkerLike {
    postMessage(message: unknown): void;
}

export interface WorkerCapabilities {
    schema?: string;
    file_name?: string;
    artifact_sha256?: string;
    sha256_verified?: boolean | null;
    engine_supported?: boolean;
    can_generate?: boolean;
    can_predict?: boolean;
    source?: string;
}

export interface WorkerMessage {
    action: string;
    id?: number;
    capabilities?: WorkerCapabilities;
    error?: string;
}

export interface ModelSourceAdapter {
    loadPublished(idOrEntry: unknown): Promise<LoadResult>;
    loadFromUrl(rawUrl: string): Promise<LoadResult>;
    loadLocalFile(file: File): Promise<LoadResult>;
    loadBundled(): Promise<LoadResult>;
    selectApi(sourceId: string, endpoint: string): void;
    testApi(id?: string): Promise<unknown>;
    handleWorkerMessage(message: WorkerMessage): void;
}

interface PendingLoad {
    operation: LoadOperation;
    result: LoadResult;
}

function sourceKind(result: LoadResult): string {
    const kind = result.meta.kind;
    if (kind === 'bundled') return 'bundled';
    if (kind === 'url' || kind === 'published') return 'url';
    return 'file';
}

export function createModelSourceAdapter(
    store: LifecycleStore,
    sources: ModelSourcesApi,
    worker: WorkerLike,
): ModelSourceAdapter {
    let pending: PendingLoad | null = null;
    let finishLoad: (() => void) | null = null;
    let workerQueue: Promise<void> = Promise.resolve();

    function start(sourceId: string): LoadOperation {
        store.dispatch({ type: 'artifact/selected', sourceId });
        const operation = store.getState().operation;
        if (!operation) throw new Error('artifact selection did not create an operation');
        return operation;
    }

    function isCurrent(operation: LoadOperation): boolean {
        const current = store.getState().operation;
        const selected = store.getState().selectedSource;
        return Boolean(
            current
            && selected?.kind === 'artifact'
            && current.operationId === operation.operationId
            && current.sourceId === operation.sourceId
            && selected.id === operation.sourceId,
        );
    }

    function errorMessage(error: unknown): string {
        return error instanceof Error ? error.message : String(error);
    }

    function failCurrent(operation: LoadOperation, error: unknown): void {
        if (isCurrent(operation)) {
            store.dispatch({ type: 'artifact/failed', operation, error: errorMessage(error) });
        }
    }

    function postToWorker(result: LoadResult, operation: LoadOperation): LoadResult {
        if (result.stale || !isCurrent(operation)) return { ...result, stale: true };
        workerQueue = workerQueue.then(async () => {
            if (!isCurrent(operation)) return;
            store.dispatch({ type: 'artifact/finalizing', operation });
            pending = { operation, result };
            const completed = new Promise<void>((resolve) => {
                finishLoad = resolve;
            });
            worker.postMessage({
                action: 'load_model',
                id: operation.operationId,
                payload: {
                    bytes: result.bytes,
                    fileName: result.fileName,
                    source: sourceKind(result),
                    sourceLabel: result.meta.label || result.meta.fileName || '',
                    expectedSha256: result.expectedSha256,
                    tokenizerArtifact: result.tokenizerArtifact,
                    operationId: operation.operationId,
                },
            });
            await completed;
        }, () => undefined);
        return result;
    }

    async function attempt(operation: LoadOperation, load: () => Promise<LoadResult>): Promise<LoadResult> {
        try {
            return postToWorker(await load(), operation);
        } catch (error) {
            failCurrent(operation, error);
            throw error;
        }
    }

    document.addEventListener(LOAD_PROGRESS_EVENT, (event) => {
        const detail = (event as CustomEvent<{
            operationId?: number;
            loaded: number;
            total: number | null;
            phase: 'downloading' | 'finalizing';
        }>).detail;
        const operation = store.getState().operation;
        if (!operation || detail.operationId !== operation.operationId) return;
        if (detail.phase === 'downloading') {
            store.dispatch({ type: 'artifact/downloadProgress', operation, loaded: detail.loaded, total: detail.total });
        } else {
            store.dispatch({ type: 'artifact/finalizing', operation });
        }
    });

    return {
        async loadPublished(idOrEntry) {
            const operation = start(typeof idOrEntry === 'string' ? idOrEntry : 'published');
            return attempt(operation, () => sources.loadPublished(idOrEntry, operation));
        },
        async loadFromUrl(rawUrl) {
            const operation = start('url');
            return attempt(operation, () => sources.loadFromUrl(rawUrl, operation));
        },
        async loadLocalFile(file) {
            const operation = start('local-file');
            return attempt(operation, () => sources.loadLocalFile(file, operation));
        },
        async loadBundled() {
            const operation = start('bundled-demo');
            return attempt(operation, () => sources.loadBundled(operation));
        },
        selectApi(sourceId, endpoint) {
            classifyEndpoint(endpoint);
            sources.setActive({ kind: 'api', id: sourceId });
            store.dispatch({ type: 'api/selected', sourceId, endpoint });
        },
        async testApi(id) {
            const selected = store.getState().selectedSource;
            const targetId = id ?? selected?.id;
            if (selected?.kind !== 'api' || !targetId || selected.id !== targetId) throw new Error('Cannot test a non-selected API source');
            const endpoint = selected.endpoint;
            const finish = (ok: boolean) => {
                const current = store.getState().selectedSource;
                if (current?.kind === 'api' && current.id === targetId && current.endpoint === endpoint) {
                    store.dispatch({ type: 'api/testFinished', sourceId: targetId, ok });
                }
            };
            store.dispatch({ type: 'api/testStarted', sourceId: targetId });
            try {
                const result = await sources.testApi(targetId);
                finish(result.ok);
                return result;
            } catch (error) {
                finish(false);
                throw error;
            }
        },
        handleWorkerMessage(message) {
            const current = pending;
            if (!current) return;
            const operation = current.operation;
            const complete = () => {
                pending = null;
                const finish = finishLoad;
                finishLoad = null;
                finish?.();
            };
            if (message.action === 'model_loaded') {
                const capabilities = message.capabilities;
                if (capabilities?.file_name && capabilities.file_name !== current.result.fileName) return;
                if (current.result.expectedSha256 && capabilities?.artifact_sha256 && capabilities.artifact_sha256 !== current.result.expectedSha256) return;
                complete();
                if (isCurrent(operation)) {
                    store.dispatch({
                        type: 'artifact/ready',
                        operation,
                        artifact: {
                            sourceId: operation.sourceId,
                            schema: capabilities?.schema || '',
                            fileName: capabilities?.file_name || current.result.fileName,
                            sha256: capabilities?.artifact_sha256,
                            verified: capabilities?.sha256_verified ?? null,
                            engineSupported: capabilities?.engine_supported,
                            canGenerate: capabilities?.can_generate,
                            canPredict: capabilities?.can_predict,
                            source: capabilities?.source,
                        },
                    });
                }
                return;
            }
            if (message.action === 'operation_error' && message.id === operation.operationId) {
                complete();
                failCurrent(operation, message.error || 'Operation failed.');
            }
        },
    };
}
