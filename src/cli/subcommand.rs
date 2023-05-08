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
use super::project_order::ProjectOrder;
use clap::{Args, Subcommand as ClapSubcommand};

#[derive(ClapSubcommand, Debug)]
pub enum Subcommand {
    #[command(
        name = "git",
        about = "Run Git command in each project directory using system Git command"
    )]
    Git(ShellCommandInfo),

    #[command(name = "info", about = "Print workspace and environment information")]
    Info,

    #[command(name = "init", about = "Initialize workspace")]
    Init,

    #[command(name = "new", about = "Create new workspace")]
    New,

    #[command(name = "run", about = "Run command in each project directory")]
    Run(ShellCommandInfo),
}

#[derive(Args, Debug)]
pub struct ShellCommandInfo {
    #[arg(help = "Fail fast", short = 'f', long = "fail-fast")]
    pub fail_fast: bool,

    #[arg(
        help = "Project traversal order",
        short = 'o',
        long = "order",
        default_value_t = ProjectOrder::Topological,
        value_enum
    )]
    pub project_order: ProjectOrder,

    #[arg(help = "Program or subcommand to run in environment")]
    pub command: String,

    #[arg(help = "Zero or more arguments to pass to program or subcommand")]
    #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}
