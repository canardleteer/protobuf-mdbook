//! Synthesize protobuf source snippets from descriptors (mdBook `protobuf` fences).
//!
//! **Comment policy:** `SourceCodeInfo` *leading* comments on the entity (message,
//! enum, service, RPC) are emitted as Markdown *before* the fence. Leading comments
//! on fields and enum values stay as `//` lines inside the synthesized block.

mod enum_;
mod fence;
mod message;
mod rpc;

use crate::options::EscapeTags;
use std::path::Path;

pub use enum_::synthesize_enum;
pub use message::synthesize_message_with_file;
pub use rpc::synthesize_service;

pub struct RenderContext<'a> {
    pub links: Option<&'a crate::render::links::LinkContext>,
    pub from_md: &'a Path,
    pub escape_tags: EscapeTags,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::EscapeTags;
    use crate::render::comments::CommentIndex;
    use buffa_descriptor::generated::descriptor::method_options::IdempotencyLevel;
    use buffa_descriptor::generated::descriptor::{
        DescriptorProto, FieldDescriptorProto, MethodDescriptorProto, MethodOptions,
        OneofDescriptorProto, ServiceDescriptorProto,
    };
    use fence::render_proto_fence;
    use rpc::{rpc_signature_markdown, synthesize_method_options_body};

    #[test]
    fn rpc_signature_server_streaming() {
        let m = MethodDescriptorProto {
            name: Some("EchoServerStream".into()),
            input_type: Some(".acme.example.v1.EchoServerStreamRequest".into()),
            output_type: Some(".acme.example.v1.EchoServerStreamResponse".into()),
            client_streaming: Some(false),
            server_streaming: Some(true),
            ..Default::default()
        };
        let line = rpc_signature_markdown(&m, None);
        assert!(line.contains("**EchoServerStream**"));
        assert!(line.contains("returns ( stream "));
    }

    #[test]
    fn method_options_idempotency() {
        let m = MethodDescriptorProto {
            options: MethodOptions {
                idempotency_level: Some(IdempotencyLevel::NO_SIDE_EFFECTS),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        };
        let body = synthesize_method_options_body(&m).expect("options");
        assert!(body.contains("option idempotency_level = NO_SIDE_EFFECTS;"));
    }

    #[test]
    fn entity_doc_outside_fence() {
        let md = render_proto_fence(
            "foo.proto",
            Some("```mermaid\nflowchart LR\n  A --> B\n```"),
            "message X {}\n",
            EscapeTags::Off,
        );
        assert!(md.contains("```mermaid"));
        let fence_start = md.find("```protobuf").expect("fence");
        let mermaid_start = md.find("```mermaid").expect("mermaid");
        assert!(mermaid_start < fence_start);
        assert!(!md[fence_start..].contains("```mermaid"));
    }

    #[test]
    fn message_oneof_block_from_descriptor() {
        let msg = DescriptorProto {
            name: Some("RelayConnectRequest".into()),
            oneof_decl: vec![OneofDescriptorProto {
                name: Some("payload".into()),
                ..Default::default()
            }],
            field: vec![
                FieldDescriptorProto {
                    name: Some("open".into()),
                    number: Some(1),
                    type_name: Some(".acme.example.v1.RelayOpen".into()),
                    oneof_index: Some(0),
                    ..Default::default()
                },
                FieldDescriptorProto {
                    name: Some("frame".into()),
                    number: Some(2),
                    type_name: Some(".acme.example.v1.RelayFrame".into()),
                    oneof_index: Some(0),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let file = crate::plugin_api::FileDescriptorProto::default();
        let idx = CommentIndex::from_file(&file);
        let md = synthesize_message_with_file("gateway.proto", &idx, 0, &msg, None, None);
        assert!(md.contains("oneof payload"));
        assert!(md.contains("RelayOpen open = 1;"));
        let oneof_pos = md.find("oneof payload").expect("oneof");
        let open_pos = md.find("RelayOpen open").expect("open");
        assert!(oneof_pos < open_pos);
    }

    #[test]
    fn service_rpc_order_signature_options_doc() {
        let file = crate::plugin_api::FileDescriptorProto::default();
        let idx = CommentIndex::from_file(&file);
        let m = MethodDescriptorProto {
            name: Some("GetCommits".into()),
            input_type: Some(".pkg.GetCommitsRequest".into()),
            output_type: Some(".pkg.GetCommitsResponse".into()),
            options: MethodOptions {
                idempotency_level: Some(IdempotencyLevel::NO_SIDE_EFFECTS),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        };
        let svc = ServiceDescriptorProto {
            name: Some("CommitService".into()),
            method: vec![m],
            ..Default::default()
        };
        let md = synthesize_service("commit.proto", &idx, 0, &svc, 3, None);
        let sig = md.find("**GetCommits**").expect("signature line");
        let opt = md.find("option idempotency_level").expect("options");
        assert!(sig < opt);
        assert!(!md.contains("service CommitService"));
    }
}
