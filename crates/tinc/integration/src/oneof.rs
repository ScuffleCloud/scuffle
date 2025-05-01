use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("oneof");
}

#[test]
fn test_oneof() {
    let mut message = pb::OneofMessage::default();
    let mut tracker = <pb::OneofMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "string_or_int32": {
            "string": "test"
        },
        "string_or_int32_tagged": {
            "tag": "int322",
            "value": 1
        },
        "tagged_nested": {
            "tag": "nested_message",
            "value": {
                "string": "nested",
                "int32": 50
            }
        },
        "nested": {
            "custom_enum2": "VALUE"
        },
        "magic_nested": {
            "string": "magic",
            "int32": 1
        },
        "flattened_tag": "magic_enum3",
        "flattened_value": "VALUE"
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();

    state.in_scope(|| {
        TrackedStructDeserializer::validate(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    OneofMessage {
        string_or_int32: Some(
            String(
                "test",
            ),
        ),
        string_or_int32_tagged: Some(
            Int322(
                1,
            ),
        ),
        tagged_nested: Some(
            NestedMessage(
                NestedMessage {
                    string: "nested",
                    int32: 50,
                },
            ),
        ),
        nested: Some(
            CustomEnum2(
                Value,
            ),
        ),
        flattened: Some(
            MagicNested(
                NestedMessage {
                    string: "magic",
                    int32: 1,
                },
            ),
        ),
        flattened_tagged: Some(
            MagicEnum3(
                Value,
            ),
        ),
    }
    "#);

    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        OneofMessageTracker {
            string_or_int32: Some(
                OneOfTracker(
                    Some(
                        String(
                            PrimitiveTracker<alloc::string::String>,
                        ),
                    ),
                ),
            ),
            string_or_int32_tagged: Some(
                TaggedOneOfTracker {
                    tracker: Some(
                        Int322(
                            PrimitiveTracker<i32>,
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "int322",
                    ),
                    value_buffer: [],
                },
            ),
            tagged_nested: Some(
                TaggedOneOfTracker {
                    tracker: Some(
                        NestedMessage(
                            StructTracker(
                                NestedMessageTracker {
                                    string: Some(
                                        PrimitiveTracker<alloc::string::String>,
                                    ),
                                    int32: Some(
                                        PrimitiveTracker<i32>,
                                    ),
                                },
                            ),
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "nested_message",
                    ),
                    value_buffer: [],
                },
            ),
            nested: Some(
                OneOfTracker(
                    Some(
                        CustomEnum2(
                            EnumTracker<tinc_integration_tests::oneof::pb::CustomEnum>,
                        ),
                    ),
                ),
            ),
            flattened: Some(
                OneOfTracker(
                    Some(
                        MagicNested(
                            StructTracker(
                                NestedMessageTracker {
                                    string: Some(
                                        PrimitiveTracker<alloc::string::String>,
                                    ),
                                    int32: Some(
                                        PrimitiveTracker<i32>,
                                    ),
                                },
                            ),
                        ),
                    ),
                ),
            ),
            flattened_tagged: Some(
                TaggedOneOfTracker {
                    tracker: Some(
                        MagicEnum3(
                            EnumTracker<tinc_integration_tests::oneof::pb::CustomEnum>,
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "magic_enum3",
                    ),
                    value_buffer: [],
                },
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "string_or_int32": {
        "string": "test"
      },
      "string_or_int32_tagged": {
        "tag": "int322",
        "value": 1
      },
      "tagged_nested": {
        "tag": "nested_message",
        "value": {
          "string": "nested",
          "int32": 50
        }
      },
      "nested": {
        "custom_enum2": "VALUE"
      },
      "magic_nested": {
        "string": "magic",
        "int32": 1
      },
      "flattened_tag": "magic_enum3",
      "flattened_value": "VALUE"
    }
    "#);
}

#[test]
fn test_oneof_buffering() {
    let mut message = pb::OneofMessage::default();
    let mut tracker = <pb::OneofMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "string_or_int32_tagged": {
            "value": 1
        },
        "tagged_nested": {
            "value": {
                "string": "nested"
            }
        },
        "flattened_value": "VALUE"
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "string_or_int32": {
            "string": "test"
        },
        "string_or_int32_tagged": {
            "tag": "int322"
        },
        "tagged_nested": {
            "tag": "nested_message",
            "value": {
                "int32": 100
            }
        },
        "nested": {
            "custom_enum2": "VALUE"
        },
        "magic_nested": {
            "string": "magic",
            "int32": 1
        },
        "flattened_tag": "magic_enum3"
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TrackedStructDeserializer::validate(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    OneofMessage {
        string_or_int32: Some(
            String(
                "test",
            ),
        ),
        string_or_int32_tagged: Some(
            Int322(
                1,
            ),
        ),
        tagged_nested: Some(
            NestedMessage(
                NestedMessage {
                    string: "nested",
                    int32: 100,
                },
            ),
        ),
        nested: Some(
            CustomEnum2(
                Value,
            ),
        ),
        flattened: Some(
            MagicNested(
                NestedMessage {
                    string: "magic",
                    int32: 1,
                },
            ),
        ),
        flattened_tagged: Some(
            MagicEnum3(
                Value,
            ),
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        OneofMessageTracker {
            string_or_int32: Some(
                OneOfTracker(
                    Some(
                        String(
                            PrimitiveTracker<alloc::string::String>,
                        ),
                    ),
                ),
            ),
            string_or_int32_tagged: Some(
                TaggedOneOfTracker {
                    tracker: Some(
                        Int322(
                            PrimitiveTracker<i32>,
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "int322",
                    ),
                    value_buffer: [],
                },
            ),
            tagged_nested: Some(
                TaggedOneOfTracker {
                    tracker: Some(
                        NestedMessage(
                            StructTracker(
                                NestedMessageTracker {
                                    string: Some(
                                        PrimitiveTracker<alloc::string::String>,
                                    ),
                                    int32: Some(
                                        PrimitiveTracker<i32>,
                                    ),
                                },
                            ),
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "nested_message",
                    ),
                    value_buffer: [],
                },
            ),
            nested: Some(
                OneOfTracker(
                    Some(
                        CustomEnum2(
                            EnumTracker<tinc_integration_tests::oneof::pb::CustomEnum>,
                        ),
                    ),
                ),
            ),
            flattened: Some(
                OneOfTracker(
                    Some(
                        MagicNested(
                            StructTracker(
                                NestedMessageTracker {
                                    string: Some(
                                        PrimitiveTracker<alloc::string::String>,
                                    ),
                                    int32: Some(
                                        PrimitiveTracker<i32>,
                                    ),
                                },
                            ),
                        ),
                    ),
                ),
            ),
            flattened_tagged: Some(
                TaggedOneOfTracker {
                    tracker: Some(
                        MagicEnum3(
                            EnumTracker<tinc_integration_tests::oneof::pb::CustomEnum>,
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "magic_enum3",
                    ),
                    value_buffer: [],
                },
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "string_or_int32": {
        "string": "test"
      },
      "string_or_int32_tagged": {
        "tag": "int322",
        "value": 1
      },
      "tagged_nested": {
        "tag": "nested_message",
        "value": {
          "string": "nested",
          "int32": 100
        }
      },
      "nested": {
        "custom_enum2": "VALUE"
      },
      "magic_nested": {
        "string": "magic",
        "int32": 1
      },
      "flattened_tag": "magic_enum3",
      "flattened_value": "VALUE"
    }
    "#);
}
