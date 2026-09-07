pub(super) fn append_string_key(parent: &str, key: &str) -> String {
    if is_identifier(key) {
        if parent == "." {
            format!(".{key}")
        } else {
            format!("{parent}.{key}")
        }
    } else {
        append_bracket(parent, &quote_string(key))
    }
}

pub(super) fn append_bracket(parent: &str, value: &str) -> String {
    let prefix = if parent == "." { "" } else { parent };
    format!("{prefix}[{value}]")
}

fn quote_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\u{08}' => quoted.push_str("\\b"),
            '\u{0c}' => quoted.push_str("\\f"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            character if character <= '\u{1f}' => {
                use std::fmt::Write as _;
                write!(quoted, "\\u{:04x}", u32::from(character))
                    .expect("writing to a String cannot fail");
            }
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

fn is_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || matches!(character, '_' | '$'))
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '$'))
        && !matches!(
            key,
            "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
                | "yield"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    mod append_string_key {
        use super::*;

        #[test]
        fn uses_dot_notation_only_for_identifiers() {
            assert_eq!(append_string_key(".", "name"), ".name");
            assert_eq!(append_string_key(".item", "value"), ".item.value");
            assert_eq!(append_string_key(".", "true"), r#"["true"]"#);
            assert_eq!(append_string_key(".", "first name"), r#"["first name"]"#);
        }

        #[test]
        fn quotes_json_control_characters() {
            assert_eq!(append_string_key(".", "a\"b\nc"), r#"["a\"b\nc"]"#);
        }
    }
}
