//! Types for the `protoc` plugin protocol (`plugin.proto`).
//!
//! Re-exported from [`buffa_descriptor::generated::compiler`](https://docs.rs/buffa-descriptor)
//! for a **buffa**-native surface (no prost).

#![forbid(unsafe_code)]

pub mod codegen;

/// `google.protobuf.FileDescriptorProto` (buffa-generated), for building [`CodeGeneratorRequest::proto_file`].
pub use buffa_descriptor::generated::FileDescriptorProto;
pub use buffa_descriptor::generated::compiler::code_generator_response::File as CodeGeneratorResponseFile;
pub use buffa_descriptor::generated::compiler::{CodeGeneratorRequest, CodeGeneratorResponse};

#[cfg(test)]
mod tests {
    use buffa::Message;

    use super::CodeGeneratorRequest;

    #[test]
    fn roundtrip_empty_request() {
        let req = CodeGeneratorRequest::default();
        let wire = req.encode_to_vec();
        let got = CodeGeneratorRequest::decode_from_slice(&wire).unwrap();
        assert_eq!(got, req);
    }
}
