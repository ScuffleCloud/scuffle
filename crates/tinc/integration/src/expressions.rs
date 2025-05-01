#![allow(clippy::approx_constant)]

use tinc::__private::{TrackerSharedState, ValidateMessage};

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
    };

    state.in_scope(|| valid.validate()).unwrap();

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
    };

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be exactly `5` characters long",
                },
                fatal: true,
                proto_path: "code",
                serde_path: "code",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be at least `3` characters long",
                },
                fatal: true,
                proto_path: "name",
                serde_path: "name",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must match the pattern `^(\\+\\d{1,2}\\s?)?\\(?\\d{3}\\)?[\\s.-]?\\d{3}[\\s.-]?\\d{4}$`",
                },
                fatal: true,
                proto_path: "phone_number",
                serde_path: "phone_number",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not match the pattern `@gmail\\.com$`",
                },
                fatal: true,
                proto_path: "email",
                serde_path: "email",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must start with `fk_`",
                },
                fatal: true,
                proto_path: "foreign_key",
                serde_path: "foreign_key",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must end with `_id`",
                },
                fatal: true,
                proto_path: "primary_key",
                serde_path: "primary_key",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must contain `e`",
                },
                fatal: true,
                proto_path: "word_with_e",
                serde_path: "word_with_e",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not contain `z`",
                },
                fatal: true,
                proto_path: "word_without_z",
                serde_path: "word_without_z",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[chocolate, vanilla]`",
                },
                fatal: true,
                proto_path: "ice_cream",
                serde_path: "ice_cream",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[troy]`",
                },
                fatal: true,
                proto_path: "best_friend",
                serde_path: "best_friend",
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

    state.in_scope(|| valid.validate()).unwrap();

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

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than or equal to 1.00",
                },
                fatal: true,
                proto_path: "zero_to_one",
                serde_path: "zero_to_one",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0.00`",
                },
                fatal: true,
                proto_path: "bigger_than_zero",
                serde_path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0.00`",
                },
                fatal: true,
                proto_path: "less_than_zero",
                serde_path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5.10, 10.20, -5.20, -10.40]`",
                },
                fatal: true,
                proto_path: "bucket",
                serde_path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3.14, 2.71]`",
                },
                fatal: true,
                proto_path: "coolest_float",
                serde_path: "coolest_float",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3.00`",
                },
                fatal: true,
                proto_path: "pi",
                serde_path: "pi",
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

    state.in_scope(|| valid.validate()).unwrap();

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

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than or equal to `1.00`",
                },
                fatal: true,
                proto_path: "zero_to_one",
                serde_path: "zero_to_one",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0.00`",
                },
                fatal: true,
                proto_path: "bigger_than_zero",
                serde_path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0.00`",
                },
                fatal: true,
                proto_path: "less_than_zero",
                serde_path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5.10, 10.20, -5.20, -10.40]`",
                },
                fatal: true,
                proto_path: "bucket",
                serde_path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3.14, 2.71]`",
                },
                fatal: true,
                proto_path: "coolest_float",
                serde_path: "coolest_float",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3.00`",
                },
                fatal: true,
                proto_path: "pi",
                serde_path: "pi",
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

    state.in_scope(|| valid.validate()).unwrap();

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

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `0`",
                },
                fatal: true,
                proto_path: "zero_to_ten",
                serde_path: "zero_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                proto_path: "bigger_than_zero",
                serde_path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0`",
                },
                fatal: true,
                proto_path: "less_than_zero",
                serde_path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, -5, -10]`",
                },
                fatal: true,
                proto_path: "bucket",
                serde_path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                proto_path: "coolest_int32",
                serde_path: "coolest_int32",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                proto_path: "pi",
                serde_path: "pi",
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

    state.in_scope(|| valid.validate()).unwrap();

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

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `0`",
                },
                fatal: true,
                proto_path: "zero_to_ten",
                serde_path: "zero_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `0`",
                },
                fatal: true,
                proto_path: "bigger_than_zero",
                serde_path: "bigger_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `0`",
                },
                fatal: true,
                proto_path: "less_than_zero",
                serde_path: "less_than_zero",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, -5, -10]`",
                },
                fatal: true,
                proto_path: "bucket",
                serde_path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                proto_path: "coolest_int64",
                serde_path: "coolest_int64",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                proto_path: "pi",
                serde_path: "pi",
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

    state.in_scope(|| valid.validate()).unwrap();

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

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `1`",
                },
                fatal: true,
                proto_path: "one_to_ten",
                serde_path: "one_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `100`",
                },
                fatal: true,
                proto_path: "bigger_than_100",
                serde_path: "bigger_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `100`",
                },
                fatal: true,
                proto_path: "less_than_100",
                serde_path: "less_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, 15, 20]`",
                },
                fatal: true,
                proto_path: "bucket",
                serde_path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                proto_path: "coolest_uint32",
                serde_path: "coolest_uint32",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                proto_path: "pi",
                serde_path: "pi",
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

    state.in_scope(|| valid.validate()).unwrap();

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

    state.in_scope(|| valid.validate()).unwrap();

    insta::assert_debug_snapshot!(state, @r#"
    TrackerSharedState {
        fail_fast: false,
        errors: [
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than or equal to `1`",
                },
                fatal: true,
                proto_path: "one_to_ten",
                serde_path: "one_to_ten",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be greater than `100`",
                },
                fatal: true,
                proto_path: "bigger_than_100",
                serde_path: "bigger_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be less than `100`",
                },
                fatal: true,
                proto_path: "less_than_100",
                serde_path: "less_than_100",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be one of `[5, 10, 15, 20]`",
                },
                fatal: true,
                proto_path: "bucket",
                serde_path: "bucket",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must not be one of `[3, 2, 1]`",
                },
                fatal: true,
                proto_path: "coolest_uint64",
                serde_path: "coolest_uint64",
            },
            TrackedError {
                kind: InvalidField {
                    message: "value must be equal to `3`",
                },
                fatal: true,
                proto_path: "pi",
                serde_path: "pi",
            },
        ],
    }
    "#);
}
