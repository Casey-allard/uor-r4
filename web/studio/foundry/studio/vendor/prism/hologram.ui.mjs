// hologram-forge (ours, not upstream's; see VENDORED.md): the renderer's ui components are shadcn/ui new-york-v4,
// read verbatim from the one vendored registry (vendor/shadcn-ui, written by tools/live_components.mjs from
// the Hologram brand kit, digests in VENDORED.sha256). Upstream's own copies under src/components/ui are shadcn
// on Base UI; every import of a name the kit has resolves to the kit's file instead, so those copies never
// enter the bundle and updating the components is one --vendor run, with nothing to edit here.
//
// The vite config (vite.shell.config.mts) and the gate both use this module, so the gate inspects the
// mapping the build actually applies.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const slash = (p) => p.replace(/\\/g, "/");
export const rendererSrc = slash(path.join(here, "src"));
export const registry = slash(path.resolve(here, "..", "shadcn-ui", "registry"));
export const kitUi = `${registry}/new-york-v4/ui`;
const kitRoot = `${registry}/new-york-v4`;

const firstFile = (base) => [".tsx", ".ts"].map((e) => base + e).find((f) => fs.existsSync(f) && fs.statSync(f).isFile()) ?? null;

/** The kit's file for a ui name (`button`), or null when the kit has no such component. */
export const kitFile = (name) => firstFile(`${kitUi}/${name}`);

/** Where an import resolves, or null to leave it to Vite. `@/components/ui/<name>` (and a relative
 * `./<name>` between upstream's ui files) is the kit's file whenever the kit has that name; the kit's own
 * `@/registry/new-york-v4/...` imports stay inside the registry. */
export function resolveUi(id, importer) {
  const from = importer ? slash(importer) : "";
  for (const p of ["@/registry/new-york-v4/", `${rendererSrc}/registry/new-york-v4/`]) {
    if (slash(id).startsWith(p)) return firstFile(`${kitRoot}/${slash(id).slice(p.length)}`);
  }
  let name = null;
  if (id.startsWith("@/components/ui/")) name = id.slice("@/components/ui/".length);
  else if (/^\.\/[^/]+$/.test(id) && from.startsWith(`${rendererSrc}/components/ui/`)) name = id.slice(2);
  else if (from.startsWith(rendererSrc) && /(^|\/)components\/ui\/[^/]+$/.test(id) && id.startsWith(".")) name = id.split("/").pop();
  // Vite's own `@` alias (upstream's config) runs first and hands over the absolute path.
  else if (slash(id).startsWith(`${rendererSrc}/components/ui/`)) name = slash(id).slice(`${rendererSrc}/components/ui/`.length);
  if (name) name = name.replace(/\.tsx?$/, "");
  if (!name) return null;
  return kitFile(name);
}

/** What refuses a bundle: An upstream copy of a component the kit has, or any Base UI module. Each problem
 * names the module. */
export function audit(moduleIds, importersOf = () => []) {
  const bad = [];
  const by = (id) => { const i = importersOf(id); return i.length ? ` (imported by ${i.map(slash).join(", ")})` : ""; };
  for (const raw of moduleIds) {
    const id = slash(raw).split("?")[0];
    if (/\/node_modules\/@base-ui\//.test(id)) bad.push(`Base UI is in the bundle: ${id}${by(raw)}`);
    const m = id.startsWith(`${rendererSrc}/components/ui/`) && id.match(/\/components\/ui\/([^/]+)\.tsx?$/);
    if (m && kitFile(m[1])) bad.push(`a hand-rolled copy of the kit's ${m[1]} is in the bundle: ${id}${by(raw)}`);
  }
  return bad;
}
