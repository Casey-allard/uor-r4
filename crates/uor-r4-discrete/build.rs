//! Exact fixed-geometry admission and ahead-of-time integer CUDA compilation.
//! The core crate is a build dependency only; no host model executes here.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::{env, fs, path::PathBuf, process::Command};
use uor_r4_core::spiralcore_operator::{
    cl06_bivectors, cl06_finite_composition_table, SignedMatrix8,
};
#[path = "src/baseline_reuse_source.rs"]
mod baseline_reuse_source;
#[path = "src/bvgk_source.rs"]
mod bvgk_source;
#[path = "src/decomposition_source.rs"]
mod decomposition_source;
#[path = "src/rows_source.rs"]
mod rows_source;
#[path = "src/rowsk_source.rs"]
mod rowsk_source;

const ENTRIES: [&str; 13] = [
    "discrete_init",
    "discrete_encode",
    "discrete_export_token_cells",
    "discrete_forward",
    "discrete_counterfactual",
    "discrete_choose",
    "discrete_apply",
    "discrete_validate",
    "discrete_evaluate",
    "discrete_evaluation_aggregate",
    "discrete_count",
    "discrete_count_score",
    "discrete_carrier_select",
];

/// Inspect the emitted module, rather than treating CUDA source spelling as
/// evidence of its instructions. Integer index products are not prohibited.
fn admit_ptx_inventory(ptx: &str, inventory: &[&str]) -> Result<(), String> {
    let mut entries = BTreeSet::new();
    let mut expect_entry = false;
    for (line_number, line) in ptx.lines().enumerate() {
        let instruction = line.split("//").next().unwrap_or_default();
        for token in instruction
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
            .filter(|token| !token.is_empty())
        {
            let components: Vec<_> = token.split('.').collect();
            if components.iter().any(|component| {
                matches!(
                    *component,
                    "f16"
                        | "f16x2"
                        | "f32"
                        | "f64"
                        | "bf16"
                        | "bf16x2"
                        | "tf32"
                        | "e4m3"
                        | "e5m2"
                        | "mma"
                        | "wmma"
                        | "wgmma"
                        | "tcgen05"
                        | "tensor"
                        | "tensormap"
                )
            }) || token.starts_with("ex2.approx")
            {
                return Err(format!(
                    "integer PTX admission rejected {token} at line {}",
                    line_number + 1,
                ));
            }
            if expect_entry {
                if token.contains('.') || !entries.insert(token.to_owned()) {
                    return Err(format!("invalid or duplicate PTX entry {token}"));
                }
                expect_entry = false;
            } else if token == ".entry" {
                expect_entry = true;
            }
        }
    }
    if expect_entry {
        return Err("unterminated PTX entry declaration".into());
    }
    let expected: BTreeSet<_> = inventory.iter().map(|entry| entry.to_string()).collect();
    if entries != expected {
        return Err(format!(
            "integer PTX entry inventory differs: missing {:?}, unexpected {:?}",
            expected.difference(&entries).collect::<Vec<_>>(),
            entries.difference(&expected).collect::<Vec<_>>(),
        ));
    }
    Ok(())
}

fn admit_ptx(ptx: &str) -> Result<(), String> {
    admit_ptx_inventory(ptx, &ENTRIES)
}

// Host metadata lexer controls only; these strings are never loaded or run.
fn ptx_admission_self_test() -> Result<(), String> {
    let inventory = ENTRIES
        .iter()
        .map(|entry| format!(".visible .entry {entry}() {{ ret; }}\n"))
        .collect::<String>();
    admit_ptx(&format!(
        "// floats such as .f32 are comments\n{inventory}mul.lo.u32 %a,%b,8;"
    ))?;
    for forbidden in [
        "add.f32 %a,%b,%c;",
        ".reg .bf16 %a;",
        "mma.sync.aligned.m16n8k8.row.col.s32.s8.s8.s32 %a,%b,%c,%d;",
        "ex2.approx %a,%b;",
        ".visible .entry unexpected() { ret; }",
        ".visible .entry discrete_init() { ret; }",
    ] {
        if admit_ptx(&format!("{inventory}{forbidden}")).is_ok() {
            return Err(format!(
                "PTX admission self-test failed to reject {forbidden}"
            ));
        }
    }
    if admit_ptx(".visible .entry discrete_init() { ret; }").is_ok() {
        return Err("PTX admission self-test failed to reject missing entries".into());
    }
    Ok(())
}

// Fixed operator/index metadata only. This is graph reachability over the
// source-bound action tables, never a host carrier or model reference.
fn exact_reachability(
    compose: &[u8],
    generators: &[u8],
    neighbors: &[Vec<u8>],
) -> Result<([u8; 64], [u8; 16]), String> {
    if compose.len() != 64 * 64
        || compose.iter().any(|&state| state >= 64)
        || generators.len() != 15
        || generators.iter().any(|&state| state >= 64)
        || neighbors.len() != 16
        || neighbors
            .iter()
            .any(|row| row.len() != 15 || row.iter().any(|&root| root >= 16))
    {
        return Err("exact reachability metadata has an invalid action/root inventory".into());
    }
    let mut frames = [0u8; 64];
    for start in 0..64 {
        let mut seen = [false; 64];
        seen[start] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(from) = queue.pop_front() {
            for &generator in generators {
                let to = usize::from(compose[usize::from(generator) * 64 + from]);
                if !seen[to] {
                    seen[to] = true;
                    queue.push_back(to);
                }
            }
        }
        frames[start] = seen.iter().filter(|&&value| value).count() as u8;
        if frames[start] != 64 {
            return Err(format!(
                "bivector graph reaches {} of 64 frames from state {start}",
                frames[start]
            ));
        }
    }
    let mut roots = [0u8; 16];
    for start in 0..16 {
        let mut seen = [false; 16];
        seen[start] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(from) = queue.pop_front() {
            for &to in &neighbors[from] {
                let to = usize::from(to);
                if !seen[to] {
                    seen[to] = true;
                    queue.push_back(to);
                }
            }
        }
        roots[start] = seen.iter().filter(|&&value| value).count() as u8;
        if roots[start] != 16 {
            return Err(format!(
                "bivector graph reaches {} of 16 orbit roots from root {start}",
                roots[start]
            ));
        }
    }
    Ok((frames, roots))
}

fn geometry_source() -> Result<String, String> {
    let table = cl06_finite_composition_table().map_err(|e| e.to_string())?;
    let states = table.states();
    let index = |matrix: SignedMatrix8| {
        states
            .iter()
            .position(|&state| state == matrix)
            .map(|i| i as u8)
            .ok_or_else(|| "operator leaves the exact 64-state group".to_string())
    };
    let identity = table.identity_index();
    let compose: Vec<u8> = table
        .composition_indexes()
        .iter()
        .flatten()
        .copied()
        .collect();
    let inverse = table.inverse_indexes();
    let mut axes = Vec::with_capacity(64 * 8);
    for state in states {
        let mut used = [false; 8];
        for row in state.entries() {
            let mut selected = None;
            for (column, &value) in row.iter().enumerate() {
                if value == 0 {
                    continue;
                }
                if selected.is_some() || used[column] || !matches!(value, -1 | 1) {
                    return Err("operator is not an exact signed-axis permutation".into());
                }
                used[column] = true;
                selected = Some(column as u8 | if value < 0 { 8 } else { 0 });
            }
            axes.push(selected.ok_or("operator has an empty axis")?);
        }
    }

    // Preserve spiralcore_fbs.rs's anchor-first, then lexicographic orbit order.
    let anchor = [2i8, 2, 0, 0, 0, 0, 0, 0];
    let mut orbit = BTreeSet::new();
    for state in states {
        orbit.insert(state.apply_root_key(anchor).map_err(|e| e.to_string())?);
    }
    if orbit.len() != 16 {
        return Err("SpiralCore anchor orbit must have exactly sixteen roots".into());
    }
    let mut roots = vec![anchor];
    roots.extend(orbit.into_iter().filter(|root| *root != anchor));
    let root_indexes: BTreeMap<_, _> = roots
        .iter()
        .copied()
        .enumerate()
        .map(|(i, root)| (root, i as u8))
        .collect();
    let root_index = |root: [i8; 8]| {
        root_indexes
            .get(&root)
            .copied()
            .ok_or_else(|| "exact transport leaves the anchor orbit".to_string())
    };
    let bivectors = cl06_bivectors().map_err(|e| e.to_string())?;
    let generators: Vec<u8> = bivectors
        .iter()
        .map(|b| index(b.matrix))
        .collect::<Result<_, _>>()?;
    let neighbors: Vec<Vec<u8>> = roots
        .iter()
        .map(|&root| {
            bivectors
                .iter()
                .map(|b| root_index(b.matrix.apply_root_key(root).map_err(|e| e.to_string())?))
                .collect()
        })
        .collect::<Result<_, String>>()?;
    let (frame_reachability, root_reachability) =
        exact_reachability(&compose, &generators, &neighbors)?;

    // Same FIFO BFS and generator ordering as the retained FBS compiler.
    let mut bfs = Vec::with_capacity(16 * 16);
    for start in 0..16 {
        let mut paths = [None; 16];
        paths[start] = Some(identity);
        let mut queue = VecDeque::from([start]);
        while let Some(from) = queue.pop_front() {
            let state = paths[from].ok_or("missing BFS prefix")?;
            for (edge, &to) in neighbors[from].iter().enumerate() {
                if paths[usize::from(to)].is_some() {
                    continue;
                }
                paths[usize::from(to)] =
                    Some(compose[usize::from(generators[edge]) * 64 + usize::from(state)]);
                queue.push_back(usize::from(to));
            }
        }
        for (destination, path) in paths.into_iter().enumerate() {
            let state = path.ok_or("disconnected SpiralCore anchor orbit")?;
            if states[usize::from(state)]
                .apply_root_key(roots[start])
                .map_err(|e| e.to_string())?
                != roots[destination]
            {
                return Err("BFS action does not reproduce its ordered root transport".into());
            }
            bfs.push(state);
        }
    }

    let first = *generators.first().ok_or("missing first bivector")?;
    let first_inverse = inverse[usize::from(first)];
    if index(
        bivectors[0]
            .matrix
            .checked_power(3)
            .map_err(|e| e.to_string())?,
    )? != first_inverse
    {
        return Err("first ternary generator's inverse differs from its cube".into());
    }
    let mut branch = Vec::with_capacity(16 * 3);
    for &root in &roots {
        let mut distinct = BTreeSet::new();
        for state in [identity, first, first_inverse] {
            let port = root_index(
                states[usize::from(state)]
                    .apply_root_key(root)
                    .map_err(|e| e.to_string())?,
            )?;
            branch.push(port);
            distinct.insert(port);
        }
        if distinct.len() != 3 {
            return Err("exact ternary branch ports coincide".into());
        }
    }
    let report = table.validate().map_err(|e| e.to_string())?;
    Ok(format!(
        "// Generated exact geometry; anchor/BFS order matches spiralcore_fbs.rs.\n\
         pub const AXES: [u8; 512] = {axes:?};\n\
         pub const COMPOSE: [u8; 4096] = {compose:?};\n\
         pub const INVERSE: [u8; 64] = {inverse:?};\n\
         pub const ROOTS: [[i8; 8]; 16] = {roots:?};\n\
         pub const BFS: [u8; 256] = {bfs:?};\n\
         pub const BRANCH: [u8; 48] = {branch:?};\n\
         pub const GENERATORS: [u8; 15] = {generators:?};\n\
         pub const IDENTITY: u8 = {identity};\n\
         // Validated frame reachability: {frame_reachability:?}\n\
         // Validated root reachability: {root_reachability:?}\n\
         pub const OPERATOR_KAPPA: &str = {:?};\n\
         pub const COMPOSITION_KAPPA: &str = {:?};\n",
        report.operator_kappa,
        table.reproduce_kappa().map_err(|e| e.to_string())?,
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../uor-r4-core/src/spiralcore_operator.rs");
    println!("cargo:rerun-if-changed=src/discrete.cu");
    println!("cargo:rerun-if-changed=src/decomposition_source.rs");
    println!("cargo:rerun-if-changed=src/baseline_reuse_source.rs");
    println!("cargo:rerun-if-changed=src/rows_source.rs");
    println!("cargo:rerun-if-changed=src/rowsk_source.rs");
    println!("cargo:rerun-if-changed=src/bvgk_source.rs");
    println!("cargo:rerun-if-env-changed=NVCC");
    println!("cargo:rerun-if-env-changed=UOR_NVCC_ARCH");
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("Cargo did not supply OUT_DIR")?);
    ptx_admission_self_test()?;
    fs::write(out.join("geometry.rs"), geometry_source()?)?;
    if env::var_os("CARGO_FEATURE_CUDA").is_none() {
        return Ok(());
    }
    let arch = env::var("UOR_NVCC_ARCH")
        .map_err(|_| "CUDA build requires the explicit admitted UOR_NVCC_ARCH")?;
    if !arch
        .strip_prefix("compute_")
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err("UOR_NVCC_ARCH must be compute_<decimal capability>".into());
    }
    let status = Command::new(env::var("NVCC").unwrap_or_else(|_| "nvcc".into()))
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .args(["-o"])
        .arg(out.join("discrete.ptx"))
        .arg("src/discrete.cu")
        .status()?;
    if !status.success() {
        return Err(format!("ahead-of-time nvcc discrete build failed: {status}").into());
    }
    admit_ptx(&fs::read_to_string(out.join("discrete.ptx"))?)?;
    // An independent module: the ordinary source, nvcc invocation and PTX stay
    // untouched. Only this diagnostic entry carries internal clocks/counters.
    let diagnostic = decomposition_source::generate(&fs::read_to_string("src/discrete.cu")?)?;
    fs::write(out.join("decomposition.cu"), diagnostic)?;
    let nvcc = env::var("NVCC").unwrap_or_else(|_| "nvcc".into());
    let status = Command::new(&nvcc)
        // Uncapped D per PM amendment6222308938; zero spills and runtime/AOT
        // consistency are checked before dose. Ordinary compilation unchanged.
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-o")
        .arg(out.join("decomposition.ptx"))
        .arg(out.join("decomposition.cu"))
        .status()?;
    if !status.success() {
        return Err(format!("diagnostic PTX build failed: {status}").into());
    }
    admit_ptx_inventory(
        &fs::read_to_string(out.join("decomposition.ptx"))?,
        &["discrete_counterfactual_decomposition"],
    )?;
    // Opt-in baseline reuse has its own source, admitted PTX and exact cubin.
    // The ordinary and diagnostic invocations above remain byte-for-byte.
    let reuse = baseline_reuse_source::generate(&fs::read_to_string("src/discrete.cu")?)?;
    fs::write(out.join("baseline-reuse.cu"), reuse)?;
    let status = Command::new(&nvcc)
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-o")
        .arg(out.join("baseline-reuse.ptx"))
        .arg(out.join("baseline-reuse.cu"))
        .status()?;
    if !status.success() {
        return Err(format!("baseline reuse PTX build failed: {status}").into());
    }
    admit_ptx_inventory(
        &fs::read_to_string(out.join("baseline-reuse.ptx"))?,
        &["discrete_counterfactual_baseline_reuse"],
    )?;
    // Opt-in row-parallel counterfactual: its own generated source, admitted
    // three-entry PTX and cubin. Earlier invocations remain byte-for-byte.
    let rows = rows_source::generate(&fs::read_to_string("src/discrete.cu")?)?;
    fs::write(out.join("rows.cu"), rows)?;
    let status = Command::new(&nvcc)
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-o")
        .arg(out.join("rows.ptx"))
        .arg(out.join("rows.cu"))
        .status()?;
    if !status.success() {
        return Err(format!("row-parallel PTX build failed: {status}").into());
    }
    admit_ptx_inventory(
        &fs::read_to_string(out.join("rows.ptx"))?,
        &rows_source::ENTRIES,
    )?;
    // Opt-in K-coordinate scoring (BVG-K Stage C): the generated rows text plus
    // two entries, its own admitted five-entry PTX and cubin. The rows image
    // above is unchanged; this is a separate module.
    let rowsk = rowsk_source::generate(&fs::read_to_string(out.join("rows.cu"))?)?;
    fs::write(out.join("rowsk.cu"), rowsk)?;
    let status = Command::new(&nvcc)
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-o")
        .arg(out.join("rowsk.ptx"))
        .arg(out.join("rowsk.cu"))
        .status()?;
    if !status.success() {
        return Err(format!("K-coordinate PTX build failed: {status}").into());
    }
    admit_ptx_inventory(
        &fs::read_to_string(out.join("rowsk.ptx"))?,
        &rowsk_source::ENTRIES,
    )?;
    // Opt-in BVG-K set selection and commit (Stage C2): the generated rows-K
    // text plus three entries, its own admitted eight-entry PTX and cubin. The
    // rows and rows-K images above are unchanged; this is a separate module.
    let bvgk = bvgk_source::generate(&fs::read_to_string(out.join("rowsk.cu"))?)?;
    fs::write(out.join("bvgk.cu"), bvgk)?;
    let status = Command::new(&nvcc)
        .args(["-ptx", "-std=c++17"])
        .arg(format!("-arch={arch}"))
        .arg("-o")
        .arg(out.join("bvgk.ptx"))
        .arg(out.join("bvgk.cu"))
        .status()?;
    if !status.success() {
        return Err(format!("BVG-K PTX build failed: {status}").into());
    }
    admit_ptx_inventory(
        &fs::read_to_string(out.join("bvgk.ptx"))?,
        &bvgk_source::ENTRIES,
    )?;
    // Assemble the exact PTX admitted above, rather than compiling the CUDA
    // source a second time. These same cubins and their assembly reports are
    // embedded by the driver; no CUDA context or model executes during build.
    let image_target = arch.replacen("compute_", "sm_", 1);
    for (label, source) in [
        ("ordinary", out.join("discrete.ptx")),
        ("decomposition", out.join("decomposition.ptx")),
        ("baseline-reuse", out.join("baseline-reuse.ptx")),
        ("rows", out.join("rows.ptx")),
        ("rowsk", out.join("rowsk.ptx")),
        ("bvgk", out.join("bvgk.ptx")),
    ] {
        let mut resource_command = Command::new(&nvcc);
        // PTX input needs assembly, not C++ preprocessing or a new resource cap.
        resource_command.args(["-cubin", "--ptxas-options=-v"]);
        let report = resource_command
            .arg(format!("-arch={arch}"))
            // PTX preprocessing retains the virtual architecture, while cubin
            // assembly requires the corresponding real GPU code target.
            .arg(format!("--gpu-code={image_target}"))
            .arg("-o")
            .arg(out.join(format!("{label}.cubin")))
            .arg(source)
            .output()?;
        let mut log = report.stdout;
        log.extend(report.stderr);
        fs::write(out.join(format!("{label}-resources.txt")), &log)?;
        if !report.status.success() {
            return Err(format!(
                "{label} resource build failed: {}",
                String::from_utf8_lossy(&log)
            )
            .into());
        }
        let image = fs::read(out.join(format!("{label}.cubin")))?;
        if image.len() < 64 || !image.starts_with(b"\x7fELF") {
            return Err(format!("{label} assembly did not emit a nonempty ELF cubin").into());
        }
        if log.is_empty() {
            return Err(format!("{label} assembly emitted no resource report").into());
        }
    }
    fs::write(out.join("cuda-image-target.txt"), image_target)?;
    Ok(())
}
