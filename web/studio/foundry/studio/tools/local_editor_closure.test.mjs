import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
for (const name of ["schema", "valid", "corrupt", "limits", "staging"]) test(`locked editor ${name} boundary`, () => {
  const output = execFileSync("python3", [fileURLToPath(new URL("./editor_assets_cases.py", import.meta.url)), name], { encoding: "utf8", timeout: 120000 });
  assert.deepEqual(JSON.parse(output), { case: name, passed: true });
});
test("actual loader staging cache and independent browser gates share the declared closure", () => {
  const read = path => fs.readFileSync(new URL(path, import.meta.url), "utf8");
  const loader = read("../vendor/prism/hologram.editor.ts"), build = read("../vendor/prism/vite.shell.config.mts");
  assert.match(loader, /loader.config\(\{ paths: \{ vs: new URL\("editor\/vs", applicationBase\).href \} \}\)/);
  assert.match(loader, /new URL\(import.meta.env.BASE_URL, window.location.origin\)/);
  assert.match(build, /import \$\{JSON.stringify\(editor\)\};/);
  assert.match(read("../scripts/stamp_shell.py"), /stage_editor_assets\(root\)/);
  assert.match(read("../shell/shell-cache.js"), /"editor\/vs\/loader.js", "editor\/package.json"/);
  assert.match(read("../shell/sw.template.js"), /path.startsWith\("editor\/"\)/);
  for (const workflow of ["studio.yml", "pages.yml"]) assert.match(read("../../.github/workflows/" + workflow), /python3 scripts\/editor_assets.py --check/);
  assert.match(read("../../.github/workflows/studio.yml"), /node tools\/editor_browser.mjs/);
});
