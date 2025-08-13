<!-- cargo-sync-rdme title [[ -->
# tinc
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/tinc.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/tinc.svg?logo=docs.rs&style=flat-square)](https://docs.rs/tinc)
[![crates.io](https://img.shields.io/crates/v/tinc.svg?logo=rust&style=flat-square)](https://crates.io/crates/tinc)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
Tinc is a GRPc to REST transcoder which generates axum routes for services defined in proto3 files.

To use this crate check out [tinc-build](https://docs.rs/tinc_build) refer to the [`annotations.proto`](./annotations.proto)

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`prost`** *(enabled by default)* —  Enables prost support
* **`tonic`** *(enabled by default)* —  Enables tonic support
* **`docs`** —  Enables changelog and documentation of feature flags

### Examples

````protobuf
service SimpleService {
    rpc Ping(PingRequest) returns (PingResponse) {
        option (tinc.method).endpoint = {
            post: "/ping"
        };
        option (tinc.method).endpoint = {
            get: "/ping/{arg}"
        };
    }
}

message PingRequest {
    string arg = 1;
}

message PingResponse {
    string result = 1;
}
````

You can also change the serialization / deserialization of the messages in json by annotating stuff like

````protobuf
message FlattenedMessage {
    SomeOtherMessage some_other = 1 [(tinc.field) = {
        flatten: true
    }];
}

message SomeOtherMessage {
    string name = 1 [(tinc.field).rename = "NAME"];
    int32 id = 2 [(tinc.field).visibility = OUTPUT_ONLY];
    int32 age = 3;

    message NestedMessage {
        int32 depth = 1;
    }

    NestedMessage nested = 4 [(tinc.field) = {
        flatten: true
    }];
    SomeOtherMessage2 address = 5 [(tinc.field) = {
        flatten: true
    }];
}

message SomeOtherMessage2 {
    string house_number = 1;
    string street = 2;
    string city = 3;
    string state = 4;
    string zip_code = 5;
}
````

Tinc also has a fully customizable CEL-based expression system which allows you to validate inputs on both GRPc / REST. Similar to <https://github.com/bufbuild/protovalidate>.
Except we compile the CEL-expressions directly into rust syntax and do not ship a interpreter for runtime.

For example you can do something like this

````protobuf
message TestRequest {
    string name = 1 [(tinc.field).constraint.string = {
        min_len: 1,
        max_len: 10,
    }];
    map<string, int32> things = 2 [(tinc.field).constraint.map = {
        key: {
            string: {
                min_len: 1,
                max_len: 10,
            }
        }
        value: {
            int32: {
                gte: 0,
                lte: 100,
            }
        }
    }];
}
````

Then every message that goes into your service handler will be validated and all validation errors will be returned to the user (either via json for http or protobuf for grpc)

````json
{
    "name": "troy",
    "things": {
        "thing1": "1000",
        "thing2": 42000
    }
}
````

returns this:

````json
{
  "code": 3,
  "details": {
    "request": {
      "violations": [
        {
          "description": "invalid type: string \"1000\", expected i32 at line 4 column 24",
          "field": "things[\"thing1\"]"
        },
        {
          "description": "value must be less than or equal to `100`",
          "field": "things[\"thing2\"]"
        }
      ]
    }
  },
  "message": "bad request"
}
````

The cel expressions can be extended to provide custom expressions:

````protobuf
message TestRequest {
    // define a custom expression specifically for this field
    string name = 1 [(tinc.field).constraint.cel = {
        expression: "input == 'troy'"
        message: "must equal `troy` but got `{input}`"
    }];
}

// --- or ---

extend google.protobuf.FieldOptions {
    // define a custom option that can be applied to multiple fields.
    string must_eq = 10200 [(tinc.predefined) = {
        expression: "input == this"
        message: "must equal `{this}` but got `{input}`"
    }];
}

message TestRequest {
    // apply said option to this field.
    string name = 1 [must_eq = "troy"];
}
````

````json
{
  "code": 3,
  "details": {
    "request": {
      "violations": [
        {
          "description": "must equal `troy` but got `notTroy`",
          "field": "name"
        }
      ]
    }
  },
  "message": "bad request"
}
````

### What is supported

* [x] Endpoint path parameters with nested keys
* [x] Mapped response bodies to a specific field
* [x] Binary request/response bodies.
* [x] Query string parsing
* [x] Custom validation expressions, including validation on unary and streaming.
* [x] OpenAPI 3.1 Spec Generation
* [ ] Documentation
* [ ] Tests
* [ ] REST streaming
* [ ] Multipart forms

### Choices made

1. Use a custom proto definition for the proto schema instead of using [google predefined ones](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto).

The reasoning is because we wanted to support additional features that google did not have, we can add a compatibility layer to convert from google to our version if we want in the future. Such as CEL based validation, openapi schema, json flatten / tagged oneofs.

2. Non-proto3-optional fields are required for JSON.

If a field is not marked as `optional` then it is required by default and not providing it will result in an error returned during deserialization. You can opt-out of this behaviour using `[(tinc.field).json_omittable = TRUE]` which will make it so if the value is not provided it will use the default value (same behaviour as protobuf)\`. The rationale behind this is from the way REST apis are typically used. Normally you provide all the fields you want and you do not have default values for rest APIs. So allowing fields to be defaulted may cause some issues related to people not providing required fields but the default value is a valid value for that field and then the endpoint misbehaves.

3. Stop on last error.

Typically when using serde we stop on the first error. We believe that makes errors less valuable since we only ever get the first error that occurred in the stream instead of every error we had. There are some libraries that aim to solve this issue such as [`eserde`](https://lib.rs/crates/eserde) however we opted to build our solution fully custom since their’s have quite a few drawbacks and we (at compile time) know the full structure since its defined in the protobuf schema, allowing us to generate better code for the deserialization process and store errors more effectively without introducing much/any runtime overhead.

### Alternatives to this

#### 1. [GRPc-Gateway](https://grpc-ecosystem.github.io/grpc-gateway/)

GRPc-Gateway is the most popular way of converting from GRPc endpoint to rest endpoints using google’s protoschema for doing so. The reason I dont like grpc-gateway stems from 2 things:

1. grpc gateway requires a reverse proxy or external service which does the transcoding and then forwards you http requests.
1. You do not have any control over how the json is structured. It uses protobuf-json schema encoding.

#### 2. [GRPc-Web](https://github.com/grpc/grpc-web)

GRPc-Web is a browser compatible version of the grpc spec. This is good for maintaining a single api across browsers / servers, but if you still want a rest API for your service it does not help with that.

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
