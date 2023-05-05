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
use super::super::variables::Variables;
use super::lua_config::translate_config_to_lua;
use super::prelude;
use crate::workspace::Workspace;
use anyhow::Result;
use joatmon::path_to_str;
use rlua::prelude::{FromLuaMulti, Lua, LuaContext, LuaTable};
use rlua::ExternalResult;

pub trait Eval: for<'lua> FromLuaMulti<'lua> {}

impl<T: for<'lua> FromLuaMulti<'lua>> Eval for T {}

pub fn eval<T>(
    workspace: &Workspace,
    preamble: &str,
    script: &str,
    use_prelude: bool,
    variables: &Variables,
) -> Result<T>
where
    T: std::fmt::Debug + Eval,
{
    Lua::new().context(|ctx| {
        create_variables(ctx, variables)?;

        if use_prelude {
            load_prelude(ctx, workspace)?;
        }

        Ok(ctx.load(&(preamble.to_string() + "\n\n" + script)).eval()?)
    })
}

fn create_variables(ctx: LuaContext, variables: &Variables) -> Result<()> {
    let globals_table = ctx.globals();
    for (name, config_object) in &variables.values {
        let value = translate_config_to_lua(ctx, config_object)?;
        let key = ctx.create_string(&name)?;
        globals_table.set(key, value)?;
    }

    Ok(())
}

fn create_git(ctx: LuaContext) -> Result<LuaTable> {
    let git = ctx.create_table()?;

    git.set(
        "clone",
        ctx.create_function(|_, arg| prelude::git::clone(arg).to_lua_err())?,
    )?;

    Ok(git)
}

fn load_prelude(ctx: LuaContext, workspace: &Workspace) -> Result<()> {
    let prelude = ctx.create_table()?;

    // Nested objects
    prelude.set("git", create_git(ctx)?)?;

    prelude.set(
        "workspace_dir",
        ctx.create_string(path_to_str(&workspace.workspace_dir))?,
    )?;

    prelude.set(
        "current_dir",
        ctx.create_function(|_, ()| prelude::current_dir().to_lua_err())?,
    )?;

    prelude.set(
        "is_file",
        ctx.create_function(|_, path| prelude::is_file(path).to_lua_err())?,
    )?;

    prelude.set(
        "is_dir",
        ctx.create_function(|_, path| prelude::is_dir(path).to_lua_err())?,
    )?;

    prelude.set(
        "copy_file",
        ctx.create_function(|_, (from, to)| prelude::copy_file(from, to).to_lua_err())?,
    )?;

    prelude.set(
        "copy_file_if_unchanged",
        ctx.create_function(|_, (from, to)| {
            prelude::copy_file_if_unchanged::main(from, to).to_lua_err()
        })?,
    )?;

    prelude.set(
        "read_file",
        ctx.create_function(|_, path| prelude::read_file(path).to_lua_err())?,
    )?;

    prelude.set(
        "read_file_lines",
        ctx.create_function(|_, path| prelude::read_file_lines(path).to_lua_err())?,
    )?;

    prelude.set(
        "trim_string",
        ctx.create_function(|_, str| prelude::trim_string(str).to_lua_err())?,
    )?;

    prelude.set(
        "xpath",
        ctx.create_function(|_, (namespaces_table, query, xml)| {
            prelude::xpath::main(namespaces_table, query, xml).to_lua_err()
        })?,
    )?;

    prelude.set(
        "git_clone",
        ctx.create_function(|_, args| prelude::git_clone(args).to_lua_err())?,
    )?;

    prelude.set(
        "percent_decode",
        ctx.create_function(|_, str| prelude::percent_decode(str).to_lua_err())?,
    )?;

    prelude.set(
        "inspect",
        ctx.create_function(|ctx, value| prelude::inspect(&ctx, value).to_lua_err())?,
    )?;

    ctx.globals().set("prelude", prelude)?;

    Ok(())
}
