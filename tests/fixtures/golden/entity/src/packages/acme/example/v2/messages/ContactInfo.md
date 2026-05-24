# ContactInfo

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

