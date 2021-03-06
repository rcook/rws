use crate::error::{internal_error, Error, Result};

use super::prelude;
use super::variables::Variables;

use rlua::{Context, FromLuaMulti, Lua};

pub trait Evaluatable: for<'lua> rlua::FromLuaMulti<'lua> {}
impl<T: for<'lua> FromLuaMulti<'lua>> Evaluatable for T {}

impl std::convert::From<rlua::Error> for Error {
    fn from(error: rlua::Error) -> Self {
        internal_error("Lua", error.to_string())
    }
}

pub fn eval<T: Evaluatable>(
    preamble: &str,
    script: &str,
    use_prelude: bool,
    variables: &Variables,
) -> Result<T> {
    Lua::new().context(|lua_ctx| {
        create_variables(lua_ctx, variables)?;

        if use_prelude {
            load_prelude(lua_ctx)?;
        }

        Ok(lua_ctx
            .load(&(preamble.to_string() + "\n\n" + script))
            .eval()?)
    })
}

fn create_variables(lua_ctx: Context, variables: &Variables) -> Result<()> {
    let globals_table = lua_ctx.globals();
    for (name, config_object) in &variables.values {
        let value = config_object.to_lua(lua_ctx)?;
        let key = lua_ctx.create_string(&name)?;
        globals_table.set(key, value)?;
    }

    Ok(())
}

fn create_git(lua_ctx: Context) -> Result<rlua::Table> {
    let git = lua_ctx.create_table()?;

    git.set(
        "clone",
        lua_ctx.create_function(|_, arg| prelude::git::clone(arg))?,
    )?;

    Ok(git)
}

fn load_prelude(lua_ctx: Context) -> rlua::Result<()> {
    let prelude = lua_ctx.create_table()?;

    // Nested objects
    prelude.set("git", create_git(lua_ctx)?)?;

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
