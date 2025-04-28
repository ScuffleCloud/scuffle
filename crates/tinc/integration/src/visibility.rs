use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tonic::include_proto!("visibility");
}

#[test]
fn test_visibility() {
    let mut message = pb::VisibilityMessage::default();
    let mut tracker = <pb::VisibilityMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState {
        fail_fast: false,
        ..Default::default()
    };
    let mut de = serde_json::Deserializer::from_str(
        r#"{
        "output_only": "output",
        "input_only": "input",
        "input_outputs": {
            "UNSPECIFIED": "UNSPECIFIED",
            "INPUT_ONLY": "INPUT_ONLY",
            "OUTPUT_ONLY": "OUTPUT_ONLY",
            "INPUT_OUTPUT": "INPUT_OUTPUT"
        },
        "nothing": "nothing"
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
                kind: UnknownField,
                fatal: false,
                proto_path: "",
                serde_path: "output_only",
            },
            TrackedError {
                kind: InvalidField {
                    message: "unknown variant `UNSPECIFIED`, expected `INPUT_ONLY` or `INPUT_OUTPUT` at line 5 column 40",
                },
                fatal: true,
                proto_path: "input_outputs[\"UNSPECIFIED\"]",
                serde_path: "input_outputs[\"UNSPECIFIED\"]",
            },
            TrackedError {
                kind: InvalidField {
                    message: "unknown variant `OUTPUT_ONLY`, expected `INPUT_ONLY` or `INPUT_OUTPUT` at line 7 column 40",
                },
                fatal: true,
                proto_path: "input_outputs[\"OUTPUT_ONLY\"]",
                serde_path: "input_outputs[\"OUTPUT_ONLY\"]",
            },
            TrackedError {
                kind: UnknownField,
                fatal: false,
                proto_path: "",
                serde_path: "nothing",
            },
        ],
    }
    "#);
    insta::assert_debug_snapshot!(message, @r#"
    VisibilityMessage {
        output_only: "",
        input_only: "input",
        input_outputs: {
            "INPUT_ONLY": InputOnly,
            "INPUT_OUTPUT": InputOutput,
        },
        nothing: "",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r#"
    StructTracker(
        VisibilityMessageTracker {
            output_only: None,
            input_only: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            input_outputs: Some(
                {
                    "UNSPECIFIED": EnumTracker<tinc_integration_tests::visibility::pb::VisibilityEnum>,
                    "INPUT_ONLY": EnumTracker<tinc_integration_tests::visibility::pb::VisibilityEnum>,
                    "OUTPUT_ONLY": EnumTracker<tinc_integration_tests::visibility::pb::VisibilityEnum>,
                    "INPUT_OUTPUT": EnumTracker<tinc_integration_tests::visibility::pb::VisibilityEnum>,
                },
            ),
            nothing: None,
        },
    )
    "#);

    insta::assert_json_snapshot!(pb::VisibilityMessage {
        input_only: "input".to_string(),
        nothing: "nothing".to_string(),
        output_only: "output".to_string(),
        input_outputs: {
            let mut map = std::collections::BTreeMap::new();
            map.insert("output_only".to_owned(), pb::VisibilityEnum::OutputOnly as i32);
            map.insert("input_output".to_owned(), pb::VisibilityEnum::InputOutput as i32);
            map
        }
    }, @r#"
    {
      "output_only": "output",
      "input_outputs": {
        "input_output": "INPUT_OUTPUT",
        "output_only": "OUTPUT_ONLY"
      }
    }
    "#);

    // we cannot output these because they are input only.
    assert!(serde_json::to_string(&pb::VisibilityEnum::InputOnly).is_err());
    assert!(serde_json::to_string(&pb::VisibilityEnum::Unspecified).is_err());
}
