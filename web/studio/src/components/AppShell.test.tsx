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
    it('provides six destinations, one main/status region, and a labeled drawer', () => {
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
        expect(Array.from(navigation.querySelectorAll('button')).map((node) => node.textContent)).toEqual(['Agent', 'Workspace', 'Changes', 'Terminal', 'Models', 'Guide']);
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

    it('creates a project through the real folder callback and enters its workspace', async () => {
        Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value(this: HTMLDialogElement) { this.open = true; } });
        Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value(this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); } });
        const onSection = vi.fn();
        const onOpenFolder = vi.fn(async () => 'fixture-project');
        act(() => { render(<AppShell activeSection="agent" onSection={onSection} onOpenFolder={onOpenFolder} status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }}><div>Content</div></AppShell>, host); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-action="new-project"]')!.click(); });
        const name = host.querySelector<HTMLInputElement>('input[aria-label="Project name"]')!;
        act(() => { name.value = 'Fixture project'; name.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { Array.from(host.querySelectorAll('button')).find((button) => button.textContent === 'Choose folder')!.click(); });
        await vi.waitFor(() => expect(host.querySelector<HTMLInputElement>('input[aria-label="Project folder"]')!.value).toBe('fixture-project'));
        act(() => { Array.from(host.querySelectorAll('button')).find((button) => button.textContent === 'Create project')!.click(); });
        expect(onOpenFolder).toHaveBeenCalledOnce();
        expect(onSection).toHaveBeenCalledWith('workspace');
    });
});
