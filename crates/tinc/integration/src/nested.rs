use serde::de::DeserializeSeed;
use tinc::__private::de::{
    DeserializeHelper, DeserializerWrapper, TrackedStructDeserializer, TrackerFor, TrackerSharedState, TrackerStateGuard,
};

#[test]
fn test_nested() {
    mod pb {
        tonic::include_proto!("nested");
    }

    let mut message = pb::NestedMessage::default();
    let mut tracker = <pb::NestedMessage as TrackerFor>::Tracker::default();
    let guard = TrackerStateGuard::new(TrackerSharedState::default());

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "someOther": {
            "name": "test",
            "id": 1,
            "nested": {
                "name": "nested",
                "id": 2,
                "age": 3,
                "nestedEnum": "SOME_VALUE",
                "nested": {
                    "depth": 100
                }
            }
        },
        "nestedEnum": "YET_ANOTHER_VALUE"
    }"#,
    );

    DeserializeHelper {
        tracker: &mut tracker,
        value: &mut message,
    }
    .deserialize(DeserializerWrapper::new(&mut de))
    .unwrap();

    TrackedStructDeserializer::verify_deserialize::<serde::de::value::Error>(&message, &mut tracker).unwrap();

    let state = guard.finish();
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: true,
        irrecoverable: false,
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
    MessageTracker(
        NestedMessageTracker {
            some_other: Some(
                OptionalTracker(
                    Some(
                        MessageTracker(
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
                                            MessageTracker(
                                                NestedMessageTracker {
                                                    nested_enum: Some(
                                                        EnumTracker<tinc_integration_tests::nested::test_nested::pb::some_other_message::nested_message::NestedEnum>,
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
                                                                MessageTracker(
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
                EnumTracker<tinc_integration_tests::nested::test_nested::pb::some_other_message::nested_message::NestedEnum>,
            ),
        },
    )
    ");
}
