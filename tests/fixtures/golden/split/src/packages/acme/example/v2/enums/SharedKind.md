# SharedKind

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

