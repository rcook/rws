use crate::git::GitInfo;
use crate::os::path_to_str;

use super::helpers::guard_io;

use percent_encoding::percent_decode_str;
use rlua::prelude::LuaResult;
use rlua::Variadic;
use std::fs::{copy, read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub fn current_dir() -> LuaResult<String> {
    Ok(path_to_str(&guard_io(std::env::current_dir())?).to_string())
}

pub fn is_file(path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).is_file())
}

pub fn is_dir(path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).is_dir())
}

pub fn copy_file(from: String, to: String) -> LuaResult<()> {
    guard_io(copy(from, to))?;
    Ok(())
}

pub mod copy_file_if_unchanged {
    use super::super::helpers::guard_io;

    use rlua::prelude::LuaResult;
    use std::fs::{copy, File};
    use std::io::Read;
    use std::path::Path;

    pub fn main(from: String, to: String) -> LuaResult<bool> {
        let from_path = Path::new(&from);
        let to_path = Path::new(&to);
        let perform_copy = !to_path.is_file()
            || guard_io(read_bytes(&from_path))? != guard_io(read_bytes(&to_path))?;
        if perform_copy {
            guard_io(copy(from_path, to_path))?;
        }
        Ok(perform_copy)
    }

    fn read_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
        let mut f = File::open(path)?;
        let mut data = Vec::new();
        let _ = f.read_to_end(&mut data)?;
        Ok(data)
    }
}

pub fn read_file(path: String) -> LuaResult<String> {
    guard_io(read_to_string(&path))
}

pub fn read_file_lines(path: String) -> LuaResult<Vec<String>> {
    let f = guard_io(File::open(&path))?;
    guard_io(BufReader::new(f).lines().collect::<std::io::Result<_>>())
}

pub fn trim_string(str: String) -> LuaResult<String> {
    Ok(str.trim().to_string())
}

pub mod xpath {
    use crate::error::Result;
    use crate::scripting::xml::{query_xpath_as_string, XmlNamespace};

    use rlua::prelude::LuaResult;
    use rlua::Table;

    pub fn main(namespaces_table: Table, query: String, xml: String) -> LuaResult<String> {
        let namespaces = decode_namespaces(namespaces_table)?;
        Ok(query_xpath_as_string(&namespaces, &query, &xml)?)
    }

    fn decode_namespaces(namespaces_table: Table) -> Result<Vec<XmlNamespace>> {
        let mut namespaces = Vec::new();
        for result in namespaces_table.sequence_values::<Table>() {
            let namespace_table = result?;
            let prefix: String = namespace_table.get("prefix")?;
            let uri: String = namespace_table.get("uri")?;
            namespaces.push(XmlNamespace::new(prefix, uri))
        }
        Ok(namespaces)
    }
}

pub fn git_clone(args: Variadic<String>) -> LuaResult<()> {
    let git_info = GitInfo::from_environment()?;
    let mut git_command = Command::new(git_info.executable_path);
    git_command.arg("clone");
    for arg in args {
        git_command.arg(arg);
    }

    let mut child: std::process::Child = guard_io(git_command.spawn())?;
    let _ = guard_io(child.wait())?;
    Ok(())
}

pub fn percent_decode(str: String) -> LuaResult<String> {
    Ok(percent_decode_str(&str).decode_utf8_lossy().to_string())
}
