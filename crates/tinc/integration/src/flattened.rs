use tinc::__private::de::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize};

#[test]
fn test_nested() {
    mod pb {
        tonic::include_proto!("flattened");
    }

    let mut message = pb::FlattenedMessage::default();
    let mut tracker = <pb::FlattenedMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "id": 1,
        "age": 2,
        "depth": 3,
        "house_number": "123",
        "street": "Main St",
        "city": "Anytown",
        "state": "CA",
        "zip_code": "12345"
    }"#,
    );

    state.in_scope(|| {
        deserialize(&mut de, &mut message, &mut tracker).unwrap();
        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: true,
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
    StructTracker(
        FlattenedMessageTracker {
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
                                age: Some(
                                    PrimitiveTracker<i32>,
                                ),
                                nested: Some(
                                    OptionalTracker(
                                        Some(
                                            StructTracker(
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
                                            StructTracker(
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
