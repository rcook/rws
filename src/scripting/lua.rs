use crate::error::{AppError, Result};
use crate::scripting::command_prelude;
use rlua::{Context, Lua};

impl std::convert::From<rlua::Error> for AppError {
    fn from(error: rlua::Error) -> Self {
        AppError::System("Lua", error.to_string())
    }
}

pub fn eval(script: &str, use_prelude: bool) -> Result<Vec<String>> {
    Lua::new().context(|lua_ctx| {
        if use_prelude {
            load_prelude(&lua_ctx)?;
        }
        let result = lua_ctx.load(script).eval()?;
        Ok(result)
    })
}

fn load_prelude(lua_ctx: &Context) -> rlua::Result<()> {
    let prelude = lua_ctx.create_table()?;

    prelude.set(
        "current_dir",
        lua_ctx.create_function(|_, ()| command_prelude::current_dir())?,
    )?;

    prelude.set(
        "is_file",
        lua_ctx.create_function(|_, arg: String| command_prelude::is_file(arg))?,
    )?;

    prelude.set(
        "read_file",
        lua_ctx.create_function(|_, arg: String| command_prelude::read_file(arg))?,
    )?;

    prelude.set(
        "read_file_lines",
        lua_ctx.create_function(|_, arg: String| command_prelude::read_file_lines(arg))?,
    )?;

    prelude.set(
        "trim_string",
        lua_ctx.create_function(|_, arg: String| command_prelude::trim_string(arg))?,
    )?;

    lua_ctx.globals().set("prelude", prelude)
}
