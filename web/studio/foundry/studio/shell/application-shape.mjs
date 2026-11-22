// FoundrySpecification.applicationShapeBoundary: source chooses one authority;
// source files are never evidence that a host-owned lifecycle was established.
export const APP_SOURCE_PATH = 'prism/App.lex.tex';
export const APP_STATE_PATH = 'prism/app.json';
export const shapeFailure = code => Object.assign(new Error(code === 'applicationStateReadOnly' ? 'Application standing is managed by verification, not file edits' : 'Application source contains conflicting or unsupported authorities'), {code});
export function isStatefulModule(module) {
  const text = String(module || '').replace(/"(?:\\.|[^"\\])*"/g, '""').replace(/%[^\n]*/g, '');
  return /\\importmodule\{Foundation\.View\.Browser\.V1\.Model\}/.test(text) || /structure\{\s*name:\s*State\b/.test(text);
}
export function applicationShape(source) {
  const files = new Map(source.files), stateful = isStatefulModule(files.get(APP_SOURCE_PATH));
  if (stateful && files.has('prism/spec.json')) throw shapeFailure('applicationShapeConflict');
  if (!stateful && files.has(APP_SOURCE_PATH) && !files.has('prism/spec.json')) throw shapeFailure('applicationShapeConflict');
  return stateful ? 'stateful' : files.has('prism/spec.json') ? 'decision' : 'generic';
}
export function activeApplicationModule(source, canonicalSource) {
  const files = new Map(source.files), shape = applicationShape(source);
  if (shape === 'stateful') return [APP_SOURCE_PATH, files.get(APP_SOURCE_PATH)];
  if (shape === 'decision') {
    try { const path = `prism/${canonicalSource(JSON.parse(files.get('prism/spec.json'))).name}.lex.tex`; return files.has(path) ? [path, files.get(path)] : null; }
    catch { throw shapeFailure('applicationShapeConflict'); }
  }
  return null;
}
export function retiredApplicationPaths(source, nextShape, nextPath, canonicalSource) {
  const files = new Map(source.files), retired = new Set();
  if (nextShape === 'stateful') {
    if (files.has('prism/spec.json')) {
      try { retired.add(`prism/${canonicalSource(JSON.parse(files.get('prism/spec.json'))).name}.lex.tex`); } catch {}
      retired.add('prism/spec.json'); retired.add('src/lib/prism.ts');
    }
  } else if (nextShape === 'decision') {
    if (isStatefulModule(files.get(APP_SOURCE_PATH))) retired.add(APP_SOURCE_PATH);
    retired.add(APP_STATE_PATH);
  }
  retired.delete(nextPath);
  return [...retired].filter(path => files.has(path));
}
export async function boundApplicationState(source, derive) {
  if (applicationShape(source) !== 'stateful') return null;
  const state = source.applicationState;
  if (!state || typeof state !== 'object' || Array.isArray(state) || !['drafting','checked','proving','proved','published'].includes(state.phase) || !/^blake3:[0-9a-f]{64}$/.test(state.sourceKappa || '')) return null;
  if (['proved','published'].includes(state.phase) && (typeof state.proof?.attestation_id !== 'string' || !state.proof.attestation_id)) return null;
  const module = new Map(source.files).get(APP_SOURCE_PATH);
  return await derive(new TextEncoder().encode(module)) === state.sourceKappa ? structuredClone(state) : null;
}
