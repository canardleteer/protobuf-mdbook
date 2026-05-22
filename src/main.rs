//! `protoc-gen-mdbook` — thin binary wrapper.

#![forbid(unsafe_code)]

use std::io::{self, Read, Write};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!(
            "protoc-gen-mdbook {} (mdbook {})",
            env!("CARGO_PKG_VERSION"),
            protoc_gen_mdbook::mdbook_version()
        );
        return Ok(());
    }

    let mut stdin = Vec::new();
    io::stdin()
        .read_to_end(&mut stdin)
        .map_err(|e| anyhow::anyhow!("read stdin: {e}"))?;

    let out = protoc_gen_mdbook::generate(&stdin)?;
    io::stdout()
        .write_all(&out)
        .map_err(|e| anyhow::anyhow!("write stdout: {e}"))?;
    Ok(())
}
