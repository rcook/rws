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
use crate::cli::{ProjectOrder, ShellCommandInfo};
use crate::util::reset_terminal;
use crate::workspace::Plan;
use anyhow::Result;
use colored::Colorize;
use joatmon::path_to_str;
use std::process::Command;

pub struct ShellRunner {
    pub cmd: Vec<String>,
    pub fail_fast: bool,
    pub project_order: ProjectOrder,
}

impl ShellRunner {
    pub fn new(shell_command_info: &ShellCommandInfo) -> Self {
        let mut cmd = Vec::new();
        cmd.push(shell_command_info.command.clone());
        for arg in &shell_command_info.args {
            cmd.push(arg.clone());
        }

        Self {
            cmd,
            fail_fast: shell_command_info.fail_fast,
            project_order: shell_command_info.project_order.clone(),
        }
    }

    pub fn run<F>(&self, plan: &Plan, build_command: F) -> Result<()>
    where
        F: Fn(&[String]) -> Command,
    {
        let mut failure_count = 0;
        let project_dirs = match (&self.project_order, &plan.project_dirs_topo) {
            (ProjectOrder::Topological, Some(ds)) => ds,
            _ => &plan.project_dirs_alpha,
        };
        for project_dir in project_dirs {
            let d = path_to_str(project_dir);
            println!("{}", d.cyan());
            let exit_status = build_command(&self.cmd)
                .current_dir(project_dir)
                .spawn()?
                .wait()?;
            reset_terminal();
            if exit_status.success() {
                println!("{}", format!("Command succeeded in {}", d).green())
            } else {
                failure_count += 1;
                match exit_status.code() {
                    Some(code) => {
                        let m = format!("Command exited with status {} in {}", code, d);
                        println!("{}", if self.fail_fast { m.red() } else { m.yellow() });
                        if self.fail_fast {
                            break;
                        }
                    }
                    None => println!("{}", format!("Command terminated by signal in {}", d).red()),
                }
            }
        }

        if !self.fail_fast && failure_count > 0 {
            println!(
                "{}",
                format!("Command failed in {} project directories", failure_count).red()
            )
        }

        Ok(())
    }
}
