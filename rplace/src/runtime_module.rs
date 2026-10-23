use std::{
    ffi::{CStr, CString, c_char},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use directories::ProjectDirs;
use libloading::Library;
use serde::Deserialize;
use serde_json::Value;

type PluginCall = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type PluginFree = unsafe extern "C" fn(*mut c_char);

#[derive(Deserialize)]
struct PluginResponse {
    ok: bool,
    value: Option<Value>,
    error: Option<String>,
}

pub struct RuntimeModule {
    _library: Library,
    call: PluginCall,
    free: PluginFree,
}

unsafe impl Send for RuntimeModule {}
unsafe impl Sync for RuntimeModule {}

impl RuntimeModule {
    pub fn load(name: &str) -> Result<Self> {
        let addons = ProjectDirs::from("io", "rplace", "rplace")
            .ok_or_else(|| anyhow!("Unable to locate RPlace data directory"))?
            .data_dir()
            .join("addons");
        let path = plugin_path(&addons, name);
        let library = unsafe { Library::new(&path) }
            .with_context(|| format!("Unable to load runtime module {}", path.display()))?;

        let call_symbol = unsafe { library.get::<PluginCall>(b"rplace_plugin_call\0") }
            .with_context(|| {
                format!(
                    "Runtime module {} is missing rplace_plugin_call",
                    path.display()
                )
            })?;
        let call = *call_symbol;
        let free_symbol = unsafe { library.get::<PluginFree>(b"rplace_plugin_free\0") }
            .with_context(|| {
                format!(
                    "Runtime module {} is missing rplace_plugin_free",
                    path.display()
                )
            })?;
        let free = *free_symbol;

        Ok(Self {
            _library: library,
            call,
            free,
        })
    }

    pub fn call(&self, request: &Value) -> Result<Value> {
        let request = CString::new(serde_json::to_vec(request)?)?;
        let response_ptr = unsafe { (self.call)(request.as_ptr()) };
        if response_ptr.is_null() {
            bail!("Runtime module returned a null response");
        }

        let response_text = unsafe {
            let response = CStr::from_ptr(response_ptr).to_string_lossy().into_owned();
            (self.free)(response_ptr);
            response
        };
        let response: PluginResponse = serde_json::from_str(&response_text)
            .context("Runtime module returned an invalid response")?;
        if !response.ok {
            bail!(
                "Runtime module error: {}",
                response
                    .error
                    .unwrap_or_else(|| "unknown error".to_string())
            );
        }

        response
            .value
            .ok_or_else(|| anyhow!("Runtime module response did not include a value"))
    }
}

fn plugin_path(addons: &std::path::Path, name: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let names = [format!("{name}.dll"), format!("lib{name}.dll")];
    #[cfg(target_os = "macos")]
    let names = [
        format!("lib{name}.dylib"),
        format!("{name}.dylib"),
        format!("lib{name}.so"),
    ];
    #[cfg(all(unix, not(target_os = "macos")))]
    let names = [format!("lib{name}.so"), format!("{name}.so")];

    names
        .iter()
        .map(|name| addons.join(name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| addons.join(&names[0]))
}

#[cfg(test)]
mod tests {
    use super::plugin_path;
    use std::path::Path;

    #[test]
    fn constructs_platform_runtime_module_filename() {
        let path = plugin_path(Path::new("/tmp/rplace-addons"), "rplace_lua");
        assert!(path.to_string_lossy().contains("rplace_lua"));
        #[cfg(target_os = "windows")]
        assert!(path.to_string_lossy().ends_with(".dll"));
        #[cfg(target_os = "macos")]
        assert!(path.to_string_lossy().ends_with(".dylib"));
        #[cfg(all(unix, not(target_os = "macos")))]
        assert!(path.to_string_lossy().ends_with(".so"));
    }
}
