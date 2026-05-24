use std::{env, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
    let dist = Path::new(&manifest_dir).join("dist");
    println!("cargo:rerun-if-changed={}", dist.display());
}
