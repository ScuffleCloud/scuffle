use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

#[test]
fn test_simple_single_pass() {
    mod pb {
        tonic::include_proto!("simple");
    }

    let mut message = pb::SimpleMessage::default();
    let mut tracker = <pb::SimpleMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "values": ["value1", "value2"],
        "key_values": {
            "key1": "value1",
            "key2": "value2"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();

        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "test",
        values: [
            "value1",
            "value2",
        ],
        key_values: {
            "key1": "value1",
            "key2": "value2",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        PrimitiveTracker<alloc::string::String>,
                        PrimitiveTracker<alloc::string::String>,
                    ],
                ),
            ),
            key_values: Some(
                {
                    "key1": PrimitiveTracker<alloc::string::String>,
                    "key2": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);
}

#[test]
fn test_simple_multiple_passes() {
    mod pb {
        tonic::include_proto!("simple");
    }

    let mut message = pb::SimpleMessage::default();
    let mut tracker = <pb::SimpleMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "key_values": {
            "key1": "value1"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "test",
        values: [],
        key_values: {
            "key1": "value1",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: None,
            key_values: Some(
                {
                    "key1": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "values": ["value1", "value2"],
        "key_values": {
            "key2": "value2"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "test",
        values: [
            "value1",
            "value2",
        ],
        key_values: {
            "key1": "value1",
            "key2": "value2",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        PrimitiveTracker<alloc::string::String>,
                        PrimitiveTracker<alloc::string::String>,
                    ],
                ),
            ),
            key_values: Some(
                {
                    "key1": PrimitiveTracker<alloc::string::String>,
                    "key2": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_simple_missing_fields() {
    mod pb {
        tonic::include_proto!("simple");
    }

    let mut message = pb::SimpleMessage::default();
    let mut tracker = <pb::SimpleMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "key_values": {
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "",
        values: [],
        key_values: {},
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        SimpleMessageTracker {
            name: None,
            values: None,
            key_values: Some(
                {},
            ),
        },
    )
    ");

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "values": ["value1", "value2"],
        "key_values": {
            "key2": "value2"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap()
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "",
        values: [
            "value1",
            "value2",
        ],
        key_values: {
            "key2": "value2",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageTracker {
            name: None,
            values: Some(
                RepeatedVecTracker(
                    [
                        PrimitiveTracker<alloc::string::String>,
                        PrimitiveTracker<alloc::string::String>,
                    ],
                ),
            ),
            key_values: Some(
                {
                    "key2": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "name",
            },
        ],
    }
    "#);
}

#[test]
fn test_simple_duplicate_fields() {
    mod pb {
        tonic::include_proto!("simple");
    }

    let mut message = pb::SimpleMessage::default();
    let mut tracker = <pb::SimpleMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "values": ["value1", "value2"],
        "key_values": {
            "key1": "value1",
            "key2": "value2"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "test",
        values: [
            "value1",
            "value2",
        ],
        key_values: {
            "key1": "value1",
            "key2": "value2",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        PrimitiveTracker<alloc::string::String>,
                        PrimitiveTracker<alloc::string::String>,
                    ],
                ),
            ),
            key_values: Some(
                {
                    "key1": PrimitiveTracker<alloc::string::String>,
                    "key2": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "values": ["value1", "value2"],
        "key_values": {
            "key1": "value1",
            "key2": "value2"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "test",
        values: [
            "value1",
            "value2",
        ],
        key_values: {
            "key1": "value1",
            "key2": "value2",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        PrimitiveTracker<alloc::string::String>,
                        PrimitiveTracker<alloc::string::String>,
                    ],
                ),
            ),
            key_values: Some(
                {
                    "key1": PrimitiveTracker<alloc::string::String>,
                    "key2": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: DuplicateField,
                fatal: true,
                path: "values",
            },
            TrackedError {
                kind: DuplicateField,
                fatal: true,
                path: "key_values.key1",
            },
            TrackedError {
                kind: DuplicateField,
                fatal: true,
                path: "key_values.key2",
            },
        ],
    }
    "#);
}

#[test]
fn test_simple_invalid_type() {
    mod pb {
        tonic::include_proto!("simple");
    }

    let mut message = pb::SimpleMessage::default();
    let mut tracker = <pb::SimpleMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": 123,
        "values": [1, 2, {}],
        "key_values": null
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();
        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessage {
        name: "",
        values: [],
        key_values: {},
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        SimpleMessageTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [],
                ),
            ),
            key_values: Some(
                {},
            ),
        },
    )
    ");

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "invalid type: integer `123`, expected a string at line 2 column 19",
                },
                fatal: true,
                path: "name",
            },
            TrackedError {
                kind: InvalidField {
                    message: "invalid type: integer `1`, expected a string at line 3 column 20",
                },
                fatal: true,
                path: "values[0]",
            },
            TrackedError {
                kind: InvalidField {
                    message: "invalid type: null, expected a map of `String`s to `String`s at line 4 column 26",
                },
                fatal: true,
                path: "key_values",
            },
        ],
    }
    "#);
}

#[test]
fn test_simple_renamed_field() {
    mod pb {
        tonic::include_proto!("simple");
    }

    let mut message = pb::SimpleMessageRenamed::default();
    let mut tracker = <pb::SimpleMessageRenamed as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "name": "test",
        "values": ["value1", "value2"],
        "key_values": {
            "key1": "value1",
            "key2": "value2"
        }
    }"#,
    );

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();

        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    SimpleMessageRenamed {
        name: "test",
        values: [
            "value1",
            "value2",
        ],
        key_values: {
            "key1": "value1",
            "key2": "value2",
        },
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleMessageRenamedTracker {
            name: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        PrimitiveTracker<alloc::string::String>,
                        PrimitiveTracker<alloc::string::String>,
                    ],
                ),
            ),
            key_values: Some(
                {
                    "key1": PrimitiveTracker<alloc::string::String>,
                    "key2": PrimitiveTracker<alloc::string::String>,
                },
            ),
        },
    )
    "#);
}
