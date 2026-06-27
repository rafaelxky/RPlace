use std::{collections::HashMap, sync::LazyLock};

type FnReturn = String;
type FnType = fn(String) -> FnReturn;
type MapType = HashMap<&'static str, FnType>;
static VAR_OPTIONS: LazyLock<MapType> = LazyLock::new(|| {
    let mut hm: MapType = HashMap::new();
    hm.insert("snakecase", to_snake_case);
    hm.insert("camelcase", to_camel_case);
    hm.insert("screaming", to_screaming_case);
    hm.insert("pascalcase", to_pascal_case);
    hm
});
pub fn to_pascal_case(input: String) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for ch in input.chars() {
        if ch == ' ' || ch == '-' || ch == '_' {
            capitalize_next = true;
            continue;
        }

        if result.is_empty() {
            result.push(ch.to_uppercase().next().unwrap());
        } else if capitalize_next {
            for c in ch.to_uppercase() {
                result.push(c);
            }
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}
pub fn exec_option(name: &str, val: String) -> String {
    let opt = VAR_OPTIONS.get(name);
    let opt = match opt {
        Some(opt) => opt,
        None => panic!("todo errro message, no option found {}", name),
    };
    opt(val)
}
pub fn to_screaming_case(input: String) -> String {
    let mut result = String::new();

    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                result.push('_');
            }
            for c in ch.to_uppercase() {
                result.push(c);
            }
        } else if ch == ' ' || ch == '-' {
            result.push('_');
        } else {
            result.push(ch.to_ascii_uppercase());
        }
    }

    result
}
pub fn to_snake_case(var: String) -> String {
    let mut result = String::new();

    for (i, ch) in var.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                result.push('_');
            }
            for lower in ch.to_lowercase() {
                result.push(lower);
            }
        } else if ch == ' ' || ch == '-' {
            result.push('_');
        } else {
            result.push(ch);
        }
    }

    result
}
pub fn to_camel_case(input: String) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for ch in input.chars() {
        if ch == ' ' || ch == '-' || ch == '_' {
            capitalize_next = true;
            continue;
        }

        if result.is_empty() {
            result.push(ch.to_lowercase().next().unwrap());
        } else if capitalize_next {
            for c in ch.to_uppercase() {
                result.push(c);
            }
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}
