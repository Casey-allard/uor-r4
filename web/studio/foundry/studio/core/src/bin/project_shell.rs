//! The shell projector: renders the landing page and the shell's closure from the generated `view()`.
//!
//! An adapter, not the model: every visible word comes from the Lean verified `View` record; this
//! file owns only markup. It writes `shell/index.html` (the renderer's UI at the root, with the appearance
//! switch: Immersive, Dark, Light), `shell/holo.html` (the runner for published
//! applications), `shell/app.webmanifest`,
//! `shell/manifest.json` (the hash list the service worker precaches from, so the shell is one
//! versioned closure) and `shell/sw.js` from `shell/sw.template.js` with the closure digest inside.
//!
//! Run from the repo root: cargo run --release --manifest-path core/Cargo.toml --bin project-shell

use hologram_forge_core::abi::view_json;
use hologram_forge_core::view;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The colours the pages and the web manifest name outside a stylesheet, read from the generated
/// `shell/brand.json` (tools/brand_kit.py, from the kit's tokens); nothing is typed here.
fn brand() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../shell/brand.json")).expect("shell/brand.json; run python3 tools/brand_kit.py")
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn page() -> String {
    // The homepage is the renderer's own UI, whole, at the root of the shell: its renderer is built by Vite
    // into assets/ with fixed names (scripts/vite.shell.config.mts), the host runs first so
    // window.electron exists when the contracts read it, and the appearance switch from the
    // landing page stays: Immersive (a curated photo behind frosted panels), Dark, Light, kept as
    // Hologram OS keeps it (holo.theme.v1) and mirrored into the renderer's own theme class.
    let v = view();
    let mut h = String::new();
    let _ = write!(
        h,
        r##"<!doctype html>
<html lang="en" class="notranslate" translate="no">
<head>
<meta charset="utf-8">
<meta name="google" content="notranslate">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="color-scheme" content="dark light">
<meta name="theme-color" content="{theme_color}">
<title>{title}</title>
<meta name="description" content="{lede}">
<link rel="icon" href="mark.svg" type="image/svg+xml">
<link rel="manifest" href="app.webmanifest">
<link id="renderer-style" rel="stylesheet" crossorigin>
<script>
// Vite identifies a stylesheet by its deployment-prefixed href attribute.
(function () {{ const style = document.getElementById("renderer-style"); style.setAttribute("href", new URL("assets/index.css", document.baseURI).pathname); }})();
</script>
<link rel="stylesheet" href="brand.css">
<link rel="stylesheet" href="appearance.css">
<script>
// Pre paint appearance, the same canonical state Hologram OS keeps (holo.theme.v1: palette, immersive,
// wallpaper) and the same hooks (data-holo-palette, data-holo-immersive, --holo-wallpaper, color-scheme),
// mirrored into the renderer's theme class and its localStorage "theme", so the first frame already wears the
// chosen look. First run, and once for anyone who chose before look 3: immersive on the first curated
// photo; Dark and Light are one click away.
(function () {{
  var root = document.documentElement, s = null;
  try {{ s = JSON.parse(localStorage.getItem("holo.theme.v1") || "null"); }} catch (e) {{}}
  if (!s || s.look !== 3) {{ s = {{ look: 3, palette: "dark", immersive: true, wallpaper: "wallpapers/galaxy.jpg", photo: {{"id":"galaxy","alt":"Galaxy sky","name":"Tiago Ferreira","user":"tiago_f_ferreira"}} }}; try {{ localStorage.setItem("holo.theme.v1", JSON.stringify(s)); }} catch (e) {{}} }}
  // Home keeps the home look; every other screen keeps one shared look, Dark until one is chosen there
  // (appearance.js). Home is the shell's own folder, read from the stylesheet link above.
  var l = document.querySelector('link[href="appearance.css"]'), base = "/", p = location.pathname, h = location.hash;
  try {{ if (l) base = new URL(".", l.href).pathname; }} catch (e) {{}}
  var homeHash = !h || h === "#" || h === "#/" || h === "#telemetry" || h.indexOf("#/?") === 0;
  if (!(p === base || p + "/" === base || p === base + "index.html") || !homeHash) {{ var b = null; try {{ b = JSON.parse(localStorage.getItem("holo.theme.build.v1") || "null"); }} catch (e) {{}} s = {{ palette: b && b.palette === "light" ? "light" : "dark", immersive: !!(b && b.immersive), wallpaper: s.wallpaper }}; }}
  var palette = s.palette === "light" ? "light" : "dark";
  root.setAttribute("data-holo-palette", palette);
  root.setAttribute("data-holo-immersive", s.immersive ? "on" : "off");
  root.style.setProperty("color-scheme", palette);
  if (s.wallpaper) root.style.setProperty("--holo-wallpaper", "url(" + JSON.stringify(s.wallpaper) + ")");
  root.classList.remove("light", "dark"); root.classList.add(palette);
  try {{ localStorage.setItem("theme", palette); }} catch (e) {{}}
}})();
</script>
<script type="application/json" id="wallpapers">{walls_json}</script>
<script type="application/json" id="view">{view_json}</script>
</head>
<body>
<div id="root"></div>
<button class="appearance" id="appearance" type="button" aria-haspopup="dialog" aria-expanded="false" aria-label="{appearance}"><svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="4"></circle><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"></path></svg></button>
<div class="popover" id="popover" role="dialog" aria-label="{appearance}" hidden>
  <button class="mode" type="button" data-mode="immersive"><span>{immersive}</span><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="3"></rect><path d="M3 15l5-5 4 4 3-3 6 6"></path><circle cx="16" cy="8" r="1.5"></circle></svg></button>
  <div class="walls" id="walls">{walls}</div>
  <button class="mode" type="button" data-mode="dark"><span>{dark}</span><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"></path></svg></button>
  <button class="mode" type="button" data-mode="light"><span>{light}</span><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="4"></circle><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"></path></svg></button>
</div>
<script type="module" src="host.bundle.js"></script>
<script type="module" src="assets/index.js"></script>
<script type="module" src="appearance.js"></script>
</body>
</html>
"##,
        title = esc(&v.headline),
        lede = esc(&v.lede),
        theme_color = brand()["theme_color"].as_str().unwrap_or("").to_owned(),
        appearance = esc(&v.appearanceLabel),
        dark = esc(&v.darkLabel),
        light = esc(&v.lightLabel),
        immersive = esc(&v.immersiveLabel),
        walls = v.wallpapers.iter().map(|w| format!(r#"<button class="wall" type="button" data-wall="wallpapers/{}" title="{}" aria-label="{}" style="background-image:url(wallpapers/{})"></button>"#, esc(&w.file), esc(&w.label), esc(&w.label), esc(&w.file))).collect::<String>(),
        walls_json = serde_json::json!(v.wallpapers.iter().map(|w| serde_json::json!({ "file": format!("wallpapers/{}", w.file), "name": w.label, "by": w.author, "byUrl": w.authorUrl })).collect::<Vec<_>>()).to_string().replace("</", "<\\/"),
        view_json = view_json().to_string().replace("</", "<\\/"),
    );
    h
}

fn webmanifest() -> String {
    let v = view();
    serde_json::json!({
        "name": "Hologram",
        "short_name": "Hologram",
        "description": v.lede,
        "start_url": "./",
        "display": "standalone",
        "background_color": brand()["background_color"],
        "theme_color": brand()["theme_color"],
        "icons": [{ "src": "mark.svg", "sizes": "any", "type": "image/svg+xml" }]
    })
    .to_string()
        + "\n"
}

/// The complete staged shell, including renderer and scaffold, in portable path order.
/// Worker, manifest and closure-bound provenance are excluded to avoid circular hashes.
/// Provider credentials cannot enter the public shell; model weights have separate ownership.
fn manifest(shell: &Path) -> String {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir).expect("shell dir").flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if path.parent() == Some(root) && name == "app" {
                    continue;
                }
                walk(&path, root, out);
            } else {
                out.push(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }
    let mut files = Vec::new();
    walk(shell, shell, &mut files);
    files.sort_by_key(|path| path.to_string_lossy().replace('\\', "/"));
    let mut rows = Vec::new();
    for rel in files {
        let name = rel.to_string_lossy().replace('\\', "/");
        assert!(name != "warmup.json" && !name.ends_with("/warmup.json"), "providerCredentialArtifact: public credential file refused");
        if name == "manifest.json" || name == "provenance.json" || name == "sw.js" || name == "sw.template.js" {
            continue;
        }
        let bytes = std::fs::read(shell.join(&rel)).expect("read shell file");
        rows.push(serde_json::json!({ "path": name, "sha256": sha256(&bytes), "bytes": bytes.len() }));
    }
    let closure = sha256(serde_json::to_string(&rows).unwrap().as_bytes());
    serde_json::json!({ "spec": "hologram-forge/shell/1", "closure": closure, "files": rows }).to_string() + "\n"
}

fn sha256(bytes: &[u8]) -> String {
    // A tiny SHA-256 so the projector carries no extra dependency; correctness is checked against
    // Python's hashlib by tools/corpus_inference.py on every run.
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut data = bytes.to_vec();
    let bit_len = (bytes.len() as u64) * 8;
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in data.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1); d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}


/// The runner page: opens a published application on any device. Served at `holo.html` and, by the
/// worker, at holoPath(κ) when the View is not yet unpacked there: the page finds the bytes in the
/// device store or asks for the file, verifies them through the core (PrismPM's own validator),
/// unpacks the View under its address and reloads. Every word is the View's.
fn holo_page() -> String {
    let v = view();
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<meta name="color-scheme" content="dark light">
<title>{title}</title>
<script>document.write('<base href="' + location.pathname.replace(/holo\/.*$/, "").replace(/[^/]*$/, "") + '">');</script>
<link rel="icon" href="mark.svg" type="image/svg+xml">
<link rel="stylesheet" href="brand.css">
<style>
/* Layout only; every colour, font and radius is brand.css's, generated from the kit's tokens. */
body {{ margin: 0; min-height: 100vh; background: var(--background); color: var(--foreground); font-family: var(--font-sans); font-size: 16px; line-height: 1.6; -webkit-font-smoothing: antialiased; }}
.mark {{ position: fixed; top: 21px; left: 21px; display: inline-flex; padding: 3px; }}
.mark .on-light {{ display: none; }} .mark .on-dark {{ display: block; }}
:root.light .mark .on-light {{ display: block; }} :root.light .mark .on-dark {{ display: none; }}
main {{ max-width: 640px; margin: 0 auto; padding: 96px 24px 64px; }}
main h1 {{ font-size: 40px; line-height: 1.1; margin: 0 0 12px; }}
.lede {{ color: var(--muted-foreground); margin: 0 0 24px; }}
.hint {{ color: var(--muted-foreground); font-size: 14px; }}
.btn {{ display: inline-flex; align-items: center; height: 36px; padding: 0 16px; border-radius: calc(var(--radius) - 2px); background: var(--brand); color: var(--brand-foreground); font: inherit; font-size: 14px; font-weight: 500; cursor: pointer; border: 0; }}
.btn:hover {{ filter: brightness(1.06); }}
.apps {{ list-style: none; padding: 0; margin: 24px 0; display: grid; gap: 8px; }}
.apps li {{ display: flex; flex-wrap: wrap; gap: 12px; align-items: baseline; padding: 12px 14px; border: 1px solid var(--border); border-radius: var(--radius); background: var(--card); color: var(--card-foreground); }}
.apps a {{ color: inherit; }}
</style>
<script>(function () {{ var root = document.documentElement, s = null; try {{ s = JSON.parse(localStorage.getItem("holo.theme.v1") || "null"); }} catch (e) {{}} var palette = s && s.palette === "light" ? "light" : "dark"; root.setAttribute("data-holo-palette", palette); root.setAttribute("data-holo-immersive", "off"); root.style.setProperty("color-scheme", palette); root.classList.add(palette); }})();</script>
<script type="application/json" id="view">{view_json}</script>
</head>
<body>
<a class="mark" href="./" aria-label="Hologram"><img class="on-dark" src="lockup-white.svg" alt="Hologram" width="157" height="30"><img class="on-light" src="lockup-black.svg" alt="Hologram" width="157" height="30"></a>
<main>
  <h1 class="font-display">{title}</h1>
  <p class="lede">{lede}</p>
  <p class="hint mono" id="status"></p>
  <p><label class="btn" for="file" id="pick">{pick}</label><input id="file" type="file" accept=".holo" hidden></p>
  <ul class="apps" id="apps"></ul>
  <p class="hint">{runs}</p>
</main>
<script type="module">
import {{ install, stored, installed }} from "./holo.js";
import {{ coreReady }} from "./inference.js";
const V = JSON.parse(document.getElementById("view").textContent);
const status = document.getElementById("status"), list = document.getElementById("apps");
const short = (k) => k.replace(/^blake3:/, "").slice(0, 12) + "…";
const wanted = (location.pathname.match(/\/holo\/([^/]+)\//) || [])[1] || null;
async function show() {{
  const rows = await installed();
  list.innerHTML = "";
  for (const r of rows) {{
    const li = document.createElement("li");
    const a = document.createElement("a"); a.href = new URL("holo/" + r.kappa + "/", document.baseURI).href; a.textContent = (r.application || "application") + " · " + short(r.kappa) + " · " + r.byteLength + " bytes";
    const d = document.createElement("a"); d.href = new URL("holo/" + r.kappa + ".holo", document.baseURI).href; d.textContent = V.downloadLabel; d.className = "hint"; d.setAttribute("download", "");
    li.append(a, " ", d); list.append(li);
  }}
}}
async function open(bytes) {{
  status.textContent = V.verifyingLabel + "…";
  try {{
    const done = await install(bytes);
    status.textContent = V.publishedLabel + " · " + short(done.kappa) + " · " + done.byteLength + " bytes · " + done.ms + " ms";
    await show();
    location.href = done.url;
  }} catch (e) {{
    status.textContent = V.refusedLabel + " · " + String((e && e.message) || e);
  }}
}}
document.getElementById("file").addEventListener("change", async (e) => {{ const f = e.target.files[0]; if (f) open(new Uint8Array(await f.arrayBuffer())); }});
(async () => {{
  await coreReady();
  await show();
  if (wanted) {{ const bytes = await stored(wanted); if (bytes) open(bytes); else status.textContent = short(wanted) + " · " + V.pickLabel; }}
}})();
</script>
</body>
</html>
"##,
        title = esc(&v.holoTitle),
        lede = esc(&v.holoLede),
        pick = esc(&v.pickLabel),
        runs = esc(&v.runsLabel),
        view_json = view_json()
    )
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let shell = root.join("shell");
    std::fs::write(shell.join("index.html"), page()).expect("write index.html");
    std::fs::write(shell.join("holo.html"), holo_page()).expect("write holo.html");
    std::fs::write(shell.join("app.webmanifest"), webmanifest()).expect("write webmanifest");
    let manifest = manifest(&shell);
    std::fs::write(shell.join("manifest.json"), &manifest).expect("write manifest.json");
    // The closure digest is written into the worker itself, so a new shell is a new worker: the
    // browser installs it, precaches the new closure, and drops the old shell cache on activation.
    let closure = serde_json::from_str::<serde_json::Value>(&manifest).unwrap()["closure"].as_str().unwrap().to_owned();
    let worker = std::fs::read_to_string(shell.join("sw.template.js")).expect("read shell/sw.template.js");
    std::fs::write(shell.join("sw.js"), worker.replace("__CLOSURE__", &closure)).expect("write sw.js");
    println!("projected shell/index.html, holo.html, app.webmanifest, manifest.json, sw.js from view(); closure {}", &closure[..12]);
}
