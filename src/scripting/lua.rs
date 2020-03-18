use crate::config::ConfigObject;
use crate::error::{AppError, Result};
use crate::scripting::prelude;

use rlua::{Context, Lua};
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

fn create_variables(lua_ctx: Context, variables: &Vec<(String, ConfigObject)>) -> Result<()> {
    let globals_table = lua_ctx.globals();
    for (name, config_object) in variables {
        let value = config_object.to_lua(lua_ctx)?;
        let key = lua_ctx.create_string(name).expect("create_string failed");
        globals_table.set(key, value).expect("set failed");
    }

    Ok(())
}

pub fn eval0(
    script: &str,
    use_prelude: bool,
    variables: &Vec<(String, ConfigObject)>,
) -> Result<Vec<String>> {
    Lua::new().context(|lua_ctx| {
        create_variables(lua_ctx, variables)?;
        if use_prelude {
            load_prelude(&lua_ctx)?;
        }
        Ok(lua_ctx.load(script).eval()?)
    })
}

pub fn eval1(
    script: &str,
    use_prelude: bool,
    variables: &Vec<(String, ConfigObject)>,
) -> Result<()> {
    Lua::new().context(|lua_ctx| {
        create_variables(lua_ctx, variables)?;
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
        lua_ctx.create_function(|_, path| prelude::is_file(path))?,
    )?;

    prelude.set(
        "is_dir",
        lua_ctx.create_function(|_, path| prelude::is_dir(path))?,
    )?;

    prelude.set(
        "copy_file",
        lua_ctx.create_function(|_, (from, to)| prelude::copy_file(from, to))?,
    )?;

    prelude.set(
        "copy_file_if_unchanged",
        lua_ctx.create_function(|_, (from, to)| prelude::copy_file_if_unchanged::main(from, to))?,
    )?;

    prelude.set(
        "read_file",
        lua_ctx.create_function(|_, path| prelude::read_file(path))?,
    )?;

    prelude.set(
        "read_file_lines",
        lua_ctx.create_function(|_, path| prelude::read_file_lines(path))?,
    )?;

    prelude.set(
        "trim_string",
        lua_ctx.create_function(|_, str| prelude::trim_string(str))?,
    )?;

    prelude.set(
        "xpath",
        lua_ctx.create_function(|_, (namespaces_table, query, xml)| {
            prelude::xpath::main(namespaces_table, query, xml)
        })?,
    )?;

    prelude.set(
        "git_clone",
        lua_ctx.create_function(|_, args| prelude::git_clone(args))?,
    )?;

    prelude.set(
        "percent_decode",
        lua_ctx.create_function(|_, str| prelude::percent_decode(str))?,
    )?;

    lua_ctx.globals().set("prelude", prelude)
}
