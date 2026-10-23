use std::sync::Mutex;

use serde_json::json;

use crate::{config::config::CompilerConfig, runtime_module::RuntimeModule};

pub struct LuaCallMap {
    module: Mutex<Option<RuntimeModule>>,
    enabled: bool,
}

impl LuaCallMap {
    pub fn empty(compiler_config: std::sync::Arc<CompilerConfig>) -> Self {
        Self {
            module: Mutex::new(None),
            enabled: compiler_config.allow_lua,
        }
    }

    pub fn load(compiler_config: std::sync::Arc<CompilerConfig>) -> Self {
        Self::empty(compiler_config)
    }

    pub fn run<T: ToString>(&self, name: T) -> String {
        self.call(json!({
            "action": "call",
            "name": name.to_string(),
            "addons_dir": addons_dir(),
        }))
    }

    pub fn execute(&self, code: &str, args: Vec<String>) -> String {
        self.call(json!({
            "action": "execute",
            "code": code,
            "args": args,
            "addons_dir": addons_dir(),
        }))
    }

    fn call(&self, request: serde_json::Value) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut module = self.module.lock().expect("Lua module lock poisoned");
        if module.is_none() {
            *module = Some(
                RuntimeModule::load("rplace_lua")
                    .unwrap_or_else(|error| panic!("Unable to load Lua module: {error:#}")),
            );
        }

        module
            .as_ref()
            .expect("Lua module was not initialized")
            .call(&request)
            .unwrap_or_else(|error| panic!("Lua module call failed: {error:#}"))
            .as_str()
            .unwrap_or_else(|| panic!("Lua module returned a non-string value"))
            .to_string()
    }
}

fn addons_dir() -> String {
    directories::ProjectDirs::from("io", "rplace", "rplace")
        .expect("Unable to locate RPlace data directory")
        .data_dir()
        .join("addons")
        .to_string_lossy()
        .into_owned()
}
