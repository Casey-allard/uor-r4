import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { ProjectHome } from './ProjectHome.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); });

describe('ProjectHome', () => {
    it('shows the project name, environment and a normal no-tasks empty state', () => {
        act(() => { render(<ProjectHome projectName="uor-r4" environment="Local browser" />, host); });
        const section = host.querySelector<HTMLElement>('section[aria-label="Project home"]')!;
        expect(section).not.toBeNull();
        expect(section.textContent).toContain('uor-r4');
        expect(section.textContent).toContain('Local browser');
        expect(section.textContent).toContain('No tasks yet');
        expect(section.textContent).not.toMatch(/runtime_unavailable|runtime is not initialized/i);
    });

    it('does not invent tasks, memory or a service connection', () => {
        act(() => { render(<ProjectHome projectName="uor-r4" environment="Local browser" />, host); });
        const section = host.querySelector<HTMLElement>('section[aria-label="Project home"]')!;
        expect(section.querySelectorAll('[data-task-id], li')).toHaveLength(0);
        expect(section.textContent).not.toMatch(/memory|synced|hosted|account/i);
    });

    it('keeps its project heading below the view h1', () => {
        act(() => { render(<ProjectHome projectName="uor-r4" environment="Local browser" />, host); });
        expect(host.querySelectorAll('h1')).toHaveLength(0);
        expect(host.querySelector('h2')?.textContent).toContain('uor-r4');
    });
});
