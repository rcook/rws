use crate::scripting::command_prelude;
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
    let prelude_script = String::from_utf8(include_bytes!("command_prelude.lua").to_vec()).unwrap();
    let prelude = lua_ctx.load(&prelude_script).eval::<Table>().unwrap();

    prelude
        .set(
            "current_dir",
            lua_ctx
                .create_function(|_, ()| command_prelude::current_dir())
                .unwrap(),
        )
        .unwrap();

    prelude
        .set(
            "is_file",
            lua_ctx
                .create_function(|_, arg: String| command_prelude::is_file(arg))
                .unwrap(),
        )
        .unwrap();

    lua_ctx.globals().set("prelude", prelude).unwrap();
}
