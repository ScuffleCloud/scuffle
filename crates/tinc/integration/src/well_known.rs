use tinc::__private::{TincValidate, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("well_known");
}

#[test]
fn test_well_known() {
    let mut message = pb::WellKnownMessage::default();
    let mut tracker = <pb::WellKnownMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "empty": null,
        "struct": {
            "field1": "value1",
            "field2": "value2"
        },
        "timestamp": "2023-10-01T12:00:00Z",
        "duration": "1.5s",
        "value": {
            "kind": {
                "string_value": "example"
            }
        },
        "list_value": [
            "item1",
            123,
            {
                "kind": {
                    "string_value": "item2"
                }
            }
        ]
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TincValidate::validate(&message, Some(&tracker)).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    WellKnownMessage {
        empty: Some(
            (),
        ),
        r#struct: Some(
            Struct {
                fields: {
                    "field1": Value {
                        kind: Some(
                            StringValue(
                                "value1",
                            ),
                        ),
                    },
                    "field2": Value {
                        kind: Some(
                            StringValue(
                                "value2",
                            ),
                        ),
                    },
                },
            },
        ),
        timestamp: Some(
            Timestamp {
                seconds: 1696161600,
                nanos: 0,
            },
        ),
        duration: Some(
            Duration {
                seconds: 1,
                nanos: 500000000,
            },
        ),
        value: Some(
            Value {
                kind: Some(
                    StructValue(
                        Struct {
                            fields: {
                                "kind": Value {
                                    kind: Some(
                                        StructValue(
                                            Struct {
                                                fields: {
                                                    "string_value": Value {
                                                        kind: Some(
                                                            StringValue(
                                                                "example",
                                                            ),
                                                        ),
                                                    },
                                                },
                                            },
                                        ),
                                    ),
                                },
                            },
                        },
                    ),
                ),
            },
        ),
        list_value: Some(
            ListValue {
                values: [
                    Value {
                        kind: Some(
                            StringValue(
                                "item1",
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            NumberValue(
                                123.0,
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            StructValue(
                                Struct {
                                    fields: {
                                        "kind": Value {
                                            kind: Some(
                                                StructValue(
                                                    Struct {
                                                        fields: {
                                                            "string_value": Value {
                                                                kind: Some(
                                                                    StringValue(
                                                                        "item2",
                                                                    ),
                                                                ),
                                                            },
                                                        },
                                                    },
                                                ),
                                            ),
                                        },
                                    },
                                },
                            ),
                        ),
                    },
                ],
            },
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        WellKnownMessageTracker {
            empty: Some(
                OptionalTracker(
                    Some(
                        WellKnownTracker<()>,
                    ),
                ),
            ),
            struct: Some(
                OptionalTracker(
                    Some(
                        WellKnownTracker<prost_types::protobuf::Struct>,
                    ),
                ),
            ),
            timestamp: Some(
                OptionalTracker(
                    Some(
                        WellKnownTracker<prost_types::protobuf::Timestamp>,
                    ),
                ),
            ),
            duration: Some(
                OptionalTracker(
                    Some(
                        WellKnownTracker<prost_types::protobuf::Duration>,
                    ),
                ),
            ),
            value: Some(
                OptionalTracker(
                    Some(
                        WellKnownTracker<prost_types::protobuf::Value>,
                    ),
                ),
            ),
            list_value: Some(
                OptionalTracker(
                    Some(
                        WellKnownTracker<prost_types::protobuf::ListValue>,
                    ),
                ),
            ),
        },
    )
    ");

    insta::assert_json_snapshot!(message, @r#"
    {
      "empty": {},
      "struct": {
        "field1": "value1",
        "field2": "value2"
      },
      "timestamp": "2023-10-01T12:00:00+00:00",
      "duration": "1.5s",
      "value": {
        "kind": {
          "string_value": "example"
        }
      },
      "list_value": [
        "item1",
        123.0,
        {
          "kind": {
            "string_value": "item2"
          }
        }
      ]
    }
    "#);
}

#[test]
fn test_well_known_map() {
    let mut message = pb::WellKnownMapMessage::default();
    let mut tracker = <pb::WellKnownMapMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "empty": {
            "null": null,
            "empty_map": {},
            "empty_array": [],
            "empty_string": "",
            "non_empty_string": "non_empty",
            "non_empty_array": [1, 2, 3],
            "non_empty_map": {
                "key1": "value1",
                "key2": "value2"
            }
        },
        "struct": {
            "first": {
                "field1": "value1"
            },
            "second": {
                "field2": "value2"
            }
        },
        "timestamp": {
            "first": "2023-10-01T12:00:00Z",
            "second": "2023-10-02T12:00:00Z"
        },
        "duration": {
            "first": "1.5s",
            "second": "2.0s",
            "third": "0.123456789s"
        },
        "value": {
            "first": "example1",
            "second": 123,
            "third": {
                "kind": {
                    "string_value": "example2"
                }
            },
            "fourth": [],
            "fifth": null
        },
        "list_value": {
            "first": [
                "item1",
                123,
                {
                    "kind": {
                        "string_value": "item2"
                    }
                },
                null,
                ["item3", "item4"],
                "item5"
            ],
            "second": []
        }
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TincValidate::validate(&message, Some(&tracker)).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "expected empty string at line 7 column 43",
                },
                fatal: true,
                path: "empty[\"non_empty_string\"]",
            },
            TrackedError {
                kind: InvalidField {
                    message: "expected empty sequence at line 8 column 40",
                },
                fatal: true,
                path: "empty[\"non_empty_array\"]",
            },
            TrackedError {
                kind: InvalidField {
                    message: "expected empty map at line 12 column 13",
                },
                fatal: true,
                path: "empty[\"non_empty_map\"]",
            },
        ],
    }
    "#);
    insta::assert_debug_snapshot!(message, @r#"
    WellKnownMapMessage {
        empty: {
            "empty_array": (),
            "empty_map": (),
            "empty_string": (),
            "null": (),
        },
        r#struct: {
            "first": Struct {
                fields: {
                    "field1": Value {
                        kind: Some(
                            StringValue(
                                "value1",
                            ),
                        ),
                    },
                },
            },
            "second": Struct {
                fields: {
                    "field2": Value {
                        kind: Some(
                            StringValue(
                                "value2",
                            ),
                        ),
                    },
                },
            },
        },
        timestamp: {
            "first": Timestamp {
                seconds: 1696161600,
                nanos: 0,
            },
            "second": Timestamp {
                seconds: 1696248000,
                nanos: 0,
            },
        },
        duration: {
            "first": Duration {
                seconds: 1,
                nanos: 500000000,
            },
            "second": Duration {
                seconds: 2,
                nanos: 0,
            },
            "third": Duration {
                seconds: 0,
                nanos: 123456789,
            },
        },
        value: {
            "fifth": Value {
                kind: Some(
                    NullValue(
                        NullValue,
                    ),
                ),
            },
            "first": Value {
                kind: Some(
                    StringValue(
                        "example1",
                    ),
                ),
            },
            "fourth": Value {
                kind: Some(
                    ListValue(
                        ListValue {
                            values: [],
                        },
                    ),
                ),
            },
            "second": Value {
                kind: Some(
                    NumberValue(
                        123.0,
                    ),
                ),
            },
            "third": Value {
                kind: Some(
                    StructValue(
                        Struct {
                            fields: {
                                "kind": Value {
                                    kind: Some(
                                        StructValue(
                                            Struct {
                                                fields: {
                                                    "string_value": Value {
                                                        kind: Some(
                                                            StringValue(
                                                                "example2",
                                                            ),
                                                        ),
                                                    },
                                                },
                                            },
                                        ),
                                    ),
                                },
                            },
                        },
                    ),
                ),
            },
        },
        list_value: {
            "first": ListValue {
                values: [
                    Value {
                        kind: Some(
                            StringValue(
                                "item1",
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            NumberValue(
                                123.0,
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            StructValue(
                                Struct {
                                    fields: {
                                        "kind": Value {
                                            kind: Some(
                                                StructValue(
                                                    Struct {
                                                        fields: {
                                                            "string_value": Value {
                                                                kind: Some(
                                                                    StringValue(
                                                                        "item2",
                                                                    ),
                                                                ),
                                                            },
                                                        },
                                                    },
                                                ),
                                            ),
                                        },
                                    },
                                },
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            NullValue(
                                NullValue,
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            ListValue(
                                ListValue {
                                    values: [
                                        Value {
                                            kind: Some(
                                                StringValue(
                                                    "item3",
                                                ),
                                            ),
                                        },
                                        Value {
                                            kind: Some(
                                                StringValue(
                                                    "item4",
                                                ),
                                            ),
                                        },
                                    ],
                                },
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            StringValue(
                                "item5",
                            ),
                        ),
                    },
                ],
            },
            "second": ListValue {
                values: [],
            },
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        WellKnownMapMessageTracker {
            empty: Some(
                {
                    "null": WellKnownTracker<()>,
                    "empty_map": WellKnownTracker<()>,
                    "empty_array": WellKnownTracker<()>,
                    "empty_string": WellKnownTracker<()>,
                    "non_empty_string": WellKnownTracker<()>,
                    "non_empty_array": WellKnownTracker<()>,
                    "non_empty_map": WellKnownTracker<()>,
                },
            ),
            struct: Some(
                {
                    "first": WellKnownTracker<prost_types::protobuf::Struct>,
                    "second": WellKnownTracker<prost_types::protobuf::Struct>,
                },
            ),
            timestamp: Some(
                {
                    "first": WellKnownTracker<prost_types::protobuf::Timestamp>,
                    "second": WellKnownTracker<prost_types::protobuf::Timestamp>,
                },
            ),
            duration: Some(
                {
                    "first": WellKnownTracker<prost_types::protobuf::Duration>,
                    "second": WellKnownTracker<prost_types::protobuf::Duration>,
                    "third": WellKnownTracker<prost_types::protobuf::Duration>,
                },
            ),
            value: Some(
                {
                    "first": WellKnownTracker<prost_types::protobuf::Value>,
                    "second": WellKnownTracker<prost_types::protobuf::Value>,
                    "third": WellKnownTracker<prost_types::protobuf::Value>,
                    "fourth": WellKnownTracker<prost_types::protobuf::Value>,
                    "fifth": WellKnownTracker<prost_types::protobuf::Value>,
                },
            ),
            list_value: Some(
                {
                    "first": WellKnownTracker<prost_types::protobuf::ListValue>,
                    "second": WellKnownTracker<prost_types::protobuf::ListValue>,
                },
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "empty": {
        "empty_array": {},
        "empty_map": {},
        "empty_string": {},
        "null": {}
      },
      "struct": {
        "first": {
          "field1": "value1"
        },
        "second": {
          "field2": "value2"
        }
      },
      "timestamp": {
        "first": "2023-10-01T12:00:00+00:00",
        "second": "2023-10-02T12:00:00+00:00"
      },
      "duration": {
        "first": "1.5s",
        "second": "2s",
        "third": "0.123456789s"
      },
      "value": {
        "fifth": null,
        "first": "example1",
        "fourth": [],
        "second": 123.0,
        "third": {
          "kind": {
            "string_value": "example2"
          }
        }
      },
      "list_value": {
        "first": [
          "item1",
          123.0,
          {
            "kind": {
              "string_value": "item2"
            }
          },
          null,
          [
            "item3",
            "item4"
          ],
          "item5"
        ],
        "second": []
      }
    }
    "#);
}

#[test]
fn test_well_known_repeated() {
    let mut message = pb::WellKnownRepeatedMessage::default();
    let mut tracker = <pb::WellKnownRepeatedMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "empty": [
            null,
            {},
            [],
            ""
        ],
        "struct": [
            {
                "field1": "value1"
            },
            {
                "field2": "value2"
            }
        ],
        "timestamp": [
            "2023-10-01T12:00:00Z",
            "2023-10-02T12:00:00Z"
        ],
        "duration": [
            "1.5s",
            "2.0s",
            "0.123456789s"
        ],
        "value": [
            "example1",
            123,
            {
                "kind": {
                    "string_value": "example2"
                }
            },
            [],
            null
        ],
        "list_value": [
            [
                "item1",
                123,
                {
                    "kind": {
                        "string_value": "item2"
                    }
                },
                null,
                ["item3", "item4"],
                "item5"
            ],
            []
        ]
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TincValidate::validate(&message, Some(&tracker)).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    WellKnownRepeatedMessage {
        empty: [
            (),
            (),
            (),
            (),
        ],
        r#struct: [
            Struct {
                fields: {
                    "field1": Value {
                        kind: Some(
                            StringValue(
                                "value1",
                            ),
                        ),
                    },
                },
            },
            Struct {
                fields: {
                    "field2": Value {
                        kind: Some(
                            StringValue(
                                "value2",
                            ),
                        ),
                    },
                },
            },
        ],
        timestamp: [
            Timestamp {
                seconds: 1696161600,
                nanos: 0,
            },
            Timestamp {
                seconds: 1696248000,
                nanos: 0,
            },
        ],
        duration: [
            Duration {
                seconds: 1,
                nanos: 500000000,
            },
            Duration {
                seconds: 2,
                nanos: 0,
            },
            Duration {
                seconds: 0,
                nanos: 123456789,
            },
        ],
        value: [
            Value {
                kind: Some(
                    StringValue(
                        "example1",
                    ),
                ),
            },
            Value {
                kind: Some(
                    NumberValue(
                        123.0,
                    ),
                ),
            },
            Value {
                kind: Some(
                    StructValue(
                        Struct {
                            fields: {
                                "kind": Value {
                                    kind: Some(
                                        StructValue(
                                            Struct {
                                                fields: {
                                                    "string_value": Value {
                                                        kind: Some(
                                                            StringValue(
                                                                "example2",
                                                            ),
                                                        ),
                                                    },
                                                },
                                            },
                                        ),
                                    ),
                                },
                            },
                        },
                    ),
                ),
            },
            Value {
                kind: Some(
                    ListValue(
                        ListValue {
                            values: [],
                        },
                    ),
                ),
            },
            Value {
                kind: Some(
                    NullValue(
                        NullValue,
                    ),
                ),
            },
        ],
        list_value: [
            ListValue {
                values: [
                    Value {
                        kind: Some(
                            StringValue(
                                "item1",
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            NumberValue(
                                123.0,
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            StructValue(
                                Struct {
                                    fields: {
                                        "kind": Value {
                                            kind: Some(
                                                StructValue(
                                                    Struct {
                                                        fields: {
                                                            "string_value": Value {
                                                                kind: Some(
                                                                    StringValue(
                                                                        "item2",
                                                                    ),
                                                                ),
                                                            },
                                                        },
                                                    },
                                                ),
                                            ),
                                        },
                                    },
                                },
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            NullValue(
                                NullValue,
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            ListValue(
                                ListValue {
                                    values: [
                                        Value {
                                            kind: Some(
                                                StringValue(
                                                    "item3",
                                                ),
                                            ),
                                        },
                                        Value {
                                            kind: Some(
                                                StringValue(
                                                    "item4",
                                                ),
                                            ),
                                        },
                                    ],
                                },
                            ),
                        ),
                    },
                    Value {
                        kind: Some(
                            StringValue(
                                "item5",
                            ),
                        ),
                    },
                ],
            },
            ListValue {
                values: [],
            },
        ],
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        WellKnownRepeatedMessageTracker {
            empty: Some(
                RepeatedVecTracker(
                    [
                        WellKnownTracker<()>,
                        WellKnownTracker<()>,
                        WellKnownTracker<()>,
                        WellKnownTracker<()>,
                    ],
                ),
            ),
            struct: Some(
                RepeatedVecTracker(
                    [
                        WellKnownTracker<prost_types::protobuf::Struct>,
                        WellKnownTracker<prost_types::protobuf::Struct>,
                    ],
                ),
            ),
            timestamp: Some(
                RepeatedVecTracker(
                    [
                        WellKnownTracker<prost_types::protobuf::Timestamp>,
                        WellKnownTracker<prost_types::protobuf::Timestamp>,
                    ],
                ),
            ),
            duration: Some(
                RepeatedVecTracker(
                    [
                        WellKnownTracker<prost_types::protobuf::Duration>,
                        WellKnownTracker<prost_types::protobuf::Duration>,
                        WellKnownTracker<prost_types::protobuf::Duration>,
                    ],
                ),
            ),
            value: Some(
                RepeatedVecTracker(
                    [
                        WellKnownTracker<prost_types::protobuf::Value>,
                        WellKnownTracker<prost_types::protobuf::Value>,
                        WellKnownTracker<prost_types::protobuf::Value>,
                        WellKnownTracker<prost_types::protobuf::Value>,
                        WellKnownTracker<prost_types::protobuf::Value>,
                    ],
                ),
            ),
            list_value: Some(
                RepeatedVecTracker(
                    [
                        WellKnownTracker<prost_types::protobuf::ListValue>,
                        WellKnownTracker<prost_types::protobuf::ListValue>,
                    ],
                ),
            ),
        },
    )
    ");

    insta::assert_json_snapshot!(message, @r#"
    {
      "empty": [
        {},
        {},
        {},
        {}
      ],
      "struct": [
        {
          "field1": "value1"
        },
        {
          "field2": "value2"
        }
      ],
      "timestamp": [
        "2023-10-01T12:00:00+00:00",
        "2023-10-02T12:00:00+00:00"
      ],
      "duration": [
        "1.5s",
        "2s",
        "0.123456789s"
      ],
      "value": [
        "example1",
        123.0,
        {
          "kind": {
            "string_value": "example2"
          }
        },
        [],
        null
      ],
      "list_value": [
        [
          "item1",
          123.0,
          {
            "kind": {
              "string_value": "item2"
            }
          },
          null,
          [
            "item3",
            "item4"
          ],
          "item5"
        ],
        []
      ]
    }
    "#);
}
