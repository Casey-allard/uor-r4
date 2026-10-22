// Self-hosted offline vendor shims and native desktop bridge for UOR-R4 Studio
// (`web/studio/index.html`). Replaces external CDN scripts (marked, KaTeX, pdf.js,
// three.js, Monaco loader) so both the web client and the native desktop client run
// 100% offline with zero external network requests.

(function initUorOfflineVendor(globalScope) {
    'use strict';

    function escapeHtml(raw) {
        return String(raw ?? '')
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    function formatInlineMarkdown(escapedLine) {
        let out = escapedLine;
        out = out.replace(/`([^`\n]+)`/g, '<code>$1</code>');
        out = out.replace(
            /\[([^\]]+)\]\(([^)\s]+)\)/g,
            (_m, label, rawHref) => {
                const trimmed = String(rawHref || '').trim();
                const safeHref = /^javascript:/i.test(trimmed) ? '#blocked' : trimmed;
                return `<a href="${safeHref}" target="_blank" rel="noopener noreferrer">${label}</a>`;
            },
        );
        out = out.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
        out = out.replace(/(^|[^*])\*([^*\n]+)\*(?!\*)/g, '$1<em>$2</em>');
        out = out.replace(/(^|[^_])_([^_\n]+)_(?!_)/g, '$1<em>$2</em>');
        return out;
    }

    // 1. Offline Markdown Renderer (`marked` compatible API)
    if (typeof globalScope.marked === 'undefined') {
        class OfflineRenderer {
            code(codeText, infostring) {
                const lang = escapeHtml((infostring || 'text').trim().split(/\s+/)[0]);
                return `<pre><code class="language-${lang}">${escapeHtml(codeText)}</code></pre>`;
            }
        }
        let customRenderer = null;
        globalScope.marked = {
            Renderer: OfflineRenderer,
            use(opts) {
                if (opts && opts.renderer) {
                    customRenderer = opts.renderer;
                }
            },
            parse(markdownText) {
                const src = String(markdownText ?? '');
                const blocks = [];
                const withPlaceholders = src.replace(/```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g, (_m, lang, body) => {
                    const idx = blocks.length;
                    if (customRenderer && typeof customRenderer.code === 'function') {
                        try {
                            blocks.push(customRenderer.code(body, lang || 'text'));
                            return `@@UOR_CODE_BLOCK_${idx}@@`;
                        } catch (_e) {
                            // Fall back to standard pre/code below.
                        }
                    }
                    blocks.push(
                        `<pre><code class="language-${escapeHtml(lang || 'text')}">${escapeHtml(body)}</code></pre>`,
                    );
                    return `@@UOR_CODE_BLOCK_${idx}@@`;
                });

                const lines = withPlaceholders.split('\n');
                const out = [];
                let i = 0;

                while (i < lines.length) {
                    const rawLine = lines[i];
                    const trimmed = rawLine.trim();

                    if (!trimmed) {
                        i++;
                        continue;
                    }

                    const codePlaceholderMatch = trimmed.match(/^@@UOR_CODE_BLOCK_(\d+)@@$/);
                    if (codePlaceholderMatch) {
                        out.push(blocks[Number(codePlaceholderMatch[1])] || '');
                        i++;
                        continue;
                    }

                    const headingMatch = rawLine.match(/^(#{1,4})\s+(.+)$/);
                    if (headingMatch) {
                        const level = headingMatch[1].length;
                        out.push(`<h${level}>${formatInlineMarkdown(escapeHtml(headingMatch[2]))}</h${level}>`);
                        i++;
                        continue;
                    }

                    if (
                        trimmed.startsWith('|') &&
                        trimmed.endsWith('|') &&
                        i + 1 < lines.length &&
                        /^\s*\|[\s|:-]+\|\s*$/.test(lines[i + 1])
                    ) {
                        const parseRow = (rowStr) =>
                            rowStr
                                .trim()
                                .replace(/^\||\|$/g, '')
                                .split('|')
                                .map((c) => formatInlineMarkdown(escapeHtml(c.trim())));
                        const headers = parseRow(rawLine);
                        i += 2;
                        const bodyRows = [];
                        while (i < lines.length && lines[i].trim().startsWith('|') && lines[i].trim().endsWith('|')) {
                            bodyRows.push(parseRow(lines[i]));
                            i++;
                        }
                        const thead = `<thead><tr>${headers.map((h) => `<th>${h}</th>`).join('')}</tr></thead>`;
                        const tbody = `<tbody>${bodyRows
                            .map((r) => `<tr>${r.map((c) => `<td>${c}</td>`).join('')}</tr>`)
                            .join('')}</tbody>`;
                        out.push(`<table>${thead}${tbody}</table>`);
                        continue;
                    }

                    if (/^>\s?/.test(trimmed)) {
                        const quoteLines = [];
                        while (i < lines.length && /^>\s?/.test(lines[i].trim())) {
                            quoteLines.push(
                                formatInlineMarkdown(escapeHtml(lines[i].trim().replace(/^>\s?/, ''))),
                            );
                            i++;
                        }
                        out.push(`<blockquote>${quoteLines.join('<br>')}</blockquote>`);
                        continue;
                    }

                    if (/^[-*+]\s+/.test(trimmed)) {
                        const items = [];
                        while (i < lines.length && /^[-*+]\s+/.test(lines[i].trim())) {
                            const itemText = lines[i].trim().replace(/^[-*+]\s+/, '');
                            items.push(`<li>${formatInlineMarkdown(escapeHtml(itemText))}</li>`);
                            i++;
                        }
                        out.push(`<ul>${items.join('')}</ul>`);
                        continue;
                    }

                    if (/^\d+\.\s+/.test(trimmed)) {
                        const items = [];
                        while (i < lines.length && /^\d+\.\s+/.test(lines[i].trim())) {
                            const itemText = lines[i].trim().replace(/^\d+\.\s+/, '');
                            items.push(`<li>${formatInlineMarkdown(escapeHtml(itemText))}</li>`);
                            i++;
                        }
                        out.push(`<ol>${items.join('')}</ol>`);
                        continue;
                    }

                    const paraLines = [];
                    while (
                        i < lines.length &&
                        lines[i].trim() &&
                        !/^(#{1,4}\s+|>\s?|[-*+]\s+|\d+\.\s+|@@UOR_CODE_BLOCK_\d+@@$)/.test(lines[i].trim())
                    ) {
                        paraLines.push(formatInlineMarkdown(escapeHtml(lines[i])));
                        i++;
                    }
                    if (paraLines.length > 0) {
                        out.push(`<p>${paraLines.join('\n')}</p>`);
                    } else {
                        i++;
                    }
                }

                let finalHtml = out.join('\n');
                finalHtml = finalHtml.replace(/@@UOR_CODE_BLOCK_(\d+)@@/g, (_m, idx) => blocks[Number(idx)] || '');
                return finalHtml;
            },
        };
    }

    // 2. Offline Math Formatter (`katex` compatible API)
    const LATEX_SYMBOLS = {
        '\\alpha': 'α',
        '\\beta': 'β',
        '\\gamma': 'γ',
        '\\delta': 'δ',
        '\\Delta': 'Δ',
        '\\epsilon': 'ε',
        '\\theta': 'θ',
        '\\lambda': 'λ',
        '\\mu': 'μ',
        '\\pi': 'π',
        '\\rho': 'ρ',
        '\\sigma': 'σ',
        '\\Sigma': 'Σ',
        '\\tau': 'τ',
        '\\phi': 'φ',
        '\\chi': 'χ',
        '\\psi': 'ψ',
        '\\omega': 'ω',
        '\\Omega': 'Ω',
        '\\le': '≤',
        '\\leq': '≤',
        '\\ge': '≥',
        '\\geq': '≥',
        '\\ne': '≠',
        '\\neq': '≠',
        '\\approx': '≈',
        '\\times': '×',
        '\\cdot': '·',
        '\\pm': '±',
        '\\infty': '∞',
        '\\to': '→',
        '\\rightarrow': '→',
        '\\in': '∈',
        '\\sum': '∑',
        '\\prod': '∏',
    };

    function formatLatexExpression(rawExpr) {
        let s = escapeHtml(String(rawExpr ?? '').trim());
        s = s.replace(
            /\\frac\{([^{}]+)\}\{([^{}]+)\}/g,
            '<span class="katex-frac"><span class="katex-num">$1</span>/<span class="katex-den">$2</span></span>',
        );
        s = s.replace(/\\sqrt\{([^{}]+)\}/g, '√($1)');
        s = s.replace(/\^\{([^{}]+)\}/g, '<sup>$1</sup>');
        s = s.replace(/\^([a-zA-Z0-9+-])/g, '<sup>$1</sup>');
        s = s.replace(/_\{([^{}]+)\}/g, '<sub>$1</sub>');
        s = s.replace(/_([a-zA-Z0-9+-])/g, '<sub>$1</sub>');
        for (const [cmd, glyph] of Object.entries(LATEX_SYMBOLS)) {
            s = s.split(cmd).join(glyph);
        }
        return s;
    }

    if (typeof globalScope.katex === 'undefined') {
        globalScope.katex = {
            renderToString(expr, opts = {}) {
                const tag = opts.displayMode ? 'div' : 'span';
                const cls = opts.displayMode ? 'katex katex-display' : 'katex katex-inline';
                return `<${tag} class="${cls}"><span class="katex-mathml">${formatLatexExpression(expr)}</span></${tag}>`;
            },
        };
    }
    if (typeof globalScope.renderMathInElement === 'undefined') {
        globalScope.renderMathInElement = function renderMathInElement(element) {
            if (!element || typeof element.innerHTML !== 'string') return;
            element.innerHTML = element.innerHTML
                .replace(/\$\$([\s\S]+?)\$\$/g, (_m, tex) =>
                    globalScope.katex.renderToString(tex, { displayMode: true }),
                )
                .replace(/\$([^$\n]+?)\$/g, (_m, tex) =>
                    globalScope.katex.renderToString(tex, { displayMode: false }),
                );
        };
    }

    // 3. Offline Code Editor & Line-by-Line Diff Shim (`monaco` + AMD `require` compatible API)
    function computeUnifiedLineDiff(origText, modText) {
        const a = String(origText ?? '').split('\n');
        const b = String(modText ?? '').split('\n');
        const n = a.length;
        const m = b.length;
        const maxMatrixCells = 160000;

        if (n * m > maxMatrixCells) {
            const out = [];
            const maxLen = Math.max(n, m);
            for (let i = 0; i < maxLen; i++) {
                if (i < n && i < m && a[i] === b[i]) {
                    out.push({ type: 'same', text: a[i], origLine: i + 1, modLine: i + 1 });
                } else {
                    if (i < n) out.push({ type: 'del', text: a[i], origLine: i + 1, modLine: null });
                    if (i < m) out.push({ type: 'ins', text: b[i], origLine: null, modLine: i + 1 });
                }
            }
            return out;
        }

        const dp = Array.from({ length: n + 1 }, () => new Int32Array(m + 1));
        for (let i = n - 1; i >= 0; i--) {
            for (let j = m - 1; j >= 0; j--) {
                if (a[i] === b[j]) {
                    dp[i][j] = dp[i + 1][j + 1] + 1;
                } else {
                    dp[i][j] = Math.max(dp[i + 1][j], dp[i][j + 1]);
                }
            }
        }

        const diff = [];
        let i = 0;
        let j = 0;
        while (i < n && j < m) {
            if (a[i] === b[j]) {
                diff.push({ type: 'same', text: a[i], origLine: i + 1, modLine: j + 1 });
                i++;
                j++;
            } else if (dp[i + 1][j] >= dp[i][j + 1]) {
                diff.push({ type: 'del', text: a[i], origLine: i + 1, modLine: null });
                i++;
            } else {
                diff.push({ type: 'ins', text: b[j], origLine: null, modLine: j + 1 });
                j++;
            }
        }
        while (i < n) {
            diff.push({ type: 'del', text: a[i], origLine: i + 1, modLine: null });
            i++;
        }
        while (j < m) {
            diff.push({ type: 'ins', text: b[j], origLine: null, modLine: j + 1 });
            j++;
        }
        return diff;
    }

    if (typeof globalScope.monaco === 'undefined') {
        function createOfflineModel(initialValue = '', language = 'plaintext') {
            let currentValue = String(initialValue ?? '');
            const listeners = [];
            return {
                language: String(language || 'plaintext'),
                getLanguageId() {
                    return this.language;
                },
                getValue() {
                    return currentValue;
                },
                setValue(next) {
                    currentValue = String(next ?? '');
                    for (const fn of listeners) {
                        try {
                            fn();
                        } catch (_e) {}
                    }
                },
                onDidChangeContent(fn) {
                    if (typeof fn === 'function') listeners.push(fn);
                },
            };
        }

        globalScope.monaco = {
            editor: {
                createModel(value, language) {
                    return createOfflineModel(value, language);
                },
                setModelLanguage(model, languageId) {
                    if (model && typeof model === 'object') {
                        model.language = String(languageId || 'plaintext');
                    }
                },
                getModelLanguage(model) {
                    return model && model.language ? model.language : 'plaintext';
                },
                setTheme() {},
                defineTheme() {},
                create(container, options = {}) {
                    let model = createOfflineModel(options.value || '', options.language || 'plaintext');
                    const changeListeners = [];
                    let textarea = null;
                    if (container && typeof container.appendChild === 'function' && globalScope.document) {
                        container.innerHTML = '';
                        textarea = globalScope.document.createElement('textarea');
                        textarea.className = 'uor-offline-code-editor';
                        textarea.style.cssText =
                            'width:100%;height:100%;background:#050608;color:#e2e8f0;border:none;outline:none;padding:12px;font-family:var(--font-mono,monospace);font-size:12px;line-height:1.5;resize:none;';
                        textarea.value = model.getValue();
                        textarea.addEventListener('input', () => {
                            model.setValue(textarea.value);
                            for (const cb of changeListeners) {
                                try {
                                    cb();
                                } catch (_e) {}
                            }
                        });
                        container.appendChild(textarea);
                    }
                    model.onDidChangeContent(() => {
                        if (textarea && textarea.value !== model.getValue()) {
                            textarea.value = model.getValue();
                        }
                    });
                    return {
                        getValue() {
                            return model.getValue();
                        },
                        setValue(val) {
                            model.setValue(val);
                            if (textarea) textarea.value = model.getValue();
                        },
                        getModel() {
                            return model;
                        },
                        setModel(newModel) {
                            if (newModel) {
                                model = newModel;
                                if (textarea) textarea.value = model.getValue();
                            }
                        },
                        onDidChangeModelContent(cb) {
                            if (typeof cb === 'function') changeListeners.push(cb);
                        },
                        layout() {},
                    };
                },
                createDiffEditor(container) {
                    let currentDiff = { original: null, modified: null };
                    let lastLineChanges = [];
                    return {
                        setModel(diffModels) {
                            currentDiff = diffModels || { original: null, modified: null };
                            const orig = currentDiff.original ? currentDiff.original.getValue() : '';
                            const mod = currentDiff.modified ? currentDiff.modified.getValue() : '';
                            lastLineChanges = computeUnifiedLineDiff(orig, mod);
                            if (container) {
                                const renderedLines = lastLineChanges.map((entry) => {
                                    if (entry.type === 'del') {
                                        return `<div class="uor-diff-del" style="background:rgba(239,68,68,0.16);color:#fca5a5;">- ${escapeHtml(entry.text)}</div>`;
                                    }
                                    if (entry.type === 'ins') {
                                        return `<div class="uor-diff-ins" style="background:rgba(16,185,129,0.16);color:#6ee7b7;">+ ${escapeHtml(entry.text)}</div>`;
                                    }
                                    return `<div class="uor-diff-same" style="color:#cbd5e1;">  ${escapeHtml(entry.text)}</div>`;
                                });
                                container.innerHTML = `<pre style="margin:0;padding:12px;color:#e2e8f0;font-family:var(--font-mono,monospace);font-size:12px;overflow:auto;height:100%;"><div style="color:#94a3b8;margin-bottom:8px;">--- original (${orig.length} B)\n+++ modified (${mod.length} B)</div>${renderedLines.join('')}</pre>`;
                            }
                        },
                        getModel() {
                            return currentDiff;
                        },
                        getLineChanges() {
                            return lastLineChanges;
                        },
                        layout() {},
                    };
                },
            },
        };
    }

    if (typeof globalScope.require === 'undefined') {
        const amdRequire = function amdRequire(_deps, callback) {
            if (typeof callback === 'function') {
                setTimeout(() => callback(), 0);
            }
        };
        amdRequire.config = function config() {};
        globalScope.require = amdRequire;
    }

    // 4. Offline 2D-Canvas Software 3D Projection Shim (`THREE` compatible API)
    if (typeof globalScope.THREE === 'undefined') {
        class Vector3 {
            constructor(x = 0, y = 0, z = 0) {
                this.x = Number(x) || 0;
                this.y = Number(y) || 0;
                this.z = Number(z) || 0;
            }
            set(x, y, z) {
                this.x = Number(x) || 0;
                this.y = Number(y) || 0;
                this.z = Number(z) || 0;
                return this;
            }
            copy(v) {
                if (v) {
                    this.x = v.x || 0;
                    this.y = v.y || 0;
                    this.z = v.z || 0;
                }
                return this;
            }
            clone() {
                return new Vector3(this.x, this.y, this.z);
            }
            distanceTo(v) {
                const dx = this.x - (v?.x || 0);
                const dy = this.y - (v?.y || 0);
                const dz = this.z - (v?.z || 0);
                return Math.sqrt(dx * dx + dy * dy + dz * dz);
            }
            lerpVectors(v1, v2, alpha) {
                const t = Number(alpha) || 0;
                this.x = v1.x + (v2.x - v1.x) * t;
                this.y = v1.y + (v2.y - v1.y) * t;
                this.z = v1.z + (v2.z - v1.z) * t;
                return this;
            }
        }

        class Color {
            constructor(hex = 0xffffff) {
                this.r = 1;
                this.g = 1;
                this.b = 1;
                if (typeof hex === 'number') {
                    this.setHex(hex);
                }
            }
            setHex(hex) {
                const n = Number(hex) >>> 0;
                this.r = ((n >> 16) & 255) / 255;
                this.g = ((n >> 8) & 255) / 255;
                this.b = (n & 255) / 255;
                return this;
            }
        }

        class BufferAttribute {
            constructor(array, itemSize) {
                this.array = array || new Float32Array(0);
                this.itemSize = itemSize || 3;
                this.needsUpdate = false;
            }
        }

        class BufferGeometry {
            constructor() {
                this.attributes = {};
            }
            setAttribute(name, attribute) {
                this.attributes[name] = attribute;
                return this;
            }
            getAttribute(name) {
                return this.attributes[name];
            }
            dispose() {}
        }

        class Object3D {
            constructor() {
                this.position = new Vector3(0, 0, 0);
                this.scale = new Vector3(1, 1, 1);
                this.rotation = new Vector3(0, 0, 0);
                this.visible = true;
                this.children = [];
            }
            add(child) {
                if (child) this.children.push(child);
                return this;
            }
            remove(child) {
                const idx = this.children.indexOf(child);
                if (idx !== -1) this.children.splice(idx, 1);
                return this;
            }
        }

        class Scene extends Object3D {}
        class Group extends Object3D {}
        class AmbientLight extends Object3D {
            constructor(color = 0xffffff, intensity = 1) {
                super();
                this.color = new Color(color);
                this.intensity = intensity;
            }
        }
        class DirectionalLight extends AmbientLight {}
        class PointLight extends AmbientLight {
            constructor(color = 0xffffff, intensity = 1, distance = 0) {
                super(color, intensity);
                this.distance = distance;
            }
        }

        class PointsMaterial {
            constructor(opts = {}) {
                Object.assign(this, opts);
            }
        }
        class LineBasicMaterial {
            constructor(opts = {}) {
                Object.assign(this, opts);
            }
        }
        class MeshBasicMaterial {
            constructor(opts = {}) {
                Object.assign(this, opts);
            }
        }
        class CanvasTexture {
            constructor(canvas) {
                this.image = canvas;
                this.needsUpdate = false;
            }
        }

        class Points extends Object3D {
            constructor(geometry, material) {
                super();
                this.isPoints = true;
                this.geometry = geometry || new BufferGeometry();
                this.material = material || new PointsMaterial();
            }
        }
        class LineSegments extends Object3D {
            constructor(geometry, material) {
                super();
                this.isLineSegments = true;
                this.geometry = geometry || new BufferGeometry();
                this.material = material || new LineBasicMaterial();
            }
        }
        class Line extends Object3D {
            constructor(geometry, material) {
                super();
                this.isLine = true;
                this.geometry = geometry || new BufferGeometry();
                this.material = material || new LineBasicMaterial();
            }
        }
        class Mesh extends Object3D {
            constructor(geometry, material) {
                super();
                this.isMesh = true;
                this.geometry = geometry || new BufferGeometry();
                this.material = material || new MeshBasicMaterial();
            }
        }

        class PerspectiveCamera extends Object3D {
            constructor(fov = 45, aspect = 1, near = 0.1, far = 100) {
                super();
                this.fov = fov;
                this.aspect = aspect || 1;
                this.near = near;
                this.far = far;
            }
            updateProjectionMatrix() {}
        }

        class OrbitControls {
            constructor(camera, domElement) {
                this.object = camera;
                this.domElement = domElement;
                this.enableDamping = true;
                this.dampingFactor = 0.06;
                this.rotateSpeed = 0.8;
                this.zoomSpeed = 0.9;
                this.autoRotate = false;
                this.autoRotateSpeed = 0.5;
                this.minDistance = 1.0;
                this.maxDistance = 10.0;
                this._angle = 0;
            }
            update() {
                if (this.autoRotate) {
                    this._angle += 0.004 * (this.autoRotateSpeed || 0.5);
                }
            }
            reset() {
                this._angle = 0;
            }
        }

        class WebGLRenderer {
            constructor(opts = {}) {
                this.domElement =
                    opts.canvas ||
                    (globalScope.document ? globalScope.document.createElement('canvas') : { width: 320, height: 240 });
                this._width = this.domElement.clientWidth || this.domElement.width || 320;
                this._height = this.domElement.clientHeight || this.domElement.height || 240;
                this._pixelRatio = 1;
                this.renderCount = 0;
            }
            setPixelRatio(r) {
                this._pixelRatio = Number(r) || 1;
            }
            setSize(w, h) {
                this._width = Math.max(1, Number(w) || 320);
                this._height = Math.max(1, Number(h) || 240);
                if (this.domElement) {
                    this.domElement.width = this._width;
                    this.domElement.height = this._height;
                }
            }
            render(scene, camera) {
                this.renderCount++;
                if (!this.domElement || typeof this.domElement.getContext !== 'function') return;
                const ctx = this.domElement.getContext('2d');
                if (!ctx || typeof ctx.clearRect !== 'function') return;
                const w = this._width;
                const h = this._height;
                ctx.clearRect(0, 0, w, h);
                if (!scene || !Array.isArray(scene.children)) return;
                const camZ = (camera && camera.position && camera.position.z) || 3.0;
                const project = (x, y, z, scale) => {
                    const sx = x * (scale?.x || 1);
                    const sy = y * (scale?.y || 1);
                    const sz = z * (scale?.z || 1);
                    const depth = Math.max(0.2, camZ - sz);
                    const focal = Math.min(w, h) * 0.85;
                    return {
                        px: w * 0.5 + (sx / depth) * focal,
                        py: h * 0.5 - (sy / depth) * focal,
                    };
                };
                for (const obj of scene.children) {
                    if (!obj || obj.visible === false || !obj.geometry) continue;
                    const posAttr = obj.geometry.attributes && obj.geometry.attributes.position;
                    if (!posAttr || !posAttr.array) continue;
                    const arr = posAttr.array;
                    if (obj.isPoints && typeof ctx.fillRect === 'function') {
                        ctx.fillStyle = 'rgba(0, 216, 255, 0.75)';
                        const step = Math.max(3, Math.floor(arr.length / 600) * 3);
                        for (let i = 0; i + 2 < arr.length; i += step) {
                            const { px, py } = project(arr[i], arr[i + 1], arr[i + 2], obj.scale);
                            ctx.fillRect(px - 1, py - 1, 2, 2);
                        }
                    } else if ((obj.isLineSegments || obj.isLine) && typeof ctx.beginPath === 'function') {
                        ctx.strokeStyle = 'rgba(56, 189, 248, 0.28)';
                        ctx.lineWidth = 1;
                        ctx.beginPath();
                        const step = Math.max(6, Math.floor(arr.length / 300) * 6);
                        for (let i = 0; i + 5 < arr.length; i += step) {
                            const p0 = project(arr[i], arr[i + 1], arr[i + 2], obj.scale);
                            const p1 = project(arr[i + 3], arr[i + 4], arr[i + 5], obj.scale);
                            ctx.moveTo(p0.px, p0.py);
                            ctx.lineTo(p1.px, p1.py);
                        }
                        ctx.stroke();
                    }
                }
            }
        }

        globalScope.THREE = {
            AdditiveBlending: 2,
            Vector3,
            Color,
            BufferAttribute,
            Float32BufferAttribute: BufferAttribute,
            BufferGeometry,
            Object3D,
            Scene,
            Group,
            AmbientLight,
            DirectionalLight,
            PointLight,
            PointsMaterial,
            LineBasicMaterial,
            MeshBasicMaterial,
            CanvasTexture,
            Points,
            LineSegments,
            Line,
            Mesh,
            PerspectiveCamera,
            OrbitControls,
            WebGLRenderer,
        };
    }

    // 5. Offline PDF Text Helper (`pdfjsLib` compatible notice shim)
    if (typeof globalScope.pdfjsLib === 'undefined') {
        globalScope.pdfjsLib = {
            GlobalWorkerOptions: { workerSrc: './assets/js/uor_offline_vendor.js' },
            getDocument(source = {}) {
                return {
                    promise: Promise.resolve({
                        numPages: 1,
                        async getPage() {
                            const rawBytes = source.data ? new Uint8Array(source.data) : new Uint8Array(0);
                            const text = new TextDecoder('utf-8', { fatal: false }).decode(rawBytes);
                            const matches = [];
                            const re = /\(([^()\\]{2,200})\)/g;
                            let m;
                            while ((m = re.exec(text)) !== null && matches.length < 64) {
                                matches.push({ str: m[1] });
                            }
                            if (matches.length === 0) {
                                matches.push({
                                    str: '[Offline PDF notice: binary PDF stream/font rasterization is not bundled in offline mode.]',
                                });
                            }
                            return {
                                async getTextContent() {
                                    return { items: matches };
                                },
                            };
                        },
                    }),
                };
            },
        };
    }

    // 6. Loopback HTTP Bridge for `uor-desktop-client --serve-studio`
    let rpcSeq = 1;
    globalScope.createLoopbackTauriCore = function createLoopbackTauriCore(endpoint = '/__tauri_invoke') {
        return {
            async invoke(command, args = {}) {
                const id = rpcSeq++;
                const token = String(globalScope.UOR_DESKTOP_SESSION_TOKEN || '');
                const response = await fetch(endpoint, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'X-UOR-Desktop-Token': token,
                    },
                    body: JSON.stringify({ id, command, args }),
                });
                const payload = await response.json();
                if (!payload || !payload.ok) {
                    throw new Error((payload && payload.error) || `Desktop RPC failed for ${command}`);
                }
                return payload.result;
            },
        };
    };

    if (
        typeof globalScope.__TAURI__ === 'undefined' &&
        (globalScope.UOR_ENABLE_LOOPBACK_TAURI === true || globalScope.UOR_DESKTOP_OFFLINE === true)
    ) {
        globalScope.__TAURI__ = {
            core: globalScope.createLoopbackTauriCore(),
        };
    }

    // 7. Native Desktop Model Worker Bridge
    // Connects the Studio `modelWorker` protocol (`prewarm`, `load_model`, `generate`,
    // `export_session`, `import_session`, `reset_session`, `close_session`,
    // `save_session_durable`, `restore_session_durable`) directly to `window.__TAURI__.core.invoke`
    // so both web and desktop clients share one identical UI interface.
    globalScope.createTauriNativeModelWorker = function createTauriNativeModelWorker(tauriCore) {
        const listeners = [];
        let capabilities = null;
        let activeJob = null;

        function emit(data) {
            const event = { data };
            if (typeof workerShim.onmessage === 'function') {
                workerShim.onmessage(event);
            }
            for (const fn of [...listeners]) {
                try {
                    fn(event);
                } catch (_e) {}
            }
        }

        function encodePromptTokens(payload, vocabSize) {
            if (Array.isArray(payload.tokenIds) && payload.tokenIds.length > 0) {
                return payload.tokenIds.map((t) => Number(t));
            }
            const userMessage = (payload.messages || []).filter((m) => m.role === 'user').pop();
            const text = typeof userMessage?.content === 'string' ? userMessage.content.trim() : '';
            if (!text) {
                throw new Error('A non-empty prompt or tokenIds array is required.');
            }
            const rawBytes = new TextEncoder().encode(text);
            const out = new Array(rawBytes.length);
            for (let i = 0; i < rawBytes.length; i++) {
                out[i] = rawBytes[i] % vocabSize;
            }
            return out;
        }

        const workerShim = {
            isTauriNativeBridge: true,
            onmessage: null,
            addEventListener(type, fn) {
                if (type === 'message' && typeof fn === 'function') {
                    listeners.push(fn);
                }
            },
            removeEventListener(type, fn) {
                if (type === 'message') {
                    const idx = listeners.indexOf(fn);
                    if (idx !== -1) listeners.splice(idx, 1);
                }
            },
            async postMessage(message) {
                const { action, id, payload = {} } = message || {};
                try {
                    if (action === 'cancel' || action === 'stop_generation') {
                        if (activeJob) activeJob.cancelled = true;
                        return;
                    }
                    if (action === 'get_storage_status') {
                        emit({
                            action: 'storage_status_result',
                            storage: { usageMB: '0.0', quotaMB: '64.0' },
                        });
                        return;
                    }
                    if (action === 'prewarm') {
                        emit({
                            action: 'runtime_loaded',
                            engine: 'uor-r4-integer/native-desktop',
                            ready: Boolean(capabilities),
                            capabilities,
                        });
                        return;
                    }
                    if (action === 'load_model' || action === 'load_local_model') {
                        if (activeJob) {
                            throw new Error('Stop generation before loading a stack artifact.');
                        }
                        let caps;
                        const filePath = payload.filePath || payload.path;
                        if (filePath) {
                            caps = await tauriCore.invoke('native_load_model', {
                                filePath,
                                expectedSha256: payload.expectedSha256 || null,
                            });
                        } else if (payload.bytes) {
                            const raw = payload.bytes instanceof Uint8Array
                                ? payload.bytes
                                : new Uint8Array(payload.bytes);
                            caps = await tauriCore.invoke('native_load_model', {
                                bytes: Array.from(raw),
                                fileName: payload.fileName || 'synthetic-stack.lut',
                                expectedSha256: payload.expectedSha256 || null,
                            });
                        } else {
                            caps = await tauriCore.invoke('native_load_synthetic_standin', {});
                        }
                        capabilities = {
                            ...caps,
                            alloc_events: 1,
                            leaked_models: 0,
                        };
                        emit({
                            action: 'model_loaded',
                            capabilities,
                            bytes: capabilities.artifact_bytes,
                        });
                        return;
                    }
                    if (!capabilities) {
                        throw new Error('Load a UORLUT01 stack file (.lut) before running a session command.');
                    }
                    if (action === 'close_session' || action === 'reset_session') {
                        if (activeJob) {
                            throw new Error('Stop generation before closing or resetting a session.');
                        }
                        const convId = payload.conversationId || 'default';
                        await tauriCore.invoke('native_reset_session', { conversationId: convId });
                        emit({
                            action: 'session_reset',
                            id,
                            conversationId: convId,
                        });
                        return;
                    }
                    if (action === 'export_session') {
                        if (activeJob) {
                            throw new Error('Stop generation before exporting a session checkpoint.');
                        }
                        const convId = payload.conversationId || 'default';
                        const res = await tauriCore.invoke('native_export_session', { conversationId: convId });
                        emit({
                            action: 'session_exported',
                            id,
                            conversationId: convId,
                            bytes: new Uint8Array(res.bytes || []),
                            lastToken: res.lastToken ?? null,
                        });
                        return;
                    }
                    if (action === 'import_session') {
                        if (activeJob) {
                            throw new Error('Stop generation before importing a session checkpoint.');
                        }
                        const convId = payload.conversationId;
                        const raw = payload.bytes instanceof Uint8Array
                            ? payload.bytes
                            : new Uint8Array(payload.bytes || []);
                        const res = await tauriCore.invoke('native_import_session', {
                            conversationId: convId,
                            bytes: Array.from(raw),
                            lastToken: payload.lastToken ?? null,
                        });
                        emit({
                            action: 'session_imported',
                            id,
                            conversationId: convId,
                            bytes: res.bytes,
                        });
                        return;
                    }
                    if (action === 'save_session_durable') {
                        const convId = payload.conversationId || 'default';
                        const receipt = await tauriCore.invoke('native_save_session_durable', {
                            userId: payload.userId || 'local-user',
                            projectId: payload.projectId || 'studio',
                            conversationId: convId,
                            storeDir: payload.storeDir || null,
                        });
                        emit({
                            action: 'session_durable_saved',
                            id,
                            conversationId: convId,
                            receipt,
                        });
                        return;
                    }
                    if (action === 'restore_session_durable') {
                        const convId = payload.conversationId || 'default';
                        const receipt = await tauriCore.invoke('native_restore_session_durable', {
                            userId: payload.userId || 'local-user',
                            projectId: payload.projectId || 'studio',
                            conversationId: convId,
                            storeDir: payload.storeDir || null,
                            manifestBytes: payload.manifestBytes ? Array.from(payload.manifestBytes) : null,
                        });
                        emit({
                            action: 'session_durable_restored',
                            id,
                            conversationId: convId,
                            receipt,
                        });
                        return;
                    }
                    if (action !== 'generate') {
                        throw new Error(`Unknown worker action: ${action}`);
                    }
                    if (activeJob) {
                        throw new Error('Another generation is active. Wait for its completion or cancellation.');
                    }
                    const convId = payload.conversationId || 'default';
                    const promptTokens = encodePromptTokens(payload, capabilities.shape.vocab);
                    const maxNewTokens = Math.min(
                        4096,
                        Math.max(1, Number(payload.options?.max_new_tokens) || 32),
                    );
                    const job = { id, conversationId: convId, cancelled: false };
                    activeJob = job;
                    const t0 = Date.now();
                    try {
                        const allTokenIds = [];
                        let fullText = '';
                        let nextInputTokens = promptTokens;
                        let lastReceipt = null;

                        for (let stepIdx = 0; stepIdx < maxNewTokens; stepIdx++) {
                            if (job.cancelled) break;
                            const out = await tauriCore.invoke('native_step_tokens', {
                                conversationId: convId,
                                tokenIds: nextInputTokens,
                                maxNewTokens: 1,
                            });
                            lastReceipt = out.receipt;
                            const ev = (out.events && out.events[0]) || {
                                token_id: out.receipt.token_ids[0],
                                margin_q16: 0,
                                position: stepIdx + 1,
                            };
                            const tok = ev.token_id;
                            allTokenIds.push(tok);
                            const chunk = stepIdx === 0 ? String(tok) : ` ${tok}`;
                            fullText += chunk;
                            nextInputTokens = [tok];

                            const elapsedSec = Math.max(0.001, (Date.now() - t0) / 1000);
                            emit({
                                action: 'stream_token',
                                id,
                                chunk,
                                fullText,
                                tokenId: tok,
                                marginQ16: ev.margin_q16,
                                stepCount: ev.position,
                                tokenCount: allTokenIds.length,
                                tps: (allTokenIds.length / elapsedSec).toFixed(1),
                            });
                        }

                        const durationSec = Math.max(0.001, (Date.now() - t0) / 1000);
                        emit({
                            action: 'generate_complete',
                            id,
                            fullText,
                            tokenIds: allTokenIds,
                            tokenCount: allTokenIds.length,
                            durationSec,
                            tps: (allTokenIds.length / durationSec).toFixed(1),
                            stoppedBy: job.cancelled ? 'cancelled' : 'length',
                            weightsPerToken: lastReceipt
                                ? lastReceipt.weights_per_token
                                : capabilities.weights_per_token,
                            allocDelta: 0,
                            allocEvents: 1,
                            statusLabel: lastReceipt
                                ? lastReceipt.status_label
                                : capabilities.status_label,
                            tokenizerLabel: lastReceipt
                                ? lastReceipt.tokenizer_label
                                : capabilities.tokenizer_label,
                        });
                    } finally {
                        if (activeJob === job) activeJob = null;
                    }
                } catch (error) {
                    emit({
                        action: action === 'generate' ? 'generate_error' : 'operation_error',
                        id,
                        error: String(error),
                    });
                }
            },
        };
        return workerShim;
    };
})(typeof globalThis !== 'undefined' ? globalThis : window);
