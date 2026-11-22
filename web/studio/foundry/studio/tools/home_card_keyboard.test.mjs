import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { after, test } from "node:test";
import { Window } from "../vendor/prism/node_modules/happy-dom/lib/index.js";
import { screenSource } from "../vendor/prism/hologram.screens.mjs";

const require = createRequire(new URL("../vendor/prism/package.json", import.meta.url));
const ts = require("typescript");
const { parse: parseYaml } = require("yaml");
const { act, createElement } = require("react");
const lock = require("./package-lock.json");
for (const name of ["typescript", "react", "react-dom", "happy-dom", "yaml"]) {
  assert.equal(
    require(`${name}/package.json`).version,
    lock.packages[`node_modules/${name}`].version,
  );
}
assert.equal(
  process.execArgv.some((argument) =>
    /^--test-(?:name-pattern|skip-pattern|only)(?:=|$)/.test(argument),
  ),
  false,
  "keyboard acceptance must not be filtered",
);
for (const file of [
  fileURLToPath(import.meta.url),
  fileURLToPath(new URL("./home_card_keyboard_browser.mjs", import.meta.url)),
]) {
  const ast = ts.createSourceFile(
    file,
    fs.readFileSync(file, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.JS,
  );
  const inspect = (node) => {
    if (
      ts.isPropertyAccessExpression(node) &&
      ts.isIdentifier(node.expression) &&
      ["test", "describe", "it"].includes(node.expression.text)
    ) {
      assert.equal(
        ["skip", "only", "todo"].includes(node.name.text),
        false,
        `${path.basename(file)} suppresses test cases`,
      );
    }
    if (ts.isPropertyAssignment(node)) {
      assert.equal(
        ["skip", "only", "todo"].includes(node.name.getText(ast)),
        false,
        `${path.basename(file)} suppresses test cases`,
      );
    }
    ts.forEachChild(node, inspect);
  };
  inspect(ast);
}

// Load the same replacement the Vite renderer uses, then compile its unchanged Card and Keep.
// Only the unused showcase hooks/router and the decorative icon are isolated from this component test.
const upstream = fileURLToPath(
  new URL("../vendor/prism/src/components/FeaturedAppShowcase.tsx", import.meta.url),
);
const source = screenSource(upstream, fs.readFileSync(upstream, "utf8"));
assert.equal(
  source,
  fs
    .readFileSync(
      new URL("../vendor/prism/hologram.screens/FeaturedAppShowcase.tsx", import.meta.url),
      "utf8",
    )
    .replace(/\r\n/g, "\n"),
);
const compiled = ts.transpileModule(`${source}\nexport { Card, Keep };`, {
  fileName: upstream,
  compilerOptions: {
    target: ts.ScriptTarget.ES2022,
    module: ts.ModuleKind.CommonJS,
    jsx: ts.JsxEmit.ReactJSX,
  },
  reportDiagnostics: true,
});
assert.deepEqual(compiled.diagnostics, []);
const module = { exports: {} };
const unusedImports = new Set([
  "@tanstack/react-router",
  "@/hooks/useLoadApps", "@/hooks/useOpenApp",
  "@/hooks/useAppThumbnails", "@/lib/sortApps",
  "../../hologram.featured-apps.mjs",
  // Theater's: the row plays and edits apps, which Card and Keep never reach.
  ...["jotai", "@/atoms/chatAtoms", "@/hooks/useRunApp", "@/hooks/useAppRun", "@/components/ui/button", "@/lib/utils"],
]);
vm.runInNewContext(
  compiled.outputText,
  {
    module,
    exports: module.exports,
    require(name) {
      if (name === "react" || name === "react/jsx-runtime") return require(name);
      if (name === "lucide-react") return { Star: () => null };
      assert.ok(unusedImports.has(name), `unexpected component dependency: ${name}`);
      return {};
    },
  },
  { filename: upstream },
);
const { Card, Keep } = module.exports;

const window = new Window();
globalThis.window = window;
globalThis.document = window.document;
globalThis.IS_REACT_ACT_ENVIRONMENT = true;
const { createRoot } = require("react-dom/client");
after(async () => {
  await window.happyDOM.close();
  delete globalThis.window;
  delete globalThis.document;
  delete globalThis.IS_REACT_ACT_ENVIRONMENT;
});

async function withCard(run) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  let opens = 0,
    toggles = 0;
  await act(async () =>
    root.render(
      createElement(Card, {
        title: "Keyboard regression",
        under: "Test",
        cover: null,
        badge: null,
        onOpen: () => opens++,
        keep: createElement(Keep, {
          on: false,
          label: "Bookmark Keyboard regression",
          onClick: () => toggles++,
        }),
      }),
    ),
  );
  const card = container.querySelector('[role="button"]');
  const keep = container.querySelector('button[aria-label="Bookmark Keyboard regression"]');
  assert.ok(card && keep && card.contains(keep));
  try {
    await run({ card, keep, counts: () => ({ opens, toggles }) });
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
}

async function keyDown(target, key) {
  const event = new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
  await act(async () => target.dispatchEvent(event));
  return event;
}

for (const key of ["Enter", " "]) {
  test(`focused card ${JSON.stringify(key)} opens once and prevents its default`, async () => {
    await withCard(async ({ card, counts }) => {
      card.focus();
      assert.equal(document.activeElement, card);
      assert.equal((await keyDown(card, key)).defaultPrevented, true);
      assert.deepEqual(counts(), { opens: 1, toggles: 0 });
    });
  });

  test(`nested Bookmark ${JSON.stringify(key)} leaves native activation uncancelled and never opens the card`, async () => {
    await withCard(async ({ keep, counts }) => {
      keep.focus();
      assert.equal(document.activeElement, keep);
      assert.equal(
        (await keyDown(keep, key)).defaultPrevented,
        false,
        "the card cancelled the Bookmark key default",
      );
      assert.deepEqual(counts(), { opens: 0, toggles: 0 });
      // Happy DOM does not synthesize a button's native keyboard click. Exercise its real click
      // handler here; the mandatory Chromium journey below proves that native default separately.
      await act(async () => keep.click());
      assert.deepEqual(counts(), { opens: 0, toggles: 1 });
      await act(async () => keep.click());
      assert.deepEqual(counts(), { opens: 0, toggles: 2 });
    });
  });
}

test("unrelated card keys do not activate or cancel the key", async () => {
  await withCard(async ({ card, counts }) => {
    for (const key of ["Tab", "Escape", "ArrowDown", "a"]) {
      assert.equal((await keyDown(card, key)).defaultPrevented, false);
      assert.deepEqual(counts(), { opens: 0, toggles: 0 });
    }
  });
});

test("both component controls and real renderer keyboard acceptance remain mandatory", () => {
  const read = (p) => fs.readFileSync(new URL(p, import.meta.url), "utf8");
  const workflow = read("../../.github/workflows/studio.yml");
  assert.match(workflow, /^\s*run: node --test tools\/home_card_keyboard\.test\.mjs\s*$/m);
  assert.match(workflow, /^\s*run node tools\/home_card_keyboard_browser\.mjs\s*$/m);
  const lane = parseYaml(workflow).jobs.lane;
  assert.equal(lane.if, undefined);
  assert.equal(lane["continue-on-error"], undefined);
  const prerequisites =
    "${{ !cancelled() && steps.shell_closure.conclusion == 'success' && steps.corpus.conclusion == 'success' }}";
  for (const [filename, command] of [
    ["home_card_keyboard.test.mjs", "node --test tools/home_card_keyboard.test.mjs"],
    ["home_card_keyboard_browser.mjs", "run node tools/home_card_keyboard_browser.mjs"],
  ]) {
    const steps = lane.steps.filter((step) => step.run?.includes(filename));
    assert.equal(steps.length, 1, `${filename} must have exactly one mandatory step`);
    const step = steps[0];
    assert.equal(step.if, prerequisites);
    assert.equal(step["continue-on-error"], undefined);
    if (filename.endsWith(".test.mjs")) assert.equal(step.run, command);
    else {
      assert.ok(step.run.split("\n").includes(command));
      assert.ok(step.run.includes('run() { "$@" || status=1; }'));
      assert.ok(step.run.trimEnd().endsWith('exit "$status"'));
    }
  }
  assert.ok(
    read("../experience/src/FoundryExperience.lex.tex").includes("name: homeIsAWorkspace,"),
  );
  const browser = read("./home_card_keyboard_browser.mjs");
  for (const required of [
    "await keep.press(key)",
    "await card.press(key)",
    "localStorage.getItem",
    "bookmarkOpens",
    "cardOpens",
  ]) {
    assert.ok(browser.includes(required), `real renderer keyboard control missing: ${required}`);
  }
  for (const file of [
    fileURLToPath(import.meta.url),
    fileURLToPath(new URL("./home_card_keyboard_browser.mjs", import.meta.url)),
  ]) {
    const gate = fs.readFileSync(file, "utf8");
    assert.equal(
      /\b(?:test|describe|it)\.(?:skip|only)\s*\(/.test(gate),
      false,
      path.basename(file),
    );
  }
});
