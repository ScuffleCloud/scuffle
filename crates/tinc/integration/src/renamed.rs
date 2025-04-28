use tinc::__private::{TrackedStructDeserializer, TrackerFor, TrackerSharedState, deserialize_tracker_target};

mod pb {
    #![allow(clippy::all)]
    tonic::include_proto!("renamed");
}

macro_rules! create_rename_test {
    ($message:ty, $field:literal) => {{
        let mut value = <$message>::default();
        let mut tracker = <$message as TrackerFor>::Tracker::default();
        let mut state = TrackerSharedState::default();
        let json = format!(r#"{{ "{}": "SOME VALUE!!!" }}"#, $field);
        let mut de = serde_json::Deserializer::from_str(&json);

        state.in_scope(|| {
            deserialize_tracker_target(&mut de, &mut value, &mut tracker).unwrap();
            TrackedStructDeserializer::validate::<serde::de::value::Error>(&value, &mut tracker).unwrap();
        });

        (state, value, tracker)
    }};
}

#[test]
fn test_screaming_snake_case() {
    let (state, value, tracker) = create_rename_test!(pb::ScreamingSnakeCaseMessage, "MY_CUSTOM_FIELD");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    ScreamingSnakeCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        ScreamingSnakeCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "MY_CUSTOM_FIELD": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_lower_case() {
    let (state, value, tracker) = create_rename_test!(pb::LowerCaseMessage, "my_custom_field");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    LowerCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        LowerCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "my_custom_field": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_upper_case() {
    let (state, value, tracker) = create_rename_test!(pb::UpperCaseMessage, "MYCUSTOMFIELD");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    UpperCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        UpperCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "MYCUSTOMFIELD": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_pascal_case() {
    let (state, value, tracker) = create_rename_test!(pb::PascalCaseMessage, "MyCustomField");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    PascalCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        PascalCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "MyCustomField": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_camel_case() {
    let (state, value, tracker) = create_rename_test!(pb::CamelCaseMessage, "myCustomField");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    CamelCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        CamelCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "myCustomField": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_kebab_case() {
    let (state, value, tracker) = create_rename_test!(pb::KebabCaseMessage, "my-custom-field");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    KebabCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        KebabCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "my-custom-field": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_screaming_kebab_case() {
    let (state, value, tracker) = create_rename_test!(pb::ScreamingKebabCaseMessage, "MY-CUSTOM-FIELD");
    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    ScreamingKebabCaseMessage {
        my_custom_field: "SOME VALUE!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        ScreamingKebabCaseMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "MY-CUSTOM-FIELD": "SOME VALUE!!!"
    }
    "#);
}

#[test]
fn test_rename_with_override() {
    let mut value = pb::RenameAllWithOverrideMessage::default();
    let mut tracker = <pb::RenameAllWithOverrideMessage as TrackerFor>::Tracker::default();
    let mut state = TrackerSharedState::default();
    let json = r#"{
        "myCustomField": "SOME VALUE!!!",
        "MY_CUSTOM_FIELD": "SOME VALUE 2!!!",
        "my_custom_field": "SOME VALUE 3!!!",
        "MY-CUSTOM-FIELD-4": "SOME VALUE 4!!!"
}"#;
    let mut de = serde_json::Deserializer::from_str(json);

    state.in_scope(|| {
        deserialize_tracker_target(&mut de, &mut value, &mut tracker).unwrap();
        TrackedStructDeserializer::validate::<serde::de::value::Error>(&value, &mut tracker).unwrap();
    });

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
    insta::assert_debug_snapshot!(value, @r#"
    RenameAllWithOverrideMessage {
        my_custom_field: "SOME VALUE!!!",
        my_custom_field2: "SOME VALUE 2!!!",
        my_custom_field3: "SOME VALUE 3!!!",
        my_custom_field4: "SOME VALUE 4!!!",
    }
    "#);
    insta::assert_debug_snapshot!(tracker, @r"
    StructTracker(
        RenameAllWithOverrideMessageTracker {
            my_custom_field: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            my_custom_field2: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            my_custom_field3: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
            my_custom_field4: Some(
                PrimitiveTracker<alloc::string::String>,
            ),
        },
    )
    ");
    insta::assert_json_snapshot!(value, @r#"
    {
      "myCustomField": "SOME VALUE!!!",
      "MY_CUSTOM_FIELD": "SOME VALUE 2!!!",
      "my_custom_field": "SOME VALUE 3!!!",
      "MY-CUSTOM-FIELD-4": "SOME VALUE 4!!!"
    }
    "#);
}

#[test]
fn test_enum_rename() {
    let cases = [
        (pb::RenameEnum::OneValue, "OneValue"),
        (pb::RenameEnum::TwoValue, "TwoValue"),
        (pb::RenameEnum::ThreeValue, "ThreeValue"),
        (pb::RenameEnum::FourValue, "four-value"),
    ];

    for (variant, expected) in cases.iter() {
        let json_str = format!("\"{}\"", *expected);
        assert_eq!(serde_json::to_string(variant).unwrap(), json_str);
        let deserialized: pb::RenameEnum = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized, *variant);
    }
}
