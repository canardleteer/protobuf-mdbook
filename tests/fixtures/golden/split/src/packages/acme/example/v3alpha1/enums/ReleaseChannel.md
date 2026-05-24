# ReleaseChannel

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

