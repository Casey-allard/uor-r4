import { beforeEach, describe, expect, it, vi } from 'vitest';
import source from '../../index.html?raw';
// @ts-expect-error The existing JavaScript API has no declaration file.
import { normalizeBaseUrl } from '../../assets/js/studio_api.js';

function section(start: string, end: string): string {
    const from = source.indexOf(start);
    const to = source.indexOf(end, from);
    if (from < 0 || to < 0) throw new Error(`Missing live Agent section ${start}`);
    return source.slice(from, to);
}

function pageHooks() {
    const app = {
        mode: 'wasm',
        worker: { postMessage: vi.fn() },
        caps: null as null | { schema: string; synthetic?: boolean; engine_supported?: boolean; can_generate?: boolean },
        apiConfig: null as null | { baseUrl: string },
        activePredictionRequestId: null as string | null,
        activeGenerationId: 7 as number | null,
        generationSerial: 7,
        chatGenerationId: null as number | null,
        activeChatRequestId: null as string | number | null,
        chatRequests: {} as Record<string, number>,
        activeGenerationKind: 'wasm',
        generating: false,
        abortController: null as AbortController | null,
        streamingBodyEl: null,
        assistantBuffer: '',
        conversations: [] as Array<{ id: string; messages: Array<Record<string, unknown>>; title: string; updatedAt: number; sourceKind: string }>,
        activeConversationId: 'conversation',
    };
    const conversation = { id: 'conversation', messages: [] as Array<Record<string, unknown>>, title: 'Conversation', updatedAt: 0, sourceKind: '' };
    app.conversations = [conversation];
    const setView = vi.fn();
    const factory = new Function(
        'app', 'document', 'window', 'toast', 'setView', 'ensureConversation', 'touchConversation',
        'appendMessageElement', 'renderConversationList', 'scrollChat', 'sendViaWorker', 'sendViaApi',
        'autoGrowTextarea', 'startNewChat', '$', 'normalizeBaseUrl', 'persistConversations', 'renderMessages', 'setMetrics',
        `${section('function isDiscreteCapabilities', 'function setPredictorMode')}
         ${section('function chatReadiness', '/* ------------------------------------------------------------ metrics */')}
         ${section('function renderEmptyState', 'function listItem')}
         ${section('function apiMessages', 'async function sendViaApi')}
         ${section('function sendFromComposer', '/* ------------------------------------------------------------ model sources */').replace('} catch (_error) {', '} catch (error) { throw error;')}
         return { updateComposerState, renderEmptyState, sendFromComposer };`,
    );
    const hooks = factory(
        app, document, window, vi.fn(), setView, () => conversation, vi.fn(), vi.fn(() => null), vi.fn(), vi.fn(),
        vi.fn(() => true), vi.fn(() => true), vi.fn(), vi.fn(() => true), (selector: string) => document.querySelector(selector), normalizeBaseUrl, vi.fn(), vi.fn(), vi.fn(),
    ) as {
        updateComposerState(): void;
        renderEmptyState(container: HTMLElement): void;
        sendFromComposer(prompt?: string): number | false;
    };
    return { app, hooks, setView };
}

function sendButton(): HTMLButtonElement {
    return document.querySelector<HTMLButtonElement>('#btnSend')!;
}

describe('live Agent source readiness', () => {
    beforeEach(() => {
        document.body.innerHTML = '<button id="btnSend"></button><button id="btnStop"></button><textarea id="chatInput"></textarea><div id="chatMessages"></div>';
    });

    it('disables Send with no source and refuses the shared send path', () => {
        const { hooks } = pageHooks();
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(true);
        expect(sendButton().title).toContain('Connections');
        expect(hooks.sendFromComposer('hello')).toBe(false);
    });

    it('disables chat for discrete M1 without disabling its prediction path', () => {
        const { app, hooks } = pageHooks();
        app.caps = { schema: 'uor-r4.lut-discrete-trunk/1' };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(true);
        expect(hooks.sendFromComposer('hello')).toBe(false);
    });

    it('enables labeled synthetic continuation for a supported non-discrete stack', () => {
        const { app, hooks } = pageHooks();
        app.caps = { schema: 'uor-r4.lut-stack/7', synthetic: true };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(false);
        expect(hooks.sendFromComposer('hello')).toBe(7);
    });

    it('does not offer generation for a research-only FBS stack', () => {
        const { app, hooks } = pageHooks();
        app.caps = { schema: 'uor-r4.lut-stack/5', synthetic: false, engine_supported: true };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(true);
        expect(hooks.sendFromComposer('hello')).toBe(false);
        const container = document.querySelector<HTMLElement>('#chatMessages')!;
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('No conversational model is configured');
        expect(container.textContent).not.toContain('Local synthetic continuation ready');
    });

    it('enables API chat from an HTTP(S) config without a reachability claim', () => {
        const { app, hooks } = pageHooks();
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(false);
        expect(hooks.sendFromComposer('hello')).toBe(7);
    });

    it('refuses malformed and non-HTTP API configurations', () => {
        const { app, hooks } = pageHooks();
        app.mode = 'api';
        for (const baseUrl of ['not-a-url', 'ftp://example.com/v1']) {
            app.apiConfig = { baseUrl };
            hooks.updateComposerState();
            expect(sendButton().disabled).toBe(true);
            expect(hooks.sendFromComposer('hello')).toBe(false);
        }
    });

    it('does not render API readiness without a provider configuration', () => {
        const { app, hooks } = pageHooks();
        app.mode = 'api';
        const container = document.querySelector<HTMLElement>('#chatMessages')!;
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('Build with your files. Connect an agent when ready.');
        expect(container.textContent).toContain('No conversational model is configured');
        expect(container.textContent).not.toContain('API configured');
        hooks.updateComposerState();
        expect(sendButton().disabled).toBe(true);
    });

    it('renders distinct unavailable, API-ready, and local-ready empty states', () => {
        const { app, hooks } = pageHooks();
        const container = document.querySelector<HTMLElement>('#chatMessages')!;
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('Build with your files. Connect an agent when ready.');
        expect(container.textContent).toContain('No conversational model is configured');
        expect(container.textContent).toContain('Explore demo workspace');
        expect(container.textContent).toContain('Open Connections');
        container.innerHTML = '';
        app.mode = 'api';
        app.apiConfig = { baseUrl: 'https://api.example.com/v1' };
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('API configured — external baseline mode.');
        expect(container.textContent).toContain('will be sent');
        expect(container.textContent).not.toContain('Agent connected');
        expect(container.textContent).not.toContain('No conversational model is configured');
        container.innerHTML = '';
        app.mode = 'wasm';
        app.caps = { schema: 'uor-r4.lut-stack/7', synthetic: true };
        hooks.renderEmptyState(container);
        expect(container.textContent).toContain('Local synthetic continuation ready.');
        expect(container.textContent).not.toContain('No conversational model is configured');
    });

    it('activates the Workspace section from the Agent demo quick action', () => {
        const openDemo = vi.fn(() => true);
        const activateSection = vi.fn();
        const legacy = window as unknown as { __studioFiles?: { openDemo(): boolean }; __studioUi?: { activateSection(section: string): void } };
        legacy.__studioFiles = { openDemo };
        legacy.__studioUi = { activateSection };
        const { hooks, setView } = pageHooks();
        const container = document.querySelector<HTMLElement>('#chatMessages')!;
        hooks.renderEmptyState(container);
        const button = Array.from(container.querySelectorAll<HTMLButtonElement>('button')).find((node) => node.textContent === 'Explore demo workspace')!;
        button.click();
        expect(openDemo).toHaveBeenCalledOnce();
        expect(activateSection).toHaveBeenCalledWith('workspace');
        expect(setView).not.toHaveBeenCalled();
        delete legacy.__studioFiles;
        delete legacy.__studioUi;
    });
});
