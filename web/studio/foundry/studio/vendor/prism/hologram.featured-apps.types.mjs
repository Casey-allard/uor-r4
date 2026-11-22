// The shell's JavaScript is the authority. Keep the renderer's declarations compiler-derived,
// rather than maintaining another description of the featured cards beside their runtime data.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import ts from "typescript";

const studio = fileURLToPath(new URL("../../", import.meta.url));
const sources = ["shell/featured-apps.mjs", "vendor/prism/hologram.featured-apps.mjs"];
const lock = JSON.parse(fs.readFileSync(new URL("./package-lock.json", import.meta.url), "utf8"));
if (ts.version !== lock.packages["node_modules/typescript"].version) {
  throw new Error("featured app declarations require the lockfile's TypeScript compiler");
}

export function generateFeaturedAppTypes(root = studio) {
  root = path.resolve(root);
  const declarations = new Set(
    sources.map((source) => path.join(root, source.replace(/\.mjs$/, ".d.mts"))),
  );
  const outDir = path.join(root, ".featured-types");
  const options = {
    allowJs: true,
    checkJs: true,
    strict: true,
    declaration: true,
    emitDeclarationOnly: true,
    noEmitOnError: true,
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    newLine: ts.NewLineKind.LineFeed,
    types: [],
    rootDir: root,
    outDir,
  };
  const host = ts.createCompilerHost(options);
  const fileExists = host.fileExists.bind(host);
  // A stale declaration must never shadow the JavaScript whose types we are regenerating.
  host.fileExists = (file) => !declarations.has(path.resolve(file)) && fileExists(file);
  const generated = new Map();
  host.writeFile = (file, text) =>
    generated.set(path.resolve(root, path.relative(outDir, file)), text);
  const program = ts.createProgram(
    sources.map((source) => path.join(root, source)),
    options,
    host,
  );
  const emitted = program.emit();
  const errors = [...ts.getPreEmitDiagnostics(program), ...emitted.diagnostics].filter(
    (diagnostic) => diagnostic.category === ts.DiagnosticCategory.Error,
  );
  if (emitted.emitSkipped || errors.length) {
    throw new Error(
      "featured app declarations refused:\n" +
        ts.formatDiagnostics(errors, {
          getCanonicalFileName: (file) => file,
          getCurrentDirectory: () => root,
          getNewLine: () => "\n",
        }),
    );
  }
  if (
    generated.size !== declarations.size ||
    [...declarations].some((file) => !generated.has(file))
  ) {
    throw new Error(
      "featured app declarations refused: compiler did not emit the exact declaration pair",
    );
  }
  return generated;
}

export function checkFeaturedAppTypes(root = studio) {
  for (const [file, expected] of generateFeaturedAppTypes(root)) {
    const name = path.relative(root, file).replace(/\\/g, "/");
    if (!fs.existsSync(file))
      throw new Error(
        `missing featured app declaration: ${name}; run node hologram.featured-apps.types.mjs --write from studio/vendor/prism`,
      );
    if (fs.readFileSync(file, "utf8") !== expected)
      throw new Error(
        `stale featured app declaration: ${name}; run node hologram.featured-apps.types.mjs --write from studio/vendor/prism`,
      );
  }
}

if (process.argv[1] && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url) {
  const mode = process.argv.slice(2);
  if (mode.length !== 1 || !["--write", "--check"].includes(mode[0])) {
    throw new Error("usage: node hologram.featured-apps.types.mjs --write|--check");
  }
  if (mode[0] === "--write") {
    for (const [file, text] of generateFeaturedAppTypes()) fs.writeFileSync(file, text);
  } else {
    checkFeaturedAppTypes();
  }
  console.log(
    `featured app declarations: ${mode[0] === "--write" ? "written" : "current"} from JavaScript with locked TypeScript ${ts.version}`,
  );
}
