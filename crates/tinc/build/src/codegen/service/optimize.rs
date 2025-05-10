pub(super) fn optimize_json_schema(schemas: impl IntoIterator<Item = serde_json::Value>) -> serde_json::Value {
    let mut all_ofs = Vec::new();

    for schema in schemas {
        if !collect_all_ofs(schema, &mut all_ofs) {
            return serde_json::Value::Bool(false);
        }
    }

    if all_ofs.is_empty() {
        return serde_json::Value::Bool(true);
    }

    let mut obj = serde_json::Map::new();

    for all_of in all_ofs.iter_mut() {
        let all_of = all_of.as_object_mut().unwrap();
        all_of.retain(|key, value| {
            if matches!(key.as_str(), "if" | "then" | "else") {
                return true;
            }

            let entry = obj.entry(key);
            match entry {
                serde_json::map::Entry::Occupied(mut o) => match (o.get() == value, key.as_ref()) {
                    (true, _) => false,
                    (false, "type") => {
                        *o.get_mut() = union_array_maybe(o.get_mut().take(), value.take());
                        false
                    }
                    (false, "enum") => {
                        *o.get_mut() = union_array(o.get_mut().take(), value.take());
                        false
                    }
                    (false, "maximum" | "maxLength" | "maxProperties" | "exclusiveMaximum" | "maxItems" | "maxContains") => {
                        min_two(o.get_mut(), value)
                    }
                    (false, "minimum" | "minLength" | "minProperties" | "exclusiveMinimum" | "minItems" | "minContains") => {
                        max_two(o.get_mut(), value)
                    }
                    (false, "uniqueItems") => set_true(o.get_mut()),
                    (false, "description" | "title") => false,
                    (false, "deprecated" | "readOnly" | "writeOnly") => set_true(o.get_mut()),
                    (false, "required" | "examples") => combine_array(o.get_mut(), value, false),
                    (false, "oneOf" | "anyOf") => combine_array(o.get_mut(), value, true),
                    (
                        false,
                        "unevaluatedItems"
                        | "unevaluatedProperties"
                        | "additionalProperties"
                        | "not"
                        | "items"
                        | "contains"
                        | "propertyNames",
                    ) => combine_subschema(o.get_mut(), value),
                    (false, "properties" | "patternProperties") => combine_properties(o.get_mut(), value),
                    _ => true,
                },
                serde_json::map::Entry::Vacant(v) => {
                    match key.as_ref() {
                        "enum" | "type" => {
                            combine_array(value, &mut serde_json::Value::Array(Vec::new()), false);
                        }
                        "required" | "examples" => {
                            combine_array(value, &mut serde_json::Value::Array(Vec::new()), false);
                        }
                        "oneOf" | "anyOf" => {
                            combine_array(value, &mut serde_json::Value::Array(Vec::new()), true);
                        }
                        "unevaluatedItems"
                        | "unevaluatedProperties"
                        | "additionalProperties"
                        | "not"
                        | "items"
                        | "contains"
                        | "propertyNames" => {
                            *value = optimize_json_schema([value.take()]);
                        }
                        "properties" | "patternProperties" => {
                            combine_properties(value, &mut serde_json::Value::Object(serde_json::Map::new()));
                        }
                        _ => {}
                    }
                    v.insert(value.take());
                    false
                }
            }
        });
    }

    all_ofs.retain(|a| a.as_object().is_some_and(|o| !o.is_empty()));

    if !all_ofs.is_empty() {
        obj.insert("allOf".into(), all_ofs.into());
    }

    obj.into()
}

fn set_true(o: &mut serde_json::Value) -> bool {
    *o = serde_json::Value::Bool(true);
    false
}

fn min_two(a: &mut serde_json::Value, b: &serde_json::Value) -> bool {
    if let (serde_json::Value::Number(a), serde_json::Value::Number(value)) = (a, b) {
        if let (Some(o_i), Some(value_i)) = (a.as_i128(), value.as_i128()) {
            *a = serde_json::Number::from_i128(o_i.min(value_i)).unwrap();
            false
        } else if let (Some(o_f), Some(value_f)) = (a.as_f64(), value.as_f64()) {
            *a = serde_json::Number::from_f64(o_f.min(value_f)).unwrap();
            false
        } else {
            true
        }
    } else {
        true
    }
}

fn max_two(a: &mut serde_json::Value, b: &serde_json::Value) -> bool {
    if let (serde_json::Value::Number(a), serde_json::Value::Number(value)) = (a, b) {
        if let (Some(o_i), Some(value_i)) = (a.as_i128(), value.as_i128()) {
            *a = serde_json::Number::from_i128(o_i.max(value_i)).unwrap();
            false
        } else if let (Some(o_f), Some(value_f)) = (a.as_f64(), value.as_f64()) {
            *a = serde_json::Number::from_f64(o_f.max(value_f)).unwrap();
            false
        } else {
            true
        }
    } else {
        true
    }
}

fn collect_all_ofs(mut value: serde_json::Value, collected: &mut Vec<serde_json::Value>) -> bool {
    let Some(obj) = value.as_object_mut() else {
        return !matches!(value, serde_json::Value::Bool(false));
    };

    let Some(serde_json::Value::Array(ofs)) = obj.remove("allOf") else {
        collected.push(value);
        return true;
    };

    for all_of in ofs {
        if !collect_all_ofs(all_of, collected) {
            return false;
        }
    }

    if value.as_object().is_some_and(|o| !o.is_empty()) {
        collected.push(value);
    }

    true
}

fn union_array_maybe(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    let a = if !a.is_array() { serde_json::Value::Array(vec![a]) } else { a };

    let b = if !b.is_array() { serde_json::Value::Array(vec![b]) } else { b };

    let serde_json::Value::Array(mut u) = union_array(a, b) else {
        unreachable!()
    };

    if u.len() == 1 {
        u.remove(0)
    } else {
        serde_json::Value::Array(u)
    }
}

fn union_array(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    let mut u = Vec::new();
    let serde_json::Value::Array(a) = a else {
        return serde_json::Value::Array(u);
    };
    let serde_json::Value::Array(b) = b else {
        return serde_json::Value::Array(u);
    };

    if a.is_empty() || b.is_empty() {
        return serde_json::Value::Array(u);
    }

    for item in a.into_iter().filter(|i| b.contains(i)) {
        if !u.contains(&item) {
            u.push(item);
        }
    }

    serde_json::Value::Array(u)
}

fn combine_array(o: &mut serde_json::Value, b: &mut serde_json::Value, optimize: bool) -> bool {
    let serde_json::Value::Array(o) = o else {
        return true;
    };
    let serde_json::Value::Array(b) = b else {
        return true;
    };

    let mut combined = Vec::new();
    for value in o.drain(..).chain(b.drain(..)) {
        let value = if optimize { optimize_json_schema([value]) } else { value };
        if !combined.contains(&value) {
            combined.push(value);
        }
    }

    *o = combined;

    false
}

fn combine_properties(o: &mut serde_json::Value, b: &mut serde_json::Value) -> bool {
    if !o.is_object() || !b.is_object() {
        return true;
    }

    let serde_json::Value::Object(a) = o.take() else {
        unreachable!()
    };
    let serde_json::Value::Object(b) = b.take() else {
        unreachable!()
    };

    let mut combined = serde_json::Map::new();

    for (key, mut value) in a.into_iter().chain(b) {
        match combined.entry(key) {
            serde_json::map::Entry::Occupied(mut o) => {
                combine_subschema(o.get_mut(), &mut value);
            }
            serde_json::map::Entry::Vacant(v) => {
                v.insert(optimize_json_schema([value]));
            }
        }
    }

    *o = combined.into();

    false
}

fn combine_subschema(o: &mut serde_json::Value, b: &mut serde_json::Value) -> bool {
    *o = optimize_json_schema([o.take(), b.take()]);

    false
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::optimize_json_schema;

    #[test]
    fn test_optimize_json_schema() {
        let opt = optimize_json_schema([serde_json::json!({
            "allOf": [
                {
                    "allOf": [
                        { "type": "string" },
                        {
                            "allOf": [
                                { "type": "string" }
                            ]
                        },
                    ]
                }
            ]
        })]);

        insta::assert_debug_snapshot!(opt, @r#"
        Object {
            "type": String("string"),
        }
        "#)
    }
}
