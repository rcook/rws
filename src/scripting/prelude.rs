use crate::os::path_to_str;

use super::helpers::guard_io;

use rlua::prelude::LuaResult;
use rlua::Variadic;
use std::fs::{read_to_string, File};
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
    let mut git_command = Command::new("git");
    git_command.arg("clone");
    for arg in args {
        git_command.arg(arg);
    }

    let mut child: std::process::Child = guard_io(git_command.spawn())?;
    let _ = guard_io(child.wait())?;
    Ok(())
}
