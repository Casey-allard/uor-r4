// Imported compiler adapter; artifact identity belongs to the verified shell.
export const compilerFailure = () => Object.assign(new Error("The browser compiler could not initialize"), { code: "browserCompilerUnavailable" });
export function createCompilerLoader(importModule = url => import(url)) {
  let ready = null;
  return base => {
    if (!ready) ready = Promise.resolve().then(async () => {
      const root = new URL("compiler/", base);
      const compiler = await importModule(new URL("esm/browser.min.js", root).href);
      await compiler.initialize({ wasmURL: new URL("esbuild.wasm", root).href });
      return compiler;
    }).catch(() => { ready = null; throw compilerFailure(); });
    return ready;
  };
}
export const loadBrowserCompiler = createCompilerLoader();
