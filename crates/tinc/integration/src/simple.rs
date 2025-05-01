use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tonic::include_proto!("simple");
}

#[test]
fn test_simple_single_pass() {
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

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();

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

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TrackedStructDeserializer::validate(&message, &mut tracker).unwrap();
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
    let mut message = pb::SimpleMessage::default();
    let mut tracker = <pb::SimpleMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "key_values": {
        }
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();

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

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| TrackedStructDeserializer::validate(&message, &mut tracker).unwrap());

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
                proto_path: "name",
                serde_path: "name",
            },
        ],
    }
    "#);
}

#[test]
fn test_simple_duplicate_fields() {
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

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();

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

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TrackedStructDeserializer::validate(&message, &mut tracker).unwrap();
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
                proto_path: "values",
                serde_path: "values",
            },
            TrackedError {
                kind: DuplicateField,
                fatal: true,
                proto_path: "key_values[\"key1\"]",
                serde_path: "key_values[\"key1\"]",
            },
            TrackedError {
                kind: DuplicateField,
                fatal: true,
                proto_path: "key_values[\"key2\"]",
                serde_path: "key_values[\"key2\"]",
            },
        ],
    }
    "#);
}

#[test]
fn test_simple_invalid_type() {
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

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TrackedStructDeserializer::validate(&message, &mut tracker).unwrap();
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
                proto_path: "name",
                serde_path: "name",
            },
            TrackedError {
                kind: InvalidField {
                    message: "invalid type: integer `1`, expected a string at line 3 column 20",
                },
                fatal: true,
                proto_path: "values[0]",
                serde_path: "values[0]",
            },
            TrackedError {
                kind: InvalidField {
                    message: "invalid type: null, expected a map of `String`s to `String`s at line 4 column 26",
                },
                fatal: true,
                proto_path: "key_values",
                serde_path: "key_values",
            },
        ],
    }
    "#);
}

#[test]
fn test_simple_renamed_field() {
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
