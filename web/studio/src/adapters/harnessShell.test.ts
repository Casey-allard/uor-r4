import { describe, expect, it } from 'vitest';
import source from '../../index.html?raw';
import uiSource from '../../assets/js/studio_ui.js?raw';
// @ts-expect-error Node's filesystem API is available in Vitest but not in the browser-only type config.
import { readFileSync } from 'node:fs';

const styles = readFileSync('index.css', 'utf8');

function setViewUnderTest() {
    const from = source.indexOf('function setView(view, options)');
    const to = source.indexOf('/* ------------------------------------------------------------ next-token predictor', from);
    const body = document.body;
    const factory = new Function('document', 'window', '$', `${source.slice(from, to)}; return setView;`);
    return factory(document, window, (selector: string) => document.querySelector(selector)) as (view: string) => void;
}

describe('unified harness shell', () => {
    it('maps Chat, Files, Changes, Terminal, Connections and Settings', () => {
        for (const view of ['changesView', 'terminalView']) {
            expect(source).toContain(`id="${view}"`);
        }
        for (const section of ['changes', 'terminal']) {
            expect(uiSource).toContain(`${section}: { view: '${section}'`);
            setViewUnderTest()(section);
            expect(document.body.dataset.view).toBe(section);
        }
        expect(source).toContain('id="navSettings"');
        expect(source).toContain('<span class="nav-item-label">Chat');
        expect(source).toContain('<span class="nav-item-label">Files');
        expect(source).not.toContain('<span class="nav-item-label">Agent');
        expect(source).not.toContain('<span class="nav-item-label">Workspace');
    });

    it('gives every workflow one clear heading and canonical component styles', () => {
        for (const [view, heading] of [['chatPanel', 'UOR-R4'], ['workspaceView', 'Files'], ['changesView', 'Changes'], ['terminalView', 'Terminal'], ['modelView', 'Connections']]) {
            const section = source.slice(source.indexOf(`id="${view}"`), source.indexOf('</section>', source.indexOf(`id="${view}"`)));
            expect(section).toContain('<h1');
            expect(section).toContain(heading);
        }
        for (const className of ['harness-nav', 'harness-view-body', 'harness-card', 'harness-modal', 'harness-menu']) {
            expect(styles).toContain(`.${className}`);
        }
        expect(styles).toContain('body[data-view="changes"] #changesView');
        expect(styles).toContain('body[data-view="terminal"] #terminalView');
        expect(styles).toContain('@media (forced-colors: active)');
        expect(styles).toContain('font-size: 16px');
        expect(styles).toMatch(/body:not\(\[data-view="chat"\]\)\s*\{[^}]*font-family:\s*-apple-system/s);
    });
});
