// UOR-R4 Studio v2 — workspace files, editor tabs and disk save.
// Browser-only: File System Access API when available; import-files fallback is read-only.
// No network, no external assets. Loaded before studio_ui.js.

(function () {
    'use strict';

    var TABS_KEY = 'uor.studio.tabs.v2';
    var RECENT_KEY = 'uor.studio.recent.v2';
    var MAX_FILES = 400;
    var MAX_DEPTH = 4;
    var TEXT_LIMIT = 2 * 1024 * 1024;

    var state = {
        rootHandle: null,
        rootName: '',
        mode: 'none', // none | directory | imported | demo
        tree: [],
        files: {}, // path -> {path,name,kind,handle? , content?, size?}
        tabs: [],
        activePath: null,
        recent: []
    };

    function $(selector, root) { return (root || document).querySelector(selector); }
    function toast(text, tone) {
        if (window.__studioUi && typeof window.__studioUi.toast === 'function') { window.__studioUi.toast(text, tone || 'info'); return; }
        var region = document.getElementById('toastRegion');
        if (!region) return;
        var node = document.createElement('div');
        node.className = 'toast';
        node.setAttribute('data-tone', tone || 'info');
        node.textContent = text;
        region.appendChild(node);
        window.setTimeout(function () { node.remove(); }, 4200);
    }

    function emit() {
        document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: summary() }));
    }

    function summary() {
        return {
            mode: state.mode,
            rootName: state.rootName,
            fileCount: Object.keys(state.files).length,
            tabs: state.tabs.map(function (tab) { return { path: tab.path, dirty: Boolean(tab.dirty) }; }),
            activePath: state.activePath
        };
    }

    function languageFor(path) {
        var ext = String(path || '').split('.').pop().toLowerCase();
        var map = { md: 'markdown', markdown: 'markdown', rs: 'rust', json: 'json', js: 'javascript', mjs: 'javascript', html: 'html', htm: 'html', css: 'css', toml: 'toml', yml: 'yaml', yaml: 'yaml', txt: 'text', sh: 'shell', py: 'python' };
        return map[ext] || 'text';
    }

    function baseName(path) { return String(path || '').split('/').pop(); }
    function dirName(path) { var parts = String(path || '').split('/'); parts.pop(); return parts.join('/'); }

    function isTextFile(path) {
        var ext = String(path || '').split('.').pop().toLowerCase();
        return ['md', 'markdown', 'txt', 'rs', 'toml', 'json', 'js', 'mjs', 'html', 'htm', 'css', 'yml', 'yaml', 'sh', 'py', 'gitignore', 'lock'].indexOf(ext) >= 0 || baseName(path).indexOf('.') === -1;
    }

    /* ------------------------------------------------------------------ storage */

    function persistTabs() {
        try {
            var payload = state.tabs.filter(function (tab) { return tab.mode !== 'directory' || !tab.dirty; }).map(function (tab) {
                return { path: tab.path, content: tab.content, dirty: tab.mode !== 'directory' && tab.dirty, mode: tab.mode };
            });
            window.localStorage.setItem(TABS_KEY, JSON.stringify({ activePath: state.activePath, tabs: payload }));
        } catch (error) { /* storage optional */ }
    }

    function restoreTabs() {
        try {
            var raw = window.localStorage.getItem(TABS_KEY);
            if (!raw) return;
            var parsed = JSON.parse(raw);
            state.tabs = (parsed.tabs || []).filter(function (tab) { return tab && typeof tab.content === 'string'; }).map(function (tab) {
                return { path: tab.path, name: baseName(tab.path), content: tab.content, dirty: Boolean(tab.dirty), mode: tab.mode || 'local', handle: null };
            });
            state.activePath = parsed.activePath || (state.tabs[0] ? state.tabs[0].path : null);
            renderTabs();
            if (state.activePath) loadTabIntoEditor(state.activePath, true);
        } catch (error) { /* ignore bad storage */ }
    }

    function persistRecent() {
        try { window.localStorage.setItem(RECENT_KEY, JSON.stringify(state.recent.slice(0, 12))); } catch (error) { /* optional */ }
    }

    function restoreRecent() {
        try {
            var parsed = JSON.parse(window.localStorage.getItem(RECENT_KEY) || '[]');
            if (Array.isArray(parsed)) state.recent = parsed.filter(function (item) { return typeof item === 'string'; }).slice(0, 12);
        } catch (error) { state.recent = []; }
        renderRecent();
    }

    /* ------------------------------------------------------------------ tree */

    async function readDirectory(handle, path, depth) {
        var entries = [];
        var iterator = handle.values();
        var count = 0;
        for (;;) {
            var next = await iterator.next();
            if (next.done) break;
            count += 1;
            if (count > MAX_FILES) break;
            var entry = next.value;
            var entryPath = path ? path + '/' + entry.name : entry.name;
            if (entry.kind === 'file') {
                var file = null;
                try { file = await entry.getFile(); } catch (error) { file = null; }
                state.files[entryPath] = { path: entryPath, name: entry.name, kind: 'file', handle: entry, size: file ? file.size : 0 };
                entries.push({ path: entryPath, name: entry.name, kind: 'file' });
            } else if (entry.kind === 'directory' && depth < MAX_DEPTH && !entry.name.startsWith('.') && entry.name !== 'node_modules' && entry.name !== 'target') {
                var children = await readDirectory(entry, entryPath, depth + 1);
                entries.push({ path: entryPath, name: entry.name, kind: 'directory', children: children });
            }
        }
        entries.sort(function (a, b) {
            if (a.kind !== b.kind) return a.kind === 'directory' ? -1 : 1;
            return a.name.localeCompare(b.name);
        });
        return entries;
    }

    async function openFolder() {
        if (!window.showDirectoryPicker) {
            importFiles('This browser does not expose the File System Access API. Import files instead.');
            return;
        }
        try {
            var handle = await window.showDirectoryPicker({ mode: 'readwrite' });
            state.rootHandle = handle;
            state.rootName = handle.name;
            state.mode = 'directory';
            state.files = {};
            state.tree = await readDirectory(handle, '', 0);
            updateWorkspaceUI();
            renderTree();
            emit();
            toast('Opened ' + handle.name + ' (' + Object.keys(state.files).length + ' files).', 'success');
        } catch (error) {
            if (error && error.name === 'AbortError') return;
            toast('Could not open that folder: ' + (error && error.message ? error.message : error), 'error');
        }
    }

    function importFiles(reason) {
        if (reason) toast(reason, 'info');
        var input = document.createElement('input');
        input.type = 'file';
        input.multiple = true;
        input.webkitdirectory = true;
        input.style.display = 'none';
        input.addEventListener('change', async function () {
            var files = Array.prototype.slice.call(input.files || []);
            if (!files.length) { input.remove(); return; }
            state.rootHandle = null;
            state.mode = 'imported';
            state.files = {};
            state.tree = [];
            var rootName = files[0].webkitRelativePath ? files[0].webkitRelativePath.split('/')[0] : 'imported-files';
            state.rootName = rootName;
            var tree = {};
            for (var i = 0; i < files.length && i < MAX_FILES; i += 1) {
                var file = files[i];
                var relative = file.webkitRelativePath || file.name;
                var parts = relative.split('/');
                var path = parts.slice(1).join('/') || parts[0];
                state.files[path] = { path: path, name: file.name, kind: 'file', size: file.size, content: null, file: file };
                var cursor = tree;
                for (var p = 1; p < parts.length; p += 1) {
                    var segment = parts[p];
                    var last = p === parts.length - 1;
                    if (last) {
                        (cursor.files = cursor.files || []).push({ path: path, name: segment, kind: 'file' });
                    } else {
                        cursor.children = cursor.children || {};
                        cursor.children[segment] = cursor.children[segment] || { files: [], children: {} };
                        cursor = cursor.children[segment];
                    }
                }
            }
            function build(node, path) {
                var entries = [];
                Object.keys(node.children || {}).forEach(function (name) {
                    entries.push({ name: name, kind: 'directory', path: path ? path + '/' + name : name, children: build(node.children[name], path ? path + '/' + name : name) });
                });
                entries.sort(function (a, b) { return a.kind !== b.kind ? (a.kind === 'directory' ? -1 : 1) : a.name.localeCompare(b.name); });
                return entries.concat((node.files || []).slice().sort(function (a, b) { return a.name.localeCompare(b.name); }));
            }
            state.tree = build(tree, '');
            updateWorkspaceUI();
            renderTree();
            emit();
            input.remove();
            toast('Imported ' + Object.keys(state.files).length + ' files (read-only session).', 'success');
        });
        document.body.appendChild(input);
        input.click();
    }

    function demoWorkspace() {
        try { window.localStorage.removeItem(TABS_KEY); } catch (error) { /* optional */ }
        state.tabs = [];
        state.activePath = null;
        var samples = {
            'web/studio/index.html': '<!DOCTYPE html>\n<html lang="en">\n<head>\n    <title>UOR-R4 Studio</title>\n    <link rel="stylesheet" href="./index.css">\n</head>\n<body>\n    <!-- sample workspace file loaded from the demo tree -->\n</body>\n</html>\n',
            'web/studio/index.css': '/* UOR-R4 Studio — warm graphite + ember */\n:root {\n    --bg: #0b0a09;\n    --accent: #e8894a;\n}\n.studio { color: var(--accent); }\n',
            'web/studio/assets/js/studio_ui.js': '// Interaction layer: theme, command palette, navigation, sessions.\nexport function init() {\n    console.log("studio ui");\n}\n',
            'web/studio/README.md': '# UOR-R4 Studio\n\nOffline-first agent workspace for the UOR-R4 project.\n\n- Open a folder in Files.\n- Edit files in Editor tabs.\n- Attach context and ask the agent.\n',
            'crates/uor-r4-integer/src/lib.rs': '// Sample Rust module from the demo tree.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n',
            'tools/README.md': '# Tools\n\nVerification scripts, tokenizer tools and build helpers.\n'
        };
        state.rootHandle = null;
        state.rootName = 'uor-r4';
        state.mode = 'demo';
        state.files = {};
        var tree = {};
        Object.keys(samples).forEach(function (path) {
            state.files[path] = { path: path, name: baseName(path), kind: 'file', content: samples[path], size: samples[path].length };
            var parts = path.split('/');
            var cursor = tree;
            for (var i = 0; i < parts.length; i += 1) {
                var segment = parts[i];
                var last = i === parts.length - 1;
                if (last) {
                    (cursor.files = cursor.files || []).push({ path: path, name: segment, kind: 'file' });
                } else {
                    cursor.children = cursor.children || {};
                    cursor.children[segment] = cursor.children[segment] || { files: [], children: {} };
                    cursor = cursor.children[segment];
                }
            }
        });
        function build(node, path) {
            var entries = [];
            Object.keys(node.children || {}).forEach(function (name) {
                entries.push({ name: name, kind: 'directory', path: path ? path + '/' + name : name, children: build(node.children[name], path ? path + '/' + name : name) });
            });
            entries.sort(function (a, b) { return a.kind !== b.kind ? (a.kind === 'directory' ? -1 : 1) : a.name.localeCompare(b.name); });
            return entries.concat((node.files || []).slice().sort(function (a, b) { return a.name.localeCompare(b.name); }));
        }
        state.tree = build(tree, '');
        updateWorkspaceUI();
        renderTree();
        openFile('web/studio/index.html').then(function () { return openFile('web/studio/index.css'); });
        window.setTimeout(function () { if (state.activePath) loadTabIntoEditor(state.activePath, true); }, 800);
        window.setTimeout(function () { if (state.activePath) loadTabIntoEditor(state.activePath, true); }, 1600);
        window.setTimeout(function () {
            if (state.tabs.length) {
                state.tabs[0].dirty = true;
                state.activePath = state.tabs[0].path;
                loadTabIntoEditor(state.tabs[0].path, true);
                renderTabs();
                var badge = document.getElementById('editorSavedBadge');
                if (badge) badge.textContent = 'Unsaved changes';
            }
        }, 1700);
        emit();
        toast('Loaded the demo workspace tree (screenshot/QA state).', 'info');
    }

    function renderTree() {
        var container = document.getElementById('fileTree');
        if (!container) return;
        container.textContent = '';
        if (!state.tree.length) {
            var empty = document.createElement('div');
            empty.className = 'file-tree-empty';
            empty.innerHTML = '';
            var title = document.createElement('p');
            title.className = 'file-tree-empty-title';
            title.textContent = 'No folder open';
            var hint = document.createElement('p');
            hint.className = 'hint';
            hint.textContent = 'Use Open folder… to grant this tab access to a local project folder (File System Access API). Files are read and written in place; nothing is uploaded. Fallback: Import files… reads a folder into this session only.';
            var button = document.createElement('button');
            button.type = 'button';
            button.className = 'primary-btn primary-btn-sm';
            button.textContent = 'Open folder…';
            button.addEventListener('click', openFolder);
            empty.appendChild(title);
            empty.appendChild(hint);
            empty.appendChild(button);
            container.appendChild(empty);
        } else {
            container.appendChild(buildTreeList(state.tree, 0));
        }
        var meta = document.getElementById('fileTreeMeta');
        if (meta) meta.textContent = state.rootName ? state.rootName + ' · ' + Object.keys(state.files).length + ' files' : 'no folder open';
    }

    function buildTreeList(entries, depth) {
        var list = document.createElement('ul');
        list.className = 'tree-list';
        list.setAttribute('role', 'group');
        entries.forEach(function (entry) {
            var item = document.createElement('li');
            item.className = 'tree-item';
            var row = document.createElement('div');
            row.className = 'tree-row';
            row.style.paddingLeft = (8 + depth * 14) + 'px';
            if (entry.kind === 'directory') {
                var folderButton = document.createElement('button');
                folderButton.type = 'button';
                folderButton.className = 'tree-button tree-folder';
                folderButton.setAttribute('aria-expanded', 'true');
                folderButton.innerHTML = '<span class="tree-caret" aria-hidden="true">▾</span><span class="tree-icon" aria-hidden="true">▸</span><span class="tree-name"></span>';
                folderButton.querySelector('.tree-name').textContent = entry.name;
                var children = buildTreeList(entry.children || [], depth + 1);
                folderButton.addEventListener('click', function () {
                    var expanded = folderButton.getAttribute('aria-expanded') !== 'false';
                    folderButton.setAttribute('aria-expanded', expanded ? 'false' : 'true');
                    folderButton.querySelector('.tree-caret').textContent = expanded ? '▸' : '▾';
                    children.hidden = expanded;
                });
                row.appendChild(folderButton);
                item.appendChild(row);
                item.appendChild(children);
            } else {
                var fileButton = document.createElement('button');
                fileButton.type = 'button';
                fileButton.className = 'tree-button tree-file';
                fileButton.title = entry.path;
                fileButton.innerHTML = '<span class="tree-caret tree-caret-empty" aria-hidden="true"></span><span class="tree-icon" aria-hidden="true">·</span><span class="tree-name"></span><span class="tree-attach" role="button" aria-label="Attach as agent context" title="Attach as agent context">＋</span>';
                fileButton.querySelector('.tree-name').textContent = entry.name;
                fileButton.addEventListener('click', function (event) {
                    if (event.target && event.target.classList.contains('tree-attach')) { event.stopPropagation(); attachPath(entry.path); return; }
                    openFile(entry.path);
                });
                row.appendChild(fileButton);
                item.appendChild(row);
            }
            list.appendChild(item);
        });
        return list;
    }

    /* ------------------------------------------------------------------ files + tabs */

    async function readPath(path) {
        var record = state.files[path];
        if (!record) return null;
        if (record.handle) {
            var file = await record.handle.getFile();
            if (file.size > TEXT_LIMIT) throw new Error('File is larger than 2 MB.');
            return file.text();
        }
        if (record.file) {
            if (record.file.size > TEXT_LIMIT) throw new Error('File is larger than 2 MB.');
            return record.file.text();
        }
        return record.content == null ? '' : record.content;
    }

    async function openFile(path) {
        if (!state.files[path]) return;
        var existing = state.tabs.find(function (tab) { return tab.path === path; });
        if (existing) { activateTab(path); return; }
        try {
            var content = await readPath(path);
            var record = state.files[path];
            state.tabs.push({
                path: path,
                name: baseName(path),
                content: content,
                dirty: false,
                mode: record.handle ? 'directory' : (state.mode === 'demo' ? 'demo' : (record.file ? 'imported' : 'local')),
                handle: record.handle || null
            });
            addRecent(path);
            activateTab(path);
        } catch (error) {
            toast('Could not open ' + path + ': ' + (error && error.message ? error.message : error), 'error');
        }
    }

    function addRecent(path) {
        state.recent = [path].concat(state.recent.filter(function (item) { return item !== path; })).slice(0, 12);
        persistRecent();
        renderRecent();
    }

    function renderRecent() {
        var list = document.getElementById('recentFiles');
        if (!list) return;
        list.textContent = '';
        if (!state.recent.length) {
            var empty = document.createElement('li');
            empty.className = 'hint';
            empty.textContent = 'No files opened yet.';
            list.appendChild(empty);
            return;
        }
        state.recent.forEach(function (path) {
            var item = document.createElement('li');
            var button = document.createElement('button');
            button.type = 'button';
            button.className = 'recent-file';
            button.textContent = path;
            button.addEventListener('click', function () { if (state.files[path]) openFile(path); });
            item.appendChild(button);
            list.appendChild(item);
        });
    }

    function activateTab(path) {
        state.activePath = path;
        loadTabIntoEditor(path, false);
        renderTabs();
        persistTabs();
        emit();
        if (document.body.dataset.view !== 'editor') {
            if (window.__studioUi && typeof window.__studioUi.setView === 'function') window.__studioUi.setView('workspace');
        }
    }

    function tabFor(path) { return state.tabs.find(function (tab) { return tab.path === path; }); }

    function loadTabIntoEditor(path, silent) {
        var tab = tabFor(path);
        if (!tab) return;
        state.loadingTab = true;
        var text = document.getElementById('editorText');
        var name = document.getElementById('editorFileName');
        var language = document.getElementById('editorLanguage');
        if (text) { text.value = tab.content; text.dispatchEvent(new Event('input', { bubbles: true })); }
        if (name) name.value = tab.name;
        if (language) {
            var lang = languageFor(tab.path);
            if (Array.from(language.options).some(function (option) { return option.value === lang; })) language.value = lang;
        }
        updateEditorHeader(tab);
        updateStats(tab.content);
        state.loadingTab = false;
        if (!silent) {
            var meta = document.getElementById('editorSavedBadge');
            if (meta) meta.textContent = tab.dirty ? 'Unsaved changes' : (tab.mode === 'directory' ? 'Saved to disk' : 'Local tab');
        }
    }

    function updateEditorHeader(tab) {
        var badge = document.getElementById('editorSavedBadge');
        if (badge) {
            badge.textContent = tab.dirty ? 'Unsaved changes' : (tab.mode === 'directory' ? 'Saved to disk' : (tab.mode === 'demo' ? 'Demo file' : 'Local tab'));
            badge.setAttribute('data-state', tab.dirty ? 'dirty' : 'saved');
        }
        var unsaved = document.getElementById('editorUnsaved');
        if (unsaved) unsaved.textContent = tab.dirty ? 'Unsaved changes' : 'Saved';
        var state = document.getElementById('editorSaveState');
        if (state) state.textContent = tab.dirty ? 'Unsaved' : (tab.mode === 'directory' ? 'Saved to disk' : 'Saved (local)');
        var stateBottom = document.getElementById('editorSaveStateBottom');
        if (stateBottom) stateBottom.textContent = tab.dirty ? '● unsaved' : 'saved';
    }

    function updateStats(content) {
        var text = String(content || '');
        var lines = text ? text.split('\n').length : 1;
        var stats = document.getElementById('editorStats');
        if (stats) stats.textContent = lines + (lines === 1 ? ' line' : ' lines') + ' · ' + text.length + ' chars';
    }

    function renderTabs() {
        var strip = document.getElementById('editorTabs');
        if (!strip) return;
        strip.textContent = '';
        if (!state.tabs.length) {
            var empty = document.createElement('span');
            empty.className = 'editor-tabs-empty';
            empty.textContent = 'No open files. Open a file from Files, or create a new one.';
            strip.appendChild(empty);
            return;
        }
        state.tabs.forEach(function (tab) {
            var item = document.createElement('div');
            item.className = 'editor-tab';
            item.setAttribute('role', 'tab');
            item.setAttribute('aria-selected', tab.path === state.activePath ? 'true' : 'false');
            item.setAttribute('tabindex', '0');
            var name = document.createElement('span');
            name.className = 'editor-tab-name';
            name.textContent = tab.name;
            item.appendChild(name);
            if (tab.dirty) {
                var dot = document.createElement('span');
                dot.className = 'editor-tab-dot';
                dot.setAttribute('aria-label', 'unsaved');
                dot.textContent = '●';
                item.appendChild(dot);
            }
            var close = document.createElement('button');
            close.type = 'button';
            close.className = 'editor-tab-close';
            close.setAttribute('aria-label', 'Close ' + tab.name);
            close.textContent = '×';
            close.addEventListener('click', function (event) { event.stopPropagation(); closeTab(tab.path); });
            item.appendChild(close);
            item.addEventListener('click', function () { activateTab(tab.path); });
            item.addEventListener('keydown', function (event) { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); activateTab(tab.path); } });
            strip.appendChild(item);
        });
    }

    function closeTab(path) {
        var index = state.tabs.findIndex(function (tab) { return tab.path === path; });
        if (index < 0) return;
        state.tabs.splice(index, 1);
        if (state.activePath === path) {
            var next = state.tabs[Math.max(0, index - 1)] || state.tabs[0];
            state.activePath = next ? next.path : null;
            if (next) loadTabIntoEditor(next.path, false);
            else {
                var text = document.getElementById('editorText');
                if (text) text.value = '';
                renderTabs();
            }
        }
        renderTabs();
        persistTabs();
        emit();
    }

    function newFile() {
        var index = 1;
        var name = 'untitled.md';
        while (state.tabs.some(function (tab) { return tab.path === name; })) { index += 1; name = 'untitled-' + index + '.md'; }
        state.tabs.push({ path: name, name: name, content: '', dirty: true, mode: state.mode === 'directory' ? 'directory' : 'local', handle: null });
        state.activePath = name;
        loadTabIntoEditor(name, false);
        renderTabs();
        persistTabs();
        emit();
        toast('New file tab: ' + name, 'info');
    }

    async function saveActive() {
        var tab = tabFor(state.activePath);
        if (!tab) { toast('No file tab open. Open a file in Files first.', 'info'); return false; }
        var text = document.getElementById('editorText');
        tab.content = text ? text.value : tab.content;
        if (tab.mode === 'directory' && tab.handle) {
            try {
                if (tab.handle.queryPermission) {
                    var permission = await tab.handle.queryPermission({ mode: 'readwrite' });
                    if (permission !== 'granted') {
                        permission = await tab.handle.requestPermission({ mode: 'readwrite' });
                    }
                    if (permission !== 'granted') { toast('Write permission was not granted; file not saved.', 'warn'); return false; }
                }
                var writable = await tab.handle.createWritable();
                await writable.write(tab.content);
                await writable.close();
                tab.dirty = false;
                updateEditorHeader(tab);
                renderTabs();
                persistTabs();
                emit();
                toast('Saved ' + tab.path + ' to disk.', 'success');
                return true;
            } catch (error) {
                toast('Save failed: ' + (error && error.message ? error.message : error), 'error');
                return false;
            }
        }
        // No writable handle: download a copy (explicit fallback; still browser-only).
        var blob = new Blob([tab.content], { type: 'text/plain;charset=utf-8' });
        var url = URL.createObjectURL(blob);
        var link = document.createElement('a');
        link.href = url;
        link.download = tab.name || 'file.txt';
        document.body.appendChild(link);
        link.click();
        link.remove();
        window.setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
        tab.dirty = false;
        updateEditorHeader(tab);
        renderTabs();
        persistTabs();
        emit();
        toast('Downloaded ' + tab.name + ' (no disk handle for this tab).', 'success');
        return true;
    }

    function saveAsCopy() {
        var tab = tabFor(state.activePath);
        if (!tab) { toast('No file tab open.', 'info'); return false; }
        var text = document.getElementById('editorText');
        tab.content = text ? text.value : tab.content;
        var blob = new Blob([tab.content], { type: 'text/plain;charset=utf-8' });
        var url = URL.createObjectURL(blob);
        var link = document.createElement('a');
        link.href = url;
        link.download = tab.name || 'file.txt';
        document.body.appendChild(link);
        link.click();
        link.remove();
        window.setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
        toast('Saved a copy of ' + (tab.name || 'file') + '.', 'success');
        return true;
    }

    function attachPath(path) {
        var record = state.files[path];
        if (!record) return;
        readPath(path).then(function (content) {
            document.dispatchEvent(new CustomEvent('uor-attach-file', { detail: { name: path, content: content, source: 'workspace' } }));
            toast('Attached ' + path + ' as agent context.', 'success');
        }).catch(function (error) { toast('Could not attach ' + path + ': ' + error, 'error'); });
    }

    function activeFile() {
        var tab = tabFor(state.activePath);
        if (!tab) return null;
        var text = document.getElementById('editorText');
        return { path: tab.path, name: tab.name, content: text ? text.value : tab.content, mode: tab.mode };
    }

    function setActiveContent(content) {
        var tab = tabFor(state.activePath);
        if (!tab) return false;
        tab.content = String(content);
        tab.dirty = true;
        var text = document.getElementById('editorText');
        if (text) text.value = tab.content;
        updateEditorHeader(tab);
        updateStats(tab.content);
        renderTabs();
        persistTabs();
        emit();
        return true;
    }

    function insertText(text, name) {
        var tabName = name || 'agent-snippet.md';
        var existing = state.tabs.find(function (tab) { return tab.path === tabName; });
        if (existing) {
            existing.content = text;
            existing.dirty = true;
            state.activePath = existing.path;
        } else {
            state.tabs.push({ path: tabName, name: tabName, content: text, dirty: true, mode: state.mode === 'directory' ? 'directory' : 'local', handle: null });
            state.activePath = tabName;
        }
        loadTabIntoEditor(state.activePath, false);
        renderTabs();
        persistTabs();
        emit();
        if (window.__studioUi && typeof window.__studioUi.setView === 'function') window.__studioUi.setView('workspace');
        return true;
    }

    function closeAll() {
        state.tabs = [];
        state.activePath = null;
        var text = document.getElementById('editorText');
        if (text) text.value = '';
        renderTabs();
        persistTabs();
        emit();
    }

    /* ------------------------------------------------------------------ wiring */

    function updateWorkspaceUI() {
        var name = document.getElementById('workspaceName');
        var meta = document.getElementById('workspaceMeta');
        var crumb = document.getElementById('crumbWorkspace');
        var folder = document.getElementById('statusFolder');
        var ctxLabel = document.getElementById('ctxWorkspaceLabel');
        var ctxMeta = document.getElementById('ctxWorkspaceMeta');
        var badge = document.getElementById('filesAccessBadge');
        var label = state.rootName || 'No folder open';
        if (name) name.textContent = label;
        if (crumb) crumb.textContent = state.rootName || 'no folder';
        var topCrumb = document.getElementById('topWorkspaceName');
        if (topCrumb) topCrumb.textContent = state.rootName ? state.rootName + ' / main' : 'uor-r4 / main';
        if (folder) folder.textContent = state.rootName || 'no folder';
        if (ctxLabel) ctxLabel.textContent = label;
        if (meta) meta.textContent = state.mode === 'directory' ? 'File System Access · read/write' : (state.mode === 'demo' ? 'demo tree · QA state' : (state.mode === 'imported' ? 'imported · read-only session' : 'browser workspace · open folder'));
        if (ctxMeta) ctxMeta.textContent = state.mode === 'directory' ? 'Read/write access to this folder in this tab.' : (state.mode === 'demo' ? 'Demo tree for screenshots; open a real folder for disk access.' : (state.mode === 'imported' ? 'Imported read-only copies; save downloads a copy.' : 'Open a folder in Files to enable editor tabs and save.'));
        if (badge) badge.textContent = state.mode === 'directory' ? 'File System Access · read/write' : (state.mode === 'demo' ? 'demo tree' : (state.mode === 'imported' ? 'imported · read-only' : 'browser workspace'));
        document.dispatchEvent(new CustomEvent('uor-workspace-changed', { detail: summary() }));
    }

    function replaceButton(id, handler, title) {
        var node = document.getElementById(id);
        if (!node) return;
        var clone = node.cloneNode(true);
        clone.id = id;
        if (title) clone.title = title;
        if (handler) clone.addEventListener('click', handler);
        node.replaceWith(clone);
    }

    function wire() {
        replaceButton('btnEditorNew', newFile, 'New file tab');
        replaceButton('btnEditorOpen', openFolder, 'Open folder');
        replaceButton('btnEditorDownload', saveActive, 'Save active file (compat)');
        var saveButton = document.getElementById('btnSaveActive');
        if (saveButton) saveButton.addEventListener('click', function () { saveActive(); });
        var saveAsButton = document.getElementById('btnSaveAs');
        if (saveAsButton) saveAsButton.addEventListener('click', saveAsCopy);
        ['btnOpenFolderFiles', 'btnOpenFolderEmpty', 'btnOpenFolderSettings'].forEach(function (id) {
            var node = document.getElementById(id);
            if (node) node.addEventListener('click', openFolder);
        });
        var reload = document.getElementById('btnReloadTree');
        if (reload) reload.addEventListener('click', async function () {
            if (state.mode === 'directory' && state.rootHandle) {
                state.files = {};
                state.tree = await readDirectory(state.rootHandle, '', 0);
                renderTree();
                toast('Workspace tree reloaded.', 'success');
            } else if (state.mode === 'demo') {
                demoWorkspace();
            } else {
                toast('Open a folder first.', 'info');
            }
        });
        var importButton = document.getElementById('btnImportFiles');
        if (importButton) importButton.addEventListener('click', function () { importFiles(); });
        var workspaceSelector = document.getElementById('workspaceSelector');
        if (workspaceSelector) workspaceSelector.addEventListener('click', openFolder);
        var text = document.getElementById('editorText');
        if (text) {
            text.addEventListener('input', function () {
                var tab = tabFor(state.activePath);
                if (!tab || state.loadingTab) return;
                tab.content = text.value;
                tab.dirty = true;
                updateEditorHeader(tab);
                updateStats(tab.content);
                renderTabs();
                persistTabs();
            });
            text.addEventListener('keydown', function (event) {
                if (event.key === 'Tab') {
                    var settings = window.__studioUi && window.__studioUi.getSettings ? window.__studioUi.getSettings() : { tabSize: 4 };
                    event.preventDefault();
                    var start = text.selectionStart;
                    var end = text.selectionEnd;
                    var padding = new Array((settings.tabSize || 4) + 1).join(' ');
                    text.setRangeText(padding, start, end, 'end');
                    text.dispatchEvent(new Event('input', { bubbles: true }));
                }
            });
        }
        document.addEventListener('keydown', function (event) {
            var meta = event.metaKey || event.ctrlKey;
            if (!meta) return;
            if ((event.key === 's' || event.key === 'S') && event.shiftKey) { event.preventDefault(); saveAsCopy(); return; }
            if (event.key === 's' || event.key === 'S') { event.preventDefault(); saveActive(); }
            if (event.key === 'o' || event.key === 'O') { event.preventDefault(); openFolder(); }
        });
        document.addEventListener('uor-attach-active-file', function () {
            var file = activeFile();
            if (!file) { toast('Open a file in the editor first.', 'info'); return; }
            document.dispatchEvent(new CustomEvent('uor-attach-file', { detail: { name: file.path, content: file.content, source: 'active-editor' } }));
        });
        document.addEventListener('uor-files-request-tab', function () { newFile(); });
    }

    function boot() {
        restoreRecent();
        restoreTabs();
        wire();
        updateWorkspaceUI();
        renderTree();
        window.setTimeout(function () { if (state.activePath) loadTabIntoEditor(state.activePath, true); }, 400);
        document.addEventListener('uor-studio-ready', function () { if (state.activePath) loadTabIntoEditor(state.activePath, true); });
        document.addEventListener('uor-files-ready', function () { if (state.activePath) loadTabIntoEditor(state.activePath, true); });
        var hash = window.location.hash || '';
        if (hash === '#demo=workspace') window.setTimeout(demoWorkspace, 260);
        window.__studioFiles = {
            version: 'v2',
            openFolder: openFolder,
            importFiles: importFiles,
            openFile: openFile,
            openDemo: demoWorkspace,
            saveActive: saveActive,
            saveAsCopy: saveAsCopy,
            newFile: newFile,
            closeTab: closeTab,
            closeAll: closeAll,
            insertText: insertText,
            setActiveContent: setActiveContent,
            activeFile: activeFile,
            workspaceName: function () { return state.rootName; },
            mode: function () { return state.mode; },
            tabs: function () { return state.tabs.map(function (tab) { return { path: tab.path, dirty: tab.dirty, mode: tab.mode }; }); },
            treeSummary: function () { return summary(); }
        };
        document.dispatchEvent(new CustomEvent('uor-files-ready'));
    }

    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
    else boot();
}());
