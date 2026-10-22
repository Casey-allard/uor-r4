# The three products

UOR-R4 aims to ship one thing people can use: a small, fast language model that runs on your own laptop or in your own browser, with nothing sent to a server. The owner defined three products around it on 19 October 2026. The first is the **model** itself, saved as one file of whole numbers. The second is a **web client** that runs that file inside a browser tab. The third is a **desktop client**, an installable app that runs the same file natively and remembers your conversations. **None of the three ships today.** The project is pre-alpha: 0 of its 8 milestones are complete and no model holds a useful conversation yet ([STATUS](../../STATUS.md), [NOW](../integration/NOW.md)). Nothing is released until a trained model passes milestones M1 to M8.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run with its artifact, **hypothesis** means an untested idea ([`docs/formal_vocabulary.md`](../formal_vocabulary.md), [glossary](../start/glossary.md)).

![One file in the middle, labelled "model artifact (UORLUT01, integer only)". Two arrows leave it. The left arrow goes to a browser window labelled "web client: WebAssembly in a tab, Studio shell". The right arrow goes to a laptop labelled "desktop client: Tauri app, native engine, durable memory, offline". Both clients share one user interface, drawn as the same Studio panel inside each. A dashed border around everything reads "on your device; no model server".](../figures/products/product-family.svg)

*One model file feeds two clients that share one interface. Everything inside the dashed line runs on the user's own device.*

## The three at a glance

| Product | What it is meant to be | What exists today | Main gate |
|---|---|---|---|
| [Language model](./language-model.md) | A geometric language model exported as one checksum-pinned integer file (`UORLUT01`) | The serving engine, tested on a synthetic stand-in model. No trained model has been exported to it | M1 to M4, then M8 |
| [Web client](./web-client.md) | A local model in the browser: download the file once, run it in WebAssembly on the device | The Studio interface (`index.html`), a WebAssembly build of the engine (`UOR-Foundation/uor-r4#2654`), and a paused Pages deploy | M7 |
| [Desktop client](./desktop-client.md) | A Tauri app sharing the web interface, with the native engine, file access, durable memory and offline use on M1-class laptops | [`src-tauri/`](../../src-tauri/) native Rust backend + `--stdio` / `--serve-studio` Studio host (`#2826`, in progress: `tauri` WebView crate linking awaits vendor-fetching Tauri v2 into `Cargo.lock`) linking `uor-r4-integer` + `uor-r4-object-store`, offline Studio shims, and 6 integration tests on the synthetic stand-in ([record](../labs/infra-desktop-client-tauri/README.md)) | M3, M5, M7 |

A fourth page, [Releases and artifacts](./releases-and-artifacts.md), covers the model file format, how a release would be cut, the Hugging Face repositories and how checksums tie them together.

## Why the products look like this

**Local by design.** The mission is "fast local AI": a model that trains fast, runs fast and approaches frontier capability on ordinary hardware ([charter](../../CHARTER.md), section 1). The lead metric is tokens per second. A product that needs a data centre would defeat the point.

**One file, many hosts.** The served model obeys one contract, called D11: no decimal (floating-point) arithmetic, no multiply or divide instruction and no transcendental functions, only adds, shifts and table look-ups ([D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded)). A file that needs nothing more exotic than integer adds can run the same way in a browser, on a laptop processor or on Apple silicon. The charter says serving this file on the user's device is the product and is allowed. The rule that research compute must run on a GPU (D24) does not apply to it ([charter](../../CHARTER.md), section 3.6).

**One interface, two shells.** The Studio, a single-page interface already in the repository, is the shell for both clients. The browser runs the engine as WebAssembly. The desktop app wraps the same page and runs the engine as native code, which should be faster (a **hypothesis**; no side-by-side timing exists).

## Where each product stands

| Item | State | Source |
|---|---|---|
| Milestones complete | 0 of 8 (**measurement** of the tracker) | [STATUS](../../STATUS.md), [NOW](../integration/NOW.md) |
| Integer serving engine, native | Built; zero-multiply audit green on 44 symbol ranges (**measurement**) | [project track](../integration/project-track.md), M4 |
| Integer serving engine, WebAssembly | Built; 7 of 7 serving functions with zero multiply, divide and float instructions; save and restore reload identically on a synthetic model (**measurement**) | [record](../labs/wasm-serving-runtime-2026-10-18/README.md) |
| A trained model in that engine | None. Training is held, and no model built under today's rules has been exported | [NOW](../integration/NOW.md), [lut-serving dossier](../mechanisms/lut-serving.md) |
| Studio connected to the new engine | In progress — web worker (`#2818`) and native desktop backend + offline Studio shim (`#2826`) verified on the synthetic stand-in; no trained model exists | [`index.html`](../../web/studio/index.html), [desktop client record](../labs/infra-desktop-client-tauri/README.md) |
| Public website deploy | Paused since 15 October; the live site is the 13 October build | [`deploy.yml`](../../.github/workflows/deploy.yml) |
| Desktop app project | Native Rust backend + `--stdio` / `--serve-studio` host built in [`src-tauri/`](../../src-tauri/) (`#2826`); verified on the `seed = 5` synthetic stand-in (`6` integration controls); `tauri` WebView crate linking awaits vendor-fetching Tauri v2 into `Cargo.lock` | [desktop client record](../labs/infra-desktop-client-tauri/README.md) |

A passing audit shows that a forbidden instruction is absent. It is not a trained capability, and it says nothing about how well a model answers.

## The order things must happen in

The products depend on the model, and the model depends on milestones. Roughly:

1. **M1 to M3** give a model that predicts text, answers from exact memory, and keeps memory across sessions.
2. **M4** exports that model through D11 and serves it from the command line.
3. **M5** measures its cost on a laptop, and **M6** adds reasoning and coding on the same model.
4. **M7** makes the browser worker, the command line and the Rust library produce bit-identical tokens. That is the web client's gate.
5. **M8** is the alpha release: one versioned, checksum-pinned model file that meets every earlier bar.

The desktop client has no milestone of its own yet. It rides on M3 for memory, M5 for laptop cost and M7 for the shared interface. The full definitions are in the [project track](../integration/project-track.md).

## Go deeper

- [Language model](./language-model.md): what the model is and the M1 to M8 path in plain words.
- [Web client](./web-client.md) and [desktop client](./desktop-client.md): architectures, what exists and what is missing.
- [Releases and artifacts](./releases-and-artifacts.md): `UORLUT01`, the release workflow, Hugging Face, checksums.
- [Serving and export](../model/serving-and-export.md): how the integer engine computes without a multiply.
- Mechanism dossiers: [integer table serving](../mechanisms/lut-serving.md), [exact object store](../mechanisms/object-store.md).
- The [charter](../../CHARTER.md), sections 2 (products) and 3.6 (serving on the user's device).
