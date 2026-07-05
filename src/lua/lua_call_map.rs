use std::{collections::HashMap, fs, path::Path};

use directories::ProjectDirs;
use mlua::{Function, Lua, LuaNativeFn, Value};

use crate::{config::config::CONFIG};

pub struct LuaCallMap {
    map: HashMap<String, Function>,
    lua: Lua,
}
impl LuaCallMap {
    pub fn load() -> Self {
        if !CONFIG.read().unwrap().allow_lua {
            return Self {
                map: HashMap::new(),
                lua: Lua::new(),
            };
        }

        let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
        let dir = dir.data_dir().join("addons");
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap();
            Self {
                map: HashMap::new(),
                lua: Lua::new(),
            }
        } else {
            let lua = Lua::new();
            let mut map = HashMap::new();

            for entry in fs::read_dir(&dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if path.extension().is_some_and(|e| e == "lua") {
                    let source = fs::read_to_string(&path).unwrap();
                    lua.load(&source).set_name(path.to_string_lossy()).exec().unwrap();

                    for pair in lua.globals().pairs::<String, Value>() {
                        let (name, value) = pair.unwrap();

                        if let Value::Function(func) = value {
                            map.insert(name, func);
                        }
                    }
                }
            }

            Self { map, lua }
        }
    }
    pub fn run<T:ToString>(&self, name: T) -> String{
        let fun = self.map.get(&name.to_string());
        let fun: Result<String, mlua::Error> = match fun {
            Some(fun) => {
                fun.call(())
            },
            None => panic!("No such lua function called {}", name.to_string()),
        };
        match fun {
            Ok(res) => {return res;},
            Err(e) => {
                panic!("Error calling function {}", &name.to_string())
            },
        }
    }
}
