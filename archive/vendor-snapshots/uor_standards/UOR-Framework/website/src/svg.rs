//! Ontology-derived SVG diagram generators.
//!
//! All diagrams are deterministic inline SVGs generated from `Ontology::full()`.
//! No external SVG library is used — pure Rust string formatting.
//! Every `<svg>` contains a `<title>` child (WCAG 4.1.2 compliance).
//! Colors use CSS custom property references with hex fallbacks for print/static rendering.

use std::collections::HashMap;

use uor_ontology::{
    model::{IndividualValue, NamespaceModule, Space},
    Ontology,
};

use crate::model::NamespaceSummary;

// ── Layout geometry constants ────────────────────────────────────────────────

const SVG_MARGIN: f64 = 40.0;
const SVG_LAYER_SPACING: f64 = 180.0;
const SVG_NODE_W: f64 = 140.0;
const SVG_NODE_H: f64 = 40.0;
const SVG_NODE_VGAP: f64 = 60.0;
const SVG_CLASS_NODE_W: f64 = 130.0;
const SVG_CLASS_ROW_H: f64 = 60.0;
const SVG_BAR_H: f64 = 28.0;
const SVG_BAR_GAP: f64 = 8.0;
const SVG_LABEL_W: f64 = 200.0;
const SVG_BAR_MAX_W: f64 = 460.0;
/// Stage box width in the pipeline diagram.
const SVG_STAGE_W: f64 = 200.0;
/// Stage box height.
const SVG_STAGE_H: f64 = 100.0;
/// Gap between stage boxes.
const SVG_STAGE_GAP: f64 = 60.0;
/// Quantum level box width.
const SVG_LEVEL_W: f64 = 90.0;
/// Witt level box height.
const SVG_LEVEL_H: f64 = 60.0;
/// Gap between Witt level boxes.
const SVG_LEVEL_GAP: f64 = 20.0;

// ── Color helpers ────────────────────────────────────────────────────────────

fn space_fill(space: &str) -> &'static str {
    match space {
        "kernel" => "#7c3aed",
        "bridge" => "#059669",
        "user" => "#0891b2",
        _ => "#6b7280",
    }
}

fn stage_color(stage_id: &str) -> &'static str {
    match stage_id {
        "stage-define" => "#7c3aed",
        "stage-resolve" => "#059669",
        "stage-certify" => "#dc2626",
        _ => "#6b7280",
    }
}

// ── SVG attribute helpers (avoid raw-string `"#` termination) ───────────────

/// Renders an SVG `fill` attribute value: `fill="<color>"`.
fn fill_attr(color: &str) -> String {
    format!("fill=\"{}\"", color)
}

/// Renders an SVG `stroke` attribute value.
fn stroke_attr(color: &str) -> String {
    format!("stroke=\"{}\"", color)
}

// ── 1. PRISM Pipeline SVG ────────────────────────────────────────────────────

/// Renders a PRISM pipeline diagram as inline SVG.
///
/// Stage definitions are driven by `crate::pipeline::PRISM_STAGES`.
pub fn render_prism_pipeline_svg(summaries: &[NamespaceSummary]) -> String {
    use crate::pipeline::PRISM_STAGES;

    let stage_count = PRISM_STAGES.len() as f64;
    let svg_w = SVG_MARGIN * 2.0 + stage_count * SVG_STAGE_W + (stage_count - 1.0) * SVG_STAGE_GAP;
    let svg_h = SVG_MARGIN * 2.0 + SVG_STAGE_H + 60.0;

    let mut out = format!(
        "<svg class=\"prism-pipeline\" viewBox=\"0 0 {w:.0} {h:.0}\" \
         xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"prism-title\">\n\
         <title id=\"prism-title\">PRISM Pipeline: Define, Resolve, Certify</title>\n\
         <defs>\n\
         <marker id=\"arr\" markerWidth=\"8\" markerHeight=\"8\" refX=\"6\" refY=\"3\" \
         orient=\"auto\"><path d=\"M0,0 L0,6 L8,3 z\" {af}/></marker>\n\
         </defs>\n",
        w = svg_w,
        h = svg_h,
        af = fill_attr("#9ca3af"),
    );

    for (i, (name, section_id, match_key, is_prefix)) in PRISM_STAGES.iter().enumerate() {
        let x = SVG_MARGIN + i as f64 * (SVG_STAGE_W + SVG_STAGE_GAP);
        let y = SVG_MARGIN;
        let color = stage_color(section_id);
        let cx = x + SVG_STAGE_W / 2.0;

        let ns_count = summaries
            .iter()
            .filter(|s| {
                if *is_prefix {
                    s.prefix == *match_key
                } else {
                    s.space == *match_key
                }
            })
            .count();

        if i > 0 {
            let arrow_x1 = x - SVG_STAGE_GAP + 4.0;
            let arrow_x2 = x - 4.0;
            let mid_y = y + SVG_STAGE_H / 2.0;
            out.push_str(&format!(
                "<line x1=\"{ax1:.0}\" y1=\"{ay:.0}\" x2=\"{ax2:.0}\" y2=\"{ay:.0}\" \
                 {sc} stroke-width=\"2\" marker-end=\"url(#arr)\" class=\"pipeline-flow\"/>\n",
                ax1 = arrow_x1,
                ay = mid_y,
                ax2 = arrow_x2,
                sc = stroke_attr("#9ca3af"),
            ));
        }

        let pl = if ns_count == 1 { "" } else { "s" };
        out.push_str(&format!(
            "<rect x=\"{x:.0}\" y=\"{y:.0}\" width=\"{w:.0}\" height=\"{h:.0}\" rx=\"6\" \
             {fc} opacity=\"0.92\"/>\n\
             <text x=\"{cx:.0}\" y=\"{ty:.0}\" text-anchor=\"middle\" {wf} \
             font-size=\"16\" font-weight=\"700\" \
             font-family=\"system-ui,sans-serif\">{name}</text>\n\
             <text x=\"{cx:.0}\" y=\"{sy:.0}\" text-anchor=\"middle\" {wf} \
             font-size=\"12\" font-family=\"system-ui,sans-serif\" opacity=\"0.85\">\
             {ns_count} namespace{pl}</text>\n",
            w = SVG_STAGE_W,
            h = SVG_STAGE_H,
            fc = fill_attr(color),
            ty = y + SVG_STAGE_H / 2.0 - 8.0,
            sy = y + SVG_STAGE_H / 2.0 + 12.0,
            wf = fill_attr("#fff"),
        ));
    }

    out.push_str("</svg>");
    out
}

/// Wrapper for use in concept page SVG hooks (takes `&Ontology` like other hooks).
pub fn render_prism_pipeline_svg_for_concept(ontology: &Ontology) -> String {
    let summaries: Vec<NamespaceSummary> = ontology
        .namespaces
        .iter()
        .map(|m| NamespaceSummary {
            prefix: m.namespace.prefix.to_string(),
            label: m.namespace.label.to_string(),
            comment: m.namespace.comment.to_string(),
            iri: m.namespace.iri.to_string(),
            space: format!("{:?}", m.namespace.space).to_lowercase(),
            url: format!("/namespaces/{}/", m.namespace.prefix),
            class_count: m.classes.len(),
            property_count: m.properties.len(),
            individual_count: m.individuals.len(),
        })
        .collect();
    render_prism_pipeline_svg(&summaries)
}

// ── 2. Namespace Dependency Graph SVG ────────────────────────────────────────

/// Renders the namespace import dependency graph as a layered inline SVG.
///
/// Layer positions are computed via Kahn's BFS topological sort from
/// `namespace.imports` — no hard-coded prefix names or pixel positions.
pub fn render_namespace_dependency_graph_svg(ontology: &Ontology, base_path: &str) -> String {
    let all_prefixes: Vec<&str> = ontology
        .namespaces
        .iter()
        .map(|m| m.namespace.prefix)
        .collect();

    let mut deps: HashMap<&str, Vec<&str>> = HashMap::new();
    for m in &ontology.namespaces {
        let imported: Vec<&str> = m
            .namespace
            .imports
            .iter()
            .filter_map(|iri| {
                all_prefixes
                    .iter()
                    .find(|&&p| iri.contains(&format!("/{p}/")))
                    .copied()
            })
            .collect();
        deps.insert(m.namespace.prefix, imported);
    }

    let layer_of = kahn_layers(&all_prefixes, &deps);
    let num_layers = layer_of.values().copied().max().unwrap_or(0) + 1;
    let mut layers: Vec<Vec<&str>> = vec![Vec::new(); num_layers];
    for prefix in &all_prefixes {
        let layer = *layer_of.get(prefix).unwrap_or(&0);
        layers[layer].push(prefix);
    }

    let max_per_layer = layers.iter().map(|l| l.len()).max().unwrap_or(1);
    let svg_w =
        SVG_MARGIN * 2.0 + num_layers as f64 * (SVG_NODE_W + SVG_LAYER_SPACING) - SVG_LAYER_SPACING;
    let svg_h = SVG_MARGIN * 2.0 + max_per_layer as f64 * SVG_NODE_VGAP + SVG_NODE_H;

    let mut out = format!(
        "<svg class=\"ns-dependency-graph\" viewBox=\"0 0 {w:.0} {h:.0}\" \
         xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"nsdep-title\">\n\
         <title id=\"nsdep-title\">Namespace dependency graph — {n} namespaces</title>\n\
         <defs>\n\
         <marker id=\"dep-arr\" markerWidth=\"8\" markerHeight=\"8\" refX=\"6\" refY=\"3\" \
         orient=\"auto\"><path d=\"M0,0 L0,6 L8,3 z\" {af}/></marker>\n\
         </defs>\n",
        w = svg_w,
        h = svg_h,
        n = all_prefixes.len(),
        af = fill_attr("#6b7280"),
    );

    let mut cx_of: HashMap<&str, f64> = HashMap::new();
    let mut cy_of: HashMap<&str, f64> = HashMap::new();

    for (li, layer) in layers.iter().enumerate() {
        let total_h = layer.len() as f64 * SVG_NODE_VGAP - (SVG_NODE_VGAP - SVG_NODE_H);
        let start_y = (svg_h - total_h) / 2.0;
        for (ni, prefix) in layer.iter().enumerate() {
            let cx = SVG_MARGIN + li as f64 * (SVG_NODE_W + SVG_LAYER_SPACING) + SVG_NODE_W / 2.0;
            let cy = start_y + ni as f64 * SVG_NODE_VGAP + SVG_NODE_H / 2.0;
            cx_of.insert(prefix, cx);
            cy_of.insert(prefix, cy);
        }
    }

    // Edges first (behind nodes)
    for m in &ontology.namespaces {
        let prefix = m.namespace.prefix;
        if let (Some(&fx), Some(&fy)) = (cx_of.get(prefix), cy_of.get(prefix)) {
            for import_iri in m.namespace.imports {
                let found = all_prefixes
                    .iter()
                    .find(|&&p| import_iri.contains(&format!("/{p}/")));
                if let Some(&to_prefix) = found {
                    if let (Some(&tx), Some(&ty)) = (cx_of.get(to_prefix), cy_of.get(to_prefix)) {
                        let x1 = tx + SVG_NODE_W / 2.0;
                        let x2 = fx - SVG_NODE_W / 2.0;
                        out.push_str(&format!(
                            "<line x1=\"{x1:.0}\" y1=\"{ty:.0}\" x2=\"{x2:.0}\" \
                             y2=\"{fy:.0}\" {sc} stroke-width=\"1.5\" opacity=\"0.7\" \
                             marker-end=\"url(#dep-arr)\"/>\n",
                            sc = stroke_attr("#9ca3af"),
                        ));
                    }
                }
            }
        }
    }

    // Nodes
    for m in &ontology.namespaces {
        let prefix = m.namespace.prefix;
        let space_str = format!("{:?}", m.namespace.space).to_lowercase();
        let fill = space_fill(&space_str);

        if let (Some(&cx), Some(&cy)) = (cx_of.get(prefix), cy_of.get(prefix)) {
            let rx = cx - SVG_NODE_W / 2.0;
            let ry = cy - SVG_NODE_H / 2.0;
            let page_url = format!("{base_path}/namespaces/{prefix}/");
            out.push_str(&format!(
                "<a href=\"{page_url}\" aria-label=\"{label} namespace\">\n\
                 <rect x=\"{rx:.0}\" y=\"{ry:.0}\" width=\"{nw:.0}\" height=\"{nh:.0}\" \
                 rx=\"4\" {fc}/>\n\
                 <text x=\"{cx:.0}\" y=\"{ty:.0}\" text-anchor=\"middle\" {wf} \
                 font-size=\"11\" font-family=\"system-ui,sans-serif\" \
                 font-weight=\"600\">{prefix}</text>\n\
                 </a>\n",
                label = m.namespace.label,
                nw = SVG_NODE_W,
                nh = SVG_NODE_H,
                fc = fill_attr(fill),
                wf = fill_attr("#fff"),
                ty = cy + 4.0,
            ));
        }
    }

    out.push_str("</svg>");
    out
}

/// Kahn's BFS topological sort returning the layer index for each prefix.
///
/// Layer 0 = no imports; each subsequent layer depends on the previous.
fn kahn_layers<'a>(
    all_prefixes: &[&'a str],
    deps: &HashMap<&'a str, Vec<&'a str>>,
) -> HashMap<&'a str, usize> {
    let mut layer: HashMap<&str, usize> = all_prefixes.iter().map(|&p| (p, 0)).collect();

    let mut importers: HashMap<&str, Vec<&str>> =
        all_prefixes.iter().map(|&p| (p, vec![])).collect();
    for (&prefix, imports) in deps {
        for &imp in imports {
            importers.entry(imp).or_default().push(prefix);
        }
    }

    let mut queue: std::collections::VecDeque<&str> = all_prefixes
        .iter()
        .filter(|&&p| deps.get(p).map(|v| v.is_empty()).unwrap_or(true))
        .copied()
        .collect();

    let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();

    while let Some(prefix) = queue.pop_front() {
        if visited.contains(prefix) {
            continue;
        }
        visited.insert(prefix);
        let my_layer = *layer.get(prefix).unwrap_or(&0);
        for &importer in importers.get(prefix).map(|v| v.as_slice()).unwrap_or(&[]) {
            let new_layer = my_layer + 1;
            let entry = layer.entry(importer).or_insert(0);
            if new_layer > *entry {
                *entry = new_layer;
            }
            queue.push_back(importer);
        }
    }

    layer
}

// ── 3. Class Hierarchy SVG ───────────────────────────────────────────────────

/// Renders a class hierarchy tree for a namespace module as inline SVG.
///
/// Returns an empty string if the namespace has fewer than `MIN_HIERARCHY_CLASSES` classes.
pub fn render_class_hierarchy_svg(module: &NamespaceModule) -> String {
    let min_classes = uor_ontology::counts::MIN_HIERARCHY_CLASSES;
    if module.classes.len() < min_classes {
        return String::new();
    }

    let class_iris: std::collections::HashSet<&str> = module.classes.iter().map(|c| c.id).collect();

    let mut children_of: HashMap<&str, Vec<&str>> =
        module.classes.iter().map(|c| (c.id, vec![])).collect();
    let mut has_parent = std::collections::HashSet::new();

    for class in &module.classes {
        for &parent_iri in class.subclass_of {
            if class_iris.contains(parent_iri) {
                children_of.entry(parent_iri).or_default().push(class.id);
                has_parent.insert(class.id);
            }
        }
    }

    let roots: Vec<&str> = module
        .classes
        .iter()
        .filter(|c| !has_parent.contains(c.id))
        .map(|c| c.id)
        .collect();

    let mut depth_of: HashMap<&str, usize> = HashMap::new();
    let mut pos_at_depth: HashMap<usize, usize> = HashMap::new();
    let mut position: HashMap<&str, usize> = HashMap::new();

    let mut queue: std::collections::VecDeque<(&str, usize)> =
        roots.iter().map(|&r| (r, 0_usize)).collect();

    while let Some((id, depth)) = queue.pop_front() {
        if depth_of.contains_key(id) {
            continue;
        }
        depth_of.insert(id, depth);
        let pos = *pos_at_depth.entry(depth).or_insert(0);
        position.insert(id, pos);
        *pos_at_depth.entry(depth).or_insert(0) += 1;

        let empty = vec![];
        for &child in children_of.get(id).unwrap_or(&empty) {
            queue.push_back((child, depth + 1));
        }
    }

    let max_depth = depth_of.values().copied().max().unwrap_or(0);
    let max_per_row = pos_at_depth.values().copied().max().unwrap_or(1);

    let svg_w = SVG_MARGIN * 2.0 + max_per_row as f64 * (SVG_CLASS_NODE_W + 20.0) - 20.0;
    let svg_h = SVG_MARGIN * 2.0 + (max_depth + 1) as f64 * SVG_CLASS_ROW_H;

    let ns_label = module.namespace.label;
    let space_str = format!("{:?}", module.namespace.space).to_lowercase();
    let fill = space_fill(&space_str);

    let mut out = format!(
        "<svg class=\"class-hierarchy\" viewBox=\"0 0 {sw:.0} {sh:.0}\" \
         xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"ch-{ns}\">\n\
         <title id=\"ch-{ns}\">Class hierarchy for {ns_label} namespace</title>\n",
        sw = svg_w,
        sh = svg_h,
        ns = ns_label,
    );

    // Edges first
    for class in &module.classes {
        let child_depth = *depth_of.get(class.id).unwrap_or(&0);
        let child_pos = *position.get(class.id).unwrap_or(&0);
        let child_cx =
            SVG_MARGIN + child_pos as f64 * (SVG_CLASS_NODE_W + 20.0) + SVG_CLASS_NODE_W / 2.0;
        let child_ty = SVG_MARGIN + child_depth as f64 * SVG_CLASS_ROW_H;

        for &parent_iri in class.subclass_of {
            if !class_iris.contains(parent_iri) {
                continue;
            }
            let parent_depth = *depth_of.get(parent_iri).unwrap_or(&0);
            let parent_pos = *position.get(parent_iri).unwrap_or(&0);
            let parent_cx =
                SVG_MARGIN + parent_pos as f64 * (SVG_CLASS_NODE_W + 20.0) + SVG_CLASS_NODE_W / 2.0;
            let parent_by = SVG_MARGIN + parent_depth as f64 * SVG_CLASS_ROW_H + 28.0;

            out.push_str(&format!(
                "<line x1=\"{pcx:.0}\" y1=\"{pby:.0}\" x2=\"{ccx:.0}\" y2=\"{cty:.0}\" \
                 {sc} stroke-width=\"1.5\"/>\n",
                pcx = parent_cx,
                pby = parent_by,
                ccx = child_cx,
                cty = child_ty,
                sc = stroke_attr("#9ca3af"),
            ));
        }
    }

    // Nodes
    for class in &module.classes {
        let depth = *depth_of.get(class.id).unwrap_or(&0);
        let pos = *position.get(class.id).unwrap_or(&0);
        let rx = SVG_MARGIN + pos as f64 * (SVG_CLASS_NODE_W + 20.0);
        let ry = SVG_MARGIN + depth as f64 * SVG_CLASS_ROW_H;
        let cx = rx + SVG_CLASS_NODE_W / 2.0;
        let label = if class.label.len() > 14 {
            &class.label[..14]
        } else {
            class.label
        };

        out.push_str(&format!(
            "<rect x=\"{rx:.0}\" y=\"{ry:.0}\" width=\"{nw:.0}\" height=\"28\" rx=\"4\" \
             {fc} opacity=\"0.85\"/>\n\
             <text x=\"{cx:.0}\" y=\"{ty:.0}\" text-anchor=\"middle\" {wf} \
             font-size=\"10\" font-family=\"system-ui,sans-serif\">{label}</text>\n",
            nw = SVG_CLASS_NODE_W,
            fc = fill_attr(fill),
            wf = fill_attr("#fff"),
            ty = ry + 18.0,
        ));
    }

    out.push_str("</svg>");
    out
}

// ── 4. Identity Distribution SVG ─────────────────────────────────────────────

/// Renders a horizontal bar chart of op:Identity counts by verification domain.
///
/// Domain count is derived from `Ontology::full()` — no hard-coded value.
pub fn render_identity_distribution_svg(ontology: &Ontology) -> String {
    let mut domain_labels: Vec<&str> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_.ends_with("VerificationDomain"))
        .map(|i| i.label)
        .collect();
    domain_labels.sort_unstable();

    if domain_labels.is_empty() {
        return String::new();
    }

    let mut counts: HashMap<&str, usize> = domain_labels.iter().map(|&l| (l, 0)).collect();

    for ind in ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_.ends_with("Identity"))
    {
        for (prop_iri, val) in ind.properties {
            if !prop_iri.ends_with("verificationDomain") {
                continue;
            }
            let domain_label = match val {
                IndividualValue::IriRef(iri) => {
                    domain_labels.iter().find(|&&l| iri.ends_with(l)).copied()
                }
                _ => None,
            };
            if let Some(label) = domain_label {
                *counts.entry(label).or_insert(0) += 1;
            }
        }
    }

    let max_count = counts.values().copied().max().unwrap_or(1).max(1);
    let svg_h =
        SVG_MARGIN * 2.0 + domain_labels.len() as f64 * (SVG_BAR_H + SVG_BAR_GAP) - SVG_BAR_GAP;
    let svg_w = SVG_MARGIN + SVG_LABEL_W + SVG_BAR_MAX_W + 60.0;

    let mut out = format!(
        "<svg class=\"identity-distribution\" viewBox=\"0 0 {sw:.0} {sh:.0}\" \
         xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"id-dist-title\">\n\
         <title id=\"id-dist-title\">Identity count distribution across {n} verification domains</title>\n",
        sw = svg_w,
        sh = svg_h,
        n = domain_labels.len()
    );

    for (i, &label) in domain_labels.iter().enumerate() {
        let y = SVG_MARGIN + i as f64 * (SVG_BAR_H + SVG_BAR_GAP);
        let count = *counts.get(label).unwrap_or(&0);
        let bar_w = (count as f64 / max_count as f64 * SVG_BAR_MAX_W).max(2.0);
        let lx = SVG_MARGIN + SVG_LABEL_W - 8.0;
        let bx = SVG_MARGIN + SVG_LABEL_W;
        let tx = bx + bar_w + 6.0;
        let ty = y + SVG_BAR_H * 0.7;

        out.push_str(&format!(
            "<text x=\"{lx:.0}\" y=\"{ty:.0}\" text-anchor=\"end\" {ef} \
             font-size=\"11\" font-family=\"system-ui,sans-serif\">{label}</text>\n\
             <rect x=\"{bx:.0}\" y=\"{y:.0}\" width=\"{bw:.0}\" height=\"{bh:.0}\" rx=\"3\" \
             {bf} opacity=\"0.85\"/>\n\
             <text x=\"{tx:.0}\" y=\"{ty:.0}\" {mf} font-size=\"11\" \
             font-family=\"system-ui,sans-serif\">{count}</text>\n",
            ef = fill_attr("#e8e8f0"),
            bw = bar_w,
            bh = SVG_BAR_H,
            bf = fill_attr("#059669"),
            mf = fill_attr("#9ca3af"),
        ));
    }

    out.push_str("</svg>");
    out
}

// ── 5. Witt Levels SVG ─────────────────────────────────────────────────────

/// Renders the Witt level hierarchy diagram, derived from WittLevel individuals.
///
/// Extracts `wittLength` and `bitsWidth` from `Ontology::full()` — no hard-coded W8–W32.
pub fn render_witt_levels_svg(ontology: &Ontology) -> String {
    let mut levels: Vec<(i64, i64, &str)> = ontology
        .namespaces
        .iter()
        .flat_map(|m| m.individuals.iter())
        .filter(|i| i.type_.ends_with("WittLevel"))
        .filter_map(|i| {
            let idx = i
                .properties
                .iter()
                .find(|(k, _)| k.ends_with("wittLength"))
                .and_then(|(_, v)| {
                    if let IndividualValue::Int(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })?;
            let bits = i
                .properties
                .iter()
                .find(|(k, _)| k.ends_with("bitsWidth"))
                .and_then(|(_, v)| {
                    if let IndividualValue::Int(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                })?;
            Some((idx, bits, i.label))
        })
        .collect();

    if levels.is_empty() {
        return String::new();
    }
    levels.sort_by_key(|(idx, _, _)| *idx);

    let n = levels.len() as f64;
    let svg_w = SVG_MARGIN * 2.0 + n * (SVG_LEVEL_W + SVG_LEVEL_GAP) - SVG_LEVEL_GAP;
    let svg_h = SVG_MARGIN * 2.0 + SVG_LEVEL_H + 40.0;
    let max_idx = levels.last().map(|(i, _, _)| *i).unwrap_or(0);
    let n_int = levels.len();

    let defs = if levels.len() > 1 {
        format!(
            "<defs><marker id=\"ql-arr\" markerWidth=\"6\" markerHeight=\"6\" refX=\"5\" \
             refY=\"3\" orient=\"auto\"><path d=\"M0,0 L0,6 L6,3 z\" {af}/></marker></defs>\n",
            af = fill_attr("#9ca3af"),
        )
    } else {
        String::new()
    };

    let title_text = if n_int > 1 {
        format!("Witt level hierarchy: {n_int} levels from W8 to W{max_idx}")
    } else {
        format!("Witt level hierarchy: {n_int} level(s)")
    };

    let mut out = format!(
        "<svg class=\"witt-levels\" viewBox=\"0 0 {sw:.0} {sh:.0}\" \
         xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-labelledby=\"ql-title\">\n\
         <title id=\"ql-title\">{title_text}</title>\n\
         {defs}",
        sw = svg_w,
        sh = svg_h,
    );

    for (i, (idx, bits, label)) in levels.iter().enumerate() {
        let x = SVG_MARGIN + i as f64 * (SVG_LEVEL_W + SVG_LEVEL_GAP);
        let y = SVG_MARGIN;
        let cx = x + SVG_LEVEL_W / 2.0;
        let opacity = 0.6 + (i as f64 / levels.len() as f64) * 0.35;

        out.push_str(&format!(
            "<rect x=\"{x:.0}\" y=\"{y:.0}\" width=\"{lw:.0}\" height=\"{lh:.0}\" rx=\"6\" \
             {fc} opacity=\"{op:.2}\"/>\n\
             <text x=\"{cx:.0}\" y=\"{ty:.0}\" text-anchor=\"middle\" {wf} \
             font-size=\"14\" font-weight=\"700\" \
             font-family=\"system-ui,sans-serif\">Q{idx}</text>\n\
             <text x=\"{cx:.0}\" y=\"{sy:.0}\" text-anchor=\"middle\" {pf} \
             font-size=\"10\" font-family=\"system-ui,sans-serif\">{bits} bits</text>\n\
             <text x=\"{cx:.0}\" y=\"{ly:.0}\" text-anchor=\"middle\" {mf} \
             font-size=\"9\" font-family=\"system-ui,sans-serif\">{label}</text>\n",
            lw = SVG_LEVEL_W,
            lh = SVG_LEVEL_H,
            fc = fill_attr("#7c3aed"),
            op = opacity,
            wf = fill_attr("#fff"),
            pf = fill_attr("#ede9fe"),
            mf = fill_attr("#9ca3af"),
            ty = y + 22.0,
            sy = y + 38.0,
            ly = y + SVG_LEVEL_H + 18.0,
        ));

        if i + 1 < levels.len() {
            let ax1 = x + SVG_LEVEL_W + 2.0;
            let ax2 = x + SVG_LEVEL_W + SVG_LEVEL_GAP - 2.0;
            let ay = y + SVG_LEVEL_H / 2.0;
            out.push_str(&format!(
                "<line x1=\"{ax1:.0}\" y1=\"{ay:.0}\" x2=\"{ax2:.0}\" y2=\"{ay:.0}\" \
                 {sc} stroke-width=\"1.5\" marker-end=\"url(#ql-arr)\"/>\n",
                sc = stroke_attr("#9ca3af"),
            ));
        }
    }

    out.push_str("</svg>");
    out
}

// ── Space helpers ────────────────────────────────────────────────────────────

/// Returns the CSS hex color for a space string (used in explore page).
pub fn space_hex_color(space: &str) -> &'static str {
    space_fill(space)
}

/// Returns the display name for a `Space` enum value.
pub fn space_display_name(space: &Space) -> &'static str {
    match space {
        Space::Kernel => "Kernel",
        Space::Bridge => "Bridge",
        Space::User => "User",
    }
}

// ── Concept map ─────────────────────────────────────────────────────────────

/// Renders a concept relationship graph as an SVG.
///
/// Nodes are grouped by space classification into rows (kernel top, bridge
/// middle, user bottom). Edges connect related concepts with gray lines.
pub fn render_concept_map_svg(concepts: &[crate::model::ConceptPage], base_path: &str) -> String {
    use crate::concepts::CONCEPT_RELATIONS;

    if concepts.is_empty() {
        return String::new();
    }

    let node_w: f64 = 140.0;
    let node_h: f64 = 32.0;
    let h_gap: f64 = 30.0;
    let v_gap: f64 = 70.0;
    let margin: f64 = 30.0;

    // Group concepts by space
    let kernel: Vec<&crate::model::ConceptPage> =
        concepts.iter().filter(|c| c.space == "kernel").collect();
    let bridge: Vec<&crate::model::ConceptPage> =
        concepts.iter().filter(|c| c.space == "bridge").collect();
    let user: Vec<&crate::model::ConceptPage> = concepts
        .iter()
        .filter(|c| c.space != "kernel" && c.space != "bridge")
        .collect();

    let rows: Vec<&Vec<&crate::model::ConceptPage>> = vec![&kernel, &bridge, &user];

    let max_row = rows.iter().map(|r| r.len()).max().unwrap_or(1);
    let svg_w = margin * 2.0 + (max_row as f64) * node_w + ((max_row as f64) - 1.0) * h_gap;
    let svg_h = margin * 2.0 + 3.0 * node_h + 2.0 * v_gap;

    // Compute node positions: (slug -> (cx, cy))
    let mut positions: HashMap<String, (f64, f64)> = HashMap::new();
    for (row_idx, row) in rows.iter().enumerate() {
        let row_width = (row.len() as f64) * node_w + ((row.len() as f64) - 1.0) * h_gap;
        let start_x = (svg_w - row_width) / 2.0;
        let y = margin + (row_idx as f64) * (node_h + v_gap);
        for (col, concept) in row.iter().enumerate() {
            let x = start_x + (col as f64) * (node_w + h_gap);
            positions.insert(concept.slug.clone(), (x + node_w / 2.0, y + node_h / 2.0));
        }
    }

    // Draw edges
    let mut edges = String::new();
    for (slug, _, related_slugs) in CONCEPT_RELATIONS {
        if let Some(&(x1, y1)) = positions.get(*slug) {
            for rel in *related_slugs {
                if let Some(&(x2, y2)) = positions.get(*rel) {
                    // Only draw each edge once (alphabetically first slug draws it)
                    if *slug < *rel {
                        edges.push_str(&format!(
                            "<line x1=\"{x1:.0}\" y1=\"{y1:.0}\" \
                             x2=\"{x2:.0}\" y2=\"{y2:.0}\" \
                             stroke=\"#94a3b8\" stroke-width=\"1\" opacity=\"0.6\"/>\n"
                        ));
                    }
                }
            }
        }
    }

    // Draw nodes
    let mut nodes = String::new();
    let space_colors: &[(&str, &str, &str)] = &[
        ("kernel", "#7c3aed", "#ede9fe"),
        ("bridge", "#059669", "#d1fae5"),
        ("user", "#0891b2", "#cffafe"),
    ];
    for concept in concepts {
        if let Some(&(cx, cy)) = positions.get(&concept.slug) {
            let (fill, _text_bg) = space_colors
                .iter()
                .find(|(s, _, _)| *s == concept.space)
                .map(|(_, f, t)| (*f, *t))
                .unwrap_or(("#7c3aed", "#ede9fe"));

            let x = cx - node_w / 2.0;
            let y = cy - node_h / 2.0;
            let label = crate::renderer::escape_html(&concept.title);
            // Truncate long titles for the SVG node
            let short_label = if label.len() > 20 {
                format!("{}\u{2026}", &label[..19])
            } else {
                label.clone()
            };

            nodes.push_str(&format!(
                "<a href=\"{base_path}/concepts/{slug}.html\">\
                 <rect x=\"{x:.0}\" y=\"{y:.0}\" width=\"{node_w:.0}\" height=\"{node_h:.0}\" \
                 rx=\"4\" fill=\"{fill}\" opacity=\"0.9\"/>\
                 <text x=\"{cx:.0}\" y=\"{ty:.0}\" text-anchor=\"middle\" \
                 fill=\"#fff\" font-size=\"11\" font-family=\"system-ui, sans-serif\">\
                 {short_label}</text></a>\n",
                slug = concept.slug,
                ty = cy + 4.0,
            ));
        }
    }

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" \
         aria-labelledby=\"concept-map-title\" \
         viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\">\n\
         <title id=\"concept-map-title\">Concept relationship map</title>\n\
         {edges}\
         {nodes}\
         </svg>",
        w = svg_w,
        h = svg_h,
    )
}
