import { afterEach, describe, expect, it, vi } from 'vitest';
import type { WorkspaceApi } from './workflow.js';

const studioFilesModule = '../../assets/js/studio_files.js';

afterEach(() => {
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
});

describe('legacy editor content hook', () => {
    it('keeps the legacy textarea and active file content in sync', async () => {
        vi.resetModules();
        document.body.innerHTML = '<textarea id="editorText"></textarea>';
        document.dispatchEvent(new Event('DOMContentLoaded'));
        await import(studioFilesModule);
        const files = (window as unknown as { __studioFiles: WorkspaceApi }).__studioFiles;
        files.newFile();
        files.setActiveContent('changed text');
        expect(document.getElementById('editorText')).toHaveProperty('value', 'changed text');
        expect(files.activeFile()?.content).toBe('changed text');
    });

    it('gives rendered tree attachments unique accessible names', async () => {
        vi.resetModules();
        document.body.innerHTML = '<div id="fileTree"></div><div id="fileTreeMeta"></div><textarea id="editorText"></textarea>';
        document.dispatchEvent(new Event('DOMContentLoaded'));
        await import(studioFilesModule);
        const files = (window as unknown as { __studioFiles: WorkspaceApi & { openDemo(): boolean } }).__studioFiles;
        expect(files.openDemo()).toBe(true);
        const controls = Array.from(document.querySelectorAll<HTMLButtonElement>('.tree-attach'));
        expect(controls.length).toBeGreaterThan(0);
        expect(controls[0].tagName).toBe('BUTTON');
        expect(controls[0].getAttribute('aria-label')).toContain('crates/uor-r4-integer/src/lib.rs');
        const names = controls.map((control) => control.getAttribute('aria-label'));
        expect(new Set(names).size).toBe(names.length);
        const attached = vi.fn();
        document.addEventListener('uor-attach-file', attached);
        controls[0].click();
        await vi.waitFor(() => expect(attached).toHaveBeenCalledOnce());
        expect(attached.mock.calls[0][0]).toMatchObject({
            detail: { name: 'crates/uor-r4-integer/src/lib.rs', source: 'workspace' },
        });
    });
});
