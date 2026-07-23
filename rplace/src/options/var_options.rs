use std::{collections::HashMap, sync::Arc};

use crate::{config::config::CompilerConfig, lua::{lua_call_map::LuaCallMap}, structs::VarOption};

type ArgType = String;
type FnReturn = String;
type FnType = fn(String, &Vec<String>, &VarOptionsContext) -> FnReturn;
type MapType = HashMap<&'static str, FnType>;

pub struct VarOptionsContext{
    config: Arc<CompilerConfig>,
    lua: LuaCallMap,
}
impl VarOptionsContext {
    pub fn new(config: Arc<CompilerConfig>, lua: LuaCallMap) -> Self{
        Self { config, lua}
    }
}
pub struct VarOptionsMap {
    hm: MapType,
    context: VarOptionsContext,
}
impl VarOptionsMap {
    pub fn new(config: Arc<CompilerConfig>, lua_map: LuaCallMap) -> Self {
        let mut hm: MapType = HashMap::new();
        hm.insert("snakecase", to_snake_case);
        hm.insert("camelcase", to_camel_case);
        hm.insert("screaming", to_screaming_case);
        hm.insert("pascalcase", to_pascal_case);
        hm.insert("lua", lua);

        let context = VarOptionsContext::new(config, lua_map);

        Self {
            hm,
            context,
        }
    }
    pub fn exec_option(&self, opt: &VarOption, val: String) -> String {
        let name = opt.option.as_str();
        let opt_name = self.hm.get(name);
        let func = match opt_name {
            Some(opt) => opt,
            None => panic!("todo error message, no option found {}", name),
        };
        func(val, &opt.args, &self.context)
    }
}

pub fn to_pascal_case(input: String, _arg: &Vec<ArgType>, _context: &VarOptionsContext) -> String {
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

pub fn lua(input: String, args: &Vec<ArgType>, context: &VarOptionsContext) -> String {
    if !context.config.allow_lua {
        return input;
    }
    let mut args_inner = vec![input];
    args_inner.extend(args.clone());
    context.lua.execute(&args[0], args_inner)
}
pub fn to_screaming_case(
    input: String,
    _args: &Vec<ArgType>,
    _context: &VarOptionsContext,
) -> String {
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
pub fn to_snake_case(
    var: String,
    _args: &Vec<ArgType>,
    _context: &VarOptionsContext,
) -> String {
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
pub fn to_camel_case(
    input: String,
    _args: &Vec<ArgType>,
    _context: &VarOptionsContext,
) -> String {
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
