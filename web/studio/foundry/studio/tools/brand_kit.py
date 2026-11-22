"""The Hologram brand kit's tokens, generated into every stylesheet the shell serves.

Source of truth: vendor/hologram-brand-kit/tokens/hologram-tokens.json (W3C DTCG, the kit at the commit
BRAND_KIT_REV pins). Nothing here names a colour, a font or a radius by hand: every value is read from
the token sets `hologram-light`, `hologram-dark`, `core` and `global`, and the two derived layers
(Immersive alpha on surfaces that sit on the photo; the renderer's literal palette classes remapped onto the
kit's warm ramp) are arithmetic on those values, spelled once below.

Writes
  shell/brand.css                  the fonts, the light and dark variables, the Immersive layer, the
                                   display and mono rules, the glass the appearance switch uses
  vendor/prism/hologram.theme.css   what the renderer build splices into the renderer's globals.css: the same
                                   variables, plus a Tailwind @theme block that remaps every palette
                                   family the renderer's screens name (blue, purple, gray, ...) onto the kit's
                                   warm ramp, floors text-xs at 14px, and names the display face
  shell/fonts/*, shell/mark.svg, shell/lockup-*.svg   copied from the vendored kit

    python3 tools/brand_kit.py            # generate
    python3 tools/brand_kit.py --record   # generate and record the vendored digests (after a kit bump)
    python3 tools/brand_kit.py --check    # the Brand gate: vendored files match the record, generated
                                          # files match a fresh generation, no colour typed by hand
"""
import hashlib, io, json, pathlib, re, shutil, sys

root = pathlib.Path(__file__).resolve().parent.parent
kit = root / "vendor" / "hologram-brand-kit"
record = root / "tools" / "brand_kit.sha256"
tokens = json.loads((kit / "tokens" / "hologram-tokens.json").read_text(encoding="utf-8"))

LIGHT = {k: v["$value"] for k, v in tokens["hologram-light"]["color"].items()}
DARK = {k: v["$value"] for k, v in tokens["hologram-dark"]["color"].items()}
CORE = {group: {k: v["$value"] for k, v in tokens["core"][group].items() if isinstance(v, dict) and "$value" in v} for group in ("neutral", "red", "blue")}
WHITE = tokens["core"]["white"]["$value"]
RADIUS = tokens["global"]["radius"]
FONT = {k: v["$value"] for k, v in tokens["global"]["font"]["family"].items()}
SIZE = {k: v["$value"] for k, v in tokens["global"]["font"]["size"].items()}
TRACKING = {k: v["$value"] for k, v in tokens["global"]["font"]["tracking"].items()}
for colour in list(LIGHT.values()) + list(DARK.values()) + [WHITE]:
    assert re.fullmatch(r"#[0-9a-f]{6}([0-9a-f]{2})?", colour), colour

# ---- the warm ramp: Tailwind's shade scale, every step a value the kit already has (no invented hex)
RAMP = {
    "50": DARK["foreground"], "100": LIGHT["background"], "200": LIGHT["secondary"], "300": LIGHT["border"],
    "400": LIGHT["ring"], "500": DARK["ring"], "600": LIGHT["muted-foreground"], "700": DARK["chart-5"],
    "800": LIGHT["foreground"], "900": DARK["card"], "950": DARK["background"],
}
# Families the renderer's screens name; every hue but red becomes the warm ramp (the kit has one accent, used
# once per view, and the roles it lacks, success and warning, are UPSTREAM.md entries).
QUIET = ["gray", "zinc", "slate", "neutral", "stone", "blue", "indigo", "violet", "purple", "sky", "teal", "cyan", "fuchsia", "pink", "rose", "lime", "emerald", "green", "amber", "yellow", "orange"]
RED = {"50": RAMP["50"], "100": RAMP["100"], "200": RAMP["200"], "300": CORE["red"]["400"], "400": CORE["red"]["400"], "500": CORE["red"]["600"],
       "600": CORE["red"]["600"], "700": CORE["red"]["600"], "800": CORE["red"]["600"], "900": CORE["red"]["600"], "950": CORE["red"]["600"]}
# Arbitrary hex classes the renderer uses for its Pro accent: the foreground, never a second accent.
ARBITRARY = {"#6c55dc": "foreground", "#7f22fe": "foreground"}

# ---- Immersive: the photo unfaded inside the workspace (the inset panel), behind its transparent main
# surface; glass on what carries text. The sidebar and the margin round the workspace are solid ground.
# Alpha per surface, the smallest at which the pairs pass AA on the three curated photos (measured in
# the browser; see HOLOGRAM/FORGE-BRAND-TOKENS-EVAL.md). The sidebar has no photo behind it: opaque.
ALPHA = {"background": 0.0, "sidebar": 1.0, "card": 0.74, "popover": 0.94, "muted": 0.7, "accent": 0.7, "secondary": 0.7}


def rgb(hex6, alpha):
    r, g, b = (int(hex6[i:i + 2], 16) for i in (1, 3, 5))
    return f"rgb({r} {g} {b} / {alpha:g})"


def block(selector, values, extra=()):
    lines = [f"  --{k}: {v};" for k, v in values.items()] + [f"  {e}" for e in extra]
    return selector + " {\n" + "\n".join(lines) + "\n}\n"


def variables(colours, dark):
    v = dict(colours)
    v["radius"] = RADIUS["lg"]["$value"]
    v["font-sans"] = FONT["sans"]
    v["font-display"] = FONT["display"]
    v["font-mono"] = FONT["mono"]
    v["tracking-display"] = TRACKING["display"]
    v["tracking-caps"] = TRACKING["caps"]
    # the renderer's own extras, each a kit value: the ramps its docs and panels use, the docs ground.
    if dark:
        v.update({"background-lightest": colours["secondary"], "background-lighter": colours["card"], "background-darker": colours["background"], "background-darkest": CORE["neutral"]["950"], "docs-bg": colours["muted"], "destructive-foreground": colours["destructive"]})
    else:
        v.update({"background-lightest": WHITE, "background-lighter": colours["card"], "background-darker": colours["secondary"], "background-darkest": colours["accent"], "docs-bg": colours["muted"], "destructive-foreground": colours["destructive"]})
    return v


def theme_block():
    rows = []
    for family in QUIET:
        for shade, value in RAMP.items():
            rows.append(f"  --color-{family}-{shade}: {value};")
    for shade, value in RED.items():
        rows.append(f"  --color-red-{shade}: {value};")
    rows += ["  --color-brand: var(--brand);", "  --color-brand-foreground: var(--brand-foreground);",
             "  --font-sans: var(--font-sans);", "  --font-display: var(--font-display);", "  --font-mono: var(--font-mono);",
             f"  --text-xs: {SIZE['sm']};", "  --text-xs--line-height: 1.35;"]
    return "@theme {\n" + "\n".join(rows) + "\n}\n"


FACES = [("Archivo", "Archivo-Medium", 500), ("Archivo", "Archivo-SemiBold", 600), ("Archivo", "Archivo-Bold", 700),
         ("Geist", "Geist-Regular", 400), ("Geist", "Geist-Medium", 500), ("Geist", "Geist-SemiBold", 600), ("Geist", "Geist-Bold", 700),
         ("Geist Mono", "GeistMono-Regular", 400), ("Geist Mono", "GeistMono-Medium", 500)]


def font_faces(prefix):
    return "".join(f'@font-face {{ font-family: "{family}"; src: url("{prefix}fonts/{file}.woff2") format("woff2"); font-weight: {weight}; font-display: swap; }}\n' for family, file, weight in FACES)


HEAD = "/* GENERATED by tools/brand_kit.py from vendor/hologram-brand-kit/tokens/hologram-tokens.json (the kit at BRAND_KIT_REV). Do not edit: every colour, font and radius is the kit's. */\n"


def immersive_block():
    v = {k: rgb(DARK[k], ALPHA[k]) for k in ALPHA}
    v["input"] = DARK["input"]
    v["border"] = DARK["border"]
    v["sidebar-border"] = DARK["sidebar-border"]
    return block(':root[data-holo-immersive="on"], :root[data-holo-immersive="on"].dark', v)


def brand_css():
    out = HEAD + font_faces("")
    out += block(":root", variables(LIGHT, False)) + block(".dark", variables(DARK, True))
    out += "\n/* Immersive: the photo, unfaded, inside the workspace window (the inset panel, clipped to its corners), behind a\n   transparent main surface; glass on the panels that carry text. The sidebar and the margin round the workspace\n   are the page's solid ground, as in every other look. */\n"
    out += ':root[data-holo-immersive="on"] body { background: var(--sidebar); }\n'
    out += ':root[data-holo-immersive="on"] [data-slot="sidebar-inset"] { background: var(--holo-wallpaper, none) center / cover no-repeat, var(--background-darker); }\n'
    out += immersive_block()
    out += ':root[data-holo-immersive="on"] .popover { -webkit-backdrop-filter: blur(24px) saturate(1.2); backdrop-filter: blur(24px) saturate(1.2); }\n'
    out += "\n/* Type: the display face for titles, the interface face for everything else, the mono face for addresses; nothing under 14px. */\n"
    out += "body { font-family: var(--font-sans); }\n"
    out += ".font-display, h1.font-display, main > h1 { font-family: var(--font-display); letter-spacing: var(--tracking-display); font-weight: 600; }\n"
    out += ".mono, code, kbd, samp { font-family: var(--font-mono); font-size: " + SIZE["sm"] + "; }\n"
    out += spectrum_block()
    return out


def spectrum_block():
    # The Prism frame's spectrum (its address-bar ring, which Holo Search's search bar and Forge's prompt box
    # wear): its nine stops, read from the frame's own source in this repository (frame/frame.css, pinned from
    # UOR-Foundation/template), so the colours are the frame's and none is typed here.
    frame = (root.parent / "frame" / "frame.css").read_text(encoding="utf-8")
    stops = re.findall(r"(--spectrum-[a-z]+):\s*(#[0-9a-fA-F]{6});", frame)
    assert len(stops) == 9, "frame/frame.css should define nine spectrum stops"
    return "\n/* The Prism frame's spectrum, from frame/frame.css. */\n:root { " + " ".join(f"{k}: {v};" for k, v in stops) + " }\n"


def theme_css():
    return HEAD + block(":root", variables(LIGHT, False)) + block(".dark", variables(DARK, True)) + theme_block()


# ---- the apps the model writes as code. Their scaffold (shadcn/ui, as the renderer vendors it) reads its
# colours as `hsl(var(--x))`, so the kit's values are written as the bare `h s% l%` the scaffold expects,
# under the scaffold's own names. The host adds this sheet to every such app's page, unlayered, after the
# app's own CSS: the scaffold declares its variables inside `@layer base`, so the kit's win whatever the
# app writes there, and every app built here wears the kit.
SCAFFOLD = ["background", "foreground", "card", "card-foreground", "popover", "popover-foreground", "primary", "primary-foreground",
            "secondary", "secondary-foreground", "muted", "muted-foreground", "accent", "accent-foreground", "destructive",
            "border", "input", "ring", "brand", "brand-foreground", "success", "success-foreground", "warning", "warning-foreground",
            "chart-1", "chart-2", "chart-3", "chart-4", "chart-5", "sidebar-foreground", "sidebar-primary", "sidebar-primary-foreground",
            "sidebar-accent", "sidebar-accent-foreground", "sidebar-border", "sidebar-ring"]


def hsl(hex_value):
    r, g, b = (int(hex_value[i:i + 2], 16) / 255 for i in (1, 3, 5))
    hi, lo = max(r, g, b), min(r, g, b)
    light = (hi + lo) / 2
    if hi == lo:
        hue = sat = 0.0
    else:
        d = hi - lo
        sat = d / (2 - hi - lo) if light > 0.5 else d / (hi + lo)
        hue = ((g - b) / d + (6 if g < b else 0)) if hi == r else ((b - r) / d + 2) if hi == g else ((r - g) / d + 4)
        hue *= 60
    out = f"{hue:.1f} {sat * 100:.1f}% {light * 100:.1f}%"
    return out + (f" / {int(hex_value[7:9], 16) / 255:.3f}" if len(hex_value) == 9 else "")


def scaffold_vars(colours):
    v = {k: hsl(colours[k]) for k in SCAFFOLD}
    v["sidebar-background"] = hsl(colours["sidebar"])
    v["destructive-foreground"] = hsl(colours["destructive-foreground"] if colours["destructive-foreground"] != colours["destructive"] else WHITE)
    v["radius"] = RADIUS["lg"]["$value"]
    v["font-sans"] = FONT["sans"]
    v["font-display"] = FONT["display"]
    v["font-mono"] = FONT["mono"]
    return v


def scaffold_css():
    out = HEAD + block(":root", scaffold_vars(LIGHT)) + block(".dark", scaffold_vars(DARK))
    out += "body { font-family: var(--font-sans); }\n"
    out += "h1, h2, .font-display { font-family: var(--font-display); letter-spacing: " + TRACKING["display"] + "; }\n"
    out += "code, kbd, samp, pre, .font-mono { font-family: var(--font-mono); }\n"
    return out


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    io.open(path, "w", encoding="utf-8", newline="\n").write(text)


def sha(path):
    # Text is hashed with one line ending, so a Windows checkout and the Linux lane agree.
    data = pathlib.Path(path).read_bytes()
    if pathlib.Path(path).suffix in (".json", ".css", ".svg", ".txt", ".md"):
        data = data.replace(b"\r\n", b"\n")
    return hashlib.sha256(data).hexdigest()


VENDORED = sorted(str(p.relative_to(kit)).replace("\\", "/") for p in kit.rglob("*") if p.is_file())
COPIES = [("logos/svg/logomark/Hologram_Logomark_White.svg", "shell/mark.svg"), ("logos/svg/lockup/Hologram_Lockup_Black.svg", "shell/lockup-black.svg"),
          ("logos/svg/lockup/Hologram_Lockup_White.svg", "shell/lockup-white.svg"), ("fonts/OFL.txt", "shell/fonts/OFL-Geist.txt"), ("fonts/OFL-Archivo.txt", "shell/fonts/OFL-Archivo.txt")]
COPIES += [(f"fonts/{file}.woff2", f"shell/fonts/{file}.woff2") for _, file, _ in FACES]
def brand_json():
    return json.dumps({"generated": "tools/brand_kit.py", "theme_color": DARK["background"], "background_color": DARK["background"], "paper": LIGHT["background"], "brand": DARK["brand"]}, indent=1) + "\n"


GENERATED = {"shell/brand.css": brand_css, "vendor/prism/hologram.theme.css": theme_css, "shell/brand.json": brand_json, "shell/scaffold-brand.css": scaffold_css}
HAND = re.compile(r"#[0-9a-fA-F]{3,8}\b|oklch\(|rgba?\(|hsla?\(")
# The projected pages carry the meta colour brand.json gives them; the check reads their source instead.
HAND_FREE = ["shell/appearance.css", "shell/appearance.js", "shell/holo.js", "core/src/bin/project_shell.rs", "vendor/prism/vite.shell.config.mts", "scripts/stamp_shell.py"]

if "--check" in sys.argv:
    bad = []
    recorded = dict(reversed(line.split()) for line in record.read_text(encoding="utf-8").split("\n") if line.strip())
    for rel in VENDORED:
        if recorded.get(rel) != sha(kit / rel):
            bad.append(f"vendored {rel} differs from the record")
    for src, dst in COPIES:
        if not (root / dst).exists() or sha(kit / src) != sha(root / dst):
            bad.append(f"{dst} is not the kit's {src}")
    for rel, make in GENERATED.items():
        if not (root / rel).exists() or (root / rel).read_bytes().replace(b"\r\n", b"\n") != make().encode("utf-8"):
            bad.append(f"{rel} differs from a fresh generation")
    for rel in HAND_FREE:
        for n, line in enumerate((root / rel).read_text(encoding="utf-8").split("\n"), 1):
            if HAND.search(line) and "generated" not in line.lower():
                bad.append(f"{rel}:{n}: a colour typed by hand: {line.strip()[:80]}")
    if bad:
        sys.exit("brand gate refused:\n" + "\n".join(bad))
    print(f"brand: {len(VENDORED)} vendored files match, {len(COPIES)} copies match, {len(GENERATED)} generated files match, no colour typed by hand")
    sys.exit(0)

for src, dst in COPIES:
    (root / dst).parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(kit / src, root / dst)
for rel, make in GENERATED.items():
    write(root / rel, make())
if "--record" in sys.argv:
    write(record, "".join(f"{sha(kit / rel)}  {rel}\n" for rel in VENDORED))
    print(f"recorded {len(VENDORED)} digests")
print(f"wrote {', '.join(GENERATED)} and {len(COPIES)} copies from the kit at {(root / 'BRAND_KIT_REV').read_text().strip()[:12]}")
