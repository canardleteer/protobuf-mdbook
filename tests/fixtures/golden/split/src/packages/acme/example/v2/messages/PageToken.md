# PageToken

PageToken supports pagination narrative in list RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message PageToken {
  string opaque = 1;
  uint32 page_size = 2;
}
```

