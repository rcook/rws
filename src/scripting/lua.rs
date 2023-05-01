// The MIT License (MIT)
//
// Copyright (c) 2020-3 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use super::prelude;
use super::variables::Variables;
use crate::workspace::Workspace;
use anyhow::Result;
use joatmon::path_to_str;
use rlua::prelude::{FromLuaMulti, Lua, LuaContext, LuaTable};

pub trait Evaluatable: for<'lua> FromLuaMulti<'lua> {}

impl<T: for<'lua> FromLuaMulti<'lua>> Evaluatable for T {}

pub fn eval<T: Evaluatable>(
    workspace: &Workspace,
    preamble: &str,
    script: &str,
    use_prelude: bool,
    variables: &Variables,
) -> Result<T> {
    Lua::new().context(|lua_ctx| {
        create_variables(lua_ctx, variables)?;

        if use_prelude {
            load_prelude(lua_ctx, workspace)?;
        }

        Ok(lua_ctx
            .load(&(preamble.to_string() + "\n\n" + script))
            .eval()?)
    })
}

fn create_variables(lua_ctx: LuaContext, variables: &Variables) -> Result<()> {
    let globals_table = lua_ctx.globals();
    for (name, config_object) in &variables.values {
        let value = config_object.to_lua(lua_ctx)?;
        let key = lua_ctx.create_string(&name)?;
        globals_table.set(key, value)?;
    }

    Ok(())
}

fn create_git(lua_ctx: LuaContext) -> Result<LuaTable> {
    let git = lua_ctx.create_table()?;

    git.set(
        "clone",
        lua_ctx.create_function(|_, arg| prelude::git::clone(arg))?,
    )?;

    Ok(git)
}

fn load_prelude(lua_ctx: LuaContext, workspace: &Workspace) -> Result<()> {
    let prelude = lua_ctx.create_table()?;

    // Nested objects
    prelude.set("git", create_git(lua_ctx)?)?;

    prelude.set(
        "workspace_dir",
        lua_ctx.create_string(path_to_str(&workspace.workspace_dir))?,
    )?;

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

    lua_ctx.globals().set("prelude", prelude)?;

    Ok(())
}
