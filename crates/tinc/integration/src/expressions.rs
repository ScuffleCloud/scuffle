#![allow(clippy::approx_constant)]

use std::collections::BTreeMap;

use tinc::__private::{TincValidate, TrackerSharedState};

mod pb {
    tinc::include_proto!("expressions");
}

#[test]
fn test_string_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::StringExpressions {
        code: "12345".into(),
        name: "troy".into(),
        phone_number: "+1 100 200 4563".into(),
        email: "troy@scuffle.cloud".into(),
        foreign_key: "fk_name".into(),
        primary_key: "user_id".into(),
        word_with_e: "elephant".into(),
        word_without_z: "friend".into(),
        ice_cream: "chocolate".into(),
        best_friend: "not_tr0y".into(),
        ipv6_only: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
        ipv4_only: "192.168.1.1".into(),
        ipv4_or_6_only: vec![
            "2::".into(),
            "2::1".into(),
            "2001:0db8:85a3::".into(),
            "2001:0db8:85a3::8a2e:0370:7334".into(),
            "192.168.1.1".into(),
        ],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_string_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::StringExpressions {
        code: "1234".into(),
        name: "ty".into(),
        phone_number: "+1 100 200 456".into(),
        email: "troy@gmail.com".into(),
        foreign_key: "fak_name".into(),
        primary_key: "user_ids".into(),
        word_with_e: "find".into(),
        word_without_z: "zoo".into(),
        ice_cream: "caramel".into(),
        best_friend: "troy".into(),
        ipv4_only: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".into(),
        ipv6_only: "192.168.1.1".into(),
        ipv4_or_6_only: vec!["hello".into(), "goodbye".into()],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be exactly `5` characters long",
                },
                fatal: true,
                path: "code",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `3` characters long",
                },
                fatal: true,
                path: "name",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must match the pattern `^(\\+\\d{1,2}\\s?)?\\(?\\d{3}\\)?[\\s.-]?\\d{3}[\\s.-]?\\d{4}$`",
                },
                fatal: true,
                path: "phone_number",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not match the pattern `@gmail\\.com$`",
                },
                fatal: true,
                path: "email",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must start with `fk_`",
                },
                fatal: true,
                path: "foreign_key",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must end with `_id`",
                },
                fatal: true,
                path: "primary_key",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must contain `e`",
                },
                fatal: true,
                path: "word_with_e",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not contain `z`",
                },
                fatal: true,
                path: "word_without_z",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[chocolate, vanilla]`",
                },
                fatal: true,
                path: "ice_cream",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[troy]`",
                },
                fatal: true,
                path: "best_friend",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be a valid ipv4 address",
                },
                fatal: true,
                path: "ipv4_only",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be a valid ipv6 address",
                },
                fatal: true,
                path: "ipv6_only",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be a valid ipv4 or ipv6 address",
                },
                fatal: true,
                path: "ipv4_or_6_only[0]",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be a valid ipv4 or ipv6 address",
                },
                fatal: true,
                path: "ipv4_or_6_only[1]",
            },
        ],
    }
    "#);
}

#[test]
fn test_float_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::FloatExpressions {
        zero_to_one: 0.5,
        bigger_than_zero: 12000.0,
        less_than_zero: -1000.0,
        bucket: -5.2,
        coolest_float: 34.4,
        pi: 3.0,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_float_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::FloatExpressions {
        zero_to_one: 1.1,
        bigger_than_zero: -1.0,
        less_than_zero: 1.0,
        bucket: -5.3,
        coolest_float: 3.14,
        pi: 3.14,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than or equal to 1.00",
                },
                fatal: true,
                path: "zero_to_one",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0.00`",
                },
                fatal: true,
                path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0.00`",
                },
                fatal: true,
                path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5.10, 10.20, -5.20, -10.40]`",
                },
                fatal: true,
                path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3.14, 2.71]`",
                },
                fatal: true,
                path: "coolest_float",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3.00`",
                },
                fatal: true,
                path: "pi",
            },
        ],
    }
    "#);
}

#[test]
fn test_double_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::DoubleExpressions {
        zero_to_one: 0.5,
        bigger_than_zero: 12000.0,
        less_than_zero: -1000.0,
        bucket: -5.2,
        coolest_float: 34.4,
        pi: 3.0,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_double_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::DoubleExpressions {
        zero_to_one: 1.1,
        bigger_than_zero: -1.0,
        less_than_zero: 1.0,
        bucket: -5.3,
        coolest_float: 3.14,
        pi: 3.14,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than or equal to `1.00`",
                },
                fatal: true,
                path: "zero_to_one",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0.00`",
                },
                fatal: true,
                path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0.00`",
                },
                fatal: true,
                path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5.10, 10.20, -5.20, -10.40]`",
                },
                fatal: true,
                path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3.14, 2.71]`",
                },
                fatal: true,
                path: "coolest_float",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3.00`",
                },
                fatal: true,
                path: "pi",
            },
        ],
    }
    "#);
}

#[test]
fn test_int32_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::Int32Expressions {
        zero_to_ten: 5,
        bigger_than_zero: 12000,
        less_than_zero: -1000,
        bucket: -5,
        coolest_int32: 5,
        pi: 3,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_int32_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::Int32Expressions {
        zero_to_ten: -5,
        bigger_than_zero: -1,
        less_than_zero: 5,
        bucket: -30,
        coolest_int32: 1,
        pi: 4,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `0`",
                },
                fatal: true,
                path: "zero_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0`",
                },
                fatal: true,
                path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, -5, -10]`",
                },
                fatal: true,
                path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                path: "coolest_int32",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                path: "pi",
            },
        ],
    }
    "#);
}

#[test]
fn test_int64_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::Int64Expressions {
        zero_to_ten: 5,
        bigger_than_zero: 12000,
        less_than_zero: -1000,
        bucket: -5,
        coolest_int64: 5,
        pi: 3,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_int64_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::Int64Expressions {
        zero_to_ten: -5,
        bigger_than_zero: -1,
        less_than_zero: 5,
        bucket: -30,
        coolest_int64: 1,
        pi: 4,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `0`",
                },
                fatal: true,
                path: "zero_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0`",
                },
                fatal: true,
                path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, -5, -10]`",
                },
                fatal: true,
                path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                path: "coolest_int64",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                path: "pi",
            },
        ],
    }
    "#);
}

#[test]
fn test_uint32_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::UInt32Expressions {
        one_to_ten: 5,
        bigger_than_100: 12000,
        less_than_100: 5,
        bucket: 5,
        coolest_uint32: 5,
        pi: 3,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_uint32_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::UInt32Expressions {
        one_to_ten: 0,
        bigger_than_100: 99,
        less_than_100: 102,
        bucket: 23,
        coolest_uint32: 1,
        pi: 4,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `1`",
                },
                fatal: true,
                path: "one_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `100`",
                },
                fatal: true,
                path: "bigger_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `100`",
                },
                fatal: true,
                path: "less_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, 15, 20]`",
                },
                fatal: true,
                path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                path: "coolest_uint32",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                path: "pi",
            },
        ],
    }
    "#);
}

#[test]
fn test_uint64_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::UInt64Expressions {
        one_to_ten: 5,
        bigger_than_100: 12000,
        less_than_100: 5,
        bucket: 5,
        coolest_uint64: 5,
        pi: 3,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_uint64_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::UInt64Expressions {
        one_to_ten: 0,
        bigger_than_100: 99,
        less_than_100: 102,
        bucket: 23,
        coolest_uint64: 1,
        pi: 4,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `1`",
                },
                fatal: true,
                path: "one_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `100`",
                },
                fatal: true,
                path: "bigger_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `100`",
                },
                fatal: true,
                path: "less_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, 15, 20]`",
                },
                fatal: true,
                path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                path: "coolest_uint64",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                path: "pi",
            },
        ],
    }
    "#);
}

#[test]
fn test_bytes_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::BytesExpressions {
        constant: b"\0\0\0".to_vec(),
        exact_len: b"troyb".to_vec(),
        min_max_len: b"0123456789".to_vec(),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_bytes_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::BytesExpressions {
        constant: b"\x001\x00".to_vec(),
        exact_len: b"troy".to_vec(),
        min_max_len: b"0123".to_vec(),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must equal `\0\0\0`",
                },
                fatal: true,
                path: "constant",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be exactly `5` bytes long",
                },
                fatal: true,
                path: "exact_len",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `5` bytes long",
                },
                fatal: true,
                path: "min_max_len",
            },
        ],
    }
    "#);
}

#[test]
fn test_enum_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::EnumExpressions {
        constant: 2,
        defined: 1,
        one_of: 1,
        none_of: 2,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_enum_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::EnumExpressions {
        constant: 1,
        defined: 3,
        one_of: 0,
        none_of: 0,
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `SPECIAL_B`",
                },
                fatal: true,
                path: "constant",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be defined in the enum",
                },
                fatal: true,
                path: "defined",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[SPECIAL_A, SPECIAL_B]`",
                },
                fatal: true,
                path: "one_of",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[SPECIAL_UNSPECIFIED]`",
                },
                fatal: true,
                path: "none_of",
            },
        ],
    }
    "#);
}

#[test]
fn test_repeated_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::RepeatedExpressions {
        numbers: vec![1, 2, 3, 4, 5],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_repeated_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::RepeatedExpressions {
        numbers: vec![1, 2, 0, 5],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must have exactly `5` elements",
                },
                fatal: true,
                path: "numbers",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                path: "numbers[2]",
            },
        ],
    }
    "#);
}

#[test]
fn test_map_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MapExpressions {
        numbers: {
            let mut map = BTreeMap::new();
            map.insert("troy_one".to_string(), 1);
            map.insert("troy_two".to_string(), 2);
            map.insert("troy_three".to_string(), 3);
            map.insert("troy_four".to_string(), 4);
            map.insert("troy_five".to_string(), 5);
            map
        },
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_map_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MapExpressions {
        numbers: {
            let mut map = BTreeMap::new();
            map.insert("one".to_string(), -1);
            map.insert("troy_two".to_string(), 2);
            map.insert("three".to_string(), 0);
            map.insert("troy_four".to_string(), 4);
            map.insert("troy_five".to_string(), -5);
            map
        },
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must match the pattern `^troy_`",
                },
                fatal: true,
                path: "numbers.one",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                path: "numbers.one",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must match the pattern `^troy_`",
                },
                fatal: true,
                path: "numbers.three",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                path: "numbers.three",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                path: "numbers.troy_five",
            },
        ],
    }
    "#);
}

#[test]
fn test_message_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MessageExpressions {
        message: Some(pb::message_expressions::SubMessage { name: "troy".into() }),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_message_expressions_not_provided() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MessageExpressions { message: None };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: MissingField,
                fatal: true,
                path: "message",
            },
        ],
    }
    "#);
}

#[test]
fn test_message_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MessageExpressions {
        message: Some(pb::message_expressions::SubMessage { name: "tr".into() }),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `3` characters long",
                },
                fatal: true,
                path: "message.name",
            },
        ],
    }
    "#);
}

#[test]
fn test_repeated_message_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::RepeatedMessageExpressions {
        messages: vec![pb::repeated_message_expressions::SubMessage { name: "troy".into() }],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_repeated_message_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::RepeatedMessageExpressions {
        messages: vec![pb::repeated_message_expressions::SubMessage { name: "tr".into() }],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `3` characters long",
                },
                fatal: true,
                path: "messages[0].name",
            },
        ],
    }
    "#);
}

#[test]
fn test_map_message_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MapMessageExpressions {
        messages: {
            let mut map = BTreeMap::new();

            map.insert(
                "first".into(),
                pb::map_message_expressions::SubMessage { name: "troy".into() },
            );

            map
        },
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_map_message_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::MapMessageExpressions {
        messages: {
            let mut map = BTreeMap::new();

            map.insert("first".into(), pb::map_message_expressions::SubMessage { name: "tr".into() });

            map
        },
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `3` characters long",
                },
                fatal: true,
                path: "messages.first.name",
            },
        ],
    }
    "#);
}

#[test]
fn test_custom_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::CustomExpressions {
        items: vec!["troy_one".into(), "troy_two".into(), "troy_three".into()],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_custom_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::CustomExpressions {
        items: vec!["troy".into(), "to".into(), "xd".into()],
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "all items must start with with 'troy_'",
                },
                fatal: true,
                path: "items",
            },
        ],
    }
    "#);
}

#[test]
fn test_oneof_expressions_valid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::OneofExpressions {
        tagged_nested: Some(pb::oneof_expressions::TaggedNested::Age(18)),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");

    let valid = pb::OneofExpressions {
        tagged_nested: Some(pb::oneof_expressions::TaggedNested::Name("troy".into())),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r"
    TrackerSharedState {
        fail_fast: false,
        errors: [],
    }
    ");
}

#[test]
fn test_oneof_expressions_invalid() {
    let mut state = TrackerSharedState::default();
    let valid = pb::OneofExpressions {
        tagged_nested: Some(pb::oneof_expressions::TaggedNested::Age(17)),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `18`",
                },
                fatal: true,
                path: "tagged_nested.age",
            },
        ],
    }
    "#);

    let valid = pb::OneofExpressions {
        tagged_nested: Some(pb::oneof_expressions::TaggedNested::Name("t".into())),
    };

    state.in_scope(|| valid.validate(None)).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `18`",
                },
                fatal: true,
                path: "tagged_nested.age",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `2` characters long",
                },
                fatal: true,
                path: "tagged_nested.name",
            },
        ],
    }
    "#);
}
