# acme.example.tagdoc

## Messages and enums

### TagDocExample

Annotation from `<environment>` CoT detail element.
Maps to ATAK's `<zMist>` inside `<zMistsMap>`.

*`escape_tags_comments.proto`*

```protobuf
message TagDocExample {
// Field note from <sensor> (stays inside fence — unescaped).
  string name = 1;
}
```

