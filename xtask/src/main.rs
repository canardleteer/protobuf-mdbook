use clap::Parser;

fn main() -> anyhow::Result<()> {
    protobuf_mdbook_xtask::run(protobuf_mdbook_xtask::Cli::parse())
}
