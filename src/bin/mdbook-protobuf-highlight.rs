//! mdBook preprocessor: build-time protobuf / CEL syntax highlighting.

use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use mdbook_preprocessor::book::Book;
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use protobuf_mdbook::highlight::{
    HighlightConfig, PREPROCESSOR_COMMAND, config_from_mdbook, install_book_toml, transform_chapter,
};
use std::io;
use std::path::PathBuf;
use std::process;

struct ProtobufHighlight;

impl Preprocessor for ProtobufHighlight {
    fn name(&self) -> &str {
        "protobuf-highlight"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let config = config_from_mdbook(ctx);
        book.for_each_chapter_mut(
            |chapter| match transform_chapter(&chapter.content, config) {
                Ok(md) => chapter.content = md,
                Err(e) => eprintln!(
                    "mdbook-protobuf-highlight: chapter {:?}: {e:#}",
                    chapter.path
                ),
            },
        );
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool, Error> {
        Ok(renderer == "html")
    }
}

fn make_app() -> Command {
    Command::new(PREPROCESSOR_COMMAND)
        .version(env!("CARGO_PKG_VERSION"))
        .about("mdBook preprocessor for build-time protobuf and CEL highlighting")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported"),
        )
        .subcommand(
            Command::new("install").arg(
                Arg::new("dir")
                    .default_value(".")
                    .help("Book root directory containing book.toml"),
            ),
        )
}

fn main() {
    let matches = make_app().get_matches();
    if let Some(sub) = matches.subcommand_matches("supports") {
        handle_supports(sub);
    } else if let Some(sub) = matches.subcommand_matches("install") {
        if let Err(e) = handle_install(sub) {
            eprintln!("{e:#}");
            process::exit(1);
        }
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("{e:#}");
        process::exit(1);
    }
}

fn handle_supports(sub: &ArgMatches) -> ! {
    let renderer = sub
        .get_one::<String>("renderer")
        .expect("renderer required");
    if ProtobufHighlight
        .supports_renderer(renderer)
        .unwrap_or(false)
    {
        process::exit(0);
    }
    process::exit(1);
}

fn handle_install(sub: &ArgMatches) -> Result<()> {
    let dir = sub.get_one::<String>("dir").expect("dir required");
    install_book_toml(PathBuf::from(dir).as_path(), HighlightConfig::all())
        .context("install preprocessor in book.toml")
}

fn handle_preprocessing() -> Result<()> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;
    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        eprintln!(
            "Warning: mdbook-protobuf-highlight built against mdbook {} but called from {}",
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }
    let processed = ProtobufHighlight.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed)?;
    Ok(())
}
