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
use anyhow::Result;
use joatmon::{open_file, path_to_str, read_text_file};
use percent_encoding::percent_decode_str;
use rlua::Variadic;
use std::fs::copy;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub mod git {
    use crate::git::clone_recursive;
    use anyhow::{anyhow, Result};
    use joat_git_url::GitUrl;
    use joat_path::absolute_path;
    use rlua::prelude::LuaTable;
    use std::env::current_dir;
    use std::path::Path;

    pub fn clone(arg: LuaTable) -> Result<()> {
        let recurse = arg.get::<_, bool>("recurse")?;
        let url_str = arg.get::<_, String>("url")?;
        let url = GitUrl::parse(&url_str).ok_or_else(|| anyhow!("Could not parse Git URL"))?;
        let dir_str = arg.get::<_, String>("dir")?;
        let base_dir = current_dir()?;
        let dir = absolute_path(base_dir, Path::new(&dir_str))?;
        let branch: String = arg.get("branch")?;

        println!(
            "git.clone: recurse={} url={} dir={} branch={}",
            recurse,
            url,
            dir.display(),
            branch
        );

        let repo = match recurse {
            true => clone_recursive(&url, &dir, &branch)?,
            false => unimplemented!("Non-recursive clone not implemented"),
        };

        println!("git.clone: dir={}", repo.path().display());
        Ok(())
    }
}

pub fn current_dir() -> Result<String> {
    Ok(path_to_str(&std::env::current_dir()?).to_string())
}

pub fn is_file(path: String) -> Result<bool> {
    Ok(Path::new(&path).is_file())
}

pub fn is_dir(path: String) -> Result<bool> {
    Ok(Path::new(&path).is_dir())
}

pub fn copy_file(from: String, to: String) -> Result<()> {
    _ = copy(from, to)?;
    Ok(())
}

pub mod copy_file_if_unchanged {
    use anyhow::Result;
    use joatmon::open_file;
    use std::fs::copy;
    use std::io::Read;
    use std::path::Path;

    pub fn main(from: String, to: String) -> Result<bool> {
        let from_path = Path::new(&from);
        let to_path = Path::new(&to);
        let perform_copy = !to_path.is_file() || read_bytes(from_path)? != read_bytes(to_path)?;
        if perform_copy {
            copy(from_path, to_path)?;
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

pub fn read_file(path: String) -> Result<String> {
    Ok(read_text_file(path)?)
}

pub fn read_file_lines(path: String) -> Result<Vec<String>> {
    let f = open_file(path)?;
    Ok(BufReader::new(f).lines().collect::<std::io::Result<_>>()?)
}

pub fn trim_string(str: String) -> Result<String> {
    Ok(str.trim().to_string())
}

pub mod xpath {
    use crate::scripting::xml::{query_xpath_as_string, XmlNamespace};
    use anyhow::Result;
    use rlua::prelude::LuaTable;

    pub fn main(namespaces_table: LuaTable, query: String, xml: String) -> Result<String> {
        let namespaces = decode_namespaces(namespaces_table)?;
        query_xpath_as_string(&namespaces, &query, &xml)
    }

    fn decode_namespaces(namespaces_table: LuaTable) -> Result<Vec<XmlNamespace>> {
        let mut namespaces = Vec::new();
        for result in namespaces_table.sequence_values::<LuaTable>() {
            let namespace_table = result?;
            let prefix: String = namespace_table.get("prefix")?;
            let uri: String = namespace_table.get("uri")?;
            namespaces.push(XmlNamespace::new(prefix, uri))
        }
        Ok(namespaces)
    }
}

pub fn git_clone(args: Variadic<String>) -> Result<()> {
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

pub fn percent_decode(str: String) -> Result<String> {
    Ok(percent_decode_str(&str).decode_utf8_lossy().to_string())
}
