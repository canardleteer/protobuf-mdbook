# UploadDraftsRequest

UploadDraftsRequest is one client-streaming product draft chunk.

*`acme/example/v2/services.proto`*

```protobuf
message UploadDraftsRequest {
  string draft_id = 1;
  ProductSku sku = 2;
  uint32 part_index = 3;
  bool last_part = 4;
}
```

