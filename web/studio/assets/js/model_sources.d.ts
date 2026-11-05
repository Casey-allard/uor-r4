export const LOAD_PROGRESS_EVENT: 'uor-model-load-progress';

export interface LoadOperation {
    operationId: number;
    sourceId: string;
}

export interface LoadResult {
    stale?: boolean;
    bytes: Uint8Array;
    fileName: string;
    expectedSha256?: string;
    tokenizerArtifact?: unknown;
    meta: {
        kind?: string;
        label?: string;
        fileName?: string;
        schema: string;
    };
}

export interface ApiProvider {
    id: string;
    label: string;
    preset: string;
    baseUrl: string;
    model: string;
    apiKey: string;
    ephemeral?: boolean;
    temperature?: number;
}

export interface ModelSourceList {
    active: { kind: string; id: string | null };
    wasm: Record<string, unknown>;
    api: ApiProvider[];
}

export interface ModelSourcesApi {
    catalog(): { models: Array<{ id?: string; label?: string; url: string; note?: string; sha256?: string }> };
    loadModelCatalog(): Promise<void>;
    list(): ModelSourceList;
    active(): { kind: string; id: string | null };
    apiConfig(id?: string): ApiProvider | null;
    setActive(next: { kind: string; id: string }): unknown;
    addApi(input: Partial<ApiProvider>): ApiProvider;
    updateApi(id: string, patch: Partial<ApiProvider>): ApiProvider;
    removeApi(id: string): ModelSourceList;
    loadPublished(idOrEntry: unknown, operation?: LoadOperation): Promise<LoadResult>;
    loadFromUrl(rawUrl: string, operation?: LoadOperation): Promise<LoadResult>;
    loadLocalFile(file: File, operation?: LoadOperation): Promise<LoadResult>;
    loadBundled(operation?: LoadOperation): Promise<LoadResult>;
    testApi(id?: string): Promise<{ ok: boolean; detail?: string; models?: unknown[] }>;
    mount?(container: HTMLElement): { render(): void; unmount(): void };
}

export const ModelSources: ModelSourcesApi;
