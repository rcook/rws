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
use crate::error::{Error, Result};

use joat_git_url::GitUrl;
use regex::Regex;
use std::fs::{copy, File};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub struct SubmoduleURLRewriter {
    submodules_path: PathBuf,
    temp_file: NamedTempFile,
}

impl SubmoduleURLRewriter {
    pub fn new(submodules_path: &Path, remote_git_url: &GitUrl) -> Result<Self> {
        let rewriter = Self {
            submodules_path: submodules_path.to_path_buf(),
            temp_file: NamedTempFile::new()?,
        };
        copy(&rewriter.submodules_path, rewriter.temp_file.path())?;
        let in_f = File::open(rewriter.temp_file.path())?;
        let out_f = File::create(&rewriter.submodules_path)?;
        let reader = BufReader::new(in_f);
        let mut writer = BufWriter::new(out_f);
        let re = Regex::new(r"(?P<prefix>\s*url\s*=\s*)(?P<url>.*)")?;
        for l in reader.lines() {
            let line = l?;
            match re.captures(&line) {
                Some(caps) => {
                    let git_url = remote_git_url.join(&caps["url"]).ok_or_else(|| {
                        Error::User(format!(
                            "Failed to resolve Git submodule URL {}",
                            &caps["url"]
                        ))
                    })?;
                    writeln!(writer, "{}{}", &caps["prefix"], git_url)?;
                    writer.flush()?;
                }
                None => {
                    writeln!(writer, "{}", line)?;
                    writer.flush()?
                }
            }
        }
        Ok(rewriter)
    }

    pub fn restore(&self) {
        // Could do git checkout -- .gitmodules instead!
        let _ = copy(self.temp_file.path(), &self.submodules_path);
    }
}
