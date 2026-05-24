# ErrorDetail

ErrorDetail mirrors a simplified rich error shape for docs.

*`acme/example/v2/types.proto`*

```protobuf
message ErrorDetail {
  string code = 1;
  string message = 2;
  map<string, string> metadata = 3;
  repeated string help_links = 4;
}
```

