use crate::error::{AppError, Result};
use crate::scripting::prelude;

use rlua::{Context, Lua, Variadic};
use std::sync::Arc;

impl std::convert::From<rlua::Error> for AppError {
    fn from(error: rlua::Error) -> Self {
        AppError::System("Lua", error.to_string())
    }
}

impl std::convert::From<AppError> for rlua::Error {
    fn from(error: AppError) -> Self {
        rlua::Error::ExternalError(Arc::new(error))
    }
}

pub fn eval0(script: &str, use_prelude: bool) -> Result<Vec<String>> {
    Lua::new().context(|lua_ctx| {
        if use_prelude {
            load_prelude(&lua_ctx)?;
        }
        Ok(lua_ctx.load(script).eval()?)
    })
}

pub fn eval1(script: &str, use_prelude: bool) -> Result<()> {
    Lua::new().context(|lua_ctx| {
        if use_prelude {
            load_prelude(&lua_ctx)?;
        }
        Ok(lua_ctx.load(script).eval()?)
    })
}

fn load_prelude(lua_ctx: &Context) -> rlua::Result<()> {
    let prelude = lua_ctx.create_table()?;

    prelude.set(
        "current_dir",
        lua_ctx.create_function(|_, ()| prelude::current_dir())?,
    )?;

    prelude.set(
        "is_file",
        lua_ctx.create_function(|_, path: String| prelude::is_file(path))?,
    )?;

    prelude.set(
        "is_dir",
        lua_ctx.create_function(|_, path: String| prelude::is_dir(path))?,
    )?;

    prelude.set(
        "read_file",
        lua_ctx.create_function(|_, path: String| prelude::read_file(path))?,
    )?;

    prelude.set(
        "read_file_lines",
        lua_ctx.create_function(|_, path: String| prelude::read_file_lines(path))?,
    )?;

    prelude.set(
        "trim_string",
        lua_ctx.create_function(|_, str: String| prelude::trim_string(str))?,
    )?;

    prelude.set(
        "xpath",
        lua_ctx.create_function(|_, (namespaces_table, query, xml)| {
            prelude::xpath::main(namespaces_table, query, xml)
        })?,
    )?;

    prelude.set(
        "git_clone",
        lua_ctx.create_function(|_, args: Variadic<String>| prelude::git_clone(args))?,
    )?;

    lua_ctx.globals().set("prelude", prelude)
}
