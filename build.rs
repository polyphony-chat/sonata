//! Buildfile for sonata.

use vergen::{BuildBuilder, Emitter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rustc-check-cfg=cfg(coverage_nightly)");
    let build = BuildBuilder::all_build()?;
    Emitter::default().add_instructions(&build)?.emit()?;
    Ok(())
}
