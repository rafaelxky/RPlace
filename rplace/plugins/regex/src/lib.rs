use std::ffi::{CStr, CString, c_char};

use regex::Regex;
use serde_json::{Value, json};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rplace_plugin_call(request: *const c_char) -> *mut c_char {
    let result = unsafe { read_request(request) }.and_then(find_matches);
    encode_response(result)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rplace_plugin_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

unsafe fn read_request(request: *const c_char) -> Result<Value, String> {
    if request.is_null() {
        return Err("Request was null".to_string());
    }
    let request = unsafe { CStr::from_ptr(request) }
        .to_str()
        .map_err(|error| error.to_string())?;
    serde_json::from_str(request).map_err(|error| error.to_string())
}

fn find_matches(request: Value) -> Result<Value, String> {
    if request.get("action").and_then(Value::as_str) != Some("find") {
        return Err("Unknown regex action".to_string());
    }
    let pattern = request
        .get("pattern")
        .and_then(Value::as_str)
        .ok_or_else(|| "Request is missing regex pattern".to_string())?;
    let text = request
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| "Request is missing text".to_string())?;
    let regex = Regex::new(pattern).map_err(|error| error.to_string())?;
    Ok(json!(
        regex
            .find_iter(text)
            .map(|matched| [matched.start(), matched.end()])
            .collect::<Vec<_>>()
    ))
}

fn encode_response(result: Result<Value, String>) -> *mut c_char {
    let response = match result {
        Ok(value) => json!({ "ok": true, "value": value }),
        Err(error) => json!({ "ok": false, "error": error }),
    };
    CString::new(response.to_string())
        .expect("JSON response cannot contain a null byte")
        .into_raw()
}

#[cfg(test)]
mod tests {
    use super::find_matches;
    use serde_json::json;

    #[test]
    fn returns_regex_match_byte_ranges() {
        let matches = find_matches(json!({
            "action": "find",
            "pattern": "[Vv]ar",
            "text": "var Var value",
        }))
        .expect("Regex should match");

        assert_eq!(matches, json!([[0, 3], [4, 7]]));
    }

    #[test]
    fn reports_invalid_patterns() {
        let error = find_matches(json!({
            "action": "find",
            "pattern": "[",
            "text": "text",
        }))
        .expect_err("Invalid pattern should be rejected");

        assert!(error.contains("regex"));
    }
}
