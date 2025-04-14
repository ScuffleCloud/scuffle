use serde::de::DeserializeSeed;
use tinc::__private::de::{
    DeserializeHelper, DeserializerWrapper, TrackedStructDeserializer, TrackerFor, TrackerSharedState, TrackerStateGuard,
};

#[test]
fn test_nested() {
    mod pb {
        tonic::include_proto!("flattened");
    }

    let mut message = pb::FlattenedMessage::default();
    let mut tracker = <pb::FlattenedMessage as TrackerFor>::Tracker::default();
    let guard = TrackerStateGuard::new(TrackerSharedState::default());

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "id": 1,
        "age": 2,
        "depth": 3,
        "houseNumber": "123",
        "street": "Main St",
        "city": "Anytown",
        "state": "CA",
        "zipCode": "12345"
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
    FlattenedMessage {
        some_other: Some(
            SomeOtherMessage {
                name: "test",
                id: 1,
                age: 2,
                nested: Some(
                    NestedMessage {
                        depth: 3,
                    },
                ),
                address: Some(
                    SomeOtherMessage2 {
                        house_number: "123",
                        street: "Main St",
                        city: "Anytown",
                        state: "CA",
                        zip_code: "12345",
                    },
                ),
            },
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    MessageTracker(
        FlattenedMessageTracker {
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
                                age: Some(
                                    PrimitiveTracker<i32>,
                                ),
                                nested: Some(
                                    OptionalTracker(
                                        Some(
                                            MessageTracker(
                                                NestedMessageTracker {
                                                    depth: Some(
                                                        PrimitiveTracker<i32>,
                                                    ),
                                                },
                                            ),
                                        ),
                                    ),
                                ),
                                address: Some(
                                    OptionalTracker(
                                        Some(
                                            MessageTracker(
                                                SomeOtherMessage2Tracker {
                                                    house_number: Some(
                                                        PrimitiveTracker<alloc::string::String>,
                                                    ),
                                                    street: Some(
                                                        PrimitiveTracker<alloc::string::String>,
                                                    ),
                                                    city: Some(
                                                        PrimitiveTracker<alloc::string::String>,
                                                    ),
                                                    state: Some(
                                                        PrimitiveTracker<alloc::string::String>,
                                                    ),
                                                    zip_code: Some(
                                                        PrimitiveTracker<alloc::string::String>,
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
    )
    ");
}
