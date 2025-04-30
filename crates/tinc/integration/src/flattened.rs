use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tonic::include_proto!("flattened");
}

#[test]
fn test_flattened() {
    let mut message = pb::FlattenedMessage::default();
    let mut tracker = <pb::FlattenedMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "id": 1,
        "age": 2,
        "depth": 3,
        "house_number": 123,
        "street": "Main St",
        "city": "Anytown",
        "state": "CA",
        "zip_code": "12345"
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TrackedStructDeserializer::validate(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "invalid type: integer `123`, expected a string at line 6 column 27",
                },
                fatal: true,
                proto_path: "some_other.address.house_number",
                serde_path: "house_number",
            },
        ],
    }
    "#);
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
                        house_number: "",
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

    insta::assert_json_snapshot!(message, @r#"
    {
      "name": "test",
      "id": 1,
      "age": 2,
      "depth": 3,
      "house_number": "",
      "street": "Main St",
      "city": "Anytown",
      "state": "CA",
      "zip_code": "12345"
    }
    "#);
}
