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

    it('renders the task workspace with rail, detail and work-surface regions', () => {
        act(() => {
            render(<AppShell
                activeSection="agent"
                onSection={vi.fn()}
                status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }}
                secondarySection="workspace"
                secondaryChildren={<p>Work surface content</p>}
                primary={<h1>Agent</h1>}
            ><p>Overlays</p></AppShell>, host);
        });
        const shell = host.querySelector<HTMLElement>('.harness-shell')!;
        expect(shell.classList.contains('studio-shell')).toBe(true);
        expect(shell.classList.contains('task-workspace')).toBe(true);
        expect(host.querySelector('[data-region="task-rail"]')).not.toBeNull();
        expect(host.querySelector('main[data-region="task-detail"]')).not.toBeNull();
        const workSurface = host.querySelector('[data-region="work-surface"]');
        expect(workSurface).not.toBeNull();
        expect(workSurface?.closest('.pane-secondary')).not.toBeNull();
        expect(host.querySelectorAll('h1')).toHaveLength(1);
    });

    it('places main before the task rail in document order', () => {
        act(() => { render(<AppShell activeSection="agent" onSection={vi.fn()} status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }} primary={<h1>Agent</h1>}><p>Overlays</p></AppShell>, host); });
        const body = host.querySelector<HTMLElement>('.harness-body')!;
        const main = body.querySelector<HTMLElement>('main[data-region="task-detail"]')!;
        const rail = body.querySelector<HTMLElement>('[data-region="task-rail"]')!;
        expect(Array.from(body.children).indexOf(main)).toBeLessThan(Array.from(body.children).indexOf(rail));
    });

    it('keeps the rail open for scoped Escape and still closes it from plain page focus', () => {
        act(() => { render(<AppShell activeSection="agent" onSection={vi.fn()} status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }} primary={<h1>Agent</h1>}><p>Overlays</p></AppShell>, host); });
        const rail = host.querySelector<HTMLElement>('[data-region="task-rail"]')!;
        expect(rail.hidden).toBe(false);
        const composer = document.createElement('div');
        composer.className = 'agent-composer';
        const textarea = document.createElement('textarea');
        composer.appendChild(textarea);
        host.appendChild(composer);
        act(() => { textarea.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })); });
        expect(rail.hidden).toBe(false);
        const dialog = document.createElement('dialog');
        dialog.setAttribute('open', '');
        host.appendChild(dialog);
        act(() => { dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })); });
        expect(rail.hidden).toBe(false);
        dialog.remove();
        const prevented = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true });
        prevented.preventDefault();
        act(() => { document.dispatchEvent(prevented); });
        expect(rail.hidden).toBe(false);
        act(() => { document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })); });
        expect(rail.hidden).toBe(true);
    });

    it('does not render a green placeholder treatment for the task workspace', () => {
        act(() => { render(<AppShell activeSection="agent" onSection={vi.fn()} status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }}><p>Content</p></AppShell>, host); });
        const shell = host.querySelector<HTMLElement>('.harness-shell')!;
        expect(shell.className.split(/\s+/)).toEqual(['harness-shell', 'studio-shell', 'task-workspace']);
        expect(shell.outerHTML.toLowerCase()).not.toMatch(/green|placeholder/);
    });

    it('shows the actual project name and environment with a normal empty task state', () => {
        act(() => { render(<AppShell activeSection="agent" onSection={vi.fn()} status={{ source: 'none', artifact: 'none', privacy: 'local', file: 'none', session: 'new' }}><p>Content</p></AppShell>, host); });
        const rail = host.querySelector<HTMLElement>('[data-region="task-rail"]')!;
        expect(rail.textContent).toContain('uor-r4');
        expect(rail.textContent).toContain('Local browser');
        expect(rail.textContent).toContain('No tasks yet');
        expect(rail.textContent).not.toMatch(/runtime_unavailable|runtime is not initialized|hosted|connected/i);
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
