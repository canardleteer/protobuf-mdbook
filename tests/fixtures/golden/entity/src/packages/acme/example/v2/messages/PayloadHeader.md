# PayloadHeader

PayloadHeader precedes user bytes in envelope examples.

*`acme/example/v2/types.proto`*

```protobuf
message PayloadHeader {
  string content_type = 1;
  uint64 content_length = 2;
  string checksum_sha256 = 3;
  Priority priority = 4;
}
```

