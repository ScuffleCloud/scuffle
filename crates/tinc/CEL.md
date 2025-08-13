# Tinc Cel Expressions

We use [CEL](https://github.com/google/cel-spec) to evaluate expressions used for validating our protobuf input.

The implementation we use is [CEL Rust](https://github.com/clarkmcc/cel-rust), and we have a custom compiler which generates native rust code from CEL expressions.

## Contexts

There are 2 contexts in which CEL expressions are evaluated:
- interpreted (happens at compile time)
- native (happens at runtime)

The interpreted context is used when all inputs to an expression are known at compile time we can evaluate the expression at compile time and just include the result in the generated code. This is useful for things like default values or error messages that are known at compile time.

The native context is used when the inputs to an expression are not known at compile time. This happens when you are working with the actual `input` value of a protobuf message or field.

We try to evaluate as much as possible at compile time so we will recursively evaluate the expression until we reach a point where we can no longer evaluate it at compile time.

During runtime; we do not run a CEL interpreter. Instead after we have evaluated as much as we can we convert the expression into native rust code and include that into the code we generate for the validation.

The following table has a list of all functions that are available in the CEL expressions the data they operate on arguments and the context in which they are available.

| Function Name | Self | Arguments | Return Type | Context | Description |
|---------------|------|-----------|-------------|---------|-------------|
| `contains` | `string` | `string` | `bool` | interpreted, native | Returns true if the string contains the substring. |
| `contains` | `bytes` | `bytes` | `bool` | interpreted, native | Returns true if the bytes contains the sub-sequence of bytes. |
| `contains` | `repeated T` | `T` | `bool` | interpreted, native | Returns true if the repeated field contains the value. |
| `contains` | `map<K, V>` | `K` | `bool` | interpreted, native | Returns true if the map contains the key. |
| `size` | `string` | None | `uint` | interpreted, native | Returns the length of the string in bytes. **Note: this is not the unicode length.** |
| `size` | `bytes` | None | `uint` | interpreted, native | Returns the length of the bytes. |
| `size` | `repeated T` | None | `uint` | interpreted, native | Returns the number of elements in the repeated field. |
| `size` | `map<K, V>` | None | `uint` | interpreted, native | Returns the number of elements in the map. |
| `has` | None | `any?` | `bool` | interpreted | Returns true if the the value provided exists. |
| `map` | `repeated T` | `<ident>`, `<expr -> U>` | `repeated U` | interpreted, native | Generator expression to transform the repeated field. |
| `map` | `map<K, V>` | `<ident>`, `<expr -> U>` | `repeated U` | interpreted, native | Generator expression to transform the map field. The input ident will be a tuple (list in the form of `[key, value]`) |
| `filter` | `repeated T` | `<ident>`, `<expr -> bool>` | `repeated T` | interpreted, native | Generator expression to filter the repeated field to only include elements where the expression returns `true`. |
| `filter` | `map<K, V>` | `<ident>`, `<expr -> U>` | `repeated U` | interpreted, native | Generator expression to filter map field to only include elements where the expression returns `true`. The input ident will be a tuple (list in the form of `[key, value]`) |
| `all` | `repeated T` | `<ident>`, `<expr -> bool>` | `bool` | interpreted, native | Returns `true` if the expression returns `true` for all `<ident>`. |
| `all` | `map<K, V>` | `<ident>`, `<expr -> bool>` | `bool` | interpreted, native | Returns `true` if the expression returns `true` for all `<ident>`. The input ident will be a tuple (list in the form of `[key, value]`) |
| `exists` | `repeated T` | `<ident>`, `<expr -> bool>` | `bool` | interpreted, native | Returns true if any element in the repeated field matches cause the expression to return `true`. |
| `exists` | `map<K, V>` | `<ident>`, `<expr -> bool>` | `bool` | interpreted, native | Returns true if any element in the map field cause the expression to return `true`. The input ident will be a tuple (list in the form of `[key, value]`) |
| `existsOne` | `repeated T` | `<ident>`, `<expr -> bool>` | `bool` | interpreted, native | Returns true if exactly one element in the repeated field causes the expression to return `true`. |
| `existsOne` | `map<K, V>` | `<ident>`, `<expr -> bool>` | `bool` | interpreted, native | Returns true if exactly one element in the map field causes the expression to return `true`. The input ident will be a tuple (list in the form of `[key, value]`) |
| `startsWith` | `string` | `string` | `bool` | interpreted, native | Returns true if the string starts with the substring. |
| `endsWith` | `string` | `string` | `bool` | interpreted, native | Returns true if the string ends with the substring. |
| `startsWith` | `bytes` | `bytes` or `string` | `bool` | interpreted, native | Returns true if the bytes starts with the sub sequence. |
| `endsWith` | `bytes` | `bytes` or `string` | `bool` | interpreted, native | Returns true if the bytes ends with the sub sequence. |
| `matches` | `string` | `string` | `bool` | interpreted, native | Returns true if the string matches the regex. |
| `matches` | `bytes` | `string` | `bool` | interpreted, native | Returns true if the bytes matches the regex, if the bytes is not valid utf-8, returns false. |
| `string` | `string` | None | `string` | interpreted, native | Converts the value to a string. (noop) |
| `string` | `bytes` | None | `string` | interpreted, native | Converts the value to a string, non-valid utf-8 characters are replaced. |
| `string` | `int` | None | `string` | interpreted, native | Converts the value to a string. |
| `string` | `uint` | None | `string` | interpreted, native | Converts the value to a string. |
| `string` | `bool` | None | `string` | interpreted, native | Converts the value to a string. |
| `string` | `double` | None | `string` | interpreted, native | Converts the value to a string. |
| `string` | `repeated T` | None | `string` | interpreted, native | Converts the value to a string, where T is any value that can be converted to a string. |
| `string` | `map<K, V>` | None | `string` | interpreted, native | Converts the value to a string, where K and V are any values that can be converted to a string. |
| `bytes` | `string` | None | `bytes` | interpreted, native | Converts the value to bytes. (noop) |
| `bytes` | `bytes` | None | `bytes` | interpreted, native | Converts the value to bytes. |
| `uint` | `string` | None | `uint` or `null` | interpreted, native | Tries to convert the value to a `uint`, if the value cannot be parsed as a `unit` returns `null` |
| `uint` | `uint` | None | `uint` | interpreted, native | Returns the value as `unit`. (noop) |
| `uint` | `int` | None | `uint` or `null` | interpreted, native | Tries to convert the value into an `uint`, if the valus is outside of the range of `uint` then returns `null` |
| `uint` | `double` | None | `uint` or `null` | interpreted, native | Tries to convert the value into an `uint`, if the `value.floor()` is outside of the range of `unit` then returns `null` |
| `uint` | `bool` | None | `uint` | interpreted, native | Converts the value to a `uint`, where `true` is 1 and `false` is 0. |
| `int` | `string` | None | `int` or `null` | interpreted, native | Tries to convert the value to an `int`, if the value cannot be parsed as an `int` returns `null` |
| `int` | `uint` | None | `int` or `null` | interpreted, native | Tries to convert the value into an `int`, if the value is outside of the range of `int` then returns `null` |
| `int` | `int` | None | `int` | interpreted, native | Returns the value as `int`. (noop) |
| `int` | `double` | None | `int` or `null` | interpreted, native | Tries to convert the value into an `int`, if the `value.floor()` is outside of the range of `int` then returns `null` |
| `int` | `bool` | None | `int` | interpreted, native | Converts the value to an `int`, where `true` is 1 and `false` is 0. |
| `double` | `string` | None | `double` or `null` | interpreted, native | Tries to convert the value to a `double`, if the value cannot be parsed as a `double` returns `null` |
| `double` | `uint` | None | `double` | interpreted, native | Returns the value as `double`. |
| `double` | `int` | None | `double` | interpreted, native | Returns the value as `double`. |
| `double` | `double` | None | `double` | interpreted, native | Returns the value as `double`. (noop) |
| `double` | `bool` | None | `double` | interpreted, native | Converts the value to a `double`, where `true` is 1.0 and `false` is 0.0. |
| `bool` | `string` | None | `bool` | interpreted, native | Converts the value to a `bool`, where `true` is any non-empty string and `false` is an empty string. |
| `dyn` | None | `<expr -> U>` | `U` | native | Forces the expression to be evaluated at runtime, otherwise an error will be returned. |
| `floor` | `double` | None | `double` | interpreted, native | Returns the largest integer less than or equal to the value. |
| `ceil` | `double` | None | `double` | interpreted, native | Returns the smallest integer greater than or equal to the value. |
| `round` | `double` | None | `double` | interpreted, native | Returns the value rounded to the nearest integer. |
| `abs` | `double` | None | `double` | interpreted, native | Returns the absolute value of the double. |
| `min` | `double` | `double` | `double` | interpreted, native | Returns the minimum of the two doubles. |
| `max` | `double` | `double` | `double` | interpreted, native | Returns the maximum of the two doubles. |
| `floor` | `uint` | None | `uint` | interpreted, native | Returns the largest integer less than or equal to the value. (noop) |
| `ceil` | `uint` | None | `uint` | interpreted, native | Returns the smallest integer greater than or equal to the value. (noop) |
| `round` | `uint` | None | `uint` | interpreted, native | Returns the value rounded to the nearest integer. (noop) |
| `abs` | `uint` | None | `uint` | interpreted, native | Returns the absolute value of the uint. (noop) |
| `min` | `uint` | `uint` | `uint` | interpreted, native | Returns the minimum of the two uint. |
| `max` | `uint` | `uint` | `uint` | interpreted, native | Returns the maximum of the two uints. |
| `floor` | `int` | None | `int` | interpreted, native | Returns the largest integer less than or equal to the value. (noop) |
| `ceil` | `int` | None | `int` | interpreted, native | Returns the smallest integer greater than or equal to the value. (noop) |
| `round` | `int` | None | `int` | interpreted, native | Returns the value rounded to the nearest integer. (noop) |
| `abs` | `int` | None | `int` | interpreted, native | Returns the absolute value of the int. |
| `min` | `int` | `int` | `int` | interpreted, native | Returns the minimum of the two int. |
| `max` | `int` | `int` | `int` | interpreted, native | Returns the maximum of the two ints. |
