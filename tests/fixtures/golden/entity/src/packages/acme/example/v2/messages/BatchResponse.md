# BatchResponse

BatchResponse collects results for batch RPC documentation.

*`acme/example/v2/types.proto`*

```protobuf
message BatchResponse {
  repeated BatchItemResult results = 1;
  PageResult page = 2;
}
```

