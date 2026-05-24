# NumericRange

NumericRange supports validation comment examples.

*`acme/example/v2/types.proto`*

```protobuf
message NumericRange {
  
  int64 min_inclusive = 1;
  int64 max_inclusive = 2;
}
```

**Protovalidate (CEL)**

```cel
id: "numeric_range.min_lte_max"
      message: "min_inclusive must not exceed max_inclusive"
      expression: "this.min_inclusive <= this.max_inclusive"
```

