import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { parse } from "yaml";
import {
  checkFeaturedAppTypes,
  generateFeaturedAppTypes,
} from "./hologram.featured-apps.types.mjs";

const studio = fileURLToPath(new URL("../../", import.meta.url));
const sources = ["shell/featured-apps.mjs", "vendor/prism/hologram.featured-apps.mjs"];

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "foundry-featured-types-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const source of sources) {
    fs.mkdirSync(path.dirname(path.join(root, source)), { recursive: true });
    fs.copyFileSync(path.join(studio, source), path.join(root, source));
  }
  return root;
}

function writeGenerated(root) {
  const generated = generateFeaturedAppTypes(root);
  for (const [file, text] of generated) fs.writeFileSync(file, text);
  return generated;
}

test("the committed declaration pair is the deterministic output of the actual JavaScript exports", () => {
  const first = generateFeaturedAppTypes(studio);
  assert.deepEqual(generateFeaturedAppTypes(studio), first);
  assert.equal(first.size, 2);
  assert.match(first.get(path.join(studio, "shell/featured-apps.d.mts")), /hue: number;/);
  assert.doesNotMatch([...first.values()].join("\n"), /\bany\b|@ts-ignore|@ts-nocheck/);
  checkFeaturedAppTypes(studio);
});

test("either missing declaration is refused without rewriting it", (t) => {
  const root = fixture(t);
  for (const [file, text] of writeGenerated(root)) {
    fs.unlinkSync(file);
    assert.throws(() => checkFeaturedAppTypes(root), /missing featured app declaration/);
    assert.equal(fs.existsSync(file), false);
    fs.writeFileSync(file, text);
  }
  checkFeaturedAppTypes(root);
});

test("either stale declaration is refused and cannot influence regeneration", (t) => {
  const root = fixture(t);
  const expected = writeGenerated(root);
  for (const [file, text] of expected) {
    fs.writeFileSync(file, "this is not a TypeScript declaration\n");
    assert.deepEqual(generateFeaturedAppTypes(root), expected);
    assert.throws(() => checkFeaturedAppTypes(root), /stale featured app declaration/);
    assert.equal(fs.readFileSync(file, "utf8"), "this is not a TypeScript declaration\n");
    fs.writeFileSync(file, text);
  }
  checkFeaturedAppTypes(root);
});

test("changes to the actual exported shape make its previous declaration stale", (t) => {
  const root = fixture(t);
  writeGenerated(root);
  fs.appendFileSync(
    path.join(root, sources[0]),
    "\nexport const declarationProbe = { enabled: true };\n",
  );
  assert.throws(
    () => checkFeaturedAppTypes(root),
    /stale featured app declaration: shell\/featured-apps.d.mts/,
  );
  const generated = writeGenerated(root);
  assert.match(generated.get(path.join(root, "shell/featured-apps.d.mts")), /declarationProbe/);
  checkFeaturedAppTypes(root);
});

test("a broken re-export is refused instead of emitting an untyped escape", (t) => {
  const root = fixture(t);
  fs.writeFileSync(
    path.join(root, sources[1]),
    'export { absent } from "../../shell/featured-apps.mjs";\n',
  );
  assert.throws(() => generateFeaturedAppTypes(root), /TS2305/);
});

test("Studio requires the full projected typecheck and declaration controls after verified prerequisites", () => {
  const source = fs.readFileSync(
    new URL("../../../.github/workflows/studio.yml", import.meta.url),
    "utf8",
  );
  const guard =
    "${{ !cancelled() && steps.shell_closure.conclusion == 'success' && steps.corpus.conclusion == 'success' }}";
  const checks = [
    [
      "Verify source-derived featured app declarations and negative controls",
      "node --test vendor/prism/hologram.featured-apps.types.test.mjs",
    ],
    [
      "Typecheck every projected renderer screen against the locked compiler",
      "node vendor/prism/hologram.typecheck.mjs",
    ],
  ];
  const audit = (document) => {
    const workflow = parse(document);
    assert.ok(workflow.on.pull_request.paths.includes("studio/**"));
    assert.ok(workflow.on.pull_request.paths.includes(".github/workflows/studio.yml"));
    const lane = workflow.jobs.lane;
    assert.equal(lane["continue-on-error"], undefined);
    assert.equal(lane.if, undefined);
    assert.equal(lane.defaults.run["working-directory"], "studio");
    for (const [name, command] of checks) {
      const matches = lane.steps.filter((step) => step.name === name);
      assert.equal(matches.length, 1, `missing or duplicate mandatory step: ${name}`);
      const step = matches[0];
      assert.deepEqual(
        Object.keys(step).sort(),
        ["if", "name", "run"],
        "no step-level bypass or alternate working directory",
      );
      assert.equal(step.if, guard, "verified prerequisites cannot be skipped or bypassed");
      assert.equal(
        step.run,
        command,
        "mandatory invocation cannot be filtered, changed or made fail-open",
      );
      for (const id of ["shell_closure", "corpus"]) {
        const prerequisite = lane.steps.findIndex((candidate) => candidate.id === id);
        assert.ok(
          prerequisite >= 0 && prerequisite < lane.steps.indexOf(step),
          "prerequisites precede the gate",
        );
      }
    }
  };
  audit(source);
  for (const [name, command] of checks) {
    assert.throws(
      () => audit(source.replace(name, "omitted mandatory check")),
      /missing or duplicate/,
    );
    assert.throws(
      () =>
        audit(
          source.replace(`run: ${command}`, `continue-on-error: true\n        run: ${command}`),
        ),
      /step-level bypass/,
    );
    assert.throws(
      () => audit(source.replace(command, `${command} || true`)),
      /mandatory invocation/,
    );
    assert.throws(
      () => audit(source.replace(command, "node --test wrong-file.test.mjs")),
      /mandatory invocation/,
    );
  }
  assert.throws(() => audit(source.replaceAll(guard, "${{ false }}")), /prerequisites cannot/);
  assert.throws(
    () =>
      audit(
        source.replace(
          checks[0][1],
          "node --test --test-name-pattern=deterministic vendor/prism/hologram.featured-apps.types.test.mjs",
        ),
      ),
    /mandatory invocation/,
  );
});

test("the production typecheck invokes freshness unconditionally", () => {
  const source = fs.readFileSync(new URL("./hologram.typecheck.mjs", import.meta.url), "utf8");
  const audit = (text) => {
    const tree = ts.createSourceFile(
      "hologram.typecheck.mjs",
      text,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.JS,
    );
    const imported = tree.statements.find(
      (statement) =>
        ts.isImportDeclaration(statement) &&
        statement.moduleSpecifier.text === "./hologram.featured-apps.types.mjs",
    );
    assert.ok(
      imported?.importClause?.namedBindings?.elements.some(
        (element) => element.name.text === "checkFeaturedAppTypes",
      ),
      "freshness function must be imported from its real implementation",
    );
    const calls = tree.statements.filter(
      (statement) =>
        ts.isExpressionStatement(statement) &&
        ts.isCallExpression(statement.expression) &&
        statement.expression.expression.getText(tree) === "checkFeaturedAppTypes",
    );
    assert.equal(calls.length, 1, "one unconditional freshness invocation is required");
    assert.equal(calls[0].expression.arguments.length, 0, "the gate must check production files");
    assert.ok(
      calls[0].pos < text.indexOf('program("tsconfig.hologram.prism.json"'),
      "freshness must run before the comparison",
    );
  };
  audit(source);
  assert.throws(() => audit(source.replace("checkFeaturedAppTypes();", "")), /unconditional/);
  assert.throws(
    () => audit(source.replace("checkFeaturedAppTypes();", "if (false) checkFeaturedAppTypes();")),
    /unconditional/,
  );
  assert.throws(
    () =>
      audit(
        source.replace("checkFeaturedAppTypes();", 'checkFeaturedAppTypes("wrong-directory");'),
      ),
    /production files/,
  );
});

// Run the integrity guard outside any test so skipping its own case cannot disable it.
assert.ok(
  !process.execArgv.some((arg) => /^--test-(?:name-pattern|skip-pattern|only)(?:=|$)/.test(arg)),
  "the mandatory declaration suite cannot be filtered",
);
const tests = ts.createSourceFile(
  "types.test.mjs",
  fs.readFileSync(fileURLToPath(import.meta.url), "utf8"),
  ts.ScriptTarget.Latest,
  true,
  ts.ScriptKind.JS,
);
const cases = tests.statements.filter(
  (statement) =>
    ts.isExpressionStatement(statement) &&
    ts.isCallExpression(statement.expression) &&
    statement.expression.expression.getText(tests) === "test",
);
assert.equal(
  cases.length,
  7,
  "the complete suite must register all seven controls unconditionally",
);
for (const statement of cases) {
  assert.equal(statement.expression.arguments.length, 2, "skip/only/todo options are forbidden");
  assert.ok(ts.isStringLiteral(statement.expression.arguments[0]));
  assert.ok(ts.isArrowFunction(statement.expression.arguments[1]));
}
const inspect = (node) => {
  if (ts.isCallExpression(node) && ts.isPropertyAccessExpression(node.expression)) {
    assert.ok(
      !["skip", "only", "todo"].includes(node.expression.name.text),
      "test filtering is forbidden",
    );
  }
  ts.forEachChild(node, inspect);
};
inspect(tests);
