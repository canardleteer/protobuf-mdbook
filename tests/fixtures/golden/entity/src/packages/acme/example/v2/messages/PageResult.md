# PageResult

PageResult completes a paginated list response.

*`acme/example/v2/types.proto`*

```protobuf
message PageResult {
  PageToken next_page_token = 1;
  uint32 total_size = 2;
}
```

