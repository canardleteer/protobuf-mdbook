# acme.example.v3alpha1

## Services

### FeatureFlagService

*`acme/example/v3alpha1/services.proto`*

FeatureFlagService documents flag and override RPCs.

**ListFeatureFlags** ( [ListFeatureFlagsRequest](#listfeatureflagsrequest) ) returns ( [ListFeatureFlagsResponse](#listfeatureflagsresponse) )

ListFeatureFlags returns a paginated flag catalog page.

**UpsertFlagOverride** ( [UpsertFlagOverrideRequest](#upsertflagoverriderequest) ) returns ( [UpsertFlagOverrideResponse](#upsertflagoverrideresponse) )

UpsertFlagOverride stores a tenant override row.

**PublishHints** ( stream [PublishHintsRequest](#publishhintsrequest) ) returns ( [PublishHintsResponse](#publishhintsresponse) )

PublishHints accepts a client stream of platform hints.

**SyncFlags** ( stream [SyncFlagsRequest](#syncflagsrequest) ) returns ( stream [SyncFlagsResponse](#syncflagsresponse) )

SyncFlags synchronizes flag revisions over a bidirectional stream.

### ExperimentService

*`acme/example/v3alpha1/services.proto`*

ExperimentService documents assignment and streaming experiment RPCs.

**AssignExperiment** ( [AssignExperimentRequest](#assignexperimentrequest) ) returns ( [AssignExperimentResponse](#assignexperimentresponse) )

AssignExperiment picks an arm for a subject.

**StreamAssignments** ( [StreamAssignmentsRequest](#streamassignmentsrequest) ) returns ( stream [StreamAssignmentsResponse](#streamassignmentsresponse) )

StreamAssignments pushes assignment events for an experiment.

### PipelineService

*`acme/example/v3alpha1/services.proto`*

PipelineService documents orchestration RPCs in the alpha package.

**StartPipeline** ( [StartPipelineRequest](#startpipelinerequest) ) returns ( [StartPipelineResponse](#startpipelineresponse) )

StartPipeline creates a pipeline run from staged inputs.

**WatchPipeline** ( [WatchPipelineRequest](#watchpipelinerequest) ) returns ( stream [WatchPipelineResponse](#watchpipelineresponse) )

WatchPipeline streams step results for a run.

**CancelPipeline** ( [CancelPipelineRequest](#cancelpipelinerequest) ) returns ( [CancelPipelineResponse](#cancelpipelineresponse) )

CancelPipeline stops a run by identifier.

## Messages and enums

### FeatureFlag

FeatureFlag describes a toggle evaluated by fictional clients.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message FeatureFlag {
// Stable identifier (slug).
  string key = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 128,
      (buf.validate.field).string.pattern = "^[a-z][a-z0-9_.-]*$"
    ];
  string display_name = 2 [(buf.validate.field).string.max_len = 256];
  string description = 3;
  bool default_enabled = 4;
  ReleaseChannel channel = 5;
  acme.example.v2.SharedMetadata metadata = 6;
  google.protobuf.Timestamp updated_at = 7;
}
```

### FlagOverride

FlagOverride pins a flag value for one tenant.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message FlagOverride {
  acme.example.v2.TenantRef tenant = 1;
  string flag_key = 2 [(buf.validate.field).string.min_len = 1];
  bool enabled = 3;
  google.protobuf.Timestamp expires_at = 4;
}
```

### ExperimentArm

ExperimentArm is one variant in an A/B style experiment.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message ExperimentArm {
  string arm_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.pattern = "^arm_[a-z0-9]+$"
    ];
  string label = 2 [(buf.validate.field).string.max_len = 64];
  uint32 weight_basis_points = 3 [
      (buf.validate.field).uint32.gte = 0,
      (buf.validate.field).uint32.lte = 10000
    ];
  map<string, string> parameters = 4;
}
```

### ExperimentSpec

ExperimentSpec defines arms and eligibility for assignment RPCs.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message ExperimentSpec {
  
  string experiment_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.uuid = true
    ];
  string hypothesis = 2;
  repeated ExperimentArm arms = 3 [(buf.validate.field).repeated.min_items = 2];
  acme.example.v2.TimeWindow enrollment_window = 4;
  acme.example.v2.LabelSet audience = 5;
}
```

**Protovalidate (CEL)**

```cel
id: "experiment.arms_weights_positive"
      message: "each arm must have positive weight in basis points"
      expression: "this.arms.all(arm, arm.weight_basis_points > 0u)"
```

### ExperimentAssignment

ExperimentAssignment records which arm a subject received.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message ExperimentAssignment {
  string experiment_id = 1;
  string subject_id = 2 [(buf.validate.field).string.min_len = 1];
  string arm_id = 3;
  google.protobuf.Timestamp assigned_at = 4;
  acme.example.v2.TraceContext trace = 5;
}
```

### SubjectRef

SubjectRef identifies a user or device for experiment assignment.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message SubjectRef {
  oneof identity {
      string user_id = 1 [(buf.validate.field).string.uuid = true];
      string device_id = 2 [(buf.validate.field).string.min_len = 8];
    }
  acme.example.v2.TenantRef tenant = 3;
}
```

### RolloutStage

RolloutStage documents progressive delivery for pipeline stories.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message RolloutStage {
  ReleaseChannel channel = 1;
  google.protobuf.Duration bake_time = 2;
  uint32 max_parallel = 3 [(buf.validate.field).uint32.lte = 1000];
}
```

### PlatformHint

PlatformHint links alpha metadata back into v2 envelopes.

*`acme/example/v3alpha1/types.proto`*

```protobuf
message PlatformHint {
  string hint_id = 1;
  acme.example.v2.PayloadEnvelope envelope = 2;
  repeated acme.example.v2.Label labels = 3;
}
```

### ReleaseChannel

ReleaseChannel classifies how aggressively a flag rolls out.

*`acme/example/v3alpha1/types.proto`*

```protobuf
enum ReleaseChannel {
  RELEASE_CHANNEL_UNSPECIFIED = 0;
  RELEASE_CHANNEL_INTERNAL = 1;
  RELEASE_CHANNEL_CANARY = 2;
  RELEASE_CHANNEL_STABLE = 3;
}
```

### PipelineStepInput

PipelineStepInput is one unit of work inside a pipeline run.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineStepInput {
  string step_name = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.min_len = 1,
      (buf.validate.field).string.max_len = 64
    ];
  acme.example.v2.PayloadEnvelope input = 2;
  RolloutStage rollout = 3;
}
```

### PipelineStepResult

PipelineStepResult reports completion for a single step.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineStepResult {
  string step_name = 1;
  PipelineStatus status = 2;
  acme.example.v2.ErrorDetail error = 3;
  acme.example.v2.StreamCursor cursor = 4;
  google.protobuf.Timestamp finished_at = 5;
}
```

### PipelineStage

PipelineStage is a oneof-heavy stage definition for doc rendering.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineStage {
  oneof stage {
      PipelineStepInput step = 1;
      RolloutStage rollout_only = 2;
      acme.example.v2.AuditBatch audit_snapshot = 3;
    }
}
```

### PipelineRun

PipelineRun identifies a long-lived orchestration instance.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message PipelineRun {
  
  string run_id = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.uuid = true
    ];
  string pipeline_name = 2 [(buf.validate.field).string.min_len = 1];
  PipelineStatus status = 3;
  repeated PipelineStepResult results = 4;
  acme.example.v2.ResourceIdentity owner = 5;
  google.protobuf.Timestamp started_at = 6;
  google.protobuf.Timestamp completed_at = 7;
}
```

**Protovalidate (CEL)**

```cel
id: "pipeline_run.completed_after_started"
      message: "completed_at must not precede started_at when both are set"
      expression: "!has(this.completed_at) || !has(this.started_at) || this.completed_at >= this.started_at"
```

### StartPipelineRequest

StartPipelineRequest kicks off a pipeline from staged inputs.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message StartPipelineRequest {
  string pipeline_name = 1 [
      (buf.validate.field).required = true,
      (buf.validate.field).string.pattern = "^[a-z][a-z0-9-]*$"
    ];
  repeated PipelineStage stages = 2 [(buf.validate.field).repeated.min_items = 1];
  acme.example.v2.ResourceIdentity actor = 3;
}
```

### StartPipelineResponse

StartPipelineResponse returns the created run handle.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message StartPipelineResponse {
  PipelineRun run = 1;
}
```

### WatchPipelineRequest

WatchPipelineRequest subscribes to pipeline step events.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message WatchPipelineRequest {
  string run_id = 1 [(buf.validate.field).string.uuid = true];
  acme.example.v2.TimeWindow window = 2;
  repeated acme.example.v2.FilterExpression filters = 3;
}
```

### WatchPipelineResponse

WatchPipelineResponse is one event on the WatchPipeline server stream.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message WatchPipelineResponse {
  string event_id = 1;
  PipelineStepResult step = 2;
  google.protobuf.Timestamp observed_at = 3;
  acme.example.v2.StreamCursor cursor = 4;
}
```

### CancelPipelineRequest

CancelPipelineRequest stops a run by id.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message CancelPipelineRequest {
  string run_id = 1 [(buf.validate.field).string.uuid = true];
  string reason = 2 [(buf.validate.field).string.max_len = 512];
}
```

### CancelPipelineResponse

CancelPipelineResponse acknowledges cancellation.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
message CancelPipelineResponse {
  PipelineRun run = 1;
}
```

### PipelineStatus

PipelineStatus tracks synthetic workflow execution.

*`acme/example/v3alpha1/pipeline.proto`*

```protobuf
enum PipelineStatus {
  PIPELINE_STATUS_UNSPECIFIED = 0;
  PIPELINE_STATUS_QUEUED = 1;
  PIPELINE_STATUS_RUNNING = 2;
  PIPELINE_STATUS_SUCCEEDED = 3;
  PIPELINE_STATUS_FAILED = 4;
  PIPELINE_STATUS_CANCELLED = 5;
}
```

### ListFeatureFlagsRequest

ListFeatureFlagsRequest paginates feature flags for a tenant.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message ListFeatureFlagsRequest {
  acme.example.v2.ListOptions options = 1;
  acme.example.v2.TenantRef tenant = 2;
  ReleaseChannel channel_filter = 3;
}
```

### ListFeatureFlagsResponse

ListFeatureFlagsResponse returns a page of flags.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message ListFeatureFlagsResponse {
  repeated FeatureFlag flags = 1;
  acme.example.v2.PageResult page = 2;
}
```

### UpsertFlagOverrideRequest

UpsertFlagOverrideRequest writes a tenant-specific override.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message UpsertFlagOverrideRequest {
  FlagOverride override = 1;
}
```

### UpsertFlagOverrideResponse

UpsertFlagOverrideResponse echoes the stored override.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message UpsertFlagOverrideResponse {
  FlagOverride override = 1;
}
```

### AssignExperimentRequest

AssignExperimentRequest assigns a subject to an experiment arm.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message AssignExperimentRequest {
  ExperimentSpec spec = 1;
  SubjectRef subject = 2;
  repeated string exclusion_keys = 3 [(buf.validate.field).repeated.max_items = 32];
}
```

### AssignExperimentResponse

AssignExperimentResponse returns the assignment row.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message AssignExperimentResponse {
  ExperimentAssignment assignment = 1;
}
```

### StreamAssignmentsRequest

StreamAssignmentsRequest opens a server stream of assignment events.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message StreamAssignmentsRequest {
  string experiment_id = 1 [(buf.validate.field).string.uuid = true];
  acme.example.v2.TimeWindow window = 2;
}
```

### StreamAssignmentsResponse

StreamAssignmentsResponse is one assignment event on the stream.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message StreamAssignmentsResponse {
  ExperimentAssignment assignment = 1;
  acme.example.v2.StreamCursor cursor = 2;
  google.protobuf.Timestamp observed_at = 3;
}
```

### PublishHintsRequest

PublishHintsRequest is one client-streaming platform hint chunk.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message PublishHintsRequest {
  string batch_id = 1;
  PlatformHint hint = 2;
  uint32 part_index = 3;
  bool last_part = 4;
}
```

### PublishHintsResponse

PublishHintsResponse aggregates uploaded hint parts.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message PublishHintsResponse {
  string batch_id = 1;
  uint32 parts_received = 2;
  acme.example.v2.PageResult page = 3;
}
```

### SyncFlagsRequest

SyncFlagsRequest is one frame in a bidirectional flag sync session.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message SyncFlagsRequest {
  uint64 sequence = 1;
  FeatureFlag flag = 2;
  bool fin = 3;
}
```

### SyncFlagsResponse

SyncFlagsResponse mirrors a bidirectional flag sync frame.

*`acme/example/v3alpha1/services.proto`*

```protobuf
message SyncFlagsResponse {
  uint64 sequence = 1;
  acme.example.v2.StreamCursor cursor = 2;
  acme.example.v2.ErrorDetail error = 3;
  bool fin = 4;
}
```

