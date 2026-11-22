"""Extract the renderer's text tag build prompt into shell/prompts.js, verbatim (Apache 2.0).

The constants BUILD_SYSTEM_PREFIX, BUILD_SYSTEM_POSTFIX and DEFAULT_AI_RULES in
vendor/prism/src/prompts/system_prompt.ts teach the model the <prism-write> family the renderer already
renders. They are template literals without interpolations, so they are copied as JSON strings.
Run from the project root: python3 scripts/extract_prompts.py
"""
import json, pathlib, re

root = pathlib.Path(__file__).resolve().parent.parent
src = (root / "vendor" / "prism" / "src" / "prompts" / "system_prompt.ts").read_text(encoding="utf-8")


def const(name):
    m = re.search(r"(?:export )?const " + name + r" = `([\s\S]*?)`;", src)
    assert m, name
    value = m.group(1)
    assert "${" not in value, name + " has an interpolation"
    return value


prefix, postfix, rules = const("BUILD_SYSTEM_PREFIX"), const("BUILD_SYSTEM_POSTFIX"), const("DEFAULT_AI_RULES")
lines = [
    "// the renderer's build prompt for the text tag path (Apache 2.0, dyad-sh/dyad 00d5f5af7fd0,",
    "// src/prompts/system_prompt.ts: BUILD_SYSTEM_PREFIX, BUILD_SYSTEM_POSTFIX, DEFAULT_AI_RULES), extracted",
    "// verbatim by scripts/extract_prompts.py so the model emits <prism-write> blocks the renderer already renders.",
    "export const BUILD_SYSTEM_PREFIX = " + json.dumps(prefix, ensure_ascii=False) + ";",
    "export const BUILD_SYSTEM_POSTFIX = " + json.dumps(postfix, ensure_ascii=False) + ";",
    "export const DEFAULT_AI_RULES = " + json.dumps(rules, ensure_ascii=False) + ";",
    'export const buildSystemPrompt = (aiRules) => BUILD_SYSTEM_PREFIX + "\\n\\n" + (aiRules || DEFAULT_AI_RULES) + "\\n\\n" + BUILD_SYSTEM_POSTFIX;',
    "",
]
(root / "shell" / "prompts.js").write_text("\n".join(lines), encoding="utf-8", newline="\n")
print("wrote shell/prompts.js")
