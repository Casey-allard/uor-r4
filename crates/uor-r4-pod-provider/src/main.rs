fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!(
            "uor-r4-pod-provider {} (Rust; curl TLS transport)",
            env!("CARGO_PKG_VERSION")
        );
        return;
    }
    match uor_r4_pod_provider::run(&args) {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("uor-r4-pod-provider: {error}");
            std::process::exit(1);
        }
    }
}
