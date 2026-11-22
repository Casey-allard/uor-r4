// Foundry's reader, watched by browser suites. The reader is IdeaReader, compiled with Prism into the
// kernel guest the shell already runs; a suite does not stand in for it. `deviceSeam` (an init script)
// wraps the loaded kernel's `ask` so a suite sees each idea it is asked to read and can hold the answer,
// keeping a turn in flight while the guest stays free for every other request.
/** The selection a suite stores: Foundry's reader, the one model Foundry offers. */
export const DEVICE_MODEL = { provider: "local", name: "foundry:reader" };
/** Install before the page loads: `await context.addInitScript(deviceSeam)`. */
export const deviceSeam = () => {
  const f = (globalThis.__deviceFixture ??= { asked: [] }); let kernel;
  const watch = (k) => Object.freeze({ ...k, ask: async (line) => {
    if (typeof line === "string" && line.startsWith("read ")) { f.asked.push(line.slice(5)); if (f.hold) await f.hold; }
    return k.ask(line);
  } });
  Object.defineProperty(globalThis, "ForgeKernel", { configurable: true, get: () => kernel, set: (v) => { kernel = Object.freeze({ ...v, load: async (...a) => watch(await v.load(...a)) }); } });
};
/** Hold the reader's next readings until `deviceRelease`. */
export const deviceHold = (page) => page.evaluate(() => { const f = (globalThis.__deviceFixture ??= { asked: [] }); f.hold = new Promise((r) => (f.release = r)); });
export const deviceRelease = (page) => page.evaluate(() => { const f = globalThis.__deviceFixture; const r = f?.release; if (f) { f.hold = null; f.release = null; } r?.(); });
/** What the reader was asked to read, one line per chat turn, since the page loaded. */
export const deviceAsked = (page) => page.evaluate(() => (globalThis.__deviceFixture?.asked ?? []).slice());
