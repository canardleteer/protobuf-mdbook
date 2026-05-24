# acme.example.v2

## Services

### CatalogService

*`acme/example/v2/services.proto`*

CatalogService documents commerce and inventory RPCs in the v2 package.

**ListProducts** ( [ListProductsRequest](#listproductsrequest) ) returns ( [ListProductsResponse](#listproductsresponse) )

ListProducts returns a paginated product catalog page.

**GetProduct** ( [GetProductRequest](#getproductrequest) ) returns ( [GetProductResponse](#getproductresponse) )

GetProduct fetches a single product by identifier.

**ApplyInventoryAdjustments** ( [ApplyInventoryAdjustmentsRequest](#applyinventoryadjustmentsrequest) ) returns ( [ApplyInventoryAdjustmentsResponse](#applyinventoryadjustmentsresponse) )

ApplyInventoryAdjustments applies batched stock deltas.

**WatchInventory** ( [WatchInventoryRequest](#watchinventoryrequest) ) returns ( stream [WatchInventoryResponse](#watchinventoryresponse) )

WatchInventory streams inventory adjustments for a warehouse.

**UploadDrafts** ( stream [UploadDraftsRequest](#uploaddraftsrequest) ) returns ( [UploadDraftsResponse](#uploaddraftsresponse) )

UploadDrafts accepts a client stream of draft SKU payloads.

**SyncCatalog** ( stream [SyncCatalogRequest](#synccatalogrequest) ) returns ( stream [SyncCatalogResponse](#synccatalogresponse) )

SyncCatalog synchronizes catalog revisions over a bidirectional stream.

### PlatformService

*`acme/example/v2/services.proto`*

PlatformService documents cross-cutting telemetry and health RPCs.

**ExportAuditBatch** ( [ExportAuditBatchRequest](#exportauditbatchrequest) ) returns ( [ExportAuditBatchResponse](#exportauditbatchresponse) )

ExportAuditBatch returns a snapshot of audit records (unary).

**StreamAuditRecords** ( [StreamAuditRecordsRequest](#streamauditrecordsrequest) ) returns ( stream [StreamAuditRecordsResponse](#streamauditrecordsresponse) )

StreamAuditRecords pushes audit rows over a server stream.

**IngestTelemetry** ( stream [IngestTelemetryRequest](#ingesttelemetryrequest) ) returns ( [IngestTelemetryResponse](#ingesttelemetryresponse) )

IngestTelemetry accepts a client stream of metric samples.

**GetAggregateHealth** ( [GetAggregateHealthRequest](#getaggregatehealthrequest) ) returns ( [GetAggregateHealthResponse](#getaggregatehealthresponse) )

GetAggregateHealth returns synthetic component health.

## Messages and enums

### SharedMetadata

SharedMetadata is referenced from v1 echo and gateway messages.

Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque
laudantium. Keep `trace_id` opaque to callers; format is service-specific.

*`acme/example/v2/types.proto`*

```protobuf
message SharedMetadata {
// Distributed trace id (W3C `traceparent` or equivalent).
  string trace_id = 1 [(buf.validate.field).string.min_len = 1];
// Optional parent span for nested calls.
  string parent_span_id = 2;
// When the trace context was minted.
  google.protobuf.Timestamp created_at = 3;
// Arbitrary baggage for examples (not a production pattern).
  map<string, string> baggage = 4;
// Classification for routing in docs.
  SharedKind kind = 5;
}
```

### TraceContext

TraceContext duplicates some metadata for nested RPC examples.

*`acme/example/v2/types.proto`*

```protobuf
message TraceContext {
  string trace_id = 1;
  string span_id = 2;
  google.protobuf.Duration sampling_delay = 3;
}
```

### PayloadHeader

PayloadHeader precedes user bytes in envelope examples.

*`acme/example/v2/types.proto`*

```protobuf
message PayloadHeader {
  string content_type = 1;
  uint64 content_length = 2;
  string checksum_sha256 = 3;
  Priority priority = 4;
}
```

### PayloadEnvelope

PayloadEnvelope wraps opaque bytes with metadata.

*`acme/example/v2/types.proto`*

```protobuf
message PayloadEnvelope {
  PayloadHeader header = 1;
  bytes body = 2;
  SharedMetadata metadata = 3;
}
```

### AuditRecord

AuditRecord captures a single synthetic audit line.

*`acme/example/v2/types.proto`*

```protobuf
message AuditRecord {
  string actor = 1;
  string action = 2;
  google.protobuf.Timestamp occurred_at = 3;
  google.protobuf.Struct details = 4;
}
```

### AuditBatch

AuditBatch groups records for batch RPC demonstrations.

*`acme/example/v2/types.proto`*

```protobuf
message AuditBatch {
  repeated AuditRecord records = 1;
  string batch_id = 2;
}
```

### Location

Location describes a fictional region for catalog cross-links.

*`acme/example/v2/types.proto`*

```protobuf
message Location {
  string region_code = 1;
  string display_name = 2;
  double latitude = 3;
  double longitude = 4;
}
```

### Address

Address is a postal-style structure used in nested messages.

*`acme/example/v2/types.proto`*

```protobuf
message Address {
  string line1 = 1;
  string line2 = 2;
  string city = 3;
  string postal_code = 4;
  Location region = 5;
}
```

### ContactInfo

ContactInfo supports oneof-based documentation rendering.

*`acme/example/v2/types.proto`*

```protobuf
message ContactInfo {
  oneof channel {
      string email = 1;
      string phone_e164 = 2;
      string slack_user_id = 3;
    }
  string display_name = 4;
}
```

### TenantRef

TenantRef identifies a fictional tenant for multi-tenant stories.

*`acme/example/v2/types.proto`*

```protobuf
message TenantRef {
  string tenant_id = 1 [(buf.validate.field).string.uuid = true];
  string slug = 2 [
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 64,
      (buf.validate.field).string.pattern = "^[a-z0-9-]+$"
    ];
  SharedKind tier = 3;
}
```

### QuotaLimits

QuotaLimits documents soft limits referenced from gateway RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message QuotaLimits {
  uint32 max_requests_per_minute = 1;
  uint32 max_stream_duration_seconds = 2;
  uint64 max_payload_bytes = 3;
}
```

### ErrorDetail

ErrorDetail mirrors a simplified rich error shape for docs.

*`acme/example/v2/types.proto`*

```protobuf
message ErrorDetail {
  string code = 1;
  string message = 2;
  map<string, string> metadata = 3;
  repeated string help_links = 4;
}
```

### RetryPolicy

RetryPolicy is embedded in long-running operation messages.

*`acme/example/v2/types.proto`*

```protobuf
message RetryPolicy {
  uint32 max_attempts = 1;
  google.protobuf.Duration initial_backoff = 2;
  google.protobuf.Duration max_backoff = 3;
  double backoff_multiplier = 4;
}
```

### LongRunningOperation

LongRunningOperation is returned by admin-style RPCs in v1.

*`acme/example/v2/types.proto`*

```protobuf
message LongRunningOperation {
  string name = 1;
  OperationStatus status = 2;
  google.protobuf.Timestamp start_time = 3;
  google.protobuf.Timestamp end_time = 4;
  RetryPolicy retry_policy = 5;
  ErrorDetail error = 6;
  double percent_complete = 7;
}
```

### Label

Label is a key/value tag used across catalog and echo fixtures.

*`acme/example/v2/types.proto`*

```protobuf
message Label {
  string key = 1;
  string value = 2;
}
```

### LabelSet

LabelSet aggregates labels for resource descriptions.

*`acme/example/v2/types.proto`*

```protobuf
message LabelSet {
  repeated Label labels = 1;
}
```

### ResourceIdentity

ResourceIdentity combines tenant, labels, and metadata.

*`acme/example/v2/types.proto`*

```protobuf
message ResourceIdentity {
  TenantRef tenant = 1;
  LabelSet labels = 2;
  SharedMetadata metadata = 3;
  string resource_name = 4;
}
```

### NumericRange

NumericRange supports validation comment examples.

*`acme/example/v2/types.proto`*

```protobuf
message NumericRange {
  
  int64 min_inclusive = 1;
  int64 max_inclusive = 2;
}
```

**Protovalidate (CEL)**

```cel
id: "numeric_range.min_lte_max"
      message: "min_inclusive must not exceed max_inclusive"
      expression: "this.min_inclusive <= this.max_inclusive"
```

### TimeWindow

TimeWindow defines inclusive bounds for queries.

*`acme/example/v2/types.proto`*

```protobuf
message TimeWindow {
  google.protobuf.Timestamp start = 1;
  google.protobuf.Timestamp end = 2;
}
```

### FilterExpression

FilterExpression is a intentionally verbose filter AST placeholder.

*`acme/example/v2/types.proto`*

```protobuf
message FilterExpression {
  string field = 1;
  string op = 2;
  string value = 3;
  repeated FilterExpression children = 4;
}
```

### PageToken

PageToken supports pagination narrative in list RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message PageToken {
  string opaque = 1;
  uint32 page_size = 2;
}
```

### PageResult

PageResult completes a paginated list response.

*`acme/example/v2/types.proto`*

```protobuf
message PageResult {
  PageToken next_page_token = 1;
  uint32 total_size = 2;
}
```

### SortKey

SortKey pairs a field name with an order.

*`acme/example/v2/types.proto`*

```protobuf
message SortKey {
  string field = 1;
  SortOrder order = 2;
}
```

### ListOptions

ListOptions bundles pagination and sorting for list RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message ListOptions {
  PageToken page = 1;
  repeated SortKey sort = 2;
  repeated FilterExpression filters = 3;
}
```

### ComponentHealth

ComponentHealth describes one sub-system in aggregate health.

*`acme/example/v2/types.proto`*

```protobuf
message ComponentHealth {
  string component = 1;
  HealthStatus status = 2;
  string detail = 3;
}
```

### AggregateHealth

AggregateHealth rolls up component statuses.

*`acme/example/v2/types.proto`*

```protobuf
message AggregateHealth {
  HealthStatus overall = 1;
  repeated ComponentHealth components = 2;
  google.protobuf.Timestamp evaluated_at = 3;
}
```

### StreamCursor

StreamCursor supports resumable stream examples.

*`acme/example/v2/types.proto`*

```protobuf
message StreamCursor {
  string stream_id = 1;
  uint64 sequence = 2;
  google.protobuf.Timestamp emitted_at = 3;
}
```

### StreamChunk

StreamChunk is a unit payload in streaming RPC fixtures.

*`acme/example/v2/types.proto`*

```protobuf
message StreamChunk {
  StreamCursor cursor = 1;
  PayloadEnvelope payload = 2;
  bool terminal = 3;
}
```

### BatchKey

BatchKey identifies an item inside batch requests.

*`acme/example/v2/types.proto`*

```protobuf
message BatchKey {
  string id = 1;
  string correlation_id = 2;
}
```

### BatchItem

BatchItem pairs a key with an envelope for batch APIs.

*`acme/example/v2/types.proto`*

```protobuf
message BatchItem {
  BatchKey key = 1;
  PayloadEnvelope envelope = 2;
}
```

### BatchRequest

BatchRequest aggregates many items for BatchEcho RPCs.

*`acme/example/v2/types.proto`*

```protobuf
message BatchRequest {
  repeated BatchItem items = 1 [(buf.validate.field).repeated.min_items = 1];
  ResourceIdentity identity = 2;
  bool partial_failure_allowed = 3;
}
```

### BatchItemResult

BatchItemResult reports per-item outcomes.

*`acme/example/v2/types.proto`*

```protobuf
message BatchItemResult {
  BatchKey key = 1;
  bool ok = 2;
  ErrorDetail error = 3;
  PayloadEnvelope response_envelope = 4;
}
```

### BatchResponse

BatchResponse collects results for batch RPC documentation.

*`acme/example/v2/types.proto`*

```protobuf
message BatchResponse {
  repeated BatchItemResult results = 1;
  PageResult page = 2;
}
```

### EchoExtension

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

### DocumentationAnchor

DocumentationAnchor is a meta-message for cross-link stress tests.

*`acme/example/v2/types.proto`*

```protobuf
message DocumentationAnchor {
  string title = 1;
  string href = 2;
  string summary = 3;
}
```

### SeeAlsoBlock

SeeAlsoBlock groups anchors for See Also sections in comments.

*`acme/example/v2/types.proto`*

```protobuf
message SeeAlsoBlock {
  repeated DocumentationAnchor anchors = 1;
}
```

### SharedKind

SharedKind classifies metadata rows in examples.

| Value | Meaning |
|-------|---------|
| UNSPECIFIED | default |
| ALPHA | first variant |
| BETA | second variant |
| GAMMA | third variant |

*`acme/example/v2/types.proto`*

```protobuf
enum SharedKind {
  SHARED_KIND_UNSPECIFIED = 0;
  SHARED_KIND_ALPHA = 1;
  SHARED_KIND_BETA = 2;
  SHARED_KIND_GAMMA = 3;
}
```

### Priority

Priority influences scheduler hints in narrative docs only.

*`acme/example/v2/types.proto`*

```protobuf
enum Priority {
  PRIORITY_UNSPECIFIED = 0;
  PRIORITY_LOW = 1;
  PRIORITY_NORMAL = 2;
  PRIORITY_HIGH = 3;
  PRIORITY_CRITICAL = 4;
}
```

### OperationStatus

OperationStatus tracks synthetic long-running work.

*`acme/example/v2/types.proto`*

```protobuf
enum OperationStatus {
  OPERATION_STATUS_UNSPECIFIED = 0;
  OPERATION_STATUS_PENDING = 1;
  OPERATION_STATUS_RUNNING = 2;
  OPERATION_STATUS_SUCCEEDED = 3;
  OPERATION_STATUS_FAILED = 4;
  OPERATION_STATUS_CANCELLED = 5;
}
```

### SortOrder

SortOrder documents ascending/descending list semantics.

*`acme/example/v2/types.proto`*

```protobuf
enum SortOrder {
  SORT_ORDER_UNSPECIFIED = 0;
  SORT_ORDER_ASC = 1;
  SORT_ORDER_DESC = 2;
}
```

### HealthStatus

HealthStatus is reported by gateway health RPCs.

*`acme/example/v2/types.proto`*

```protobuf
enum HealthStatus {
  HEALTH_STATUS_UNSPECIFIED = 0;
  HEALTH_STATUS_SERVING = 1;
  HEALTH_STATUS_NOT_SERVING = 2;
  HEALTH_STATUS_DEGRADED = 3;
}
```

### ProductSku

ProductSku identifies a sellable item in documentation tables.

*`acme/example/v2/catalog.proto`*

```protobuf
message ProductSku {
  string sku = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 64
    ];
  string title = 2;
  string description = 3;
  Money price = 4;
  ProductStatus status = 5;
  repeated Label labels = 6;
}
```

### Money

Money represents a decimal amount with currency code.

*`acme/example/v2/catalog.proto`*

```protobuf
message Money {
  int64 units = 1;
  int32 nanos = 2;
  string currency_code = 3 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.len = 3
    ];
}
```

### Product

Product bundles many SKUs for list RPC examples.

*`acme/example/v2/catalog.proto`*

```protobuf
message Product {
  string product_id = 1;
  string display_name = 2;
  repeated ProductSku skus = 3;
  TenantRef owner = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp updated_at = 6;
  Address warehouse = 7;
}
```

### ListProductsRequest

ListProductsRequest paginates catalog inventory.

*`acme/example/v2/catalog.proto`*

```protobuf
message ListProductsRequest {
  ListOptions options = 1;
  ProductStatus status_filter = 2;
  string search_query = 3 [(buf.validate.field).string.max_len = 256];
}
```

### ListProductsResponse

ListProductsResponse returns a page of products.

*`acme/example/v2/catalog.proto`*

```protobuf
message ListProductsResponse {
  repeated Product products = 1;
  PageResult page = 2;
}
```

### GetProductRequest

GetProductRequest fetches a single product by id.

*`acme/example/v2/catalog.proto`*

```protobuf
message GetProductRequest {
  string product_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.uuid = true
    ];
}
```

### GetProductResponse

GetProductResponse returns one product.

*`acme/example/v2/catalog.proto`*

```protobuf
message GetProductResponse {
  Product product = 1;
}
```

### InventoryAdjustment

InventoryAdjustment documents stock changes for admin RPCs.

*`acme/example/v2/catalog.proto`*

```protobuf
message InventoryAdjustment {
  string sku = 1;
  int64 delta = 2;
  string reason = 3;
  google.protobuf.Timestamp adjusted_at = 4;
}
```

### ApplyInventoryAdjustmentsRequest

ApplyInventoryAdjustmentsRequest batches adjustments.

*`acme/example/v2/catalog.proto`*

```protobuf
message ApplyInventoryAdjustmentsRequest {
  repeated InventoryAdjustment adjustments = 1;
  ResourceIdentity actor = 2;
}
```

### ApplyInventoryAdjustmentsResponse

ApplyInventoryAdjustmentsResponse summarizes applied rows.

*`acme/example/v2/catalog.proto`*

```protobuf
message ApplyInventoryAdjustmentsResponse {
  uint32 applied = 1;
  repeated ErrorDetail failures = 2;
}
```

### ProductStatus

ProductStatus lifecycle for catalog stories.

*`acme/example/v2/catalog.proto`*

```protobuf
enum ProductStatus {
  PRODUCT_STATUS_UNSPECIFIED = 0;
  PRODUCT_STATUS_DRAFT = 1;
  PRODUCT_STATUS_ACTIVE = 2;
  PRODUCT_STATUS_ARCHIVED = 3;
  PRODUCT_STATUS_DISCONTINUED = 4;
}
```

### WatchInventoryRequest

WatchInventoryRequest subscribes to inventory change events.

*`acme/example/v2/services.proto`*

```protobuf
message WatchInventoryRequest {
  string warehouse_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 128
    ];
  TimeWindow window = 2;
  repeated FilterExpression filters = 3;
}
```

### WatchInventoryResponse

WatchInventoryResponse is one event on the WatchInventory server stream.

*`acme/example/v2/services.proto`*

```protobuf
message WatchInventoryResponse {
  string event_id = 1;
  InventoryAdjustment adjustment = 2;
  google.protobuf.Timestamp observed_at = 3;
  StreamCursor cursor = 4;
}
```

### UploadDraftsRequest

UploadDraftsRequest is one client-streaming product draft chunk.

*`acme/example/v2/services.proto`*

```protobuf
message UploadDraftsRequest {
  string draft_id = 1;
  ProductSku sku = 2;
  uint32 part_index = 3;
  bool last_part = 4;
}
```

### UploadDraftsResponse

UploadDraftsResponse aggregates uploaded draft parts.

*`acme/example/v2/services.proto`*

```protobuf
message UploadDraftsResponse {
  string draft_id = 1;
  uint32 parts_received = 2;
  Product product = 3;
}
```

### SyncCatalogRequest

SyncCatalogRequest is one frame in a bidirectional catalog sync session.

*`acme/example/v2/services.proto`*

```protobuf
message SyncCatalogRequest {
  uint64 sequence = 1;
  string catalog_revision = 2;
  PayloadEnvelope payload = 3;
  bool fin = 4;
}
```

### SyncCatalogResponse

SyncCatalogResponse mirrors a bidirectional catalog sync frame.

*`acme/example/v2/services.proto`*

```protobuf
message SyncCatalogResponse {
  uint64 sequence = 1;
  StreamCursor cursor = 2;
  ErrorDetail error = 3;
  bool fin = 4;
}
```

### ExportAuditBatchRequest

ExportAuditBatchRequest exports a batch of audit rows (unary).

*`acme/example/v2/services.proto`*

```protobuf
message ExportAuditBatchRequest {
  ResourceIdentity identity = 1;
  TimeWindow window = 2;
  ListOptions options = 3;
}
```

### ExportAuditBatchResponse

ExportAuditBatchResponse returns exported audit data.

*`acme/example/v2/services.proto`*

```protobuf
message ExportAuditBatchResponse {
  AuditBatch batch = 1;
  PageResult page = 2;
}
```

### StreamAuditRecordsRequest

StreamAuditRecordsRequest opens a server stream of audit rows.

*`acme/example/v2/services.proto`*

```protobuf
message StreamAuditRecordsRequest {
  TenantRef tenant = 1;
  repeated FilterExpression filters = 2;
}
```

### StreamAuditRecordsResponse

StreamAuditRecordsResponse is one audit row on the stream.

*`acme/example/v2/services.proto`*

```protobuf
message StreamAuditRecordsResponse {
  AuditRecord record = 1;
  StreamCursor cursor = 2;
}
```

### IngestTelemetryRequest

IngestTelemetryRequest is one client-streaming telemetry point.

*`acme/example/v2/services.proto`*

```protobuf
message IngestTelemetryRequest {
  string metric_name = 1;
  double value = 2;
  google.protobuf.Timestamp observed_at = 3;
  LabelSet labels = 4;
}
```

### IngestTelemetryResponse

IngestTelemetryResponse acknowledges ingested points.

*`acme/example/v2/services.proto`*

```protobuf
message IngestTelemetryResponse {
  uint64 accepted = 1;
  uint64 rejected = 2;
}
```

### GetAggregateHealthRequest

GetAggregateHealthRequest is an empty platform health request.

*`acme/example/v2/services.proto`*

```protobuf
message GetAggregateHealthRequest {
}
```

### GetAggregateHealthResponse

GetAggregateHealthResponse returns rolled-up health from types.proto.

*`acme/example/v2/services.proto`*

```protobuf
message GetAggregateHealthResponse {
  AggregateHealth health = 1;
}
```

