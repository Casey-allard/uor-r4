// Spec patches: the one way anything on the live screen changes. A click on a word in the frame, and a
// line the agent writes, both become the same patch, a list of operations on the spec by path; nothing
// ever edits the screen or the generated code. Pure: the host persists what this returns.
//
//   { op: "set", path: ["rules", 2, "then"], value: "none" }
//   { op: "insert", path: ["examples", 0], value: { "request": "yes 2 5", "response": "confirmed" } }
//   { op: "remove", path: ["examples", 3] }
export class PatchError extends Error {}

const isIndex = (k) => Number.isInteger(k) && k >= 0;
const json = (v) => v === null || ["string", "number", "boolean"].includes(typeof v) || (typeof v === "object" && JSON.stringify(v) !== undefined);

/** The spec with the operations applied, as a new object; the input is never changed. */
export function applyOps(spec, ops) {
  if (!Array.isArray(ops) || ops.length === 0) throw new PatchError("A patch is a list of operations.");
  let out = structuredClone(spec);
  for (const o of ops) {
    if (!o || !Array.isArray(o.path) || o.path.length === 0) throw new PatchError("Each operation names a path in the spec.");
    const parentPath = o.path.slice(0, -1), key = o.path[o.path.length - 1];
    let parent = out;
    for (const k of parentPath) {
      if (parent === null || typeof parent !== "object" || !(k in parent)) throw new PatchError(`The spec has no ${o.path.join(".")}.`);
      parent = parent[k];
    }
    if (parent === null || typeof parent !== "object") throw new PatchError(`The spec has no ${o.path.join(".")}.`);
    if (o.op === "set") {
      if (!json(o.value)) throw new PatchError("A value is plain JSON.");
      if (Array.isArray(parent) ? !(isIndex(key) && key < parent.length) : typeof key !== "string") throw new PatchError(`The spec has no ${o.path.join(".")}.`);
      parent[key] = structuredClone(o.value);
    } else if (o.op === "insert") {
      if (!Array.isArray(parent) || !isIndex(key) || key > parent.length) throw new PatchError(`${o.path.join(".")} is not a place in a list.`);
      if (!json(o.value)) throw new PatchError("A value is plain JSON.");
      parent.splice(key, 0, structuredClone(o.value));
    } else if (o.op === "remove") {
      if (Array.isArray(parent) ? !(isIndex(key) && key < parent.length) : !(key in parent)) throw new PatchError(`The spec has no ${o.path.join(".")}.`);
      if (Array.isArray(parent)) parent.splice(key, 1); else delete parent[key];
    } else throw new PatchError(`Unknown operation ${o.op}; use set, insert or remove.`);
  }
  return out;
}

/** The value a path names, or undefined. */
export function at(spec, path) {
  let v = spec;
  for (const k of path) { if (v === null || typeof v !== "object" || !(k in v)) return undefined; v = v[k]; }
  return v;
}

/**
 * One app's edit history. `check(spec)` is Forge's checker ({ ok, issues }); a patch that fails it is
 * refused and the last good spec stands. Undo and redo move between whole specs, so the screen they give
 * is the projection of that spec, root kappa and all.
 */
export function createEditor(initial, check) {
  const history = [structuredClone(initial)];
  let index = 0;
  return {
    get spec() { return history[index]; },
    get canUndo() { return index > 0; },
    get canRedo() { return index < history.length - 1; },
    /** Try a patch: { ok: true, spec } once it checks, else { ok: false, issues } with nothing changed. */
    apply(ops) {
      let next;
      try { next = applyOps(history[index], ops); } catch (e) { if (e instanceof PatchError) return { ok: false, issues: [e.message] }; throw e; }
      const verdict = check(next);
      if (!verdict.ok) return { ok: false, issues: verdict.issues && verdict.issues.length ? verdict.issues : ["The rules do not check."] };
      history.splice(index + 1, history.length, next);
      index = history.length - 1;
      return { ok: true, spec: next };
    },
    undo() { if (index > 0) index--; return history[index]; },
    redo() { if (index < history.length - 1) index++; return history[index]; },
    /** The spec changed elsewhere (a chat turn wrote a whole new one): it becomes the newest entry. */
    sync(spec) { if (JSON.stringify(spec) !== JSON.stringify(history[index])) { history.splice(index + 1, history.length, structuredClone(spec)); index = history.length - 1; } },
  };
}
