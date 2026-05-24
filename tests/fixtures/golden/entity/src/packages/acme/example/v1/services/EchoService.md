# EchoService

*`acme/example/v1/echo.proto`*

EchoService exposes echo RPCs for documentation tests.

```mermaid
sequenceDiagram
  participant C as Client
  participant E as EchoService
  C->>E: EchoUnary
  E-->>C: EchoUnaryResponse
  C->>E: EchoServerStream
  loop chunks
    E-->>C: EchoServerStreamResponse
  end
  C->>E: EchoBidiStream
  E-->>C: EchoBidiStreamResponse
```

**EchoUnary** ( [EchoUnaryRequest](../messages/EchoUnaryRequest.md) ) returns ( [EchoUnaryResponse](../messages/EchoUnaryResponse.md) )

```protobuf
option idempotency_level = NO_SIDE_EFFECTS;
```

EchoUnary is the hello-world RPC for this fixture.

| Direction | Type |
|-----------|------|
| request | EchoUnaryRequest |
| response | EchoUnaryResponse |

hello-world RPC — leading comment preserved for mdBook heading link tests.

**EchoServerStream** ( [EchoServerStreamRequest](../messages/EchoServerStreamRequest.md) ) returns ( stream [EchoServerStreamResponse](../messages/EchoServerStreamResponse.md) )

EchoServerStream demonstrates server streaming in generated docs.

**EchoClientStream** ( stream [EchoClientStreamRequest](../messages/EchoClientStreamRequest.md) ) returns ( [EchoClientStreamResponse](../messages/EchoClientStreamResponse.md) )

EchoClientStream demonstrates client streaming uploads.

**EchoBidiStream** ( stream [EchoBidiStreamRequest](../messages/EchoBidiStreamRequest.md) ) returns ( stream [EchoBidiStreamResponse](../messages/EchoBidiStreamResponse.md) )

EchoBidiStream demonstrates bidirectional streaming.

**BatchEcho** ( [BatchEchoRequest](../messages/BatchEchoRequest.md) ) returns ( [BatchEchoResponse](../messages/BatchEchoResponse.md) )

BatchEcho sends many envelopes in one unary call.

Response type BatchEchoResponse is defined in this package; payload uses v2 batch types.

**WatchEcho** ( [WatchEchoRequest](../messages/WatchEchoRequest.md) ) returns ( stream [WatchEchoResponse](../messages/WatchEchoResponse.md) )

WatchEcho pushes events for a subscription (server streaming).

