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
use crate::git::GitInfo;
use crate::marshal::JsonValue;
use anyhow::{anyhow, Result};
use joatmon::{open_file, read_text_file};
use percent_encoding::percent_decode_str;
use std::fs::copy;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub mod git {
    use crate::git::clone_recursive;
    use crate::marshal::{JsonValue, RequiredFields};
    use anyhow::Result;
    use joat_git_url::GitUrl;
    use joat_path::absolute_path;
    use std::env::current_dir;
    use std::path::Path;

    pub fn clone(obj: &JsonValue) -> Result<()> {
        let recurse = obj.get_required_bool("recurse")?;
        let url_str = obj.get_required_str("url")?;
        let dir_str = obj.get_required_str("dir")?;
        let branch = obj.get_required_str("branch")?;

        let url = url_str.parse::<GitUrl>()?;
        let base_dir = current_dir()?;
        let dir = absolute_path(base_dir, Path::new(&dir_str))?;

        println!(
            "git.clone: recurse={} url={} dir={} branch={}",
            recurse,
            url,
            dir.display(),
            branch
        );

        let repo = if recurse {
            clone_recursive(&url, &dir, branch)?
        } else {
            unimplemented!("Non-recursive clone not implemented")
        };

        println!("git.clone: dir={}", repo.path().display());

        Ok(())
    }
}

pub fn current_dir() -> Result<String> {
    Ok(String::from(
        std::env::current_dir()?
            .to_str()
            .ok_or_else(|| anyhow!("cannot convert path to string"))?,
    ))
}

#[allow(clippy::unnecessary_wraps)]
pub fn is_file(path: &Path) -> Result<bool> {
    Ok(path.is_file())
}

#[allow(clippy::unnecessary_wraps)]
pub fn is_dir(path: &Path) -> Result<bool> {
    Ok(path.is_dir())
}

pub fn copy_file(from: &Path, to: &Path) -> Result<()> {
    _ = copy(from, to)?;
    Ok(())
}

pub mod copy_file_if_unchanged {
    use anyhow::Result;
    use joatmon::open_file;
    use std::fs::copy;
    use std::io::Read;
    use std::path::Path;

    pub fn main(from: &Path, to: &Path) -> Result<bool> {
        let perform_copy = !to.is_file() || read_bytes(from)? != read_bytes(to)?;
        if perform_copy {
            copy(from, to)?;
        }
        Ok(perform_copy)
    }

    fn read_bytes(path: &Path) -> Result<Vec<u8>> {
        let mut f = open_file(path)?;
        let mut data = Vec::new();
        let _ = f.read_to_end(&mut data)?;
        Ok(data)
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn read_file(path: String) -> Result<String> {
    Ok(read_text_file(Path::new(&path))?)
}

#[allow(clippy::needless_pass_by_value)]
pub fn read_file_lines(path: String) -> Result<Vec<String>> {
    let f = open_file(Path::new(&path))?;
    Ok(BufReader::new(f).lines().collect::<std::io::Result<_>>()?)
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn trim_string(s: String) -> Result<String> {
    Ok(s.trim().to_string())
}

pub mod xpath {
    use crate::marshal::{JsonValue, RequiredFields};
    use crate::scripting::xml::{query_xpath_as_string, XmlNamespace};
    use anyhow::{anyhow, Result};

    #[allow(clippy::needless_pass_by_value)]
    pub fn main(namespace_objs_obj: &JsonValue, query: String, xml: String) -> Result<String> {
        let namespaces = decode_namespaces(namespace_objs_obj)?;
        query_xpath_as_string(&namespaces, &query, &xml)
    }

    fn decode_namespaces(namespace_objs_obj: &JsonValue) -> Result<Vec<XmlNamespace>> {
        let namespace_objs = namespace_objs_obj
            .as_array()
            .ok_or_else(|| anyhow!("Must be an array"))?;

        let mut namespaces = Vec::new();
        for namespace_obj in namespace_objs {
            let prefix = namespace_obj.get_required_str("prefix")?;
            let uri = namespace_obj.get_required_str("uri")?;
            namespaces.push(XmlNamespace::new(prefix, uri));
        }

        Ok(namespaces)
    }
}

pub fn git_clone(args: Vec<String>) -> Result<()> {
    let git_info = GitInfo::from_environment()?;
    let mut git_command = Command::new(git_info.executable_path);
    git_command.arg("clone");
    for arg in args {
        git_command.arg(arg);
    }

    let mut child: std::process::Child = git_command.spawn()?;
    _ = child.wait()?;
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub fn percent_decode(s: String) -> Result<String> {
    Ok(percent_decode_str(&s).decode_utf8_lossy().to_string())
}

pub fn inspect(obj: &JsonValue) -> Result<String> {
    Ok(serde_json::to_string_pretty(obj)?)
}
