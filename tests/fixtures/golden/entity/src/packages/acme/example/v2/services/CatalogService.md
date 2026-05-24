# CatalogService

*`acme/example/v2/services.proto`*

CatalogService documents commerce and inventory RPCs in the v2 package.

**ListProducts** ( [ListProductsRequest](../messages/ListProductsRequest.md) ) returns ( [ListProductsResponse](../messages/ListProductsResponse.md) )

ListProducts returns a paginated product catalog page.

**GetProduct** ( [GetProductRequest](../messages/GetProductRequest.md) ) returns ( [GetProductResponse](../messages/GetProductResponse.md) )

GetProduct fetches a single product by identifier.

**ApplyInventoryAdjustments** ( [ApplyInventoryAdjustmentsRequest](../messages/ApplyInventoryAdjustmentsRequest.md) ) returns ( [ApplyInventoryAdjustmentsResponse](../messages/ApplyInventoryAdjustmentsResponse.md) )

ApplyInventoryAdjustments applies batched stock deltas.

**WatchInventory** ( [WatchInventoryRequest](../messages/WatchInventoryRequest.md) ) returns ( stream [WatchInventoryResponse](../messages/WatchInventoryResponse.md) )

WatchInventory streams inventory adjustments for a warehouse.

**UploadDrafts** ( stream [UploadDraftsRequest](../messages/UploadDraftsRequest.md) ) returns ( [UploadDraftsResponse](../messages/UploadDraftsResponse.md) )

UploadDrafts accepts a client stream of draft SKU payloads.

**SyncCatalog** ( stream [SyncCatalogRequest](../messages/SyncCatalogRequest.md) ) returns ( stream [SyncCatalogResponse](../messages/SyncCatalogResponse.md) )

SyncCatalog synchronizes catalog revisions over a bidirectional stream.

