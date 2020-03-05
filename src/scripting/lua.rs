use crate::scripting::lua_rws_prelude;
use rlua::{Context, Lua, Table};

pub fn eval(script: &str, use_prelude: bool) -> Vec<String> {
    Lua::new().context(|lua_ctx| {
        if use_prelude {
            load_prelude(&lua_ctx)
        }
        lua_ctx.load(script).eval().unwrap()
    })
}

fn load_prelude(lua_ctx: &Context) {
    let prelude_script = String::from_utf8(include_bytes!("prelude.lua").to_vec()).unwrap();
    let prelude = lua_ctx.load(&prelude_script).eval::<Table>().unwrap();

    let current_dir = lua_ctx
        .create_function(|_, ()| lua_rws_prelude::current_dir())
        .unwrap();
    prelude.set("current_dir", current_dir).unwrap();

    let is_file = lua_ctx
        .create_function(|_, arg: String| lua_rws_prelude::is_file(arg))
        .unwrap();
    prelude.set("is_file", is_file).unwrap();

    lua_ctx.globals().set("prelude", prelude).unwrap();
}