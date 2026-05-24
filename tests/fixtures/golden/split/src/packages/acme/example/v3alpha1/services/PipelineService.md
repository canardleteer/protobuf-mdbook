# PipelineService

*`acme/example/v3alpha1/services.proto`*

PipelineService documents orchestration RPCs in the alpha package.

**StartPipeline** ( [StartPipelineRequest](../messages/StartPipelineRequest.md) ) returns ( [StartPipelineResponse](../messages/StartPipelineResponse.md) )

StartPipeline creates a pipeline run from staged inputs.

**WatchPipeline** ( [WatchPipelineRequest](../messages/WatchPipelineRequest.md) ) returns ( stream [WatchPipelineResponse](../messages/WatchPipelineResponse.md) )

WatchPipeline streams step results for a run.

**CancelPipeline** ( [CancelPipelineRequest](../messages/CancelPipelineRequest.md) ) returns ( [CancelPipelineResponse](../messages/CancelPipelineResponse.md) )

CancelPipeline stops a run by identifier.

