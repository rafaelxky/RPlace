use std::sync::{Arc, LazyLock, RwLock};

use crate::lua::lua_call_map::LuaCallMap;

pub static LUA_ENGINE: LazyLock<Arc<RwLock<LuaCallMap>>> = LazyLock::new(||{
    let lua = LuaCallMap::load();
    Arc::new(RwLock::new(lua))
});