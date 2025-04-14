use serde::de::DeserializeSeed;
use tinc::__private::de::{
    DeserializeHelper, DeserializerWrapper, TrackedStructDeserializer, TrackerFor, TrackerSharedState, TrackerStateGuard,
};

#[test]
fn test_recursive() {
    mod pb {
        tonic::include_proto!("recursive");
    }

    let mut message = pb::RecursiveMessage::default();
    let mut tracker = <pb::RecursiveMessage as TrackerFor>::Tracker::default();
    let guard = TrackerStateGuard::new(TrackerSharedState::default());

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "another": [
            {
                "another": null,
                "nested": null
            }
        ],
        "anotherOptional": null,
        "anotherMap": {
            "key1": {
                "another": null,
                "nested": null
            },
            "key2": {
                "another": null,
                "nested": {
                    "another": [],
                    "anotherOptional": null,
                    "anotherMap": {},
                    "depth": 2
                }
            }
        },
        "depth": 1
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
    RecursiveMessage {
        another: [
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
                        another: [],
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
    MessageTracker(
        RecursiveMessageTracker {
            another: Some(
                RepeatedVecTracker(
                    [
                        MessageTracker(
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
                    "key1": MessageTracker(
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
                    "key2": MessageTracker(
                        AnotherMessageTracker {
                            another: Some(
                                OptionalTracker(
                                    None,
                                ),
                            ),
                            nested: Some(
                                OptionalTracker(
                                    Some(
                                        MessageTracker(
                                            RecursiveMessageTracker {
                                                another: Some(
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
}
