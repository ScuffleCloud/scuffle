use heck::{ToSnakeCase, ToUpperCamelCase};

pub fn sanitize_identifier(s: impl AsRef<str>) -> String {
    let ident = s.as_ref();
    // Use a raw identifier if the identifier matches a Rust keyword:
    // https://doc.rust-lang.org/reference/keywords.html.
    match ident {
        // 2015 strict keywords.
        | "as" | "break" | "const" | "continue" | "else" | "enum" | "false"
        | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "static" | "struct" | "trait" | "true"
        | "type" | "unsafe" | "use" | "where" | "while"
        // 2018 strict keywords.
        | "dyn"
        // 2015 reserved keywords.
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "typeof"
        | "unsized" | "virtual" | "yield"
        // 2018 reserved keywords.
        | "async" | "await" | "try"
        // 2024 reserved keywords.
        | "gen" => format!("r#{}", ident),
        // the following keywords are not supported as raw identifiers and are therefore suffixed with an underscore.
        "_" | "super" | "self" | "Self" | "extern" | "crate" => format!("{}_", ident),
        // the following keywords begin with a number and are therefore prefixed with an underscore.
        s if s.starts_with(|c: char| c.is_numeric()) => format!("_{}", ident),
        _ => ident.to_string(),
    }
}

/// Converts a `camelCase` or `SCREAMING_SNAKE_CASE` identifier to a `lower_snake` case Rust field
/// identifier.
pub fn to_snake(s: impl AsRef<str>) -> String {
    sanitize_identifier(s.as_ref().to_snake_case())
}

/// Converts a `snake_case` identifier to an `UpperCamel` case Rust type identifier.
pub fn to_upper_camel(s: impl AsRef<str>) -> String {
    sanitize_identifier(s.as_ref().to_upper_camel_case())
}

pub fn strip_enum_prefix(prefix: &str, name: &str) -> String {
    let stripped = name.strip_prefix(prefix).unwrap_or(name);

    // If the next character after the stripped prefix is not
    // uppercase, then it means that we didn't have a true prefix -
    // for example, "Foo" should not be stripped from "Foobar".
    let stripped = if stripped.chars().next().map(char::is_uppercase).unwrap_or(false) {
        stripped
    } else {
        name
    };
    sanitize_identifier(stripped)
}
