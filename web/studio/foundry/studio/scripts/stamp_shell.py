"""Place the renderer's built output beside the shell, and stage the scaffold beside it.

The homepage is shell/index.html, projected by the core from the verified View; it references the
renderer's assets by fixed names (vendor/prism/vite.shell.config.mts, ours). This script copies the Vite output
under shell/assets/, writes shell/404.html as a copy of the homepage so Pages answers a refresh on any
of the renderer's routes with the app (the router reads the URL), and stages the renderer's scaffold with a file list,
which the host creates apps from. Run after the Vite build: python3 scripts/stamp_shell.py
"""
import json, pathlib, re, shutil, subprocess

root = pathlib.Path(__file__).resolve().parent.parent
dist = root / "vendor" / "prism" / "dist" / "assets"
assets = root / "shell" / "assets"
if not dist.is_dir():
    raise SystemExit(f"{dist} is missing: run the Vite build first (see vendor/prism/vite.shell.config.mts)")
if assets.exists():
    shutil.rmtree(assets)
shutil.copytree(dist, assets)
# The app covers' capture library (shell/covers.mjs): the renderer's own locked html-to-image, the build upstream
# injects into previews too, placed beside the renderer so the host can add it to each preview it builds.
library = root / "vendor" / "prism" / "node_modules" / "html-to-image" / "dist" / "html-to-image.js"
if not library.is_file():
    raise SystemExit(f"{library} is missing: run npm ci in vendor/prism first")
shutil.copy(library, assets / "html-to-image.js")
for stale in ("app",):
    if (root / "shell" / stale).exists():
        shutil.rmtree(root / "shell" / stale)
index = (root / "shell" / "index.html").read_text(encoding="utf-8")
(root / "shell" / "404.html").write_text(index, encoding="utf-8", newline="\n")
print(f"placed {sum(1 for p in assets.rglob('*') if p.is_file())} renderer files under shell/assets and wrote shell/404.html")

src = root / "vendor" / "prism" / "scaffold"
dst = root / "shell" / "scaffold"
if dst.exists():
    shutil.rmtree(dst)
shutil.copytree(src / "src", dst / "src")
for name in ("package.json", "tailwind.config.ts", "index.html", "AI_RULES.md"):
    if (src / name).exists():
        shutil.copy(src / name, dst / name)
# The product is Prism Foundry: the scaffold's badge links to the Foundry, not to the upstream site.
for p in dst.rglob("*"):
    if p.is_file() and p.suffix in (".tsx", ".ts", ".md", ".html", ".json"):
        text = p.read_text(encoding="utf-8")
        new = text.replace("https://www.dyad.sh/", "https://uor-foundation.github.io/foundry/")
        if new != text:
            p.write_text(new, encoding="utf-8", newline="\n")
# Every app built here wears the Hologram brand kit (the host adds shell/scaffold-brand.css to its page), so
# the model is told to style through the theme, not around it.
rules = dst / "AI_RULES.md"
rules.write_text(rules.read_text(encoding="utf-8").rstrip("\n") + "\n\n## Brand\n\n- The app wears the Hologram brand kit: its colours, fonts and radius are set for you. Use the shadcn/ui components and the theme classes (`bg-background`, `text-foreground`, `bg-primary`, `text-muted-foreground`, `border-border`, `bg-card`, `text-destructive`); never raw palette colours (`bg-blue-500`), hex values, or a new `:root` theme.\n", encoding="utf-8", newline="\n")
app = dst / "src" / "App.tsx"
app.write_text(app.read_text(encoding="utf-8").replace("<BrowserRouter>", "<BrowserRouter basename={import.meta.env.BASE_URL}>", 1), encoding="utf-8", newline="\n")
files = sorted(str(p.relative_to(dst)).replace("\\", "/") for p in (dst / "src").rglob("*") if p.is_file())
(dst / "files.json").write_text(json.dumps(files), encoding="utf-8", newline="\n")
print(f"staged {len(files)} scaffold files")

# Exact locked editor bytes join the shell before its manifest is projected.
from editor_assets import stage_editor_assets
editor = stage_editor_assets(root)
print(f"staged {len(editor['files'])} locked editor assets and licenses")

from compiler_assets import compiler_assets
compiler_receipt = compiler_assets(root)
print(f"staged {len(compiler_receipt['files'])} locked compiler assets and licenses")

# Application instances carry the exact project-owned model source.
shutil.copyfile(root.parent / "app" / "src" / "InteractionModel.lex.tex", root / "shell" / "interaction-model.lex.tex")

# The locked renderer compiler projects the host before the complete manifest is written.
subprocess.run(["node", str(root / "tools" / "host_bundle.mjs")], check=True)
subprocess.run(["node", str(root / "tools" / "host_bundle.mjs"), "--check"], check=True)
