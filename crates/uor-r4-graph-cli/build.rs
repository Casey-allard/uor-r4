use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=OPT_LEVEL");

    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned());
    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=UOR_R4_GRAPH_CLI_BUILD_PROFILE={profile}");
    println!("cargo:rustc-env=UOR_R4_GRAPH_CLI_OPT_LEVEL={opt_level}");
}
