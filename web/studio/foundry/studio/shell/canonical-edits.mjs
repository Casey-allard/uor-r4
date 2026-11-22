// FoundrySpecification.canonicalEditBoundary: prepare intent; the host commits it atomically.
import {APP_SOURCE_PATH, APP_STATE_PATH, isStatefulModule, shapeFailure} from './application-shape.mjs';
const invalid = () => Object.assign(new Error("The canonical model edit is invalid or unsupported"), { code: "invalidCanonicalEdit" });
export const canonicalEditPath = path => path === APP_STATE_PATH || path === "prism/spec.json" || path === "src/lib/prism.ts" || (typeof path === "string" && /^prism\/[A-Za-z_]\w*\.lex\.tex$/.test(path));
export function planCanonicalEdit(input, source, { lexlean, prism }) {
  try {
    if (!input || typeof input !== "object" || typeof input.filePath !== "string") throw invalid();
    if (!canonicalEditPath(input.filePath)) return null;
    if (input.filePath === APP_STATE_PATH) throw shapeFailure('applicationStateReadOnly');
    if (input.filePath === APP_SOURCE_PATH && (typeof input.content !== 'string' || input.content.length > 1024 * 1024 || new TextEncoder().encode(input.content).length > 1024 * 1024)) throw invalid();
    const files = new Map(source.files), previousText = files.get("prism/spec.json");
    if (input.filePath === APP_SOURCE_PATH && isStatefulModule(input.content)) {
      return Object.freeze({shape:'stateful',moduleText:input.content,warning:null});
    }
    const replacingStateful = input.filePath === APP_SOURCE_PATH && isStatefulModule(files.get(APP_SOURCE_PATH));
    if (input.filePath === "src/lib/prism.ts") {
      if (files.has("prism/spec.json")) throw Object.assign(new Error("Edit the canonical model to change its generated rule runtime"), { code: "canonicalProjectionReadOnly" });
      return null;
    }
    let previous = null, previousPath = null;
    if (previousText !== undefined) {
      try { previous = JSON.parse(previousText); previousPath = `prism/${lexlean.canonicalSource(previous).name}.lex.tex`; }
      catch { if (input.filePath !== "prism/spec.json") throw invalid(); }
    }
    if (!replacingStateful && input.filePath !== "prism/spec.json" && previousPath && input.filePath !== previousPath) return null;
    if (typeof input.content !== "string" || input.content.length > 1024 * 1024 || new TextEncoder().encode(input.content).length > 1024 * 1024) throw invalid();
    let spec;
    if (input.filePath === "prism/spec.json") spec = JSON.parse(input.content);
    else {
      spec = lexlean.readLexLean(input.content);
      if (!previousPath && !replacingStateful && input.filePath !== `prism/${lexlean.canonicalSource(spec).name}.lex.tex`) throw invalid();
      for (const field of ["purpose", "view", "meanings"]) if (previous?.[field] !== undefined) spec[field] = structuredClone(previous[field]);
    }
    const checked = prism.checkSpec(spec);
    if (checked.ok !== true) throw invalid();
    lexlean.canonicalSource(checked.spec);
    return Object.freeze({ text: "<prism-spec>" + JSON.stringify(checked.spec).replaceAll("<", "\\u003c") + "</prism-spec>", warning: checked.notes?.length ? checked.notes.join("\n").slice(0, 2000) : null });
  } catch (error) {
    if (["canonicalProjectionReadOnly", "applicationStateReadOnly"].includes(error?.code)) throw error;
    throw invalid();
  }
}
