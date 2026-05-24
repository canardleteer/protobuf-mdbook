# Address

Address is a postal-style structure used in nested messages.

*`acme/example/v2/types.proto`*

```protobuf
message Address {
  string line1 = 1;
  string line2 = 2;
  string city = 3;
  string postal_code = 4;
  Location region = 5;
}
```

