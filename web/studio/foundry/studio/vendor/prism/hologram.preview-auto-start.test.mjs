import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  shouldAutoStartBrowserPreview,
  PREVIEW_AUTO_START_PATCHES,
} from "./hologram.preview-auto-start.mjs";
const initial = { appId: 7, phase: "idle", operation: null, startedAt: null, lastSettlement: null };
test("only a matching initial idle snapshot authorizes automatic startup", () => {
  assert.equal(shouldAutoStartBrowserPreview(initial, 7), true);
  for (const id of [null, undefined, 0, -1, 8, NaN, Infinity, 1.5, Number.MAX_SAFE_INTEGER + 1])
    assert.equal(shouldAutoStartBrowserPreview(initial, id), false);
  for (const id of [1.5, Number.MAX_SAFE_INTEGER + 1])
    assert.equal(shouldAutoStartBrowserPreview({ ...initial, appId: id }, id), false);
  for (const state of [null, undefined, {}, { ...initial, appId: 8 }])
    assert.equal(shouldAutoStartBrowserPreview(state, 7), false);
});
test("active stopped ready and errored snapshots never automatically retry", () => {
  for (const phase of ["starting", "ready", "reloading", "stopped", "errored", "unknown"])
    assert.equal(shouldAutoStartBrowserPreview({ ...initial, phase }, 7), false);
  for (const delta of [
    { operation: "run" },
    { startedAt: 0 },
    { startedAt: 123 },
    { lastSettlement: { outcome: "failed" } },
  ])
    assert.equal(shouldAutoStartBrowserPreview({ ...initial, ...delta }, 7), false);
});
test("actual preview patch reads imported run state and guards only automatic startup", () => {
  const root = new URL("./", import.meta.url),
    p = "src/components/preview_panel/PreviewPanel.tsx";
  let source = fs.readFileSync(new URL(p, root), "utf8");
  assert.equal(PREVIEW_AUTO_START_PATCHES.length, 4);
  for (const patch of PREVIEW_AUTO_START_PATCHES) {
    assert.equal(patch.file, p);
    assert.equal(source.split(patch.from).length, 2);
    source = source.replace(patch.from, patch.to);
  }
  assert.ok(source.includes("const browserRunState = useAppRunState(selectedAppId);"));
  assert.ok(
    source.includes(
      "const autoStartBrowserPreview = shouldAutoStartBrowserPreview(browserRunState, selectedAppId);",
    ),
  );
  assert.ok(source.includes("if (autoStartBrowserPreview)"));
  assert.ok(source.includes("nodeVersion, autoStartBrowserPreview]"));
  assert.equal(
    (source.match(/runAppLifecycleInBackground\("start", runApp\(selectedAppId\)\);/g) || [])
      .length,
    1,
  );
  assert.ok(
    fs.readFileSync(new URL("src/hooks/useRunApp.ts", root), "utf8").includes('type: "START"'),
  );
});
test("mandatory renderer model and browser gates include the state guard", () => {
  const read = (p) => fs.readFileSync(new URL("../../../" + p, import.meta.url), "utf8");
  assert.ok(
    read("studio/vendor/prism/hologram.screens.mjs").includes("...PREVIEW_AUTO_START_PATCHES,"),
  );
  assert.ok(
    read(".github/workflows/studio.yml").includes("node tools/preview_auto_start_browser.mjs"),
  );
  assert.ok(
    read("app/src/FoundrySpecification.lex.tex").includes("name: previewAutoStartBoundary,"),
  );
});
