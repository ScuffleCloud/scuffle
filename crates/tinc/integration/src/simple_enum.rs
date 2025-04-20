use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

#[test]
fn test_simple_enum() {
    mod pb {
        tonic::include_proto!("simple_enum");
    }

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
                EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum::pb::SimpleEnum>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum::pb::SimpleEnum>,
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum::pb::SimpleEnum>,
                    ],
                ),
            ),
            map: Some(
                {
                    "key1": EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum::pb::SimpleEnum>,
                    "key2": EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum::pb::SimpleEnum>,
                },
            ),
            optional: Some(
                OptionalTracker(
                    Some(
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum::pb::SimpleEnum>,
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
    mod pb {
        tonic::include_proto!("simple_enum");
    }

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
                EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed::pb::SimpleEnumRenamed>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed::pb::SimpleEnumRenamed>,
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed::pb::SimpleEnumRenamed>,
                    ],
                ),
            ),
            map: Some(
                {
                    "key1": EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed::pb::SimpleEnumRenamed>,
                    "key2": EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed::pb::SimpleEnumRenamed>,
                },
            ),
            optional: Some(
                OptionalTracker(
                    Some(
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed::pb::SimpleEnumRenamed>,
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
    mod pb {
        tonic::include_proto!("simple_enum");
    }

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
                EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr::pb::SimpleEnumRepr>,
            ),
            values: Some(
                RepeatedVecTracker(
                    [
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr::pb::SimpleEnumRepr>,
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr::pb::SimpleEnumRepr>,
                    ],
                ),
            ),
            map: Some(
                {
                    "key1": EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr::pb::SimpleEnumRepr>,
                    "key2": EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr::pb::SimpleEnumRepr>,
                },
            ),
            optional: Some(
                OptionalTracker(
                    Some(
                        EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr::pb::SimpleEnumRepr>,
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
    mod pb {
        tonic::include_proto!("simple_enum");
    }

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

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();

        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
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
                path: "value",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "values",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "map",
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
                EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_invalid::pb::SimpleEnum>,
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
    mod pb {
        tonic::include_proto!("simple_enum");
    }

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

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();

        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
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
                path: "value",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "values",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "map",
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
                EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_renamed_invalid::pb::SimpleEnumRenamed>,
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
    mod pb {
        tonic::include_proto!("simple_enum");
    }

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

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut message, &mut tracker).unwrap();

        TrackedStructDeserializer::validate::<serde::de::value::Error>(&message, &mut tracker).unwrap();
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
                path: "value",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "values",
            },
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "map",
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
                EnumTracker<tinc_integration_tests::simple_enum::test_simple_enum_repr_invalid::pb::SimpleEnumRepr>,
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
