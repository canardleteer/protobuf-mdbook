# PlatformService

*`acme/example/v2/services.proto`*

PlatformService documents cross-cutting telemetry and health RPCs.

**ExportAuditBatch** ( [ExportAuditBatchRequest](../messages/ExportAuditBatchRequest.md) ) returns ( [ExportAuditBatchResponse](../messages/ExportAuditBatchResponse.md) )

ExportAuditBatch returns a snapshot of audit records (unary).

**StreamAuditRecords** ( [StreamAuditRecordsRequest](../messages/StreamAuditRecordsRequest.md) ) returns ( stream [StreamAuditRecordsResponse](../messages/StreamAuditRecordsResponse.md) )

StreamAuditRecords pushes audit rows over a server stream.

**IngestTelemetry** ( stream [IngestTelemetryRequest](../messages/IngestTelemetryRequest.md) ) returns ( [IngestTelemetryResponse](../messages/IngestTelemetryResponse.md) )

IngestTelemetry accepts a client stream of metric samples.

**GetAggregateHealth** ( [GetAggregateHealthRequest](../messages/GetAggregateHealthRequest.md) ) returns ( [GetAggregateHealthResponse](../messages/GetAggregateHealthResponse.md) )

GetAggregateHealth returns synthetic component health.

