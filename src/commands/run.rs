use crate::cli::ShellCommandInfo;
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
use crate::shell_runner::ShellRunner;
use crate::workspace::{Plan, Session};
use anyhow::Result;
use std::process::Command;

pub fn do_run(session: &Session, shell_command_info: &ShellCommandInfo) -> Result<()> {
    ShellRunner::new(shell_command_info).run(&Plan::new(session)?, |cmd| build_run_command(cmd))
}

fn build_run_command(cmd: &[String]) -> Command {
    let mut command = Command::new(&cmd[0]);
    for c in cmd.iter().skip(1) {
        command.arg(c);
    }
    command
}

#[cfg(test)]
mod tests {
    use super::build_run_command;

    #[test]
    fn build_run_command_basics() {
        let cmd = vec![
            String::from("one"),
            String::from("two"),
            String::from("three"),
        ];
        let command = build_run_command(&cmd);

        assert_eq!("one", command.get_program());
        assert_eq!(vec!["two", "three"], command.get_args().collect::<Vec<_>>());
    }
}
