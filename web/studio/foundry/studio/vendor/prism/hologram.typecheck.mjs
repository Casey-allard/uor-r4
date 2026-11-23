// hologram-forge (ours, not upstream's; see VENDORED.md): typechecks the renderer as the shell builds it, every
// screen rewritten for Radix (hologram.radix.mjs) and every ui component the kit has read from the vendored
// new-york-v4 registry (tsconfig.hologram.json), against upstream's own sources as they are. Upstream
// carries type errors of its own, so the gate is relative: it refuses an error the shell's build has that
// Upstream's does not, and a call the rewrite could not convert. Run from vendor/prism: node hologram.typecheck.mjs
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { toRadix } from "./hologram.radix.mjs";
import { screenSource, unapplied } from "./hologram.screens.mjs";
import { checkFeaturedAppTypes } from "./hologram.featured-apps.types.mjs";

checkFeaturedAppTypes();

const here = path.dirname(fileURLToPath(import.meta.url));
// Files the shell's build replaces with ours (vite.shell.config.mts resolveId): their errors do not ship.
const REPLACED = new Set(["src/components/ChatModeSelector.tsx", "src/components/ModelPicker.tsx", "src/components/ProviderSettings.tsx", "src/components/GitHubIntegration.tsx", "src/components/GitHubConnector.tsx", "src/components/ImportAppDialog.tsx",
  "src/components/preview_panel/DeploymentSection.tsx", "src/components/VercelIntegration.tsx", "src/components/SupabaseIntegration.tsx", "src/components/NeonIntegration.tsx",
  "src/components/SupabaseConnector.tsx", "src/components/NeonConnector.tsx", "src/components/preview_panel/NeonConfigure.tsx"]);
const slash = (p) => p.replace(/\\/g, "/");
const src = slash(path.join(here, "src")) + "/";

function program(configName, rewrite) {
  const cfgPath = path.join(here, configName);
  const cfg = ts.readConfigFile(cfgPath, ts.sys.readFile);
  const parsed = ts.parseJsonConfigFileContent(cfg.config, ts.sys, here, undefined, cfgPath);
  const host = ts.createCompilerHost(parsed.options);
  const problems = [];
  if (rewrite) {
    const get = host.getSourceFile.bind(host);
    host.getSourceFile = (file, lang, onError, fresh) => {
      if (slash(file).startsWith(src) && /\.ts$/.test(file) && !file.endsWith(".d.ts")) {
        const text = ts.sys.readFile(file);
        if (text !== undefined) return ts.createSourceFile(file, screenSource(file, text), lang, true);
      }
      if (slash(file).startsWith(src) && /\.tsx$/.test(file)) {
        const text = ts.sys.readFile(file);
        if (text !== undefined && slash(file).includes("/components/ui/")) return ts.createSourceFile(file, screenSource(file, text), lang, true);
        if (text !== undefined) {
          const r = toRadix(screenSource(file, text), slash(path.relative(here, file)));
          if (!REPLACED.has(slash(path.relative(here, file)))) problems.push(...r.problems);
          return ts.createSourceFile(file, r.code, lang, true);
        }
      }
      return get(file, lang, onError, fresh);
    };
  }
  const p = ts.createProgram({ rootNames: parsed.fileNames, options: { ...parsed.options, noEmit: true }, host });
  const errors = ts.getPreEmitDiagnostics(p).filter((d) => d.category === ts.DiagnosticCategory.Error);
  return { errors, problems };
}

const shipped = (d) => !d.file || !REPLACED.has(slash(path.relative(here, d.file.fileName)));
const key = (d) => `${d.file ? slash(path.relative(here, d.file.fileName)) : "-"} TS${d.code}`;
const show = (d) => {
  const where = d.file ? `${slash(path.relative(here, d.file.fileName))}:${d.file.getLineAndCharacterOfPosition(d.start ?? 0).line + 1}` : "-";
  return `${where} TS${d.code} ${ts.flattenDiagnosticMessageText(d.messageText, "\n").split("\n")[0].slice(0, 200)}`;
};

const t0 = performance.now();
const prism = program("tsconfig.hologram.prism.json", false);
const shell = program("tsconfig.hologram.json", true);
const budget = new Map();
for (const d of prism.errors.filter(shipped)) budget.set(key(d), (budget.get(key(d)) ?? 0) + 1);
const extra = [];
for (const d of shell.errors.filter(shipped)) {
  const k = key(d);
  if ((budget.get(k) ?? 0) > 0) budget.set(k, budget.get(k) - 1);
  else extra.push(d);
}
const ms = Math.round(performance.now() - t0);
// Every patch must have met its file in the program; one that did not was never checked.
for (const m of unapplied()) shell.problems.push("patch never applied: " + m);
if (extra.length || shell.problems.length) {
  console.log(`typecheck refused: ${extra.length} errors the shell's build adds, ${shell.problems.length} calls not rewritten`);
  for (const p of shell.problems) console.log("  not rewritten: " + p);
  for (const d of extra) console.log("  " + show(d));
  process.exit(1);
}
console.log(`typecheck: the shell's build adds no type error to upstream's own (${prism.errors.length}), every ui call is rewritten for Radix (${ms} ms)`);
