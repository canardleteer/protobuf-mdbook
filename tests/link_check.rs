//! Layout variants: generated markdown links must resolve.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ensure_proto_deps_export(manifest_dir: &Path) -> PathBuf {
    let proto_dir = manifest_dir.join("examples/proto");
    let export_dir = manifest_dir.join("target/proto-deps");
    protoc_gen_mdbook::proto_deps::ensure_proto_deps_export(&proto_dir, &export_dir, false)
        .expect("ensure proto deps export")
}

fn run_protoc_in(out: &Path, layout: &str, extra_opt: &str) {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let proto_dir = manifest_dir.join("examples/proto");
    let deps = ensure_proto_deps_export(&manifest_dir);

    let mut opt = format!("layout={layout},proto_path=.");
    if !extra_opt.is_empty() {
        opt.push(',');
        opt.push_str(extra_opt);
    }

    let status = Command::new(protoc)
        .current_dir(&proto_dir)
        .args([
            "-I",
            ".",
            "-I",
            deps.to_str().expect("utf8 proto-deps path"),
            &format!("--plugin=protoc-gen-mdbook={}", plugin.display()),
            &format!("--mdbook_out={}", out.display()),
            &format!("--mdbook_opt={opt}"),
            "acme/example/v1/echo.proto",
            "acme/example/v1/gateway.proto",
            "acme/example/v2/types.proto",
            "acme/example/v2/catalog.proto",
            "acme/example/v2/services.proto",
            "acme/example/v3alpha1/types.proto",
            "acme/example/v3alpha1/pipeline.proto",
            "acme/example/v3alpha1/services.proto",
        ])
        .status()
        .expect("spawn protoc");
    assert!(status.success(), "protoc layout={layout} opt={extra_opt}");
}

fn run_protoc(layout: &str, extra_opt: &str) -> tempfile::TempDir {
    let out = tempfile::tempdir().expect("tempdir");
    run_protoc_in(out.path(), layout, extra_opt);
    out
}

#[test]
fn links_resolve_package_layout() {
    let out = run_protoc("package", "");
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("package links");
    assert!(!out.path().join("book.toml").exists());
    assert!(
        !out.path().join("src/packages/buf.validate.md").exists(),
        "BSR protovalidate export must not be a protoc input"
    );
    assert!(
        out.path()
            .join("src/packages/acme.example.v1.README.md")
            .is_file(),
        "companion markdown copied beside protos"
    );
}

#[test]
fn optional_fields_preserved_in_generated_markdown() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("vendored protoc");
    let plugin: PathBuf = env!("CARGO_BIN_EXE_protoc-gen-mdbook").into();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let proto_dir = manifest_dir.join("examples/proto");
    let deps = ensure_proto_deps_export(&manifest_dir);
    let out = tempfile::tempdir().expect("tempdir");
    let status = std::process::Command::new(protoc)
        .current_dir(&proto_dir)
        .args([
            "-I",
            ".",
            "-I",
            deps.to_str().expect("utf8 proto-deps path"),
            &format!("--plugin=protoc-gen-mdbook={}", plugin.display()),
            &format!("--mdbook_out={}", out.path().display()),
            "--mdbook_opt=layout=package",
            "acme/example/v1/echo.proto",
        ])
        .status()
        .expect("spawn protoc");
    assert!(status.success());
    let pkg = std::fs::read_to_string(out.path().join("src/packages/acme.example.v1.md"))
        .expect("v1 package md");
    assert!(pkg.contains("optional string locale"));
}

#[test]
fn links_resolve_entity_layout() {
    let out = run_protoc("entity", "");
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("entity links");
}

#[test]
fn links_resolve_split_layout() {
    let out = run_protoc("split", "");
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("split links");
}

#[test]
fn summary_without_init_emits_summary_only() {
    let out = run_protoc("package", "summary");
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("summary links");
    assert!(out.path().join("src/SUMMARY.md").is_file());
    assert!(!out.path().join("book.toml").exists());
}

#[test]
fn custom_markdown_root_and_summary_path() {
    let out = run_protoc(
        "package",
        "markdown_root=content/api,summary_path=content/SUMMARY.md,summary",
    );
    assert!(out.path().join("content/api/acme.example.v1.md").is_file());
    assert!(!out.path().join("src/packages").exists());
    assert!(out.path().join("content/SUMMARY.md").is_file());
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("custom path links");
}

#[test]
fn book_root_prefixes_all_output() {
    let out = run_protoc("package", "book_root=docs");
    assert!(
        out.path()
            .join("docs/src/packages/acme.example.v1.md")
            .is_file()
    );
    assert!(!out.path().join("src/packages").exists());
}

#[test]
fn book_option_infers_paths_from_book_toml() {
    let out = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        out.path().join("book.toml"),
        "[book]\ntitle = \"Test\"\nsrc = \"content\"\n",
    )
    .expect("book.toml");
    std::fs::create_dir_all(out.path().join("content")).expect("content dir");
    let book = out.path().to_string_lossy();
    let opt = format!("book={book},mdbook_out={book},summary");
    run_protoc_in(out.path(), "package", &opt);
    assert!(
        out.path()
            .join("content/packages/acme.example.v1.md")
            .is_file()
    );
    assert!(!out.path().join("src/packages").exists());
    assert!(out.path().join("content/SUMMARY.md").is_file());
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("book= links");
}

#[test]
fn book_option_explicit_markdown_root_overrides_inference() {
    let out = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        out.path().join("book.toml"),
        "[book]\ntitle = \"Test\"\nsrc = \"content\"\n",
    )
    .expect("book.toml");
    std::fs::create_dir_all(out.path().join("content")).expect("content dir");
    let book = out.path().to_string_lossy();
    let opt = format!("book={book},mdbook_out={book},markdown_root=content/api");
    run_protoc_in(out.path(), "package", &opt);
    assert!(out.path().join("content/api/acme.example.v1.md").is_file());
    assert!(!out.path().join("content/packages").exists());
}

#[test]
fn init_writes_mdbook_tree_and_readme() {
    let out = run_protoc("package", "init");
    assert!(out.path().join("book.toml").is_file());
    assert!(out.path().join("README.md").is_file());
    assert!(out.path().join("src/SUMMARY.md").is_file());
    let sum = std::fs::read_to_string(out.path().join("src/SUMMARY.md")).expect("SUMMARY");
    assert!(sum.starts_with("# Protobuf documentation"));
    assert!(sum.contains("acme.example.v1"));
    assert!(sum.contains("acme.README.md"));
    assert!(!sum.contains("Message "));
    assert!(!sum.contains("chapter_1"));
    let readme = std::fs::read_to_string(out.path().join("README.md")).expect("README");
    assert!(readme.contains("mdbook-mermaid"));
    assert!(readme.contains("rumdl"));
    assert!(readme.contains("Syntax highlighting") || readme.contains("CEL"));
    let book = std::fs::read_to_string(out.path().join("book.toml")).expect("book.toml");
    assert!(book.contains("protoc-gen-mdbook: syntax highlighting"));
    assert!(out.path().join("theme/highlight-protobuf.js").is_file());
    assert!(out.path().join("theme/highlight-cel.js").is_file());
    let index = std::fs::read_to_string(out.path().join("theme/index.hbs")).expect("index.hbs");
    assert!(index.contains("protoc-gen-mdbook: syntax highlight begin"));
    assert!(index.contains("hljs.registerLanguage(\"protobuf\""));
    assert!(index.contains("hljs.registerLanguage(\"cel\""));
    assert!(!index.contains(r#"resource "highlight-protobuf.js""#));
    let hl = index.find("highlight.js").expect("highlight.js");
    let marker = index
        .find("protoc-gen-mdbook: syntax highlight begin")
        .expect("marker");
    let bk = index.find("book.js").expect("book.js");
    assert!(hl < marker && marker < bk);
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("init links");
}

#[test]
fn init_no_proto_highlight_skips_protobuf_grammar_only() {
    let out = run_protoc("package", "init,no_proto_highlight");
    assert!(out.path().join("book.toml").is_file());
    assert!(!out.path().join("theme/highlight-protobuf.js").exists());
    assert!(out.path().join("theme/highlight-cel.js").is_file());
    let index = std::fs::read_to_string(out.path().join("theme/index.hbs")).expect("index.hbs");
    assert!(index.contains("protoc-gen-mdbook: syntax highlight begin"));
    assert!(!index.contains("hljs.registerLanguage(\"protobuf\""));
    assert!(index.contains("hljs.registerLanguage(\"cel\""));
}

#[test]
fn init_no_cel_highlight_skips_cel_grammar_only() {
    let out = run_protoc("package", "init,no_cel_highlight");
    assert!(out.path().join("theme/highlight-protobuf.js").is_file());
    assert!(!out.path().join("theme/highlight-cel.js").exists());
    let index = std::fs::read_to_string(out.path().join("theme/index.hbs")).expect("index.hbs");
    assert!(index.contains("hljs.registerLanguage(\"protobuf\""));
    assert!(!index.contains("hljs.registerLanguage(\"cel\""));
}

#[test]
fn package_layout_splits_message_cel_into_cel_fence() {
    let out = run_protoc("package", "");
    let v2 = std::fs::read_to_string(out.path().join("src/packages/acme.example.v2.md"))
        .expect("v2 package md");
    assert!(v2.contains("```cel"));
    assert!(v2.contains("numeric_range.min_lte_max"));
    let numeric = v2
        .split("### NumericRange")
        .nth(1)
        .expect("NumericRange section");
    let proto_fence = numeric
        .split("```protobuf")
        .nth(1)
        .and_then(|s| s.split("```").next())
        .expect("protobuf fence");
    assert!(!proto_fence.contains("buf.validate.message).cel"));
}

#[test]
fn init_entity_layout_summary_is_package_only() {
    let out = run_protoc("entity", "init");
    let sum = std::fs::read_to_string(out.path().join("src/SUMMARY.md")).expect("SUMMARY");
    assert!(!sum.contains("Message "));
    assert!(!sum.contains("Enum "));
    protoc_gen_mdbook::link_check::assert_tree(out.path()).expect("init entity links");
}

#[test]
fn package_layout_preserves_comments_and_mermaid_fences() {
    let out = run_protoc("package", "init");
    let book = std::fs::read_to_string(out.path().join("book.toml")).expect("book.toml");
    assert!(
        !book.contains("[preprocessor.mermaid]"),
        "plugin does not wire mdbook-mermaid; users configure diagrams themselves"
    );
    let pkg = std::fs::read_to_string(out.path().join("src/packages/acme.example.v1.md"))
        .expect("package md");
    let services = pkg.split("## Services").nth(1).expect("Services section");
    assert!(
        services.find("## Messages and enums").unwrap_or(usize::MAX)
            > services.find("### EchoService").unwrap_or(0),
        "Services section should precede Messages and enums"
    );
    let svc = services
        .split("### EchoService")
        .nth(1)
        .expect("EchoService section");
    let mermaid = svc.find("```mermaid").expect("mermaid in service docs");
    let unary_sig = svc.find("**EchoUnary** (").expect("EchoUnary signature");
    assert!(!svc.contains("### EchoUnary ("));
    assert!(mermaid < unary_sig);
    assert!(
        pkg.contains("hello-world RPC"),
        "EchoUnary method leading comments should appear as Markdown"
    );
    let unary_doc = svc.find("hello-world RPC").expect("EchoUnary doc");
    assert!(
        unary_sig < unary_doc,
        "signature and options before RPC prose"
    );
    assert!(
        !svc.contains("service EchoService"),
        "no monolithic service protobuf block"
    );
    assert!(svc.contains("option idempotency_level = NO_SIDE_EFFECTS;"));
}

#[test]
fn package_rpc_type_links_use_mdbook_heading_anchors() {
    let out = run_protoc("package", "");
    let pkg = std::fs::read_to_string(out.path().join("src/packages/acme.example.v1.md"))
        .expect("v1 package md");
    assert!(
        pkg.contains("[BatchEchoResponse](#batchechoresponse)"),
        "in-page links must use mdBook heading ids for RPC response types"
    );
    assert!(
        pkg.contains("**EchoBidiStream**"),
        "bidi RPC should appear in service docs"
    );
}
