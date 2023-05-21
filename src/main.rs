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
#![warn(clippy::all)]
#![warn(clippy::cargo)]
//#![warn(clippy::expect_used)]
#![warn(clippy::nursery)]
//#![warn(clippy::panic_in_result_fn)]
#![warn(clippy::pedantic)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::option_if_let_else)]
mod cli;
mod commands;
mod git;
mod marshal;
mod scripting;
mod shell_runner;
mod util;
mod workspace;

use crate::cli::{Args, Subcommand};
use crate::commands::{do_git, do_info, do_init, do_new, do_run};
use crate::shell_runner::{ShellResult, FAILURE_EXIT_CODE};
use crate::util::reset_terminal;
use crate::workspace::Session;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rlua::prelude::LuaError;
use std::env::current_dir;
use std::process::exit;

fn main() {
    reset_terminal();
    exit(match run() {
        Ok(shell_result) => shell_result.exit_code(),
        Err(e) => {
            // TBD: Figure out how to wrap Lua errors better!
            match e.downcast_ref::<LuaError>() {
                Some(lua_error) => {
                    println!("{}", format!("Lua error: {lua_error:#?}").red());
                }
                None => println!("{}", format!("{e}").bright_red()),
            }
            FAILURE_EXIT_CODE
        }
    })
}

fn get_session() -> Result<(Args, Session)> {
    let args = Args::parse();
    let cwd = current_dir()?;
    let session = Session::new(
        &cwd,
        args.workspace_dir.as_deref(),
        args.config_path.as_deref(),
    )?;
    Ok((args, session))
}

fn run() -> Result<ShellResult> {
    let (args, session) = get_session()?;
    Ok(match args.subcommand {
        Subcommand::Git(shell_command_info) => do_git(&session, &shell_command_info)?,
        Subcommand::Info => {
            do_info(&session, true)?;
            ShellResult::Success
        }
        Subcommand::Init => {
            do_init(&session)?;
            ShellResult::Success
        }
        Subcommand::New => {
            do_new(&session)?;
            ShellResult::Success
        }
        Subcommand::Run(shell_command_info) => do_run(&session, &shell_command_info)?,
    })
}
