use tinc::__private::{TincValidate, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("recursive");
}

#[test]
fn test_recursive() {
    let mut message = pb::RecursiveMessage::default();
    let mut tracker = <pb::RecursiveMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "anothers": [
            {
                "another": null,
                "nested": null
            }
        ],
        "another_optional": null,
        "another_map": {
            "key1": {
                "another": null,
                "nested": null
            },
            "key2": {
                "another": null,
                "nested": {
                    "anothers": [],
                    "another_optional": null,
                    "another_map": {},
                    "depth": 2
                }
            }
        },
        "depth": 1
    }"#,
    );

    deserialize_tracker_target(&mut state, &mut de, &mut tracker, &mut message).unwrap();
    state.in_scope(|| {
        TincValidate::validate(&message, Some(&tracker)).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(message, @r#"
    RecursiveMessage {
        anothers: [
            AnotherMessage {
                another: None,
                nested: None,
            },
        ],
        another_optional: None,
        another_map: {
            "key1": AnotherMessage {
                another: None,
                nested: None,
            },
            "key2": AnotherMessage {
                another: None,
                nested: Some(
                    RecursiveMessage {
                        anothers: [],
                        another_optional: None,
                        another_map: {},
                        depth: 2,
                    },
                ),
            },
        },
        depth: 1,
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        RecursiveMessageTracker {
            anothers: Some(
                RepeatedVecTracker(
                    [
                        StructTracker(
                            AnotherMessageTracker {
                                another: Some(
                                    OptionalTracker(
                                        None,
                                    ),
                                ),
                                nested: Some(
                                    OptionalTracker(
                                        None,
                                    ),
                                ),
                            },
                        ),
                    ],
                ),
            ),
            another_optional: Some(
                OptionalTracker(
                    None,
                ),
            ),
            another_map: Some(
                {
                    "key1": StructTracker(
                        AnotherMessageTracker {
                            another: Some(
                                OptionalTracker(
                                    None,
                                ),
                            ),
                            nested: Some(
                                OptionalTracker(
                                    None,
                                ),
                            ),
                        },
                    ),
                    "key2": StructTracker(
                        AnotherMessageTracker {
                            another: Some(
                                OptionalTracker(
                                    None,
                                ),
                            ),
                            nested: Some(
                                OptionalTracker(
                                    Some(
                                        StructTracker(
                                            RecursiveMessageTracker {
                                                anothers: Some(
                                                    RepeatedVecTracker(
                                                        [],
                                                    ),
                                                ),
                                                another_optional: Some(
                                                    OptionalTracker(
                                                        None,
                                                    ),
                                                ),
                                                another_map: Some(
                                                    {},
                                                ),
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
                },
            ),
            depth: Some(
                PrimitiveTracker<i32>,
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "anothers": [
        {
          "another": null,
          "nested": null
        }
      ],
      "another_optional": null,
      "another_map": {
        "key1": {
          "another": null,
          "nested": null
        },
        "key2": {
          "another": null,
          "nested": {
            "anothers": [],
            "another_optional": null,
            "another_map": {},
            "depth": 2
          }
        }
      },
      "depth": 1
    }
    "#);
}
