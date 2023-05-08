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
mod args;
mod command_info;
mod commands;
mod config;
mod git;
mod marshal;
mod order;
mod scripting;
mod util;
mod workspace;

use crate::args::{Args, Command};
use crate::command_info::CommandInfo;
use crate::commands::{do_git, do_info, do_init, do_run};
use crate::util::reset_terminal;
use crate::workspace::Session;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rlua::prelude::LuaError;
use std::env::current_dir;
use std::process::exit;

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            // TBD: Figure out how to wrap Lua errors better!
            match e.downcast_ref::<LuaError>() {
                Some(lua_error) => {
                    println!("{}", format!("Lua error: {:#?}", lua_error).red())
                }
                None => println!("{}", format!("{}", e).bright_red()),
            }
            1
        }
    })
}

fn run() -> Result<()> {
    reset_terminal();
    let args = Args::parse();
    let cwd = current_dir()?;
    let session = Session::new(
        &cwd,
        args.workspace_dir.as_deref(),
        args.config_path.as_deref(),
    )?;
    match args.command {
        Command::Git {
            fail_fast,
            order,
            command,
            args,
        } => do_git(&session, &CommandInfo::new(command, args, fail_fast, order))?,
        Command::Info => do_info(&session, true)?,
        Command::Init => do_init(&session)?,
        Command::Run {
            fail_fast,
            order,
            command,
            args,
        } => do_run(&session, &CommandInfo::new(command, args, fail_fast, order))?,
    }
    Ok(())
}
