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
use super::helpers::run_helper;
use crate::command_info::CommandInfo;
use crate::git::GitInfo;
use crate::workspace::{Plan, Workspace};
use anyhow::Result;
use std::process::Command;

pub fn do_git(workspace: &Workspace, command_info: &CommandInfo) -> Result<()> {
    let git_info = GitInfo::from_environment()?;
    run_helper(&Plan::new(workspace)?, command_info, |cmd| {
        build_git_command(&git_info, cmd)
    })
}

fn build_git_command(git_info: &GitInfo, cmd: &[String]) -> Command {
    let mut command = Command::new(&git_info.executable_path);
    for c in cmd.iter() {
        command.arg(c);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::build_git_command;
    use crate::git::GitInfo;
    use std::path::Path;

    #[test]
    fn build_git_command_basics() {
        let git_info = GitInfo::new(Path::new("GIT"), "VERSION");
        let cmd = vec![
            String::from("one"),
            String::from("two"),
            String::from("three"),
        ];
        let command = build_git_command(&git_info, &cmd);

        assert_eq!("GIT", command.get_program());
        assert_eq!(
            vec!["one", "two", "three"],
            command.get_args().collect::<Vec<_>>()
        );
    }
}
