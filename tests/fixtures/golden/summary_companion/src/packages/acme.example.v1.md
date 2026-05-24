# acme.example.v1

## Services

### EchoService

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

**EchoUnary** ( [EchoUnaryRequest](#echounaryrequest) ) returns ( [EchoUnaryResponse](#echounaryresponse) )

```protobuf
option idempotency_level = NO_SIDE_EFFECTS;
```

EchoUnary is the hello-world RPC for this fixture.

| Direction | Type |
|-----------|------|
| request | EchoUnaryRequest |
| response | EchoUnaryResponse |

hello-world RPC — leading comment preserved for mdBook heading link tests.

**EchoServerStream** ( [EchoServerStreamRequest](#echoserverstreamrequest) ) returns ( stream [EchoServerStreamResponse](#echoserverstreamresponse) )

EchoServerStream demonstrates server streaming in generated docs.

**EchoClientStream** ( stream [EchoClientStreamRequest](#echoclientstreamrequest) ) returns ( [EchoClientStreamResponse](#echoclientstreamresponse) )

EchoClientStream demonstrates client streaming uploads.

**EchoBidiStream** ( stream [EchoBidiStreamRequest](#echobidistreamrequest) ) returns ( stream [EchoBidiStreamResponse](#echobidistreamresponse) )

EchoBidiStream demonstrates bidirectional streaming.

**BatchEcho** ( [BatchEchoRequest](#batchechorequest) ) returns ( [BatchEchoResponse](#batchechoresponse) )

BatchEcho sends many envelopes in one unary call.

Response type BatchEchoResponse is defined in this package; payload uses v2 batch types.

**WatchEcho** ( [WatchEchoRequest](#watchechorequest) ) returns ( stream [WatchEchoResponse](#watchechoresponse) )

WatchEcho pushes events for a subscription (server streaming).

### GatewayService

*`acme/example/v1/gateway.proto`*

GatewayService focuses on streaming patterns and catalog adjacency.

**RelayConnect** ( stream [RelayConnectRequest](#relayconnectrequest) ) returns ( stream [RelayConnectResponse](#relayconnectresponse) )

RelayConnect opens a bidirectional stream for frame exchange.

**PublishEvents** ( stream [PublishEventsRequest](#publisheventsrequest) ) returns ( [PublishEventsResponse](#publisheventsresponse) )

PublishEvents accepts a client stream of PublishEventsRequest messages.

**ListCatalogProducts** ( [ListCatalogProductsRequest](#listcatalogproductsrequest) ) returns ( [ListCatalogProductsResponse](#listcatalogproductsresponse) )

ListCatalogProducts bridges to v2 catalog types for cross-package links.

### AdminService

*`acme/example/v1/gateway.proto`*

AdminService documents infrequent control-plane RPCs.

**RotateKeys** ( [RotateKeysRequest](#rotatekeysrequest) ) returns ( [RotateKeysResponse](#rotatekeysresponse) )

RotateKeys starts a synthetic long-running operation.

## Messages and enums

### EchoUnaryRequest

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

### EchoUnaryResponse

EchoUnaryResponse returns the echoed text.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoUnaryResponse {
  string message = 1;
  google.protobuf.Timestamp echoed_at = 2;
  acme.example.v2.SharedMetadata metadata = 3;
}
```

### EchoServerStreamRequest

EchoServerStreamRequest opens a server-streaming RPC.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoServerStreamRequest {
  string message = 1;
  uint32 chunk_count = 2;
  google.protobuf.Duration inter_chunk_delay = 3;
  acme.example.v2.SharedMetadata metadata = 4;
}
```

### EchoServerStreamResponse

EchoServerStreamResponse is one chunk in a server stream.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoServerStreamResponse {
  string message = 1;
  acme.example.v2.StreamCursor cursor = 2;
  uint32 index = 3;
}
```

### EchoClientStreamRequest

EchoClientStreamRequest is one chunk in a client stream upload.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoClientStreamRequest {
  string message = 1;
  uint32 part_index = 2;
  bool last_part = 3;
}
```

### EchoClientStreamResponse

EchoClientStreamResponse aggregates a client stream on the server.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoClientStreamResponse {
  string joined_message = 1;
  uint32 parts_received = 2;
  google.protobuf.Timestamp completed_at = 3;
}
```

### EchoBidiStreamRequest

EchoBidiStreamRequest is one frame in a bidirectional echo session.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoBidiStreamRequest {
  string message = 1;
  uint64 sequence = 2;
  bool fin = 3;
}
```

### EchoBidiStreamResponse

EchoBidiStreamResponse mirrors a bidirectional frame back to the client.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoBidiStreamResponse {
  string message = 1;
  uint64 sequence = 2;
  bool fin = 3;
  acme.example.v2.StreamCursor cursor = 4;
}
```

### WatchEchoRequest

WatchEchoRequest subscribes to echo events for a topic.

*`acme/example/v1/echo.proto`*

```protobuf
message WatchEchoRequest {
  string topic = 1;
  acme.example.v2.TimeWindow window = 2;
  repeated acme.example.v2.FilterExpression filters = 3;
}
```

### WatchEchoResponse

WatchEchoResponse is pushed on the WatchEcho server stream.

*`acme/example/v1/echo.proto`*

```protobuf
message WatchEchoResponse {
  string event_id = 1;
  EchoUnaryResponse payload = 2;
  google.protobuf.Timestamp observed_at = 3;
}
```

### BatchEchoRequest

BatchEchoRequest wraps a v2 batch payload for STANDARD RPC naming.

*`acme/example/v1/echo.proto`*

```protobuf
message BatchEchoRequest {
  acme.example.v2.BatchRequest batch = 1;
}
```

### BatchEchoResponse

BatchEchoResponse wraps a v2 batch result for STANDARD RPC naming.

*`acme/example/v1/echo.proto`*

```protobuf
message BatchEchoResponse {
  acme.example.v2.BatchResponse batch = 1;
}
```

### LegacyEchoNote

LegacyEchoNote is an ancillary message for cross-link examples.

*`acme/example/v1/echo.proto`*

```protobuf
message LegacyEchoNote {
  string note_id = 1;
  string body = 2;
  acme.example.v2.SeeAlsoBlock see_also = 3;
}
```

### EchoConfiguration

EchoConfiguration captures static config referenced in comments only.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoConfiguration {
  uint32 default_chunk_size = 1;
  google.protobuf.Duration default_timeout = 2;
  acme.example.v2.RetryPolicy retry_policy = 3;
}
```

### EchoConfigurationRequest

EchoConfigurationRequest fetches EchoConfiguration.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoConfigurationRequest {
  acme.example.v2.TenantRef tenant = 1;
}
```

### EchoConfigurationResponse

EchoConfigurationResponse returns EchoConfiguration.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoConfigurationResponse {
  EchoConfiguration config = 1;
}
```

### DocumentedEchoPair

DocumentedEchoPair ties request/response for See Also blocks.

*`acme/example/v1/echo.proto`*

```protobuf
message DocumentedEchoPair {
  EchoUnaryRequest request = 1;
  EchoUnaryResponse response = 2;
}
```

### EchoHistoryEntry

EchoHistoryEntry is one row in a fictional audit trail.

*`acme/example/v1/echo.proto`*

```protobuf
message EchoHistoryEntry {
  google.protobuf.Timestamp at = 1;
  string rpc_name = 2;
  EchoUnaryRequest unary_request = 3;
  acme.example.v2.ErrorDetail error = 4;
}
```

### ListEchoHistoryRequest

ListEchoHistoryRequest lists EchoHistoryEntry rows.

*`acme/example/v1/echo.proto`*

```protobuf
message ListEchoHistoryRequest {
  acme.example.v2.ListOptions options = 1;
  acme.example.v2.ResourceIdentity identity = 2;
}
```

### ListEchoHistoryResponse

ListEchoHistoryResponse returns history rows.

*`acme/example/v1/echo.proto`*

```protobuf
message ListEchoHistoryResponse {
  repeated EchoHistoryEntry entries = 1;
  acme.example.v2.PageResult page = 2;
}
```

### RelayFrame

RelayFrame is one unit in a bidirectional relay stream.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayFrame {
  uint64 sequence = 1;
  bytes payload = 2;
  acme.example.v2.SharedMetadata metadata = 3;
  bool fin = 4;
}
```

### RelayOpen

RelayOpen carries handshake metadata for RelayConnect.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayOpen {
  string session_name = 1;
  acme.example.v2.ResourceIdentity identity = 2;
  acme.example.v2.QuotaLimits requested_limits = 3;
  repeated acme.example.v2.Label labels = 4;
}
```

### RelayAck

RelayAck confirms a relay session is ready.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayAck {
  string session_id = 1;
  google.protobuf.Timestamp opened_at = 2;
  acme.example.v2.AggregateHealth health = 3;
}
```

### RelayClose

RelayClose ends a relay session gracefully.

*`acme/example/v1/gateway.proto`*

```protobuf
message RelayClose {
  string session_id = 1;
  string reason = 2;
}
```

### RelayConnectRequest

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

### RelayConnectResponse

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

### PublishEventsRequest

PublishEventsRequest is one client-streaming upload chunk.

*`acme/example/v1/gateway.proto`*

```protobuf
message PublishEventsRequest {
  string event_type = 1;
  acme.example.v2.PayloadEnvelope envelope = 2;
  google.protobuf.Timestamp client_time = 3;
}
```

### PublishEventsResponse

PublishEventsResponse acknowledges a published event stream.

*`acme/example/v1/gateway.proto`*

```protobuf
message PublishEventsResponse {
  string event_id = 1;
  acme.example.v2.StreamCursor cursor = 2;
}
```

### ListCatalogProductsRequest

ListCatalogProductsRequest wraps the v2 catalog list request.

*`acme/example/v1/gateway.proto`*

```protobuf
message ListCatalogProductsRequest {
  acme.example.v2.ListProductsRequest request = 1;
}
```

### ListCatalogProductsResponse

ListCatalogProductsResponse wraps the v2 catalog list response.

*`acme/example/v1/gateway.proto`*

```protobuf
message ListCatalogProductsResponse {
  acme.example.v2.ListProductsResponse response = 1;
}
```

### RotateKeysRequest

RotateKeysRequest triggers a long-running operation example.

*`acme/example/v1/gateway.proto`*

```protobuf
message RotateKeysRequest {
  acme.example.v2.TenantRef tenant = 1;
  bool dry_run = 2;
}
```

### RotateKeysResponse

RotateKeysResponse returns an operation handle.

*`acme/example/v1/gateway.proto`*

```protobuf
message RotateKeysResponse {
  acme.example.v2.LongRunningOperation operation = 1;
}
```

