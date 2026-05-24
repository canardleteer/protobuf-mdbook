# RelayConnectResponse

RelayConnectResponse is one server-to-client frame in RelayConnect.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayConnectResponse {
  oneof payload {
      RelayAck ack = 1;
      RelayFrame frame = 2;
      RelayClose close = 3;
    }
}
```

