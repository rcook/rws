use crate::scripting::command_prelude;
use rlua::{Context, Lua};

pub fn eval(script: &str, use_prelude: bool) -> Vec<String> {
    Lua::new().context(|lua_ctx| {
        if use_prelude {
            load_prelude(&lua_ctx)
        }
        lua_ctx.load(script).eval().unwrap()
    })
}

fn load_prelude(lua_ctx: &Context) {
    let prelude = lua_ctx.create_table().unwrap();

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

    prelude
        .set(
            "read_file",
            lua_ctx
                .create_function(|_, arg: String| command_prelude::read_file(arg))
                .unwrap(),
        )
        .unwrap();

    prelude
        .set(
            "read_file_lines",
            lua_ctx
                .create_function(|_, arg: String| command_prelude::read_file_lines(arg))
                .unwrap(),
        )
        .unwrap();

    prelude
        .set(
            "trim_string",
            lua_ctx
                .create_function(|_, arg: String| command_prelude::trim_string(arg))
                .unwrap(),
        )
        .unwrap();

    lua_ctx.globals().set("prelude", prelude).unwrap();
}
