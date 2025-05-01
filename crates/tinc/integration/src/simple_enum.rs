use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("simple_enum");
}

#[test]
fn test_simple_enum() {
    let mut message = pb::Simple::default();
    let mut tracker = <pb::Simple as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "value": "ONE",
        "values": ["ONE", "TWO"],
        "map": {
            "key1": "ONE",
            "key2": "TWO"
        },
        "optional": "THREE"
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
    Simple {
        value: One,
        values: [
            One,
            Two,
        ],
        map: {
            "key1": One,
            "key2": Two,
        },
        optional: Some(
            Three,
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleTracker {
            value: Some(
                EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
                    ],
                ),
            ),
            map: Some(
                {
                    "key1": EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
                    "key2": EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
                },
            ),
            optional: Some(
                OptionalTracker(
                    Some(
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
                    ),
                ),
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "value": "ONE",
      "values": [
        "ONE",
        "TWO"
      ],
      "map": {
        "key1": "ONE",
        "key2": "TWO"
      },
      "optional": "THREE"
    }
    "#);
}

#[test]
fn test_simple_enum_renamed() {
    let mut message = pb::SimpleRenamed::default();
    let mut tracker = <pb::SimpleRenamed as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "value": "one",
        "values": ["one", "two"],
        "map": {
            "key1": "one",
            "key2": "two"
        },
        "optional": "three"
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
    SimpleRenamed {
        value: One,
        values: [
            One,
            Two,
        ],
        map: {
            "key1": One,
            "key2": Two,
        },
        optional: Some(
            Three,
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleRenamedTracker {
            value: Some(
                EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
                    ],
                ),
            ),
            map: Some(
                {
                    "key1": EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
                    "key2": EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
                },
            ),
            optional: Some(
                OptionalTracker(
                    Some(
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
                    ),
                ),
            ),
        },
    )
    "#);
    insta::assert_json_snapshot!(message, @r#"
    {
      "value": "one",
      "values": [
        "one",
        "two"
      ],
      "map": {
        "key1": "one",
        "key2": "two"
      },
      "optional": "three"
    }
    "#);
}

#[test]
fn test_simple_enum_repr() {
    let mut message = pb::SimpleRepr::default();
    let mut tracker = <pb::SimpleRepr as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "value": 1,
        "values": [1, 2],
        "map": {
            "key1": 1,
            "key2": 2
        },
        "optional": 3
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
    SimpleRepr {
        value: One,
        values: [
            One,
            Two,
        ],
        map: {
            "key1": One,
            "key2": Two,
        },
        optional: Some(
            Three,
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        SimpleReprTracker {
            value: Some(
                EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
                    ],
                ),
            ),
            map: Some(
                {
                    "key1": EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
                    "key2": EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
                },
            ),
            optional: Some(
                OptionalTracker(
                    Some(
                        EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
                    ),
                ),
            ),
        },
    )
    "#);
    insta::assert_json_snapshot!(message, @r#"
    {
      "value": 1,
      "values": [
        1,
        2
      ],
      "map": {
        "key1": 1,
        "key2": 2
      },
      "optional": 3
    }
    "#);
}

#[test]
fn test_simple_enum_invalid() {
    let mut message = pb::Simple::default();
    let mut tracker = <pb::Simple as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "value": "FOUR"
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
                    message: "unknown variant `FOUR`, expected one of `UNSPECIFIED`, `ONE`, `TWO`, `THREE` at line 2 column 23",
                },
                fatal: true,
                proto_path: "value",
                serde_path: "value",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                proto_path: "values",
                serde_path: "values",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                proto_path: "map",
                serde_path: "map",
            },
        ],
    }
    "#);
    insta::assert_debug_snapshot!(message, @r"
    Simple {
        value: Unspecified,
        values: [],
        map: {},
        optional: None,
    }
    ");
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        SimpleTracker {
            value: Some(
                EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnum>,
            ),
            values: None,
            map: None,
            optional: None,
        },
    )
    ");
    insta::assert_json_snapshot!(message, @r#"
    {
      "value": "UNSPECIFIED",
      "values": [],
      "map": {},
      "optional": null
    }
    "#);
}

#[test]
fn test_simple_enum_renamed_invalid() {
    let mut message = pb::SimpleRenamed::default();
    let mut tracker = <pb::SimpleRenamed as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "value": "four"
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
                    message: "unknown variant `four`, expected one of `unspecified`, `one`, `two`, `three` at line 2 column 23",
                },
                fatal: true,
                proto_path: "value",
                serde_path: "value",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                proto_path: "values",
                serde_path: "values",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                proto_path: "map",
                serde_path: "map",
            },
        ],
    }
    "#);
    insta::assert_debug_snapshot!(message, @r"
    SimpleRenamed {
        value: Unspecified,
        values: [],
        map: {},
        optional: None,
    }
    ");
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        SimpleRenamedTracker {
            value: Some(
                EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRenamed>,
            ),
            values: None,
            map: None,
            optional: None,
        },
    )
    ");
    insta::assert_json_snapshot!(message, @r#"
    {
      "value": "unspecified",
      "values": [],
      "map": {},
      "optional": null
    }
    "#);
}

#[test]
fn test_simple_enum_repr_invalid() {
    let mut message = pb::SimpleRepr::default();
    let mut tracker = <pb::SimpleRepr as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "value": 4
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
                    message: "invalid value: 4, expected one of: 0, 1, 2, 3",
                },
                fatal: true,
                proto_path: "value",
                serde_path: "value",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                proto_path: "values",
                serde_path: "values",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                proto_path: "map",
                serde_path: "map",
            },
        ],
    }
    "#);
    insta::assert_debug_snapshot!(message, @r"
    SimpleRepr {
        value: Unspecified,
        values: [],
        map: {},
        optional: None,
    }
    ");
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        SimpleReprTracker {
            value: Some(
                EnumTracker<tinc_integration_tests::simple_enum::pb::SimpleEnumRepr>,
            ),
            values: None,
            map: None,
            optional: None,
        },
    )
    ");
    insta::assert_json_snapshot!(message, @r#"
    {
      "value": 0,
      "values": [],
      "map": {},
      "optional": null
    }
    "#);
}
