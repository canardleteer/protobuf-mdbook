//! Comma-separated protoc token parsing and CLI argv emission.

use super::{
    CliOptionsInput, EscapeTags, Layout, Options, default_markdown_root, default_summary_path,
};
use crate::paths::normalize_rel_path;
use anyhow::{Result, bail};
use std::path::PathBuf;

pub(crate) fn normalize_book_root(root: &str) -> String {
    normalize_rel_path(root, ".").expect("book_root validated")
}

pub(crate) fn layout_name(layout: Layout) -> &'static str {
    match layout {
        Layout::Package => "package",
        Layout::Entity => "entity",
        Layout::Split => "split",
    }
}

/// Parsed comma-separated plugin option (before apply).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ParsedToken {
    Init,
    Summary,
    NoProtoHighlight,
    NoCelHighlight,
    NoProtoMarkdown,
    MarkdownOnly,
    EscapeTags(EscapeTags),
    Layout(Layout),
    BookRoot(String),
    MarkdownRoot(String),
    SummaryPath(String),
    Book(String),
    MdbookOut(String),
    Title(String),
    IgnoreGit(bool),
    ProtoPath(Vec<PathBuf>),
}

pub(crate) fn parse_token(token: &str) -> Result<ParsedToken> {
    if let Some(v) = token.strip_prefix("book_root=") {
        return Ok(ParsedToken::BookRoot(normalize_book_root(v)));
    }
    if let Some(v) = token.strip_prefix("markdown_root=") {
        return Ok(ParsedToken::MarkdownRoot(normalize_rel_path(
            v,
            default_markdown_root(),
        )?));
    }
    if let Some(v) = token.strip_prefix("summary_path=") {
        return Ok(ParsedToken::SummaryPath(normalize_rel_path(
            v,
            default_summary_path(),
        )?));
    }
    if let Some(v) = token.strip_prefix("book=") {
        return Ok(ParsedToken::Book(v.to_string()));
    }
    if let Some(v) = token.strip_prefix("mdbook_out=") {
        return Ok(ParsedToken::MdbookOut(v.to_string()));
    }
    if let Some(v) = token.strip_prefix("title=") {
        return Ok(ParsedToken::Title(v.to_string()));
    }
    if let Some(v) = token.strip_prefix("ignore=") {
        return Ok(ParsedToken::IgnoreGit(match v {
            "git" => true,
            "none" => false,
            other => bail!("unknown ignore value {other:?}; use git or none"),
        }));
    }
    if let Some(v) = token.strip_prefix("proto_path=") {
        return Ok(ParsedToken::ProtoPath(
            v.split(':')
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect(),
        ));
    }
    if let Some(v) = token.strip_prefix("layout=") {
        return Ok(ParsedToken::Layout(match v {
            "package" => Layout::Package,
            "entity" => Layout::Entity,
            "split" => Layout::Split,
            other => bail!("unknown layout {other:?}; use package, entity, or split"),
        }));
    }
    if token == "escape_tags" {
        return Ok(ParsedToken::EscapeTags(EscapeTags::Backticks));
    }
    if let Some(v) = token.strip_prefix("escape_tags=") {
        return Ok(ParsedToken::EscapeTags(match v {
            "backticks" => EscapeTags::Backticks,
            "entities" => EscapeTags::Entities,
            other => bail!("unknown escape_tags value {other:?}; use backticks or entities"),
        }));
    }

    match token {
        "init" => Ok(ParsedToken::Init),
        "summary" => Ok(ParsedToken::Summary),
        "no_proto_highlight" => Ok(ParsedToken::NoProtoHighlight),
        "no_cel_highlight" => Ok(ParsedToken::NoCelHighlight),
        "no_proto_markdown" => Ok(ParsedToken::NoProtoMarkdown),
        "markdown_only" => Ok(ParsedToken::MarkdownOnly),
        other => bail!("unknown plugin option: {other:?}"),
    }
}

pub(crate) fn apply_parsed(opts: &mut Options, token: ParsedToken) {
    match token {
        ParsedToken::Init => opts.init = true,
        ParsedToken::Summary => opts.summary = true,
        ParsedToken::NoProtoHighlight => opts.no_proto_highlight = true,
        ParsedToken::NoCelHighlight => opts.no_cel_highlight = true,
        ParsedToken::NoProtoMarkdown => opts.no_proto_markdown = true,
        ParsedToken::MarkdownOnly => {}
        ParsedToken::EscapeTags(mode) => opts.escape_tags = mode,
        ParsedToken::Layout(layout) => opts.layout = layout,
        ParsedToken::BookRoot(v) => {
            opts.book_root = v;
            opts.explicit_book_root = true;
        }
        ParsedToken::MarkdownRoot(v) => {
            opts.markdown_root = v;
            opts.explicit_markdown_root = true;
        }
        ParsedToken::SummaryPath(v) => {
            opts.summary_path = v;
            opts.explicit_summary_path = true;
        }
        ParsedToken::Book(v) => opts.book = Some(v),
        ParsedToken::MdbookOut(v) => opts.mdbook_out = Some(v),
        ParsedToken::Title(v) => opts.title = Some(v),
        ParsedToken::IgnoreGit(v) => opts.ignore_git = v,
        ParsedToken::ProtoPath(paths) => opts.proto_search_path = paths,
    }
}

pub(crate) fn is_markdown_only(token: &ParsedToken) -> bool {
    matches!(token, ParsedToken::MarkdownOnly)
}

pub fn options_to_cli_args(opts: &Options) -> Vec<String> {
    let mut args = Vec::new();
    push_bool_flag(&mut args, opts.init, "--init");
    push_bool_flag(&mut args, opts.summary, "--summary");
    if opts.layout != Layout::Package {
        args.push("--layout".into());
        args.push(layout_name(opts.layout).into());
    }
    if opts.explicit_book_root {
        push_kv(&mut args, "--book-root", &opts.book_root);
    }
    if let Some(book) = &opts.book {
        push_kv(&mut args, "--book", book);
    }
    if opts.explicit_markdown_root {
        push_kv(&mut args, "--markdown-root", &opts.markdown_root);
    }
    if opts.explicit_summary_path {
        push_kv(&mut args, "--summary-path", &opts.summary_path);
    }
    if let Some(title) = &opts.title {
        push_kv(&mut args, "--title", title);
    }
    if !opts.ignore_git {
        args.push("--ignore".into());
        args.push("none".into());
    }
    push_bool_flag(&mut args, opts.no_proto_highlight, "--no-proto-highlight");
    push_bool_flag(&mut args, opts.no_cel_highlight, "--no-cel-highlight");
    push_bool_flag(&mut args, opts.no_proto_markdown, "--no-proto-markdown");
    match opts.escape_tags {
        EscapeTags::Off => {}
        EscapeTags::Backticks => args.push("--escape-tags".into()),
        EscapeTags::Entities => {
            args.push("--escape-tags".into());
            args.push("entities".into());
        }
    }
    args
}

pub(crate) fn build_options_from_cli(input: CliOptionsInput) -> Result<Options> {
    let mut opts = Options {
        init: input.init,
        summary: input.summary,
        layout: input.layout,
        book: input.book,
        title: input.title,
        ignore_git: input.ignore_git,
        no_proto_highlight: input.no_proto_highlight,
        no_cel_highlight: input.no_cel_highlight,
        no_proto_markdown: input.no_proto_markdown,
        escape_tags: input.escape_tags,
        ..Options::default()
    };

    if let Some(v) = input.book_root {
        apply_parsed(&mut opts, ParsedToken::BookRoot(normalize_book_root(&v)));
    }
    if let Some(v) = input.markdown_root {
        apply_parsed(
            &mut opts,
            ParsedToken::MarkdownRoot(normalize_rel_path(&v, default_markdown_root())?),
        );
    }
    if let Some(v) = input.summary_path {
        apply_parsed(
            &mut opts,
            ParsedToken::SummaryPath(normalize_rel_path(&v, default_summary_path())?),
        );
    }

    Ok(opts)
}

fn push_bool_flag(args: &mut Vec<String>, enabled: bool, flag: &str) {
    if enabled {
        args.push(flag.into());
    }
}

fn push_kv(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.into());
    args.push(value.into());
}
