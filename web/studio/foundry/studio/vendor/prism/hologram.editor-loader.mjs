// FoundrySpecification.editorLoadLifecycleBoundary. Exact locked loader 1.5.0 source.
import { createHash } from "node:crypto";
const hashes = {
  es: "1b18d3d000702c9f457b3cfdb0cdb22c2c57ed521afbc4adc435dffbe2311441",
  cjs: "18751fee26ddf6c90a47e9f0e5a31af7ab23cd783958801e90578620cdb71f9e",
};
export function editorLoaderSource(id, source) {
  const match = id
    .replace(/\\/g, "/")
    .match(/\/node_modules\/@monaco-editor\/loader\/lib\/(es|cjs)\/utils\/makeCancelable\.js$/);
  if (!match) return null;
  if (createHash("sha256").update(source).digest("hex") !== hashes[match[1]])
    throw Object.assign(new Error("The locked editor loader projection changed"), {
      code: "editorLoaderProjectionMismatch",
    });
  // One then owns both paths. The upstream success-only then detached a rejected
  // promise even though a separate catch settled the caller's wrapped promise.
  return source.replace('    });\n    promise["catch"](reject);', "    }, reject);");
}
