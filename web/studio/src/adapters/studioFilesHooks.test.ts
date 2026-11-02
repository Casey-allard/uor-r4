import { afterEach, describe, expect, it } from 'vitest';
import '../../assets/js/studio_files.js';
import type { WorkspaceApi } from './workflow.js';

afterEach(() => {
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
});

describe('legacy editor content hook', () => {
    it('keeps the legacy textarea and active file content in sync', () => {
        document.body.innerHTML = '<textarea id="editorText"></textarea>';
        const files = (window as unknown as { __studioFiles: WorkspaceApi }).__studioFiles;
        files.newFile();
        files.setActiveContent('changed text');
        expect(document.getElementById('editorText')).toHaveProperty('value', 'changed text');
        expect(files.activeFile()?.content).toBe('changed text');
    });
});
