# FeatureFlagService

*`acme/example/v3alpha1/services.proto`*

FeatureFlagService documents flag and override RPCs.

**ListFeatureFlags** ( [ListFeatureFlagsRequest](../messages/ListFeatureFlagsRequest.md) ) returns ( [ListFeatureFlagsResponse](../messages/ListFeatureFlagsResponse.md) )

ListFeatureFlags returns a paginated flag catalog page.

**UpsertFlagOverride** ( [UpsertFlagOverrideRequest](../messages/UpsertFlagOverrideRequest.md) ) returns ( [UpsertFlagOverrideResponse](../messages/UpsertFlagOverrideResponse.md) )

UpsertFlagOverride stores a tenant override row.

**PublishHints** ( stream [PublishHintsRequest](../messages/PublishHintsRequest.md) ) returns ( [PublishHintsResponse](../messages/PublishHintsResponse.md) )

PublishHints accepts a client stream of platform hints.

**SyncFlags** ( stream [SyncFlagsRequest](../messages/SyncFlagsRequest.md) ) returns ( stream [SyncFlagsResponse](../messages/SyncFlagsResponse.md) )

SyncFlags synchronizes flag revisions over a bidirectional stream.

