import { useEffect, useRef, useState } from 'preact/hooks';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { OperationGate } from '../adapters/operations.js';
import type { ApiProvider, ModelSourcesApi } from '../../assets/js/model_sources.js';
import type { LifecycleStore } from '../state/lifecycle.js';
import { classifyEndpoint } from '../state/lifecycle.js';
import { useLifecycle } from '../state/useLifecycle.js';
import { Modal } from './Modal.js';

interface ModelPickerProps {
    store: LifecycleStore;
    adapter: ModelSourceAdapter;
    sources: ModelSourcesApi;
    operations: OperationGate;
    onError(message: string, operation: string): void;
    onSuccess?(operation: string): void;
}

interface ProviderDraft {
    label: string;
    preset: string;
    baseUrl: string;
    model: string;
    apiKey: string;
}

const emptyDraft: ProviderDraft = { label: '', preset: 'custom', baseUrl: '', model: '', apiKey: '' };

export function ModelPicker({ store, adapter, sources, operations, onError, onSuccess }: ModelPickerProps) {
    const state = useLifecycle(store);
    const [url, setUrl] = useState('');
    const [providerDraft, setProviderDraft] = useState<ProviderDraft>(emptyDraft);
    const [editingId, setEditingId] = useState<string | null>(null);
    const [providerOpen, setProviderOpen] = useState(false);
    const [providerError, setProviderError] = useState<{ message: string; operation: string } | null>(null);
    const [removing, setRemoving] = useState<{ id: string; label: string } | null>(null);
    const [testResults, setTestResults] = useState<Record<string, 'testing' | 'passed' | 'failed'>>({});
    const testVersions = useRef<Record<string, number>>({});
    const mounted = useRef(true);
    const [, setRevision] = useState(0);
    const refresh = () => setRevision((value) => value + 1);
    const messageFor = (error: unknown) => error instanceof Error ? error.message : String(error);
    const report = (error: unknown, operation: string) => { if (mounted.current) onError(messageFor(error), operation); };
    const reportProvider = (error: unknown, operation: string) => {
        if (!mounted.current) return;
        setProviderError({ message: messageFor(error), operation });
        report(error, operation);
    };
    const clearProviderError = (operation: string) => setProviderError((current) => current?.operation === operation ? null : current);
    const succeed = (operation: string) => {
        if (!mounted.current) return;
        clearProviderError(operation);
        onSuccess?.(operation);
    };
    useEffect(() => {
        mounted.current = true;
        return () => { mounted.current = false; };
    }, []);
    const run = (action: Promise<unknown>, operation: string) => {
        void operations.run(operation, action, succeed, report);
    };

    useEffect(() => {
        let active = true;
        const changed = () => { if (active) refresh(); };
        document.addEventListener('uor-model-catalog-changed', changed);
        sources.loadModelCatalog().then(() => { changed(); succeed('catalog:load'); }).catch((error) => report(error, 'catalog:load'));
        return () => {
            active = false;
            document.removeEventListener('uor-model-catalog-changed', changed);
        };
    }, [sources]);

    const progress = state.progress;
    const total = progress?.total ?? null;
    const resident = state.residentArtifact;

    function testProvider(provider: ApiProvider) {
        const operation = `provider:test:${provider.id}`;
        const version = (testVersions.current[provider.id] || 0) + 1;
        testVersions.current[provider.id] = version;
        setTestResults((current) => ({ ...current, [provider.id]: 'testing' }));
        sources.testApi(provider.id).then((result) => {
            if (mounted.current && testVersions.current[provider.id] === version) {
                setTestResults((current) => ({ ...current, [provider.id]: result.ok ? 'passed' : 'failed' }));
                if (result.ok) succeed(operation);
                else reportProvider(new Error(result.detail || 'Connection test failed.'), operation);
            }
        }).catch((error) => {
            if (mounted.current && testVersions.current[provider.id] === version) {
                setTestResults((current) => ({ ...current, [provider.id]: 'failed' }));
                reportProvider(error, operation);
            }
        });
    }

    function editProvider(provider?: ApiProvider) {
        setEditingId(provider?.id ?? null);
        setProviderError(null);
        setProviderDraft(provider ? { label: provider.label, preset: provider.preset, baseUrl: provider.baseUrl, model: provider.model, apiKey: '' } : emptyDraft);
        setProviderOpen(true);
    }

    function saveProvider(event: Event) {
        event.preventDefault();
        const providerId = editingId;
        const operation = `provider:save:${providerId || 'new'}`;
        clearProviderError(operation);
        try {
            classifyEndpoint(providerDraft.baseUrl);
            if (editingId) {
                const patch: Partial<ApiProvider> = {
                    label: providerDraft.label,
                    preset: providerDraft.preset,
                    baseUrl: providerDraft.baseUrl,
                    model: providerDraft.model,
                };
                if (providerDraft.apiKey) patch.apiKey = providerDraft.apiKey;
                sources.updateApi(editingId, patch);
                testVersions.current[editingId] = (testVersions.current[editingId] || 0) + 1;
                setTestResults((current) => {
                    const next = { ...current };
                    delete next[editingId];
                    return next;
                });
                if (state.selectedSource?.kind === 'api' && state.selectedSource.id === editingId) {
                    store.dispatch({ type: 'api/selected', sourceId: editingId, endpoint: providerDraft.baseUrl });
                    document.dispatchEvent(new CustomEvent('uor-api-config-changed', { detail: { id: editingId } }));
                }
            } else {
                sources.addApi(providerDraft);
            }
            setProviderOpen(false);
            setEditingId(null);
            setProviderDraft(emptyDraft);
            refresh();
            succeed(operation);
            if (providerId) succeed(`provider:test:${providerId}`);
        } catch (error) {
            reportProvider(error, operation);
        }
    }

    function selectProvider(provider: ApiProvider) {
        const operation = `provider:select:${provider.id}`;
        try {
            clearProviderError(operation);
            classifyEndpoint(provider.baseUrl);
            adapter.selectApi(provider.id, provider.baseUrl);
            succeed(operation);
        } catch (error) {
            reportProvider(error, operation);
        }
    }

    function removeProvider() {
        if (!removing || state.session.phase === 'generating') return;
        const id = removing.id;
        const operation = `provider:remove:${id}`;
        try {
            clearProviderError(operation);
            sources.removeApi(id);
            testVersions.current[id] = (testVersions.current[id] || 0) + 1;
            setTestResults((current) => {
                const next = { ...current };
                delete next[id];
                return next;
            });
            if (state.selectedSource?.kind === 'api' && state.selectedSource.id === id) {
                const active = sources.active();
                store.dispatch({ type: 'artifact/selected', sourceId: active.id || 'bundled-demo' });
                run(adapter.loadBundled(), 'artifact:load');
            }
            setRemoving(null);
            refresh();
            succeed(operation);
            succeed(`provider:test:${id}`);
            succeed(`provider:select:${id}`);
            succeed(`provider:save:${id}`);
        } catch (error) {
            reportProvider(error, operation);
        }
    }

    return (
        <section aria-label="Model sources">
            <h1>Models</h1>
            <p>{state.selectedSource ? `${state.selectedSource.kind === 'api' ? 'API' : 'Artifact'}: ${state.selectedSource.id}` : 'No model source selected'}</p>
            <p>{state.selectedSource?.kind === 'artifact' && (state.artifact.phase !== 'ready' || state.residentArtifact?.sourceId !== state.selectedSource.id) ? 'Selected model; not ready' : `Artifact: ${state.artifact.phase}`}</p>
            {state.artifact.error && <p role="alert">{state.artifact.error}</p>}

            <section aria-label="Local engine">
                <h2>Local engine</h2>
                <button type="button" onClick={() => run(adapter.loadBundled(), 'artifact:load')}>Load bundled demo</button>
                <label>Model file<input aria-label="Model file" type="file" accept=".lut,.bin,.d11" onChange={(event) => {
                    const file = event.currentTarget.files?.[0];
                    if (file) run(adapter.loadLocalFile(file), 'artifact:load');
                }} /></label>
                <form data-action="load-url" onSubmit={(event) => {
                    event.preventDefault();
                    if (url.trim()) run(adapter.loadFromUrl(url.trim()), 'artifact:load');
                }}>
                    <label>Model URL<input aria-label="Model URL" type="url" value={url} onInput={(event) => setUrl(event.currentTarget.value)} /></label>
                    <button type="submit">Load URL</button>
                </form>
            </section>

            <section aria-label="Published models">
                <h2>Published models</h2>
                {sources.catalog().models.map((entry) => (
                    <button type="button" onClick={() => run(adapter.loadPublished(entry.id || entry.url), 'artifact:load')}>{`Load ${entry.label || entry.id || entry.url}`}</button>
                ))}
            </section>

            {(state.artifact.phase === 'downloading' || state.artifact.phase === 'finalizing') && (
                <div>
                    <progress
                        aria-label="Model download progress"
                        max={total ?? undefined}
                        value={state.artifact.phase === 'downloading' && total !== null ? progress?.loaded : undefined}
                    />
                    {state.artifact.phase === 'finalizing' ? (
                        <p>Finalizing model; progress is indeterminate</p>
                    ) : total !== null ? (
                        <p>{progress?.loaded ?? 0} / {total} bytes ({progress?.percent ?? 0}%)</p>
                    ) : (
                        <p>{progress?.loaded ?? 0} bytes downloaded; total unknown</p>
                    )}
                </div>
            )}

            {resident && (
                <dl aria-label="Resident artifact">
                    <dt>File</dt><dd>{resident.fileName || 'Unnamed artifact'}</dd>
                    <dt>Schema</dt><dd>{resident.schema || 'Unknown schema'}</dd>
                    {resident.sha256 && <><dt>Hash</dt><dd>SHA-256 {resident.sha256} ({resident.verified === true ? 'verified' : 'not verified'})</dd></>}
                    <dt>Engine</dt><dd>{resident.engineSupported === false ? 'Engine unsupported' : 'Engine supported'}</dd>
                    {resident.engineSupported !== false && resident.canGenerate === false && resident.canPredict === true && <><dt>Capability</dt><dd>Next-token prediction only</dd></>}
                    {(resident.engineSupported === false || (resident.canGenerate === false && resident.canPredict !== true)) && <><dt>Capability</dt><dd>Generation unavailable</dd></>}
                </dl>
            )}

            <section aria-label="API providers">
                <h2>API providers</h2>
                <p>Connection test: {state.selectedSource?.kind === 'api' ? testResults[state.selectedSource.id] || state.api.testStatus : 'untested'}. Selection does not prove reachability.</p>
                {providerError && !providerOpen && !removing && <p role="alert">{providerError.message}</p>}
                <button type="button" onClick={() => editProvider()}>Add API provider</button>
                <ul>
                    {sources.list().api.map((provider) => (
                        <li>
                            <strong>{provider.label}</strong>
                            <span>{provider.baseUrl} · {provider.model}</span>
                            <span>Test: {testResults[provider.id] || (state.selectedSource?.kind === 'api' && state.selectedSource.id === provider.id ? state.api.testStatus : 'untested')}</span>
                            <button type="button" onClick={() => selectProvider(provider)}>{`Use ${provider.label}`}</button>
                            <button type="button" onClick={() => testProvider(provider)}>{`Test ${provider.label}`}</button>
                            <button type="button" onClick={() => editProvider(provider)}>{`Edit ${provider.label}`}</button>
                            <button type="button" disabled={state.session.phase === 'generating'} onClick={() => { setProviderError(null); setRemoving({ id: provider.id, label: provider.label }); }}>{`Remove ${provider.label}`}</button>
                        </li>
                    ))}
                </ul>
            </section>

            <details>
                <summary>Research</summary>
                <p>Experimental sources are labeled separately and never presented as qualified conversational models.</p>
            </details>

            {providerOpen && (
                <Modal title={editingId ? 'Edit API provider' : 'Add API provider'} onClose={() => { setProviderOpen(false); setEditingId(null); setProviderDraft(emptyDraft); }}>
                    {providerError && providerError.operation === `provider:save:${editingId || 'new'}` && <p role="alert">{providerError.message}</p>}
                    <form data-action="save-provider" onSubmit={saveProvider}>
                        <label>Provider name<input aria-label="Provider name" required value={providerDraft.label} onInput={(event) => setProviderDraft({ ...providerDraft, label: event.currentTarget.value })} /></label>
                        <label>Provider base URL<input aria-label="Provider base URL" required type="url" value={providerDraft.baseUrl} onInput={(event) => setProviderDraft({ ...providerDraft, baseUrl: event.currentTarget.value })} /></label>
                        <label>Provider model<input aria-label="Provider model" value={providerDraft.model} onInput={(event) => setProviderDraft({ ...providerDraft, model: event.currentTarget.value })} /></label>
                        <label>API key<input aria-label="API key" type="password" autocomplete="off" value={providerDraft.apiKey} placeholder={editingId ? 'Leave blank to keep current key' : ''} onInput={(event) => setProviderDraft({ ...providerDraft, apiKey: event.currentTarget.value })} /></label>
                        <button type="submit">Save provider</button>
                    </form>
                </Modal>
            )}

            {removing && (
                <Modal title="Remove API provider" onClose={() => setRemoving(null)}>
                    {providerError && providerError.operation === `provider:remove:${removing.id}` && <p role="alert">{providerError.message}</p>}
                    <p>Remove {removing.label}? Existing conversations are preserved.</p>
                    <button type="button" autofocus onClick={() => setRemoving(null)}>Keep provider</button>
                    <button type="button" onClick={removeProvider}>Confirm removal</button>
                </Modal>
            )}
        </section>
    );
}
