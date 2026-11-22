// FoundrySpecification.canonicalBuildBoundary: byte identity does not establish model coherence.
import { checkSpec } from "./prism.js";
import { canonicalSource } from "./live/lexlean.mjs";
import {applicationShape, shapeFailure} from './application-shape.mjs';
const fail = () => Object.assign(new Error("Build source does not match its canonical model projections"), { code: "canonicalBuildMismatch" });
export function assertCanonicalBuild(rows) {
  if (applicationShape({files:rows}) === 'stateful') throw shapeFailure('statefulBuildUnsupported');
  try {
    const files = new Map(rows);
    if (!files.has("prism/spec.json")) return;
    const text = files.get("prism/spec.json");
    if (typeof text !== "string" || text.length > 1024 * 1024 || new TextEncoder().encode(text).length > 1024 * 1024) throw fail();
    const checked = checkSpec(JSON.parse(text));
    if (checked.ok !== true) throw fail();
    const canonical = canonicalSource(checked.spec);
    if (files.get(`prism/${canonical.name}.lex.tex`) !== canonical.source || files.get("src/lib/prism.ts") !== checked.module) throw fail();
  } catch { throw fail(); }
}
