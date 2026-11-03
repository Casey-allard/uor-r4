import { describe, expect, it } from 'vitest';
import source from '../../index.html?raw';
import uiSource from '../../assets/js/studio_ui.js?raw';

describe('native assistant role labels', () => {
    it('labels native assistant messages Native uor-chat instead of API', () => {
        const from = source.indexOf('function appendMessageElement');
        const to = source.indexOf('function renderEmptyState', from);
        const factory = new Function('app', 'document', '$', `${source.slice(from, to)} return appendMessageElement;`);
        const app = { mode: 'api', apiConfig: { id: 'native-loopback', ephemeral: true } };
        document.body.innerHTML = '<div id="chatMessages"></div>';
        const append = factory(app, document, (selector: string) => document.querySelector(selector));
        append('assistant', 'I.', '3 tokens');
        expect(document.querySelector('.msg-role')!.textContent).toBe('Native uor-chat');
    });

    it('labels the connection group as native and custom rather than custom only', () => {
        expect(uiSource).toContain('Native & custom chat sources');
    });
});
