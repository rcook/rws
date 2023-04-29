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
use anyhow::Result;
use std::env::{current_dir, set_current_dir};
use std::path::{Path, PathBuf};

pub struct WorkingDirectory {
    saved_dir: Option<PathBuf>,
}

impl WorkingDirectory {
    pub fn change<P>(dir: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let saved_dir = current_dir()?;
        set_current_dir(dir)?;
        Ok(Self {
            saved_dir: Some(saved_dir),
        })
    }

    pub fn close(&mut self) -> Result<()> {
        Ok(match &self.saved_dir {
            Some(d) => {
                set_current_dir(&d)?;
                self.saved_dir = None
            }
            None => (),
        })
    }
}

impl Drop for WorkingDirectory {
    fn drop(&mut self) {
        _ = self.close()
    }
}

pub fn with_working_dir<P, F, R>(dir: P, f: F) -> std::io::Result<R>
where
    P: AsRef<Path>,
    F: FnOnce() -> R,
{
    let working_dir = WorkingDirectory::change(dir);
    let result = f();
    drop(working_dir);
    Ok(result)
}
