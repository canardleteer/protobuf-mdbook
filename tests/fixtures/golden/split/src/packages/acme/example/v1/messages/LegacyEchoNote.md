# LegacyEchoNote

LegacyEchoNote is an ancillary message for cross-link examples.

*`acme/example/v1/echo.proto`*

```protobuf
message LegacyEchoNote {
  string note_id = 1;
  string body = 2;
  acme.example.v2.SeeAlsoBlock see_also = 3;
}
```

