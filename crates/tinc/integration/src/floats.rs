use tinc::__private::{TincValidate, TrackerFor, TrackerSharedState, deserialize_tracker_target};
use tinc::TincService;

mod pb {
    #![allow(clippy::all)]
    tinc::include_proto!("floats");
}

struct Svc {}

#[tonic::async_trait]
impl pb::float_service_server::FloatService for Svc {
    async fn float(&self, _: tonic::Request<pb::FloatRequest>) -> tonic::Result<tonic::Response<pb::FloatResponse>> {
        Ok(pb::FloatResponse { param: "OK".to_string() }.into())
    }
}

#[test]
fn test_parse_simple() {
    let mut message = pb::FloatMessage::default();
    let mut tracker = <pb::FloatMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "simple_f32": 0.5,
        "simple_f64": 0.25,
        "rep_f32": [8, 0.125],
        "rep_f64": [32],
        "opt_f32": 0.5,
        "google_f64": 64,
        "map_f32": {"k1": 2.0},
        "map_f64": {"k2": 16.0},
        "variant": {
          "oneof_f64": 0.5
        }
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
    FloatMessage {
        simple_f32: 0.5,
        simple_f64: 0.25,
        rep_f32: [
            8.0,
            0.125,
        ],
        rep_f64: [
            32.0,
        ],
        opt_f32: Some(
            0.5,
        ),
        opt_f64: None,
        google_f32: None,
        google_f64: Some(
            64.0,
        ),
        map_f32: {
            "k1": 2.0,
        },
        map_f64: {
            "k2": 16.0,
        },
        variant: Some(
            OneofF64(
                0.5,
            ),
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        FloatMessageTracker {
            simple_f32: Some(
                FloatWithNonFinTracker<f32>,
            ),
            simple_f64: Some(
                FloatWithNonFinTracker<f64>,
            ),
            rep_f32: Some(
                RepeatedVecTracker(
                    [
                        FloatWithNonFinTracker<f32>,
                        FloatWithNonFinTracker<f32>,
                    ],
                ),
            ),
            rep_f64: Some(
                RepeatedVecTracker(
                    [
                        FloatWithNonFinTracker<f64>,
                    ],
                ),
            ),
            opt_f32: Some(
                OptionalTracker(
                    Some(
                        FloatWithNonFinTracker<f32>,
                    ),
                ),
            ),
            opt_f64: None,
            google_f32: None,
            google_f64: Some(
                OptionalTracker(
                    Some(
                        FloatWithNonFinTracker<f64>,
                    ),
                ),
            ),
            map_f32: Some(
                {
                    "k1": FloatWithNonFinTracker<f32>,
                },
            ),
            map_f64: Some(
                {
                    "k2": FloatWithNonFinTracker<f64>,
                },
            ),
            variant: Some(
                OneOfTracker(
                    Some(
                        OneofF64(
                            FloatWithNonFinTracker<f64>,
                        ),
                    ),
                ),
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "simple_f32": 0.5,
      "simple_f64": 0.25,
      "rep_f32": [
        8.0,
        0.125
      ],
      "rep_f64": [
        32.0
      ],
      "opt_f32": 0.5,
      "opt_f64": null,
      "google_f32": null,
      "google_f64": 64.0,
      "map_f32": {
        "k1": 2.0
      },
      "map_f64": {
        "k2": 16.0
      },
      "variant": {
        "oneof_f64": 0.5
      }
    }
    "#);
}

#[test]
fn test_parse_with_special_values() {
    let mut message = pb::FloatMessage::default();
    let mut tracker = <pb::FloatMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();

    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "simple_f32": "NaN",
        "simple_f64": "Infinity",
        "rep_f32": ["-Infinity", "NaN"],
        "rep_f64": ["Infinity"],
        "opt_f64": "NaN",
        "google_f32": "Infinity",
        "google_f64": "-Infinity",
        "map_f32": {"k1": "NaN"},
        "map_f64": {"k2": "Infinity"},
        "variant": {
          "oneof_f64": "NaN"
        }
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
    FloatMessage {
        simple_f32: NaN,
        simple_f64: inf,
        rep_f32: [
            -inf,
            NaN,
        ],
        rep_f64: [
            inf,
        ],
        opt_f32: None,
        opt_f64: Some(
            NaN,
        ),
        google_f32: Some(
            inf,
        ),
        google_f64: Some(
            -inf,
        ),
        map_f32: {
            "k1": NaN,
        },
        map_f64: {
            "k2": inf,
        },
        variant: Some(
            OneofF64(
                NaN,
            ),
        ),
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        FloatMessageTracker {
            simple_f32: Some(
                FloatWithNonFinTracker<f32>,
            ),
            simple_f64: Some(
                FloatWithNonFinTracker<f64>,
            ),
            rep_f32: Some(
                RepeatedVecTracker(
                    [
                        FloatWithNonFinTracker<f32>,
                        FloatWithNonFinTracker<f32>,
                    ],
                ),
            ),
            rep_f64: Some(
                RepeatedVecTracker(
                    [
                        FloatWithNonFinTracker<f64>,
                    ],
                ),
            ),
            opt_f32: None,
            opt_f64: Some(
                OptionalTracker(
                    Some(
                        FloatWithNonFinTracker<f64>,
                    ),
                ),
            ),
            google_f32: Some(
                OptionalTracker(
                    Some(
                        FloatWithNonFinTracker<f32>,
                    ),
                ),
            ),
            google_f64: Some(
                OptionalTracker(
                    Some(
                        FloatWithNonFinTracker<f64>,
                    ),
                ),
            ),
            map_f32: Some(
                {
                    "k1": FloatWithNonFinTracker<f32>,
                },
            ),
            map_f64: Some(
                {
                    "k2": FloatWithNonFinTracker<f64>,
                },
            ),
            variant: Some(
                OneOfTracker(
                    Some(
                        OneofF64(
                            FloatWithNonFinTracker<f64>,
                        ),
                    ),
                ),
            ),
        },
    )
    "#);

    insta::assert_json_snapshot!(message, @r#"
    {
      "simple_f32": "NaN",
      "simple_f64": "Infinity",
      "rep_f32": [
        "-Infinity",
        "NaN"
      ],
      "rep_f64": [
        "Infinity"
      ],
      "opt_f32": null,
      "opt_f64": "NaN",
      "google_f32": "Infinity",
      "google_f64": "-Infinity",
      "map_f32": {
        "k1": "NaN"
      },
      "map_f64": {
        "k2": "Infinity"
      },
      "variant": {
        "oneof_f64": "NaN"
      }
    }
    "#);
}

#[test]
fn test_float_service_rest_schema() {
    let svc = pb::float_service_tinc::FloatServiceTinc::new(Svc {});

    insta::assert_json_snapshot!(svc.openapi_schema());
}
