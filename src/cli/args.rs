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
use super::constants::{
    PACKAGE_BUILD_VERSION, PACKAGE_DESCRIPTION, PACKAGE_HOME_PAGE, PACKAGE_NAME, PACKAGE_VERSION,
};
use super::funcs::parse_absolute_path;
use super::subcommand::Subcommand;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = PACKAGE_NAME,
    version = PACKAGE_VERSION,
    about = format!("{} {}", PACKAGE_DESCRIPTION, PACKAGE_VERSION),
    after_help = format!("{}\n{}", PACKAGE_HOME_PAGE, PACKAGE_BUILD_VERSION.map(|x| format!("\n{}", x)).unwrap_or(String::from("")))
)]
pub struct Args {
    #[arg(global = true, short = 'c', long = "config", value_parser = parse_absolute_path)]
    pub config_path: Option<PathBuf>,
    #[arg(global = true, short = 'd', long = "dir", value_parser = parse_absolute_path)]
    pub workspace_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub subcommand: Subcommand,
}
