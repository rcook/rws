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
#![allow(unused)]
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Definition {
    #[serde(rename = "variables")]
    pub variables: Option<Variables>,

    #[serde(rename = "lua_config")]
    pub lua_config: Option<LanguageConfig>,

    #[serde(rename = "default_language")]
    pub default_language: Option<Language>,

    #[serde(rename = "excluded_projects")]
    pub excluded_projects: Option<Vec<String>>,

    #[serde(rename = "init_command")]
    pub init_command: Option<Command>,

    #[serde(flatten)]
    pub dependency_source: Option<DependencySource>,
}

pub type Variables = HashMap<String, serde_yaml::Value>;

#[derive(Clone, Debug, Deserialize)]
pub struct LanguageConfig {
    #[serde(rename = "preamble")]
    pub preamble: Option<String>,

    #[serde(rename = "use_prelude")]
    pub use_prelude: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Language {
    #[serde(rename = "lua")]
    Lua,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Command {
    #[serde(rename = "language")]
    pub language: Option<Language>,

    #[serde(rename = "use_prelude")]
    pub use_prelude: Option<bool>,

    #[serde(rename = "script")]
    pub script: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum DependencySource {
    #[serde(rename = "dependencies")]
    Static(StaticDependencies),

    #[serde(rename = "dependency_command")]
    Command(Command),
}

pub type StaticDependencies = HashMap<String, Vec<String>>;

#[cfg(test)]
mod tests {
    use super::{Command, Definition, DependencySource, Language};
    use anyhow::Result;
    use rstest::rstest;
    use serde_yaml::{from_str, Value};
    use std::collections::HashMap;

    #[test]
    fn basics() -> Result<()> {
        let input = r#"
variables:
  KEY0: VALUE0
  KEY1: VALUE1
  KEY2:
    - one
    - two

default_language: lua

excluded_projects:
  - project0
  - project1

init_command:
  language: lua
  use_prelude: true
  script: |
    dofile(prelude.workspace_dir .. "/shared.lua")
    print("Hello from init_command")

dependency_command:
  script: |
    dofile(prelude.workspace_dir .. "/shared.lua")
    local function f()
    end
    local w = { x = 0, y = 0, label = "console", func = f }
    print("INSPECT: " .. prelude.inspect(w))
    if prelude.is_file(DEPS_FILE_NAME) then
      return parse_config_lines(prelude.read_file_lines(DEPS_FILE_NAME))
    else
      return { }
    end
"#;
        let definition = from_str::<Definition>(input)?;

        let variables = definition.variables.expect("must be present");
        assert_eq!(3, variables.len());
        assert_eq!(Value::String(String::from("VALUE0")), variables["KEY0"]);
        assert_eq!(Value::String(String::from("VALUE1")), variables["KEY1"]);
        assert_eq!(
            Value::Sequence(vec![
                Value::String(String::from("one")),
                Value::String(String::from("two"))
            ]),
            variables["KEY2"]
        );

        let default_language = definition.default_language.expect("must be present");
        assert_eq!(Language::Lua, default_language);

        let excluded_projects = definition.excluded_projects.expect("must be present");
        assert_eq!(vec!["project0", "project1"], excluded_projects);

        let init_command = definition.init_command.expect("must be present");
        assert_eq!(
            Language::Lua,
            init_command.language.expect("must be present")
        );
        assert!(init_command.use_prelude.expect("must be present"));
        assert_eq!(
            r#"dofile(prelude.workspace_dir .. "/shared.lua")
print("Hello from init_command")
"#,
            init_command.script,
        );

        let dependency_command = match definition.dependency_source {
            Some(DependencySource::Command(command)) => command,
            _ => panic!("Expected dependency_command"),
        };
        assert!(dependency_command.language.is_none());
        assert!(dependency_command.use_prelude.is_none());
        assert_eq!(
            r#"dofile(prelude.workspace_dir .. "/shared.lua")
local function f()
end
local w = { x = 0, y = 0, label = "console", func = f }
print("INSPECT: " .. prelude.inspect(w))
if prelude.is_file(DEPS_FILE_NAME) then
  return parse_config_lines(prelude.read_file_lines(DEPS_FILE_NAME))
else
  return { }
end
"#,
            dependency_command.script,
        );
        Ok(())
    }

    #[rstest]
    #[case(
        Some(DependencySource::Static(HashMap::from([
            (String::from("aaa"), vec![String::from("bbb"), String::from("ccc")]),
            (String::from("ddd"), vec![String::from("eee"), String::from("fff")])
        ]))),
        r#"dependencies:
  aaa:
    - bbb
    - ccc
  ddd:
    - eee
    - fff"#
    )]
    #[case(
        Some(DependencySource::Command(Command{language:None, use_prelude:None, script: String::from("SCRIPT")})),
        r#"dependency_command:
  script: SCRIPT"#
    )]
    #[case(None, "")]
    fn dependency_source(
        #[case] expected_dependency_source: Option<DependencySource>,
        #[case] input: &str,
    ) -> Result<()> {
        let definition = from_str::<Definition>(input)?;
        assert_eq!(expected_dependency_source, definition.dependency_source);
        Ok(())
    }
}
