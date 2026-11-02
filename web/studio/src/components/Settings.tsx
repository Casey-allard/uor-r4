import { useState } from 'preact/hooks';
import type { StudioSettings, StudioUiApi } from '../adapters/workflow.js';
import { Modal } from './Modal.js';

interface SettingsProps {
    ui: StudioUiApi;
    onClose(): void;
    onChange?(kind: 'sessions' | 'editor'): void;
    onSuccess?(kind: 'sessions' | 'editor'): void;
    onError?(message: string, operation: string): void;
    onSettingsChange?(settings: StudioSettings): void;
    generating?: boolean;
}

function clamp(value: number, min: number, max: number, fallback: number): number {
    return Number.isFinite(value) ? Math.max(min, Math.min(max, value)) : fallback;
}

export function Settings({ ui, onClose, onChange, onSuccess, onError, onSettingsChange, generating = false }: SettingsProps) {
    const [settings, setSettings] = useState<StudioSettings>(() => ui.getSettings());
    const [fontSizeDraft, setFontSizeDraft] = useState(String(settings.fontSize));
    const [tabSizeDraft, setTabSizeDraft] = useState(String(settings.tabSize));
    const [clearing, setClearing] = useState<'sessions' | 'editor' | null>(null);
    const [error, setError] = useState<string | null>(null);

    function reportError(message: string) {
        setError(message);
        onError?.(message, `settings:${clearing || 'other'}`);
    }

    function update<K extends keyof StudioSettings>(key: K, value: StudioSettings[K]) {
        const next = { ...settings, [key]: value };
        setSettings(next);
        ui.updateSettings({ [key]: value });
        onSettingsChange?.(next);
    }

    function commitNumber(key: 'fontSize' | 'tabSize', raw: string) {
        const font = key === 'fontSize';
        const value = clamp(Number(raw), font ? 10 : 1, font ? 24 : 8, font ? 13 : 4);
        if (font) setFontSizeDraft(String(value));
        else setTabSizeDraft(String(value));
        update(key, value);
    }

    function confirmClear() {
        let changed = false;
        let succeeded = false;
        setError(null);
        if (clearing === 'sessions') {
            if (generating) { setClearing(null); return; }
            const result = ui.clearSessions();
            if (result.error) reportError(result.error);
            else {
                changed = result.cleared > 0;
                succeeded = result.cleared > 0 && result.cleared === result.total;
                if (result.cleared < result.total) reportError(`Cleared ${result.cleared} of ${result.total} sessions; stop the current reply before deleting the rest.`);
            }
        }
        if (clearing === 'editor') {
            ui.clearEditor();
            changed = true;
            succeeded = true;
        }
        const completed = clearing;
        setClearing(null);
        if (changed && completed) onChange?.(completed);
        if (succeeded && completed) onSuccess?.(completed);
    }

    return (
        <Modal title="Studio settings" onClose={onClose}>
            {error && <p role="alert">{error}</p>}
            <section aria-label="Appearance">
                <h3>Appearance</h3>
                <label>Theme
                    <select aria-label="Theme" value={settings.theme} onChange={(event) => update('theme', event.currentTarget.value as StudioSettings['theme'])}>
                        <option value="dark">Dark</option><option value="light">Light</option><option value="system">System</option>
                    </select>
                </label>
                <label><input aria-label="Reduce motion" type="checkbox" checked={settings.reduceMotion} onChange={(event) => update('reduceMotion', event.currentTarget.checked)} /> Reduce motion</label>
            </section>
            <section aria-label="Editor preferences">
                <h3>Editor</h3>
                <label>Font size<input aria-label="Editor font size" type="number" min="10" max="24" value={fontSizeDraft} onInput={(event) => setFontSizeDraft(event.currentTarget.value)} onBlur={(event) => commitNumber('fontSize', event.currentTarget.value)} /></label>
                <label>Tab size<input aria-label="Tab size" type="number" min="1" max="8" value={tabSizeDraft} onInput={(event) => setTabSizeDraft(event.currentTarget.value)} onBlur={(event) => commitNumber('tabSize', event.currentTarget.value)} /></label>
                <label><input aria-label="Word wrap" type="checkbox" checked={settings.wordWrap} onChange={(event) => update('wordWrap', event.currentTarget.checked)} /> Word wrap</label>
            </section>
            <section aria-label="Data controls">
                <h3>Data</h3>
                <p>Clearing uses the existing session and editor stores. Provider settings are not included.</p>
                <button type="button" disabled={generating} title={generating ? 'Stop the current reply before clearing sessions.' : undefined} onClick={() => setClearing('sessions')}>Clear sessions</button>
                <button type="button" onClick={() => setClearing('editor')}>Clear editor tabs</button>
                {clearing && (
                    <section aria-label="Confirm data clearing">
                        <p>{clearing === 'sessions' ? 'Clear all local sessions?' : 'Close all editor tabs?'}</p>
                        <button type="button" autofocus onClick={() => setClearing(null)}>Cancel</button>
                        <button type="button" onClick={confirmClear}>{clearing === 'sessions' ? 'Confirm clear sessions' : 'Confirm clear editor tabs'}</button>
                    </section>
                )}
            </section>
            <section aria-label="About">
                <h3>About</h3>
                <p>UOR-R4 Studio is pre-alpha. M1 supports next-token prediction, not general conversation.</p>
            </section>
        </Modal>
    );
}
