# EchoUnaryRequest

EchoUnaryRequest carries the unary payload.

Fields reference `acme.example.v2.SharedMetadata` for trace identifiers.
See also `acme.example.v2.BatchRequest` for multi-item batch uploads.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoUnaryRequest {
// User-visible text to echo back.
// 
//  Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea
//  commodo consequat.
  string message = 1;
  acme.example.v2.SharedMetadata metadata = 2;
  oneof _locale {
    optional string locale = 3;
  }
  repeated string tags = 4;
  acme.example.v2.EchoExtension extension = 5;
}
```

