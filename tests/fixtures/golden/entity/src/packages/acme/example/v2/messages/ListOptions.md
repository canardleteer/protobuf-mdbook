# ListOptions

ListOptions bundles pagination and sorting for list RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message ListOptions {
  PageToken page = 1;
  repeated SortKey sort = 2;
  repeated FilterExpression filters = 3;
}
```

