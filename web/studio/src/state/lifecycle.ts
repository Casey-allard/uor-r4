export type SourceKind = 'artifact' | 'api';
export type ArtifactPhase = 'unselected' | 'selected' | 'downloading' | 'finalizing' | 'ready' | 'error';
export type ApiTestStatus = 'untested' | 'testing' | 'passed' | 'failed';
export type FilePhase = 'no-folder' | 'no-open-file' | 'clean' | 'dirty' | 'saving' | 'saved' | 'error';
export type SessionPhase = 'new' | 'active' | 'generating' | 'complete' | 'stopped' | 'error';
export type Privacy = 'localWASM' | 'loopbackAPI' | 'privateNetworkAPI' | 'remoteAPI';

export interface SourceRef {
    kind: SourceKind;
    id: string;
    endpoint?: string;
}

export interface LoadOperation {
    operationId: number;
    sourceId: string;
}

export interface DownloadProgress {
    loaded: number;
    total: number | null;
    percent: number | null;
}

export interface ResidentArtifact {
    sourceId: string;
    schema: string;
    fileName?: string;
    sha256?: string;
    verified?: boolean | null;
    engineSupported?: boolean;
    canGenerate?: boolean;
    canPredict?: boolean;
    source?: string;
}

export interface ArtifactState {
    phase: ArtifactPhase;
    error: string | null;
}

export interface ApiState {
    testStatus: ApiTestStatus;
}

export interface FileState {
    phase: FilePhase;
    path: string | null;
    savePath: string | null;
    saveId: number | null;
    error: string | null;
}

export interface GenerationSnapshot {
    generationId: number;
    source: SourceRef;
    privacy: Privacy;
    canStop: boolean;
}

export interface SessionState {
    phase: SessionPhase;
    activeGeneration: GenerationSnapshot | null;
    lastGeneration: GenerationSnapshot | null;
    error: string | null;
}

export interface LifecycleState {
    selectedSource: SourceRef | null;
    operation: LoadOperation | null;
    artifact: ArtifactState;
    residentArtifact: ResidentArtifact | null;
    api: ApiState;
    progress: DownloadProgress | null;
    file: FileState;
    session: SessionState;
}

interface OperationAction {
    operation: LoadOperation;
}

export type LifecycleAction =
    | { type: 'artifact/selected'; sourceId: string }
    | ({ type: 'artifact/downloadStarted' } & OperationAction)
    | ({ type: 'artifact/downloadProgress'; loaded: number; total: number | null } & OperationAction)
    | ({ type: 'artifact/finalizing' } & OperationAction)
    | ({ type: 'artifact/ready'; artifact: Omit<ResidentArtifact, 'sourceId'> & { sourceId?: string } } & OperationAction)
    | ({ type: 'artifact/failed'; error: string } & OperationAction)
    | { type: 'api/selected'; sourceId: string; endpoint: string }
    | { type: 'api/testStarted'; sourceId: string }
    | { type: 'api/testFinished'; sourceId: string; ok: boolean }
    | { type: 'folder/opened'; path: string }
    | { type: 'file/opened'; path: string }
    | { type: 'file/closed' }
    | { type: 'session/activated' }
    | { type: 'session/cleared' }
    | { type: 'file/dirty' }
    | { type: 'file/saveStarted'; path: string }
    | { type: 'file/saveFinished'; path: string; saveId?: number }
    | { type: 'file/saveFailed'; path: string; saveId?: number; error: string }
    | { type: 'generation/started'; source: SourceRef; generationId?: number }
    | { type: 'generation/finished'; outcome: 'complete' | 'stopped' | 'error'; generationId: number; error?: string };

function initialState(): LifecycleState {
    return {
        selectedSource: null,
        operation: null,
        artifact: { phase: 'unselected', error: null },
        residentArtifact: null,
        api: { testStatus: 'untested' },
        progress: null,
        file: { phase: 'no-folder', path: null, savePath: null, saveId: null, error: null },
        session: { phase: 'new', activeGeneration: null, lastGeneration: null, error: null },
    };
}

export class LifecycleStore {
    private state: LifecycleState = initialState();
    private nextOperationId = 0;
    private nextGenerationId = 0;
    private nextSaveId = 0;
    private listeners = new Set<() => void>();

    getState(): LifecycleState {
        return this.state;
    }

    subscribe(listener: () => void): () => void {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    canRun(): boolean {
        if (this.state.session.phase === 'generating' || !this.state.selectedSource) return false;
        if (this.state.selectedSource.kind === 'api') {
            try {
                if (!this.state.selectedSource.endpoint) return false;
                classifyEndpoint(this.state.selectedSource.endpoint);
                return true;
            } catch {
                return false;
            }
        }
        if (this.state.artifact.phase !== 'ready') return false;
        const resident = this.state.residentArtifact;
        if (!resident || resident.sourceId !== this.state.selectedSource.id) return false;
        if (resident.engineSupported === false) return false;
        return !(resident?.canGenerate === false && resident.canPredict !== true);
    }

    canSave(): boolean {
        return this.state.file.path !== null && this.state.file.phase !== 'saving';
    }

    dispatch(action: LifecycleAction): void {
        const previous = this.state;
        this.apply(action);
        if (this.state !== previous) {
            for (const listener of this.listeners) listener();
        }
    }

    private apply(action: LifecycleAction): void {
        switch (action.type) {
            case 'artifact/selected': {
                this.nextOperationId += 1;
                this.state = {
                    ...this.state,
                    selectedSource: { kind: 'artifact', id: action.sourceId },
                    operation: { operationId: this.nextOperationId, sourceId: action.sourceId },
                    artifact: { ...this.state.artifact, phase: 'selected', error: null },
                    api: { testStatus: 'untested' },
                    progress: null,
                };
                return;
            }
            case 'artifact/downloadStarted':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'downloading', error: null },
                        progress: null,
                    };
                }
                return;
            case 'artifact/downloadProgress':
                if (this.isCurrent(action.operation)) {
                    const total = action.total !== null && action.total > 0 ? action.total : null;
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'downloading', error: null },
                        progress: {
                            loaded: action.loaded,
                            total,
                            percent: total === null ? null : Math.min(100, Math.floor((action.loaded * 100) / total)),
                        },
                    };
                }
                return;
            case 'artifact/finalizing':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'finalizing', error: null },
                        progress: this.state.progress
                            ? { ...this.state.progress, percent: null }
                            : { loaded: 0, total: null, percent: null },
                    };
                }
                return;
            case 'artifact/ready':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { phase: 'ready', error: null },
                        residentArtifact: { ...action.artifact, sourceId: action.artifact.sourceId || action.operation.sourceId },
                        progress: null,
                    };
                }
                return;
            case 'artifact/failed':
                if (this.isCurrent(action.operation)) {
                    this.state = {
                        ...this.state,
                        artifact: { ...this.state.artifact, phase: 'error', error: action.error },
                        progress: null,
                    };
                }
                return;
            case 'api/selected':
                this.state = {
                    ...this.state,
                    selectedSource: { kind: 'api', id: action.sourceId, endpoint: action.endpoint },
                    operation: null,
                    artifact: { ...this.state.artifact, phase: 'unselected', error: null },
                    api: { testStatus: 'untested' },
                    progress: null,
                };
                return;
            case 'api/testStarted':
                if (this.state.selectedSource?.kind === 'api' && this.state.selectedSource.id === action.sourceId) {
                    this.state = { ...this.state, api: { testStatus: 'testing' } };
                }
                return;
            case 'api/testFinished':
                if (this.state.selectedSource?.kind === 'api' && this.state.selectedSource.id === action.sourceId) {
                    this.state = { ...this.state, api: { testStatus: action.ok ? 'passed' : 'failed' } };
                }
                return;
            case 'folder/opened':
                this.state = {
                    ...this.state,
                    file: { phase: 'no-open-file', path: null, savePath: null, saveId: null, error: null },
                };
                return;
            case 'file/closed':
                this.state = {
                    ...this.state,
                    file: { phase: 'no-open-file', path: null, savePath: null, saveId: null, error: null },
                };
                return;
            case 'session/cleared':
                this.state = {
                    ...this.state,
                    session: { phase: 'new', activeGeneration: null, lastGeneration: null, error: null },
                };
                return;
            case 'session/activated':
                if (this.state.session.phase !== 'generating') {
                    this.state = {
                        ...this.state,
                        session: { ...this.state.session, phase: 'active', activeGeneration: null, error: null },
                    };
                }
                return;
            case 'file/opened':
                this.state = {
                    ...this.state,
                    file: { phase: 'clean', path: action.path, savePath: null, saveId: null, error: null },
                };
                return;
            case 'file/dirty':
                if (this.state.file.path !== null) {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'dirty', error: null } };
                }
                return;
            case 'file/saveStarted':
                if (this.state.file.path === action.path && this.state.file.phase !== 'saving') {
                    this.nextSaveId += 1;
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'saving', savePath: action.path, saveId: this.nextSaveId, error: null } };
                }
                return;
            case 'file/saveFinished':
                if (this.state.file.path === action.path && this.state.file.phase === 'saving' && this.state.file.savePath === action.path && (action.saveId === undefined || this.state.file.saveId === action.saveId)) {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'saved', savePath: null, saveId: null, error: null } };
                }
                return;
            case 'file/saveFailed':
                if (this.state.file.path === action.path && this.state.file.phase === 'saving' && this.state.file.savePath === action.path && (action.saveId === undefined || this.state.file.saveId === action.saveId)) {
                    this.state = { ...this.state, file: { ...this.state.file, phase: 'error', savePath: null, saveId: null, error: action.error } };
                }
                return;
            case 'generation/started': {
                const selected = this.state.selectedSource;
                if (this.state.session.phase === 'generating' || !selected || selected.kind !== action.source.kind || selected.id !== action.source.id) return;
                let privacy: Privacy;
                try {
                    if (selected.kind === 'api' && !selected.endpoint) throw new TypeError('An API endpoint is required.');
                    privacy = selected.kind === 'api' ? classifyEndpoint(selected.endpoint) : 'localWASM';
                } catch {
                    this.state = {
                        ...this.state,
                        session: { ...this.state.session, phase: 'error', activeGeneration: null, error: 'Invalid API endpoint.' },
                    };
                    return;
                }
                if (!this.canRun()) return;
                const generationId = action.generationId ?? this.nextGenerationId + 1;
                this.nextGenerationId = Math.max(this.nextGenerationId, generationId);
                const canStop = selected.kind === 'api'
                    || !(this.state.residentArtifact?.canGenerate === false && this.state.residentArtifact.canPredict === true);
                const generation = { generationId, source: { ...selected }, privacy, canStop };
                this.state = {
                    ...this.state,
                    session: { phase: 'generating', activeGeneration: generation, lastGeneration: this.state.session.lastGeneration, error: null },
                };
                return;
            }
            case 'generation/finished': {
                const active = this.state.session.activeGeneration;
                if (!active || active.generationId !== action.generationId) return;
                this.state = {
                    ...this.state,
                    session: { phase: action.outcome, activeGeneration: null, lastGeneration: active, error: action.error ?? null },
                };
                return;
            }
        }
    }

    private isCurrent(operation: LoadOperation): boolean {
        const current = this.state.operation;
        const selected = this.state.selectedSource;
        return Boolean(
            current
            && selected?.kind === 'artifact'
            && current.operationId === operation.operationId
            && current.sourceId === operation.sourceId
            && selected.id === operation.sourceId,
        );
    }
}

function isIpv4Literal(hostname: string): boolean {
    const parts = hostname.split('.');
    return parts.length === 4 && parts.every((part) => /^\d{1,3}$/.test(part) && Number(part) <= 255);
}

function isIpv6Literal(hostname: string): boolean {
    return /^[0-9a-f]*(?::[0-9a-f]*)+$/.test(hostname);
}

export function classifyEndpoint(input?: string): Privacy {
    if (input === undefined) return 'localWASM';
    const endpoint = new URL(input);
    if (endpoint.protocol !== 'http:' && endpoint.protocol !== 'https:') throw new TypeError('Only HTTP(S) API endpoints are supported.');
    const hostname = endpoint.hostname.toLowerCase().replace(/^\[|\]$/g, '');
    if (hostname === 'localhost') return 'loopbackAPI';
    if (isIpv4Literal(hostname)) {
        if (hostname.startsWith('127.')) return 'loopbackAPI';
        if (
            hostname.startsWith('10.')
            || hostname.startsWith('192.168.')
            || hostname.startsWith('169.254.')
            || /^172\.(1[6-9]|2[0-9]|3[0-1])\./.test(hostname)
        ) {
            return 'privateNetworkAPI';
        }
        return 'remoteAPI';
    }
    if (isIpv6Literal(hostname)) {
        if (hostname === '::1') return 'loopbackAPI';
        const first = Number.parseInt(hostname.split(':')[0] || '0', 16);
        if ((first & 0xff00) === 0xfc00 || (first & 0xff00) === 0xfd00) return 'privateNetworkAPI';
        if (first >= 0xfe80 && first <= 0xfebf) return 'privateNetworkAPI';
    }
    return 'remoteAPI';
}
