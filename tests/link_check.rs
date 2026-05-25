//! Layout variants: generated markdown links must resolve (protoc plugin and CLI).

mod common;

use common::{run_examples, run_examples_in, run_fixture, run_single_echo_package};

mirrored_test!(links_resolve_package_layout, |backend| {
    let out = run_examples("package", "", backend);
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("package links ({backend:?}): {e}"));
    assert!(!out.path().join("book.toml").exists());
    assert!(
        !out.path().join("src/packages/buf.validate.md").exists(),
        "BSR protovalidate export must not be a protoc input ({backend:?})"
    );
    assert!(
        out.path()
            .join("src/packages/acme.example.v1.README.md")
            .is_file(),
        "companion markdown copied beside protos ({backend:?})"
    );
});

mirrored_test!(optional_fields_preserved_in_generated_markdown, |backend| {
    let out = tempfile::tempdir().expect("tempdir");
    run_single_echo_package(out.path(), backend);
    let pkg = std::fs::read_to_string(out.path().join("src/packages/acme.example.v1.md"))
        .expect("v1 package md");
    assert!(
        pkg.contains("optional string locale"),
        "optional fields preserved ({backend:?})"
    );
});

mirrored_test!(links_resolve_entity_layout, |backend| {
    let out = run_examples("entity", "", backend);
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("entity links ({backend:?}): {e}"));
});

mirrored_test!(links_resolve_split_layout, |backend| {
    let out = run_examples("split", "", backend);
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("split links ({backend:?}): {e}"));
});

mirrored_test!(summary_without_init_emits_summary_only, |backend| {
    let out = run_examples("package", "summary", backend);
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("summary links ({backend:?}): {e}"));
    assert!(out.path().join("src/SUMMARY.md").is_file());
    assert!(!out.path().join("book.toml").exists());
});

mirrored_test!(summary_entity_layout_lists_entities, |backend| {
    let out = run_examples("entity", "summary,no_proto_markdown", backend);
    let sum = std::fs::read_to_string(out.path().join("src/SUMMARY.md")).expect("SUMMARY");
    assert!(sum.contains("Message "));
    assert!(sum.contains("Service "));
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("entity summary links ({backend:?}): {e}"));
});

mirrored_test!(summary_split_layout_lists_entities, |backend| {
    let out = run_examples("split", "summary,no_proto_markdown", backend);
    let sum = std::fs::read_to_string(out.path().join("src/SUMMARY.md")).expect("SUMMARY");
    assert!(sum.contains("Message "));
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("split summary links ({backend:?}): {e}"));
});

#[test]
fn summary_no_proto_markdown_entity_on_fixture() {
    for backend in common::mirrored_fixture_backends() {
        let out = run_fixture("summary,layout=entity,no_proto_markdown", backend);
        let sum = std::fs::read_to_string(out.path().join("src/SUMMARY.md")).expect("SUMMARY");
        assert!(sum.contains("Message "));
        assert!(
            !out.path()
                .join("src/packages/acme.example.v1.README.md")
                .exists(),
            "no_proto_markdown skips companions ({backend:?})"
        );
        protobuf_mdbook::link_check::assert_tree(out.path())
            .unwrap_or_else(|e| panic!("fixture entity summary ({backend:?}): {e}"));
    }
}

mirrored_test!(custom_markdown_root_and_summary_path, |backend| {
    let out = run_examples(
        "package",
        "markdown_root=content/api,summary_path=content/SUMMARY.md,summary",
        backend,
    );
    assert!(out.path().join("content/api/acme.example.v1.md").is_file());
    assert!(!out.path().join("src/packages").exists());
    assert!(out.path().join("content/SUMMARY.md").is_file());
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("custom path links ({backend:?}): {e}"));
});

mirrored_test!(book_root_prefixes_all_output, |backend| {
    let out = run_examples("package", "book_root=docs", backend);
    assert!(
        out.path()
            .join("docs/src/packages/acme.example.v1.md")
            .is_file()
    );
    assert!(!out.path().join("src/packages").exists());
});

mirrored_test!(book_option_infers_paths_from_book_toml, |backend| {
    let out = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        out.path().join("book.toml"),
        "[book]\ntitle = \"Test\"\nsrc = \"content\"\n",
    )
    .expect("book.toml");
    std::fs::create_dir_all(out.path().join("content")).expect("content dir");
    let book = out.path().to_string_lossy();
    let opt = format!("book={book},mdbook_out={book},summary");
    run_examples_in(out.path(), "package", &opt, backend);
    assert!(
        out.path()
            .join("content/packages/acme.example.v1.md")
            .is_file()
    );
    assert!(!out.path().join("src/packages").exists());
    assert!(out.path().join("content/SUMMARY.md").is_file());
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("book= links ({backend:?}): {e}"));
});

mirrored_test!(
    book_option_explicit_markdown_root_overrides_inference,
    |backend| {
        let out = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            out.path().join("book.toml"),
            "[book]\ntitle = \"Test\"\nsrc = \"content\"\n",
        )
        .expect("book.toml");
        std::fs::create_dir_all(out.path().join("content")).expect("content dir");
        let book = out.path().to_string_lossy();
        let opt = format!("book={book},mdbook_out={book},markdown_root=content/api");
        run_examples_in(out.path(), "package", &opt, backend);
        assert!(out.path().join("content/api/acme.example.v1.md").is_file());
        assert!(!out.path().join("content/packages").exists());
    }
);

mirrored_test!(init_writes_mdbook_tree_and_readme, |backend| {
    let out = run_examples("package", "init", backend);
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
    assert!(book.contains("protobuf-mdbook: syntax highlighting"));
    assert!(book.contains("[preprocessor.protobuf-highlight]"));
    assert!(book.contains("mdbook-protobuf-highlight"));
    assert!(book.contains("theme/protobuf-highlight.css"));
    assert!(out.path().join("theme/protobuf-highlight.css").is_file());
    assert!(!out.path().join("theme/highlight-protobuf.js").exists());
    assert!(!out.path().join("theme/highlight-cel.js").exists());
    let index = std::fs::read_to_string(out.path().join("theme/index.hbs")).expect("index.hbs");
    assert!(!index.contains("protobuf-mdbook: syntax highlight begin"));
    assert!(!index.contains("hljs.registerLanguage(\"protobuf\""));
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("init links ({backend:?}): {e}"));
});

mirrored_test!(
    init_no_proto_highlight_skips_protobuf_grammar_only,
    |backend| {
        let out = run_examples("package", "init,no_proto_highlight", backend);
        let book = std::fs::read_to_string(out.path().join("book.toml")).expect("book.toml");
        assert!(book.contains("protobuf = false"));
        assert!(book.contains("cel = true"));
    }
);

mirrored_test!(init_no_cel_highlight_skips_cel_grammar_only, |backend| {
    let out = run_examples("package", "init,no_cel_highlight", backend);
    let book = std::fs::read_to_string(out.path().join("book.toml")).expect("book.toml");
    assert!(book.contains("protobuf = true"));
    assert!(book.contains("cel = false"));
});

mirrored_test!(
    package_layout_splits_message_cel_into_cel_fence,
    |backend| {
        let out = run_examples("package", "", backend);
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
);

mirrored_test!(init_entity_layout_summary_is_package_only, |backend| {
    let out = run_examples("entity", "init", backend);
    let sum = std::fs::read_to_string(out.path().join("src/SUMMARY.md")).expect("SUMMARY");
    assert!(!sum.contains("Message "));
    assert!(!sum.contains("Enum "));
    protobuf_mdbook::link_check::assert_tree(out.path())
        .unwrap_or_else(|e| panic!("init entity links ({backend:?}): {e}"));
});

mirrored_test!(
    package_layout_preserves_comments_and_mermaid_fences,
    |backend| {
        let out = run_examples("package", "init", backend);
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
);

mirrored_test!(
    package_rpc_type_links_use_mdbook_heading_anchors,
    |backend| {
        let out = run_examples("package", "", backend);
        let pkg = std::fs::read_to_string(out.path().join("src/packages/acme.example.v1.md"))
            .expect("v1 package md");
        assert!(
            pkg.contains("[BatchEchoResponse](#batchechoresponse)"),
            "in-page links must use mdBook heading ids for RPC response types ({backend:?})"
        );
        assert!(
            pkg.contains("**EchoBidiStream**"),
            "bidi RPC should appear in service docs ({backend:?})"
        );
    }
);
