# RelayClose

RelayClose ends a relay session gracefully.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayClose {
  string session_id = 1;
  string reason = 2;
}
```

