import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AppShell } from './AppShell.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); });

describe('AppShell', () => {
    it('reports desktop navigation as expanded', () => {
        act(() => { render(<AppShell activeSection="agent" onSection={vi.fn()} status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }}><div>Content</div></AppShell>, host); });
        expect(host.querySelector('button[aria-controls="studio-navigation"]')?.getAttribute('aria-expanded')).toBe('true');
    });
    it('provides four destinations, one main/status region, and a labeled drawer', () => {
        const onSection = vi.fn();
        const onInspector = vi.fn();
        act(() => {
            render(<AppShell
                activeSection="agent"
                onSection={onSection}
                status={{ source: 'API A', artifact: 'No artifact', privacy: 'Remote API', file: 'No file open', session: 'New session' }}
                sessions={<div>Session list</div>}
                onInspector={onInspector}
                inspectorOpen={false}
            ><p>Primary content</p></AppShell>, host);
        });
        expect(host.querySelectorAll('main')).toHaveLength(1);
        expect(host.querySelectorAll('[role="status"]')).toHaveLength(1);
        expect(host.querySelector('[role="status"]')?.textContent).toContain('API A');
        const navigation = host.querySelector<HTMLElement>('nav[aria-label="Studio"]')!;
        expect(Array.from(navigation.querySelectorAll('button')).map((node) => node.textContent)).toEqual(['Agent', 'Workspace', 'Models', 'Guide']);
        const workspace = Array.from(navigation.querySelectorAll('button')).find((node) => node.textContent === 'Workspace')!;
        workspace.click();
        expect(onSection).toHaveBeenCalledWith('workspace');
        const drawer = host.querySelector<HTMLButtonElement>('button[aria-controls="studio-navigation"]')!;
        expect(drawer.getAttribute('aria-expanded')).toBe('true');
        act(() => { drawer.click(); });
        expect(drawer.getAttribute('aria-expanded')).toBe('false');
        expect(navigation.hidden).toBe(true);
        act(() => { drawer.click(); });
        expect(drawer.getAttribute('aria-expanded')).toBe('true');
        expect(navigation.hidden).toBe(false);
        const inspector = host.querySelector<HTMLButtonElement>('button[aria-controls="studio-inspector"]')!;
        inspector.click();
        expect(onInspector).toHaveBeenCalledOnce();
        expect(host.textContent).toContain('Primary content');
        expect(host.textContent).toContain('Session list');
    });
});
