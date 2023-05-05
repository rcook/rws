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
use super::super::prelude;
use super::super::variables::Variables;
use super::lua_config::translate_config_to_lua;
use super::marshal::lua_to_object;
use crate::workspace::Workspace;
use anyhow::Result;
use joatmon::path_to_str;
use rlua::prelude::{
    FromLuaMulti, Lua, LuaContext, LuaExternalResult, LuaResult, LuaTable, LuaValue,
};
use rlua::Variadic;
use std::path::Path;

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
    // Workspace variables go into global "namespace"...
    let globals = ctx.globals();

    // ... and are aliased under "vars"
    let vars = ctx.create_table()?;

    for (name, config_object) in &variables.values {
        // TBD: Figure out how to avoid create two of everything...
        let value = translate_config_to_lua(ctx, config_object)?;
        let key = ctx.create_string(&name)?;
        globals.set(key, value)?;

        let value = translate_config_to_lua(ctx, config_object)?;
        let key = ctx.create_string(&name)?;
        vars.set(key, value)?;
    }

    ctx.globals().set("vars", vars)?;
    Ok(())
}

fn create_git(ctx: LuaContext) -> Result<LuaTable> {
    let git = ctx.create_table()?;

    git.set(
        "clone",
        ctx.create_function(|_ctx, value| -> LuaResult<()> {
            let obj = lua_to_object(value, true).to_lua_err()?;
            prelude::git::clone(&obj).to_lua_err()
        })?,
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
        ctx.create_function(|_ctx, ()| prelude::current_dir().to_lua_err())?,
    )?;

    prelude.set(
        "is_file",
        ctx.create_function(|_ctx, path: String| prelude::is_file(Path::new(&path)).to_lua_err())?,
    )?;

    prelude.set(
        "is_dir",
        ctx.create_function(|_ctx, path: String| prelude::is_dir(Path::new(&path)).to_lua_err())?,
    )?;

    prelude.set(
        "copy_file",
        ctx.create_function(|_ctx, (from, to): (String, String)| {
            prelude::copy_file(Path::new(&from), Path::new(&to)).to_lua_err()
        })?,
    )?;

    prelude.set(
        "copy_file_if_unchanged",
        ctx.create_function(|_ctx, (from, to): (String, String)| {
            prelude::copy_file_if_unchanged::main(Path::new(&from), Path::new(&to)).to_lua_err()
        })?,
    )?;

    prelude.set(
        "read_file",
        ctx.create_function(|_ctx, path| prelude::read_file(path).to_lua_err())?,
    )?;

    prelude.set(
        "read_file_lines",
        ctx.create_function(|_ctx, path| prelude::read_file_lines(path).to_lua_err())?,
    )?;

    prelude.set(
        "trim_string",
        ctx.create_function(|_ctx, s| prelude::trim_string(s).to_lua_err())?,
    )?;

    prelude.set(
        "xpath",
        ctx.create_function(|_ctx, (namespaces, query, xml)| {
            let namespace_objs_obj = lua_to_object(namespaces, true).to_lua_err()?;
            prelude::xpath::main(&namespace_objs_obj, query, xml).to_lua_err()
        })?,
    )?;

    prelude.set(
        "git_clone",
        ctx.create_function(|_ctx, args: Variadic<String>| {
            prelude::git_clone(args.to_vec()).to_lua_err()
        })?,
    )?;

    prelude.set(
        "percent_decode",
        ctx.create_function(|_ctx, s| prelude::percent_decode(s).to_lua_err())?,
    )?;

    prelude.set(
        "inspect",
        ctx.create_function(|ctx, value| {
            let obj = lua_to_object(value, true).to_lua_err()?;
            let s = prelude::inspect(&obj).to_lua_err()?;
            let lua_string = ctx.create_string(&s)?;
            Ok(LuaValue::String(lua_string))
        })?,
    )?;

    ctx.globals().set("prelude", prelude)?;

    Ok(())
}
