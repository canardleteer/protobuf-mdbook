//! Types for the `protoc` plugin protocol (`plugin.proto`).
//!
//! Re-exported from [`buffa_descriptor::generated::compiler`](https://docs.rs/buffa-descriptor)
//! for a **buffa**-native surface (no prost).

#![forbid(unsafe_code)]

pub mod codegen;

use buffa::{DecodeOptions, Message};

/// `google.protobuf.FileDescriptorProto` (buffa-generated), for building [`CodeGeneratorRequest::proto_file`].
pub use buffa_descriptor::generated::FileDescriptorProto;
pub use buffa_descriptor::generated::compiler::code_generator_response::File as CodeGeneratorResponseFile;
pub use buffa_descriptor::generated::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};

/// Element-memory budget for trusted descriptor input (plugin request or FDS).
///
/// Buffa 0.9+ defaults to 32 MiB for untrusted wire. Descriptor structs amplify
/// roughly 6x, so that budget rejects schemas of a few hundred files. This
/// matches buffa-codegen's tooling bound.
pub(crate) const DESCRIPTOR_ELEMENT_MEMORY_LIMIT: usize = 1024 * 1024 * 1024;

pub(crate) fn descriptor_decode_options() -> DecodeOptions {
    DecodeOptions::new().with_element_memory_limit(DESCRIPTOR_ELEMENT_MEMORY_LIMIT)
}

pub(crate) fn decode_descriptor_message<M: Message>(bytes: &[u8]) -> Result<M, buffa::DecodeError> {
    descriptor_decode_options().decode_from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use buffa::Message;

    use super::{
        CodeGeneratorRequest, DESCRIPTOR_ELEMENT_MEMORY_LIMIT, decode_descriptor_message,
        descriptor_decode_options,
    };

    #[test]
    fn descriptor_decode_uses_tooling_budget() {
        let opts = descriptor_decode_options();
        assert_eq!(opts.element_memory_limit(), DESCRIPTOR_ELEMENT_MEMORY_LIMIT);
        assert!(opts.element_memory_limit() > buffa::DEFAULT_ELEMENT_MEMORY_LIMIT);
    }

    #[test]
    fn roundtrip_empty_request() {
        let req = CodeGeneratorRequest::default();
        let wire = req.encode_to_vec();
        let got: CodeGeneratorRequest = decode_descriptor_message(&wire).unwrap();
        assert_eq!(got, req);
    }
}
