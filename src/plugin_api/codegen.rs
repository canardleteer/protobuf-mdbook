//! Descriptor helpers for protoc code generation.

use buffa_descriptor::generated::descriptor::MethodDescriptorProto;

/// gRPC / Connect streaming classification for a single `MethodDescriptorProto`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpcKind {
    Unary,
    ClientStreaming,
    ServerStreaming,
    BidiStreaming,
}

pub fn rpc_kind(method: &MethodDescriptorProto) -> RpcKind {
    let client = method.client_streaming.unwrap_or(false);
    let server = method.server_streaming.unwrap_or(false);
    match (client, server) {
        (false, false) => RpcKind::Unary,
        (true, false) => RpcKind::ClientStreaming,
        (false, true) => RpcKind::ServerStreaming,
        (true, true) => RpcKind::BidiStreaming,
    }
}

/// Split a fully-qualified protobuf type name (`.pkg.Type`) into package + message.
///
/// Returns `None` if `fqn` is missing a leading `.`, has no inner `.`, or has empty segments.
pub fn split_proto_type_name(fqn: &str) -> Option<(&str, &str)> {
    let s = fqn.strip_prefix('.')?;
    let (pkg, msg) = s.rsplit_once('.')?;
    if pkg.is_empty() || msg.is_empty() {
        return None;
    }
    Some((pkg, msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa_descriptor::generated::descriptor::MethodDescriptorProto;

    #[test]
    fn rpc_kind_matrix() {
        let mut m = MethodDescriptorProto::default();
        assert_eq!(rpc_kind(&m), RpcKind::Unary);
        m.client_streaming = Some(true);
        assert_eq!(rpc_kind(&m), RpcKind::ClientStreaming);
        m.server_streaming = Some(true);
        assert_eq!(rpc_kind(&m), RpcKind::BidiStreaming);
        m.client_streaming = Some(false);
        assert_eq!(rpc_kind(&m), RpcKind::ServerStreaming);
    }

    #[test]
    fn split_proto_type_name_acme_echo() {
        assert_eq!(
            split_proto_type_name(".acme.example.v1.EchoUnaryRequest"),
            Some(("acme.example.v1", "EchoUnaryRequest"))
        );
    }
}
