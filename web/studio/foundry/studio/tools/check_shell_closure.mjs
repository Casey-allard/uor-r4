// Independent staged-file inventory, followed by the actual worker byte verifier.
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
await import("../shell/shell-cache.js");
export async function checkStagedShell(directory) {
  const excluded = new Set(["manifest.json", "provenance.json", "warmup.json", "sw.js", "sw.template.js"]), paths = [];
  const walk = async (dir, prefix = "") => {
    for (const entry of await fs.readdir(dir, { withFileTypes: true })) {
      const relative = prefix + entry.name;
      if (entry.name === "warmup.json") throw Object.assign(new Error("Public credential artifact refused"), {code:"providerCredentialArtifact"});
      if (entry.isSymbolicLink()) throw Error("Staged shell contains a symbolic link");
      if (relative === "app" || excluded.has(relative)) continue;
      if (entry.isDirectory()) await walk(path.join(dir, entry.name), relative + "/");
      else if (entry.isFile()) paths.push(relative);
      else throw Error("Staged shell contains a non-file entry");
    }
  };
  await walk(directory); paths.sort();
  const manifest = JSON.parse(await fs.readFile(path.join(directory, "manifest.json"), "utf8"));
  if (JSON.stringify(paths) !== JSON.stringify(manifest.files.map(row => row.path))) throw Error("Shell manifest does not name the complete staged inventory in byte order");
  const base = "https://staged.invalid/";
  const entries = await globalThis.ShellClosure.prepare(base, manifest.closure, { fetchFn: async url => {
    if (url.origin !== new URL(base).origin) throw Error("Unexpected shell origin");
    return new Response(await fs.readFile(path.join(directory, decodeURIComponent(url.pathname.slice(1)))));
  } });
  return { closure: manifest.closure, files: paths.length, responses: entries.length, bytes: manifest.files.reduce((n, row) => n + row.bytes, 0) };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) console.log(JSON.stringify(await checkStagedShell(process.argv[2] || fileURLToPath(new URL("../shell/", import.meta.url)))));
