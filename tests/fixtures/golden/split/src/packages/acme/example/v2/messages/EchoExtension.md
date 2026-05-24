# EchoExtension

EchoExtension carries optional hints from v2 into v1 echo flows.

*`acme/example/v2/types.proto`*

```protobuf
message EchoExtension {
  string locale = 1;
  repeated string keywords = 2;
  map<string, string> annotations = 3;
  NumericRange length_bounds = 4;
}
```

