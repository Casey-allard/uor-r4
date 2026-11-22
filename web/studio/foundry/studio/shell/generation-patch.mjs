// FoundrySpecification.generationPatchBoundary: typed staging, then one commit.
import { captureProjectSource, commitPreparedProject } from "./project-edit.mjs";
import { prepareExistingProject } from "./project-creation.mjs";
const failure = code => Object.assign(new Error(code === "generationCancelled" ? "Generation cancelled" : code === "unlockedDependencyRequest" ? "Dependency requests require exact package versions" : "Invalid generation patch"), { code });
const portable = path => typeof path === "string" && path.length > 0 && !/[\\\u0000-\u001f:]/.test(path) && path.split("/").every(part => part && part !== "." && part !== "..");
export function assertGenerationActive(signal) { if (signal?.aborted) throw failure("generationCancelled"); }
export function stageGenerationPatch(source, input) {
  if (!source || typeof source.name !== "string" || typeof source.reference !== "string" || !/^blake3:[0-9a-f]{64}$/.test(source.reference) || !Array.isArray(source.files) || source.files.length > 1000 || source.files.some(row => !Array.isArray(row) || row.length !== 2 || typeof row[0] !== "string" || typeof row[1] !== "string")) throw failure("invalidGenerationPatch");
  let patch;
  try { patch = structuredClone(input); } catch { throw failure("invalidGenerationPatch"); }
  if (!patch || ![patch.writes, patch.deletes, patch.renames, patch.deps].every(Array.isArray) || patch.writes.length + patch.deletes.length + patch.renames.length > 1000 || patch.deps.length > 128) throw failure("invalidGenerationPatch");
  const files = new Map(source.files), used = new Set(), changedPaths = [];
  const claim = (path, internal = false) => { if (!portable(path) || used.has(path) || (!internal && path === "prism/dependencies.json")) throw failure("invalidGenerationPatch"); used.add(path); };
  let changed = 0;
  for (const item of patch.writes) {
    if (!item || typeof item.content !== "string") throw failure("invalidGenerationPatch"); claim(item.path);
    if (files.get(item.path) !== item.content) { files.set(item.path, item.content); changedPaths.push(item.path); changed++; }
  }
  for (const item of patch.deletes) {
    if (!item) throw failure("invalidGenerationPatch"); claim(item.path);
    if (!files.has(item.path)) throw failure("invalidGenerationPatch"); files.delete(item.path); changedPaths.push(item.path); changed++;
  }
  for (const item of patch.renames) {
    if (!item) throw failure("invalidGenerationPatch"); claim(item.from); claim(item.to);
    if (!files.has(item.from) || files.has(item.to)) throw failure("invalidGenerationPatch");
    files.set(item.to, files.get(item.from)); files.delete(item.from); changedPaths.push(item.from, item.to); changed++;
  }
  let dependencies;
  if (patch.deps.length) {
    if (!Array.isArray(source.dependencies) || source.dependencies.length > 128) throw failure("invalidGenerationPatch");
    const selected = new Map();
    const pattern = /^((?:@[a-z0-9][a-z0-9._-]*\/)?[a-z0-9][a-z0-9._-]*)@((?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*))$/;
    for (const dependency of [...source.dependencies, ...patch.deps]) {
      const match = typeof dependency === "string" && dependency.length <= 256 ? dependency.match(pattern) : null;
      if (!match) throw failure("unlockedDependencyRequest");
      if (selected.has(match[1]) && selected.get(match[1]) !== match[2]) throw failure("invalidGenerationPatch");
      selected.set(match[1], match[2]);
    }
    if (selected.size > 128) throw failure("invalidGenerationPatch");
    const entries = [...selected].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
    dependencies = entries.map(([name, version]) => `${name}@${version}`);
    const path = "prism/dependencies.json"; claim(path, true);
    const text = JSON.stringify({ profile: "foundry/direct-dependency-requests/1", dependencies: entries.map(([name, version]) => ({ name, version })) }, null, 2) + "\n";
    if (files.get(path) !== text || JSON.stringify(dependencies) !== JSON.stringify(source.dependencies)) { files.set(path, text); changedPaths.push(path); changed++; }
  }
  return { files, changedPaths, changed, dependencies };
}
export async function applyStoredProjectPatch(db, appId, patch, boundary) {
  assertGenerationActive(boundary.signal);
  if (!Number.isSafeInteger(appId) || appId < 1) throw failure("invalidGenerationPatch");
  let copied, rulesUpdate, appUpdate; try { copied = structuredClone(patch); rulesUpdate = structuredClone(boundary.rulesUpdate); appUpdate = structuredClone(boundary.appUpdate); } catch { throw failure("invalidGenerationPatch"); }
  let source;
  if (boundary.source) { try { source = structuredClone(boundary.source); } catch { throw failure("invalidGenerationPatch"); } }
  else source = await captureProjectSource(db, appId);
  const staged = stageGenerationPatch(source, copied);
  assertGenerationActive(boundary.signal);
  if (!staged.changed && !rulesUpdate && !appUpdate) return { changed: 0, oid: source.reference, source };
  const prepared = staged.changed ? await prepareExistingProject({ name: source.name, files: Object.fromEntries(staged.files) }, boundary, source.reference) : { oid: source.reference, files: source.files, objects: [] };
  assertGenerationActive(boundary.signal);
  const oid = await commitPreparedProject(db, appId, staged.changedPaths, source, prepared, { signal: boundary.signal, dependencies: staged.dependencies, message: "Apply generated file batch", rulesUpdate, appUpdate, recordVersion: !!staged.changed });
  if (staged.changed) boundary.onCommit?.(oid, staged.changed);
  const applicationState = appUpdate ? appUpdate.state ?? undefined : staged.changedPaths.some(path => ['prism/App.lex.tex','prism/spec.json'].includes(path)) ? undefined : source.applicationState;
  return { changed: staged.changed, oid, source: { name: source.name, reference: oid, files: prepared.files, dependencies: staged.dependencies || source.dependencies, ...(applicationState === undefined ? {} : {applicationState}) } };
}
