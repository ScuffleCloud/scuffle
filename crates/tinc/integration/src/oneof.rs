use serde::de::DeserializeSeed;
use tinc::__private::de::{
    DeserializeHelper, DeserializerWrapper, TrackedStructDeserializer, TrackerFor, TrackerSharedState, TrackerStateGuard,
};

#[test]
fn test_oneof() {
    mod pb {
        tonic::include_proto!("oneof");
    }

    let mut message = pb::OneofMessage::default();
    let mut tracker = <pb::OneofMessage as TrackerFor>::Tracker::default();
    let guard = TrackerStateGuard::new(TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    });

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
            "tag": "nestedMessage",
            "value": {
                "string": "nested"
            }
        },
        "nested": {
            "customEnum2": "VALUE"
        },
        "magicNested": {
            "string": "magic",
            "int32": 1
        },
        "flattened_tag": "magicEnum3",
        "flattened_value": "VALUE"
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
        fail_fast: false,
        irrecoverable: false,
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
                    int32: 0,
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
    MessageTracker(
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
                            MessageTracker(
                                NestedMessageTracker {
                                    string: Some(
                                        PrimitiveTracker<alloc::string::String>,
                                    ),
                                    int32: None,
                                },
                            ),
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "nestedMessage",
                    ),
                    value_buffer: [],
                },
            ),
            nested: Some(
                OneOfTracker(
                    Some(
                        CustomEnum2(
                            EnumTracker<tinc_integration_tests::oneof::test_oneof::pb::CustomEnum>,
                        ),
                    ),
                ),
            ),
            flattened: Some(
                OneOfTracker(
                    Some(
                        MagicNested(
                            MessageTracker(
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
                            EnumTracker<tinc_integration_tests::oneof::test_oneof::pb::CustomEnum>,
                        ),
                    ),
                    state: 2,
                    tag_buffer: Some(
                        "magicEnum3",
                    ),
                    value_buffer: [],
                },
            ),
        },
    )
    "#);
}
