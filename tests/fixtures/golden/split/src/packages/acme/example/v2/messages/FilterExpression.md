# FilterExpression

FilterExpression is a intentionally verbose filter AST placeholder.

*`acme/example/v2/types.proto`*

```protobuf
message FilterExpression {
  string field = 1;
  string op = 2;
  string value = 3;
  repeated FilterExpression children = 4;
}
```

