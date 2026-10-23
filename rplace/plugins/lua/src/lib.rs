use std::{
    ffi::{CStr, CString, c_char},
    fs,
    path::Path,
};

use mlua::{Function, Lua};
use serde_json::{Value, json};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rplace_plugin_call(request: *const c_char) -> *mut c_char {
    let result = unsafe { read_request(request) }.and_then(process);
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

fn process(request: Value) -> Result<Value, String> {
    let lua = Lua::new();
    let addons_dir = request
        .get("addons_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| "Request is missing addons_dir".to_string())?;
    load_lua_addons(&lua, Path::new(addons_dir))?;

    let action = request
        .get("action")
        .and_then(Value::as_str)
        .ok_or_else(|| "Request is missing action".to_string())?;
    let result = match action {
        "execute" => {
            let code = request
                .get("code")
                .and_then(Value::as_str)
                .ok_or_else(|| "Request is missing Lua code".to_string())?;
            let args = request
                .get("args")
                .and_then(Value::as_array)
                .ok_or_else(|| "Request args must be an array".to_string())?
                .iter()
                .map(|arg| {
                    arg.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "Lua arguments must be strings".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            lua.globals()
                .set("args", args)
                .map_err(|error| error.to_string())?;
            lua.load(code)
                .eval::<String>()
                .map_err(|error| error.to_string())?
        }
        "call" => {
            let name = request
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| "Request is missing function name".to_string())?;
            let function = lua
                .globals()
                .get::<Function>(name)
                .map_err(|error| error.to_string())?;
            function.call(()).map_err(|error| error.to_string())?
        }
        _ => return Err(format!("Unknown Lua action {action}")),
    };

    Ok(json!(result))
}

fn load_lua_addons(lua: &Lua, addons_dir: &Path) -> Result<(), String> {
    let entries = match fs::read_dir(addons_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    let mut paths = entries
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();

    for path in paths {
        if path.extension().is_none_or(|extension| extension != "lua") {
            continue;
        }
        let source = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        lua.load(&source)
            .set_name(path.to_string_lossy())
            .exec()
            .map_err(|error| error.to_string())?;
    }
    Ok(())
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
    use super::process;
    use serde_json::json;

    #[test]
    fn executes_lua_with_template_args() {
        let response = process(json!({
            "action": "execute",
            "code": "return args[1] .. args[2]",
            "args": ["hello", " world"],
            "addons_dir": "/path/that/does/not/exist",
        }))
        .expect("Lua execution should succeed");

        assert_eq!(response, json!("hello world"));
    }
}
