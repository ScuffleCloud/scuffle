use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("nested");
}

#[test]
fn test_nested() {
    let mut message = pb::NestedMessage::default();
    let mut tracker = <pb::NestedMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "some_other": {
            "name": "test",
            "id": 1,
            "nested": {
                "name": "nested",
                "id": 2,
                "age": 3,
                "nested_enum": "SOME_VALUE",
                "nested": {
                    "depth": 100
                }
            }
        },
        "nested_enum": "YET_ANOTHER_VALUE"
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
    NestedMessage {
        some_other: Some(
            SomeOtherMessage {
                name: "test",
                id: 1,
                nested: Some(
                    NestedMessage {
                        nested_enum: SomeValue,
                        name: "nested",
                        id: 2,
                        age: 3,
                        nested: Some(
                            NestedNestedMessage {
                                depth: 100,
                            },
                        ),
                    },
                ),
            },
        ),
        nested_enum: YetAnotherValue,
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        NestedMessageTracker {
            some_other: Some(
                OptionalTracker(
                    Some(
                        StructTracker(
                            SomeOtherMessageTracker {
                                name: Some(
                                    PrimitiveTracker<alloc::string::String>,
                                ),
                                id: Some(
                                    PrimitiveTracker<i32>,
                                ),
                                nested: Some(
                                    OptionalTracker(
                                        Some(
                                            StructTracker(
                                                NestedMessageTracker {
                                                    nested_enum: Some(
                                                        EnumTracker<tinc_integration_tests::nested::pb::some_other_message::nested_message::NestedEnum>,
                                                    ),
                                                    name: Some(
                                                        PrimitiveTracker<alloc::string::String>,
                                                    ),
                                                    id: Some(
                                                        PrimitiveTracker<i32>,
                                                    ),
                                                    age: Some(
                                                        PrimitiveTracker<i32>,
                                                    ),
                                                    nested: Some(
                                                        OptionalTracker(
                                                            Some(
                                                                StructTracker(
                                                                    NestedNestedMessageTracker {
                                                                        depth: Some(
                                                                            PrimitiveTracker<i32>,
                                                                        ),
                                                                    },
                                                                ),
                                                            ),
                                                        ),
                                                    ),
                                                },
                                            ),
                                        ),
                                    ),
                                ),
                            },
                        ),
                    ),
                ),
            ),
            nested_enum: Some(
                EnumTracker<tinc_integration_tests::nested::pb::some_other_message::nested_message::NestedEnum>,
            ),
        },
    )
    ");

    insta::assert_json_snapshot!(message, @r#"
    {
      "some_other": {
        "name": "test",
        "id": 1,
        "nested": {
          "nested_enum": "SOME_VALUE",
          "name": "nested",
          "id": 2,
          "age": 3,
          "nested": {
            "depth": 100
          }
        }
      },
      "nested_enum": "YET_ANOTHER_VALUE"
    }
    "#);
}
