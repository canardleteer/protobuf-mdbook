# ExperimentService

*`acme/example/v3alpha1/services.proto`*

ExperimentService documents assignment and streaming experiment RPCs.

**AssignExperiment** ( [AssignExperimentRequest](../messages/AssignExperimentRequest.md) ) returns ( [AssignExperimentResponse](../messages/AssignExperimentResponse.md) )

AssignExperiment picks an arm for a subject.

**StreamAssignments** ( [StreamAssignmentsRequest](../messages/StreamAssignmentsRequest.md) ) returns ( stream [StreamAssignmentsResponse](../messages/StreamAssignmentsResponse.md) )

StreamAssignments pushes assignment events for an experiment.

