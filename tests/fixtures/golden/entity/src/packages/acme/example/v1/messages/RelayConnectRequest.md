# RelayConnectRequest

RelayConnectRequest is one client-to-server frame in RelayConnect.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayConnectRequest {
  oneof payload {
      RelayOpen open = 1;
      RelayFrame frame = 2;
      RelayClose close = 3;
    }
}
```

