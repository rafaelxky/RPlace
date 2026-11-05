use std::ffi::{CStr, CString, c_char};

use rplace::lexer::TokenResult;
use serde_json::{Value, json};

pub mod fuzzy_lexer;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rplace_plugin_call(request: *const c_char) -> *mut c_char {
    let result = unsafe { read_request(request) }.and_then(fuzzy_lex);
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

fn fuzzy_lex(request: Value) -> Result<Value, String> {
    if request.get("action").and_then(Value::as_str) != Some("fuzzy_lex") {
        return Err("Unknown fuzzy syntax action".to_string());
    }
    let tokens = request
        .get("tokens")
        .cloned()
        .ok_or_else(|| "Request is missing tokens".to_string())?;
    let tokens: TokenResult = serde_json::from_value(tokens).map_err(|error| error.to_string())?;
    let result = fuzzy_lexer::FuzzyLexer::new(tokens).parse();
    serde_json::to_value(result).map_err(|error| error.to_string())
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
    use super::fuzzy_lex;
    use rplace::lexer::{Lexer, Token, TokenResult};
    use serde_json::json;

    #[test]
    fn processes_token_result_through_plugin_request() {
        let tokens = Lexer::new("test", "def example:\nend:".to_string()).parse();
        let response = fuzzy_lex(json!({
            "action": "fuzzy_lex",
            "tokens": serde_json::to_value(tokens).expect("Tokens should serialize"),
        }))
        .expect("Fuzzy lexer request should succeed");
        let result: TokenResult =
            serde_json::from_value(response).expect("Plugin response should contain tokens");

        assert!(matches!(
            result.tokens.first(),
            Some(Token::MARK { kind }) if kind == "//-"
        ));
    }
}
