const DESKTOP_UNAVAILABLE = 'Native host actions are unavailable in this browser build.';

function unavailable() {
    return Promise.reject(new Error(DESKTOP_UNAVAILABLE));
}

export function createDesktopActions({ invoke } = {}) {
    if (typeof invoke !== 'function') {
        return {
            available: false,
            chooseProjectFolder: unavailable,
            chooseModelFolder: unavailable,
            listDirectory: unavailable,
            readFile: unavailable,
            writeFile: unavailable,
            listLocalModels: unavailable,
            gitStatus: unavailable,
            gitDiff: unavailable,
            enableTerminal: unavailable,
            confirmTerminalCommand: unavailable,
            runTerminalCommand: unavailable,
        };
    }
    const call = (command, args = {}) => invoke(command, args);
    return {
        available: true,
        chooseProjectFolder: () => call('native_choose_project_folder'),
        chooseModelFolder: () => call('native_choose_model_folder'),
        listDirectory: (path, depth = 4) => call('native_list_directory', { folderPath: path, maxDepth: depth }),
        readFile: (path) => call('native_read_file', { filePath: path }),
        writeFile: (path, content) => call('native_write_file', { filePath: path, content }),
        listLocalModels: (path) => call('native_list_local_models', path ? { modelDir: path } : {}),
        gitStatus: (repoPath) => call('native_git_status', { repoPath }),
        gitDiff: (repoPath) => call('native_git_diff', { repoPath }),
        enableTerminal: () => call('native_enable_terminal'),
        confirmTerminalCommand: (cwd, command) => call('native_confirm_terminal_command', { cwd, command }),
        runTerminalCommand: async (cwd, command) => {
            const approval = await call('native_confirm_terminal_command', { cwd, command });
            if (!approval || approval.approved !== true) return null;
            return call('native_run_terminal_command', {
                cwd,
                command,
                userConfirmed: true,
                confirmedCommand: command,
            });
        },
    };
}

function toast(message, tone = 'info') {
    if (typeof globalThis.__studioUi?.toast === 'function') globalThis.__studioUi.toast(message, tone);
}

export function mountDesktopActions({ actions, root = globalThis.document } = {}) {
    if (!root || !actions) return;
    const byId = (id) => root.getElementById(id);
    const refreshButton = byId('btnRefreshChanges');
    const copyButton = byId('btnCopyDiff');
    const enableButton = byId('btnTerminalEnable');
    const runButton = byId('btnRunTerminal');
    const cwdInput = byId('terminalCwd');
    const commandInput = byId('terminalCommand');
    let currentDiff = '';

    if (!actions.available) {
        if (byId('gitStatusSummary')) byId('gitStatusSummary').textContent = 'Native Git is available in the desktop build.';
        if (byId('terminalConsentState')) byId('terminalConsentState').textContent = 'Guarded terminal actions are unavailable in this browser build.';
        return;
    }

    for (const button of [refreshButton, enableButton, runButton]) {
        if (button) {
            button.disabled = false;
            button.removeAttribute('title');
        }
    }

    const repoPath = () => {
        const typed = cwdInput && cwdInput.value.trim();
        if (typed) return typed;
        const summary = globalThis.__studioFiles?.treeSummary?.();
        return String(summary?.rootPath || '');
    };

    async function refresh() {
        const path = repoPath();
        if (!path) {
            byId('gitStatusSummary').textContent = 'Open a project folder in Files before inspecting changes.';
            byId('gitDiffOutput').textContent = 'No repository selected.';
            return;
        }
        if (cwdInput) cwdInput.value = path;
        refreshButton.disabled = true;
        try {
            const [status, report] = await Promise.all([actions.gitStatus(path), actions.gitDiff(path)]);
            const files = (status.modified_files?.length || 0) + (status.untracked_files?.length || 0);
            const limits = [
                status.truncated || report.truncated ? 'truncated' : '',
                status.timed_out || report.timed_out ? 'timed out' : '',
            ].filter(Boolean).join(' · ');
            byId('gitStatusSummary').textContent = `${status.branch} · ${files} changed file${files === 1 ? '' : 's'}${limits ? ' · ' + limits : ''}`;
            currentDiff = String(report.diff || '');
            byId('gitDiffOutput').textContent = currentDiff + (report.truncated ? '\n[diff truncated]' : '') + (report.timed_out ? '\n[diff timed out]' : '');
            copyButton.disabled = !currentDiff;
        } catch (error) {
            byId('gitStatusSummary').textContent = 'Git inspection failed: ' + String(error?.message || error);
            byId('gitDiffOutput').textContent = 'No diff available.';
            copyButton.disabled = true;
        } finally {
            refreshButton.disabled = false;
        }
    }

    refreshButton?.addEventListener('click', refresh);
    copyButton?.addEventListener('click', async () => {
        if (!currentDiff) return;
        try {
            await globalThis.navigator.clipboard.writeText(currentDiff);
            toast('Diff copied.', 'success');
        } catch (error) {
            toast('Could not copy the diff: ' + String(error?.message || error), 'error');
        }
    });
    enableButton?.addEventListener('click', async () => {
        try {
            const result = await actions.enableTerminal();
            byId('terminalConsentState').textContent = result?.enabled
                ? 'Terminal is enabled for this app session. Every command still needs separate native approval.'
                : 'Terminal enablement was cancelled.';
        } catch (error) {
            byId('terminalConsentState').textContent = 'Could not enable terminal: ' + String(error?.message || error);
        }
    });
    runButton?.addEventListener('click', async () => {
        const cwd = repoPath();
        const command = String(commandInput?.value || '').trim();
        if (!cwd || !command) {
            byId('terminalOutput').textContent = 'Choose a project folder and enter an allowed command.';
            return;
        }
        runButton.disabled = true;
        byId('terminalOutput').textContent = 'Waiting for native command approval…';
        try {
            const output = await actions.runTerminalCommand(cwd, command);
            byId('terminalOutput').textContent = output === null ? 'Command approval was cancelled; nothing ran.' : String(output || '(no output)');
        } catch (error) {
            byId('terminalOutput').textContent = 'Command failed: ' + String(error?.message || error);
        } finally {
            runButton.disabled = false;
        }
    });
    commandInput?.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            event.preventDefault();
            runButton?.click();
        }
    });
    root.addEventListener('uor-files-changed', (event) => {
        const path = String(event.detail?.rootPath || '');
        if (path && cwdInput) cwdInput.value = path;
    });

    for (const element of root.querySelectorAll('[data-action="load-url"], [data-action="add-api"]')) {
        element.disabled = true;
        element.setAttribute('aria-disabled', 'true');
        element.title = 'Available in the browser build.';
    }

    const modelLabel = root.querySelector('label[for="modelFileInput"]');
    if (modelLabel) {
        const button = root.createElement('button');
        button.type = 'button';
        button.id = 'nativeModelFolderButton';
        button.className = modelLabel.className;
        button.textContent = 'Choose model folder';
        button.addEventListener('click', async () => {
            const status = byId('nativeModelFolderStatus') || root.createElement('p');
            status.id = 'nativeModelFolderStatus';
            status.className = 'hint';
            if (!status.isConnected) byId('modelSourceMount')?.appendChild(status);
            try {
                const selected = await actions.chooseModelFolder();
                if (!selected) return;
                const models = await actions.listLocalModels(selected.path);
                const names = models.map((model) => model.name).filter(Boolean);
                status.textContent = `${names.length} pinned model${names.length === 1 ? '' : 's'}: ${names.join(', ') || 'none found'}. This list does not switch the running sidecar weights.`;
            } catch (error) {
                status.textContent = 'Could not inspect that model folder: ' + String(error?.message || error);
            }
        });
        modelLabel.replaceWith(button);
        const input = byId('modelFileInput');
        if (input) input.disabled = true;
    }
}

function bootDesktopActions() {
    const core = globalThis.__TAURI__?.core;
    if (typeof core?.invoke !== 'function' || typeof globalThis.__TAURI__?.core?.Channel !== 'function') return;
    const actions = createDesktopActions({ invoke: core.invoke.bind(core) });
    globalThis.UORDesktopActions = actions;
    mountDesktopActions({ actions });
}

if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', bootDesktopActions, { once: true });
    else bootDesktopActions();
}
