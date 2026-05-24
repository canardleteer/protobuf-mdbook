# UploadDraftsResponse

UploadDraftsResponse aggregates uploaded draft parts.

*`acme/example/v2/services.proto`*

```protobuf
message UploadDraftsResponse {
  string draft_id = 1;
  uint32 parts_received = 2;
  Product product = 3;
}
```

