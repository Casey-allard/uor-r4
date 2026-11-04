import { beforeEach, describe, expect, it, vi } from 'vitest';
import source from '../../index.html?raw';
// @ts-expect-error The existing JavaScript API has no declaration file.
import { normalizeBaseUrl } from '../../assets/js/studio_api.js';
// @ts-expect-error The metrics formatter has no declaration file.
import { createStreamMetrics } from '../../assets/js/stream_metrics.js';
import { LifecycleStore } from '../state/lifecycle.js';
import { createGenerationAdapter } from './generation.js';

function section(start: string, end: string): string {
    const from = source.indexOf(start);
    const to = source.indexOf(end, from);
    if (from < 0 || to < 0) throw new Error(`Missing page hook section ${start}`);
    return source.slice(from, to);
}

function pageHooks() {
    const app = {
        worker: { postMessage: vi.fn() },
        workerState: 'ready',
        caps: { schema: 'uor-r4.lut-discrete-trunk/1', shape: { vocab: 17, context: 64 }, synthetic: false },
        tokenizerArtifact: { sentinel: true },
        generationSerial: 0,
        conversations: [] as Array<{ id: string; messages: Array<Record<string, unknown>>; sourceKind: string }>,
        activeConversationId: 'conversation',
        activeGenerationId: null as number | null,
        chatGenerationId: null as number | null,
        activeChatRequestId: null as string | number | null,
        chatRequests: {} as Record<string, number>,
        activePredictionRequestId: null as string | null,
        predictionRequests: {},
        generating: false,
        mode: 'wasm',
        activeGenerationKind: 'wasm',
        apiConfig: null as null | { baseUrl: string },
        abortController: null as AbortController | null,
        streamingBodyEl: null,
        assistantBuffer: '',
        lastWorkerError: '',
    };
    interface CapturedApiRequest {
        messages?: Array<{ role: string; content: string }>;
        onDelta(text: string, meta: { uor?: { token_id?: number } }): void;
        onDone(result: { stopped?: boolean; usage?: { prompt_tokens: number; completion_tokens: number; total_tokens: number }; uor?: { completion_ids: number[]; elapsed_ns: number; first_token_ns: number; tokens_per_second_milli: number; stop: string }; finishReason?: string }): void;
        onError(error: Error): void;
    }
    const conversation = { id: 'conversation', messages: [] as Array<Record<string, unknown>>, sourceKind: '' };
    let currentConversation: typeof conversation | null = conversation;
    const apiRequests: CapturedApiRequest[] = [];
    let synchronousApiError: string | null = null;
    let requestSequence = 0;
    const setMetrics = vi.fn();
    const decomposeNativeImage = vi.fn(async ({ signal }: { signal: AbortSignal }) => new Promise<never>((_resolve, reject) => {
        signal.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')), { once: true });
    }));
    const factory = new Function(
        'app', 'document', 'window', 'toast', 'setMetrics', 'setEngineStatus', 'encodeTextWithArtifact', 'fillPredictorSha', '$',
        'activeConversation', 'appendMessageElement', 'touchConversation', 'updateComposerState', 'requestCheckpoint',
        'renderConversationList', 'scrollChat', 'formatTps', 'setView', 'makeId', 'streamChatCompletions', 'configFromSources', 'MAX_NEW_TOKENS',
        'ensureConversation', 'autoGrowTextarea', 'ModelSources', 'setModelStatus', 'renderMessages', 'persistConversations', 'normalizeBaseUrl', 'createStreamMetrics', 'decomposeNativeImage',
        `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
         ${section('function renderDiscretePrediction', 'function runPrediction')}
         ${section('function runPrediction', '/* ------------------------------------------------------------ status & toasts */')}
         ${section('function setChip', 'function toast')}
         ${section('function updatePrivacyStatus', 'function updateComposerState')}
         ${section('function chatReadiness', '/* ------------------------------------------------------------ metrics */')}
         ${section('function nextGenerationId', 'function generationTokenLimit')}
         ${section('function generationTokenLimit', '/* ------------------------------------------------------------ composer */')}
         ${section('function stopGeneration', 'function sendFromComposer')}
         ${section('function sendFromComposer', '/* ------------------------------------------------------------ model sources */')}
         ${section('function abortApiForLocalSource', 'function onModelSourceChanged')}
         ${section('function onModelSourceChanged', 'function onApiConfigChanged')}
         ${section('function onWorkerMessage', 'function activeAssistantElement')}
         return { runPrediction, renderDiscretePrediction, onModelSourceChanged, onWorkerMessage, sendFromComposer, sendViaWorker, sendViaApi, stopGeneration, finishLocalGeneration, finishGenerationWithError };`,
    );
    const hooks = factory(
        app, document, window, vi.fn(), setMetrics, vi.fn(), vi.fn(() => [1, 2]), vi.fn(), (selector: string) => document.querySelector(selector),
        () => currentConversation, vi.fn(() => null), vi.fn(), vi.fn(), vi.fn(), vi.fn(), vi.fn(), vi.fn(() => '1'), vi.fn(),
        () => `gen-${++requestSequence}`, vi.fn(async (options: CapturedApiRequest) => {
            if (synchronousApiError) {
                const message = synchronousApiError;
                synchronousApiError = null;
                options.onError(new Error(message));
                return;
            }
            for (const message of options.messages || []) await message.content;
            apiRequests.push(options);
        }), vi.fn(() => null), 32,
        () => conversation, vi.fn(),
        { active: () => ({ kind: 'api', id: 'api-a' }), list: () => ({ wasm: {} }) }, vi.fn(), vi.fn(), vi.fn(), normalizeBaseUrl, createStreamMetrics, decomposeNativeImage,
    ) as {
        runPrediction(prompt?: string): boolean;
        renderDiscretePrediction(message: unknown): void;
        onModelSourceChanged(event: { detail: Record<string, unknown> }): void;
        onWorkerMessage(event: { data: Record<string, unknown> }): void;
        sendFromComposer(prompt?: string): number | false;
        sendViaWorker(prompt: string, target: typeof conversation): boolean;
        sendViaApi(prompt: string, target: typeof conversation): Promise<void>;
        stopGeneration(): void;
        finishLocalGeneration(message: { id?: string; fullText?: string; tokenCount?: number }): void;
        finishGenerationWithError(error: string, requestId?: string | number, generationId?: number | null): boolean | void;
    };
    return { app, hooks, conversation, apiRequests, setMetrics, decomposeNativeImage, clearConversation: () => { currentConversation = null; }, failNextApi: (message: string) => { synchronousApiError = message; } };
}

function events(type: string, listener: (event: CustomEvent) => void) {
    document.addEventListener(type, listener as EventListener);
    return () => document.removeEventListener(type, listener as EventListener);
}

describe('actual page prediction hooks', () => {
    beforeEach(() => { document.body.innerHTML = ''; });

    it('ignores a diagnostic result and completes only the matching prediction request', () => {
        const { app, hooks } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.runPrediction('hello')).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        expect(request.action).toBe('predict_discrete');
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: 'diagnostic-predict', top8: [1], decoded: [] } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(1);
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: request.id, top8: [1], decoded: [] } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 1 });
        expect(app.activeGenerationId).toBeNull();
        dispose();
    });

    it('marks prediction as generating only for its correlated lifetime', () => {
        const { app, hooks } = pageHooks();
        expect(hooks.runPrediction('hello')).toBe(true);
        expect(app.generating).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: request.id, top8: [1], decoded: [] } });
        expect(app.generating).toBe(false);
        expect(app.activeGenerationId).toBeNull();
    });

    it('renders diagnostic results without completing the typed prediction', () => {
        const { app, hooks } = pageHooks();
        document.body.innerHTML = '<div id="predictorResult"></div><ol id="predictorTop"></ol><span id="predictorPrediction"></span><span id="predictorMs"></span><span id="predictorStatus"></span>';
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.runPrediction('hello')).toBe(true);
        hooks.onWorkerMessage({ data: { action: 'discrete_prediction_result', id: 'diagnostic-predict', top8: [7], decoded: [{ id: 7, text: 'x' }] } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(1);
        expect(document.documentElement.dataset.predictorTop).toBe('[7]');
        expect(document.documentElement.dataset.predictorTopCount).toBe('1');
        expect(document.documentElement.dataset.predictorDecoded).toBe('[{"id":7,"text":"x"}]');
        expect(document.getElementById('predictorTop')?.textContent).toContain('id 7');
        dispose();
    });

    it('refuses a chat send while prediction is in flight', () => {
        const { app, hooks } = pageHooks();
        app.activePredictionRequestId = 'predict-existing';
        app.activeGenerationId = 7;
        expect(hooks.sendFromComposer('chat')).toBe(false);
        expect(app.activeGenerationId).toBe(7);
        expect(app.worker.postMessage).not.toHaveBeenCalled();
    });

    it('refuses prediction while a chat generation is active', () => {
        const { app, hooks } = pageHooks();
        app.generating = true;
        app.activeGenerationId = 42;
        app.generationSerial = 42;
        expect(hooks.runPrediction('hello')).toBe(false);
        expect(app.worker.postMessage).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(42);
    });

    it('ignores late chat completion and error while prediction is active', () => {
        const { app, hooks } = pageHooks();
        app.activePredictionRequestId = 'predict-existing';
        app.activeGenerationId = 7;
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        hooks.finishLocalGeneration({ fullText: 'late', tokenCount: 1 });
        hooks.finishGenerationWithError('late error');
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(7);
        dispose();
    });

    it('returns a correlated worker error without completing the active prediction', () => {
        const { app, hooks } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.runPrediction('hello')).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        hooks.onWorkerMessage({ data: { action: 'operation_error', id: 'diagnostic-predict', error: 'unrelated' } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.activeGenerationId).toBe(1);
        hooks.onWorkerMessage({ data: { action: 'operation_error', id: request.id, error: 'context full' } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'error', generationId: 1, error: 'context full' });
        expect(app.activeGenerationId).toBeNull();
        dispose();
    });

    it('ignores stale and duplicate local terminals while a newer chat is active', () => {
        const { app, hooks, conversation } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.sendViaWorker('older', conversation)).toBe(true);
        const older = app.worker.postMessage.mock.calls[0][0];
        app.generating = false;
        expect(hooks.sendViaWorker('newer', conversation)).toBe(true);
        const newer = app.worker.postMessage.mock.calls[1][0];
        expect(app.chatGenerationId).toBe(2);
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: older.id, fullText: 'stale', tokenCount: 1 } });
        hooks.onWorkerMessage({ data: { action: 'generate_error', id: older.id, error: 'stale error' } });
        expect(completed).not.toHaveBeenCalled();
        expect(app.generating).toBe(true);
        expect(app.chatGenerationId).toBe(2);
        expect(app.activeChatRequestId).toBe(newer.id);
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: newer.id, fullText: 'current', tokenCount: 1 } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 2 });
        expect(app.activeChatRequestId).toBeNull();
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: newer.id, fullText: 'duplicate', tokenCount: 1 } });
        expect(completed).toHaveBeenCalledTimes(1);
        dispose();
    });

    it('returns false when the local worker cannot start a composer send', () => {
        const { app, hooks, conversation } = pageHooks();
        app.conversations = [conversation];
        app.activeConversationId = conversation.id;
        app.worker = null as unknown as typeof app.worker;
        expect(hooks.sendFromComposer('chat')).toBe(false);
        expect(app.activeGenerationId).toBeNull();
        expect(app.chatGenerationId).toBeNull();
        expect(conversation.messages).toHaveLength(0);
    });

    it('rolls back a worker start exception without persisting the turn', () => {
        const { app, hooks, conversation, setMetrics } = pageHooks();
        app.conversations = [conversation];
        app.activeConversationId = conversation.id;
        app.caps = { schema: 'uor-r4.lut-stack/1', shape: { vocab: 17, context: 64 }, synthetic: true };
        app.worker.postMessage = vi.fn(() => { throw new Error('clone failed'); });
        expect(hooks.sendFromComposer('chat')).toBe(false);
        expect(conversation.messages).toHaveLength(0);
        expect(app.generating).toBe(false);
        expect(app.activeGenerationId).toBeNull();
        expect(app.chatGenerationId).toBeNull();
        expect(app.chatRequests).toEqual({});
        expect(setMetrics).toHaveBeenLastCalledWith('Ready');
    });

    it('rolls back an API attachment failure before generation', () => {
        const { app, hooks, conversation } = pageHooks();
        app.conversations = [conversation];
        app.activeConversationId = conversation.id;
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        const legacy = window as unknown as { __studioUi?: { getAttachments(): never[] } };
        legacy.__studioUi = { getAttachments: () => { throw new Error('attachment bridge failed'); } };
        try {
            expect(hooks.sendFromComposer('chat')).toBe(false);
            expect(conversation.messages).toHaveLength(0);
            expect(app.generating).toBe(false);
            expect(app.activeGenerationId).toBeNull();
            expect(app.abortController).toBeNull();
        } finally {
            delete legacy.__studioUi;
        }
    });

    it('dispatches a matching local terminal when no conversation is active', () => {
        const { app, hooks, conversation, clearConversation } = pageHooks();
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        expect(hooks.sendViaWorker('chat', conversation)).toBe(true);
        const request = app.worker.postMessage.mock.calls[0][0];
        clearConversation();
        hooks.onWorkerMessage({ data: { action: 'generate_complete', id: request.id, fullText: 'done', tokenCount: 1 } });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 1 });
        expect(app.activeChatRequestId).toBeNull();
        expect(app.generating).toBe(false);
        dispose();
    });

    it('ignores stale API completion and error callbacks', async () => {
        const { app, hooks, conversation, apiRequests } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        await hooks.sendViaApi('older', conversation);
        app.generating = false;
        await hooks.sendViaApi('newer', conversation);
        expect(apiRequests).toHaveLength(2);
        apiRequests[0].onDone({ stopped: false });
        apiRequests[0].onError(new Error('stale error'));
        expect(completed).not.toHaveBeenCalled();
        expect(app.generating).toBe(true);
        expect(app.chatGenerationId).toBe(2);
        apiRequests[1].onDone({ stopped: false });
        expect(completed).toHaveBeenCalledTimes(1);
        expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'complete', generationId: 2 });
        apiRequests[1].onDone({ stopped: false });
        expect(completed).toHaveBeenCalledTimes(1);
        dispose();
    });

    it('uses the started generation channel after the picker mode changes', async () => {
        const { app, hooks, conversation } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        await hooks.sendViaApi('api', conversation);
        const abort = vi.spyOn(app.abortController!, 'abort');
        app.mode = 'wasm';
        hooks.stopGeneration();
        expect(abort).toHaveBeenCalledOnce();
        expect(app.worker.postMessage).not.toHaveBeenCalled();
        app.mode = 'wasm';
        hooks.sendViaWorker('local', conversation);
        app.mode = 'api';
        hooks.stopGeneration();
        expect(app.worker.postMessage).toHaveBeenCalledWith({ action: 'cancel' });
    });

    it('prepends stored attachments to API prompts', async () => {
        const { app, hooks, conversation, apiRequests } = pageHooks();
        const legacy = window as unknown as { __studioUi?: { getAttachments(): Array<{ name: string; content: string }> } };
        legacy.__studioUi = { getAttachments: () => [{ name: 'notes.md', content: 'attached context' }] };
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        try {
            await hooks.sendViaApi('prompt', conversation);
            expect(apiRequests).toHaveLength(1);
            expect(apiRequests[0].messages?.[0]).toEqual({ role: 'user', content: 'Workspace context:\n--- notes.md ---\nattached context' });
        } finally {
            delete legacy.__studioUi;
        }
    });

    it('allows text-only sends with recalled metadata and cleans up completion', async () => {
        const { app, hooks, conversation, apiRequests } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { id: 'native-loopback', ephemeral: true, baseUrl: 'http://127.0.0.1:8765/v1' } as { baseUrl: string };
        const legacy = window as unknown as { __studioUi?: { getAttachments(): Array<Record<string, unknown>>; getImageRecallCount?(): number } };
        legacy.__studioUi = { getAttachments: () => [], getImageRecallCount: () => 1 };
        conversation.messages.push({ role: 'user', content: 'Hi' });
        try {
            await hooks.sendViaApi('Hi', conversation);
            expect(apiRequests).toHaveLength(1);
            expect(conversation.messages.some((message) => message.role === 'error')).toBe(false);
            apiRequests[0].onDone({ stopped: false });
            expect(app.generating).toBe(false);
            expect(app.activeChatRequestId).toBeNull();
        } finally {
            delete legacy.__studioUi;
        }
    });

    it('aborts native image preparation and finishes as stopped', async () => {
        const { app, hooks, conversation, decomposeNativeImage } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { id: 'native-loopback', ephemeral: true, baseUrl: 'http://127.0.0.1:8765/v1' } as { baseUrl: string };
        const legacy = window as unknown as { __studioUi?: { getAttachments(): Array<Record<string, unknown>> } };
        legacy.__studioUi = { getAttachments: () => [{ name: 'pixel.png', kind: 'image', content: '', rgba: new Uint8Array(4), width: 1, height: 1, grid: { columns: 1, rows: 1 } }] };
        conversation.messages.push({ role: 'user', content: 'Hi' });
        const completed = vi.fn();
        const dispose = events('uor-generation-finished', completed);
        try {
            const pending = hooks.sendViaApi('Hi', conversation);
            hooks.stopGeneration();
            await pending;
            expect(decomposeNativeImage).toHaveBeenCalledOnce();
            expect(decomposeNativeImage.mock.calls[0][0].signal.aborted).toBe(true);
            expect(app.generating).toBe(false);
            expect(completed).toHaveBeenCalledTimes(1);
            expect(completed.mock.calls[0][0].detail).toMatchObject({ outcome: 'stopped' });
            expect(conversation.messages.some((message) => message.role === 'error')).toBe(false);
        } finally {
            dispose();
            delete legacy.__studioUi;
        }
    });

    it('does not offer or send cancellation for synchronous prediction', () => {
        const { app, hooks } = pageHooks();
        app.activeGenerationKind = 'api';
        app.abortController = null;
        expect(hooks.runPrediction('hello')).toBe(true);
        hooks.stopGeneration();
        expect(app.worker.postMessage).toHaveBeenCalledTimes(1);
        expect(app.worker.postMessage).toHaveBeenLastCalledWith(expect.objectContaining({ action: 'predict_discrete' }));
    });

    it('aborts and pins API privacy until the removed stream terminates', async () => {
        const { app, hooks, conversation, apiRequests } = pageHooks();
        document.body.innerHTML = '<span id="privacyStatus"></span><span id="chatSourceBadge"></span><button id="btnSend"></button><button id="btnStop"></button>';
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        await hooks.sendViaApi('prompt', conversation);
        const abort = vi.spyOn(app.abortController!, 'abort');
        hooks.onModelSourceChanged({ detail: { kind: 'state', removedActive: true } });
        expect(abort).toHaveBeenCalledOnce();
        expect(app.mode).toBe('wasm');
        expect(app.apiConfig).toBeNull();
        expect(document.getElementById('privacyStatus')?.textContent).toContain('API mode: prompts leave this device');
        apiRequests[0].onDone({ stopped: true });
        expect(document.getElementById('privacyStatus')?.textContent).toContain('Local only');
    });

    it('finishes a synchronous page API error after the adapter starts generation', () => {
        const { app, hooks, failNextApi } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        const sessions = { getMode: () => app.mode, sendPrompt: hooks.sendFromComposer, predict: vi.fn(), stopGeneration: vi.fn() } as unknown as Parameters<typeof createGenerationAdapter>[1];
        const adapter = createGenerationAdapter(store, sessions);
        failNextApi('synchronous config error');
        expect(adapter.send('hello')).toBe(true);
        expect(store.getState().session.phase).toBe('error');
        expect(store.getState().session.error).toBe('synchronous config error');
        expect(store.getState().session.activeGeneration).toBeNull();
        expect(app.generating).toBe(false);
        adapter.dispose();
    });
});

describe('actual page API configuration hook', () => {
    it('updates the active API config from the selected provider without navigation', () => {
        const app = { apiConfig: { baseUrl: 'https://old.example.com/v1' } };
        const ModelSources = { apiConfig: vi.fn(() => ({ baseUrl: 'https://new.example.com/v1', model: 'model-b' })) };
        const setModelStatus = vi.fn();
        const updatePrivacyStatus = vi.fn();
        const factory = new Function('app', 'ModelSources', 'updateComposerState', 'setModelStatus', 'updatePrivacyStatus', `${section('function onApiConfigChanged', 'function bindHeader')} return onApiConfigChanged;`);
        const onApiConfigChanged = factory(app, ModelSources, vi.fn(), setModelStatus, updatePrivacyStatus) as (event: { detail: { id: string } }) => void;
        onApiConfigChanged({ detail: { id: 'api-a' } });
        expect(ModelSources.apiConfig).toHaveBeenCalledWith('api-a');
        expect(app.apiConfig.baseUrl).toBe('https://new.example.com/v1');
        expect(setModelStatus).toHaveBeenCalledWith('API · model-b', 'accent', 'https://new.example.com/v1');
        expect(updatePrivacyStatus).toHaveBeenCalledOnce();
    });
});

describe('actual page local fallback hook', () => {
    it('restores local mode and privacy from a loaded fallback artifact', () => {
        const app = {
            mode: 'api', apiConfig: { baseUrl: 'https://old.example.com/v1' } as null | { baseUrl: string },
            worker: { postMessage: vi.fn() }, loadStartedAt: 0,
        };
        const updatePrivacyStatus = vi.fn();
        const setView = vi.fn();
        const factory = new Function(
            'app', 'ModelSources', 'setModelStatus', 'updatePrivacyStatus', 'updateComposerState', 'renderMessages', 'setView', 'toast', 'window',
            `${section('function abortApiForLocalSource', 'function onModelSourceChanged')}
         ${section('function onModelSourceChanged', 'function onApiConfigChanged')} return onModelSourceChanged;`,
        );
        const onModelSourceChanged = factory(app, {}, vi.fn(), updatePrivacyStatus, vi.fn(), vi.fn(), setView, vi.fn(), window) as (event: { detail: Record<string, unknown> }) => void;
        onModelSourceChanged({ detail: { kind: 'wasm', bytes: new Uint8Array([1]), fileName: 'demo.lut', meta: { kind: 'bundled' } } });
        expect(app.mode).toBe('wasm');
        expect(app.apiConfig).toBeNull();
        expect(updatePrivacyStatus).toHaveBeenCalledOnce();
        expect(app.worker.postMessage).toHaveBeenCalledWith(expect.objectContaining({ action: 'load_model' }));
        expect(setView).toHaveBeenCalledWith('chat');
    });

    it('fails closed immediately when the manager removes the active provider', async () => {
        window.localStorage.clear();
        document.body.innerHTML = '<button id="btnSend"></button><button id="btnStop"></button>';
        vi.resetModules();
        const { ModelSources } = await import('../../assets/js/model_sources.js');
        const app = {
            mode: 'wasm', apiConfig: null as null | { baseUrl: string }, caps: { schema: 'uor-r4.lut-stack/1' } as null | { schema: string },
            generating: false,
        };
        const updatePrivacyStatus = vi.fn();
        const factory = new Function(
            'app', 'ModelSources', 'setModelStatus', 'updatePrivacyStatus', 'renderMessages', 'setView', 'toast', 'window', '$', 'normalizeBaseUrl', 'isNativeLoopback',
            `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
             ${section('function chatReadiness', '/* ------------------------------------------------------------ metrics */')}
             ${section('function abortApiForLocalSource', 'function onModelSourceChanged')}
         ${section('function onModelSourceChanged', 'function onApiConfigChanged')}
             return onModelSourceChanged;`,
        );
        const onModelSourceChanged = factory(app, ModelSources, vi.fn(), updatePrivacyStatus, vi.fn(), vi.fn(), vi.fn(), window, (selector: string) => document.querySelector(selector), normalizeBaseUrl, (config: { id?: string; ephemeral?: boolean }) => Boolean(config && config.id === 'native-loopback' && config.ephemeral === true)) as (event: Event) => void;
        document.addEventListener('uor-model-source-changed', onModelSourceChanged);
        const provider = ModelSources.addApi({ label: 'Remove me', baseUrl: 'https://old.example.com/v1', model: 'old' });
        ModelSources.setActive({ kind: 'api', id: provider.id });
        expect(app.mode).toBe('api');
        ModelSources.removeApi(provider.id);
        expect(app.mode).toBe('wasm');
        expect(app.apiConfig).toBeNull();
        expect(app.caps).toBeNull();
        expect(document.querySelector<HTMLButtonElement>('#btnSend')!.disabled).toBe(true);
        expect(ModelSources.apiConfig(provider.id)).toBeNull();
        document.removeEventListener('uor-model-source-changed', onModelSourceChanged);
        window.localStorage.clear();
    });

    it('updates the live API config and privacy after editing the selected provider', async () => {
        window.localStorage.clear();
        document.body.innerHTML = '<button id="btnSend"></button><button id="btnStop"></button>';
        vi.resetModules();
        const { ModelSources } = await import('../../assets/js/model_sources.js');
        const app = {
            mode: 'wasm', apiConfig: null as null | { baseUrl: string; model: string }, caps: { schema: 'uor-r4.lut-stack/1' } as null | { schema: string },
            generating: false,
        };
        const updatePrivacyStatus = vi.fn();
        const setModelStatus = vi.fn();
        const factory = new Function(
            'app', 'ModelSources', 'setModelStatus', 'updatePrivacyStatus', 'renderMessages', 'setView', 'toast', 'window', '$', 'normalizeBaseUrl', 'isNativeLoopback',
            `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
             ${section('function chatReadiness', '/* ------------------------------------------------------------ metrics */')}
             ${section('function abortApiForLocalSource', 'function onModelSourceChanged')}
         ${section('function onModelSourceChanged', 'function onApiConfigChanged')}
             return onModelSourceChanged;`,
        );
        const onModelSourceChanged = factory(app, ModelSources, setModelStatus, updatePrivacyStatus, vi.fn(), vi.fn(), vi.fn(), window, (selector: string) => document.querySelector(selector), normalizeBaseUrl, (config: { id?: string; ephemeral?: boolean }) => Boolean(config && config.id === 'native-loopback' && config.ephemeral === true)) as (event: Event) => void;
        document.addEventListener('uor-model-source-changed', onModelSourceChanged);
        const provider = ModelSources.addApi({ label: 'Edit me', baseUrl: 'https://old.example.com/v1', model: 'old' });
        ModelSources.setActive({ kind: 'api', id: provider.id });
        ModelSources.updateApi(provider.id, { baseUrl: 'http://127.0.0.1:8000/v1', model: 'new' });
        expect(app.apiConfig).toMatchObject({ baseUrl: 'http://127.0.0.1:8000/v1', model: 'new' });
        expect(setModelStatus).toHaveBeenLastCalledWith('API · new', 'accent', 'http://127.0.0.1:8000/v1');
        expect(updatePrivacyStatus).toHaveBeenCalled();
        expect(document.querySelector<HTMLButtonElement>('#btnSend')!.disabled).toBe(false);
        document.removeEventListener('uor-model-source-changed', onModelSourceChanged);
        window.localStorage.clear();
    });
});

describe('actual page native API metrics', () => {
    it('uses final usage and native timing instead of character estimates', async () => {
        const { app, hooks, conversation, apiRequests, setMetrics } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'http://127.0.0.1:8765/v1' };
        document.body.innerHTML = '<span id="privacyStatus"></span><span id="chatSourceBadge"></span><button id="btnSend"></button><button id="btnStop"></button>';
        await hooks.sendViaApi('prompt', conversation);
        expect(apiRequests).toHaveLength(1);
        apiRequests[0].onDone({
            stopped: false,
            usage: { prompt_tokens: 12, completion_tokens: 3, total_tokens: 15 },
            uor: { completion_ids: [340, 16, 1], elapsed_ns: 5000000, first_token_ns: 2000000, tokens_per_second_milli: 600000, stop: 'eos' },
            finishReason: 'stop',
        });
        expect(setMetrics).toHaveBeenLastCalledWith('3 tokens · 600.0 tok/s · TTFT 2.000 ms · 5.000 ms · 12/3/15 · stop');
    });

    it('labels the native loopback as local instead of an external provider', () => {
        const { app, hooks } = pageHooks();
        document.body.innerHTML = '<span id="privacyStatus"></span><span id="chatSourceBadge"></span>';
        hooks.onModelSourceChanged({
            detail: {
                kind: 'api',
                id: 'native-loopback',
                apiConfig: { id: 'native-loopback', ephemeral: true, baseUrl: 'http://127.0.0.1:8765/v1', model: 'uor-chat' },
            },
        });
        expect(app.mode).toBe('api');
        expect(document.querySelector('#privacyStatus')!.textContent).toContain('Native loopback');
        expect(document.querySelector('#privacyStatus')!.textContent).not.toContain('leave this device');
        expect(document.querySelector('#chatSourceBadge')!.textContent).toBe('Native uor-chat');
    });

    it('keeps non-native empty-ID streams character-only', async () => {
        const { app, hooks, conversation, apiRequests, setMetrics } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        document.body.innerHTML = '<span id="privacyStatus"></span><span id="chatSourceBadge"></span><button id="btnSend"></button><button id="btnStop"></button>';
        await hooks.sendViaApi('prompt', conversation);
        apiRequests[0].onDelta('x', { uor: {} });
        apiRequests[0].onDone({ stopped: false });
        const rendered = setMetrics.mock.calls.at(-1)?.[0] as string;
        expect(rendered).toContain('1 chars');
        expect(rendered).toContain('usage unavailable');
        expect(rendered).not.toContain('0 tokens');
    });
});
