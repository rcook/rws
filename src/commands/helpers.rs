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
use crate::command_info::CommandInfo;
use crate::order::DirectoryOrder;
use crate::session::Plan;
use crate::util::reset_terminal;
use anyhow::Result;
use colored::Colorize;
use joatmon::path_to_str;
use std::path::PathBuf;
use std::process::Command;

pub fn show_project_dirs(order: &str, project_dirs: &[PathBuf]) {
    if project_dirs.is_empty() {
        println!(
            "Project directories ({} order): {}",
            order,
            "(none)".red().italic()
        );
    } else {
        println!("Project directories ({} order):", order);
        for project_dir in project_dirs {
            println!("  {}", path_to_str(project_dir).cyan())
        }
    }
}

pub fn run_helper<F>(plan: &Plan, command_info: &CommandInfo, f: F) -> Result<()>
where
    F: Fn(&Vec<String>) -> Command,
{
    let mut failure_count = 0;
    let project_dirs = match (&command_info.order, &plan.project_dirs_topo) {
        (DirectoryOrder::Topological, Some(ds)) => ds,
        _ => &plan.project_dirs_alpha,
    };
    for project_dir in project_dirs {
        let d = path_to_str(project_dir);
        println!("{}", d.cyan());
        let exit_status = f(&command_info.cmd)
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
                    println!(
                        "{}",
                        if command_info.fail_fast {
                            m.red()
                        } else {
                            m.yellow()
                        }
                    );
                    if command_info.fail_fast {
                        break;
                    }
                }
                None => println!("{}", format!("Command terminated by signal in {}", d).red()),
            }
        }
    }

    if !command_info.fail_fast && failure_count > 0 {
        println!(
            "{}",
            format!("Command failed in {} project directories", failure_count).red()
        )
    }

    Ok(())
}
