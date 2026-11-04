// UOR-R4 Studio v2 — shell interactions: labeled navigation, grouped command palette,
// sessions menu, context attachments, build-task templates, message actions, settings,
// theme/chrome persistence and live status mirrors. Browser-only, no external assets.

(function () {
    'use strict';

    var SETTINGS_KEY = 'uor.studio.settings.v1';
    var UI_KEY = 'uor.studio.ui.v2';
    var ATTACH_KEY = 'uor.studio.attachments.v1';
    var IMAGE_RECALL_KEY = 'uor.studio.attachments.images.v1';
    var MAX_ATTACHMENT_TOTAL_BYTES = 4 * 1024 * 1024;
    var MAX_IMAGE_TOTAL_BYTES = 16 * 1024 * 1024;
    var MAX_IMAGES = 4;
    var THEME_KEY = 'uor.studio.theme';

    var app = document.getElementById('app');
    var ready = false;
    var clockTimer = null;
    var lastFocus = null;

    function $(selector, root) { return (root || document).querySelector(selector); }
    function $all(selector, root) { return Array.prototype.slice.call((root || document).querySelectorAll(selector)); }
    function hook(name) { var ui = window.__studioUi; return ui && typeof ui[name] === 'function' ? ui[name] : null; }

    function readJson(key, fallback) { try { var raw = window.localStorage.getItem(key); return raw ? JSON.parse(raw) : fallback; } catch (error) { return fallback; } }
    function writeJson(key, value) { try { window.localStorage.setItem(key, JSON.stringify(value)); } catch (error) { /* optional */ } }
    function readUi() { var parsed = readJson(UI_KEY, {}); return parsed && typeof parsed === 'object' ? parsed : {}; }
    function writeUi(patch) { var state = readUi(); Object.keys(patch).forEach(function (key) { state[key] = patch[key]; }); writeJson(UI_KEY, state); }

    var toastHooks = window.__studioUi;
    function toast(text, tone) {
        if (toastHooks && typeof toastHooks.toast === 'function') { toastHooks.toast(text, tone || 'info'); return; }
        var region = document.getElementById('toastRegion');
        if (!region) return;
        var node = document.createElement('div');
        node.className = 'toast';
        node.setAttribute('data-tone', tone || 'info');
        node.textContent = text;
        region.appendChild(node);
        window.setTimeout(function () { node.remove(); }, 4200);
    }

    /* settings */
    var defaults = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };
    var settings = Object.assign({}, defaults, readJson(SETTINGS_KEY, {}));
    function saveSettings() { writeJson(SETTINGS_KEY, settings); }
    function systemTheme() { return window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'; }
    function effectiveTheme() { return settings.theme === 'system' ? systemTheme() : (settings.theme === 'light' ? 'light' : 'dark'); }
    function applySettings() {
        var theme = effectiveTheme();
        document.documentElement.setAttribute('data-theme', theme);
        document.documentElement.style.colorScheme = theme;
        try { window.localStorage.setItem(THEME_KEY, theme); } catch (error) { /* optional */ }
        document.documentElement.setAttribute('data-reduce-motion', settings.reduceMotion ? '1' : '0');
        document.documentElement.style.setProperty('--editor-font-size', (settings.fontSize || 13) + 'px');
        var editor = document.getElementById('editorText');
        if (editor) editor.setAttribute('wrap', settings.wordWrap ? 'soft' : 'off');
        $all('[data-setting-theme]').forEach(function (button) { button.setAttribute('aria-pressed', button.getAttribute('data-setting-theme') === settings.theme ? 'true' : 'false'); });
        var reduce = document.getElementById('settingReduceMotion');
        if (reduce) reduce.checked = Boolean(settings.reduceMotion);
        var fontSize = document.getElementById('settingFontSize');
        if (fontSize) fontSize.value = String(settings.fontSize || 13);
        var tabSize = document.getElementById('settingTabSize');
        if (tabSize) tabSize.value = String(settings.tabSize || 4);
        var wrap = document.getElementById('settingWordWrap');
        if (wrap) wrap.checked = Boolean(settings.wordWrap);
        window.dispatchEvent(new CustomEvent('uor-settings-changed', { detail: Object.assign({}, settings) }));
    }
    function setThemeChoice(theme) { settings.theme = theme === 'light' ? 'light' : (theme === 'system' ? 'system' : 'dark'); saveSettings(); applySettings(); updateThemeButton(); }
    function updateSettings(patch) {
        if (!patch || typeof patch !== 'object') return;
        if (patch.theme !== undefined) settings.theme = patch.theme === 'light' ? 'light' : (patch.theme === 'system' ? 'system' : 'dark');
        if (patch.reduceMotion !== undefined) settings.reduceMotion = Boolean(patch.reduceMotion);
        if (patch.fontSize !== undefined) settings.fontSize = Math.max(10, Math.min(24, Number.isFinite(Number(patch.fontSize)) ? Number(patch.fontSize) : defaults.fontSize));
        if (patch.tabSize !== undefined) settings.tabSize = Math.max(1, Math.min(8, Number.isFinite(Number(patch.tabSize)) ? Number(patch.tabSize) : defaults.tabSize));
        if (patch.wordWrap !== undefined) settings.wordWrap = Boolean(patch.wordWrap);
        saveSettings();
        applySettings();
        updateThemeButton();
    }

    function updateThemeButton() {
        var label = effectiveTheme() === 'light' ? 'Switch to dark theme' : 'Switch to light theme';
        ['btnTheme', 'railTheme'].forEach(function (id) { var node = document.getElementById(id); if (node) node.setAttribute('title', label); });
    }
    function toggleTheme() { var next = effectiveTheme() === 'light' ? 'dark' : 'light'; setThemeChoice(next); toast('Theme: ' + next, 'info'); }

    /* navigation */
    var SECTIONS = {
        agent: { view: 'chat', label: 'Agent', purpose: 'Chat with the agent, manage sessions, attach context and run build tasks.' },
        workspace: { view: 'workspace', label: 'Workspace', purpose: 'Open a folder, edit files in tabs and save them back to disk.' },
        models: { view: 'model', label: 'Connections', purpose: 'UOR Chat, custom API providers, and artifact diagnostics.' },
        guide: { view: 'guide', label: 'Guide', purpose: 'Quick start, the build-this-studio workflow, shortcuts and honest limits.' }
    };
    function sectionForView(view) { return view === 'chat' ? 'agent' : (view === 'model' ? 'models' : (view === 'workspace' || view === 'editor' || view === 'files' ? 'workspace' : (SECTIONS[view] ? view : 'agent'))); }
    function syncSectionNavigation(section) {
        $all('.nav-item').forEach(function (item) { var active = item.getAttribute('data-section') === section; item.setAttribute('aria-current', active ? 'page' : 'false'); item.classList.toggle('is-active', active); });
    }
    function activateSection(name, options) {
        var section = SECTIONS[name] ? name : 'agent';
        var meta = SECTIONS[section];
        var setView = hook('setView');
        if (setView) setView(meta.view);
        else { var button = document.getElementById('viewBtn' + meta.view.charAt(0).toUpperCase() + meta.view.slice(1)); if (button) button.click(); }
        document.body.setAttribute('data-section', section);
        syncSectionNavigation(section);
        var label = document.getElementById('activeSectionLabel');
        if (label) label.textContent = meta.label;
        var sessions = document.getElementById('sessionSection');
        if (sessions) sessions.classList.remove('is-hidden');
        if (!options || !options.silent) {
            if (window.matchMedia('(max-width: 900px)').matches) setChrome('nav', 'collapsed', true);
            updateStatus();
        }
    }
    function chromeValue(name, fallback) { var stored = readUi()[name]; return stored === 'expanded' || stored === 'collapsed' ? stored : fallback; }
    function syncNavPresentation() {
        if (!app) return;
        var expanded = app.getAttribute('data-nav') === 'expanded';
        var mobile = window.matchMedia('(max-width: 900px)').matches;
        document.documentElement.classList.toggle('nav-open', expanded && mobile);
        var scrim = document.getElementById('btnNavScrim');
        if (scrim) scrim.hidden = !(expanded && mobile);
        var main = document.getElementById('mainColumn');
        if (main) main.inert = expanded && mobile;
        if (expanded && mobile && !paletteOpen && !document.querySelector('.modal-backdrop:not([hidden])')) {
            var sidebar = document.getElementById('navSidebar');
            if (sidebar && !sidebar.contains(document.activeElement)) {
                var firstDestination = sidebar.querySelector('button[data-section]');
                if (firstDestination) firstDestination.focus();
            }
        }
    }
    function setChrome(name, value, persist) {
        if (!app) return;
        var wasExpanded = app.getAttribute('data-nav') === 'expanded';
        app.setAttribute('data-' + name, value);
        var expanded = value === 'expanded';
        var map = { nav: ['btnToggleNav', 'btnNavCollapse', 'btnMobileNav'], inspector: ['btnToggleInspector', 'btnToggleInspector2', 'btnToggleInspector3', 'btnToggleInspector4', 'btnToggleInspectorTop'] };
        (map[name] || []).forEach(function (id) { var node = document.getElementById(id); if (node) node.setAttribute('aria-expanded', expanded ? 'true' : 'false'); });
        if (persist) { var patch = {}; patch[name] = value; writeUi(patch); }
        if (name === 'nav') {
            syncNavPresentation();
            var mobile = window.matchMedia('(max-width: 900px)').matches;
            if (expanded && mobile && !wasExpanded) {
                var firstDestination = document.querySelector('#navSidebar button[data-section]');
                if (firstDestination) firstDestination.focus();
            } else if (!expanded && mobile && wasExpanded) {
                var trigger = document.getElementById('btnMobileNav');
                if (trigger) trigger.focus();
            }
        }
    }
    function toggleChrome(name) { if (!app) return; var current = app.getAttribute('data-' + name) === 'collapsed' ? 'collapsed' : 'expanded'; setChrome(name, current === 'expanded' ? 'collapsed' : 'expanded', true); }

    /* models page grouping */
    var MODELS_SECTIONS = [
        { key: 'local', title: 'UOR Chat — experimental', desc: 'Synthetic continuation through the existing local WASM worker; not a trained conversational model.', match: /local wasm engine/i },
        { key: 'trained', title: 'Published models', desc: 'Trained 19-table M1 trunk exports; next-token prediction only, not a chat assistant.', match: /trained m1/i },
        { key: 'api', title: 'Native & custom chat sources', desc: 'Native loopback stays on this device; custom OpenAI-compatible providers send prompts to the configured provider.', match: /openai-compatible api/i },
        { key: 'research', title: 'Research artifacts (experimental)', desc: 'Earlier exports kept for inspection; not generation-capable in this Studio.', match: /research|experimental/i }
    ];
    function regroupModels() {
        var mount = document.getElementById('modelSourceMount');
        if (!mount || !document.getElementById('modelView')) return;
        var sections = document.getElementById('modelsSections');
        if (!sections) {
            sections = document.createElement('div');
            sections.id = 'modelsSections';
            sections.className = 'models-sections';
            var panel = document.getElementById('modelSourcesPanel');
            var providersHead = panel ? panel.querySelector('.providers-head') : null;
            if (panel && providersHead) providersHead.insertAdjacentElement('afterend', sections);
            else if (panel) panel.appendChild(sections);
            mount.classList.add('visually-hidden');
            MODELS_SECTIONS.filter(function (def) { return def.key === 'local' || def.key === 'api'; }).forEach(function (def) {
                var section = document.createElement('section');
                section.className = 'models-section';
                section.setAttribute('data-kind', def.key);
                var header = document.createElement('header');
                header.className = 'models-section-head';
                var title = document.createElement('h2');
                title.textContent = def.title;
                var desc = document.createElement('p');
                desc.className = 'models-section-desc';
                desc.textContent = def.desc;
                header.appendChild(title); header.appendChild(desc);
                var body = document.createElement('div');
                body.className = 'models-section-body';
                body.id = 'modelsSection-' + def.key;
                section.appendChild(header); section.appendChild(body);
                sections.appendChild(section);
            });
            var diagnostics = document.createElement('details');
            diagnostics.id = 'artifactDiagnostics';
            diagnostics.className = 'models-section artifact-diagnostics';
            var summary = document.createElement('summary');
            summary.textContent = 'Artifact diagnostics';
            var diagnosticsDesc = document.createElement('p');
            diagnosticsDesc.className = 'models-section-desc';
            diagnosticsDesc.textContent = 'Next-token prediction, local artifact tools, and research exports.';
            diagnostics.appendChild(summary);
            diagnostics.appendChild(diagnosticsDesc);
            ['trained', 'research', 'local-artifacts'].forEach(function (key) {
                var body = document.createElement('div');
                body.className = 'models-section-body';
                body.id = 'modelsSection-' + key;
                diagnostics.appendChild(body);
            });
            sections.appendChild(diagnostics);
            var status = document.createElement('div');
            status.className = 'models-verify-row';
            status.id = 'modelsVerifyRow';
            status.setAttribute('role', 'status');
            status.setAttribute('aria-live', 'polite');
            sections.appendChild(status);
            var verifyNote = document.createElement('div');
            verifyNote.className = 'models-verify-note';
            verifyNote.id = 'modelsVerifyNote';
            sections.appendChild(verifyNote);
        }
        var nativeGuide = mount.querySelector('#nativeRunnerGuide');
        if (nativeGuide && nativeGuide.parentElement !== sections) {
            sections.insertBefore(nativeGuide, sections.firstChild);
        }
        var freshGroups = mount.querySelectorAll('.source-group');
        if (freshGroups.length > 1) {
            Array.prototype.slice.call(document.querySelectorAll('.models-section-body')).forEach(function (body) { body.textContent = ''; });
        }
        Array.prototype.slice.call(freshGroups).forEach(function (group) {
            var heading = group.querySelector('h3');
            var text = heading ? heading.textContent : '';
            var def = MODELS_SECTIONS.find(function (item) { return item.match.test(text); }) || MODELS_SECTIONS[0];
            if (def.key === 'local') {
                var bundledBody = document.getElementById('modelsSection-local');
                var localArtifacts = document.getElementById('modelsSection-local-artifacts');
                Array.prototype.slice.call(group.querySelectorAll(':scope > .source-card')).forEach(function (card) {
                    var bundled = card.querySelector('[data-action="load-bundled"]');
                    var body = bundled ? bundledBody : localArtifacts;
                    if (body && card.parentElement !== body) body.appendChild(card);
                    if (bundled) bundled.textContent = 'Connect UOR Chat';
                });
                return;
            }
            var body = document.getElementById('modelsSection-' + def.key);
            if (def.key === 'trained' && !group.querySelector('.models-note')) {
                var note = document.createElement('p');
                note.className = 'models-note';
                note.textContent = "M1 seed 2 evidence is tracked on the roadmap item and ships when its release URL exists.";
                group.appendChild(note);
            }
            if (body && group.parentElement !== body) {
                if (heading) heading.classList.add('source-group-heading');
                body.appendChild(group);
            }
        });
        var verify = document.getElementById('modelsVerifyNote');
        if (verify) verify.textContent = 'Downloads, sizes, hashes and tokenizer bindings are verified in this tab before the worker receives an artifact. Remote contact is always user-initiated.';
    }
    function watchModels() {
        var mount = document.getElementById('modelSourceMount');
        if (!mount) return;
        var scheduled = false;
        new MutationObserver(function () { if (scheduled) return; scheduled = true; window.setTimeout(function () { scheduled = false; regroupModels(); }, 30); }).observe(mount, { childList: true, subtree: false });
        window.setTimeout(regroupModels, 400);
    }

    /* sessions */
    function sessionList() { var fn = hook('listSessions'); return fn ? fn() : []; }
    function activeSessionId() { var fn = hook('activeSessionId'); return fn ? fn() : null; }
    function sessionMenuButton(item) {
        if (item.querySelector('.conv-menu')) return;
        var button = document.createElement('button');
        button.type = 'button';
        button.className = 'conv-menu';
        button.setAttribute('aria-label', 'Session actions');
        button.setAttribute('aria-haspopup', 'menu');
        button.textContent = '⋯';
        button.addEventListener('click', function (event) { event.stopPropagation(); openSessionMenu(button, item); });
        item.appendChild(button);
    }
    var sessionPopover = null;
    var sessionMenuTrigger = null;
    function closeSessionPopover(restoreFocus) {
        if (sessionPopover) sessionPopover.remove();
        sessionPopover = null;
        if (sessionMenuTrigger) {
            sessionMenuTrigger.setAttribute('aria-expanded', 'false');
            if (restoreFocus !== false) sessionMenuTrigger.focus();
            sessionMenuTrigger = null;
        }
    }
    function openSessionMenu(trigger, item) {
        closeSessionPopover();
        var id = item.getAttribute('data-id') || item.getAttribute('data-conversation-id') || '';
        var title = (item.querySelector('.conv-title') || {}).textContent || 'Untitled session';
        sessionPopover = document.createElement('div');
        sessionPopover.className = 'session-popover';
        sessionPopover.setAttribute('role', 'menu');
        [['Rename…', function () { renameInline(item, id, title); }], ['Duplicate', function () { duplicateSession(id); }], ['Export JSON', function () { exportSession(id, title); }], ['Delete', function () { deleteSessionWithConfirm(id, trigger); }]].forEach(function (row) {
            var button = document.createElement('button');
            button.type = 'button';
            button.className = 'session-popover-item' + (row[0] === 'Delete' ? ' is-danger' : '');
            button.setAttribute('role', 'menuitem');
            button.textContent = row[0];
            button.addEventListener('click', function () { closeSessionPopover(); row[1](); });
            sessionPopover.appendChild(button);
        });
        document.body.appendChild(sessionPopover);
        var rect = trigger.getBoundingClientRect();
        sessionPopover.style.top = Math.round(rect.bottom + 4) + 'px';
        sessionPopover.style.left = Math.round(Math.min(rect.left, window.innerWidth - 190)) + 'px';
        window.setTimeout(function () {
            document.addEventListener('click', function outside(event) { if (sessionPopover && !sessionPopover.contains(event.target)) { closeSessionPopover(false); document.removeEventListener('click', outside); } });
        }, 0);
    }
    function renameInline(item, id, title) {
        var titleNode = item.querySelector('.conv-title, .session-title');
        if (!titleNode) return;
        var input = document.createElement('input');
        input.className = 'conv-rename';
        input.value = title;
        input.setAttribute('aria-label', 'Session title');
        titleNode.replaceWith(input);
        input.focus(); input.select();
        function commit() { var value = input.value.trim(); var rename = hook('renameSession'); if (value && rename) rename(id, value); else renderSessionEnhancements(); }
        input.addEventListener('keydown', function (event) { if (event.key === 'Enter') { event.preventDefault(); commit(); } if (event.key === 'Escape') { event.preventDefault(); renderSessionEnhancements(); } });
        input.addEventListener('blur', commit);
    }
    function duplicateSession(id) { var fn = hook('duplicateSession'); if (!fn) return; var newId = fn(id); window.setTimeout(renderSessionEnhancements, 30); toast(newId ? 'Session duplicated.' : 'Could not duplicate that session.', newId ? 'success' : 'error'); }
    function exportSession(id, title) {
        var fn = hook('exportSession');
        var payload = fn ? fn(id) : null;
        if (!payload) { toast('No session to export.', 'error'); return; }
        var blob = new Blob([payload], { type: 'application/json;charset=utf-8' });
        var url = URL.createObjectURL(blob);
        var link = document.createElement('a');
        link.href = url;
        link.download = 'uor-session-' + String(title || 'session').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 60) + '.json';
        document.body.appendChild(link); link.click(); link.remove();
        window.setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
        toast('Session exported as JSON.', 'success');
    }
    function deleteSessionWithConfirm(id, trigger) {
        if (trigger.dataset.confirm !== '1') { trigger.dataset.confirm = '1'; trigger.textContent = 'Delete?'; window.setTimeout(function () { trigger.dataset.confirm = '0'; trigger.textContent = 'Delete'; }, 2500); return; }
        var fn = hook('deleteSession'); if (fn) fn(id);
        window.setTimeout(renderSessionEnhancements, 30);
        toast('Session deleted.', 'success');
    }
    function renderSessionEnhancements() { $all('#conversationList .conversation-item').forEach(sessionMenuButton); updateStatus(); }
    function watchSessions() {
        var list = document.getElementById('conversationList');
        if (!list) return;
        var scheduled = false;
        new MutationObserver(function () { if (scheduled) return; scheduled = true; window.setTimeout(function () { scheduled = false; renderSessionEnhancements(); }, 40); }).observe(list, { childList: true, subtree: false });
        renderSessionEnhancements();
    }

    /* custom session rows: title / preview / relative time / sibling menu button */
    function relativeTime(ms) {
        if (!ms) return '';
        var diff = Math.max(0, Date.now() - Number(ms));
        var mins = Math.round(diff / 60000);
        if (mins < 1) return 'now';
        if (mins < 60) return mins + ' min ago';
        var hours = Math.round(mins / 60);
        if (hours < 24) return hours + ' hr ago';
        return Math.round(hours / 24) + ' d ago';
    }
    function renderSessionRows() {
        var rows = document.getElementById('sessionRowsV2');
        if (!rows) return;
        var sessions = sessionList();
        var active = activeSessionId();
        rows.textContent = '';
        var original = document.getElementById('conversationList');
        if (original) original.setAttribute('hidden', 'hidden');
        if (!sessions.length) {
            var empty = document.createElement('div');
            empty.className = 'sessions-empty';
            empty.textContent = 'No sessions yet. Start one to see the transcript here.';
            rows.appendChild(empty);
            return;
        }
        sessions.forEach(function (session) {
            var row = document.createElement('div');
            row.className = 'session-row';
            row.setAttribute('data-id', session.id);
            if (session.id === active) row.classList.add('is-active');
            var main = document.createElement('button');
            main.type = 'button';
            main.className = 'session-main';
            main.setAttribute('aria-label', 'Open session ' + session.title);
            var title = document.createElement('span'); title.className = 'session-title'; title.textContent = session.title;
            var preview = document.createElement('span'); preview.className = 'session-preview'; preview.textContent = session.preview || 'No messages yet';
            var meta = document.createElement('span'); meta.className = 'session-meta';
            meta.textContent = (session.messageCount || 0) + ' msg' + (session.messageCount === 1 ? '' : 's') + ' · ' + relativeTime(session.updatedAt || session.createdAt);
            main.appendChild(title); main.appendChild(preview); main.appendChild(meta);
            main.addEventListener('click', function () { var fn = hook('selectSession'); if (fn) fn(session.id); window.setTimeout(renderSessionRows, 30); });
                var menuButton = document.createElement('button');
            menuButton.type = 'button';
            menuButton.className = 'conv-menu';
            menuButton.setAttribute('aria-label', 'Session actions for ' + session.title);
            menuButton.setAttribute('aria-haspopup', 'menu');
            menuButton.textContent = '⋯';
            menuButton.addEventListener('click', function (event) { event.stopPropagation(); openSessionMenu(menuButton, row, session.id, session.title); });
            row.appendChild(main); row.appendChild(menuButton);
            rows.appendChild(row);
        });
    }
    function openSessionMenu(trigger, rowOrItem, id, title) {
        closeOtherModals(null, false);
        sessionMenuTrigger = trigger;
        trigger.setAttribute('aria-expanded', 'true');
        var sessionId = id || rowOrItem.getAttribute('data-id') || '';
        var sessionTitle = title || 'Untitled session';
        sessionPopover = document.createElement('div');
        sessionPopover.className = 'session-popover';
        sessionPopover.setAttribute('role', 'menu');
        [['Rename', function () { renameInline(rowOrItem, sessionId, sessionTitle); }], ['Duplicate', function () { duplicateSession(sessionId); }], ['Export JSON', function () { exportSession(sessionId, sessionTitle); }], ['Delete', function () { confirmDeleteSession(sessionId, trigger); }]].forEach(function (pair) {
            var button = document.createElement('button');
            button.type = 'button'; button.className = 'session-popover-item'; button.setAttribute('role', 'menuitem'); button.textContent = pair[0];
            button.addEventListener('click', function () { closeSessionPopover(); pair[1](); });
            sessionPopover.appendChild(button);
        });
        document.body.appendChild(sessionPopover);
        var firstItem = sessionPopover.querySelector('button');
        if (firstItem) firstItem.focus();
        var rect = trigger.getBoundingClientRect();
        sessionPopover.style.top = Math.round(rect.bottom + 4) + 'px';
        sessionPopover.style.left = Math.round(Math.max(8, Math.min(rect.left, window.innerWidth - 200))) + 'px';
        window.setTimeout(function () {
            document.addEventListener('click', function outside(event) { if (sessionPopover && !sessionPopover.contains(event.target)) { closeSessionPopover(false); document.removeEventListener('click', outside); } });
        }, 0);
    }
    function confirmDeleteSession(id, trigger) {
        var dialog = document.createElement('div');
        dialog.className = 'modal-backdrop';
        dialog.dataset.dynamic = 'true';
        dialog.hidden = true;
        dialog.setAttribute('role', 'dialog');
        dialog.setAttribute('aria-modal', 'true');
        dialog.setAttribute('aria-labelledby', 'confirmDeleteTitle');
        var box = document.createElement('div');
        box.className = 'modal modal-narrow';
        var head = document.createElement('header');
        head.className = 'modal-head';
        var title = document.createElement('h2');
        title.id = 'confirmDeleteTitle'; title.textContent = 'Delete this session?';
        head.appendChild(title);
        var body = document.createElement('div');
        body.className = 'modal-body';
        var text = document.createElement('p');
        text.className = 'hint';
        text.textContent = 'This cannot be undone.';
        body.appendChild(text);
        var footer = document.createElement('footer');
        footer.className = 'modal-footer';
        var cancel = document.createElement('button'); cancel.type = 'button'; cancel.className = 'ghost-btn'; cancel.textContent = 'Cancel';
        var del = document.createElement('button'); del.type = 'button'; del.className = 'btn btn-danger'; del.textContent = 'Delete';
        footer.appendChild(cancel); footer.appendChild(del);
        box.appendChild(head); box.appendChild(body); box.appendChild(footer);
        dialog.appendChild(box);
        document.body.appendChild(dialog);
        if (trigger) trigger.focus();
        openModal(dialog);
        cancel.addEventListener('click', function () { closeModal(dialog); });
        dialog.addEventListener('click', function (event) { if (event.target === dialog) closeModal(dialog); });
        del.addEventListener('click', function () {
            var fn = hook('deleteSession'); if (fn) fn(id);
            closeModal(dialog);
            window.setTimeout(renderSessionRows, 40);
            toast('Session deleted.', 'success');
        });
    }
    function newSessionAction() { var fn = hook('newSession'); if (fn) fn(); activateSection('agent'); window.setTimeout(renderSessionRows, 40); }
    function renderSessionEnhancements() { renderSessionRows(); updateStatus(); }
    function watchSessions() {
        var list = document.getElementById('conversationList');
        if (!list) return;
        new MutationObserver(function () { window.setTimeout(renderSessionRows, 30); }).observe(list, { childList: true, subtree: false });
        renderSessionRows();
    }

    /* attachments */
    var attachments = readJson(ATTACH_KEY, []);
    if (!Array.isArray(attachments)) attachments = [];
    attachments = attachments.filter(function (item) { return item && item.kind !== 'image'; });
    writeJson(ATTACH_KEY, attachments);
    var recalledImages = 0;
    try { recalledImages = Number(window.sessionStorage.getItem(IMAGE_RECALL_KEY)) || 0; } catch (error) { recalledImages = 0; }
    function rememberImageLifecycle() {
        var count = attachments.filter(function (item) { return item.kind === 'image'; }).length;
        try {
            if (count) window.sessionStorage.setItem(IMAGE_RECALL_KEY, String(count));
            else window.sessionStorage.removeItem(IMAGE_RECALL_KEY);
        } catch (error) { /* session metadata is optional */ }
    }
    function saveAttachments() {
        writeJson(ATTACH_KEY, attachments.filter(function (item) { return item.kind !== 'image'; }));
        rememberImageLifecycle();
    }
    function addAttachment(entry) {
        if (!entry || (entry.kind !== 'image' && !entry.content)) { toast('No file content to attach.', 'info'); return; }
        var name = String(entry.name || 'attachment.txt');
        var content = entry.content == null ? '' : String(entry.content);
        var size = Number.isFinite(entry.size) ? entry.size : new TextEncoder().encode(content).byteLength;
        var next = attachments.filter(function (item) { return item.name !== name; });
        if (entry.kind === 'image') {
            var imageCount = next.filter(function (item) { return item.kind === 'image'; }).length;
            var imageTotal = next.filter(function (item) { return item.kind === 'image'; }).reduce(function (sum, item) { return sum + (Number(item.size) || 0); }, size);
            if (imageCount >= MAX_IMAGES || imageTotal > MAX_IMAGE_TOTAL_BYTES) { toast('Images exceed the 4-file or 16 MiB total image limit.', 'error'); return; }
            recalledImages = 0;
        } else {
            var total = next.filter(function (item) { return item.kind !== 'image'; }).reduce(function (sum, item) { return sum + (Number.isFinite(item.size) ? item.size : new TextEncoder().encode(String(item.content || '')).byteLength); }, size);
            if (total > MAX_ATTACHMENT_TOTAL_BYTES) { toast('Attachments exceed the 4 MiB total limit; remove a file first.', 'error'); return; }
        }
        attachments = next;
        attachments.push(Object.assign({}, entry, { name: name, kind: entry.kind || 'text', content: content, source: entry.source || 'file', size: size }));
        saveAttachments(); renderAttachments();
        toast('Attached ' + name + ' (' + Math.max(1, Math.round(size / 1024)) + ' KB).', 'success');
    }
    function renderAttachments() {
        var chips = document.getElementById('attachmentChips');
        var bar = document.getElementById('attachmentBar');
        var note = document.getElementById('composerContextNote');
        var inspector = document.getElementById('inspectorAttachments');
        var badge = document.getElementById('attachmentCountBadge');
        var tokenHint = document.getElementById('attachmentTokenHint');
        if (chips) chips.textContent = '';
        if (inspector) inspector.textContent = '';
        if (bar) bar.hidden = attachments.length === 0 && recalledImages === 0;
        var images = attachments.filter(function (item) { return item.kind === 'image'; });
        var totalChars = attachments.reduce(function (sum, item) { return sum + String(item.content || '').length; }, 0);
        var approxTokens = Math.max(0, Math.round(totalChars / 4));
        if (badge) badge.textContent = attachments.length + ' file' + (attachments.length === 1 ? '' : 's');
        if (tokenHint) tokenHint.textContent = attachments.length
            ? (images.length ? images.length + ' image' + (images.length === 1 ? '' : 's') + ' memory-only · R4 context added on Send · ~' + approxTokens + ' text tokens' : '~' + approxTokens + ' tokens · prepended to API prompts only')
            : 'Attachments are prepended to API prompts only.';
        if (inspector && !attachments.length) { var hint = document.createElement('li'); hint.className = 'hint'; hint.textContent = 'No context attached. Use Attach context in the composer.'; inspector.appendChild(hint); }
        var mode = hook('getMode') ? hook('getMode')() : 'wasm';
        var native = textOf('#chatSourceBadge') === 'Native uor-chat';
        if (note) {
            note.hidden = attachments.length === 0 && recalledImages === 0;
            note.textContent = recalledImages
                ? 'Image attachments are memory-only and were not restored. Reattach them.'
                : (attachments.length ? attachments.length + ' context file' + (attachments.length === 1 ? '' : 's') + (images.length ? ' · R4 patch input — experimental; not visual understanding' : '') + ' · ~' + approxTokens + ' text tokens · ' + (native ? 'prepended to the next native prompt (stays on this device).' : (mode === 'api' ? 'prepended to the next API prompt (leaves device).' : 'local WASM mode ignores context; switch to an API provider to use it.')) : '');
        }
        attachments.forEach(function (item) {
            var chip = document.createElement('span');
            chip.className = 'attachment-chip';
            chip.title = item.name;
            if (item.kind === 'image' && item.dataUrl) {
                var preview = document.createElement('img');
                preview.className = 'attachment-chip-preview';
                preview.src = item.dataUrl;
                preview.alt = '';
                chip.appendChild(preview);
            }
            var name = document.createElement('span'); name.className = 'attachment-chip-name'; name.textContent = item.name;
            var size = document.createElement('span'); size.className = 'attachment-chip-meta'; size.textContent = (item.kind === 'image' ? item.width + '×' + item.height + ' · ' : '') + Math.max(1, Math.round((Number(item.size) || String(item.content || '').length) / 1024)) + ' KB';
            var remove = document.createElement('button'); remove.type = 'button'; remove.className = 'attachment-chip-remove'; remove.setAttribute('aria-label', 'Remove ' + item.name); remove.textContent = '×';
            remove.addEventListener('click', function () { attachments = attachments.filter(function (candidate) { return candidate.name !== item.name; }); saveAttachments(); renderAttachments(); });
            chip.appendChild(name); chip.appendChild(size); chip.appendChild(remove);
            if (chips) chips.appendChild(chip);
            if (inspector) { var row = document.createElement('li'); row.className = 'attachment-row'; row.textContent = item.name + (item.kind === 'image' ? ' · R4 patch input — experimental; not visual understanding' : ''); inspector.appendChild(row); }
        });
        document.dispatchEvent(new CustomEvent('uor-attachments-changed', { detail: { count: attachments.length } }));
    }
    function removeAttachment(name) { attachments = attachments.filter(function (item) { return item.name !== name; }); saveAttachments(); renderAttachments(); }
    function clearAttachments() { attachments = []; recalledImages = 0; saveAttachments(); renderAttachments(); }
    function attachContext() {
        var files = window.__studioFiles;
        if (files && files.activeFile) { var active = files.activeFile(); if (active && active.content) { addAttachment({ name: active.path, content: active.content, source: 'editor' }); return; } }
        document.dispatchEvent(new CustomEvent('uor-attach-active-file'));
        window.setTimeout(function () { if (!attachments.length) toast('Open a file in the editor (or Files) first, then attach it as context.', 'info'); }, 20);
    }
    function openTemplates(anchor) {
        var menu = document.getElementById('templateMenu');
        if (!menu) return;
        if (!menu.hidden) { closeTemplateMenu(); return; }
        closeOtherModals(null, false);
        menu.hidden = false;
        if (anchor) anchor.setAttribute('aria-expanded', 'true');
        var first = menu.querySelector('button');
        if (first) first.focus();
        var close = function (event) { if (!menu.contains(event.target) && event.target !== anchor) { closeTemplateMenu(false); document.removeEventListener('click', close); } };
        window.setTimeout(function () { document.addEventListener('click', close); }, 0);
    }
    function activeFileLabel() { var files = window.__studioFiles; var active = files && files.activeFile ? files.activeFile() : null; return active ? active.path : 'no file open'; }
    function templatePrompt(kind) {
        var base = 'Studio workspace context: ' + activeFileLabel() + '.\n\nScope: web/studio/** only. Do not modify the engine, WASM worker, model-source manager, API client, crates or verifiers. Browser-only, no external assets or URLs. Keep all existing ids, handlers, status markers and the boot contract intact; the four Studio verifiers must pass unchanged.\n\n';
        var map = {
            ux: base + 'TASK — Studio UX improvement brief.\n\nProblem: <describe the UX problem and where it appears>.\nDesired outcome: <what a user should be able to do>.\nConstraints: four labeled destinations (Agent/Workspace/Connections/Guide), one control per action, AA contrast, no horizontal overflow at 390px, no duplicate controls.\n\nSteps: read the relevant web/studio files, propose the smallest coherent change, implement it, then run node web/studio/verify_studio_page.mjs, verify_studio_page_load.mjs, verify_studio_worker.mjs and verify_desktop_client.mjs and report exact results.',
            api: base + 'TASK — API integration checklist (Claude owns API code; do not edit it here).\n\nIntegration: <endpoint + model + auth expectation>.\nDeliver: request/response contract sketch, error/failure modes, privacy line shown to users, test plan, and the exact Studio fields/events needed. Keep prompts local unless the user explicitly selects an API provider.',
            audit: base + 'TASK — Audit this file.\n\nReview for: broken contract ids, verifier weakening, duplicate controls, false keyboard hints, AA contrast failures, mobile overflow, external requests at load, and scope creep into engine/API files. Report findings with file:line and a concrete fix for each.',
            report: base + 'TASK — Write a milestone REPORT with negatives first.\n\nInclude: what was attempted and rejected, exact commands and outputs, contract preservation evidence, remaining limits, and links to screenshots. Do not claim capability beyond pre-alpha / M1 next-token prediction.',
            error: base + 'TASK — Explain this error.\n\nCommand/output: <paste exact command and error>.\nDeliver: the first true failure, the cause, the smallest fix, and the command that proves it is fixed.',
            test: base + 'TASK — Write a verification plan.\n\nChange: <describe>.\nDeliver: exact commands, expected output, negative cases and the screenshot/interaction states a human should inspect.'
        };
        return map[kind] || map.ux;
    }
    function useTemplate(kind) {
        var setText = hook('setComposerText');
        if (setText) setText(templatePrompt(kind));
        closeTemplateMenu();
        toast('Inserted a build-task template. Fill in the <placeholders>.', 'success');
    }

    /* message actions */
    function messageText(node) { var body = node.querySelector('.msg-body'); return body ? body.textContent : ''; }
    function enhanceMessages() {
        $all('#chatMessages .msg').forEach(function (node) {
            if (node.dataset.actionsAdded === '1') return;
            node.dataset.actionsAdded = '1';
            var actions = document.createElement('div');
            actions.className = 'msg-actions';
            var buttons = [
                ['Copy', function () { navigator.clipboard.writeText(messageText(node)).then(function () { toast('Message copied.', 'success'); }, function () { toast('Clipboard unavailable.', 'error'); }); }],
                ['Insert into editor', function () { if (window.__studioFiles && window.__studioFiles.insertText) window.__studioFiles.insertText(messageText(node), 'agent-message-' + Date.now() + '.md'); else toast('Editor module not ready.', 'error'); }],
                ['Save as file', function () {
                    var blob = new Blob([messageText(node)], { type: 'text/markdown;charset=utf-8' });
                    var url = URL.createObjectURL(blob);
                    var link = document.createElement('a'); link.href = url; link.download = 'studio-message-' + Date.now() + '.md';
                    document.body.appendChild(link); link.click(); link.remove();
                    window.setTimeout(function () { URL.revokeObjectURL(url); }, 800);
                }]
            ];
            if (node.getAttribute('data-role') === 'assistant') {
                buttons.push(['Retry', function () {
                    var messages = hook('getConversationMessages') ? hook('getConversationMessages')() : [];
                    var user = null;
                    for (var i = messages.length - 1; i >= 0; i -= 1) { if (messages[i].role === 'user') { user = messages[i]; break; } }
                    if (user && hook('setComposerText')) { hook('setComposerText')(user.content); var send = document.getElementById('btnSend'); if (send && !send.disabled) send.click(); }
                    else toast('No user prompt to retry.', 'info');
                }]);
            }
            buttons.forEach(function (pair) {
                var button = document.createElement('button');
                button.type = 'button'; button.className = 'msg-action'; button.textContent = pair[0];
                button.addEventListener('click', function (event) { event.stopPropagation(); pair[1](); });
                actions.appendChild(button);
            });
            node.appendChild(actions);
        });
    }
    function watchMessages() {
        var list = document.getElementById('chatMessages');
        if (!list) return;
        var scheduled = false;
        new MutationObserver(function () { if (scheduled) return; scheduled = true; window.setTimeout(function () { scheduled = false; enhanceMessages(); }, 40); }).observe(list, { childList: true, subtree: true });
        enhanceMessages();
    }

    /* palette */
    var paletteOpen = false;
    var paletteIndex = 0;
    var paletteCommands = [];
    function cmd(group, label, description, shortcut, run) { return { group: group, label: label, description: description, shortcut: shortcut || '', run: run }; }
    function commands() {
        var list = [
            cmd('NAVIGATION', 'Go to Agent', 'Chat, context and build tasks.', '⌘1', function () { activateSection('agent'); }),
            cmd('NAVIGATION', 'Go to Workspace', 'File tree, editor tabs, save.', '⌘2', function () { activateSection('workspace'); }),
            cmd('NAVIGATION', 'Go to Connections', 'UOR Chat, custom providers, artifact diagnostics.', '⌘3', function () { activateSection('models'); }),
            cmd('NAVIGATION', 'Go to Guide', 'Quick start, workflows, shortcuts, limits.', '⌘4', function () { activateSection('guide'); }),

            cmd('SESSIONS', 'New session', 'Start a clean agent thread.', '', newSessionAction),
            cmd('SESSIONS', 'Rename current session…', 'Give the active thread a clear title.', '', function () { var item = document.querySelector('#sessionRowsV2 .session-row.is-active'); if (item) { var menu = item.querySelector('.conv-menu'); if (menu) menu.click(); } else toast('No active session.', 'info'); }),
            cmd('SESSIONS', 'Duplicate current session', 'Copy the active thread and its messages.', '', function () { var id = activeSessionId(); if (id) duplicateSession(id); else toast('No active session.', 'info'); }),
            cmd('SESSIONS', 'Export current session as JSON…', 'Download the active thread.', '', function () { var id = activeSessionId(); if (id) exportSession(id, 'session'); else toast('No active session.', 'info'); }),
            cmd('SESSIONS', 'Delete current session…', 'Remove the active thread after confirmation.', '', function () { var id = activeSessionId(); if (id) confirmDeleteSession(id); else toast('No active session.', 'info'); }),

            cmd('WORKSPACE', 'Open folder…', 'Grant this tab read/write access to a local folder.', '', function () { var files = window.__studioFiles; if (files) files.openFolder(); }),
            cmd('WORKSPACE', 'Reload file tree', 'Re-read the opened folder.', '', function () { var button = document.getElementById('btnReloadTree'); if (button) button.click(); }),
            cmd('WORKSPACE', 'New file…', 'Create an empty editor tab.', '', function () { var files = window.__studioFiles; if (files) files.newFile(); }),
            cmd('WORKSPACE', 'Save file', 'Write the active tab back to disk (or download).', '⌘S', function () { var files = window.__studioFiles; if (files) files.saveActive(); }),
            cmd('WORKSPACE', 'Save as…', 'Download a copy of the active file.', '⇧⌘S', function () { var files = window.__studioFiles; if (files) files.saveAsCopy(); }),
            cmd('WORKSPACE', 'Close tab', 'Close the active editor tab.', '', function () { var files = window.__studioFiles; if (files) { var active = files.activeFile(); if (active) files.closeTab(active.path); } }),

            cmd('AGENT', 'Attach current file', 'Add the active editor file to context.', '', attachContext),
            cmd('AGENT', 'Attach selection', 'Attach the active file text as context.', '', attachContext),
            cmd('AGENT', 'Clear attachments', 'Remove all context files.', '', clearAttachments),
            cmd('AGENT', 'Focus composer', 'Jump to the chat input.', '', function () { activateSection('agent'); var fn = hook('focusComposer'); if (fn) fn(); }),

            cmd('MODELS', 'Load synthetic demo', 'Exercise the local WASM serving path.', '', function () { activateSection('models'); var button = document.querySelector('[data-action="load-bundled"]'); if (button) button.click(); }),
            cmd('MODELS', 'Load M1 model (seed 1)', 'Top-8 next-token prediction from the 19-table trunk.', '', function () { activateSection('models'); var button = document.querySelector('[data-action="load-published"]'); if (button) button.click(); }),
            cmd('MODELS', 'Load local model file…', 'Choose a .lut / .bin stack artifact from this device.', '', function () { activateSection('models'); var button = document.querySelector('[data-role="model-file"], #modelFileInput'); if (button) button.click(); }),
            cmd('MODELS', 'Load model from URL…', 'User-initiated download of a stack artifact.', '', function () { promptUrl(); }),
            cmd('MODELS', 'Add API provider…', 'Configure an OpenAI-compatible endpoint.', '', function () { activateSection('models'); var button = document.querySelector('[data-action="add-api"]'); if (button) button.click(); }),

            cmd('BUILD', 'Studio UX improvement brief', 'Insert the UX brief template.', '', function () { activateSection('agent'); useTemplate('ux'); }),
            cmd('BUILD', 'API integration checklist', 'Insert the API checklist template.', '', function () { activateSection('agent'); useTemplate('api'); }),
            cmd('BUILD', 'Audit this file', 'Insert an adversarial audit template.', '', function () { activateSection('agent'); useTemplate('audit'); }),
            cmd('BUILD', 'Write a milestone REPORT', 'Insert a REPORT template.', '', function () { activateSection('agent'); useTemplate('report'); }),
            cmd('BUILD', 'Explain this error', 'Insert a diagnosis template.', '', function () { activateSection('agent'); useTemplate('error'); }),

            cmd('INTERFACE', 'Use light theme', 'Warm paper theme.', '', function () { setThemeChoice('light'); }),
            cmd('INTERFACE', 'Use dark theme', 'Warm graphite theme.', '', function () { setThemeChoice('dark'); }),
            cmd('INTERFACE', 'Use system theme', 'Follow the OS theme.', '', function () { setThemeChoice('system'); }),
            cmd('INTERFACE', 'Toggle sidebar', 'Collapse or expand the navigation.', '⌘B', function () { toggleChrome('nav'); }),
            cmd('INTERFACE', 'Toggle context panel', 'Show or hide the context column.', '⌘I', function () { toggleChrome('inspector'); }),
            cmd('INTERFACE', 'Open settings', 'Theme, editor, data, about.', '⌘,', openSettings),
            cmd('INTERFACE', 'Keyboard shortcuts', 'Keyboard reference.', '?', openHelp),
            cmd('INTERFACE', 'Open guide', 'Read the Studio guide.', '', function () { activateSection('guide'); }),

            cmd('HELP', 'About UOR-R4 Studio', 'Version, scope and honest status.', '', function () { openSettings(); window.setTimeout(function () { updateSettingsInfo(); }, 40); }),
            cmd('HELP', 'Offline status', 'Show the offline-first statement.', '', function () { toast('Offline-first: the page makes no external requests at load.', 'info'); })
        ];
        // searchable sessions
        sessionList().slice(0, 8).forEach(function (session) {
            list.push(cmd('SESSIONS', 'Session: ' + session.title, session.preview || (session.messageCount + ' messages'), '', function () { var fn = hook('selectSession'); if (fn) fn(session.id); activateSection('agent'); }));
        });
        // searchable open editor tabs
        var files = window.__studioFiles;
        if (files && files.tabs) {
            files.tabs().slice(0, 8).forEach(function (tab) {
                list.push(cmd('WORKSPACE', 'File: ' + tab.path, tab.dirty ? 'Open tab · unsaved' : 'Open tab', '', function () { if (files.openFile) files.openFile(tab.path); }));
            });
        }
        var ORDER = ['NAVIGATION', 'SESSIONS', 'WORKSPACE', 'AGENT', 'MODELS', 'BUILD', 'INTERFACE', 'HELP'];
        list.sort(function (a, b) { return ORDER.indexOf(a.group) - ORDER.indexOf(b.group); });
        return list;
    }

    function openPalette() {
        var backdrop = document.getElementById('commandPalette');
        if (!backdrop || paletteOpen) return;
        var returnFocus = overlayReturnFocus();
        closeOtherModals(null, false);
        paletteOpen = true; lastFocus = returnFocus; backdrop.hidden = false;
        var input = document.getElementById('paletteInput');
        if (input) { input.value = ''; input.focus(); }
        paletteIndex = 0; buildPalette('');
    }
    function closePalette(restoreFocus) {
        var backdrop = document.getElementById('commandPalette');
        if (!backdrop || !paletteOpen) return;
        paletteOpen = false; backdrop.hidden = true;
        if (restoreFocus !== false && lastFocus && typeof lastFocus.focus === 'function') lastFocus.focus();
        lastFocus = null;
    }
    function filtered(query) {
        var base = commands();
        var q = String(query || '').trim().toLowerCase();
        if (!q) return base;
        return base.filter(function (command) { return (command.label + ' ' + command.description + ' ' + command.group).toLowerCase().indexOf(q) >= 0; });
    }
    function buildPalette(query) {
        var list = document.getElementById('paletteList');
        if (!list) return;
        paletteCommands = filtered(query);
        if (paletteIndex >= paletteCommands.length) paletteIndex = Math.max(0, paletteCommands.length - 1);
        list.textContent = '';
        if (!paletteCommands.length) { var empty = document.createElement('div'); empty.className = 'palette-empty'; empty.textContent = 'No matching commands.'; list.appendChild(empty); return; }
        var currentGroup = null;
        paletteCommands.forEach(function (command, index) {
            if (command.group !== currentGroup) {
                currentGroup = command.group;
                var groupLabel = document.createElement('div');
                groupLabel.className = 'palette-group-label';
                groupLabel.textContent = currentGroup;
                list.appendChild(groupLabel);
            }
            var item = document.createElement('button');
            item.type = 'button'; item.className = 'palette-item'; item.id = 'palette-item-' + index;
            item.setAttribute('role', 'option'); item.setAttribute('aria-selected', index === paletteIndex ? 'true' : 'false');
            var bodyText = document.createElement('span'); bodyText.className = 'palette-item-text';
            var label = document.createElement('span'); label.className = 'palette-item-label'; label.textContent = command.label;
            var desc = document.createElement('span'); desc.className = 'palette-item-desc'; desc.textContent = command.description;
            bodyText.appendChild(label); bodyText.appendChild(desc);
            var shortcut = document.createElement('span'); shortcut.className = 'palette-item-hint'; shortcut.textContent = command.shortcut || '';
            item.appendChild(bodyText); item.appendChild(shortcut);
            item.addEventListener('mouseenter', function () { paletteIndex = index; syncSelection(); });
            item.addEventListener('click', function () { runPalette(index); });
            list.appendChild(item);
        });
        var active = document.getElementById('palette-item-' + paletteIndex);
        if (active) active.scrollIntoView({ block: 'nearest' });
        var input = document.getElementById('paletteInput');
        if (input) input.setAttribute('aria-activedescendant', 'palette-item-' + paletteIndex);
    }
    function syncSelection() {
        $all('.palette-item').forEach(function (item, index) { item.setAttribute('aria-selected', index === paletteIndex ? 'true' : 'false'); });
        var input = document.getElementById('paletteInput');
        if (input) input.setAttribute('aria-activedescendant', 'palette-item-' + paletteIndex);
        var selected = document.getElementById('palette-item-' + paletteIndex);
        if (selected) selected.scrollIntoView({ block: 'nearest' });
    }
    function moveSelection(delta) { if (!paletteCommands.length) return; paletteIndex = (paletteIndex + delta + paletteCommands.length) % paletteCommands.length; syncSelection(); }
    function runPalette(index) { var command = paletteCommands[index]; if (!command) return; closePalette(); window.setTimeout(function () { try { command.run(); } catch (error) { toast(String(error && error.message ? error.message : error), 'error'); } }, 20); }

    /* dialogs */
    var modalReturnFocus = null;
    function modalFocusables(dialog) {
        return Array.prototype.slice.call(dialog.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')).filter(function (node) { return !node.disabled && node.getClientRects().length; });
    }
    function closeTemplateMenu(restoreFocus) {
        var menu = document.getElementById('templateMenu');
        var wasOpen = menu && !menu.hidden;
        if (menu) menu.hidden = true;
        var anchor = document.getElementById('btnBuildTask');
        if (anchor) {
            anchor.setAttribute('aria-expanded', 'false');
            if (restoreFocus !== false && wasOpen) anchor.focus();
        }
    }
    function closeOtherModals(except, restoreFocus) {
        closeSessionPopover(restoreFocus);
        closeTemplateMenu(restoreFocus);
        $all('.modal-backdrop:not([hidden])').forEach(function (other) { if (other !== except) closeModal(other, restoreFocus); });
    }
    function overlayReturnFocus() {
        if (document.querySelector('.modal-backdrop:not([hidden])')) return modalReturnFocus || document.activeElement;
        if (paletteOpen) return lastFocus || document.activeElement;
        if (sessionMenuTrigger) return sessionMenuTrigger;
        var menu = document.getElementById('templateMenu');
        if (menu && !menu.hidden) return document.getElementById('btnBuildTask') || document.activeElement;
        return document.activeElement;
    }
    function openModal(dialog) {
        if (!dialog || !dialog.hidden) return;
        var returnFocus = overlayReturnFocus();
        closeOtherModals(dialog, false);
        closePalette(false);
        modalReturnFocus = returnFocus;
        dialog.hidden = false;
        var first = modalFocusables(dialog)[0];
        if (first) first.focus();
    }
    function closeModal(dialog, restoreFocus) {
        if (!dialog) return;
        dialog.hidden = true;
        if (restoreFocus !== false && modalReturnFocus && modalReturnFocus.focus) modalReturnFocus.focus();
        modalReturnFocus = null;
        if (dialog.dataset.dynamic === 'true') dialog.remove();
    }
    function trapModalTab(event, dialog) {
        var nodes = modalFocusables(dialog);
        if (!nodes.length) return;
        var first = nodes[0];
        var last = nodes[nodes.length - 1];
        if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
        else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    }
    function trapNavTab(event) {
        var sidebar = document.getElementById('navSidebar');
        if (!sidebar) return;
        var nodes = modalFocusables(sidebar);
        if (!nodes.length) return;
        var first = nodes[0];
        var last = nodes[nodes.length - 1];
        var active = document.activeElement;
        if (event.shiftKey && (active === first || !sidebar.contains(active))) { event.preventDefault(); last.focus(); }
        else if (!event.shiftKey && (active === last || !sidebar.contains(active))) { event.preventDefault(); first.focus(); }
    }
    function openSettings() { var dialog = document.getElementById('settingsDialog'); if (!dialog) return; openModal(dialog); updateSettingsInfo(); }
    function closeSettings() { closeModal(document.getElementById('settingsDialog')); }
    function openHelp() { openModal(document.getElementById('helpDialog')); }
    function closeHelp() { closeModal(document.getElementById('helpDialog')); }
    function updateSettingsInfo() {
        var label = document.getElementById('settingsWorkspaceLabel');
        var files = window.__studioFiles;
        if (label) {
            var name = files && files.workspaceName ? files.workspaceName() : '';
            var mode = files && files.mode ? files.mode() : 'none';
            label.textContent = name ? name + ' · ' + (mode === 'directory' ? 'read/write via File System Access API' : (mode === 'demo' ? 'demo tree (QA state)' : 'imported copies (read-only)')) : 'No folder open. File System Access API support is detected at open time.';
        }
        var storage = document.getElementById('settingsStorage');
        if (storage) {
            var chars = 0;
            try { for (var i = 0; i < window.localStorage.length; i += 1) { var key = window.localStorage.key(i); chars += key.length + String(window.localStorage.getItem(key) || '').length; } } catch (error) { chars = 0; }
            storage.textContent = (chars / 1024).toFixed(1) + ' KB across ' + window.localStorage.length + ' keys.';
        }
    }
    function clearSessions() {
        var list = hook('listSessions');
        var deleteFn = hook('deleteSession');
        if (!list || !deleteFn) { toast('Session store not ready.', 'error'); return { cleared: 0, total: 0, error: 'Session store not ready.' }; }
        var sessions = list();
        var cleared = 0;
        sessions.forEach(function (session) { if (deleteFn(session.id, false)) cleared += 1; });
        if (cleared === sessions.length) toast('Cleared ' + cleared + ' session(s).', 'success');
        else toast('Cleared ' + cleared + ' of ' + sessions.length + ' session(s); stop the current reply before deleting the rest.', 'warn');
        window.setTimeout(renderSessionEnhancements, 60);
        return { cleared: cleared, total: sessions.length };
    }
    function clearEditor() { var files = window.__studioFiles; if (files && files.closeAll) files.closeAll(); else { var text = document.getElementById('editorText'); if (text) text.value = ''; } toast('Editor tabs cleared.', 'success'); }

    /* status */
    function textOf(selector) { var node = $(selector); return node ? String(node.textContent || '').replace(/\s+/g, ' ').trim() : ''; }
    function setText(selector, value) { var node = $(selector); if (node && node.textContent !== value) node.textContent = value; }
    function updateStatus() {
        var engine = textOf('#engineStatus').replace(/^Engine:\s*/i, '') || 'starting';
        var model = textOf('#modelStatus').replace(/^Model:\s*/i, '') || 'none';
        var tokenizer = textOf('#footerTokenizer').replace(/^Tokenizer:\s*/i, '');
        var privacy = textOf('#privacyStatus');
        var storage = textOf('#footerStorage').replace(/^Storage:\s*/i, '');
        var mode = hook('getMode') ? hook('getMode')() : 'wasm';
        var api = mode === 'api';
        var native = privacy === 'Native loopback' || textOf('#chatSourceBadge') === 'Native uor-chat';
        var attachmentCount = hook('getAttachments') ? hook('getAttachments')().length : 0;
        setText('#engineStatusText', native ? 'native uor-chat' : 'local WASM · ' + engine);
        var modelShort = native ? 'native uor-chat' : (/no model/i.test(model) ? 'none' : model.split(' · ')[0]);
        if (modelShort.length > 42) modelShort = modelShort.slice(0, 41) + '…';
        setText('#modelStatusText', modelShort);
        setText('#privacyStatusText', native ? 'Native loopback' : (api ? 'API (leaves device)' : 'Local only'));
        setText('#footerStorage', String(storage || 'unknown').replace('MB used of ', ' MB / ').replace(/\s+/g, ' '));
        setText('#statusWorkspaceLabel', (window.__studioFiles && window.__studioFiles.workspaceName && window.__studioFiles.workspaceName()) ? window.__studioFiles.workspaceName() : 'uor-r4');
        setText('#statusContext', attachmentCount + ' file' + (attachmentCount === 1 ? '' : 's'));
        setText('#composerModeNote', native ? 'Native uor-chat — prompts stay on this device' : (api ? 'API — prompts leave this device' : 'Local WASM — prompts stay on this device'));
        var modeNote = document.getElementById('composerModeNote');
        if (modeNote) modeNote.setAttribute('data-mode', native ? 'native' : (api ? 'api' : 'local'));
        setText('#envEngine', native ? 'native uor-chat' : engine);
        setText('#envModel', native ? 'native uor-chat' : (/no model/i.test(model) ? 'none' : model));
        setText('#envTokenizer', native ? 'native artifact tokenizer' : (tokenizer || 'awaiting model'));
        setText('#ctxEngine', native ? 'native uor-chat' : engine);
        setText('#ctxTokenizer', native ? 'native artifact tokenizer' : (tokenizer || 'awaiting model'));
        setText('#ctxModelLabel', native ? 'Native uor-chat' : 'No model loaded');
        setText('#ctxModelMeta', native ? 'Native loopback chat artifact · prompts stay on this device' : 'Load the synthetic demo or the M1 trunk in Connections.');
        setText('#ctxPrivacy', native ? 'native loopback' : (api ? 'api (leaves device)' : 'local only'));
        setText('#ctxStorage', storage || 'unknown');
        var sessions = sessionList();
        var active = sessions.find(function (session) { return session.id === activeSessionId(); });
        setText('#statusSession', sessions.length ? (active ? 'session: ' + active.title.slice(0, 34) : 'session: ' + sessions.length) : 'session: new');
        var files = window.__studioFiles;
        var folder = files && files.workspaceName ? files.workspaceName() : '';
        setText('#statusFolder', folder || 'no folder');
        var ctxLabel = document.getElementById('ctxWorkspaceLabel');
        if (ctxLabel) ctxLabel.textContent = folder || 'No folder open';
        var ctxMeta = document.getElementById('ctxWorkspaceMeta');
        if (ctxMeta) ctxMeta.textContent = folder ? (files.mode() === 'directory' ? 'Read/write access to this folder.' : 'Demo/imported files for this session.') : 'Open a folder in Workspace.';
        var legacySaved = document.getElementById('editorSavedBadge');
        if (legacySaved) {
            var saveState = document.getElementById('editorSaveState');
            legacySaved.textContent = saveState && saveState.textContent ? saveState.textContent : (document.getElementById('editorUnsaved') ? document.getElementById('editorUnsaved').textContent : 'Saved locally');
        }
        var sessionsCount = document.getElementById('settingsSessionCount');
        if (sessionsCount) sessionsCount.textContent = sessions.length + ' session' + (sessions.length === 1 ? '' : 's') + ' · ' + sessions.reduce(function (sum, session) { return sum + (session.messageCount || 0); }, 0) + ' messages';
        var draftCount = document.getElementById('settingsDraftCount');
        if (draftCount && files && files.tabs) { var tabs = files.tabs(); draftCount.textContent = tabs.length ? tabs.length + ' open tab(s) · ' + tabs.filter(function (tab) { return tab.dirty; }).length + ' unsaved' : 'No open tabs.'; }
    }
    function clock() {
        var node = document.getElementById('statusClock');
        if (!node) return;
        var now = new Date();
        function pad(value) { return value < 10 ? '0' + value : String(value); }
        node.textContent = pad(now.getHours()) + ':' + pad(now.getMinutes()) + ':' + pad(now.getSeconds());
    }

    function on(id, event, handler) { var node = document.getElementById(id); if (node) node.addEventListener(event, handler); }

    function wire() {
        ['navAgent', 'navWorkspace', 'navModels', 'navGuide'].forEach(function (id) {
            var name = id.replace('nav', '').toLowerCase();
            on(id, 'click', function () { activateSection(name); });
        });
        on('btnToggleNav', 'click', function () { toggleChrome('nav'); });
        on('btnNavCollapse', 'click', function () { toggleChrome('nav'); });
        on('btnMobileNav', 'click', function () { toggleChrome('nav'); });
        on('btnNavScrim', 'click', function () { setChrome('nav', 'collapsed', true); });
        ['btnToggleInspector', 'btnToggleInspector2', 'btnToggleInspector3', 'btnToggleInspector4', 'btnInspectorCollapse'].forEach(function (id) { on(id, 'click', function () { toggleChrome('inspector'); }); });
        on('btnOpenModelSources', 'click', function () { activateSection('models'); });
        on('ctxOpenFiles', 'click', function () { activateSection('files'); });
        on('btnGuideAgent', 'click', function () { activateSection('agent'); });
        $all('[data-guide-nav]').forEach(function (button) { button.addEventListener('click', function () { activateSection(button.getAttribute('data-guide-nav')); }); });
        ['btnTheme', 'railTheme'].forEach(function (id) { on(id, 'click', toggleTheme); });
        on('btnSettings', 'click', openSettings);
        on('btnCloseSettings', 'click', closeSettings);
        on('btnHelp', 'click', openHelp);
        on('btnHelpNav', 'click', openHelp);
        on('btnCloseHelp', 'click', closeHelp);
        on('btnHelpGuide', 'click', function () { closeHelp(); activateSection('guide'); });
        on('btnNewChat', 'click', function () { var fn = hook('newSession'); if (fn) fn(); activateSection('agent'); renderSessionEnhancements(); });
        on('btnNewChatTop', 'click', function () { var fn = hook('newSession'); if (fn) fn(); activateSection('agent'); renderSessionEnhancements(); });
        var search = document.getElementById('sessionSearch');
        if (search) search.addEventListener('input', function () {
            var query = search.value.trim().toLowerCase();
            $all('#conversationList .conversation-item').forEach(function (item) { item.style.display = !query || item.textContent.toLowerCase().indexOf(query) >= 0 ? '' : 'none'; });
        });
        on('commandSearch', 'click', openPalette);
        on('btnCommandPaletteNav', 'click', openPalette);
        var palette = document.getElementById('commandPalette');
        if (palette) palette.addEventListener('click', function (event) { if (event.target === palette) closePalette(); });
        var paletteInput = document.getElementById('paletteInput');
        if (paletteInput) {
            paletteInput.addEventListener('input', function () { paletteIndex = 0; buildPalette(paletteInput.value); });
            paletteInput.addEventListener('keydown', function (event) {
                if (event.key === 'ArrowDown') { event.preventDefault(); moveSelection(1); }
                else if (event.key === 'ArrowUp') { event.preventDefault(); moveSelection(-1); }
                else if (event.key === 'Enter') { event.preventDefault(); runPalette(paletteIndex); }
                else if (event.key === 'Escape') { event.preventDefault(); closePalette(); }
            });
        }
        on('topWorkspaceSelector', 'click', function () { var files = window.__studioFiles; if (files) files.openFolder(); });
        on('btnToggleInspectorTop', 'click', function () { toggleChrome('inspector'); });
        on('btnDoneSettings', 'click', closeSettings);
        on('btnAboutGuide', 'click', function () { closeSettings(); activateSection('guide'); });
        on('btnExportSessions', 'click', function () {
            var sessions = sessionList();
            var payload = sessions.map(function (session) { var fn = hook('exportSession'); return fn ? JSON.parse(fn(session.id)) : null; }).filter(Boolean);
            var blob = new Blob([JSON.stringify({ schema: 'uor-r4.studio-sessions/1', exportedAt: new Date().toISOString(), sessions: payload }, null, 2)], { type: 'application/json;charset=utf-8' });
            var url = URL.createObjectURL(blob);
            var link = document.createElement('a'); link.href = url; link.download = 'uor-sessions-export.json';
            document.body.appendChild(link); link.click(); link.remove();
            window.setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
            toast('Exported ' + sessions.length + ' session(s).', 'success');
        });
        // legacy id compatibility wiring (elements are display:none but must work when queried)
        on('railChat', 'click', function () { activateSection('agent'); });
        on('railModel', 'click', function () { activateSection('models'); });
        on('railEditor', 'click', function () { activateSection('workspace'); });
        on('railPalette', 'click', openPalette);
        on('railTheme', 'click', toggleTheme);
        on('railHelp', 'click', openHelp);
        on('railSessionsBottom', 'click', newSessionAction);
        on('btnToggleSidebar', 'click', function () { toggleChrome('nav'); });
        on('btnSidebarCollapse', 'click', function () { toggleChrome('nav'); });
        on('btnMobileMenu', 'click', function () { toggleChrome('nav'); });
        on('btnNewChat', 'click', newSessionAction);
        on('btnAttachContext', 'click', attachContext);
        on('btnClearAttachments', 'click', clearAttachments);
        on('btnBuildTask', 'click', function () { openTemplates(document.getElementById('btnBuildTask')); });
        $all('.template-item').forEach(function (button) { button.addEventListener('click', function () { useTemplate(button.getAttribute('data-template')); }); });
        document.addEventListener('uor-attach-file', function (event) { if (event.detail) addAttachment(event.detail); });
        $all('[data-setting-theme]').forEach(function (button) { button.addEventListener('click', function () { setThemeChoice(button.getAttribute('data-setting-theme')); }); });
        on('settingReduceMotion', 'change', function (event) { settings.reduceMotion = event.target.checked; saveSettings(); applySettings(); });
        on('settingFontSize', 'change', function (event) { settings.fontSize = Number(event.target.value) || 13; saveSettings(); applySettings(); });
        on('settingTabSize', 'change', function (event) { settings.tabSize = Number(event.target.value) || 4; saveSettings(); applySettings(); });
        on('settingWordWrap', 'change', function (event) { settings.wordWrap = event.target.checked; saveSettings(); applySettings(); });
        on('btnOpenFolderSettings', 'click', function () { var files = window.__studioFiles; if (files) files.openFolder(); });
        on('btnClearSessions', 'click', clearSessions);
        on('btnClearEditor', 'click', clearEditor);
        document.addEventListener('uor-view-changed', function (event) { syncSectionNavigation(sectionForView(event.detail && event.detail.view)); });
        document.addEventListener('uor-files-changed', function () { updateStatus(); updateSettingsInfo(); });
        document.addEventListener('uor-workspace-changed', function () { updateStatus(); updateSettingsInfo(); });
        document.addEventListener('uor-model-source-changed', function () { window.setTimeout(updateStatus, 80); });
        document.addEventListener('keydown', function (event) {
            var meta = event.metaKey || event.ctrlKey;
            var target = event.target;
            var typing = target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT' || target.isContentEditable);
            var templateMenuState = document.getElementById('templateMenu');
            var overlayActive = paletteOpen || sessionPopover || (templateMenuState && !templateMenuState.hidden) || document.querySelector('.modal-backdrop:not([hidden])');
            if (overlayActive && meta && ['1', '2', '3', '4', 'b', 'B', 'i', 'I'].indexOf(event.key) >= 0) { event.preventDefault(); return; }
            if (meta && (event.key === 'k' || event.key === 'K')) { event.preventDefault(); if (paletteOpen) closePalette(); else openPalette(); return; }
            if (meta && event.key === ',') { event.preventDefault(); openSettings(); return; }
            if (meta && ['1', '2', '3', '4'].indexOf(event.key) >= 0 && !event.shiftKey && !event.altKey) { event.preventDefault(); activateSection({ '1': 'agent', '2': 'workspace', '3': 'models', '4': 'guide' }[event.key]); return; }
            if (meta && (event.key === 'b' || event.key === 'B')) { event.preventDefault(); toggleChrome('nav'); return; }
            if (meta && (event.key === 'i' || event.key === 'I')) { event.preventDefault(); toggleChrome('inspector'); return; }
            if (!typing && !event.metaKey && !event.ctrlKey && event.key === '?') { event.preventDefault(); openHelp(); return; }
            if (!typing && event.key === 'F2') {
                var focused = document.activeElement;
                var row = focused ? focused.closest('.session-row') : null;
                if (row) { event.preventDefault(); event.stopPropagation(); var id = row.getAttribute('data-id'); var title = (row.querySelector('.session-title') || {}).textContent || 'Untitled session'; var menu = row.querySelector('.conv-menu'); renameInline(row, id, title); if (menu) menu.setAttribute('aria-expanded', 'false'); }
                return;
            }
            var openModalDialog = document.querySelector('.modal-backdrop:not([hidden])');
            var openTemplateMenu = document.getElementById('templateMenu');
            if (event.key === 'Tab' && openModalDialog) { trapModalTab(event, openModalDialog); return; }
            if (event.key === 'Tab' && paletteOpen) { trapModalTab(event, document.getElementById('commandPalette')); return; }
            if (event.key === 'Tab' && sessionPopover) { trapModalTab(event, sessionPopover); return; }
            if (event.key === 'Tab' && openTemplateMenu && !openTemplateMenu.hidden) { trapModalTab(event, openTemplateMenu); return; }
            if (event.key === 'Tab' && app && app.getAttribute('data-nav') === 'expanded' && window.matchMedia('(max-width: 900px)').matches) { trapNavTab(event); return; }
            if (event.key === 'Escape' && sessionPopover) { event.preventDefault(); event.stopPropagation(); closeSessionPopover(); return; }
            if (event.key === 'Escape') {
                if (openTemplateMenu && !openTemplateMenu.hidden) { event.preventDefault(); event.stopPropagation(); closeTemplateMenu(); return; }
                if (openModalDialog) { event.preventDefault(); event.stopPropagation(); closeModal(openModalDialog); return; }
                if (paletteOpen) { event.preventDefault(); event.stopPropagation(); closePalette(); return; }
                if (app && app.getAttribute('data-nav') === 'expanded' && window.matchMedia('(max-width: 900px)').matches) {
                    event.preventDefault();
                    setChrome('nav', 'collapsed', true);
                    return;
                }
            }
        }, true);
        var navMedia = window.matchMedia('(max-width: 900px)');
        if (navMedia && navMedia.addEventListener) navMedia.addEventListener('change', syncNavPresentation);
        window.addEventListener('pagehide', function () { if (clockTimer) window.clearInterval(clockTimer); });
        window.addEventListener('focus', updateStatus);
    }

    function applyHash() {
        var hash = window.location.hash || '';
        if (hash === '#settings' || hash === '#settings-modal') { window.setTimeout(openSettings, 150); return; }
        if (hash === '#command-palette' || hash === '#palette') { window.setTimeout(openPalette, 150); return; }
        if (hash === '#demo=sessions-menu') { window.location.hash = '#demo=sessions'; window.setTimeout(function () { var menu = document.querySelector('#sessionRowsV2 .session-row .conv-menu'); if (menu) menu.click(); }, 900); return; }
        if (hash === '#demo=nav-open') { window.setTimeout(function () { activateSection('agent'); setChrome('nav', 'expanded', false); }, 160); return; }
        if (hash === '#palette-scrolled') { window.setTimeout(function () { openPalette(); var list = document.getElementById('paletteList'); if (list) list.scrollTop = list.scrollHeight; var input = document.getElementById('paletteInput'); if (input) input.value = ''; }, 220); return; }
        if (hash === '#demo=sessions' || hash === '#demo=sessions-menu') {
            window.setTimeout(function () {
                var seed = hook('seedSessions');
                if (seed) {
                    var now = Date.now();
                    seed([
                        { title: 'Studio UX audit', updatedAt: now - 2 * 60000, messages: [
                            { role: 'user', content: 'Audit the Studio v2 navigation and list the three worst UX problems.' },
                            { role: 'assistant', content: 'Findings: (1) the old segmented Chat/Model/Editor switch duplicated the nav; (2) status bar mixed legacy and new labels; (3) mobile pa item leaked a hidden footer. Fixes: single labeled nav, one coherent status bar, true mobile drawer.' }
                        ] },
                        { title: 'API provider wiring notes', updatedAt: now - 9 * 60000, messages: [
                            { role: 'user', content: 'Sketch how the OpenAI-compatible provider fields should appear on Connections.' },
                            { role: 'assistant', content: 'Remote providers section: Name, Base URL, Model, Key state + Use/Test/Edit/Remove; warning line: Local only until you select an API provider.' }
                        ] },
                        { title: 'M1 card copy review', updatedAt: now - 26 * 60000, messages: [
                            { role: 'user', content: 'Shorten the M1 seed-1 card copy; keep Format, Size and SHA-256 labeled.' },
                            { role: 'assistant', content: 'Proposed: M1 model (19-table trunk, seed 1) - PRIMARY. Format UORLUT01 · Size 82.5 MB · SHA-256 04652a3d... Trained geometric language base; next-token prediction only.' }
                        ] }
                    ]);
                }
                window.setTimeout(function () {
                    renderSessionRows();
                    if (hash === '#demo=sessions-menu') { var menu = document.querySelector('#sessionRowsV2 .session-row .conv-menu'); if (menu) menu.click(); }
                }, 160);
            }, 320);
            return;
        }
        if (hash === '#demo=attachments') {
            attachments = [
                { name: 'web/studio/index.html', content: '<!DOCTYPE html>\\n<html lang="en">\\n<!-- Studio page excerpt for the screenshot state -->\\n</html>\\n', source: 'demo' },
                { name: 'web/studio/index.css', content: '/* warm graphite + ember design tokens */\\n:root { --accent: #e8894a; }\\n', source: 'demo' }
            ];
            saveAttachments(); renderAttachments();
            window.setTimeout(function () { activateSection('agent'); }, 120);
            return;
        }
        var match = hash.match(/^#view=(agent|files|editor|models|guide|chat|model)$/);
        if (match) {
            var name = match[1];
            if (name === 'chat') name = 'agent';
            if (name === 'model') name = 'models';
            window.setTimeout(function () { activateSection(name); }, 120);
        }
    }

    function promptUrl() {
        activateSection('models');
        regroupModels();
        var diagnostics = document.getElementById('artifactDiagnostics');
        if (diagnostics) diagnostics.open = true;
        var input = document.querySelector('#modelUrlInput');
        if (input) { input.focus(); return true; }
        return false;
    }

    function initialChrome() {
        var wide = window.matchMedia('(min-width: 1181px)').matches;
        var mobile = window.matchMedia('(max-width: 900px)').matches;
        var medium = window.matchMedia('(min-width: 901px)').matches;
        setChrome('nav', mobile ? 'collapsed' : chromeValue('nav', medium ? 'expanded' : 'collapsed'), false);
        var inspectorValue = wide ? chromeValue('inspector', 'expanded') : 'collapsed';
        setChrome('inspector', inspectorValue, false);
    }

    function boot() {
        if (ready) return;
        ready = true;
        initialChrome();
        applySettings();
        updateThemeButton();
        updateStatus();
        renderAttachments();
        wire();
        updateStatus();
        watchSessions();
        watchMessages();
        watchModels();
        clock();
        clockTimer = window.setInterval(clock, 1000);
        window.setInterval(function () { updateStatus(); if (window.__studioUi && window.__studioUi.getSettings) { /* keep settings stable */ } }, 1500);
        applyHash();
        var systemMedia = window.matchMedia ? window.matchMedia('(prefers-color-scheme: light)') : null;
        if (systemMedia && systemMedia.addEventListener) systemMedia.addEventListener('change', function () { if (settings.theme === 'system') applySettings(); });
        window.__studioUi = Object.assign(window.__studioUi || {}, {
            activateSection: activateSection,
            promptUrl: promptUrl,
            openPalette: openPalette,
            closePalette: closePalette,
            openSettings: openSettings,
            openHelp: openHelp,
            toggleTheme: toggleTheme,
            setTheme: setThemeChoice,
            getSettings: function () { return Object.assign({}, settings); },
            updateSettings: updateSettings,
            updateStatus: updateStatus,
            getAttachments: function () { return attachments.slice(); },
            getImageRecallCount: function () { return recalledImages; },
            addAttachment: addAttachment,
            removeAttachment: removeAttachment,
            clearAttachments: clearAttachments,
            templatePrompt: templatePrompt,
            clearSessions: clearSessions,
            clearEditor: clearEditor
        });
        window.dispatchEvent(new CustomEvent('uor-studio-v2-ready'));
    }

    if (document.documentElement.dataset.studioReady === '1' || document.readyState === 'complete') window.setTimeout(boot, 0);
    else { document.addEventListener('uor-studio-ready', boot, { once: true }); window.setTimeout(boot, 2500); }
}());
