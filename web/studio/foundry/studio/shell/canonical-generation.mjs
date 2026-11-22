// FoundrySpecification.canonicalGenerationBoundary: generic writes carry no model authority.
import {APP_SOURCE_PATH,APP_STATE_PATH} from './application-shape.mjs';
const invalid = () => Object.assign(new Error("Invalid generation patch"), { code: "invalidGenerationPatch" });
const refused = () => Object.assign(new Error("Use canonical authoring commands to change model-owned files"), { code: "canonicalGenerationRefused" });
export function copyGenerationInput(tags, source) {
  try { return { tags: structuredClone(tags), source: source === undefined ? undefined : structuredClone(source) }; }
  catch { throw invalid(); }
}
export function assertCanonicalGeneration(source, patch, lexlean) {
  try {
    if (!patch || ![patch.writes, patch.deletes, patch.renames, patch.deps].every(Array.isArray) || patch.writes.length + patch.deletes.length + patch.renames.length > 1000 || patch.deps.length > 128 || !Array.isArray(source?.files)) throw invalid();
    const touched = [];
    for (const item of [...patch.writes, ...patch.deletes]) {
      if (!item || typeof item.path !== "string") throw invalid(); touched.push(item.path);
    }
    for (const item of patch.renames) {
      if (!item || typeof item.from !== "string" || typeof item.to !== "string") throw invalid(); touched.push(item.from, item.to);
    }
    const files = new Map(source.files);
    if (files.has(APP_SOURCE_PATH) && touched.some(path => [APP_SOURCE_PATH, APP_STATE_PATH].includes(path))) throw refused();
    if (files.has("prism/interaction.json") && touched.some(path => ["prism/interaction.json", "prism/Application.lex.tex", "prism/InteractionModel.lex.tex"].includes(path))) throw refused();
    if (!files.has("prism/spec.json")) return;
    const owned = new Set(["prism/spec.json", "src/lib/prism.ts"]);
    let unknownModule = false;
    try { owned.add(`prism/${lexlean.canonicalSource(JSON.parse(files.get("prism/spec.json"))).name}.lex.tex`); }
    catch { unknownModule = true; }
    if (touched.some(path => owned.has(path) || (unknownModule && /^prism\/[A-Za-z_]\w*\.lex\.tex$/.test(path)))) throw refused();
  } catch (error) {
    if (error?.code === "canonicalGenerationRefused") throw error;
    throw invalid();
  }
}
