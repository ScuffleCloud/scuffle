// ^[_a-zA-Z][_a-zA-Z0-9]*(?:::[_a-zA-Z][_a-zA-Z0-9]*)*$

const fn validate_part(part: &str) {
    let invalid_parts = &["true", "false", "if", "then", "else", "in", "is", "like", "has", "__cedar"];
    konst::for_range! {idx in 0..invalid_parts.len() =>
        if konst::string::eq_str(invalid_parts[idx], part) {
            const_panic::concat_panic!("part cannot be a reserved keyword: ", part);
        }
    }
}

const fn validate(input: &str) {
    konst::iter::for_each! {part in konst::string::split(input, "::") =>
        validate_part(part);

        let mut chars = konst::string::chars(part);

        let Some(c) = chars.next() else {
            panic!("part must be non empty");
        };

        if c != '_' && !c.is_ascii_alphabetic() {
            const_panic::concat_panic!("part must start with either a '_' or any alphabetic character but started with: ", c);
        }

        konst::iter::for_each!{c in chars =>
            if c != '_' && !c.is_ascii_alphanumeric() {
                const_panic::concat_panic!("part can only contain '_' or alphanumeric characters but contained: ", c);
            }
        }
    }
}

/// A compile time checked entity type name
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EntityTypeName(&'static str);

impl serde::Serialize for EntityTypeName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0)
    }
}

impl std::fmt::Display for EntityTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl EntityTypeName {
    #[doc(hidden)]
    pub const fn new(input: &'static str) -> Self {
        validate(input);
        EntityTypeName(input)
    }

    /// Get the string value out of the entity type name.
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

/// A macro for creating a compile time checked entity type name
/// ```rust
/// entity_type_name!("SomeEntityType");
/// ```
#[macro_export]
macro_rules! entity_type_name {
    ($text:expr) => {
        const { $crate::EntityTypeName::new($text) }
    };
}
